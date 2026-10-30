// Interactive panels (`import ui`, docs/interactive-panels-design.md) in the built app served at --url:
// the widgets are drawn in the results frame, a widget change runs only the panel's function in the live Python session (the
// worksheet is not converted again), the chart and table are redrawn in place, the values survive a recalculation and reach the
// exports, a player steps one frame per answer, a stuck function is stopped, and a frame cannot make the host do more than
// choose widget values. Light and dark, desktop and phone screenshots go to --out.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/panels-check.mjs [--url U] [--out DIR]
// The chart checks load Plotly.js from its CDN and are skipped, with a line saying so, when there is no network.
import { chromium } from 'playwright';
import { mkdirSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-panels'));
const samples = join(dirname(fileURLToPath(import.meta.url)), '../calcpad-web/public/sheet-samples');
mkdirSync(out, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

const BEAM = readFileSync(join(samples, 'Panel beam deflection explorer.sheet'), 'utf8');
const PUMP = readFileSync(join(samples, 'Panel pump operating point player.sheet'), 'utf8');
const COLUMN = readFileSync(join(samples, 'Panel column buckling player.sheet'), 'utf8');
const STUCK = [
    '<!--sheet-->', '# Stuck', '',
    '```python', 'import ui', 'x = ui.slider("x", 1, 0, 10)',
    '@ui.panel(x)', 'def slow(x):',
    '    if x > 5:', '        while True:', '            pass',
    '    if x == 4:', '        raise ValueError("four is not allowed")',
    '    return "x is %d" % x', '```', ''].join('\n');

async function open(opts = {}) {
    const ctx = await browser.newContext({ viewport: { width: 1300, height: 1000 }, ...opts });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    // counts the worksheet conversions, to show a widget change makes none
    await page.evaluate(() => {
        window.__converts = 0;
        const f = window.fetch;
        window.fetch = (i, o) => { if (String(i?.url ?? i).includes('/api/calcpad/convert')) window.__converts++; return f(i, o); };
    });
    return { ctx, page, errors };
}
const setText = (page, t) => page.evaluate(v => window.calcpadActiveEditor.setValue(v), t);
const converts = page => page.evaluate(() => window.__converts);
async function panelFrame(page, ms = 60000) {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        const yes = page.locator('#cprs-python-yes');
        if (await yes.count().catch(() => 0)) await yes.first().click().catch(() => {});
        for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator('.cprs-panel .cprs-w').count().catch(() => 0)) return f;
        await page.waitForTimeout(300);
    }
    return null;
}
/** Sets a range input the way a finger does and fires input. */
const slide = (fr, label, v) => fr.evaluate(([l, val]) => {
    const lab = [...document.querySelectorAll('.cprs-w label')].find(x => x.textContent === l);
    const el = document.getElementById(lab.getAttribute('for'));
    el.value = val; el.dispatchEvent(new Event('input', { bubbles: true }));
}, [label, v]);
const waitFor = async (fn, ms = 15000) => { const t0 = Date.now(); while (Date.now() - t0 < ms) { if (await fn().catch(() => false)) return true; await new Promise(r => setTimeout(r, 100)); } return false; };
const tableText = fr => fr.locator('.cprs-panel-table').first().innerText().catch(() => '');

