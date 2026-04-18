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
    state.redis_reader
        .get_system_health()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, e))
}

async fn agents_list_handler(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    state.redis_reader
        .get_agents_status()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn agents_detail_handler(
    State(state): State<Arc<AppState>>,
    Path(_agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agents = state.redis_reader
        .get_agents_status()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let runs = state.postgres_reader.get_agents_runs(10).await.ok();
    let usage = state.postgres_reader.get_model_usage(10).await.ok();

    Ok(Json(json!({
        "agent": agents.first(),
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

