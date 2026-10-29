// The layout rules of the ribbon (docs/ui-review.md): no two buttons of a tab share a label, every button has a tooltip that
// says more than its label, the tab strip fits a 1024 px window without scrolling, every tab's buttons fit a 1300 px window,
// the Plots & Diagrams tab keeps Diagrams and Image together in a Figures group, the flowchart icon is the Diagrams
// button's alone, and the tabs follow the work: write, calculate, plot, format, document.
//   node e2e/ribbon-layout-check.mjs [--url U]
import { chromium } from 'playwright';
import { QUICK, RIBBON } from '../calcpad-web/src/wasm/ribbon-data.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const all = RIBBON.flatMap(t => t.groups.flatMap(g => g.buttons.map(b => ({ tab: t, group: g, b }))));

// ---- the data ------------------------------------------------------------------------------------
{
    const dup = [];
    for (const t of RIBBON) {
        const seen = new Set();
        for (const g of t.groups) for (const b of g.buttons) { if (seen.has(b.label)) dup.push(`${t.label}: ${b.label}`); seen.add(b.label); }
    }
    check(dup.length === 0, `no two buttons of a tab share a label (${dup.join(', ') || 'none'})`);
    const ids = all.map(x => x.b.id);
    check(new Set(ids).size === ids.length, 'every button id is unique');
    const bare = [...all.map(x => x.b), ...QUICK].filter(b => !b.title || b.title.trim().toLowerCase() === b.label.trim().toLowerCase());
    check(bare.length === 0, `every button has a tooltip that says more than its label (${bare.map(b => b.id).join(', ') || 'none'})`);
    const dots = all.filter(x => /…|\.\.\.$/.test(x.b.label));
    check(dots.length === 0, `no ribbon label ends in an ellipsis; a caret marks a menu (${dots.map(x => x.b.id).join(', ') || 'none'})`);
    const plots = RIBBON.find(t => t.id === 'plots');
    const figures = plots?.groups.find(g => g.name === 'Figures');
    check(!!figures && figures.buttons.some(b => b.id === 'dg-diagrams') && figures.buttons.some(b => b.id === 'pl-image' && b.label === 'Image'),
        'the Plots & Diagrams tab has a Figures group with Diagrams and Image');
    check(!plots?.groups.some(g => g.name === 'Options' && g.buttons.some(b => b.id === 'pl-image')), 'Image is not under the plot Options');
    const flow = all.filter(x => x.b.icon === 'diagram').map(x => x.b.id);
    check(flow.length === 1 && flow[0] === 'dg-diagrams', `the flowchart icon belongs to the Diagrams button alone (${flow.join(', ')})`);
    const order = RIBBON.map(t => t.id).join(',');
    check(order === 'math,io,functions,matrices,calc,plots,mathfmt,textfmt,document,resources', `the tabs follow the work: write, calculate, plot, format, document (${order})`);
    const help = RIBBON.find(t => t.id === 'resources')?.groups.find(g => g.name === 'Help')?.buttons.map(b => b.id).join(',');
    check(help === 'x-docs,x-learn,x-keys,x-changelog,x-about', `Resources > Help: Documentation, Calculators and guides, Shortcuts, Changelog, About (${help})`);
    const empty = RIBBON.flatMap(t => t.groups.filter(g => !g.buttons.length).map(g => `${t.label}: ${g.name}`));
    check(empty.length === 0, `no empty group (${empty.join(', ') || 'none'})`);
}

// ---- the page ------------------------------------------------------------------------------------
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1300, height: 800 } });
const page = await ctx.newPage();
const errors = [];
page.on('pageerror', e => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
await page.goto(url);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
await page.waitForSelector('.rb-tabs');

for (const w of [1300, 1024, 920]) {
    await page.setViewportSize({ width: w, height: 800 });
    await page.waitForTimeout(150);
    const m = await page.locator('.rb-tabs').evaluate(el => ({ sw: el.scrollWidth, cw: el.clientWidth }));
    check(m.sw <= m.cw + 1, `the tab strip fits at ${w} px without scrolling (${m.sw} in ${m.cw})`);
}
await page.setViewportSize({ width: 1300, height: 800 });
const wide = [];
const untitled = [];
for (const t of RIBBON) {
    await page.locator(`[data-ribbon-tab=${t.id}]`).click();
    await page.waitForTimeout(100);
    const m = await page.locator('.rb-panel').evaluate(el => ({ sw: el.scrollWidth, cw: el.clientWidth }));
    if (m.sw > m.cw + 1) wide.push(`${t.label} (${m.sw})`);
    untitled.push(...await page.evaluate(() => [...document.querySelectorAll('.rb-panel [data-ribbon-btn]')]
        .filter(b => !(b.getAttribute('title') ?? '').trim()).map(b => b.dataset.ribbonBtn)));
    const names = await page.locator('.rb-panel .rb-gname').allInnerTexts();
    check(JSON.stringify(names) === JSON.stringify(t.groups.map(g => g.name)), `${t.label}: the groups read ${names.join(', ')}`);
}
check(wide.length === 0, `every tab's buttons fit a 1300 px window (${wide.join(', ') || 'all fit'})`);
check(untitled.length === 0, `every rendered ribbon button has a tooltip (${untitled.join(', ') || 'all'})`);
{
    await page.locator('[data-ribbon-tab=plots]').click();
    const g = page.locator('.rb-group', { has: page.locator('[data-ribbon-btn=dg-diagrams]') });
    check(await g.locator('.rb-gname').innerText() === 'Figures' && await g.locator('[data-ribbon-btn=pl-image]').count() === 1, 'on the page, Diagrams and Image share the Figures group');
    const btn = page.locator('[data-ribbon-btn=dg-diagrams]');
    check(await btn.getAttribute('aria-haspopup') === 'dialog' && await btn.getAttribute('aria-expanded') === 'false', 'a menu button says it opens a popover, and that it is closed');
    await btn.click();
    check(await btn.getAttribute('aria-expanded') === 'true', 'and that it is open once clicked');
    const box = await page.locator('.rb-pop').boundingBox();
    check(!!box && box.x >= 0 && box.x + box.width <= 1300 && box.y + box.height <= 800, `the popover stays on screen (${JSON.stringify(box)})`);
    await page.keyboard.press('Escape');
}
// Keyboard: Tab reaches the ribbon buttons in their visual order, and Enter runs one.
{
    await page.locator('[data-ribbon-tab=calc]').click();
    await page.locator('[data-ribbon-btn=k-calc]').focus();
    const seen = [];
    for (let i = 0; i < 4; i++) { seen.push(await page.evaluate(() => document.activeElement?.dataset?.ribbonBtn)); await page.keyboard.press('Tab'); }
    check(seen.join(',') === 'k-calc,k-auto,k-preview,k-unwrapped', `Tab moves through the Calculation buttons in order (${seen.join(',')})`);
}
check(errors.length === 0, `no page errors (${errors.slice(0, 2).join(' | ')})`);
await browser.close();
console.log(fails.length ? `${fails.length} FAILED` : 'ribbon layout check passed');
process.exit(fails.length ? 1 : 0);
