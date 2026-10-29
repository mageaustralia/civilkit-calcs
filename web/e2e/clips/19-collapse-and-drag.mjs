// Unreleased: fold a heading's section with the arrow under its grip, open it again, then drag a section by its grip.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Loads\n\nw: 12 kN/m | load\n\nP: 20 kN | point load\n\n# Geometry\n\nL: 6 m | span\n\n# Results\n\nM: w * L^2 / 8 + P * L / 4 -> kN*m | maximum moment\n';
const MOVED = /Geometry.*span.*Loads.*point load.*Results/;

/** The visible block that holds `text`, looked up fresh (the preview is rebuilt after every edit). */
async function box(page, text) {
    const f = await frameWith(page, '.ck-grip');
    return f.locator('[data-source-line]', { hasText: text }).first().boundingBox();
}

export default {
    id: 'collapse-and-drag',
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
        // hover the Loads heading: the arrow under its grip folds the section
        let b = await box(page, 'Loads');
        await rec.glide(b.x + 120, b.y + b.height / 2, 700);
        await rec.glide(b.x + 60, b.y + b.height / 2, 300);
        await sleep(400);
        let f = await frameWith(page, '.ck-chev.on');
        await rec.click(f.locator('.ck-chev.on'), { ms: 450 });
        await frameWith(page, '.ck-col');
        await sleep(1600);
        f = await frameWith(page, '.ck-chev.col');
        await rec.click(f.locator('.ck-chev.col'), { ms: 400 });
        await sleep(1300);
        // drag the Geometry heading above Loads: its section goes with it
        b = await box(page, 'Geometry');
        await rec.glide(b.x + 80, b.y + b.height / 2, 600);
        await rec.glide(b.x + 40, b.y + b.height / 2, 300);
        await sleep(400);
        f = await frameWith(page, '.ck-grip.on');
        const g = await rec.centre(f.locator('.ck-grip.on'));
        const t = await box(page, 'Loads');
        await rec.glide(g.x, g.y, 400);
        await page.mouse.down();
        await rec.glide(g.x, g.y + 4, 120);
        await rec.glide(g.x + 2, t.y - 3, 1300);
        await sleep(600);
        await page.mouse.up();
        await rec.waitFor(MOVED);
        rec.mark('poster');
        await sleep(1500);
        // the toast offers Undo
        await rec.click(page.locator('.block-toast button'), { ms: 700 });
        await rec.waitGone(MOVED);
        await sleep(1200);
        await page.evaluate((s) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: s }]); }, START);
        await rec.glide(640, 650, 400);
        await sleep(500);
        await rec.finish();
    },
};
