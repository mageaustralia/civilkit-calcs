// End-to-end check of the editor navigation features, in both editors (Calcpad worksheet and sheet):
//   1. completion of names, functions, units and directives while typing, and nothing in text
//   2. rename everywhere (F2, and the Variables tab's context menu): scope, preview, collisions, undo
//   3. the variable palette (Ctrl+;) and the Variables tab's search
// Everything is done the way a person does it: keys typed into the editor, the menu clicked.
//
//   node e2e/editor-nav-check.mjs [--url http://127.0.0.1:4311/] [--port 4191] [--dist DIR] [--shots DIR]
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const port = Number(arg('--port', '4191'));
const dist = resolve(arg('--dist', join(here, '..', 'calcpad-web', 'dist')));
const shots = arg('--shots', process.env.TMPDIR ?? tmpdir());
mkdirSync(shots, { recursive: true });
let base = arg('--url', '');
const failures = [];
let passed = 0;
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (ok) passed++; else failures.push(what); };

let server = null;
if (!base) {
    const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf' };
    server = createServer((req, res) => {
        let p = decodeURIComponent(new URL(req.url, `http://127.0.0.1:${port}`).pathname);
        if (p.endsWith('/')) p += 'index.html';
        const f = normalize(join(dist, p));
        if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); return res.end(); }
        res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream' });
        res.end(readFileSync(f));
    });
    await new Promise(ok => server.listen(port, '127.0.0.1', ok));
    base = `http://127.0.0.1:${port}/`;
}

const browser = await chromium.launch();
const errors = [];

async function open(theme, viewport = { width: 1500, height: 900 }, touch = false) {
    const ctx = await browser.newContext({ viewport, hasTouch: touch, isMobile: touch, deviceScaleFactor: 1 });
    const page = await ctx.newPage();
    page.on('pageerror', e => errors.push(e.message));
    await page.goto(`${base}${base.includes('?') ? '&' : '?'}theme=${theme}`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    // No automatic preview: nothing but the editor is under test (and no Python question).
    await page.evaluate(() => window.calcpadBridge.setExtraSetting('autoRun', 'false'));
    return page;
}

const setDoc = async (page, text, line, col = 1) => {
    await page.evaluate(([v, l, c]) => {
        const e = window.calcpadActiveEditor;
        e.setValue(v);
        e.focus();
        const n = e.getModel().getLineCount();
        e.setPosition({ lineNumber: l < 0 ? n : l, column: c < 0 ? e.getModel().getLineMaxColumn(l < 0 ? n : l) : c });
    }, [text, line ?? -1, col]);
};
const docText = page => page.evaluate(() => window.calcpadActiveEditor.getValue());
const rows = page => page.evaluate(() => [...document.querySelectorAll('.suggest-widget.visible .monaco-list-row')].map(r => r.innerText.replace(/\s+/g, ' ').trim()));
const settle = async (page, ms = 900) => { await page.waitForTimeout(ms); };
const closeList = async page => { await page.keyboard.press('Escape'); await page.waitForTimeout(150); };
const hasRow = (list, re) => list.some(r => re.test(r));
const widgetVisible = page => page.evaluate(() => !!document.querySelector('.suggest-widget.visible'));
// Ctrl+Z once, as a person would: the whole rename is one step.
const undoAll = async page => { await page.keyboard.press('ControlOrMeta+z'); await page.waitForTimeout(150); };

const SRC = [
    "'Title a",                       // 1
    'length = 5m',                    // 2
    'load = 12kN',                    // 3
    'f(x) = x^2 + length',            // 4
    'σ_c = 3',                        // 5
    'α₂ = 4',                         // 6
    '#markdown',                      // 7
    'some prose about lo',            // 8
    '#end markdown',                  // 9
    '',                               // 10
].join('\n');

// =============================================================================================
// 1. Completion in a Calcpad worksheet
// =============================================================================================
console.log('-- completion, Calcpad worksheet');
let page = await open('light');
await setDoc(page, SRC + '\nzeta = 1\n', 10, 1);
await settle(page, 2500);
await page.keyboard.type('Tot = lo');
await settle(page);
let r = await rows(page);
check(hasRow(r, /^load .*= 12 kN/), `a variable defined above shows its value in the list (${JSON.stringify(r.slice(0, 2))})`);
check(hasRow(r, /^log \(x\) .*logarithm/), 'built-in functions show a signature and a one-line description');
check(!hasRow(r, /^zeta/), 'a variable defined below the cursor is not offered');
check(!hasRow(r, /^lo$/), 'the word being typed is not offered back (no word-based suggestions)');
await page.screenshot({ path: join(shots, 'nav-complete-calcpad-light.png') });
await page.keyboard.press('Tab');
check((await docText(page)).split('\n')[9] === 'Tot = load', `Tab accepts the suggestion (${JSON.stringify((await docText(page)).split('\n')[9])})`);

// Functions: signature in the list, snippet insert.
await page.keyboard.press('Enter');
await page.keyboard.type('atan');
await settle(page, 700);
r = await rows(page);
check(hasRow(r, /^atan2 \(x; y\)/), `a two-argument function shows both parameters (${JSON.stringify(r.slice(0, 3))})`);
await closeList(page);
await page.keyboard.press('ControlOrMeta+a');
await page.keyboard.press('Delete');

// A user function defined above, with its parameters.
await setDoc(page, SRC, 10, 1);
await settle(page, 2200);
await page.keyboard.type('y = f');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^f \(x\)/), `a user function lists its parameters (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);

// The parameters of the function being written.
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('g(speed) = spe');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^speed/), `the parameter of the function being written is offered (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);

