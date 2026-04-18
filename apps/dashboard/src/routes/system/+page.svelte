<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { getHealthServices } from '$lib/api';

  let services: Array<{ name: string; status: string; message?: string }> = [];
  let loading = true;
  let timestamp = '';

  async function fetchSystem() {
    loading = true;
    const { data } = await getHealthServices();
    if (data) {
      services = data.services;
      timestamp = data.timestamp;
    }
    loading = false;
  }

  onMount(fetchSystem);
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Diagnostics</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-rose-500 rounded-full shadow-[0_0_8px_#f43f5e]"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Low-level platform integrity and infrastructure health</p>
      </div>
    </div>
    <button on:click={fetchSystem} class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10" disabled={loading}>
      {loading ? 'Analyzing...' : 'Run Diagnostics'}
    </button>
  </div>

  <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
    {#each services as service}
      <div class="glass-card p-10 flex flex-col justify-between group glass-card-hover" in:scale={{ start: 0.98 }}>
        <div class="flex items-start justify-between">
          <div class="space-y-2">
            <h3 class="text-sm font-black text-white uppercase tracking-widest group-hover:text-indigo-400 transition-colors">{service.name}</h3>
            <p class="text-[11px] font-medium text-slate-500 leading-relaxed uppercase tracking-tighter">{service.message || 'Operational stability verified.'}</p>
          </div>
          <span class="status-pill {service.status === 'healthy' || service.status === 'ok' ? 'status-pill-ok' : 'status-pill-error'}">
            {service.status}
          </span>
        </div>

        <div class="mt-12 flex items-center gap-4">
          <div class="h-1 flex-1 rounded-full bg-white/5 overflow-hidden">
            <div class="h-full bg-indigo-500 opacity-20 group-hover:opacity-40 transition-opacity" style="width: 100%"></div>
          </div>
          <span class="text-[9px] font-black text-slate-600 uppercase tracking-widest font-mono">STABLE</span>
        </div>
      </div>
    {:else}
      {#each Array(6) as _}
        <div class="glass-card h-48 animate-pulse bg-white/5 border-none"></div>
      {/each}
    {/each}
  </div>

  {#if timestamp}
    <div class="text-center pt-20" in:fade>
       <div class="inline-flex items-center gap-3 px-6 py-2 rounded-full border border-white/5 bg-black/20">
           <div class="h-1 w-1 bg-slate-600 rounded-full"></div>
           <p class="text-[9px] font-black uppercase tracking-[0.2em] text-slate-500">
             Verification Authority Signature: {new Date(timestamp).toLocaleTimeString([], { hour12: false })}
           </p>
       </div>
    </div>
  {/if}
</div>
