// 26.10.19: the gear at the right end of the top strip opens Settings from any tab; the dialog has General, Calculation,
// Editor, Export and Advanced tabs, and its search box looks through all of them.
import { sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Slab load\n\nA: 36 m^2 | floor area\nq: 3 kPa | live load\nP: q*A -> kN | total load\n';
const DLG = '.tab-content.as-dialog';

export default {
    id: 'settings-tabs',
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30, quickSuggestions: false, suggestOnTriggerCharacters: false }));
        await page.click('[data-ribbon-tab=insert]').catch(() => {});
        await rec.waitFor(/total load/);
        await sleep(1200);
        await rec.begin();
        await sleep(1200);
        // the gear, from a tab that has no Settings button of its own
        await rec.click(page.locator('[data-ribbon-btn=q-settings]'), { ms: 800 });
        await page.locator(`${DLG} [role=tablist]`).waitFor({ timeout: 15000 });
        await sleep(1300);
        for (const id of ['calculation', 'editor', 'export', 'advanced']) {
            await rec.click(page.locator(`${DLG} [role=tab][data-settings-tab=${id}]`), { ms: 500 });
            await sleep(1000);
            if (id === 'calculation') rec.mark('poster');
        }
        // the search looks through every tab at once
        await rec.click(page.locator(`${DLG} .search-input`), { ms: 600 });
        await rec.type('decimals', 110);
        await sleep(1800);
        await page.keyboard.press('Escape');
        await page.locator(DLG).waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
        await sleep(600);
        await rec.finish();
    },
};
