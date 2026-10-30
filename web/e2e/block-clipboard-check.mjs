// Cut, copy, paste below, disable / enable from the block grip menu, Paste in the "+" menu, and Ctrl/Cmd+Shift+K to delete the
// block at the caret.   node e2e/block-clipboard-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

// The desktop window may read and write the system clipboard; the phone context may not, so Paste falls back to the remembered block.
for (const [name, opts, phone] of [
    ['desktop', { viewport: { width: 1300, height: 900 }, permissions: ['clipboard-read', 'clipboard-write'] }, false],
    ['phone', { ...devices['iPhone 14'] }, true],
]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    const sheet = ['<!--sheet-->', '# Beam', '', 'a: 1 m | first', '', 'b: 2 m | second', '', 'c: 3 m | third'].join('\n');
    const setSheet = async (t = sheet) => { await page.evaluate(x => window.calcpadActiveEditor.setValue(x), t); await page.waitForTimeout(800); };
    await setSheet();
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const front = async (need = /Beam/) => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('.ck-grip').count().catch(() => 0) >= 3 && need.test(await f.locator('body').innerText().catch(() => ''))) return f;
            await page.waitForTimeout(500);
        }
        return null;
    };
    let frame = await front();
    check(!!frame, `${name}: the preview shows the sheet`);
    if (!frame) { await ctx.close(); continue; }
    await page.waitForTimeout(1500);
    const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
    const sys = () => page.evaluate(() => navigator.clipboard.readText().catch(() => null));
    const open = async (needle) => {
        frame = await front();
        const n = await frame.locator('[data-source-line]').evaluateAll((els, k) => els.findIndex(e => e.textContent.includes(k)), needle);
        const target = frame.locator('[data-source-line]').nth(n);
        await target.scrollIntoViewIfNeeded();
        if (phone) await target.tap(); else { const bb = await target.boundingBox(); await page.mouse.move(bb.x + bb.width / 2 - 30, bb.y + bb.height / 2 + 1); await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 }); }
        await page.waitForTimeout(250);
        const on = frame.locator('.ck-grip.on');
        phone ? await on.tap() : await on.click();
        await page.waitForSelector('.block-menu');
    };
    const act = async (label) => { const b = page.locator('.block-menu button', { hasText: label }); phone ? await b.tap() : await b.click(); await page.waitForTimeout(600); };
    const undo = async () => { phone ? await page.locator('.block-toast button').tap() : await page.locator('.block-toast button').click(); await page.waitForTimeout(500); };
    const labels = () => page.locator('.block-menu button').allInnerTexts();

    // Menu: the three clipboard actions and Disable are there.
    await open('second');
    const l = (await labels()).join('|');
    check(/Cut/.test(l) && /Copy/.test(l) && /Paste below/.test(l) && /Disable/.test(l), `${name}: the menu lists Cut, Copy, Paste below and Disable (${l.replace(/\s+/g, ' ')})`);
    await act('Copy');
    check(await page.locator('.block-toast').count() === 1 && await page.locator('.block-toast button').count() === 0, `${name}: Copy shows a toast with nothing to undo`);
    check((await text()) === sheet, `${name}: Copy leaves the sheet alone`);
    if (!phone) check(await sys() === 'b: 2 m | second', `${name}: Copy puts the block source on the system clipboard`);

    // Paste below the third block: one blank line between, the sheet changes once.
    await page.waitForTimeout(1500);
    await open('third');
    await act('Paste below');
    const pasted = await text();
    check(pasted === sheet + '\n\nb: 2 m | second', `${name}: Paste below adds the copied block after the block`);
    await undo();
    check((await text()) === sheet, `${name}: one Undo takes the paste back`);
    await page.waitForTimeout(1500);

    // Cut the first block: gone from the sheet, on the clipboard, Undo restores it.
    await open('first');
    await act('Cut');
    const cut = await text();
    check(!cut.includes('first') && cut.includes('second'), `${name}: Cut removes the block`);
    check(await page.locator('.block-toast button').count() === 1, `${name}: Cut offers Undo`);
    if (!phone) check(await sys() === 'a: 1 m | first', `${name}: Cut puts the block on the system clipboard`);
    await undo();
    check((await text()) === sheet, `${name}: Undo restores the cut block exactly`);
    await page.waitForTimeout(1500);

    // The "+" menu offers Paste once something is copied, and pastes the clipboard.
    if (!phone) await page.evaluate(() => navigator.clipboard.writeText('z: 9 m | pasted'));
    frame = await front();
    let btn = frame.locator('.ck-ins-btn').nth(1);
    await btn.scrollIntoViewIfNeeded();
    if (phone) { await frame.locator('[data-source-line]').nth(2).tap(); btn = frame.locator('.ck-ins.on .ck-ins-btn').first(); }
    else { const bb = await frame.locator('.ck-ins').nth(1).boundingBox(); await page.mouse.move(bb.x + bb.width / 2 + 200, bb.y + bb.height / 2); await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 3 }); }
    if (phone) await btn.tap();
    else {
        // The preview swaps its frames after a render: look the strip up again and retry rather than click a replaced element.
        for (let i = 0; i < 4; i++) {
            try {
                frame = await front();
                const bb = await frame.locator('.ck-ins').nth(1).boundingBox();
                await page.mouse.move(bb.x + bb.width / 2 + 200, bb.y + bb.height / 2);
                await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 });
                await frame.locator('.ck-ins-btn').nth(1).click({ timeout: 4000 });
                break;
            } catch (e) { if (i === 3) throw e; await page.waitForTimeout(800); }
        }
    }
    await page.waitForSelector('.insert-menu');
    await page.waitForTimeout(400);
    check(await page.locator('.insert-menu button', { hasText: 'Paste' }).count() === 1, `${name}: the "+" menu offers Paste`);
    await page.locator('.insert-menu button', { hasText: 'Paste' }).click();
    const viaPlus = await text();
    const want = phone ? 'a: 1 m | first' : 'z: 9 m | pasted'; // phone: the remembered cut block
    check(viaPlus.includes('\n' + want + '\n\n') || viaPlus.includes(want), `${name}: Paste from "+" inserts ${phone ? 'the remembered block' : 'the system clipboard text'}`);
    check(viaPlus.split('\n').length === sheet.split('\n').length + 2 + (want.split('\n').length - 1), `${name}: Paste from "+" adds the text and one blank line`);
    await setSheet();
    await page.waitForTimeout(1500);

    // Disable: the block is commented out line by line, leaves the preview, and Undo / the Enable shortcut bring it back.
    await open('second');
    await act('Disable');
    const disabled = await text();
    check(disabled === sheet.replace('b: 2 m | second', '<!-- b: 2 m | second -->'), `${name}: Disable comments the block out`);
    check(/hidden from the preview/.test(await page.locator('.block-toast').innerText()), `${name}: the toast says the block is hidden`);
    await page.waitForTimeout(1800);
    check(!/second/.test(await (await front(/Beam/)).locator('body').innerText()), `${name}: a disabled block is not in the preview`);
    await undo();
    check((await text()) === sheet, `${name}: Undo re-enables it`);
    await page.waitForTimeout(500);
    // Disable a fenced block with a blank line inside, then Enable from the keyboard.
    const fenced = ['<!--sheet-->', '# Beam', '', 'a: 1 m | first', '', '```', 'x = 1', '', 'print(x)', '```', '', 'c: 3 m | third'].join('\n');
    await setSheet(fenced);
    await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 7, column: 1 }); });
    if (!phone) {
        await page.waitForTimeout(1800);
        await page.keyboard.press('ControlOrMeta+Shift+/');
        await page.waitForTimeout(500);
        const off = await text();
        check(off.split('\n').slice(5, 10).join('|') === '<!-- ``` -->|<!-- x = 1 -->|<!-- -->|<!-- print(x) -->|<!-- ``` -->', `${name}: the shortcut disables a fenced block, blank line included`);
        await page.evaluate(() => window.calcpadActiveEditor.setPosition({ lineNumber: 8, column: 1 }));
        await page.keyboard.press('ControlOrMeta+Shift+/');
        await page.waitForTimeout(500);
        check((await text()) === fenced, `${name}: the same shortcut enables it again, exactly`);
    }

    // Ctrl/Cmd+Shift+K deletes the block at the caret, with the Undo toast.
    if (!phone) {
        await setSheet();
        await page.waitForTimeout(1800);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 6, column: 3 }); });
        await page.keyboard.press('ControlOrMeta+Shift+K');
        await page.waitForTimeout(500);
        const k = await text();
        check(k === sheet.replace('b: 2 m | second\n\n', ''), `${name}: Ctrl/Cmd+Shift+K deletes the block at the caret${k === sheet ? ' (nothing happened)' : ''}`);
        check(await page.locator('.block-toast button').count() === 1, `${name}: ... and offers Undo`);
        await undo();
        check((await text()) === sheet, `${name}: Undo restores it`);
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
