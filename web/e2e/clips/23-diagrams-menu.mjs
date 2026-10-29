// 26.10.15: the Plots & Diagrams ribbon tab: the Diagrams button in the Figures group opens a menu of starter diagrams;
// a click inserts one as its own block, drawn in the results.
import { frameWith, sleep } from './lib.mjs';

const START = [
    '<!--sheet-->',
    '# Design programme',
    '',
    'The programme for the footbridge, from concept to issue.',
    '',
    'L: 24 m | span',
    'w: 5 kN/m | design load',
    'Mstar: w * L^2 / 8 -> kN*m | design moment',
    '',
].join('\n');

export default {
    id: 'diagrams-menu',
    crop: { x: 0, y: 0, w: 1280, h: 720 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.updateOptions({ fontSize: 17, lineHeight: 28, quickSuggestions: false }); e.setPosition({ lineNumber: 4, column: 1 }); });
        await page.click('[data-ribbon-tab=calc]');
        await rec.waitFor(/Design programme/);
        await rec.begin();
        await sleep(300);
        await rec.click(page.locator('[data-ribbon-tab=plots]'), { ms: 600 });
        await sleep(700);
        const btn = page.locator('[data-ribbon-btn=dg-diagrams]');
        await rec.click(btn, { ms: 700 });
        const pop = page.locator('[data-ribbon-popover=dg-diagrams]');
        await pop.waitFor();
        await sleep(800);
        const item = pop.locator('.rb-glyph', { hasText: /^Flowchart$/ });
        const c = await rec.centre(item);
        await rec.glide(c.x, c.y - 70, 500);
        await sleep(250);
        await rec.glide(c.x, c.y, 400);
        await sleep(300);
        await rec.click(item, { ms: 200 });
        await rec.skip(async () => { const f = await frameWith(page, '.sheet-mermaid svg'); await sleep(800); await f.locator('.sheet-mermaid').first().evaluate((e) => e.scrollIntoView({ block: 'center' })); await sleep(500); });
        rec.mark('poster');
        await rec.glide(900, 520, 500);
        await sleep(1800);
        // undo, so the clip ends where it began
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.trigger('clip', 'undo', null); });
        await rec.skip(async () => { for (let i = 0; i < 40 && (await page.evaluate(() => window.calcpadActiveEditor.getValue().includes('mermaid'))); i++) await sleep(200); await sleep(1200); });
        await sleep(400);
        await rec.finish();
    },
};
