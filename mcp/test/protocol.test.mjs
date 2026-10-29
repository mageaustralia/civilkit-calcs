// The JSON-RPC and Streamable HTTP layer: both protocol eras, header validation, errors, CORS, Origin, GET, limits.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { makeApp, post, MODERN, URL_MCP } from './helpers.mjs';
import { originOk, decodeHeaderValue } from '../src/http.js';

const { handler } = makeApp();

const raw = (body, headers = {}, method = 'POST', url = URL_MCP) =>
    handler(new Request(url, { method, headers: { 'Content-Type': 'application/json', ...headers }, body }));

test('legacy initialize negotiates the version and issues no session', async () => {
    for (const [asked, got] of [['2025-06-18', '2025-06-18'], ['2025-03-26', '2025-03-26'], ['2025-11-25', '2025-11-25'], ['2024-11-05', '2025-11-25'], ['1999-01-01', '2025-11-25']]) {
        const r = await post(handler, 'initialize', { protocolVersion: asked, capabilities: {}, clientInfo: { name: 't', version: '1' } }, { headers: { 'MCP-Protocol-Version': null } });
        assert.equal(r.status, 200);
        assert.equal(r.body.result.protocolVersion, got);
        assert.ok(r.body.result.capabilities.tools);
        assert.equal(r.body.result.serverInfo.name, 'civilkit-calcs');
        assert.ok(r.body.result.instructions.length > 50);
        assert.equal(r.headers.get('Mcp-Session-Id'), null);
        assert.equal(r.body.result.resultType, undefined, 'legacy results carry no resultType');
    }
});

test('notifications and client responses get 202 with no body', async () => {
    const r = await post(handler, 'notifications/initialized', {}, { notification: true });
    assert.equal(r.status, 202);
    assert.equal(r.body, null);
    const res = await raw(JSON.stringify({ jsonrpc: '2.0', id: 5, result: {} }));
    assert.equal(res.status, 202);
});

test('ping, tools/list, resources and prompts (legacy)', async () => {
    assert.deepEqual((await post(handler, 'ping')).body.result, {});
    const tools = (await post(handler, 'tools/list')).body.result.tools;
    const names = tools.map((t) => t.name);
    for (const n of ['calcs_evaluate_sheet', 'calcs_evaluate_calcpad', 'calcs_convert_units', 'calcs_check_hand_calc', 'calcs_search_functions',
        'calcs_function_help', 'calcs_search_docs', 'calcs_get_guide_page', 'calcs_list_samples', 'calcs_get_sample', 'calcs_sheet_syntax']) {
        assert.ok(names.includes(n), n);
    }
    for (const t of tools) {
        assert.match(t.name, /^[A-Za-z0-9_.-]{1,128}$/);
        assert.equal(t.inputSchema.type, 'object');
        assert.ok(t.description.length > 80, `${t.name} has a real description`);
        assert.equal(t.annotations.readOnlyHint, true);
        assert.equal(t.annotations.destructiveHint, false);
        assert.equal(t.annotations.openWorldHint, false);
        assert.equal(t.run, undefined);
    }
    const res = (await post(handler, 'resources/list')).body.result.resources;
    assert.ok(res.some((r) => r.uri === 'civilkit-calcs://syntax'));
    assert.ok(res.some((r) => r.uri === 'civilkit-calcs://guide/units'));
    assert.ok(res.some((r) => r.uri === 'civilkit-calcs://samples/sheet-beam-bending'));
    const read = (await post(handler, 'resources/read', { uri: 'civilkit-calcs://samples/sheet-beam-bending' })).body.result.contents[0];
    assert.match(read.text, /^<!--sheet-->/);
    const g = (await post(handler, 'resources/read', { uri: 'civilkit-calcs://guide/sheet/quantities' })).body.result.contents[0];
    assert.match(g.text, /^# /);
    const missing = await post(handler, 'resources/read', { uri: 'civilkit-calcs://guide/nope' });
    assert.equal(missing.body.error.code, -32002, 'legacy resource not found');
    assert.ok((await post(handler, 'resources/templates/list')).body.result.resourceTemplates.length >= 2);
    const prompts = (await post(handler, 'prompts/list')).body.result.prompts;
    assert.deepEqual(prompts.map((p) => p.name).sort(), ['check_my_hand_calc', 'write_a_sheet']);
    const pg = await post(handler, 'prompts/get', { name: 'write_a_sheet', arguments: { task: 'a pad footing' } });
    assert.match(pg.body.result.messages[0].content.text, /pad footing/);
    const pe = await post(handler, 'prompts/get', { name: 'write_a_sheet', arguments: {} });
    assert.equal(pe.body.error.code, -32602);
});

test('unknown method and unknown tool are protocol errors', async () => {
    const r = await post(handler, 'does/not/exist');
    assert.equal(r.status, 200);
    assert.equal(r.body.error.code, -32601);
    const t = await post(handler, 'tools/call', { name: 'nope', arguments: {} });
    assert.equal(t.body.error.code, -32602);
    assert.match(t.body.error.message, /Unknown tool/);
});

test('malformed JSON-RPC', async () => {
    assert.equal((await raw('{not json')).status, 400);
    assert.equal((await raw('{not json').then((r) => r.json())).error.code, -32700);
    const batch = await raw(JSON.stringify([{ jsonrpc: '2.0', id: 1, method: 'ping' }]));
    assert.equal(batch.status, 400);
    assert.equal((await batch.json()).error.code, -32600);
    assert.equal((await raw(JSON.stringify({ id: 1, method: 'ping' }))).status, 400, 'no jsonrpc member');
    assert.equal((await raw(JSON.stringify({ jsonrpc: '2.0', id: null, method: 'ping' }))).status, 400, 'null id');
    assert.equal((await raw(JSON.stringify({ jsonrpc: '2.0', id: 1.5, method: 'ping' }))).status, 400, 'fractional id');
    assert.equal((await raw(JSON.stringify({ jsonrpc: '2.0', id: 1, method: 7 }))).status, 400);
    const p = await post(handler, 'tools/call', 'x');
    assert.ok(p.body.error, 'params must be an object');
    const ct = await raw(JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'ping' }), { 'Content-Type': 'text/plain' });
    assert.equal(ct.status, 415);
});

