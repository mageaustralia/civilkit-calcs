// The Cloudflare Worker: https://civilkit.io/mcp (routes civilkit.io/mcp and civilkit.io/mcp/*). See docs/mcp.md.
// The wasm modules are static imports, so wrangler uploads them as compiled modules (no WebAssembly.compile at run time).
import engineModule from './generated/calcpad_wasm_bg.wasm';
import sheetModule from './generated/calcpad_sheet_wasm_bg.wasm';
import { createApp } from './app.js';

/** The per-IP limiters (wrangler.toml [[ratelimits]]); without the binding a request is let through. */
async function rateLimit(env, kind, request) {
    const limiter = kind === 'eval' ? env.EVAL_LIMITER : env.REQUEST_LIMITER;
    if (!limiter) return true;
    const ip = request.headers.get('CF-Connecting-IP') ?? 'unknown';
    try {
        const { success } = await limiter.limit({ key: `${kind}:${ip}` });
        return success;
    } catch {
        return true; // a limiter outage must not take the server down
    }
}

let app = null;
let bindings = {};

export default {
    async fetch(request, env) {
        bindings = env ?? {};
        app ??= createApp({ engineModule, sheetModule, rateLimit: (kind, req) => rateLimit(bindings, kind, req) });
        try {
            return await app.handler(request);
        } catch {
            return new Response(JSON.stringify({ jsonrpc: '2.0', error: { code: -32603, message: 'Internal error' } }), {
                status: 500,
                headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
            });
        }
    },
};