// Units after a number, and only units.
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('W = 5 k');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^kA Kiloampere/) && hasRow(r, /^K Kelvin/), `units are offered after a number, with their names (${JSON.stringify(r.slice(0, 4))})`);
check(!hasRow(r, /^length/) && !hasRow(r, /^sin\b/), 'after a number nothing but units is offered');
await page.screenshot({ path: join(shots, 'nav-complete-units-light.png') });
await closeList(page);
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('W = 5kN*');
await settle(page, 400);
await page.keyboard.type('m');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^m\b/) && !hasRow(r, /^length/), `a unit chain (5kN*m) still offers units only (${JSON.stringify(r.slice(0, 3))})`);
await closeList(page);

// Directives.
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('#fo');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^#for\b/), `a # directive completes (${JSON.stringify(r.slice(0, 3))})`);
await page.keyboard.press('Tab');
const dir = (await docText(page)).split('\n')[0];
check(dir.startsWith('#for') && !dir.startsWith('##'), `accepting a directive does not double the # (${JSON.stringify(dir)})`);
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');

// Unicode and subscript names.
await setDoc(page, SRC, 10, 1);
await settle(page, 2200);
await page.keyboard.type('x = σ');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^σ_c .*= 3/), `a Greek name with a subscript completes (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);
await setDoc(page, SRC, 10, 1);
await settle(page, 2200);
await page.keyboard.type('x = α');
await settle(page, 800);
r = await rows(page);
check(hasRow(r, /^α₂ .*= 4/), `a name with a subscript digit completes (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);

