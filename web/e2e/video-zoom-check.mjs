// Click to enlarge on the What's new clips and the guide videos, in a real browser.
//   node e2e/video-zoom-check.mjs [--book DIR] [--clips DIR] [--shots DIR] [--port 4196]
// Serves calcpad-web/public under /calcs/ (so the page is at /calcs/whats-new/ and the shared script at /calcs/zoom/),
// the built guide (--book, default target/book from book/build.sh) at /calcs/guide/ and, if given, the clip files at
// /calcs/whats-new/clips/. Without the clip files the lightbox still opens (poster, "could not be loaded" note);
// the guide part is skipped when the guide is not built.
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const pub = resolve(here, '../calcpad-web/public');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const book = resolve(arg('--book', join(here, '../../target/book')));
const clipsDir = arg('--clips', '') ? resolve(arg('--clips', '')) : '';
const shots = arg('--shots', '');
const port = Number(arg('--port', '4196'));
if (shots) mkdirSync(shots, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const TYPES = { '.html': 'text/html; charset=utf-8', '.css': 'text/css', '.png': 'image/png', '.jpg': 'image/jpeg', '.webm': 'video/webm', '.mp4': 'video/mp4', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.json': 'application/json' };
const server = createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    let file = '';
    const under = (root, prefix) => normalize(join(root, path.slice(prefix.length)));
    if (path.startsWith('/calcs/whats-new/clips/')) file = clipsDir ? under(clipsDir, '/calcs/whats-new/clips/') : '';
    else if (path.startsWith('/calcs/guide/')) file = under(book, '/calcs/guide/');
    else if (path.startsWith('/calcs/')) file = under(pub, '/calcs/');
    if (file.endsWith('/')) file += 'index.html';
    if (!file || !existsSync(file) || !statSync(file).isFile()) { res.writeHead(404); res.end('not found'); return; }
    const size = statSync(file).size, type = TYPES[extname(file)] ?? 'application/octet-stream';
    const range = /bytes=(\d*)-(\d*)/.exec(req.headers.range ?? '');
    if (range) {
        const a = range[1] ? Number(range[1]) : 0, b = range[2] ? Number(range[2]) : size - 1;
        res.writeHead(206, { 'Content-Type': type, 'Content-Range': `bytes ${a}-${b}/${size}`, 'Accept-Ranges': 'bytes', 'Content-Length': b - a + 1 });
        res.end(readFileSync(file).subarray(a, b + 1));
    } else { res.writeHead(200, { 'Content-Type': type, 'Accept-Ranges': 'bytes', 'Content-Length': size }); res.end(readFileSync(file)); }
});
await new Promise((ok) => server.listen(port, '127.0.0.1', ok));
const base = `http://127.0.0.1:${port}/calcs/`;
const browser = await chromium.launch();
const sleep = (p, ms) => p.waitForTimeout(ms);
const dlg = '.vz-back';
const box = (p, sel) => p.locator(sel).first().boundingBox();
const active = (p) => p.evaluate(() => { const a = document.activeElement; return { inDialog: !!a?.closest('.vz-back'), onFrame: a?.closest?.('figure.clip')?.id ?? '', tag: a?.tagName ?? '' }; });

