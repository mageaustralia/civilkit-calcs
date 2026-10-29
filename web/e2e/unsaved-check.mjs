// The unsaved-changes marker and the warnings before work is lost: an edit marks the tab and the window title, closing the tab offers
// Save, Save clears the marker, and leaving the page with unsaved changes asks first. Desktop window and phone.
//   node e2e/unsaved-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

for (const [name, opts] of [['desktop', { viewport: { width: 1300, height: 900 } }], ['phone', { ...devices['iPhone 14'] }]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    // Stand in for the browser's save picker: record what was written instead of opening a dialog.
    await page.addInitScript(() => {
        window.__saved = [];
        window.showSaveFilePicker = async (o) => ({
            name: o?.suggestedName ?? 'x.sheet',
            createWritable: async () => ({ write: async (t) => { window.__saved.push(t); }, close: async () => {} }),
        });
    });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    const title = () => page.title();
    const tab = page.locator('.tab-strip .tab.active');

    check(!(await title()).startsWith('●'), `${name}: a fresh document has no marker in the title (${await title()})`);
    await page.evaluate(() => window.calcpadActiveEditor.setValue('<!--sheet-->\n# Beam\n\nL = 6 m\n'));
    await page.waitForTimeout(200);
    check(await tab.evaluate(e => e.classList.contains('dirty')), `${name}: an edit marks the tab as unsaved`);
    check(await tab.locator('.tab-dirty-dot').isVisible(), `${name}: the unsaved dot is visible on the tab`);
    check((await title()).startsWith('● '), `${name}: the window title starts with the marker (${await title()})`);

    // Closing the tab: Cancel keeps it, the prompt names the document.
    const tabsBefore = await page.locator('.tab-strip .tab').count();
    await tab.locator('.tab-close').click();
    await page.waitForSelector('.modal-backdrop');
    check(/Save changes to .+ before closing/.test(await page.locator('.modal-message').innerText()), `${name}: closing asks to save first`);
    await page.locator('.modal-backdrop').click({ position: { x: 5, y: 5 } });
    await page.waitForTimeout(200);
    check(await page.locator('.tab-strip .tab').count() === tabsBefore && await tab.evaluate(e => e.classList.contains('dirty')), `${name}: cancel keeps the tab and its changes`);

    // Save writes the file and clears both markers.
    await page.locator('.tab-strip .tab.active .tab-close').click();
    await page.waitForSelector('.modal-backdrop');
    await page.locator('.modal-btn.primary').click();
    await page.waitForTimeout(500);
    check(await page.evaluate(() => window.__saved.length === 1 && window.__saved[0].includes('L = 6 m')), `${name}: Save from the prompt wrote the document`);
    check(await page.locator('.tab-strip .tab', { hasText: 'Beam' }).count() === 0 || !(await title()).startsWith('●'), `${name}: after saving the tab closed without a marker`);

    // A clean document closes without asking; then leaving with unsaved work asks the browser to confirm.
    await page.evaluate(() => window.calcpadActiveEditor.setValue('<!--sheet-->\nx = 1\n'));
    await page.waitForTimeout(200);
    await page.keyboard.press('ControlOrMeta+s');
    await page.waitForTimeout(500);
    check(!(await title()).startsWith('●') && !await page.locator('.tab-strip .tab.active').evaluate(e => e.classList.contains('dirty')), `${name}: Ctrl+S clears the marker`);
    await page.evaluate(() => window.calcpadActiveEditor.setValue('<!--sheet-->\nx = 2\n'));
    await page.waitForTimeout(200);
    // Browsers only show the leave prompt after the person has interacted with the page.
    await page.locator('.monaco-editor').first().click();
    let unloadAsked = false;
    page.on('dialog', d => { if (d.type() === 'beforeunload') unloadAsked = true; void d.dismiss(); });
    await page.close({ runBeforeUnload: true });
    await new Promise(r => setTimeout(r, 1000));
    check(unloadAsked, `${name}: leaving the page with unsaved changes asks first`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
