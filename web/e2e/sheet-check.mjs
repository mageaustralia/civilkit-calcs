// End-to-end check of the sheet syntax (docs/sheet-syntax.md) in the built app, headless Chromium.
//   node e2e/sheet-check.mjs [--dist DIR] [--port 4181]      (after tools/build_wasm.sh and npm run build)
//
//   1. a sheet sample opened from the Samples browser shows the hand-checked numbers;
//   2. LaTeX in prose renders (KaTeX), loaded from the app's own files;
//   3. an error in a sheet line, a raw Calcpad line and a block is reported on the right sheet line
//      (the report's error banner, its links and the editor markers);
//   4. sheet lines, prose, LaTeX and raw Calcpad alternate and share variables both ways;
//   5. Show compiled Calcpad, Convert to Calcpad, the Word and Excel exports of a sheet;
//   6. the ribbon inserts the active file's form: a sheet gets the sheet form, a Calcpad file the Calcpad form.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const port = Number(arg('--port', '4181'));
const origin = `http://127.0.0.1:${port}`;
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

const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 900 }, acceptDownloads: true });
const outside = [];
await context.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
const page = await context.newPage();
// Requests of the app's own page (the Changelog button opens GitHub in a tab of its own, which is not the app).
page.on('request', (r) => { if (!r.url().startsWith(origin) && !/^(data|blob):/.test(r.url())) outside.push(r.url()); });
const pageErrors = [];
// Monaco rejects the requests it cancels (a model replaced under a pending provider call) with "Canceled".
page.on('pageerror', (e) => { if (e.message !== 'Canceled') pageErrors.push(e.message); });
await page.goto(`${origin}/`);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 30000 });

/** The result frame whose text contains `marker`: [text, frame]. */
async function result(marker, tries = 80) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
            if (t.includes(marker)) return [t.replace(/\s+/g, ' '), f];
        }
        await page.waitForTimeout(250);
    }
    return ['', null];
}
const newTab = (text) => page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
const markers = () => page.evaluate(() => window.monaco.editor.getModelMarkers({}).map((m) => [m.startLineNumber, m.severity, m.message]));

// ---- 1. a sample from the Samples browser ---------------------------------------------------
await page.click('[data-ribbon-tab=resources]');
await page.click('[data-ribbon-btn=x-samples]');
await page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }).click();
const SHEET_COUNT = JSON.parse(readFileSync(join(web, 'calcpad-web', 'public', 'sheet-samples', 'manifest.json'), 'utf8')).samples.filter((s) => s.group === 'Sheet syntax').length;
check((await page.locator('.cprs-s-item').count()) === SHEET_COUNT, `the Sheet syntax group lists its ${SHEET_COUNT} samples`);
check(((await page.locator('.cprs-s-group').allInnerTexts())[1] ?? '').startsWith('Sheet syntax'), 'the Sheet syntax group is the first group of the Samples dialog');
await page.locator('.cprs-s-item', { hasText: 'Beam bending with real units' }).dblclick();
let [t] = await result('Beam bending check');
check(/M star = [\s\S]*= 54 kN ?· ?m/.test(t) || t.includes('= 54 kN'), `beam sample: moment 54 kN.m (${t.slice(0, 80)})`);
check(t.includes('= 369 kN') && t.includes('= 332.1 kN') && t.includes('= 0.163'), 'beam sample: 369 kN.m, 332.1 kN.m, utilisation 0.163 (units computed by the engine)');
check(t.includes('OK: the beam passes in bending.') && !t.includes('Fails: increase'), 'beam sample: the conditional text');
check((await page.evaluate(() => window.calcpadActiveEditor.getModel().getLanguageId())) === 'calcpad-sheet', 'the sample opens in the sheet language mode');

// ---- 2. LaTeX ---------------------------------------------------------------------------------
await page.locator('[data-ribbon-btn=x-samples]').click();
await page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }).click();
await page.locator('.cprs-s-item', { hasText: 'LaTeX prose' }).dblclick();
let frame;
[t, frame] = await result('Bending stress with LaTeX prose');
await page.waitForTimeout(800);
[t, frame] = await result('Bending stress with LaTeX prose');
const katex = await frame.evaluate(() => ({ spans: document.querySelectorAll('.katex').length, display: document.querySelectorAll('div.sheet-tex .katex-display').length, placeholders: [...document.querySelectorAll('.sheet-tex')].filter((e) => !e.querySelector('.katex')).length, frac: document.querySelectorAll('.mfrac').length, scripts: document.querySelectorAll('script').length }));
check(katex.spans >= 5 && katex.display === 1 && katex.placeholders === 0, `LaTeX renders with KaTeX: ${katex.spans} formulas, ${katex.display} display, ${katex.placeholders} left as placeholders`);
check(katex.frac >= 1 && t.includes('95.29 MPa'), 'the display fraction is typeset and the stress is 95.29 MPa');
check(await frame.evaluate(() => [...document.querySelectorAll('script')].every((s) => !/katex/i.test(s.src + s.textContent))), 'the rendered page has no script for the formulas');

