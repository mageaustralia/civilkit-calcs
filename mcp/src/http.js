// Streamable HTTP for the MCP endpoint, stateless (docs/mcp.md):
//   POST /mcp      one JSON-RPC request or notification per request; the answer is application/json (never a stream)
//   GET  /mcp      a page for people (HTML, or JSON with Accept: application/json); a client asking for an SSE stream
//                  (Accept: text/event-stream only) gets 405, as the transport allows
//   OPTIONS        CORS preflight
//   GET  /mcp/server-card   the MCP Server Card (SEP-2127)
// No cookies are read or set, no credentials are allowed in CORS, nothing is stored and request bodies are never
// logged. Errors never carry stack traces.
import { dispatch, ERR, LEGACY_VERSIONS, META, MODERN_VERSIONS, RpcError, SUPPORTED_VERSIONS } from './mcp.js';
import { humanPage, serverCard, describeJson } from './page.js';

export const MAX_BODY_BYTES = 256 * 1024;
export const MAX_RESPONSE_BYTES = 2 * 1024 * 1024;

const CORS = {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Accept, MCP-Protocol-Version, Mcp-Method, Mcp-Name, Mcp-Session-Id, Last-Event-ID, traceparent, tracestate',
    'Access-Control-Expose-Headers': 'MCP-Protocol-Version, Retry-After',
    'Access-Control-Max-Age': '86400',
};
const BASE_HEADERS = {
    ...CORS,
    'X-Content-Type-Options': 'nosniff',
    'Referrer-Policy': 'no-referrer',
    'Cache-Control': 'no-store',
};

function json(body, status = 200, extra = {}) {
    return new Response(JSON.stringify(body), { status, headers: { ...BASE_HEADERS, 'Content-Type': 'application/json; charset=utf-8', ...extra } });
}

function rpcError(id, code, message, data) {
    const error = { code, message };
    if (data !== undefined) error.data = data;
    const out = { jsonrpc: '2.0', error };
    if (id !== undefined && id !== null) out.id = id;
    return out;
}

/** An Origin header is accepted when it is "null" or a well-formed origin (scheme://host[:port]); any site may call. */
export function originOk(origin) {
    if (origin === null) return true;
    if (origin === 'null') return true;
    if (origin.length > 512) return false;
    try {
        const u = new URL(origin);
        return u.origin !== 'null' && u.origin === origin.replace(/\/$/, '') && /^(https?|chrome-extension|moz-extension|safari-web-extension|vscode-webview|tauri|app|file):$/.test(u.protocol);
    } catch {
        return false;
    }
}

/** Mcp-Name may be sent as =?base64?...?= (UTF-8). */
export function decodeHeaderValue(v) {
    const m = /^=\?base64\?([A-Za-z0-9+/=]*)\?=$/.exec(v);
    if (!m) return v;
    try {
        const bin = atob(m[1]);
        return new TextDecoder('utf-8', { fatal: true }).decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
    } catch {
        return null;
    }
}

async function readBody(request) {
    const len = Number(request.headers.get('Content-Length'));
    if (Number.isFinite(len) && len > MAX_BODY_BYTES) return { tooLarge: true };
    if (!request.body) return { text: '' };
    const reader = request.body.getReader();
    const chunks = [];
    let size = 0;
    for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > MAX_BODY_BYTES) {
            try { await reader.cancel(); } catch { /* ignore */ }
            return { tooLarge: true };
        }
        chunks.push(value);
    }
    const all = new Uint8Array(size);
    let at = 0;
    for (const c of chunks) { all.set(c, at); at += c.byteLength; }
    try {
        return { text: new TextDecoder('utf-8', { fatal: true }).decode(all) };
    } catch {
        return { text: null };
    }
}

/**
 * The fetch handler. deps: { evaluator, data, index, serverVersion, rateLimit(kind, request) -> Promise<boolean> }.
 * kind is 'request' (every POST) or 'eval' (a call that runs the engine).
 */
