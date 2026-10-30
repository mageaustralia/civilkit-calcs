// Spreadsheet import, the pure logic: reading .xlsx (shared strings, inline strings, numbers, booleans, a formula with its stored
// value; stored and deflated zip parts) and .csv/.tsv (quotes, delimiter, decimal comma), the size and format guards, name and unit
// guessing, and the sheet lines each action writes.   node e2e/sheet-import-logic-check.mjs
import { deflateRawSync } from 'node:zlib';
import { buildZip } from '../calcpad-frontend/src/services/zip-writer.ts';
import { parseWorkbook, parseCsv, cellAt, a1, colName, colIndex, rangeA1, displayValue } from '../calcpad-web/src/wasm/sheet-import-parse.ts';
import {
    toSheetName, splitLabelUnit, cleanUnit, uniqueName, namesInSheet, guessLabel, proposeForCell, proposeForRange, numStr, provenance,
    buildInput, buildValue, buildTable, buildLabelled, inputLine, valueLine,
} from '../calcpad-web/src/wasm/sheet-import-lines.ts';

const fails = [];
const eq = (got, want, what) => { const ok = JSON.stringify(got) === JSON.stringify(want); console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) { console.log('   got ', JSON.stringify(got), '\n   want', JSON.stringify(want)); fails.push(what); } };
const rejects = async (fn, re, what) => { try { await fn(); eq('no error', 'an error', what); } catch (e) { const ok = re.test(e.message); console.log(`${ok ? 'ok  ' : 'FAIL'} ${what} (${e.message})`); if (!ok) fails.push(what); } };
const enc = (s) => new TextEncoder().encode(s);

// A zip with some parts deflated (as Excel writes it) and the rest stored (buildZip).
function zipMixed(entries) {
    const stored = buildZip(entries.map((e) => ({ name: e.name, bytes: enc(e.text) })));
    return stored;
}
function zipDeflated(entries) {
    const parts = []; const central = []; let off = 0;
    for (const e of entries) {
        const raw = Buffer.from(e.text); const comp = deflateRawSync(raw); const name = Buffer.from(e.name);
        const lh = Buffer.alloc(30); lh.writeUInt32LE(0x04034b50, 0); lh.writeUInt16LE(20, 4); lh.writeUInt16LE(8, 8); lh.writeUInt32LE(0, 14); lh.writeUInt32LE(comp.length, 18); lh.writeUInt32LE(raw.length, 22); lh.writeUInt16LE(name.length, 26);
        parts.push(lh, name, comp);
        const ch = Buffer.alloc(46); ch.writeUInt32LE(0x02014b50, 0); ch.writeUInt16LE(20, 4); ch.writeUInt16LE(20, 6); ch.writeUInt16LE(8, 10); ch.writeUInt32LE(comp.length, 20); ch.writeUInt32LE(raw.length, 24); ch.writeUInt16LE(name.length, 28); ch.writeUInt32LE(off, 42);
        central.push(ch, name); off += 30 + name.length + comp.length;
    }
    const cd = Buffer.concat(central); const end = Buffer.alloc(22); end.writeUInt32LE(0x06054b50, 0); end.writeUInt16LE(entries.length, 8); end.writeUInt16LE(entries.length, 10); end.writeUInt32LE(cd.length, 12); end.writeUInt32LE(off, 16);
    return new Uint8Array(Buffer.concat([...parts, cd, end]));
}

