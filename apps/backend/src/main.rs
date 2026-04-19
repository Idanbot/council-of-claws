mod config;
mod models;
mod health;
mod redis_reader;
mod postgres_reader;
mod summary_builder;
mod routes;
mod audit;
mod obsidian_writer;
mod websocket_hub;

use config::Config;
use models::{ConfiguredAgent, DashboardEvent, HostMetrics, QueueSummary, ServiceMetrics, SystemHealth, ContainerMetrics};
use metrics_exporter_prometheus::PrometheusBuilder;
use postgres_reader::PostgresReader;
use redis_reader::RedisReader;
use summary_builder::SummaryBuilder;
use audit::AuditService;
use obsidian_writer::ObsidianWriter;
use websocket_hub::WsHub;
use routes::{configured_agents, create_routes, AppState};
use redis::AsyncCommands;
use std::fs;
use std::net::SocketAddr;
use sqlx::postgres::PgPoolOptions;
use tokio::time::{self, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    fs::create_dir_all("logs").expect("failed to create backend log directory");
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.clone()));
    let file_appender = tracing_appender::rolling::daily("logs", "backend.log");
    let (non_blocking, _file_guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .with(tracing_subscriber::fmt::layer().json().with_writer(non_blocking))
        .init();

    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install prometheus recorder");

    tracing::info!("starting council-backend");
    tracing::info!(
        "config: app_port={}, redis_url={}, timezone={}",
        config.app_port,
        config.redis_url,
        config.timezone
    );

    // Connect to Redis
    let redis_client = match redis::Client::open(config.redis_url.clone()) {
        Ok(client) => {
            tracing::info!("Successfully created Redis client");
            client
        }
        Err(e) => {
            tracing::error!("Failed to create Redis client: {}", e);
            panic!("Cannot start without Redis connection");
        }
    };

    let redis_manager = match redis_client.get_connection_manager().await {
        Ok(manager) => {
            tracing::info!("Successfully connected to Redis");
            manager
        }
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            panic!("Cannot start without Redis connection");
        }
    };

    // Initialize PostgreSQL
    let pg_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("PostgreSQL database connected and migrations applied");

    // Create domain readers
    let redis_reader = RedisReader::new(redis_manager.clone());
    let postgres_reader = PostgresReader::new(pg_pool.clone());
    let summary_builder = SummaryBuilder::new(redis_reader.clone(), postgres_reader.clone());
    let ws_hub = WsHub::new();
    let audit_service = AuditService::new(pg_pool.clone(), redis_manager.clone(), ws_hub.clone());
    let obsidian_writer = ObsidianWriter::new();

    // Create app state
    let state = AppState {
        redis_reader,
        postgres_reader,
        summary_builder,
        audit_service,
        obsidian_writer,
        ws_hub,
        prometheus_handle,
    };

    state
        .audit_service
        .log(
            Some("system-bootstrap"),
            Some("system"),
            models::AuditOperation::AgentStatusSet,
            Some("stack"),
            Some("bootstrap"),
            true,
            Some("success"),
            Some("Stack boot completed; dashboard runtime caches are being initialized"),
            Some(serde_json::json!({
                "event_type": "stack_boot",
                "timezone": config.timezone,
            })),
        )
        .await;

    if let Err(err) = refresh_dashboard_cache(&state).await {
        tracing::warn!("failed to warm dashboard cache on boot: {}", err);
    }

    let cache_state = state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        interval.tick().await;

        loop {
            interval.tick().await;
            if let Err(err) = refresh_dashboard_cache(&cache_state).await {
                tracing::warn!("dashboard cache refresh failed: {}", err);
            }
        }
    });

    // Create router
    let router = create_routes(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.app_port));
    tracing::info!("backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind backend listener");

    axum::serve(listener, router)
        .await
        .expect("backend server error");
}

async fn refresh_dashboard_cache(state: &AppState) -> Result<(), String> {
    let mut conn = state.audit_service.redis_connection();

    let redis_health = health::check_redis(&state.redis_reader.connection_manager()).await;
    let postgres_health = health::check_postgres(&state.postgres_reader.pool()).await;
    let system_health = SystemHealth {
        timestamp: chrono::Utc::now().timestamp(),
        host: HostMetrics {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            disk_percent: 0.0,
        },
        redis: ServiceMetrics {
            status: health::health_status_label(&redis_health.status).to_string(),
            message: redis_health.message,
        },
        postgres: ServiceMetrics {
            status: health::health_status_label(&postgres_health.status).to_string(),
            message: postgres_health.message,
        },
        backend: ServiceMetrics {
            status: "ok".to_string(),
            message: Some("Backend router active".to_string()),
        },
        frontend: ServiceMetrics {
            status: "unknown".to_string(),
            message: Some("Frontend is checked from the dashboard container, not the backend".to_string()),
        },
        containers: ContainerMetrics {
            running: 0,
            stopped: 0,
            unhealthy: 0,
        },
    };
    let _: redis::RedisResult<()> = conn
        .set("dash:system:health", serde_json::to_string(&system_health).map_err(|e| e.to_string())?)
        .await;

    let configured_agents_value: Vec<ConfiguredAgent> = configured_agents();
    let _: redis::RedisResult<()> = conn
        .set(
            "dash:agents:configured",
            serde_json::json!({ "agents": configured_agents_value }).to_string(),
        )
        .await;

    let existing_queue: Option<String> = conn.get("dash:queue:summary").await.map_err(|e| e.to_string())?;
    if existing_queue.is_none() {
        let queue_summary = QueueSummary {
            pending_critical: 0,
            pending_high: 0,
            pending_normal: 0,
            pending_low: 0,
            in_progress: 0,
            reviewing: 0,
            blocked: 0,
            completed: 0,
            failed: 0,
        };
        let _: redis::RedisResult<()> = conn
            .set("dash:queue:summary", serde_json::to_string(&queue_summary).map_err(|e| e.to_string())?)
            .await;
    }

    let existing_events: Option<String> = conn.get("dash:events:recent").await.map_err(|e| e.to_string())?;
    if existing_events.is_none() {
        let boot_event = DashboardEvent {
            level: crate::models::EventLevel::Info,
            summary: "Dashboard cache primed; waiting for live events".to_string(),
            stream_connection: "bootstrap".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        let _: redis::RedisResult<()> = conn
            .set(
                "dash:events:recent",
                serde_json::json!({ "events": [boot_event] }).to_string(),
            )
            .await;
    }

    Ok(())
}
