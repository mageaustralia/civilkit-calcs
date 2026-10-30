// 26.10.19: in Input mode a run of input lines shares its columns: the names end at one place (the equals signs meet) and
// the boxes, sliders, units and notes start together, whatever the length of each name. The rows stay live.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Slab\n\nL: ? 6 m [2 .. 12 step 0.5] | span\nw_d: ? 12 kN/m [2 .. 40 step 1] | design load\nt_slab: ? 200 mm [100 .. 400 step 10] | thickness\nf_ck: ? 32 MPa [20 .. 50 step 2] | concrete strength\n\nM: w_d * L^2 / 8 -> kN*m | maximum moment\n';

export default {
    id: 'input-rows-line-up',
    viewport: { width: 1280, height: 720 },
    crop: { x: 0, y: 215, w: 1000, h: 400 },
    async run(rec) {
        const { page } = rec;
        // the rows of the form itself, not the docked panel
        await page.evaluate(() => { try { sessionStorage.setItem('calcpad.inputDock', 'off'); } catch { /* none */ } });
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
        await frameWith(page, 'input.calcpad-ui-slider');
        await sleep(1800);
        await rec.begin();
        await sleep(2200);
        // the span slider from 6 m to 10 m: the moment follows
        const drag = async (from, to, ms) => {
            const f = await frameWith(page, 'input.calcpad-ui-slider');
            const b = await f.locator('input.calcpad-ui-slider').first().boundingBox();
            const at = (v) => b.x + 8 + (b.width - 16) * (v - 2) / 10;
            const y = b.y + b.height / 2;
            await rec.glide(at(from), y, 650);
            await sleep(250);
            await page.mouse.down();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, true), [at(from), y]);
            await rec.glide(at(to), y, ms);
            await page.mouse.up();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, false), [at(to), y]);
        };
        await drag(6, 10, 1500);
        await rec.waitFor(/= 150 kN/);
        rec.mark('poster');
        await sleep(1200);
        // a value typed into the second row
        const f = await frameWith(page, 'input.calcpad-ui-input');
        const box = f.locator('input.calcpad-ui-input').nth(1);
        await rec.click(box, { ms: 650 });
        await page.keyboard.press('ControlOrMeta+a');
        await rec.type('20', 160);
        await sleep(300);
        await page.keyboard.press('Enter');
        await rec.waitFor(/= 250 kN/);
        await rec.glide(900, 560, 500);
        await sleep(1800);
        await rec.finish();
    },
};
