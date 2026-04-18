import { redirect } from '@sveltejs/kit';
import { env } from 'node:process';

export const GET = ({ url }) => {
  const gatewayPort = env.GATEWAY_PORT || '18789';
  const gatewayToken = env.OPENCLAW_GATEWAY_TOKEN || 'council-local-gateway-token';
  const gatewayUrl = new URL(`http://${url.hostname}:${gatewayPort}/`);

  gatewayUrl.searchParams.set('token', gatewayToken);

  throw redirect(307, gatewayUrl.toString());
};
