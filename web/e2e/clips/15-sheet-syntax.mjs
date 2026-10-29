// 26.10.3: sheet syntax: `name: expression -> unit | note` lines render as typeset quantities; the Samples dialog has sheet samples.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Slab load\n\n';

export default {
    id: 'sheet-syntax',
    crop: { x: 0, y: 40, w: 1280, h: 640 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false, autoClosingBrackets: 'never' }));
        await rec.waitFor(/Slab load/);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.focus(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
        await rec.begin();
        await sleep(300);
        await rec.type('t: 200 mm | thickness\n', 26);
        await rec.type('q: t * 25 kN/m^3 -> kPa | self weight', 26);
        await rec.waitFor(/self weight/);
        rec.mark('poster');
        await sleep(800);
        // the Samples dialog opens with the sheet samples
        await rec.click(page.locator('[data-ribbon-btn=x-samples]'), { ms: 500 });
        await page.waitForSelector('.cprs-s-group');
        await sleep(350);
        await rec.click(page.locator('.cprs-s-group', { hasText: /^Sheet syntax/ }), { ms: 400 });
        await sleep(1100);
        await page.keyboard.press('Escape');
        await page.waitForFunction(() => !document.querySelector('.cprs-s-group'), null, { timeout: 10000 }).catch(() => {});
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/self weight/);
        await sleep(400);
        await rec.finish();
    },
};
