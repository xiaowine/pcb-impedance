// server.js - High-performance Node.js Proxy & Static Server
const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 3000;
const PUBLIC_DIR = path.join(__dirname, 'public');
const TARGET_ORIGIN = 'https://tools.jlc.com';

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.ico': 'image/x-icon'
};

const server = http.createServer(async (req, res) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', '*');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const parsedUrl = new URL(req.url, `http://${req.headers.host}`);
  const pathname = parsedUrl.pathname;

  // 1. Proxy API requests to JLC backend
  if (pathname.startsWith('/api/')) {
    try {
      const targetUrl = TARGET_ORIGIN + req.url;
      const headers = {
        'Content-Type': req.headers['content-type'] || 'application/json',
        'Origin': TARGET_ORIGIN,
        'Referer': `${TARGET_ORIGIN}/jlcTools/index.html`,
        'User-Agent': req.headers['user-agent'] || 'Mozilla/5.0'
      };

      let bodyData = null;
      if (req.method !== 'GET' && req.method !== 'HEAD') {
        const chunks = [];
        for await (const chunk of req) {
          chunks.push(chunk);
        }
        bodyData = Buffer.concat(chunks);
      }

      const proxyResp = await fetch(targetUrl, {
        method: req.method,
        headers: headers,
        body: bodyData
      });

      res.writeHead(proxyResp.status, {
        'Content-Type': proxyResp.headers.get('content-type') || 'application/json'
      });

      const respBuffer = await proxyResp.arrayBuffer();
      res.end(Buffer.from(respBuffer));
    } catch (err) {
      console.error('[Proxy Error]:', err);
      res.writeHead(502, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'Proxy Error', message: err.message }));
    }
    return;
  }

  // 2. Serve static files from public directory
  let filePath = path.join(PUBLIC_DIR, pathname === '/' ? 'index.html' : pathname);
  
  if (!filePath.startsWith(PUBLIC_DIR)) {
    res.writeHead(403);
    res.end('Forbidden');
    return;
  }

  fs.stat(filePath, (err, stats) => {
    if (err || !stats.isFile()) {
      if (!path.extname(pathname)) {
        filePath = path.join(PUBLIC_DIR, 'index.html');
      } else {
        res.writeHead(404);
        res.end('Not Found');
        return;
      }
    }

    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    fs.readFile(filePath, (err, content) => {
      if (err) {
        res.writeHead(500);
        res.end('Server Error');
        return;
      }
      res.writeHead(200, {
        'Content-Type': contentType,
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'Pragma': 'no-cache',
        'Expires': '0'
      });
      res.end(content);
    });
  });
});

// Node 默认 keepAliveTimeout = 5000ms。使用连接池的客户端（Java HttpClient / Go / Python requests /
// axios / OkHttp）在空闲超过 5s 后复用连接时，会写进一条已被服务端关闭的 TCP 连接：
// 请求静默挂死（无响应，直到客户端自身超时）或抛 ConnectionAbortedError —— 表现为
// “上游没变慢、网关也不报错，但对端总是请求超时”。必须大于客户端连接池的空闲阈值。
server.keepAliveTimeout = 65000;
server.headersTimeout = 66000; // 必须大于 keepAliveTimeout

server.listen(PORT, () => {
  console.log(`[PCB Impedance Server] running at http://localhost:${PORT}`);
});
