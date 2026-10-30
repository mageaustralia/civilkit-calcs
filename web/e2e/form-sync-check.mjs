// Input form values and the defaults in the source: "Update defaults" writes the entered values into their lines, "Reset"
// drops them, a mark on the line shows where the form and the source disagree, and the "Keep Input Form Values in the
// Source" setting writes each value as it is entered.   node e2e/form-sync-check.mjs [--url http://127.0.0.1:5192/]
import { chromium } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5192/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const SHEET = `<!--sheet-->
# Beam

L: ? 6 m | span
E: ? [Steel = 200 GPa, *Timber = 11 GPa] | material
k: ? (Low = 1, High = 2) | class
ribs: ? on | ribs
M: L * 2
`;

const browser = await chromium.launch();
const run = async (title, sync, body) => {
    const ctx = await browser.newContext({ viewport: { width: 1360, height: 860 } });
    if (sync) await ctx.addInitScript(() => { try { localStorage.setItem('calcpad-sync-form-to-source', 'true'); } catch {} });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!/^(Canceled|Event)/.test(String(e.message))) errors.push(String(e.message)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t), SHEET);
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
    const front = async () => {
        for (let i = 0; i < 80; i++) {
            const h = await page.$('iframe.preview-frame:not(.preview-frame-back)');
            const f = h && await h.contentFrame();
            if (f && await f.$('input[data-ui-var^="L:"]')) return f;
            await page.waitForTimeout(250);
        }
        return null;
    };
    const text = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
    const ghosts = () => page.evaluate(() => {
        const m = window.calcpadActiveEditor.getModel();
        return m.getAllDecorations().filter((d) => d.options.after?.inlineClassName === 'form-value-ghost').map((d) => d.range.startLineNumber);
    });
    const type = async (selector, value) => {
        const f = await front();
        await f.evaluate(([s, v]) => { const n = document.querySelector(s); n.value = v; n.dispatchEvent(new Event('change', { bubbles: true })); }, [selector, value]);
        await page.waitForTimeout(1500);
    };
    await front();
    console.log(`-- ${title}`);
    await body({ page, front, text, ghosts, type, check });
    check(errors.length === 0, `${title}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
};

await run('update defaults', false, async ({ page, front, text, ghosts, type }) => {
    check((await text()).includes('L: ? 6 m | span'), 'the sheet starts with the default 6');
    await type('input[data-ui-var^="L:"]', '8');
    const f = await front();
    await f.evaluate(() => { const s = document.querySelector('select[data-ui-var^="E:"]'); s.selectedIndex = 0; s.dispatchEvent(new Event('change', { bubbles: true })); });
    await page.waitForTimeout(1500);
    await (await front()).evaluate(() => [...document.querySelectorAll('input[type=radio]')].pop().click());
    await page.waitForTimeout(1500);
    await (await front()).evaluate(() => document.querySelector('input[type=checkbox]').click());
    await page.waitForTimeout(1500);
    check((await text()).includes('L: ? 6 m | span'), 'typing in the form does not change the source');
    const g = await ghosts();
    check(g.length === 4, `a mark on each of the four lines that differ (${g.join(',')})`);
    const upd = page.locator('button', { hasText: 'Update defaults' });
    check(await upd.isVisible(), 'the Update defaults button is shown');
    await upd.click();
    await page.waitForTimeout(1500);
    const t = await text();
    check(t.includes('L: ? 8 m | span'), 'the number is written into its line');
    check(t.includes('E: ? [*Steel = 200 GPa, Timber = 11 GPa]'), 'the drop-down star moves');
    check(t.includes('k: ? (Low = 1, *High = 2)'), 'the radio star moves');
    check(t.includes('ribs: ? off'), 'the checkbox is written');
    check(!t.includes('uiOverrides'), 'no uiOverrides comment is left');
    check((await ghosts()).length === 0, 'the marks are gone');
    check(!(await upd.isVisible().catch(() => false)), 'the Update defaults button is gone');
    const shown = await (await front()).evaluate(() => document.querySelector('input[data-ui-var^="L:"]').value);
    check(shown === '8', `the form shows the new default (${shown})`);
});

await run('reset', false, async ({ page, front, text, ghosts, type }) => {
    await type('input[data-ui-var^="L:"]', '9');
    check((await ghosts()).length === 1, 'a mark on the changed line');
    await page.locator('button', { hasText: /^Reset$/ }).click();
    await page.waitForTimeout(1800);
    check((await text()).includes('L: ? 6 m | span'), 'the source is as it was');
    check((await ghosts()).length === 0, 'the mark is gone');
    const shown = await (await front()).evaluate(() => document.querySelector('input[data-ui-var^="L:"]').value);
    check(shown === '6', `the form is back to the default (${shown})`);
});

await run('saved values', false, async ({ page, text, ghosts, type }) => {
    await type('input[data-ui-var^="L:"]', '7');
    await page.locator('button', { hasText: 'Save values' }).click();
    await page.waitForTimeout(1000);
    check((await text()).includes('"uiOverrides"'), 'Save values writes the comment');
    check((await ghosts()).length === 1, 'the line is still marked after saving');
    await page.locator('button', { hasText: 'Exit input mode' }).click();
    await page.waitForTimeout(1500);
    const shownMark = await page.locator('.monaco-editor .form-value-ghost').first().innerText().catch(() => '');
    check(/form:\s*7/.test(shownMark), `back in the editor the line carries the mark (${shownMark.trim()})`);
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
    await page.waitForTimeout(2500);
    await page.locator('button', { hasText: 'Update defaults' }).click();
    await page.waitForTimeout(1500);
    const t = await text();
    check(t.includes('L: ? 7 m | span') && !t.includes('uiOverrides'), 'Update defaults writes the line and removes the saved comment');
    check((await ghosts()).length === 0, 'no mark left');
});

await run('keep in step', true, async ({ page, text, ghosts, type }) => {
    await type('input[data-ui-var^="L:"]', '9');
    check((await text()).includes('L: ? 9 m | span'), 'with the setting on, the value goes straight into the source');
    check((await ghosts()).length === 0, 'so there is nothing to mark');
    check(!(await page.locator('button', { hasText: 'Save values' }).isVisible().catch(() => false)), 'and nothing to save');
    check(!(await text()).includes('uiOverrides'), 'no comment is written');
});

// A sheet with a slider opens with the docked panel; a value typed there follows the same rules.
{
    const ctx = await browser.newContext({ viewport: { width: 1360, height: 860 } });
    await ctx.addInitScript(() => { try { localStorage.setItem('calcpad-sync-form-to-source', 'true'); } catch {} });
    const page = await ctx.newPage();
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t), '<!--sheet-->\n# Dock\n\nw: ? 12 kN/m [2 .. 30 step 0.5] | load\nM: w * 2\n');
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
    const box = page.locator('.input-dock .ce-field[data-name^="w:"] .ce-box');
    await box.waitFor({ timeout: 60000 });
    console.log('-- docked slider');
    await box.fill('20');
    await box.press('Enter');
    await page.waitForTimeout(1500);
    const t = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    check(t.includes('w: ? 20 kN/m [2 .. 30 step 0.5] | load'), 'a value typed in the dock goes into the line and keeps the slider bounds');
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
