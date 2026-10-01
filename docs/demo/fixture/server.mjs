import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';

const routes = new Map([
  ['/', ['src/index.html', 'text/html']],
  ['/app.js', ['src/app.js', 'text/javascript']],
  ['/model.mjs', ['src/model.mjs', 'text/javascript']],
  ['/style.css', ['src/style.css', 'text/css']],
]);
createServer(async (request, response) => {
  const route = routes.get(new URL(request.url, 'http://127.0.0.1').pathname);
  if (!route || request.method !== 'GET') { response.writeHead(404).end(); return; }
  try {
    const bytes = await readFile(new URL(route[0], import.meta.url));
    response.writeHead(200, { 'Content-Type': `${route[1]}; charset=utf-8`, 'Cache-Control': 'no-store' });
    response.end(bytes);
  } catch { response.writeHead(500).end('Local fixture file unavailable'); }
}).listen(4387, '127.0.0.1', () => console.log('Disposable task board: http://127.0.0.1:4387'));
