import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ fetch }) => {
  const backendBase = process.env.INTERNAL_API_BASE_URL ?? 'http://backend:8080';

  try {
    const response = await fetch(`${backendBase}/api/health`, { method: 'GET' });
    const body = await response.text();

    return new Response(body, {
      status: response.status,
      headers: {
        'content-type': 'application/json'
      }
    });
  } catch (error) {
    return json(
      {
        service: 'dashboard',
        status: 'error',
        reason: error instanceof Error ? error.message : 'backend-unreachable'
      },
      { status: 502 }
    );
  }
};