const WB = '<?xml version="1.0"?><workbook xmlns="x" xmlns:r="r"><sheets><sheet name="Loads &amp; spans" sheetId="1" r:id="rId1"/><sheet name="Second" sheetId="2" r:id="rId2"/></sheets></workbook>';
const RELS = '<Relationships><Relationship Id="rId1" Type="t" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="t" Target="/xl/worksheets/sheet2.xml"/></Relationships>';
const SST = '<sst><si><t>Span (m)</t></si><si><t>Design load [kN/m]</t></si><si><r><t>Bending </t></r><r><t>moment</t></r></si><si><t>kN*m</t></si><si><t xml:space="preserve">a &lt; b</t></si></sst>';
const S1 = `<worksheet><sheetData>
<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1"><v>6</v></c></row>
<row r="2"><c r="A2" t="s"><v>1</v></c><c r="B2"><v>12.5</v></c></row>
<row r="3"><c r="A3" t="s"><v>2</v></c><c r="B3"><f>B2*B1^2/8</f><v>56.25</v></c><c r="C3" t="s"><v>3</v></c></row>
<row r="4"><c r="A4" t="inlineStr"><is><t>Fitted</t></is></c><c r="B4" t="b"><v>1</v></c><c r="C4" t="str"><f>"x"</f><v>text result</v></c><c r="D4" t="e"><v>#DIV/0!</v></c></row>
<row r="5"><c r="A5" t="s"><v>4</v></c><c r="B5"><v>45000</v></c></row>
</sheetData><mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells></worksheet>`;
const S2 = '<worksheet><sheetData><row r="2"><c r="C2"><v>1.5</v></c></row></sheetData></worksheet>';
const parts = [
    { name: '[Content_Types].xml', text: '<Types/>' }, { name: 'xl/workbook.xml', text: WB }, { name: 'xl/_rels/workbook.xml.rels', text: RELS },
    { name: 'xl/sharedStrings.xml', text: SST }, { name: 'xl/worksheets/sheet1.xml', text: S1 }, { name: 'xl/worksheets/sheet2.xml', text: S2 },
];

for (const [label, bytes] of [['stored', zipMixed(parts)], ['deflated', zipDeflated(parts)]]) {
    const wb = await parseWorkbook('loads.xlsx', bytes);
    eq(wb.sheets.map((s) => s.name), ['Loads & spans', 'Second'], `${label}: sheet names, entities decoded, relationship targets resolved`);
    const s = wb.sheets[0];
    eq([s.nrows, s.ncols], [5, 4], `${label}: used size`);
    eq(cellAt(s, 0, 0), { t: 's', v: 'Span (m)' }, `${label}: shared string`);
    eq(cellAt(s, 0, 1), { t: 'n', v: 6 }, `${label}: number`);
    eq(cellAt(s, 2, 0).v, 'Bending moment', `${label}: rich-text shared string joined`);
    eq(cellAt(s, 2, 1), { t: 'n', v: 56.25, f: 'B2*B1^2/8' }, `${label}: a formula cell keeps its stored value and formula text`);
    eq(cellAt(s, 3, 0), { t: 's', v: 'Fitted' }, `${label}: inline string`);
    eq(cellAt(s, 3, 1), { t: 'b', v: true }, `${label}: boolean`);
    eq(cellAt(s, 3, 2), { t: 's', v: 'text result', f: '"x"' }, `${label}: a formula that gives text`);
    eq(cellAt(s, 3, 3).t, 'e', `${label}: an error value`);
    eq(cellAt(s, 4, 0).v, 'a < b', `${label}: preserved-space text with entities`);
    eq(cellAt(s, 4, 1).v, 45000, `${label}: a date serial stays a number`);
    eq(cellAt(wb.sheets[1], 1, 2), { t: 'n', v: 1.5 }, `${label}: second sheet, absolute target`);
}

