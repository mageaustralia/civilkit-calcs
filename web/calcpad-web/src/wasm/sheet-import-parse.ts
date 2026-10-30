// calcpad-rs: reading a spreadsheet file (.xlsx, .xlsm, .csv, .tsv) into plain cell values, for "Import spreadsheet".
// Nothing is recalculated: a formula cell keeps the value the file stored for it (Excel writes one) and the formula text, so
// the panel can show which cells are formulas. The file is untrusted: sizes are capped, the XML is scanned as text (no HTML is
// ever built from it) and every failure is an Error with a message a person can read. Plain TypeScript with no imports, so a
// Node check can load it (it uses only DecompressionStream and TextDecoder, which Node 22 and the browsers have).

export type CellKind = 'n' | 's' | 'b' | 'e';

export interface Cell {
    /** n number, s text, b boolean, e error value (#DIV/0! and the like). */
    t: CellKind;
    v: number | string | boolean;
    /** The formula text (no leading =) when the cell holds a formula; `v` is then the value stored in the file. */
    f?: string;
}

export interface SheetData {
    name: string;
    /** Rows and columns in use (the last used row/column + 1). */
    nrows: number;
    ncols: number;
    cells: Map<number, Cell>;
}

export interface Workbook {
    fileName: string;
    sheets: SheetData[];
}

export const MAX_BYTES = 5 * 1024 * 1024;
export const MAX_CELLS = 200000;
const MAX_XML_BYTES = 40 * 1024 * 1024;
const MAX_COLS = 16384;
const MAX_ROWS = 1048576;

/** The map key of a cell (zero-based row and column). */
export const cellKey = (r: number, c: number): number => r * MAX_COLS + c;
export const cellAt = (s: SheetData, r: number, c: number): Cell | undefined => s.cells.get(cellKey(r, c));

/** Column letters: 0 is A, 25 is Z, 26 is AA. */
export function colName(c: number): string {
    let n = c + 1;
    let s = '';
    while (n > 0) {
        const m = (n - 1) % 26;
        s = String.fromCharCode(65 + m) + s;
        n = Math.floor((n - 1) / 26);
    }
    return s;
}

/** Column letters to a zero-based index (A is 0); -1 when they are not letters. */
export function colIndex(letters: string): number {
    if (!/^[A-Za-z]{1,3}$/.test(letters)) return -1;
    let n = 0;
    for (const ch of letters.toUpperCase()) n = n * 26 + (ch.charCodeAt(0) - 64);
    return n - 1;
}

/** `B3` for row 2, column 1. */
export const a1 = (r: number, c: number): string => `${colName(c)}${r + 1}`;

/** `B3` or `B3:D8` for a range, zero-based and inclusive. */
export function rangeA1(r0: number, c0: number, r1: number, c1: number): string {
    return r0 === r1 && c0 === c1 ? a1(r0, c0) : `${a1(r0, c0)}:${a1(r1, c1)}`;
}

function parseRef(ref: string): { r: number; c: number } | null {
    const m = /^([A-Za-z]{1,3})(\d{1,7})$/.exec(ref);
    if (!m) return null;
    const c = colIndex(m[1]);
    const r = Number(m[2]) - 1;
    return c < 0 || r < 0 || c >= MAX_COLS || r >= MAX_ROWS ? null : { r, c };
}

// ---------------------------------------------------------------------------------------------------------------------
// ZIP

interface ZipEntry { name: string; method: number; csize: number; usize: number; offset: number; flags: number }

