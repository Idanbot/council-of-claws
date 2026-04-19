import { a as attr, e as ensure_array_like, b as attr_class, d as escape_html, c as stringify } from './dev-C8SO_9bI.js';

//#region src/routes/tasks/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let sortedTasks;
		let tasks = [];
		let loading = true;
		sortedTasks = [...tasks].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
		$$renderer.push(`<div class="space-y-10"><div class="flex items-center justify-between border-b border-white/5 pb-10"><div><h1 class="text-3xl font-black tracking-tighter text-white uppercase">Operations</h1> <div class="mt-2 flex items-center gap-2"><div class="h-1 w-1 bg-amber-500 rounded-full"></div> <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Durable Task Registry &amp; lifecycle tracking</p></div></div> <button class="btn-primary uppercase text-[10px] tracking-widest px-6 h-10 gap-2"${attr("disabled", loading, true)}>`);
		$$renderer.push("<!--[0-->");
		$$renderer.push(`<span class="inline-block animate-spin">◌</span>`);
		$$renderer.push(`<!--]--> Synchronize</button></div> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--> `);
		if (tasks.length === 0) {
			$$renderer.push("<!--[1-->");
			$$renderer.push(`<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3"><!--[-->`);
			const each_array = ensure_array_like(Array(6));
			for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
				each_array[$$index];
				$$renderer.push(`<div class="glass-card animate-pulse h-48 bg-white/5 border-none opacity-50"></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		} else if (tasks.length === 0) {
			$$renderer.push("<!--[2-->");
			$$renderer.push(`<div class="glass-card border-dashed py-24 text-center space-y-4"><div class="text-xs font-black text-slate-600 uppercase tracking-[0.4em]">No active tasks</div> <p class="text-[10px] font-bold text-slate-700 uppercase tracking-widest">Awaiting autonomous mission assignment</p></div>`);
		} else {
			$$renderer.push("<!--[-1-->");
			$$renderer.push(`<div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3"><!--[-->`);
			const each_array_1 = ensure_array_like(sortedTasks);
			for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
				let task = each_array_1[$$index_1];
				$$renderer.push(`<div${attr_class("glass-card p-6 group relative flex flex-col justify-between border-l-4 glass-card-hover transition-all", void 0, {
					"border-l-indigo-500": task.priority === "high" || task.priority === "critical",
					"border-l-slate-600": task.priority === "normal",
					"opacity-60": task.status === "completed"
				})}><div><div class="flex items-start justify-between gap-4"><h3 class="font-black text-xs text-white uppercase tracking-tight group-hover:text-indigo-400 transition-colors leading-relaxed">${escape_html(task.title)}</h3> <span${attr_class(`status-pill ${stringify(task.status === "completed" ? "status-pill-ok" : task.status === "failed" ? "status-pill-error" : "status-pill-info")}`)}>${escape_html(task.status)}</span></div> <p class="mt-4 text-[11px] leading-relaxed text-slate-500 font-medium line-clamp-3 min-h-[3rem]">${escape_html(task.blocked_reason || "Autonomous execution in progress...")}</p></div> <div class="mt-8 flex items-center justify-between border-t border-white/5 pt-4"><div class="flex items-center gap-2"><div class="h-1.5 w-1.5 rounded-full bg-indigo-500/50"></div> <span class="text-[9px] text-indigo-400 font-black uppercase tracking-widest">${escape_html(task.owner_agent)}</span></div> <time class="text-[9px] text-slate-600 font-black tabular-nums">${escape_html(new Date(task.created_at).toLocaleDateString([], {
					month: "short",
					day: "numeric"
				}).toUpperCase())}</time></div></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		}
		$$renderer.push(`<!--]--></div>`);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-CN6ThC9E.js.map
