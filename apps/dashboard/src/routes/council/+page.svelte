<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { getCouncils } from '$lib/api';
  import type { CouncilRun } from '$lib/models';

  let councils: CouncilRun[] = [];
  let loading = true;

  async function fetchCouncils() {
    loading = true;
    const { data } = await getCouncils();
    if (data) councils = data;
    loading = false;
  }

  onMount(fetchCouncils);
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Council Chambers</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-purple-500 rounded-full shadow-[0_0_8px_#a855f7]"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Collaborative decision records and consensus history</p>
      </div>
    </div>
    <button on:click={fetchCouncils} class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10" disabled={loading}>
      {loading ? 'Consulting archives...' : 'Refresh Councils'}
    </button>
  </div>

  {#if loading && councils.length === 0}
    <div class="grid gap-8 sm:grid-cols-2">
      {#each Array(4) as _}
        <div class="glass-card animate-pulse h-48 bg-white/5 border-none"></div>
      {/each}
    </div>
  {:else if councils.length === 0}
    <div class="glass-card border-dashed py-24 text-center space-y-4" in:fade>
      <div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">The Council is silent</div>
      <p class="text-[10px] font-bold text-slate-700 uppercase tracking-widest italic">No formal deliberations have been recorded yet</p>
    </div>
  {:else}
    <div class="grid gap-8 lg:grid-cols-1">
      {#each councils as run}
        <div class="glass-card p-10 group glass-card-hover transition-all" in:fly={{ y: 20 }}>
          <div class="flex flex-col md:flex-row md:items-center justify-between gap-6">
            <div class="flex items-center gap-6">
              <div class="h-16 w-16 rounded-2xl bg-purple-500/10 flex items-center justify-center text-xs font-black text-purple-400 border border-purple-500/20 group-hover:scale-105 transition-all">
                🏛️
              </div>
              <div>
                <h3 class="text-xl font-black text-white tracking-tight uppercase group-hover:text-purple-400 transition-colors">{run.title}</h3>
                <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">ID: {run.id}</p>
              </div>
            </div>
            <div class="flex items-center gap-3 self-end md:self-auto">
              <span class="status-pill status-pill-info">
                {run.phase}
              </span>
              <span class="status-pill {run.status === 'completed' ? 'status-pill-ok' : 'status-pill-warn'}">
                {run.status}
              </span>
            </div>
          </div>

          {#if run.ruling_summary}
            <div class="mt-10 p-8 rounded-2xl bg-black/40 border border-white/5 relative overflow-hidden">
              <div class="text-[9px] font-black text-slate-500 uppercase tracking-[0.3em] mb-4">Final Ruling</div>
              <p class="text-base text-slate-300 italic font-medium leading-relaxed">"{run.ruling_summary}"</p>
              {#if run.confidence}
                <div class="mt-8 flex items-center gap-4">
                  <div class="h-1 flex-1 rounded-full bg-white/5">
                    <div class="h-full bg-purple-500 shadow-[0_0_10px_rgba(168,85,247,0.5)] transition-all duration-1000" style="width: {run.confidence * 100}%"></div>
                  </div>
                  <span class="text-[10px] font-black tabular-nums text-purple-400 uppercase tracking-tighter">{(run.confidence * 100).toFixed(0)}% CONFIDENCE</span>
                </div>
              {/if}
            </div>
          {/if}

          <div class="mt-10 flex flex-wrap items-center justify-between gap-6 border-t border-white/5 pt-8">
            <div class="flex items-center gap-6">
              <div class="flex flex-col gap-1">
                  <span class="text-[8px] font-black text-slate-600 uppercase tracking-widest">Authority</span>
                  <span class="text-[10px] font-black text-slate-400 uppercase">{run.director_agent}</span>
              </div>
              <div class="flex flex-col gap-1 border-l border-white/5 pl-6">
                  <span class="text-[8px] font-black text-slate-600 uppercase tracking-widest">Last Transmission</span>
                  <span class="text-[10px] font-black text-slate-400 uppercase tabular-nums">{new Date(run.updated_at).toLocaleString([], { hour12: false })}</span>
              </div>
            </div>
            {#if run.obsidian_path}
              <a href={run.obsidian_path} class="btn-secondary h-9 px-6 text-[10px] uppercase tracking-[0.2em] font-black hover:border-purple-500/50 hover:text-purple-400 transition-all">
                  Open Records
              </a>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
