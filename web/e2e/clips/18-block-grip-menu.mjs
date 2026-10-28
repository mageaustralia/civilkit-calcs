// Unreleased: the grip at the left of a rendered block opens its menu; Duplicate, Move down, then Undo from the toast.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam\n\nw: 12 kN/m | load\n\nL: 6 m | span\n\nM: w * L^2 / 8 -> kN*m | maximum moment\n';

/** The visible block that holds `text`, looked up fresh (the preview is rebuilt after every edit). */
async function block(page, text) {
    const f = await frameWith(page, '.ck-grip');
    return f.locator('[data-source-line]', { hasText: text }).first();
}

export default {
    id: 'block-grip-menu',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/maximum moment/);
        await frameWith(page, '.ck-grip');
        await sleep(1500);   // the grips are placed once the page has settled
        await rec.begin();
        // hover the moment line: its grip shows; click it and pick Duplicate
        let b = await (await block(page, 'maximum moment')).boundingBox();
        await rec.glide(b.x + b.width / 2 + 60, b.y + b.height / 2, 700);
        await rec.glide(b.x + b.width / 2, b.y + b.height / 2, 300);
        await sleep(400);
        let f = await frameWith(page, '.ck-grip.on');
        await rec.click(f.locator('.ck-grip.on'), { ms: 500 });
        await page.waitForSelector('.block-menu');
        await sleep(700);
        await rec.click(page.locator('.block-menu button', { hasText: 'Duplicate' }), { ms: 450 });
        await page.waitForSelector('.block-toast');
        await rec.skip(async () => { await page.waitForFunction(() => window.calcpadActiveEditor.getValue().split('maximum moment').length === 3); await frameWith(page, '.ck-grip'); });
        await rec.waitFor(/maximum moment.*maximum moment/);
        await sleep(1400);
        // the load line: Move down puts it under the span
        b = await (await block(page, 'load')).boundingBox();
        await rec.glide(b.x + b.width / 2 + 60, b.y + b.height / 2, 600);
        await rec.glide(b.x + b.width / 2, b.y + b.height / 2, 300);
        await sleep(400);
        f = await frameWith(page, '.ck-grip.on');
        await rec.click(f.locator('.ck-grip.on'), { ms: 500 });
        await page.waitForSelector('.block-menu');
        await sleep(700);
        await rec.click(page.locator('.block-menu button', { hasText: 'Move down' }), { ms: 450 });
        await rec.waitFor(/span.*load.*maximum moment/);
        rec.mark('poster');
        await sleep(1200);
        // the toast at the bottom offers Undo
        await rec.click(page.locator('.block-toast button'), { ms: 700 });
        await rec.waitFor(/load.*span.*maximum moment/);
        await sleep(1300);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/maximum moment.*maximum moment/);
        await page.waitForSelector('.block-toast', { state: 'detached', timeout: 10000 }).catch(() => {});
        await rec.glide(640, 650, 400);
        await sleep(500);
        await rec.finish();
    },
};
