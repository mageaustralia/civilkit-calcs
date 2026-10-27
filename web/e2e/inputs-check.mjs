// End-to-end check of the sheet inputs (docs/sheet-syntax.md, "Inputs and templates") in the app, headless Chromium.
//   node e2e/inputs-check.mjs [--dist DIR] [--port 4182] [--url http://127.0.0.1:4801/] [--shots DIR]
//   (after tools/build_wasm.sh and npm run build; with --url it uses a server that is already running)
//
//   1. the Templates group lists its five samples, each tagged as an input form;
//   2. a template opened in Input mode shows the controls (entries, drop-downs, radio buttons, a checkbox);
//   3. changing an entry, a drop-down, a radio button and the checkbox each re-runs the sheet and the result follows;
//   4. Save values writes the entered values under the marker line, the file is still a sheet, and Preview shows them;
//   5. Show compiled and Convert to Calcpad give #UI lines, the report and the Word and Excel exports carry the values;
//   6. Format Document tidies input lines, and typing a bad input line shows an error on its line.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const port = Number(arg('--port', '4182'));
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

const manifest = JSON.parse(readFileSync(join(web, 'calcpad-web', 'public', 'sheet-samples', 'manifest.json'), 'utf8'));
const TEMPLATES = manifest.samples.filter((s) => s.group === 'Templates');

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
const lang = () => page.evaluate(() => window.calcpadActiveEditor.getModel().getLanguageId());
const markers = () => page.evaluate(() => window.monaco.editor.getModelMarkers({}).map((m) => [m.startLineNumber, m.severity, m.message]));
const spaced = (s) => s.replace(/\s+/g, ' ');

/** The text of every result frame (the preview is double buffered, so the stale one can still be there). */
async function frameTexts() {
    const out = [];
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        out.push([await f.evaluate(() => document.body?.innerText ?? '').catch(() => ''), f]);
    }
    return out;
}
/** Waits until a result frame has `marker` and `pattern` in its text; the text, or '' on a timeout. */
async function until(marker, pattern, tries = 100) {
    for (let i = 0; i < tries; i++) {
        for (const [t] of await frameTexts()) if (t.includes(marker) && pattern.test(spaced(t))) return spaced(t);
        await page.waitForTimeout(250);
    }
    return '';
}
/** The frame that holds the form (it has controls), looked up fresh each time. */
async function form(tries = 60) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            if (await f.$('input.calcpad-ui-input').catch(() => null)) return f;
        }
        await page.waitForTimeout(250);
    }
    return null;
}
const mode = (name) => page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 10000 });
/** Runs `act` on the live form until the page shows `pattern`; the form can re-render while it settles. */
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

// ---- 1. the Templates group ----------------------------------------------------------------------
await page.click('[data-ribbon-tab=resources]');
await page.click('[data-ribbon-btn=x-samples]');
await page.waitForSelector('.cprs-s-group');
const groups = await page.locator('.cprs-s-group').allInnerTexts();
check(groups.some((g) => /^Templates \(5\)/.test(g.trim())), `the Samples dialog has a Templates group of 5 (${groups.slice(0, 4).join(', ')})`);
await page.locator('.cprs-s-group', { hasText: /^Templates/ }).click();
check((await page.locator('.cprs-s-item').count()) === TEMPLATES.length && TEMPLATES.length === 5, `the Templates group lists its ${TEMPLATES.length} samples`);
const items = (await page.locator('.cprs-s-item').allInnerTexts()).join(' | ');
check(/input form/.test(items), 'the templates carry the "input form" tag');
await page.locator('.cprs-s-item', { hasText: 'simply supported beam' }).dblclick();
await page.waitForFunction(() => window.calcpadActiveEditor.getValue().includes('Simply supported beam check'));
check((await lang()) === 'calcpad-sheet', 'the template opens in the sheet language mode');

