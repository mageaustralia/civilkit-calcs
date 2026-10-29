// End to end: starts the Worker with `wrangler dev --local` (workerd, the Workers runtime, with the wasm bundled as
// wrangler bundles it for deploy) on a free port, and speaks MCP to it over HTTP with test/client.mjs.
//   node test/e2e.mjs                   start, test, stop
//   E2E_URL=http://host/mcp node test/e2e.mjs   test a server that is already running (no rate limit check)
import { spawn } from 'node:child_process';
import { mkdtempSync, mkdirSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { McpClient } from './client.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function freePort() {
    return new Promise((resolve, reject) => {
        const s = createServer();
        s.unref();
        s.on('error', reject);
        s.listen(0, '127.0.0.1', () => {
            const { port } = s.address();
            s.close(() => resolve(port));
        });
    });
}

async function startWorker() {
    const port = await freePort();
    // Fresh local state each run, so the simulated rate limiter starts from zero.
    mkdirSync(join(root, '.wrangler'), { recursive: true });
    const state = mkdtempSync(join(root, '.wrangler', 'e2e-'));
    const bin = join(root, 'node_modules', '.bin', process.platform === 'win32' ? 'wrangler.cmd' : 'wrangler');
    const child = spawn(bin, ['dev', '--local', '--env', 'e2e', '--ip', '127.0.0.1', '--port', String(port), '--inspector-port', String(await freePort()), '--persist-to', state, '--show-interactive-dev-session=false', '--log-level', 'warn'], {
        cwd: root,
        env: { ...process.env, WRANGLER_SEND_METRICS: 'false', CI: '1' },
        stdio: ['ignore', 'pipe', 'pipe'],
    });
    let log = '';
    child.stdout.on('data', (d) => { log += d; });
    child.stderr.on('data', (d) => { log += d; });
    const url = `http://127.0.0.1:${port}/mcp`;
    for (let i = 0; i < 120; i++) {
        if (child.exitCode !== null) throw new Error(`wrangler dev exited:\n${log}`);
        try {
            const r = await fetch(url, { headers: { Accept: 'application/json' } });
            if (r.ok) return { url, stop: () => { child.kill('SIGTERM'); try { rmSync(state, { recursive: true, force: true }); } catch { /* left for the next run */ } }, log: () => log };
        } catch { /* not up yet */ }
        await new Promise((r) => setTimeout(r, 500));
    }
    child.kill('SIGTERM');
    throw new Error(`wrangler dev did not answer on ${url}:\n${log}`);
}

const steps = [];
const step = async (name, fn) => {
    const t = Date.now();
    await fn();
    steps.push(name);
    console.log(`ok - ${name} (${Date.now() - t} ms)`);
};

