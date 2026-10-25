// End-to-end check of the Symbolic dialog in the browser build (headless Chromium, Playwright).
//
// Opens the dialog from the Functions tab and runs the operations on the examples a structural calculation uses:
// a derivative, an expansion, solving an equation for one variable, a beam integrated twice with limits, a linear
// system, a value substitution and an evaluation, then an error from the library. It checks the typeset and text
// results, that Insert writes the right text into a Calcpad worksheet and into a sheet (and that the engine then
// calculates it), that the symbolic module is not requested until the first Run, the light and dark themes, a phone
// width, and that the page has no errors.
//
//   node e2e/symbolic-check.mjs [--port 4181] [--dist DIR]
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (name, def) => {
    const i = process.argv.indexOf(name);
    return i > 0 ? process.argv[i + 1] : def;
};
const port = Number(arg('--port', '4181'));
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const origin = `http://127.0.0.1:${port}`;

const failures = [];
const check = (ok, what) => {
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
};

// ---- a plain static file server for dist ------------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf', '.cpd': 'text/plain' };
const server = createServer((req, res) => {
    let p = decodeURIComponent(new URL(req.url, origin).pathname);
    if (p.endsWith('/')) p += 'index.html';
    const f = normalize(join(dist, p));
    if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); return res.end(); }
    res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream' });
    res.end(readFileSync(f));
});
await new Promise(ok => server.listen(port, '127.0.0.1', ok));

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1400, height: 950 } });
const page = await ctx.newPage();
const errors = [];
page.on('pageerror', e => { if (e.message !== 'Canceled') errors.push(String(e)); });
const requests = [];
page.on('request', r => requests.push(r.url()));
await page.goto(`${origin}/`);
await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });

const D = '#cprs-sym-dialog';
const editorText = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
const newTab = (text) => page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
/** The cursor at the end of the document, where a user who is adding a result would have it. */
const cursorAtEnd = () => page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
/** The preview frame whose text holds `marker`. */
async function result(marker, tries = 80) {
    for (let i = 0; i < tries; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
            if (t.includes(marker)) return t.replace(/\s+/g, ' ');
        }
        await page.waitForTimeout(250);
    }
    return '';
}
const openDialog = async () => {
    await page.locator('[data-ribbon-tab=functions]').click();
    await page.locator('[data-ribbon-btn=fn-symbolic]').click();
    await page.locator(D).waitFor();
};
const closeDialog = async () => { if (await page.locator(D).count()) { await page.keyboard.press('Escape'); await page.locator(D).waitFor({ state: 'detached' }); } };
/** Sets the operation and the fields, runs with Enter in the expression box, and waits for an answer or an error. */
async function run(op, expr, fields = {}) {
    await page.locator(`${D} select[name=op]`).selectOption(op);
    for (const [k, v] of Object.entries(fields)) await page.locator(`${D} [name=${k}]`).fill(v);
    await page.locator(`${D} [name=expr]`).fill(expr);
    await page.locator(`${D} [data-cprs-sym-ascii]`).evaluate(e => { e.textContent = ''; e.closest('.sym-out').hidden = true; });
    await page.locator(`${D} [data-cprs-sym-error]`).evaluate(e => { e.hidden = true; e.textContent = ''; });
    await page.locator(`${D} [name=expr]`).press('Enter');
    await page.waitForFunction(() => {
        const d = document.getElementById('cprs-sym-dialog');
        return d && (!d.querySelector('.sym-out').hidden || !d.querySelector('[data-cprs-sym-error]').hidden) && d.querySelector('[data-cprs-sym-run]').textContent === 'Run';
    }, null, { timeout: 30000 });
    // The typeset form arrives after the text one.
    const ok = await page.locator(`${D} .sym-out`).isVisible();
    if (ok) await page.waitForFunction(() => document.querySelector('#cprs-sym-dialog [data-cprs-sym-math]')?.getAttribute('data-typeset') === '1', null, { timeout: 15000 }).catch(() => {});
    return {
        ok,
        ascii: ok ? (await page.locator(`${D} [data-cprs-sym-ascii]`).innerText()).trim() : '',
        typeset: ok ? await page.locator(`${D} [data-cprs-sym-math] .katex`).count() : 0,
        error: ok ? '' : (await page.locator(`${D} [data-cprs-sym-error]`).innerText()).trim(),
    };
}

