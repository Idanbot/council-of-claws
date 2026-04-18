<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { getAgents } from '$lib/api';
  import type { Agent } from '$lib/models';

  let agents: Agent[] = [];
  let loading = true;

  async function fetchAgents() {
    loading = true;
    const { data } = await getAgents();
    if (data) agents = data;
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
  </div>

  {#if loading && agents.length === 0}
    <div class="grid gap-8 sm:grid-cols-2">
      {#each Array(4) as _}
        <div class="glass-card animate-pulse h-48 bg-white/5 border-none"></div>
      {/each}
    </div>
  {:else}
    <div class="grid gap-8 sm:grid-cols-2">
      {#each agents as agent (agent.agent_id)}
        <div class="glass-card p-8 group glass-card-hover transition-all" in:scale={{ start: 0.98 }}>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-6">
              <div class="h-16 w-16 rounded-2xl bg-indigo-500/10 flex items-center justify-center text-xs font-black text-indigo-400 border border-indigo-500/20 group-hover:scale-105 group-hover:bg-indigo-500/20 transition-all">
                {agent.agent_id.substring(0, 2).toUpperCase()}
              </div>
              <div>
                <h3 class="text-lg font-black text-white tracking-tight uppercase">{agent.agent_id}</h3>
                <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">{agent.model}</p>
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
              <p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Persistence</p>
              <p class="mt-2 text-xs font-bold text-slate-300 tabular-nums uppercase">
                {agent.elapsed_seconds}s cycle
              </p>
            </div>
          </div>

          <div class="mt-8 flex items-center gap-3">
            <div class="h-1 flex-1 rounded-full bg-white/5 overflow-hidden">
              <div class="h-full bg-emerald-500 shadow-[0_0_10px_#10b981] animate-pulse" style="width: 100%"></div>
            </div>
            <span class="text-[8px] font-black text-emerald-500 uppercase tracking-widest">Ready</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
