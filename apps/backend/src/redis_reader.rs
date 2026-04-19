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

    pub async fn get_agents_status(&self) -> Result<Vec<Agent>, String> {
        let mut conn = self.client.clone();
        let data: Option<String> = conn.get("dash:agents:status")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        let Some(data) = data else {
            return Ok(Vec::new());
        };

        let json: Value = serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))?;

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
        let mut conn = self.client.clone();
        let data: Option<String> = conn
            .get("dash:agents:configured")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        let Some(data) = data else {
            return Ok(Vec::new());
        };

        let json: Value = serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))?;

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
        let data: Option<String> = conn.get("dash:queue:summary")
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

        serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    pub async fn get_recent_events(&self) -> Result<Vec<DashboardEvent>, String> {
        let mut conn = self.client.clone();
        let data: Option<String> = conn.get("dash:events:recent")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        let Some(data) = data else {
            return Ok(Vec::new());
        };

        let json: Value = serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))?;

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
}
