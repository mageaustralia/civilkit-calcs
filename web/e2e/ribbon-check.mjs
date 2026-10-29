// Checks the ribbon in the built app served at --url (default http://127.0.0.1:4175/):
// all ten tabs, palette popovers, inserts, quick access, exports and a phone-width pass, with screenshots.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/ribbon-check.mjs [--url U] [--out DIR] [--lint] [--dump-library FILE]
// --dump-library saves the snippet library the page uses, for e2e/verify-ribbon-snippets.mjs --library.
// --lint also loads every line snippet into the editor and fails on any linter error marker.
import { chromium } from 'playwright';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { allSnippets, PRELUDE, RIBBON } from '../calcpad-web/src/wasm/ribbon-data.ts';
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const here = new URL('.', import.meta.url).pathname;
const url = arg('--url', 'http://127.0.0.1:4175/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-shots'));
mkdirSync(out, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
async function open(w, h, scheme) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, colorScheme: scheme, acceptDownloads: true });
    await ctx.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(String(e) + (process.env.STACK ? '\n' + e.stack : '')));
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    return { page, errors };
}
const clickKey = (page, key) => page.evaluate(k => { const el = [...document.querySelectorAll('.rb-pop [data-snip-key]')].find(e => e.getAttribute('data-snip-key') === k); if (!el) throw new Error('no snippet ' + k); el.click(); }, key);
const text = page => page.evaluate(() => window.calcpadActiveEditor.getValue());
const tab = (page, t) => page.locator(`[data-ribbon-tab=${t}]`).click();
const btn = (page, id) => page.locator(`[data-ribbon-btn="${id}"]`);
const popVisible = page => page.locator('.rb-pop').isVisible();
const editorFocused = page => page.evaluate(() => !!document.activeElement?.closest('.monaco-editor'));
/** True once a results frame holds rendered output (frames are sandboxed, so ask Playwright, not the page). */
async function calculated(page) {
    for (let i = 0; i < 60; i++) {
        const texts = [];
        for (const f of page.frames()) if (f !== page.mainFrame()) texts.push(await f.evaluate(() => document.body?.innerText ?? '').catch(() => ''));
        const busy = await page.evaluate(() => /Calculating[…]|Calculating\.\.\./.test(document.body.innerText));
        if (texts.some(t => t.length > 100) && !texts.some(t => /Calculating/.test(t)) && !busy) { await page.waitForTimeout(800); const again = await page.evaluate(() => /Calculating[…]|Calculating\.\.\./.test(document.body.innerText)); if (!again) return true; }
        await page.waitForTimeout(500);
    }
    return false;
}

// ---- desktop, light ----------------------------------------------------------------------
let { page, errors } = await open(1400, 820, 'light');
for (const t of RIBBON) {
    await tab(page, t.id);
    const n = await page.locator('.rb-panel .rb-big, .rb-panel .rb-small').count();
    check(n > 0 && await page.locator(`[data-ribbon-panel=${t.id}]`).isVisible(), `tab ${t.label} shows ${n} buttons`);
}
check(RIBBON.length === 10, 'the ribbon has ten tabs');

