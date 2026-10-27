// Editing a sheet block where it stands (mouse): one click opens it, clicking away applies, Esc cancels, a drag that
// selects text does not open it, and a click on another block only closes the open editor.   node e2e/inline-edit-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

{
    const ctx = await browser.newContext({ viewport: { width: 1300, height: 900 } });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    const frame = page.locator('iframe').first().contentFrame();
    await frame.locator('.ck-ins-btn').first().waitFor({ timeout: 30000 });
    await page.waitForTimeout(2000);
    const src = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
    const base = await src();
    const at = async (n, dx = 20) => { const b = await frame.locator(`[data-source-line="${n}"]`).first().boundingBox(); return { x: b.x + dx, y: b.y + Math.min(8, b.height / 2), b }; };
    const away = async () => { await page.locator('.monaco-editor').first().click({ position: { x: 300, y: 300 } }); await page.waitForTimeout(500); };

    // one click opens the one-line editor, with no Apply button
    let p = await at(9);
    await page.mouse.click(p.x, p.y);
    await page.waitForSelector('.line-edit-input', { timeout: 10000 });
    check(await page.locator('.line-edit-input').inputValue() === 'a: 3 m | first side', 'one click opens the line editor');
    check(!(await page.locator('.line-edit-foot button').first().isVisible()), 'no Apply or Cancel buttons with a mouse');
    // click away applies
    await page.locator('.line-edit-input').fill('a: 5 m | first side');
    await away();
    check((await src()).split('\n')[8] === 'a: 5 m | first side', 'clicking away applied the change');
    check(await page.locator('.line-edit').count() === 0, 'the editor closed');
    await page.waitForTimeout(1200);

    // Esc cancels
    p = await at(9);
    await page.mouse.click(p.x, p.y);
    await page.waitForSelector('.line-edit-input');
    await page.locator('.line-edit-input').fill('a: 99 m');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(300);
    check((await src()).split('\n')[8] === 'a: 5 m | first side' && await page.locator('.line-edit').count() === 0, 'Esc throws the typing away');
    await page.waitForTimeout(500);

    // clicking away with no change leaves the source exactly as it was
    p = await at(9);
    await page.mouse.click(p.x, p.y);
    await page.waitForSelector('.line-edit-input');
    const before = await src();
    await away();
    check(await src() === before && await page.locator('.line-edit').count() === 0, 'clicking away without typing changes nothing');
    await page.waitForTimeout(500);

    // a click on another block while editing closes the editor, and opens nothing
    p = await at(9);
    await page.mouse.click(p.x, p.y);
    await page.waitForSelector('.line-edit-input');
    await page.locator('.line-edit-input').fill('a: 6 m | first side');
    await page.waitForTimeout(500);
    const q = await at(10);
    await page.mouse.click(q.x, q.y + 150); // somewhere on the preview, below the lines
    await page.waitForTimeout(900);
    check((await src()).split('\n')[8] === 'a: 6 m | first side' && await page.locator('.line-edit').count() === 0, 'a click elsewhere on the preview applies and opens no second editor');
    await page.waitForTimeout(500);

    // dragging across text selects it and does not open an editor
    p = await at(4, 4);
    await page.mouse.move(p.x, p.y);
    await page.mouse.down();
    await page.mouse.move(p.x + 200, p.y, { steps: 6 });
    await page.mouse.up();
    await page.waitForTimeout(600);
    check(await page.locator('.line-edit').count() === 0 && await page.locator('.md-edit').count() === 0, 'a drag that selects text does not open the editor');

    check(errors.length === 0, `no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// a finger: a tap shows the insert handles and opens nothing
{
    const ctx = await browser.newContext({ ...devices['iPhone 14'] });
    const page = await ctx.newPage();
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const frame = page.locator('iframe').first().contentFrame();
    await frame.locator('.ck-ins-btn').first().waitFor({ timeout: 30000 });
    await page.waitForTimeout(2000);
    await frame.locator('[data-source-line="9"]').first().tap();
    await page.waitForTimeout(800);
    check(await page.locator('.line-edit, .md-edit').count() === 0, 'phone: a tap opens no editor (long-press does)');
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
