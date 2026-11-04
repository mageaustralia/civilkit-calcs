// A review sheet of every ribbon icon, big (32px) and small (16px), on the light and dark themes.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/icon-sheet.mjs [--url U] [--out DIR]
import { chromium } from 'playwright';
import { mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4175/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-shots'));
mkdirSync(out, { recursive: true });
const browser = await chromium.launch();
for (const theme of ['light', 'dark']) {
    const page = await browser.newPage({ viewport: { width: 1100, height: 900 }, colorScheme: theme });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadRibbonIcons != null);
    await page.evaluate((theme) => {
        document.documentElement.dataset.theme = theme;
        const { big, small } = window.calcpadRibbonIcons;
        const bg = theme === 'dark' ? '#252526' : '#ffffff';
        const fg = theme === 'dark' ? '#cccccc' : '#333333';
        const wrap = document.createElement('div');
        wrap.className = 'rb';
        wrap.style.cssText = `position:fixed;inset:0;z-index:9999;overflow:auto;padding:16px;background:${bg};color:${fg};font:12px sans-serif`;
        const cell = (name, svg, size, vb, stroke) => `<div style="display:flex;flex-direction:column;align-items:center;width:84px;margin:6px 0"><svg ${stroke ? 'class="rb-sm"' : 'class="rb-ic"'} viewBox="${vb}" width="${size}" height="${size}">${svg}</svg><span style="margin-top:4px;font-size:10px">${name}</span></div>`;
        wrap.innerHTML = `<h3 style="margin:0 0 8px">Big icons (32px, ${Object.keys(big).length})</h3><div style="display:flex;flex-wrap:wrap">${Object.entries(big).map(([n, s]) => cell(n, s, 32, '0 0 32 32', false)).join('')}</div>` +
            `<h3 style="margin:14px 0 8px">Small icons (16px, ${Object.keys(small).length})</h3><div style="display:flex;flex-wrap:wrap">${Object.entries(small).map(([n, s]) => cell(n, s, 16, '0 0 24 24', true)).join('')}</div>`;
        document.body.appendChild(wrap);
    }, theme);
    await page.screenshot({ path: `${out}/icon-sheet-${theme}-v3.png` });
    await page.close();
}
await browser.close();
