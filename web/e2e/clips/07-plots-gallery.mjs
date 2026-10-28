// 26.10.11: the "Plots" sample, a gallery of Plotly plots; the results pane scrolls through them.
import { frameWith, openSample, sleep } from './lib.mjs';

export default {
    id: 'plots-gallery',
    crop: { x: 400, y: 190, w: 880, h: 530 },
    async run(rec) {
        const { page } = rec;
        await openSample(rec, /^Plots/, /Plot gallery/i);
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        const f = await frameWith(page, '.cprs-plotly .main-svg', 600);
        await f.locator('.ck-chips').first().waitFor({ timeout: 120000 });
        await f.evaluate(() => { document.scrollingElement.scrollTop = Math.max(0, document.querySelector('.cprs-plotly').getBoundingClientRect().top + document.scrollingElement.scrollTop - 40); });
        await sleep(1500);
        await rec.begin();
        await sleep(1200);
        rec.mark('poster');
        const total = await f.evaluate(() => document.scrollingElement.scrollHeight - innerHeight);
        const from = await f.evaluate(() => document.scrollingElement.scrollTop);
        for (let i = 1; i <= 24; i++) { await f.evaluate((y) => { document.scrollingElement.scrollTop = y; }, from + (Math.min(total, from + 1500) - from) * i / 24); await sleep(110); }
        await sleep(1000);
        await f.evaluate((y) => document.scrollingElement.scrollTo({ top: y, behavior: 'smooth' }), from);
        await sleep(1500);
        await rec.finish();
    },
};