// ---- 1. reachable, and the module is not requested before the first Run ----------------------------
await page.locator('[data-ribbon-tab=functions]').click();
check(await page.locator('[data-ribbon-btn=fn-symbolic]').count() === 1, 'the Functions tab has a Symbolic button');
await openDialog();
check(await page.locator(`${D}[role=dialog], ${D} [role=dialog]`).count() >= 1, 'the dialog has the dialog role');
check(await page.evaluate(() => document.activeElement?.getAttribute('name')) === 'expr', 'the expression box has the focus');
const ops = await page.locator(`${D} select[name=op] option`).allInnerTexts();
check(['Simplify', 'Expand', 'Factor', 'Differentiate', 'Integrate', 'Solve for', 'Substitute', 'Evaluate'].every((o, i) => ops[i] === o), `operations: ${ops.join(', ')}`);
check(!requests.some(u => /caskit/.test(u)), 'no request for the symbolic module while the dialog is only open');
check(await page.locator(`${D} [data-cprs-sym-insert]`).isDisabled(), 'Insert is disabled before there is a result');

// ---- 2. the operations ---------------------------------------------------------------------------------
let r = await run('differentiate', 'w*x^4/24', { var: 'x' });
check(requests.some(u => /calcpad_caskit_wasm_bg\.wasm/.test(u)), 'the module was requested by the first Run');
check(r.ok && r.ascii === 'w*x^3/6', `differentiate w*x^4/24: ${r.ascii}`);
check(r.typeset >= 1, 'the result is typeset with KaTeX');
const cssLoaded = await page.evaluate(() => [...document.styleSheets].some(s => { try { return [...s.cssRules].some(x => x.cssText.includes('.katex')); } catch { return false; } }));
check(cssLoaded, 'the KaTeX stylesheet is applied');

r = await run('expand', '(x-2)*(x-3)');
check(r.ok && r.ascii === 'x^2 - 5*x + 6', `expand (x-2)*(x-3): ${r.ascii}`);

r = await run('factor', 'x^2 - 5*x + 6');
check(r.ok && /\(x - 2\)\*\(x - 3\)|\(x - 3\)\*\(x - 2\)/.test(r.ascii), `factor x^2 - 5*x + 6: ${r.ascii}`);

r = await run('simplify', '-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)');
check(r.ok && r.ascii === '0', `simplify cancels to zero: ${r.ascii}`);

r = await run('solve', 'M = w*L^2/8', { var: 'L' });
check(r.ok && /L = sqrt\(8\*M\/w\); L = -sqrt\(8\*M\/w\)|L = .*sqrt/.test(r.ascii) && r.ascii.split(';').length === 2, `solve M = w*L^2/8 for L: ${r.ascii}`);

r = await run('solve', 'M = w*L^2/8', { var: 'w' });
check(r.ok && r.ascii === 'w = 8*M/L^2', `rearrange for w: ${r.ascii}`);

// The beam derivation: integrate the shear twice, then with limits.
r = await run('integrate', 'w*L*x/2 - w*x^2/2', { var: 'x' });
check(r.ok && r.ascii === '-w*x^3/6 + L*w*x^2/4', `integrate once: ${r.ascii}`);
r = await run('integrate', 'L*w*x^2/4 - w*x^3/6', { var: 'x' });
check(r.ok && r.ascii === '-w*x^4/24 + L*w*x^3/12', `integrate twice: ${r.ascii}`);
r = await run('integrate', 'w*L*x/2 - w*x^2/2', { var: 'x', lower: '0', upper: 'L' });
check(r.ok && r.ascii === 'L^3*w/12', `definite integral 0..L: ${r.ascii}`);
r = await run('integrate', 'L*w*x^2/4 - w*x^3/6', { var: 'x', lower: '0', upper: 'L/2' });
check(r.ok && r.ascii === 'L^4*w/128', `second integral 0..L/2: ${r.ascii}`);

