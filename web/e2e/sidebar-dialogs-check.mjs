// The sidebar has three tabs (Insert, TOC, Variables); Settings, Export, Errors and Prettify open as dialogs from the ribbon.
//   node e2e/sidebar-dialogs-check.mjs [--url U] [--shots DIR]
import { chromium, devices } from 'playwright';
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const shots = arg('--shots', '');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Insert' }).tap();
    const tabs = await page.locator('.calcpad-vue-ui .tab-container .tab').allInnerTexts();
    check(tabs.join(',') === 'Insert,TOC,Variables', `${name}: the sidebar has three tabs (${tabs.join(',')})`);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Edit' }).tap();
    const ribbon = async (tab, btn) => {
        const b = page.locator(`[data-ribbon-btn=${btn}]`).first();
        // a tab that is already showing is left alone (clicking it again may fold the ribbon on a phone)
        if (!await b.isVisible()) await page.locator('.ribbon-tab, [role=tab]', { hasText: new RegExp(`^\\s*${tab}\\s*$`) }).first().click();
        await b.click();
    };
    for (const [tab, btn, title, probe] of [['Calculation', 'k-settings', 'Settings', /./], ['Document', 'd-export', 'Export', /PDF|Word|HTML/], ['Document', 'd-prettify', 'Prettify', /Prettify Document/], ['Calculation', 'k-errors', 'Errors and warnings', /./]]) {
        await ribbon(tab, btn);
        const dlg = page.locator('.tab-content.as-dialog');
        await dlg.waitFor({ timeout: 10000 });
        check((await dlg.locator('.cpd-dialog-head').innerText()).includes(title), `${name}: ${btn} opens the ${title} dialog`);
        check(probe.test(await dlg.innerText()), `${name}: the ${title} dialog has its content`);
        const bb = await dlg.boundingBox();
        check(bb.width > 300 && bb.x >= 0 && bb.x + bb.width <= (page.viewportSize()?.width ?? 9999) + 1, `${name}: the ${title} dialog is ${Math.round(bb.width)}px wide and on screen`);
        if (shots && title !== 'Errors and warnings') await page.screenshot({ path: `${shots}/${name}-${title.toLowerCase()}.png` });
        await page.keyboard.press('Escape');
        await page.waitForTimeout(200);
        check(await page.locator('.tab-content.as-dialog').count() === 0, `${name}: Esc closes the ${title} dialog`);
    }
    // a click on the backdrop closes it too (on a phone the dialog fills the screen: there is no backdrop to hit)
    if (!phone) {
        await ribbon('Calculation', 'k-settings');
        await page.locator('.cpd-dialog-backdrop').click({ position: { x: 5, y: 5 } });
        check(await page.locator('.tab-content.as-dialog').count() === 0, `${name}: the backdrop closes the dialog`);
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
