use crate::models::*;
use crate::postgres_reader::PostgresReader;
use crate::redis_reader::RedisReader;
use crate::summary_builder::SummaryBuilder;
use crate::audit::AuditService;
use crate::obsidian_writer::ObsidianWriter;
use crate::websocket_hub::WsHub;
use crate::health;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use sqlx::Row;
use redis::AsyncCommands;
use chrono::Utc;
use axum::{
    extract::{Path, State, ws::WebSocketUpgrade},
    response::Json,
    routing::{get, post},
    http::{header::AUTHORIZATION, HeaderMap, HeaderName, Method, StatusCode},
    Router,
};
use futures::SinkExt;
use futures::stream::StreamExt;
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Duration;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub redis_reader: RedisReader,
    pub postgres_reader: PostgresReader,
    pub summary_builder: SummaryBuilder,
    pub audit_service: AuditService,
    pub obsidian_writer: ObsidianWriter,
    pub ws_hub: WsHub,
    pub prometheus_handle: PrometheusHandle,
}

const ALLOWED_AGENT_IDS: &[&str] = &[
    "contractor",
    "director",
    "architect",
    "senior-engineer",
    "senior_engineer",
    "junior-engineer",
    "junior_engineer",
    "intern",
];

const CONFIGURED_AGENTS: &[(&str, &str, &str, &[&str], TaskPriority)] = &[
    (
        "contractor",
        "front-door",
        "github-copilot/gpt-4o",
        &[
            "groq/llama-3.3-70b-versatile",
            "google/gemini-3-flash-preview",
            "openai/gpt-5.4-mini",
        ],
        TaskPriority::Normal,
    ),
    (
        "director",
        "planner",
        "google/gemini-3.1-pro-preview",
        &[
            "github-copilot/gpt-4o",
            "openai/gpt-5.4",
            "groq/llama-3.3-70b-versatile",
        ],
        TaskPriority::High,
    ),
    (
        "architect",
        "design-review",
        "google/gemini-3.1-pro-preview",
        &["github-copilot/gpt-4o", "openai/gpt-5.4"],
        TaskPriority::High,
    ),
    (
        "senior-engineer",
        "implementation",
        "github-copilot/gpt-4o",
        &[
            "openai/gpt-5.4",
            "google/gemini-3.1-pro-preview",
            "groq/llama-3.3-70b-versatile",
        ],
        TaskPriority::High,
    ),
    (
        "junior-engineer",
        "qa-review",
        "groq/llama-3.3-70b-versatile",
        &[
            "google/gemini-3-flash-preview",
            "github-copilot/gpt-4o",
            "openai/gpt-5.4-mini",
        ],
        TaskPriority::Normal,
    ),
    (
        "intern",
        "notifications",
        "groq/llama-3.1-8b-instant",
        &["google/gemini-3-flash-preview"],
        TaskPriority::Low,
    ),
];

pub fn create_routes(state: AppState) -> Router {
    let state = Arc::new(state);
    let request_id_header = HeaderName::from_static("x-request-id");
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        // Health endpoints
        .route("/api/health", get(health_handler))
        .route("/api/health/services", get(health_services_handler))

        // Overview endpoints
        .route("/api/overview", get(overview_handler))
        .route("/api/overview/system", get(overview_system_handler))

        // Agent endpoints
        .route("/api/agents", get(agents_list_handler))
        .route("/api/agents/configured", get(agents_configured_handler))
        .route("/api/agents/status", get(agents_status_handler))
        .route("/api/agents/{agent_id}", get(agents_detail_handler))
        .route("/api/agents/heartbeat", post(agent_heartbeat_handler))

        // Task endpoints
        .route("/api/tasks", get(tasks_list_handler))
        .route("/api/tasks/{task_id}", get(tasks_detail_handler))

        // Internal write endpoints (agent tools)
        .route("/internal/tasks/create", post(task_create_handler))
        .route("/internal/missions", post(mission_create_handler))
        .route("/internal/missions/{mission_id}/close", post(mission_close_handler))

        // Council endpoints
        .route("/api/council", get(council_list_handler))
        .route("/api/council/{council_id}", get(council_detail_handler))
        .route("/api/council/propose", post(council_propose_handler))
        .route("/api/council/{council_id}/vote", post(council_vote_handler))
        .route("/api/council/{council_id}/finalize", post(council_finalize_handler))

        // History endpoints
        .route("/api/history/missions", get(mission_history_handler))
        .route("/api/audit", get(audit_list_handler))

        // Admin endpoints
        .route("/api/admin/rotate-secret", post(secret_rotate_handler))

        // Usage endpoints
        .route("/api/usage", get(usage_summary_handler))
        .route("/api/usage/agents", get(usage_agents_handler))
        .route("/api/usage/models", get(usage_models_handler))

        // Events endpoint
        .route("/api/events", get(events_handler))
        .route("/api/diagnostics/report", get(diagnostics_report_handler))
        .route("/api/models/status", get(models_status_handler))
        .route("/api/admin/runtime-status", get(admin_runtime_status_handler))

        // Metrics endpoint
        .route("/api/metrics", get(metrics_handler))

        // WebSocket stream
        .route("/ws", get(websocket_handler))

        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .layer(cors)
        .with_state(state)
}

