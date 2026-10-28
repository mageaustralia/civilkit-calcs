// End-to-end check of folding sections and key results (docs/sheet-syntax.md, "Folding sections and key results"),
// headless Chromium on the built app.
//   node e2e/fold-key-check.mjs [--dist DIR] [--port 4183] [--url http://127.0.0.1:4801/] [--shots DIR]
//   (after tools/build_wasm.sh and npm run build; with --url it uses a server that is already running)
//
//   1. a `## Workings {collapse}` heading folds its section in the preview: the section is hidden, its calculation is kept,
//      a click on the heading (and Enter on it) opens and closes it, `{collapse open}` starts open;
//   2. `{key}` lines carry a star and a highlight, and `{key results}` lists them with their values;
//   3. the report folds the same way, Print PDF opens every folded section, the Word and Excel exports carry the whole section;
//   4. the editor: Format Document writes the markers in one form, the ribbon inserts the folding heading and the key results
//      list in a sheet, a `{collapse}` heading inside a block is a warning;
//   5. screenshots (light and dark, desktop and phone width) with --shots.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
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

const SHEET = [
    '<!--sheet-->', '# Beam moment', '',
    'L: ? 6 m [2 .. 12 step 0.5] | span {key}', 'w: 12 kN/m | design load', '',
    '## Workings {collapse}', 'The moment at mid-span is $w L^2 / 8$.', 'M: w * L^2 / 8 -> kN*m | maximum bending moment {key}',
    'Ms: 180 kN*m | moment capacity', '', '### Check', 'x: 1 | a deeper heading', '',
    '## Result', 'utilisation: M / Ms | bending utilisation {key}', '',
    '## Key results', '{key results}', '',
].join('\n');

const browser = await chromium.launch();
async function open(width, height, scheme, query = '') {
    const phone = width < 600;
    const context = await browser.newContext({ viewport: { width, height }, colorScheme: scheme, acceptDownloads: true, ...(phone ? { hasTouch: true, isMobile: true, deviceScaleFactor: 2 } : {}) });
    await context.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
    // Printing: record the page the print frame holds instead of opening a dialog.
    await context.addInitScript(() => {
        if (window !== window.top) return;
        new MutationObserver((records) => {
            for (const r of records) for (const n of r.addedNodes) {
                if (!(n instanceof HTMLIFrameElement) || !(n.getAttribute('sandbox') ?? '').includes('allow-modals')) continue;
                n.addEventListener('load', () => {
                    const w = n.contentWindow;
                    w.print = () => { window.__printed = (window.__printed ?? 0) + 1; window.__printedHtml = w.document.documentElement.outerHTML; };
                });
            }
        }).observe(document, { childList: true, subtree: true });
    });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (e.message !== 'Canceled' && e.message !== 'Event') errors.push(e.message); });
    await page.goto(`${origin}/${query}`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    return { page, errors, context };
}

const { page, errors } = await open(1500, 900, 'light');
const newTab = (text) => page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
const editorText = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
const markers = () => page.evaluate(() => window.monaco.editor.getModelMarkers({}).map((m) => [m.startLineNumber, m.severity, m.message]));
const mode = async (name) => {
    const dont = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
    if (await dont.isVisible().catch(() => false)) await dont.click();
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 30000 });
    if (await dont.waitFor({ timeout: 1000 }).then(() => true, () => false)) await dont.click();
};
/** The preview frame that is shown (the other buffer can hold an older page), as a locator. */
const front = () => page.frameLocator('iframe.preview-frame:not(.preview-frame-back)');
/** Runs `fn` in the shown frame's document. */
const inFront = (fn, arg) => front().locator('html').evaluate(fn, arg);
/** Waits until the shown frame's text (textContent, so a folded section counts) holds `marker`. */
async function shown(marker, tries = 100) {
    for (let i = 0; i < tries; i++) {
        const ok = await inFront((el, m) => (el.textContent ?? '').includes(m), marker).catch(() => false);
        if (ok) return true;
        await page.waitForTimeout(250);
    }
    return false;
}
const spaced = (s) => s.replace(/\s+/g, ' ');

