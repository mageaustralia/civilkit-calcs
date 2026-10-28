// The grip at the left of a rendered block: it shows on hover or tap, and its menu moves, duplicates and deletes the block
// (Undo after a delete), on a desktop window and on a phone.   node e2e/block-menu-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    // A small sheet of known blocks, so the expected text is exact.
    const sheet = ['<!--sheet-->', '# Beam', '', 'a: 1 m | first', '', 'b: 2 m | second', '', 'c: 3 m | third'].join('\n');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    // The preview keeps a front and a back frame and swaps them after each render; the front one is the one the pointer reaches.
    const front = async () => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('.ck-grip').count().catch(() => 0) >= 4 && /Beam/.test(await f.locator('body').innerText().catch(() => ''))) return f;
            await page.waitForTimeout(500);
        }
        return null;
    };
    let frame = await front();
    check(!!frame, `${name}: the preview shows the sheet`);
    if (!frame) { await ctx.close(); continue; }
    await page.waitForTimeout(1500);
    const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
    const grips = await frame.locator('.ck-grip').count();
    check(grips >= 4, `${name}: a grip for each block (${grips})`);
    const open = async (n) => {
        frame = await front();
        const target = frame.locator('[data-source-line]').nth(n);
        await target.scrollIntoViewIfNeeded();
        if (phone) await target.tap(); else { const bb = await target.boundingBox(); await page.mouse.move(bb.x + bb.width / 2 - 30, bb.y + bb.height / 2 + 1); await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 }); }
        await page.waitForTimeout(250);
        const on = frame.locator('.ck-grip.on');
        if (await on.count() !== 1) console.log('DEBUG', n, await frame.locator('[data-source-line]').evaluateAll((els, k) => { const e = els[k]; const r = e.getBoundingClientRect(); return [e.tagName, e.textContent.slice(0, 30), r.top, r.height, window.pageYOffset, innerHeight]; }, n), await frame.locator('.ck-grip').evaluateAll(g => g.map(x => [x.style.top, x._top, x._bottom, x.className])));
        check(await on.count() === 1, `${name}: only the grip of the block under the pointer shows`);
        phone ? await on.tap() : await on.click();
        await page.waitForSelector('.block-menu');
    };
    const act = async (label) => { const b = page.locator('.block-menu button', { hasText: label }); phone ? await b.tap() : await b.click(); await page.waitForTimeout(500); };
    const idx = async (needle) => (frame = await front()).locator('[data-source-line]').evaluateAll((els, n) => els.findIndex(e => e.textContent.includes(n)), needle);

    // Move down: b swaps with c.
    await open(await idx('second'));
    check(await page.locator('.block-menu button').count() === 4, `${name}: four actions`);
    await act('Move down');
    check((await text()).endsWith('a: 1 m | first\n\nc: 3 m | third\n\nb: 2 m | second'), `${name}: move down swaps the block with the next`); if (fails.length) console.log(JSON.stringify(await text()));
    await page.waitForTimeout(1500);
    // Duplicate a.
    await open(await idx('first'));
    await act('Duplicate');
    check((await text()).split('a: 1 m | first').length === 3, `${name}: duplicate adds a copy`);
    await page.waitForTimeout(1500);
    // Delete c, then Undo.
    const before = await text();
    await open(await idx('third'));
    await act('Delete');
    const gone = await text();
    check(!gone.includes('third') && gone.length < before.length, `${name}: delete removes the block`);
    check(await page.locator('.block-toast').count() === 1, `${name}: a toast offers Undo`);
    phone ? await page.locator('.block-toast button').tap() : await page.locator('.block-toast button').click();
    await page.waitForTimeout(500);
    const restored = await text();
    check(restored === before, `${name}: Undo restores the sheet exactly`);
    if (restored !== before) console.log(JSON.stringify(before), '\n', JSON.stringify(restored));
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
