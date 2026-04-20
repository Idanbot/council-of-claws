// TypeScript type definitions matching backend models

export interface Agent {
    agent_id: string;
    state: 'idle' | 'working' | 'reviewing' | 'blocked' | 'completed' | 'failed';
    current_task_id: string | null;
    priority: string;
    model: string;
    last_heartbeat_ts: number;
    elapsed_seconds: number;
}

export interface ConfiguredAgent {
    agent_id: string;
    role: string;
    primary_model: string;
    fallbacks: string[];
    priority: string;
}

export interface AgentStatusSnapshot {
    configured: ConfiguredAgent;
    live?: Agent | null;
    heartbeat_age_seconds?: number | null;
    status: 'live' | 'stale' | 'configured' | string;
}

export interface AgentsStatusReport {
    generated_at: string;
    heartbeat_ttl_seconds: number;
    configured_count: number;
    live_count: number;
    stale_count: number;
    agents: AgentStatusSnapshot[];
}

export interface ProviderStatus {
    provider: string;
    enabled: boolean;
    configured: boolean;
    discovered: boolean;
    status: string;
    via?: string | null;
    base_url?: string | null;
    model_count: number;
    available_models: string[];
    configured_model_refs: string[];
    auth_profiles: string[];
    issues: string[];
}

export interface OpenClawSnapshotMeta {
    schema_version: number;
    snapshot_fingerprint: string;
    status: string;
    generated_at: string;
    last_success_at: string;
    source_mtime?: string | null;
    snapshot_age_seconds: number;
}

export interface OpenClawSnapshotHistorySummary {
    snapshot_count: number;
    latest_generated_at?: string | null;
    latest_persisted_at?: string | null;
    latest_snapshot_fingerprint?: string | null;
}

export interface ModelProviderStatus {
    generated_at: string;
    snapshot: OpenClawSnapshotMeta;
    providers: ProviderStatus[];
    configured_agents: ConfiguredAgent[];
    available_model_refs: string[];
    invalid_model_refs: string[];
    issues: string[];
}

export interface AdminRuntimeStatus {
    generated_at: string;
    snapshot: OpenClawSnapshotMeta;
    history: OpenClawSnapshotHistorySummary;
    gateway: { status: string; message?: string };
    providers: ProviderStatus[];
    backend_log_tail: string[];
    notes: string[];
    openclaw_source_path: string;
    runtime_state_available: boolean;
    issues: string[];
}

export interface Task {
    id: string;
    title: string;
    priority: string;
    status: string;
    owner_agent: string;
    created_at: string;
    updated_at: string;
    blocked_reason?: string;
}

export interface SystemHealth {
    timestamp: number;
    host?: {
        cpu_percent: number;
        memory_percent: number;
        disk_percent: number;
    };
    redis?: {
        status: string;
        message?: string;
    };
    postgres?: {
        status: string;
        message?: string;
    };
    backend?: {
        status: string;
        message?: string;
    };
    frontend?: {
        status: string;
        message?: string;
    };
    containers?: {
        running: number;
        stopped: number;
        unhealthy: number;
    };
}

export interface DiagnosticsCheck {
    name: string;
    status: string;
    info: string;
    duration_ms: number;
}

export interface DiagnosticsReport {
    generated_at: string;
    overall_status: string;
    checks: DiagnosticsCheck[];
}

export interface QueueSummary {
    pending_critical: number;
    pending_high: number;
    pending_normal: number;
    pending_low: number;
    in_progress: number;
    reviewing: number;
    blocked: number;
    completed: number;
    failed: number;
}

export interface DashboardEvent {
    level: 'info' | 'warn' | 'error';
    summary: string;
    stream_connection: string;
    timestamp: number;
}

export interface AuditEvent {
    id: string;
    request_id?: string | null;
    agent_id?: string | null;
    operation: string;
    resource_type?: string | null;
    resource_id?: string | null;
    allowed: boolean;
    result?: string | null;
    reason?: string | null;
    metadata?: unknown;
    created_at: string;
}

export interface CouncilRun {
    id: string;
    title: string;
    status: string;
    phase: 'debating' | 'voting' | 'concluded';
    director_agent: string;
    participants?: string[];
    ruling_summary?: string;
    confidence?: number;
    obsidian_path?: string;
    created_at: string;
    updated_at: string;
}

export interface Overview {
    system_health: SystemHealth;
    active_agents: Agent[];
    configured_agents: ConfiguredAgent[];
    queue_summary: QueueSummary;
    recent_events: DashboardEvent[];
    council_summaries: CouncilRun[];
    failed_tasks: Task[];
    blocked_tasks: Task[];
}

export interface UsageSummary {
    total_tokens: number;
    total_cost_usd: number;
    by_agent: Array<{ agent_id: string; tokens: number; cost_usd: number }>;
    by_model: Array<{ model_name: string; tokens: number; cost_usd: number }>;
    by_day: Array<{ day: string; tokens: number; cost_usd: number }>;
}

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface AnalyticsSummary {
    providers: ProviderAnalytics[];
    hourly_usage: UsageDataPoint[];
}

export interface ProviderAnalytics {
    provider: string;
    avg_latency_ms: number;
    total_cost_usd: number;
    total_tokens: number;
    success_rate: number;
}

export interface UsageDataPoint {
    timestamp: string;
    tokens: number;
    cost_usd: number;
}

export interface ModelUsage {
    id: string;
    agent_id: string;
    model_name: string;
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
    estimated_cost_usd: number;
    latency_ms: number | null;
    created_at: string;
}
