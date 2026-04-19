import { writable, derived } from 'svelte/store';
import type { Agent, DashboardEvent, Overview } from './models';
import { createWebSocket, getOverview } from './api';
import { configuredAgents } from './configured-agents';

export const systemState = writable<Overview | null>(null);
export const streamEvents = writable<DashboardEvent[]>([]);
export const wsConnected = writable(false);
export const apiHealthy = writable(false);
export const refreshing = writable(false);
export const lastRefreshed = writable<Date>(new Date());

// Derived stores for easier access
export const agents = derived(systemState, ($state) => {
    const liveAgents = $state?.active_agents || [];
    return liveAgents.length > 0 ? liveAgents : configuredAgents;
});
export const activeTasks = derived(systemState, ($state) => {
    return [
        ...($state?.failed_tasks || []),
        ...($state?.blocked_tasks || [])
    ];
});

export const systemHealth = derived(systemState, ($state) => $state?.system_health);
export const liveAgents = derived(systemState, ($state) => $state?.active_agents || []);
export const transportStatus = derived(
    [wsConnected, apiHealthy],
    ([$wsConnected, $apiHealthy]) => {
        if ($wsConnected) return 'LIVE';
        if ($apiHealthy) return 'POLLING';
        return 'OFFLINE';
    }
);
export const infrastructureOnline = derived(
    [wsConnected, apiHealthy],
    ([$wsConnected, $apiHealthy]) => $wsConnected || $apiHealthy
);

export async function refreshData() {
    refreshing.set(true);
    const { data } = await getOverview();
    if (data) {
        systemState.set(data);
        streamEvents.set(data.recent_events || []);
        apiHealthy.set(true);
        lastRefreshed.set(new Date());
    } else {
        apiHealthy.set(false);
    }
    refreshing.set(false);
}

export function initRealtime() {
    if (typeof window === 'undefined') return;

    let ws: WebSocket;

    const connect = () => {
        ws = createWebSocket((msg: any) => {
            if (msg.event_type === 'connection') {
                wsConnected.set(true);
                apiHealthy.set(true);
            } else if (msg.summary && typeof msg.timestamp === 'number') {
                streamEvents.update(logs => [msg as DashboardEvent, ...logs].slice(0, 100));
                // Auto-refresh summary when state changes
                refreshData();
            }
        }, (connected) => {
            wsConnected.set(connected);
        });

        ws.onclose = () => {
            wsConnected.set(false);
            setTimeout(connect, 2000);
        };
    };

    connect();

    return () => {
        if (ws) ws.close();
    };
}
