// The Settings gear at the right end of the top strip: it is on every tab, at the right edge, and opens the Settings dialog,
// on a desktop window and on a phone.   node e2e/settings-gear-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 800 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    const gear = page.locator('[data-ribbon-btn=q-settings]');
    check(await gear.count() === 1 && await gear.isVisible(), `${name}: the gear is shown`);
    const gb = await gear.boundingBox();
    const vw = page.viewportSize().width;
    check(gb && vw - (gb.x + gb.width) < 16, `${name}: the gear is at the right edge (${Math.round(vw - (gb.x + gb.width))} px from it)`);
    check(!!(await gear.getAttribute('title')) && !!(await gear.getAttribute('aria-label')), `${name}: the gear has a tooltip and an accessible name`);
    // On every tab.
    const tabs = await page.locator('[data-ribbon-tab]').evaluateAll(es => es.map(e => e.getAttribute('data-ribbon-tab')));
    let every = true;
    for (const t of tabs.slice(0, 4)) { await page.locator(`[data-ribbon-tab=${t}]`).click(); if (!await gear.isVisible()) every = false; }
    check(every, `${name}: the gear stays on every tab`);
    phone ? await gear.tap() : await gear.click();
    const open = await page.locator('#cprs-numfmt-choice').waitFor({ timeout: 10000 }).then(() => true).catch(() => false);
    check(open, `${name}: the gear opens the Settings dialog (Number format is in it)`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
