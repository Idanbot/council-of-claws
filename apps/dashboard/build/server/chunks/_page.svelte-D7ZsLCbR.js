import { e as ensure_array_like } from './dev-C8SO_9bI.js';

//#region src/routes/usage/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		$$renderer.push(`<div class="space-y-10"><div class="flex items-center justify-between border-b border-white/5 pb-10"><div><h1 class="text-3xl font-black tracking-tighter text-white uppercase">Economics</h1> <div class="mt-2 flex items-center gap-2"><div class="h-1 w-1 bg-indigo-500 rounded-full"></div> <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Total token throughput and estimated operational overhead</p></div></div></div> `);
		{
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="grid gap-8 sm:grid-cols-2 lg:grid-cols-3"><!--[-->`);
			const each_array = ensure_array_like(Array(3));
			for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
				each_array[$$index];
				$$renderer.push(`<div class="glass-card animate-pulse h-64 bg-white/5 border-none"></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		}
		$$renderer.push(`<!--]--></div>`);
	});
}

export { _page as default };
//# sourceMappingURL=_page.svelte-D7ZsLCbR.js.map