r = await run('solve', 'x + y = 3; x - y = 1', { var: 'x, y' });
check(r.ok && r.ascii === 'x = 2; y = 1', `linear system: ${r.ascii}`);
r = await run('solve', '2*x + y = 5\nx - y = 1', { var: 'x, y' });
check(r.ok && r.ascii === 'x = 2; y = 1', `linear system on two lines: ${r.ascii}`);

r = await run('differentiate', 'sin(2*x) + x^3', { var: 'x', order: '2' });
check(r.ok && r.ascii === '6*x - 4*sin(2*x)', `second derivative: ${r.ascii}`);

r = await run('substitute', 'a*x^2', { values: 'x = 2*b' });
check(r.ok && r.ascii === '4*a*b^2', `substitute: ${r.ascii}`);
r = await run('evaluate', 'x^2 + 1', { values: 'x = 3' });
check(r.ok && r.ascii === '10', `evaluate: ${r.ascii}`);
r = await run('evaluate', 'sqrt(2)', { values: '' });
check(r.ok && r.ascii === '1.41421356237', `evaluate sqrt(2): ${r.ascii}`);

// ---- 3. errors in plain words ----------------------------------------------------------------------------
r = await run('integrate', 'sin(x)^2', { var: 'x' });
check(!r.ok && /^Cannot integrate: .+\.$/.test(r.error), `an unsupported integral shows the library's message: ${r.error}`);
check(await page.locator(`${D} [data-cprs-sym-error]`).getAttribute('role') === 'alert', 'the error is an alert');
check(await page.locator(`${D} [data-cprs-sym-insert]`).isDisabled(), 'Insert is disabled after an error');
r = await run('simplify', '2x');
check(!r.ok && /parse error/i.test(r.error), `a syntax error: ${r.error}`);
r = await run('solve', 'x^3 + x = 1', { var: 'x' });
check(!r.ok && /degree 3/.test(r.error), `an unsupported equation: ${r.error}`);
r = await run('integrate', 'x', { var: 'x', lower: '0', upper: '' });
check(!r.ok && /both limits/i.test(r.error), `one limit only: ${r.error}`);
r = await run('substitute', 'x', { values: 'x 2' });
check(!r.ok && /name = value/.test(r.error), `a malformed value: ${r.error}`);

// ---- 4. Insert into a Calcpad worksheet ------------------------------------------------------------------
await closeDialog();
await newTab('w = 2\nx = 3\nL = 5\n');
await cursorAtEnd();
await openDialog();
r = await run('differentiate', 'w*x^4/24', { var: 'x' });
await page.locator(`${D} [name=name]`).fill('slope');
await page.locator(`${D} [data-cprs-sym-insert]`).click();
check(await page.locator(D).count() === 0, 'Insert closes the dialog');
let text = await editorText();
check(text === 'w = 2\nx = 3\nL = 5\nslope = w*x^3/6\n', `Calcpad worksheet: a calculation line (${JSON.stringify(text)})`);
let body = await result('slope');
check(/slope\s*=\s*w\s*x3\s*6\s*=\s*2.*=\s*9\b|slope[^]*=\s*9\b/.test(body), `slope = w*x^3/6 evaluates to 9 (${body})`);
// A solved value gets its own name, and two roots are numbered.
await cursorAtEnd();
await openDialog();
r = await run('solve', 'M = w*L^2/8', { var: 'L' });
await page.locator(`${D} [data-cprs-sym-insert]`).click();
text = await editorText();
check(/^L_1 = 2\*sqrt\(M\)\*sqrt\(2\)\/sqrt\(w\)\nL_2 = -2\*sqrt\(M\)\*sqrt\(2\)\/sqrt\(w\)\n/m.test(text), `two roots are numbered L_1, L_2 (${JSON.stringify(text.split('\n').slice(-3))})`);
// The comment form and the name check.
await cursorAtEnd();
await openDialog();
await run('expand', '(x-2)*(x-3)');
await page.locator(`${D} select[name=form]`).selectOption('comment');
await page.locator(`${D} [data-cprs-sym-insert]`).click();
text = await editorText();
check(text.endsWith("'result = x^2 - 5*x + 6\n"), 'the comment form writes a text line');
await openDialog();
await run('expand', '(x-2)*(x-3)');
await page.locator(`${D} [name=name]`).fill('2bad name');
await page.locator(`${D} [data-cprs-sym-insert]`).click();
check(await page.locator(D).count() === 1 && /name made of letters/.test(await page.locator(`${D} [data-cprs-sym-error]`).innerText()), 'a bad name is refused in words');
await closeDialog();

