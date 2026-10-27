// The dashed line and centred "+" between rendered blocks of a sheet: handles appear, the menu inserts the block at the right line and leaves its text selected,
// on a desktop window and on a phone.   node e2e/insert-strip-check.mjs [--url U]
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
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); }); // Monaco cancels pending work on teardown
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const frame = page.locator('iframe').first().contentFrame();
    await frame.locator('.ck-ins-btn').first().waitFor({ timeout: 30000 });
    await page.waitForTimeout(2000); // the handles are rebuilt as the page settles
    check(await frame.locator('.ck-ins-btn').count() >= 5, `${name}: handles between blocks`);
    const before = await page.evaluate(() => window.calcpadActiveEditor.getValue().split('\n'));
    let btn = frame.locator('.ck-ins-btn').nth(3);
    await btn.scrollIntoViewIfNeeded();
    if (phone) {
        check(await frame.locator('.ck-ins.on').count() === 0, `${name}: no handles until a block is tapped`);
        await frame.locator('[data-source-line]').nth(3).tap();
        check(await frame.locator('.ck-ins.on').count() <= 2 && await frame.locator('.ck-ins.on').count() >= 1, `${name}: tapping a block shows only its neighbouring handles`);
        btn = frame.locator('.ck-ins.on .ck-ins-btn').first();
    }
    if (!phone) {
        const zone = frame.locator('.ck-ins').nth(3);
        const line = () => zone.locator('.ck-ins-line').evaluate(e => getComputedStyle(e).opacity);
        check(await line() === '0', `${name}: no dashed line until hover`);
        const bb = await zone.boundingBox();
        await page.mouse.move(bb.x + bb.width / 2 + 200, bb.y + bb.height / 2);
        await page.waitForTimeout(300);
        check(+await line() > 0.3, `${name}: dashed line shows on hover`);
        check(+await btn.evaluate(e => getComputedStyle(e).opacity) === 1, `${name}: the + shows on hover`);
        check(await zone.evaluate(e => getComputedStyle(e).pointerEvents) === 'none', `${name}: the gap itself takes no clicks`);
        await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
        await page.waitForTimeout(200);
    }
    phone ? await btn.tap() : await btn.click();
    await page.waitForSelector('.insert-menu');
    check(await page.locator('.insert-menu button').count() === 8, `${name}: the menu lists eight blocks`);
    await page.locator('.insert-menu button', { hasText: 'Heading' }).click();
    const after = await page.evaluate(() => window.calcpadActiveEditor.getValue().split('\n'));
    const sel = await page.evaluate(() => { const e = window.calcpadActiveEditor; return e.getModel().getValueInRange(e.getSelection()); });
    check(after.length === before.length + 2 && after.includes('## Heading'), `${name}: a heading and a blank line were inserted`);
    check(sel === 'Heading', `${name}: the heading text is selected (${sel})`);
    check(after.filter(l => l === '## Heading').length === 1 && before.every(l => after.includes(l)), `${name}: nothing else changed`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
