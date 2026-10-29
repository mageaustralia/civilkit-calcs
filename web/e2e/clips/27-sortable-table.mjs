// 26.10.13: a click on a column heading of a sheet's table in the results sorts it ascending, a second click descending,
// a third puts the rows back in the source order. The source text is not changed.
import { frameWith, sleep } from './lib.mjs';

const START = [
    '<!--sheet-->',
    '# Member loads',
    '',
    '| Member | Span | Load |',
    '|---|---|---|',
    '| B2 | 6 m | 10 kN |',
    '| A1 | 4.5 m | 2.5 kN |',
    '| C3 | 3 m | 7 kN |',
    '| A4 | 8 m | 12.5 kN |',
    '| B5 | 5 m | 4 kN |',
    '',
].join('\n');

/** The Load column as shown, top to bottom. */
async function loads(page) {
    const f = await frameWith(page, 'th .ck-sort');
    return f.locator('tbody tr').evaluateAll((rows) => rows.map((r) => r.cells[2].textContent.trim()).join(','));
}

export default {
    id: 'sortable-table',
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
        await rec.waitFor(/Member loads/);
        await frameWith(page, 'th .ck-sort');
        await sleep(2000);
        await rec.begin();
        await sleep(700);
        const press = async (label, want) => {
            const f = await frameWith(page, 'th .ck-sort');
            await rec.click(f.locator('th .ck-sort', { hasText: label }), { ms: 550 });
            await rec.skip(async () => { for (let i = 0; i < 60 && (await loads(page)) !== want; i++) await sleep(100); });
            if ((await loads(page)) !== want) throw new Error(`after ${label}: ${await loads(page)}, not ${want}`);
        };
        await press('Load', '2.5 kN,4 kN,7 kN,10 kN,12.5 kN');
        await sleep(1500);
        await press('Load', '12.5 kN,10 kN,7 kN,4 kN,2.5 kN');
        rec.mark('poster');
        await sleep(1500);
        await press('Load', '10 kN,2.5 kN,7 kN,12.5 kN,4 kN');
        await sleep(1300);
        if (await page.evaluate(() => window.calcpadActiveEditor.getValue()) !== START) throw new Error('the source changed');
        await rec.glide(640, 650, 500);
        await sleep(500);
        await rec.finish();
    },
};