test('a legacy request with an unknown MCP-Protocol-Version header is refused', async () => {
    const r = await post(handler, 'ping', {}, { headers: { 'MCP-Protocol-Version': '2030-01-01' } });
    assert.equal(r.status, 400);
    assert.equal(r.body.error.code, -32022);
    assert.deepEqual(r.body.error.data.supported.includes('2025-06-18'), true);
    const none = await post(handler, 'ping', {}, { headers: { 'MCP-Protocol-Version': null } });
    assert.equal(none.status, 200, 'no header means 2025-03-26');
});

test('modern requests: per-request metadata, resultType, serverInfo', async () => {
    const d = await post(handler, 'server/discover', {}, { era: 'modern' });
    assert.equal(d.status, 200);
    assert.equal(d.body.result.resultType, 'complete');
    assert.ok(d.body.result.supportedVersions.includes(MODERN));
    assert.equal(d.body.result._meta['io.modelcontextprotocol/serverInfo'].name, 'civilkit-calcs');
    assert.equal(d.headers.get('MCP-Protocol-Version'), MODERN);
    const l = await post(handler, 'tools/list', {}, { era: 'modern' });
    assert.equal(l.body.result.resultType, 'complete');
    assert.ok(l.body.result.tools.length >= 11);
    const c = await post(handler, 'tools/call', { name: 'calcs_convert_units', arguments: { expression: '1 m -> mm' } }, { era: 'modern' });
    assert.equal(c.body.result.resultType, 'complete');
    assert.equal(c.body.result.structuredContent.value, 1000);
    const missing = await post(handler, 'resources/read', { uri: 'civilkit-calcs://nope' }, { era: 'modern' });
    assert.equal(missing.body.error.code, -32602, 'modern resource not found');
    const init = await post(handler, 'initialize', { protocolVersion: MODERN }, { era: 'modern' });
    assert.equal(init.status, 404);
    assert.equal(init.body.error.code, -32601);
    assert.match(init.body.error.message, /2025-11-25/);
    const unknown = await post(handler, 'nope/nope', {}, { era: 'modern' });
    assert.equal(unknown.status, 404);
    assert.equal(unknown.body.error.code, -32601);
});