function readZipDirectory(b: Uint8Array): Map<string, ZipEntry> {
    const dv = new DataView(b.buffer, b.byteOffset, b.byteLength);
    let eocd = -1;
    for (let i = b.length - 22; i >= Math.max(0, b.length - 22 - 65535); i--) {
        if (dv.getUint32(i, true) === 0x06054b50) { eocd = i; break; }
    }
    if (eocd < 0) throw new Error('This is not a valid .xlsx file (the zip directory is missing). It may be damaged.');
    const count = dv.getUint16(eocd + 10, true);
    let p = dv.getUint32(eocd + 16, true);
    if (count === 0xffff || p === 0xffffffff) throw new Error('This workbook uses the zip64 format, which is not supported. Save it as .csv instead.');
    const out = new Map<string, ZipEntry>();
    const td = new TextDecoder();
    for (let i = 0; i < count; i++) {
        if (p + 46 > b.length || dv.getUint32(p, true) !== 0x02014b50) throw new Error('This is not a valid .xlsx file (the zip directory is damaged).');
        const flags = dv.getUint16(p + 8, true);
        const method = dv.getUint16(p + 10, true);
        const csize = dv.getUint32(p + 20, true);
        const usize = dv.getUint32(p + 24, true);
        const nl = dv.getUint16(p + 28, true);
        const xl = dv.getUint16(p + 30, true);
        const cl = dv.getUint16(p + 32, true);
        const offset = dv.getUint32(p + 42, true);
        const name = td.decode(b.subarray(p + 46, p + 46 + nl)).replace(/\\/g, '/');
        out.set(name.toLowerCase(), { name, method, csize, usize, offset, flags });
        p += 46 + nl + xl + cl;
    }
    return out;
}

async function inflateRaw(data: Uint8Array, limit: number): Promise<Uint8Array> {
    if (typeof DecompressionStream === 'undefined') throw new Error('This browser cannot unzip files (no DecompressionStream). Use a current browser, or save the workbook as .csv.');
    const stream = new Blob([data as BlobPart]).stream().pipeThrough(new DecompressionStream('deflate-raw'));
    const reader = stream.getReader();
    const chunks: Uint8Array[] = [];
    let total = 0;
    for (;;) {
        let r: ReadableStreamReadResult<Uint8Array>;
        try { r = await reader.read(); } catch { throw new Error('This workbook is damaged (a part of it does not unzip).'); }
        if (r.done) break;
        total += r.value.length;
        if (total > limit) { await reader.cancel().catch(() => {}); throw new Error('This workbook is too large once unzipped.'); }
        chunks.push(r.value);
    }
    const out = new Uint8Array(total);
    let o = 0;
    for (const c of chunks) { out.set(c, o); o += c.length; }
    return out;
}

async function readEntry(b: Uint8Array, dir: Map<string, ZipEntry>, name: string): Promise<string | null> {
    const e = dir.get(name.toLowerCase());
    if (!e) return null;
    if (e.flags & 1) throw new Error('This workbook is password-protected. Remove the password in the spreadsheet program and try again.');
    if (e.usize > MAX_XML_BYTES) throw new Error('This workbook has a part that is too large to read.');
    const dv = new DataView(b.buffer, b.byteOffset, b.byteLength);
    if (e.offset + 30 > b.length || dv.getUint32(e.offset, true) !== 0x04034b50) throw new Error('This is not a valid .xlsx file (a part is damaged).');
    const start = e.offset + 30 + dv.getUint16(e.offset + 26, true) + dv.getUint16(e.offset + 28, true);
    const raw = b.subarray(start, start + e.csize);
    let bytes: Uint8Array;
    if (e.method === 0) bytes = raw;
    else if (e.method === 8) bytes = await inflateRaw(raw, MAX_XML_BYTES);
    else throw new Error('This workbook uses a zip compression that is not supported. Save it as .csv instead.');
    return new TextDecoder('utf-8').decode(bytes);
}

// ---------------------------------------------------------------------------------------------------------------------
// XML, scanned as text

const ENTITIES: Record<string, string> = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'" };
export function xmlText(s: string): string {
    return s.replace(/&(#x[0-9a-fA-F]+|#\d+|[a-zA-Z]+);/g, (m, g: string) => {
        if (g[0] === '#') {
            const n = g[1] === 'x' || g[1] === 'X' ? parseInt(g.slice(2), 16) : parseInt(g.slice(1), 10);
            return Number.isFinite(n) && n > 0 && n <= 0x10ffff ? String.fromCodePoint(n) : '';
        }
        return ENTITIES[g] ?? m;
    });
}