export function createHandler(deps) {
    return async function handle(request) {
        const url = new URL(request.url);
        const path = url.pathname.replace(/\/+$/, '') || '/';
        const origin = request.headers.get('Origin');
        if (!originOk(origin)) {
            return json(rpcError(null, ERR.INVALID_REQUEST, 'Forbidden: invalid Origin header'), 403);
        }
        if (path === '/mcp/server-card') {
            if (request.method === 'OPTIONS') return new Response(null, { status: 204, headers: BASE_HEADERS });
            if (request.method !== 'GET' && request.method !== 'HEAD') return json(rpcError(null, ERR.INVALID_REQUEST, 'Method not allowed'), 405, { Allow: 'GET, HEAD, OPTIONS' });
            const body = JSON.stringify(serverCard(deps.serverVersion), null, 2);
            const etag = `"${deps.serverVersion}-${body.length}"`;
            const headers = { ...BASE_HEADERS, 'Content-Type': 'application/mcp-server-card+json', 'Cache-Control': 'public, max-age=3600', ETag: etag };
            if (request.headers.get('If-None-Match') === etag) return new Response(null, { status: 304, headers });
            return new Response(request.method === 'HEAD' ? null : body, { status: 200, headers });
        }
        if (path !== '/mcp') {
            return json({ error: 'Not found', mcp_endpoint: 'https://civilkit.io/mcp' }, 404);
        }
        switch (request.method) {
            case 'OPTIONS':
                return new Response(null, { status: 204, headers: BASE_HEADERS });
            case 'GET':
            case 'HEAD': {
                const accept = (request.headers.get('Accept') ?? '').toLowerCase();
                // A client opening the old standalone SSE stream: this server has none.
                if (accept.includes('text/event-stream') && !accept.includes('text/html') && !accept.includes('application/json')) {
                    return json(rpcError(null, ERR.INVALID_REQUEST, 'This server has no SSE stream; POST JSON-RPC messages to this URL.'), 405, { Allow: 'GET, POST, OPTIONS' });
                }
                const wantsJson = accept.includes('application/json') && !accept.includes('text/html');
                const headers = { ...BASE_HEADERS, 'Cache-Control': 'public, max-age=300' };
                if (wantsJson) return new Response(request.method === 'HEAD' ? null : JSON.stringify(describeJson(deps.serverVersion), null, 2), { status: 200, headers: { ...headers, 'Content-Type': 'application/json; charset=utf-8' } });
                return new Response(request.method === 'HEAD' ? null : humanPage(deps.serverVersion), {
                    status: 200,
                    headers: {
                        ...headers,
                        'Content-Type': 'text/html; charset=utf-8',
                        'Content-Security-Policy': "default-src 'none'; style-src 'unsafe-inline'; img-src https://civilkit.io; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
                    },
                });
            }
            case 'POST':
                return post(request, deps);
            default:
                return json(rpcError(null, ERR.INVALID_REQUEST, 'Method not allowed'), 405, { Allow: 'GET, POST, OPTIONS' });
        }
    };
}

