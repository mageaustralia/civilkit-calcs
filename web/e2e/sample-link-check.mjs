// The ?sample= address parameter opens a sample (the landing pages in public/learn link to it).
//   node e2e/sample-link-check.mjs [--dist DIR] [--port 4183] [--base /calcs/]   (--base for a build made with CALCS_BASE)
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const port = Number(arg('--port', '4183'));
const base = arg('--base', '/');
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };
const TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.sheet': 'text/plain' };
const server = createServer((req, res) => {
    let path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    if (!path.startsWith(base)) { res.writeHead(404); res.end('not found'); return; }
    path = '/' + path.slice(base.length);
    if (path.endsWith('/')) path += 'index.html';
    const file = normalize(join(dist, path));
    if (!file.startsWith(dist) || !existsSync(file) || !statSync(file).isFile()) { res.writeHead(404); res.end('not found'); return; }
    res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
const browser = await chromium.launch();

async function open(query, expectText) {
    const context = await browser.newContext({ viewport: { width: 1400, height: 850 } });
    const page = await context.newPage();
    await page.goto(`http://127.0.0.1:${port}${base}${query}`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    let got = false;
    if (expectText) got = await page.waitForFunction((t) => window.calcpadActiveEditor.getValue().includes(t), expectText, { timeout: 15000 }).then(() => true, () => false);
    else await page.waitForTimeout(2500);
    const text = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    const url = page.url();
    await context.close();
    return { got, text, url };
}

let r = await open('?sample=template-simply-supported-beam', 'Simply supported beam check');
check(r.got, 'the id opens the simply supported beam template');
check(!/sample=/.test(r.url), `the parameter is removed from the address afterwards (${r.url})`);
r = await open('?sample=' + encodeURIComponent('Template simply supported beam'), 'Simply supported beam check');
check(r.got, 'the file name (without extension) opens the same template');
r = await open('?sample=retaining-wall-stability', 'etaining');
check(r.got, 'the id without the template- prefix opens the retaining wall template');
r = await open('?sample=no-such-sample', '');
check(!/etaining|Simply supported beam check/.test(r.text), 'an unknown name opens nothing extra and does not break the page');
await browser.close(); server.close();
if (failures.length) { console.log(`\n${failures.length} failed`); process.exit(1); }
console.log('\nall passed');
