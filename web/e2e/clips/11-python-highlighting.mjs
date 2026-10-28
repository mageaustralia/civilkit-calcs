// 26.10.7: a Python block in a sheet is coloured with the Python grammar while it is typed.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Moment\n\n```python\nw = 12\nL = 6\n```\n';

export default {
    id: 'python-highlighting',
    crop: { x: 0, y: 190, w: 700, h: 400 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        // the first run in a tab asks for consent
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 21, lineHeight: 32, quickSuggestions: false, suggestOnTriggerCharacters: false, autoClosingBrackets: 'never', autoIndent: 'none' }));
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); const m = e.getModel(); e.setPosition({ lineNumber: 6, column: m.getLineMaxColumn(6) }); });
        await rec.begin();
        await sleep(500);
        await rec.type('\ndef moment(w, L):\n', 55);
        await rec.type('    return w * L**2 / 8  # kN.m\n', 55);
        await rec.type('print("M =", moment(w, L))', 55);
        rec.mark('poster');
        await sleep(1100);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await sleep(300);
        await rec.finish();
    },
};
