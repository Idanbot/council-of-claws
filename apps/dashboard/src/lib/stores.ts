import { writable, derived } from 'svelte/store';
import type { Agent, Task, SystemHealth, Overview, AuditEvent } from './models';
import { createWebSocket, getOverview } from './api';

export const systemState = writable<Overview | null>(null);
export const auditLogs = writable<AuditEvent[]>([]);
export const wsConnected = writable(false);
export const lastRefreshed = writable<Date>(new Date());

// Derived stores for easier access
export const agents = derived(systemState, ($state) => $state?.active_agents || []);
export const activeTasks = derived(systemState, ($state) => {
    return [
        ...($state?.failed_tasks || []),
        ...($state?.blocked_tasks || [])
    ];
});

export const systemHealth = derived(systemState, ($state) => $state?.system_health);

export async function refreshData() {
    const { data } = await getOverview();
    if (data) {
        systemState.set(data);
        lastRefreshed.set(new Date());
    }
}

export function initRealtime() {
    if (typeof window === 'undefined') return;

    let ws: WebSocket;
    
    const connect = () => {
        ws = createWebSocket((msg: any) => {
            if (msg.event_type === 'connection') {
                wsConnected.set(true);
            } else if (msg.event_type === 'audit') {
                auditLogs.update(logs => [msg as AuditEvent, ...logs].slice(0, 100));
                // Auto-refresh summary when state changes
                refreshData();
            }
        });

        ws.onclose = () => {
            wsConnected.set(false);
            // Reconnect after 2 seconds
            setTimeout(connect, 2000);
        };
    };

    connect();

    return () => {
        if (ws) ws.close();
    };
}
