// 26.10.20: `-> base` shows a result in SI base units (kg, m, s, A, K, mol, cd).
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Moment in base units\n\nF: 2 kN | force\nL: 5 m | lever arm\nM: F * L -> kN*m | moment\n';

export default {
    id: 'base-units',
    viewport: { width: 1280, height: 720 },
    crop: { x: 0, y: 190, w: 1280, h: 400 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false, autoClosingBrackets: 'never' }));
        await rec.waitFor(/moment/);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.focus(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
        await rec.begin();
        await sleep(1800);
        await rec.type('\nM_si: F * L -> base | in SI base units', 30);
        await rec.waitFor(/in SI base units/);
        await rec.waitFor(/kg/);
        rec.mark('poster');
        await sleep(2000);
        await rec.type('\nE: 3 kPa * 4 m^2 -> base | pressure times area', 30);
        await rec.waitFor(/pressure times area/);
        await sleep(2800);
        await rec.finish();
    },
};
