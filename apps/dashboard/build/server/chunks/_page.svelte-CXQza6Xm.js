import { a as attr, d as escape_html, e as ensure_array_like, ab as attr_style, c as stringify } from './dev-C8SO_9bI.js';
import { c as configuredAgentLabels } from './configured-agents-CMIbisL5.js';

//#region src/routes/agents/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let liveAgents = [];
		let configured = [];
		$$renderer.push(`<div class="space-y-10"><div class="flex items-center justify-between border-b border-white/5 pb-10"><div><h1 class="text-3xl font-black tracking-tighter text-white uppercase">Intelligence</h1> <div class="mt-2 flex items-center gap-2"><div class="h-1 w-1 bg-emerald-500 rounded-full"></div> <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Registered autonomous agents and model assignments</p></div></div> <button class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10"${attr("disabled", true, true)}>${escape_html("Refreshing…")}</button></div> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> <div class="space-y-6"><div class="section-title"><div class="h-1 w-4 bg-emerald-500 rounded-full"></div> Live Telemetry</div> `);
		if (liveAgents.length === 0) {
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="grid gap-8 sm:grid-cols-2"><!--[-->`);
			const each_array = ensure_array_like(Array(4));
			for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
				each_array[$$index];
				$$renderer.push(`<div class="glass-card animate-pulse h-48 bg-white/5 border-none"></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		} else {
			$$renderer.push("<!--[-1-->");
			if (liveAgents.length === 0) {
				$$renderer.push("<!--[0-->");
				$$renderer.push(`<div class="glass-card border-amber-500/20 bg-amber-500/5 p-8 text-sm text-slate-300">No live heartbeats yet. The configured roster below is still valid, but runtime telemetry will only appear after agents start sending heartbeats.</div>`);
			} else {
				$$renderer.push("<!--[-1-->");
				$$renderer.push(`<div class="grid gap-8 sm:grid-cols-2"><!--[-->`);
				const each_array_1 = ensure_array_like(liveAgents);
				for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
					let agent = each_array_1[$$index_1];
					$$renderer.push(`<div class="glass-card p-8 group glass-card-hover transition-all"><div class="flex items-center justify-between"><div class="flex items-center gap-6"><div class="h-16 w-16 rounded-2xl bg-indigo-500/10 flex items-center justify-center text-xs font-black text-indigo-400 border border-indigo-500/20 group-hover:scale-105 group-hover:bg-indigo-500/20 transition-all">${escape_html(agent.agent_id.substring(0, 2).toUpperCase())}</div> <div><h3 class="text-lg font-black text-white tracking-tight uppercase">${escape_html(agent.agent_id)}</h3> <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">${escape_html(configuredAgentLabels[agent.agent_id] || agent.model)}</p></div></div> <div class="flex flex-col items-end"><span class="status-pill status-pill-ok">${escape_html(agent.state)}</span></div></div> <div class="mt-10 grid grid-cols-2 gap-6 border-t border-white/5 pt-6"><div><p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Active Objective</p> <p class="mt-2 text-xs font-bold text-slate-300 truncate">${escape_html(agent.current_task_id || "STANDBY")}</p></div> <div class="text-right"><p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Primary Model</p> <p class="mt-2 text-xs font-bold text-slate-300 tabular-nums uppercase">${escape_html(agent.model)}</p></div></div> <div class="mt-8 flex items-center gap-3"><div class="h-1 flex-1 rounded-full bg-white/5 overflow-hidden"><div class="h-full bg-emerald-500 shadow-[0_0_10px_#10b981] animate-pulse"${attr_style(`width: ${stringify(agent.elapsed_seconds > 0 ? "100%" : "35%")}`)}></div></div> <span class="text-[8px] font-black text-emerald-500 uppercase tracking-widest">${escape_html(agent.elapsed_seconds > 0 ? "Live" : "Configured")}</span></div></div>`);
				}
				$$renderer.push(`<!--]--></div>`);
			}
			$$renderer.push(`<!--]-->`);
		}
		$$renderer.push(`<!--]--></div> <div class="space-y-6"><div class="section-title"><div class="h-1 w-4 bg-indigo-500 rounded-full"></div> Configured Roster</div> <div class="grid gap-8 sm:grid-cols-2"><!--[-->`);
		const each_array_2 = ensure_array_like(configured);
		for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
			let agent = each_array_2[$$index_2];
			$$renderer.push(`<div class="glass-card p-8 group glass-card-hover transition-all"><div class="flex items-center justify-between"><div class="flex items-center gap-6"><div class="h-16 w-16 rounded-2xl bg-indigo-500/10 flex items-center justify-center text-xs font-black text-indigo-400 border border-indigo-500/20">${escape_html(agent.agent_id.substring(0, 2).toUpperCase())}</div> <div><h3 class="text-lg font-black text-white tracking-tight uppercase">${escape_html(agent.agent_id)}</h3> <p class="text-[9px] uppercase tracking-[0.2em] text-slate-500 font-bold mt-1">${escape_html(agent.role)}</p></div></div> <span class="status-pill status-pill-info">Configured</span></div> <div class="mt-8 grid gap-4 border-t border-white/5 pt-6"><div><p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Primary Model</p> <p class="mt-2 text-xs font-bold text-slate-300 uppercase">${escape_html(agent.primary_model)}</p></div> <div><p class="text-[9px] uppercase font-black text-slate-500 tracking-widest">Fallbacks</p> <p class="mt-2 text-xs font-bold text-slate-300 uppercase">${escape_html(agent.fallbacks.join(" -> ") || "None")}</p></div></div></div>`);
		}
		$$renderer.push(`<!--]--></div></div></div>`);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-CXQza6Xm.js.map
