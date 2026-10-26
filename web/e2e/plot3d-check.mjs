// Each 3D sample sheet: loads it, waits for the Plotly scene (a WebGL canvas) and checks the quantity lines, light and dark.
//   node e2e/plot3d-check.mjs [--url U] [--out DIR]
import { chromium } from 'playwright';
import { readFileSync, readdirSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-3d'));
mkdirSync(out, { recursive: true });
const dir = join(dirname(fileURLToPath(import.meta.url)), '../calcpad-web/public/sheet-samples');
const files = readdirSync(dir).filter(f => f.startsWith('3D ') && f.endsWith('.sheet'));
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });

for (const scheme of ['light', 'dark']) {
    for (const f of files) {
        const ctx = await browser.newContext({ viewport: { width: 1300, height: 1000 }, colorScheme: scheme });
        const page = await ctx.newPage();
        const errors = [];
        page.on('pageerror', e => errors.push(String(e)));
        await page.goto(url);
        await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
        await page.evaluate(v => window.calcpadActiveEditor.setValue(v), readFileSync(join(dir, f), 'utf8'));
        // the once-per-tab question for Python
        const yes = page.locator('#cprs-python-yes');
        let frame = null, body = '';
        for (let i = 0; i < 150 && !frame; i++) {
            for (const fr of page.frames()) {
                if (fr === page.mainFrame()) continue;
                if (await yes.count().catch(() => 0)) await yes.first().click().catch(() => {});
                if (await fr.locator('.cprs-plotly canvas, .cprs-plotly .gl-container').count().catch(() => 0)) { frame = fr; break; }
                const t = await fr.evaluate(() => document.body?.innerText ?? '').catch(() => '');
                if (/could not be drawn|Error|NotImplemented/.test(t)) { body = t; break; }
            }
            if (body) break;
            await page.waitForTimeout(400);
        }
        const name = f.replace('.sheet', '');
        check(!!frame, `${scheme}: ${name} draws a 3D scene${body ? ' (' + body.slice(0, 200).replace(/\n/g, ' ') + ')' : ''}`);
        if (frame) {
            await page.waitForTimeout(1500);
            const txt = await frame.evaluate(() => document.body.innerText);
            check(!/could not be drawn|Error/.test(txt), `${scheme}: ${name} has no error text`);
            const note = await frame.evaluate(() => { const n = document.querySelector('.cprs-plotly')?.nextElementSibling; return n && getComputedStyle(n).display !== 'none' ? n.textContent : ''; });
            check(!note, `${scheme}: ${name} shows no fallback note${note ? ': ' + note : ''}`);
            await frame.locator('.cprs-plotly').first().scrollIntoViewIfNeeded().catch(() => {});
            await page.waitForTimeout(800);
            await page.screenshot({ path: join(out, `${name}-${scheme}.png`) });
        }
        check(errors.length === 0, `${scheme}: ${name} no page errors${errors[0] ? ': ' + errors[0] : ''}`);
        await ctx.close();
    }
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok', '- screenshots in', out);
process.exit(fails.length ? 1 : 0);
