// 26.10.12: type `L: ? 6 m | span` in a sheet and the form shows a number box for it.
import { frameWith, resultHas, settle, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam moment\n\nw: 12 kN/m | load\nL: 6 m | span\nM: w * L^2 / 8 -> kN*m | moment\n';

export default {
    id: 'typing-an-input-line',
    crop: { x: 0, y: 190, w: 1100, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        // a larger editor font and no sidebar, so the line being typed is legible
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await resultHas(page, /Beam moment/);
        await settle(page);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 5, column: 1 }); });
        await page.keyboard.press('Shift+End');
        rec.mark('start');
        await sleep(500);
        await rec.type('L: ? 6 m | span', 60);
        await rec.waitFor(/L = 6 m/);
        rec.mark('poster');
        await sleep(700);
        const input = page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ });
        await rec.click(input, { ms: 700 });
        await rec.skip(async () => { await frameWith(page, 'input.calcpad-ui-input'); await settle(page); });
        await sleep(1300);
        await rec.click(page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Preview$/ }), { ms: 500 });
        await sleep(300);
        // back to the starting text, so the clip loops
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.focus(); e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitFor(/L = 6 m/);
        await sleep(700);
        await rec.finish();
    },
};
