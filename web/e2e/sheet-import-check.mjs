// "Import spreadsheet": pick an .xlsx (built here with the zip writer, nothing committed) and a .csv, see the cells in the panel,
// add an input, a value, a table and labelled rows to a sheet at the cursor, and see the sheet calculate with them. Also a damaged
// file, a file that is not a sheet's, and no page errors on desktop and on a phone (iPhone 14).
//   node e2e/sheet-import-check.mjs [--url http://127.0.0.1:5194/] [--shots DIR]
import { chromium, devices } from 'playwright';
import { mkdirSync } from 'node:fs';
import { buildZip } from '../calcpad-frontend/src/services/zip-writer.ts';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5194/');
const shots = arg('--shots', '/Volumes/second_drive/Development/civilkit-calcs/.trash/shots');
mkdirSync(shots, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const enc = (s) => new TextEncoder().encode(s);

const xlsx = Buffer.from(buildZip([
    ['[Content_Types].xml', '<Types/>'],
    ['xl/workbook.xml', '<workbook xmlns:r="r"><sheets><sheet name="Loads" sheetId="1" r:id="rId1"/><sheet name="Notes" sheetId="2" r:id="rId2"/></sheets></workbook>'],
    ['xl/_rels/workbook.xml.rels', '<Relationships><Relationship Id="rId1" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Target="worksheets/sheet2.xml"/></Relationships>'],
    ['xl/sharedStrings.xml', '<sst><si><t>Span (m)</t></si><si><t>Design load [kN/m]</t></si><si><t>Bending moment</t></si><si><t>kN*m</t></si></sst>'],
    ['xl/worksheets/sheet1.xml', '<worksheet><sheetData><row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1"><v>6</v></c></row><row r="2"><c r="A2" t="s"><v>1</v></c><c r="B2"><v>12.5</v></c></row><row r="3"><c r="A3" t="s"><v>2</v></c><c r="B3"><f>B2*B1^2/8</f><v>56.25</v></c><c r="C3" t="s"><v>3</v></c></row></sheetData></worksheet>'],
    ['xl/worksheets/sheet2.xml', '<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Only a note</t></is></c></row></sheetData></worksheet>'],
].map(([name, text]) => ({ name, bytes: enc(text) }))));
const csvTable = Buffer.from('Length (m);Width (m)\n6;4\n8,5;5\n');
const csvRows = Buffer.from('Span (m);6\nDesign load [kN/m];12,5\n');
const bad = Buffer.from('this is not a zip file at all');

const SHEET = '<!--sheet-->\n# Beam\n\n';
const browser = await chromium.launch();

async function setup(opts, phone) {
    const ctx = await browser.newContext(opts);
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!/^(Canceled|Event)/.test(String(e.message))) errors.push(String(e.message)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.evaluate((t) => window.calcpadTabs.newUntitled(t), SHEET);
    await page.evaluate(() => { const ed = window.calcpadActiveEditor; const n = ed.getModel().getLineCount(); ed.setPosition({ lineNumber: n, column: 1 }); });
    return { ctx, page, errors, phone };
}
const text = (page) => page.evaluate(() => window.calcpadActiveEditor.getValue());
const front = async (page, need) => {
    for (let i = 0; i < 80; i++) {
        const h = await page.$('iframe.preview-frame:not(.preview-frame-back)');
        const f = h && await h.contentFrame();
        if (f && await f.evaluate((n) => !!document.body && document.body.innerText.includes(n), need).catch(() => false)) return f;
        await page.waitForTimeout(250);
    }
    return null;
};
const cell = (page, r, c) => page.locator(`.sip td[data-r="${r}"][data-c="${c}"]`);
async function openFile(page, via, file) {
    const chooser = page.waitForEvent('filechooser', { timeout: 15000 });
    await via();
    await (await chooser).setFiles(file);
}
const hook = (page) => () => page.evaluate(() => void window.calcpadFiles.files.o.sheetImport.pick());
const tapOrClick = (loc, phone) => (phone ? loc.tap() : loc.click());

// ------------------------------------------------------------------------------------------------ desktop
{
    const { ctx, page, errors } = await setup({ viewport: { width: 1360, height: 900 } }, false);
    // the ribbon button (Document > Import) and the File menu item
    await page.locator('[data-ribbon-tab=document]').click();
    const btn = page.locator('[data-ribbon-btn=d-importsheet]');
    check(await btn.isVisible(), 'desktop: the Import spreadsheet button is in the Document tab');
    await openFile(page, () => btn.click(), { name: 'loads.xlsx', mimeType: 'application/octet-stream', buffer: xlsx });
    await page.waitForSelector('.sip');
    check((await page.locator('.sip-tabs button').allInnerTexts()).join() === 'Loads,Notes', 'desktop: a tab for each sheet');
    check(await cell(page, 0, 1).innerText() === '6' && await cell(page, 1, 1).innerText() === '12.5', 'desktop: the cell values are shown');
    check(await cell(page, 2, 1).innerText() === '56.25' && await cell(page, 2, 1).getAttribute('class').then((c) => /\bf\b/.test(c)), 'desktop: a formula cell shows its stored value and is marked');
    check(await cell(page, 2, 1).getAttribute('title') === '=B2*B1^2/8', 'desktop: the formula text is the tooltip');
    check(await page.locator('.sip-grid th', { hasText: /^C$/ }).count() === 1 && await page.locator('.sip-grid tbody th', { hasText: /^3$/ }).count() === 1, 'desktop: column letters and row numbers');
    // a constant: an input
    await cell(page, 0, 1).click();
    check(await page.locator('[data-sip=name]').inputValue() === 'span' && await page.locator('[data-sip=unit]').inputValue() === 'm', 'desktop: name and unit guessed from "Span (m)"');
    await page.locator('[data-sip=add-input]').click();
    let t = await text(page);
    check(/<!-- loads\.xlsx, Loads!B1 -->\nspan: \? 6 m \| Span(\n|$)/m.test(t), 'desktop: Add as input writes a source comment and an input line at the cursor');
    check(await cell(page, 0, 1).getAttribute('data-name') === 'span' && /added/.test(await cell(page, 0, 1).getAttribute('class')), 'desktop: the added cell is marked with its name');
    // an edited name and unit
    await cell(page, 1, 1).click();
    check(await page.locator('[data-sip=unit]').inputValue() === 'kN/m', 'desktop: unit guessed from [kN/m]');
    await page.locator('[data-sip=name]').fill('w');
    await page.locator('[data-sip=add-value]').click();
    // a formula cell: not an input, but a value
    await cell(page, 2, 1).click();
    check(await page.locator('[data-sip=add-input]').isDisabled(), 'desktop: a formula cell cannot be added as an input');
    check(await page.locator('[data-sip=unit]').inputValue() === 'kN*m', 'desktop: unit from the cell to the right');
    await page.locator('[data-sip=add-value]').click();
    t = await text(page);
    check(/\nw: 12\.5 kN\/m \| Design load\n/.test(t) && /\nbendingMoment: 56\.25 kN\*m \| Bending moment\n?/.test(t), 'desktop: values, the edited name kept, the formula cell gives its stored result');
    // the sheet calculates with them
    await page.evaluate(() => { const ed = window.calcpadActiveEditor; const m = ed.getModel(); const n = m.getLineCount(); ed.executeEdits('t', [{ range: { startLineNumber: n, startColumn: m.getLineMaxColumn(n), endLineNumber: n, endColumn: m.getLineMaxColumn(n) }, text: '\ncheck: bendingMoment / (span * w) -> m\n' }]); });
    const f = await front(page, 'check');
    const body = f ? await f.evaluate(() => document.body.innerText) : '';
    check(/0\.75\s*m/.test(body), `desktop: the sheet evaluates with the imported values (check = 0.75 m)`);
    check(!/<!--/.test(body) && !/xlsx/.test(body), 'desktop: the source comment does not show in the preview');
    // marks stay when the tab is changed back and forth
    await page.locator('[data-sip-tab="1"]').click();
    check(await page.locator('.sip td[data-r="0"][data-c="0"]').innerText() === 'Only a note', 'desktop: the second sheet shows');
    await page.locator('[data-sip-tab="0"]').click();
    check(await cell(page, 0, 1).getAttribute('data-name') === 'span', 'desktop: the marks survive a change of tab');
    await page.screenshot({ path: `${shots}/desktop-xlsx.png` });
    // a damaged file
    await openFile(page, () => page.locator('[data-ribbon-btn=d-importsheet]').click(), { name: 'broken.xlsx', mimeType: 'application/octet-stream', buffer: bad });
    await page.waitForSelector('.sip-modal');
    check(/broken\.xlsx: .*not a valid \.xlsx/.test(await page.locator('.sip-modal').innerText()), 'desktop: a damaged file gives a clear message');
    await page.keyboard.press('Escape');
    check(await page.locator('.sip-modal').count() === 0, 'desktop: the message closes');
    check(await page.locator('.sip').count() === 1, 'desktop: the earlier panel stays after a failed import');
    // a csv with semicolons: a table by dragging over a range
    await openFile(page, () => page.locator('[data-ribbon-btn=d-importsheet]').click(), { name: 'dims.csv', mimeType: 'text/csv', buffer: csvTable });
    await page.waitForFunction(() => document.querySelector('.sip-file')?.textContent === 'dims.csv');
    const a = await cell(page, 1, 0).boundingBox(); const z = await cell(page, 2, 1).boundingBox();
    await page.mouse.move(a.x + 5, a.y + 5); await page.mouse.down(); await page.mouse.move(z.x + 5, z.y + 5, { steps: 4 }); await page.mouse.up();
    check(await page.locator('[data-sip=selection]').innerText() === 'dims!A2:B3', 'desktop: dragging selects a range');
    check(await page.locator('[data-sip=name]').inputValue() === 'length' && await page.locator('[data-sip=unit]').inputValue() === 'm', 'desktop: a range is named from its header');
    check(await page.locator('[data-sip=add-input]').isHidden() && await page.locator('[data-sip=add-table]').isVisible(), 'desktop: a range offers the table actions');
    await page.locator('[data-sip=add-table]').click();
    t = await text(page);
    check(/\nlength: \[6 m, 4 m; 8\.5 m, 5 m\]\n?/.test(t), 'desktop: Add as table writes a matrix (decimal comma read)');
    // labelled rows
    await openFile(page, () => page.locator('[data-ribbon-btn=d-importsheet]').click(), { name: 'rows.csv', mimeType: 'text/csv', buffer: csvRows });
    await page.waitForFunction(() => document.querySelector('.sip-file')?.textContent === 'rows.csv');
    const r0 = await cell(page, 0, 0).boundingBox(); const r1 = await cell(page, 1, 1).boundingBox();
    await page.mouse.move(r0.x + 5, r0.y + 5); await page.mouse.down(); await page.mouse.move(r1.x + 5, r1.y + 5, { steps: 4 }); await page.mouse.up();
    await page.locator('[data-sip=add-rows]').click();
    t = await text(page);
    check(/\nspan2: 6 m \| Span\ndesignLoad: 12\.5 kN\/m \| Design load\n?/.test(t), 'desktop: Add labelled rows writes a line per row, names unique in the sheet');
    check(/added/.test(await cell(page, 1, 1).getAttribute('class')) && await cell(page, 1, 1).getAttribute('data-name') === 'designLoad', 'desktop: the used cells are marked');
    // the whole sheet still evaluates
    await page.evaluate(() => { const ed = window.calcpadActiveEditor; const m = ed.getModel(); const n = m.getLineCount(); ed.executeEdits('t', [{ range: { startLineNumber: n, startColumn: m.getLineMaxColumn(n), endLineNumber: n, endColumn: m.getLineMaxColumn(n) }, text: '\narea: length.(2, 1) * span2 -> m^2\n' }]); });
    const f2 = await front(page, 'area');
    const body2 = f2 ? await f2.evaluate(() => document.body.innerText) : '';
    check(/51\s*m/.test(body2) || /8\.5\D+6\D+51/.test(body2), `desktop: the table and the rows are usable in calculations (area = 51 m^2)`);
    if (!/51/.test(body2)) console.log('   preview text:', body2.slice(0, 600).replace(/\n/g, ' | '));
    // the File menu has the item too
    await page.locator('[data-ribbon-btn=file]').click();
    check(await page.locator('text=Import spreadsheet...').count() > 0, 'desktop: the File menu has Import spreadsheet');
    await page.keyboard.press('Escape');
    // the active file is not a sheet
    await page.evaluate(() => window.calcpadTabs.newUntitled('a = 1\n'));
    await cell(page, 0, 0).click().catch(() => {});
    await page.locator('.sip td[data-r="1"][data-c="1"]').click();
    await page.locator('[data-sip=add-value]').click();
    check(/not a sheet/.test(await page.locator('[data-sip=status]').innerText()), 'desktop: adding into a Calcpad worksheet says it is not a sheet');
    check(errors.length === 0, `desktop: no page errors ${errors.join('; ')}`);
    await ctx.close();
}

// dark theme look
{
    const { ctx, page, errors } = await setup({ viewport: { width: 1360, height: 900 }, colorScheme: 'dark' }, false);
    await openFile(page, hook(page), { name: 'loads.xlsx', mimeType: 'application/octet-stream', buffer: xlsx });
    await page.waitForSelector('.sip');
    await cell(page, 1, 1).click();
    await page.locator('[data-sip=add-value]').click();
    await page.screenshot({ path: `${shots}/desktop-dark.png` });
    check(errors.length === 0, `dark: no page errors ${errors.join('; ')}`);
    await ctx.close();
}

// ------------------------------------------------------------------------------------------------ phone
{
    const { ctx, page, errors } = await setup({ ...devices['iPhone 14'] }, true);
    await openFile(page, hook(page), { name: 'loads.xlsx', mimeType: 'application/octet-stream', buffer: xlsx });
    await page.waitForSelector('.sip');
    const fit = await page.evaluate(() => { const r = document.querySelector('.sip').getBoundingClientRect(); return { l: r.left, r: r.right, b: r.bottom, w: innerWidth, h: innerHeight, sw: document.documentElement.scrollWidth, bw: document.body.scrollWidth }; });
    check(fit.l >= 0 && fit.r <= fit.w + 0.5 && fit.b <= fit.h + 0.5, `phone: the panel fits the screen (${JSON.stringify(fit)})`);
    check(fit.sw <= fit.w && fit.bw <= fit.w, 'phone: no sideways page scroll');
    await cell(page, 0, 1).tap();
    check(await page.locator('[data-sip=name]').inputValue() === 'span', 'phone: a tap selects a cell and proposes a name');
    await page.locator('[data-sip=add-input]').tap();
    check(/\nspan: \? 6 m \| Span\n?/.test(await text(page)), 'phone: Add as input works by touch');
    // a range by touch: tap, switch on Select range, tap the far corner
    await cell(page, 0, 0).tap();
    await page.locator('.sip-bar button', { hasText: 'Select range' }).tap();
    await cell(page, 1, 1).tap();
    check(await page.locator('[data-sip=selection]').innerText() === 'Loads!A1:B2', 'phone: a range by two taps');
    await page.locator('[data-sip=add-rows]').tap();
    check(/\nspan2: 6 m \| Span\n/.test(await text(page)), 'phone: Add labelled rows by touch');
    await page.screenshot({ path: `${shots}/phone-xlsx.png` });
    const fit2 = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, w: innerWidth, grid: document.querySelector('.sip-grid').scrollWidth > document.querySelector('.sip-grid').clientWidth }));
    check(fit2.sw <= fit2.w, 'phone: still no sideways page scroll after adding');
    await page.locator('[data-sip=close]').tap();
    check(await page.locator('.sip').count() === 0, 'phone: Close removes the panel');
    check(errors.length === 0, `phone: no page errors ${errors.join('; ')}`);
    await ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
