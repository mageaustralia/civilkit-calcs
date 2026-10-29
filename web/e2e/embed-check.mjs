// The embed view and its loader (docs/embedding.md), end to end in headless Chromium.
//   cd web && CALCS_BASE=/ npm run build && node e2e/embed-check.mjs [--port 5101] [--host-port 5102] [--shots DIR]
// The build (calcpad-web/dist) is served with `python3 -m http.server` on 127.0.0.1:<port>; a second server on
// <host-port> is a different origin that plays the host page.
//   1. ?embed=1&sample=template-simply-supported-beam: no app chrome, the six inputs, results; a typed number and the
//      slider recalculate; a value out of the slider's bounds is refused beside the box;
//   2. the Open in CivilKit Calcs link; theme=dark; 320 px and 1000 px wide without a horizontal scroll; mode, hide, edit;
//   3. a sheet with Python: the note and the code, no question and no run;
//   4. in a host page: the height, ready and results messages; civilkit-set from the parent sets and recalculates,
//      bad values are refused with civilkit-error, a message from another frame is ignored;
//   5. embed.js on the host page replaces the placeholder, sandboxes the frame and fits its height;
//   6. nothing stored: no localStorage, IndexedDB or service worker after an embed, and the app opens normally after it;
//   7. no console errors on any embed load.
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { existsSync, mkdirSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(here, '..', 'calcpad-web', 'dist')));
const port = Number(arg('--port', '5101'));
const hostPort = Number(arg('--host-port', '5102'));
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const origin = `http://127.0.0.1:${port}`;
const hostOrigin = `http://127.0.0.1:${hostPort}`;
const SAMPLE = 'template-simply-supported-beam';
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };
if (!existsSync(join(dist, 'index.html'))) { console.log(`no build in ${dist}: cd web && CALCS_BASE=/ npm run build`); process.exit(1); }
check(statSync(join(dist, 'embed.js')).size < 3072, `embed.js is under 3 KB (${statSync(join(dist, 'embed.js')).size} bytes)`);

// ---- servers ----------------------------------------------------------------------------------------------------
const py = spawn('python3', ['-m', 'http.server', String(port), '--bind', '127.0.0.1'], { cwd: dist, stdio: 'ignore' });
const HOST_PAGES = {
    '/direct.html': `<!doctype html><meta charset="utf-8"><title>host</title>
<iframe id="f" src="${origin}/?embed=1&sample=${SAMPLE}" width="100%" height="560" style="border:1px solid #ccc" allow=""
  sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"></iframe>
<iframe id="other" srcdoc="<p>another frame</p>"></iframe>
<script>window.msgs=[];addEventListener('message',function(e){if(e.source===document.getElementById('f').contentWindow)msgs.push({origin:e.origin,data:e.data})});</script>`,
    '/loader.html': `<!doctype html><meta charset="utf-8"><title>loader host</title><p>A host page.</p>
<div id="calc" data-civilkit-sample="template-pipe-flow-darcy" data-origin="${origin}" data-path="/" data-height="400" data-theme="dark"></div>
<div id="prod" data-civilkit-sample="template-cantilever-tip-load"></div>
<script>window.events=[];document.addEventListener('civilkit',function(e){events.push(e.detail.type)});</script>
<script src="${origin}/embed.js"></script>`,
};
const host = createServer((req, res) => {
    const page = HOST_PAGES[new URL(req.url, 'http://x').pathname];
    res.writeHead(page ? 200 : 404, { 'Content-Type': 'text/html; charset=utf-8' });
    res.end(page ?? 'not found');
});
await new Promise((ok) => host.listen(hostPort, '127.0.0.1', ok));
for (let i = 0; i < 50; i++) { try { if ((await fetch(`${origin}/index.html`)).ok) break; } catch { /* starting */ } await new Promise((r) => setTimeout(r, 200)); }

