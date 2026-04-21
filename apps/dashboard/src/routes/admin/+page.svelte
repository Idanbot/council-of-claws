<script lang="ts">
  import { onMount } from 'svelte';
  import { getSkills, getAdminConfig, updateAdminConfig, reloadAdminConfig } from '$lib/api';
  import type { SkillDefinition } from '$lib/models';
  import { fade } from 'svelte/transition';

  let skills: SkillDefinition[] = [];
  let rawConfig = '';
  let loading = true;
  let saving = false;
  let message = '';
  let error = '';

  async function loadData() {
    loading = true;
    const [skillsRes, configRes] = await Promise.all([getSkills(), getAdminConfig()]);

    if (skillsRes.data) skills = skillsRes.data;
    if (configRes.data) rawConfig = configRes.data.content;

    loading = false;
  }

  async function handleSave() {
    saving = true;
    message = '';
    error = '';

    const { error: apiError } = await updateAdminConfig(rawConfig);
    if (apiError) {
      error = apiError.message;
    } else {
      message = 'Configuration updated successfully.';
      setTimeout(() => message = '', 3000);
    }
    saving = false;
  }

  async function handleReload() {
    message = 'Reloading config...';
    await reloadAdminConfig();
    await loadData();
    message = 'Registry refreshed.';
    setTimeout(() => message = '', 3000);
  }

  onMount(loadData);
</script>

<div class="space-y-10 pb-20">
  <div class="flex items-center justify-between border-b border-white/5 pb-10">
    <div>
      <h1 class="text-3xl font-black tracking-tighter text-white uppercase">Admin Control</h1>
      <div class="mt-2 flex items-center gap-2">
          <div class="h-1 w-1 bg-rose-500 rounded-full"></div>
          <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Manage registry, skills, and agent configurations</p>
      </div>
    </div>
    <div class="flex gap-4">
      <button on:click={handleReload} class="btn-secondary uppercase text-[10px] tracking-widest px-6 h-10">
        Reload Registry
      </button>
      <button on:click={handleSave} class="btn-primary uppercase text-[10px] tracking-widest px-8 h-10" disabled={saving}>
        {saving ? 'Saving...' : 'Save Config'}
      </button>
    </div>
  </div>

  {#if loading}
    <div class="h-64 flex items-center justify-center text-slate-500 italic animate-pulse uppercase tracking-widest text-xs">
      Initialising control plane…
    </div>
  {:else}
    <div class="grid gap-10 lg:grid-cols-2">
      <!-- Skills Registry -->
      <section class="space-y-6">
        <div class="section-title">
          <div class="h-1 w-4 bg-emerald-500 rounded-full"></div>
          Dynamic Skill Registry
        </div>
        <div class="grid gap-4">
          {#each skills as skill}
            <div class="glass-card p-6 border-white/5 bg-white/2 hover:bg-white/[0.04] transition-all group">
              <div class="flex items-center justify-between mb-2">
                <h4 class="text-[11px] font-black uppercase tracking-widest text-indigo-400">{skill.name}</h4>
                <span class="text-[8px] font-bold text-slate-600 bg-white/5 px-2 py-0.5 rounded uppercase">{skill.id}</span>
              </div>
              <p class="text-[10px] leading-relaxed text-slate-400 group-hover:text-slate-300 transition-colors">
                {skill.description}
              </p>
            </div>
          {/each}
        </div>
      </section>

      <!-- Config Editor -->
      <section class="space-y-6">
        <div class="section-title">
          <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
          OpenClaw Configuration (JSON5)
        </div>

        {#if message}
          <div in:fade class="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded text-[10px] font-bold text-emerald-400 uppercase tracking-wide">
            {message}
          </div>
        {/if}

        {#if error}
          <div in:fade class="p-4 bg-rose-500/10 border border-rose-500/30 rounded text-[10px] font-bold text-rose-400 uppercase tracking-wide">
            {error}
          </div>
        {/if}

        <div class="glass-card overflow-hidden border-white/10 bg-slate-950 shadow-2xl shadow-black/50">
          <textarea
            bind:value={rawConfig}
            spellcheck="false"
            class="w-full h-[600px] bg-transparent p-8 font-mono text-[11px] text-indigo-100/80 leading-relaxed focus:outline-none selection:bg-indigo-500/30 resize-none"
          ></textarea>
        </div>
        <p class="text-[9px] text-slate-600 italic">
          Caution: Modifying openclaw.json5 directly affects agent behavior and gateway routing.
          Use 'Save Config' followed by 'Reload Registry' to apply changes.
        </p>
      </section>
    </div>
  {/if}
</div>
