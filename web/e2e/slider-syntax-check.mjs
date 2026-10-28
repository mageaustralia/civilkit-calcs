// The one slider syntax, end to end in the app (docs/sheet-syntax.md, "Inputs and templates"), headless Chromium.
//   node e2e/slider-syntax-check.mjs [--dist DIR] [--port 4183] [--url http://127.0.0.1:4801/] [--shots DIR]
//
//   1. the line `L: ? 6 m [2 .. 12 step 0.5] | span` is typed into a sheet; Input mode shows a slider with those bounds;
//   2. moving the slider by the keyboard recalculates the result;
//   3. a typed value outside the bounds shows `must be from 2 to 12` beside the box and is not used;
//   4. Format Document writes a loosely spaced slider line in one form and a second format changes nothing;
//   5. the old word form (`L: ? 6 m slider 2..12 step 0.5`) is not syntax: an error on its line, no slider, not rewritten.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const port = Number(arg('--port', '4183'));
const external = arg('--url', '');
const origin = external ? external.replace(/\/$/, '') : `http://127.0.0.1:${port}`;
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };

const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.sheet': 'text/plain' };
let server = null;
if (!external) {
    server = createServer((req, res) => {
        let path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
        if (path.endsWith('/')) path += 'index.html';
        const file = normalize(join(dist, path));
        if (!file.startsWith(dist) || !existsSync(file) || !statSync(file).isFile()) { res.writeHead(404); res.end('not found'); return; }
        res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
        res.end(readFileSync(file));
    });
    await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
}


const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 900 }, acceptDownloads: true });
await context.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
const page = await context.newPage();
const pageErrors = [];
page.on('pageerror', (e) => { if (e.message !== 'Canceled' && e.message !== 'Event') pageErrors.push(e.message); });
await page.goto(`${origin}/`);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });

const newTab = (text) => page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
const editorText = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
const markers = () => page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).map((m) => [m.startLineNumber, m.severity, m.message]));
const spaced = (s) => s.replace(/\s+/g, ' ');
const format = async () => { await page.evaluate(() => window.calcpadActiveEditor.focus()); await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run()); await page.waitForTimeout(500); };
async function frameTexts() {
    const out = [];
    for (const f of page.frames()) { if (f !== page.mainFrame()) out.push(await f.evaluate(() => document.body?.innerText ?? '').catch(() => '')); }
    return out;
}
async function until(marker, pattern, tries = 100) {
    for (let i = 0; i < tries; i++) {
        for (const t of await frameTexts()) if (t.includes(marker) && pattern.test(spaced(t))) return spaced(t);
        await page.waitForTimeout(250);
    }
    return '';
}
async function form(tries = 60) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) { if (f !== page.mainFrame() && await f.$('input.calcpad-ui-input').catch(() => null)) return f; }
        await page.waitForTimeout(250);
    }
    return null;
}
async function change(marker, pattern, act) {
    let t = '';
    for (let attempt = 0; attempt < 4 && !t; attempt++) {
        const f = await form();
        if (!f) continue;
        await act(f).catch(() => {});
        t = await until(marker, pattern, 24);
    }
    return t;
}
const mode = (name) => page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 10000 });

// ---- 1. type the line, Input mode shows the slider ------------------------------------------------------------
await newTab('<!--sheet-->\n# Slider syntax\nw: 10 kN/m\n\nV: w * L / 2 -> kN | shear\n');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 4, column: 1 }); });
await page.keyboard.type('L: ? 6 m [2 .. 12 step 0.5] | span');
await page.keyboard.press('Escape');
const typed = (await editorText()).split('\n')[3];
check(typed === 'L: ? 6 m [2 .. 12 step 0.5] | span', `the slider line is typed as written (${JSON.stringify(typed)})`);
let t = await until('Slider syntax', /V = .*= 30 kN/);
check(/= 30 kN/.test(t), `Preview shows the default: V = 30 kN (${t.slice(0, 60)})`);
await mode('Input');
const ff = await form();
check(ff != null, 'Input mode shows the entry');
if (ff) {
    const info = await ff.evaluate(() => [...document.querySelectorAll('input.calcpad-ui-slider')].map((x) => [x.min, x.max, x.step, x.value]));
    check(info.length === 1 && JSON.stringify(info[0]) === '["2","12","0.5","6"]', `one slider with the bounds 2 to 12, step 0.5, at 6 (${JSON.stringify(info)})`);
}
if (shots) await page.screenshot({ path: join(shots, 'slider-syntax-input.png') });

// ---- 2. moving the slider recalculates --------------------------------------------------------------------------
t = await change('Slider syntax', /V = .*= 35 kN/, async (f) => {
    await f.locator('input.calcpad-ui-slider').first().focus();
    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('ArrowRight');
});
check(/= 35 kN/.test(t), `two key presses move L from 6 m to 7 m and V follows to 35 kN (${t.slice(t.indexOf('V ='), t.indexOf('V =') + 50)})`);

// ---- 3. a typed value outside the bounds ------------------------------------------------------------------------
{
    const f = await form();
    await f.fill('input.calcpad-ui-input >> nth=0', '20');
    await f.press('input.calcpad-ui-input >> nth=0', 'Enter');
    await page.waitForTimeout(1200);
    const e = await (await form()).evaluate(() => { const i = document.querySelector('input.calcpad-ui-input'); const m = (i.closest('p') || i.parentElement).querySelector('.calcpad-ui-error'); return { cls: i.className, msg: m && !m.hidden ? m.textContent : '', value: i.value }; });
    check(/calcpad-ui-invalid/.test(e.cls) && e.msg === 'must be from 2 to 12' && e.value === '20', `a typed 20 shows "must be from 2 to 12" (${JSON.stringify(e)})`);
    if (shots) await page.screenshot({ path: join(shots, 'slider-syntax-error.png') });
    t = await until('Slider syntax', /V = /);
    check(/= 35 kN/.test(t) && !/= 100 kN/.test(t), 'the out-of-range value is not used: V stays at 35 kN');
}

// ---- 4. the formatter round trip --------------------------------------------------------------------------------
await newTab('<!--sheet-->\nL:?6 m[2..12 step .5]|span\nw :  ?  12 kN/m  [ 2 ..  40 ]\n');
await format();
const tidy = await editorText();
check(tidy === '<!--sheet-->\nL: ? 6 m [2 .. 12 step 0.5] | span\nw: ? 12 kN/m [2 .. 40]\n', `Format Document writes the bracket form in one shape (${JSON.stringify(tidy)})`);
await format();
check((await editorText()) === tidy, 'a second format changes nothing');

// ---- 5. the old word form is not syntax -------------------------------------------------------------------------
await newTab('<!--sheet-->\nL: ? 6 m slider 2..12 step 0.5 | span\nV: L * 2\n');
let mk = [];
for (let i = 0; i < 30 && !mk.some((x) => x[1] === 8); i++) { await page.waitForTimeout(300); mk = await markers(); }
check(mk.some((x) => x[0] === 2 && x[1] === 8), `\`slider 2..12\` is an error on its line (${JSON.stringify(mk)})`);
await mode('Input');
await page.waitForTimeout(1500);
let sliders = 0;
for (const f of page.frames()) if (f !== page.mainFrame()) sliders += await f.evaluate(() => document.querySelectorAll('input.calcpad-ui-slider').length).catch(() => 0);
check(sliders === 0, `the word form draws no slider (${sliders})`);
const before = await editorText();
await format();
check((await editorText()) === before, 'the formatter leaves the word form as written');

check(pageErrors.length === 0, `no page errors (${pageErrors.slice(0, 2).join(' | ')})`);
await browser.close();
server?.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