const browser = await chromium.launch();
const consoleErrors = [];
async function newPage(width = 1000, ctx = null) {
    const context = ctx ?? await browser.newContext({ viewport: { width, height: 800 } });
    // The production build is served without the app's real host: the cross-origin production frame of the loader
    // test (https://civilkit.io) is answered with a stub, so the check never touches the network.
    await context.route('https://civilkit.io/**', (r) => r.fulfill({ status: 200, contentType: 'text/html', body: '<p>stub</p>' }));
    const page = await context.newPage();
    page.on('console', (m) => { if (m.type() === 'error') consoleErrors.push(`${page.url()}: ${m.text()}`); });
    page.on('pageerror', (e) => consoleErrors.push(`${page.url()}: ${e.message}`));
    return { context, page };
}
const embedUrl = (q = '') => `${origin}/?embed=1&sample=${SAMPLE}${q}`;
const frameText = (page) => page.frameLocator('.ce-frame').locator('body').innerText({ timeout: 30000 }).then((t) => t.replace(/\s+/g, ' '));
async function until(fn, ok, tries = 80) {
    let v;
    for (let i = 0; i < tries; i++) { v = await fn().catch(() => null); if (v != null && ok(v)) return v; await new Promise((r) => setTimeout(r, 250)); }
    return v;
}
const calculated = (page) => until(() => page.locator('.ce-status').textContent(), (t) => /^Calculated/.test(t));
/** M_max from the results text, as the engine writes it: "Mmax = ... = 56.26 kN · m". */
const mOf = (t) => /Mmax = .*?= ([\d.]+) kN · m/.exec(t)?.[1] ?? null;

