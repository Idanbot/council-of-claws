import { s as store_get, h as head, e as ensure_array_like, a as attr, b as attr_class, c as stringify, d as escape_html, f as slot, g as unsubscribe_stores, i as getContext } from './dev-C8SO_9bI.js';
import './client-DhOBbMpb.js';
import { i as infrastructureOnline, t as transportStatus, l as lastRefreshed, r as refreshing } from './stores-DfDuB3E0.js';
import './internal-Bvy5PcMZ.js';
import './index-DBqjc0Yf.js';
import './configured-agents-CMIbisL5.js';

//#region node_modules/@sveltejs/kit/src/runtime/app/stores.js
/**
* A function that returns all of the contextual stores. On the server, this must be called during component initialization.
* Only use this if you need to defer store subscription until after the component has mounted, for some reason.
*
* @deprecated Use `$app/state` instead (requires Svelte 5, [see docs for more info](https://svelte.dev/docs/kit/migrating-to-sveltekit-2#SvelteKit-2.12:-$app-stores-deprecated))
*/
var getStores = () => {
	const stores$1 = getContext("__svelte__");
	return {
		page: { subscribe: stores$1.page.subscribe },
		navigating: { subscribe: stores$1.navigating.subscribe },
		updated: stores$1.updated
	};
};
/**
* A readable store whose value contains page data.
*
* On the server, this store can only be subscribed to during component initialization. In the browser, it can be subscribed to at any time.
*
* @deprecated Use `page` from `$app/state` instead (requires Svelte 5, [see docs for more info](https://svelte.dev/docs/kit/migrating-to-sveltekit-2#SvelteKit-2.12:-$app-stores-deprecated))
* @type {import('svelte/store').Readable<import('@sveltejs/kit').Page>}
*/
var page = { subscribe(fn) {
	return getStores().page.subscribe(fn);
} };
//#endregion
//#region src/routes/+layout.svelte
function _layout($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		let currentRoute;
		const navItems = [
			{
				href: "/",
				label: "Overview",
				dot: "bg-indigo-500"
			},
			{
				href: "/agents",
				label: "Agents",
				dot: "bg-emerald-500"
			},
			{
				href: "/tasks",
				label: "Tasks",
				dot: "bg-amber-500"
			},
			{
				href: "/council",
				label: "Council",
				dot: "bg-purple-500"
			},
			{
				href: "/usage",
				label: "Usage",
				dot: "bg-blue-500"
			},
			{
				href: "/events",
				label: "Audit Log",
				dot: "bg-slate-500"
			},
			{
				href: "/system",
				label: "System",
				dot: "bg-rose-500"
			}
		];
		currentRoute = navItems.find((i) => i.href === store_get($$store_subs ??= {}, "$page", page).url.pathname);
		head("12qhfyh", $$renderer, ($$renderer) => {
			$$renderer.title(($$renderer) => {
				$$renderer.push(`<title>Council of Claws</title>`);
			});
		});
		$$renderer.push(`<div class="flex min-h-screen text-slate-100"><aside class="hidden w-72 glass-sidebar md:flex md:flex-col fixed h-full z-40"><div class="flex h-20 items-center px-8 border-b border-white/5"><a href="/" class="text-xs font-black uppercase tracking-[0.4em] text-white">Council of <span class="text-indigo-500">Claws</span></a></div> <nav class="flex-1 space-y-1.5 px-4 py-8"><!--[-->`);
		const each_array = ensure_array_like(navItems);
		for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
			let item = each_array[$$index];
			$$renderer.push(`<a${attr("href", item.href)}${attr_class(`group flex items-center rounded-xl px-4 py-3 text-xs font-black uppercase tracking-widest transition-all duration-300 ${stringify(store_get($$store_subs ??= {}, "$page", page).url.pathname === item.href ? "bg-white/10 text-white shadow-lg" : "text-slate-500 hover:text-slate-200 hover:bg-white/5")}`)}><span${attr_class(`mr-4 h-1.5 w-1.5 rounded-full ${stringify(item.dot)} shadow-[0_0_8px_rgba(0,0,0,0.5)] group-hover:scale-150 transition-transform`)}></span> ${escape_html(item.label)}</a>`);
		}
		$$renderer.push(`<!--]--></nav> <div class="p-6 border-t border-white/5 bg-black/20"><div class="flex flex-col gap-4"><div class="flex items-center justify-between"><span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Infrastructure</span> <div${attr_class(`h-2 w-2 rounded-full animate-pulse ${stringify(store_get($$store_subs ??= {}, "$infrastructureOnline", infrastructureOnline) ? "bg-emerald-500 shadow-[0_0_10px_#10b981]" : "bg-rose-500 shadow-[0_0_10px_#f43f5e]")}`)}></div></div> <div class="flex flex-col gap-1"><span class="text-[9px] font-bold text-slate-600 uppercase tracking-tighter">Stream: ${escape_html(store_get($$store_subs ??= {}, "$transportStatus", transportStatus))}</span> <span class="text-[9px] font-bold text-slate-600 uppercase tracking-tighter">Updated: ${escape_html(store_get($$store_subs ??= {}, "$lastRefreshed", lastRefreshed).toLocaleTimeString())}</span></div></div></div></aside> <div class="flex flex-1 flex-col md:pl-72"><header class="glass-header h-20 flex items-center justify-between px-8"><div class="flex items-center gap-4"><button class="md:hidden btn-secondary p-2"><span class="text-xl">☰</span></button> <h1 class="text-sm font-black uppercase tracking-[0.2em] text-white">${escape_html(currentRoute?.label || "Platform")}</h1></div> <div class="flex items-center gap-3"><a href="/gateway" class="btn-secondary text-[10px] uppercase tracking-widest px-4 h-9 inline-flex items-center">Open Gateway</a> <button class="btn-secondary text-[10px] uppercase tracking-widest px-4 h-9 min-w-[8rem]"${attr("disabled", store_get($$store_subs ??= {}, "$refreshing", refreshing), true)}>${escape_html(store_get($$store_subs ??= {}, "$refreshing", refreshing) ? "Syncing…" : "Synchronize")}</button></div></header> <main class="flex-1 p-8 overflow-x-hidden"><div class="mx-auto max-w-6xl"><!--[-->`);
		slot($$renderer, $$props, "default", {});
		$$renderer.push(`<!--]--></div></main></div> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]--></div>`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}

export { _layout as default };
//# sourceMappingURL=_layout.svelte-BlSXDudh.js.map
