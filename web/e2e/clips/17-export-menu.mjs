// 26.10.1: Word export: File, Export, Save Word starts the download (the file is not opened).
import { openSample, sleep } from './lib.mjs';

export default {
    id: 'export-menu',
    crop: { x: 0, y: 30, w: 1280, h: 640 },
    async run(rec) {
        const { page } = rec;
        await openSample(rec, /^Beams/, /Cantilever: Uniformly Distributed Load/);
        await page.click('[data-ribbon-btn=x-sidebar]').catch(() => {});
        await rec.waitFor(/[Cc]antilever/);
        await rec.begin();
        await sleep(300);
        await rec.click(page.locator('[data-ribbon-btn=file]'), { ms: 600 });
        await sleep(600);
        await rec.click(page.getByText('Export...', { exact: true }), { ms: 600 });
        await page.waitForSelector('.as-dialog');
        await sleep(900);
        const saved = page.waitForEvent('download', { timeout: 60000 }).catch(() => null);
        await rec.click(page.locator('.as-dialog').getByText('Save Word…').first(), { ms: 700 });
        const dl = await rec.skip(() => saved, 1.0);
        if (!dl) throw new Error('Save Word did not start a download');
        rec.mark('poster');
        await sleep(1200);
        await page.keyboard.press('Escape');
        await sleep(500);
        await sleep(300);
        await rec.finish();
    },
};