// ---- 1. the embed on its own ------------------------------------------------------------------------------------
{
    const { context, page } = await newPage(1000);
    await page.goto(embedUrl());
    await calculated(page);
    const chrome = await page.evaluate(() => ({
        ribbon: !!document.querySelector('.ribbon, #cprs-ribbon, [class*="ribbon"]'),
        monaco: !!document.querySelector('.monaco-editor'),
        tabs: !!document.querySelector('.tab-strip, .tabs-bar, [role="tablist"]'),
        editor: typeof window.calcpadActiveEditor !== 'undefined',
        suitebar: !!document.querySelector('[class*="suitebar"], ck-suite-bar'),
    }));
    check(!chrome.ribbon && !chrome.monaco && !chrome.tabs && !chrome.editor && !chrome.suitebar, `no ribbon, editor, tabs or suite bar (${JSON.stringify(chrome)})`);
    const names = await page.locator('.ce-field').evaluateAll((els) => els.map((e) => e.dataset.name));
    check(JSON.stringify(names) === '["L","w","h","mat","limit","sw"]', `the six inputs are shown (${names})`);
    const kinds = await page.evaluate(() => ({ sliders: document.querySelectorAll('.ce-slider').length, selects: document.querySelectorAll('.ce-form select').length, radios: document.querySelectorAll('.ce-form input[type=radio]').length, checks: document.querySelectorAll('.ce-form input[type=checkbox]').length }));
    check(JSON.stringify(kinds) === '{"sliders":2,"selects":2,"radios":3,"checks":1}', `two sliders, two lists, three radio buttons, a checkbox (${JSON.stringify(kinds)})`);
    const slider = await page.locator('.ce-slider').first().evaluate((s) => [s.min, s.max, s.step, s.value]);
    check(JSON.stringify(slider) === '["2","12","0.5","6"]', `the span slider is 2 to 12 step 0.5 at 6 (${slider})`);
    const t0 = await until(() => frameText(page), (t) => mOf(t) != null);
    const m0 = mOf(t0);
    check(m0 === '56.26', `results: M_max = 56.26 kN·m by default (${m0})`);

    // A typed number recalculates.
    await page.locator('.ce-field[data-name="w"] .ce-box').fill('20');
    await page.locator('.ce-field[data-name="w"] .ce-box').press('Tab');
    const t1 = await until(() => frameText(page), (t) => mOf(t) != null && mOf(t) !== m0);
    check(mOf(t1) === '92.26', `typing w = 20 kN/m recalculates M_max to 92.26 kN·m (${mOf(t1)})`);

    // The slider by the keyboard: 6 m -> 7 m.
    await page.locator('.ce-field[data-name="L"] .ce-slider').focus();
    await page.keyboard.press('ArrowRight');
    await page.keyboard.press('ArrowRight');
    const box = await page.locator('.ce-field[data-name="L"] .ce-box').inputValue();
    const t2 = await until(() => frameText(page), (t) => mOf(t) != null && mOf(t) !== mOf(t1));
    check(box === '7' && mOf(t2) === '125.58', `two arrow presses move L to 7 m and M_max follows to 125.58 kN·m (box ${box}, M ${mOf(t2)})`);
    const focus = await page.evaluate(() => { const s = document.querySelector('.ce-slider'); return getComputedStyle(s).outlineStyle; });
    check(focus !== 'none', `the focused slider shows a focus outline (${focus})`);

    // Out of bounds: refused beside the box, not used.
    await page.locator('.ce-field[data-name="L"] .ce-box').fill('50');
    await page.locator('.ce-field[data-name="L"] .ce-box').press('Enter');
    await page.waitForTimeout(800);
    const err = await page.locator('.ce-field[data-name="L"] .ce-err').evaluate((e) => (e.hidden ? '' : e.textContent));
    const t3 = await frameText(page);
    check(err === 'must be from 2 to 12' && mOf(t3) === '125.58', `L = 50 shows "must be from 2 to 12" and is not used (${err}, M ${mOf(t3)})`);

    // A list recalculates too.
    await page.locator('.ce-field[data-name="h"] select').selectOption('400mm');
    const t4 = await until(() => frameText(page), (t) => /h = 400 ?mm/.test(t));
    check(/h = 400 ?mm/.test(t4), 'choosing the 400 mm section recalculates');

    const open = await page.locator('a.ce-open').evaluate((a) => [a.href, a.target, a.rel]);
    check(open[0] === `${origin}/?sample=${SAMPLE}` && open[1] === '_blank' && /noopener/.test(open[2]), `Open in CivilKit Calcs opens the same sample in a new tab, noopener (${open})`);
    const scroll = await page.evaluate(() => [document.documentElement.scrollWidth, window.innerWidth]);
    const side = await page.evaluate(() => { const f = document.querySelector('.ce-form').getBoundingClientRect(); const r = document.querySelector('.ce-results').getBoundingClientRect(); return f.right <= r.left && f.top < r.bottom; });
    check(scroll[0] <= scroll[1] && side, `1000 px: the form beside the results, no horizontal scroll (${scroll})`);
    if (shots) await page.screenshot({ path: join(shots, 'embed-1000.png'), fullPage: true });
    await context.close();
}

