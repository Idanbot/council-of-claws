use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============ Health & Status ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

// ============ Agent Related ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub agent_id: String,
    pub state: AgentState,
    pub current_task_id: Option<String>,
    pub priority: TaskPriority,
    pub model: String,
    pub last_heartbeat_ts: i64,
    pub elapsed_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Working,
    Reviewing,
    Blocked,
    Completed,
    Failed,
}

// ============ Task Related ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub owner_agent: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Reviewing,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Active,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub root_task_id: String,
    pub title: String,
    pub description: String,
    pub status: MissionStatus,
    pub created_by_agent: String,
    pub closed_by_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateRequest {
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub target_agent_id: String,
    pub mission_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateResponse {
    pub task_id: String,
    pub status: String,
    pub assigned_to: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCreateRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCreateResponse {
    pub mission_id: String,
    pub root_task_id: String,
    pub status: MissionStatus,
    pub created_at: DateTime<Utc>,
    pub created_by_agent: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCloseRequest {
    pub notes: Option<String>,
    pub obsidian_vault_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCloseSummary {
    pub all_valid_end_state: bool,
    pub failed_tasks: Vec<String>,
    pub warning_tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionCloseResponse {
    pub mission_id: String,
    pub status: MissionStatus,
    pub closed_at: DateTime<Utc>,
    pub closed_by_agent: String,
    pub subtask_count: i64,
    pub subtask_complete_count: i64,
    pub obsidian_doc_url: String,
    pub summary: MissionCloseSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub status: String,
    pub current_task_id: Option<String>,
}

// ============ Agent Runs ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub agent_id: String,
    pub task_id: String,
    pub model_name: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

// ============ Model Usage ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub id: String,
    pub agent_id: String,
    pub model_name: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub estimated_cost_usd: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_tokens: i32,
    pub total_cost_usd: f64,
    pub by_agent: Vec<UsageByAgent>,
    pub by_model: Vec<UsageByModel>,
    pub by_day: Vec<UsageByDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByAgent {
    pub agent_id: String,
    pub tokens: i32,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByModel {
    pub model_name: String,
    pub tokens: i32,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByDay {
    pub day: String,
    pub tokens: i32,
    pub cost_usd: f64,
}

// ============ Council Related ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilRun {
    pub id: String,
    pub title: String,
    pub status: String,
    pub phase: CouncilPhase,
    pub director_agent: String,
    pub participants: Vec<String>,
    pub ruling_summary: Option<String>,
    pub confidence: Option<f64>,
    pub obsidian_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CouncilPhase {
    Debating,
    Voting,
    Concluded,
}

// ============ Events ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub level: EventLevel,
    pub summary: String,
    pub stream_connection: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    Info,
    Warn,
    Error,
}

// ============ System Health ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub timestamp: i64,
    pub host: HostMetrics,
    pub redis: ServiceMetrics,
    pub postgres: ServiceMetrics,
    pub backend: ServiceMetrics,
    pub frontend: ServiceMetrics,
    pub containers: ContainerMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub running: i32,
    pub stopped: i32,
    pub unhealthy: i32,
}

// ============ Queue Summary ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSummary {
    pub pending_critical: i32,
    pub pending_high: i32,
    pub pending_normal: i32,
    pub pending_low: i32,
    pub in_progress: i32,
    pub reviewing: i32,
    pub blocked: i32,
    pub completed: i32,
    pub failed: i32,
}

// ============ Overview ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    pub system_health: SystemHealth,
    pub active_agents: Vec<Agent>,
    pub queue_summary: QueueSummary,
    pub recent_events: Vec<DashboardEvent>,
    pub council_summaries: Vec<CouncilRun>,
    pub failed_tasks: Vec<Task>,
    pub blocked_tasks: Vec<Task>,
}

// ============ Agent Identity ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeProfile {
    #[serde(default)]
    pub allow_task_create: bool,
    #[serde(default)]
    pub allow_task_claim: bool,
    #[serde(default)]
    pub allow_mission_close: bool,
    #[serde(default)]
    pub allow_council_propose: bool,
    #[serde(default)]
    pub allow_council_finalize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub id: String,
    pub display_name: String,
    pub role: String,
    pub scope_profile: Option<serde_json::Value>,
    pub secret_hash: Option<String>,
}

// ============ Council State Machine ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilProposeRequest {
    pub title: String,
    pub initial_summary: String,
    pub participants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilProposeResponse {
    pub council_id: String,
    pub status: String,
    pub phase: CouncilPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVoteRequest {
    pub vote: String, // approve, reject, abstain
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilFinalizeRequest {
    pub ruling_summary: String,
    pub confidence: f64,
}

// ============ Secret Rotation ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRotateRequest {
    pub new_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRotateResponse {
    pub rotated_at: DateTime<Utc>,
}

// ============ Audit Related ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub request_id: Option<String>,
    pub agent_id: Option<String>,
    pub operation: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub allowed: bool,
    pub result: Option<String>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOperation {
    TaskCreate,
    TaskClaim,
    TaskStatusSet,
    TaskComplete,
    TaskFail,
    MissionCreate,
    MissionClose,
    CouncilPropose,
    CouncilCreate,
    CouncilRoundRecord,
    CouncilFinalize,
    AgentHeartbeat,
    AgentStatusSet,
}

impl ToString for AuditOperation {
    fn to_string(&self) -> String {
        serde_json::to_value(self)
            .unwrap_or_default()
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