// No noise in text.
const quiet = async (text, label) => {
    await setDoc(page, text, -1, -1);
    await page.keyboard.type(label.type);
    await settle(page, 900);
    check(!(await widgetVisible(page)), `nothing is suggested ${label.where}`);
    await closeList(page);
};
await quiet(SRC + "'the le", { type: 'ng', where: 'in a text line' });
await quiet(SRC + 'v = 3 "the le', { type: 'ng', where: 'after a quote mark in code' });
await quiet(SRC + '#markdown\nthe le', { type: 'ng', where: 'in a #markdown block' });
await quiet(SRC + '#html\n<p>the le', { type: 'ng', where: 'in an #html block' });
await quiet(SRC + '#python\nimport os\nle', { type: 'n', where: 'in a #python block' });
// A custom unit of the worksheet is offered after a number.
await setDoc(page, SRC + '.furlong = 201m\n', -1, -1);
await settle(page, 2200);
await page.keyboard.type('D = 3 fur');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^furlong/), `a custom unit is offered after a number (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);
// The right side of a ? input is not code.
await setDoc(page, SRC, 10, 1);
await settle(page, 2200);
await page.keyboard.type('Q = ? 5 lo');
await settle(page, 600);
check(!(await widgetVisible(page)), 'nothing is suggested after the ? of a Calcpad input');
await closeList(page);
await page.context().close();

// =============================================================================================
// 2. Completion in a sheet
// =============================================================================================
console.log('-- completion, sheet');
const SHEET = [
    '<!--sheet-->',                          // 1
    '# Beam check',                          // 2
    '',                                      // 3
    'The span L of the beam is checked.',    // 4
    'L: 6 m | span of the beam',             // 5
    'w: 12 kN/m',                            // 6
    'Mstar: w * L^2 / 8 -> kN*m',            // 7
    'f(a, b): a * b + w',                    // 8
    '',                                      // 9
].join('\n');
page = await open('light');
await setDoc(page, SHEET + '\nzz: 1\n', 9, 1);
await page.waitForFunction(() => window.calcpadActiveEditor.getModel().getLanguageId() === 'calcpad-sheet', null, { timeout: 20000 }).catch(() => { });
await settle(page, 2500);
await page.keyboard.type('Mu: w * L');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^L .*= 6 m/), `a sheet name is offered with its value (${JSON.stringify(r.slice(0, 3))})`);
check(!hasRow(r, /^zz/), 'a sheet name defined below the cursor is not offered');
await page.screenshot({ path: join(shots, 'nav-complete-sheet-light.png') });
await closeList(page);
await page.keyboard.type(' + sq');
await settle(page, 700);
r = await rows(page);
check(hasRow(r, /^sqr \(x\)/), `built-in functions complete in a sheet, with their signature (${JSON.stringify(r.slice(0, 3))})`);
await page.keyboard.press('Tab');
check(/sqr\(x\)$/.test((await docText(page)).split('\n')[8]), `a sheet function inserts with commas, not semicolons (${JSON.stringify((await docText(page)).split('\n')[8])})`);
await page.keyboard.press('Escape');
await setDoc(page, SHEET, 9, 1);
await settle(page, 1500);
await page.keyboard.type('P: 5 k');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^kA Kiloampere/) && !hasRow(r, /^w = /) && !hasRow(r, /^sqr/), `units are offered after a number in a sheet, and only units (${JSON.stringify(r.slice(0, 3))})`);
await closeList(page);

await setDoc(page, SHEET, 9, 1);
await settle(page, 1500);
await page.keyboard.type('f(p, q): p');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^p\b.*parameter|^p\b/) , `a sheet function's own parameters are offered (${JSON.stringify(r.slice(0, 2))})`);
await closeList(page);
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('<!--sheet-->\n\nwh');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^while\b/), `block keywords complete at the start of a line (${JSON.stringify(r.slice(0, 3))})`);
await closeList(page);
await page.keyboard.press('ControlOrMeta+a'); await page.keyboard.press('Delete');
await page.keyboard.type('<!--sheet-->\n\n>');
await settle(page, 900);
r = await rows(page);
check(hasRow(r, /^>>/), `the >> marker is offered (${JSON.stringify(r.slice(0, 3))})`);
await closeList(page);

const quietSheet = async (text, type, where) => {
    await setDoc(page, text, -1, -1);
    await page.keyboard.type(type);
    await settle(page, 900);
    check(!(await widgetVisible(page)), `nothing is suggested ${where}`);
    await closeList(page);
};
await quietSheet(SHEET + 'ok: ? [Steel = 200 GPa, *Timb', 'er', 'in the options of an input');
await quietSheet(SHEET + 'The load w of the beam is', ' ba', 'in sheet prose');
await quietSheet(SHEET + 'Note: see the dra', 'wing', 'in a line that only looks like a quantity');
await quietSheet(SHEET + 'k: w * 2 | the load w and the sp', 'an', 'in the note of a quantity');
await quietSheet(SHEET + 'if w > 1: "the lo', 'ad', 'in a string');
await quietSheet(SHEET + '```python\nimport math\nthe_lo', 'ad', 'in a Python block');
await quietSheet(SHEET + '$$\nw + L', '', 'in a display formula');
await quietSheet(SHEET + 'The load $w + L', '', 'in inline math');
await page.context().close();