// ---- 5. Insert into a sheet ------------------------------------------------------------------------------
await newTab('<!--sheet-->\n# Beam\n\nw: 2 kN/m\nL: 5 m\nx: 3 m\n');
await cursorAtEnd();
check(await page.evaluate(() => window.calcpadActiveEditor.getModel().getLanguageId()) === 'calcpad-sheet', 'the new tab is a sheet');
await openDialog();
const forms = await page.locator(`${D} select[name=form] option`).allInnerTexts();
check(/Formula block/.test(forms[0]) && /Quantity line/.test(forms[1]), `a sheet offers formula and quantity forms (${forms.join(' | ')})`);
r = await run('integrate', 'w*L*x/2 - w*x^2/2', { var: 'x', lower: '0', upper: 'L' });
await page.locator(`${D} [data-cprs-sym-insert]`).click();
text = await editorText();
check(text.endsWith('$$\n\\frac{L^{3} w}{12}\n$$\n'), `sheet: a display formula block (${JSON.stringify(text.slice(-40))})`);
const katexInPreview = await (async () => {
    for (let i = 0; i < 40; i++) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const n = await f.evaluate(() => document.querySelectorAll('.katex').length).catch(() => 0);
            if (n) return n;
        }
        await page.waitForTimeout(250);
    }
    return 0;
})();
check(katexInPreview >= 1, `the preview typesets the inserted formula (${katexInPreview})`);
await cursorAtEnd();
await openDialog();
r = await run('differentiate', 'w*x^4/24', { var: 'x' });
await page.locator(`${D} select[name=form]`).selectOption('line');
await page.locator(`${D} [name=name]`).fill('shear');
await page.locator(`${D} [data-cprs-sym-insert]`).click();
text = await editorText();
check(text.endsWith('shear: w*x^3/6\n'), `sheet: a quantity line (${JSON.stringify(text.slice(-24))})`);
await cursorAtEnd();
await openDialog();
r = await run('evaluate', 'w*L^2/8', { values: 'w = 2, L = 5' });
await page.locator(`${D} select[name=form]`).selectOption('line');
await page.locator(`${D} [name=name]`).fill('M');
await page.locator(`${D} [data-cprs-sym-insert]`).click();
text = await editorText();
check(text.endsWith('M: 6.25\n'), `sheet: an evaluated quantity line (${JSON.stringify(text.slice(-12))})`);
body = await result('M = 6.25', 60);
check(/shear = /.test(body) && /M = 6\.25/.test(body), `the sheet calculates the inserted quantity lines (${body.slice(-80)})`);

// ---- 6. keyboard, copy -----------------------------------------------------------------------------------
await openDialog();
await run('expand', '(a+b)^2');
await ctx.grantPermissions(['clipboard-read', 'clipboard-write'], { origin });
await page.locator(`${D} [data-cprs-sym-copy]`).click();
const clip = await page.evaluate(() => navigator.clipboard.readText()).catch(() => null);
check(clip === 'a^2 + b^2 + 2*a*b' || clip === null, `Copy puts the text result on the clipboard (${clip})`);
await page.locator(`${D} [name=expr]`).focus();
await page.keyboard.press('Shift+Tab');
const trapped = await page.evaluate(() => document.getElementById('cprs-sym-dialog').contains(document.activeElement));
check(trapped, 'Shift+Tab from the first field stays inside the dialog');
await page.keyboard.press('Escape');
check(await page.locator(D).count() === 0, 'Escape closes the dialog');