test('modern header validation', async () => {
    const mismatch = await post(handler, 'ping', {}, { era: 'modern', headers: { 'MCP-Protocol-Version': '2025-06-18' } });
    assert.equal(mismatch.status, 400);
    assert.equal(mismatch.body.error.code, -32020);
    const noHeader = await post(handler, 'ping', {}, { era: 'modern', headers: { 'MCP-Protocol-Version': null } });
    assert.equal(noHeader.body.error.code, -32020);
    const method = await post(handler, 'ping', {}, { era: 'modern', headers: { 'Mcp-Method': 'tools/list' } });
    assert.equal(method.body.error.code, -32020);
    const noMethod = await post(handler, 'ping', {}, { era: 'modern', headers: { 'Mcp-Method': null } });
    assert.equal(noMethod.body.error.code, -32020);
    const name = await post(handler, 'tools/call', { name: 'calcs_sheet_syntax', arguments: {} }, { era: 'modern', headers: { 'Mcp-Name': 'calcs_list_samples' } });
    assert.equal(name.status, 400);
    assert.equal(name.body.error.code, -32020);
    const noName = await post(handler, 'tools/call', { name: 'calcs_sheet_syntax', arguments: {} }, { era: 'modern', headers: { 'Mcp-Name': null } });
    assert.equal(noName.body.error.code, -32020);
    const b64 = await post(handler, 'tools/call', { name: 'calcs_sheet_syntax', arguments: {} }, { era: 'modern', headers: { 'Mcp-Name': `=?base64?${btoa('calcs_sheet_syntax')}?=` } });
    assert.equal(b64.status, 200);
    assert.equal(b64.body.result.isError, undefined);
    const unsupported = await post(handler, 'ping', { _meta: { 'io.modelcontextprotocol/protocolVersion': '1900-01-01' } }, { era: 'modern', headers: { 'MCP-Protocol-Version': '1900-01-01' } });
    assert.equal(unsupported.status, 400);
    assert.equal(unsupported.body.error.code, -32022);
    assert.deepEqual(unsupported.body.error.data.requested, '1900-01-01');
    assert.ok(unsupported.body.error.data.supported.includes(MODERN));
    const caps = await post(handler, 'ping', { _meta: { 'io.modelcontextprotocol/clientCapabilities': undefined } }, { era: 'modern' });
    assert.equal(caps.status, 400);
    assert.equal(caps.body.error.code, -32602);
});

test('header value decoding and Origin checks', () => {
    assert.equal(decodeHeaderValue('plain'), 'plain');
    assert.equal(decodeHeaderValue('=?base64?SGVsbG8sIOS4lueVjA==?='), 'Hello, 世界');
    assert.equal(decodeHeaderValue('=?base64?////?='), null);
    assert.equal(originOk(null), true);
    assert.equal(originOk('null'), true);
    assert.equal(originOk('https://claude.ai'), true);
    assert.equal(originOk('http://localhost:6274'), true);
    assert.equal(originOk('https://example.com/path'), false);
    assert.equal(originOk('not an origin'), false);
    assert.equal(originOk('javascript:alert(1)'), false);
});

test('Origin: any site may call, a malformed Origin is 403, credentials are never allowed', async () => {
    const ok = await post(handler, 'ping', {}, { headers: { Origin: 'https://some-site.example' } });
    assert.equal(ok.status, 200);
    assert.equal(ok.headers.get('Access-Control-Allow-Origin'), '*');
    assert.equal(ok.headers.get('Access-Control-Allow-Credentials'), null);
    assert.equal(ok.headers.get('Set-Cookie'), null);
    const bad = await post(handler, 'ping', {}, { headers: { Origin: 'https://evil.example/x?y' } });
    assert.equal(bad.status, 403);
    assert.equal(bad.body.error.code, -32600);
    assert.equal(bad.body.id, undefined);
});

test('OPTIONS preflight', async () => {
    const r = await handler(new Request(URL_MCP, { method: 'OPTIONS', headers: { Origin: 'https://claude.ai', 'Access-Control-Request-Method': 'POST', 'Access-Control-Request-Headers': 'content-type, mcp-protocol-version' } }));
    assert.equal(r.status, 204);
    assert.equal(r.headers.get('Access-Control-Allow-Origin'), '*');
    assert.match(r.headers.get('Access-Control-Allow-Methods'), /POST/);
    assert.match(r.headers.get('Access-Control-Allow-Headers'), /MCP-Protocol-Version/);
    assert.match(r.headers.get('Access-Control-Allow-Headers'), /Mcp-Name/);
    assert.equal(r.headers.get('Access-Control-Allow-Credentials'), null);
});

