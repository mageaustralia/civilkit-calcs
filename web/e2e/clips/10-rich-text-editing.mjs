// 26.10.8: double-click a paragraph of a sheet and edit it as rich text: bold a word, click away, see it rendered.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Design basis\n\nThe load is applied at midspan of the beam.\n\na: 3 m | first side\nb: 4 m | second side\n';

export default {
    id: 'rich-text-editing',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/applied at midspan/);
        const f = await frameWith(page, '[data-source-line="4"]');
        await rec.begin();
        await sleep(250);
        const b = await f.locator('[data-source-line="4"]').first().boundingBox();
        await rec.glide(b.x + 120, b.y + 10, 500);
        await sleep(100);
        await page.mouse.dblclick(b.x + 120, b.y + 10);
        await page.waitForSelector('.md-edit .ProseMirror');
        await sleep(600);
        // double-click the word "load" in the editor: it is selected; the B button makes it bold
        const w = await page.evaluate(() => {
            const p = document.querySelector('.md-edit .ProseMirror p');
            const t = document.createTreeWalker(p, NodeFilter.SHOW_TEXT);
            for (let n = t.nextNode(); n; n = t.nextNode()) {
                const i = n.data.indexOf('load');
                if (i >= 0) { const r = document.createRange(); r.setStart(n, i); r.setEnd(n, i + 4); const q = r.getBoundingClientRect(); return { x: q.x + q.width / 2, y: q.y + q.height / 2 }; }
            }
            return null;
        });
        await rec.glide(w.x, w.y, 400);
        await page.mouse.dblclick(w.x, w.y);
        await sleep(350);
        await rec.click(page.locator('.md-block-bar button[title^="Bold"]'), { ms: 450 });
        await page.waitForSelector('.md-edit .ProseMirror strong');
        await sleep(700);
        rec.mark('poster');
        // click away: the paragraph is applied and rendered
        await rec.clickAt(200, 640, 550);
        await page.waitForSelector('.md-edit', { state: 'detached' });
        await rec.waitFor(/applied at midspan/);
        await sleep(1100);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/\*\*load/);
        await sleep(500);
        await rec.finish();
    },
};
