<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { fade, fly, slide } from 'svelte/transition';
  import { initRealtime, infrastructureOnline, transportStatus, lastRefreshed, refreshData, refreshing } from '$lib/stores';

  const navItems = [
    { href: '/', label: 'Overview', dot: 'bg-indigo-500' },
    { href: '/agents', label: 'Agents', dot: 'bg-emerald-500' },
    { href: '/tasks', label: 'Tasks', dot: 'bg-amber-500' },
    { href: '/council', label: 'Council', dot: 'bg-purple-500' },
    { href: '/usage', label: 'Usage', dot: 'bg-blue-500' },
    { href: '/events', label: 'Audit Log', dot: 'bg-slate-500' },
    { href: '/system', label: 'System', dot: 'bg-rose-500' }
  ];

  let mobileOpen = false;

  onMount(() => {
    const cleanup = initRealtime();
    refreshData();
    const interval = setInterval(refreshData, 30000);

    return () => {
      if (cleanup) cleanup();
      clearInterval(interval);
    };
  });

  $: currentRoute = navItems.find(i => i.href === $page.url.pathname);
</script>

<svelte:head>
  <title>Council of Claws</title>
</svelte:head>

<div class="flex min-h-screen text-slate-100">
  <!-- Desktop Sidebar -->
  <aside class="hidden w-72 glass-sidebar md:flex md:flex-col fixed h-full z-40">
    <div class="flex h-20 items-center px-8 border-b border-white/5">
      <a href="/" class="text-xs font-black uppercase tracking-[0.4em] text-white">
        Council of <span class="text-indigo-500">Claws</span>
      </a>
    </div>

    <nav class="flex-1 space-y-1.5 px-4 py-8">
      {#each navItems as item}
        <a
          href={item.href}
          class="group flex items-center rounded-xl px-4 py-3 text-xs font-black uppercase tracking-widest transition-all duration-300
                 { $page.url.pathname === item.href
                   ? 'bg-white/10 text-white shadow-lg'
                   : 'text-slate-500 hover:text-slate-200 hover:bg-white/5' }"
        >
          <span class="mr-4 h-1.5 w-1.5 rounded-full {item.dot} shadow-[0_0_8px_rgba(0,0,0,0.5)] group-hover:scale-150 transition-transform"></span>
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="p-6 border-t border-white/5 bg-black/20">
      <div class="flex flex-col gap-4">
        <div class="flex items-center justify-between">
          <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Infrastructure</span>
          <div class="h-2 w-2 rounded-full animate-pulse { $infrastructureOnline ? 'bg-emerald-500 shadow-[0_0_10px_#10b981]' : 'bg-rose-500 shadow-[0_0_10px_#f43f5e]' }"></div>
        </div>
        <div class="flex flex-col gap-1">
            <span class="text-[9px] font-bold text-slate-600 uppercase tracking-tighter">Stream: {$transportStatus}</span>
            <span class="text-[9px] font-bold text-slate-600 uppercase tracking-tighter">Updated: {$lastRefreshed.toLocaleTimeString()}</span>
        </div>
      </div>
    </div>
  </aside>

  <!-- Main Content Area -->
  <div class="flex flex-1 flex-col md:pl-72">
    <header class="glass-header h-20 flex items-center justify-between px-8">
      <div class="flex items-center gap-4">
        <button
          class="md:hidden btn-secondary p-2"
          on:click={() => (mobileOpen = !mobileOpen)}
        >
          <span class="text-xl">☰</span>
        </button>
        <h1 class="text-sm font-black uppercase tracking-[0.2em] text-white" in:fade>
          {currentRoute?.label || 'Platform'}
        </h1>
      </div>

      <div class="flex items-center gap-3">
        <a href="/gateway" class="btn-secondary text-[10px] uppercase tracking-widest px-4 h-9 inline-flex items-center">
          Open Gateway
        </a>
        <button on:click={refreshData} class="btn-secondary text-[10px] uppercase tracking-widest px-4 h-9 min-w-[8rem]" disabled={$refreshing}>
            {$refreshing ? 'Syncing…' : 'Synchronize'}
        </button>
      </div>
    </header>

    <main class="flex-1 p-8 overflow-x-hidden">
      <div class="mx-auto max-w-6xl">
        <slot />
      </div>
    </main>
  </div>

  <!-- Mobile Overlay -->
  {#if mobileOpen}
    <button
      class="fixed inset-0 z-50 bg-black/80 backdrop-blur-md md:hidden w-full h-full border-none outline-none"
      on:click={() => (mobileOpen = false)}
      transition:fade
      aria-label="Close menu"
    ></button>
    <aside
      class="fixed inset-y-0 left-0 z-[60] w-80 bg-slate-950 border-r border-white/10 md:hidden flex flex-col"
      transition:slide={{ axis: 'x' }}
    >
      <div class="h-20 flex items-center px-8 border-b border-white/5">
        <a href="/" class="text-xs font-black uppercase tracking-[0.4em]">Council of Claws</a>
      </div>
      <nav class="flex-1 px-4 py-8 space-y-2">
        {#each navItems as item}
          <a
            href={item.href}
            class="flex items-center rounded-xl px-4 py-4 text-xs font-black uppercase tracking-widest border border-transparent"
            class:bg-indigo-500={ $page.url.pathname === item.href }
            on:click={() => (mobileOpen = false)}
          >
            <span class="mr-4 h-2 w-2 rounded-full {item.dot}"></span>
            {item.label}
          </a>
        {/each}
      </nav>
    </aside>
  {/if}
</div>
