// The prose toolbar (source editor and in-place Markdown editor) and name / function completion in a sheet,
// on a desktop window and a phone (where the floating bar stays away).   node e2e/prose-toolbar-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4983/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

const SHEET = ['<!--sheet-->', '# Beam', '', 'The load is large today.', '', 'w: 12 kN/m | load', 'L: 6 m', 'M: w * L', '', '```', 'fenced words here', '```', '', 'Closing note.'].join('\n');
// 1-based lines: 4 prose, 6-8 quantities, 11 inside a fence, 14 prose.

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    await page.waitForFunction(() => window.calcpadActiveEditor.getModel().getLanguageId() === 'calcpad-sheet');
    await page.waitForTimeout(2500); // the sheet compiler loads on demand
    const line = n => page.evaluate(i => window.calcpadActiveEditor.getModel().getLineContent(i), n);
    const select = (l, c1, c2) => page.evaluate(([l, a, b]) => { const e = window.calcpadActiveEditor; e.focus(); e.setSelection({ startLineNumber: l, startColumn: a, endLineNumber: l, endColumn: b }); }, [l, c1, c2]);
    const bar = page.locator('.editor-container .prose-toolbar:not([hidden])');

    if (phone) {
        await select(4, 5, 9);
        await page.waitForTimeout(400);
        check(await page.locator('.prose-toolbar').count() === 0, 'phone: no floating bar over the source editor');
        check(errors.length === 0, `phone: no page errors ${errors.join(' | ')}`);
        await ctx.close();
        continue;
    }

    // ---- The bar in the source editor ----
    await select(4, 5, 9); // "load"
    await bar.waitFor({ timeout: 5000 }).catch(() => {});
    check(await bar.count() === 1, 'a selection in a prose line shows the bar');
    const tips = await bar.locator('button').evaluateAll(b => b.map(x => x.title));
    for (const t of ['Bold: **text**', 'Italic: *text*', 'Code: `text`', 'Link: [text](address)', 'Bulleted list: - item', 'Numbered list: 1. item']) check(tips.includes(t), `tooltip "${t}"`);
    check(tips.some(t => /^Heading 1: # /.test(t) || /# Heading 1/.test(t)), 'the style button names the heading syntax');
    const pos = await bar.evaluate(el => { const r = el.getBoundingClientRect(); return { top: r.top, bottom: r.bottom }; });
    const selTop = await page.evaluate(() => { const e = window.calcpadActiveEditor; const p = e.getScrolledVisiblePosition(e.getSelection().getStartPosition()); return e.getDomNode().getBoundingClientRect().top + p.top; });
    check(pos.bottom <= selTop + 1, 'the bar sits above the selected line');

    await bar.locator('[data-act=bold]').click();
    check(await line(4) === 'The **load** is large today.', `bold wraps the selection: ${await line(4)}`);
    check(await page.evaluate(() => window.calcpadActiveEditor.hasTextFocus()), 'the editor keeps focus after a button press');
    check(await bar.locator('[data-act=bold].on').count() === 1, 'Bold shows as pressed on bold text');
    await bar.locator('[data-act=bold]').click();
    check(await line(4) === 'The load is large today.', `bold on bold text unwraps: ${await line(4)}`);
    await bar.locator('[data-act=italic]').click();
    check(await line(4) === 'The *load* is large today.', `italic: ${await line(4)}`);
    await page.evaluate(() => window.calcpadActiveEditor.trigger('t', 'undo', null));
    check(await line(4) === 'The load is large today.', 'one undo reverts one button press');
    await bar.locator('[data-act=code]').click();
    check(await line(4) === 'The `load` is large today.', `code: ${await line(4)}`);
    await page.evaluate(() => window.calcpadActiveEditor.trigger('t', 'undo', null));
    await select(4, 5, 9);
    await bar.locator('[data-act=link]').click();
    check(await line(4) === 'The [load](url) is large today.', `link: ${await line(4)}`);
    check(await page.evaluate(() => { const e = window.calcpadActiveEditor; return e.getModel().getValueInRange(e.getSelection()); }) === 'url', 'the url placeholder is selected');
    await page.evaluate(() => window.calcpadActiveEditor.trigger('t', 'undo', null));
    await select(4, 5, 9);
    await bar.locator('.pt-style').click();
    await bar.locator('[data-act=h2]').click();
    check(await line(4) === '## The load is large today.', `Heading 2: ${await line(4)}`);
    await bar.locator('.pt-style').click();
    await bar.locator('[data-act=normal]').click();
    check(await line(4) === 'The load is large today.', 'Normal removes the heading');
    await bar.locator('[data-act=bullet]').click();
    check(await line(4) === '- The load is large today.', `bulleted list: ${await line(4)}`);
    await bar.locator('[data-act=number]').click();
    check(await line(4) === '1. The load is large today.', `numbered list: ${await line(4)}`);
    await bar.locator('[data-act=number]').click();
    check(await line(4) === 'The load is large today.', 'the numbered list toggles off');

    // Not on math lines or fenced text.
    await select(6, 1, 2);
    await page.waitForTimeout(300);
    check(await bar.count() === 0, 'no bar on a quantity line');
    await select(11, 1, 7);
    await page.waitForTimeout(300);
    check(await bar.count() === 0, 'no bar inside a code fence');
    await select(14, 1, 8);
    await page.waitForTimeout(300);
    check(await bar.count() === 1, 'a later prose line shows it again');

    // ---- Completion ----
    const widget = page.locator('.monaco-editor .suggest-widget.visible');
    const typeAt = async (l, text) => {
        await page.evaluate(([l]) => { const e = window.calcpadActiveEditor; e.focus(); const m = e.getModel(); e.setPosition({ lineNumber: l, column: m.getLineMaxColumn(l) }); }, [l]);
        await page.keyboard.type(text, { delay: 60 });
        await page.waitForTimeout(700);
    };
    const labels = async () => (await widget.locator('.monaco-list-row .label-name').allInnerTexts()).map(s => s.trim());
    await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue(e.getValue() + '\n\nN: 2 * '); });
    await page.waitForTimeout(800);
    const last = await page.evaluate(() => window.calcpadActiveEditor.getModel().getLineCount());
    await typeAt(last, 'w');
    let got = await labels();
    check(got.includes('w'), `typing a letter in a math line offers a name defined above (${got.slice(0, 6)})`);
    await page.keyboard.press('Escape');
    await page.keyboard.press('Backspace');
    await page.keyboard.type('ma', { delay: 60 });
    await page.waitForTimeout(700);
    got = await labels();
    check(got.includes('max'), `built-in function names are offered (${got.slice(0, 6)})`);
    await page.keyboard.press('Escape');
    // A name is not offered before its definition, and nothing pops up in prose.
    await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue(e.getValue().split('\n').slice(0, 14).join('\n') + '\nQ: 1 m | later\nA sentence about the load w and max values'); });
    await page.waitForTimeout(800);
    const lastP = await page.evaluate(() => window.calcpadActiveEditor.getModel().getLineCount());
    await typeAt(lastP, ' ma');
    check(await widget.count() === 0, `no completion popup while typing prose ${await labels()}`);
    await page.keyboard.press('Escape');

    // ---- The bar in the in-place editor ----
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), SHEET);
    await page.waitForTimeout(1500);
    let frame = null;
    for (let i = 0; i < 60 && !frame; i++) {
        const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
        const f = h ? await h.contentFrame() : null;
        if (f && /large today/.test(await f.locator('body').innerText().catch(() => ''))) frame = f; else await page.waitForTimeout(500);
    }
    check(!!frame, 'the preview shows the sheet');
    if (frame) {
        await frame.locator('[data-source-line]', { hasText: 'large today' }).first().click();
        await page.waitForSelector('.md-block .ProseMirror', { timeout: 15000 });
        const pm = page.locator('.md-block .ProseMirror');
        await pm.locator('p').first().dblclick({ position: { x: 10, y: 6 } }); // selects a word
        await page.waitForTimeout(500);
        const mb = page.locator('.md-block .prose-toolbar:not([hidden])');
        check(await mb.count() === 1, 'a selection in the in-place editor shows the bar');
        const mt = await mb.locator('button').evaluateAll(b => b.map(x => x.title));
        check(mt.includes('Bold: **text**') && mt.includes('Numbered list: 1. item'), 'the in-place bar has the same tooltips');
        await mb.locator('[data-act=bold]').click();
        check(await pm.locator('strong').count() === 1, 'Bold makes the selection strong');
        await page.waitForTimeout(150);
        check(await mb.locator('[data-act=bold].on').count() === 1, 'Bold shows as pressed');
        await mb.locator('[data-act=bold]').click();
        check(await pm.locator('strong').count() === 0, 'Bold again removes it');
        await mb.locator('[data-act=bullet]').click();
        check(await pm.locator('ul li').count() === 1, `Bulleted list ${await pm.innerHTML()}`);
        await mb.locator('[data-act=bullet]').click();
        check(await pm.locator('ul').count() === 0, 'the bulleted list toggles off');
        await mb.locator('.pt-style').click();
        await mb.locator('[data-act=h2]').click();
        check(await pm.locator('h2').count() === 1, 'Heading 2');
        await page.keyboard.press('Escape');
    }
    check(errors.length === 0, `desktop: no page errors ${errors.join(' | ')}`);
    await ctx.close();
}
await browser.close();
if (fails.length) { console.log(`\n${fails.length} failed`); process.exit(1); }
console.log('\nall passed');
