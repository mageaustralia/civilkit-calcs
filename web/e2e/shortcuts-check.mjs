// In a browser the shortcuts the browser owns (address bar, tabs, zoom, bookmarks) must not be taken by the app:
// the keydown is left unhandled (default not prevented) and the editor does not act on it.   node e2e/shortcuts-check.mjs [--url U]
import { chromium } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const page = await (await browser.newContext({ viewport: { width: 1300, height: 900 } })).newPage();
await page.addInitScript(() => { window.__keys = []; window.addEventListener('keydown', e => { window.__keys.push(e); }, true); });
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue('alpha\nbeta'); e.setPosition({ lineNumber: 1, column: 2 }); e.focus(); });
for (const k of ['Meta+L', 'Control+L', 'Meta+D', 'Meta+R', 'Meta+N', 'Meta+P', 'Meta+1', 'Meta+Equal', 'Meta+Shift+V']) {
    await page.evaluate(() => { window.__keys.length = 0; });
    await page.keyboard.press(k);
    const r = await page.evaluate(() => ({
        prevented: window.__keys.some(e => e.defaultPrevented),
        sel: (() => { const e = window.calcpadActiveEditor; return e.getModel().getValueInRange(e.getSelection()).length; })(),
        text: window.calcpadActiveEditor.getValue(),
    }));
    check(!r.prevented && r.sel === 0 && r.text === 'alpha\nbeta', `${k} is left to the browser`);
}
await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setPosition({ lineNumber: 1, column: 1 }); e.focus(); window.__keys.length = 0; });
await page.keyboard.press('Meta+A');
check(await page.evaluate(() => { const e = window.calcpadActiveEditor; return e.getModel().getValueInRange(e.getSelection()).length; }) === 10, 'Select all still works in the editor');
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
