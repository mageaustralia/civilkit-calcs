// Shared by the tests: the app built from the same generated files the Worker bundles (src/generated, made by
// scripts/build.mjs), with the wasm compiled by Node instead of by wrangler. The handler is the Worker's.
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createApp } from '../src/app.js';

const gen = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'generated');
export const engineModule = new WebAssembly.Module(readFileSync(join(gen, 'calcpad_wasm_bg.wasm')));
export const sheetModule = new WebAssembly.Module(readFileSync(join(gen, 'calcpad_sheet_wasm_bg.wasm')));

export function makeApp(opts = {}) {
    return createApp({ engineModule, sheetModule, ...opts });
}

export const MODERN = '2026-07-28';
export const URL_MCP = 'https://civilkit.io/mcp';

let seq = 0;

/** POSTs one JSON-RPC message. era 'legacy' (default) sends MCP-Protocol-Version 2025-06-18; 'modern' adds the _meta and headers. */
export async function post(handler, method, params = {}, { era = 'legacy', id = ++seq, headers = {}, notification = false } = {}) {
    const msg = { jsonrpc: '2.0', method, params: { ...params } };
    if (!notification) msg.id = id;
    const h = { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream', ...headers };
    if (era === 'modern') {
        msg.params._meta = {
            'io.modelcontextprotocol/protocolVersion': MODERN,
            'io.modelcontextprotocol/clientInfo': { name: 'test', version: '1' },
            'io.modelcontextprotocol/clientCapabilities': {},
            ...(params._meta ?? {}),
        };
        const def = (k, v) => { if (!(k in h)) h[k] = v; };
        def('MCP-Protocol-Version', MODERN);
        def('Mcp-Method', method);
        if (['tools/call', 'prompts/get'].includes(method)) def('Mcp-Name', params.name);
        if (method === 'resources/read') def('Mcp-Name', params.uri);
    } else if (era === 'legacy') {
        if (!('MCP-Protocol-Version' in h)) h['MCP-Protocol-Version'] = '2025-06-18';
    }
    for (const [k, v] of Object.entries(h)) if (v === null) delete h[k];
    const res = await handler(new Request(URL_MCP, { method: 'POST', headers: h, body: JSON.stringify(msg) }));
    const text = await res.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch { body = text; }
    return { status: res.status, headers: res.headers, body };
}

/** Calls a tool (legacy era) and returns { result, structured, isError, text }. */
export async function callTool(handler, name, args, opts = {}) {
    const r = await post(handler, 'tools/call', { name, arguments: args }, opts);
    if (r.body?.error) return { error: r.body.error, status: r.status };
    const result = r.body.result;
    return { result, structured: result.structuredContent, isError: Boolean(result.isError), text: result.content?.[0]?.text ?? '', status: r.status };
}
