// A block next to hidden code (a `>> #hide` run with a loop): deleting or moving the block through its grip leaves the hidden
// code alone, so the sheet still calculates.   node e2e/block-hidden-check.mjs [--url U]
import { chromium } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1300, height: 900 } });
const page = await ctx.newPage();
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
await page.waitForSelector('.monaco-editor');
const HIDDEN = ['>> #hide', '>> a = vector(n + 1)', 'for k in 0..n:', '    >> a.(k + 1) = (-1)^k', '>> #show'];
const base = ['<!--sheet-->', '# Roots', '', 'n: 5', '', ...HIDDEN, '', 'coefficients: a'];
const withNew = ['<!--sheet-->', '# Roots', '', 'n: 5', '', 'quantity: 1 m | note', '', ...HIDDEN, '', 'coefficients: a'];
await page.evaluate(t => window.calcpadActiveEditor.setValue(t), withNew.join('\n'));
const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
const front = async () => {
    for (let i = 0; i < 60; i++) {
        const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
        const f = h ? await h.contentFrame() : null;
        if (f && await f.locator('.ck-grip').count().catch(() => 0) >= 3 && /quantity/.test(await f.locator('body').innerText().catch(() => ''))) return f;
        await page.waitForTimeout(500);
    }
    return null;
};
const openMenu = async () => {
    const frame = await front();
    const target = frame.locator('[data-source-line]', { hasText: 'quantity' }).first();
    await target.scrollIntoViewIfNeeded();
    const bb = await target.boundingBox();
    await page.mouse.move(bb.x + bb.width / 2 - 30, bb.y + bb.height / 2 + 1);
    await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 });
    await page.waitForTimeout(300);
    await frame.locator('.ck-grip.on').click();
    await page.waitForSelector('.block-menu');
};
let frame = await front();
check(!!frame, 'the preview shows the sheet');
await page.waitForTimeout(1500);

await openMenu();
await page.locator('.block-menu button', { hasText: 'Delete' }).click();
await page.waitForTimeout(800);
check(await text() === base.join('\n'), 'delete removes only the block: the hidden code and the lines around it are untouched');
frame = await front();
const body = frame ? await frame.locator('body').innerText() : '';
check(!/Error|Undefined/.test(body) && /coefficients/.test(body), 'the sheet still calculates after the delete (no errors)');

await page.evaluate(t => window.calcpadActiveEditor.setValue(t), withNew.join('\n'));
await page.waitForTimeout(1500);
await openMenu();
await page.locator('.block-menu button', { hasText: 'Move down' }).click();
await page.waitForTimeout(800);
const moved = (await text()).split('\n');
check(moved.indexOf('quantity: 1 m | note') > moved.indexOf('>> #show') && moved.indexOf('quantity: 1 m | note') < moved.indexOf('coefficients: a') && HIDDEN.every(l => moved.includes(l)), 'move down steps over the hidden code as a block and keeps every line of it');

await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
