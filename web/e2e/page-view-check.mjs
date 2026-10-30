// Page view: the Report (and Preview) laid out on pages of the PDF's paper size and margins, with the automatic page
// breaks drawn. A long sheet becomes several A4 pages; no block sits across a page's bottom margin or in the gap between
// pages; a manual page break starts a new page; the button toggles it off and the plain layout returns; the setting is
// kept; a phone fits the page to the width of the pane.   node e2e/page-view-check.mjs [--url http://127.0.0.1:5193/]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5193/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const rows = Array.from({ length: 70 }, (_, i) => `a${i + 1}: ${i + 1} m * 2`).join('\n');
const SHEET = `<!--sheet-->\n# Long report\n\n${rows}\n\n<div style="page-break-after:always"></div>\n\n# After the break\n\nz: a1 + a2\n`;

const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1360, height: 900 } }, false], ['desktop dark', { viewport: { width: 1360, height: 900 }, colorScheme: 'dark' }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!/^(Canceled|Event)/.test(String(e.message))) errors.push(String(e.message)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t), SHEET);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Report$/ }).click();
    const front = async (need) => {
        for (let i = 0; i < 80; i++) {
            const h = await page.$('iframe.preview-frame:not(.preview-frame-back)');
            const f = h && await h.contentFrame();
            if (f && await f.evaluate((n) => !!document.body && document.body.innerText.includes('Long report') && (!n || !!document.getElementById(n)), need).catch(() => false)) return f;
            await page.waitForTimeout(250);
        }
        return null;
    };
    await front();
    const btn = page.locator('.page-view-btn');
    check(await btn.isVisible(), `${name}: the Pages button is in the results bar`);
    check(await btn.getAttribute('aria-pressed') === 'false', `${name}: it starts off`);
    phone ? await btn.tap() : await btn.click();
    const f = await front('cpd-page-view');
    check(!!f, `${name}: turning it on draws the pages`);
    await page.waitForTimeout(800);
    const info = await f.evaluate(() => {
        const host = document.getElementById('cpd-page-view');
        const sheets = [...host.children];
        const stride = sheets[1] ? sheets[1].offsetTop - sheets[0].offsetTop : 0;
        const h = sheets[0].offsetHeight;
        const hostRect = host.getBoundingClientRect(), body = document.body.getBoundingClientRect();
        const scale = hostRect.width / host.offsetWidth;
        // Every block's top and bottom, in page coordinates, against the margins of the page it starts on.
        const bad = [];
        const first = sheets[0].getBoundingClientRect().top;
        for (const el of document.querySelectorAll('body > *:not(#cpd-page-view):not(script):not(style)')) {
            const r = el.getBoundingClientRect();
            if (r.height === 0) continue;
            const top = (r.top - first) / scale, bottom = (r.bottom - first) / scale;
            const p = Math.floor(top / stride);
            const content = (p * stride) + h - 72;
            if (bottom > content + 1 && r.height < h - 144) bad.push(el.innerText.slice(0, 30));
            if (top > p * stride + h - 0.5 && top < (p + 1) * stride) bad.push('in gap: ' + el.innerText.slice(0, 30));
        }
        const labels = sheets.map((s) => s.lastElementChild.textContent);
        return { pages: sheets.length, ratio: sheets[0].offsetHeight / sheets[0].offsetWidth, bad, labels, scale, texts: [...document.querySelectorAll('h1')].map((e) => ({ t: e.innerText, top: (e.getBoundingClientRect().top - first) / scale, stride })) };
    });
    const ink = await f.evaluate(() => { const e = document.querySelector('body > p.line'); return e ? getComputedStyle(e).color : ''; });
    check(/^rgb\(0, 0, 0\)$/.test(ink) || /^rgb\((\d+), \1, \1\)$/.test(ink) && Number(/\d+/.exec(ink)[0]) < 80, `${name}: the text is dark on the white page (${ink})`);
    check(info.pages >= 3, `${name}: a long sheet is several pages (${info.pages})`);
    check(Math.abs(info.ratio - 279.4 / 215.9) < 0.01, `${name}: a page has the proportions of the PDF setting, Letter by default (${info.ratio.toFixed(3)})`);
    check(info.bad.length === 0, `${name}: no block crosses a bottom margin or sits in a gap${info.bad[0] ? ': ' + info.bad.slice(0, 3).join(' | ') : ''}`);
    check(info.labels[0] === `Page 1 of ${info.pages}`, `${name}: pages are labelled (${info.labels[0]})`);
    const after = info.texts.find((x) => /After the break/.test(x.t));
    check(!!after && (() => { const at = ((after.top % after.stride) + after.stride) % after.stride; return at >= 70 && at < 72 + 60; })(), `${name}: the manual page break starts a new page (heading at ${after && Math.round(after.top % after.stride)} px of its page)`);
    if (phone) check(info.scale < 0.6, `${name}: the page is fitted to the pane (scale ${info.scale.toFixed(2)})`);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);
    check(overflow || !phone, `${name}: the host page has no sideways scroll`);
    // Off again: the plain layout.
    phone ? await btn.tap() : await btn.click();
    await page.waitForTimeout(2500);
    const g = await page.$('iframe.preview-frame:not(.preview-frame-back)').then((h) => h.contentFrame());
    check(await g.evaluate(() => !document.getElementById('cpd-page-view')), `${name}: turning it off removes the pages`);
    phone ? await btn.tap() : await btn.click();
    await page.reload();
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    check(await page.evaluate(() => localStorage.getItem('calcpad.pageView')) === 'on', `${name}: the choice is kept`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
// The document's own PDF settings (a pdf comment) set the page: A4 landscape here.
{
    const ctx = await browser.newContext({ viewport: { width: 1360, height: 900 } });
    const page = await ctx.newPage();
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t),
        `<!--sheet-->\n'<!--{"pdf":{"format":"A4","orientation":"landscape","marginTop":"20mm","marginBottom":"20mm"}}-->\n# Landscape\n\n${rows}\n`);
    await page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Report$/ }).click();
    await page.waitForTimeout(3000);
    const btn = page.locator('.page-view-btn');
    if (await btn.getAttribute('aria-pressed') !== 'true') await btn.click();
    let ratio = 0;
    for (let i = 0; i < 40 && !ratio; i++) {
        const h = await page.$('iframe.preview-frame:not(.preview-frame-back)');
        const f = h && await h.contentFrame();
        ratio = (await f?.evaluate(() => { const s = document.getElementById('cpd-page-view')?.firstElementChild; return s ? s.offsetHeight / s.offsetWidth : 0; }).catch(() => 0)) || 0;
        if (!ratio) await page.waitForTimeout(500);
    }
    check(Math.abs(ratio - 210 / 297) < 0.01, `a pdf comment in the document sets the page: A4 landscape (${ratio.toFixed(3)})`);
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
