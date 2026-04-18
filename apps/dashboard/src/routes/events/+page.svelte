<script lang="ts">
  import { auditLogs } from '$lib/stores';
  import { fade, fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { getEvents } from '$lib/api';
  import { onMount } from 'svelte';

  async function fetchHistory() {
    const { data } = await getEvents();
    if (data) {
        // We could merge or replace here
    }
  }

  onMount(fetchHistory);
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Audit log</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-slate-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Durable immutable record of autonomous agent operations</p>
      </div>
    </div>
  </div>

  <div class="glass-card overflow-hidden">
    <div class="bg-black/40 px-10 py-5 border-b border-white/5">
      <div class="grid grid-cols-12 text-[9px] font-black uppercase tracking-[0.2em] text-slate-500">
        <div class="col-span-1 text-center">Status</div>
        <div class="col-span-2">Authority</div>
        <div class="col-span-3">Operation</div>
        <div class="col-span-3">Entity Reference</div>
        <div class="col-span-3 text-right">Verification</div>
      </div>
    </div>

    <div class="divide-y divide-white/5">
        {#each $auditLogs as log (log.id)}
        <div
            class="px-10 py-6 grid grid-cols-12 items-center gap-6 transition-colors hover:bg-white/[0.02]"
            in:fly={{ y: 10, duration: 300 }}
            animate:flip={{ duration: 400 }}
        >
            <div class="col-span-1 flex justify-center">
                <div class="h-1.5 w-1.5 rounded-full {log.allowed ? 'bg-emerald-500 shadow-[0_0_8px_#10b981]' : 'bg-rose-500'}"></div>
            </div>
            
            <div class="col-span-2">
                <span class="text-[10px] font-black text-white uppercase tracking-wider">
                    {log.agent_id || 'SYSTEM'}
                </span>
            </div>

            <div class="col-span-3 text-[10px] font-black text-indigo-400 uppercase tracking-widest">
                {log.operation.replace('_', ' ')}
            </div>

            <div class="col-span-3 text-[10px] font-bold text-slate-500 truncate uppercase">
                {log.resource_type || '-'}: <span class="text-slate-400">{log.resource_id || '-'}</span>
            </div>

            <div class="col-span-3 text-right flex flex-col items-end gap-1">
                <div class="text-[10px] font-black tabular-nums text-slate-500 uppercase">
                    {new Date(log.created_at).toLocaleTimeString([], { hour12: false })}
                </div>
                <div class="text-[8px] font-bold text-slate-600 uppercase tracking-tighter">
                    SIG: {log.id.split('-')[1]}
                </div>
            </div>
        </div>
        {:else}
        <div class="py-32 text-center space-y-4">
            <div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">Signal void detected</div>
            <p class="text-[10px] font-bold text-slate-700 uppercase tracking-widest italic">Awaiting initial autonomous transmission...</p>
        </div>
        {/each}
    </div>
  </div>
</div>