// ---- no duplicate buttons -----------------------------------------------------------------
{
    // Same label twice on one tab is a duplicate. (Allow-list: none today; add "tab:Label" here with the reason.)
    const ALLOW = new Set([]);
    const dup = [];
    const quick = new Set();
    for (const t of RIBBON) {
        const seen = new Set();
        for (const g of t.groups) for (const b of g.buttons) {
            const k = `${t.id}:${b.label}`;
            if (seen.has(b.label) && !ALLOW.has(k)) dup.push(k);
            seen.add(b.label);
        }
    }
    check(dup.length === 0, `no two buttons on a tab share a label (${dup.join(', ') || 'none'})`);
    // One popover per library list: no two buttons of the whole ribbon open the same list.
    const lists = new Map();
    for (const t of RIBBON) for (const g of t.groups) for (const b of g.buttons) if (b.more && b.more !== 'all') lists.set(b.more, [...(lists.get(b.more) ?? []), `${t.id}:${b.label}`]);
    const twice = [...lists].filter(([, v]) => v.length > 1).map(([k, v]) => `${k} -> ${v.join(' & ')}`);
    check(twice.length === 0, `each library list is opened by one button (${twice.join('; ') || 'none'})`);
    // And no two lists share a library category.
    const { MORE } = await import('../calcpad-web/src/wasm/ribbon-data.ts');
    const cats = new Map();
    for (const [k, v] of Object.entries(MORE)) if (k !== 'all') for (const c of v.categories) cats.set(c, [...(cats.get(c) ?? []), k]);
    const shared = [...cats].filter(([, v]) => v.length > 1).map(([c, v]) => `${c} in ${v.join(' & ')}`);
    check(shared.length === 0, `no library category is in two lists (${shared.join('; ') || 'none'})`);
}
{
    const have = await page.evaluate(() => Object.keys(window.calcpadRibbonIcons.big));
    const need = new Set();
    for (const t of RIBBON) for (const g of t.groups) for (const b of g.buttons) if (b.size !== 'small' && b.icon) need.add(b.icon);
    const missing = [...need].filter(i => !have.includes(i));
    check(missing.length === 0, `every big button has 32px artwork (${need.size} icons; missing: ${missing.join(', ') || 'none'})`);
}
await page.evaluate(async () => {
    const m = await (await fetch('samples/manifest.json')).json();
    const s = m.samples.find(x => /beam/i.test(x.title) && !x.ui && !x.python) ?? m.samples[0];
    await window.calcpadFiles.samples.openSample(s);
});
const done = await calculated(page);
await tab(page, 'math');
// Constants palette: open, insert pi, closes, focus returns to the editor
await page.evaluate(() => window.calcpadActiveEditor.setValue('r = '));
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 1, column: 5 }); e.focus(); });
await btn(page, 'p-constants').click();
check(await popVisible(page), 'Constants popover opens');
{
    const heads = await page.locator('.rb-pop-h').allInnerTexts();
    check(/^common/i.test(heads[0] ?? '') && heads.some(h => /physics/i.test(h)) && heads.some(h => /^constants$/i.test(h)), `the Constants popover opens with Common, then the library list (${heads.slice(0, 4).join(' | ')})`);
}
await page.locator('[data-ribbon-glyph="π"]').first().click();
check((await text(page)).includes('r = π'), 'Constants > pi inserts at the cursor');
check(!(await popVisible(page)) && await editorFocused(page), 'the popover closes after a pick and the editor has focus');
await btn(page, 'p-constants').click();
await page.keyboard.press('Escape');
check(!(await popVisible(page)) && await editorFocused(page), 'Escape closes the popover and returns focus to the editor');
await btn(page, 'p-operators').click();
await page.mouse.click(700, 500);
check(!(await popVisible(page)), 'a click outside closes the popover');
// regions insert on their own line
await page.evaluate(() => window.calcpadActiveEditor.setValue('a = 1'));
await btn(page, 'r-heading').click();
check((await text(page)) === 'a = 1\n"Heading\n', `Math > Heading inserts on its own line (${JSON.stringify(await text(page))})`);
// quick access undo
await btn(page, 'q-undo').click();
check((await text(page)) === 'a = 1', 'quick access Undo reverts the insert');
await btn(page, 'q-redo').click();
check((await text(page)).includes('"Heading'), 'quick access Redo restores it');
// input/output tab
await tab(page, 'io');
await btn(page, 'u-entry').click();
check((await text(page)).includes('#UI L = 10m'), 'Input/Output > Entry inserts a #UI line');
// functions tab popover
await tab(page, 'functions');
await page.evaluate(() => { window.calcpadActiveEditor.setValue('r = '); window.calcpadActiveEditor.setPosition({ lineNumber: 1, column: 5 }); });
await btn(page, 'fn-trig').click();
await page.locator('[data-ribbon-glyph="sin"]').first().click();
check((await text(page)).includes('r = sin(x)'), 'Functions > Trigonometric > sin inserts sin(x)');
// text formatting wraps the selection
await tab(page, 'textfmt');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue("'hello world"); e.setSelection({ startLineNumber: 1, startColumn: 2, endLineNumber: 1, endColumn: 7 }); });
await btn(page, 'tf-b').click();
check((await text(page)) === "'<b>hello</b> world", `Text Formatting > Bold wraps the selection (${JSON.stringify(await text(page))})`);
// exports (Document tab)
await tab(page, 'document');
// Document > Excel really downloads a workbook: it opens in openpyxl, has the Report sheet and a chart part.
await page.evaluate(() => window.calcpadActiveEditor.setValue("'Ribbon Excel\nL = 4\nM = L^2\n$Plot{x^2 @ x = 0 : 2}"));
await page.waitForTimeout(1500);
const xl = page.waitForEvent('download', { timeout: 120000 }).catch(() => null);
await btn(page, 'd-excel').click();
const xd = await xl;
check(!!xd && /\.xlsx$/i.test(xd.suggestedFilename()), `Document > Excel triggers a download (${xd?.suggestedFilename()})`);
if (xd) {
    const xp = `${out}/ribbon-excel.xlsx`;
    await xd.saveAs(xp);
    const { execFileSync } = await import('node:child_process');
    let info;
    try {
        info = JSON.parse(execFileSync('python3', ['-I', '-c', 'import json,sys,zipfile,openpyxl\nwb=openpyxl.load_workbook(sys.argv[1])\nz=zipfile.ZipFile(sys.argv[1])\nprint(json.dumps({"sheets":wb.sheetnames,"charts":len(wb["Report"]._charts),"parts":[n for n in z.namelist() if n.startswith("xl/charts/")]}))', xp], { encoding: 'utf8' }));
    } catch (e) { info = { error: String(e).slice(0, 200) }; }
    check(info.sheets?.[0] === 'Report' && info.charts === 1 && info.parts?.length === 1, `the workbook opens in openpyxl with a Report sheet and a chart part (${JSON.stringify(info)})`);
}
let dl, d;
dl = page.waitForEvent('download', { timeout: 20000 }).catch(() => null);
await btn(page, 'd-word').click();
d = await dl;
check(!!d && /\.docx$/i.test(d.suggestedFilename()), `Document > Word triggers a download (${d?.suggestedFilename()})`);
// File button: the full menu
await btn(page, 'file').click();
const menu = await page.locator('.cprs-menu').innerText();
check(/Save as/.test(menu) && /Samples/.test(menu) && /Export Excel/.test(menu) && /recent/i.test(menu), 'the File button opens the full file menu');
await page.keyboard.press('Escape');
// calculation / resources
await tab(page, 'calc');
await btn(page, 'k-report').click();
await page.waitForTimeout(300);
check(await btn(page, 'k-report').evaluate(e => e.classList.contains('on')), 'Calculation > Report mode shows as active');
await btn(page, 'k-preview').click();
await tab(page, 'resources');
await btn(page, 'x-dark').click();
await page.waitForTimeout(300);
check(await page.evaluate(() => document.documentElement.dataset.theme) === 'dark', 'Resources > Dark sets the dark theme');
await btn(page, 'x-light').click();
await page.waitForTimeout(300);
check(await page.evaluate(() => document.documentElement.dataset.theme) === 'light', 'Resources > Light sets the light theme');
await btn(page, 'x-keys').click();
check(/shortcuts/i.test(await page.locator('[role=dialog]').innerText()), 'Resources > Shortcuts opens the shortcut list');
await page.keyboard.press('Escape');
check(errors.length === 0, `no page errors (${errors.slice(0, 2).join(' | ')})`);

