// Runs every ```sheet python example of the guide's "Interactive panels" chapter (book/src/python-panels.md) in the built app
// served at --url, the way a reader would, so the chapter stays true: the sheet is put in the editor, "Run Python" is answered,
// and then, for each example,
//   - one panel is drawn per `@ui.panel`, with one control per `ui.slider/select/checkbox/player` call of the block
//     (a widget passed to two panels is drawn in each),
//   - a chart is drawn when the block makes a `go.Figure` (a WebGL canvas for a 3D trace), a table for `ui.table`,
//   - no Python error and no panel error is shown,
//   - moving the first control of each panel changes what the panel shows.
// It also checks the facts the chapter's "Limits" section states (a reused widget is not shared, the missing numpy and math
// functions, a long list literal).
//   node e2e/panels-guide-check.mjs [--url http://127.0.0.1:5201/]
// The chart checks load Plotly.js from its CDN.
import { chromium } from 'playwright';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5201/');
const md = join(dirname(fileURLToPath(import.meta.url)), '../../book/src/python-panels.md');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

// The examples: each ````sheet python fence of the chapter.
const examples = [];
{
    const src = readFileSync(md, 'utf8').split('\n');
    for (let i = 0; i < src.length; i++) {
        const m = /^(`{3,})sheet python\s*$/.exec(src[i]);
        if (!m) continue;
        let j = i + 1;
        while (j < src.length && src[j] !== m[1]) j++;
        examples.push({ line: i + 2, text: src.slice(i + 1, j).join('\n') + '\n' });
        i = j;
    }
}

const browser = await chromium.launch();
const pageErrors = [];
let page;
/** A new tab: the values panels were left at are kept per tab, so a check of starting values needs a fresh one. */
async function freshPage() {
    if (page) await page.context().close();
    page = await (await browser.newContext({ viewport: { width: 1300, height: 1000 } })).newPage();
    page.on('pageerror', (e) => { if (!/^Canceled/.test(String(e))) pageErrors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 90000 });
}
await freshPage();

const waitFor = async (fn, ms = 20000) => { const t0 = Date.now(); while (Date.now() - t0 < ms) { if (await fn().catch(() => false)) return true; await new Promise((r) => setTimeout(r, 200)); } return false; };

/** Puts the sheet in the editor, answers the Python prompt and returns the results frame once `ready(frame)` holds. */
async function run(text, ready, ms = 90000) {
    // a blank sheet first, so that the frame of the last example cannot be taken for this one
    await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), '<!--sheet-->\n# Blank\n');
    await waitFor(async () => {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            if (await f.locator('.cprs-panel').count().catch(() => 0) || /Python|Fact/.test(await f.locator('body').innerText().catch(() => ''))) return false;
        }
        return true;
    }, 30000);
    await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), /^\s*<!--sheet-->/.test(text) ? text : '<!--sheet-->\n' + text);
    // the frame of this sheet: it shows the sheet's title
    const title = (/^#\s+(.+)$/m.exec(text) ?? [, ''])[1].trim();
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        const yes = page.locator('#cprs-python-yes');
        if (await yes.count().catch(() => 0)) await yes.first().click().catch(() => {});
        for (const f of page.frames()) if (f !== page.mainFrame() && (await f.locator('body').innerText().catch(() => '')).includes(title) && await ready(f).catch(() => false)) return f;
        await page.waitForTimeout(400);
    }
    return null;
}

/** What a panel shows: its text, its tables and the data of its charts. */
const shown = (fr, k) => fr.evaluate((k) => {
    const p = document.querySelectorAll('.cprs-panel')[k];
    const out = p.querySelector('.cprs-panel-out');
    const figs = [...out.querySelectorAll('.js-plotly-plot')].map((g) => JSON.stringify(g.data));
    return out.innerText + '|' + figs.join('|');
}, k);

/** Moves the first control of panel k: a slider to its far end, a drop-down to its next option, a checkbox over. */
const move = (fr, k) => fr.evaluate((k) => {
    const p = document.querySelectorAll('.cprs-panel')[k];
    const el = p.querySelector('.cprs-panel-ui input, .cprs-panel-ui select');
    if (el.type === 'range') { el.value = Number(el.value) === Number(el.max) ? el.min : el.max; el.dispatchEvent(new Event('input', { bubbles: true })); }
    else if (el.type === 'checkbox') { el.checked = !el.checked; el.dispatchEvent(new Event('change', { bubbles: true })); }
    else { el.selectedIndex = (el.selectedIndex + 1) % el.options.length; el.dispatchEvent(new Event('change', { bubbles: true })); }
}, k);

for (const ex of examples) {
    const where = `python-panels.md:${ex.line}`;
    const py = (/```python\n([\s\S]*?)^```/m.exec(ex.text) ?? [, ''])[1];
    const panels = (py.match(/^\s*@ui\.panel\(/gm) ?? []).length;
    const panelArgs = [...py.matchAll(/^\s*@ui\.panel\(([^)]*)\)/gm)].map((m) => m[1].split(',').filter((s) => s.trim()).length);
    const widgets = panelArgs.reduce((a, b) => a + b, 0);
    const chart = /go\.Figure\(/.test(py);
    const three = /go\.(Surface|Scatter3d|Mesh3d)\(/.test(py);
    const table = /ui\.table\(/.test(py);
    const fr = await run(ex.text, async (f) => (await f.locator('.cprs-panel').count()) >= panels && (await f.locator('.cprs-panel .cprs-w').count()) >= widgets);
    check(!!fr, `${where}: ${panels} panel(s) with ${widgets} control(s) drawn`);
    if (!fr) continue;
    const body = await fr.locator('body').innerText();
    check(!/Python error/.test(body), `${where}: no Python error${/Python error/.test(body) ? ': ' + body.match(/Python error.{0,160}/)[0] : ''}`);
    check(await fr.locator('.cprs-w').count() === widgets, `${where}: exactly ${widgets} control(s)`);
    if (chart) check(await waitFor(() => fr.locator('.cprs-panel .js-plotly-plot').count().then((n) => n > 0), 45000), `${where}: the chart is drawn`);
    if (three) check(await waitFor(() => fr.locator('.cprs-panel .js-plotly-plot canvas').count().then((n) => n > 0), 30000), `${where}: the 3D chart has a WebGL canvas`);
    if (table) check(await fr.locator('.cprs-panel-table').count() > 0, `${where}: the table is drawn`);
    for (let k = 0; k < panels; k++) {
        const before = await shown(fr, k);
        await move(fr, k);
        check(await waitFor(async () => (await shown(fr, k)) !== before), `${where}: moving the first control of panel ${k + 1} redraws it`);
    }
    check(await fr.locator('.cprs-panel-err').count() === 0, `${where}: no panel error`);
}
check(examples.length >= 5, `${examples.length} examples found in the chapter`);

// ---- the facts the chapter states ---------------------------------------------------------------------------------------------
{
    // the beam example's numbers and its print note, as the chapter quotes them
    await freshPage();
    const beam = examples.find((e) => e.text.includes('# Beam moment explorer'));
    const fr = beam && await run(beam.text, async (f) => (await f.locator('.cprs-panel .cprs-w').count()) >= 2);
    const txt = () => fr.locator('.cprs-panel-text').first().innerText();
    check(!!fr && (await txt()) === 'Maximum moment 54.0 kN.m at midspan', 'beam: starts at Maximum moment 54.0 kN.m at midspan');
    if (fr) {
        await fr.evaluate(() => { const el = document.querySelector('.cprs-panel-ui input[type=range]'); el.value = 10; el.dispatchEvent(new Event('input', { bubbles: true })); });
        check(await waitFor(async () => (await txt()) === 'Maximum moment 150.0 kN.m at midspan'), 'beam: Span 10 m gives Maximum moment 150.0 kN.m at midspan');
        const html = await page.evaluate(async (content) => {
            const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, includeLineAnchors: false }) });
            return new TextDecoder().decode(await r.arrayBuffer());
        }, '<!--sheet-->\n' + beam.text);
        check(html.includes('Interactive panel moment: Span = 10 m, Load = 12 kN/m. It is drawn in the browser; print and export show it at these values.'), 'beam: print carries the note with the values the sliders were left at');
        check(html.includes('Maximum moment 150.0 kN.m at midspan'), 'beam: and the text at those values');
    }
    const wave = examples.find((e) => e.text.includes('# Travelling wave'));
    const fw = wave && await run(wave.text, async (f) => (await f.locator('.cprs-panel .cprs-w').count()) >= 1);
    check(!!fw && await fw.locator('.cprs-player button', { hasText: 'Play' }).count() === 1, 'player: a Play button is drawn');
}
let factN = 0;
const fence = (py) => `# Fact ${++factN}\n\n\`\`\`python\n${py}\n\`\`\`\n`;
{
    // a widget given to two panels is drawn in each, and the two do not move together
    const fr = await run(fence([
        'import ui', 'L = ui.slider("L", 6, 2, 12, unit="m")',
        '@ui.panel(L)', 'def one(L):', '    return "one %g" % L',
        '@ui.panel(L)', 'def two(L):', '    return "two %g" % L'].join('\n')), async (f) => (await f.locator('.cprs-panel .cprs-w').count()) >= 2);
    check(!!fr && await fr.locator('.cprs-panel').count() === 2, 'fact: a widget given to two panels gives each panel its own slider');
    if (fr) {
        await move(fr, 0);
        check(await waitFor(async () => (await shown(fr, 0)).startsWith('one 12')), 'fact: the first panel follows its slider');
        check((await shown(fr, 1)).startsWith('two 6'), 'fact: the second panel does not move with it');
    }
}
for (const [py, re, what] of [
    ['import numpy as np\nX, Y = np.meshgrid([1, 2], [3, 4])', /NotImplementedError/, 'numpy.meshgrid raises NotImplementedError'],
    ['import math\nprint(math.hypot(3, 4))', /AttributeError/, 'math.hypot is missing (AttributeError)'],
    ['z = [' + Array.from({ length: 4000 }, (_, i) => (i % 97) / 10).join(', ') + ']\nprint(len(z))', /pystack exhausted/, 'a 4,000-number list literal fails with pystack exhausted'],
    ['z = []\nfor i in range(4000):\n    z.append((i % 97) / 10)\nprint("built", len(z))', null, 'the same list built in a loop works'],
]) {
    const fr = await run(fence(py), async (f) => /Python error|built 4000/.test(await f.locator('body').innerText()));
    const body = fr ? await fr.locator('body').innerText() : '';
    check(re ? re.test(body) : /built 4000/.test(body) && !/Python error/.test(body), `fact: ${what}${fr ? '' : ' (no result)'}`);
    if (process.argv.includes('--show')) console.log('     ' + body.replace(/\s+/g, ' ').slice(0, 300));
}

check(pageErrors.length === 0, `no page errors${pageErrors.length ? ': ' + pageErrors[0] : ''}`);
await browser.close();
console.log(fails.length ? `\n${fails.length} failed` : '\nall ok');
process.exit(fails.length ? 1 : 0);
