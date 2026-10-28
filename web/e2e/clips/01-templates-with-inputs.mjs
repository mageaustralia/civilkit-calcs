// 26.10.12: a template opened in Input mode; pick another soil from the drop-down and the verdict below follows.
import { frameWith, openSample, resultHas, settle, sleep } from './lib.mjs';

/** Smooth-scrolls the result frame to the verdict line or back to the Inputs heading. */
async function scrollTo(page, where) {
    const f = await frameWith(page, 'select.calcpad-ui-dropdown');
    await f.evaluate((w) => {
        const el = w === 'verdict'
            ? [...document.querySelectorAll('p')].find((x) => /OK: the largest|Fails: the largest/.test(x.textContent))
            : [...document.querySelectorAll('h2')].find((x) => /Inputs/.test(x.textContent));
        const top = w === 'verdict'
            ? el.getBoundingClientRect().bottom + document.scrollingElement.scrollTop - innerHeight + 40
            : el.getBoundingClientRect().top + document.scrollingElement.scrollTop - 12;
        document.scrollingElement.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
    }, where);
}

export default {
    id: 'templates-with-inputs',
    crop: { x: 320, y: 200, w: 960, h: 520 },
    async run(rec) {
        const { page } = rec;
        await openSample(rec, /^Templates/, /pad footing/);
        await resultHas(page, /Pad footing: bearing pressure check/);
        await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ }).click();
        let f = await frameWith(page, 'select.calcpad-ui-dropdown');
        await resultHas(page, /OK: the largest pressure/);
        await f.evaluate(() => { const h = [...document.querySelectorAll('h2')].find((x) => /Inputs/.test(x.textContent)); document.scrollingElement.scrollTop = h.getBoundingClientRect().top + document.scrollingElement.scrollTop - 12; });
        await sleep(800);
        rec.mark('start');
        await sleep(300);
        const sel = f.locator('select.calcpad-ui-dropdown').first();
        const c = await rec.centre(sel);
        await rec.glide(c.x, c.y, 500);
        await sleep(150);
        await sel.selectOption({ label: 'Soft clay' });          // allowable bearing pressure 80 kPa
        await rec.waitFor(/Fails: the largest pressure/);
        await sleep(300);
        await scrollTo(page, 'verdict');
        await sleep(1100);
        rec.mark('poster');
        await sleep(1000);
        await scrollTo(page, 'inputs');
        await sleep(700);
        f = await frameWith(page, 'select.calcpad-ui-dropdown');
        await f.locator('select.calcpad-ui-dropdown').first().selectOption({ label: 'Medium dense sand' });
        await rec.waitFor(/OK: the largest pressure/);
        await sleep(900);
        await rec.finish();
    },
};