// =============================================================================================
// 3. Rename everywhere, Calcpad worksheet
// =============================================================================================
console.log('-- rename, Calcpad worksheet');
const RN = [
    'a = 5',                          // 1
    'b = a + 2',                      // 2
    'f(x) = x^2 + a',                 // 3
    'c = f(b)',                       // 4
    "'a is a comment about a",        // 5
    'g(a) = a*2',                     // 6
    'd = g(3) + a',                   // 7
    '"text with a inside',            // 8
    '#markdown',                      // 9
    'a markdown a',                   // 10
    '#end markdown',                  // 11
    'L = ?',                          // 12
    'A = L*2',                        // 13
    '#python',                        // 14
    'export(M_max=3)',                // 15
    'M_max = 9 # a in python',        // 16
    '#end python',                    // 17
    'M = M_max*2',                    // 18
    '',
].join('\n');
page = await open('light');
await setDoc(page, RN, 2, 5);
await settle(page, 1500);
const box = () => page.locator('.ck-rename');
const boxText = async () => (await box().innerText().catch(() => '')).replace(/\s+/g, ' ');
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input', { timeout: 8000 });
let t = await boxText();
check(/Rename a/.test(t) && /4 places on 4 lines/.test(t), `F2 opens the rename box with the count (${t.slice(0, 90)})`);
check(/Everywhere in this document/.test(t) && /not in text, comments, strings or prose/.test(t), 'the box says what is and is not renamed');
check(await page.locator('.monaco-editor .ck-rename-hit').count() === 4, 'every place that will change is marked in the editor');
await page.screenshot({ path: join(shots, 'nav-rename-calcpad-light.png') });
await page.keyboard.type('alpha_1');
await page.waitForTimeout(200);
t = await boxText();
check(/4 places on 4 lines/.test(t) && /b = alpha_1 \+ 2/.test(t), `typing a name previews the changed lines (${t.slice(0, 150)})`);
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
let after = (await docText(page)).split('\n');
check(after[0] === 'alpha_1 = 5' && after[1] === 'b = alpha_1 + 2' && after[2] === 'f(x) = x^2 + alpha_1' && after[6] === 'd = g(3) + alpha_1', 'the variable is renamed in every use');
check(after[4] === "'a is a comment about a" && after[7] === '"text with a inside' && after[9] === 'a markdown a', 'comments, text lines and #markdown blocks are left alone');
check(after[5] === 'g(a) = a*2', "a function parameter of the same name is not touched by renaming the variable");
await undoAll(page);
check(await docText(page) === RN, 'one undo puts every place back');

// A parameter: renamed only inside its own function.
await setDoc(page, RN, 6, 3);
await settle(page, 600);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/2 places on 1 line/.test(t) && /Only in its own function/.test(t), `a parameter is scoped to its function (${t.slice(0, 80)})`);
await page.keyboard.type('z');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[5] === 'g(z) = z*2' && after[0] === 'a = 5' && after[6] === 'd = g(3) + a', 'renaming a parameter leaves the variable alone');
await undoAll(page);

// A function, with its calls.
await setDoc(page, RN, 3, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('flex');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[2] === 'flex(x) = x^2 + a' && after[3] === 'c = flex(b)', 'a function is renamed with its calls');
await undoAll(page);

// Collisions are refused.
await setDoc(page, RN, 1, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('b');
t = await boxText();
check(/already used/.test(t) && await page.locator('.ck-rename .ck-nav-primary').isDisabled(), `a name already in use is refused (${t.slice(0, 120)})`);
await page.keyboard.press('Enter');
check(await docText(page) === RN, 'Enter does not apply a refused rename');
await page.screenshot({ path: join(shots, 'nav-rename-refused-light.png') });
await page.keyboard.press('ControlOrMeta+a');
await page.keyboard.type('sin');
t = await boxText();
check(/keyword or built-in/.test(t), 'a built-in function name is refused');
await page.keyboard.press('ControlOrMeta+a');
await page.keyboard.type('2x');
t = await boxText();
check(/starts with a letter/.test(t), 'an invalid name is refused');
await page.keyboard.press('ControlOrMeta+a');
await page.keyboard.type('x');
t = await boxText();
check(/would capture it/.test(t), `a name a parameter on a changed line would capture is refused (${t.slice(0, 100)})`);
await page.keyboard.press('Escape');
check(await box().count() === 0 && await docText(page) === RN, 'Esc closes the box and changes nothing');

// Input names and Python exports.
await setDoc(page, RN, 12, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('Span');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[11] === 'Span = ?' && after[12] === 'A = Span*2', 'the name of a ? input is renamed with its uses');
await undoAll(page);
await setDoc(page, RN, 18, 6);
await settle(page, 600);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/1 Python export name/.test(t), `the count names the Python export (${t.slice(0, 100)})`);
await page.keyboard.type('M_peak');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[14] === 'export(M_peak=3)' && after[17] === 'M = M_peak*2', 'a name a Python block exports is renamed with the worksheet');
check(after[15] === 'M_max = 9 # a in python', 'other Python code is left alone');
await undoAll(page);

