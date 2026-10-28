// Convert selection to formula in the running app: the prose toolbar's "= Formula" button, the shortcut and the
// context menu, typing a name renames both copies, one undo step, a Calcpad worksheet, and the button under the
// selection on a phone.   node e2e/to-formula-ui-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

const SHEET = ['<!--sheet-->', '# Beam', '', 'w: 12 kN/m', 'L: 6 m', '', 'The moment is w*L^2/8 at midspan.', '', 'M: w*L^2/8 -> kN*m'].join('\n');
const CPD = ["'Beam", 'w = 12kN/m', 'L = 6m', "'The moment is w*L^2/8 at midspan", 'M = w*L^2/8|kN*m'].join('\n');

const open = async (page) => {
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
};
const setText = (page, text) => page.evaluate(t => { const e = window.calcpadActiveEditor; e.setValue(t); e.focus(); }, text);
/** Selects the nth `needle` in the editor (1 based lines). */
const select = (page, needle, nth = 0) => page.evaluate(([n, k]) => {
    const e = window.calcpadActiveEditor;
    const lines = e.getModel().getLinesContent();
    let line = -1;
    for (let i = 0, seen = 0; i < lines.length; i++) if (lines[i].includes(n) && seen++ === k) { line = i; break; }
    const c = lines[line].indexOf(n) + 1;
    e.focus();
    e.setSelection({ startLineNumber: line + 1, startColumn: c, endLineNumber: line + 1, endColumn: c + n.length });
}, [needle, nth]);
const value = page => page.evaluate(() => window.calcpadActiveEditor.getValue().split(/\r?\n/));
const selected = page => page.evaluate(() => { const e = window.calcpadActiveEditor; return e.getSelections().map(s => e.getModel().getValueInRange(s)); });

// ---- Desktop ----
{
    const ctx = await browser.newContext({ viewport: { width: 1300, height: 900 } });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await open(page);
    await setText(page, SHEET);
    await page.waitForTimeout(800);

    // In prose: the toolbar has the button, and it puts a named line after the paragraph.
    await select(page, 'w*L^2/8');
    const btn = page.locator('.prose-toolbar .pt-formula');
    await btn.waitFor({ state: 'visible', timeout: 5000 });
    check(/Ctrl\+Shift\+E/.test(await btn.getAttribute('title')), 'desktop: the toolbar button says its shortcut');
    await btn.click();
    let v = await value(page);
    check(v[6] === 'The moment is w*L^2/8 at midspan.' && v[7] === 'res1: w*L^2/8' && v.length === 10, `desktop: prose selection becomes a named line after the paragraph (${v[7]})`);
    check(JSON.stringify(await selected(page)) === '["res1"]', 'desktop: the new name is selected');
    await page.keyboard.type('Mmid');
    v = await value(page);
    check(v[7] === 'Mmid: w*L^2/8', 'desktop: typing names it');

    // Undo takes back the typing, then the conversion in one step.
    await page.keyboard.press('ControlOrMeta+z');
    await page.keyboard.press('ControlOrMeta+z');
    v = await value(page);
    check(v.join('\n') === SHEET, 'desktop: undo restores the sheet in one step after the typing');

    // In a calculation, by the shortcut: pulled out above, both names selected, typing renames both.
    await select(page, 'w*L^2/8', 1);
    await page.keyboard.press('ControlOrMeta+Shift+E');
    v = await value(page);
    check(v[8] === 'res1: w*L^2/8' && v[9] === 'M: res1 -> kN*m', `desktop: Ctrl+Shift+E pulls the expression out of a calculation (${v[8]} / ${v[9]})`);
    check(JSON.stringify(await selected(page)) === '["res1","res1"]', 'desktop: both copies of the name are selected');
    await page.keyboard.type('M0');
    v = await value(page);
    check(v[8] === 'M0: w*L^2/8' && v[9] === 'M: M0 -> kN*m', 'desktop: typing renames both');
    // The preview calculates it.
    // The preview frame is swapped on each render, so it is found again each time.
    let shown = '';
    for (let i = 0; i < 60 && !(/M\s*0/.test(shown) && /54/.test(shown)); i++) {
        await page.waitForTimeout(500);
        shown = await page.locator('iframe.preview-frame:not(.preview-frame-back)').contentFrame().locator('body').innerText().catch(() => '');
    }
    check(/M\s*0/.test(shown) && /54/.test(shown), 'desktop: the preview shows the new formula and its value');

    // The context menu offers it on a selection.
    await setText(page, SHEET);
    await select(page, 'w*L^2/8');
    const pos = await page.evaluate(() => { const e = window.calcpadActiveEditor; const p = e.getScrolledVisiblePosition(e.getSelection().getStartPosition()); const r = e.getDomNode().getBoundingClientRect(); return { x: r.left + p.left + 20, y: r.top + p.top + 8 }; });
    await page.mouse.click(pos.x, pos.y, { button: 'right' });
    const item = page.locator('.monaco-menu .action-item', { hasText: 'Convert Selection to Formula' });
    await item.first().waitFor({ timeout: 5000 }).then(() => check(true, 'desktop: the context menu lists Convert Selection to Formula'), () => check(false, 'desktop: the context menu lists Convert Selection to Formula'));
    await page.keyboard.press('Escape');

    // Nothing to convert: a message, no edit.
    await select(page, '# Beam');
    await page.keyboard.press('ControlOrMeta+Shift+E');
    check((await value(page)).join('\n') === SHEET, 'desktop: a selection that is not an expression changes nothing');

    // A Calcpad worksheet: text gives a plain calculation line, a calculation a named line above.
    await setText(page, CPD);
    await page.waitForTimeout(500);
    await select(page, 'w*L^2/8');
    await page.keyboard.press('ControlOrMeta+Shift+E');
    v = await value(page);
    check(v[3] === "'The moment is w*L^2/8 at midspan" && v[4] === 'w*L^2/8', `cpd: text becomes a plain calculation line (${v[4]})`);
    await select(page, 'w*L^2/8', 2);
    await page.keyboard.press('ControlOrMeta+Shift+E');
    v = await value(page);
    check(v[5] === 'res1 = w*L^2/8' && v[6] === 'M = res1|kN*m', `cpd: a calculation is pulled out to a named line (${v[5]} / ${v[6]})`);
    check(errors.length === 0, `desktop: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// ---- Phone ----
{
    const ctx = await browser.newContext({ ...devices['iPhone 14'] });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await open(page);
    await setText(page, SHEET);
    await page.waitForTimeout(800);
    const chip = page.locator('.to-formula-chip');
    check(await chip.isHidden(), 'phone: no button without a selection');
    await select(page, 'w*L^2/8');
    await chip.waitFor({ state: 'visible', timeout: 5000 }).then(() => check(true, 'phone: a button under the selection'), () => check(false, 'phone: a button under the selection'));
    check(await page.locator('.prose-toolbar:not([hidden])').count() === 0, 'phone: no prose toolbar');
    await chip.tap();
    const v = await value(page);
    check(v[7] === 'res1: w*L^2/8', `phone: tapping it converts (${v[7]})`);
    await page.waitForTimeout(200);
    await select(page, '# Beam');
    check(await chip.isHidden(), 'phone: no button for a selection that is not an expression');
    check(errors.length === 0, `phone: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