function attrs(s: string): Record<string, string> {
    const o: Record<string, string> = {};
    for (const m of s.matchAll(/([\w:.-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g)) o[m[1]] = xmlText(m[2] ?? m[3] ?? '');
    return o;
}

/** The text of all <t> elements in a fragment (not the phonetic runs). */
function textOf(frag: string): string {
    const clean = frag.replace(/<rPh\b[\s\S]*?<\/rPh>/g, '');
    let out = '';
    for (const m of clean.matchAll(/<t\b[^>]*?(?:\/>|>([\s\S]*?)<\/t>)/g)) out += xmlText(m[1] ?? '');
    return out;
}

function parseSharedStrings(xml: string): string[] {
    const out: string[] = [];
    for (const m of xml.matchAll(/<si\b[^>]*?(?:\/>|>([\s\S]*?)<\/si>)/g)) out.push(textOf(m[1] ?? ''));
    return out;
}

function parseSheetXml(name: string, xml: string, shared: string[], budget: { cells: number }): SheetData {
    const cells = new Map<number, Cell>();
    let nrows = 0;
    let ncols = 0;
    let nextR = -1;
    for (const rowM of xml.matchAll(/<row\b([^>]*?)(?:\/>|>([\s\S]*?)<\/row>)/g)) {
        const ra = attrs(rowM[1]);
        let r = ra.r ? Number(ra.r) - 1 : nextR + 1;
        if (!Number.isInteger(r) || r < 0 || r >= MAX_ROWS) r = nextR + 1;
        nextR = r;
        const body = rowM[2];
        if (!body) continue;
        let nextC = -1;
        for (const cm of body.matchAll(/<c\b([^>]*?)(?:\/>|>([\s\S]*?)<\/c>)/g)) {
            const ca = attrs(cm[1]);
            let c = nextC + 1;
            if (ca.r) {
                const ref = parseRef(ca.r);
                if (ref) { c = ref.c; r = ref.r; }
            }
            nextC = c;
            if (c >= MAX_COLS) continue;
            const inner = cm[2] ?? '';
            const vM = /<v\b[^>]*?(?:\/>|>([\s\S]*?)<\/v>)/.exec(inner);
            const fM = /<f\b([^>]*?)(?:\/>|>([\s\S]*?)<\/f>)/.exec(inner);
            const hasF = !!fM;
            const f = hasF ? xmlText(fM![2] ?? '').trim() || (attrs(fM![1]).t === 'shared' ? '(shared formula)' : '(formula)') : undefined;
            const raw = vM ? xmlText(vM[1] ?? '') : '';
            let cell: Cell | null = null;
            const type = ca.t ?? 'n';
            if (type === 's') {
                const idx = Number(raw);
                if (raw !== '' && Number.isInteger(idx) && idx >= 0 && idx < shared.length) cell = { t: 's', v: shared[idx] };
            } else if (type === 'inlineStr') {
                const is = /<is\b[^>]*?>([\s\S]*?)<\/is>/.exec(inner);
                cell = { t: 's', v: is ? textOf(is[1]) : '' };
            } else if (type === 'str' || type === 'd') {
                if (vM) cell = { t: 's', v: raw };
            } else if (type === 'b') {
                if (vM) cell = { t: 'b', v: raw === '1' || raw.toLowerCase() === 'true' };
            } else if (type === 'e') {
                cell = { t: 'e', v: raw || '#ERROR' };
            } else if (vM && raw.trim() !== '') {
                const n = Number(raw);
                cell = Number.isFinite(n) ? { t: 'n', v: n } : { t: 's', v: raw };
            }
            if (!cell && hasF) cell = { t: 's', v: '' };
            if (!cell) continue;
            if (cell.t === 's' && cell.v === '' && !hasF) continue;
            if (f !== undefined) cell.f = f;
            if (++budget.cells > MAX_CELLS) throw new Error(`This workbook has more than ${MAX_CELLS.toLocaleString('en')} cells, which is too many to import. Save the part you need as .csv.`);
            cells.set(cellKey(r, c), cell);
            if (r + 1 > nrows) nrows = r + 1;
            if (c + 1 > ncols) ncols = c + 1;
        }
    }
    return { name, nrows, ncols, cells };
}

function relTarget(base: string, target: string): string {
    if (target.startsWith('/')) return target.slice(1);
    const parts = (base + target).split('/');
    const out: string[] = [];
    for (const p of parts) { if (p === '..') out.pop(); else if (p && p !== '.') out.push(p); }
    return out.join('/');
}

export async function parseXlsx(bytes: Uint8Array, fileName = 'workbook.xlsx'): Promise<Workbook> {
    const dir = readZipDirectory(bytes);
    if (dir.has('encryptedpackage')) throw new Error('This workbook is password-protected. Remove the password in the spreadsheet program and try again.');
    const wbXml = await readEntry(bytes, dir, 'xl/workbook.xml');
    if (wbXml == null) throw new Error('This zip file is not an Excel workbook (xl/workbook.xml is missing).');
    const relsXml = (await readEntry(bytes, dir, 'xl/_rels/workbook.xml.rels')) ?? '';
    const rels = new Map<string, string>();
    for (const m of relsXml.matchAll(/<Relationship\b([^>]*?)\/?>/g)) {
        const a = attrs(m[1]);
        if (a.Id && a.Target) rels.set(a.Id, relTarget('xl/', a.Target));
    }
    const ssXml = await readEntry(bytes, dir, 'xl/sharedStrings.xml');
    const shared = ssXml ? parseSharedStrings(ssXml) : [];
    const budget = { cells: 0 };
    const sheets: SheetData[] = [];
    let n = 0;
    for (const m of wbXml.matchAll(/<sheet\b([^>]*?)\/?>/g)) {
        const a = attrs(m[1]);
        n++;
        const name = a.name || `Sheet${n}`;
        const target = (a['r:id'] && rels.get(a['r:id'])) || `xl/worksheets/sheet${n}.xml`;
        const xml = await readEntry(bytes, dir, target);
        if (xml == null || !/<worksheet\b/.test(xml)) continue; // a chart sheet or a missing part
        sheets.push(parseSheetXml(name, xml, shared, budget));
    }
    if (!sheets.length) throw new Error('This workbook has no worksheets to show.');
    return { fileName, sheets };
}

// ---------------------------------------------------------------------------------------------------------------------
// CSV and TSV

/** The delimiter that splits the first lines into the same number of fields, more than one. */
export function detectDelimiter(text: string): string {
    const lines: string[] = [];
    let cur = '';
    let q = false;
    for (let i = 0; i < text.length && lines.length < 12; i++) {
        const ch = text[i];
        if (ch === '"') q = !q;
        if (!q && (ch === '\n' || ch === '\r')) {
            if (ch === '\r' && text[i + 1] === '\n') i++;
            if (cur.trim()) lines.push(cur);
            cur = '';
        } else cur += ch;
    }
    if (cur.trim() && lines.length < 12) lines.push(cur);
    let best = ',';
    let bestScore = -1;
    for (const d of [',', ';', '\t']) {
        const counts = lines.map((l) => splitFields(l, d).length);
        if (!counts.length) continue;
        const first = counts[0];
        if (first < 2) continue;
        const same = counts.filter((c) => c === first).length;
        const score = same * 1000 + first;
        if (score > bestScore) { bestScore = score; best = d; }
    }
    return best;
}

function splitFields(line: string, d: string): string[] {
    const out: string[] = [];
    let cur = '';
    let q = false;
    for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (q) {
            if (ch === '"') { if (line[i + 1] === '"') { cur += '"'; i++; } else q = false; } else cur += ch;
        } else if (ch === '"' && cur === '') q = true;
        else if (ch === d) { out.push(cur); cur = ''; } else cur += ch;
    }
    out.push(cur);
    return out;
}

/** A number written in a text cell: 12.5, -3, .5, 1e3, and 12,5 when the file's delimiter is not a comma. */
export function textNumber(s: string, decimalComma: boolean): number | null {
    const t = s.trim();
    if (!t) return null;
    const u = decimalComma && /^[-+]?\d+,\d+(e[-+]?\d+)?$/i.test(t) ? t.replace(',', '.') : t;
    if (!/^[-+]?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?$/i.test(u)) return null;
    const n = Number(u);
    return Number.isFinite(n) ? n : null;
}

export function parseCsv(text: string, fileName = 'data.csv', forceDelimiter?: string): Workbook {
    if (text.charCodeAt(0) === 0xfeff) text = text.slice(1);
    if (text.includes('\u0000')) throw new Error('This file is not text. A spreadsheet needs to be .xlsx, .csv or .tsv.');
    const d = forceDelimiter ?? (/\.tsv$/i.test(fileName) ? '\t' : detectDelimiter(text));
    const decimalComma = d !== ',';
    const cells = new Map<number, Cell>();
    let nrows = 0;
    let ncols = 0;
    let count = 0;
    let row: string[] = [];
    let field = '';
    let q = false;
    let r = 0;
    const flushRow = () => {
        row.push(field);
        field = '';
        let any = false;
        for (let c = 0; c < row.length && c < MAX_COLS; c++) {
            const raw = row[c];
            if (raw.trim() === '') continue;
            any = true;
            if (++count > MAX_CELLS) throw new Error(`This file has more than ${MAX_CELLS.toLocaleString('en')} cells, which is too many to import.`);
            const n = textNumber(raw, decimalComma);
            cells.set(cellKey(r, c), n != null ? { t: 'n', v: n } : { t: 's', v: raw });
            if (c + 1 > ncols) ncols = c + 1;
        }
        if (any && r + 1 > nrows) nrows = r + 1;
        r++;
        row = [];
    };
    for (let i = 0; i < text.length; i++) {
        const ch = text[i];
        if (q) {
            if (ch === '"') { if (text[i + 1] === '"') { field += '"'; i++; } else q = false; } else field += ch;
        } else if (ch === '"' && field === '') q = true;
        else if (ch === d) { row.push(field); field = ''; } else if (ch === '\n' || ch === '\r') {
            if (ch === '\r' && text[i + 1] === '\n') i++;
            flushRow();
        } else field += ch;
    }
    if (field !== '' || row.length) flushRow();
    if (q) throw new Error('This file has a quote that is never closed, so it is not a valid CSV.');
    if (!cells.size) throw new Error('This file has no data in it.');
    return { fileName, sheets: [{ name: fileName.replace(/\.[^.]+$/, '') || 'Data', nrows, ncols, cells }] };
}

// ---------------------------------------------------------------------------------------------------------------------

export const SPREADSHEET_EXT = /\.(xlsx|xlsm|csv|tsv)$/i;

/** Reads a file by its name. Refuses what cannot be read with a message that says what to do. */
export async function parseWorkbook(fileName: string, bytes: Uint8Array): Promise<Workbook> {
    if (bytes.length > MAX_BYTES) throw new Error(`This file is ${(bytes.length / 1048576).toFixed(1)} MB. Files over ${MAX_BYTES / 1048576} MB are not imported; save the part you need as .csv.`);
    if (!bytes.length) throw new Error('This file is empty.');
    const isZip = bytes.length > 3 && bytes[0] === 0x50 && bytes[1] === 0x4b;
    const isOle = bytes.length > 8 && bytes[0] === 0xd0 && bytes[1] === 0xcf && bytes[2] === 0x11 && bytes[3] === 0xe0;
    if (/\.xls$/i.test(fileName) || (isOle && !/\.(csv|tsv)$/i.test(fileName))) {
        throw new Error(isOle && /\.(xlsx|xlsm)$/i.test(fileName)
            ? 'This workbook is password-protected or in the old .xls format. Remove the password, or save it as .xlsx or .csv, and try again.'
            : 'The old .xls format is not read. Save the workbook as .xlsx or .csv and try again.');
    }
    if (/\.(xlsx|xlsm)$/i.test(fileName)) {
        if (!isZip) throw new Error('This file is not a valid .xlsx workbook (it is not a zip file). It may be damaged or have the wrong extension.');
        return parseXlsx(bytes, fileName);
    }
    if (/\.(csv|tsv)$/i.test(fileName)) {
        if (isZip) throw new Error('This file is a zip archive, not text. If it is a workbook, rename it to .xlsx.');
        return parseCsv(new TextDecoder('utf-8').decode(bytes), fileName);
    }
    throw new Error('Choose a .xlsx, .xlsm, .csv or .tsv file.');
}

/** A cell's value as the grid shows it: numbers to ten significant digits, booleans as TRUE/FALSE. */
export function displayValue(c: Cell | undefined): string {
    if (!c) return '';
    if (c.t === 'n') return String(Number((c.v as number).toPrecision(10)));
    if (c.t === 'b') return c.v ? 'TRUE' : 'FALSE';
    return String(c.v);
}