// ---- 7. light and dark, phone width ----------------------------------------------------------------------
const lum = (rgb) => { const [r_, g, b] = rgb.match(/\d+(\.\d+)?/g).slice(0, 3).map(Number).map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r_ + 0.7152 * g + 0.0722 * b; };
const contrast = (a, b) => { const [hi, lo] = [lum(a), lum(b)].sort((x, y) => y - x); return (hi + 0.05) / (lo + 0.05); };
const colours = () => page.evaluate(() => {
    const b = document.querySelector('#cprs-sym-dialog .sym-box');
    const cs = (e) => getComputedStyle(e);
    const err = document.querySelector('#cprs-sym-dialog [data-cprs-sym-error]');
    return { bg: cs(b).backgroundColor, fg: cs(b).color, inputBg: cs(b.querySelector('textarea')).backgroundColor, inputFg: cs(b.querySelector('textarea')).color, errFg: cs(err).color };
});
const shots = {};
for (const theme of ['Light', 'Dark']) {
    await page.evaluate((th) => window.calcpadBridge.handleMessage({ type: 'updateColorTheme', theme: th }), theme);
    await page.waitForTimeout(600);
    await openDialog();
    r = await run('solve', 'M = w*L^2/8', { var: 'L' });
    await page.screenshot({ path: join(process.env.TMPDIR || '.', `symbolic-${theme.toLowerCase()}.png`) });
    const c = await colours();
    shots[theme] = c;
    const bgLum = lum(c.bg);
    check(theme === 'Light' ? bgLum > 0.5 : bgLum < 0.2, `${theme}: dialog background is ${theme === 'Light' ? 'light' : 'dark'} (${c.bg})`);
    check(contrast(c.fg, c.bg) >= 4.5, `${theme}: text contrast ${contrast(c.fg, c.bg).toFixed(1)}`);
    check(contrast(c.inputFg, c.inputBg) >= 4.5, `${theme}: input contrast ${contrast(c.inputFg, c.inputBg).toFixed(1)}`);
    const mathColour = await page.locator(`${D} [data-cprs-sym-math] .katex`).first().evaluate(e => getComputedStyle(e).color);
    check(contrast(mathColour, c.bg) >= 4.5, `${theme}: typeset result contrast ${contrast(mathColour, c.bg).toFixed(1)}`);
    await closeDialog();
}
check(lum(shots.Light.bg) !== lum(shots.Dark.bg), 'the two themes differ');
await page.setViewportSize({ width: 390, height: 800 });
await page.waitForTimeout(300);
await openDialog();
r = await run('integrate', 'w*L*x/2 - w*x^2/2', { var: 'x', lower: '0', upper: 'L' });
const fit = await page.evaluate(() => { const b = document.querySelector('#cprs-sym-dialog .sym-box'); const r_ = b.getBoundingClientRect(); return { left: r_.left, right: r_.right, vw: innerWidth, hscroll: b.scrollWidth > b.clientWidth + 1, pageScroll: document.documentElement.scrollWidth > innerWidth }; });
check(fit.left >= 0 && fit.right <= fit.vw && !fit.hscroll, `phone width: the dialog fits (${Math.round(fit.left)}..${Math.round(fit.right)} of ${fit.vw})`);
const small = await page.evaluate(() => [...document.querySelectorAll('#cprs-sym-dialog button, #cprs-sym-dialog input, #cprs-sym-dialog select')].filter(e => !e.closest('[hidden]')).map(e => e.getBoundingClientRect()).every(b => b.width >= 40 && b.height >= 24));
check(small, 'phone width: controls are not tiny');
await closeDialog();

check(errors.length === 0, `no page errors${errors.length ? ': ' + errors.join(' | ') : ''}`);
await browser.close();
server.close();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nall passed');
process.exit(failures.length ? 1 : 0);
