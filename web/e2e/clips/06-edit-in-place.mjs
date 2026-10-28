// 26.10.12: one click on a rendered block opens it where it stands; clicking away applies the change.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Right triangle\n\na: 3 m | first side\nb: 4 m | second side\nc: sqrt(a^2 + b^2) | hypotenuse\n';

export default {
    id: 'edit-in-place',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/hypotenuse/);
        let f = await frameWith(page, '[data-source-line="5"]');
        await sleep(1500);
        rec.mark('start');
        await sleep(500);
        const block = f.locator('[data-source-line="5"]').first();
        const b = await block.boundingBox();
        await rec.glide(b.x + 220, b.y + 10, 700);
        await sleep(200);
        await rec.clickAt(b.x + 40, b.y + Math.min(8, b.height / 2), 500);
        await page.waitForSelector('.line-edit-input');
        await sleep(500);
        await page.keyboard.press('ControlOrMeta+A');
        await rec.type('b: 6 m | second side', 55);
        await sleep(400);
        // click away: the change is applied
        await rec.clickAt(260, 640, 600);
        await page.waitForSelector('.line-edit', { state: 'detached' });
        await rec.waitFor(/= 7\.21 m|7\.21/);
        rec.mark('poster');
        await sleep(1300);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/7\.21/);
        await sleep(700);
        await rec.finish();
    },
};