// ---- 1. the folding section -----------------------------------------------------------------------------
await newTab(SHEET);
await mode('Preview');
check(await shown('utilisation'), 'the sheet renders');
{
    const s = await inFront(() => {
        const d = document.querySelector('details.sheet-collapse');
        const m = [...document.querySelectorAll('p')].find((p) => /maximum bending moment/.test(p.textContent));
        return {
            n: document.querySelectorAll('details.sheet-collapse').length, open: d?.open, summary: d?.querySelector('summary h2')?.textContent.replace(/[^A-Za-z]/g, ''),
            mText: m?.textContent.replace(/\s+/g, ' ').trim().slice(0, 90),
            result: [...document.querySelectorAll('p')].find((p) => /bending utilisation/.test(p.textContent))?.textContent.replace(/\s+/g, ' '),
        };
    });
    check(s.n === 1 && s.open === false && s.summary === 'Workings', `one folding section, closed, with the heading as its summary (${JSON.stringify([s.n, s.open, s.summary])})`);
    check(!(await front().locator('p', { hasText: 'maximum bending moment' }).first().isVisible()) && !(await front().locator('h3', { hasText: 'Check' }).isVisible()) && await front().locator('details.sheet-collapse > summary').isVisible(),
        'the folded section is hidden (its lines and its deeper heading) and its heading shows');
    check(/= 0\.3/.test(s.result ?? ''), `its calculation is kept: the result below uses M (${s.result})`);
    check(/= 54 kN/.test(s.mText ?? ''), `the hidden line holds its value (${s.mText})`);
    if (shots) await page.screenshot({ path: join(shots, 'fold-closed-light.png') });
    const summary = () => front().locator('details.sheet-collapse > summary');
    const line = () => front().locator('p', { hasText: 'maximum bending moment' }).first();
    // A click on the heading opens it, a second click closes it.
    await summary().click();
    await page.waitForTimeout(400);
    check(await line().isVisible() && await front().locator('details.sheet-collapse').evaluate((d) => d.open), 'a click on the heading opens the section and shows its lines');
    if (shots) await page.screenshot({ path: join(shots, 'fold-open-light.png') });
    await summary().click();
    await page.waitForTimeout(400);
    check(!(await line().isVisible()), 'a second click folds it again');
    // From the keyboard.
    await summary().focus();
    await page.keyboard.press('Enter');
    await page.waitForTimeout(400);
    check(await line().isVisible(), 'Enter on the focused heading opens it');
    await page.keyboard.press('Enter');
}

// ---- 2. key lines and the list -----------------------------------------------------------------------------
{
    const k = await inFront(() => {
        const stars = [...document.querySelectorAll('.sheet-key')];
        const line = stars[0]?.closest('p');
        const cs = line ? getComputedStyle(line) : null;
        const list = [...document.querySelectorAll('.sheet-key-results p')].map((p) => p.textContent.replace(/\s+/g, ' ').replace('←', '').trim());
        return { stars: stars.length, text: stars.map((s) => s.textContent), bg: cs?.backgroundColor, border: cs?.borderLeftWidth, list };
    });
    check(k.stars === 3, `the three key lines carry a star (${k.stars})`);
    check(k.text.every((t) => t === '\u2605'), 'each star is a star');
    check(k.bg && k.bg !== 'rgba(0, 0, 0, 0)' && k.border === '3px', `a key line is highlighted (${k.bg}, ${k.border})`);
    check(k.list.length === 3 && /^L = 6 m span$/.test(k.list[0]) && /^M = 54 kN.*moment/.test(k.list[1]) && /^utilisation = 0\.3/.test(k.list[2]), `the key results list has the three key lines with their values (${JSON.stringify(k.list)})`);
}

// {collapse open} starts open and shows its lines.
await newTab('<!--sheet-->\n# T\n\n## Shown {collapse open}\nMo: 2 kN*m | open section\n\n## Hidden {collapse}\nMh: 3 kN*m | closed section\nutilisation: Mo / Mh\n');
{
    check(await shown('closed section'), 'the second sheet renders');
    const s = await inFront(() => [...document.querySelectorAll('details.sheet-collapse')].map((d) => [d.open, d.querySelector('summary').textContent.replace(/[^A-Za-z]/g, '')]));
    check(JSON.stringify(s) === '[[true,"Shown"],[false,"Hidden"]]', `{collapse open} starts open, {collapse} closed (${JSON.stringify(s)})`);
    check(await front().locator('p', { hasText: 'open section' }).first().isVisible() && !(await front().locator('p', { hasText: 'closed section' }).first().isVisible()), 'the open section shows its line and the closed one does not');
}

// ---- 3. report, print, exports -------------------------------------------------------------------------------
await newTab(SHEET);
await mode('Report');
check(await shown('utilisation'), 'the report renders');
{
    const r = await inFront(() => [document.querySelectorAll('details.sheet-collapse').length, document.querySelector('details.sheet-collapse').open, document.querySelectorAll('.sheet-key').length]);
    check(r[0] === 1 && r[1] === false && r[2] >= 3, `the report folds the section and marks the key lines (${JSON.stringify(r)})`);
}
await page.evaluate(() => { window.__printed = 0; });
await page.locator('.preview-toolbar .toolbar-btn', { hasText: /^Print PDF$/ }).click();
await page.waitForFunction(() => window.__printed > 0, null, { timeout: 15000 }).catch(() => {});
const printed = await page.evaluate(() => ({ n: window.__printed, html: window.__printedHtml ?? '' }));
check(printed.n === 1 && /<details[^>]*\sopen/.test(printed.html) && printed.html.includes('sheet-key'),
    `Print PDF opens the folded section and keeps the key highlight (${printed.n} print call)`);
await mode('Preview');