async function post(request, deps) {
    if (!(await deps.rateLimit('request', request))) {
        return json(rpcError(null, ERR.RATE_LIMITED, 'Rate limited: too many requests from this address. Try again in a minute.'), 429, { 'Retry-After': '60' });
    }
    const type = (request.headers.get('Content-Type') ?? '').toLowerCase();
    if (!type.startsWith('application/json')) {
        return json(rpcError(null, ERR.INVALID_REQUEST, 'Content-Type must be application/json'), 415);
    }
    const body = await readBody(request);
    if (body.tooLarge) return json(rpcError(null, ERR.INVALID_REQUEST, `Request body over ${MAX_BODY_BYTES / 1024} KB`), 413);
    if (body.text === null) return json(rpcError(null, ERR.PARSE, 'Parse error: the body is not UTF-8'), 400);
    let msg;
    try {
        msg = JSON.parse(body.text);
    } catch {
        return json(rpcError(null, ERR.PARSE, 'Parse error'), 400);
    }
    if (Array.isArray(msg)) return json(rpcError(null, ERR.INVALID_REQUEST, 'Batches are not supported; send one message per request'), 400);
    if (typeof msg !== 'object' || msg === null || msg.jsonrpc !== '2.0') return json(rpcError(null, ERR.INVALID_REQUEST, 'Invalid Request: not a JSON-RPC 2.0 message'), 400);
    const hasId = Object.hasOwn(msg, 'id');
    // A response from the client (legacy clients may answer server requests; this server sends none).
    if (!Object.hasOwn(msg, 'method')) {
        if (hasId && (Object.hasOwn(msg, 'result') || Object.hasOwn(msg, 'error'))) return new Response(null, { status: 202, headers: BASE_HEADERS });
        return json(rpcError(null, ERR.INVALID_REQUEST, 'Invalid Request: no method'), 400);
    }
    if (typeof msg.method !== 'string') return json(rpcError(null, ERR.INVALID_REQUEST, 'Invalid Request: method must be a string'), 400);
    if (!hasId) return new Response(null, { status: 202, headers: BASE_HEADERS }); // a notification: accepted, nothing to answer
    const id = msg.id;
    if (!(typeof id === 'string' || (typeof id === 'number' && Number.isInteger(id)))) {
        return json(rpcError(null, ERR.INVALID_REQUEST, 'Invalid Request: id must be a string or an integer'), 400);
    }

    // Which era: modern requests carry their version in params._meta.
    const meta = msg.params && typeof msg.params === 'object' && !Array.isArray(msg.params) ? msg.params._meta : undefined;
    const metaVersion = meta && typeof meta === 'object' ? meta[`${META}protocolVersion`] : undefined;
    const headerVersion = request.headers.get('MCP-Protocol-Version');
    let era;
    let version;
    if (metaVersion !== undefined) {
        era = 'modern';
        version = metaVersion;
        if (typeof version !== 'string' || !SUPPORTED_VERSIONS.includes(version) || LEGACY_VERSIONS.includes(version)) {
            return json(rpcError(id, ERR.UNSUPPORTED_VERSION, 'Unsupported protocol version', { supported: SUPPORTED_VERSIONS, requested: version }), 400);
        }
        if (headerVersion === null) return json(rpcError(id, ERR.HEADER_MISMATCH, 'Header mismatch: MCP-Protocol-Version header is missing'), 400);
        if (headerVersion !== version) return json(rpcError(id, ERR.HEADER_MISMATCH, `Header mismatch: MCP-Protocol-Version header value '${headerVersion.slice(0, 40)}' does not match body value '${version}'`), 400);
        const method = request.headers.get('Mcp-Method');
        if (method === null) return json(rpcError(id, ERR.HEADER_MISMATCH, 'Header mismatch: Mcp-Method header is missing'), 400);
        if (method !== msg.method) return json(rpcError(id, ERR.HEADER_MISMATCH, 'Header mismatch: Mcp-Method header does not match the body'), 400);
        if (['tools/call', 'resources/read', 'prompts/get'].includes(msg.method)) {
            const nameHeader = request.headers.get('Mcp-Name');
            const bodyName = msg.method === 'resources/read' ? msg.params.uri : msg.params.name;
            if (nameHeader === null) return json(rpcError(id, ERR.HEADER_MISMATCH, 'Header mismatch: Mcp-Name header is missing'), 400);
            if (decodeHeaderValue(nameHeader) !== bodyName) return json(rpcError(id, ERR.HEADER_MISMATCH, 'Header mismatch: Mcp-Name header does not match the body'), 400);
        }
        const caps = meta[`${META}clientCapabilities`];
        if (typeof caps !== 'object' || caps === null || Array.isArray(caps)) {
            return json(rpcError(id, ERR.INVALID_PARAMS, `Invalid params: _meta["${META}clientCapabilities"] is required`), 400);
        }
    } else {
        era = 'legacy';
        if (msg.method === 'initialize') {
            version = LEGACY_VERSIONS[0];
        } else if (headerVersion === null) {
            version = '2025-03-26'; // clients before 2025-06-18 sent no header
        } else if (LEGACY_VERSIONS.includes(headerVersion)) {
            version = headerVersion;
        } else if (MODERN_VERSIONS.includes(headerVersion)) {
            return json(rpcError(id, ERR.INVALID_PARAMS, `Invalid params: protocol ${headerVersion} requests carry _meta["${META}protocolVersion"]`), 400);
        } else {
            return json(rpcError(id, ERR.UNSUPPORTED_VERSION, 'Unsupported protocol version', { supported: SUPPORTED_VERSIONS, requested: headerVersion.slice(0, 40) }), 400);
        }
    }

    const ctx = {
        era,
        version,
        evaluator: deps.evaluator,
        data: deps.data,
        index: deps.index,
        serverVersion: deps.serverVersion,
        limitEval: () => deps.rateLimit('eval', request),
    };
    const versionHeader = { 'MCP-Protocol-Version': era === 'modern' ? version : msg.method === 'initialize' ? '' : version };
    if (!versionHeader['MCP-Protocol-Version']) delete versionHeader['MCP-Protocol-Version'];
    try {
        const result = await dispatch(msg, ctx);
        const text = JSON.stringify({ jsonrpc: '2.0', id, result });
        if (text.length > MAX_RESPONSE_BYTES) {
            return json(rpcError(id, ERR.INTERNAL, 'The result is too large to return; make the worksheet smaller.'), 200, versionHeader);
        }
        return new Response(text, { status: 200, headers: { ...BASE_HEADERS, 'Content-Type': 'application/json; charset=utf-8', ...versionHeader } });
    } catch (e) {
        if (e instanceof RpcError) {
            // Modern: an unknown method is 404 (with the JSON-RPC error, so it is not mistaken for a legacy server);
            // other errors of a request that was read are answered with 200, like legacy.
            const status = era === 'modern' && e.code === ERR.METHOD_NOT_FOUND ? 404 : 200;
            return json(rpcError(id, e.code, e.message, e.data), status, versionHeader);
        }
        // Never a stack trace or the input.
        return json(rpcError(id, ERR.INTERNAL, 'Internal error'), 500, versionHeader);
    }
}
