// Unreleased: a ```mermaid fence in a sheet is drawn as a diagram in the results; typing a new step into its text redraws it,
// and the calculation under it is worked out as usual. (A shorter version of the sample "Design check as a flow diagram".)
import { frameWith, sleep } from './lib.mjs';

const START = [
    '<!--sheet-->',
    '# Design check as a flow diagram',
    '',
    '```mermaid',
    'flowchart LR',
    '  A[Loads w, L] --> B[Moment M*]',
    '  B --> C{M* <= phi Ms?}',
    '  C -- yes --> D[Passes]',
    '```',
    '',
    'L: 6 m | span',
    'w: 12 kN/m | design load',
    'Mstar: w * L^2 / 8 -> kN*m | design moment',
    'phiMs: 0.9 * 300 MPa * 1.23e6 mm³ -> kN*m | design capacity',
    'if Mstar <= phiMs: "OK: the section passes." else: "Fails: increase the section."',
    '',
].join('\n');

/** The number of boxes in the diagram on screen. */
async function nodes(page) {
    const f = await frameWith(page, '.sheet-mermaid svg');
    return f.locator('.sheet-mermaid svg .node').count();
}

export default {
    id: 'mermaid-diagram',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        // the divider a little to the left: the diagram gets more room, the short source lines still fit
        await page.mouse.move(702, 450);
        await page.mouse.down();
        await page.mouse.move(640, 450, { steps: 5 });
        await page.mouse.move(575, 450, { steps: 5 });
        await page.mouse.up();
        await rec.waitFor(/section passes/);
        await frameWith(page, '.sheet-mermaid svg');
        await sleep(1500);
        await rec.begin();
        await sleep(1000);
        // a new step at the end of the diagram's text: the "no" branch
        const end = await page.evaluate(() => {
            const e = window.calcpadActiveEditor;
            const p = e.getScrolledVisiblePosition({ lineNumber: 8, column: e.getModel().getLineMaxColumn(8) });
            const r = e.getDomNode().getBoundingClientRect();
            return { x: r.x + p.left, y: r.y + p.top + p.height / 2 };
        });
        await rec.clickAt(end.x + 4, end.y, 800);
        await page.evaluate(() => { const e = window.calcpadActiveEditor; e.focus(); e.setPosition({ lineNumber: 8, column: e.getModel().getLineMaxColumn(8) }); });
        await rec.glide(end.x + 160, end.y + 40, 400);
        await page.keyboard.press('Enter');
        await rec.type('C -- no --> E[Deeper section]', 75);   // Enter keeps the indent
        await rec.skip(async () => { for (let i = 0; i < 120 && (await nodes(page)) < 5; i++) await sleep(250); });
        await sleep(1400);
        // and back round to the moment check
        await page.keyboard.press('Enter');
        await rec.type('E --> B', 90);
        await rec.skip(async () => { await sleep(1500); await frameWith(page, '.sheet-mermaid svg'); });
        rec.mark('poster');
        await sleep(2200);
        // back to the starting text, so the clip loops
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); e.setPosition({ lineNumber: 1, column: 1 }); }, START);
        await rec.skip(async () => { for (let i = 0; i < 120 && (await nodes(page)) > 4; i++) await sleep(250); });
        await rec.glide(640, 650, 400);
        await sleep(600);
        await rec.finish();
    },
};