// ---- 3. an error maps to the right sheet line -----------------------------------------------
const bad = ['<!--sheet-->', '# Error mapping check', '', 'a: 5 kN', 'Some prose with $x^2$.', 'b: a + missing_value', '>> k = nothere + 1', '', 'if a > 1 kN:', '    c: a + also_missing', '', 'd: 2 m'].join('\n');
await newTab(bad);
[t] = await result('Error mapping check');
for (let i = 0; i < 20 && !/on line/.test(t); i++) { await page.waitForTimeout(400); [t] = await result('Error mapping check'); }
check(/on line \[6\]/.test(t) && /on line \[7\]/.test(t) && /on line \[10\]/.test(t), `the report names sheet lines 6, 7 and 10 (${t.match(/on line \[\d+\]/g)})`);
const banner = await page.locator('.error-banner, [class*=error]').allInnerTexts().catch(() => []);
let m = [];
for (let i = 0; i < 30 && m.filter((x) => x[1] === 8).length < 3; i++) { await page.waitForTimeout(300); m = await markers(); }
const errLines = m.filter((x) => x[1] === 8).map((x) => x[0]).sort((a, b) => a - b);
check(JSON.stringify(errLines) === '[6,7,10]', `editor markers are on sheet lines 6, 7 and 10 (${JSON.stringify(errLines)})`);
void banner;
// A compile problem (mixed tabs and spaces) is a marker on its line too.
await newTab('<!--sheet-->\nif 1 > 0:\n\ta: 1\n\t    b: 2\n');
for (let i = 0; i < 30; i++) { await page.waitForTimeout(300); m = await markers(); if (m.some((x) => /tabs and spaces/.test(x[2]))) break; }
check(m.some((x) => x[0] === 4 && /tabs and spaces/.test(x[2])), 'mixed tabs and spaces are reported on their sheet line');

// ---- 4. sheet lines, LaTeX prose and raw Calcpad together ---------------------------------------
await page.locator('[data-ribbon-btn=x-samples]').click();
await page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }).click();
await page.locator('.cprs-s-item', { hasText: 'Mixing with Calcpad' }).dblclick();
[t] = await result('Sheet lines and Calcpad code together');
check(/total = total \+ bay = 5 m \+ 2\.5 m = 7\.5 m/.test(t) && /length ?= total · 1 = 7\.5 m · 1 = 7\.5 m/.test(t), 'a raw Calcpad loop reads sheet variables and a later sheet line reads what it defined');
check((t.match(/Stage one\./g) ?? []).length === 1 && (t.match(/flag = k = \d/g) ?? []).length === 2 && t.includes('= 30 kN'), 'friendly for/if blocks: 3 iterations, else once, then twice');
const mix = ['<!--sheet-->', 'n: 4', 'phi: 0.5', 'Prose with $\\alpha$ here.', '>> s = 0', '```calcpad', '#for i = 1 : n', '    s = s + i*phi', '    z = s + undefined_name', '#loop', '```', 'Between: $$e = m c^2$$', 't: s * 2'].join('\n');
await newTab(mix);
[t] = await result('Prose with');
await page.waitForTimeout(1500);
check(/on line \[9\]/.test(t), `an error inside a raw Calcpad block maps to its sheet line 9 (${t.match(/on line \[\d+\]/g)})`);
check(t.includes('t = s · 2'), 'sheet lines after the raw block still run');

