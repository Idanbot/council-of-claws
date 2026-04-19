import { aa as derived$1, w as writable } from './dev-C8SO_9bI.js';
import { a as configuredAgents } from './configured-agents-CMIbisL5.js';

//#region src/lib/stores.ts
var systemState = writable(null);
var streamEvents = writable([]);
var wsConnected = writable(false);
var apiHealthy = writable(false);
var refreshing = writable(false);
var lastRefreshed = writable(/* @__PURE__ */ new Date());
derived$1(systemState, ($state) => {
	const liveAgents = $state?.active_agents || [];
	return liveAgents.length > 0 ? liveAgents : configuredAgents;
});
var activeTasks = derived$1(systemState, ($state) => {
	return [...$state?.failed_tasks || [], ...$state?.blocked_tasks || []];
});
derived$1(systemState, ($state) => $state?.system_health);
var liveAgents = derived$1(systemState, ($state) => $state?.active_agents || []);
var transportStatus = derived$1([wsConnected, apiHealthy], ([$wsConnected, $apiHealthy]) => {
	if ($wsConnected) return "LIVE";
	if ($apiHealthy) return "POLLING";
	return "OFFLINE";
});
var infrastructureOnline = derived$1([wsConnected, apiHealthy], ([$wsConnected, $apiHealthy]) => $wsConnected || $apiHealthy);

export { liveAgents as a, activeTasks as b, systemState as c, infrastructureOnline as i, lastRefreshed as l, refreshing as r, streamEvents as s, transportStatus as t };
//# sourceMappingURL=stores-DfDuB3E0.js.map
