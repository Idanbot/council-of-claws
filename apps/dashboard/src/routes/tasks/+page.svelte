<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { getTasks } from '$lib/api';
  import type { Task } from '$lib/models';

  let tasks: Task[] = [];
  let loading = true;
  let error: string | null = null;

  async function fetchTasks() {
    loading = true;
    const { data, error: apiError } = await getTasks();
    if (apiError) {
      error = apiError.message;
    } else if (data) {
      tasks = data;
    }
    loading = false;
  }

  onMount(fetchTasks);

  $: sortedTasks = [...tasks].sort((a, b) => 
    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );
</script>

<div class="space-y-10">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Operations</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-amber-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Durable Task Registry & lifecycle tracking</p>
      </div>
    </div>
    <button
      on:click={fetchTasks}
      class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10 gap-2"
      disabled={loading}
    >
      {#if loading}
        <span class="inline-block animate-spin">◌</span>
      {/if}
      Synchronize
    </button>
  </div>

  {#if error}
    <div class="glass-card border-rose-500/30 bg-rose-500/5 p-12 text-center" in:fade>
      <div class="text-rose-500 text-xs font-black uppercase tracking-widest mb-4 flex items-center justify-center gap-2">
          <div class="h-1 w-1 bg-rose-500 rounded-full"></div>
          Critical Fetch Error
      </div>
      <p class="text-slate-400 text-sm font-medium">{error}</p>
      <button on:click={fetchTasks} class="mt-8 btn-secondary px-8">Retry Protocol</button>
    </div>
  {:else if loading && tasks.length === 0}
    <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
      {#each Array(6) as _}
        <div class="glass-card animate-pulse h-48 bg-white/5 border-none opacity-50"></div>
      {/each}
    </div>
  {:else if tasks.length === 0}
    <div class="glass-card border-dashed py-24 text-center space-y-4" in:fade>
      <div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">No active tasks</div>
      <p class="text-[10px] font-bold text-slate-700 uppercase tracking-widest">Awaiting autonomous mission assignment</p>
    </div>
  {:else}
    <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
      {#each sortedTasks as task (task.id)}
        <div
          class="glass-card p-6 group relative flex flex-col justify-between border-l-4 glass-card-hover transition-all"
          class:border-l-indigo-500={task.priority === 'high' || task.priority === 'critical'}
          class:border-l-slate-600={task.priority === 'normal'}
          class:opacity-60={task.status === 'completed'}
          in:fly={{ y: 20 }}
          animate:flip={{ duration: 400 }}
        >
          <div>
            <div class="flex items-start justify-between gap-4">
              <h3 class="font-black text-xs text-white uppercase tracking-tight group-hover:text-indigo-400 transition-colors leading-relaxed">
                {task.title}
              </h3>
              <span class="status-pill {task.status === 'completed' ? 'status-pill-ok' : task.status === 'failed' ? 'status-pill-error' : 'status-pill-info'}">
                {task.status}
              </span>
            </div>
            <p class="mt-4 text-[11px] leading-relaxed text-slate-500 font-medium line-clamp-3 min-h-[3rem]">
              {task.blocked_reason || 'Autonomous execution in progress...'}
            </p>
          </div>

          <div class="mt-8 flex items-center justify-between border-t border-white/5 pt-4">
            <div class="flex items-center gap-2">
              <div class="h-1.5 w-1.5 rounded-full bg-indigo-500/50"></div>
              <span class="text-[9px] text-indigo-400 font-black uppercase tracking-widest">{task.owner_agent}</span>
            </div>
            <time class="text-[9px] text-slate-600 font-black tabular-nums">
              {new Date(task.created_at).toLocaleDateString([], { month: 'short', day: 'numeric' }).toUpperCase()}
            </time>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
