// Results match the app: the samples and the guide's own expected results, through the Worker's code path
// (compile -> policy -> engine -> extraction), with the same wasm files the Worker bundles.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { makeApp, callTool } from './helpers.mjs';

const { handler, evaluator, data } = makeApp();
const book = join(dirname(fileURLToPath(import.meta.url)), '..', '..', 'book', 'src');

const sample = (id) => data.samples.samples.find((s) => s.id === id).text;
const q = (r, name) => {
    const x = r.quantities.find((v) => v.name === name);
    assert.ok(x, `quantity ${name} in ${r.quantities.map((v) => v.name).join(', ')}`);
    return x;
};
const near = (a, b, tol = 1e-9) => assert.ok(Math.abs(a - b) <= tol * Math.max(1, Math.abs(b)), `${a} is not ${b}`);

test('beam bending sample: M* = 54 kN·m, capacity 332.1 kN·m, utilisation 0.163', async () => {
    const r = (await callTool(handler, 'calcs_evaluate_sheet', { sheet: sample('sheet-beam-bending') })).structured;
    assert.equal(r.ok, true);
    const m = q(r, 'Mstar');
    assert.equal(m.value, 54);
    assert.equal(m.unit, 'kN*m');
    assert.equal(m.symbol, 'M_star');
    assert.equal(m.line, 15);
    assert.equal(m.note, 'maximum bending moment at mid-span');
    assert.match(m.text, /= 54 kN·m$/);
    near(q(r, 'phiMs').value, 332.1);
    near(q(r, 'utilisation').value, 0.16260162601626);
    assert.match(r.report, /OK: the beam passes in bending\./);
    assert.doesNotMatch(r.report, /Fails/);
    assert.match(r.report, /^# Beam bending check/);
});

test('gable frame sample: thrust 38.9 kN pinned, 48.6 kN rigid', () => {
    const r = evaluator.evaluate({ source: sample('sheet-gable-frame-thrust') });
    assert.equal(r.ok, true);
    assert.equal(q(r, 'H_pin').unit, 'kN');
    assert.equal(q(r, 'H_pin').value.toFixed(1), '38.9');
    assert.equal(q(r, 'H_rigid').value.toFixed(1), '48.6');
    assert.equal(q(r, 'ratio').value.toFixed(2), '1.25');
});

test('principal stresses sample: 85 and -45 MPa, tau max 65 MPa, vectors and matrices as arrays', () => {
    const r = evaluator.evaluate({ source: sample('sheet-principal-stresses') });
    assert.equal(r.ok, true);
    const s1 = r.quantities.find((x) => x.symbol === 'σ_1' || x.name === 'σ_1');
    const s2 = r.quantities.find((x) => x.symbol === 'σ_2' || x.name === 'σ_2');
    near(s1.value, 85);
    near(s2.value, -45);
    assert.equal(s1.unit, 'MPa');
    near(r.quantities.find((x) => x.symbol === 'τ_max' || x.name === 'τ_max').value, 65);
    const S = r.quantities.find((x) => x.name === 'S');
    assert.deepEqual(S.value, [[80, 25], [25, -40]]);
    assert.equal(S.unit, 'MPa');
    const sp = r.quantities.find((x) => x.symbol === 'σ_p' || x.name === 'σ_p');
    assert.deepEqual(sp.value, [-45, 85]);
});

test('template inputs: defaults, and overrides by the sheet name with or without a unit', () => {
    const src = sample('template-simply-supported-beam');
    const base = evaluator.evaluate({ source: src });
    assert.equal(base.ok, true);
    assert.equal(q(base, 'L').input, true);
    assert.equal(q(base, 'L').value, 6);
    const longer = evaluator.evaluate({ source: src, inputs: { L: 8 } });
    assert.equal(longer.ok, true);
    assert.equal(q(longer, 'L').value, 8);
    assert.ok(q(longer, 'M_max').value > q(base, 'M_max').value);
    near(q(evaluator.evaluate({ source: src, inputs: { L: '8' } }), 'M_max').value, q(longer, 'M_max').value, 1e-12);
    const withUnit = evaluator.evaluate({ source: src, inputs: { L: '8 m' } });
    assert.equal(withUnit.ran, false);
    assert.match(withUnit.errors[0].message, /Input "L": a number in the unit of its default \(6m\)/);
    // Choices by the option's name or value, a checkbox by true/false.
    const timber = evaluator.evaluate({ source: src, inputs: { mat: 'Timber', h: '400 mm deep', sw: false, limit: 'L/500' } });
    assert.equal(timber.ok, true, JSON.stringify(timber.errors));
    assert.equal(q(timber, 'E').value, 11);
    assert.equal(q(timber, 'h').value, 400);
    assert.equal(q(timber, 'sw').value, 0);
    assert.equal(q(timber, 'limit').value, 500);
    assert.equal(q(evaluator.evaluate({ source: src, inputs: { h: '200mm' } }), 'h').value, 200);
    const badChoice = evaluator.evaluate({ source: src, inputs: { mat: 'Concrete' } });
    assert.match(badChoice.errors[0].message, /"Structural steel" \(1\), "Aluminium" \(2\), "Timber" \(3\)/);
    // The value is taken in the unit of the default, as the app's input form does; a slider's range is checked.
    const outside = evaluator.evaluate({ source: src, inputs: { L: 8000 } });
    assert.equal(outside.ok, false);
    assert.match(outside.errors[0].message, /outside the range 2 to 12/);
    assert.equal(outside.errors[0].line, 11);
    const unknown = evaluator.evaluate({ source: src, inputs: { Span: 8 } });
    assert.equal(unknown.ok, false);
    assert.match(unknown.errors[0].message, /"Span" is not an input.*Inputs: L, w, h, mat, limit, sw\./);
});

test('every sheet sample without Python runs without errors; Python samples are refused, not run', () => {
    let ran = 0;
    for (const s of data.samples.samples) {
        const r = evaluator.evaluate({ source: s.text });
        if (s.python) {
            assert.equal(r.ran, false, s.id);
            assert.ok(r.errors.every((e) => e.source === 'policy'), s.id);
            continue;
        }
        assert.equal(r.ok, true, `${s.id}: ${JSON.stringify(r.errors)}`);
        assert.ok(r.quantities.length > 0, s.id);
        ran++;
    }
    assert.ok(ran >= 30, `${ran} samples ran`);
});

// The guide's examples carry `<!-- expect: a | b -->` lines that book/check-examples.mjs checks against the engine;
// the same fragments must be in what the Worker's engine path renders (flattened the same way as there).
const flatText = (html) => html.replace(/<(script|style)[\s\S]*?<\/\1>/g, '').replace(/<sup>([\s\S]*?)<\/sup>/g, '^$1').replace(/<sub>([\s\S]*?)<\/sub>/g, '_$1').replace(/<svg[\s\S]*?<\/svg>/g, '[drawing]').replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;/g, ' ').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();
const walk = (d) => readdirSync(d).flatMap((n) => (statSync(join(d, n)).isDirectory() ? walk(join(d, n)) : n.endsWith('.md') ? [join(d, n)] : []));

