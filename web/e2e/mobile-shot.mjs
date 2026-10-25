// Screenshots the built app at phone sizes.  node e2e/mobile-shot.mjs <outdir> [w h]
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname, normalize } from 'node:path';
const dist = new URL('../calcpad-web/dist/', import.meta.url).pathname;
const T = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.otf': 'font/otf', '.webmanifest': 'application/manifest+json' };
const srv = createServer((q, r) => { let p = decodeURIComponent(new URL(q.url, 'http://x').pathname); if (p.endsWith('/')) p += 'index.html'; const f = normalize(join(dist, p)); if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { r.writeHead(404); return r.end(); } r.writeHead(200, { 'Content-Type': T[extname(f)] ?? 'application/octet-stream' }); r.end(readFileSync(f)); });
await new Promise(ok => srv.listen(4198, '127.0.0.1', ok));
const out = process.argv[2], w = +(process.argv[3] ?? 390), h = +(process.argv[4] ?? 800);
const b = await chromium.launch();
const ctx = await b.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 2, hasTouch: true, isMobile: true });
const page = await ctx.newPage();
await page.goto('http://127.0.0.1:4198/');
await page.waitForFunction(() => window.calcpadActiveEditor != null);
await page.evaluate(() => window.calcpadActiveEditor.setValue('a = 3m\nb = 4m\nc = sqrt(a^2 + b^2)'));
await page.waitForTimeout(2500);
await page.screenshot({ path: `${out}/m-${w}.png` });
for (const [name, sel] of [['results', '.mobile-bar button:nth-child(2)'], ['insert', '.mobile-bar button:nth-child(3)']]) {
  await page.tap(sel); await page.waitForTimeout(800);
  await page.screenshot({ path: `${out}/m-${w}-${name}.png` });
}
await page.tap('.mobile-bar button:nth-child(1)'); await page.tap('.rb-tab'); await page.waitForTimeout(400);
await page.screenshot({ path: `${out}/m-${w}-ribbon.png` });
console.log(JSON.stringify(await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, iw: innerWidth }))));
await b.close(); srv.close();
