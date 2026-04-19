//#region src/routes/api/[...path]/+server.ts
var backendBase = process.env.INTERNAL_API_BASE_URL ?? "http://backend:8080";
function forwardHeaders(headers) {
	const next = new Headers();
	for (const [key, value] of headers.entries()) {
		if (key === "host" || key === "content-length") continue;
		next.set(key, value);
	}
	return next;
}
async function proxy({ fetch, params, request }) {
	const targetUrl = `${backendBase}/api/${params.path ?? ""}${new URL(request.url).search}`;
	const method = request.method;
	const init = {
		method,
		headers: forwardHeaders(request.headers)
	};
	if (!["GET", "HEAD"].includes(method)) init.body = await request.arrayBuffer();
	const upstream = await fetch(targetUrl, init);
	const responseHeaders = new Headers(upstream.headers);
	responseHeaders.delete("content-length");
	responseHeaders.delete("connection");
	return new Response(upstream.body, {
		status: upstream.status,
		statusText: upstream.statusText,
		headers: responseHeaders
	});
}
var GET = proxy;
var POST = proxy;
var PUT = proxy;
var PATCH = proxy;
var DELETE = proxy;

export { DELETE, GET, PATCH, POST, PUT };
//# sourceMappingURL=_server.ts-kzGCCfnY.js.map
