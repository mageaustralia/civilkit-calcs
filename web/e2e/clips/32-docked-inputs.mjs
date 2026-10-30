// 26.10.19: Input mode with Dock inputs on. A sheet with sliders opens with its inputs in a panel beside the results; the
// results are scrolled down to the plot, and dragging a slider in the panel redraws the plot while it stays in view.
import { frameWith, settle, sleep } from './lib.mjs';

const NOTE = 'Notes between the inputs and the plot. They are here so that the plot is far below the inputs.';
const START = `<!--sheet-->
# Wave on a beam

## Inputs
a: ? 2 [1 .. 5 step 0.5] | amplitude
k: ? 1 [0.5 .. 3 step 0.25] | frequency
c: ? 0 [-2 .. 2 step 0.5] | offset

${NOTE}

${NOTE}

${NOTE}

${NOTE}

## The curve
y(x): a * sin(k * x) + c
plot y(x) for x in 0..360
`;

export default {
    id: 'docked-inputs',
    viewport: { width: 1280, height: 900 },
    crop: { x: 0, y: 215, w: 1280, h: 670 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
        const dock = page.locator('.input-dock-host .input-dock');
        await dock.locator('.ce-slider').first().waitFor({ timeout: 90000 });
        await settle(page);
        await sleep(1500);
        // the plot is at the foot of the results: scroll them there
        const plotIntoView = async () => {
            const f = await frameWith(page, 'svg');
            await f.evaluate(() => { const s = [...document.querySelectorAll('svg')].find((x) => x.querySelector('path, polyline')); s?.scrollIntoView({ block: 'center' }); });
        };
        await plotIntoView();
        await sleep(800);
        await rec.begin();
        await sleep(1500);
        const drag = async (name, from, to, ms) => {
            const s = dock.locator(`.ce-field[data-name^="${name}:"] .ce-slider`);
            const b = await s.boundingBox();
            const y = b.y + b.height / 2;
            const at = (f) => b.x + 8 + (b.width - 16) * f;
            await rec.glide(at(from), y, 650);
            await sleep(250);
            await page.mouse.down();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, true), [at(from), y]);
            await rec.glide(at(to), y, ms);
            await page.mouse.up();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, false), [at(to), y]);
        };
        // frequency: the sine gets more waves while the plot stays in view
        await drag('k', 0.2, 0.8, 1900);
        await settle(page);
        await sleep(900);
        rec.mark('poster');
        // amplitude: taller
        await drag('a', 0.25, 0.9, 1500);
        await settle(page);
        await sleep(1800);
        await rec.finish();
    },
};
