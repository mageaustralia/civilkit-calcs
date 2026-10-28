// 26.10.12: the on-screen math keyboard on a desktop: open it from the status bar and type a line with it.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Circle area\n\nr: 2 m | radius\n';

export default {
    id: 'math-keyboard',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 22, lineHeight: 34, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/Circle area/);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 5, column: 1 }); });
        rec.mark('start');
        await sleep(200);
        await rec.click(page.locator('.status-kbd'), { ms: 500 });
        await page.waitForSelector('.mk');
        await sleep(150);
        const key = (label) => page.locator('.mk-body button', { hasText: new RegExp(`^${label}$`) }).first();
        const press = async (label) => { await rec.click(key(label), { ms: 80 }); };
        for (const k of ['y', '=', '2', '×', 'π']) await press(k);
        await rec.waitFor(/y = .*6\.28/);
        rec.mark('poster');
        await sleep(1100);
        await rec.click(page.locator('.status-kbd'), { ms: 400 });
        await page.waitForSelector('.mk', { state: 'detached' });
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/6\.28/);
        await sleep(500);
        await rec.finish();
    },
};
