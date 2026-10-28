// 26.10.6: the "Beam Explorer: Plotly from Python" sample: pick a load case from Plotly's own menu.
import { frameWith, openSample, sleep, widenResults } from './lib.mjs';

export default {
    id: 'plotly-from-python',
    crop: { x: 340, y: 190, w: 940, h: 530 },
    async run(rec) {
        const { page } = rec;
        await openSample(rec, /^Python/, /Plotly from Python/);
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await widenResults(page);
        let f = await frameWith(page, '.cprs-plotly .main-svg', 600);
        await f.locator('.updatemenu-header').first().waitFor({ timeout: 120000 });
        await f.evaluate(() => { const g = document.querySelector('.cprs-plotly'); document.scrollingElement.scrollTop = g.getBoundingClientRect().top + document.scrollingElement.scrollTop - 10; });
        await sleep(1000);
        await rec.begin();
        await sleep(500);
        f = await frameWith(page, '.updatemenu-header');
        await rec.click(f.locator('.updatemenu-header').first(), { ms: 700 });
        await sleep(500);
        rec.mark('poster');
        await rec.click(f.locator('.updatemenu-item-text', { hasText: 'UDL on span 2 only' }).first(), { ms: 600 });
        await sleep(1200);
        f = await frameWith(page, '.updatemenu-header');
        await rec.click(f.locator('.updatemenu-header').first(), { ms: 600 });
        await sleep(400);
        await rec.click(f.locator('.updatemenu-item-text', { hasText: 'UDL on all spans' }).first(), { ms: 500 });
        await sleep(500);
        await rec.finish();
    },
};
