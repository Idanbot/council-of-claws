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
        let data: String = conn.get("dash:agents:status")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

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

        Ok(result)
    }

    pub async fn get_queue_summary(&self) -> Result<QueueSummary, String> {
        let mut conn = self.client.clone();
        let data: String = conn.get("dash:queue:summary")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    pub async fn get_recent_events(&self) -> Result<Vec<DashboardEvent>, String> {
        let mut conn = self.client.clone();
        let data: String = conn.get("dash:events:recent")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

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

    pub async fn get_system_health(&self) -> Result<SystemHealth, String> {
        let mut conn = self.client.clone();
        let data: String = conn.get("dash:system:health")
            .await
            .map_err(|e| format!("Redis error: {}", e))?;

        serde_json::from_str(&data)
            .map_err(|e| format!("JSON parse error: {}", e))
    }
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
