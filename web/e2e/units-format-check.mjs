// Percentages in the app (book/src/units.md, "Percentages"),
// on a desktop window and on a phone.   node e2e/units-format-check.mjs [--url U]
//
//   1. `-> %` and `-> ‰` show a ratio as a percentage in the preview;
//   2. a percent input shows its number and `%` in Input mode, and a changed value flows into the results in percent;
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const spaced = (s) => s.replace(/\s+/g, ' ');

const PERCENT = `<!--sheet-->
# Percent check
Ed: 245.25 kN
Rd: 355.1 kN
ratio: Ed / Rd
util: ratio -> %
slope: 1 / 40 -> ‰
`;
const INPUT = `<!--sheet-->
# Rate check
rate: ? 12.5 % | rate
F: rate * 200 kN
twice: rate * 2 -> %
`;

const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!/^Canceled|^Event$/.test(String(e.message ?? e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();

    /** Waits until a result frame (the live one or the one being swapped in) shows `marker` and matches `re`. */
    const until = async (marker, re, tries = 120) => {
        for (let i = 0; i < tries; i++) {
            for (const f of page.frames()) {
                if (f === page.mainFrame()) continue;
                const t = spaced(await f.evaluate(() => document.body?.innerText ?? '').catch(() => ''));
                if (t.includes(marker) && re.test(t)) return t;
            }
            await page.waitForTimeout(250);
        }
        return '';
    };
    const setText = (t) => page.evaluate((s) => window.calcpadActiveEditor.setValue(s), t);

    // 1. Percent and per mille conversions.
    await setText(PERCENT);
    let t = await until('Percent check', /util = ratio = 0\.691 = 69\.07 %/);
    check(t !== '', `${name}: -> % shows the ratio as 69.07 %`);
    check(/slope = 1 40 = 25 ‰/.test(t), `${name}: -> ‰ shows 25 ‰`);

    // 2. A percent input: shown as 12.5 with %, and a new value re-runs the sheet.
    if (!phone) {
        await setText(INPUT);
        t = await until('Rate check', /twice = .*= 25 %/);
        check(/F = rate · 200 kN = 12\.5 % · 200 kN = 25 kN/.test(t), `${name}: a percent input is 0.125 in the formulas (${t.slice(t.indexOf('F ='), t.indexOf('F =') + 50)})`);
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click({ timeout: 10000 });
        let done = '';
        for (let attempt = 0; attempt < 4 && !done; attempt++) {
            let form = null;
            for (let i = 0; i < 60 && !form; i++) {
                for (const f of page.frames()) if (f !== page.mainFrame() && (await f.$('input.calcpad-ui-input').catch(() => null))) form = f;
                if (!form) await page.waitForTimeout(250);
            }
            if (!form) continue;
            if (attempt === 0) {
                const box = await form.evaluate(() => { const i = document.querySelector('input.calcpad-ui-input'); return [i.value, i.parentElement?.textContent ?? '']; }).catch(() => ['', '']);
                check(box[0] === '12.5' && /%/.test(box[1]), `${name}: the input box holds 12.5 with % after it (${JSON.stringify(box)})`);
            }
            await form.fill('input.calcpad-ui-input >> nth=0', '20').catch(() => {});
            await form.press('input.calcpad-ui-input >> nth=0', 'Enter').catch(() => {});
            done = await until('Rate check', /twice = .*= 40 %/, 24);
        }
        check(/= 40 kN/.test(done) && /= 40 %/.test(done), `${name}: 20 % in the box gives 40 kN and 40 %`);
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Preview$/ }).click({ timeout: 10000 });
        const dontSave = page.locator('.modal-backdrop button', { hasText: /^Don't Save$/ });
        if (await dontSave.waitFor({ timeout: 1500 }).then(() => true, () => false)) await dontSave.click();
    }

    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
