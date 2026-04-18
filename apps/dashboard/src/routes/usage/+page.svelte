<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { getUsageSummary } from '$lib/api';
  import type { UsageSummary } from '$lib/models';

  let summary: UsageSummary | null = null;
  let loading = true;

  async function fetchUsage() {
    loading = true;
    const { data } = await getUsageSummary();
    if (data) summary = data;
    loading = false;
  }

  onMount(fetchUsage);

  $: modelUsage = summary?.by_model || [];
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Economics</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-indigo-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Total token throughput and estimated operational overhead</p>
      </div>
    </div>
  </div>

  {#if loading && !summary}
    <div class="grid gap-8 sm:grid-cols-2 lg:grid-cols-3">
      {#each Array(3) as _}
        <div class="glass-card animate-pulse h-64 bg-white/5 border-none"></div>
      {/each}
    </div>
  {:else if summary}
    <div class="grid gap-8 lg:grid-cols-3">
      <!-- Total Tokens -->
      <div class="glass-card p-10 bg-indigo-600/10 border-indigo-500/20" in:fly={{ y: 20 }}>
        <div class="text-[9px] font-black uppercase tracking-[0.3em] text-indigo-400">Total Throughput</div>
        <p class="mt-6 text-5xl font-black tabular-nums tracking-tighter text-white">{summary.total_tokens.toLocaleString()}</p>
        <p class="text-[10px] font-bold text-indigo-500/60 uppercase mt-2">Tokens Processed</p>

        <div class="mt-12 flex justify-between items-end border-t border-indigo-500/10 pt-6">
          <div class="text-[9px] font-black text-slate-500 uppercase tracking-widest text-left">Projected Cost</div>
          <div class="text-2xl font-black font-mono text-white tracking-tighter">${summary.total_cost_usd.toFixed(4)}</div>
        </div>
      </div>

      <!-- By Model -->
      <div class="glass-card p-10 lg:col-span-2" in:fly={{ y: 20, delay: 100 }}>
        <div class="text-[9px] font-black uppercase tracking-[0.3em] text-slate-500">Distribution by Provider</div>
        <div class="mt-8 space-y-8">
          {#each modelUsage as model}
            <div class="space-y-3">
              <div class="flex justify-between items-end">
                <span class="text-[11px] font-black text-white uppercase tracking-wider">{model.model_name}</span>
                <span class="text-[10px] font-bold text-slate-500 tabular-nums uppercase">{model.tokens.toLocaleString()} TOKENS</span>
              </div>
              <div class="h-1.5 w-full rounded-full bg-white/5 overflow-hidden">
                <div
                  class="h-full bg-indigo-500 shadow-[0_0_10px_rgba(99,102,241,0.5)] transition-all duration-1000 ease-out"
                  style="width: {(model.tokens / summary.total_tokens * 100)}%"
                ></div>
              </div>
            </div>
          {:else}
             <p class="text-center py-12 text-xs font-bold text-slate-600 uppercase tracking-widest italic">Awaiting model usage telemetry...</p>
          {/each}
        </div>
      </div>
    </div>

    <!-- Agent Breakdown -->
    <div class="glass-card overflow-hidden" in:fade={{ delay: 300 }}>
      <div class="px-10 py-6 border-b border-white/5 bg-white/[0.02]">
          <h2 class="text-[10px] font-black uppercase tracking-[0.3em] text-slate-500">Agent Performance metrics</h2>
      </div>
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="text-[9px] uppercase text-slate-500 font-black border-b border-white/5 bg-black/20">
            <tr>
              <th class="py-4 px-10 tracking-widest">Autonomous Entity</th>
              <th class="py-4 px-6 tracking-widest">Throughput</th>
              <th class="py-4 px-6 tracking-widest">Cost (USD)</th>
              <th class="py-4 px-10 text-right tracking-widest">Efficiency</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-white/5">
            {#each summary.by_agent as agent}
              <tr class="hover:bg-white/[0.02] transition-colors group">
                <td class="py-5 px-10 font-black text-white uppercase group-hover:text-indigo-400 transition-colors">{agent.agent_id}</td>
                <td class="py-5 px-6 tabular-nums font-bold text-slate-400">{agent.tokens.toLocaleString()}</td>
                <td class="py-5 px-6 tabular-nums font-mono text-slate-400 font-bold">${agent.cost_usd.toFixed(4)}</td>
                <td class="py-5 px-10 text-right">
                    <span class="status-pill status-pill-info">OPTIMAL</span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
