// 26.10.13: select an expression in a prose line of a sheet and press = Formula on the prose toolbar (or Ctrl+Shift+E);
// it becomes a named formula line after the paragraph, the new name is selected to type over, and the preview works it out.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam\n\nw: 12 kN/m | load\n\nL: 6 m | span\n\nThe moment is w*L^2/8 at midspan.\n';
const NEEDLE = 'w*L^2/8';

export default {
    id: 'quick-convert',
    crop: { x: 0, y: 190, w: 1000, h: 414 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        // the divider to the left: the results get the room the short source lines do not need
        await page.mouse.move(702, 450);
        await page.mouse.down();
        await page.mouse.move(620, 450, { steps: 5 });
        await page.mouse.move(540, 450, { steps: 5 });
        await page.mouse.up();
        await rec.waitFor(/moment is/);
        await frameWith(page, '[data-source-line]');
        await sleep(1500);
        await rec.begin();
        await sleep(600);
        // drag across the expression in the source
        const span = await page.evaluate((n) => {
            const e = window.calcpadActiveEditor;
            const lines = e.getModel().getLinesContent();
            const line = lines.findIndex((l) => l.includes(n)) + 1;
            const c = lines[line - 1].indexOf(n) + 1;
            const r = e.getDomNode().getBoundingClientRect();
            const a = e.getScrolledVisiblePosition({ lineNumber: line, column: c });
            const z = e.getScrolledVisiblePosition({ lineNumber: line, column: c + n.length });
            return { x0: r.x + a.left + 1, x1: r.x + z.left - 1, y: r.y + a.top + a.height / 2 };
        }, NEEDLE);
        await rec.glide(span.x0, span.y, 700);
        await page.mouse.down();
        await page.evaluate(([x, y]) => window.__clipPtr?.(x, y, true), [span.x0, span.y]);
        await rec.glide(span.x1, span.y, 650);
        await page.mouse.up();
        await page.evaluate(([x, y]) => window.__clipPtr?.(x, y, false), [span.x1, span.y]);
        const sel = await page.evaluate(() => { const e = window.calcpadActiveEditor; return e.getModel().getValueInRange(e.getSelection()); });
        if (sel !== NEEDLE) throw new Error(`selected "${sel}", not the expression`);
        const btn = page.locator('.prose-toolbar .pt-formula');
        await btn.waitFor({ state: 'visible', timeout: 5000 });
        await sleep(700);
        await rec.click(btn, { ms: 500 });
        await page.waitForFunction(() => /^res1: w\*L\^2\/8$/m.test(window.calcpadActiveEditor.getValue()));
        await sleep(700);
        // the new name is selected: type over it
        await rec.type('Mmid', 120);
        await rec.waitFor(/Mmid.*54/);
        rec.mark('poster');
        await rec.glide(820, 520, 500);
        await sleep(2000);
        // back to the starting text, so the clip loops
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); e.setPosition({ lineNumber: 1, column: 1 }); }, START);
        await rec.waitGone(/Mmid/);
        await sleep(500);
        await rec.finish();
    },
};
