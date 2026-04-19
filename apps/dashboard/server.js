import http from 'http';
import net from 'net';
import { handler } from './build/handler.js';
import { env } from 'process';

const PORT = Number(env.DASHBOARD_PORT || 3000);
const BACKEND_URL = env.INTERNAL_API_BASE_URL || 'http://backend:8080';
const ORCHESTRATOR_URL = 'http://gateway:18789';

const server = http.createServer((req, res) => {
  const start = Date.now();

  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`[${new Date().toISOString()}] ${req.method} ${req.url} -> ${res.statusCode} (${duration}ms)`);
  });

  if (req.url === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ service: 'dashboard', status: 'ok' }));
    return;
  }

  // 1. Legacy HTTP proxy to OpenClaw Gateway API
  if (req.url.startsWith('/orchestrator/') || req.url.startsWith('/gateway-api/')) {
      try {
        const target = new URL(
          req.url.startsWith('/gateway-api/')
            ? req.url.replace('/gateway-api/', '/')
            : req.url.replace('/orchestrator/', '/'),
          ORCHESTRATOR_URL
        );
        const proxyReq = http.request(
          {
            hostname: target.hostname,
            port: target.port,
            path: target.pathname + target.search,
            method: req.method,
            headers: { ...req.headers, host: target.host },
            timeout: 5000
          },
          (proxyRes) => {
            res.writeHead(proxyRes.statusCode, proxyRes.headers);
            proxyRes.pipe(res, { end: true });
          }
        );
        proxyReq.on('error', (err) => {
            res.writeHead(502);
            res.end(`Orchestrator error: ${err.message}`);
        });
        req.pipe(proxyReq, { end: true });
        return;
      } catch (err) {
          console.error(err);
      }
  }

  // 2. Proxy /api/ to Rust Backend
  if (req.url.startsWith('/api/')) {
    try {
      const target = new URL(req.url, BACKEND_URL);
      const proxyReq = http.request(
        {
          hostname: target.hostname,
          port: target.port,
          path: target.pathname + target.search,
          method: req.method,
          headers: { ...req.headers, host: target.host },
          timeout: 10000
        },
        (proxyRes) => {
          res.writeHead(proxyRes.statusCode, proxyRes.headers);
          proxyRes.pipe(res, { end: true });
        }
      );

      proxyReq.on('error', (err) => {
        console.error(`[PROXY ERROR] ${req.url}: ${err.message}`);
        res.writeHead(502, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Backend unreachable', detail: err.message }));
      });

      req.pipe(proxyReq, { end: true });
      return;
    } catch (err) {
      console.error(`[URL ERROR] ${req.url}: ${err.message}`);
    }
  }

  // 3. UI and Assets
  handler(req, res);
});

server.on('upgrade', (req, socket, head) => {
  if (req.url === '/ws') {
    const backendUrl = new URL(BACKEND_URL);
    const proxySocket = net.connect(
      Number(backendUrl.port || 80),
      backendUrl.hostname,
      () => {
        const headerLines = Object.entries(req.headers)
          .map(([key, value]) => `${key}: ${Array.isArray(value) ? value.join(', ') : value}`)
          .join('\r\n');

        proxySocket.write(
          `GET /ws HTTP/${req.httpVersion}\r\n` +
          `${headerLines}\r\n` +
          `\r\n`
        );

        if (head?.length) {
          proxySocket.write(head);
        }

        socket.pipe(proxySocket);
        proxySocket.pipe(socket);
      }
    );

    proxySocket.on('error', (err) => {
      console.error(`[WS PROXY ERROR]: ${err.message}`);
      socket.destroy();
    });

    socket.on('error', () => proxySocket.destroy());
  } else {
    socket.destroy();
  }
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`Council Platform running on port ${PORT}`);
  console.log(`- Dashboard: http://localhost:${PORT}`);
  console.log(`- Gateway UI redirect: http://localhost:${PORT}/gateway`);
  console.log(`- Gateway API proxy: http://localhost:${PORT}/gateway-api/`);
});
