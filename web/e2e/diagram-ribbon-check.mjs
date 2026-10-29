// The Diagrams menu on the ribbon's Plots & Diagrams tab: it lists the starter kinds (and not the ones left out),
// each starter inserted into a sheet becomes its own ```mermaid block with its source selected, is one undo step, and
// draws in the preview with no error; in a Calcpad worksheet it goes in a #markdown block and draws too. The tab
// label fits the tab strip. Then a phone (iPhone 14, touch) inserts two kinds.   node e2e/diagram-ribbon-check.mjs [--url U]
import { chromium, devices } from 'playwright';
import { DIAGRAM_STARTERS } from '../calcpad-web/src/wasm/diagram-starters.ts';
import { diagramId } from '../calcpad-web/src/wasm/sheet-mermaid.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const KINDS = DIAGRAM_STARTERS.flatMap(s => s.items);
const LEFT_OUT = ['ZenUML', 'Class', 'C4', 'Use case', 'User journey', 'Event model', 'Cynefin', 'Wardley', 'Kanban', 'Git graph', 'Packet', 'Railroad', 'Venn'];
const SHEET = ['<!--sheet-->', '# Diagram test', '', 'Some text before.', 'And a second line.', '', 'L: 6 m | span'].join('\n');
const CPD = ['"Worksheet', 'x = 4', 'y = 2*x'].join('\n');

check(KINDS.length >= 15 && KINDS.length <= 20, `the menu has ${KINDS.length} starters`);
check(!KINDS.some(k => LEFT_OUT.some(o => k.label.toLowerCase().includes(o.toLowerCase()))), 'no left-out kind is a starter');
check(KINDS.every(k => k.title && !k.title.includes('\n')), 'every starter has a one-line tooltip');

const browser = await chromium.launch();

async function open(opts) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await page.waitForSelector('.monaco-editor');
    return { ctx, page, errors };
}
const setDoc = (page, t, line) => page.evaluate(([v, l]) => {
    const ed = window.calcpadActiveEditor;
    ed.setValue(v);
    ed.setPosition({ lineNumber: l, column: 1 });
}, [t, line]);
const value = page => page.evaluate(() => window.calcpadActiveEditor.getValue());
const selected = page => page.evaluate(() => { const ed = window.calcpadActiveEditor; return ed.getModel().getValueInRange(ed.getSelection()); });
/** The front preview frame once it has `n` drawn or failed diagrams (and the needle text). */
async function drawn(page, needle, src) {
    const ids = [diagramId(src), diagramId(src + '\n')];
    for (let i = 0; i < 60; i++) {
        const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
        const f = h ? await h.contentFrame() : null;
        if (f && (await f.locator('body').innerText().catch(() => '')).includes(needle)) {
            const info = await f.evaluate((ids) => ({
                svgs: [...document.querySelectorAll('.sheet-mermaid svg')].filter(s => ids.includes(s.id)).length,
                notes: [...document.querySelectorAll('.sheet-mermaid-error')].map(e => e.textContent),
                text: document.body.innerText,
            }), ids).catch(() => null);
            if (info && info.svgs + info.notes.length >= 1) return info;
        }
        await page.waitForTimeout(500);
    }
    return null;
}
const ok = info => !!info && info.svgs >= 1 && info.notes.length === 0 && !/Diagram not drawn|Syntax error in text|Error in |Undefined variable/i.test(info.text);

