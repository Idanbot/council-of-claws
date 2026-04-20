use crate::models::AuditOperation;
use crate::websocket_hub::WsHub;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    tx: mpsc::Sender<AuditEventTask>,
}

pub struct AuditEventTask {
    pub request_id: Option<String>,
    pub agent_id: Option<String>,
    pub operation: AuditOperation,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub allowed: bool,
    pub result: Option<String>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl AuditService {
    pub fn new(pool: PgPool, redis: ConnectionManager, ws_hub: WsHub) -> Self {
        let (tx, mut rx) = mpsc::channel::<AuditEventTask>(1024);

        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                Self::process_log(&pool, &redis, &ws_hub, task).await;
            }
        });

        AuditService { tx }
    }

    pub async fn log(
        &self,
        request_id: Option<&str>,
        agent_id: Option<&str>,
        operation: AuditOperation,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        allowed: bool,
        result: Option<&str>,
        reason: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) {
        let task = AuditEventTask {
            request_id: request_id.map(|s| s.to_string()),
            agent_id: agent_id.map(|s| s.to_string()),
            operation,
            resource_type: resource_type.map(|s| s.to_string()),
            resource_id: resource_id.map(|s| s.to_string()),
            allowed,
            result: result.map(|s| s.to_string()),
            reason: reason.map(|s| s.to_string()),
            metadata,
        };

        if let Err(e) = self.tx.send(task).await {
            tracing::error!("Failed to send audit event to worker: {}", e);
        }
    }

    async fn process_log(
        pool: &PgPool,
        redis: &ConnectionManager,
        ws_hub: &WsHub,
        task: AuditEventTask,
    ) {
        let id = format!("audit-{}", Uuid::new_v4());
        let op_str = task.operation.to_string();

        // 1. Persist to SQL
        let res = sqlx::query(
            "INSERT INTO audit_events (id, request_id, agent_id, operation, resource_type, resource_id, allowed, result, reason, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(&id)
        .bind(&task.request_id)
        .bind(&task.agent_id)
        .bind(&op_str)
        .bind(&task.resource_type)
        .bind(&task.resource_id)
        .bind(task.allowed)
        .bind(&task.result)
        .bind(&task.reason)
        .bind(&task.metadata)
        .execute(pool)
        .await;

        if let Err(e) = res {
            tracing::error!("Failed to write audit event to SQL: {}", e);
        }

        // 2. Broadcast to WebSockets
        let event_payload = json!({
            "event_type": "audit",
            "id": id,
            "agent_id": task.agent_id,
            "operation": op_str,
            "resource_type": task.resource_type,
            "resource_id": task.resource_id,
            "allowed": task.allowed,
            "result": task.result,
            "level": if task.allowed { "info" } else { "error" },
            "summary": task.reason.as_deref().unwrap_or("audit event"),
            "stream_connection": "audit",
            "timestamp": chrono::Utc::now().timestamp(),
            "created_at": chrono::Utc::now()
        });
        ws_hub.broadcast(event_payload.clone());

        // 3. Push to Redis Stream for durability/replay
        let mut conn = redis.clone();
        let _: redis::RedisResult<()> = conn
            .xadd(
                "coc:events:audit",
                "*",
                &[("data", event_payload.to_string())],
            )
            .await;

        let mut recent_events: Vec<serde_json::Value> = conn
            .get::<_, Option<String>>("dash:events:recent")
            .await
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_str::<serde_json::Value>(&data).ok())
            .and_then(|json| {
                json.get("events")
                    .and_then(|items| items.as_array())
                    .cloned()
            })
            .unwrap_or_default();

        recent_events.insert(
            0,
            json!({
                "level": if task.allowed { "info" } else { "error" },
                "summary": task.reason.as_deref().unwrap_or("audit event"),
                "stream_connection": "audit",
                "timestamp": chrono::Utc::now().timestamp(),
            }),
        );
        recent_events.truncate(25);

        let _: redis::RedisResult<()> = conn
            .set(
                "dash:events:recent",
                json!({ "events": recent_events }).to_string(),
            )
            .await;
    }
}
