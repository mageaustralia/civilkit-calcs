// Manual page breaks in a sheet: the "+" menu and the ribbon insert the break line, the preview draws it as a labelled
// dashed line, the printed page (Print PDF) breaks there and draws no marker, and the Word file has a page break in
// the same place. The menu entry is checked on a phone too.   node e2e/page-break-check.mjs [--url U]
import { chromium, devices } from 'playwright';
import { execFileSync } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { PAGE_BREAK_SHEET } from '../calcpad-web/src/wasm/page-break.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const SHEET = '<!--sheet-->\n# Loads\n\nw: 12 kN/m | design load\n\n# Bending\n\nL: 6 m\nM: w * L^2 / 8 -> kN*m\n';
const preview = (page) => page.frameLocator('iframe.preview-frame:not(.preview-frame-back)');
const text = (page) => page.evaluate(() => window.calcpadActiveEditor.getValue());

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 }, acceptDownloads: true }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    // Keep the HTML handed to the print frame, which is what the browser prints.
    await page.addInitScript(() => {
        window.__printed = [];
        new MutationObserver(ms => ms.forEach(m => m.addedNodes.forEach(n => {
            if (n.tagName === 'IFRAME' && n.getAttribute('sandbox') === 'allow-same-origin allow-modals') window.__printed.push(n.srcdoc);
        }))).observe(document, { childList: true, subtree: true });
    });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), SHEET);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();

    // The "+" before the second heading inserts the break line there.
    const frame = preview(page);
    await frame.locator('.ck-ins-btn').first().waitFor({ timeout: 30000 });
    await page.waitForTimeout(2000);
    const second = frame.locator('[data-source-line="6"]');
    if (phone) {
        await second.tap();
        await frame.locator('.ck-ins.on .ck-ins-btn').first().tap();
    } else {
        let clicked = false;
        for (let attempt = 0; attempt < 4 && !clicked; attempt++) {
            const bb = await second.boundingBox();
            await page.mouse.move(bb.x + bb.width / 2 + 100 + attempt * 7, bb.y - 2, { steps: 4 });
            await page.mouse.move(bb.x + bb.width / 2, bb.y - 1, { steps: 2 });
            await frame.locator('.ck-ins.on .ck-ins-btn').first().waitFor({ timeout: 2000 }).catch(() => {});
            await page.waitForTimeout(500);
            const btns = frame.locator('.ck-ins-btn');
            for (let i = 0; i < await btns.count() && !clicked; i++) {
                if (+await btns.nth(i).evaluate(e => getComputedStyle(e).opacity) === 1) { await btns.nth(i).click(); clicked = true; }
            }
        }
        check(clicked, `${name}: a + shows above the second heading`);
    }
    await page.waitForSelector('.insert-menu');
    const item = page.locator('.insert-menu button', { hasText: 'Page break' });
    check(await item.count() === 1, `${name}: the + menu offers a page break`);
    phone ? await item.tap() : await item.click();
    let lines = (await text(page)).split('\n');
    const at = lines.indexOf(PAGE_BREAK_SHEET);
    check(at > 0 && lines[at + 2] === '# Bending' && lines.filter(l => l === PAGE_BREAK_SHEET).length === 1, `${name}: the break line went in before the second heading (line ${at + 1})`);

    // The preview shows it as a dashed line labelled "Page break"; the marker is screen only.
    await page.waitForTimeout(2500);
    const mark = preview(page).locator('div[style*="page-break-after"]');
    await mark.waitFor({ timeout: 20000 });
    const look = await mark.evaluate(e => ({ border: getComputedStyle(e).borderTopStyle, label: getComputedStyle(e, '::after').content }));
    check(look.border === 'dashed' && look.label === '"Page break"', `${name}: the preview draws a labelled dashed line (${JSON.stringify(look)})`);

    if (!phone) {
        // The ribbon's Page break writes the same sheet line, not a raw Calcpad line.
        await page.evaluate(() => { const e = window.calcpadActiveEditor; const n = e.getModel().getLineCount(); e.setPosition({ lineNumber: n, column: 1 }); e.focus(); });
        await page.locator('[data-ribbon-tab=document]').click();
        await page.locator('[data-ribbon-btn="d-pagebreak"]').click();
        lines = (await text(page)).split('\n');
        check(lines.filter(l => l === PAGE_BREAK_SHEET).length === 2 && !lines.some(l => l.startsWith('>>')), `${name}: the ribbon's Page break inserts the sheet line`);
        await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), SHEET.replace('# Bending', `${PAGE_BREAK_SHEET}\n\n# Bending`));
        await page.waitForTimeout(2500);

        // Print PDF: the printed page breaks before "Bending" and has no marker.
        await page.locator('[data-ribbon-tab=document]').click();
        await page.locator('[data-ribbon-btn="d-pdf"]').click();
        await page.waitForFunction(() => window.__printed.length > 0, null, { timeout: 30000 });
        const printed = await page.evaluate(() => window.__printed[0]);
        const p2 = await ctx.newPage();
        await p2.setContent(printed);
        await p2.emulateMedia({ media: 'print' });
        const brk = p2.locator('div[style*="page-break-after"]');
        const css = await brk.evaluate(e => ({ after: getComputedStyle(e).breakAfter, border: getComputedStyle(e).borderTopStyle, label: getComputedStyle(e, '::after').content }));
        check(css.after === 'page' && css.border === 'none' && (css.label === 'none' || css.label === 'normal'), `${name}: in print the break forces a page and draws nothing (${JSON.stringify(css)})`);
        const pdf = await p2.pdf({ format: 'A4' });
        const pages = (pdf.toString('latin1').match(/\/Type\s*\/Page[^s]/g) ?? []).length;
        await brk.evaluate(e => e.removeAttribute('style'));
        const without = ((await p2.pdf({ format: 'A4' })).toString('latin1').match(/\/Type\s*\/Page[^s]/g) ?? []).length;
        check(pages === 2 && without === 1, `${name}: the printed report is two pages, one without the break (${pages}, ${without})`);
        await p2.close();

        // Word: one page break, between the two sections.
        await page.locator('[data-ribbon-tab=document]').click();
        const [download] = await Promise.all([page.waitForEvent('download', { timeout: 120000 }), page.locator('[data-ribbon-btn="d-word"]').click()]);
        const file = join(mkdtempSync(join(tmpdir(), 'page-break-')), 'r.docx');
        await download.saveAs(file);
        const xml = execFileSync('unzip', ['-p', file, 'word/document.xml'], { encoding: 'utf8', maxBuffer: 64 << 20 });
        const BRK = '<w:br w:type="page"/>';
        check(xml.split(BRK).length === 2 && xml.indexOf('Loads') < xml.indexOf(BRK) && xml.indexOf(BRK) < xml.indexOf('Bending'), `${name}: the Word file has one page break, between the sections`);
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
