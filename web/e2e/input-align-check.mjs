// Input rows line up (docs/sheet-syntax.md, "Inputs and templates"): a run of consecutive input rows in the Input
// mode shares one column for the label, the number box (or drop-down), the unit and the note. Headless Chromium.
//   node e2e/input-align-check.mjs [--dist DIR] [--port 4184] [--shots DIR]
//
//   1. a run of four slider rows with labels of different widths (a plain name, a long one, a subscript, a Greek letter):
//      the number boxes, the sliders, the units and the notes start at the same x (within 1 px), the "=" signs line up;
//   2. a run of a drop-down, a checkbox and a number box shares the label column, the drop-down and the box start together;
//   3. a text line ends a run: the rows after it align among themselves, in a column of their own;
//   4. a single row is left as it is (nothing wrapped, no widths set);
//   5. a very long label is capped (at most 40% of the row), wraps, and does not push the others or overflow the page;
//   6. a phone-wide pane: no horizontal overflow, and the rows of a run still share one x for their boxes;
//   7. print keeps the plain flow (the widths are screen rules), values and tab order are unchanged, and a box edited
//      in the form still re-runs the sheet;
//   8. a label edited in the sheet text makes the next render line up on the new widest label.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const port = Number(arg('--port', '4184'));
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };

const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.sheet': 'text/plain' };
const server = createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (path.endsWith('/')) path += 'index.html';
    const file = normalize(join(dist, path));
    if (!file.startsWith(dist) || !existsSync(file) || !statSync(file).isFile()) { res.writeHead(404); res.end('not found'); return; }
    res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));

const LONG = 'ridge_height_above_the_eaves_level_of_the_building';
const SHEET = [
    '<!--sheet-->',
    '# Align check',
    '',
    'len: ? 20 m [5 .. 50 step 1] | length, along the ridge',
    'span: ? 15 m [5 .. 40] | span',
    'h_e: ? 5 m [1 .. 10] | eaves height',
    'alpha: ? 25 ° [0 .. 45] | pitch',
    '',
    'Material and options follow.',
    '',
    'mat: ? [Steel = 1, Timber = 3] | material',
    'ribs: ? on | stiffened',
    'b: ? 300 mm | width',
    '',
    'A text line ends the run above.',
    'q: ? 2 kN/m [1 .. 9] | load',
    'Qk: ? 4 kN [1 .. 9] | point load',
    '',
    'A line of text.',
    'x: ? 2 m [0 .. 4] | alone',
    'A line of text again.',
    `${LONG}: ? 3 m [1 .. 10] | very long name`,
    'w: ? 6 m [1 .. 10] | short name',
    '',
    'V: len * span -> m^2 | area',
    '',
].join('\n');

const browser = await chromium.launch();
const pageErrors = [];

/** A page with the sheet open in the Input mode, and the frame that holds the form. */
async function open(opts = {}, text = SHEET) {
    const context = await browser.newContext({ viewport: { width: 1300, height: 900 }, ...opts });
    await context.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
    const page = await context.newPage();
    page.on('pageerror', (e) => { if (e.message !== 'Canceled' && e.message !== 'Event') pageErrors.push(e.message); });
    await page.goto(`http://127.0.0.1:${port}/`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 120000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
    if (opts.isMobile) await page.getByText('Results', { exact: true }).last().click().catch(() => {});
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click({ timeout: 10000 }).catch(() => {});
    return { context, page };
}

/** The frame that shows the form, once the pass has run (or has had its time to). */
async function formFrame(page, marker = 'Align check', tries = 120) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const ok = await f.evaluate((m) => !!document.querySelector('input.calcpad-ui-slider') && (document.body.innerText || '').includes(m), marker).catch(() => false);
            if (ok) { await page.waitForTimeout(700); return f; }
        }
        await page.waitForTimeout(300);
    }
    return null;
}