async fn health_handler(State(_state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(health::get_backend_health())
}

async fn health_services_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let redis_conn = state.redis_reader.connection_manager();
    let postgres_pool = state.postgres_reader.pool();

    let redis = health::check_redis(&redis_conn).await;
    let postgres = health::check_postgres(&postgres_pool).await;

    let services = vec![
        serde_json::json!({"name": "backend", "status": "healthy"}),
        serde_json::json!({"name": redis.name, "status": redis.status, "message": redis.message}),
        serde_json::json!({"name": postgres.name, "status": postgres.status, "message": postgres.message}),
    ];

    Json(json!({"services": services, "timestamp": chrono::Utc::now()}))
}

async fn council_propose_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CouncilProposeRequest>,
) -> Result<Json<CouncilProposeResponse>, (StatusCode, Json<serde_json::Value>)> {
    let agent_id = enforce_agent_scope(&state, &headers, Some(|s| s.allow_council_propose)).await?;

    let council_id = format!("council-{}", uuid::Uuid::new_v4());

    sqlx::query(
        "INSERT INTO council_runs (id, title, status, phase, director_agent, created_at, updated_at)
         VALUES ($1, $2, 'active', 'debating', $3, NOW(), NOW())"
    )
    .bind(&council_id)
    .bind(&payload.title)
    .bind(&agent_id)
    .execute(&state.postgres_reader.pool())
    .await
    .map_err(|e| internal_json_error(e.to_string()))?;

    // Record initial round
    sqlx::query(
        "INSERT INTO council_rounds (id, council_id, round_number, round_type, summary, created_at)
         VALUES ($1, $2, 1, 'opening', $3, NOW())"
    )
    .bind(format!("round-{}", uuid::Uuid::new_v4()))
    .bind(&council_id)
    .bind(&payload.initial_summary)
    .execute(&state.postgres_reader.pool())
    .await
    .map_err(|e| internal_json_error(e.to_string()))?;

    Ok(Json(CouncilProposeResponse {
        council_id,
        status: "active".to_string(),
        phase: CouncilPhase::Debating,
    }))
}

async fn council_vote_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(council_id): Path<String>,
    Json(payload): Json<CouncilVoteRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let agent_id = enforce_agent_scope(&state, &headers, None).await?; // Any active agent can vote

    sqlx::query(
        "INSERT INTO council_votes (id, council_id, agent_id, vote, reason, created_at)
         VALUES ($1, $2, $3, $4, $5, NOW())
         ON CONFLICT (id) DO UPDATE SET vote = EXCLUDED.vote, reason = EXCLUDED.reason"
    )
    .bind(format!("vote-{}-{}", council_id, agent_id))
    .bind(&council_id)
    .bind(&agent_id)
    .bind(&payload.vote)
    .bind(&payload.reason)
    .execute(&state.postgres_reader.pool())
    .await
    .map_err(|e| internal_json_error(e.to_string()))?;

    Ok(StatusCode::CREATED)
}

async fn council_finalize_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(council_id): Path<String>,
    Json(payload): Json<CouncilFinalizeRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let _ = enforce_agent_scope(&state, &headers, Some(|s| s.allow_council_finalize)).await?;

    sqlx::query(
        "UPDATE council_runs SET status = 'completed', phase = 'concluded', ruling_summary = $1, confidence = $2, updated_at = NOW() WHERE id = $3"
    )
    .bind(&payload.ruling_summary)
    .bind(payload.confidence)
    .bind(&council_id)
    .execute(&state.postgres_reader.pool())
    .await
    .map_err(|e| internal_json_error(e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn secret_rotate_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SecretRotateRequest>,
) -> Result<Json<SecretRotateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let agent_id = enforce_agent_scope(&state, &headers, None).await?; // Agent rotates their own secret

    use argon2::{password_hash::SaltString, PasswordHasher};
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.new_secret.as_bytes(), &salt)
        .map_err(|e| internal_json_error(e.to_string()))?
        .to_string();

    sqlx::query("UPDATE agents SET secret_hash = $1, rotated_at = NOW() WHERE id = $2")
        .bind(password_hash)
        .bind(&agent_id)
        .execute(&state.postgres_reader.pool())
        .await
        .map_err(|e| internal_json_error(e.to_string()))?;

    Ok(Json(SecretRotateResponse {
        rotated_at: Utc::now(),
    }))
}

