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
    configured: boolean;
    via?: string | null;
}

export interface ModelProviderStatus {
    generated_at: string;
    providers: ProviderStatus[];
    configured_agents: ConfiguredAgent[];
}

export interface AdminRuntimeStatus {
    generated_at: string;
    gateway: { status: string; message?: string };
    providers: ProviderStatus[];
    backend_log_tail: string[];
    notes: string[];
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
