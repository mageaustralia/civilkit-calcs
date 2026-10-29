// Unreleased: a range in square brackets after an input's value, `L: ? 6 m [2 .. 12 step 0.5] | span`, gives it a slider in
// Input mode; dragging it recalculates the moment below as it moves.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam\n\nw: 12 kN/m | load\n\nL: ? 6 m [2 .. 12 step 0.5] | span\n\nM: w * L^2 / 8 -> kN*m | maximum moment\n';

export default {
    id: 'input-slider',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/maximum moment/);
        // the slider line selected in the source, so it reads first
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setSelection({ startLineNumber: 6, startColumn: 1, endLineNumber: 6, endColumn: 100 }); });
        await sleep(1500);
        await rec.begin();
        await sleep(1200);
        // Input mode: the line becomes a number box with a slider beside it
        await rec.click(page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }), { ms: 700 });
        await rec.skip(async () => { await frameWith(page, 'input.calcpad-ui-slider, .calcpad-ui-slider input'); await sleep(600); });
        await rec.waitFor(/54 kN/);
        await sleep(700);
        // press on the thumb and drag; the moment follows while the slider moves (looked up fresh: the form re-renders)
        const drag = async (from, to, ms) => {
            const f = await frameWith(page, 'input.calcpad-ui-slider, .calcpad-ui-slider input');
            const b = await f.locator('input.calcpad-ui-slider, .calcpad-ui-slider input').first().boundingBox();
            const at = (v) => b.x + 8 + (b.width - 16) * (v - 2) / 10;
            const y = b.y + b.height / 2;
            await rec.glide(at(from), y, 600);
            await sleep(250);
            await page.mouse.down();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, true), [at(from), y]);
            await rec.glide(at(to), y, ms);
            await page.mouse.up();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, false), [at(to), y]);
        };
        await drag(6, 10, 1600);
        await rec.waitFor(/= 150 kN/);
        rec.mark('poster');
        await sleep(1300);
        await drag(10, 3, 2000);
        await rec.waitFor(/= 13\.5 kN/);
        await sleep(1400);
        // back to the source view, the starting text, so the clip loops
        await rec.click(page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Preview$/ }), { ms: 600 });
        // leaving Input mode asks whether to keep the values typed in the form
        const dontSave = page.getByRole('button', { name: "Don't Save" });
        await dontSave.waitFor({ timeout: 10000 });
        await sleep(500);
        await rec.click(dontSave, { ms: 500 });
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); e.setSelection({ startLineNumber: 6, startColumn: 1, endLineNumber: 6, endColumn: 100 }); }, START);
        await rec.waitFor(/= 54 kN/);
        await rec.glide(640, 650, 400);
        await sleep(500);
        await rec.finish();
    },
};
