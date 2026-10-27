// Offline check: load the built app, wait for the service worker to cache it, then go offline and reload.
//   node e2e/pwa-check.mjs
//   PWA_BASE=/calcs/ node e2e/pwa-check.mjs   for a build made with CALCS_BASE=/calcs/ (the server strips the prefix)
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, extname, normalize } from 'node:path';
const dist = new URL('../calcpad-web/dist/', import.meta.url).pathname;
const T = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.woff2': 'font/woff2', '.otf': 'font/otf', '.webmanifest': 'application/manifest+json' };
const base = process.env.PWA_BASE || '/';
const srv = createServer((q, r) => { let p = decodeURIComponent(new URL(q.url, 'http://x').pathname); if (!p.startsWith(base)) { r.writeHead(404); return r.end(); } p = '/' + p.slice(base.length); if (p.endsWith('/')) p += 'index.html'; const f = normalize(join(dist, p)); if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { r.writeHead(404); return r.end(); } r.writeHead(200, { 'Content-Type': T[extname(f)] ?? 'application/octet-stream' }); r.end(readFileSync(f)); });
await new Promise(ok => srv.listen(4197, '127.0.0.1', ok));
let bad = 0;
const check = (ok, what) => { console.log(ok ? 'ok  ' : 'FAIL', what); if (!ok) bad++; };
const b = await chromium.launch();
const ctx = await b.newContext({ viewport: { width: 1400, height: 820 } });
const page = await ctx.newPage();
await page.goto('http://127.0.0.1:4197' + base);
await page.waitForFunction(() => window.calcpadActiveEditor != null);
const ready = await page.evaluate(async () => { const r = await navigator.serviceWorker.ready; await new Promise(ok => setTimeout(ok, 500)); return !!r.active; });
check(ready, 'service worker active');
// Wait until the precache holds the build.
const want = (await (await fetch('http://127.0.0.1:4197' + base + 'sw.js')).text()).match(/const FILES = (\[.*\]);/)[1];
const n = JSON.parse(want).length;
let have = 0;
for (let i = 0; i < 60 && have < n; i++) { have = await page.evaluate(async () => { const k = (await caches.keys())[0]; return k ? (await (await caches.open(k)).keys()).length : 0; }); await page.waitForTimeout(500); }
check(have >= n, `precache holds ${have} of ${n} files`);
const manifest = await page.evaluate(async () => (await fetch(document.querySelector('link[rel=manifest]').href)).json());
check(manifest.name === 'CivilKit Calcs' && manifest.icons.length >= 2, 'manifest');
await ctx.setOffline(true);
await page.reload();
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 30000 }).then(() => check(true, 'app boots offline'), () => check(false, 'app boots offline'));
await page.evaluate(() => window.calcpadActiveEditor.setValue('a = 3m\nb = 4m\nc = sqrt(a^2 + b^2)'));
await page.waitForTimeout(3000);
if (process.env.PWA_SHOT) await page.screenshot({ path: process.env.PWA_SHOT });
const texts = await Promise.all(page.frames().map(f => f.evaluate(() => document.body.innerText).catch(() => '')));
check(/=\s*5\s*m/.test(texts.join(' ')), 'calculates offline (wasm)');
await b.close(); srv.close();
process.exit(bad ? 1 : 0);