test('the guide examples with expected results give those results', () => {
    let checked = 0;
    let skipped = 0;
    const failures = [];
    for (const md of walk(book)) {
        const src = readFileSync(md, 'utf8').split('\n');
        for (let i = 0; i < src.length; i++) {
            const m = /^(`{3,})(sheet|cpd)\s*$/.exec(src[i]);
            if (!m) continue;
            const start = i + 1;
            let j = start;
            while (j < src.length && !src[j].startsWith(m[1])) j++;
            const body = src.slice(start, j).join('\n') + '\n';
            i = j;
            let expect = null;
            for (let k = start - 2; k >= 0 && /^\s*$|^<details>|^<summary>|^<!--\s*(sample|expect):/.test(src[k]); k--) {
                const mm = /^<!--\s*expect:\s*(.*?)\s*-->\s*$/.exec(src[k]);
                if (mm) expect = mm[1];
            }
            if (!expect) continue;
            const r = evaluator.evaluate({ source: body, kind: m[2] === 'sheet' ? 'sheet' : 'calcpad', includeHtml: true });
            if (!r.ran) { skipped++; continue; } // refused by the server's policy (#read, Python): not run here
            const got = flatText(r.html).replace(/\s+/g, '');
            for (const frag of expect.split('|')) {
                checked++;
                if (!got.includes(frag.replace(/\s+/g, ''))) failures.push(`${md.replace(book, '')}:${start} expected "${frag.trim()}"`);
            }
        }
    }
    assert.deepEqual(failures, []);
    assert.ok(checked >= 50, `${checked} fragments checked, ${skipped} examples skipped`);
});
