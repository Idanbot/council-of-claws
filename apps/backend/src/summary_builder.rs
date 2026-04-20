use crate::health;
use crate::models::*;
use crate::postgres_reader::PostgresReader;
use crate::redis_reader::RedisReader;

#[derive(Clone)]
pub struct SummaryBuilder {
    redis_reader: RedisReader,
    postgres_reader: PostgresReader,
}

impl SummaryBuilder {
    pub fn new(redis_reader: RedisReader, postgres_reader: PostgresReader) -> Self {
        SummaryBuilder {
            redis_reader,
            postgres_reader,
        }
    }

    pub async fn build_overview(&self) -> Result<Overview, AppError> {
        // Fetch from Redis
        let agents = self
            .redis_reader
            .get_agents_status()
            .await
            .unwrap_or_default();
        let configured_agents = self
            .redis_reader
            .get_configured_agents()
            .await
            .unwrap_or_default();
        let queue_summary = self
            .redis_reader
            .get_queue_summary()
            .await
            .unwrap_or(QueueSummary {
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
        let recent_events = self
            .redis_reader
            .get_recent_events()
            .await
            .unwrap_or_default();
        let redis_health = health::check_redis(&self.redis_reader.connection_manager()).await;
        let postgres_health = health::check_postgres(&self.postgres_reader.pool()).await;
        let system_health = SystemHealth {
            timestamp: chrono::Utc::now().timestamp(),
            host: crate::models::HostMetrics {
                cpu_percent: 0.0,
                memory_percent: 0.0,
                disk_percent: 0.0,
            },
            redis: crate::models::ServiceMetrics {
                status: health::health_status_label(&redis_health.status).to_string(),
                message: redis_health.message,
            },
            postgres: crate::models::ServiceMetrics {
                status: health::health_status_label(&postgres_health.status).to_string(),
                message: postgres_health.message,
            },
            backend: crate::models::ServiceMetrics {
                status: "ok".to_string(),
                message: Some("Backend router active".to_string()),
            },
            frontend: crate::models::ServiceMetrics {
                status: "unknown".to_string(),
                message: Some(
                    "Frontend is checked from the dashboard container, not the backend".to_string(),
                ),
            },
            containers: crate::models::ContainerMetrics {
                running: 0,
                stopped: 0,
                unhealthy: 0,
            },
        };

        // Fetch from PostgreSQL
        let active_agents = agents
            .into_iter()
            .filter(|a| a.state != AgentState::Idle)
            .collect();

        let council_summaries = self.postgres_reader.get_council_runs(5).await?;
        let failed_tasks = self.postgres_reader.get_failed_tasks().await?;
        let blocked_tasks = self.postgres_reader.get_blocked_tasks().await?;

        Ok(Overview {
            system_health,
            active_agents,
            configured_agents,
            queue_summary,
            recent_events,
            council_summaries,
            failed_tasks,
            blocked_tasks,
        })
    }

    pub async fn build_usage_summary(&self) -> Result<UsageSummary, AppError> {
        let usages = self.postgres_reader.get_model_usage(100).await?;

        let total_tokens: i32 = usages.iter().map(|u| u.total_tokens).sum();
        let total_cost_usd: f64 = usages.iter().map(|u| u.estimated_cost_usd).sum();

        // Aggregate by agent
        let mut by_agent_map: std::collections::HashMap<String, (i32, f64)> =
            std::collections::HashMap::new();
        for usage in &usages {
            let entry = by_agent_map
                .entry(usage.agent_id.clone())
                .or_insert((0, 0.0));
            entry.0 += usage.total_tokens;
            entry.1 += usage.estimated_cost_usd;
        }
        let by_agent: Vec<UsageByAgent> = by_agent_map
            .into_iter()
            .map(|(agent_id, (tokens, cost))| UsageByAgent {
                agent_id,
                tokens,
                cost_usd: cost,
            })
            .collect();

        // Aggregate by model
        let mut by_model_map: std::collections::HashMap<String, (i32, f64)> =
            std::collections::HashMap::new();
        for usage in &usages {
            let entry = by_model_map
                .entry(usage.model_name.clone())
                .or_insert((0, 0.0));
            entry.0 += usage.total_tokens;
            entry.1 += usage.estimated_cost_usd;
        }
        let by_model: Vec<UsageByModel> = by_model_map
            .into_iter()
            .map(|(model_name, (tokens, cost))| UsageByModel {
                model_name,
                tokens,
                cost_usd: cost,
            })
            .collect();

        // Simple day aggregation (using created_at dates)
        let mut by_day_map: std::collections::HashMap<String, (i32, f64)> =
            std::collections::HashMap::new();
        for usage in &usages {
            let day = usage.created_at.format("%Y-%m-%d").to_string();
            let entry = by_day_map.entry(day).or_insert((0, 0.0));
            entry.0 += usage.total_tokens;
            entry.1 += usage.estimated_cost_usd;
        }
        let by_day: Vec<UsageByDay> = by_day_map
            .into_iter()
            .map(|(day, (tokens, cost))| UsageByDay {
                day,
                tokens,
                cost_usd: cost,
            })
            .collect();

        Ok(UsageSummary {
            total_tokens,
            total_cost_usd,
            by_agent,
            by_model,
            by_day,
        })
    }
}
