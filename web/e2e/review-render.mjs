// Imports each worksheet through the app and saves the rendered preview as page-sized PNGs for review.
// Usage: node review-render.mjs --origin http://127.0.0.1:4191/calcs/ --out DIR [--theme Light] [--max 10] FILE...
import { chromium } from 'playwright';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, join } from 'node:path';

const arg = (name, def) => { const i = process.argv.indexOf(name); return i > 0 ? process.argv[i + 1] : def; };
const origin = arg('--origin', 'http://127.0.0.1:4191/calcs/');
const out = arg('--out', '.');
const theme = arg('--theme', 'Light');
const maxPages = Number(arg('--max', '10'));
const skip = new Set(['--origin', '--out', '--theme', '--max', '--settle']);
const files = process.argv.slice(2).filter((a, i, all) => !a.startsWith('--') && !skip.has(all[i - 1]));
mkdirSync(out, { recursive: true });

const browser = await chromium.launch();
const summary = [];
for (const file of files) {
    const name = basename(file).replace(/\.(mcdx|xmcd)$/i, '').replace(/[^\w.-]+/g, '_');
    const ctx = await browser.newContext({ viewport: { width: 1700, height: 1000 } });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(String(e).slice(0, 200)));
    try {
        await page.goto(origin);
        await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
        await page.evaluate(() => { delete window.showSaveFilePicker; delete window.showOpenFilePicker; });
        await page.evaluate((th) => window.calcpadBridge.handleMessage({ type: 'updateColorTheme', theme: th }), theme);
        await page.locator('[data-ribbon-btn=file]').click();
        const [fc] = await Promise.all([page.waitForEvent('filechooser'), page.getByRole('menuitem', { name: /Import worksheet/ }).click()]);
        await fc.setFiles({ name: basename(file), mimeType: 'application/octet-stream', buffer: readFileSync(file) });
        await page.locator('[data-cprs-import-summary]').waitFor({ timeout: 120000 });
        const dlg = await page.locator('#cprs-import-dialog').innerText();
        await page.keyboard.press('Escape');
        writeFileSync(join(out, `${name}.import.txt`), dlg);
        writeFileSync(join(out, `${name}.cpd`), await page.evaluate(() => window.calcpadActiveEditor.getValue()));
        // wait for the preview to settle
        let fr = null;
        for (let i = 0; i < 120 && !fr; i++) {
            for (const f of page.frames().slice(1)) {
                try { if (await f.evaluate(() => document.body && document.body.innerText.length > 20 && !/Calculating/.test(document.body.innerText))) fr = f; } catch { /* detached */ }
            }
            if (!fr) await page.waitForTimeout(1000);
        }
        if (!fr) { summary.push(`${name}: no preview`); continue; }
        await page.waitForTimeout(Number(arg("--settle", "2500")));
        const h = await fr.evaluate(() => document.documentElement.scrollHeight);
        const el = await (await fr.frameElement()).boundingBox();
        const step = Math.floor(el.height);
        let n = 0;
        for (let y = 0; y < h && n < maxPages; y += step, n++) {
            await fr.evaluate((yy) => window.scrollTo(0, yy), y);
            await page.waitForTimeout(250);
            await page.screenshot({ path: join(out, `${name}-p${String(n + 1).padStart(2, '0')}.png`), clip: el });
        }
        const bad = await fr.evaluate(() => ({
            errors: document.querySelectorAll('.err, .error, .errorHeader').length,
            vscroll: [...document.querySelectorAll('*')].filter(e => /auto|scroll/.test(getComputedStyle(e).overflowY) && e.scrollHeight > e.clientHeight + 1 && e !== document.documentElement && e !== document.body).length,
        }));
        summary.push(`${name}: ${n} page(s) of ${Math.ceil(h / step)}, height ${h}px, error banners ${bad.errors}, inner-scroll boxes ${bad.vscroll}${errors.length ? ', page errors ' + errors.length : ''}`);
    } catch (e) {
        summary.push(`${name}: FAILED ${String(e).slice(0, 160)}`);
    }
    await ctx.close();
}
await browser.close();
console.log(summary.join('\n'));
writeFileSync(join(out, 'summary.txt'), summary.join('\n'));
