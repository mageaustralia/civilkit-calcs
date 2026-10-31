// 26.10.20: values entered in the Input form can be written into the source. A mark at the end of an input line shows where
// the form and the source disagree; Update defaults writes the values into the lines; Reset drops them.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam\n\nL: ? 6 m [2 .. 12 step 0.5] | span\nw: ? 12 kN/m | design load\nE: ? [Steel = 200 GPa, *Timber = 11 GPa] | material\n\nM: w * L^2 / 8 -> kN*m | maximum moment\n';

export default {
    id: 'form-values-in-source',
    viewport: { width: 1280, height: 720 },
    crop: { x: 0, y: 140, w: 1280, h: 460 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate(() => { try { sessionStorage.setItem('calcpad.inputDock', 'off'); } catch { /* none */ } });
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 17, lineHeight: 28 }));
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
        await frameWith(page, 'input[data-ui-var^="L:"]');
        await sleep(1500);
        await rec.begin();
        await sleep(1500);
        const enter = async (name, value) => {
            const f = await frameWith(page, `input[data-ui-var^="${name}:"]`);
            await rec.click(f.locator(`input[data-ui-var^="${name}:"]`), { ms: 700 });
            await page.keyboard.press('ControlOrMeta+a');
            await rec.type(value, 160);
            await page.keyboard.press('Enter');
            await sleep(1500);
        };
        const input = () => rec.click(page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }), { ms: 700 });
        // Exit input mode asks whether to keep values entered in the form: Save keeps them (as marks on the lines)
        const exit = async () => {
            await rec.click(page.locator('button', { hasText: 'Exit input mode' }), { ms: 700 });
            const save = page.locator('button', { hasText: /^Save$/ });
            if (await save.waitFor({ timeout: 2500 }).then(() => true, () => false)) { await sleep(900); await rec.click(save, { ms: 600 }); }
        };
        // values typed into the form...
        await enter('L', '8');
        await rec.waitFor(/= 96 kN/);
        await enter('w', '15');
        await rec.waitFor(/= 120 kN/);
        await sleep(700);
        // ...show as a mark on the source lines
        await exit();
        await sleep(2600);
        rec.mark('poster');
        // Update defaults writes them into the lines
        await input();
        await frameWith(page, 'input[data-ui-var^="L:"]');
        await sleep(1000);
        await rec.click(page.locator('button', { hasText: 'Update defaults' }), { ms: 800 });
        await sleep(1500);
        await exit();
        await sleep(2800);
        // a new value, then Reset drops it and the form is back to the source
        await input();
        await frameWith(page, 'input[data-ui-var^="L:"]');
        await sleep(1000);
        await enter('w', '20');
        await rec.waitFor(/= 160 kN/);
        await sleep(900);
        await rec.click(page.locator('button', { hasText: /^Reset$/ }), { ms: 800 });
        await rec.waitFor(/= 120 kN/);
        await sleep(2000);
        await rec.finish();
    },
};
