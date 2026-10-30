// Docked inputs in Input mode (input-dock.ts): every input of the sheet in a panel of the results pane, outside the result
// frames, so a slider can be dragged with the plot it moves in view.
//   node e2e/input-dock-check.mjs [--url http://127.0.0.1:5191/] [--shots DIR]
// On a desktop window (1360x860, the dock at the left of the results) and an iPhone 14 (a strip above them):
//   1. a sheet with three sliders and a plot far below opens in Input mode with the dock on (its default with a slider),
//      the three sliders in it and the sheet's own input lines hidden;
//   2. with the results scrolled to the plot, dragging a dock slider redraws the plot (its path data changes) while the
//      mouse is still down, the plot stays in view and so does the slider;
//   3. no horizontal overflow at 390 px; the dock button toggles it off (the input lines come back) and on again;
//   4. a number typed into a dock box outside the bounds is refused with a message beside it, one inside is used;
//   5. inputs written after the plot are in the dock too; a sheet of twelve inputs scrolls inside the dock;
//   6. a sheet without inputs has no dock and no button; the dock is not printed; no page errors.
import { chromium, devices } from 'playwright';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5191/');
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const prose = Array.from({ length: 14 }, (_, i) => `Paragraph ${i + 1} of the notes between the inputs and the plot. It is here so that the plot is far below the inputs and has to be scrolled to.`).join('\n\n');
const SLIDERS = `<!--sheet-->
# Dock test

## Inputs
a: ? 2 [1 .. 5 step 0.5] | amplitude
k: ? 1 [0.5 .. 3 step 0.25] | frequency
c: ? 0 [-2 .. 2 step 0.5] | offset

${prose}

## The curve
y(x): a * sin(k * x) + c
plot y(x) for x in 0..360
`;
const AFTER = `<!--sheet-->
# Inputs after the plot
f(x): p * x^2
plot f(x) for x in 0..3
p: ? 1 [0.5 .. 4 step 0.5] | factor
q: ? [Low = 1, *High = 2] | class
`;
const MANY = `<!--sheet-->
# Many inputs
${Array.from({ length: 12 }, (_, i) => `x${i + 1}: ? ${i + 1} [0 .. 20 step 1] | input number ${i + 1}`).join('\n')}
s: x1 + x2 + x3 + x4 + x5 + x6 + x7 + x8 + x9 + x10 + x11 + x12
`;
const NONE = `<!--sheet-->
# No inputs
s: 2 + 3
`;