// Format and size guards
await rejects(() => parseWorkbook('x.xlsx', enc('this is not a zip')), /not a valid \.xlsx/, 'text with an .xlsx name is refused');
await rejects(() => parseWorkbook('x.xlsx', zipMixed([{ name: 'readme.txt', text: 'hi' }])), /not an Excel workbook/, 'a zip without a workbook is refused');
await rejects(() => parseWorkbook('x.xlsx', new Uint8Array([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1, 0, 0, 0, 0])), /password-protected/, 'an OLE container (a protected workbook) is refused with a clear message');
await rejects(() => parseWorkbook('old.xls', enc('x')), /\.xls format/, 'old .xls is refused');
await rejects(() => parseWorkbook('x.xlsx', zipDeflated(parts).subarray(0, 60)), /not a valid \.xlsx/, 'a truncated zip is refused');
await rejects(() => parseWorkbook('big.csv', new Uint8Array(5 * 1024 * 1024 + 1)), /MB/, 'over 5 MB is refused');
await rejects(() => parseWorkbook('e.csv', new Uint8Array(0)), /empty/, 'an empty file is refused');
await rejects(() => parseWorkbook('note.docx', enc('x')), /Choose a/, 'an unknown extension is refused');
const manyCells = '<worksheet><sheetData>' + Array.from({ length: 2100 }, (_, r) => `<row r="${r + 1}">` + Array.from({ length: 100 }, (_, c) => `<c r="${colName(c)}${r + 1}"><v>1</v></c>`).join('') + '</row>').join('') + '</worksheet>';
await rejects(() => parseWorkbook('many.xlsx', zipDeflated([{ name: 'xl/workbook.xml', text: '<workbook><sheets><sheet name="S" r:id="rId1"/></sheets></workbook>' }, { name: 'xl/_rels/workbook.xml.rels', text: '<Relationships><Relationship Id="rId1" Target="worksheets/sheet1.xml"/></Relationships>' }, { name: 'xl/worksheets/sheet1.xml', text: manyCells }])), /more than 200,000 cells/, 'more than 200,000 cells is refused');

// CSV
let c = parseCsv('Name;Value;Unit\nSpan;6,5;m\n"Load; design";12.5;"kN/m"\n', 'a.csv');
eq([c.sheets[0].nrows, c.sheets[0].ncols], [3, 3], 'csv with semicolons: size');
eq(cellAt(c.sheets[0], 1, 1), { t: 'n', v: 6.5 }, 'decimal comma accepted when the delimiter is a semicolon');
eq(cellAt(c.sheets[0], 2, 0).v, 'Load; design', 'a quoted field keeps its delimiter');
c = parseCsv('a,b\n"x, y",2\n"say ""hi""",3.5\r\n', 'b.csv');
eq([cellAt(c.sheets[0], 1, 0).v, cellAt(c.sheets[0], 2, 0).v, cellAt(c.sheets[0], 2, 1).v], ['x, y', 'say "hi"', 3.5], 'csv with quoted commas and doubled quotes');
c = parseCsv('a\tb\n1\t2,5\n', 'c.tsv');
eq(cellAt(c.sheets[0], 1, 1).v, 2.5, 'tsv with a decimal comma');
c = parseCsv('﻿a,b\n1,2\n', 'd.csv');
eq(cellAt(c.sheets[0], 0, 0).v, 'a', 'a byte order mark is dropped');
c = parseCsv('"multi\nline",5\n', 'e.csv');
eq([cellAt(c.sheets[0], 0, 0).v, c.sheets[0].nrows], ['multi\nline', 1], 'a quoted field can hold a line break');
await rejects(async () => parseCsv('a,"b\n1,2', 'f.csv'), /quote/, 'an unclosed quote is refused');
await rejects(async () => parseCsv('a\u0000b', 'g.csv'), /not text/, 'binary data in a .csv is refused');

// Helpers
eq([colName(0), colName(25), colName(26), colName(701), colIndex('AA'), a1(2, 1), rangeA1(0, 0, 2, 1)], ['A', 'Z', 'AA', 'ZZ', 26, 'B3', 'A1:B3'], 'column letters and ranges');
eq(displayValue({ t: 'n', v: 0.1 + 0.2 }), '0.3', 'display rounds float noise');