const out = join(tmpdir(), `fold-key-${process.pid}`);
mkdirSync(out, { recursive: true });
const bytes = await page.evaluate(async () => {
    const content = window.calcpadActiveEditor.getValue();
    const get = async (ep) => { const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true }) }); return [r.status, [...new Uint8Array(await r.arrayBuffer())]]; };
    return { docx: await get('docx'), xlsx: await get('xlsx') };
});
writeFileSync(join(out, 'a.docx'), Buffer.from(bytes.docx[1]));
writeFileSync(join(out, 'a.xlsx'), Buffer.from(bytes.xlsx[1]));
const py = (code, file) => execFileSync('python3', ['-I', '-c', code, file], { encoding: 'utf8' });
const docxText = py('import sys,zipfile,re\nx=zipfile.ZipFile(sys.argv[1]).read("word/document.xml").decode()\nprint(re.sub(r"<[^>]+>"," ",x))', join(out, 'a.docx')).replace(/\s+/g, ' ');
check(bytes.docx[0] === 200 && /Workings/.test(docxText) && /moment at mid-span/.test(docxText) && /maximum bending moment/.test(docxText) && /Check/.test(docxText) && /★/.test(docxText), `the Word export has the whole folded section and the star (${docxText.length} chars)`);
const xlsxText = py('import sys,openpyxl\nwb=openpyxl.load_workbook(sys.argv[1])\nprint(" ".join(str(c.value) for ws in wb for row in ws.iter_rows() for c in row if c.value is not None))', join(out, 'a.xlsx')).replace(/\s+/g, ' ');
check(bytes.xlsx[0] === 200 && /Workings/.test(xlsxText) && /maximum bending moment/.test(xlsxText), `the Excel export has the folded section's lines (${xlsxText.length} chars)`);

// ---- 4. the editor -------------------------------------------------------------------------------------------------
await newTab('<!--sheet-->\n##   Workings   {collapse}\nM:2 kN*m|moment    {key}\nx=3   {key}\n{key results}\n');
await page.evaluate(() => window.calcpadActiveEditor.focus());
await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run());
await page.waitForFunction(() => window.calcpadActiveEditor.getValue().includes('M: 2 kN*m | moment {key}'), null, { timeout: 8000 }).catch(() => {});
const pretty = await editorText();
check(pretty === '<!--sheet-->\n## Workings {collapse}\nM: 2 kN*m | moment {key}\nx = 3 {key}\n{key results}\n', `Format Document writes the markers in one form (${JSON.stringify(pretty)})`);
await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run());
await page.waitForTimeout(400);
check((await editorText()) === pretty, 'formatting twice changes nothing');

await newTab('<!--sheet-->\nif 1 == 1:\n    ## Inner {collapse}\n    a: 1\n');
{
    let m = [];
    for (let i = 0; i < 30 && !m.some((x) => /outside any if/.test(x[2])); i++) { await page.waitForTimeout(300); m = await markers(); }
    check(m.some((x) => x[0] === 3 && x[1] === 4 && /outside any if/.test(x[2])), `a {collapse} heading inside a block is a warning on its line (${JSON.stringify(m)})`);
}

// The ribbon: in a sheet, Fold inserts a folding heading and Key results the heading and the list.
await newTab('<!--sheet-->\nx: 1\n');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 3, column: 1 }); e.focus(); });
await page.locator('[data-ribbon-tab=textfmt]').click();
await page.locator('[data-ribbon-btn="tf-fold"]').click();
check((await editorText()).includes('## Workings {collapse}'), `Text formatting > Fold inserts a folding heading in a sheet (${JSON.stringify(await editorText())})`);
await page.locator('[data-ribbon-btn="tf-key"]').click();
check((await editorText()).includes('## Key results\n{key results}'), `Text formatting > Key results inserts the heading and the list (${JSON.stringify(await editorText())})`);

// ---- 5. pictures: light and dark, desktop and phone ---------------------------------------------------------------------
if (shots) {
    for (const [name, w, h, scheme] of [['dark', 1500, 900, 'dark'], ['phone-light', 390, 844, 'light'], ['phone-dark', 390, 844, 'dark']]) {
        const o = await open(w, h, scheme, `?theme=${scheme}`);
        await o.page.evaluate((t) => window.calcpadTabs.newUntitled(t), SHEET);
        if (w < 600) { await o.page.tap('.mobile-bar button:nth-child(2)'); }
        await o.page.waitForTimeout(3500);
        await o.page.screenshot({ path: join(shots, `fold-closed-${name}.png`) });
        const sm = o.page.frameLocator('iframe.preview-frame:not(.preview-frame-back)').locator('details.sheet-collapse > summary');
        if (w < 600) await sm.tap(); else await sm.click();
        await o.page.waitForTimeout(600);
        await o.page.screenshot({ path: join(shots, `fold-open-${name}.png`) });
        await o.context.close();
    }
}

check(errors.length === 0, `no page errors (${errors.slice(0, 2).join(' | ')})`);
await browser.close();
server?.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