// A name that is also a unit is allowed, with a note.
await setDoc(page, RN, 1, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('kN');
t = await boxText();
check(/also a unit name/.test(t) && !(await page.locator('.ck-rename .ck-nav-primary').isDisabled()), `a name that is also a unit is allowed with a note (${t.slice(0, 120)})`);
await page.keyboard.press('Escape');

// Macros, and the name bound inside $Sum{...}.
const MAC = [
    '#def mm$(p$; q$)',               // 1
    '    p$ + q$ + 1',                // 2
    '#end def',                       // 3
    'p = 1',                          // 4
    'y = $Sum{ j*p @ j = 1 : 4 }',    // 5
    'z = mm$(1; 2)',                  // 6
    '',
].join('\n');
await setDoc(page, MAC, 2, 6);
await settle(page, 1200);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/macro parameter/.test(t) && /2 places on 2 lines/.test(t) && /Only in its own macro \(line 1 to 3\)/.test(t), `a macro parameter is scoped to its #def block (${t.slice(0, 120)})`);
await page.keyboard.type('r$');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[0] === '#def mm$(r$; q$)' && after[1] === '    r$ + q$ + 1' && after[3] === 'p = 1', 'renaming it changes the block and nothing outside');
await undoAll(page);
await setDoc(page, MAC, 1, 6);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('kk$');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[0] === '#def kk$(p$; q$)' && after[5] === 'z = kk$(1; 2)', 'a macro is renamed with its calls');
await undoAll(page);
await setDoc(page, MAC, 5, 12);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/2 places on 1 line/.test(t) && /Only in its own function/.test(t), `the name bound in $Sum{...} is scoped to its line (${t.slice(0, 80)})`);
await page.keyboard.press('Escape');
await setDoc(page, MAC, 4, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/Rename p variable/.test(t) && /2 places on 2 lines/.test(t), `renaming p does not touch the macro parameter p$ (${t.slice(0, 80)})`);
await page.keyboard.press('Escape');

// Not on a name.
await setDoc(page, RN, 5, 3);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-err');
check(/Put the cursor on the name/.test(await boxText()), 'F2 in a comment says there is no name to rename');
await page.keyboard.press('Escape');
await page.context().close();

// =============================================================================================
// 4. Rename everywhere, sheet
// =============================================================================================
console.log('-- rename, sheet');
const SRN = [
    '<!--sheet-->',                                  // 1
    '# Beam with L',                                 // 2
    '',                                              // 3
    'L: 6 m | span of L',                            // 4
    'w: 12 kN/m',                                    // 5
    'M: w * L^2 / 8 -> kN*m',                        // 6
    'f(L): L^2 + w',                                 // 7
    'if M > 3 kN*m: "L is large"',                   // 8
    'm: 2',                                          // 9
    'x: 3 m + m * m',                                // 10
    'for i in 1..3:',                                // 11
    '    P: 10 kN * i',                              // 12
    'Prose about L and $L$ in a formula.',           // 13
    '```python',                                     // 14
    'export(T_max=3)',                               // 15
    'T_max = 7',                                     // 16
    '```',                                           // 17
    'T2: T_max * 2',                                 // 18
    '>> y = L + 1',                                  // 19
    '',
].join('\n');
page = await open('light');
await setDoc(page, SRN, 4, 1);
await page.waitForFunction(() => window.calcpadActiveEditor.getModel().getLanguageId() === 'calcpad-sheet', null, { timeout: 20000 }).catch(() => { });
await settle(page, 1500);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input', { timeout: 8000 });
t = await boxText();
check(/Rename L/.test(t) && /3 places on 3 lines/.test(t), `a sheet name is counted (${t.slice(0, 80)})`);
await page.screenshot({ path: join(shots, 'nav-rename-sheet-light.png') });
await page.keyboard.type('span');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[3] === 'span: 6 m | span of L' && after[5] === 'M: w * span^2 / 8 -> kN*m' && after[18] === '>> y = span + 1', 'a sheet name is renamed in quantities and in raw lines');
check(after[1] === '# Beam with L' && after[7] === 'if M > 3 kN*m: "L is large"' && after[12] === 'Prose about L and $L$ in a formula.', 'headings, strings, notes, prose and formulas are left alone');
check(after[6] === 'f(L): L^2 + w', "a function's parameter is not renamed with the variable");
await undoAll(page);
check(await docText(page) === SRN, 'one undo restores the sheet');

