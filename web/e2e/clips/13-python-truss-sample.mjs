// 26.10.5: the "Plane Truss by the Stiffness Method" sample runs in the page and its result scrolls into view.
import { frameWith, openSample, settle, sleep, widenResults } from './lib.mjs';

export default {
    id: 'python-truss-sample',
    crop: { x: 340, y: 190, w: 940, h: 530 },
    async run(rec) {
        const { page } = rec;
        await openSample(rec, /^Python/, /Plane Truss/);
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await widenResults(page);
        await rec.waitFor(/[Mm]ember|[Rr]eaction/);
        await settle(page);
        await sleep(2500);
        // the preview swaps in a new frame after each run: look the front one up each time
        const cur = () => frameWith(page, 'h1, h2, p');
        const scroll = async (y) => { const f = await cur(); await f.evaluate((v) => { document.scrollingElement.scrollTop = v; }, y); };
        await scroll(0);
        await rec.begin();
        await scroll(0);
        await sleep(800);
        rec.mark('poster');
        const total = await (await cur()).evaluate(() => document.scrollingElement.scrollHeight - innerHeight);
        const to = Math.min(total, 760);
        for (let i = 1; i <= 10; i++) { await scroll(to * i / 10); await sleep(60); }
        await sleep(1100);
        for (let i = 5; i >= 0; i--) { await scroll(to * i / 5); await sleep(40); }
        await sleep(300);
        await rec.finish();
    },
};