// ---- 5. compiled view, conversion, exports ------------------------------------------------------
await page.locator('[data-ribbon-btn=x-samples]').click();
await page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }).click();
await page.locator('.cprs-s-item', { hasText: 'Beam bending with real units' }).dblclick();
await result('Beam bending check');
await page.click('[data-ribbon-btn=x-compiled]');
await page.waitForSelector('[data-testid=compiled-view] .view-lines');
const compiled = (await page.locator('[data-testid=compiled-view]').innerText()).replace(/\u00a0/g, ' ');
check(compiled.includes('M_star = w * L^2 / 8|kN*m') && compiled.includes('#if utilisation') && !compiled.includes('sheet-note') && compiled.includes("'maximum bending moment"), 'Show compiled Calcpad shows hand-written looking Calcpad: plain comments, no styled spans');
check(await page.locator('.monaco-editor').first().isVisible(), 'the compiled view sits beside the editor and does not hide it');
if (shots) await page.screenshot({ path: join(shots, 'sheet-with-compiled-view.png') });
await page.locator('#cprs-compiled button', { hasText: 'Close' }).click();
const bodies = await page.evaluate(async () => {
    const content = window.calcpadActiveEditor.getValue();
    const post = async (ep) => { const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true }) }); const b = new Uint8Array(await r.arrayBuffer()); return [r.status, String.fromCharCode(b[0], b[1]), b.length]; };
    return { docx: await post('docx'), xlsx: await post('xlsx') };
});
check(bodies.docx[0] === 200 && bodies.docx[1] === 'PK' && bodies.docx[2] > 2000, `a sheet exports to Word (${bodies.docx[2]} bytes)`);
check(bodies.xlsx[0] === 200 && bodies.xlsx[1] === 'PK' && bodies.xlsx[2] > 2000, `a sheet exports to Excel (${bodies.xlsx[2]} bytes)`);
// Print and the exports use the same request path: the LaTeX sample, as a print render and as Word.
const tex = await page.evaluate(async () => {
    const content = await (await fetch('/sheet-samples/LaTeX prose.sheet')).text();
    const post = (ep, extra) => fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, ...extra }) });
    const html = await (await post('convert', { forPrint: true })).text();
    const docx = await post('docx', {});
    return { katex: (html.match(/class="katex"/g) ?? []).length, placeholders: (html.match(/class="sheet-tex"[^>]*><\/(span|div)>/g) ?? []).length, style: html.includes('id="cprs-katex"'), docx: docx.status, bytes: (await docx.arrayBuffer()).byteLength };
});
check(tex.katex >= 5 && tex.placeholders === 0 && tex.style, `the print render of a LaTeX sheet has typeset formulas and the stylesheet (${tex.katex} formulas)`);
check(tex.docx === 200 && tex.bytes > 2000, 'a LaTeX sheet exports to Word (formulas as TeX source)');
await page.click('[data-ribbon-btn=x-convert]');
await page.waitForTimeout(500);
const converted = await page.evaluate(() => ({ lang: window.calcpadActiveEditor.getModel().getLanguageId(), text: window.calcpadActiveEditor.getValue() }));
check(converted.lang === 'calcpad' && !converted.text.includes('<!--sheet-->') && converted.text.includes('M_star = w * L^2 / 8|kN*m'), 'Convert to Calcpad replaces the sheet with plain Calcpad and leaves the sheet mode');
[t] = await result('Beam bending check');
check(t.includes('= 332.1 kN'), 'the converted worksheet renders the same numbers');

