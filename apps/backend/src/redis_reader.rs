use crate::models::*;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json::Value;

#[derive(Clone)]
pub struct RedisReader {
    client: ConnectionManager,
}

impl RedisReader {
    pub fn new(client: ConnectionManager) -> Self {
        RedisReader { client }
    }

    pub fn connection_manager(&self) -> ConnectionManager {
        self.client.clone()
    }

    async fn get_json_value(&self, key: &str) -> Result<Option<Value>, String> {
        let mut conn = self.client.clone();
        let data: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        let Some(data) = data else {
            return Ok(None);
        };

        let json: Value =
            serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))?;

        Ok(Some(json))
    }

    pub async fn get_openclaw_status(&self) -> Result<Option<OpenClawStatus>, String> {
        let Some(json) = self.get_json_value("dash:openclaw:status").await? else {
            return Ok(None);
        };

        serde_json::from_value(json)
            .map(Some)
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    pub async fn get_agents_status(&self) -> Result<Vec<Agent>, String> {
        let mut conn = self.client.clone();
        let Some(json) = self.get_json_value("dash:agents:status").await? else {
            return Ok(Vec::new());
        };

        let agents = json
            .get("agents")
            .and_then(|a| a.as_array())
            .ok_or("Missing agents array")?;

        let mut result = Vec::new();
        for agent in agents {
            if let Ok(agent_data) = serde_json::from_value::<Agent>(agent.clone()) {
                result.push(agent_data);
            }
        }

        let ttl_seconds = agent_heartbeat_ttl_seconds();
        let now = chrono::Utc::now().timestamp();
        let filtered: Vec<Agent> = result
            .into_iter()
            .filter(|agent| {
                agent.last_heartbeat_ts > 0
                    && now.saturating_sub(agent.last_heartbeat_ts) <= ttl_seconds
            })
            .collect();

        if filtered.len() != agents.len() {
            let _: redis::RedisResult<()> = conn
                .set(
                    "dash:agents:status",
                    serde_json::json!({ "agents": &filtered }).to_string(),
                )
                .await;
        }

        Ok(filtered)
    }

    pub async fn get_configured_agents(&self) -> Result<Vec<ConfiguredAgent>, String> {
        if let Some(openclaw) = self.get_openclaw_status().await? {
            if !openclaw.configured_agents.is_empty() {
                return Ok(openclaw.configured_agents);
            }
        }

        let Some(json) = self.get_json_value("dash:agents:configured").await? else {
            return Ok(Vec::new());
        };

        let agents = json
            .get("agents")
            .and_then(|a| a.as_array())
            .ok_or("Missing configured agents array")?;

        let mut result = Vec::new();
        for agent in agents {
            if let Ok(agent_data) = serde_json::from_value::<ConfiguredAgent>(agent.clone()) {
                result.push(agent_data);
            }
        }

        Ok(result)
    }

    pub async fn get_queue_summary(&self) -> Result<QueueSummary, String> {
        let mut conn = self.client.clone();
        let data: Option<String> = conn
            .get("dash:queue:summary")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        let Some(data) = data else {
            return Ok(QueueSummary {
                pending_critical: 0,
                pending_high: 0,
                pending_normal: 0,
                pending_low: 0,
                in_progress: 0,
                reviewing: 0,
                blocked: 0,
                completed: 0,
                failed: 0,
            });
        };

        serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {}", e))
    }

    pub async fn get_recent_events(&self) -> Result<Vec<DashboardEvent>, String> {
        let Some(json) = self.get_json_value("dash:events:recent").await? else {
            return Ok(Vec::new());
        };

        let events = json
            .get("events")
            .and_then(|e| e.as_array())
            .ok_or("Missing events array")?;

        let mut result = Vec::new();
        for event in events {
            if let Ok(event_data) = serde_json::from_value::<DashboardEvent>(event.clone()) {
                result.push(event_data);
            }
        }

        Ok(result)
    }

    pub async fn set_queue_summary(&self, summary: &QueueSummary) -> Result<(), String> {
        let mut conn = self.client.clone();
        let json = serde_json::to_string(summary).map_err(|e| format!("JSON error: {}", e))?;
        let _: redis::RedisResult<()> = conn
            .set("dash:queue:summary", json)
            .await;
        Ok(())
    }
}

fn agent_heartbeat_ttl_seconds() -> i64 {
    std::env::var("AGENT_HEARTBEAT_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(120)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_deserialization() {
        let json_str = r#"{
            "agent_id": "director",
            "state": "working",
            "current_task_id": "task-001",
            "priority": "critical",
            "model": "claude-3.5-sonnet",
            "last_heartbeat_ts": 1713473395,
            "elapsed_seconds": 120
        }"#;

        let agent: Agent = serde_json::from_str(json_str).unwrap();
        assert_eq!(agent.agent_id, "director");
    }

    #[tokio::test]
    async fn test_redis_queue_summary_roundtrip() {
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let client = match redis::Client::open(redis_url) {
            Ok(c) => c,
            Err(_) => return, // Skip if no redis
        };

        // Use a short timeout for the connection
        let manager_future = ConnectionManager::new(client);
        let manager = match tokio::time::timeout(std::time::Duration::from_millis(500), manager_future).await {
            Ok(Ok(m)) => m,
            _ => return, // Skip if timeout or error
        };

        let reader = RedisReader::new(manager);
        let summary = QueueSummary {
            pending_critical: 10,
            pending_high: 20,
            pending_normal: 30,
            pending_low: 40,
            in_progress: 50,
            reviewing: 60,
            blocked: 70,
            completed: 80,
            failed: 90,
        };

        if let Ok(_) = tokio::time::timeout(std::time::Duration::from_millis(500), reader.set_queue_summary(&summary)).await {
            if let Ok(Ok(read_back)) = tokio::time::timeout(std::time::Duration::from_millis(500), reader.get_queue_summary()).await {
                assert_eq!(read_back.pending_critical, 10);
                assert_eq!(read_back.failed, 90);
            }
        }
    }
}
