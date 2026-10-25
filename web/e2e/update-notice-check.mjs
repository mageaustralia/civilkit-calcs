// A running tab is told when a newer build is available (service worker update), and Reload loads it.
//
//   node e2e/update-notice-check.mjs [--port 4182] [--dist DIR]
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const port = Number(arg('--port', '4182'));
const dist = resolve(arg('--dist', join(here, '..', 'calcpad-web', 'dist')));
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };

let build = 1; // the server pretends to be build 1, then build 2 (a different sw.js)
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.webmanifest': 'application/manifest+json' };
const server = createServer((req, res) => {
    let p = decodeURIComponent(new URL(req.url, `http://127.0.0.1:${port}`).pathname);
    if (p.endsWith('/')) p += 'index.html';
    const f = normalize(join(dist, p));
    if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); return res.end(); }
    let body = readFileSync(f);
    if (p === '/sw.js') body = Buffer.from(body.toString().replace(/const CACHE = '([^']*)'/, `const CACHE = '$1-b${build}'`));
    res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream', 'Cache-Control': 'no-cache' });
    res.end(body);
});
await new Promise(ok => server.listen(port, '127.0.0.1', ok));

const browser = await chromium.launch();
const page = await (await browser.newContext({ viewport: { width: 1300, height: 800 } })).newPage();
await page.goto(`http://127.0.0.1:${port}/`);
await page.waitForFunction(() => navigator.serviceWorker?.controller, null, { timeout: 30000 }).catch(async () => {
    await page.reload();
    await page.waitForFunction(() => navigator.serviceWorker?.controller, null, { timeout: 30000 });
});
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
check(await page.locator('#cprs-update-notice').count() === 0, 'no notice on a fresh load');
build = 2;
await page.evaluate(() => navigator.serviceWorker.getRegistration().then(r => r.update()));
await page.waitForSelector('#cprs-update-notice', { timeout: 30000 }).catch(() => {});
check(await page.locator('#cprs-update-notice').count() === 1, 'a newer build shows "new version available"');
check(/Save your work/.test(await page.locator('#cprs-update-notice').innerText().catch(() => '')), 'it asks the person to save first');
await page.getByRole('button', { name: 'Later' }).click();
check(await page.locator('#cprs-update-notice').count() === 0, 'Later dismisses it');
await page.evaluate(() => { window.__marker = 1; });
build = 3;
await page.evaluate(() => navigator.serviceWorker.getRegistration().then(r => r.update()));
await page.waitForSelector('#cprs-update-notice', { timeout: 30000 }).catch(() => {});
await Promise.all([page.waitForNavigation({ timeout: 30000 }), page.locator('[data-cprs-update-reload]').click()]);
check(await page.evaluate(() => window.__marker) === undefined, 'Reload reloads the page');
await browser.close();
server.close();
console.log(failures.length ? `${failures.length} FAILED` : 'all update-notice checks passed');
process.exit(failures.length ? 1 : 0);