// ---- 2. narrow, dark, the options ---------------------------------------------------------------------------------
{
    const { context, page } = await newPage(320);
    await page.goto(embedUrl('&theme=dark'));
    await calculated(page);
    const info = await page.evaluate(() => {
        const f = document.querySelector('.ce-form').getBoundingClientRect(); const r = document.querySelector('.ce-results').getBoundingClientRect();
        return { theme: document.documentElement.dataset.theme, bg: getComputedStyle(document.body).backgroundColor, sw: document.documentElement.scrollWidth, iw: window.innerWidth, stacked: r.top >= f.bottom };
    });
    check(info.sw <= info.iw && info.stacked, `320 px: the form above the results, no horizontal scroll (${info.sw} <= ${info.iw})`);
    const lum = (info.bg.match(/\d+/g) ?? [255, 255, 255]).slice(0, 3).reduce((a, b) => a + Number(b), 0);
    const darkFrame = await page.frameLocator('.ce-frame').locator('body').evaluate((b) => b.classList.contains('dark-theme'));
    check(info.theme === 'dark' && lum < 200 && darkFrame, `theme=dark: a dark page and dark results (${info.theme}, ${info.bg}, frame ${darkFrame})`);
    if (shots) await page.screenshot({ path: join(shots, 'embed-320-dark.png'), fullPage: true });
    await context.close();
}
{
    const { context, page } = await newPage(1000);
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto(embedUrl('&mode=input&hide=title'));
    await until(() => page.locator('.ce-field').count(), (n) => n === 6);
    const r = await page.evaluate(() => ({ results: !!document.querySelector('.ce-results'), form: !!document.querySelector('.ce-form'), titleHidden: document.querySelector('.ce-title').classList.contains('ce-sr'), theme: document.documentElement.dataset.theme }));
    check(!r.results && r.form && r.titleHidden && r.theme === 'dark', `mode=input&hide=title: the form only, the title hidden, theme auto follows a dark system (${JSON.stringify(r)})`);
    await page.goto(embedUrl('&mode=results&theme=light'));
    await calculated(page);
    const r2 = await page.evaluate(() => ({ results: !!document.querySelector('.ce-results'), form: !!document.querySelector('.ce-form'), theme: document.documentElement.dataset.theme }));
    check(r2.results && !r2.form && r2.theme === 'light', `mode=results&theme=light: the results only, light (${JSON.stringify(r2)})`);
    await page.goto(`${origin}/?embed=edit&sample=${SAMPLE}`);
    await calculated(page);
    await page.locator('.ce-edit summary').click();
    const area = page.locator('textarea.ce-editor');
    const text = await area.inputValue();
    await area.fill(text.replace('L:     ? 6 m [2 .. 12 step 0.5] | span', 'L:     ? 6 m [2 .. 12 step 0.5] | span\nP_x:   ? 3 kN | an extra input'));
    const n = await until(() => page.locator('.ce-field').count(), (c) => c === 7);
    check(n === 7, `embed=edit: editing the sheet adds an input to the form (${n})`);
    // 3. Python.
    await page.goto(`${origin}/?embed=1&sample=3d-plate-deflection`);
    await calculated(page);
    const pyInfo = await page.evaluate(() => ({ note: !document.querySelector('.ce-note').hidden, code: document.querySelector('.ce-note pre')?.textContent.length ?? 0, asked: !!document.querySelector('#cprs-python-consent') }));
    check(pyInfo.note && pyInfo.code > 20 && !pyInfo.asked, `a sheet with Python: a note and its code, no question, nothing run (${JSON.stringify(pyInfo)})`);
    // A sheet without ? inputs gets its number lines as inputs.
    await page.goto(`${origin}/?embed=1&sample=sheet-gable-frame-thrust`);
    await calculated(page);
    const gable = await page.locator('.ce-field').evaluateAll((els) => els.map((e) => e.dataset.name));
    check(gable.includes('L') && gable.includes('pitch'), `a sheet without inputs offers its number lines (${gable.slice(0, 8)})`);
    await page.goto(`${origin}/?embed=1&sample=no-such-sample`);
    const missing = await until(() => page.locator('.ce-missing').textContent(), (t) => /no sample/.test(t));
    check(/no sample "no-such-sample"/.test(missing ?? ''), 'an unknown sample says so');
    await context.close();
}

