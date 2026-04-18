<script lang="ts">
  import { systemState, agents, activeTasks, auditLogs } from '$lib/stores';
  import { fade, fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';

  $: stats = [
    { label: 'Active Agents', value: $agents.length, color: 'text-indigo-500' },
    { label: 'Queue Depth', value: ($systemState?.queue_summary.pending_normal || 0) + ($systemState?.queue_summary.pending_critical || 0), color: 'text-amber-500' },
    { label: 'Token Burn', value: `$${$systemState?.queue_summary.completed || 0}`, color: 'text-emerald-500' },
    { label: 'Platform', value: $systemState?.system_health.backend.status === 'ok' ? 'NOMINAL' : 'DEGRADED', color: 'text-white' }
  ];
</script>

<div class="space-y-12">
  <!-- Metrics Grid -->
  <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
    {#each stats as stat, i}
      <div class="metric-card group" in:fly={{ y: 20, delay: i * 100 }}>
        <dt class="metric-label">{stat.label}</dt>
        <dd class="mt-4 flex items-baseline justify-between">
          <span class="text-4xl font-black tracking-tighter {stat.color}">{stat.value}</span>
        </dd>
        <div class="absolute -right-4 -bottom-4 opacity-5 group-hover:opacity-10 transition-opacity">
            <div class="h-24 w-24 rounded-full bg-white"></div>
        </div>
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-1 gap-12 lg:grid-cols-2">
    <!-- Active Intelligence -->
    <section>
      <div class="section-title">
        <div class="h-1 w-4 bg-emerald-500 rounded-full"></div>
        Active Intelligence
      </div>
      <div class="grid gap-4 mt-6">
        {#each $agents as agent (agent.agent_id)}
          <div
            class="glass-card p-6 flex items-center justify-between group glass-card-hover"
            animate:flip={{ duration: 400 }}
            in:fade
          >
            <div class="flex items-center gap-5">
              <div class="h-12 w-12 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center text-xs font-black text-slate-500 group-hover:border-indigo-500/50 transition-colors">
                {agent.agent_id.substring(0, 2).toUpperCase()}
              </div>
              <div>
                <div class="text-sm font-black text-white tracking-tight">{agent.agent_id}</div>
                <div class="text-[10px] text-slate-500 uppercase tracking-widest mt-0.5">{agent.model}</div>
              </div>
            </div>
            <div class="text-right">
              <span class="status-pill status-pill-ok">
                {agent.state}
              </span>
              <div class="mt-2 text-[10px] font-bold text-slate-600 tabular-nums uppercase">{agent.elapsed_seconds}s SESSION</div>
            </div>
          </div>
        {:else}
          <div class="glass-card bg-white/[0.02] border-dashed py-12 text-center text-xs font-bold text-slate-600 uppercase tracking-[0.2em]">
            No autonomous agents online
          </div>
        {/each}
      </div>
    </section>

    <!-- Signal Stream -->
    <section>
      <div class="section-title">
        <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
        Signal Stream
      </div>
      <div class="glass-card mt-6 divide-y divide-white/5 overflow-hidden">
        {#each $auditLogs.slice(0, 8) as log (log.id)}
          <div class="px-6 py-4 flex items-center gap-4 transition-colors hover:bg-white/[0.03]" in:fly={{ x: 20 }}>
            <div class="h-1.5 w-1.5 rounded-full {log.allowed ? 'bg-emerald-500 shadow-[0_0_8px_#10b981]' : 'bg-rose-500'}"></div>
            <div class="min-w-0 flex-1">
              <div class="text-[11px] font-black text-white uppercase tracking-tight">
                <span class="text-indigo-400">{log.agent_id || 'SYSTEM'}</span>
                <span class="mx-1 text-slate-600">»</span>
                {log.operation.replace('_', ' ')}
              </div>
              <div class="mt-0.5 truncate text-[10px] font-bold text-slate-500 uppercase tracking-tighter">
                {log.resource_type}: {log.resource_id}
              </div>
            </div>
            <time class="text-[9px] font-black text-slate-600 tabular-nums">
              {new Date(log.created_at).toLocaleTimeString([], { hour12: false })}
            </time>
          </div>
        {:else}
          <div class="p-12 text-center text-[10px] font-bold text-slate-600 uppercase tracking-[0.2em]">
            Awaiting incoming signals...
          </div>
        {/each}
      </div>
    </section>
  </div>

  <!-- Priority Objectives -->
  <section>
    <div class="section-title text-rose-500">
      <div class="h-1 w-4 bg-rose-500 rounded-full"></div>
      Priority Objectives
    </div>
    <div class="grid gap-6 mt-6 sm:grid-cols-2 lg:grid-cols-3">
      {#each $activeTasks as task (task.id)}
        <div class="glass-card p-6 border-l-4 border-l-rose-500 group glass-card-hover" in:fade>
          <div class="flex justify-between items-start">
            <h3 class="font-black text-sm text-white tracking-tight leading-tight uppercase">{task.title}</h3>
            <span class="status-pill status-pill-error ml-4">
              {task.status}
            </span>
          </div>
          <p class="mt-4 text-xs font-medium text-slate-500 line-clamp-2 leading-relaxed">{task.blocked_reason || 'No description provided'}</p>
          <div class="mt-6 flex justify-between items-center border-t border-white/5 pt-4">
            <span class="text-[9px] font-black text-slate-600 uppercase tracking-widest">ID: {task.id.split('-')[1]}</span>
            <span class="text-[9px] font-black text-indigo-400 uppercase tracking-widest">{task.owner_agent}</span>
          </div>
        </div>
      {:else}
        <div class="col-span-full glass-card bg-emerald-500/[0.02] border-emerald-500/20 py-10 text-center">
          <div class="text-[10px] font-black text-emerald-500 uppercase tracking-[0.3em]">System state: All objectives clear</div>
        </div>
      {/each}
    </div>
  </section>
</div>
