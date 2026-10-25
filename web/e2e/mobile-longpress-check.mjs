// Long press on a rendered paragraph opens the in-place editor (phone layout).
// Screenshots the built app at phone sizes.  node e2e/mobile-shot.mjs <outdir> [w h]
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname, normalize } from 'node:path';
const dist = new URL('../calcpad-web/dist/', import.meta.url).pathname;
const T = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.otf': 'font/otf', '.webmanifest': 'application/manifest+json' };
const srv = createServer((q, r) => { let p = decodeURIComponent(new URL(q.url, 'http://x').pathname); if (p.endsWith('/')) p += 'index.html'; const f = normalize(join(dist, p)); if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { r.writeHead(404); return r.end(); } r.writeHead(200, { 'Content-Type': T[extname(f)] ?? 'application/octet-stream' }); r.end(readFileSync(f)); });
await new Promise(ok => srv.listen(4198, '127.0.0.1', ok));
const out = process.argv[2];
const br = await chromium.launch();
const ctx = await br.newContext({ viewport: { width: 390, height: 800 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true });
const page = await ctx.newPage();
await page.goto('http://127.0.0.1:4198/');
await page.waitForFunction(() => window.calcpadActiveEditor != null);
await page.evaluate(() => window.calcpadActiveEditor.setValue('#markdown\nSome *prose* to edit.\n\nSecond paragraph.\n#end markdown\nc = 5m'));
await page.waitForTimeout(500);
await page.tap('.mobile-bar button:nth-child(2)');
await page.waitForTimeout(2500);
let hit = null;
for (const f of page.frames()) {
  if (f === page.mainFrame()) continue;
  const r = await f.evaluate(() => { const el = [...document.querySelectorAll('[data-source-line]')].find(e => /Some/.test(e.textContent)); if (!el) return null; const q = el.getBoundingClientRect(); return { x: q.left + 20, y: q.top + q.height / 2 }; }).catch(() => null);
  if (!r) continue;
  const fe = await (await f.frameElement()).boundingBox();
  if (fe && fe.width > 100) { hit = { x: fe.x + r.x, y: fe.y + r.y }; break; }
}
if (!hit) { console.log('no rendered paragraph found'); process.exit(1); }
const cdp = await ctx.newCDPSession(page);
const pt = [{ x: hit.x, y: hit.y }];
await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: pt });
await page.waitForTimeout(800);
await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
await page.waitForTimeout(1500);
const open = await page.evaluate(() => !!document.querySelector('.md-edit, .line-edit'));
console.log('editor opened by long press:', open);
await page.screenshot({ path: `${out}/m-longpress.png` });
await br.close(); srv.close();
process.exit(open ? 0 : 1);
