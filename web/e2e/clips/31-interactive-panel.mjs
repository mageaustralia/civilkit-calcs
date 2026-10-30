// 26.10.18: the "Beam deflection explorer (panel)" sample. Its Python block draws a panel of sliders and a drop-down in the
// results; moving the Span slider and choosing Bending moment re-run only the panel's function, and the chart and table
// are redrawn in place.
import { settle, sleep, widenResults } from './lib.mjs';

/** The front result frame that holds the drawn panel (the preview is double buffered). */
async function panelFrame(page, ms = 120000) {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const front = await f.frameElement().then((e) => e.evaluate((x) => !x.classList.contains('preview-frame-back'))).catch(() => false);
            if (front && await f.locator('.cprs-panel .cprs-w').count().catch(() => 0)) return f;
        }
        await sleep(300);
    }
    throw new Error('the panel was not drawn');
}
const tableText = (fr) => fr.locator('.cprs-panel-table').first().innerText().catch(() => '');
async function until(fn, ms = 60000) {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) { if (await fn().catch(() => false)) return true; await sleep(150); }
    return false;
}

export default {
    id: 'interactive-panel',
    viewport: { width: 1280, height: 900 },
    crop: { x: 360, y: 190, w: 920, h: 710 },
    async run(rec, { url }) {
        const { page } = rec;
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await widenResults(page);
        const u = new URL(url);
        u.searchParams.set('sample', 'panels-beam');
        u.searchParams.set('theme', 'light');
        await page.goto(u.toString(), { timeout: 240000 });
        await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 240000 });
        await page.waitForFunction(() => /import ui/.test(window.calcpadActiveEditor.getValue()), null, { timeout: 60000 });
        if (await page.locator('.sidebar:visible, [data-sidebar]:visible').count().catch(() => 0)) {
            await page.click('[data-ribbon-tab=resources]').catch(() => {});
        }
        const yes = page.locator('#cprs-python-yes').first();
        await yes.waitFor({ state: 'visible', timeout: 120000 });
        await sleep(1500);
        await rec.begin();
        await sleep(1200);
        // the sheet asks before it runs Python
        await rec.click(yes, { ms: 650 });
        // the Python worker starts and the panel is drawn (a wait of several seconds, cut from the clip)
        const fr = await rec.skip(async () => {
            const f = await panelFrame(page);
            await until(() => f.locator('.cprs-panel .js-plotly-plot').count().then((n) => n > 0), 60000);
            await until(async () => (await tableText(f)).includes('54.00 kN.m'), 60000);
            await f.evaluate(() => { const p = document.querySelector('.cprs-panel'); document.scrollingElement.scrollTop += p.getBoundingClientRect().top - 8; });
            await settle(page);
            await sleep(800);
            return f;
        }, 0.3);
        await sleep(1200);
        // drag the Span slider from 6 m to 10 m
        const span = fr.locator('.cprs-w', { hasText: 'Span' }).locator('input[type=range]').first();
        const b = await span.boundingBox();
        const at = (v) => b.x + 8 + (b.width - 16) * (v - 2) / 10;
        const y = b.y + b.height / 2;
        await rec.glide(at(6), y, 700);
        await sleep(250);
        await page.mouse.down();
        await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, true), [at(6), y]);
        await rec.glide(at(10), y, 1600);
        await page.mouse.up();
        await page.evaluate(([x, yy]) => window.__clipPtr?.(x, yy, false), [at(10), y]);
        await rec.skip(() => until(async () => !(await tableText(fr)).includes('54.00 kN.m'), 30000), 0.4);
        if ((await tableText(fr)).includes('54.00 kN.m')) throw new Error('the table did not follow the slider');
        await sleep(1500);
        // Show: Bending moment
        const show = fr.locator('.cprs-w select').first();
        await rec.click(show, { ms: 600 });
        await sleep(300);
        await show.selectOption('1');
        await rec.skip(() => until(() => fr.locator('.cprs-panel .js-plotly-plot').evaluate((g) => g.layout.yaxis.title.text === 'Bending moment (kN.m)'), 30000), 0.4);
        await sleep(400);
        rec.mark('poster');
        await rec.glide(b.x + b.width + 120, y + 260, 500);
        await sleep(2200);
        await rec.finish();
    },
};
