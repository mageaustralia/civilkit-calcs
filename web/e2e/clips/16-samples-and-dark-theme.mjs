// 26.10.1: the Samples browser opens a worksheet; the Dark button switches the preview to the dark theme and back.
import { frameWith, resultHas, settle, sleep } from './lib.mjs';

export default {
    id: 'samples-and-dark-theme',
    crop: { x: 0, y: 50, w: 1280, h: 620 },
    async run(rec) {
        const { page } = rec;
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await rec.begin();
        await sleep(200);
        await rec.click(page.locator('[data-ribbon-btn=x-samples]'), { ms: 500 });
        await page.waitForSelector('.cprs-s-group');
        await sleep(450);
        // the first sample of the list is a sheet: hover it, then open it with a double-click
        const item = page.locator('.cprs-s-item').first();
        const ic = await rec.centre(item);
        await rec.glide(ic.x, ic.y, 450);
        await sleep(300);
        await item.dblclick();
        await page.waitForFunction(() => !document.querySelector('.cprs-s-group'), null, { timeout: 15000 }).catch(() => {});
        await rec.waitFor(/[Bb]eam/);
        await sleep(500);
        await rec.click(page.locator('[data-ribbon-btn=x-dark]'), { ms: 700 });
        await rec.skip(async () => { await settle(page); }, 0.3);
        await sleep(1000);
        rec.mark('poster');
        await rec.click(page.locator('[data-ribbon-btn=x-light]'), { ms: 600 });
        await rec.skip(async () => { await settle(page); }, 0.3);
        await sleep(300);
        // close the sample's tab: the clip ends where it began
        await page.evaluate(() => { const t = window.calcpadTabs; t.close(t.all[t.all.length - 1].id); });
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await sleep(250);
        await rec.finish();
    },
};
