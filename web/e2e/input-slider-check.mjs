// A slider under a number input with a range (`L: ? 6 m slider 2..12 step 0.5 | span`): present only on lines that have a range,
// shares the number box (both ways), recalculates live (debounced) and on release, keeps unit/min/max/step, is labelled, and
// works with a mouse, the keyboard and touch.   node e2e/input-slider-check.mjs [--url U]
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4984/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();
const sheet = ['<!--sheet-->', '# Span', '', 'L: ? 6 m slider 2..12 step 0.5 | span', 'w: ? 3 kN/m | load', 'k: ? 1 slider 0..1 | factor', '', 'M: w * L^2 / 8 * k -> kN*m | moment'].join('\n');

for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 900 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    await page.evaluate(t => window.calcpadActiveEditor.setValue(t), sheet);
    if (phone) await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
    const mode = page.locator('.preview-mode-group .toolbar-btn', { hasText: /^Input$/ });
    phone ? await mode.tap() : await mode.click();
    // The live frame is the one without `preview-frame-back`; it is swapped after every render, so look it up each time.
    const front = async (pred = () => true) => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('.ck-slider input').count().catch(() => 0) >= 2 && await Promise.resolve().then(() => pred(f)).catch(() => false)) return f;
            await page.waitForTimeout(400);
        }
        return null;
    };
    // The result is in the report beside the form (another frame), so read every frame.
    const moment = async () => {
        const out = [];
        for (const g of page.frames()) {
            const t = (await g.locator('body').innerText().catch(() => '')).replace(/\s+/g, ' ');
            const m = /M = .*= ([\d.]+) ?kN/.exec(t);
            if (m) out.push(Number(m[1]));
        }
        return out;
    };
    const until = async (want) => { for (let i = 0; i < 40; i++) { if ((await moment()).some(v => Math.abs(v - Number(want)) < 0.01)) return true; await page.waitForTimeout(250); } return false; };
    let f = await front();
    check(!!f, `${name}: Input mode shows the sliders`);
    if (!f) { await ctx.close(); continue; }
    check(await f.locator('.ck-slider input').count() === 2, `${name}: a slider for each ranged line and none for the plain one`);
    const a = await f.locator('.ck-slider input').first().evaluate(e => ({ min: e.min, max: e.max, step: e.step, v: e.value, label: e.getAttribute('aria-label'), type: e.type }));
    check(a.type === 'range' && a.min === '2' && a.max === '12' && a.step === '0.5' && a.v === '6', `${name}: min, max, step and value come from the line (${JSON.stringify(a)})`);
    check(/span/.test(a.label) && /\bm\b/.test(a.label), `${name}: the slider is labelled with the note and the unit (${a.label})`);
    const b = await f.locator('.ck-slider input').nth(1).evaluate(e => ({ step: e.step, min: e.min, max: e.max }));
    check(b.step === '0.01' && b.max === '1', `${name}: without a step it is about a hundredth of the range (${JSON.stringify(b)})`);
    check(await f.locator('[data-ui-var="w:1"] .ck-slider').count() === 0, `${name}: the line without a range has no slider`);
    check(await until('13.5'), `${name}: the result starts at the default (M = ${await moment()})`);

    // Slider -> box -> result. The keyboard works the same on both; a pointer/touch is tried after.
    const slider = () => f.locator('[data-ui-var="L:1"] .ck-slider input');
    await slider().focus();
    await slider().press('End');
    f = await front(async g => (await g.locator('[data-ui-var="L:1"] input.calcpad-ui-input').inputValue()) === '12');
    check(!!f && (await f.locator('[data-ui-var="L:1"] input.calcpad-ui-input').inputValue()) === '12', `${name}: End moves the slider and the box to the maximum`);
    check(await until('54'), `${name}: the result recalculates (M = ${await moment()})`);
    check(!!f && (await slider().inputValue()) === '12' && (await slider().evaluate(e => e.getAttribute('aria-valuetext'))) === '12 m', `${name}: the slider keeps the value after the re-render, with a spoken value`);

    // Pointer: click or tap the middle of the track.
    f = await front();
    const bb = await slider().boundingBox();
    if (phone) await page.touchscreen.tap(bb.x + bb.width / 2, bb.y + bb.height / 2);
    else { await page.mouse.move(bb.x + 20, bb.y + bb.height / 2); await page.mouse.down(); await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 6 }); await page.mouse.up(); }
    f = await front(async g => { const v = Number(await g.locator('[data-ui-var="L:1"] input.calcpad-ui-input').inputValue()); return v > 6 && v < 8.5 && (await g.locator('[data-ui-var="L:1"] input.calcpad-ui-input').inputValue()) !== '12'; });
    const mid = f && await f.locator('[data-ui-var="L:1"] input.calcpad-ui-input').inputValue();
    check(!!f && Number(mid) >= 6 && Number(mid) <= 8.5 && Number(mid) % 0.5 === 0, `${name}: ${phone ? 'a touch' : 'a drag'} on the track sets the value on a step (${mid})`);
    check(await until(String(3 * Number(mid) ** 2 / 8)), `${name}: and the result follows (M = ${await moment()})`);

    // Box -> slider: type a value.
    f = await front();
    const box = f.locator('[data-ui-var="L:1"] input.calcpad-ui-input');
    await box.fill('4'); await box.press('Enter');
    f = await front(async g => (await g.locator('[data-ui-var="L:1"] .ck-slider input').inputValue()) === '4');
    check(!!f && await until('6'), `${name}: typing in the box moves the slider and recalculates`);
    check(errors.length === 0, `${name}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
