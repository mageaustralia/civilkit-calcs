// A Plotly chart from a Python block in print, Word and Excel: once the page has drawn it, the page sends the host a still image
// of it (light theme), and the print layout and the Word and Excel exports carry that image instead of the note. Needs the CDN
// for Plotly.js; without it the checks are skipped.
//   node e2e/plotly-print-check.mjs [--url U]
import { chromium } from 'playwright';
import { readFileSync, writeFileSync, mkdtempSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const sample = join(dirname(fileURLToPath(import.meta.url)), '../calcpad-web/public/sheet-samples/3D plate deflection surface.sheet');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
const ctx = await browser.newContext({ viewport: { width: 1300, height: 1000 }, colorScheme: 'dark', acceptDownloads: true });
const page = await ctx.newPage();
const errors = [];
page.on('pageerror', e => errors.push(String(e)));
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
const content = readFileSync(sample, 'utf8');
await page.evaluate(v => window.calcpadActiveEditor.setValue(v), content);

// Draw the chart (the once-per-tab question for Python first).
const yes = page.locator('#cprs-python-yes');
let frame = null;
for (let i = 0; i < 150 && !frame; i++) {
    for (const fr of page.frames()) {
        if (fr === page.mainFrame()) continue;
        if (await yes.count().catch(() => 0)) await yes.first().click().catch(() => {});
        if (await fr.locator('.cprs-plotly canvas, .cprs-plotly .gl-container').count().catch(() => 0)) { frame = fr; break; }
    }
    await page.waitForTimeout(400);
}
if (!frame) {
    console.log('skip: the chart was not drawn (offline?)');
    await browser.close();
    process.exit(0);
}
await page.waitForTimeout(4000); // the still image is sent after the chart is drawn

// Print: the layout the print frame is given.
await page.evaluate(() => {
    window.__printed = null;
    new MutationObserver(ms => {
        for (const m of ms) for (const n of m.addedNodes) if (n.tagName === 'IFRAME' && n.srcdoc) window.__printed = n.srcdoc;
    }).observe(document.body, { childList: true });
});
await page.evaluate(() => { window.calcpadFiles.files.o.print(); });
await page.waitForFunction(() => window.__printed, null, { timeout: 30000 }).catch(() => {});
const printed = await page.evaluate(() => window.__printed || '');
check(printed.length > 0, 'Print PDF gives the print frame a page');
check(/<p class="cprs-plotly-img"[^>]*><img src="data:image\/png;base64,/.test(printed), 'the printed page carries the chart as a PNG image');
check(!/Interactive chart/.test(printed), 'the printed page no longer carries the "interactive chart" note');
const png = /data:image\/png;base64,([A-Za-z0-9+/=]+)/.exec(printed)?.[1] ?? '';
const bytes = Buffer.from(png, 'base64');
check(bytes.length > 5000 && bytes.subarray(1, 4).toString() === 'PNG', `the image is a real PNG (${bytes.length} bytes)`);
const dir = mkdtempSync(join(tmpdir(), 'ck-plotly-print-'));
writeFileSync(join(dir, 'chart.png'), bytes);
console.log('image saved to', join(dir, 'chart.png'));

// Word and Excel carry the image too.
const out = await page.evaluate(async (c) => {
    const post = (ep) => fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: c, forPrint: true }) });
    const bytesOf = async (r) => Array.from(new Uint8Array(await r.arrayBuffer()));
    const d = await post('docx'), x = await post('xlsx');
    return { docx: [d.status, await bytesOf(d)], xlsx: [x.status, await bytesOf(x)] };
}, content);
writeFileSync(join(dir, 'a.docx'), Buffer.from(out.docx[1]));
writeFileSync(join(dir, 'a.xlsx'), Buffer.from(out.xlsx[1]));
check(out.docx[0] === 200 && out.xlsx[0] === 200, 'the Word and Excel exports are produced');
const names = (f) => execFileSync('unzip', ['-Z1', join(dir, f)]).toString().split('\n');
check(names('a.docx').some(n => /^word\/media\/.*\.png$/i.test(n)), 'the Word document holds the chart image');
check(names('a.xlsx').some(n => /^xl\/media\/.*\.(png|jpe?g)$/i.test(n)), 'the Excel workbook holds the chart image');
check(errors.length === 0, `no page errors${errors[0] ? ': ' + errors[0] : ''}`);
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
