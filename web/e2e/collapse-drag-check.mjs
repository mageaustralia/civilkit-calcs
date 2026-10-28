// Folding a heading's section in the sheet preview, and dragging a block grip to reorder blocks (a heading takes its section
// along), on a desktop window and on a phone.   node e2e/collapse-drag-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const SHEET = ['<!--sheet-->', '# Alpha', '', 'a: 1 m | alpha one', '', '## Sub', '', 'b: 2 m | sub one', '', '# Beta', '', 'c: 3 m | beta one', '', '# Alpha', '', 'd: 4 m | alpha again'].join('\n');

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    if (arg('--only', name) !== name) continue;
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const front = async (need = 'Beta') => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('.ck-grip').count().catch(() => 0) >= 3 && (await f.locator('body').innerText().catch(() => '')).includes(need)) return f;
            await page.waitForTimeout(400);
        }
        return null;
    };
    let frame = await front();
    check(!!frame, `${name}: the preview shows the sheet`);
    if (!frame) { await ctx.close(); continue; }
    await page.waitForTimeout(1500);
    const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
    const idx = async (needle, nth = 0) => (frame = await front()).locator('[data-source-line]').evaluateAll((els, [n, k]) => { const m = els.filter(e => e.textContent.includes(n)); return els.indexOf(m[k]); }, [needle, nth]);
    const block = async (needle, nth = 0) => (await front()).locator('[data-source-line]').nth(await idx(needle, nth));
    const shown = (needle, nth = 0) => block(needle, nth).then(b => b.evaluate(e => getComputedStyle(e).display !== 'none'));
    // Brings the grip (and chevron) of a block up: hover with a mouse, tap on a phone.
    const reveal = async (needle, nth = 0) => {
        const target = await block(needle, nth);
        await target.scrollIntoViewIfNeeded();
        if (phone) await target.tap();
        else { const bb = await target.boundingBox(); await page.mouse.move(bb.x + bb.width / 2 - 30, bb.y + bb.height / 2 + 1); await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 }); }
        await page.waitForTimeout(250);
        return front();
    };
    const press = async (loc) => phone ? loc.tap() : loc.click();

    // --- Collapse ---
    frame = await reveal('Alpha');
    const chev = frame.locator('.ck-chev.on');
    check(await chev.count() === 1, `${name}: a heading shows its chevron beside the grip`);
    await press(chev);
    await page.waitForTimeout(500);
    frame = await front();
    check(!(await shown('alpha one')) && !(await shown('Sub')) && !(await shown('sub one')), `${name}: collapsing hides the blocks up to the next heading of the same level`);
    check(await shown('beta one') && await shown('Beta'), `${name}: later sections stay`);
    check(await shown('alpha again', 0) && await shown('Alpha', 1), `${name}: a second heading with the same text is its own section`);
    const note = await (await block('Alpha')).evaluate(e => getComputedStyle(e, '::after').content);
    check(/3 hidden/.test(note), `${name}: the collapsed heading says how many blocks are hidden (${note})`);
    check(await frame.locator('.ck-chev.col').count() === 1, `${name}: the collapsed chevron stays visible`);
    check((await text()) === SHEET, `${name}: the sheet text is untouched`);
    // Print and export show everything.
    await page.emulateMedia({ media: 'print' });
    check(await shown('alpha one') && await shown('sub one'), `${name}: print shows the folded blocks`);
    await page.emulateMedia({ media: 'screen' });
    // A re-render (an edit elsewhere) keeps the fold.
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET + '\n\ne: 5 m | epsilon');
    frame = await front('epsilon');
    await page.waitForTimeout(1200);
    frame = await front('epsilon');
    check(!(await shown('alpha one')) && await frame.locator('.ck-col').count() === 1, `${name}: the fold survives the preview re-render`);
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    frame = await front();
    await page.waitForTimeout(1200);
    // Expand again with the chevron.
    frame = await reveal('Alpha');
    await press(frame.locator('.ck-chev.on'));
    await page.waitForTimeout(500);
    check(await shown('alpha one') && await shown('sub one'), `${name}: the chevron expands the section`);
    // Collapse all / Expand all from the heading's menu.
    frame = await reveal('Beta');
    await press(frame.locator('.ck-grip.on'));
    await page.waitForSelector('.block-menu');
    check(await page.locator('.block-menu button').count() === 6, `${name}: a heading's menu has Collapse all and Expand all`);
    await press(page.locator('.block-menu button', { hasText: 'Collapse all' }));
    await page.waitForTimeout(600);
    frame = await front();
    check(await frame.locator('.ck-col').count() === 3 && !(await shown('Sub')), `${name}: Collapse all folds every heading`);
    frame = await reveal('Beta');
    await press(frame.locator('.ck-grip.on'));
    await page.waitForSelector('.block-menu');
    await press(page.locator('.block-menu button', { hasText: 'Expand all' }));
    await page.waitForTimeout(600);
    frame = await front();
    check(await frame.locator('.ck-col').count() === 0 && await shown('sub one'), `${name}: Expand all opens them again`);

    // --- Drag ---
    const touch = phone ? await ctx.newCDPSession(page) : null;
    const drag = async (needle, nth, toNeedle, toNth, where, hold = false) => {
        frame = await reveal(needle, nth);
        const g = await frame.locator('.ck-grip.on').boundingBox();
        const tb = await (await block(toNeedle, toNth)).boundingBox();
        const x0 = g.x + g.width / 2, y0 = g.y + g.height / 2;
        const y1 = where === 'above' ? tb.y - 3 : tb.y + tb.height + 3;
        if (phone) {
            const pt = (x, y) => [{ x, y, id: 1 }];
            await touch.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: pt(x0, y0) });
            for (let i = 1; i <= 8; i++) await touch.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: pt(x0, y0 + (y1 - y0) * i / 8) });
            await page.waitForTimeout(100);
            const bar = await (await front()).locator('.ck-drop').evaluate(e => e.style.display).catch(() => 'none');
            if (!hold) await touch.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
            await page.waitForTimeout(500);
            return bar;
        }
        await page.mouse.move(x0, y0);
        await page.mouse.down();
        await page.mouse.move(x0, y0 + 2, { steps: 2 });
        await page.mouse.move(x0 + 1, y1, { steps: 10 });
        const bar = await (await front()).locator('.ck-drop').evaluate(e => e.style.display).catch(() => 'none');
        if (!hold) await page.mouse.up();
        await page.waitForTimeout(500);
        return bar;
    };
    const before = await text();
    // Beta's section (one heading and one block) goes above the first Alpha.
    const bar = await drag('Beta', 0, 'Alpha', 0, 'above');
    check(bar === 'block', `${name}: a drop line shows while dragging`);
    const moved = await text();
    check(moved.startsWith('<!--sheet-->\n# Beta\n\nc: 3 m | beta one\n\n# Alpha\n'), `${name}: dragging a heading moves its section to the drop place`);
    check(moved.split('\n').length === before.split('\n').length && moved.includes('## Sub'), `${name}: nothing is lost`);
    check(await page.locator('.block-toast').count() === 1, `${name}: the Undo toast shows`);
    await press(page.locator('.block-toast button'));
    await page.waitForTimeout(500);
    check(await text() === before, `${name}: one Undo puts the section back`);
    await page.waitForTimeout(1200);
    // The first Alpha section (with Sub) goes to the very end.
    await drag('Alpha', 0, 'alpha again', 0, 'below');
    check((await text()).replace('<!--sheet-->\n', '') === ['# Beta', '', 'c: 3 m | beta one', '', '# Alpha', '', 'd: 4 m | alpha again', '', '# Alpha', '', 'a: 1 m | alpha one', '', '## Sub', '', 'b: 2 m | sub one'].join('\n'), `${name}: a heading dropped at the end carries its sub-sections`);
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    await page.waitForTimeout(2800);
    // A plain block moves alone.
    await drag('beta one', 0, 'alpha one', 0, 'above');
    check((await text()).includes('# Alpha\n\nc: 3 m | beta one\n\na: 1 m | alpha one'), `${name}: a plain block moves alone`);
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    await page.waitForTimeout(2800);
    // Dropping a section inside itself does nothing.
    await drag('Alpha', 0, 'sub one', 0, 'above');
    check(await text() === SHEET, `${name}: a section dropped inside itself stays put`);
    await page.waitForTimeout(800);
    // A small movement is still a click: the menu opens.
    frame = await reveal('beta one');
    const g = await frame.locator('.ck-grip.on').boundingBox();
    if (phone) await frame.locator('.ck-grip.on').tap();
    else { await page.mouse.move(g.x + 5, g.y + 5); await page.mouse.down(); await page.mouse.move(g.x + 8, g.y + 7); await page.mouse.up(); }
    await page.waitForTimeout(400);
    check(await page.locator('.block-menu').count() === 1, `${name}: a click (under 6px of movement) still opens the menu`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
