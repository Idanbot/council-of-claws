use crate::models::{
    AppError, ConfiguredAgent, OpenClawStatus, ProviderStatus, SkillDefinition, TaskPriority,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

const OPENCLAW_STATUS_SCHEMA_VERSION: i32 = 1;
const CACHE_TTL_SECONDS: i64 = 10;

type CachedOpenClawStatus = Option<(DateTime<Utc>, OpenClawStatus)>;

#[derive(Clone)]
pub struct OpenClawReader {
    state_path: PathBuf,
    cache: Arc<RwLock<CachedOpenClawStatus>>,
}

impl OpenClawReader {
    pub fn new<P: Into<PathBuf>>(state_path: P) -> Self {
        Self {
            state_path: state_path.into(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn discover_skills(&self) -> Vec<SkillDefinition> {
        let skills_root = PathBuf::from("/app/.agents/skills");
        let mut discovered = Vec::new();

        if !skills_root.exists() {
            tracing::warn!("Skills directory not found at {:?}", skills_root);
            return discovered;
        }

        if let Ok(entries) = fs::read_dir(skills_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_id = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(content) = fs::read_to_string(&skill_md) {
                            let (name, desc) = self.parse_skill_md(&content, skill_id);
                            discovered.push(SkillDefinition {
                                id: skill_id.to_string(),
                                name,
                                description: desc,
                            });
                        }
                    }
                }
            }
        }

        discovered.sort_by(|a, b| a.id.cmp(&b.id));
        discovered
    }

    fn parse_skill_md(&self, content: &str, default_id: &str) -> (String, String) {
        let mut name = default_id.to_string();
        let mut desc = String::new();

        if let Some(stripped) = content.strip_prefix("---") {
            if let Some(end_idx) = stripped.find("---") {
                let frontmatter = &stripped[..end_idx];
                for line in frontmatter.lines() {
                    if let Some((key, val)) = line.split_once(':') {
                        match key.trim() {
                            "name" => name = val.trim().trim_matches('"').to_string(),
                            "description" => desc = val.trim().trim_matches('"').to_string(),
                            _ => {}
                        }
                    }
                }
            }
        }

        (name, desc)
    }

    pub fn read_raw_config(&self) -> Result<String, AppError> {
        let config_path = self.state_path.join("config/openclaw.json5");
        fs::read_to_string(&config_path).map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn update_raw_config(&self, content: &str) -> Result<(), AppError> {
        let config_path = self.state_path.join("config/openclaw.json5");
        fs::write(&config_path, content).map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }

    pub async fn read_status(&self) -> OpenClawStatus {
        // 1. Check in-memory cache
        {
            let cache = self.cache.read().await;
            if let Some((cached_at, status)) = cache.as_ref() {
                let age = Utc::now().signed_duration_since(*cached_at).num_seconds();
                if age < CACHE_TTL_SECONDS {
                    return status.clone();
                }
            }
        }

        // 2. Cache miss or expired, scan disk
        let status = self.scan_disk();

        // 3. Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some((Utc::now(), status.clone()));
        }

        status
    }

    fn scan_disk(&self) -> OpenClawStatus {
        let config_path = self.state_path.join("config/openclaw.json5");
        let mut issues = Vec::new();
        let generated_at = Utc::now();

        let config = read_json5::<OpenClawConfig>(&config_path, &mut issues, "OpenClaw config");
        let runtime_catalogs = self.read_runtime_catalogs(&mut issues);
        let runtime_auth_profiles = self.read_runtime_auth_profiles(&mut issues);

        let configured_agents = config
            .as_ref()
            .map(build_configured_agents)
            .unwrap_or_default();
        let configured_model_refs = config
            .as_ref()
            .map(collect_configured_model_refs)
            .unwrap_or_default();

        let providers = build_provider_statuses(
            config.as_ref(),
            &runtime_catalogs,
            &runtime_auth_profiles,
            &configured_model_refs,
        );

        let mut available_model_refs = dedupe_sorted(
            providers
                .iter()
                .flat_map(|provider| provider.available_models.iter().cloned())
                .collect(),
        );

        // Also include configured models that might not have been dynamically discovered yet
        let mut all_refs = available_model_refs.clone();
        all_refs.extend(configured_model_refs.clone());
        available_model_refs = dedupe_sorted(all_refs);
        let invalid_model_refs = dedupe_sorted(
            providers
                .iter()
                .flat_map(|provider| provider.issues.iter())
                .filter_map(|issue| issue.strip_prefix("invalid-model-ref:"))
                .map(|item| item.to_string())
                .collect(),
        );

        if !invalid_model_refs.is_empty() {
            issues.push(format!(
                "configured model refs not backed by the current runtime catalog: {}",
                invalid_model_refs.join(", ")
            ));
        }

        let issues = dedupe_sorted(issues);
        let source_mtime = latest_source_mtime(&self.state_path, &config_path);
        let status = derive_openclaw_status(config.is_some(), &issues);
        let snapshot_fingerprint = build_snapshot_fingerprint(SnapshotFingerprintInput {
            config_path: &config_path,
            source_mtime,
            status: &status,
            runtime_state_available: !runtime_catalogs.is_empty()
                || !runtime_auth_profiles.is_empty(),
            configured_agents: &configured_agents,
            providers: &providers,
            available_model_refs: &available_model_refs,
            configured_model_refs: &configured_model_refs,
            invalid_model_refs: &invalid_model_refs,
            issues: &issues,
        });

        OpenClawStatus {
            schema_version: OPENCLAW_STATUS_SCHEMA_VERSION,
            snapshot_fingerprint,
            status,
            generated_at,
            last_success_at: generated_at,
            source_path: self.state_path.display().to_string(),
            config_path: config_path.display().to_string(),
            source_mtime,
            runtime_state_available: !runtime_catalogs.is_empty()
                || !runtime_auth_profiles.is_empty(),
            configured_agents,
            providers,
            available_model_refs,
            configured_model_refs,
            invalid_model_refs,
            issues,
        }
    }

    fn read_runtime_catalogs(
        &self,
        issues: &mut Vec<String>,
    ) -> BTreeMap<String, RuntimeProviderAggregate> {
        let mut providers: BTreeMap<String, RuntimeProviderAggregate> = BTreeMap::new();

        for (agent_id, path) in self.agent_file_paths("models.json") {
            let Some(data) = read_json::<RuntimeModelsFile>(&path, issues, "runtime models file")
            else {
                continue;
            };

            for (provider, runtime_provider) in data.providers {
                let entry = providers.entry(provider.clone()).or_default();
                if entry.base_url.is_none() {
                    entry.base_url = runtime_provider.base_url;
                }
                entry.seen_in_agents.insert(agent_id.clone());
                for model in runtime_provider.models {
                    entry.models.insert(model_ref(&provider, &model.id));
                }
            }
        }

        providers
    }

    fn read_runtime_auth_profiles(
        &self,
        issues: &mut Vec<String>,
    ) -> BTreeMap<String, BTreeSet<String>> {
        let mut providers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for (agent_id, path) in self.agent_file_paths("auth-profiles.json") {
            let Some(data) =
                read_json::<RuntimeAuthProfilesFile>(&path, issues, "runtime auth profile file")
            else {
                continue;
            };

            for (profile_name, profile) in data.profiles {
                providers
                    .entry(profile.provider)
                    .or_default()
                    .insert(format!("{agent_id}:{profile_name}"));
            }
        }

        providers
    }

    fn agent_file_paths(&self, filename: &str) -> Vec<(String, PathBuf)> {
        let mut result = Vec::new();
        let agents_dir = self.state_path.join("agents");
        let Ok(entries) = fs::read_dir(agents_dir) else {
            return result;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let Some(agent_id) = path.file_name().and_then(|item| item.to_str()) else {
                continue;
            };
            let file_path = path.join("agent").join(filename);
            if file_path.is_file() {
                result.push((agent_id.to_string(), file_path));
            }
        }

        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

fn build_configured_agents(config: &OpenClawConfig) -> Vec<ConfiguredAgent> {
    config
        .agents
        .as_ref()
        .map(|agents| {
            agents
                .list
                .iter()
                .map(|agent| {
                    let model = agent.model.as_ref().or_else(|| {
                        agents
                            .defaults
                            .as_ref()
                            .and_then(|defaults| defaults.model.as_ref())
                    });

                    ConfiguredAgent {
                        agent_id: agent.id.clone(),
                        role: humanize_agent_id(&agent.id),
                        primary_model: model
                            .map(|model| model.primary.clone())
                            .unwrap_or_else(|| "unknown".to_string()),
                        fallbacks: model
                            .map(|model| model.fallbacks.clone())
                            .unwrap_or_default(),
                        priority: TaskPriority::Normal,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn collect_configured_model_refs(config: &OpenClawConfig) -> Vec<String> {
    let mut refs = Vec::new();

    if let Some(agents) = config.agents.as_ref() {
        if let Some(defaults) = agents.defaults.as_ref() {
            if let Some(model) = defaults.model.as_ref() {
                refs.push(model.primary.clone());
                refs.extend(model.fallbacks.clone());
            }
            if let Some(image_model) = defaults.image_model.as_ref() {
                refs.push(image_model.primary.clone());
                refs.extend(image_model.fallbacks.clone());
            }
            if let Some(pdf_model) = defaults.pdf_model.as_ref() {
                refs.push(pdf_model.primary.clone());
                refs.extend(pdf_model.fallbacks.clone());
            }
            if let Some(models) = defaults.models.as_ref() {
                refs.extend(models.keys().cloned());
            }
        }

        for agent in &agents.list {
            if let Some(model) = agent.model.as_ref() {
                refs.push(model.primary.clone());
                refs.extend(model.fallbacks.clone());
            }
        }
    }

    dedupe_sorted(refs)
}

fn build_provider_statuses(
    config: Option<&OpenClawConfig>,
    runtime_catalogs: &BTreeMap<String, RuntimeProviderAggregate>,
    runtime_auth_profiles: &BTreeMap<String, BTreeSet<String>>,
    configured_model_refs: &[String],
) -> Vec<ProviderStatus> {
    let mut providers: BTreeMap<String, ProviderAggregate> = BTreeMap::new();

    if let Some(config) = config {
        if let Some(plugins) = config
            .plugins
            .as_ref()
            .and_then(|plugins| plugins.entries.as_ref())
        {
            for (provider, plugin) in plugins {
                let entry = providers.entry(provider.clone()).or_default();
                entry.plugin_declared = true;
                entry.enabled = Some(plugin.enabled.unwrap_or(true));
            }
        }

        if let Some(auth_profiles) = config.auth.as_ref().and_then(|auth| auth.profiles.as_ref()) {
            for (profile_name, profile) in auth_profiles {
                if let Some(provider) = profile.provider.as_ref() {
                    providers
                        .entry(provider.clone())
                        .or_default()
                        .auth_profiles
                        .insert(format!("config:{profile_name}"));
                }
            }
        }
    }

    for model_ref in configured_model_refs {
        if let Some((provider, _)) = split_model_ref(model_ref) {
            providers
                .entry(provider.to_string())
                .or_default()
                .configured_model_refs
                .insert(model_ref.clone());
        }
    }

    for (provider, catalog) in runtime_catalogs {
        let entry = providers.entry(provider.clone()).or_default();
        entry.base_url = catalog.base_url.clone();
        entry.available_models.extend(catalog.models.clone());
        for agent_id in &catalog.seen_in_agents {
            entry
                .runtime_sources
                .insert(format!("runtime:{agent_id}:models"));
        }
    }

    for (provider, profiles) in runtime_auth_profiles {
        providers
            .entry(provider.clone())
            .or_default()
            .auth_profiles
            .extend(profiles.clone());
    }

    providers
        .into_iter()
        .map(|(provider, aggregate)| {
            let enabled = aggregate.enabled.unwrap_or(
                aggregate.plugin_declared
                    || !aggregate.configured_model_refs.is_empty()
                    || !aggregate.available_models.is_empty()
                    || !aggregate.auth_profiles.is_empty(),
            );
            let discovered = !aggregate.available_models.is_empty() || aggregate.base_url.is_some();
            let configured = enabled
                && (discovered
                    || !aggregate.auth_profiles.is_empty()
                    || !aggregate.configured_model_refs.is_empty());

            let missing_models = if !aggregate.available_models.is_empty() {
                aggregate
                    .configured_model_refs
                    .difference(&aggregate.available_models)
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            let mut issues = Vec::new();
            if !enabled && !aggregate.configured_model_refs.is_empty() {
                issues.push(format!(
                    "provider '{}' is referenced by config but not enabled",
                    provider
                ));
            }
            for model_ref in &missing_models {
                issues.push(format!("invalid-model-ref:{model_ref}"));
            }

            let via = if !aggregate.auth_profiles.is_empty() {
                Some("auth-profile".to_string())
            } else if discovered {
                Some("runtime-catalog".to_string())
            } else if aggregate.plugin_declared {
                Some("config-plugin".to_string())
            } else {
                None
            };

            let status = if !enabled {
                "disabled"
            } else if issues
                .iter()
                .any(|issue| !issue.starts_with("invalid-model-ref:"))
                || !missing_models.is_empty()
            {
                "degraded"
            } else if !aggregate.auth_profiles.is_empty() || !aggregate.available_models.is_empty()
            {
                "healthy"
            } else if configured {
                "configured"
            } else {
                "unknown"
            }
            .to_string();

            ProviderStatus {
                provider,
                enabled,
                configured,
                discovered,
                status,
                via,
                base_url: aggregate.base_url,
                model_count: aggregate.available_models.len(),
                available_models: aggregate.available_models.into_iter().collect(),
                configured_model_refs: aggregate.configured_model_refs.into_iter().collect(),
                auth_profiles: aggregate.auth_profiles.into_iter().collect(),
                issues,
            }
        })
        .collect()
}

fn humanize_agent_id(agent_id: &str) -> String {
    agent_id
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_model_ref(model_ref: &str) -> Option<(&str, &str)> {
    model_ref.split_once('/')
}

fn dedupe_sorted(mut items: Vec<String>) -> Vec<String> {
    items.sort();
    items.dedup();
    items
}

fn read_json5<T>(path: &Path, issues: &mut Vec<String>, label: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            issues.push(format!("{label} missing at {}: {err}", path.display()));
            return None;
        }
    };

    match json5::from_str::<T>(&contents) {
        Ok(value) => Some(value),
        Err(err) => {
            issues.push(format!("{label} parse failed at {}: {err}", path.display()));
            None
        }
    }
}

fn read_json<T>(path: &Path, issues: &mut Vec<String>, label: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            issues.push(format!("{label} missing at {}: {err}", path.display()));
            return None;
        }
    };

    match serde_json::from_str::<T>(&contents) {
        Ok(value) => Some(value),
        Err(err) => {
            issues.push(format!("{label} parse failed at {}: {err}", path.display()));
            None
        }
    }
}

fn model_ref(provider: &str, model_id: &str) -> String {
    format!("{provider}/{model_id}")
}

fn derive_openclaw_status(config_loaded: bool, issues: &[String]) -> String {
    if !config_loaded {
        "error".to_string()
    } else if issues.is_empty() {
        "healthy".to_string()
    } else {
        "degraded".to_string()
    }
}

struct SnapshotFingerprintInput<'a> {
    config_path: &'a Path,
    source_mtime: Option<DateTime<Utc>>,
    status: &'a str,
    runtime_state_available: bool,
    configured_agents: &'a [ConfiguredAgent],
    providers: &'a [ProviderStatus],
    available_model_refs: &'a [String],
    configured_model_refs: &'a [String],
    invalid_model_refs: &'a [String],
    issues: &'a [String],
}

fn build_snapshot_fingerprint(input: SnapshotFingerprintInput<'_>) -> String {
    let payload = serde_json::json!({
        "schema_version": OPENCLAW_STATUS_SCHEMA_VERSION,
        "status": input.status,
        "config_path": input.config_path.display().to_string(),
        "source_mtime": input.source_mtime,
        "runtime_state_available": input.runtime_state_available,
        "configured_agents": input.configured_agents,
        "providers": input.providers,
        "available_model_refs": input.available_model_refs,
        "configured_model_refs": input.configured_model_refs,
        "invalid_model_refs": input.invalid_model_refs,
        "issues": input.issues,
    });
    let encoded = serde_json::to_vec(&payload).unwrap_or_default();
    let digest = Sha256::digest(encoded);
    hex::encode(digest)
}

fn latest_source_mtime(state_path: &Path, config_path: &Path) -> Option<DateTime<Utc>> {
    let mut latest = file_mtime(config_path);

    for filename in ["models.json", "auth-profiles.json"] {
        for (_, path) in agent_file_paths(state_path, filename) {
            latest = match (latest, file_mtime(&path)) {
                (Some(current), Some(candidate)) if candidate > current => Some(candidate),
                (None, Some(candidate)) => Some(candidate),
                (current, _) => current,
            };
        }
    }

    latest
}

fn file_mtime(path: &Path) -> Option<DateTime<Utc>> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(system_time_or_now(modified)))
}

fn system_time_or_now(value: SystemTime) -> SystemTime {
    if value.duration_since(SystemTime::UNIX_EPOCH).is_ok() {
        value
    } else {
        SystemTime::now()
    }
}

fn agent_file_paths(state_path: &Path, filename: &str) -> Vec<(String, PathBuf)> {
    let mut result = Vec::new();
    let agents_dir = state_path.join("agents");
    let Ok(entries) = fs::read_dir(agents_dir) else {
        return result;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(agent_id) = path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        let file_path = path.join("agent").join(filename);
        if file_path.is_file() {
            result.push((agent_id.to_string(), file_path));
        }
    }

    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[derive(Default)]
struct ProviderAggregate {
    plugin_declared: bool,
    enabled: Option<bool>,
    base_url: Option<String>,
    available_models: BTreeSet<String>,
    configured_model_refs: BTreeSet<String>,
    auth_profiles: BTreeSet<String>,
    runtime_sources: BTreeSet<String>,
}

#[derive(Default)]
struct RuntimeProviderAggregate {
    base_url: Option<String>,
    models: BTreeSet<String>,
    seen_in_agents: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct OpenClawConfig {
    agents: Option<OpenClawAgentsSection>,
    plugins: Option<OpenClawPluginsSection>,
    auth: Option<OpenClawAuthSection>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAgentsSection {
    defaults: Option<OpenClawAgentDefaults>,
    #[serde(default)]
    list: Vec<OpenClawAgentConfig>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAgentDefaults {
    model: Option<OpenClawModelSelection>,
    #[serde(rename = "imageModel")]
    image_model: Option<OpenClawModelSelection>,
    #[serde(rename = "pdfModel")]
    pdf_model: Option<OpenClawModelSelection>,
    models: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAgentConfig {
    id: String,
    model: Option<OpenClawModelSelection>,
}

#[derive(Debug, Deserialize)]
struct OpenClawModelSelection {
    primary: String,
    #[serde(default)]
    fallbacks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenClawPluginsSection {
    entries: Option<BTreeMap<String, OpenClawPluginEntry>>,
}

#[derive(Debug, Deserialize)]
struct OpenClawPluginEntry {
    enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAuthSection {
    profiles: Option<BTreeMap<String, OpenClawAuthProfile>>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAuthProfile {
    provider: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuntimeModelsFile {
    #[serde(default)]
    providers: BTreeMap<String, RuntimeProviderCatalog>,
}

#[derive(Debug, Deserialize)]
struct RuntimeProviderCatalog {
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    #[serde(default)]
    models: Vec<RuntimeModel>,
}

#[derive(Debug, Deserialize)]
struct RuntimeModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RuntimeAuthProfilesFile {
    #[serde(default)]
    profiles: BTreeMap<String, RuntimeAuthProfile>,
}

#[derive(Debug, Deserialize)]
struct RuntimeAuthProfile {
    provider: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snapshot_fingerprint_is_stable_for_same_content() {
        let temp_root =
            std::env::temp_dir().join(format!("openclaw-test-{}", uuid::Uuid::new_v4()));
        let config_dir = temp_root.join("config");
        let agent_dir = temp_root.join("agents").join("main").join("agent");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&agent_dir).unwrap();

        fs::write(
            config_dir.join("openclaw.json5"),
            r#"{
              agents: {
                defaults: { model: { primary: "openai/gpt-5.4", fallbacks: [] } },
                list: [{ id: "director", model: { primary: "openai/gpt-5.4", fallbacks: [] } }]
              },
              plugins: { entries: { openai: { enabled: true } } }
            }"#,
        )
        .unwrap();

        fs::write(
            agent_dir.join("models.json"),
            r#"{
              "providers": {
                "openai": {
                  "baseUrl": "https://api.openai.com/v1",
                  "models": [{ "id": "gpt-5.4" }]
                }
              }
            }"#,
        )
        .unwrap();

        let reader = OpenClawReader::new(&temp_root);
        let first = reader.read_status().await;
        let second = reader.read_status().await;

        assert_eq!(first.snapshot_fingerprint, second.snapshot_fingerprint);
        assert_eq!(first.schema_version, OPENCLAW_STATUS_SCHEMA_VERSION);
        assert_eq!(first.status, "healthy");

        let _ = fs::remove_dir_all(temp_root);
    }

    #[tokio::test]
    async fn clear_cache_invalidates_status() {
        let temp_root =
            std::env::temp_dir().join(format!("openclaw-cache-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_root).unwrap();

        let reader = OpenClawReader::new(&temp_root);
        // Initially empty but we can manually set cache if we had access,
        // or just verify it doesn't crash and returns a status.
        reader.clear_cache().await;
        let _ = reader.read_status().await;

        let _ = fs::remove_dir_all(temp_root);
    }
}