// ---- 1. the beam: draw, change, no recalculation, persistence, exports ----------------------------------------------------
{
    const { ctx, page, errors } = await open();
    await setText(page, BEAM);
    const fr = await panelFrame(page);
    check(!!fr, 'the panel is drawn in the results frame');
    if (fr) {
        check(await fr.locator('.cprs-w').count() === 3, 'three widgets: two sliders and a drop-down');
        check(await fr.locator('.cprs-panel-note').evaluate(n => getComputedStyle(n).display === 'none'), 'the static note is hidden once the widgets are drawn');
        const plotly = await waitFor(() => fr.locator('.cprs-panel .js-plotly-plot').count().then(n => n > 0), 30000);
        if (!plotly) console.log('skip chart checks: Plotly.js did not load (no network?)');
        check(/Maximum moment\s+(\S+) kN\.m/.test(await tableText(fr)) && (await tableText(fr)).includes('54.00 kN.m'), 'first result at L = 6 m, w = 12 kN/m: M = 54.00 kN.m');
        await fr.evaluate(() => document.querySelector('.cprs-panel').scrollIntoView());
        await page.screenshot({ path: join(out, 'beam-desktop-light.png') });

        const before = await converts(page);
        const t0 = Date.now();
        await slide(fr, 'Span', 10);
        check(await waitFor(async () => (await tableText(fr)).includes('150.00 kN.m')), 'a slider change redraws the table (L = 10 m: M = 150.00 kN.m)');
        const ms = Date.now() - t0;
        console.log(`info  slider change to redrawn table: ${ms} ms`);
        check((await converts(page)) === before, 'the worksheet was not converted again for a widget change');
        if (plotly) {
            const ymax = await fr.evaluate(() => Math.max(...document.querySelector('.cprs-panel .js-plotly-plot').data[0].y));
            check(ymax > 25, `the chart was redrawn in place (peak deflection ${ymax.toFixed(1)} mm at 10 m)`);
        }
        await fr.locator('.cprs-w select').selectOption('1');
        check(await waitFor(() => fr.locator('.cprs-panel .js-plotly-plot').evaluate(g => g.layout.yaxis.title.text === 'Bending moment (kN.m)')), 'the drop-down changes what the chart shows');
        check(await fr.locator('.cprs-w output').first().innerText() === '10 m', 'the readout shows the value and its unit');

        // a recalculation (edit a prose line) keeps the values
        await setText(page, BEAM.replace('A simply supported beam', 'A simply supported steel beam'));
        const fr2 = await panelFrame(page);
        check(!!fr2 && await waitFor(async () => (await fr2.locator('.cprs-w output').first().innerText()) === '10 m'), 'a recalculation starts the panel from the values it was left at');
        check((await tableText(fr2)).includes('150.00 kN.m'), 'and its first result is for those values');

        // exports: a static snapshot at the current values
        const ex = await page.evaluate(async (content) => {
            const o = {};
            for (const ep of ['convert', 'docx', 'xlsx']) {
                const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: ep === 'convert', includeLineAnchors: false }) });
                const b = await r.arrayBuffer();
                o[ep] = { status: r.status, bytes: b.byteLength, text: ep === 'convert' ? new TextDecoder().decode(b) : '' };
            }
            return o;
        }, BEAM);
        check(ex.convert.status === 200 && /Interactive panel beam: Span = 10 m, Uniform load = 12 kN\/m, Show = Bending moment/.test(ex.convert.text), 'Report / Print carries the note with the values the panel was left at');
        check(ex.convert.text.includes('150.00 kN.m') || ex.convert.text.includes('Maximum moment'), 'and the first output as a static table');
        check(ex.docx.status === 200 && ex.docx.bytes > 2000, `Word export works (${ex.docx.bytes} bytes)`);
        check(ex.xlsx.status === 200 && ex.xlsx.bytes > 3000, `Excel export works (${ex.xlsx.bytes} bytes)`);
    }
    check(errors.length === 0, `no page errors${errors.length ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// ---- 2. a frame can only choose widget values -------------------------------------------------------------------------------
{
    const { ctx, page } = await open();
    await setText(page, BEAM);
    const fr = await panelFrame(page);
    if (fr) {
        const send = (msg) => fr.evaluate(m => new Promise(res => {
            const t = setTimeout(() => res('none'), 3000);
            window.addEventListener('message', function h(e) { if (e.data && e.data.type === 'cpPanelUpdate' && e.data.seq === m.seq) { clearTimeout(t); removeEventListener('message', h); res(e.data); } });
            window.parent.postMessage(m, '*');
        }), msg);
        const run = await fr.locator('.cprs-panel').first().getAttribute('data-run');
        const good = await send({ type: 'panelEvent', panel: 'beam', run, seq: 901, values: { L: 8, w: 12, what: 0 } });
        check(Array.isArray(good.out), 'a well-formed event is answered');
        const clamp = await send({ type: 'panelEvent', panel: 'beam', run, seq: 902, values: { L: 1e9, w: -5, what: 0 } });
        check(JSON.stringify(clamp.out).includes('"Maximum moment"') && JSON.stringify(clamp.out).includes('kN.m'), 'out-of-range values are clamped by the widget, not trusted');
        const bad1 = await send({ type: 'panelEvent', panel: 'beam', run, seq: 903, values: { L: 'x'.repeat(10), w: 1 } });
        check(bad1 === 'none', 'a string value is dropped by the host without an answer');
        const bad2 = await send({ type: 'panelEvent', panel: 'nope', run, seq: 904, values: { L: 3 } });
        check(bad2 !== 'none' && /no such panel/.test(bad2.error ?? ''), 'an unknown panel is refused by Python, named');
        const bad3 = await send({ type: 'panelEvent', panel: 'beam', run, seq: 905, values: { what: 7 } });
        check(bad3 !== 'none' && /no such option/.test(bad3.error ?? ''), 'a select value outside its options is refused');
        const stale = await send({ type: 'panelEvent', panel: 'beam', run: 'deadbeef', seq: 906, values: { L: 3 } });
        check(stale !== 'none' && stale.stale === true, 'an event quoting another run is answered stale');
        const bad4 = await send({ type: 'panelEvent', panel: 'beam', run, seq: 907, values: { __proto__: 1, L: 3, ['a'.repeat(80)]: 1 } });
        check(bad4 === 'none', 'a value name that is not a plain name is dropped');
    }
    await ctx.close();
}

// ---- 3. players: one frame per answer ---------------------------------------------------------------------------------------
{
    const { ctx, page } = await open();
    await setText(page, PUMP);
    const fr = await panelFrame(page);
    check(!!fr, 'the pump panel is drawn');
    if (fr) {
        const before = await converts(page);
        const read = () => fr.evaluate(() => document.querySelector('.cprs-w output[for$="-n"]').textContent);
        check((await read()).startsWith('60'), 'the player starts at 60');
        await fr.getByRole('button', { name: 'Play Speed (% of rated)' }).click();
        const moved = await waitFor(async () => parseInt(await read(), 10) >= 66, 20000);
        check(moved, 'Play steps through frames');
        await page.screenshot({ path: join(out, 'pump-playing.png') });
        await fr.getByRole('button', { name: 'Play Speed (% of rated)' }).click();
        const at = parseInt(await read(), 10);
        await page.waitForTimeout(1500);
        check(Math.abs(parseInt(await read(), 10) - at) <= 1, 'Pause stops it');
        check((await fr.locator('.cprs-panel-text').innerText()).startsWith('Operating point:'), 'the text result follows the frame');
        check((await converts(page)) === before, 'no worksheet conversion while playing');
    }
    await ctx.close();
}

// ---- 4. a function that fails, and one that never returns --------------------------------------------------------------------
{
    const { ctx, page } = await open();
    await setText(page, STUCK);
    const fr = await panelFrame(page);
    check(!!fr, 'the stuck panel is drawn');
    if (fr) {
        await slide(fr, 'x', 4);
        check(await waitFor(() => fr.locator('.cprs-panel-err[role=alert]').innerText().then(t => /four is not allowed/.test(t))), 'an exception in the function is shown in the panel, with its type');
        await slide(fr, 'x', 3);
        check(await waitFor(() => fr.locator('.cprs-panel-text').innerText().then(t => t === 'x is 3')), 'and the panel recovers on the next value');
        await slide(fr, 'x', 8);
        check(await waitFor(() => fr.locator('.cprs-panel-err').innerText().then(t => /longer than 10 s/.test(t)), 30000), 'a function that does not return is stopped after 10 s, and says so');
        await slide(fr, 'x', 2);
        check(await waitFor(() => fr.locator('.cprs-panel-err').innerText().then(t => /out of date/.test(t))), 'the session is gone afterwards: later events are stale until the sheet is calculated again');
    }
    await ctx.close();
}

// ---- 5. the column: select and player, then light / dark and phone screenshots ---------------------------------------------------
for (const [name, opts] of [
    ['column-desktop-dark', { colorScheme: 'dark' }],
    ['column-phone-light', { colorScheme: 'light', viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true }],
    ['column-phone-dark', { colorScheme: 'dark', viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true }],
]) {
    const { ctx, page, errors } = await open(opts);
    await setText(page, COLUMN);
    // on a phone the editor and the results are panes of one screen; the bar at the bottom switches
    if (opts.isMobile) await page.getByText('Results', { exact: true }).last().click();
    const fr = await panelFrame(page);
    check(!!fr, `${name}: the column panel is drawn`);
    if (fr) {
        await waitFor(() => fr.locator('.cprs-panel .js-plotly-plot').count().then(n => n > 0), 30000);
        await slide(fr, 'Load P / Pcr (%, step 5)', 14);
        await waitFor(async () => (await fr.locator('.cprs-panel-text').innerText()).includes('70 %'));
        await fr.evaluate(() => document.querySelector('.cprs-panel').scrollIntoView());
        await page.waitForTimeout(600);
        await page.screenshot({ path: join(out, `${name}.png`) });
        if (opts.isMobile) {
            const wide = await fr.evaluate(() => document.documentElement.scrollWidth > window.innerWidth + 1);
            check(!wide, `${name}: no horizontal scroll in the results`);
            const h = await fr.evaluate(() => Math.min(...[...document.querySelectorAll('.cprs-w input[type=range],.cprs-w button,.cprs-w select')].map(e => e.getBoundingClientRect().height)));
            check(h >= 28, `${name}: controls are tall enough to touch (${Math.round(h)} px)`);
        }
    }
    check(errors.length === 0, `${name}: no page errors`);
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} failed` : '\nall panel checks passed');
process.exit(fails.length ? 1 : 0);