async fn overview_handler(State(state): State<Arc<AppState>>) -> Result<Json<Overview>, (StatusCode, String)> {
    state.summary_builder
        .build_overview()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn overview_system_handler(State(state): State<Arc<AppState>>) -> Result<Json<SystemHealth>, (StatusCode, String)> {
    state.summary_builder
        .build_overview()
        .await
        .map(|overview| Json(overview.system_health))
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, e))
}

async fn agents_list_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let agents = state.redis_reader
        .get_agents_status()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        agents
            .into_iter()
            .filter(|agent| agent.last_heartbeat_ts > 0)
            .collect(),
    ))
}

async fn agents_configured_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ConfiguredAgent>> {
    let cached = state.redis_reader.get_configured_agents().await.unwrap_or_default();
    if cached.is_empty() {
        Json(configured_agents())
    } else {
        Json(cached)
    }
}

async fn agents_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AgentsStatusReport>, (StatusCode, String)> {
    let configured = {
        let cached = state.redis_reader.get_configured_agents().await.unwrap_or_default();
        if cached.is_empty() {
            configured_agents()
        } else {
            cached
        }
    };
    let live_agents = state.redis_reader.get_agents_status().await.unwrap_or_default();
    let report = build_agents_status_report(&configured, &live_agents, heartbeat_ttl_seconds());
    Ok(Json(report))
}

async fn agents_detail_handler(
    State(state): State<Arc<AppState>>,
    Path(_agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agent = state.redis_reader
        .get_agents_status()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .into_iter()
        .find(|agent| agent.agent_id == _agent_id && agent.last_heartbeat_ts > 0);

    let runs = state.postgres_reader.get_agents_runs(10).await.ok();
    let usage = state.postgres_reader.get_model_usage(10).await.ok();

    Ok(Json(json!({
        "agent": agent,
        "runs": runs,
        "usage": usage
    })))
}

async fn tasks_list_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    state.postgres_reader
        .get_tasks(50, 0)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn tasks_detail_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let task = state.postgres_reader
        .get_task(&task_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((StatusCode::NOT_FOUND, "Task not found".to_string()))?;

    let runs = state.postgres_reader.get_agents_runs(10).await.ok();

    Ok(Json(json!({"task": task, "runs": runs})))
}

async fn task_create_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<TaskCreateRequest>,
) -> Result<(StatusCode, Json<TaskCreateResponse>), (StatusCode, Json<serde_json::Value>)> {
    let request_id = headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let agent_id = enforce_agent_scope(&state, &headers, Some(|s| s.allow_task_create)).await?;

    if !is_known_agent(&payload.target_agent_id) {
        state.audit_service.log(
            request_id.as_deref(),
            Some(&agent_id),
            AuditOperation::TaskCreate,
            Some("task"),
            None,
            false,
            Some("error"),
            Some(&format!("unknown target_agent_id '{}'", payload.target_agent_id)),
            None,
        ).await;

        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": "INVALID_TARGET_AGENT",
                "message": format!("unknown target_agent_id '{}'", payload.target_agent_id),
            })),
        ));
    }

    let result = state
        .postgres_reader
        .create_task(
            &payload.title,
            &payload.description,
            payload.priority.clone(),
            &payload.target_agent_id,
            "director",
            payload.mission_id.as_deref(),
        )
        .await;
match result {
    Ok((task_id, created_at)) => {
        // Write Obsidian summary for the new task
        let task_for_summary = Task {
            id: task_id.clone(),
            title: payload.title.clone(),
            priority: payload.priority.clone(),
            status: TaskStatus::Pending,
            owner_agent: payload.target_agent_id.clone(),
            created_at,
            updated_at: created_at,
            blocked_reason: Some(payload.description.clone()),
        };

        let _ = state.obsidian_writer.write_task_summary(
            &task_for_summary,
            payload.mission_id.as_deref(),
            None
        ).await;

        state.audit_service.log(
            request_id.as_deref(),
            Some(&agent_id),
            AuditOperation::TaskCreate,
            Some("task"),
            Some(&task_id),
            true,
            Some("success"),
            None,
            Some(json!({
                "title": payload.title,
                "target_agent": payload.target_agent_id,
                "priority": payload.priority,
                "mission_id": payload.mission_id
            })),
        ).await;

            Ok((
                StatusCode::CREATED,
                Json(TaskCreateResponse {
                    task_id,
                    status: "created".to_string(),
                    assigned_to: payload.target_agent_id,
                    created_at,
                }),
            ))
        }
        Err(e) => {
            state.audit_service.log(
                request_id.as_deref(),
                Some(&agent_id),
                AuditOperation::TaskCreate,
                Some("task"),
                None,
                false,
                Some("error"),
                Some(&e),
                None,
            ).await;

            Err(internal_json_error(e))
        }
    }
}

