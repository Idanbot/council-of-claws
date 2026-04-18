const http = require('http');

const port = Number(process.env.GATEWAY_PORT || 18789);

const server = http.createServer((req, res) => {
  const payload = {
    service: 'gateway',
    status: 'ok',
    path: req.url,
    timestamp: new Date().toISOString()
  };

  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(payload));
});

server.listen(port, '0.0.0.0', () => {
  console.log(`gateway listening on ${port}`);
});