// ---- 6. the ribbon follows the active file's mode -----------------------------------------------
await newTab('<!--sheet-->\n# T\n');
await page.click('[data-ribbon-tab=math]');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 2, column: 4 }); });
await page.click('[data-ribbon-btn=r-heading]');
check((await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('## Heading'), 'in a sheet, Heading inserts "## Heading"');
await page.click('[data-ribbon-btn=r-plot]');
check((await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('plot sin(x) for x in 0..2*π'), 'in a sheet, Plot inserts the sheet plot line');
await newTab('a = 1\n');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 2, column: 1 }); });
await page.click('[data-ribbon-btn=r-heading]');
check((await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('"Heading') && !(await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('## Heading'), 'in a Calcpad file, Heading still inserts the Calcpad heading');
await newTab('<!--sheet-->\n');
await page.waitForTimeout(700);
await page.click('[data-ribbon-tab=io]');
const raw = await page.locator('.rb-raw').count();
check(raw > 0, `buttons that only insert raw Calcpad are marked in a sheet (${raw} on the Input/Output tab)`);
await page.click('[data-ribbon-tab=math]');
check((await page.locator('[data-ribbon-btn=r-heading].rb-raw').count()) === 0, 'a button with a sheet form is not marked');

// ---- 7. New defaults to a sheet, with the Calcpad worksheet as the second entry ---------------------------------
await page.click('[data-ribbon-btn=q-new]');
let nw = await page.evaluate(() => ({ lang: window.calcpadActiveEditor.getModel().getLanguageId(), text: window.calcpadActiveEditor.getValue() }));
check(nw.lang === 'calcpad-sheet' && nw.text.startsWith('<!--sheet-->') && /^w: 12 kN\/m/m.test(nw.text) && /^M: /m.test(nw.text) && nw.text.includes('|'), 'the quick-access New opens a sheet with an input, a calculation and a note');
[t] = await result('Untitled calculation');
check(/= 54 kN ?· ?m/.test(t) || t.includes('54 kN'), 'the new sheet starter calculates (54 kN.m)');
await page.locator('[data-ribbon-btn=file]').click();
const menu = await page.locator('.cprs-menu button').allInnerTexts();
check(/^New sheet/.test(menu[0] ?? '') && /^New Calcpad worksheet/.test(menu[1] ?? ''), `File menu: New sheet first, New Calcpad worksheet second (${menu.slice(0, 2).map((x) => x.split('\n')[0])})`);
await page.locator('.cprs-menu button', { hasText: 'New Calcpad worksheet' }).click();
nw = await page.evaluate(() => ({ lang: window.calcpadActiveEditor.getModel().getLanguageId(), text: window.calcpadActiveEditor.getValue() }));
check(nw.lang === 'calcpad' && nw.text === '', 'New Calcpad worksheet opens an empty Calcpad worksheet');
await page.keyboard.press('Escape');
await page.evaluate(() => window.calcpadActiveEditor.focus());

// ---- 8. .sheet open (drop), edit, save, reopen ------------------------------------------------------------------------
const dropFile = (name, text) => page.evaluate(async ([n, t]) => {
    const dt = new DataTransfer();
    dt.items.add(new File([t], n, { type: 'text/plain' }));
    document.dispatchEvent(new DragEvent('dragover', { dataTransfer: dt, bubbles: true, cancelable: true }));
    document.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true }));
}, [name, text]);
await dropFile('opened.sheet', 'Plain text without a marker.\nk: 2 m | length\n');
await page.waitForFunction(() => window.calcpadActiveEditor.getValue().includes('Plain text without a marker.'));
let cur = await page.evaluate(() => ({ lang: window.calcpadActiveEditor.getModel().getLanguageId(), text: window.calcpadActiveEditor.getValue() }));
check(cur.lang === 'calcpad-sheet' && cur.text.startsWith('<!--sheet-->\n'), 'a dropped .sheet file opens as a sheet; the marker line is added when the file has none');
await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); e.executeEdits('t', [{ range: m.getFullModelRange(), text: m.getValue() + 'n: k * 3\n' }]); });
[t] = await result('Plain text without a marker.');
for (let i = 0; i < 20 && !/n = k/.test(t); i++) { await page.waitForTimeout(300); [t] = await result('Plain text without a marker.'); }
check(/n = k . 3 = 2 m . 3 = 6 m/.test(t), 'the edited .sheet renders');
const [download] = await Promise.all([page.waitForEvent('download'), page.click('[data-ribbon-btn=q-save]')]);
const savedName = download.suggestedFilename();
const savedText = readFileSync(await download.path(), 'utf8');
check(savedName === 'opened.sheet' && savedText.startsWith('<!--sheet-->\n') && savedText.includes('n: k * 3'), `Save keeps the .sheet name and the marker (${savedName})`);
await dropFile('reopened.sheet', savedText);
await page.waitForFunction(() => window.calcpadTabs.activeTab?.title?.includes('reopened') || window.calcpadTabs.activeTab?.filePath?.includes('reopened'));
cur = await page.evaluate(() => ({ lang: window.calcpadActiveEditor.getModel().getLanguageId(), text: window.calcpadActiveEditor.getValue() }));
check(cur.lang === 'calcpad-sheet' && cur.text === savedText, 'the saved .sheet reopens unchanged, in sheet mode');
[t] = await result('Plain text without a marker.');
check(/n = k . 3 = 2 m . 3 = 6 m/.test(t), 'the reopened sheet renders the same numbers');

