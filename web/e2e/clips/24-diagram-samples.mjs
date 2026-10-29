// 26.10.15: new engineering samples draw their diagram beside the numbers: the gable frame thrust sheet, then a pump
// energy flow (Sankey) sheet, both opened from the Samples dialog.
import { frameWith, openSample, sleep } from './lib.mjs';

export default {
    id: 'diagram-samples',
    crop: { x: 0, y: 50, w: 1280, h: 620 },
    async run(rec) {
        const { page } = rec;
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await rec.begin();
        await sleep(300);
        await rec.click(page.locator('[data-ribbon-btn=x-samples]'), { ms: 500 });
        await page.waitForSelector('.cprs-s-group');
        await sleep(400);
        await rec.click(page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }), { ms: 500 });
        const item = page.locator('.cprs-s-item', { hasText: /Gable frame thrust/ }).first();
        await item.scrollIntoViewIfNeeded();
        await sleep(300);
        const ic = await rec.centre(item);
        await rec.glide(ic.x, ic.y, 450);
        await sleep(300);
        await item.dblclick();
        await page.waitForFunction(() => !document.querySelector('.cprs-s-group'), null, { timeout: 15000 }).catch(() => {});
        await rec.skip(async () => { const f = await frameWith(page, '.sheet-mermaid svg'); await sleep(500); await f.locator('.sheet-mermaid').first().evaluate((e) => e.scrollIntoView({ block: 'center' })); await sleep(600); });
        rec.mark('poster');
        await sleep(2400);
        // and a second sample, from the Plots and diagrams group
        await rec.click(page.locator('[data-ribbon-btn=x-samples]'), { ms: 600 });
        await page.waitForSelector('.cprs-s-group');
        await sleep(300);
        await rec.click(page.locator('.cprs-s-group', { hasText: /^Plots and diagrams/ }), { ms: 500 });
        const item2 = page.locator('.cprs-s-item', { hasText: /Energy flow through a pump/ }).first();
        await item2.scrollIntoViewIfNeeded();
        await sleep(300);
        const jc = await rec.centre(item2);
        await rec.glide(jc.x, jc.y, 450);
        await sleep(300);
        await item2.dblclick();
        await page.waitForFunction(() => !document.querySelector('.cprs-s-group'), null, { timeout: 15000 }).catch(() => {});
        await rec.skip(async () => { await frameWith(page, '.sheet-mermaid svg'); await sleep(900); });
        await sleep(2200);
        // close both sample tabs so the clip ends where it began
        await page.evaluate(() => { const t = window.calcpadTabs; while (t.all.length > 1) t.close(t.all[t.all.length - 1].id); });
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await sleep(250);
        await rec.finish();
    },
};
