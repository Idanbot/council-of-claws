import { redirect } from '@sveltejs/kit';
import { env } from 'node:process';

export const GET = ({ url }) => {
  const gatewayToken = env.OPENCLAW_GATEWAY_TOKEN || 'council-local-gateway-token';
  const publicGatewayUrl = env.PUBLIC_GATEWAY_URL;
  const gatewayPort = env.GATEWAY_PORT || '18789';
  const gatewayUrl = publicGatewayUrl
    ? new URL(publicGatewayUrl)
    : new URL(`${url.protocol}//${url.hostname}:${gatewayPort}/`);

  gatewayUrl.hash = new URLSearchParams({ token: gatewayToken }).toString();

  throw redirect(307, gatewayUrl.toString());
};
