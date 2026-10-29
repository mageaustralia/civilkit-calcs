// Unreleased: select words in a paragraph opened in place and a small toolbar floats over them (Bold, then a bulleted list);
// then a math line typed in the source offers the names defined above it.
import { frameWith, sleep } from './lib.mjs';

const START = '<!--sheet-->\n# Beam\n\nThe load is applied at midspan.\n\nw: 12 kN/m | load\nL: 6 m | span\n';

export default {
    id: 'text-toolbar',
    crop: { x: 0, y: 190, w: 1280, h: 530 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        // completion stays on: it is part of the clip
        await page.evaluate(() => window.calcpadActiveEditor.updateOptions({ fontSize: 19, lineHeight: 30 }));
        await rec.waitFor(/applied at midspan/);
        const f = await frameWith(page, '[data-source-line="4"]');
        await sleep(2500);   // the sheet compiler and the grips settle
        await rec.begin();
        // one click opens the paragraph in place
        const b = await f.locator('[data-source-line="4"]').first().boundingBox();
        await rec.glide(b.x + 150, b.y + 10, 600);
        await rec.clickAt(b.x + 150, b.y + 10, 200);
        await page.waitForSelector('.md-block .ProseMirror');
        await sleep(700);
        // drag across "applied at midspan"
        const span = await page.evaluate(() => {
            const p = document.querySelector('.md-block .ProseMirror p');
            const t = document.createTreeWalker(p, NodeFilter.SHOW_TEXT);
            for (let n = t.nextNode(); n; n = t.nextNode()) {
                const i = n.data.indexOf('applied at midspan');
                if (i >= 0) {
                    const r = document.createRange();
                    r.setStart(n, i); r.setEnd(n, i + 1); const a = r.getBoundingClientRect();
                    r.setStart(n, i + 17); r.setEnd(n, i + 18); const z = r.getBoundingClientRect();
                    return { x0: a.x + 1, x1: z.x + z.width, y: a.y + a.height / 2 };
                }
            }
            return null;
        });
        await rec.glide(span.x0, span.y, 450);
        await page.mouse.down();
        await rec.glide(span.x1, span.y, 700);
        await page.mouse.up();
        const bar = page.locator('.md-block .prose-toolbar:not([hidden])');
        await bar.waitFor({ timeout: 5000 });
        await sleep(700);
        await rec.click(bar.locator('[data-act=bold]'), { ms: 450 });
        await page.waitForSelector('.md-block .ProseMirror strong');
        await sleep(800);
        await rec.click(bar.locator('[data-act=bullet]'), { ms: 400 });
        await page.waitForSelector('.md-block .ProseMirror ul li');
        rec.mark('poster');
        await sleep(1100);
        // click away: the paragraph is written back as Markdown
        await rec.clickAt(1100, 650, 550);
        await page.waitForSelector('.md-block', { state: 'detached' }).catch(() => {});
        await rec.skip(() => page.waitForFunction(() => /midspan\*\*/.test(window.calcpadActiveEditor.getValue())));
        await sleep(900);
        // a new math line in the source: typing a letter offers the names defined above
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const m = e.getModel(); const n = m.getLineCount(); e.focus(); e.setPosition({ lineNumber: n, column: m.getLineMaxColumn(n) }); });
        await rec.type('M: w * L^2 / 8 -> kN*m | moment\nV: ', 45);
        await sleep(300);
        await rec.type('w', 90);
        await page.waitForSelector('.monaco-editor .suggest-widget.visible', { timeout: 5000 }).catch(() => {});
        await sleep(1300);
        await page.keyboard.press('Escape');
        await rec.type(' * L / 2 -> kN | shear', 45);
        await rec.waitFor(/V = .*18/);
        await sleep(1200);
        await page.evaluate((t) => { const e = window.calcpadActiveEditor; e.executeEdits('clip', [{ range: e.getModel().getFullModelRange(), text: t }]); e.setPosition({ lineNumber: 1, column: 1 }); }, START);
        await rec.waitGone(/shear/);
        await rec.glide(640, 650, 400);
        await sleep(500);
        await rec.finish();
    },
};
