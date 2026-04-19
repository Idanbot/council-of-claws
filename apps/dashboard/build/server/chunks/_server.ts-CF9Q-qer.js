import { r as redirect } from './index-BWBZNJEj.js';
import { env } from 'node:process';
import './index-DBqjc0Yf.js';

//#region src/routes/gateway/+server.ts
var GET = ({ url }) => {
	const gatewayToken = env.OPENCLAW_GATEWAY_TOKEN || "council-local-gateway-token";
	const publicGatewayUrl = env.PUBLIC_GATEWAY_URL;
	const gatewayPort = env.GATEWAY_PORT || "18789";
	const gatewayUrl = publicGatewayUrl ? new URL(publicGatewayUrl) : new URL(`${url.protocol}//${url.hostname}:${gatewayPort}/`);
	gatewayUrl.hash = new URLSearchParams({ token: gatewayToken }).toString();
	throw redirect(307, gatewayUrl.toString());
};

export { GET };
//# sourceMappingURL=_server.ts-CF9Q-qer.js.map
