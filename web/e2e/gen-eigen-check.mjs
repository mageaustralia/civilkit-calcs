// Generalized eigenvalues in the app: a sheet and a Calcpad document compute eigenvals(K, M) / eigenvecs(K, M)
// (K x = λ M x), a mass matrix that is not positive definite is refused with a clear message, the editor documents
// the two-matrix form, and the Shear frame natural frequencies sample uses it and still gives 3.97 Hz and 9.02 Hz,
// on a desktop window and on a phone.   node e2e/gen-eigen-check.mjs [--url U]
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
    // The text of the preview frame that shows `want` (front or back frame, whichever has it).
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

    // Three equal masses on equal springs, fixed at one end: λ = 4 k/m sin²((2r - 1) π / 14).
    const sheet = ['<!--sheet-->', '# Chain', '', 'k: 1000 N/m', 'm: 2 kg', 'K: [2*k, -k, 0 N/m; -k, 2*k, -k; 0 N/m, -k, k]',
        'M: [m, 0 kg, 0 kg; 0 kg, m, 0 kg; 0 kg, 0 kg, m]', 'lam: eigenvals(K, M)', 'Phi: eigenvecs(K, M)',
        'mm: dot(row(Phi, 2), M*row(Phi, 2))', 'bad: eigenvals([2, 1; 1, 2], [1, 2; 2, 1])', 'size: eigenvecs(K, [1, 0; 0, 1])'].join('\n');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    let text = await preview(/Matrix dimensions/);
    const want = [1, 3, 5].map((r) => 2000 * Math.sin(r * Math.PI / 14) ** 2);
    const got = (/lam = eigenvals\s*\(\s*K\s*;\s*M\s*\)\s*=\s*\[\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*\]\s*s/.exec(text) ?? []).slice(1).map(Number);
    check(got.length === 3 && got.every((g, i) => Math.abs(g - want[i]) < 1e-3 * want[i]), `${name}: a sheet gives the closed-form ω² of a spring chain in s⁻² (${got.join(', ')} vs ${want.map(w => w.toFixed(3)).join(', ')})`);
    check(/mm = .*=\s*1(\s|$)/.test(text), `${name}: the mode shapes are mass normalised (xᵀ M x = 1)`);
    check(/bad = eigenvals.*The second matrix \(M\) must be symmetric and positive definite/.test(text), `${name}: a mass matrix that is not positive definite is refused with a clear message`);
    check(/size = eigenvecs.*Matrix dimensions do not match/.test(text), `${name}: matrices of different sizes are refused`);

    if (phone) await page.locator('.mobile-bar button', { hasText: 'Editor' }).tap().catch(() => {});
    // A Calcpad document, its own tab.
    await page.evaluate(t => window.calcpadTabs.newUntitled(t), "'Pair\nK = [2; 1|1; 2]\nB = [1; 0|0; 4]\nL = eigenvals(K; B)\nE = eigen(K; B)\n");
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    text = await preview(/E = eigen/);
    const r = Math.sqrt(13);
    const wantL = [(5 - r) / 4, (5 + r) / 4].map((x) => x.toPrecision(3));
    check(new RegExp(`L = eigenvals\\s*\\(\\s*K\\s*;\\s*B\\s*\\)\\s*=\\s*\\[\\s*${wantL[0]}\\d*\\s+${wantL[1]}\\d*\\s*\\]`).test(text), `${name}: a Calcpad document gives the eigenvalues of the pair (${text.slice(0, 140)})`);

    if (!phone) {
        // Hover documents the two-matrix form.
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.setValue('L = eigenvals(K; M)'); e.setPosition({ lineNumber: 1, column: 8 }); e.focus(); e.trigger('t', 'editor.action.showHover', {}); });
        await page.waitForSelector('.monaco-hover-content', { timeout: 8000 }).catch(() => {});
        const hover = flat(await page.locator('.monaco-hover-content').first().innerText().catch(() => ''));
        check(/K x = λ M x/.test(hover) && /positive definite/.test(hover), `${name}: hover on eigenvals explains eigenvals(K; M) (${hover.slice(0, 100)})`);
        await page.keyboard.press('Escape');
    }

    // The sample uses the two-matrix form and keeps its results.
    const src = await page.evaluate(async () => {
        const m = await (await fetch('sheet-samples/manifest.json')).json();
        const s = m.samples.find((x) => x.id === 'sheet-shear-frame-frequencies');
        return s ? (await (await fetch(s.file)).text()) : '';
    });
    check(/eigenvals\(K, M\)/.test(src) && !/inverse\(M\)/.test(src), `${name}: the Shear frame sample uses eigenvals(K, M)`);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Editor' }).tap().catch(() => {});
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), src);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    text = await preview(/check = /);
    check(/3\.97\d* Hz/.test(text) && /9\.01\d* Hz|9\.02 Hz/.test(text) && /0\.25(2|17\d*) s/.test(text) && !/Error/.test(text), `${name}: the sample still gives 3.97 Hz and 9.02 Hz without errors`);
    check(/\[\s*0\.58\d*\s+1\s*\]/.test(text) && /\[\s*-1\.14\d*\s+1\s*\]/.test(text), `${name}: the sample's mode shapes are unchanged`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