// Names and units
eq(splitLabelUnit('Length (m)'), { base: 'Length', unit: 'm' }, 'unit in round brackets');
eq(splitLabelUnit('Load [kN/m]'), { base: 'Load', unit: 'kN/m' }, 'unit in square brackets');
eq(splitLabelUnit('Moment [kN·m]'), { base: 'Moment', unit: 'kN*m' }, 'a middle dot becomes *');
eq(splitLabelUnit('Area (mm²)'), { base: 'Area', unit: 'mm^2' }, 'a superscript becomes ^');
eq(splitLabelUnit('Cover (optional)'), { base: 'Cover (optional)', unit: '' }, 'a word in brackets is not a unit');
eq(splitLabelUnit('Gravity (see note 3)'), { base: 'Gravity (see note 3)', unit: '' }, 'a phrase in brackets is not a unit');
eq(cleanUnit('kN m'), '', 'a unit with a space is not accepted');
eq([toSheetName('Design load (kN/m)'), toSheetName('Span'), toSheetName('Yield strength fy'), toSheetName('φ factor'), toSheetName('Ω'), toSheetName('3rd floor'), toSheetName(''), toSheetName('if'), toSheetName('Bending moment, M*')], ['designLoad', 'span', 'yieldStrengthFy', 'phiFactor', 'Omega', 'n3rdFloor', 'value', 'if_', 'bendingMomentM'], 'names from labels: camel case, Greek spelled, no leading digit, no keyword');
eq(uniqueName('span', new Set(['span', 'span2'])), 'span3', 'a name is made unique');
eq(uniqueName('phi', namesInSheet('φ: 0.9')), 'phi2', 'a Greek letter and its spelling count as the same name');
const present = namesInSheet('<!--sheet-->\n# T\nL: ? 6 m | span\nw = 3 kN/m\na: 1; b: 2\nf(x): x^2\n>> raw = 1\nA note: see below\nphi: 0.9');
eq([...present].sort(), ['L', 'a', 'b', 'f', 'phi', 'w'].sort(), 'names already in the sheet');
eq([numStr(1e21), numStr(1.5e-7), numStr(0.1 + 0.2), numStr(-12.5), numStr(1230000)], ['1000000000000000000000', '0.00000015', '0.3', '-12.5', '1230000'], 'numbers are written without an exponent');
eq(provenance('loads.xlsx', 'Loads & spans', 'B1'), "<!-- loads.xlsx, 'Loads & spans'!B1 -->", 'provenance comment quotes a sheet name with spaces');
eq(provenance('a--b>.csv', 'Sheet1', 'A1:B2'), '<!-- a-b.csv, Sheet1!A1:B2 -->', 'provenance cannot close its own comment');

// Guessing from the workbook
const wb = await parseWorkbook('loads.xlsx', zipDeflated(parts));
const s = wb.sheets[0];
eq(guessLabel(s, 0, 1)?.text, 'Span (m)', 'label from the cell on the left');
let p = proposeForCell(s, 0, 1, new Set());
eq([p.name, p.unit], ['span', 'm'], 'name and unit from "Span (m)"');
p = proposeForCell(s, 1, 1, new Set(['designLoad']));
eq([p.name, p.unit], ['designLoad2', 'kN/m'], 'a taken name gets a number; unit from [kN/m]');
p = proposeForCell(s, 2, 1, new Set());
eq([p.name, p.unit], ['bendingMoment', 'kN*m'], 'unit from an adjacent unit cell');
// header above
const csv = parseCsv('Length (m),Width (m)\n6,4\n8,5\n', 'dims.csv').sheets[0];
p = proposeForCell(csv, 1, 1, new Set());
eq([p.name, p.unit], ['width', 'm'], 'name and unit from the header above');
p = proposeForRange(csv, 1, 0, new Set());
eq([p.name, p.unit], ['length', 'm'], 'a range is named from its header');

