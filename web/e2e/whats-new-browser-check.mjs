// The What's new page in a real browser, without and with the clip files.
//   node e2e/whats-new-browser-check.mjs [--clips DIR] [--shots DIR] [--port 4195]
// Serves calcpad-web/public (so the page is at /whats-new/). Without clips the page must look right from the posters
// alone; with them (--clips, the folder record.mjs wrote) every <video> must play, advance and loop.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const pub = resolve(here, '../calcpad-web/public');
const clipCount = Object.values(JSON.parse(readFileSync(resolve(here, '../calcpad-web/whats-new/clips.json'), 'utf8'))).reduce((n, l) => n + l.length, 0);
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const clipsDir = arg('--clips', '') ? resolve(arg('--clips', '')) : '';
const shots = arg('--shots', '');
const port = Number(arg('--port', '4195'));
if (shots) mkdirSync(shots, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const TYPES = { '.html': 'text/html; charset=utf-8', '.css': 'text/css', '.png': 'image/png', '.webm': 'video/webm', '.mp4': 'video/mp4', '.js': 'text/javascript' };
let withClips = false;
const server = createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    let file = normalize(join(pub, path.endsWith('/') ? path + 'index.html' : path));
    if (path.startsWith('/whats-new/clips/')) file = withClips && clipsDir ? normalize(join(clipsDir, path.slice('/whats-new/clips/'.length))) : '';
    if (!file || !existsSync(file) || !statSync(file).isFile()) { res.writeHead(404); res.end('not found'); return; }
    const size = statSync(file).size;
    const range = /bytes=(\d*)-(\d*)/.exec(req.headers.range ?? '');
    const type = TYPES[extname(file)] ?? 'application/octet-stream';
    if (range) {
        const a = range[1] ? Number(range[1]) : 0, b = range[2] ? Number(range[2]) : size - 1;
        res.writeHead(206, { 'Content-Type': type, 'Content-Range': `bytes ${a}-${b}/${size}`, 'Accept-Ranges': 'bytes', 'Content-Length': b - a + 1 });
        res.end(readFileSync(file).subarray(a, b + 1));
    } else { res.writeHead(200, { 'Content-Type': type, 'Accept-Ranges': 'bytes', 'Content-Length': size }); res.end(readFileSync(file)); }
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
const url = `http://127.0.0.1:${port}/whats-new/`;
const browser = await chromium.launch();

async function open(opts) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(String(e)));
    await page.goto(url, { waitUntil: 'load' });
    return { ctx, page, errors };
}

