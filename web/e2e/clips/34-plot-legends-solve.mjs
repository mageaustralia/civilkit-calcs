// 26.10.19: a plot line takes names with `as "name"` and draws a legend; a quantity line can be solve, integral of or
// slope of. Typed into a sheet with the pump and the pipeline curves: the three numerical lines first, then the legend.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Pump and pipeline\n\nH_p(Q): 40 - 0.01 * Q^2\nH_s(Q): 10 + 0.004 * Q^2\n\n';

export default {
    id: 'plot-legends-solve',
    viewport: { width: 1280, height: 900 },
    crop: { x: 0, y: 190, w: 1280, h: 710 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false, autoClosingBrackets: 'never' }));
        await rec.waitFor(/Pump and pipeline/);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.focus(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
        await rec.begin();
        await sleep(400);
        await rec.type('Q_op: solve H_p(Q) = H_s(Q) for Q in 0..50 | duty point', 24);
        await rec.waitFor(/duty point/);
        await rec.waitFor(/46\.2/);
        await sleep(900);
        await rec.type('\nA: integral of H_p(Q) for Q in 0..30 | area under the curve', 24);
        await rec.waitFor(/area under the curve/);
        await sleep(900);
        await rec.type('\nk: slope of H_p(Q) at Q = 20 | slope at 20', 24);
        await rec.waitFor(/slope at 20/);
        await sleep(900);
        await rec.type('\n\nplot H_p(Q) as "pump", H_s(Q) as "pipeline" for Q in 0..50', 24);
        await rec.waitFor(/pump\s+pipeline|pipeline\s+pump/);
        rec.mark('poster');
        await sleep(2800);
        await rec.finish();
    },
};
