use crate::models::*;
use chrono::Utc;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct PostgresReader {
    pool: PgPool,
}

impl PostgresReader {
    pub fn new(pool: PgPool) -> Self {
        PostgresReader { pool }
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn get_tasks(&self, limit: usize, offset: usize) -> Result<Vec<Task>, String> {
        let rows = sqlx::query(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter().map(Self::row_to_task).collect()
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let row = sqlx::query(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        row.map(Self::row_to_task).transpose()
    }

    pub async fn create_task(
        &self,
        title: &str,
        description: &str,
        priority: TaskPriority,
        owner_agent: &str,
        created_by_agent: &str,
        mission_id: Option<&str>,
    ) -> Result<(String, chrono::DateTime<Utc>), String> {
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let now = Utc::now();
        let priority_str = to_priority_str(&priority);

        sqlx::query(
            "INSERT INTO tasks (id, title, description, priority, status, owner_agent, mission_id, created_by_agent, started_at, completed_at, created_at, updated_at, blocked_reason)
             VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, NULL, $8, $8, NULL)",
        )
        .bind(&task_id)
        .bind(title)
        .bind(description)
        .bind(priority_str)
        .bind(owner_agent)
        .bind(mission_id)
        .bind(created_by_agent)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Insert task error: {e}"))?;

        let event_id = format!("evt-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO task_events (id, task_id, event_type, summary, created_at) VALUES ($1, $2, 'created', $3, $4)",
        )
        .bind(event_id)
        .bind(&task_id)
        .bind(format!("Task created: {}", title))
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Insert task event error: {e}"))?;

        Ok((task_id, now))
    }

    pub async fn create_mission(
        &self,
        title: &str,
        description: &str,
        created_by_agent: &str,
    ) -> Result<(String, String, chrono::DateTime<Utc>), String> {
        let mission_id = format!("mission-{}", uuid::Uuid::new_v4());
        let (root_task_id, created_at) = self
            .create_task(
                &format!("Mission Root: {}", title),
                description,
                TaskPriority::High,
                created_by_agent,
                created_by_agent,
                Some(&mission_id),
            )
            .await?;

        sqlx::query(
            "INSERT INTO missions (id, root_task_id, title, description, status, created_by_agent, closed_by_agent, created_at, closed_at)
             VALUES ($1, $2, $3, $4, 'active', $5, NULL, $6, NULL)",
        )
        .bind(&mission_id)
        .bind(&root_task_id)
        .bind(title)
        .bind(description)
        .bind(created_by_agent)
        .bind(created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Insert mission error: {e}"))?;

        Ok((mission_id, root_task_id, created_at))
    }

    pub async fn get_mission(&self, mission_id: &str) -> Result<Option<Mission>, String> {
        let row = sqlx::query(
            "SELECT * FROM missions WHERE id = $1",
        )
        .bind(mission_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Query mission error: {e}"))?;

        row.map(|r| map_mission_row(&r)).transpose()
    }

    pub async fn get_all_missions(&self) -> Result<Vec<Mission>, String> {
        let rows = sqlx::query(
            "SELECT * FROM missions ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query all missions error: {e}"))?;

        rows.iter().map(map_mission_row).collect()
    }

    pub async fn get_mission_subtasks(&self, mission_id: &str, root_task_id: &str) -> Result<Vec<Task>, String> {
        let rows = sqlx::query(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason
             FROM tasks
             WHERE mission_id = $1 AND id != $2
             ORDER BY created_at ASC",
        )
        .bind(mission_id)
        .bind(root_task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query mission subtasks error: {e}"))?;

        rows.into_iter().map(Self::row_to_task).collect()
    }

    pub async fn close_mission(&self, mission_id: &str, closed_by_agent: &str) -> Result<chrono::DateTime<Utc>, String> {
        let closed_at = Utc::now();
        sqlx::query(
            "UPDATE missions SET status = 'closed', closed_by_agent = $1, closed_at = $2 WHERE id = $3",
        )
        .bind(closed_by_agent)
        .bind(closed_at)
        .bind(mission_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Close mission error: {e}"))?;

        Ok(closed_at)
    }

    pub async fn get_agents_runs(&self, limit: usize) -> Result<Vec<AgentRun>, String> {
        let rows = sqlx::query(
            "SELECT id, agent_id, task_id, model_name, status, started_at, ended_at FROM agent_runs ORDER BY started_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter()
            .map(|row| {
                Ok(AgentRun {
                    id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
                    agent_id: row.try_get("agent_id").map_err(|e| format!("Row error: {e}"))?,
                    task_id: row.try_get("task_id").map_err(|e| format!("Row error: {e}"))?,
                    model_name: row.try_get("model_name").map_err(|e| format!("Row error: {e}"))?,
                    status: row.try_get("status").map_err(|e| format!("Row error: {e}"))?,
                    started_at: row.try_get("started_at").map_err(|e| format!("Row error: {e}"))?,
                    ended_at: row.try_get("ended_at").map_err(|e| format!("Row error: {e}"))?,
                })
            })
            .collect()
    }

    pub async fn get_model_usage(&self, limit: usize) -> Result<Vec<ModelUsage>, String> {
        let rows = sqlx::query(
            "SELECT id, agent_id, model_name, prompt_tokens, completion_tokens, total_tokens, estimated_cost_usd, created_at FROM model_usage ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter()
            .map(|row| {
                Ok(ModelUsage {
                    id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
                    agent_id: row.try_get("agent_id").map_err(|e| format!("Row error: {e}"))?,
                    model_name: row.try_get("model_name").map_err(|e| format!("Row error: {e}"))?,
                    prompt_tokens: row.try_get("prompt_tokens").map_err(|e| format!("Row error: {e}"))?,
                    completion_tokens: row.try_get("completion_tokens").map_err(|e| format!("Row error: {e}"))?,
                    total_tokens: row.try_get("total_tokens").map_err(|e| format!("Row error: {e}"))?,
                    estimated_cost_usd: row.try_get("estimated_cost_usd").map_err(|e| format!("Row error: {e}"))?,
                    created_at: row.try_get("created_at").map_err(|e| format!("Row error: {e}"))?,
                })
            })
            .collect()
    }

    pub async fn get_council_runs(&self, limit: usize) -> Result<Vec<CouncilRun>, String> {
        let rows = sqlx::query(
            "SELECT id, title, status, phase, director_agent, ruling_summary, confidence, obsidian_path, created_at, updated_at FROM council_runs ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter()
            .map(|row| {
                Ok(CouncilRun {
                    id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
                    title: row.try_get("title").map_err(|e| format!("Row error: {e}"))?,
                    status: row.try_get("status").map_err(|e| format!("Row error: {e}"))?,
                    phase: parse_council_phase(&row.try_get::<String, _>("phase").map_err(|e| format!("Row error: {e}"))?),
                    director_agent: row.try_get("director_agent").map_err(|e| format!("Row error: {e}"))?,
                    participants: vec![],
                    ruling_summary: row.try_get("ruling_summary").map_err(|e| format!("Row error: {e}"))?,
                    confidence: row.try_get("confidence").map_err(|e| format!("Row error: {e}"))?,
                    obsidian_path: row.try_get("obsidian_path").map_err(|e| format!("Row error: {e}"))?,
                    created_at: row.try_get("created_at").map_err(|e| format!("Row error: {e}"))?,
                    updated_at: row.try_get("updated_at").map_err(|e| format!("Row error: {e}"))?,
                })
            })
            .collect()
    }

    pub async fn get_failed_tasks(&self) -> Result<Vec<Task>, String> {
        let rows = sqlx::query(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE status = 'failed' ORDER BY updated_at DESC LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter().map(Self::row_to_task).collect()
    }

    pub async fn get_blocked_tasks(&self) -> Result<Vec<Task>, String> {
        let rows = sqlx::query(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE status = 'blocked' ORDER BY updated_at DESC LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query error: {e}"))?;

        rows.into_iter().map(Self::row_to_task).collect()
    }

    pub async fn get_audit_events(&self, limit: usize) -> Result<Vec<AuditEvent>, String> {
        let rows = sqlx::query(
            "SELECT id, request_id, agent_id, operation, resource_type, resource_id, allowed, result, reason, metadata, created_at
             FROM audit_events ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Query audit events error: {e}"))?;

        rows.into_iter()
            .map(|row| {
                Ok(AuditEvent {
                    id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
                    request_id: row.try_get("request_id").map_err(|e| format!("Row error: {e}"))?,
                    agent_id: row.try_get("agent_id").map_err(|e| format!("Row error: {e}"))?,
                    operation: row.try_get("operation").map_err(|e| format!("Row error: {e}"))?,
                    resource_type: row.try_get("resource_type").map_err(|e| format!("Row error: {e}"))?,
                    resource_id: row.try_get("resource_id").map_err(|e| format!("Row error: {e}"))?,
                    allowed: row.try_get("allowed").map_err(|e| format!("Row error: {e}"))?,
                    result: row.try_get("result").map_err(|e| format!("Row error: {e}"))?,
                    reason: row.try_get("reason").map_err(|e| format!("Row error: {e}"))?,
                    metadata: row.try_get("metadata").map_err(|e| format!("Row error: {e}"))?,
                    created_at: row.try_get("created_at").map_err(|e| format!("Row error: {e}"))?,
                })
            })
            .collect()
    }

    fn row_to_task(row: sqlx::postgres::PgRow) -> Result<Task, String> {
        Ok(Task {
            id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
            title: row.try_get("title").map_err(|e| format!("Row error: {e}"))?,
            priority: parse_priority(&row.try_get::<String, _>("priority").map_err(|e| format!("Row error: {e}"))?),
            status: parse_status(&row.try_get::<String, _>("status").map_err(|e| format!("Row error: {e}"))?),
            owner_agent: row.try_get("owner_agent").map_err(|e| format!("Row error: {e}"))?,
            created_at: row.try_get("created_at").map_err(|e| format!("Row error: {e}"))?,
            updated_at: row.try_get("updated_at").map_err(|e| format!("Row error: {e}"))?,
            blocked_reason: row.try_get("blocked_reason").map_err(|e| format!("Row error: {e}"))?,
        })
    }
}

fn map_mission_row(row: &sqlx::postgres::PgRow) -> Result<Mission, String> {
    Ok(Mission {
        id: row.try_get("id").map_err(|e| format!("Row error: {e}"))?,
        root_task_id: row.try_get("root_task_id").map_err(|e| format!("Row error: {e}"))?,
        title: row.try_get("title").map_err(|e| format!("Row error: {e}"))?,
        description: row.try_get("description").map_err(|e| format!("Row error: {e}"))?,
        status: parse_mission_status(&row.try_get::<String, _>("status").map_err(|e| format!("Row error: {e}"))?),
        created_by_agent: row.try_get("created_by_agent").map_err(|e| format!("Row error: {e}"))?,
        closed_by_agent: row.try_get("closed_by_agent").map_err(|e| format!("Row error: {e}"))?,
        created_at: row.try_get("created_at").map_err(|e| format!("Row error: {e}"))?,
        closed_at: row.try_get("closed_at").map_err(|e| format!("Row error: {e}"))?,
    })
}

fn parse_priority(s: &str) -> TaskPriority {
    match s {
        "critical" => TaskPriority::Critical,
        "high" => TaskPriority::High,
        "normal" => TaskPriority::Normal,
        "low" => TaskPriority::Low,
        _ => TaskPriority::Normal,
    }
}

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "pending" => TaskStatus::Pending,
        "in_progress" => TaskStatus::InProgress,
        "reviewing" => TaskStatus::Reviewing,
        "blocked" => TaskStatus::Blocked,
        "completed" => TaskStatus::Completed,
        "failed" => TaskStatus::Failed,
        "cancelled" => TaskStatus::Cancelled,
        _ => TaskStatus::Pending,
    }
}

fn parse_mission_status(s: &str) -> MissionStatus {
    match s {
        "active" => MissionStatus::Active,
        "closed" => MissionStatus::Closed,
        _ => MissionStatus::Active,
    }
}

fn to_priority_str(priority: &TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Critical => "critical",
        TaskPriority::High => "high",
        TaskPriority::Normal => "normal",
        TaskPriority::Low => "low",
    }
}

fn parse_council_phase(s: &str) -> CouncilPhase {
    match s {
        "debating" => CouncilPhase::Debating,
        "voting" => CouncilPhase::Voting,
        "concluded" => CouncilPhase::Concluded,
        _ => CouncilPhase::Debating,
    }
}