test('GET: HTML for people, JSON on request, 405 for an SSE stream; DELETE is 405', async () => {
    const html = await handler(new Request(URL_MCP, { headers: { Accept: 'text/html,application/xhtml+xml' } }));
    assert.equal(html.status, 200);
    assert.match(html.headers.get('Content-Type'), /text\/html/);
    const page = await html.text();
    assert.match(page, /claude mcp add --transport http civilkit-calcs https:\/\/civilkit\.io\/mcp/);
    assert.match(page, /calcs_evaluate_sheet/);
    assert.match(page, /stateless/i);
    assert.match(html.headers.get('Content-Security-Policy'), /default-src 'none'/);
    const j = await handler(new Request(`${URL_MCP}/`, { headers: { Accept: 'application/json' } }));
    const body = await j.json();
    assert.equal(body.endpoint, 'https://civilkit.io/mcp');
    assert.ok(body.tools.length >= 11);
    const sse = await handler(new Request(URL_MCP, { headers: { Accept: 'text/event-stream' } }));
    assert.equal(sse.status, 405);
    const del = await handler(new Request(URL_MCP, { method: 'DELETE' }));
    assert.equal(del.status, 405);
    const other = await handler(new Request('https://civilkit.io/mcp/elsewhere'));
    assert.equal(other.status, 404);
});

test('server card', async () => {
    const r = await handler(new Request(`${URL_MCP}/server-card`, { headers: { Accept: 'application/mcp-server-card+json' } }));
    assert.equal(r.status, 200);
    assert.equal(r.headers.get('Content-Type'), 'application/mcp-server-card+json');
    assert.equal(r.headers.get('Access-Control-Allow-Origin'), '*');
    assert.match(r.headers.get('Cache-Control'), /max-age=3600/);
    const card = await r.json();
    assert.equal(card.$schema, 'https://static.modelcontextprotocol.io/schemas/v1/server-card.schema.json');
    assert.match(card.name, /^[^/]+\/[^/]+$/);
    assert.equal(card.remotes[0].type, 'streamable-http');
    assert.equal(card.remotes[0].url, 'https://civilkit.io/mcp');
    const again = await handler(new Request(`${URL_MCP}/server-card`, { headers: { 'If-None-Match': r.headers.get('ETag') } }));
    assert.equal(again.status, 304);
});

test('request size cap', async () => {
    const big = JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'tools/call', params: { name: 'calcs_evaluate_sheet', arguments: { sheet: 'x'.repeat(300 * 1024) } } });
    const r = await raw(big, { 'MCP-Protocol-Version': '2025-06-18' });
    assert.equal(r.status, 413);
    const lying = await handler(new Request(URL_MCP, { method: 'POST', headers: { 'Content-Type': 'application/json', 'Content-Length': '999999999' }, body: '{}' }));
    assert.equal(lying.status, 413);
});

test('rate limits: 429 for requests, a tool error for calculations', async () => {
    let calls = 0;
    const limited = makeApp({ rateLimit: async (kind) => (kind === 'request' ? true : ++calls <= 2) }).handler;
    const conv = (h) => post(h, 'tools/call', { name: 'calcs_convert_units', arguments: { expression: '1 m -> mm' } });
    assert.equal((await conv(limited)).body.result.isError, undefined);
    assert.equal((await conv(limited)).body.result.isError, undefined);
    const third = await conv(limited);
    assert.equal(third.body.result.isError, true);
    assert.match(third.body.result.content[0].text, /Rate limited/);
    // Lookups are not counted as calculations.
    assert.equal((await post(limited, 'tools/call', { name: 'calcs_sheet_syntax', arguments: {} })).body.result.isError, undefined);
    const blocked = makeApp({ rateLimit: async (kind) => kind !== 'request' }).handler;
    const r = await post(blocked, 'ping');
    assert.equal(r.status, 429);
    assert.equal(r.headers.get('Retry-After'), '60');
    assert.ok(r.body.error.code < -32768 || r.body.error.code > -32000, 'not a reserved JSON-RPC code');
});

test('internal errors never carry a stack trace or the input', async () => {
    const app = makeApp();
    app.evaluator.evaluate = () => { throw new Error('secret detail at /some/internal/path.js:12'); };
    const r = await post(app.handler, 'tools/call', { name: 'calcs_evaluate_sheet', arguments: { sheet: 'a: 1 | private note' } });
    assert.equal(r.status, 500);
    assert.equal(r.body.error.code, -32603);
    assert.equal(r.body.error.message, 'Internal error');
    assert.doesNotMatch(JSON.stringify(r.body), /secret|internal\/path|private note/);
});
