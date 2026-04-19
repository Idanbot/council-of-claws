const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.9C_D9za-.js",app:"_app/immutable/entry/app.ZyZVOi7z.js",imports:["_app/immutable/entry/start.9C_D9za-.js","_app/immutable/chunks/D-Jbm_jl.js","_app/immutable/chunks/B6dYteId.js","_app/immutable/chunks/BIuoBiyh.js","_app/immutable/chunks/BU1s0yoG.js","_app/immutable/entry/app.ZyZVOi7z.js","_app/immutable/chunks/B6dYteId.js","_app/immutable/chunks/BIuoBiyh.js","_app/immutable/chunks/CsaauQz1.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./chunks/0-g_Mo_C_b.js')),
			__memo(() => import('./chunks/1-BfA_odv4.js')),
			__memo(() => import('./chunks/2-DWiwCE9U.js')),
			__memo(() => import('./chunks/3-DpmAN9Do.js')),
			__memo(() => import('./chunks/4-B1MsmAso.js')),
			__memo(() => import('./chunks/5-BSAi93oB.js')),
			__memo(() => import('./chunks/6-DfYczCs0.js')),
			__memo(() => import('./chunks/7-XeOC7Fld.js')),
			__memo(() => import('./chunks/8-Cdle6x--.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/agents",
				pattern: /^\/agents\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/api/backend-health",
				pattern: /^\/api\/backend-health\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./chunks/_server.ts-Biws9Mwf.js'))
			},
			{
				id: "/api/[...path]",
				pattern: /^\/api(?:\/([^]*))?\/?$/,
				params: [{"name":"path","optional":false,"rest":true,"chained":true}],
				page: null,
				endpoint: __memo(() => import('./chunks/_server.ts-kzGCCfnY.js'))
			},
			{
				id: "/council",
				pattern: /^\/council\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/events",
				pattern: /^\/events\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/gateway",
				pattern: /^\/gateway\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./chunks/_server.ts-CF9Q-qer.js'))
			},
			{
				id: "/health",
				pattern: /^\/health\/?$/,
				params: [],
				page: null,
				endpoint: __memo(() => import('./chunks/_server.ts-BPVks5cS.js'))
			},
			{
				id: "/system",
				pattern: /^\/system\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			},
			{
				id: "/tasks",
				pattern: /^\/tasks\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 7 },
				endpoint: null
			},
			{
				id: "/usage",
				pattern: /^\/usage\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 8 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();

const prerendered = new Set([]);

const base = "";

export { base, manifest, prerendered };
//# sourceMappingURL=manifest.js.map
