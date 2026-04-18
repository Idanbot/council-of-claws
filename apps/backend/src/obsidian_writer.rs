use crate::models::{Mission, Task, TaskStatus};
use chrono::{DateTime, Utc};
use std::env;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
pub struct ObsidianWriter {
    vault_path: PathBuf,
}

impl ObsidianWriter {
    pub fn new() -> Self {
        let vault_path = env::var("OBSIDIAN_VAULT_PATH")
            .unwrap_or_else(|_| "./data/obsidian".to_string());
        ObsidianWriter {
            vault_path: PathBuf::from(vault_path),
        }
    }

    fn get_base_path(&self, namespace: Option<&str>) -> PathBuf {
        match namespace {
            Some(ns) => self.vault_path.join(ns),
            None => self.vault_path.clone(),
        }
    }

    pub async fn write_task_summary(
        &self,
        task: &Task,
        mission_id: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<String, String> {
        let base = self.get_base_path(namespace);
        let task_rel_path = format!("Tasks/{}.md", task.id);
        let file_path = base.join(&task_rel_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create task directory: {e}"))?;
        }

        let mission_link = mission_id.map(|id| {
            format!("\nPart of Mission: [[Missions/{}]]\n", id)
        }).unwrap_or_default();

        let content = format!(
            "# Task: {}\n\nStatus: **{:?}**  \nTask ID: `{}`  \nPriority: **{:?}**  \nAssigned To: `{}`  \nCreated: {}  \nUpdated: {}  \n{}\n## Description\n\n{}\n\n## Audit Trail\n\nSee database for detailed event history.\n",
            task.title,
            task.status,
            task.id,
            task.priority,
            task.owner_agent,
            format_local_ts(task.created_at),
            format_local_ts(task.updated_at),
            mission_link,
            task.blocked_reason.as_deref().unwrap_or("No description provided"),
        );

        fs::write(&file_path, content)
            .await
            .map_err(|e| format!("failed to write task summary file: {e}"))?;

        Ok(format!(
            "obsidian://open?vault=vault_name&file={}",
            task_rel_path.replace(' ', "%20")
        ))
    }

    pub async fn write_mission_summary(
        &self,
        mission: &Mission,
        subtasks: &[Task],
        closed_at: DateTime<Utc>,
        notes: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<String, String> {
        let base = self.get_base_path(namespace);
        let mission_rel_path = format!("Missions/{}.md", mission.id);
        let file_path = base.join(&mission_rel_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("failed to create mission directory: {e}"))?;
        }

        let first_task_start = subtasks
            .iter()
            .map(|task| task.created_at)
            .min()
            .unwrap_or(mission.created_at);
        let last_task_completed = subtasks
            .iter()
            .map(|task| task.updated_at)
            .max()
            .unwrap_or(closed_at);

        let mut subtask_rows = String::new();
        for task in subtasks {
            let started = task.created_at;
            let completed = task.updated_at;
            let duration = completed.signed_duration_since(started);
            let duration_str = format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60);
            
            // Use internal wiki link format for bidirectional support
            let wiki_link = format!("[[Tasks/{}]]", task.id);

            subtask_rows.push_str(&format!(
                "| {} | {} | {} | {:?} | {} | {} | {} |\n",
                wiki_link,
                task.title,
                task.owner_agent,
                task.status,
                format_local_ts(started),
                format_local_ts(completed),
                duration_str,
            ));
        }

        let completed = subtasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
        let failed = subtasks.iter().filter(|t| matches!(t.status, TaskStatus::Failed)).count();
        let cancelled = subtasks.iter().filter(|t| matches!(t.status, TaskStatus::Cancelled)).count();
        let success_rate = if subtasks.is_empty() {
            0.0
        } else {
            (completed as f64 / subtasks.len() as f64) * 100.0
        };

        let content = format!(
            "# Mission: {}\n\nStatus: **CLOSED**  \nMission ID: `{}`  \nRoot Task ID: `{}`  \nCreated: {}  \nClosed: {}  \nCreated by: {}  \nClosed by: director  \n\n## Summary\n\n{}\n\n## Subtasks\n\n| Task | Title | Assigned To | Status | Started | Completed | Duration |\n|---------|-------|-------------|--------|---------|-----------|----------|\n{}\n## Timeline\n\n- **{}**: Mission created by {}\n- **{}**: First subtask started\n- **{}**: Final subtask completed\n- **{}**: Mission closed by director\n\n## Completion Summary\n\n- **Total subtasks**: {}\n- **Completed**: {}\n- **Failed**: {}\n- **Cancelled**: {}\n- **Success rate**: {:.1}%\n\n## Notes\n\n{}\n\n## Audit Trail\n\nAll subtask operations are logged. See individual task records for detailed audit history.\n",
            mission.title,
            mission.id,
            mission.root_task_id,
            format_local_ts(mission.created_at),
            format_local_ts(closed_at),
            mission.created_by_agent,
            mission.description,
            subtask_rows,
            format_local_ts(mission.created_at),
            mission.created_by_agent,
            format_local_ts(first_task_start),
            format_local_ts(last_task_completed),
            format_local_ts(closed_at),
            subtasks.len(),
            completed,
            failed,
            cancelled,
            success_rate,
            notes.unwrap_or("No closing notes"),
        );

        fs::write(&file_path, content)
            .await
            .map_err(|e| format!("failed to write mission summary file: {e}"))?;

        Ok(format!(
            "obsidian://open?vault=vault_name&file={}",
            mission_rel_path.replace(' ', "%20")
        ))
    }

}

fn format_local_ts(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}
