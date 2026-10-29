// A small MCP client over Streamable HTTP, for the end-to-end test and for trying the server by hand:
//   node test/client.mjs https://civilkit.io/mcp            initialize, list the tools, evaluate the beam sample
// It speaks the handshake era (initialize, then requests with MCP-Protocol-Version) and the 2026-07-28 era
// (per-request _meta and the Mcp-Method / Mcp-Name headers).

export class McpClient {
    constructor(url, { era = 'legacy', version } = {}) {
        this.url = url;
        this.era = era;
        this.version = version ?? (era === 'modern' ? '2026-07-28' : '2025-06-18');
        this.id = 0;
    }

    async send(method, params = {}, { notification = false, headers = {} } = {}) {
        const msg = { jsonrpc: '2.0', method, params: { ...params } };
        if (!notification) msg.id = ++this.id;
        const h = { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream', ...headers };
        if (this.era === 'modern') {
            msg.params._meta = {
                'io.modelcontextprotocol/protocolVersion': this.version,
                'io.modelcontextprotocol/clientInfo': { name: 'civilkit-e2e', version: '1.0.0' },
                'io.modelcontextprotocol/clientCapabilities': {},
            };
            h['MCP-Protocol-Version'] ??= this.version;
            h['Mcp-Method'] ??= method;
            if (method === 'tools/call' || method === 'prompts/get') h['Mcp-Name'] ??= params.name;
            if (method === 'resources/read') h['Mcp-Name'] ??= params.uri;
        } else if (method !== 'initialize') {
            h['MCP-Protocol-Version'] ??= this.version;
        }
        const res = await fetch(this.url, { method: 'POST', headers: h, body: JSON.stringify(msg) });
        const text = await res.text();
        let body = null;
        if (text) {
            // A server may answer with an SSE stream; take the last data line.
            if ((res.headers.get('Content-Type') ?? '').startsWith('text/event-stream')) {
                const data = text.split('\n').filter((l) => l.startsWith('data:')).pop();
                body = data ? JSON.parse(data.slice(5)) : null;
            } else {
                body = JSON.parse(text);
            }
        }
        return { status: res.status, headers: res.headers, body };
    }

    async initialize() {
        const r = await this.send('initialize', { protocolVersion: this.version, capabilities: {}, clientInfo: { name: 'civilkit-e2e', version: '1.0.0' } });
        if (r.body?.result?.protocolVersion) this.version = r.body.result.protocolVersion;
        await this.send('notifications/initialized', {}, { notification: true });
        return r;
    }

    async call(name, args) {
        const r = await this.send('tools/call', { name, arguments: args });
        if (r.body?.error) throw new Error(`${name}: ${r.body.error.code} ${r.body.error.message}`);
        return r.body.result;
    }
}

if (process.argv[1] && process.argv[1].endsWith('client.mjs')) {
    const url = process.argv[2] ?? 'http://127.0.0.1:8787/mcp';
    const c = new McpClient(url);
    const init = await c.initialize();
    console.log('initialize:', init.status, init.body.result.serverInfo.name, init.body.result.protocolVersion);
    const tools = (await c.send('tools/list')).body.result.tools;
    console.log('tools:', tools.map((t) => t.name).join(', '));
    const sample = (await c.call('calcs_get_sample', { id: 'sheet-beam-bending' })).structuredContent;
    const r = (await c.call('calcs_evaluate_sheet', { sheet: sample.sheet })).structuredContent;
    for (const q of r.quantities) console.log(`  ${q.name} = ${q.value} ${q.unit}`);
}
