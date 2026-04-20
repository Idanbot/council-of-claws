<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { createWebSocket } from './api';

  export let agentId: string | null = null;
  export let maxLines = 100;

  interface LogEntry {
    agent_id: string;
    level: string;
    message: string;
    target?: string;
    timestamp: string;
  }

  let logs: LogEntry[] = [];
  let socket: WebSocket;
  let scrollContainer: HTMLDivElement;
  let autoScroll = true;
  let filter = '';

  $: filteredLogs = logs.filter(log => {
    if (agentId && log.agent_id !== agentId) return false;
    if (filter && !log.message.toLowerCase().includes(filter.toLowerCase())) return false;
    return true;
  });

  function handleMessage(data: any) {
    if (data.event_type === 'log') {
      logs = [...logs, data].slice(-maxLines);
      if (autoScroll && scrollContainer) {
        setTimeout(() => {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }, 10);
      }
    }
  }

  onMount(() => {
    socket = createWebSocket(handleMessage);
  });

  onDestroy(() => {
    if (socket) socket.close();
  });

  const levelColors: Record<string, string> = {
    trace: 'text-slate-500',
    debug: 'text-blue-400',
    info: 'text-emerald-400',
    warn: 'text-amber-400',
    error: 'text-rose-400'
  };
</script>

<div class="glass-card flex flex-col h-[400px] overflow-hidden border-white/5 bg-slate-950/50">
  <div class="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-white/5">
    <div class="flex items-center gap-3">
      <div class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></div>
      <h3 class="text-[10px] font-black uppercase tracking-widest text-slate-400">Live Console</h3>
    </div>
    <div class="flex items-center gap-4">
      <input 
        type="text" 
        bind:value={filter} 
        placeholder="Filter logs..." 
        class="bg-white/5 border border-white/10 rounded px-3 py-1 text-[10px] text-slate-300 focus:outline-none focus:border-indigo-500/50 w-48"
      />
      <button 
        on:click={() => logs = []}
        class="text-[9px] font-bold text-slate-500 hover:text-slate-300 uppercase tracking-tighter"
      >
        Clear
      </button>
      <label class="flex items-center gap-2 cursor-pointer">
        <input type="checkbox" bind:checked={autoScroll} class="sr-only peer" />
        <div class="w-6 h-3 bg-white/10 rounded-full peer peer-checked:bg-emerald-500/50 relative transition-all">
          <div class="absolute top-0.5 left-0.5 w-2 h-2 bg-slate-400 rounded-full peer-checked:translate-x-3 transition-all"></div>
        </div>
        <span class="text-[8px] font-bold text-slate-500 uppercase">Auto-scroll</span>
      </label>
    </div>
  </div>

  <div 
    bind:this={scrollContainer}
    class="flex-1 overflow-y-auto p-6 font-mono text-[11px] leading-relaxed selection:bg-indigo-500/30"
  >
    {#if filteredLogs.length === 0}
      <div class="h-full flex items-center justify-center text-slate-600 italic">
        Waiting for transmission…
      </div>
    {:else}
      {#each filteredLogs as log}
        <div class="flex gap-4 mb-1 group" in:fade={{ duration: 100 }}>
          <span class="text-slate-700 shrink-0">{new Date(log.timestamp).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span>
          <span class="w-16 shrink-0 font-black uppercase {levelColors[log.level.toLowerCase()] || 'text-slate-400'}">[{log.level}]</span>
          {#if !agentId}
            <span class="text-indigo-400 shrink-0 font-bold">@{log.agent_id}</span>
          {/if}
          <span class="text-slate-300 break-all">{log.message}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>