/** Left x of the parts of every input row, by variable name. */
const geometry = (f) => f.evaluate(() => {
    const out = {};
    document.querySelectorAll('p[data-ui-type]').forEach((p) => {
        const name = (p.getAttribute('data-ui-var') || '').split(':')[0];
        const box = p.querySelector('input.calcpad-ui-input, select, input[type=checkbox]');
        const lbl = p.querySelector('.cprs-lbl');
        const left = (el) => (el ? Math.round(el.getBoundingClientRect().left * 10) / 10 : null);
        const right = (el) => (el ? Math.round(el.getBoundingClientRect().right * 10) / 10 : null);
        out[name] = {
            box: left(box), slider: left(p.querySelector('.calcpad-ui-slider')), unit: left(p.querySelector('.cprs-unit')),
            note: left(p.querySelector('.sheet-note')), lblRight: right(lbl), lblW: lbl ? lbl.getBoundingClientRect().width : null,
            pw: p.clientWidth, wrapped: !!lbl, run: p.hasAttribute('data-cprs-run'), cssVars: p.getAttribute('style') || '',
            lblNowrapHeight: lbl ? lbl.getBoundingClientRect().height : null,
        };
    });
    return out;
});
const same = (vals) => vals.length > 0 && vals.every((v) => v !== null && Math.abs(v - vals[0]) <= 1);
const col = (g, names, key) => names.map((n) => g[n]?.[key] ?? null);
const fmt = (a) => JSON.stringify(a);

const A = ['len', 'span', 'h_e', 'α'];
const B = ['mat', 'ribs', 'b'];
const C = ['q', 'Qk'];
const D = [LONG, 'w'];

// ---- desktop, light -------------------------------------------------------------------------------------------------------
{
    const { context, page } = await open();
    const f = await formFrame(page);
    check(f != null, 'Input mode shows the form');
    const g = await geometry(f);
    check(Object.keys(g).length === 12, `twelve input rows (${Object.keys(g).join(' ')})`);
    // 1
    check(same(col(g, A, 'box')), `run A: the boxes start at one x ${fmt(col(g, A, 'box'))}`);
    check(same(col(g, A, 'slider')), `run A: the sliders start at one x ${fmt(col(g, A, 'slider'))}`);
    check(same(col(g, A, 'unit')), `run A: the units start at one x ${fmt(col(g, A, 'unit'))}`);
    check(same(col(g, A, 'note')), `run A: the notes start at one x ${fmt(col(g, A, 'note'))}`);
    check(same(col(g, A, 'lblRight')), `run A: the "=" signs line up (label right edges) ${fmt(col(g, A, 'lblRight'))}`);
    // 2
    check(same(col(g, B, 'box')), `run B (drop-down, checkbox, number box): the controls start at one x ${fmt(col(g, B, 'box'))}`);
    check(same(col(g, B, 'lblRight')), `run B: the labels end at one x ${fmt(col(g, B, 'lblRight'))}`);
    // 3
    check(same(col(g, C, 'box')) && same(col(g, C, 'slider')) && same(col(g, C, 'unit')) && same(col(g, C, 'note')),
        `run C after a text line aligns among itself ${fmt(col(g, C, 'box'))} ${fmt(col(g, C, 'unit'))} ${fmt(col(g, C, 'note'))}`);
    check(Math.abs(g.q.box - g.len.box) > 5 && Math.abs(g.q.box - g.mat.box) > 5,
        `run C is its own column, not run A's or B's (${g.q.box} against ${g.len.box} and ${g.mat.box})`);
    check(g.len.cssVars.includes('--cprs-lw') && g.len.run && g.q.run && g.mat.run, 'rows of a run carry the widths');
    // 4
    check(!g.x.wrapped && !g.x.run && !g.x.cssVars.includes('--cprs'), `the single row is untouched (${fmt(g.x)})`);
    // 5
    const L = g[LONG];
    check(L.lblW != null && L.lblW <= 0.4 * L.pw + 1, `the long label is capped at 40% of the row (${Math.round(L.lblW)} of ${L.pw})`);
    check(same(col(g, D, 'box')) && same(col(g, D, 'slider')), `run D: the long and the short row share the box and slider columns ${fmt(col(g, D, 'box'))} ${fmt(col(g, D, 'slider'))}`);
    check(L.lblNowrapHeight != null && L.lblNowrapHeight > g.w.lblNowrapHeight * 1.5, `the long label wraps inside its cell (${Math.round(L.lblNowrapHeight)} px against ${Math.round(g.w.lblNowrapHeight)})`);
    const over = await f.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    check(over <= 1, `no horizontal overflow on a desktop pane (${over})`);
    if (shots) await page.screenshot({ path: join(shots, 'align-desktop.png') });

    // 7. values, tab order, print, and the form still works
    const vals = await f.evaluate(() => [...document.querySelectorAll('input.calcpad-ui-input')].map((i) => i.value).join(','));
    check(vals === '20,15,5,25,300,2,4,2,3,6', `the boxes still hold their values (${vals})`);
    const order = await f.evaluate(() => {
        const lines = [...document.querySelectorAll('input,select')].filter((e) => e.closest('p[data-ui-type]')).map((e) => Number(e.closest('p').getAttribute('data-ui-line')));
        return lines.every((x, i) => i === 0 || x >= lines[i - 1]);
    });
    check(order, 'tab order follows the lines of the sheet');
    await f.evaluate(() => { document.querySelector('input.calcpad-ui-input[data-ui-var^="span"]').focus(); });
    await page.keyboard.press('Tab');
    const next = await f.evaluate(() => document.activeElement.className);
    check(next.includes('calcpad-ui-slider'), `Tab from a box goes to its slider (${next})`);
    await page.emulateMedia({ media: 'print' });
    const pr = await f.evaluate(() => { const l = document.querySelector('.cprs-lbl'); if (!l) return 'no label cell'; return getComputedStyle(l).minWidth + ' ' + getComputedStyle(l).display; });
    check(pr === '0px inline', `print keeps the plain flow (${pr})`);
    await page.emulateMedia({ media: 'screen' });
    let area = '';
    for (let attempt = 0; attempt < 4 && !area; attempt++) {
        const ff = await formFrame(page, 'Align check', 20);
        if (!ff) continue;
        const sel = 'input.calcpad-ui-input[data-ui-var^="span"]';
        await ff.fill(sel, '30').catch(() => {});
        await ff.press(sel, 'Enter').catch(() => {});
        for (let i = 0; i < 24 && !area; i++) {
            await page.waitForTimeout(300);
            for (const fr of page.frames()) {
                if (fr === page.mainFrame()) continue;
                const txt = await fr.evaluate(() => document.body?.innerText ?? '').catch(() => '');
                if (/= 600\s*m/.test(txt)) area = txt;
            }
        }
    }
    check(/600/.test(area), `editing a box still re-runs the sheet (the area follows: 600 m2) ${JSON.stringify(area.slice(-160))}`);
    const g2 = await geometry((await formFrame(page)) ?? f);
    check(g2.span && same(col(g2, A, 'box')) && same(col(g2, A, 'note')), `after the re-run the run is still aligned ${fmt(col(g2, A, 'box'))} ${fmt(col(g2, A, 'note'))}`);
    await context.close();
}

