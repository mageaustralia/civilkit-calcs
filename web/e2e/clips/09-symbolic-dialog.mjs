// 26.10.9: the Symbolic dialog on the Functions tab: integrate the bending moment of a cantilever and insert the result into the sheet.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Bending moment\n\nw: 12 kN/m | load\nL: 6 m | span\n';
const D = '#cprs-sym-dialog';

export default {
    id: 'symbolic-dialog',
    crop: { x: 0, y: 50, w: 1280, h: 620 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await rec.waitFor(/Bending moment/);
        await page.click('[data-ribbon-tab=functions]');
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.focus(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
        await rec.begin();
        await sleep(250);
        await rec.click(page.locator('[data-ribbon-btn=fn-symbolic]'), { ms: 500 });
        await page.locator(D).waitFor();
        await sleep(300);
        const expr = page.locator(`${D} [name=expr]`);
        await rec.click(expr, { ms: 400 });
        await rec.type('w*x^2/2', 60);
        await sleep(150);
        await page.locator(`${D} select[name=op]`).selectOption('integrate');
        await page.locator(`${D} [name=var]`).fill('x');
        await page.locator(`${D} [name=lower]`).fill('0');
        await page.locator(`${D} [name=upper]`).fill('L');
        await sleep(350);
        await rec.click(page.locator(`${D} [data-cprs-sym-run]`), { ms: 450 });
        await rec.skip(async () => {
            await page.waitForFunction(() => { const d = document.getElementById('cprs-sym-dialog'); return d && !d.querySelector('.sym-out').hidden && d.querySelector('[data-cprs-sym-run]').textContent === 'Run'; }, null, { timeout: 120000 });
            await page.waitForFunction(() => document.querySelector('#cprs-sym-dialog [data-cprs-sym-math]')?.getAttribute('data-typeset') === '1', null, { timeout: 30000 }).catch(() => {});
        });
        rec.mark('poster');
        await sleep(900);
        await rec.click(page.locator(`${D} [data-cprs-sym-insert]`), { ms: 450 });
        await page.locator(D).waitFor({ state: 'detached' });
        await rec.waitFor(/L3\s*w|L\^3/);
        await sleep(800);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/L3\s*w|L\^3/);
        await sleep(400);
        await rec.finish();
    },
};
