// Minimal static server for web/dist (no dependencies).
// Usage: node scripts/serve.mjs [port] [base-path]
// e.g.   node scripts/serve.mjs 8080 /cobalt-transformer-hackathon/
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const dist = resolve(dirname(fileURLToPath(import.meta.url)), '..', 'dist');
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm', '.cob': 'text/plain; charset=utf-8', '.cbl': 'text/plain; charset=utf-8',
};

export function startServer(port = 8080, base = '/') {
  if (!base.endsWith('/')) base += '/';
  const server = createServer(async (req, res) => {
    const url = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (!url.startsWith(base)) { res.writeHead(404).end('not found'); return; }
    let rel = normalize(url.slice(base.length)).replace(/^[/\\]+/, '');
    if (rel.startsWith('..')) { res.writeHead(403).end(); return; }
    let file = join(dist, rel);
    try {
      if ((await stat(file)).isDirectory()) file = join(file, 'index.html');
      const body = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] || 'application/octet-stream' });
      res.end(body);
    } catch {
      res.writeHead(404).end('not found');
    }
  });
  return new Promise((ok) => server.listen(port, '127.0.0.1', () => ok(server)));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const port = Number(process.argv[2] || 8080);
  const base = process.argv[3] || '/';
  await startServer(port, base);
  console.log(`serving ${dist} at http://127.0.0.1:${port}${base.endsWith('/') ? base : base + '/'}`);
}
