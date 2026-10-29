// 26.10.12: hover between two blocks of a sheet, click the +, pick a block from the menu.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Load check\n\nw: 12 kN/m | load\nL: 6 m | span\n\nM: w * L^2 / 8 -> kN*m | maximum moment\n';

export default {
    id: 'insert-strip',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/Load check/);
        let f = await frameWith(page, '.ck-ins-btn');
        await sleep(1500);   // the handles are rebuilt as the page settles
        // the strip under the heading
        const zones = f.locator('.ck-ins');
        const at = 1;
        const zone = zones.nth(at);
        await zone.scrollIntoViewIfNeeded();
        const z = await zone.boundingBox();
        rec.mark('start');
        await sleep(500);
        await rec.glide(z.x + z.width / 2 + 160, z.y + z.height / 2 + 30, 600);
        await rec.glide(z.x + z.width / 2 + 20, z.y + z.height / 2, 400);
        await sleep(500);
        f = await frameWith(page, '.ck-ins-btn');
        const btn = f.locator('.ck-ins-btn').nth(at);
        await rec.click(btn, { ms: 350 });
        await page.waitForSelector('.insert-menu');
        await sleep(700);
        await rec.click(page.locator('.insert-menu button', { hasText: 'Table' }), { ms: 500 });
        await rec.waitFor(/Item\s+Value/);
        rec.mark('poster');
        await sleep(1600);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/Item\s+Value/);
        await sleep(700);
        await rec.finish();
    },
};
