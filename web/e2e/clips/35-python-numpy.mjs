// 26.10.19: numpy.polyfit and polyval in a Python block (and meshgrid, interp, hypot, arctan2, outer). The fit of a
// measured series is drawn on top of its points once the polyval line is typed.
import { sleep } from './lib.mjs';

const START = `<!--sheet-->
# Fit a test series

\`\`\`python
import numpy as np
import matplotlib.pyplot as plt

load = np.array([0, 1, 2, 3, 4, 5, 6])
defl = np.array([0.2, 1.1, 3.9, 9.2, 15.8, 25.3, 36.1])

c = np.polyfit(load, defl, 2)
t = np.linspace(0, 6, 61)

plt.figure(figsize=(7, 3))
plt.scatter(load, defl, label="tests")
plt.xlabel("load (kN)")
plt.ylabel("deflection (mm)")
plt.grid(True)
plt.legend()
plt.show()
export(c2=float(c[0]))
\`\`\`

C_2: c2 * 1 | quadratic term
`;
const PLOT_LINE = 18;

export default {
    id: 'python-numpy',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        for (let i = 0; i < 200; i++) { const y = page.locator('#cprs-python-yes'); if (await y.count()) { await y.first().click(); break; } await sleep(300); }
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 17, lineHeight: 26, quickSuggestions: false, suggestOnTriggerCharacters: false, autoClosingBrackets: 'never' }));
        await rec.waitFor(/quadratic term/);
        await rec.waitFor(/0\.9\d*/);
        await page.evaluate((n) => { const e = window.calcpadActiveEditor; const m = e.getModel(); e.focus(); e.revealLineInCenter(n); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); }, PLOT_LINE);
        await rec.begin();
        await sleep(1500);
        // the fitted curve joins the points
        await rec.type('\nplt.plot(t, np.polyval(c, t), "r-", label="quadratic fit")', 30);
        await rec.waitFor(/quadratic term/);
        await sleep(600);
        rec.mark('poster');
        await sleep(2400);
        await rec.finish();
    },
};
