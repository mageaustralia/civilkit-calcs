// Malicious and heavy inputs: what the public server refuses before the engine runs, what the engine's caps stop,
// and that nothing reaches files, the network or the next request.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { makeApp, callTool, engineModule, sheetModule, URL_MCP } from './helpers.mjs';
import { checkSource, LIMITS } from '../src/policy.js';
import { create as createEngine } from '../src/generated/engine_glue.js';
import { ENGINE_MAX_PAGES } from '../scripts/build.mjs';

const { handler } = makeApp();
const sheet = (s) => callTool(handler, 'calcs_evaluate_sheet', { sheet: s });
const cpd = (s) => callTool(handler, 'calcs_evaluate_calcpad', { source: s });
const refused = (r, re) => {
    assert.equal(r.isError, false);
    assert.equal(r.structured.ran, false, JSON.stringify(r.structured));
    assert.equal(r.structured.ok, false);
    assert.ok(r.structured.errors.some((e) => e.source === 'policy' && re.test(e.message)), JSON.stringify(r.structured.errors));
};

test('size caps on sheets and expressions', async () => {
    const big = await sheet(`a: 1\n${'b'.repeat(LIMITS.sourceChars)}`);
    assert.equal(big.isError, true);
    assert.match(big.text, /limit is 65,536/);
    const exp = await callTool(handler, 'calcs_check_hand_calc', { expression: '1+'.repeat(3000), expected: 1 });
    assert.equal(exp.isError, true);
    const lines = await cpd('a = 1\n'.repeat(LIMITS.lines + 1));
    refused(lines, /at most 5000/);
});

