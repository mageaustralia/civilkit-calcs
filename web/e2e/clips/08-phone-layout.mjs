// 26.10.10: the phone layout: one pane at a time, switched from the bar along the bottom.
import { sleep } from './lib.mjs';

export default {
    id: 'phone-layout',
    viewport: { width: 390, height: 844 },
    mobile: true,
    width: 390,
    async run(rec) {
        const { page } = rec;
        await page.locator('.mobile-bar button', { hasText: 'Edit' }).first().waitFor({ timeout: 30000 });
        rec.mark('start');
        await sleep(900);
        for (const label of ['Results', 'Insert', 'Edit']) {
            const b = page.locator('.mobile-bar button', { hasText: label });
            const c = await rec.centre(b);
            await rec.clickAt(c.x, c.y, 450);
            await sleep(1100);
            if (label === 'Results') rec.mark('poster');
        }
        await sleep(300);
        await rec.finish();
    },
};