async fn mission_create_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<MissionCreateRequest>,
) -> Result<(StatusCode, Json<MissionCreateResponse>), (StatusCode, Json<serde_json::Value>)> {
    let request_id = headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let agent_id = enforce_agent_scope(&state, &headers, Some(|s| s.allow_task_create)).await?;

    let result = state
        .postgres_reader
        .create_mission(&payload.title, &payload.description, "director")
        .await;

    match result {
        Ok((mission_id, root_task_id, created_at)) => {
            state.audit_service.log(
                request_id.as_deref(),
                Some(&agent_id),
                AuditOperation::MissionCreate,
                Some("mission"),
                Some(&mission_id),
                true,
                Some("success"),
                None,
                Some(json!({
                    "title": payload.title,
                    "root_task_id": root_task_id
                })),
            ).await;

            Ok((
                StatusCode::CREATED,
                Json(MissionCreateResponse {
                    mission_id,
                    root_task_id,
                    status: MissionStatus::Active,
                    created_at,
                    created_by_agent: "director".to_string(),
                    title: payload.title,
                }),
            ))
        }
        Err(e) => {
            state.audit_service.log(
                request_id.as_deref(),
                Some(&agent_id),
                AuditOperation::MissionCreate,
                Some("mission"),
                None,
                false,
                Some("error"),
                Some(&e),
                None,
            ).await;

            Err(internal_json_error(e))
        }
    }
}

async fn mission_close_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(mission_id): Path<String>,
    Json(payload): Json<MissionCloseRequest>,
) -> Result<Json<MissionCloseResponse>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let agent_id = enforce_agent_scope(&state, &headers, Some(|s| s.allow_mission_close)).await?;

    let mission = state
        .postgres_reader
        .get_mission(&mission_id)
        .await
        .map_err(internal_json_error)?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({"code": "MISSION_NOT_FOUND", "message": "mission not found"})),
        ))?;

    if mission.status != MissionStatus::Active {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": "MISSION_NOT_ACTIVE",
                "message": "mission must be active before close",
            })),
        ));
    }

    let subtasks = state
        .postgres_reader
        .get_mission_subtasks(&mission_id, &mission.root_task_id)
        .await
        .map_err(internal_json_error)?;

    if subtasks.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": "MISSION_EMPTY",
                "message": "cannot close mission with zero subtasks",
            })),
        ));
    }

    let pending: Vec<String> = subtasks
        .iter()
        .filter(|task| {
            !matches!(task.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
        })
        .map(|task| task.id.clone())
        .collect();

    if !pending.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": "MISSION_INCOMPLETE",
                "message": format!("cannot close mission: {} subtasks still in progress", pending.len()),
                "total_subtasks": subtasks.len(),
                "pending_count": pending.len(),
                "pending_task_ids": pending,
            })),
        ));
    }

    let closed_at = state
        .postgres_reader
        .close_mission(&mission_id, "director")
        .await
        .map_err(internal_json_error)?;

    let failed_tasks: Vec<String> = subtasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Failed))
        .map(|task| task.id.clone())
        .collect();

    let warning_tasks: Vec<String> = subtasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Cancelled))
        .map(|task| task.id.clone())
        .collect();

    let obsidian_doc_url = state.obsidian_writer.write_mission_summary(
        &mission,
        &subtasks,
        closed_at,
        payload.notes.as_deref(),
        None
    )
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": "OBSIDIAN_WRITE_FAILED",
                "message": err,
            })),
        )
    })?;

    state.audit_service.log(
        request_id.as_deref(),
        Some(&agent_id),
        AuditOperation::MissionClose,
        Some("mission"),
        Some(&mission_id),
        true,
        Some("success"),
        None,
        Some(json!({
            "obsidian_url": obsidian_doc_url,
            "subtask_count": subtasks.len()
        })),
    ).await;

    let completed_count = subtasks
        .iter()
        .filter(|task| matches!(task.status, TaskStatus::Completed))
        .count() as i64;

    Ok(Json(MissionCloseResponse {
        mission_id,
        status: MissionStatus::Closed,
        closed_at,
        closed_by_agent: "director".to_string(),
        subtask_count: subtasks.len() as i64,
        subtask_complete_count: completed_count,
        obsidian_doc_url,
        summary: MissionCloseSummary {
            all_valid_end_state: true,
            failed_tasks,
            warning_tasks,
        },
    }))
}

