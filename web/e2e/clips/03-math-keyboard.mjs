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
        await sleep(400);
        await rec.click(page.locator('.status-kbd'), { ms: 600 });
        await page.waitForSelector('.mk');
        await sleep(300);
        const key = (label) => page.locator('.mk-body button', { hasText: new RegExp(`^${label}$`) }).first();
        const press = async (label) => { await rec.click(key(label), { ms: 170 }); };
        for (const k of ['y', '=', 'π', '×', '2', 'xʸ', '2']) await press(k);
        await rec.waitFor(/y = .*12\.57/);
        rec.mark('poster');
        await sleep(900);
        await rec.click(page.locator('.status-kbd'), { ms: 500 });
        await page.waitForSelector('.mk', { state: 'detached' });
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); }, START);
        await rec.waitGone(/12\.57/);
        await sleep(500);
        await rec.finish();
    },
};