const external = process.env.E2E_URL;
const worker = external ? { url: external, stop() {}, log: () => '' } : await startWorker();
console.log(`# MCP server at ${worker.url}`);
let failed = false;
try {
    const c = new McpClient(worker.url);

    await step('GET /mcp is the page for people, with the endpoint and the Claude Code command', async () => {
        const r = await fetch(worker.url, { headers: { Accept: 'text/html' } });
        assert.equal(r.status, 200);
        const html = await r.text();
        assert.match(html, /claude mcp add --transport http civilkit-calcs https:\/\/civilkit\.io\/mcp/);
        const card = await fetch(`${worker.url}/server-card`);
        assert.equal((await card.json()).remotes[0].url, 'https://civilkit.io/mcp');
    });

    await step('initialize (2025-06-18) and notifications/initialized', async () => {
        const r = await c.initialize();
        assert.equal(r.status, 200);
        assert.equal(r.body.result.protocolVersion, '2025-06-18');
        assert.equal(r.body.result.serverInfo.name, 'civilkit-calcs');
        assert.equal(r.headers.get('Mcp-Session-Id'), null);
    });

    await step('tools/list has the 11 tools with read-only annotations', async () => {
        const tools = (await c.send('tools/list')).body.result.tools;
        assert.equal(tools.length, 11);
        assert.ok(tools.every((t) => t.annotations.readOnlyHint === true && t.annotations.openWorldHint === false));
    });

    await step('the 2026-07-28 era: server/discover and a call with per-request metadata', async () => {
        const m = new McpClient(worker.url, { era: 'modern' });
        const d = await m.send('server/discover');
        assert.equal(d.status, 200);
        assert.equal(d.body.result.resultType, 'complete');
        const r = await m.call('calcs_check_hand_calc', { expression: '12 kN/m * (6 m)^2 / 8', expected: 54, unit: 'kN*m' });
        assert.equal(r.structuredContent.within_tolerance, true);
        const bad = await m.send('tools/list', {}, { headers: { 'Mcp-Method': 'tools/call' } });
        assert.equal(bad.status, 400);
        assert.equal(bad.body.error.code, -32020);
    });

    let beam;
    await step('calcs_get_sample returns the beam sample and its app link', async () => {
        const s = (await c.call('calcs_get_sample', { id: 'sheet-beam-bending' })).structuredContent;
        assert.equal(s.app_url, 'https://civilkit.io/calcs/?sample=sheet-beam-bending');
        beam = s.sheet;
    });

    await step('calcs_evaluate_sheet on the beam sample gives M* = 54 kN·m in the Workers runtime', async () => {
        const r = await c.call('calcs_evaluate_sheet', { sheet: beam });
        const q = r.structuredContent.quantities.find((x) => x.name === 'Mstar');
        assert.equal(q.value, 54);
        assert.equal(q.unit, 'kN*m');
        assert.match(r.structuredContent.report, /OK: the beam passes in bending/);
        const gable = (await c.call('calcs_get_sample', { id: 'sheet-gable-frame-thrust' })).structuredContent.sheet;
        const g = (await c.call('calcs_evaluate_sheet', { sheet: gable })).structuredContent;
        assert.equal(g.quantities.find((x) => x.name === 'H_pin').value.toFixed(1), '38.9');
        assert.equal(g.quantities.find((x) => x.name === 'H_rigid').value.toFixed(1), '48.6');
    });

    await step('calcs_search_docs finds the units chapter', async () => {
        const r = (await c.call('calcs_search_docs', { query: 'convert a ratio to a percentage' })).structuredContent;
        assert.ok(r.results.some((x) => x.url.startsWith('https://civilkit.io/calcs/guide/units')));
    });

    await step('policy refusal and engine memory cap answer cleanly in workerd', async () => {
        const p = (await c.call('calcs_evaluate_calcpad', { source: '#include x.cpd' })).structuredContent;
        assert.equal(p.ran, false);
        const m = await c.call('calcs_evaluate_calcpad', { source: 'n = 20000\nM = matrix(n; n)\nM.(1; 1)' });
        assert.equal(m.isError, true);
        assert.match(m.content[0].text, /The engine stopped/);
        const after = (await c.call('calcs_convert_units', { expression: '1 in -> mm' })).structuredContent;
        assert.equal(after.value, 25.4);
    });

    if (!external) {
        await step('rate limit: the e2e calculation limit (8 a minute, simulated by wrangler dev) answers with a tool error', async () => {
            // 6 calculations were made above (check, 2 evaluate_sheet, 2 evaluate_calcpad, convert). The limiter counts
            // approximately, so: within 4 more, calls start to be refused, and stay refused.
            const results = [];
            for (let i = 0; i < 4; i++) results.push(await c.call('calcs_convert_units', { expression: '1 m -> mm' }));
            const limited = results.map((r) => r.isError === true);
            const first = limited.indexOf(true);
            assert.ok(first >= 0, 'refused within 4 calls');
            assert.ok(limited.slice(first).every(Boolean), 'and stays refused');
            assert.match(results[first].content[0].text, /Rate limited/);
            console.log(`  (${first} more calculations passed before the limit)`);
            const lookup = await c.call('calcs_sheet_syntax', {});
            assert.notEqual(lookup.isError, true, 'lookups are not calculations');
        });
    }
} catch (e) {
    failed = true;
    console.error(`not ok - ${e.message}`);
    if (e.stack) console.error(e.stack.split('\n').slice(1, 4).join('\n'));
    const log = worker.log();
    if (log) console.error(`--- wrangler output ---\n${log.slice(-4000)}`);
} finally {
    worker.stop();
}
console.log(`# e2e: ${steps.length} passed${failed ? ', 1 failed' : ''}`);
process.exit(failed ? 1 : 0);
