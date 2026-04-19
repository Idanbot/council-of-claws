import { a as attr, d as escape_html, e as ensure_array_like, b as attr_class, c as stringify } from './dev-C8SO_9bI.js';

//#region src/routes/system/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let services = [];
		let loading = true;
		let reports = [];
		$$renderer.push(`<div class="space-y-10"><div class="flex items-center justify-between border-b border-white/5 pb-10"><div><h1 class="text-3xl font-black tracking-tighter text-white uppercase">Diagnostics</h1> <div class="mt-2 flex items-center gap-2"><div class="h-1 w-1 bg-rose-500 rounded-full shadow-[0_0_8px_#f43f5e]"></div> <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Low-level platform integrity and infrastructure health</p></div></div> <button class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10"${attr("disabled", loading, true)}>${escape_html("Analyzing...")}</button></div> <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">`);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		const each_array = ensure_array_like(services);
		if (each_array.length !== 0) {
			$$renderer.push("<!--[-->");
			for (let $$index_1 = 0, $$length = each_array.length; $$index_1 < $$length; $$index_1++) {
				let service = each_array[$$index_1];
				$$renderer.push(`<div class="glass-card p-10 flex flex-col justify-between group glass-card-hover"><div class="flex items-start justify-between"><div class="space-y-2"><h3 class="text-sm font-black text-white uppercase tracking-widest group-hover:text-indigo-400 transition-colors">${escape_html(service.name)}</h3> <p class="text-[11px] font-medium text-slate-500 leading-relaxed uppercase tracking-tighter">${escape_html(service.message || "Operational stability verified.")}</p></div> <span${attr_class(`status-pill ${stringify(service.status === "healthy" || service.status === "ok" ? "status-pill-ok" : "status-pill-error")}`)}>${escape_html(service.status)}</span></div> <div class="mt-12 flex items-center gap-4"><div class="h-1 flex-1 rounded-full bg-white/5 overflow-hidden"><div class="h-full bg-indigo-500 opacity-20 group-hover:opacity-40 transition-opacity" style="width: 100%"></div></div> <span class="text-[9px] font-black text-slate-600 uppercase tracking-widest font-mono">STABLE</span></div></div>`);
			}
		} else {
			$$renderer.push("<!--[!-->");
			$$renderer.push(`<!--[-->`);
			const each_array_1 = ensure_array_like(Array(6));
			for (let $$index = 0, $$length = each_array_1.length; $$index < $$length; $$index++) {
				each_array_1[$$index];
				$$renderer.push(`<div class="glass-card h-48 animate-pulse bg-white/5 border-none"></div>`);
			}
			$$renderer.push(`<!--]-->`);
		}
		$$renderer.push(`<!--]--></div> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		if (reports.length > 0) {
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<section class="space-y-6"><div class="section-title"><div class="h-1 w-4 bg-indigo-500 rounded-full"></div> Diagnostics Run Log</div> <div class="glass-card divide-y divide-white/5 overflow-hidden"><!--[-->`);
			const each_array_2 = ensure_array_like(reports);
			for (let index = 0, $$length = each_array_2.length; index < $$length; index++) {
				let report = each_array_2[index];
				$$renderer.push(`<div class="px-8 py-6"><div class="flex items-center justify-between"><div class="text-xs font-black uppercase tracking-[0.2em] text-white">Run #${escape_html(reports.length - index)}</div> <div${attr_class(`status-pill ${stringify(report.overall_status === "healthy" ? "status-pill-ok" : report.overall_status === "partial" ? "status-pill-warn" : "status-pill-error")}`)}>${escape_html(report.overall_status)}</div></div> <div class="mt-2 text-[10px] font-bold uppercase tracking-widest text-slate-500">${escape_html(new Date(report.generated_at).toLocaleString([], { hour12: false }))}</div> <div class="mt-4 space-y-3"><!--[-->`);
				const each_array_3 = ensure_array_like(report.checks);
				for (let $$index_2 = 0, $$length = each_array_3.length; $$index_2 < $$length; $$index_2++) {
					let check = each_array_3[$$index_2];
					$$renderer.push(`<div class="flex items-start gap-4 text-[11px]"><div${attr_class(`mt-1 h-1.5 w-1.5 rounded-full ${stringify(check.status === "healthy" || check.status === "ok" ? "bg-emerald-500" : check.status === "partial" || check.status === "degraded" ? "bg-amber-500" : "bg-rose-500")}`)}></div> <div class="flex-1"><div class="font-black uppercase tracking-widest text-white">${escape_html(check.name)}</div> <div class="mt-1 font-medium text-slate-400">${escape_html(check.info)}</div></div> <div class="font-black text-slate-500">${escape_html(check.duration_ms)}ms</div></div>`);
				}
				$$renderer.push(`<!--]--></div></div>`);
			}
			$$renderer.push(`<!--]--></div></section>`);
		} else $$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--></div>`);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-CfM_OsQM.js.map