const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1360, height: 860 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    for (const scheme of ['light', 'dark']) {
        const ctx = await browser.newContext({ ...opts, colorScheme: scheme });
        const page = await ctx.newPage();
        const errors = [];
        page.on('pageerror', (e) => { if (!/^(Canceled|Event)/.test(String(e.message))) errors.push(String(e.message)); });
        await page.goto(url);
        await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
        const tag = `${name} ${scheme}`;
        const front = async () => {
            for (let i = 0; i < 80; i++) {
                const h = await page.$('.preview-pane.fullscreen .preview-frame:not(.preview-frame-back)');
                const f = h && await h.contentFrame();
                if (f) return f;
                await page.waitForTimeout(250);
            }
            return null;
        };
        const open = async (text) => {
            await page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
            if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
            const input = page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ });
            if (!(await input.evaluate((b) => b.classList.contains('active')).catch(() => false))) await input.click({ timeout: 20000 });
        };
        const dock = page.locator('.input-dock-host .input-dock');
        const dockButton = page.locator('.input-dock-btn');

        // ---- 1 ----
        await open(SLIDERS);
        const shown = await dock.locator('.ce-slider').first().waitFor({ timeout: 60000 }).then(() => true).catch(() => false);
        const n = await dock.locator('.ce-slider').count();
        check(shown && n === 3, `${tag}: the dock shows the three sliders (${n})`);
        check(await dockButton.getAttribute('aria-pressed') === 'true', `${tag}: the Dock inputs button is pressed (on by default with a slider)`);
        const labels = await dock.locator('.ce-label').allInnerTexts();
        check(/amplitude/.test(labels[0] ?? ''), `${tag}: the dock labels come from the sheet (${labels.join(' | ')})`);
        let f = await front();
        let hidden = null;
        for (let i = 0; i < 40 && hidden !== true; i++) {
            f = await front();
            hidden = await f?.evaluate(() => { const e = document.querySelector('[data-ui-type][data-ui-var]'); return !!e && getComputedStyle(e).display === 'none'; }).catch(() => null);
            if (hidden !== true) await page.waitForTimeout(250);
        }
        check(hidden === true, `${tag}: the sheet's own input lines are hidden while docked`);
        const side = await page.locator('.preview-body').evaluate((e) => e.classList.contains('dock-side'));
        check(side === !phone, `${tag}: the dock is ${phone ? 'above' : 'beside'} the results`);
        if (phone) {
            const dockBox = await page.locator('.input-dock-host').boundingBox();
            const frameBox = await page.locator('.preview-pane.fullscreen .preview-frames').boundingBox();
            check(dockBox && frameBox && dockBox.y + dockBox.height <= frameBox.y + 1, `${tag}: the dock strip sits above the results`);
            const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
            const dockOverflow = await page.locator('.input-dock-list').evaluate((e) => e.scrollWidth - e.clientWidth);
            check(overflow <= 0 && dockOverflow <= 0, `${tag}: no horizontal overflow at ${page.viewportSize().width} px (page ${overflow}, dock ${dockOverflow})`);
        }
        // Tab order: the dock comes before the result frames.
        const order = await page.evaluate(() => {
            const d = document.querySelector('.input-dock input'), fr = document.querySelector('.preview-pane.fullscreen .preview-frame');
            return !!d && !!fr && !!(d.compareDocumentPosition(fr) & Node.DOCUMENT_POSITION_FOLLOWING);
        });
        check(order, `${tag}: the dock is before the results in the tab order`);
        if (shots) await page.screenshot({ path: join(shots, `dock-${name}-${scheme}.png`) });

        // ---- 2: drag while the plot is in view ----
        const plotState = async () => (await front())?.evaluate(() => {
            const svg = [...document.querySelectorAll('svg')].find((s) => s.querySelector('path, polyline'));
            if (!svg) return null;
            const r = svg.getBoundingClientRect();
            const d = [...svg.querySelectorAll('path, polyline')].map((p) => p.getAttribute('d') ?? p.getAttribute('points')).join('|');
            return { d, top: r.top, bottom: r.bottom, h: innerHeight };
        }).catch(() => null);
        // Bring the plot into view in the results.
        await (await front())?.evaluate(() => { const s = [...document.querySelectorAll('svg')].find((x) => x.querySelector('path, polyline')); s?.scrollIntoView({ block: 'center' }); });
        await page.waitForTimeout(400);
        const before = await plotState();
        check(before && before.top < before.h && before.bottom > 0, `${tag}: the plot is scrolled into view (${before && Math.round(before.top)}..${before && Math.round(before.bottom)} of ${before?.h})`);
        const slider = dock.locator('.ce-field[data-name^="k:"] .ce-slider');
        await slider.scrollIntoViewIfNeeded();
        const sb = await slider.boundingBox();
        let during = null;
        if (sb) {
            const y = sb.y + sb.height / 2;
            await page.mouse.move(sb.x + sb.width * 0.25, y);
            await page.mouse.down();
            for (let i = 1; i <= 12 && !during; i++) {
                await page.mouse.move(sb.x + sb.width * (0.25 + i * 0.05), y, { steps: 3 });
                await page.waitForTimeout(300);
                const s = await plotState();
                if (s && before && s.d && s.d !== before.d) during = s;
            }
            for (let i = 0; i < 20 && !during; i++) {
                await page.waitForTimeout(300);
                const s = await plotState();
                if (s && before && s.d && s.d !== before.d) during = s;
            }
            const sliderStill = await slider.boundingBox();
            const vp = page.viewportSize();
            check(!!sliderStill && sliderStill.y >= 0 && sliderStill.y + sliderStill.height <= vp.height, `${tag}: the slider stays in view while dragging`);
            await page.mouse.up();
        }
        check(!!during, `${tag}: the plot redraws (path data changes) while the slider is still held`);
        check(!!during && during.top < during.h && during.bottom > 0, `${tag}: the redrawn plot is still in view (${during && Math.round(during.top)}..${during && Math.round(during.bottom)})`);
        const box = dock.locator('.ce-field[data-name^="k:"] .ce-box');
        const sliderValue = await slider.inputValue();
        check(await box.inputValue() === sliderValue && Number(sliderValue) > 1, `${tag}: the box follows the slider (${sliderValue})`);
        if (shots) await page.screenshot({ path: join(shots, `dock-${name}-${scheme}-dragged.png`) });

        // ---- 4: a typed value checked against the bounds ----
        await box.fill('9');
        await box.press('Enter');
        const err = dock.locator('.ce-field[data-name^="k:"] .ce-err');
        check(await err.isVisible() && (await err.innerText()) === 'must be from 0.5 to 3' && await box.getAttribute('aria-invalid') === 'true', `${tag}: a typed 9 is refused with "must be from 0.5 to 3" beside the box`);
        await page.waitForTimeout(800);
        const after9 = await plotState();
        check(after9?.d === (await plotState())?.d && after9?.d !== before?.d, `${tag}: the refused value is not used`);
        if (shots && name === 'desktop' && scheme === 'light') await page.screenshot({ path: join(shots, 'dock-error.png') });
        await box.fill('2.5');
        await box.press('Enter');
        let typed = null;
        for (let i = 0; i < 40 && !typed; i++) { await page.waitForTimeout(250); const s = await plotState(); if (s?.d && s.d !== after9?.d) typed = s; }
        check(!!typed && !(await err.isVisible()), `${tag}: a typed 2.5 is used and the plot follows`);
        check(await slider.inputValue() === '2.5', `${tag}: the slider follows the typed value`);

        // ---- 3: toggle off and on ----
        await dockButton.click();
        check(await dockButton.getAttribute('aria-pressed') === 'false' && !(await page.locator('.input-dock-host').isVisible()), `${tag}: the button undocks the inputs`);
        let back = null;
        for (let i = 0; i < 40 && back !== true; i++) {
            await page.waitForTimeout(250);
            back = await (await front())?.evaluate(() => { const e = document.querySelector('input.calcpad-ui-slider[data-ui-slider-for^="k:"]'); return !!e && e.offsetParent !== null && e.value === '2.5'; }).catch(() => null);
        }
        check(back === true, `${tag}: undocked, the sheet's own sliders are back, with the value set in the dock`);
        if (shots && scheme === 'light') await page.screenshot({ path: join(shots, `dock-${name}-off.png`) });
        await dockButton.click();
        check(await dockButton.getAttribute('aria-pressed') === 'true' && await page.locator('.input-dock-host').isVisible(), `${tag}: the button docks them again`);
        const remembered = await page.evaluate(() => sessionStorage.getItem('calcpad.inputDock'));
        check(remembered === 'on', `${tag}: the choice is remembered for the session (${remembered})`);

        // ---- collapse to one line ----
        await dock.locator('.input-dock-toggle').click();
        const summary = await dock.locator('.input-dock-summary').innerText();
        check(await dock.locator('.input-dock-list').isHidden() && /k = 2\.5/.test(summary), `${tag}: collapsed, the dock is one line of values (${summary})`);
        if (shots && phone) await page.screenshot({ path: join(shots, `dock-${name}-${scheme}-collapsed.png`) });
        await dock.locator('.input-dock-toggle').click();
        check(await dock.locator('.input-dock-list').isVisible(), `${tag}: expanded again`);

        // ---- 6: not printed ----
        await page.emulateMedia({ media: 'print' });
        check(!(await page.locator('.input-dock-host').isVisible()), `${tag}: the dock is not printed`);
        await page.emulateMedia({ media: 'screen' });

        if (scheme === 'light') {
            // ---- 5: inputs after the plot; many inputs ----
            await open(AFTER);
            const ok = await dock.locator('.ce-field[data-name^="q:"]').waitFor({ timeout: 60000 }).then(() => true).catch(() => false);
            check(ok && await dock.locator('.ce-field').count() === 2 && await dock.locator('.ce-slider').count() === 1, `${tag}: inputs written after the plot are in the dock (a slider and a list)`);
            await open(MANY);
            const many = await dock.locator('.ce-field[data-name^="x12:"]').waitFor({ timeout: 60000 }).then(() => true).catch(() => false);
            const scroll = await page.locator('.input-dock-list').evaluate((e) => [e.scrollHeight, e.clientHeight]);
            check(many && await dock.locator('.ce-field').count() === 12 && scroll[0] > scroll[1], `${tag}: twelve inputs, and the dock scrolls (${scroll.join(' > ')})`);
            if (shots) await page.screenshot({ path: join(shots, `dock-${name}-many.png`) });
            await open(NONE);
            await page.waitForTimeout(2500);
            check(await dockButton.count() === 0 && !(await page.locator('.input-dock-host').isVisible()), `${tag}: a sheet without inputs has no dock and no button`);
        }
        check(errors.length === 0, `${tag}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
        await ctx.close();
    }
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
