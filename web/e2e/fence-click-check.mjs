// Clicking a diagram or a code fence in the preview selects the whole fence in the editor (the one-line editor cannot hold
// it), while a plain line still opens the line editor.   node e2e/fence-click-check.mjs [--url U]
import { chromium } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1300, height: 900 } });
const page = await ctx.newPage();
const errors = [];
page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
const SHEET = ['<!--sheet-->', '# Flow', '', '```mermaid', 'graph TD', '  A --> B', '  B --> C', '```', '', 'a: 3 m | first side', '', '```', 'plain', '', 'code', '```'].join('\n');
await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
const front = async () => {
    for (let i = 0; i < 60; i++) {
        const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
        const f = h ? await h.contentFrame() : null;
        if (f && await f.locator('[data-source-line="4"]').count().catch(() => 0) && await f.locator('[data-source-line="10"]').count().catch(() => 0)) return f;
        await page.waitForTimeout(500);
    }
    return null;
};
let frame = await front();
check(!!frame, 'the preview shows the sheet');
await page.waitForTimeout(2500);
const sel = () => page.evaluate(() => { const s = window.calcpadActiveEditor.getSelection(); return [s.startLineNumber, s.endLineNumber]; });
const clickLine = async (n) => {
    frame = await front();
    const t = frame.locator(`[data-source-line="${n}"]`).first();
    await t.scrollIntoViewIfNeeded();
    const b = await t.boundingBox();
    await page.mouse.click(b.x + Math.min(60, b.width / 2), b.y + Math.min(20, b.height / 2));
    await page.waitForTimeout(700);
};

await clickLine(4);
check(JSON.stringify(await sel()) === '[4,8]', `clicking the diagram selects its whole fence in the editor (${JSON.stringify(await sel())})`);
check(await page.locator('.line-edit').count() === 0, 'no one-line editor opens over the diagram');
check(await page.evaluate(() => document.activeElement?.closest('.monaco-editor') != null), 'the editor has the focus');
await page.waitForTimeout(1500);
await clickLine(12);
check(JSON.stringify(await sel()) === '[12,16]', `a code fence with a blank line inside is selected whole (${JSON.stringify(await sel())})`);
await page.waitForTimeout(1500);
await clickLine(10);
await page.waitForSelector('.line-edit-input', { timeout: 10000 }).catch(() => {});
check(await page.locator('.line-edit-input').count() === 1 && await page.locator('.line-edit-input').inputValue() === 'a: 3 m | first side', 'a plain line still opens the line editor');
check(errors.length === 0, `no page errors${errors[0] ? ': ' + errors[0] : ''}`);
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
