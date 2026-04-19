use crate::models::{HealthResponse, ServiceHealth, HealthStatus};
use chrono::Utc;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

pub async fn check_redis(client: &ConnectionManager) -> ServiceHealth {
    let mut conn = client.clone();
    let ping_result: Result<String, redis::RedisError> = redis::cmd("PING").query_async(&mut conn).await;

    match ping_result {
        Ok(_) => ServiceHealth {
            name: "redis".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Redis connection OK".to_string()),
        },
        Err(e) => ServiceHealth {
            name: "redis".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("Redis error: {}", e)),
        },
    }
}

pub async fn check_postgres(pool: &PgPool) -> ServiceHealth {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => ServiceHealth {
            name: "postgres".to_string(),
            status: HealthStatus::Healthy,
            message: Some("PostgreSQL connection OK".to_string()),
        },
        Err(e) => ServiceHealth {
            name: "postgres".to_string(),
            status: HealthStatus::Unhealthy,
            message: Some(format!("PostgreSQL error: {}", e)),
        },
    }
}

pub fn get_backend_health() -> HealthResponse {
    HealthResponse {
        service: "backend".to_string(),
        status: "ok".to_string(),
        timestamp: Utc::now(),
    }
}

pub fn health_status_label(status: &HealthStatus) -> &'static str {
    match status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_health() {
        let health = get_backend_health();
        assert_eq!(health.service, "backend");
        assert_eq!(health.status, "ok");
    }
}