// ---- 4. the host page and the postMessage API -------------------------------------------------------------------
{
    const { context, page } = await newPage(1000);
    await page.goto(`${hostOrigin}/direct.html`);
    const msgs = () => page.evaluate(() => window.msgs);
    const got = await until(msgs, (m) => m.some((x) => x.data.type === 'civilkit-results'));
    const types = new Set(got.map((x) => x.data.type));
    const height = got.find((x) => x.data.type === 'civilkit-embed-height');
    check(!!height && height.data.height > 200 && height.origin === origin, `the embed posts its height to the host (${height?.data.height})`);
    const ready = got.find((x) => x.data.type === 'civilkit-ready');
    check(!!ready && ready.data.inputs.length === 6 && ready.data.inputs[0].name === 'L' && ready.data.inputs[0].max === 12, 'civilkit-ready lists the inputs with their bounds');
    const res0 = got.find((x) => x.data.type === 'civilkit-results');
    check(types.has('civilkit-results') && res0.data.values.M_max?.value > 56 && res0.data.values.M_max.value < 57 && /kN/.test(res0.data.values.M_max.unit), `civilkit-results carries M_max with its unit (${JSON.stringify(res0?.data.values.M_max)})`);
    const embed = page.frameLocator('#f');
    const count = async () => (await msgs()).length;

    // Set from the parent.
    let n = await count();
    await page.evaluate(() => document.getElementById('f').contentWindow.postMessage({ type: 'civilkit-set', values: { L: 8 } }, '*'));
    let after = await until(msgs, (m) => m.slice(n).some((x) => x.data.type === 'civilkit-results'));
    let res = after.slice(n).find((x) => x.data.type === 'civilkit-results');
    const box = await embed.locator('.ce-field[data-name="L"] .ce-box').inputValue();
    check(res?.data.values.L?.value === 8 && Math.abs(res.data.values.M_max.value - 100.02) < 0.05 && box === '8', `civilkit-set {L: 8}: the box shows 8 and the results come back with M_max = 100.02 (${res?.data.values.M_max?.value}, box ${box})`);

    // Refused values.
    for (const bad of [{ L: 50 }, { nope: 1 }, { L: 'abc' }, { L: Number.POSITIVE_INFINITY }, { h: '999mm' }, { sw: 'maybe' }]) {
        n = await count();
        await page.evaluate((v) => document.getElementById('f').contentWindow.postMessage({ type: 'civilkit-set', values: v }, '*'), bad);
        after = await until(msgs, (m) => m.slice(n).some((x) => x.data.type === 'civilkit-error'), 20);
        const e = after.slice(n).find((x) => x.data.type === 'civilkit-error');
        check(!!e && !after.slice(n).some((x) => x.data.type === 'civilkit-results'), `civilkit-set ${JSON.stringify(bad)} is refused (${e?.data.errors})`);
    }
    // Choices by value and by label, a checkbox by boolean.
    n = await count();
    await page.evaluate(() => document.getElementById('f').contentWindow.postMessage({ type: 'civilkit-set', values: { h: '400 mm deep', sw: false, limit: 360 } }, '*'));
    after = await until(msgs, (m) => m.slice(n).some((x) => x.data.type === 'civilkit-results'));
    res = after.slice(n).find((x) => x.data.type === 'civilkit-results');
    check(res?.data.values.h?.value === 400 && res.data.values.sw?.value === 0 && res.data.values.limit?.value === 360, `a list by its label, a checkbox by false and a radio by value (${JSON.stringify([res?.data.values.h, res?.data.values.sw?.value, res?.data.values.limit?.value])})`);

    // A message from another frame of the host page is ignored.
    n = await count();
    await page.evaluate(() => {
        const other = document.getElementById('other').contentWindow;
        const s = other.document.createElement('script');
        s.textContent = "parent.document.getElementById('f').contentWindow.postMessage({type:'civilkit-set',values:{L:3}},'*')";
        other.document.body.appendChild(s);
    });
    await page.waitForTimeout(1500);
    const box2 = await embed.locator('.ce-field[data-name="L"] .ce-box').inputValue();
    check(box2 === '8' && !(await msgs()).slice(n).some((x) => x.data.type === 'civilkit-results' || x.data.type === 'civilkit-error'), `a civilkit-set from a frame that is not the parent is ignored (L stays ${box2})`);
    n = await count();
    await page.evaluate(() => document.getElementById('f').contentWindow.postMessage({ type: 'civilkit-get' }, '*'));
    after = await until(msgs, (m) => m.slice(n).some((x) => x.data.type === 'civilkit-results'), 20);
    check(after.slice(n).some((x) => x.data.type === 'civilkit-results'), 'civilkit-get sends the last results again');
    await context.close();
}

