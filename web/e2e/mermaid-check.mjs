// ```mermaid fences in a sheet (and in a #markdown block of a worksheet) render as diagrams in the preview. Mermaid is
// loaded only when a page has one: a page without a diagram makes no request for it. A fence Mermaid cannot read shows
// its source and the reason. The report for print and export carries the drawn SVG, the preview's content security policy
// reports nothing, and the service worker precaches the chunks so it works offline.   node e2e/mermaid-check.mjs [--url U]
import { chromium, devices } from 'playwright';
import { decodeCode, diagramId } from '../calcpad-web/src/wasm/sheet-mermaid.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

check(decodeCode('A--&gt;B &amp; &quot;c&quot; &#39;d&#39; &#8594;') === 'A-->B & "c" \'d\' →', 'code text decodes to the diagram source');
check(diagramId('graph TD') === diagramId('graph TD') && diagramId('graph TD') !== diagramId('graph LR'), 'diagram ids follow the source');

const plain = ['<!--sheet-->', '# No diagram', '', 'a: 2 m', '', '```text', 'just text', '```'].join('\n');
const sheet = ['<!--sheet-->', '# Load path', '', '```mermaid', 'graph TD', '  Slab --> Beam', '  Beam --> Column', '```', '', 'a: 2 m'].join('\n');
const bad = ['<!--sheet-->', '# Broken', '', '```mermaid', 'graph TD', '  A --> ((((', '```', '', 'b: 3 m'].join('\n');
const cpd = ['"Worksheet', '#markdown', '```mermaid', 'graph LR', '  Input --> Check', '```', '#end markdown', 'x = 4'].join('\n');

const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    const requests = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    page.on('request', r => requests.push(r.url()));
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const front = async (needle, sel = 'body') => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && (await f.locator('body').innerText().catch(() => '')).includes(needle) && await f.locator(sel).count().catch(() => 0) > 0) return f;
            await page.waitForTimeout(500);
        }
        return null;
    };
    const set = (t) => page.evaluate(v => window.calcpadActiveEditor.setValue(v), t);

    // No diagram: Mermaid is not requested.
    await set(plain);
    check(!!await front('No diagram'), `${name}: a sheet without a diagram renders`);
    await page.waitForTimeout(1000);
    check(!requests.some(u => /mermaid/i.test(u)), `${name}: no request for Mermaid without a diagram`);

    // A diagram: drawn as SVG in place of the fence, still a block of the sheet.
    await set(sheet);
    let frame = await front('Load path', '.sheet-mermaid svg');
    check(!!frame, `${name}: the fence renders as a diagram`);
    if (frame) {
        const info = await frame.locator('.sheet-mermaid').first().evaluate(el => ({
            line: el.getAttribute('data-source-line'), text: el.textContent, svg: !!el.querySelector('svg'), fo: el.querySelectorAll('foreignObject').length,
            w: el.querySelector('svg').getBoundingClientRect().width, code: !!el.querySelector('code'),
        }));
        check(info.svg && /Slab/.test(info.text) && /Column/.test(info.text), `${name}: the diagram shows its labels`);
        check(info.line === '4', `${name}: the diagram keeps its source line (${info.line})`);
        check(info.fo === 0, `${name}: labels are SVG text`);
        check(info.w > 20 && info.w <= (phone ? 400 : 1300), `${name}: the diagram fits the page (${Math.round(info.w)} px)`);
        check(!info.code, `${name}: the source is not shown under the diagram`);
        check(await frame.locator('.cprs-blocked').count() === 0, `${name}: the content security policy reports nothing`);
        check(requests.some(u => /mermaid\.core/.test(u)), `${name}: Mermaid loads once a diagram is there`);
        check(await frame.locator('.ck-grip').count() >= 3, `${name}: the block tools still work`);
    }

    // A fence Mermaid cannot read: its source, with the reason.
    await set(bad);
    frame = await front('Broken', '.sheet-mermaid-error');
    check(!!frame, `${name}: an unreadable diagram shows a note`);
    if (frame) {
        check(/\(\(\(\(/.test(await frame.locator('code.language-mermaid').innerText()), `${name}: and its source`);
        check(/b = 3/.test(await frame.locator('body').innerText()), `${name}: the rest of the sheet still renders`);
    }
    check(await page.locator('body > [id^="dck-mermaid"], body > svg[id^="ck-mermaid"]').count() === 0, `${name}: nothing left behind in the app page`);

    if (!phone) {
        // The report that print and the HTML export use carries the drawn SVG.
        const report = await page.evaluate(async (c) => {
            const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: c, settings: {}, outputFormat: 'html', forPrint: true }) });
            return r.text();
        }, sheet);
        check(/class="sheet-mermaid[^"]*"[^>]*>\s*<svg/.test(report) && !/language-mermaid/.test(report), 'the print and export report has the SVG');
        // A #markdown block of a Calcpad worksheet shares the renderer.
        await set(cpd);
        frame = await front('Worksheet', '.sheet-mermaid svg');
        check(!!frame && /Check/.test(await frame.locator('.sheet-mermaid').innerText()), 'a #markdown block in a worksheet renders the diagram');
        // Offline: the service worker precaches Mermaid's chunks.
        const sw = await page.evaluate(async () => (await fetch('sw.js')).text());
        check(/assets\/mermaid\.core-[^"]+\.js/.test(sw), 'the service worker precaches Mermaid');
    }
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} failed` : 'all passed');
process.exit(fails.length ? 1 : 0);
