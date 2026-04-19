import type { RequestHandler } from './$types';

const backendBase = process.env.INTERNAL_API_BASE_URL ?? 'http://backend:8080';

function forwardHeaders(headers: Headers): Headers {
	const next = new Headers();

	for (const [key, value] of headers.entries()) {
		if (key === 'host' || key === 'content-length') continue;
		next.set(key, value);
	}

	return next;
}

async function proxy({ fetch, params, request }: Parameters<RequestHandler>[0]) {
	const path = params.path ?? '';
	const targetUrl = `${backendBase}/api/${path}${new URL(request.url).search}`;
	const method = request.method;
	const headers = forwardHeaders(request.headers);
	const init: RequestInit = { method, headers };

	if (!['GET', 'HEAD'].includes(method)) {
		init.body = await request.arrayBuffer();
	}

	const upstream = await fetch(targetUrl, init);
	const responseHeaders = new Headers(upstream.headers);
	responseHeaders.delete('content-length');
	responseHeaders.delete('connection');

	return new Response(upstream.body, {
		status: upstream.status,
		statusText: upstream.statusText,
		headers: responseHeaders
	});
}

export const GET: RequestHandler = proxy;
export const POST: RequestHandler = proxy;
export const PUT: RequestHandler = proxy;
export const PATCH: RequestHandler = proxy;
export const DELETE: RequestHandler = proxy;
