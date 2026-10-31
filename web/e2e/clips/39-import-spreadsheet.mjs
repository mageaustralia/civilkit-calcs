// 26.10.20: Import spreadsheet opens an Excel or CSV file in a panel; a cell or a range is added to the sheet as an input,
// a value, a table or one quantity per labelled row. The file here is generated for the clip: a small load table.
import { Buffer } from 'node:buffer';
import { sleep } from './lib.mjs';
import { buildZip } from '../../calcpad-frontend/src/services/zip-writer.ts';

const enc = (s) => new TextEncoder().encode(s);
const strings = ['Load', 'Value', 'Dead load', 'Live load', 'Wind load', 'Span (m)', 'Unit', 'kN/m'];
const cellS = (ref, i) => `<c r="${ref}" t="s"><v>${i}</v></c>`;
const XLSX = Buffer.from(buildZip([
    ['[Content_Types].xml', '<Types/>'],
    ['xl/workbook.xml', '<workbook xmlns:r="r"><sheets><sheet name="Loads" sheetId="1" r:id="rId1"/></sheets></workbook>'],
    ['xl/_rels/workbook.xml.rels', '<Relationships><Relationship Id="rId1" Target="worksheets/sheet1.xml"/></Relationships>'],
    ['xl/sharedStrings.xml', `<sst>${strings.map((s) => `<si><t>${s}</t></si>`).join('')}</sst>`],
    ['xl/worksheets/sheet1.xml', `<worksheet><sheetData><row r="1">${cellS('A1', 0)}${cellS('B1', 1)}${cellS('C1', 6)}</row><row r="2">${cellS('A2', 2)}<c r="B2"><v>4.5</v></c>${cellS('C2', 7)}</row><row r="3">${cellS('A3', 3)}<c r="B3"><v>3</v></c>${cellS('C3', 7)}</row><row r="4">${cellS('A4', 4)}<c r="B4"><v>1.2</v></c>${cellS('C4', 7)}</row><row r="5">${cellS('A5', 5)}<c r="B5"><v>6</v></c></row></sheetData></worksheet>`],
].map(([name, text]) => ({ name, bytes: enc(text) }))));

const START = '<!--sheet-->\n# Beam loads\n\n';

export default {
    id: 'import-spreadsheet',
    viewport: { width: 1280, height: 800 },
    async run(rec) {
        const { page } = rec;
        await page.evaluate((t) => window.calcpadTabs.newUntitled(t), START);
        await page.waitForFunction(() => window.calcpadActiveEditor.getValue().startsWith('<!--sheet-->'));
        await page.click('[data-ribbon-tab=resources]');
        await page.click('[data-ribbon-btn=x-sidebar]');
        await page.evaluate(() => { const ed = window.calcpadActiveEditor; ed.updateOptions({ fontSize: 17, lineHeight: 28 }); const n = ed.getModel().getLineCount(); ed.focus(); ed.setPosition({ lineNumber: n, column: 1 }); });
        await page.click('[data-ribbon-tab=document]');
        await sleep(1200);
        await rec.begin();
        await sleep(1200);
        const chooser = page.waitForEvent('filechooser', { timeout: 15000 });
        await rec.click(page.locator('[data-ribbon-btn=d-importsheet]'), { ms: 800 });
        await (await chooser).setFiles({ name: 'loads.xlsx', mimeType: 'application/octet-stream', buffer: XLSX });
        await page.waitForSelector('.sip');
        await sleep(2000);
        rec.mark('poster');
        const cell = (r, c) => page.locator(`.sip td[data-r="${r}"][data-c="${c}"]`);
        // one labelled range: three loads, each a line
        const a = await cell(1, 0).boundingBox(); const z = await cell(3, 2).boundingBox();
        await rec.glide(a.x + 8, a.y + 8, 800);
        await page.mouse.down();
        await rec.glide(z.x + z.width - 8, z.y + 8, 900);
        await page.mouse.up();
        await sleep(1300);
        await rec.click(page.locator('[data-sip=add-rows]'), { ms: 700 });
        await sleep(2200);
        // one cell as an input
        await rec.click(cell(4, 1), { ms: 800 });
        await sleep(1100);
        await rec.click(page.locator('[data-sip=add-input]'), { ms: 700 });
        await sleep(2600);
        await rec.finish();
    },
};
