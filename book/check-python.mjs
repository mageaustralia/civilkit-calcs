// Runs the examples of the book that need the page, in the built app (headless Chromium), the way a reader would:
// every ```sheet python block is put in the editor, the Python prompt is answered, and the calculated
// report is read back. A block fails when the report holds an error line or a Python error.
// Every ```sheet block with a `#UI` input line is opened in Input mode, and its controls are counted.
//   node check-python.mjs [--dist DIR] [--port 4190] [--show] [file.md ...]
// Needs a built app (npm run build in web/, which also needs tools/build_wasm.sh) and Playwright
// with a Chromium (PLAYWRIGHT_BROWSERS_PATH if it is not in the default place).
import { createServer } from 'node:http';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const argv = process.argv.slice(2);
const opt = (n, d) => { const i = argv.indexOf(n); return i >= 0 ? argv[i + 1] : d; };
const show = argv.includes('--show');
const dist = resolve(opt('--dist', join(here, '..', 'web', 'calcpad-web', 'dist')));
const port = Number(opt('--port', '4190'));
const pw = process.env.PLAYWRIGHT_MODULE ?? join(here, '..', 'web', 'node_modules', 'playwright', 'index.mjs');
const files = argv.filter((a, i) => !a.startsWith('--') && !['--dist', '--port'].includes(argv[i - 1])).map((f) => resolve(f));
if (!existsSync(join(dist, 'index.html'))) { console.error(`No built app in ${dist}. Build it (see the repository README) or pass --dist.`); process.exit(2); }
const { chromium } = await import(pathToFileURL(pw).href);

const walk = (d) => readdirSync(d).flatMap((n) => { const p = join(d, n); return statSync(p).isDirectory() ? walk(p) : p.endsWith('.md') ? [p] : []; });
const mds = files.length ? files.flatMap((f) => (statSync(f).isDirectory() ? walk(f) : [f])) : walk(join(here, 'src')).concat(walk(join(here, 'videos')));
const blocks = [];
for (const md of mds) {
    const src = readFileSync(md, 'utf8').split('\n');
    for (let i = 0; i < src.length; i++) {
        const m = /^(`{3,})(sheet python|sheet)\s*$/.exec(src[i]);
        if (!m) continue;
        let j = i + 1;
        while (j < src.length && !src[j].startsWith(m[1])) j++;
        const text = src.slice(i + 1, j).join('\n') + '\n';
        if (m[2] === 'sheet python' || />>\s*#UI/.test(text)) blocks.push({ where: `${md.replace(here + '/', '')}:${i + 2}`, text, form: m[2] === 'sheet' });
        i = j;
    }
}
if (!blocks.length) { console.log('no Python examples'); process.exit(0); }

const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.py': 'text/plain' };
// The app may be built for a base path (CALCS_BASE); serve it there.
const base = (/(?:src|href)="(\/[^"]*?\/)assets\//.exec(readFileSync(join(dist, 'index.html'), 'utf8')) ?? [, '/'])[1];
const server = createServer((req, res) => {
    let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (base !== '/' && p.startsWith(base)) p = '/' + p.slice(base.length);
    if (p.endsWith('/')) p += 'index.html';
    const f = normalize(join(dist, p));
    if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); res.end(); return; }
    res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream' });
    res.end(readFileSync(f));
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));

const browser = await chromium.launch();
const page = await (await browser.newContext({ viewport: { width: 1300, height: 900 } })).newPage();
await page.goto(`http://127.0.0.1:${port}${base}`);
await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });

let bad = 0;
for (const b of blocks) {
    const text = /^\s*<!--sheet-->/.test(b.text) ? b.text : '<!--sheet-->\n' + b.text;
    await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), text);
    const yes = page.locator('#cprs-python-yes');
    await yes.waitFor({ timeout: 15000 }).then(() => yes.click()).catch(() => {});
    let html = '';
    for (let k = 0; k < 90; k++) {
        html = await page.evaluate(async (content) => {
            const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, includeLineAnchors: false }) });
            return new TextDecoder().decode(await r.arrayBuffer());
        }, text);
        if (!/Python was not run/.test(html)) break;
        await page.waitForTimeout(1000);
    }
    const plain = html.replace(/<(script|style)[\s\S]*?<\/\1>/g, '').replace(/<svg[\s\S]*?<\/svg>/g, '[figure]').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ');
    const problems = [...html.matchAll(/class="err"[^>]*>([\s\S]*?)<\//g)].map((m) => m[1].replace(/<[^>]+>/g, '').trim().slice(0, 200));
    if (!html.length) problems.push('the engine answered nothing');
    if (/Python error/.test(plain)) problems.push((plain.match(/Python error[^.]{0,160}/) ?? [''])[0]);
    const figs = (html.match(/cprs-py-figure/g) ?? []).length;
    const charts = (html.match(/data-plotly|cprs-plotly|plotly-chart/g) ?? []).length;
    let formNote = '';
    if (b.form && !problems.length) {
        // The input form: switch to Input and count the controls the form drew.
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click({ timeout: 10000 });
        let n = 0;
        for (let k = 0; k < 40 && !n; k++) {
            await page.waitForTimeout(500);
            for (const f of page.frames()) {
                if (f === page.mainFrame()) continue;
                n = Math.max(n, await f.locator('input.calcpad-ui-input, select.calcpad-ui-dropdown, .calcpad-ui-radio, input.calcpad-ui-checkbox, .calcpad-ui-datagrid').count().catch(() => 0));
            }
        }
        const expect = (b.text.match(/^>>\s*#UI/gm) ?? []).length;
        if (n < expect) problems.push(`the form drew ${n} controls for ${expect} #UI lines`);
        formNote = `, form: ${n} controls`;
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Preview$/ }).first().click({ timeout: 10000 });
        const dont = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
        if (await dont.waitFor({ timeout: 1500 }).then(() => true, () => false)) await dont.click();
    }
    console.log(`${problems.length ? 'FAIL' : 'ok  '} ${b.where} (${figs} figures, ${charts} chart marks${formNote})`);
    if (problems.length) { bad++; for (const p of problems) console.log(`     ${p}`); }
    if (show) console.log(plain.slice(0, 1500) + '\n');
}
await browser.close();
server.close();
console.log(`${blocks.length} browser examples: ${blocks.length - bad} ok, ${bad} failed`);
process.exit(bad ? 1 : 0);
