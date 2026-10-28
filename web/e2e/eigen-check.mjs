// Eigenvalues in the app: a sheet and a Calcpad document compute eigenvals / ceigenvals of non-symmetric matrices,
// the editor completes and documents ceigenvals, and the Principal stresses sample opens, on a desktop window and on
// a phone.   node e2e/eigen-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const flat = (s) => s.replace(/\s+/g, ' ');

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    // The text of the preview frame that shows `want`. The preview keeps a front and a back frame and swaps them
    // after each render, so both are read; the back one only ever holds an earlier document.
    const preview = async (want) => {
        let last = '';
        for (let i = 0; i < 60; i++) {
            for (const f of page.frames().filter((x) => x !== page.mainFrame())) {
                const t = flat(await f.locator('body').innerText().catch(() => ''));
                if (want.test(t)) return t;
                if (t) last = t;
            }
            await page.waitForTimeout(500);
        }
        return last;
    };

    const sheet = ['<!--sheet-->', '# Eigen', '', 'A: [4, 1; 2, 3]', 'lam: eigenvals(A)', '', 'R: [1, -2; 2, 1]', 'C: ceigenvals(R)', 'bad: eigenvals(R)'].join('\n');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    let text = await preview(/complex eigenvalues/);
    check(/lam = eigenvals\s*\(\s*A\s*\)\s*=\s*\[\s*2\s+5\s*\]/.test(text), `${name}: a sheet shows the eigenvalues of a non-symmetric matrix (${text.slice(0, 120)})`);
    check(/C = ceigenvals\s*\(\s*R\s*\)\s*=\s*1\s+1\s+-2\s+2/.test(text), `${name}: ceigenvals shows real and imaginary rows`);
    check(/The matrix has complex eigenvalues/.test(text), `${name}: eigenvals of a rotation says to use ceigenvals`);

    if (phone) await page.locator('.mobile-bar button', { hasText: 'Editor' }).tap().catch(() => {});
    // A Calcpad document is its own tab (the mode of a tab follows its first text).
    await page.evaluate(t => window.calcpadTabs.newUntitled(t), "'Stress\nS = [80; 25|25; -40]\nsigma = eigenvals(S)\nW = ceigenvecs([0; -1|1; 0])\n");
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    text = await preview(/W = ceigenvecs/);
    check(/=\s*\[\s*-45\s+85\s*\]/.test(text), `${name}: a Calcpad document gives the principal stresses (${text.slice(0, 120)})`);
    check(/W = ceigenvecs.*0\.707/.test(text), `${name}: ceigenvecs in a Calcpad document`);

    if (!phone) {
        // Completion and hover come from the snippet library.
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue("x = ceig"); e.setPosition({ lineNumber: 1, column: 9 }); e.focus(); e.trigger('t', 'editor.action.triggerSuggest', {}); });
        await page.waitForSelector('.suggest-widget .monaco-list-row', { timeout: 8000 }).catch(() => {});
        const rows = flat(await page.locator('.suggest-widget').innerText().catch(() => ''));
        check(/ceigenvals/.test(rows) && /ceigenvecs/.test(rows), `${name}: typing ceig offers ceigenvals and ceigenvecs (${rows.slice(0, 80)})`);
        await page.keyboard.press('Escape');
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue('C = ceigenvals([0; -1|1; 0])'); e.setPosition({ lineNumber: 1, column: 7 }); e.focus(); e.trigger('t', 'editor.action.showHover', {}); });
        await page.waitForSelector('.monaco-hover-content', { timeout: 8000 }).catch(() => {});
        const hover = flat(await page.locator('.monaco-hover-content').first().innerText().catch(() => ''));
        check(/imaginary parts in row 2/.test(hover), `${name}: hover on ceigenvals explains the layout (${hover.slice(0, 80)})`);
        await page.keyboard.press('Escape');
    }

    // The sample is listed and loads.
    const ok = await page.evaluate(async () => {
        const r = await fetch('sheet-samples/manifest.json');
        const m = await r.json();
        const s = m.samples.find((x) => x.id === 'sheet-principal-stresses');
        return s ? (await (await fetch(s.file)).text()) : '';
    });
    check(/eigenvals\(S\)/.test(ok), `${name}: the Principal stresses sample is in the manifest`);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Editor' }).tap().catch(() => {});
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), ok);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    text = await preview(/I _?1c?\s*=.*70 MPa/);
    check(/=\s*\[\s*-45\s+85\s*\]\s*MPa/.test(text) && /11\.31/.test(text) && !/Error/.test(text), `${name}: the sample renders the principal stresses and angle without errors`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
