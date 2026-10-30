// 26.10.18: Settings > Language. Choosing Deutsch switches the Settings dialog at once and, once it is closed, the ribbon;
// the clip then switches back to English the same way, so it loops.
import { sleep } from './lib.mjs';

const lang = (page) => page.evaluate(() => document.documentElement.lang);

async function pick(rec, value) {
    const { page } = rec;
    await page.click('[data-ribbon-tab=calc]');
    await rec.click(page.locator('[data-ribbon-btn=k-settings]').first(), { ms: 650 });
    const sel = page.locator('.tab-content.as-dialog #cprs-language-select');
    await sel.waitFor({ timeout: 15000 });
    await sleep(800);
    await rec.click(sel, { ms: 600 });
    await sleep(300);
    await sel.selectOption(value);
    await page.waitForFunction((v) => document.documentElement.lang === v, value, { timeout: 15000 });
    await sleep(1600);
    await page.keyboard.press('Escape');
    await page.locator('.tab-content.as-dialog').waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
    await sleep(400);
}

export default {
    id: 'interface-language',
    async run(rec) {
        const { page } = rec;
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.click('[data-ribbon-tab=calc]');
        await rec.waitFor(/Welcome to CivilKit Calcs/);
        await rec.begin();
        await sleep(1200);
        await pick(rec, 'de');
        if (await lang(page) !== 'de') throw new Error('the page did not switch to German');
        // the ribbon is in German: a look at two tabs
        await rec.click(page.locator('[data-ribbon-tab=plots]'), { ms: 600 });
        await sleep(1300);
        rec.mark('poster');
        await rec.click(page.locator('[data-ribbon-tab=calc]'), { ms: 600 });
        await sleep(1300);
        // and back to English
        await pick(rec, 'en');
        await rec.glide(640, 500, 500);
        await sleep(900);
        await rec.finish();
    },
};