for (const clips of clipsDir ? [false, true] : [false]) {
    withClips = clips;
    const tag = clips ? 'with clips' : 'posters only';
    for (const [name, opts] of [
        ['desktop light', { viewport: { width: 1280, height: 900 }, colorScheme: 'light' }],
        ['desktop dark', { viewport: { width: 1280, height: 900 }, colorScheme: 'dark' }],
        ['phone light', { viewport: { width: 390, height: 844 }, colorScheme: 'light', deviceScaleFactor: 2, isMobile: true, hasTouch: true }],
        ['phone dark', { viewport: { width: 390, height: 844 }, colorScheme: 'dark', deviceScaleFactor: 2, isMobile: true, hasTouch: true }],
    ]) {
        const { ctx, page, errors } = await open(opts);
        await page.waitForTimeout(clips ? 2500 : 800);
        const m = await page.evaluate(() => ({
            sw: document.documentElement.scrollWidth, iw: window.innerWidth,
            bg: getComputedStyle(document.body).backgroundColor,
            videos: document.querySelectorAll('video').length,
            posters: [...document.querySelectorAll('video')].map((v) => v.poster),
            h1: document.querySelectorAll('h1').length,
            imgs: [...document.querySelectorAll('img')].filter((i) => getComputedStyle(i).display !== 'none' && !i.closest('video')).map((i) => [i.complete, i.naturalWidth]),
        }));
        check(m.sw <= m.iw, `${tag}, ${name}: no horizontal scroll (${m.sw} <= ${m.iw})`);
        check(m.h1 === 1 && m.videos === clipCount, `${tag}, ${name}: one h1 and ${m.videos} videos`);
        const dark = name.includes('dark');
        const lum = m.bg.match(/\d+/g).slice(0, 3).map(Number).reduce((a, b) => a + b, 0) / 3;
        check(dark ? lum < 60 : lum > 200, `${tag}, ${name}: background ${m.bg}`);
        if (!clips) {
            // each poster loads as an image
            const loaded = await page.evaluate(async () => Promise.all([...document.querySelectorAll('video')].map((v) => new Promise((ok) => { const i = new Image(); i.onload = () => ok(i.naturalWidth > 0); i.onerror = () => ok(false); i.src = v.poster; }))));
            check(loaded.every(Boolean), `${tag}, ${name}: all ${loaded.length} posters load`);
            const states = await page.evaluate(() => [...document.querySelectorAll('video')].map((v) => v.readyState));
            check(states.every((s) => s === 0), `${tag}, ${name}: no clip data without the files (readyState ${[...new Set(states)]})`);
        }
        if (shots) {
            await page.screenshot({ path: join(shots, `${clips ? 'clips' : 'posters'}-${name.replace(' ', '-')}.png`), fullPage: false, timeout: 150000 });
            if (name === 'desktop light' || name === 'phone light') await page.screenshot({ path: join(shots, `${clips ? 'clips' : 'posters'}-${name.replace(' ', '-')}-full.png`), fullPage: true, timeout: 150000 });
        }
        check(errors.length === 0, `${tag}, ${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
        await ctx.close();
    }
}

if (clipsDir) {
    withClips = true;
    const { ctx, page } = await open({ viewport: { width: 1280, height: 900 }, colorScheme: 'light' });
    const ids = await page.evaluate(() => [...document.querySelectorAll('video')].map((v) => v.closest('figure').id.replace(/^clip-/, '')));
    for (const id of ids) {
        const sel = `#clip-${id} video`;
        await page.mouse.move(1, 1);   // park the pointer: scrolling a clip under it would fire the hover-to-play script's mouseleave
        await page.locator(sel).scrollIntoViewIfNeeded();
        // advances: currentTime moves while it plays
        const t = await page.evaluate(async (s) => {
            const v = document.querySelector(s);
            const t0 = Date.now();
            while (v.readyState < 3 && Date.now() - t0 < 20000) await new Promise((ok) => setTimeout(ok, 100));
            if (v.paused) await v.play().catch(() => {});
            const a = v.currentTime;
            // software decoding on a busy machine can play slowly: give it up to 8 s to move half a second
            for (let i = 0; i < 80 && v.currentTime < a + 0.6; i++) await new Promise((ok) => setTimeout(ok, 100));
            return { a, b: v.currentTime, d: v.duration, loop: v.loop, muted: v.muted, src: v.currentSrc.split('/').pop(), paused: v.paused, w: v.videoWidth, h: v.videoHeight };
        }, sel);
        check(t.b > t.a + 0.5 && !t.paused && t.loop && t.muted, `${id}: plays (${t.src}, ${t.w}x${t.h}, ${t.d.toFixed(1)} s): currentTime ${t.a.toFixed(2)} -> ${t.b.toFixed(2)}`);
        // loops: seek near the end and watch the time wrap round
        const wrapped = await page.evaluate(async (s) => {
            const v = document.querySelector(s);
            if (v.paused) await v.play().catch(() => {});
            v.currentTime = Math.max(0, v.duration - 0.6);
            let prev = v.currentTime, wrap = false;
            const t0 = Date.now();
            while (Date.now() - t0 < 12000 && !wrap) { await new Promise((ok) => setTimeout(ok, 50)); if (v.currentTime < prev - 0.3) wrap = true; prev = v.currentTime; }
            const after = v.currentTime;
            await new Promise((ok) => setTimeout(ok, 500));
            return { wrap, moving: v.currentTime > after, ended: v.ended };
        }, sel);
        check(wrapped.wrap && wrapped.moving && !wrapped.ended, `${id}: loops (time wraps to the start and keeps playing)`);
    }
    await ctx.close();
    // reduced motion: the still replaces the video
    const rm = await browser.newContext({ viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });
    const p = await rm.newPage();
    await p.goto(url);
    const vis = await p.evaluate(() => ({ v: [...document.querySelectorAll('video')].every((x) => getComputedStyle(x).display === 'none'), s: [...document.querySelectorAll('img.still')].every((x) => getComputedStyle(x).display !== 'none') }));
    check(vis.v && vis.s, 'reduced motion: the videos are hidden and the stills shown');
    await rm.close();
}
await browser.close();
server.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