// Lines
eq(inputLine('span', 6, 'm', 'Span'), 'span: ? 6 m | Span', 'input line');
eq(valueLine('Mstar', 56.25, 'kN*m'), 'Mstar: 56.25 kN*m', 'value line');
let b = buildInput(s, 'loads.xlsx', 0, 1, 'span', 'm', 'Span');
eq(b.text, '<!-- loads.xlsx, Sheet1!B1 -->\nspan: ? 6 m | Span'.replace('Sheet1', "'Loads & spans'"), 'add as input');
b = buildValue(s, 'loads.xlsx', 2, 1, 'M', 'kN*m');
eq(b.text.split('\n')[1], 'M: 56.25 kN*m', 'add as value from a formula cell uses the stored result');
eq(b.marks, [{ r: 2, c: 1, name: 'M' }], 'the cell is marked with its name');
try { buildInput(s, 'f', 2, 1, 'M', ''); eq('ok', 'error', 'a formula cell cannot be an input'); } catch (e) { eq(/formula/.test(e.message), true, 'a formula cell cannot be an input'); }
try { buildValue(s, 'f', 0, 0, 'x', ''); eq('ok', 'error', 'text cannot be a value'); } catch (e) { eq(/number/.test(e.message), true, 'text cannot be a value'); }
const grid = parseCsv('x,y,z\n1,2,3\n4,5,6\n', 'g.csv').sheets[0];
b = buildTable(grid, 'g.csv', { r0: 1, c0: 0, r1: 2, c1: 2 }, 'm1', '');
eq(b.text.split('\n')[1], 'm1: [1, 2, 3; 4, 5, 6]', 'numbers as a matrix');
b = buildTable(grid, 'g.csv', { r0: 0, c0: 0, r1: 2, c1: 2 }, 'm1', 'm');
eq(b.text, '<!-- g.csv, g!A1:C3 (columns: x, y, z) -->\nm1: [1 m, 2 m, 3 m; 4 m, 5 m, 6 m]', 'a text header row is left out of the matrix and listed in the comment');
b = buildTable(grid, 'g.csv', { r0: 1, c0: 1, r1: 2, c1: 1 }, 'col', '');
eq(b.text.split('\n')[1], 'col: [2; 5]', 'a column as a vector');
b = buildTable(grid, 'g.csv', { r0: 2, c0: 0, r1: 2, c1: 2 }, 'row', '');
eq(b.text.split('\n')[1], 'row: [4, 5, 6]', 'a row as a vector');
const mixed = parseCsv('item,qty\nbolt,4\nnut,x|y\n', 'mixed.csv').sheets[0];
b = buildTable(mixed, 'mixed.csv', { r0: 0, c0: 0, r1: 2, c1: 1 }, 't', '');
eq(b.text, '<!-- mixed.csv, mixed!A1:B3 -->\n\n| item | qty |\n| --- | --- |\n| bolt | 4 |\n| nut | x/y |\n', 'text as a pipe table, pipes replaced');
const rows = parseCsv('Span (m);6\nDesign load [kN/m];12,5\nNote;see drawing\nFitted;1\n', 'rows.csv').sheets[0];
b = buildLabelled(rows, 'rows.csv', { r0: 0, c0: 0, r1: 3, c1: 1 }, new Set(['span']), false);
eq(b.text.split('\n').slice(1), ['span2: 6 m | Span', 'designLoad: 12.5 kN/m | Design load', 'fitted: 1 | Fitted'], 'labelled rows: names, units from labels, unique against the sheet, text value row skipped');
eq(b.skipped, 1, 'the skipped row is counted');
const withUnits = parseCsv('Depth;300;mm\nWidth;150;mm\n', 'u.csv').sheets[0];
b = buildLabelled(withUnits, 'u.csv', { r0: 0, c0: 0, r1: 1, c1: 2 }, new Set(), true);
eq(b.text.split('\n').slice(1), ['depth: ? 300 mm | Depth', 'width: ? 150 mm | Width'], 'labelled rows from a unit column, as inputs');
try { buildLabelled(rows, 'r', { r0: 0, c0: 0, r1: 0, c1: 0 }, new Set(), false); eq('ok', 'error', 'one column is refused'); } catch (e) { eq(/two columns/.test(e.message), true, 'one column is refused'); }

console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
