<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { getAdminRuntimeStatus, getAgentsStatus, getDiagnosticsReport, getModelsStatus } from '$lib/api';
  import type { AdminRuntimeStatus, AgentsStatusReport, DiagnosticsReport, ModelProviderStatus } from '$lib/models';

  let services: Array<{ name: string; status: string; message?: string }> = [];
  let loading = true;
  let timestamp = '';
  let error: string | null = null;
  let reports: DiagnosticsReport[] = [];
  let models: ModelProviderStatus | null = null;
  let agentsStatus: AgentsStatusReport | null = null;
  let runtime: AdminRuntimeStatus | null = null;

  async function fetchSystem() {
    loading = true;
    error = null;
    const [{ data, error: diagnosticsError }, { data: modelsData }, { data: statusData }, { data: runtimeData }] = await Promise.all([
      getDiagnosticsReport(),
      getModelsStatus(),
      getAgentsStatus(),
      getAdminRuntimeStatus()
    ]);

    if (diagnosticsError) {
      error = diagnosticsError.message;
    }
    if (data) {
      services = data.checks.map((check) => ({
        name: check.name,
        status: check.status,
        message: `${check.info} (${check.duration_ms}ms)`
      }));
      timestamp = data.generated_at;
      reports = [data, ...reports].slice(0, 8);
    }
    models = modelsData || null;
    agentsStatus = statusData || null;
    runtime = runtimeData || null;
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
    {#if error}
      <div class="glass-card border-rose-500/30 bg-rose-500/5 p-8 text-sm text-slate-300 sm:col-span-2 lg:col-span-3">
        Diagnostics request failed: {error}
      </div>
    {/if}
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

  {#if reports.length > 0}
    <section class="space-y-6" in:fade>
      <div class="section-title">
        <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
        Diagnostics Run Log
      </div>
      <div class="glass-card divide-y divide-white/5 overflow-hidden">
        {#each reports as report, index}
          <div class="px-8 py-6">
            <div class="flex items-center justify-between">
              <div class="text-xs font-black uppercase tracking-[0.2em] text-white">
                Run #{reports.length - index}
              </div>
              <div class="status-pill {report.overall_status === 'healthy' ? 'status-pill-ok' : report.overall_status === 'partial' ? 'status-pill-warn' : 'status-pill-error'}">
                {report.overall_status}
              </div>
            </div>
            <div class="mt-2 text-[10px] font-bold uppercase tracking-widest text-slate-500">
              {new Date(report.generated_at).toLocaleString([], { hour12: false })}
            </div>
            <div class="mt-4 space-y-3">
              {#each report.checks as check}
                <div class="flex items-start gap-4 text-[11px]">
                  <div class="mt-1 h-1.5 w-1.5 rounded-full {check.status === 'healthy' || check.status === 'ok' ? 'bg-emerald-500' : check.status === 'partial' || check.status === 'degraded' ? 'bg-amber-500' : 'bg-rose-500'}"></div>
                  <div class="flex-1">
                    <div class="font-black uppercase tracking-widest text-white">{check.name}</div>
                    <div class="mt-1 font-medium text-slate-400">{check.info}</div>
                  </div>
                  <div class="font-black text-slate-500">{check.duration_ms}ms</div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </section>
  {/if}

  {#if models}
    <section class="space-y-6" in:fade>
      <div class="section-title">
        <div class="h-1 w-4 bg-emerald-500 rounded-full"></div>
        Providers & Models
      </div>
      <div class="glass-card p-6 space-y-4">
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {#each models.providers as provider}
            <div class="rounded-xl border border-white/10 bg-white/[0.02] px-4 py-3">
              <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">{provider.provider}</div>
              <div class="mt-2 flex items-center justify-between">
                <span class="status-pill {provider.configured ? 'status-pill-ok' : 'status-pill-warn'}">
                  {provider.configured ? 'configured' : 'missing'}
                </span>
                <span class="text-[10px] font-bold text-slate-400">{provider.via || 'n/a'}</span>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </section>
  {/if}

  {#if agentsStatus}
    <section class="space-y-6" in:fade>
      <div class="section-title">
        <div class="h-1 w-4 bg-amber-500 rounded-full"></div>
        Agent Status Diff
      </div>
      <div class="glass-card p-6 space-y-4">
        <div class="grid gap-4 sm:grid-cols-3">
          <div class="rounded-xl border border-white/10 bg-white/[0.02] px-4 py-3">
            <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Configured</div>
            <div class="mt-2 text-2xl font-black text-white">{agentsStatus.configured_count}</div>
          </div>
          <div class="rounded-xl border border-white/10 bg-white/[0.02] px-4 py-3">
            <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Live</div>
            <div class="mt-2 text-2xl font-black text-emerald-400">{agentsStatus.live_count}</div>
          </div>
          <div class="rounded-xl border border-white/10 bg-white/[0.02] px-4 py-3">
            <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Stale</div>
            <div class="mt-2 text-2xl font-black text-amber-400">{agentsStatus.stale_count}</div>
          </div>
        </div>
        <div class="text-[10px] font-bold uppercase tracking-widest text-slate-500">
          Heartbeat TTL: {agentsStatus.heartbeat_ttl_seconds}s
        </div>
      </div>
    </section>
  {/if}

  {#if runtime}
    <section class="space-y-6" in:fade>
      <div class="section-title">
        <div class="h-1 w-4 bg-indigo-500 rounded-full"></div>
        Runtime Logs
      </div>
      <div class="glass-card p-6 space-y-5">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Gateway Reachability</div>
          <span class="status-pill {runtime.gateway.status === 'healthy' ? 'status-pill-ok' : 'status-pill-warn'}">
            {runtime.gateway.status}
          </span>
        </div>
        <div class="text-[11px] text-slate-300">{runtime.gateway.message}</div>
        <div class="space-y-2">
          <div class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">Backend Log Tail</div>
          <pre class="max-h-64 overflow-auto rounded-xl border border-white/10 bg-black/40 p-3 text-[10px] text-slate-300">{runtime.backend_log_tail.join('\n') || 'No backend logs yet'}</pre>
        </div>
      </div>
    </section>
  {/if}
</div>
