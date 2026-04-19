use crate::models::AuditOperation;
use crate::websocket_hub::WsHub;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct AuditService {
    pool: PgPool,
    redis: ConnectionManager,
    ws_hub: WsHub,
}

impl AuditService {
    pub fn new(pool: PgPool, redis: ConnectionManager, ws_hub: WsHub) -> Self {
        AuditService { pool, redis, ws_hub }
    }

    pub fn redis_connection(&self) -> ConnectionManager {
        self.redis.clone()
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
        let id = format!("audit-{}", Uuid::new_v4());
        let op_str = operation.to_string();

        // 1. Persist to SQL
        let res = sqlx::query(
            "INSERT INTO audit_events (id, request_id, agent_id, operation, resource_type, resource_id, allowed, result, reason, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(&id)
        .bind(request_id)
        .bind(agent_id)
        .bind(&op_str)
        .bind(resource_type)
        .bind(resource_id)
        .bind(allowed)
        .bind(result)
        .bind(reason)
        .bind(&metadata)
        .execute(&self.pool)
        .await;

        if let Err(e) = res {
            tracing::error!("Failed to write audit event to SQL: {}", e);
        }

        // 2. Broadcast to WebSockets
        let event_payload = json!({
            "event_type": "audit",
            "id": id,
            "agent_id": agent_id,
            "operation": op_str,
            "resource_type": resource_type,
            "resource_id": resource_id,
            "allowed": allowed,
            "result": result,
            "level": if allowed { "info" } else { "error" },
            "summary": reason.unwrap_or("audit event"),
            "stream_connection": "audit",
            "timestamp": chrono::Utc::now().timestamp(),
            "created_at": chrono::Utc::now()
        });
        self.ws_hub.broadcast(event_payload.clone());

        // 3. Push to Redis Stream for durability/replay
        let mut conn = self.redis.clone();
        let _: redis::RedisResult<()> = conn.xadd(
            "coc:events:audit",
            "*",
            &[("data", event_payload.to_string())]
        ).await;

        let mut recent_events: Vec<serde_json::Value> = conn
            .get::<_, Option<String>>("dash:events:recent")
            .await
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_str::<serde_json::Value>(&data).ok())
            .and_then(|json| json.get("events").and_then(|items| items.as_array()).cloned())
            .unwrap_or_default();

        recent_events.insert(0, json!({
            "level": if allowed { "info" } else { "error" },
            "summary": reason.unwrap_or("audit event"),
            "stream_connection": "audit",
            "timestamp": chrono::Utc::now().timestamp(),
        }));
        recent_events.truncate(25);

        let _: redis::RedisResult<()> = conn
            .set("dash:events:recent", json!({ "events": recent_events }).to_string())
            .await;
    }
}
