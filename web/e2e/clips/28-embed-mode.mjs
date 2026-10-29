// Unreleased: the app at ?embed=1&sample=template-simply-supported-beam shows one sample as a compact form beside its
// results, the view a host page puts in an iframe. Dragging the span slider works the moment and the checks out again.
import { sleep } from './lib.mjs';

const SAMPLE = 'template-simply-supported-beam';

/** M_max from the results, as the engine writes it ("Mmax = ... = 56.26 kN · m"). */
async function mmax(page) {
    const t = await page.frameLocator('.ce-frame').locator('body').innerText({ timeout: 30000 }).catch(() => '');
    return /Mmax = .*?= ([\d.]+) kN · m/.exec(t.replace(/\s+/g, ' '))?.[1] ?? null;
}
const calculated = (page) => page.waitForFunction(() => /^Calculated/.test(document.querySelector('.ce-status')?.textContent ?? ''), null, { timeout: 120000 });

export default {
    id: 'embed-mode',
    viewport: { width: 1000, height: 600 },
    async run(rec, { url }) {
        const { page } = rec;
        const u = new URL(url);
        u.searchParams.set('embed', '1');
        u.searchParams.set('sample', SAMPLE);
        u.searchParams.set('theme', 'light');
        await page.goto(u.toString(), { timeout: 240000 });
        await calculated(page);
        for (let i = 0; i < 120 && (await mmax(page)) !== '56.26'; i++) await sleep(250);
        await sleep(1500);
        await rec.begin();
        await sleep(900);
        // scroll the results down to the moment and the checks; the form stays put beside them
        await rec.glide(640, 330, 500);
        const h = await page.frameLocator('.ce-frame').locator('h2', { hasText: /^Results$/ }).first().boundingBox();
        const steps = Math.round((h.y - 12) / 100);
        for (let i = 0; i < steps; i++) { await page.mouse.wheel(0, 100); await sleep(45); }
        await sleep(900);
        // drag the span slider from 6 m to 9 m (looked up fresh: its box moves when the page scrolls)
        const slider = page.locator('.ce-field[data-name="L"] .ce-slider');
        const drag = async (from, to, ms) => {
            const b = await slider.boundingBox();
            const at = (v) => b.x + 8 + (b.width - 16) * (v - 2) / 10;
            const y = b.y + b.height / 2;
            await rec.glide(at(from), y, 600);
            await sleep(200);
            await page.mouse.down();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, true), [at(from), y]);
            await rec.glide(at(to), y, ms);
            await page.mouse.up();
            await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, false), [at(to), y]);
        };
        await drag(6, 9, 1300);
        await rec.skip(async () => { for (let i = 0; i < 120 && (await mmax(page)) === '56.26'; i++) await sleep(150); await calculated(page); });
        if ((await mmax(page)) === '56.26') throw new Error('the moment did not change');
        rec.mark('poster');
        await sleep(2200);
        // and back to 6 m, so the clip loops
        await drag(9, 6, 1000);
        await rec.skip(async () => { for (let i = 0; i < 120 && (await mmax(page)) !== '56.26'; i++) await sleep(150); await calculated(page); });
        await sleep(600);
        await rec.glide(640, 330, 400);
        for (let i = 0; i < steps; i++) { await page.mouse.wheel(0, -100); await sleep(45); }
        await sleep(700);
        await rec.finish();
    },
};
