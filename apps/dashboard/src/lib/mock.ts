export const environmentLabel = 'local-dev';
export const hostLabel = 'council-of-claws';

export const globalStatus = {
  state: 'healthy',
  stream: 'polling+ws',
  refreshedAt: new Date().toISOString()
};

export const quickCounters = {
  activeAgents: 5,
  blockedTasks: 2,
  pendingTasks: 14,
  staleLeases: 1
};

export const systemHealth = [
  { label: 'Host CPU', value: '31%', status: 'ok' },
  { label: 'Host RAM', value: '58%', status: 'ok' },
  { label: 'Disk Free', value: '64%', status: 'ok' },
  { label: 'Redis', value: 'reachable', status: 'ok' },
  { label: 'PostgreSQL', value: 'read/write', status: 'ok' },
  { label: 'Backend', value: 'reachable', status: 'ok' },
  { label: 'Frontend', value: 'reachable', status: 'ok' },
  { label: 'Stale Tasks', value: '1', status: 'warn' },
  { label: 'Failed 24h', value: '3', status: 'warn' }
];

export const agents = [
  {
    id: 'contractor',
    state: 'working',
    taskId: 'task_2026_0042',
    priority: 'council',
    model: 'gpt-5.4',
    heartbeat: '10s ago',
    elapsed: '04m 12s',
    tasksToday: 18,
    tokensToday: 28400,
    recentEvents: 'Queued 2 tasks, summary sent'
  },
  {
    id: 'director',
    state: 'reviewing',
    taskId: 'task_2026_0044',
    priority: 'critical',
    model: 'gpt-5.4',
    heartbeat: '6s ago',
    elapsed: '08m 01s',
    tasksToday: 12,
    tokensToday: 21400,
    recentEvents: 'Split request into 3 subtasks'
  },
  {
    id: 'architect',
    state: 'idle',
    taskId: '-',
    priority: 'regular',
    model: 'gpt-5.4',
    heartbeat: '24s ago',
    elapsed: '00m 00s',
    tasksToday: 5,
    tokensToday: 9400,
    recentEvents: 'No pending assignments'
  },
  {
    id: 'senior_engineer',
    state: 'working',
    taskId: 'task_2026_0045',
    priority: 'critical',
    model: 'gpt-5.4',
    heartbeat: '5s ago',
    elapsed: '17m 54s',
    tasksToday: 9,
    tokensToday: 48900,
    recentEvents: 'Applied migration patch'
  },
  {
    id: 'junior_engineer',
    state: 'blocked',
    taskId: 'task_2026_0041',
    priority: 'regular',
    model: 'gpt-5.4-mini',
    heartbeat: '14s ago',
    elapsed: '13m 44s',
    tasksToday: 15,
    tokensToday: 17600,
    recentEvents: 'Waiting on DB seed data'
  }
];

export const tasks = [
  {
    id: 'task_2026_0045',
    title: 'Add infra smoke checks',
    lane: 'critical',
    status: 'in_progress',
    owner: 'senior_engineer',
    created: '2026-04-18T08:15:00Z',
    updated: '2026-04-18T08:31:00Z',
    reason: '-'
  },
  {
    id: 'task_2026_0044',
    title: 'Review council recommendation',
    lane: 'council',
    status: 'reviewing',
    owner: 'director',
    created: '2026-04-18T08:10:00Z',
    updated: '2026-04-18T08:28:00Z',
    reason: '-'
  },
  {
    id: 'task_2026_0041',
    title: 'Validate migration rollback',
    lane: 'regular',
    status: 'blocked',
    owner: 'junior_engineer',
    created: '2026-04-18T07:49:00Z',
    updated: '2026-04-18T08:23:00Z',
    reason: 'seed data unavailable'
  }
];

export const councilRuns = [
  {
    id: 'council_113',
    title: 'Choose migration strategy',
    phase: 'critique',
    participants: 'director, architect, senior_engineer',
    rounds: 2,
    status: 'active',
    ruling: 'Favor phased rollout with fallback',
    confidence: '0.76',
    for: 'Lower blast radius, faster rollback',
    against: 'Slightly more orchestration overhead',
    obsidianPath: 'Projects/platform/decisions/ADR-021.md'
  }
];

export const usageByAgent = [
  { agent: 'contractor', tokens: 28400, costUsd: 1.24 },
  { agent: 'director', tokens: 21400, costUsd: 1.01 },
  { agent: 'architect', tokens: 9400, costUsd: 0.45 },
  { agent: 'senior_engineer', tokens: 48900, costUsd: 2.31 },
  { agent: 'junior_engineer', tokens: 17600, costUsd: 0.82 }
];

export const usageByModel = [
  { model: 'gpt-5.4', tokens: 93500 },
  { model: 'gpt-5.4-mini', tokens: 32200 }
];

export const usageByDay = [
  { day: 'Mon', tokens: 82200 },
  { day: 'Tue', tokens: 90300 },
  { day: 'Wed', tokens: 99800 },
  { day: 'Thu', tokens: 74500 },
  { day: 'Fri', tokens: 121600 }
];

export const events = [
  { level: 'info', summary: 'Queue rebalance completed', stream: 'ws', at: '2026-04-18T08:30:12Z' },
  { level: 'warn', summary: '1 stale lease reclaimed', stream: 'ws', at: '2026-04-18T08:26:44Z' },
  { level: 'error', summary: 'Task task_2026_0036 failed validation', stream: 'poll', at: '2026-04-18T08:18:05Z' }
];

export const systemPanels = [
  { label: 'Host', status: 'ok', detail: 'cpu 31% · ram 58% · disk 64% free' },
  { label: 'Redis', status: 'ok', detail: 'reachable · memory 123MB' },
  { label: 'PostgreSQL', status: 'ok', detail: 'reachable · read/write' },
  { label: 'Backend', status: 'ok', detail: 'api reachable' },
  { label: 'Frontend', status: 'ok', detail: 'ui reachable' },
  { label: 'Containers', status: 'ok', detail: '6 running · 0 restarting' }
];
