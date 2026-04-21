use crate::models::*;
use chrono::Utc;
use sqlx::{types::Json, PgPool};

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

    pub async fn get_tasks(&self, limit: usize, offset: usize) -> Result<Vec<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn create_task(
        &self,
        title: &str,
        description: &str,
        priority: TaskPriority,
        owner_agent: &str,
        created_by_agent: &str,
        mission_id: Option<&str>,
    ) -> Result<(String, chrono::DateTime<Utc>), AppError> {
        let task_id = format!("task-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO tasks (id, title, description, priority, status, owner_agent, mission_id, created_by_agent, started_at, completed_at, created_at, updated_at, blocked_reason)
             VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, $8, NULL, $8, $8, NULL)",
        )
        .bind(&task_id)
        .bind(title)
        .bind(description)
        .bind(&priority)
        .bind(owner_agent)
        .bind(mission_id)
        .bind(created_by_agent)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

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
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok((task_id, now))
    }

    pub async fn create_mission(
        &self,
        title: &str,
        description: &str,
        created_by_agent: &str,
    ) -> Result<(String, String, chrono::DateTime<Utc>), AppError> {
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
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok((mission_id, root_task_id, created_at))
    }

    pub async fn get_mission(&self, mission_id: &str) -> Result<Option<Mission>, AppError> {
        sqlx::query_as::<_, Mission>("SELECT * FROM missions WHERE id = $1")
            .bind(mission_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_all_missions(&self) -> Result<Vec<Mission>, AppError> {
        sqlx::query_as::<_, Mission>("SELECT * FROM missions ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_mission_subtasks(
        &self,
        mission_id: &str,
        root_task_id: &str,
    ) -> Result<Vec<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason
             FROM tasks
             WHERE mission_id = $1 AND id != $2
             ORDER BY created_at ASC",
        )
        .bind(mission_id)
        .bind(root_task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn close_mission(
        &self,
        mission_id: &str,
        closed_by_agent: &str,
    ) -> Result<chrono::DateTime<Utc>, AppError> {
        let closed_at = Utc::now();
        sqlx::query(
            "UPDATE missions SET status = 'closed', closed_by_agent = $1, closed_at = $2 WHERE id = $3",
        )
        .bind(closed_by_agent)
        .bind(closed_at)
        .bind(mission_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(closed_at)
    }

    pub async fn get_agents_runs(&self, limit: usize) -> Result<Vec<AgentRun>, AppError> {
        sqlx::query_as::<_, AgentRun>(
            "SELECT id, agent_id, task_id, model_name, status, started_at, ended_at FROM agent_runs ORDER BY started_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_model_usage(&self, limit: usize) -> Result<Vec<ModelUsage>, AppError> {
        sqlx::query_as::<_, ModelUsage>(
            "SELECT id, agent_id, model_name, prompt_tokens, completion_tokens, total_tokens, estimated_cost_usd, created_at FROM model_usage ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_council_runs(&self, limit: usize) -> Result<Vec<CouncilRun>, AppError> {
        sqlx::query_as::<_, CouncilRun>(
            "SELECT id, title, status, phase, director_agent, ruling_summary, confidence, obsidian_path, created_at, updated_at FROM council_runs ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_failed_tasks(&self) -> Result<Vec<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE status = 'failed' ORDER BY updated_at DESC LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_blocked_tasks(&self) -> Result<Vec<Task>, AppError> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, priority, status, owner_agent, created_at, updated_at, blocked_reason FROM tasks WHERE status = 'blocked' ORDER BY updated_at DESC LIMIT 10",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_audit_events(&self, limit: usize) -> Result<Vec<AuditEvent>, AppError> {
        sqlx::query_as::<_, AuditEvent>(
            "SELECT id, request_id, agent_id, operation, resource_type, resource_id, allowed, result, reason, metadata, created_at
             FROM audit_events ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn persist_openclaw_snapshot(
        &self,
        snapshot: &OpenClawStatus,
    ) -> Result<bool, AppError> {
        let payload = serde_json::to_value(snapshot)
            .map_err(|e| AppError::Internal(format!("Serialize OpenClaw snapshot error: {e}")))?;

        let result = sqlx::query(
            "INSERT INTO openclaw_snapshots (
                id,
                schema_version,
                snapshot_fingerprint,
                status,
                source_path,
                config_path,
                source_mtime,
                generated_at,
                last_success_at,
                provider_count,
                configured_agent_count,
                available_model_ref_count,
                invalid_model_ref_count,
                payload
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
            ON CONFLICT (snapshot_fingerprint) DO NOTHING",
        )
        .bind(format!("openclaw-{}", snapshot.snapshot_fingerprint))
        .bind(snapshot.schema_version)
        .bind(&snapshot.snapshot_fingerprint)
        .bind(&snapshot.status)
        .bind(&snapshot.source_path)
        .bind(&snapshot.config_path)
        .bind(snapshot.source_mtime)
        .bind(snapshot.generated_at)
        .bind(snapshot.last_success_at)
        .bind(snapshot.providers.len() as i32)
        .bind(snapshot.configured_agents.len() as i32)
        .bind(snapshot.available_model_refs.len() as i32)
        .bind(snapshot.invalid_model_refs.len() as i32)
        .bind(Json(payload))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_queue_summary(&self) -> Result<QueueSummary, AppError> {
        let row = sqlx::query(
            "SELECT
                COUNT(*) FILTER (WHERE status = 'pending' AND priority = 'critical')::INT as pending_critical,
                COUNT(*) FILTER (WHERE status = 'pending' AND priority = 'high')::INT as pending_high,
                COUNT(*) FILTER (WHERE status = 'pending' AND priority = 'normal')::INT as pending_normal,
                COUNT(*) FILTER (WHERE status = 'pending' AND priority = 'low')::INT as pending_low,
                COUNT(*) FILTER (WHERE status = 'in_progress')::INT as in_progress,
                COUNT(*) FILTER (WHERE status = 'reviewing')::INT as reviewing,
                COUNT(*) FILTER (WHERE status = 'blocked')::INT as blocked,
                COUNT(*) FILTER (WHERE status = 'completed')::INT as completed,
                COUNT(*) FILTER (WHERE status = 'failed')::INT as failed
             FROM tasks"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        use sqlx::Row;
        Ok(QueueSummary {
            pending_critical: row.get("pending_critical"),
            pending_high: row.get("pending_high"),
            pending_normal: row.get("pending_normal"),
            pending_low: row.get("pending_low"),
            in_progress: row.get("in_progress"),
            reviewing: row.get("reviewing"),
            blocked: row.get("blocked"),
            completed: row.get("completed"),
            failed: row.get("failed"),
        })
    }

    pub async fn claim_task(&self, task_id: &str, agent_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE tasks SET status = 'in_progress', owner_agent = $1, started_at = NOW(), updated_at = NOW() WHERE id = $2"
        )
        .bind(agent_id)
        .bind(task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn complete_task(&self, task_id: &str, notes: Option<&str>) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE tasks SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(n) = notes {
            let event_id = format!("evt-{}", uuid::Uuid::new_v4());
            sqlx::query(
                "INSERT INTO task_events (id, task_id, event_type, summary, created_at) VALUES ($1, $2, 'completed', $3, NOW())",
            )
            .bind(event_id)
            .bind(task_id)
            .bind(n)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn fail_task(&self, task_id: &str, reason: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE tasks SET status = 'failed', blocked_reason = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(reason)
        .bind(task_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let event_id = format!("evt-{}", uuid::Uuid::new_v4());
        sqlx::query(
            "INSERT INTO task_events (id, task_id, event_type, summary, created_at) VALUES ($1, $2, 'failed', $3, NOW())",
        )
        .bind(event_id)
        .bind(task_id)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_openclaw_snapshot_history_summary(
        &self,
    ) -> Result<OpenClawSnapshotHistorySummary, AppError> {
        let row = sqlx::query(
            "SELECT
                COUNT(*)::BIGINT AS snapshot_count,
                MAX(generated_at) AS latest_generated_at,
                MAX(created_at) AS latest_persisted_at
             FROM openclaw_snapshots",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Query OpenClaw snapshot history summary error: {e}")))?;

        let latest_fingerprint = sqlx::query(
            "SELECT snapshot_fingerprint
             FROM openclaw_snapshots
             ORDER BY generated_at DESC, created_at DESC
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Query latest OpenClaw snapshot fingerprint error: {e}")))?
        .map(|row| {
            use sqlx::Row;
            row.try_get("snapshot_fingerprint")
        })
        .transpose()
        .map_err(|e| AppError::Database(format!("Row error: {e}")))?;

        use sqlx::Row;
        Ok(OpenClawSnapshotHistorySummary {
            snapshot_count: row
                .try_get::<i64, _>("snapshot_count")
                .map_err(|e| AppError::Database(format!("Row error: {e}")))?,
            latest_generated_at: row
                .try_get("latest_generated_at")
                .map_err(|e| AppError::Database(format!("Row error: {e}")))?,
            latest_persisted_at: row
                .try_get("latest_persisted_at")
                .map_err(|e| AppError::Database(format!("Row error: {e}")))?,
            latest_snapshot_fingerprint: latest_fingerprint,
        })
    }
}
