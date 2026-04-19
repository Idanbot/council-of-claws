<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { getAgents, getConfiguredAgents } from '$lib/api';
  import type { Agent, ConfiguredAgent } from '$lib/models';
  import { configuredAgents, configuredAgentLabels } from '$lib/configured-agents';

  let liveAgents: Agent[] = [];
  let configured: ConfiguredAgent[] = [];
  let loading = true;
  let error: string | null = null;
  let syncedAt = '';

  async function fetchAgents() {
    loading = true;
    error = null;
    const [{ data, error: apiError }, { data: configuredData }] = await Promise.all([
      getAgents(),
      getConfiguredAgents()
    ]);
    if (apiError) {
      error = apiError.message;
    }
    liveAgents = (data || []).filter((agent) => agent.last_heartbeat_ts > 0);
    configured = configuredData && configuredData.length > 0
      ? configuredData
      : configuredAgents.map((agent) => ({
          agent_id: agent.agent_id,
          role: configuredAgentLabels[agent.agent_id] || 'configured',
          primary_model: agent.model,
          fallbacks: [],
          priority: agent.priority
        }));
    syncedAt = new Date().toLocaleTimeString([], { hour12: false });
    loading = false;
  }

  onMount(fetchAgents);
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Intelligence</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-emerald-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Registered autonomous agents and model assignments</p>
      </div>
    </div>
    <button on:click={fetchAgents} class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10" disabled={loading}>
      {loading ? 'Refreshing…' : 'Refresh Roster'}
    </button>
  </div>

  {#if error}
    <div class="glass-card border-rose-500/30 bg-rose-500/5 p-8 text-center text-sm text-slate-300">
      Agent telemetry request failed: {error}. Showing configured roster instead.
    </div>
  {/if}

  {#if syncedAt}
    <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Last sync: {syncedAt}</div>
  {/if}

  <div class="space-y-6">
    <div class="section-title">
      <div class="h-1 w-4 bg-emerald-500 rounded-full"></div>
      Live Telemetry
    </div>

  {#if loading && liveAgents.length === 0}
    <div class="grid gap-8 sm:grid-cols-2">
      {#each Array(4) as _}
        <div class="glass-card animate-pulse h-48 bg-white/5 border-none"></div>
      {/each}
    </div>
  {:else}
    {#if liveAgents.length === 0}
      <div class="glass-card border-amber-500/20 bg-amber-500/5 p-8 text-sm text-slate-300">
        No live heartbeats yet. The configured roster below is still valid, but runtime telemetry will only appear after agents start sending heartbeats.
      </div>
    {:else}
      <div class="grid gap-8 sm:grid-cols-2">
        {#each liveAgents as agent (agent.agent_id)}
          <div class="glass-card p-8 group glass-card-hover transition-all" in:scale={{ start: 0.98 }}>
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-6">
                <div class="h-16 w-16 rounded-2xl bg-indigo-500/10 flex items-center justify-center text-xs font-black text-indigo-400 border border-indigo-500/20 group-hover:scale-105 group-hover:bg-indigo-500/20 transition-all">
                  {agent.agent_id.substring(0, 2).toUpperCase()}
                </div>
                <div>
                  <h3 class="text-lg font-black text-white tracking-tight uppercase">{agent.agent_id}</h3>
                  <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">{configuredAgentLabels[agent.agent_id] || agent.model}</p>
                </div>
              </div>
              <div class="flex flex-col items-end">
                <span class="status-pill status-pill-ok">
                  {agent.state}
                </span>
              </div>
            </div>

            <div class="mt-10 grid grid-cols-2 gap-6 border-t border-white/5 pt-6">
              <div>
                <p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Active Objective</p>
                <p class="mt-2 text-xs font-bold text-slate-300 truncate">
                  {agent.current_task_id || 'STANDBY'}
                </p>
              </div>
              <div class="text-right">
                <p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Primary Model</p>
                <p class="mt-2 text-xs font-bold text-slate-300 tabular-nums uppercase">
                  {agent.model}
                </p>
              </div>
            </div>

            <div class="mt-8 flex items-center gap-3">
              <div class="h-1 flex-1 rounded-full bg-white/5 overflow-hidden">
                <div class="h-full bg-emerald-500 shadow-[0_0_10px_#10b981] animate-pulse" style="width: {agent.elapsed_seconds > 0 ? '100%' : '35%'}"></div>
              </div>
              <span class="text-[8px] font-black text-emerald-500 uppercase tracking-widest">{agent.elapsed_seconds > 0 ? 'Live' : 'Configured'}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
  </div>

  <div class="space-y-6">
    <div class="section-title">
      <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
      Configured Roster
    </div>

    <div class="grid gap-8 sm:grid-cols-2">
      {#each configured as agent (agent.agent_id)}
        <div class="glass-card p-8 group glass-card-hover transition-all" in:scale={{ start: 0.98 }}>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-6">
              <div class="h-16 w-16 rounded-2xl bg-indigo-500/10 flex items-center justify-center text-xs font-black text-indigo-400 border border-indigo-500/20">
                {agent.agent_id.substring(0, 2).toUpperCase()}
              </div>
              <div>
                <h3 class="text-lg font-black text-white tracking-tight uppercase">{agent.agent_id}</h3>
                <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">{agent.role}</p>
              </div>
            </div>
            <span class="status-pill status-pill-info">Configured</span>
          </div>

          <div class="mt-8 grid gap-4 border-t border-white/5 pt-6">
            <div>
              <p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Primary Model</p>
              <p class="mt-2 text-xs font-bold text-slate-300 uppercase">{agent.primary_model}</p>
            </div>
            <div>
              <p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Fallbacks</p>
              <p class="mt-2 text-xs font-bold text-slate-300 uppercase">{agent.fallbacks.join(' -> ') || 'None'}</p>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
