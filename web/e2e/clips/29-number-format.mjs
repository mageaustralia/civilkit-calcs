// 26.10.18: Settings > Number format. Space and comma with digit groups shows the results of a sheet as 15 431,25; in Input
// mode the number box shows 12,5 and takes 6,5 typed with a comma.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Slab load\n\nA: 1234.5 m^2 | floor area\n\nq: ? 12.5 kN/m^2 | load\n\nP: q*A -> kN | total load\n';

export default {
    id: 'number-format',
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await page.click('[data-ribbon-tab=calc]');
        await rec.waitFor(/total load/);
        await rec.waitFor(/15431\.25/);
        await sleep(1500);
        await rec.begin();
        await sleep(1300);
        // Settings, on the Calculation tab
        await rec.click(page.locator('[data-ribbon-btn=k-settings]').first(), { ms: 650 });
        await page.locator('#cprs-numfmt-choice').waitFor({ timeout: 15000 });
        await sleep(500);
        await sleep(400);
        const choice = page.locator('#cprs-numfmt-choice');
        await rec.click(choice, { ms: 600 });
        await sleep(300);
        await choice.selectOption('space');
        await sleep(900);
        await rec.click(page.locator('#cprs-numfmt-group'), { ms: 500 });
        await sleep(1000);
        // close Settings: the results are in the new style
        await page.keyboard.press('Escape');
        await page.locator('.tab-content.as-dialog').waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
        await rec.waitFor(/15 431,25/);
        await rec.glide(900, 420, 500);
        await sleep(1600);
        rec.mark('poster');
        // Input mode: the box shows 12,5 and takes 6,5
        await rec.click(page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }), { ms: 650 });
        const f = await rec.skip(async () => { const fr = await frameWith(page, 'input.calcpad-ui-input'); await sleep(600); return fr; });
        await sleep(700);
        const box = f.locator('input.calcpad-ui-input').first();
        await rec.click(box, { ms: 600 });
        await page.keyboard.press('ControlOrMeta+a');
        await rec.type('6,5', 180);
        await sleep(300);
        await page.keyboard.press('Enter');
        await rec.waitFor(/8 024,25/);
        await rec.glide(900, 520, 500);
        await sleep(2000);
        await rec.finish();
    },
};
