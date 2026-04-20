<script lang="ts">
  import { onMount } from 'svelte';
  import { getAnalyticsSummary } from '$lib/api';
  import type { AnalyticsSummary } from '$lib/models';
  import ProviderPerformance from '$lib/ProviderPerformance.svelte';

  let summary: AnalyticsSummary | null = null;
  let loading = true;
  let error: string | null = null;

  async function fetchAnalytics() {
    loading = true;
    error = null;
    const { data, error: apiError } = await getAnalyticsSummary();
    if (apiError) {
      error = apiError.message;
    } else if (data) {
      summary = data;
    }
    loading = false;
  }

  onMount(fetchAnalytics);
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Performance</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-indigo-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Model provider latency and cost analytics</p>
      </div>
    </div>
    <button on:click={fetchAnalytics} class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10" disabled={loading}>
      {loading ? 'Analyzing…' : 'Refresh Metrics'}
    </button>
  </div>

  {#if loading && !summary}
    <div class="grid gap-6 sm:grid-cols-4">
      {#each Array(4) as _}
        <div class="glass-card h-32 animate-pulse bg-white/5 border-none"></div>
      {/each}
    </div>
  {:else if error}
    <div class="glass-card border-rose-500/30 bg-rose-500/5 p-8 text-center text-sm text-slate-300">
      Failed to load analytics: {error}
    </div>
  {:else if summary}
    <div class="space-y-12">
      <section>
        <div class="section-title mb-6">
          <div class="h-1 w-4 bg-emerald-500 rounded-full"></div>
          Provider Health & Efficiency
        </div>
        <ProviderPerformance providers={summary.providers} />
      </section>

      <section>
        <div class="section-title mb-6">
          <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
          Historical Volume
        </div>
        <div class="glass-card p-8 min-h-[300px] flex items-center justify-center border-white/5">
          {#if summary.hourly_usage.length === 0}
            <p class="text-slate-500 italic text-sm">No historical usage data recorded in the last 24 hours.</p>
          {:else}
            <!-- Chart visualization would go here, using ProviderPerformance component for now -->
            <div class="w-full space-y-4">
              <div class="flex justify-between items-end h-32 gap-1 px-4">
                {#each summary.hourly_usage.slice(-24) as point}
                  <div 
                    class="bg-indigo-500/40 hover:bg-indigo-400/60 transition-all rounded-t-sm flex-1 group relative" 
                    style="height: {Math.max(10, (point.tokens / Math.max(...summary.hourly_usage.map(p => p.tokens))) * 100)}%"
                  >
                    <div class="absolute -top-10 left-1/2 -translate-x-1/2 bg-slate-900 border border-white/10 rounded px-2 py-1 text-[8px] text-white opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-10">
                      {point.tokens} tokens
                    </div>
                  </div>
                {/each}
              </div>
              <div class="flex justify-between text-[7px] font-black text-slate-600 uppercase tracking-widest px-4">
                <span>{new Date(summary.hourly_usage[0].timestamp).getHours()}:00</span>
                <span>Timeline (24h)</span>
                <span>Now</span>
              </div>
            </div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