// ---- 5. embed.js --------------------------------------------------------------------------------------------------
{
    const { context, page } = await newPage(1000);
    await page.goto(`${hostOrigin}/loader.html`);
    const f = page.locator('iframe#calc');
    await f.waitFor({ timeout: 10000 });
    const attrs = await f.evaluate((el) => ({ src: el.src, sandbox: el.getAttribute('sandbox'), loading: el.loading, title: el.title, div: !!document.querySelector('div[data-civilkit-sample]') }));
    check(attrs.src === `${origin}/?embed=1&sample=template-pipe-flow-darcy&theme=dark` && /allow-scripts/.test(attrs.sandbox) && /allow-popups-to-escape-sandbox/.test(attrs.sandbox) && attrs.loading === 'lazy' && !attrs.div, `embed.js replaces the placeholders with sandboxed lazy frames (${attrs.src})`);
    const prod = await page.locator('iframe#prod').getAttribute('src');
    check(prod === 'https://civilkit.io/calcs/?embed=1&sample=template-cantilever-tip-load', `without data-origin the frame comes from https://civilkit.io/calcs/ (${prod})`);
    const h = await until(() => f.evaluate((el) => el.style.height), (v) => v && v !== '400px', 120);
    check(!!h && h !== '400px' && parseInt(h, 10) > 200, `embed.js fits the frame to the height the embed posts (${h})`);
    const events = await until(() => page.evaluate(() => window.events), (e) => e.includes('civilkit-results'));
    check(events.includes('civilkit-results'), `embed.js fires the embed's messages as civilkit events (${[...new Set(events)]})`);
    if (shots) await page.screenshot({ path: join(shots, 'embed-loader.png'), fullPage: true });
    await context.close();
}

// ---- 6. nothing stored, the app unaffected ----------------------------------------------------------------------------
{
    const { context, page } = await newPage(1200);
    await page.goto(embedUrl());
    await calculated(page);
    await page.locator('.ce-field[data-name="L"] .ce-box').fill('9');
    await page.locator('.ce-field[data-name="L"] .ce-box').press('Enter');
    await calculated(page);
    await page.waitForTimeout(1500);
    const stored = await page.evaluate(async () => ({
        local: Object.keys(localStorage), session: Object.keys(sessionStorage),
        idb: indexedDB.databases ? (await indexedDB.databases()).map((d) => d.name) : [],
        sw: (await navigator.serviceWorker.getRegistrations()).length,
    }));
    check(stored.local.length === 0 && stored.session.length === 0 && stored.idb.length === 0 && stored.sw === 0, `an embed stores nothing and registers no service worker (${JSON.stringify(stored)})`);
    await page.goto(`${origin}/?nosw`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    const app = await page.evaluate(() => ({ embed: !!document.querySelector('.ce'), monaco: !!document.querySelector('.monaco-editor'), text: window.calcpadActiveEditor.getValue().slice(0, 40) }));
    check(!app.embed && app.monaco && !/L:\s*\?\s*9/.test(app.text), `the full app opens normally afterwards (editor shown, no embed state: ${JSON.stringify(app.text)})`);
    await context.close();
}

check(consoleErrors.length === 0, `no console errors on the embed loads${consoleErrors.length ? ': ' + consoleErrors.slice(0, 5).join(' | ') : ''}`);
await browser.close();
host.close();
py.kill();
console.log(failures.length ? `\n${failures.length} FAILED` : '\nembed check passed');
process.exit(failures.length ? 1 : 0);
