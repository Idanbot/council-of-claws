import { e as ensure_array_like, b as attr_class, c as stringify, d as escape_html } from './dev-C8SO_9bI.js';

//#region src/routes/events/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let auditLogs = [];
		$$renderer.push(`<div class="space-y-10"><div class="flex items-center justify-between border-b border-white/5 pb-10"><div><h1 class="text-3xl font-black tracking-tighter text-white uppercase">Audit log</h1> <div class="mt-2 flex items-center gap-2"><div class="h-1 w-1 bg-slate-500 rounded-full"></div> <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Durable immutable record of autonomous agent operations</p></div></div></div> <div class="glass-card overflow-hidden"><div class="bg-black/40 px-10 py-5 border-b border-white/5"><div class="grid grid-cols-12 text-[9px] font-black uppercase tracking-[0.2em] text-slate-500"><div class="col-span-1 text-center">Status</div> <div class="col-span-2">Authority</div> <div class="col-span-3">Operation</div> <div class="col-span-3">Entity Reference</div> <div class="col-span-3 text-right">Verification</div></div></div> <div class="divide-y divide-white/5">`);
		if (auditLogs.length === 0) {
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="py-32 text-center space-y-4"><div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">Loading audit stream</div></div>`);
		} else {
			$$renderer.push("<!--[-1-->");
			const each_array = ensure_array_like(auditLogs);
			if (each_array.length !== 0) {
				$$renderer.push("<!--[-->");
				for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
					let log = each_array[$$index];
					$$renderer.push(`<div class="px-10 py-6 grid grid-cols-12 items-center gap-6 transition-colors hover:bg-white/[0.02]"><div class="col-span-1 flex justify-center"><div${attr_class(`h-1.5 w-1.5 rounded-full ${stringify(log.allowed ? "bg-emerald-500 shadow-[0_0_8px_#10b981]" : "bg-rose-500")}`)}></div></div> <div class="col-span-2"><span class="text-[10px] font-black text-white uppercase tracking-wider">${escape_html(log.agent_id || "SYSTEM")}</span></div> <div class="col-span-3 text-[10px] font-black text-indigo-400 uppercase tracking-widest">${escape_html(log.operation.replace("_", " "))}</div> <div class="col-span-3 text-[10px] font-bold text-slate-500 truncate uppercase">${escape_html(log.resource_type || "-")}: <span class="text-slate-400">${escape_html(log.resource_id || "-")}</span></div> <div class="col-span-3 text-right flex flex-col items-end gap-1"><div class="text-[10px] font-black tabular-nums text-slate-500 uppercase">${escape_html(new Date(log.created_at).toLocaleTimeString([], { hour12: false }))}</div> <div class="text-[8px] font-bold text-slate-600 uppercase tracking-tighter">SIG: ${escape_html(log.id.split("-")[1])}</div></div></div>`);
				}
			} else {
				$$renderer.push("<!--[!-->");
				$$renderer.push(`<div class="py-32 text-center space-y-4"><div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">Signal void detected</div> <p class="text-[10px] font-bold text-slate-700 uppercase tracking-widest italic">Awaiting initial autonomous transmission...</p></div>`);
			}
			$$renderer.push(`<!--]-->`);
		}
		$$renderer.push(`<!--]--></div></div></div>`);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-BL7Q3Uj1.js.map