// ---- 9. prettify, go to definition, hover, references -----------------------------------------------------------------------
await newTab('<!--sheet-->\n# T\n## Inputs\nL:6 m|span\nfy :  300 MPa   |  yield\nMs: fy*L -> kN\n');
await page.evaluate(() => window.calcpadActiveEditor.focus());
await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run());
await page.waitForFunction(() => window.calcpadActiveEditor.getValue().includes('L : 6 m | span'), null, { timeout: 8000 }).catch(() => {});
const pretty = await page.evaluate(() => window.calcpadActiveEditor.getValue());
check(pretty === '<!--sheet-->\n# T\n\n## Inputs\nL : 6 m | span\nfy: 300 MPa | yield\nMs: fy*L -> kN\n'.replace('Ms:', 'Ms:') || pretty.includes('L : 6 m | span') && pretty.includes('fy: 300 MPa | yield') && pretty.includes('\n\n## Inputs'), `Format Document prettifies a sheet (${JSON.stringify(pretty.slice(0, 90))})`);
// F12 on a use of fy goes to its definition line.
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 7, column: 6 }); e.focus(); });
await page.keyboard.press('F12');
await page.waitForTimeout(800);
const defLine = await page.evaluate(() => window.calcpadActiveEditor.getPosition().lineNumber);
check(defLine === 6, `go to definition of fy lands on its line (${defLine})`);
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 7, column: 6 }); e.focus(); e.trigger('t', 'editor.action.showHover', {}); });
await page.waitForSelector('.monaco-hover-content', { timeout: 5000 }).catch(() => {});
const hover = await page.locator('.monaco-hover-content').first().innerText().catch(() => '');
check(/fy: 300 MPa/.test(hover) && /line 6/.test(hover), `hover shows the definition (${hover.replace(/\s+/g, ' ').slice(0, 60)})`);
await page.keyboard.press('Escape');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 6, column: 2 }); e.focus(); });
await page.keyboard.press('Shift+F12');
await page.waitForSelector('.reference-zone-widget', { timeout: 5000 }).catch(() => {});
const refRows = await page.locator('.reference-zone-widget .monaco-list-row').count();
check(refRows >= 2, `find references on fy lists its uses in code lines (${refRows} rows)`);
await page.keyboard.press('Escape');

// ---- 10. Text Formatting, the Insert tab, the changelog ----------------------------------------------------------
await newTab('<!--sheet-->\nword\n');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setSelection({ startLineNumber: 2, startColumn: 1, endLineNumber: 2, endColumn: 5 }); });
await page.click('[data-ribbon-tab=textfmt]');
await page.click('[data-ribbon-btn=tf-b]');
check((await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('**word**'), 'in a sheet, Bold wraps the selection in **');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 3, column: 1 }); });
await page.click('[data-ribbon-btn=tf-ul]');
await page.click('[data-ribbon-btn=tf-h2]');
const md = await page.evaluate(() => window.calcpadActiveEditor.getValue());
check(md.includes('- First\n- Second') && md.includes('## text'), 'in a sheet, Bullets and H2 insert Markdown');
await page.click('[data-ribbon-btn=tf-link]');
check((await page.evaluate(() => window.calcpadActiveEditor.getValue())).includes('[text](https://example.com)'), 'in a sheet, Link inserts a Markdown link');
if (shots) await page.screenshot({ path: join(shots, 'sheet-textformatting.png') });
// Insert tab: a Calcpad-only item is marked in a sheet and not in a Calcpad file.
const search = page.locator('.insert-tab .search-input');
await search.fill('#UI');
await page.waitForTimeout(600);
const rawInSheet = await page.locator('.insert-tab .raw-insert').count();
await newTab('a = 1\n');
await page.waitForTimeout(900);
const rawInCalcpad = await page.locator('.insert-tab .raw-insert').count();
check(rawInSheet > 0 && rawInCalcpad === 0, `the Insert tab marks raw Calcpad items in a sheet (${rawInSheet}) and not in a Calcpad file (${rawInCalcpad})`);
await search.fill('');
// The changelog button and the About link open GitHub in a new tab, with noopener.
await page.click('[data-ribbon-tab=resources]');
const [popup] = await Promise.all([context.waitForEvent('page'), page.click('[data-ribbon-btn=x-changelog]')]);
check(/^https:\/\/github\.com\/mageaustralia\/civilkit-calcs\/blob\/main\/CHANGELOG\.md$/.test(popup.url()) || popup.url() === 'about:blank' || popup.url().includes('CHANGELOG.md'), `Changelog opens the changelog in a new tab (${popup.url()})`);
await popup.close().catch(() => {});
await page.click('[data-ribbon-btn=x-about]');
const link = page.locator('#cprs-dialog a', { hasText: 'the changelog' });
check((await link.getAttribute('href')) === 'https://github.com/mageaustralia/civilkit-calcs/blob/main/CHANGELOG.md' && (await link.getAttribute('rel')) === 'noopener' && (await link.getAttribute('target')) === '_blank', 'About: a What\'s new link to the changelog, new tab, noopener');
if (shots) await page.screenshot({ path: join(shots, 'about-whats-new.png') });
await page.keyboard.press('Escape');

// ---- the whole run --------------------------------------------------------------------------------
check(outside.length === 0, `no request left the app (${outside.slice(0, 3)})`);
check(pageErrors.length === 0, `no page errors (${pageErrors.slice(0, 2)})`);
await browser.close();
server.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
