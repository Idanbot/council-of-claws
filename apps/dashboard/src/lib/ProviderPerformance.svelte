<script lang="ts">
  import type { ProviderAnalytics } from './models';

  export let providers: ProviderAnalytics[] = [];

  $: sortedByLatency = [...providers].sort((a, b) => a.avg_latency_ms - b.avg_latency_ms);
</script>

<div class="space-y-6">
  <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
    {#each sortedByLatency as p}
      <div class="glass-card p-6 border-white/5 bg-white/2 hover:bg-white/[0.04] transition-all group">
        <div class="flex items-center justify-between">
          <h4 class="text-[10px] font-black uppercase tracking-widest text-slate-500 group-hover:text-indigo-400 transition-colors">{p.provider}</h4>
          <div class="h-1.5 w-1.5 rounded-full {p.avg_latency_ms < 1000 ? 'bg-emerald-500 shadow-[0_0_8px_#10b981]' : 'bg-amber-500 shadow-[0_0_8px_#f59e0b]'}"></div>
        </div>
        
        <div class="mt-6 flex items-baseline gap-2">
          <span class="text-2xl font-black text-white tracking-tighter">{p.avg_latency_ms.toFixed(0)}</span>
          <span class="text-[9px] font-bold text-slate-500 uppercase tracking-widest">ms avg</span>
        </div>

        <div class="mt-8 pt-4 border-t border-white/5 grid grid-cols-2 gap-4">
          <div>
            <p class="text-[8px] font-black text-slate-500 uppercase tracking-tighter">Total Cost</p>
            <p class="text-[11px] font-bold text-slate-300 mt-1">${p.total_cost_usd.toFixed(4)}</p>
          </div>
          <div class="text-right">
            <p class="text-[8px] font-black text-slate-500 uppercase tracking-tighter">Volume</p>
            <p class="text-[11px] font-bold text-slate-300 mt-1">{(p.total_tokens / 1000).toFixed(1)}k</p>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <div class="glass-card p-8 border-white/5 bg-slate-950/20">
    <div class="section-title mb-8">
      <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
      Latency Benchmarks
    </div>
    
    <div class="space-y-6">
      {#each sortedByLatency as p}
        <div class="space-y-2">
          <div class="flex justify-between items-center text-[9px] font-black uppercase tracking-widest">
            <span class="text-slate-400">{p.provider}</span>
            <span class="text-slate-300">{p.avg_latency_ms.toFixed(1)}ms</span>
          </div>
          <div class="h-1.5 w-full bg-white/5 rounded-full overflow-hidden">
            <div 
              class="h-full bg-gradient-to-r from-indigo-500 to-emerald-500 transition-all duration-1000" 
              style="width: {Math.min(100, (p.avg_latency_ms / 3000) * 100)}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
