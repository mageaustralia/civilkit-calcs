// 26.10.15: the ribbon follows the work: Calculation comes straight after the insert tabs, then Plots & Diagrams, Text
// Formatting, Document and Resources, each with its groups in the new order.
import { sleep } from './lib.mjs';

export default {
    id: 'ribbon-groups',
    crop: { x: 0, y: 0, w: 1280, h: 430 },
    async run(rec) {
        const { page } = rec;
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.click('[data-ribbon-tab=calc]');
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await rec.begin();
        await sleep(1300);
        for (const t of ['plots', 'textfmt', 'document', 'resources']) {
            await rec.click(page.locator(`[data-ribbon-tab=${t}]`), { ms: 650 });
            await sleep(t === 'plots' ? 1900 : 1600);
        }
        rec.mark('poster');
        await sleep(300);
        await rec.click(page.locator('[data-ribbon-tab=calc]'), { ms: 650 });
        await sleep(700);
        await rec.finish();
    },
};