// ---- 2. Input mode shows the controls ---------------------------------------------------------------
let t = await until('Simply supported beam check', /Mmax .*= 56\.26 kN/);
check(/= 56\.26 kN ?· ?m/.test(t) && /Bending: OK/.test(t), `Preview shows the defaults (M = 56.26 kN.m, bending OK) (${t.slice(0, 40)})`);
await mode('Input');
let f = await form();
check(f != null, 'Input mode shows the entries');
if (f) {
    const c = await f.evaluate(() => ({
        entries: [...document.querySelectorAll('input.calcpad-ui-input')].map((e) => e.value),
        dropdowns: [...document.querySelectorAll('select.calcpad-ui-dropdown')].map((s) => [...s.options].map((o) => o.textContent)),
        selected: [...document.querySelectorAll('select.calcpad-ui-dropdown')].map((s) => s.selectedOptions[0]?.textContent),
        radios: [...document.querySelectorAll('input[type=radio]')].map((r) => [r.checked, r.parentElement?.textContent?.trim()]),
        checks: [...document.querySelectorAll('input.calcpad-ui-checkbox')].map((x) => x.checked),
        wrapper: document.querySelectorAll('.sheet-form').length,
    }));
    check(JSON.stringify(c.entries) === '["6","12"]', `two entries hold the defaults without units (${JSON.stringify(c.entries)})`);
    check(c.dropdowns.length === 2 && c.dropdowns[0].join() === '200 mm deep,300 mm deep,400 mm deep' && c.dropdowns[1].join() === 'Structural steel,Aluminium,Timber', `two drop-downs list their options (${JSON.stringify(c.dropdowns)})`);
    check(c.selected[0] === '300 mm deep', `the option marked * is the one selected (${c.selected[0]})`);
    check(c.radios.length === 3 && c.radios[0][0] === true && /L\/250/.test(c.radios[0][1]), `three radio buttons, the first checked (${JSON.stringify(c.radios)})`);
    check(c.checks.length === 1 && c.checks[0] === true, 'one checkbox, checked');
    check(c.wrapper === 1, 'the run of inputs is one sheet-form block');
}
if (shots) await page.screenshot({ path: join(shots, 'template-input-form.png') });

// ---- 3. each kind of input re-runs the sheet -------------------------------------------------------------
t = await change('Simply supported beam check', /Mmax .*= 100\.0\d kN/, async (ff) => { await ff.fill('input.calcpad-ui-input >> nth=0', '8'); await ff.press('input.calcpad-ui-input >> nth=0', 'Enter'); });
check(/= 100\.0\d kN ?· ?m/.test(t), `an entry: L = 8 m gives M = 100.0 kN.m (${t.slice(t.indexOf('Mmax'), t.indexOf('Mmax') + 60)})`);
t = await change('Simply supported beam check', /The material is timber\./, async (ff) => { await ff.locator('select.calcpad-ui-dropdown').nth(1).selectOption({ label: 'Timber' }); });
check(/The material is timber\./.test(t) && /Bending: the stress exceeds/.test(t), 'a drop-down: Timber changes the table lookups and fails the bending check');
t = await change('Simply supported beam check', /δlim = L limit = 8 m 500 = 16 mm/, async (ff) => { await ff.locator('input[type=radio]').nth(2).evaluate((e) => e.click()); });
check(/= 16 mm/.test(t), `a radio button: the limit L/500 gives 16 mm over 8 m (${t.slice(t.indexOf('δlim'), t.indexOf('δlim') + 50)})`);
t = await change('Simply supported beam check', /wtot = w \+ wsw = 12 kN ∕ m \+ 0 kN/, async (ff) => { await ff.locator('input.calcpad-ui-checkbox').evaluate((e) => e.click()); });
check(/wtot = w \+ wsw = 12 kN ∕ m \+ 0 kN ∕ m = 12 kN/.test(t), 'the checkbox: own weight off leaves the load at 12 kN/m');
if (shots) await page.screenshot({ path: join(shots, 'template-input-changed.png') });

// ---- 4. Save values ---------------------------------------------------------------------------------------
await page.locator('.toolbar-btn', { hasText: /^Save values$/ }).click({ timeout: 10000 });
await page.waitForTimeout(800);
let src = await editorText();
const lines = src.split('\n');
check(lines[0] === '<!--sheet-->' && lines[1].startsWith("'<!--{\"uiOverrides\""), `Save values writes the comment under the sheet marker (${JSON.stringify(lines.slice(0, 2)).slice(0, 120)})`);
check((await lang()) === 'calcpad-sheet', 'the document is still a sheet after saving the values');
for (const k of ['L:1', 'mat:1', 'sw:1']) check(src.includes(`"${k}"`), `the saved comment has ${k}`);
const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
await mode('Report');
if (await dontSave.waitFor({ timeout: 1500 }).then(() => true, () => false)) await dontSave.click();
t = await until('Simply supported beam check', /L = 8 m/);
check(/Mmax .*= 96 kN ?· ?m/.test(t) && /The material is timber\./.test(t), `after leaving the form, the report applies the saved values (L = 8 m, timber, no own weight) (${t.slice(t.indexOf('Inputs'), t.indexOf('Inputs') + 80)})`);
await mode('Preview');

