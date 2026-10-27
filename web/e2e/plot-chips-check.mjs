// The chips under a plot: the engine's SVG plot (copy data, save) and a Plotly figure (log, 1:1, copy, save), desktop and phone.
//   node e2e/plot-chips-check.mjs [--url U]
import { chromium, devices } from 'playwright';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const dir = join(dirname(fileURLToPath(import.meta.url)), '../calcpad-web/public/sheet-samples');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

async function open(opts, sheet) {
    const ctx = await browser.newContext({ ...opts, acceptDownloads: true, permissions: ['clipboard-read', 'clipboard-write'] });
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.evaluate(v => window.calcpadActiveEditor.setValue(v), readFileSync(join(dir, sheet), 'utf8'));
    return { ctx, page, errors };
}
async function frameWith(page, sel) {
    for (let i = 0; i < 150; i++) {
        const yes = page.locator('#cprs-python-yes');
        if (await yes.count().catch(() => 0)) await yes.first().click().catch(() => {});
        for (const fr of page.frames()) if (fr !== page.mainFrame() && await fr.locator(sel).count().catch(() => 0)) return fr;
        await page.waitForTimeout(400);
    }
    return null;
}
const clip = page => page.evaluate(() => navigator.clipboard.readText());

// the engine's plot: M(x) = w x (L - x) / 2 with w = 12, L = 6, so M(3) = 54 and M(6) = 0
{
    const { ctx, page, errors } = await open({ viewport: { width: 1300, height: 900 } }, 'Plot.sheet');
    const fr = await frameWith(page, 'svg.plot[data-plot]');
    check(!!fr, 'svg: the plot is drawn');
    await fr.locator('.ck-chips').first().waitFor({ timeout: 10000 });
    check(await fr.locator('.ck-chips .ck-chip').count() === 3, 'svg: three chips (Copy data, PNG, SVG)');
    await fr.locator('.ck-chip', { hasText: 'Copy data' }).click();
    await page.waitForTimeout(500);
    const rows = (await clip(page)).split('\n').slice(1).map(l => l.split('\t').map(Number));
    const near = (x) => rows.reduce((a, r) => Math.abs(r[1] - x) < Math.abs(a[1] - x) ? r : a);
    const mid = near(3);
    check(rows.length > 20 && Math.abs(mid[2] - 54) < 0.5, `svg: copied curve peaks at 54 near x = 3 (got ${mid})`);
    const end = near(6);
    check(Math.abs(end[1] - 6) < 0.05 && Math.abs(end[2]) < 0.5, `svg: copied curve ends near (6, 0) (got ${end})`);
    for (const [label, ext] of [['PNG', '.png'], ['SVG', '.svg']]) {
        const [dl] = await Promise.all([page.waitForEvent('download', { timeout: 10000 }), fr.locator('.ck-chip', { hasText: label }).click()]);
        check(dl.suggestedFilename().endsWith(ext), `svg: ${label} downloads ${dl.suggestedFilename()}`);
    }
    check(errors.length === 0, `svg: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// a Plotly figure
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 1000 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const { ctx, page, errors } = await open(opts, 'Plot gallery implicit curves.sheet');
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const fr = await frameWith(page, '.cprs-plotly .main-svg');
    check(!!fr, `${name}: the Plotly figure is drawn`);
    await fr.locator('.ck-chips').first().waitFor({ timeout: 15000 });
    const chips = fr.locator('.ck-chips').first();
    check(await chips.locator('.ck-chip').count() === 6, `${name}: six chips (Log x, Log y, 1:1, Copy data, PNG, SVG)`);
    const click = l => phone ? chips.locator('.ck-chip', { hasText: l }).tap() : chips.locator('.ck-chip', { hasText: l }).click();
    const axisType = () => fr.locator('.cprs-plotly').first().evaluate(gd => gd._fullLayout.xaxis.type + '/' + gd._fullLayout.yaxis.type);
    await click('Log x'); await page.waitForTimeout(500);
    check(/^log\//.test(await axisType()), `${name}: Log x switches the x axis to log (${await axisType()})`);
    await click('Log y'); await page.waitForTimeout(500);
    check(/^log\/log$/.test(await axisType()), `${name}: Log y too (${await axisType()})`);
    await click('Log x'); await click('Log y'); await page.waitForTimeout(500);
    check(/^linear\/linear$/.test(await axisType()), `${name}: both back to linear (${await axisType()})`);
    const anchor = () => fr.locator('.cprs-plotly').first().evaluate(gd => gd._fullLayout.yaxis.scaleanchor);
    const was = await anchor(); // this figure already draws to scale, and its chip starts pressed
    check(was === 'x' && await chips.locator('.ck-chip', { hasText: '1:1' }).getAttribute('aria-pressed') === 'true', `${name}: 1:1 starts pressed on a figure drawn to scale`);
    await click('1:1'); await page.waitForTimeout(500);
    check(await anchor() !== 'x', `${name}: 1:1 releases the scale lock`);
    await click('1:1'); await page.waitForTimeout(500);
    check(await anchor() === 'x', `${name}: 1:1 locks it again`);
    await click('Copy data'); await page.waitForTimeout(500);
    const t = await clip(page).catch(() => '');
    check(/^series\tx\ty\tz\n/.test(t) && t.split('\n').length > 10, `${name}: Copy data puts a table on the clipboard (${t.split('\n').length} lines)`);
    if (!phone) {
        const [dl] = await Promise.all([page.waitForEvent('download', { timeout: 15000 }), chips.locator('.ck-chip', { hasText: 'PNG' }).click()]);
        check(dl.suggestedFilename() === 'plot.png', `${name}: PNG downloads`);
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
