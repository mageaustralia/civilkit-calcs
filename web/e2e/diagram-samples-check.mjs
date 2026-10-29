// Every sample of the "Plots and diagrams" group that has a ```mermaid fence, opened from the Samples dialog of the
// built app: each fence is drawn as an SVG, no fence is left as code with a "Diagram not drawn" note, and the sheet
// has no engine error. Desktop only.   node e2e/diagram-samples-check.mjs [--url U]
import { chromium } from 'playwright';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const GROUP = 'Plots and diagrams';
const pub = join(dirname(fileURLToPath(import.meta.url)), '..', 'calcpad-web', 'public');
const manifest = JSON.parse(readFileSync(join(pub, 'sheet-samples', 'manifest.json'), 'utf8'));
const samples = manifest.samples.filter(s => s.group === GROUP).map(s => ({ ...s, text: readFileSync(join(pub, s.file), 'utf8') }))
    .filter(s => s.text.includes('```mermaid'));
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
check(samples.length >= 14, `the ${GROUP} group has ${samples.length} samples with a diagram`);

const browser = await chromium.launch();
const page = await (await browser.newContext({ viewport: { width: 1400, height: 900 } })).newPage();
const errors = [];
page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
await page.waitForSelector('.monaco-editor');

/** The front result frame once it shows `heading` and `fences` drawn or failed diagrams. */
async function front(heading, fences) {
    for (let i = 0; i < 80; i++) {
        const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
        const f = h ? await h.contentFrame() : null;
        if (f && (await f.locator('body').innerText().catch(() => '')).includes(heading)) {
            const done = await f.evaluate(() => document.querySelectorAll('.sheet-mermaid svg, .sheet-mermaid-error').length).catch(() => 0);
            if (done >= fences) return f;
        }
        await page.waitForTimeout(500);
    }
    return null;
}

for (const s of samples) {
    const heading = /^# (.+)$/m.exec(s.text)[1].trim();
    const fences = (s.text.match(/^```mermaid\s*$/gm) ?? []).length;
    await page.click('[data-ribbon-tab=resources]');
    await page.click('[data-ribbon-btn=x-samples]');
    await page.locator('.cprs-s-group', { hasText: new RegExp(`^${GROUP}`) }).click();
    await page.locator('.cprs-s-item', { hasText: s.title }).first().dblclick();
    const f = await front(heading, fences);
    check(!!f, `${s.title}: opens and draws`);
    if (!f) continue;
    const info = await f.evaluate(() => ({
        svgs: document.querySelectorAll('.sheet-mermaid svg').length,
        notes: [...document.querySelectorAll('.sheet-mermaid-error')].map(e => e.textContent),
        code: document.querySelectorAll('code.language-mermaid').length,
        text: document.body.innerText,
    }));
    check(info.svgs === fences && info.code === 0, `${s.title}: ${info.svgs} of ${fences} diagrams drawn as SVG`);
    check(info.notes.length === 0 && !/Diagram not drawn|Syntax error in text/i.test(info.text), `${s.title}: no diagram error${info.notes[0] ? ': ' + info.notes[0] : ''}`);
    check(!/Error in |Undefined variable|Invalid syntax/.test(info.text), `${s.title}: no engine error`);
}
check(errors.length === 0, `no page errors${errors[0] ? ': ' + errors[0] : ''}`);
await browser.close();
console.log(fails.length ? `${fails.length} failed` : 'all passed');
process.exit(fails.length ? 1 : 0);
