import { j as json } from './index-BWBZNJEj.js';
import './index-DBqjc0Yf.js';

//#region src/routes/api/backend-health/+server.ts
var GET = async ({ fetch }) => {
	const backendBase = process.env.INTERNAL_API_BASE_URL ?? "http://backend:8080";
	try {
		const response = await fetch(`${backendBase}/api/health`, { method: "GET" });
		const body = await response.text();
		return new Response(body, {
			status: response.status,
			headers: { "content-type": "application/json" }
		});
	} catch (error) {
		return json({
			service: "dashboard",
			status: "error",
			reason: error instanceof Error ? error.message : "backend-unreachable"
		}, { status: 502 });
	}
};

export { GET };
//# sourceMappingURL=_server.ts-Biws9Mwf.js.map
