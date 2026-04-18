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
