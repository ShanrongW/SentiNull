import { createServer } from 'node:http';

import { findAsset } from './static.mjs';

const headers = {
  'Cache-Control': 'no-store',
  'Content-Security-Policy': "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'",
  'X-Content-Type-Options': 'nosniff',
};

function send(response, status, contentType, body) {
  response.writeHead(status, { ...headers, 'Content-Type': contentType });
  response.end(body);
}

function json(response, status, body) {
  send(response, status, 'application/json; charset=utf-8', JSON.stringify(body));
}

export function createHttpServer(config, telemetry) {
  const hosts = new Set([config.host, 'localhost', `${config.host}:${config.port}`, `localhost:${config.port}`]);

  return createServer(async (request, response) => {
    try {
      if (!hosts.has(request.headers.host?.toLowerCase())) {
        return json(response, 400, { error: 'Use the local dashboard address' });
      }
      if (request.method !== 'GET' && request.method !== 'HEAD') {
        response.setHeader('Allow', 'GET, HEAD');
        return json(response, 405, { error: 'Method not allowed' });
      }
      const { pathname, searchParams } = new URL(request.url, 'http://localhost');
      if (pathname === '/api/v1/overview') {
        const rawWindow = searchParams.get('window') ?? '300';
        const windowSeconds = /^\d+$/.test(rawWindow) ? Number(rawWindow) : NaN;
        if (!Number.isInteger(windowSeconds) || windowSeconds < 30 || windowSeconds > 3600) {
          return json(response, 400, { error: 'window must be an integer from 30 to 3600 seconds' });
        }
        return json(response, 200, telemetry.overview(windowSeconds));
      }
      if (pathname === '/api/v1/health') {
        const status = telemetry.status();
        return json(response, status === 'live' ? 200 : 503, { status });
      }
      if (pathname.startsWith('/api/')) return json(response, 404, { error: 'Not found' });
      const asset = await findAsset(config.frontendPath, pathname);
      if (!asset) return json(response, 404, { error: 'Not found' });
      send(response, 200, asset.contentType, asset.body);
    } catch (error) {
      if (error instanceof TypeError && error.code === 'ERR_INVALID_URL') {
        return json(response, 400, { error: 'Invalid request URL' });
      }
      console.error(error);
      json(response, 500, { error: 'Internal server error' });
    }
  });
}