await setDoc(page, SRN, 9, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('mm');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[8] === 'mm: 2' && after[9] === 'x: 3 m + mm * mm', `a name that is also a unit is renamed, the unit after a number is not (${JSON.stringify(after[9])})`);
await undoAll(page);

await setDoc(page, SRN, 7, 3);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/2 places on 1 line/.test(t) && /Only in its own function/.test(t), `a sheet parameter is scoped to its statement (${t.slice(0, 80)})`);
await page.keyboard.press('Escape');

await setDoc(page, SRN, 18, 8);
await settle(page, 600);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/Python export name/.test(t), `a Python export is part of the count in a sheet (${t.slice(0, 90)})`);
await page.keyboard.type('T_top');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
after = (await docText(page)).split('\n');
check(after[14] === 'export(T_top=3)' && after[17] === 'T2: T_top * 2' && after[15] === 'T_max = 7', 'the exported name and its uses change, the Python does not');
await undoAll(page);

const SPL = ['<!--sheet-->', 'L: 6 m', 'w: 2 kN/m', 'plot w * x for x in 0..L', ''].join('\n');
await setDoc(page, SPL, 4, 11);
await settle(page, 600);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/local name/.test(t) && /2 places on 1 line/.test(t), `the variable of a plot line is scoped to that line (${t.slice(0, 80)})`);
await page.keyboard.press('Escape');
await setDoc(page, SPL, 2, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
t = await boxText();
check(/Rename L variable/.test(t) && /2 places on 2 lines/.test(t), `a name used as a plot range is counted (${t.slice(0, 80)})`);
await page.keyboard.press('Escape');

await setDoc(page, SRN, 4, 1);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('w');
t = await boxText();
check(/already used/.test(t), 'a sheet rename onto an existing name is refused');
await page.keyboard.press('ControlOrMeta+a');
await page.keyboard.type('phi_2');
await page.keyboard.press('Escape');
await page.context().close();

// =============================================================================================
// 5. The variable palette, and the Variables tab
// =============================================================================================
console.log('-- palette and Variables tab');
const PAL = [
    "'Loads",                         // 1
    'load = 12kN',                    // 2
    'length = 5m',                    // 3
    'f(x) = x^2 + length',            // 4
    'sigma_c = 30MPa',                // 5
    'area = length*length',           // 6
    '',
].join('\n');
page = await open('light');
await setDoc(page, PAL, 7, 1);
await settle(page, 1800);
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette .ck-pal-row', { timeout: 8000 });
await page.waitForFunction(() => /12 kN/.test(document.querySelector('.ck-palette')?.innerText ?? ''), null, { timeout: 8000 }).catch(() => { });
let pal = await page.evaluate(() => [...document.querySelectorAll('.ck-pal-row')].map(r => r.innerText.replace(/\s+/g, ' ').trim()));
check(pal.length === 5 && /^load .*12 kN/.test(pal[0]), `Ctrl+; opens the palette listing the document's names with their values (${JSON.stringify(pal.slice(0, 2))})`);
check(pal.some(p => /^f\(x\)/.test(p)) && pal.some(p => /^area .*25 m/.test(p)), `functions and computed values are listed (${JSON.stringify(pal.slice(3))})`);
check(await page.evaluate(() => document.activeElement === document.querySelector('.ck-pal-input')), 'the search box has the focus');
await page.screenshot({ path: join(shots, 'nav-palette-light.png') });
await page.keyboard.type('sc');
await page.waitForTimeout(250);
pal = await page.evaluate(() => [...document.querySelectorAll('.ck-pal-row')].map(r => r.innerText.replace(/\s+/g, ' ').trim()));
check(/^sigma_c .*30 MPa/.test(pal[0]), `the search is fuzzy: letters in order find sigma_c (${JSON.stringify(pal.slice(0, 2))})`);
await page.keyboard.press('Enter');
await page.waitForTimeout(250);
check(!(await page.locator('.ck-palette').count()) && (await docText(page)) === PAL + 'sigma_c', 'Enter inserts the name at the caret and closes the palette');
check(await page.evaluate(() => document.activeElement?.closest('.monaco-editor') !== null), 'the focus returns to the editor');
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette .ck-pal-row');
await page.keyboard.type('load');
await page.waitForTimeout(250);
await page.keyboard.press('Alt+Enter');
await page.waitForTimeout(250);
check(await page.evaluate(() => window.calcpadActiveEditor.getPosition().lineNumber) === 2, 'Alt+Enter jumps to the definition');
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette .ck-pal-row');
await page.keyboard.type('sin');
await page.waitForTimeout(250);
pal = await page.evaluate(() => [...document.querySelectorAll('.ck-pal-row')].map(r => r.innerText.replace(/\s+/g, ' ').trim()));
check(pal.some(p => /^sin\(x\).*built in/.test(p)), 'built-in functions are searchable too');
await page.keyboard.press('ArrowDown');
await page.keyboard.press('Enter');
await page.waitForTimeout(200);
check(/sin\(\)$|sinh\(\)$|sin[a-z0-9_]*\(\)$/.test((await docText(page)).split('\n').slice(-1)[0]) || /\(\)/.test(await docText(page)), 'a function is inserted with its parentheses');
await undoAll(page);
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette');
await page.keyboard.press('Escape');
check(await page.locator('.ck-palette').count() === 0, 'Esc closes the palette');
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette');
await page.keyboard.press('ControlOrMeta+;');
check(await page.locator('.ck-palette').count() === 0, 'the same key closes it again');

// The Variables tab: search, and the context menu.
await setDoc(page, PAL, 7, 1);
await page.getByText('Variables', { exact: true }).first().click();
await page.waitForSelector('.variables-tab .variable-item', { timeout: 10000 });
const names = () => page.evaluate(() => [...document.querySelectorAll('.variables-tab .variable-item .variable-name')].map(e => e.textContent.trim()));
check((await names()).length >= 5, `the Variables tab lists the names (${JSON.stringify(await names())})`);
await page.locator('.variables-tab .search-input').fill('MPa');
await page.waitForTimeout(250);
check(JSON.stringify(await names()) === '["sigma_c"]', `the Variables tab search also matches the definition (${JSON.stringify(await names())})`);
await page.locator('.variables-tab .search-input').fill('len');
await page.waitForTimeout(250);
check((await names()).includes('length'), 'and the name');
await page.locator('.variables-tab .search-input').fill('');
await page.locator('.variables-tab .variable-item', { hasText: 'length' }).first().click({ button: 'right' });
await page.waitForSelector('.calcpad-context-menu');
await page.screenshot({ path: join(shots, 'nav-variables-menu-light.png') });
await page.locator('.calcpad-context-item', { hasText: 'Rename Everywhere' }).click();
await page.waitForSelector('.ck-rename .ck-nav-input', { timeout: 8000 });
t = await boxText();
check(/Rename length/.test(t) && /4 places on 3 lines/.test(t), `the Variables tab's Rename Everywhere opens the rename box (${t.slice(0, 80)})`);
await page.keyboard.type('span');
await page.keyboard.press('Enter');
await page.waitForTimeout(300);
check((await docText(page)).includes('span = 5m') && (await docText(page)).includes('area = span*span'), 'and renames it');
await undoAll(page);
await page.locator('.variables-tab .variable-item', { hasText: 'sigma_c' }).first().click({ button: 'right' });
await page.locator('.calcpad-context-item', { hasText: 'Go to Definition' }).click();
await page.waitForTimeout(400);
check(await page.evaluate(() => window.calcpadActiveEditor.getPosition().lineNumber) === 5, 'Go to Definition in the Variables tab moves the caret');
await page.context().close();

// Palette in a sheet.
page = await open('light');
await setDoc(page, SHEET, 9, 1);
await page.waitForFunction(() => window.calcpadActiveEditor.getModel().getLanguageId() === 'calcpad-sheet', null, { timeout: 20000 }).catch(() => { });
await settle(page, 1500);
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette .ck-pal-row');
await page.waitForFunction(() => /6 m/.test(document.querySelector('.ck-palette')?.innerText ?? ''), null, { timeout: 8000 }).catch(() => { });
pal = await page.evaluate(() => [...document.querySelectorAll('.ck-pal-row')].map(r => r.innerText.replace(/\s+/g, ' ').trim()));
check(pal.some(p => /^L .*6 m/.test(p)) && pal.some(p => /^w .*12 kN\/m/.test(p)) && pal.some(p => /^f\(a, b\)/.test(p)), `the palette works in a sheet (${JSON.stringify(pal.slice(0, 4))})`);
await page.keyboard.press('Escape');
await page.context().close();

// =============================================================================================
// 6. Dark theme, and a phone
// =============================================================================================
console.log('-- dark theme and phone width');
page = await open('dark');
await setDoc(page, SRC, 10, 1);
await settle(page, 2200);
await page.keyboard.type('Tot = lo');
await settle(page, 1000);
await page.screenshot({ path: join(shots, 'nav-complete-calcpad-dark.png') });
await closeList(page);
await setDoc(page, RN, 2, 5);
await settle(page, 1200);
await page.keyboard.press('F2');
await page.waitForSelector('.ck-rename .ck-nav-input');
await page.keyboard.type('alpha_1');
await page.screenshot({ path: join(shots, 'nav-rename-calcpad-dark.png') });
await page.keyboard.press('Escape');
await setDoc(page, PAL, 7, 1);
await settle(page, 1800);
await page.keyboard.press('ControlOrMeta+;');
await page.waitForSelector('.ck-palette .ck-pal-row');
await page.waitForFunction(() => /12 kN/.test(document.querySelector('.ck-palette')?.innerText ?? ''), null, { timeout: 8000 }).catch(() => { });
await page.screenshot({ path: join(shots, 'nav-palette-dark.png') });
await page.keyboard.press('Escape');
await page.context().close();

const phone = { width: 390, height: 844 };
page = await open('light', phone, true);
await setDoc(page, RN, 2, 5);
await settle(page, 1500);
await page.evaluate(() => window.calcpadActiveEditor.getAction('calcpad.renameEverywhere').run());
await page.waitForSelector('.ck-rename .ck-nav-input', { timeout: 8000 });
let bb = await page.locator('.ck-rename').boundingBox();
check(bb && bb.x >= 0 && bb.x + bb.width <= phone.width && bb.y >= 0 && bb.y < 200, `phone: the rename box is on screen at the top (${JSON.stringify(bb && { x: Math.round(bb.x), y: Math.round(bb.y), w: Math.round(bb.width) })})`);
await page.screenshot({ path: join(shots, 'nav-rename-phone.png') });
await page.locator('.ck-rename .ck-nav-input').fill('alpha_1');
await page.locator('.ck-rename .ck-nav-primary').tap();
await page.waitForTimeout(300);
check((await docText(page)).startsWith('alpha_1 = 5'), 'phone: the Rename button applies it (no keyboard needed)');
await undoAll(page);
await setDoc(page, PAL, 7, 1);
await settle(page, 1800);
await page.evaluate(() => window.calcpadActiveEditor.getAction('calcpad.quickRecall').run());
await page.waitForSelector('.ck-palette .ck-pal-row');
bb = await page.locator('.ck-palette').boundingBox();
check(bb && bb.x >= 0 && bb.x + bb.width <= phone.width && bb.y < 100, `phone: the palette is on screen at the top (${JSON.stringify(bb && { x: Math.round(bb.x), y: Math.round(bb.y), w: Math.round(bb.width), h: Math.round(bb.height) })})`);
await page.screenshot({ path: join(shots, 'nav-palette-phone.png') });
await page.locator('.ck-pal-row', { hasText: 'length' }).first().tap();
await page.waitForTimeout(300);
check(/length$/.test(await docText(page)), 'phone: a tap on a row inserts the name');
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); });
await page.keyboard.type(' + lo');
await settle(page, 1200);
const sw = await page.evaluate(() => { const w = document.querySelector('.suggest-widget.visible'); if (!w) return null; const r = w.getBoundingClientRect(); return { x: r.x, r: r.right, w: r.width }; });
check(sw !== null && sw.x >= 0 && sw.r <= phone.width + 1, `phone: the completion list stays inside the screen (${JSON.stringify(sw)})`);
await page.screenshot({ path: join(shots, 'nav-complete-phone.png') });
check(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), 'phone: the page does not scroll sideways');
await page.context().close();

check(errors.length === 0, `no page errors (${errors.slice(0, 3).join(' | ')})`);
await browser.close();
server?.close();
console.log(`${passed} ok, ${failures.length} FAIL`);
if (failures.length) { console.log(failures.map(f => '  - ' + f).join('\n')); process.exit(1); }