async function whatsNew(name, opts) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(String(e)));
    await page.goto(`${base}whats-new/`, { waitUntil: 'load' });
    await sleep(page, 400);
    const phone = !!opts.isMobile;
    const n = await page.locator('figure.clip .frame').count();
    const roles = await page.$$eval('figure.clip .frame', (fs) => fs.every((f) => f.getAttribute('role') === 'button' && /^Enlarge: .+/.test(f.getAttribute('aria-label') ?? '') && f.tabIndex === 0 && getComputedStyle(f).cursor === 'zoom-in' && !!f.querySelector('.vz-cue')));
    check(n > 10 && roles, `${name}: all ${n} clips are buttons labelled "Enlarge: <title>" with a zoom cursor and a cue icon`);
    check(await page.evaluate(() => typeof window.CivilKitZoom === 'object' && CivilKitZoom.count() === document.querySelectorAll('figure.clip').length), `${name}: the lightbox knows every clip`);

    const idx = 2;
    const frame = page.locator('figure.clip .frame').nth(idx);
    await frame.scrollIntoViewIfNeeded();
    const thumb = await frame.boundingBox();
    const title = (await page.locator('figure.clip h3').nth(idx).textContent()).trim();
    if (phone) await frame.tap(); else await frame.click();
    await page.waitForSelector(dlg, { timeout: 5000 });
    await sleep(page, 700);
    const vb = await box(page, `${dlg} video`);
    check(vb.width > thumb.width + 20, `${name}: the enlarged clip (${Math.round(vb.width)}x${Math.round(vb.height)}) is larger than the thumbnail (${Math.round(thumb.width)}x${Math.round(thumb.height)})`);
    const nativeW = Number(await page.locator('figure.clip video').nth(idx).getAttribute('width'));
    check(vb.width <= nativeW + 1, `${name}: never past the clip's own width (${Math.round(vb.width)} <= ${nativeW})`);
    if (!phone) check(Math.abs(vb.width - nativeW) <= 1, `${name}: at the clip's native ${nativeW} px on a large window`);
    else check(Math.abs(vb.width - 390) <= 1, `${name}: fills the phone width (${Math.round(vb.width)} of 390)`);
    check((await page.locator('.vz-title').textContent()).trim() === title, `${name}: the dialog shows the clip title "${title}"`);
    check((await page.locator('.vz-caption').textContent()).trim().length > 20, `${name}: the dialog shows the caption`);
    check(await page.evaluate(() => { const b = document.querySelector('.vz-back'); return b.getAttribute('role') === 'dialog' && b.getAttribute('aria-modal') === 'true'; }), `${name}: role=dialog, aria-modal`);
    const ov = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, iw: innerWidth, bw: document.querySelector('.vz-box').getBoundingClientRect().right, ofl: document.documentElement.style.overflow }));
    check(ov.sw <= ov.iw && ov.bw <= ov.iw + 0.5, `${name}: no horizontal overflow with the lightbox open (${ov.sw} <= ${ov.iw}, box right ${Math.round(ov.bw)})`);
    check(ov.ofl === 'hidden', `${name}: the page behind does not scroll`);
    check((await active(page)).inDialog, `${name}: focus moves into the dialog`);
    if (shots) await page.screenshot({ path: join(shots, `whats-new-${name.replace(' ', '-')}-open.png`) });

    // focus trap
    let escaped = false;
    for (let i = 0; i < 9; i++) { await page.keyboard.press('Tab'); if (!(await active(page)).inDialog) escaped = true; }
    for (let i = 0; i < 9; i++) { await page.keyboard.press('Shift+Tab'); if (!(await active(page)).inDialog) escaped = true; }
    check(!escaped, `${name}: Tab and Shift+Tab stay inside the dialog`);

    // arrows step
    const titleAt = async () => (await page.locator('.vz-title').textContent()).trim();
    const t2 = await page.locator('figure.clip h3').nth(idx + 1).textContent();
    const tPrev = await page.locator('figure.clip h3').nth(idx - 1).textContent();
    await page.keyboard.press('ArrowRight');
    check((await titleAt()) === t2.trim() && /^4 of /.test(await page.locator('.vz-count').textContent()), `${name}: ArrowRight steps to the next clip ("${t2.trim()}")`);
    await page.keyboard.press('ArrowLeft'); await page.keyboard.press('ArrowLeft');
    check((await titleAt()) === tPrev.trim(), `${name}: ArrowLeft steps back twice to the previous clip ("${tPrev.trim()}")`);
    await page.locator('.vz-next').click();
    check((await titleAt()) === title, `${name}: the Next button steps forward`);
    await page.locator('.vz-prev').click();
    check((await titleAt()) === tPrev.trim(), `${name}: the Previous button steps back`);
    await page.keyboard.press('ArrowRight');   // back on the opened clip

    await page.keyboard.press('Escape');
    await sleep(page, 200);
    const a = await active(page);
    check((await page.locator(dlg).count()) === 0 && a.onFrame === await page.locator('figure.clip').nth(idx).getAttribute('id'), `${name}: Esc closes and focus returns to the clip (${a.onFrame})`);
    check(await page.evaluate(() => document.documentElement.style.overflow !== 'hidden' && ![...document.body.children].some((c) => c.inert)), `${name}: scrolling and the page are restored`);

    // Enter opens from the keyboard, the Close button and a click outside close
    await page.locator('figure.clip .frame').nth(idx).focus();
    await page.keyboard.press('Enter');
    await page.waitForSelector(dlg, { timeout: 5000 });
    check((await titleAt()) === title, `${name}: Enter on a focused clip opens it`);
    await page.locator('.vz-close').click();
    check((await page.locator(dlg).count()) === 0, `${name}: the Close button closes`);
    await page.locator('figure.clip .frame').nth(idx).focus();
    await page.keyboard.press(' ');
    await page.waitForSelector(dlg, { timeout: 5000 });
    if (phone) await page.touchscreen.tap(195, 4); else await page.mouse.click(8, 8);
    await sleep(page, 200);
    check((await page.locator(dlg).count()) === 0, `${name}: ${phone ? 'a tap' : 'a click'} outside closes`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

const D = { viewport: { width: 1280, height: 900 } };
const P = { viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true };
await whatsNew('desktop light', { ...D, colorScheme: 'light' });
await whatsNew('desktop dark', { ...D, colorScheme: 'dark' });
await whatsNew('phone light', { ...P, colorScheme: 'light' });
await whatsNew('phone dark', { ...P, colorScheme: 'dark' });

// the first and last clip: Previous / Next are disabled at the ends and arrows do nothing there
{
    const ctx = await browser.newContext({ ...D });
    const page = await ctx.newPage();
    await page.goto(`${base}whats-new/`, { waitUntil: 'load' });
    await page.locator('figure.clip .frame').first().click();
    await page.waitForSelector(dlg);
    const t = await page.locator('.vz-title').textContent();
    await page.keyboard.press('ArrowLeft');
    check((await page.locator('.vz-prev').isDisabled()) && (await page.locator('.vz-title').textContent()) === t, 'first clip: Previous is disabled and ArrowLeft stays put');
    await ctx.close();
}
// reduced motion: the lightbox opens but does not start the clip
{
    const ctx = await browser.newContext({ ...D, reducedMotion: 'reduce' });
    const page = await ctx.newPage();
    await page.goto(`${base}whats-new/`, { waitUntil: 'load' });
    await page.locator('figure.clip .frame').nth(1).click();
    await page.waitForSelector(dlg);
    await sleep(page, 800);
    check(await page.$eval('.vz-video', (v) => v.paused && v.controls), 'reduced motion: the enlarged clip is paused until played, with controls');
    await ctx.close();
}
// the guide
if (existsSync(join(book, 'units.html'))) {
    for (const [name, opts] of [['guide desktop light', { ...D, colorScheme: 'light' }], ['guide desktop dark', { ...D, colorScheme: 'dark' }], ['guide phone', { ...P, colorScheme: 'light' }]]) {
        const ctx = await browser.newContext(opts);
        const page = await ctx.newPage();
        const errors = [];
        page.on('pageerror', (e) => errors.push(String(e)));
        await page.goto(`${base}guide/units.html`, { waitUntil: 'load' });
        await sleep(page, 400);
        const phone = !!opts.isMobile;
        const btn = page.locator('main .vz-enlarge');
        check((await btn.count()) === 1 && /^Enlarge: /.test(await btn.getAttribute('aria-label')), `${name}: the chapter video has an "Enlarge: <chapter>" button`);
        check(await page.$eval('main video', (v) => v.controls && v.closest('.vz-wrap') !== null), `${name}: the video keeps its native controls`);
        const thumb = await page.locator('main video').boundingBox();
        if (phone) await btn.tap(); else await btn.click();
        await page.waitForSelector(dlg);
        await sleep(page, 700);
        const vb = await box(page, `${dlg} video`);
        check(vb.width > thumb.width + (phone ? -1 : 20) || phone, `${name}: enlarged ${Math.round(vb.width)} px vs ${Math.round(thumb.width)} px in the page`);
        check(!phone ? vb.width > thumb.width + 20 : vb.width >= 389, `${name}: ${phone ? 'fills the width' : 'larger than the thumbnail'}`);
        check((await page.locator('.vz-prev').isHidden()) && (await page.locator('.vz-next').isHidden()), `${name}: a lone video has no previous/next buttons`);
        check(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${name}: no horizontal overflow`);
        if (shots) await page.screenshot({ path: join(shots, `${name.replace(/ /g, '-')}-open.png`) });
        await page.keyboard.press('Escape');
        await sleep(page, 200);
        check((await page.locator(dlg).count()) === 0 && await page.evaluate(() => document.activeElement.classList.contains('vz-enlarge')), `${name}: Esc closes and focus returns to the Enlarge button`);
        check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
        if (shots && !phone) await page.screenshot({ path: join(shots, `${name.replace(/ /g, '-')}-page.png`) });
        await ctx.close();
    }
} else console.log(`skip guide: ${book} is not built (run book/build.sh)`);
// thumbnails with the cue, for the eyes
if (shots) {
    for (const [name, opts] of [['desktop-light', { ...D, colorScheme: 'light' }], ['phone-dark', { ...P, colorScheme: 'dark' }]]) {
        const ctx = await browser.newContext(opts);
        const page = await ctx.newPage();
        await page.goto(`${base}whats-new/`, { waitUntil: 'load' });
        await page.locator('figure.clip').first().scrollIntoViewIfNeeded();
        await sleep(page, 400);
        await page.screenshot({ path: join(shots, `whats-new-${name}-thumbs.png`) });
        await ctx.close();
    }
}
await browser.close();
server.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
