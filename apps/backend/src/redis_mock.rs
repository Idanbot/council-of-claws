use redis::{AsyncCommands, RedisResult};
use serde_json::json;

pub async fn init_mock_data(client: &redis::aio::ConnectionManager) -> RedisResult<()> {
    // Set up agent status data
    let agents_status = json!({
        "agents": [
            {
                "agent_id": "contractor",
                "state": "idle",
                "current_task_id": null,
                "priority": "normal",
                "model": "claude-3-haiku",
                "last_heartbeat_ts": 1713473400i64,
                "elapsed_seconds": 0
            },
            {
                "agent_id": "director",
                "state": "working",
                "current_task_id": "task-005",
                "priority": "critical",
                "model": "claude-3.5-sonnet",
                "last_heartbeat_ts": 1713473395i64,
                "elapsed_seconds": 120
            },
            {
                "agent_id": "architect",
                "state": "reviewing",
                "current_task_id": "task-001",
                "priority": "high",
                "model": "claude-3-opus",
                "last_heartbeat_ts": 1713473390i64,
                "elapsed_seconds": 300
            },
            {
                "agent_id": "senior-engineer",
                "state": "working",
                "current_task_id": "task-001",
                "priority": "high",
                "model": "claude-3.5-sonnet",
                "last_heartbeat_ts": 1713473398i64,
                "elapsed_seconds": 45
            },
            {
                "agent_id": "junior-engineer",
                "state": "blocked",
                "current_task_id": "task-003",
                "priority": "normal",
                "model": "claude-3-haiku",
                "last_heartbeat_ts": 1713472800i64,
                "elapsed_seconds": 1650
            },
            {
                "agent_id": "intern",
                "state": "idle",
                "current_task_id": null,
                "priority": "low",
                "model": "claude-3-haiku",
                "last_heartbeat_ts": 1713473200i64,
                "elapsed_seconds": 0
            }
        ]
    });

    let mut conn = client.clone();
    let _: () = conn.set("dash:agents:status", agents_status.to_string()).await?;

    // Set up queue summary
    let queue_summary = json!({
        "pending_critical": 1,
        "pending_high": 2,
        "pending_normal": 3,
        "pending_low": 0,
        "in_progress": 4,
        "reviewing": 1,
        "blocked": 2,
        "completed": 5,
        "failed": 1
    });

    let _: () = conn.set("dash:queue:summary", queue_summary.to_string()).await?;

    // Set up recent events
    let recent_events = json!({
        "events": [
            {
                "level": "info",
                "summary": "senior-engineer started work on task-001",
                "stream_connection": "ws",
                "timestamp": 1713473398
            },
            {
                "level": "info",
                "summary": "director completed task-005 review",
                "stream_connection": "ws",
                "timestamp": 1713473395
            },
            {
                "level": "warn",
                "summary": "junior-engineer stalled on task-003 for 27 minutes",
                "stream_connection": "poll",
                "timestamp": 1713473200
            },
            {
                "level": "info",
                "summary": "architect transitioned to reviewing state",
                "stream_connection": "ws",
                "timestamp": 1713473150
            },
            {
                "level": "error",
                "summary": "task-004 failed: session timeout error",
                "stream_connection": "poll",
                "timestamp": 1713470000
            },
            {
                "level": "info",
                "summary": "council-001 concluded with agreement on schema",
                "stream_connection": "ws",
                "timestamp": 1713369600
            }
        ]
    });

    let _: () = conn.set("dash:events:recent", recent_events.to_string()).await?;

    // Set up council live data
    let council_live = json!({
        "active_councils": [
            {
                "council_id": "council-003",
                "title": "WebSocket vs Polling for Live Updates",
                "phase": "debating",
                "participants": ["director", "senior-engineer", "architect", "contractor"],
                "participants_count": 4,
                "current_round": 2,
                "total_rounds_completed": 1
            }
        ],
        "recent_councils": [
            {
                "council_id": "council-002",
                "title": "Session Timeout Strategy Discussion",
                "phase": "concluded",
                "ruling": "Rejected time-based approach",
                "confidence": 0.72
            },
            {
                "council_id": "council-001",
                "title": "Architecture Review: Dashboard Schema",
                "phase": "concluded",
                "ruling": "Approved normalized schema",
                "confidence": 0.95
            }
        ]
    });

    let _: () = conn.set("dash:council:live", council_live.to_string()).await?;

    // Set up system health
    let system_health = json!({
        "timestamp": 1713473400i64,
        "metrics": {
            "host": {
                "cpu_percent": 35.2,
                "memory_percent": 62.1,
                "disk_percent": 45.8
            },
            "redis": {
                "status": "healthy",
                "memory_usage_mb": 128.5,
                "connected_clients": 5
            },
            "postgres": {
                "status": "healthy",
                "connections_active": 3,
                "connections_max": 20
            },
            "backend": {
                "status": "healthy",
                "uptime_seconds": 86400,
                "request_count": 12450
            },
            "frontend": {
                "status": "healthy",
                "uptime_seconds": 86400,
                "users_online": 2
            },
            "containers": {
                "running": 6,
                "stopped": 0,
                "unhealthy": 0
            }
        }
    });

    let _: () = conn.set("dash:system:health", system_health.to_string()).await?;

    // Set expiration for live data (1 hour)
    let _: bool = conn.expire("dash:agents:status", 3600).await?;
    let _: bool = conn.expire("dash:queue:summary", 3600).await?;
    let _: bool = conn.expire("dash:events:recent", 3600).await?;
    let _: bool = conn.expire("dash:council:live", 3600).await?;
    let _: bool = conn.expire("dash:system:health", 3600).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_data_structure() {
        // Validate JSON structures are well-formed
        let agents_status = json!({
            "agents": [
                {
                    "agent_id": "director",
                    "state": "working",
                    "current_task_id": "task-005"
                }
            ]
        });

        assert!(agents_status["agents"][0]["agent_id"].as_str().is_some());
    }
}