// ---- desktop ----------------------------------------------------------------------------------
{
    const { ctx, page, errors } = await open({ viewport: { width: 1400, height: 900 } });
    const tab = page.locator('[data-ribbon-tab=plots]');
    check((await tab.innerText()).trim() === 'Plots & Diagrams', 'the tab reads "Plots & Diagrams"');
    for (const w of [1400, 1280, 1100]) {
        await page.setViewportSize({ width: w, height: 900 });
        const fit = await page.locator('.rb-tabs').evaluate(el => el.scrollWidth <= el.clientWidth + 1);
        check(fit, `the tab strip fits at ${w} px`);
    }
    await page.setViewportSize({ width: 1400, height: 900 });
    await tab.click();
    const btn = page.locator('[data-ribbon-btn=dg-diagrams]');
    check(await btn.isVisible(), 'the Diagrams button is on the tab');
    check(await page.locator('.rb-group', { has: btn }).locator('.rb-gname').innerText() === 'Figures', 'in the Figures group');
    await btn.click();
    const pop = page.locator('[data-ribbon-popover=dg-diagrams]');
    await pop.waitFor();
    const labels = await pop.locator('.rb-glyph').allInnerTexts();
    check(JSON.stringify(labels) === JSON.stringify(KINDS.map(k => k.label)), `the menu lists the ${KINDS.length} kinds in order`);
    const heads = await pop.locator('.rb-pop-h').allInnerTexts();
    check(heads.length === DIAGRAM_STARTERS.length, `with ${heads.length} section headings`);
    check(!LEFT_OUT.some(o => labels.some(l => l.toLowerCase().includes(o.toLowerCase()))), 'and none of the left-out kinds (ZenUML, Class, C4, ...)');
    const tip = await pop.locator('.rb-glyph').first().getAttribute('title');
    check(tip === KINDS[0].title, 'a starter shows its one-line tooltip');
    const style = await pop.evaluate(el => ({ mh: getComputedStyle(el).maxHeight, ov: getComputedStyle(el).overflowY }));
    check(style.mh !== 'none' && /auto|scroll/.test(style.ov), `the menu scrolls when long (max-height ${style.mh})`);
    await page.keyboard.press('Escape');

    for (const k of KINDS) {
        await setDoc(page, SHEET, 4);
        await btn.click();
        await pop.locator('.rb-glyph', { hasText: new RegExp(`^${k.label}$`) }).click();
        const v = await value(page);
        const want = ['<!--sheet-->', '# Diagram test', '', 'Some text before.', 'And a second line.', '', '```mermaid', k.source, '```', '', 'L: 6 m | span'].join('\n');
        check(v === want, `${k.label}: inserted as its own block after the paragraph`);
        check(await selected(page) === k.source, `${k.label}: its source is selected`);
        const info = await drawn(page, 'Diagram test', k.source);
        check(ok(info), `${k.label}: draws in the preview${info?.notes[0] ? ': ' + info.notes[0] : ''}`);
        await page.evaluate(() => { const ed = window.calcpadActiveEditor; ed.focus(); ed.trigger('check', 'undo', null); });
        check(await value(page) === SHEET, `${k.label}: one undo takes it out`);
    }

    // A Calcpad worksheet: a #markdown block after the caret's line.
    await setDoc(page, CPD, 2);
    await page.waitForTimeout(500);
    await btn.click();
    await pop.locator('.rb-glyph', { hasText: /^Flowchart$/ }).click();
    const v = await value(page);
    check(v === ['"Worksheet', 'x = 4', '#markdown', '```mermaid', KINDS[0].source, '```', '#end markdown', 'y = 2*x'].join('\n'), 'worksheet: inserted as a #markdown block');
    const info = await drawn(page, 'Worksheet', KINDS[0].source);
    check(ok(info), `worksheet: the diagram draws${info?.notes[0] ? ': ' + info.notes[0] : ''}`);
    check(errors.length === 0, `desktop: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// ---- phone ------------------------------------------------------------------------------------
{
    const { ctx, page, errors } = await open({ ...devices['iPhone 14'] });
    for (const label of ['Flowchart', 'Gantt']) {
        const k = KINDS.find(x => x.label === label);
        await setDoc(page, SHEET, 4);
        if (!(await page.locator('[data-ribbon-btn=dg-diagrams]').isVisible())) await page.locator('[data-ribbon-tab=plots]').tap();
        const btn = page.locator('[data-ribbon-btn=dg-diagrams]');
        await btn.scrollIntoViewIfNeeded();
        await btn.tap();
        const pop = page.locator('[data-ribbon-popover=dg-diagrams]');
        await pop.waitFor();
        const box = await pop.boundingBox();
        const vp = page.viewportSize();
        check(!!box && box.y >= 0 && box.y + box.height <= vp.height + 1 && box.width <= vp.width + 1, `phone ${label}: the menu fits the screen`);
        const item = pop.locator('.rb-glyph', { hasText: new RegExp(`^${label}$`) });
        await item.scrollIntoViewIfNeeded();
        await item.tap();
        check((await value(page)).includes('```mermaid\n' + k.source + '\n```'), `phone ${label}: inserted`);
        await page.locator('.mobile-bar button', { hasText: 'Results' }).tap();
        const info = await drawn(page, 'Diagram test', k.source);
        check(ok(info), `phone ${label}: draws in the preview${info?.notes[0] ? ': ' + info.notes[0] : ''}`);
        await page.locator('.mobile-bar button', { hasText: 'Edit' }).tap();
    }
    check(errors.length === 0, `phone: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
    await ctx.close();
}
await browser.close();
console.log(fails.length ? `${fails.length} failed` : 'all passed');
process.exit(fails.length ? 1 : 0);
