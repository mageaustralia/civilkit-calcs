// 26.10.4: a Python block in a sheet computes a number and exports it; the lines after it use it.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Bending moment\n\n```python\nw = 12\nL = 6\nM = w * L**2 / 8\nexport(M_py=M)\n```\n\nM_d: M_py * 1.5 | design moment\n';

export default {
    id: 'python-export-values',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await rec.waitFor(/design moment/);
        await rec.waitFor(/81/);
        await page.evaluate(() => window.calcpadActiveEditor.focus());
        await rec.begin();
        await sleep(700);
        // change L from 6 to 8 in the Python block: the line after the block follows
        const sel = (line) => page.evaluate((n) => { const e = window.calcpadActiveEditor; const m = e.getModel(); const t = m.getLineContent(n); e.focus(); e.setSelection({ startLineNumber: n, startColumn: t.indexOf('=') + 3, endLineNumber: n, endColumn: t.length + 1 }); }, line);
        await sel(6);
        await sleep(500);
        await rec.type('8', 120);
        await rec.waitFor(/144/);
        rec.mark('poster');
        await sleep(1700);
        await sel(6);
        await rec.type('6', 120);
        await rec.waitFor(/81/);
        await sleep(900);
        await rec.finish();
    },
};