async fn council_list_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<CouncilRun>>, (StatusCode, String)> {
    state.postgres_reader
        .get_council_runs(20)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn council_detail_handler(
    State(state): State<Arc<AppState>>,
    Path(_council_id): Path<String>,
) -> Result<Json<CouncilRun>, (StatusCode, String)> {
    let councils = state.postgres_reader
        .get_council_runs(1)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    councils.into_iter().next()
        .ok_or((StatusCode::NOT_FOUND, "Council not found".to_string()))
        .map(Json)
}

async fn usage_summary_handler(State(state): State<Arc<AppState>>) -> Result<Json<UsageSummary>, (StatusCode, String)> {
    state.summary_builder
        .build_usage_summary()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn usage_agents_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ModelUsage>>, (StatusCode, String)> {
    state.postgres_reader
        .get_model_usage(100)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn usage_models_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<ModelUsage>>, (StatusCode, String)> {
    state.postgres_reader
        .get_model_usage(100)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn events_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DashboardEvent>>, (StatusCode, String)> {
    state.redis_reader
        .get_recent_events()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn diagnostics_report_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DiagnosticsReport>, (StatusCode, String)> {
    let mut checks = Vec::new();

    let started = std::time::Instant::now();
    let backend_health = health::get_backend_health();
    checks.push(DiagnosticsCheck {
        name: "backend".to_string(),
        status: backend_health.status.clone(),
        info: "Backend router active".to_string(),
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let started = std::time::Instant::now();
    let redis_health = health::check_redis(&state.redis_reader.connection_manager()).await;
    checks.push(DiagnosticsCheck {
        name: redis_health.name,
        status: health::health_status_label(&redis_health.status).to_string(),
        info: redis_health.message.unwrap_or_else(|| "Redis check completed".to_string()),
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let started = std::time::Instant::now();
    let postgres_health = health::check_postgres(&state.postgres_reader.pool()).await;
    checks.push(DiagnosticsCheck {
        name: postgres_health.name,
        status: health::health_status_label(&postgres_health.status).to_string(),
        info: postgres_health.message.unwrap_or_else(|| "PostgreSQL check completed".to_string()),
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let started = std::time::Instant::now();
    let audit_count = state.postgres_reader.get_audit_events(5).await.map(|items| items.len()).unwrap_or(0);
    checks.push(DiagnosticsCheck {
        name: "audit-log".to_string(),
        status: if audit_count > 0 { "healthy" } else { "degraded" }.to_string(),
        info: if audit_count > 0 {
            format!("{audit_count} recent audit events available")
        } else {
            "No durable audit events recorded yet".to_string()
        },
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let started = std::time::Instant::now();
    let live_agents = state.redis_reader.get_agents_status().await.unwrap_or_default();
    let configured_agents_count = {
        let cached = state.redis_reader.get_configured_agents().await.unwrap_or_default();
        if cached.is_empty() {
            configured_agents().len()
        } else {
            cached.len()
        }
    };
    let reporting_agents = live_agents.len();
    checks.push(DiagnosticsCheck {
        name: "agent-telemetry".to_string(),
        status: if reporting_agents > 0 { "healthy" } else { "degraded" }.to_string(),
        info: if reporting_agents > 0 {
            format!("{reporting_agents} live agents reporting heartbeats")
        } else if configured_agents_count > 0 {
            format!("{configured_agents_count} configured agents available; waiting for live heartbeats")
        } else {
            "No live agent heartbeats in Redis; dashboard will fall back to configured roster".to_string()
        },
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let started = std::time::Instant::now();
    let stream_events = state.redis_reader.get_recent_events().await.map(|items| items.len()).unwrap_or(0);
    checks.push(DiagnosticsCheck {
        name: "stream-cache".to_string(),
        status: if stream_events > 0 { "healthy" } else { "degraded" }.to_string(),
        info: if stream_events > 0 {
            format!("{stream_events} recent stream events cached")
        } else {
            "No recent stream events cached in Redis".to_string()
        },
        duration_ms: started.elapsed().as_millis() as i64,
    });

    let overall_status = if checks.iter().any(|c| c.status == "unhealthy") {
        "degraded"
    } else if checks.iter().any(|c| c.status == "degraded") {
        "partial"
    } else {
        "healthy"
    };

    Ok(Json(DiagnosticsReport {
        generated_at: Utc::now(),
        overall_status: overall_status.to_string(),
        checks,
    }))
}

async fn models_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ModelProviderStatus>, (StatusCode, String)> {
    let configured = {
        let cached = state.redis_reader.get_configured_agents().await.unwrap_or_default();
        if cached.is_empty() {
            configured_agents()
        } else {
            cached
        }
    };

    Ok(Json(ModelProviderStatus {
        generated_at: Utc::now(),
        providers: provider_statuses_from_env(),
        configured_agents: configured,
    }))
}

async fn admin_runtime_status_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AdminRuntimeStatus>, (StatusCode, String)> {
    let gateway_health = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::net::TcpStream::connect((gateway_host(), gateway_port())),
    )
    .await
    {
        Ok(Ok(_)) => ServiceMetrics {
            status: "healthy".to_string(),
            message: Some("Gateway TCP endpoint reachable from backend container".to_string()),
        },
        Ok(Err(err)) => ServiceMetrics {
            status: "degraded".to_string(),
            message: Some(format!("Gateway connect failed: {}", err)),
        },
        Err(_) => ServiceMetrics {
            status: "degraded".to_string(),
            message: Some("Gateway connect timed out".to_string()),
        },
    };

    let backend_log_tail = tail_backend_log(60);
    let notes = vec![
        format!("heartbeat_ttl_seconds={}", heartbeat_ttl_seconds()),
        format!("gateway_target={}:{}", gateway_host(), gateway_port()),
        "runtime logs only include backend container logs from /app/logs/backend.log".to_string(),
    ];

    Ok(Json(AdminRuntimeStatus {
        generated_at: Utc::now(),
        gateway: gateway_health,
        providers: provider_statuses_from_env(),
        backend_log_tail,
        notes,
    }))
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> String {
    state.prometheus_handle.render()
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(ws: axum::extract::ws::WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = ws.split();
    let mut hub_rx = state.ws_hub.subscribe();

    // Initial welcome message
    let _ = sender.send(axum::extract::ws::Message::Text(
        json!({
            "event_type": "connection",
            "message": "Connected to Council of Claws real-time stream"
        }).to_string().into()
    )).await;

    // Task to forward hub messages to this websocket
    let mut sender_task = tokio::spawn(async move {
        while let Ok(msg) = hub_rx.recv().await {
            if sender.send(axum::extract::ws::Message::Text(msg.to_string().into())).await.is_err() {
                break;
            }
        }
    });

    // Task to handle incoming messages from this websocket (mostly for keepalive/echo)
    let mut receiver_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                // We could handle incoming commands here if needed
                tracing::debug!("Received WS message: {}", text);
            }
        }
    });

    // If either task finishes, clean up the other
    tokio::select! {
        _ = (&mut sender_task) => receiver_task.abort(),
        _ = (&mut receiver_task) => sender_task.abort(),
    }
}

async fn agent_heartbeat_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let agent_id = headers
        .get("x-agent-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if agent_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"code": "MISSING_AGENT_ID", "message": "x-agent-id header required"})),
        ));
    }

    // Update agent's updated_at in DB
    sqlx::query("UPDATE agents SET updated_at = NOW() WHERE id = $1")
        .bind(&agent_id)
        .execute(&state.postgres_reader.pool())
        .await
        .map_err(|e| internal_json_error(e.to_string()))?;

    // Push heartbeat to Redis Stream for real-time monitoring
    let mut conn = state.audit_service.redis_connection();
    let current_task_id = payload.current_task_id.as_deref().unwrap_or("");
    let _: redis::RedisResult<()> = conn.xadd(
        "coc:events:heartbeat",
        "*",
        &[
            ("agent_id", agent_id.as_ref()),
            ("status", payload.status.as_str()),
            ("task_id", current_task_id),
        ]
    ).await;

    let mut agents = state.redis_reader.get_agents_status().await.unwrap_or_default();
    let configured = configured_agents()
        .into_iter()
        .find(|agent| agent.agent_id == agent_id)
        .unwrap_or(ConfiguredAgent {
            agent_id: agent_id.to_string(),
            role: "dynamic".to_string(),
            primary_model: "unknown".to_string(),
            fallbacks: vec![],
            priority: TaskPriority::Normal,
        });

    if let Some(existing) = agents.iter_mut().find(|agent| agent.agent_id == agent_id) {
        existing.state = parse_agent_state(payload.status.as_str());
        existing.current_task_id = payload.current_task_id.clone();
        existing.last_heartbeat_ts = Utc::now().timestamp();
        existing.elapsed_seconds = 0;
    } else {
        agents.push(Agent {
            agent_id: agent_id.to_string(),
            state: parse_agent_state(payload.status.as_str()),
            current_task_id: payload.current_task_id.clone(),
            priority: configured.priority.clone(),
            model: configured.primary_model,
            last_heartbeat_ts: Utc::now().timestamp(),
            elapsed_seconds: 0,
        });
    }

    let cache_payload = json!({ "agents": agents });
    let _: redis::RedisResult<()> = conn.set("dash:agents:status", cache_payload.to_string()).await;

    Ok(StatusCode::OK)
}

async fn mission_history_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Mission>>, (StatusCode, Json<serde_json::Value>)> {
    let missions = state
        .postgres_reader
        .get_all_missions()
        .await
        .map_err(internal_json_error)?;

    Ok(Json(missions))
}

async fn audit_list_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AuditEvent>>, (StatusCode, Json<serde_json::Value>)> {
    let events = state
        .postgres_reader
        .get_audit_events(50)
        .await
        .map_err(internal_json_error)?;

    Ok(Json(events))
}

fn is_known_agent(agent_id: &str) -> bool {
    ALLOWED_AGENT_IDS.contains(&agent_id)
}

pub(crate) fn configured_agents() -> Vec<ConfiguredAgent> {
    CONFIGURED_AGENTS
        .iter()
        .map(|(agent_id, role, primary_model, fallbacks, priority)| ConfiguredAgent {
            agent_id: (*agent_id).to_string(),
            role: (*role).to_string(),
            primary_model: (*primary_model).to_string(),
            fallbacks: fallbacks.iter().map(|item| (*item).to_string()).collect(),
            priority: priority.clone(),
        })
        .collect()
}

fn parse_agent_state(status: &str) -> AgentState {
    match status {
        "working" => AgentState::Working,
        "reviewing" => AgentState::Reviewing,
        "blocked" => AgentState::Blocked,
        "completed" => AgentState::Completed,
        "failed" => AgentState::Failed,
        _ => AgentState::Idle,
    }
}

fn build_agents_status_report(
    configured: &[ConfiguredAgent],
    live_agents: &[Agent],
    ttl_seconds: i64,
) -> AgentsStatusReport {
    let now = Utc::now().timestamp();
    let live_by_id: HashMap<&str, &Agent> = live_agents
        .iter()
        .map(|agent| (agent.agent_id.as_str(), agent))
        .collect();
    let mut stale_count = 0usize;

    let agents = configured
        .iter()
        .map(|item| {
            let live = live_by_id.get(item.agent_id.as_str()).copied().cloned();
            let age = live
                .as_ref()
                .map(|agent| now.saturating_sub(agent.last_heartbeat_ts));
            let status = match age {
                Some(value) if value <= ttl_seconds => "live",
                Some(_) => {
                    stale_count += 1;
                    "stale"
                }
                None => "configured",
            }
            .to_string();

            AgentStatusSnapshot {
                configured: item.clone(),
                live,
                heartbeat_age_seconds: age,
                status,
            }
        })
        .collect::<Vec<_>>();

    AgentsStatusReport {
        generated_at: Utc::now(),
        heartbeat_ttl_seconds: ttl_seconds,
        configured_count: configured.len(),
        live_count: live_agents.len(),
        stale_count,
        agents,
    }
}

fn provider_statuses_from_env() -> Vec<ProviderStatus> {
    provider_specs()
        .into_iter()
        .map(|(provider, keys)| {
            let via = keys
                .iter()
                .find_map(|key| env::var(key).ok().filter(|value| !value.trim().is_empty()).map(|_| (*key).to_string()));

            ProviderStatus {
                provider: provider.to_string(),
                configured: via.is_some(),
                via,
            }
        })
        .collect()
}

fn provider_specs() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("openai", vec!["OPENAI_API_KEY"]),
        ("google", vec!["GEMINI_API_KEY"]),
        ("groq", vec!["GROQ_API_KEY"]),
        ("anthropic", vec!["ANTHROPIC_API_KEY"]),
        ("github-copilot", vec!["COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"]),
    ]
}

fn heartbeat_ttl_seconds() -> i64 {
    env::var("AGENT_HEARTBEAT_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(120)
}

fn gateway_host() -> String {
    env::var("GATEWAY_HOST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gateway".to_string())
}

fn gateway_port() -> u16 {
    env::var("GATEWAY_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(18789)
}

fn tail_backend_log(max_lines: usize) -> Vec<String> {
    let path = env::var("BACKEND_LOG_PATH").unwrap_or_else(|_| "/app/logs/backend.log".to_string());
    let contents = fs::read_to_string(path).unwrap_or_default();
    let mut lines = contents.lines().map(|line| line.to_string()).collect::<Vec<_>>();
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[test]
    fn builds_agent_status_report_diff() {
        let configured = vec![ConfiguredAgent {
            agent_id: "director".to_string(),
            role: "planner".to_string(),
            primary_model: "google/gemini-3.1-pro-preview".to_string(),
            fallbacks: vec![],
            priority: TaskPriority::High,
        }];
        let live = vec![Agent {
            agent_id: "director".to_string(),
            state: AgentState::Working,
            current_task_id: None,
            priority: TaskPriority::High,
            model: "google/gemini-3.1-pro-preview".to_string(),
            last_heartbeat_ts: Utc::now().timestamp(),
            elapsed_seconds: 5,
        }];
        let report = build_agents_status_report(&configured, &live, 120);

        assert_eq!(report.configured_count, 1);
        assert_eq!(report.live_count, 1);
        assert_eq!(report.agents.len(), 1);
        assert_eq!(report.agents[0].status, "live");
    }

    #[test]
    fn marks_stale_agents() {
        let configured = vec![ConfiguredAgent {
            agent_id: "intern".to_string(),
            role: "notifications".to_string(),
            primary_model: "groq/llama-3.1-8b-instant".to_string(),
            fallbacks: vec![],
            priority: TaskPriority::Low,
        }];
        let live = vec![Agent {
            agent_id: "intern".to_string(),
            state: AgentState::Idle,
            current_task_id: None,
            priority: TaskPriority::Low,
            model: "groq/llama-3.1-8b-instant".to_string(),
            last_heartbeat_ts: Utc::now().timestamp() - 9999,
            elapsed_seconds: 0,
        }];
        let report = build_agents_status_report(&configured, &live, 60);
        assert_eq!(report.stale_count, 1);
        assert_eq!(report.agents[0].status, "stale");
    }

    #[test]
    fn provider_specs_include_copilot() {
        let specs = provider_specs();
        let copilot = specs
            .iter()
            .find(|(provider, _)| *provider == "github-copilot")
            .expect("copilot provider should be present");
        assert!(copilot.1.contains(&"COPILOT_GITHUB_TOKEN"));
    }
}

async fn enforce_agent_scope(
    state: &AppState,
    headers: &HeaderMap,
    required_permission: Option<fn(&ScopeProfile) -> bool>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let x_agent_id = headers
        .get("x-agent-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    if x_agent_id.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"code": "MISSING_AGENT_ID", "message": "x-agent-id header required"})),
        ));
    }

    // 1. Fetch identity and scope
    let agent_row = sqlx::query(
        "SELECT id, secret_hash, scope_profile FROM agents WHERE id = $1 AND enabled = true"
    )
    .bind(&x_agent_id)
    .fetch_optional(&state.postgres_reader.pool())
    .await
    .map_err(|e| internal_json_error(e.to_string()))?;

    let row = agent_row.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "code": "AGENT_NOT_FOUND_OR_DISABLED",
            "message": format!("agent '{}' is not registered or is disabled", x_agent_id),
        })),
    ))?;

    // 2. Token verification (STRICT - No fallback once secret_hash exists)
    let provided_token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "code": "MISSING_BEARER_TOKEN",
                "message": "missing Authorization: Bearer <token> header",
            })),
        ))?;

    let stored_hash: Option<String> = row.get("secret_hash");
    if let Some(hash) = stored_hash {
        let parsed_hash = PasswordHash::new(&hash).map_err(|e| {
            tracing::error!("Invalid password hash in DB: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": "AUTH_CONFIG_ERROR", "message": "invalid stored hash"})),
            )
        })?;

        if Argon2::default()
            .verify_password(provided_token.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "code": "INVALID_AGENT_TOKEN",
                    "message": "bearer token did not match stored hash",
                })),
            ));
        }
    } else {
        // Fallback ONLY if hash is NULL in DB (bootstrap mode)
        let expected_token = agent_tokens_from_env()
            .get(x_agent_id)
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "code": "AGENT_NOT_READY",
                    "message": "agent has no secret_hash and no bootstrap token configured",
                })),
            ))?;

        if provided_token != expected_token {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "code": "INVALID_AGENT_TOKEN",
                    "message": "bearer token did not match bootstrap token",
                })),
            ));
        }
    }

    // 3. Permission Check (RBAC)
    if let Some(check) = required_permission {
        let scope_val: Option<serde_json::Value> = row.get("scope_profile");
        let scope: ScopeProfile = serde_json::from_value(scope_val.unwrap_or(json!({})))
            .unwrap_or(ScopeProfile {
                allow_task_create: false,
                allow_task_claim: false,
                allow_mission_close: false,
                allow_council_propose: false,
                allow_council_finalize: false,
            });

        if !check(&scope) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "agent scope profile does not permit this operation",
                })),
            ));
        }
    }

    Ok(x_agent_id.to_string())
}

fn agent_tokens_from_env() -> HashMap<String, String> {
    env::var("AGENT_TOKENS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|pair| pair.split_once('='))
                .map(|(agent, token)| (agent.trim().to_string(), token.trim().to_string()))
                .filter(|(agent, token)| !agent.is_empty() && !token.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn internal_json_error(err: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "code": "INTERNAL_ERROR",
            "message": err,
        })),
    )
}