// ---- 8. a label edited in the text ----------------------------------------------------------------------------------------
{
    const { context, page } = await open();
    const f = await formFrame(page);
    const before = f ? (await geometry(f)).len.box : null;
    await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue(e.getValue().replace('span: ?', 'span_of_the_building: ?')); });
    await page.waitForTimeout(2500);
    let g = null;
    for (let i = 0; i < 40; i++) {
        const ff = await formFrame(page, 'Align check', 4);
        if (ff) { g = await geometry(ff); if (g.len && g.len.box !== before) break; }
        await page.waitForTimeout(300);
    }
    check(g != null && g.len.box > before + 5 && same(col(g, A.map((n) => (n === 'span' ? 'span_of_the_building' : n)), 'box')),
        `a longer name moves the column and the run follows (${before} to ${g && g.len.box})`);
    await context.close();
}

// ---- 6. phone width, light and dark ---------------------------------------------------------------------------------------
for (const scheme of ['light', 'dark']) {
    const { context, page } = await open({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, colorScheme: scheme });
    const f = await formFrame(page);
    check(f != null, `phone ${scheme}: the form is shown`);
    if (f) {
        const g = await geometry(f);
        const wide = await f.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
        check(wide <= 1, `phone ${scheme}: no horizontal overflow in the results (${wide})`);
        check(same(col(g, A, 'box')) && same(col(g, A, 'slider')), `phone ${scheme}: the boxes and sliders of a run share an x ${fmt(col(g, A, 'box'))} ${fmt(col(g, A, 'slider'))}`);
        check(same(col(g, B, 'box')), `phone ${scheme}: the controls of run B share an x ${fmt(col(g, B, 'box'))}`);
        const top = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
        check(top <= 1, `phone ${scheme}: the app page itself does not scroll sideways (${top})`);
        await f.evaluate(() => window.scrollTo(0, 0));
        if (shots) await page.screenshot({ path: join(shots, `align-phone-${scheme}.png`) });
    }
    await context.close();
}

check(pageErrors.length === 0, `no page errors (${pageErrors.slice(0, 2).join(' | ')})`);
await browser.close();
server.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
