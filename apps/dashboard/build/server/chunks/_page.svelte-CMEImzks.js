import { s as store_get, e as ensure_array_like, d as escape_html, b as attr_class, c as stringify, g as unsubscribe_stores } from './dev-C8SO_9bI.js';
import { c as configuredAgentLabels } from './configured-agents-CMIbisL5.js';
import { a as liveAgents, s as streamEvents, b as activeTasks, c as systemState } from './stores-DfDuB3E0.js';

//#region src/routes/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		let backendStatus, platformLabel, stats;
		backendStatus = store_get($$store_subs ??= {}, "$systemState", systemState)?.system_health?.backend?.status;
		platformLabel = backendStatus === "ok" || backendStatus === "healthy" ? "NOMINAL" : backendStatus === "unknown" ? "UNKNOWN" : backendStatus ? "DEGRADED" : "UNKNOWN";
		stats = [
			{
				label: "Active Agents",
				value: store_get($$store_subs ??= {}, "$liveAgents", liveAgents).length,
				color: "text-indigo-500"
			},
			{
				label: "Queue Depth",
				value: (store_get($$store_subs ??= {}, "$systemState", systemState)?.queue_summary.pending_normal || 0) + (store_get($$store_subs ??= {}, "$systemState", systemState)?.queue_summary.pending_critical || 0),
				color: "text-amber-500"
			},
			{
				label: "Token Burn",
				value: `$${store_get($$store_subs ??= {}, "$systemState", systemState)?.queue_summary.completed || 0}`,
				color: "text-emerald-500"
			},
			{
				label: "Platform",
				value: platformLabel,
				color: platformLabel === "NOMINAL" ? "text-white" : platformLabel === "UNKNOWN" ? "text-slate-400" : "text-rose-400"
			}
		];
		$$renderer.push(`<div class="space-y-12"><div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4"><!--[-->`);
		const each_array = ensure_array_like(stats);
		for (let i = 0, $$length = each_array.length; i < $$length; i++) {
			let stat = each_array[i];
			$$renderer.push(`<div class="metric-card group"><dt class="metric-label">${escape_html(stat.label)}</dt> <dd class="mt-4 flex items-baseline justify-between"><span${attr_class(`text-4xl font-black tracking-tighter ${stringify(stat.color)}`)}>${escape_html(stat.value)}</span></dd> <div class="absolute -right-4 -bottom-4 opacity-5 group-hover:opacity-10 transition-opacity"><div class="h-24 w-24 rounded-full bg-white"></div></div></div>`);
		}
		$$renderer.push(`<!--]--></div> <div class="grid grid-cols-1 gap-12 lg:grid-cols-2"><section><div class="section-title"><div class="h-1 w-4 bg-emerald-500 rounded-full"></div> Active Intelligence</div> <div class="grid gap-4 mt-6">`);
		const each_array_1 = ensure_array_like(store_get($$store_subs ??= {}, "$liveAgents", liveAgents));
		if (each_array_1.length !== 0) {
			$$renderer.push("<!--[-->");
			for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
				let agent = each_array_1[$$index_1];
				$$renderer.push(`<div class="glass-card p-6 flex items-center justify-between group glass-card-hover"><div class="flex items-center gap-5"><div class="h-12 w-12 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center text-xs font-black text-slate-500 group-hover:border-indigo-500/50 transition-colors">${escape_html(agent.agent_id.substring(0, 2).toUpperCase())}</div> <div><div class="text-sm font-black text-white tracking-tight">${escape_html(agent.agent_id)}</div> <div class="text-[10px] text-slate-500 uppercase tracking-widest mt-0.5">${escape_html(configuredAgentLabels[agent.agent_id] || agent.model)}</div></div></div> <div class="text-right"><span class="status-pill status-pill-ok">${escape_html(agent.state)}</span> <div class="mt-2 text-[10px] font-bold text-slate-600 tabular-nums uppercase">${escape_html(agent.elapsed_seconds)}s SESSION</div></div></div>`);
			}
		} else {
			$$renderer.push("<!--[!-->");
			$$renderer.push(`<div class="glass-card bg-white/[0.02] border-dashed py-12 text-center text-xs font-bold text-slate-600 uppercase tracking-[0.2em]">No live agents online. See the Agents page for configured roster and model assignments.</div>`);
		}
		$$renderer.push(`<!--]--></div></section> <section><div class="section-title"><div class="h-1 w-4 bg-indigo-500 rounded-full"></div> Signal Stream</div> <div class="glass-card mt-6 divide-y divide-white/5 overflow-hidden">`);
		const each_array_2 = ensure_array_like(store_get($$store_subs ??= {}, "$streamEvents", streamEvents).slice(0, 8));
		if (each_array_2.length !== 0) {
			$$renderer.push("<!--[-->");
			for (let i = 0, $$length = each_array_2.length; i < $$length; i++) {
				let log = each_array_2[i];
				$$renderer.push(`<div class="px-6 py-4 flex items-center gap-4 transition-colors hover:bg-white/[0.03]"><div${attr_class(`h-1.5 w-1.5 rounded-full ${stringify(log.level === "error" ? "bg-rose-500" : log.level === "warn" ? "bg-amber-500" : "bg-emerald-500 shadow-[0_0_8px_#10b981]")}`)}></div> <div class="min-w-0 flex-1"><div class="text-[11px] font-black text-white uppercase tracking-tight"><span class="text-indigo-400">${escape_html(log.stream_connection || "SYSTEM")}</span></div> <div class="mt-0.5 truncate text-[10px] font-bold text-slate-500 uppercase tracking-tighter">${escape_html(log.summary)}</div></div> <time class="text-[9px] font-black text-slate-600 tabular-nums">${escape_html((/* @__PURE__ */ new Date(log.timestamp * 1e3)).toLocaleTimeString([], { hour12: false }))}</time></div>`);
			}
		} else {
			$$renderer.push("<!--[!-->");
			$$renderer.push(`<div class="p-12 text-center text-[10px] font-bold text-slate-600 uppercase tracking-[0.2em]">Awaiting incoming signals...</div>`);
		}
		$$renderer.push(`<!--]--></div></section></div> <section><div class="section-title text-rose-500"><div class="h-1 w-4 bg-rose-500 rounded-full"></div> Priority Objectives</div> <div class="grid gap-6 mt-6 sm:grid-cols-2 lg:grid-cols-3">`);
		const each_array_3 = ensure_array_like(store_get($$store_subs ??= {}, "$activeTasks", activeTasks));
		if (each_array_3.length !== 0) {
			$$renderer.push("<!--[-->");
			for (let $$index_3 = 0, $$length = each_array_3.length; $$index_3 < $$length; $$index_3++) {
				let task = each_array_3[$$index_3];
				$$renderer.push(`<div class="glass-card p-6 border-l-4 border-l-rose-500 group glass-card-hover"><div class="flex justify-between items-start"><h3 class="font-black text-sm text-white tracking-tight leading-tight uppercase">${escape_html(task.title)}</h3> <span class="status-pill status-pill-error ml-4">${escape_html(task.status)}</span></div> <p class="mt-4 text-xs font-medium text-slate-500 line-clamp-2 leading-relaxed">${escape_html(task.blocked_reason || "No description provided")}</p> <div class="mt-6 flex justify-between items-center border-t border-white/5 pt-4"><span class="text-[9px] font-black text-slate-600 uppercase tracking-widest">ID: ${escape_html(task.id.split("-")[1])}</span> <span class="text-[9px] font-black text-indigo-400 uppercase tracking-widest">${escape_html(task.owner_agent)}</span></div></div>`);
			}
		} else {
			$$renderer.push("<!--[!-->");
			$$renderer.push(`<div class="col-span-full glass-card bg-emerald-500/[0.02] border-emerald-500/20 py-10 text-center"><div class="text-[10px] font-black text-emerald-500 uppercase tracking-[0.3em]">System state: All objectives clear</div></div>`);
		}
		$$renderer.push(`<!--]--></div></section></div>`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-CMEImzks.js.map
