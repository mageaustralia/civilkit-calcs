// 26.10.20: the quick reference page at /calcs/reference/: the sheet language on one printable page.
import { sleep } from './lib.mjs';

export default {
    id: 'quick-reference',
    viewport: { width: 1280, height: 720 },
    async run(rec, { url }) {
        const { page } = rec;
        // The page links its stylesheet as /calcs/learn/learn.css (the live site's path); the local build is served at the root.
        const base = new URL(url);
        await page.route('**/calcs/**', (route) => route.continue({ url: new URL(new URL(route.request().url()).pathname.replace(/^\/calcs\//, '/'), base).toString() }));
        await page.goto(new URL('reference/', url).toString());
        await page.waitForSelector('h1');
        await sleep(1500);
        await rec.begin();
        await sleep(1800);
        rec.mark('poster');
        for (let i = 0; i < 6; i++) {
            for (let s = 0; s < 16; s++) {
                await page.evaluate(() => window.scrollBy(0, 34));
                await sleep(40);
            }
            await sleep(900);
        }
        await rec.finish();
    },
};
