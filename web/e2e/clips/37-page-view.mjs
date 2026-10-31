// 26.10.20: the Pages button in the results bar lays the report out on sheets of the PDF's paper size, with the margins
// drawn, a "Page 2 of 3" label on each sheet and the automatic page breaks shown. Scrolled down through two or three pages.
import { frameWith, settle, sleep, widenResults } from './lib.mjs';

const block = (n, name) => Array.from({ length: n }, (_, i) => `${name}${i + 1}: ${(i + 2) * 1.5} kN/m * ${i + 3} m -> kN | ${name} ${i + 1}`).join('\n');
const START = `<!--sheet-->
# Floor beam report

## Loads
${block(14, 'D')}

## Reactions
${block(16, 'R')}

## Checks
${block(14, 'C')}
`;

export default {
    id: 'page-view',
    viewport: { width: 1280, height: 800 },
    crop: { x: 0, y: 170, w: 1280, h: 630 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Report$/ }).click();
        await rec.waitFor(/Floor beam report/);
        await frameWith(page, 'h1');
        await widenResults(page);
        await sleep(1500);
        await rec.begin();
        await sleep(1800);
        await rec.click(page.locator('.page-view-btn'), { ms: 800 });
        await frameWith(page, '#cpd-page-view');
        await settle(page);
        await sleep(1800);
        rec.mark('poster');
        // scroll down the pages a little at a time
        for (let i = 0; i < 4; i++) {
            const f = await frameWith(page, '#cpd-page-view');
            for (let s = 0; s < 16; s++) {
                await f.evaluate(() => window.scrollBy(0, 46));
                await sleep(45);
            }
            await sleep(1100);
        }
        await rec.finish();
    },
};
