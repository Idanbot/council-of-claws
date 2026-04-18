mod config;
mod models;
mod health;
mod redis_reader;
mod postgres_reader;
mod summary_builder;
mod redis_mock;
mod routes;
mod audit;
mod obsidian_writer;
mod websocket_hub;

use config::Config;
use metrics_exporter_prometheus::PrometheusBuilder;
use postgres_reader::PostgresReader;
use redis_reader::RedisReader;
use summary_builder::SummaryBuilder;
use audit::AuditService;
use obsidian_writer::ObsidianWriter;
use websocket_hub::WsHub;
use routes::{create_routes, AppState};
use std::fs;
use std::net::SocketAddr;
use sqlx::postgres::PgPoolOptions;
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
    tracing::info!("config: app_port={}, redis_url={}, mock_mode={}, timezone={}",
        config.app_port, config.redis_url, config.mock_mode, config.timezone);

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

    // Initialize Redis with mock data if in mock mode
    // if config.mock_mode {
    //     tracing::info!("Initializing Redis with mock data");
    //     if let Err(e) = redis_mock::init_mock_data(&redis_manager).await {
    //         tracing::warn!("Failed to initialize mock data: {}", e);
    //     }
    // }

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
