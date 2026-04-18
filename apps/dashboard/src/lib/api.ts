// API client for council-of-claws dashboard
// Makes HTTP calls to backend at /api/*

import type { Agent, CouncilRun, DashboardEvent, Overview, SystemHealth, Task, UsageSummary } from './models';

const API_BASE = '/api';

export interface ApiError {
    message: string;
    status?: number;
}

async function apiCall<T>(
    endpoint: string,
    options: RequestInit = {}
): Promise<{ data?: T; error?: ApiError }> {
    try {
        const url = `${API_BASE}${endpoint}`;
        const response = await fetch(url, {
            headers: {
                'Content-Type': 'application/json',
                ...options.headers,
            },
            ...options,
        });

        if (!response.ok) {
            return {
                error: {
                    message: `HTTP ${response.status}: ${response.statusText}`,
                    status: response.status,
                },
            };
        }

        const data = await response.json();
        return { data };
    } catch (err) {
        return {
            error: {
                message: err instanceof Error ? err.message : 'Unknown error',
            },
        };
    }
}

// Health endpoints
export async function getHealth() {
    return apiCall<{ service: string; status: string; timestamp?: string }>('/health');
}

export async function getHealthServices() {
    return apiCall<{ services: Array<{ name: string; status: string }>; timestamp: string }>('/health/services');
}

// Overview endpoints
export async function getOverview() {
    return apiCall<Overview>('/overview');
}

export async function getOverviewSystem() {
    return apiCall<SystemHealth>('/overview/system');
}

// Agent endpoints
export async function getAgents() {
    return apiCall<Agent[]>('/agents');
}

export async function getAgent(agentId: string) {
    return apiCall<{ agent?: Agent; runs?: unknown[]; usage?: unknown[] }>(`/agents/${agentId}`);
}

// Task endpoints
export async function getTasks() {
    return apiCall<Task[]>('/tasks');
}

export async function getTask(taskId: string) {
    return apiCall<{ task: Task; runs?: unknown[] }>(`/tasks/${taskId}`);
}

// Council endpoints
export async function getCouncils() {
    return apiCall<CouncilRun[]>('/council');
}

export async function getCouncil(councilId: string) {
    return apiCall<CouncilRun>(`/council/${councilId}`);
}

// Usage endpoints
export async function getUsageSummary() {
    return apiCall<UsageSummary>('/usage');
}

export async function getUsageAgents() {
    return apiCall<Array<{ id: string; agent_id: string; model_name: string; total_tokens: number; estimated_cost_usd: number; created_at: string }>>('/usage/agents');
}

export async function getUsageModels() {
    return apiCall<Array<{ id: string; agent_id: string; model_name: string; total_tokens: number; estimated_cost_usd: number; created_at: string }>>('/usage/models');
}

// Events endpoints
export async function getEvents() {
    return apiCall<DashboardEvent[]>('/events');
}

// WebSocket connection
export function createWebSocket(onMessage: (data: unknown) => void): WebSocket {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // Use window.location.host which includes the port
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    console.log(`[WS] Connecting to ${wsUrl}`);
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('[WS] Connection established');
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            onMessage(data);
        } catch (err) {
            console.error('Failed to parse WebSocket message:', err);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };

    return ws;
}
