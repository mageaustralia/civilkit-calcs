// Sorting a sheet's table in the preview: a click or tap on a column heading sorts ascending, then descending, then back to
// the source order; the source text is never changed, the sort survives a re-render, and printing shows the source order.
// The sort helper is checked in Node first.   node e2e/table-sort-check.mjs [--url U]
import { chromium, devices } from 'playwright';
import { sortOrder } from '../calcpad-web/src/wasm/table-sort.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);

// Numbers by value (units and thousands separators allowed), text naturally, empty cells last, ties in source order.
check(same(sortOrder(['10 kN', '2 kN', '', '1,200', '-3'], 1), [4, 1, 0, 3, 2]), 'numbers sort by value, empty last');
check(same(sortOrder(['10 kN', '2 kN', '', '1,200', '-3'], -1), [3, 0, 1, 4, 2]), 'descending keeps empty last');
check(same(sortOrder(['a10', 'a2', 'B', 'b'], 1), [1, 0, 2, 3]), 'text sorts naturally, ties keep source order');
check(same(sortOrder(['12', 'x', '3'], 1), [2, 0, 1]), 'a column with any text sorts as text');

const browser = await chromium.launch();
const sheet = ['<!--sheet-->', '# Members', '', '| Member | Load |', '|---|---|', '| B2 | 10 kN |', '| A1 | 2.5 kN |', '| C3 | 7 kN |', '', 'x: 3 m'].join('\n');

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const front = async (needle = 'Members') => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('th .ck-sort').count().catch(() => 0) === 2 && (await f.locator('body').innerText().catch(() => '')).includes(needle)) return f;
            await page.waitForTimeout(500);
        }
        return null;
    };
    let frame = await front();
    check(!!frame, `${name}: each heading is a sort button`);
    if (!frame) { await ctx.close(); continue; }
    await page.waitForTimeout(800);
    const col = (f, n) => f.locator('tbody tr').evaluateAll((rows, k) => rows.map(r => r.cells[k].textContent.trim()), n);
    const press = async (label) => { const b = frame.locator('th .ck-sort', { hasText: label }); phone ? await b.tap() : await b.click(); await page.waitForTimeout(200); };
    const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());

    check(same(await col(frame, 0), ['B2', 'A1', 'C3']), `${name}: source order at first`);
    check(await frame.locator('th[aria-sort]').count() === 0, `${name}: no heading marked sorted`);
    await press('Load');
    check(same(await col(frame, 1), ['2.5 kN', '7 kN', '10 kN']), `${name}: first press sorts ascending by value`);
    check(await frame.locator('th', { hasText: 'Load' }).getAttribute('aria-sort') === 'ascending', `${name}: aria-sort ascending`);
    check((await frame.locator('th', { hasText: 'Load' }).innerText()).includes('▲'), `${name}: an up arrow on the sorted column`);
    await press('Load');
    check(same(await col(frame, 1), ['10 kN', '7 kN', '2.5 kN']), `${name}: second press sorts descending`);
    check(await frame.locator('th', { hasText: 'Load' }).getAttribute('aria-sort') === 'descending', `${name}: aria-sort descending`);
    await press('Load');
    check(same(await col(frame, 0), ['B2', 'A1', 'C3']), `${name}: third press restores the source order`);
    check(await frame.locator('th[aria-sort]').count() === 0, `${name}: and clears the mark`);
    check(await text() === sheet, `${name}: the source is unchanged`);
    check(await page.locator('.line-edit, .md-edit').count() === 0, `${name}: a press does not open the in-place editor`);

    // Sorted by Member, then the sheet changes and the preview renders again: still sorted.
    await press('Member');
    check(same(await col(frame, 0), ['A1', 'B2', 'C3']), `${name}: text column sorts ascending`);
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet.replace('x: 3 m', 'x: 4 m'));
    await page.waitForTimeout(1500);
    frame = await front('4');
    check(!!frame && same(await col(frame, 0), ['A1', 'B2', 'C3']), `${name}: the sort survives a re-render`);

    // Printing shows the source order, and the view comes back afterwards.
    if (!phone && frame) {
        await frame.evaluate(() => window.dispatchEvent(new Event('beforeprint')));
        check(same(await col(frame, 0), ['B2', 'A1', 'C3']), `${name}: print shows the source order`);
        check(await frame.locator('th[aria-sort]').count() === 0, `${name}: print shows no sort mark`);
        await frame.evaluate(() => window.dispatchEvent(new Event('afterprint')));
        check(same(await col(frame, 0), ['A1', 'B2', 'C3']), `${name}: the sort is back after printing`);
        // Keyboard: Tab reaches a heading button, Enter sorts.
        await frame.locator('th .ck-sort', { hasText: 'Load' }).focus();
        await page.keyboard.press('Enter');
        await page.waitForTimeout(200);
        check(same(await col(frame, 1), ['2.5 kN', '7 kN', '10 kN']), `${name}: Enter on a focused heading sorts`);
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} failed` : 'all passed');
process.exit(fails.length ? 1 : 0);