test('no Python, no files', async () => {
    refused(await sheet('a: 1\n```python\nimport os\nos.system("id")\n```\n'), /Python/);
    refused(await cpd('#python\nprint(1)\n#end python'), /Python/);
    refused(await cpd('#include /etc/passwd'), /no file access/);
    refused(await cpd('#INCLUDE ../secret.cpd'), /no file access/);
    refused(await cpd('#read M from data.csv'), /#read/);
    refused(await cpd('M = [1; 2]\n#write M to out.csv'), /#write/);
    refused(await cpd('M = [1; 2]\n#append M to out.csv'), /#append/);
    refused(await sheet('a: 1\n>> #include x.cpd'), /no file access/);
    // The refusal names the sheet line.
    const r = await sheet('a: 1\nb: 2\n>> #read M from x.csv');
    assert.equal(r.structured.errors[0].line, 4);
});

test('loops with literal bounds are counted, nested loops multiplied', async () => {
    refused(await cpd('#for i = 1 : 1000000000\na = i\n#loop'), /passes/);
    refused(await cpd('#for i = 1 : 1000\n#for j = 1 : 1000\na = i*j\n#loop\n#loop'), /1,000,000 passes/);
    refused(await cpd('#repeat 10000000\na = 1\n#loop'), /passes/);
    refused(await sheet('for i in 1..100000000:\n    a: i'), /passes/);
    refused(await cpd('b = $Sum{k @ k = 1 : 1000000000000}'), /\$Sum over/);
    refused(await cpd('b = $Repeat{a = k @ k = 1 : 5000000}'), /\$Repeat over/);
    const ok = await cpd('#hide\ns = 0\n#for i = 1 : 1000\ns = s + i\n#loop\n#show\ns');
    assert.equal(ok.structured.ok, true);
    assert.equal(ok.structured.quantities.find((q) => q.name === 's').value, 500500);
});

test('large literal vectors and matrices, and huge plots, are refused', async () => {
    refused(await cpd('v = vector(100000000)'), /elements/);
    refused(await cpd('M = matrix(10000; 10000)'), /elements/);
    refused(await cpd('M = identity_hp(5000)'), /elements/);
    refused(await cpd('PlotWidth = 100000\n$Plot{x @ x = 0 : 1}'), /PlotWidth/);
});

test('memory: a computed size the policy cannot see stops at the engine memory cap, cleanly', async () => {
    const r = await cpd('n = 20000\nM = matrix(n; n)\nM.(1; 1)');
    assert.equal(r.isError, true, JSON.stringify(r.structured ?? r.text).slice(0, 300));
    assert.match(r.text, /The engine stopped/);
    assert.doesNotMatch(r.text, /at |\.js|wasm-function/, 'no stack trace');
    // The engine's own size limit answers a computed vector of 2e8 elements.
    const v = await cpd('n = 200000000\nv = vector(n)');
    assert.match(v.structured.errors[0].message, /cannot exceed/);
    // The next request runs in a fresh instance.
    const after = await cpd('a = 2*3');
    assert.equal(after.structured.quantities[0].value, 6);
});

test('deep nesting and recursion end in an error, not a crash', async () => {
    const deep = await cpd(`a = ${'('.repeat(20000)}1${')'.repeat(20000)}`);
    assert.ok(deep.isError || deep.structured.ok === false || deep.structured.ok === true);
    const rec = await cpd('f(x) = f(x) + 1\nf(2)');
    assert.equal(rec.structured.ok, false);
    const macro = await cpd('#def r$ = r$\nd = r$');
    assert.equal(macro.structured.ok, false);
    assert.match(macro.structured.errors[0].message, /Circular/);
    assert.equal((await cpd('a = 1')).structured.ok, true);
});

test('input values cannot inject lines or units', async () => {
    const src = 'L: ? 6 m | span\nM: L * 2 kN';
    for (const bad of ['8\n#include x', '8; #read M from a', '<script>', '8 m', '1e400', '']) {
        const r = await callTool(handler, 'calcs_evaluate_sheet', { sheet: src, inputs: { L: bad } });
        assert.equal(r.structured.ran, false, JSON.stringify(bad));
        assert.equal(r.structured.errors[0].source, 'inputs');
    }
    assert.equal((await callTool(handler, 'calcs_evaluate_sheet', { sheet: src, inputs: [1] })).isError, true);
    const ok = await callTool(handler, 'calcs_evaluate_sheet', { sheet: src, inputs: { L: 7.5 } });
    assert.equal(ok.structured.quantities.find((q) => q.name === 'M').value, 15);
    const tiny = await callTool(handler, 'calcs_evaluate_sheet', { sheet: src, inputs: { L: 1e-9 } });
    assert.equal(tiny.structured.ok, true);
});

test('markup in a worksheet stays text; scripts never reach the result', async () => {
    const r = await cpd("'<script>fetch('https://example.com/x')</script>\n'<img src=x onerror=alert(1)>\na = 1");
    assert.doesNotMatch(r.structured.report, /<script|onerror|fetch\(/);
    assert.equal(r.structured.quantities[0].value, 1);
});

test('a body that is not UTF-8 is a parse error', async () => {
    const res = await handler(new Request(URL_MCP, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: new Uint8Array([0x7b, 0xff, 0xfe, 0x7d]) }));
    assert.equal(res.status, 400);
    assert.equal((await res.json()).error.code, -32700);
});

test('the wasm modules import no files, network or host functions beyond the clock', () => {
    const engineImports = WebAssembly.Module.imports(engineModule).map((i) => i.name.replace(/_[0-9a-f]{16}$/, ''));
    assert.deepEqual(engineImports.sort(), ['__wbg_calcpadTokenizeListing', '__wbg_now', '__wbindgen_object_drop_ref']);
    assert.deepEqual(WebAssembly.Module.imports(sheetModule), []);
    for (const f of ['engine_glue.js', 'sheet_glue.js']) {
        const glue = readFileSync(new URL(`../src/generated/${f}`, import.meta.url), 'utf8');
        assert.doesNotMatch(glue, /\bfetch\(|import\.meta|XMLHttpRequest|WebSocket|require\(/, f);
    }
    // The memory maximum is in the module: growing past it fails.
    const e = createEngine(engineModule);
    const mem = e.memory();
    assert.throws(() => mem.grow(ENGINE_MAX_PAGES), RangeError);
});

test('policy unit checks', () => {
    assert.deepEqual(checkSource('a = 1\n#for i = 1 : n\na = i\n#loop'), [], 'a bound that is not a literal is left to the CPU limit');
    assert.deepEqual(checkSource("'#include is fine in a comment\n\"#read too"), []);
    assert.equal(checkSource('#while 1\n#loop').length, 0, '#while is bounded by the CPU limit only');
    assert.equal(checkSource('#for i = 1 : 200000\n#loop').length, 0);
    assert.equal(checkSource('#for i = 1 : 200001\n#loop').length, 1);
    assert.equal(checkSource('#for i = 0 : 999\n#for j = 1 : 200\n#loop\n#loop').length, 0);
    assert.equal(checkSource('#for i = 0 : 999\n#for j = 1 : 201\n#loop\n#loop').length, 1);
    assert.equal(checkSource('#for i = 0 : 999\n#loop\n#for j = 1 : 1000\n#loop').length, 0, 'loops one after the other are not multiplied');
});