// ---- 5. compiled view, conversion, exports ---------------------------------------------------------------------
await page.click('[data-ribbon-btn=x-compiled]');
await page.waitForSelector('[data-testid=compiled-view] .view-lines');
const compiled = (await page.locator('[data-testid=compiled-view]').innerText()).replace(/ /g, ' ');
check(compiled.includes('#UI L = 6m') && compiled.includes('#UI {"type":"dropdown"') && compiled.includes('#UI {"type":"checkbox"}') && !compiled.includes('sheet-form') && !compiled.includes('sheet-note'), 'Show compiled gives #UI lines, with no wrapper block and no styled spans');
check(compiled.includes("'span") && compiled.includes("'material -'mat = 1"), 'the notes are plain Calcpad comments, the label of a drop-down comes before its name');
await page.locator('#cprs-compiled button', { hasText: 'Close' }).click();
const exports = await page.evaluate(async () => {
    const content = window.calcpadActiveEditor.getValue();
    const post = async (ep, extra) => { const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, ...extra }) }); return r; };
    const html = await (await post('convert', { uiOverrides: { 'L:1': '9', 'mat:1': '3' } })).text();
    const docx = await post('docx', {});
    const xlsx = await post('xlsx', {});
    return { html, docx: [docx.status, (await docx.arrayBuffer()).byteLength], xlsx: [xlsx.status, (await xlsx.arrayBuffer()).byteLength] };
});
const reportText = spaced(exports.html.replace(/<[^>]+>/g, ' '));
check(/L = 9 m/.test(reportText) && /timber/.test(reportText), `the report applies the values the form sends (L = 9 m, timber) (${reportText.slice(reportText.indexOf('Inputs'), reportText.indexOf('Inputs') + 200)})`);
check(exports.docx[0] === 200 && exports.docx[1] > 2000, `a template exports to Word (${exports.docx[1]} bytes)`);
check(exports.xlsx[0] === 200 && exports.xlsx[1] > 2000, `a template exports to Excel (${exports.xlsx[1]} bytes)`);
await page.click('[data-ribbon-btn=x-convert]');
await page.waitForTimeout(600);
const conv = { lang: await lang(), text: await editorText() };
check(conv.lang === 'calcpad' && conv.text.includes('#UI L = 6m') && !conv.text.includes('<!--sheet-->') && !conv.text.includes('<div'), 'Convert to Calcpad leaves #UI lines and no wrapper');
t = await until('Simply supported beam check', /Mmax/);
check(/= 56\.26 kN|= 96 kN/.test(t) && !/Error in/.test(t), 'the converted worksheet still runs');

// ---- 6. every template runs, formatting, a bad input line ---------------------------------------------------------------
for (const s of TEMPLATES) {
    const text = readFileSync(join(web, 'calcpad-web', 'public', s.file.replace(/^/, '')), 'utf8');
    await newTab(text);
    const title = text.split('\n').find((l) => l.startsWith('# ')).slice(2);
    const r = await until(title, /./, 60);
    await page.waitForTimeout(1200);
    const r2 = await until(title, /./, 20);
    check(r2.length > 200 && !/Error in|Sheet syntax:/.test(r2), `template "${s.title}" runs without errors in the preview`);
}
await newTab('<!--sheet-->\n## Inputs\nL:?6 m|span\nwidth :  ?  300  mm\nE: ?[ Steel=200 GPa,*Timber =11 GPa ]|material\nM: L * 1\n');
await page.evaluate(() => window.calcpadActiveEditor.focus());
await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run());
await page.waitForFunction(() => window.calcpadActiveEditor.getValue().includes('L    : ? 6 m | span'), null, { timeout: 8000 }).catch(() => {});
const pretty = await editorText();
check(pretty === '<!--sheet-->\n## Inputs\nL    : ? 6 m | span\nwidth: ? 300 mm\nE    : ? [Steel = 200 GPa, *Timber = 11 GPa] | material\nM    : L * 1\n' || pretty.includes('L    : ? 6 m | span') && pretty.includes('E    : ? [Steel = 200 GPa, *Timber = 11 GPa] | material'), `Format Document tidies input lines (${JSON.stringify(pretty.slice(0, 120))})`);
await page.evaluate(() => window.calcpadActiveEditor.getAction('editor.action.formatDocument').run());
await page.waitForTimeout(500);
check((await editorText()) === pretty, 'formatting twice changes nothing');

await newTab('<!--sheet-->\n# Bad input\nL: ? 6 m\nw: ? [a = 1]\nq: ? 2*3\nr: ? [a = 1, b = 2\nok: ? 4 m\n');
let m = [];
for (let i = 0; i < 30 && m.filter((x) => x[1] === 8).length < 3; i++) { await page.waitForTimeout(300); m = await markers(); }
const bad = m.filter((x) => x[1] === 8).map((x) => x[0]).sort((a, b) => a - b);
check(JSON.stringify([...new Set(bad)]) === '[4,5,6]', `a bad input line is an error marker on its own line (${JSON.stringify(m.filter((x) => x[1] === 8))})`);
check(m.some((x) => x[0] === 4 && /at least two options/.test(x[2])) && m.some((x) => x[0] === 5 && /number with an optional unit/.test(x[2])), 'the messages say what to write');
t = await until('Bad input', /Sheet syntax: input/);
check(/Sheet syntax: input: a choice needs at least two options/.test(t) && /ok = 4 m|ok =/.test(t), 'the report shows each error in place and still runs the lines after it');

check(pageErrors.length === 0, `no page errors (${pageErrors.slice(0, 2).join(' | ')})`);
await browser.close();
server?.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