// ---- the version: one source of truth, shown in the page and reported by the engine ---------
{
    let agree = '';
    try { agree = execFileSync('node', [`${here}../../tools/set_version.mjs`, '--check']).toString().trim(); } catch (e) { agree = `FAILED: ${e.stderr?.toString() ?? e.message}`; }
    check(/^all manifests agree: \d{2}\.\d+\.\d+$/.test(agree), `every manifest carries the same version (${agree.split('\n')[0]})`);
    const want = /: (\S+)$/.exec(agree)?.[1];
    const meta = await page.evaluate(() => document.querySelector('meta[name=version]')?.content);
    check(meta === want, `the page's <meta name="version"> is ${want} (${meta})`);
    const bar = await page.evaluate(() => document.querySelector('[data-testid=status-version]')?.textContent ?? '');
    check(bar.includes(`CivilKit Calcs ${want}`) && /\(built on CalcpadCE\)/.test(bar), `the status bar shows the version and the upstream (${bar})`);
    const engine = await page.evaluate(async () => { const m = await import(new URL('wasm/calcpad_wasm.js', location.href).href); await m.default({ module_or_path: new URL('wasm/calcpad_wasm_bg.wasm', location.href).href }); return m.version(); });
    check(engine === want, `the engine wasm's version() is ${want} (${engine})`);
    check((await page.title()).includes('CivilKit Calcs'), `the window title names CivilKit Calcs (${await page.title()})`);
    await tab(page, 'resources');
    await btn(page, 'x-about').click();
    await page.waitForSelector('[role=dialog]');
    const aboutText = await page.evaluate(() => document.querySelector('[role=dialog]').innerText + ' ' + [...document.querySelectorAll('[role=dialog] a')].map(a => a.href).join(' '));
    check(/CivilKit Calcs \d+\.\d+\.\d+/.test(aboutText) && /Built on CalcpadCE/.test(aboutText) && aboutText.includes('github.com/imartincei/CalcpadCE') && aboutText.includes('github.com/mageaustralia/civilkit-calcs'), 'Resources > About credits CalcpadCE and links both repositories');
    await page.screenshot({ path: `${out}/about-credits-v4.png` });
    await page.keyboard.press('Escape');
    check(!(await page.locator('[role=dialog]').count()), 'Escape closes the About dialog');
}

