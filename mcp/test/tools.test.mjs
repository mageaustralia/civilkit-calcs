// Every tool through tools/call, as a client calls it.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { makeApp, callTool, post } from './helpers.mjs';
import { RATE_LIMITS } from '../src/page.js';
import { readFileSync } from 'node:fs';

const { handler } = makeApp();
const call = (name, args) => callTool(handler, name, args);

test('calcs_evaluate_sheet: quantities, errors with sheet lines, warnings, report', async () => {
    const ok = await call('calcs_evaluate_sheet', { sheet: '# Check\nL: 6 m | span\nw: 12 kN/m\nM: w * L^2 / 8 -> kN*m | moment\nif M < 60 kN*m: "OK" else: "Fails"' });
    assert.equal(ok.isError, false);
    assert.equal(ok.structured.ok, true);
    const m = ok.structured.quantities.find((q) => q.name === 'M');
    assert.deepEqual({ value: m.value, unit: m.unit, note: m.note, line: m.line }, { value: 54, unit: 'kN*m', note: 'moment', line: 5 });
    assert.match(ok.structured.report, /^# Check\n/);
    assert.match(ok.structured.report, /\nOK$/);
    assert.deepEqual(JSON.parse(ok.text), ok.structured, 'the text block is the structured result as JSON');
    const bad = await call('calcs_evaluate_sheet', { sheet: '<!--sheet-->\na: 2 m\nb: a + 3 s\nc: nope * 2' });
    assert.equal(bad.isError, false, 'calculation errors are results, not tool failures');
    assert.equal(bad.structured.ok, false);
    assert.deepEqual(bad.structured.errors.map((e) => e.line), [3, 4]);
    assert.match(bad.structured.errors[0].message, /Inconsistent units/);
    assert.match(bad.structured.errors[1].message, /Undefined variable/);
    const warn = await call('calcs_evaluate_sheet', { sheet: 'Why: ? is it' });
    assert.ok(warn.structured.warnings.length >= 1 || warn.structured.errors.length >= 1);
    const empty = await call('calcs_evaluate_sheet', { sheet: '   ' });
    assert.equal(empty.isError, true);
    const wrongType = await call('calcs_evaluate_sheet', { sheet: 5 });
    assert.equal(wrongType.isError, true);
    const dec = await call('calcs_evaluate_sheet', { sheet: 'x: 1/3', decimals: 5 });
    assert.match(dec.structured.quantities[0].text, /0\.33333/);
    assert.equal((await call('calcs_evaluate_sheet', { sheet: 'x: 1', decimals: 99 })).isError, true);
});

test('calcs_evaluate_calcpad', async () => {
    const r = await call('calcs_evaluate_calcpad', { source: "'Beam\nL = 6m\nw = 12kN/m\nM = w*L^2/8|kN*m\n#if M < 60kN*m\n'OK\n#else\n'Fails\n#end if" });
    assert.equal(r.structured.ok, true);
    const m = r.structured.quantities.find((q) => q.name === 'M');
    assert.equal(m.value, 54);
    assert.equal(m.line, 4);
    assert.match(r.structured.report, /OK/);
    assert.doesNotMatch(r.structured.report, /Fails/);
    const err = await call('calcs_evaluate_calcpad', { source: 'a = 1\nb = a + c' });
    assert.equal(err.structured.errors[0].line, 2);
    const sheet = await call('calcs_evaluate_calcpad', { source: '<!--sheet-->\nx: 2 m' });
    assert.equal(sheet.structured.quantities[0].value, 2, 'a sheet marker makes it a sheet');
});

test('calcs_convert_units', async () => {
    const a = await call('calcs_convert_units', { expression: '25 kN/m^2 -> kPa' });
    assert.deepEqual([a.structured.value, a.structured.unit], [25, 'kPa']);
    const b = await call('calcs_convert_units', { expression: '1 ft|mm' });
    assert.equal(b.structured.value, 304.8);
    const c = await call('calcs_convert_units', { expression: '2 h to min' });
    assert.equal(c.structured.value, 120);
    const d = await call('calcs_convert_units', { expression: '5 m -> kg' });
    assert.equal(d.structured.ok, false);
    assert.match(d.structured.errors[0].message, /inconsistent|Inconsistent/);
    assert.equal((await call('calcs_convert_units', { expression: '1 m\n#include x' })).isError, true);
    assert.equal((await call('calcs_convert_units', { expression: 'x'.repeat(5000) })).isError, true);
});

test('calcs_check_hand_calc', async () => {
    const yes = await call('calcs_check_hand_calc', { expression: 'L: 6 m\nw: 12 kN/m\nM: w * L^2 / 8', expected: 54, unit: 'kN*m' });
    assert.equal(yes.structured.within_tolerance, true);
    assert.equal(yes.structured.computed, 54);
    assert.match(yes.structured.verdict, /^Agrees/);
    const no = await call('calcs_check_hand_calc', { expression: '12 kN/m * (6 m)^2 / 8', expected: 56, unit: 'kN*m' });
    assert.equal(no.structured.within_tolerance, false);
    assert.equal(no.structured.ok, false);
    assert.equal(no.structured.difference, 2);
    assert.match(no.structured.verdict, /^Does not agree/);
    const conv = await call('calcs_check_hand_calc', { expression: '12 kN/m * (6 m)^2 / 8', expected: 54000, unit: 'N*m' });
    assert.equal(conv.structured.within_tolerance, true, 'compared in the unit given');
    const tol = await call('calcs_check_hand_calc', { expression: '10 kN', expected: 10.4, unit: 'kN', tolerance: 0.05 });
    assert.equal(tol.structured.within_tolerance, true);
    const units = await call('calcs_check_hand_calc', { expression: '10 kN', expected: 10, unit: 'm' });
    assert.equal(units.structured.ok, false);
    assert.equal(units.structured.within_tolerance, null);
    assert.match(units.structured.verdict, /errors/);
    assert.equal((await call('calcs_check_hand_calc', { expression: '1', expected: 'one' })).isError, true);
    assert.equal((await call('calcs_check_hand_calc', { expression: '1', expected: 1, unit: 'm"; #include x' })).isError, true);
});

test('calcs_search_functions and calcs_function_help', async () => {
    const s = await call('calcs_search_functions', { query: 'square root', limit: 5 });
    assert.ok(s.structured.results.some((r) => r.name === 'sqrt'));
    assert.ok(s.structured.results.length <= 5);
    const e = await call('calcs_search_functions', { query: 'eigenvalues' });
    assert.ok(e.structured.results.some((r) => /eigen/i.test(r.name + r.description)));
    const h = await call('calcs_function_help', { name: 'lsolve' });
    assert.equal(h.structured.entries[0].signature, 'lsolve(A; b)');
    assert.deepEqual(h.structured.entries[0].parameters.map((p) => p.name), ['A', 'b']);
    assert.ok(h.structured.entries[0].documentation.length > 20);
    assert.equal((await call('calcs_function_help', { name: 'sqrt(x)' })).structured.entries[0].name, 'sqrt');
    assert.ok((await call('calcs_function_help', { name: '$Root' })).structured.entries.length >= 1);
    const miss = await call('calcs_function_help', { name: 'sqrtt' });
    assert.equal(miss.isError, true);
    assert.match(miss.text, /Did you mean/);
});

test('calcs_search_docs and calcs_get_guide_page', async () => {
    const s = await call('calcs_search_docs', { query: 'percentage of a ratio', limit: 3 });
    assert.ok(s.structured.results.length >= 1);
    const first = s.structured.results[0];
    assert.match(first.url, /^https:\/\/civilkit\.io\/calcs\/guide\/[a-z/-]*(#[a-z0-9-]+)?$/);
    assert.doesNotMatch(first.url, /\.html|\.md/);
    assert.ok(first.snippet.length > 20);
    assert.doesNotMatch(first.snippet, /<!--/);
    const t = await call('calcs_search_docs', { query: 'templates with inputs slider' });
    assert.ok(t.structured.results.some((r) => r.page === 'templates' || r.page === 'sheet-syntax'));
    for (const key of ['units', 'units.md', 'https://civilkit.io/calcs/guide/units', 'Units']) {
        const g = await call('calcs_get_guide_page', { page: key });
        assert.equal(g.structured.page, 'units', key);
        assert.equal(g.structured.url, 'https://civilkit.io/calcs/guide/units');
        assert.match(g.structured.markdown, /^# Units/);
    }
    assert.equal((await call('calcs_get_guide_page', { page: 'sheet/' })).structured.url, 'https://civilkit.io/calcs/guide/sheet/');
    assert.equal((await call('calcs_get_guide_page', { page: '' })).isError, true);
    assert.equal((await call('calcs_get_guide_page', { page: 'introduction' })).structured.url, 'https://civilkit.io/calcs/guide/');
    const nope = await call('calcs_get_guide_page', { page: 'nope' });
    assert.equal(nope.isError, true);
    assert.match(nope.text, /Pages: introduction/);
});

test('calcs_list_samples, calcs_get_sample and calcs_sheet_syntax', async () => {
    const all = await call('calcs_list_samples', {});
    assert.ok(all.structured.count >= 40);
    assert.ok(all.structured.groups.includes('Templates'));
    const t = await call('calcs_list_samples', { group: 'templates' });
    assert.ok(t.structured.samples.every((s) => s.group === 'Templates' && s.has_inputs));
    assert.equal((await call('calcs_list_samples', { group: 'nope' })).isError, true);
    const s = await call('calcs_get_sample', { id: 'sheet-beam-bending' });
    assert.equal(s.structured.app_url, 'https://civilkit.io/calcs/?sample=sheet-beam-bending');
    assert.match(s.structured.sheet, /^<!--sheet-->\n# Beam bending check/);
    assert.equal((await call('calcs_get_sample', { id: 'Beam bending.sheet' })).structured.id, 'sheet-beam-bending');
    assert.equal((await call('calcs_get_sample', { id: 'simply-supported-beam' })).structured.id, 'template-simply-supported-beam');
    assert.equal((await call('calcs_get_sample', { id: 'nope' })).isError, true);
    const syn = await call('calcs_sheet_syntax', {});
    assert.match(syn.structured.markdown, /^# Sheet syntax/);
    assert.ok(syn.structured.markdown.length > 20000);
});

test('outputSchema: required fields are present in the structured results', async () => {
    const tools = (await post(handler, 'tools/list')).body.result.tools;
    const examples = {
        calcs_evaluate_sheet: { sheet: 'a: 1 m' },
        calcs_evaluate_calcpad: { source: 'a = 1' },
        calcs_convert_units: { expression: '1 m -> mm' },
        calcs_check_hand_calc: { expression: '1 m', expected: 1, unit: 'm' },
    };
    for (const t of tools.filter((x) => x.outputSchema)) {
        const r = await call(t.name, examples[t.name]);
        for (const k of t.outputSchema.required) assert.ok(k in r.structured, `${t.name}.${k}`);
    }
});

test('the rate limits shown on the page are the ones in wrangler.toml', () => {
    const toml = readFileSync(new URL('../wrangler.toml', import.meta.url), 'utf8');
    const top = toml.split('[env.')[0];
    assert.match(top, new RegExp(`name = "REQUEST_LIMITER"[\\s\\S]*?limit = ${RATE_LIMITS.requestsPerMinute}, period = 60`));
    assert.match(top, new RegExp(`name = "EVAL_LIMITER"[\\s\\S]*?limit = ${RATE_LIMITS.calculationsPerMinute}, period = 60`));
});