// ---- coverage: every library snippet is reachable from a ribbon popover ----------------------
{
    const api = await page.evaluate(async () => (await (await fetch('/api/calcpad/snippets')).json()).snippets.map(s => ({ key: `${s.category}|${s.insert}`, d: s.description, i: s.insert, c: s.category })));
    const moreButtons = RIBBON.flatMap(t => t.groups.flatMap(g => g.buttons.filter(b => b.more).map(b => ({ tab: t.id, id: b.id, more: b.more }))));
    const reachedBy = new Map();
    for (const mb of moreButtons) {
        await tab(page, mb.tab);
        await btn(page, mb.id).click();
        await page.waitForFunction(() => !/Loading/.test(document.querySelector('.rb-pop')?.innerText ?? 'Loading'), null, { timeout: 15000 });
        const keys = await page.evaluate(() => [...document.querySelectorAll('.rb-pop [data-snip-key]')].map(e => e.getAttribute('data-snip-key')));
        for (const k of keys) { if (!reachedBy.has(k)) reachedBy.set(k, new Set()); reachedBy.get(k).add(mb.more); }
        await page.keyboard.press('Escape');
    }
    if (arg('--dump-library', '')) writeFileSync(arg('--dump-library', ''), JSON.stringify(await page.evaluate(async () => (await fetch('/api/calcpad/snippets')).json())));
    const missing = api.filter(a => !reachedBy.has(a.key));
    check(missing.length === 0, `coverage: ${api.length - missing.length} of ${api.length} library snippets are reachable from a ribbon popover (${missing.slice(0, 5).map(m => m.key.slice(0, 40)).join(' | ')})`);
    const onlyAll = api.filter(a => reachedBy.get(a.key) && [...reachedBy.get(a.key)].every(m => m === 'all'));
    check(onlyAll.length === 0, `coverage: ${onlyAll.length} snippets are reachable only through the catch-all list (${[...new Set(onlyAll.map(a => a.c))].join(', ')})`);
    // The 958 entries of snippets.json (matched on insert text and description).
    const file = JSON.parse(readFileSync(`${here}../../calcpad-highlighter/src/snippets/snippets.json`, 'utf8'));
    const fileEntries = Object.values(file).flat();
    const apiIdx = new Map(api.map(a => [`${a.i}\u0000${a.d}`, a]));
    let hit = 0; const lost = [];
    for (const e of fileEntries) {
        const a = apiIdx.get(`${e.Insert}\u0000${e.Description}`);
        if (a && reachedBy.has(a.key)) hit++; else lost.push(e.Insert.slice(0, 30));
    }
    check(hit === fileEntries.length, `coverage: ${hit} of ${fileEntries.length} entries of snippets.json are reachable (${lost.slice(0, 5).join(' | ')})`);
    // Library inserts honour the first marker: the parameter is selected.
    await page.evaluate(() => { window.calcpadActiveEditor.setValue('r = '); window.calcpadActiveEditor.setPosition({ lineNumber: 1, column: 5 }); window.calcpadActiveEditor.focus(); });
    await tab(page, 'functions');
    await btn(page, 'fn-trig').click();
    await page.waitForSelector('.rb-pop [data-snip-key]');
    await clickKey(page, 'Functions/Trigonometric|sin(§)');
    const got = await page.evaluate(() => { const e = window.calcpadActiveEditor; return { t: e.getValue(), sel: e.getModel().getValueInRange(e.getSelection()) }; });
    check(got.t === 'r = sin(x)' && got.sel === 'x', `a library function inserts with its parameter selected (${JSON.stringify(got)})`);
    // Programming: Common first, then the library under the four headings asked for.
    await tab(page, 'math');
    await btn(page, 'p-programming').click();
    await page.waitForSelector('.rb-pop [data-snip-key]');
    {
        const heads = (await page.locator('.rb-pop-h').allInnerTexts()).map(h => h.toLowerCase());
        const idx = ['conditions', 'loops', 'macros and modules', 'other keywords'].map(h => heads.lastIndexOf(h));
        check(heads[0].startsWith('common') && idx.every(i => i > 0) && idx.every((v, i) => !i || v > idx[i - 1]), `Programming opens with Common, then Conditions, Loops, Macros and modules, Other keywords (${heads.join(' | ')})`);
    }
    await page.keyboard.press('Escape');
    // A line snippet from the keyword list goes on its own line.
    await page.evaluate(() => { window.calcpadActiveEditor.setValue('a = 1'); window.calcpadActiveEditor.setPosition({ lineNumber: 1, column: 6 }); });
    await tab(page, 'math');
    await btn(page, 'p-programming').click();
    await page.waitForSelector('.rb-pop [data-snip-key]');
    await clickKey(page, 'Iteration Blocks|#for i = 1 : n\n\texpression\n#loop');
    const kt = await text(page);
    check(/^a = 1\n#for i = 1 : n\n\s+expression\n#loop/.test(kt), `a keyword block goes on its own line (${JSON.stringify(kt)})`);
    // The filter box narrows a long list.
    await btn(page, 'p-units').click();
    await page.waitForSelector('.rb-pop [data-snip-key]');
    const before = await page.locator('.rb-pop .rb-glyph').count();
    await page.locator('.rb-filter').fill('kilonewton');
    const after = await page.locator('.rb-pop .rb-glyph').count();
    check(before > 300 && after > 0 && after < 10, `the units list filters (${before} -> ${after})`);
    await page.keyboard.press('Escape');
}

// ---- optional: every line snippet through the editor's linter -----------------------------
if (process.argv.includes('--lint')) {
    const snips = allSnippets().filter(s => s.line && !s.raw && !s.cx && !s.test && !/xlsx|#read|#include/.test(s.text));
    const seen = new Set();
    const bad = [];
    for (const s of snips) {
        if (seen.has(s.text)) continue;
        seen.add(s.text);
        await page.evaluate(t => window.calcpadActiveEditor.setValue(t), PRELUDE + s.text);
        await page.waitForTimeout(800);
        // A #python snippet asks before it runs; the linter does not need it to.
        if (await page.locator('#cprs-python-no').count()) await page.locator('#cprs-python-no').click();
        const m = await page.evaluate(() => window.monaco.editor.getModelMarkers({}).filter(k => k.severity >= 4).map(k => `${k.startLineNumber}: ${k.message}`));
        if (m.length) bad.push(`${s.id}: ${m[0]}`);
    }
    check(bad.length === 0, `${seen.size} line snippets lint clean (${bad.slice(0, 5).join(' | ')})`);
}

// ---- screenshots ---------------------------------------------------------------------------
await page.evaluate(async () => {
    const m = await (await fetch('samples/manifest.json')).json();
    const s = m.samples.find(x => /beam/i.test(x.title) && !x.ui && !x.python) ?? m.samples[0];
    await window.calcpadFiles.samples.openSample(s);
});
await tab(page, 'math');
const done1 = await calculated(page);
await btn(page, 'p-programming').click();
await page.waitForSelector('.rb-pop [data-snip-key]');
await calculated(page);
await page.screenshot({ path: `${out}/math-programming-light-v4.png` });
await page.keyboard.press('Escape');
console.log('results calculated before the light shot:', done1);
await tab(page, 'resources');
await btn(page, 'x-dark').click();
await tab(page, 'math');
await btn(page, 'p-constants').click();
await page.waitForSelector('.rb-pop [data-snip-key]');
console.log('results calculated before the dark shot:', await calculated(page));
await page.screenshot({ path: `${out}/math-constants-dark-v4.png` });
await page.keyboard.press('Escape');
await tab(page, 'functions');
await calculated(page);
await page.screenshot({ path: `${out}/functions-dark-v4.png` });
await tab(page, 'resources');
await btn(page, 'x-light').click();
await tab(page, 'matrices');
await calculated(page);
await page.screenshot({ path: `${out}/matrices-tables-light-v4.png` });
// A contact sheet: the ribbon of every tab, one under the other.
{
    const shots = [];
    for (const t of RIBBON) {
        await tab(page, t.id);
        await page.waitForTimeout(150);
        shots.push({ id: t.label, png: (await page.screenshot({ clip: { x: 0, y: 0, width: 1400, height: 155 } })).toString('base64') });
    }
    const sheet = await page.context().newPage();
    await sheet.setViewportSize({ width: 1420, height: 155 * shots.length + 30 * shots.length + 20 });
    await sheet.setContent(`<body style="margin:10px;font:13px sans-serif;background:#888">${shots.map(s => `<div style="margin-bottom:6px;color:#fff">${s.id}</div><img style="display:block;margin-bottom:12px" src="data:image/png;base64,${s.png}">`).join('')}</body>`);
    await sheet.screenshot({ path: `${out}/all-tabs-v4.png`, fullPage: true });
    await sheet.close();
}

// ---- phone ---------------------------------------------------------------------------------
({ page, errors } = await open(390, 780, 'light'));
await page.evaluate(() => window.calcpadActiveEditor.setValue('a = 3\nb = a*2\n'));
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 3, column: 1 }); });
await tab(page, 'math');
let sw = await page.evaluate(() => ({ panel: document.querySelector('.rb-panel').scrollWidth, client: document.querySelector('.rb-panel').clientWidth, doc: document.documentElement.scrollWidth }));
check(sw.panel > sw.client && sw.doc <= 392, `phone: the panel scrolls, the page does not (${JSON.stringify(sw)})`);
await btn(page, 'p-units').click();
await page.waitForSelector('.rb-pop [data-snip-key]');
const box = await page.locator('.rb-pop').boundingBox();
check(!!box && box.width >= 388 && Math.abs(box.y + box.height - 780) < 3, `phone: the popover is a bottom sheet (${JSON.stringify(box)})`);
await page.screenshot({ path: `${out}/phone-long-popover-v4.png` });
await clickKey(page, 'Units/Metric/Length|km');
check((await text(page)).includes('km'), 'phone: a library unit inserts');
sw = await page.evaluate(() => document.documentElement.scrollWidth);
check(sw <= 392, `phone: no page-level horizontal scroll with the popover used (${sw})`);
check(errors.length === 0, 'phone: no page errors');
await browser.close();
console.log(fails.length ? `${fails.length} FAILED` : 'ribbon check passed');
process.exit(fails.length ? 1 : 0);
