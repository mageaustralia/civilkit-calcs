// calcpad-rs: from spreadsheet cells to sheet lines, for "Import spreadsheet". Guesses a quantity name and a unit from the
// cell's label (the text to its left, or the header above), makes the name unique in the sheet, and writes the lines an
// "Add as input", "Add as value", "Add as table" or "Add labelled rows" produces. Plain TypeScript; the only import is the
// cell model of the reader (with its .ts extension, so a Node check can load both).
import { a1, cellAt, displayValue, rangeA1, type Cell, type SheetData } from './sheet-import-parse.ts';

// ---------------------------------------------------------------------------------------------------------------------
// Names

const GREEK: Record<string, string> = {
    α: 'alpha', β: 'beta', γ: 'gamma', δ: 'delta', ε: 'epsilon', ζ: 'zeta', η: 'eta', θ: 'theta', ι: 'iota', κ: 'kappa',
    λ: 'lambda', μ: 'mu', ν: 'nu', ξ: 'xi', π: 'pi', ρ: 'rho', σ: 'sigma', ς: 'sigma', τ: 'tau', υ: 'upsilon', φ: 'phi',
    χ: 'chi', ψ: 'psi', ω: 'omega',
    Γ: 'Gamma', Δ: 'Delta', Θ: 'Theta', Λ: 'Lambda', Ξ: 'Xi', Π: 'Pi', Σ: 'Sigma', Φ: 'Phi', Ψ: 'Psi', Ω: 'Omega',
};

/** Words a name must not be: sheet keywords and common engine functions. */
const RESERVED = new Set([
    'if', 'else', 'for', 'in', 'step', 'and', 'or', 'not', 'plot', 'solve', 'integral', 'of', 'at', 'slope', 'import', 'on', 'off',
    'sin', 'cos', 'tan', 'exp', 'ln', 'log', 'abs', 'sqrt', 'min', 'max', 'sum', 'round', 'floor', 'ceil', 'pi', 'e', 'i', 'j',
    'true', 'false', 'mod', 'sign', 'min', 'avg', 'count', 'len', 'size', 'det', 'norm', 'unit', 'key',
]);

/** Text in any alphabet to the ASCII letters a sheet name is made of (Greek letters become their spellings). */
function asciiWords(label: string): string[] {
    let s = '';
    for (const ch of label) s += GREEK[ch] ? ` ${GREEK[ch]} ` : ch;
    s = s.normalize('NFKD').replace(/[̀-ͯ]/g, '');
    s = s.replace(/[²]/g, '2').replace(/[³]/g, '3').replace(/%/g, ' percent ');
    return s.split(/[^A-Za-z0-9]+/).filter(Boolean);
}

/** The label without the unit written in brackets at its end, and that unit (`Length (m)`, `Load [kN/m]`). */
export function splitLabelUnit(label: string): { base: string; unit: string } {
    const t = label.replace(/\s+/g, ' ').trim().replace(/[:=]\s*$/, '').trim();
    const m = /^(.*?)\s*[(\[]\s*([^()\[\]]+?)\s*[)\]]$/.exec(t);
    if (m && m[1].trim()) {
        const unit = cleanUnit(m[2]);
        if (unit) return { base: m[1].trim(), unit };
    }
    return { base: t, unit: '' };
}

const NOT_UNITS = new Set(['optional', 'note', 'notes', 'see', 'ref', 'assumed', 'typ', 'typical', 'approx', 'calc', 'input', 'output', 'total', 'each', 'text', 'name', 'value', 'values', 'unit', 'units', 'tbc', 'yes', 'no', 'n/a', 'na', 'none', 'info', 'comment', 'comments', 'description', 'type', 'min', 'max', 'avg', 'default', 'result', 'results']);
const UNIT_SUP: Record<string, string> = { '⁻': '-', '¹': '1', '²': '2', '³': '3', '⁰': '0', '⁴': '4', '⁵': '5', '⁶': '6', '⁷': '7', '⁸': '8', '⁹': '9' };

/** A unit as the sheet takes it, or '' when the text is not a unit (a word, a sentence, a number). */
export function cleanUnit(text: string): string {
    let u = text.trim().replace(/^\[|\]$/g, '').replace(/^\(|\)$/g, '').trim();
    if (!u || u.length > 18 || /\s/.test(u) && !/^[A-Za-z]+\s*[/·*]\s*[A-Za-z]+/.test(u)) return '';
    u = u.replace(/\s+/g, '').replace(/·/g, '*').replace(/μ/g, 'µ').replace(/[⁻¹²³⁰⁴⁵⁶⁷⁸⁹]+/g, (m) => '^' + [...m].map((c) => UNIT_SUP[c] ?? c).join(''));
    if (!/^[A-Za-zµΩ°%][A-Za-z0-9µΩ°%/*^()\-.]*$/.test(u)) return '';
    if (NOT_UNITS.has(u.toLowerCase())) return '';
    if (/^\d/.test(u)) return '';
    return u;
}

/** Turns a label into a sheet name: letters and digits, camel case for several words, never a keyword or a digit first. */
export function toSheetName(label: string, fallback = 'value'): string {
    const { base } = splitLabelUnit(label);
    const words = asciiWords(base);
    let name = '';
    words.forEach((w, i) => {
        // A Greek spelling stays as written, so the sheet draws the letter; other words are lower case then capitalised.
        const lower = w.toLowerCase();
        const keep = /^[A-Z][a-z]+$/.test(w) && Object.values(GREEK).includes(w);
        if (i === 0) name += keep ? w : (w.length <= 3 && w === w.toUpperCase() ? w : lower);
        else name += keep ? w : lower.charAt(0).toUpperCase() + lower.slice(1);
    });
    name = name.slice(0, 32);
    if (!name) name = fallback;
    if (/^\d/.test(name)) name = `n${name}`;
    if (RESERVED.has(name.toLowerCase())) name = `${name}_`;
    return name;
}

/** A name compared as the sheet does: Greek letters as their spellings, case kept. */
export function nameKey(n: string): string {
    let s = '';
    for (const ch of n) s += GREEK[ch] ?? ch;
    return s;
}

/** The names a sheet's text assigns (`name:`, `name =`, `f(x):` and several to a line with `;`). */
export function namesInSheet(text: string): Set<string> {
    const out = new Set<string>();
    for (const line of text.split(/\r?\n/)) {
        if (/^\s*(#|>>|\/\/|<!--|```|\\)/.test(line)) continue;
        for (const m of line.matchAll(/(?:^|;)\s*([\p{L}_][\p{L}\p{N}_']*)\s*(?:\([^)]*\))?\s*(?::(?!:)|=(?!=))/gu)) out.add(nameKey(m[1]));
    }
    return out;
}

/** The name, or the name with a number added so it is not in `used`. */
export function uniqueName(name: string, used: Set<string>): string {
    let n = name;
    let i = 2;
    while (used.has(nameKey(n)) || RESERVED.has(n.toLowerCase())) n = `${name}${i++}`;
    return n;
}

// ---------------------------------------------------------------------------------------------------------------------
// Labels and units around a cell

export interface Label { text: string; addr: string; from: 'left' | 'above' }

const isText = (c: Cell | undefined): c is Cell => !!c && c.t === 's' && String(c.v).trim() !== '';

const KNOWN_UNITS = new Set(['m', 'mm', 'cm', 'km', 'in', 'ft', 'N', 'kN', 'MN', 'Pa', 'kPa', 'MPa', 'GPa', 'kg', 'g', 't', 's', 'min', 'h', 'Hz', 'rad', 'deg', 'W', 'kW', 'MW', 'J', 'kJ', 'V', 'A', 'K', 'L', 'psi', 'ksi', 'lbf', 'kip', 'lb', 'mol', 'm2', 'm3', 'mm2', 'mm3', 'mm4', 'm4']);

/** Text in a cell that is a unit and not a word: a known unit, or one with a sign such as / * ^ ° % in it. */
export function unitFromCell(text: string): string {
    const u = cleanUnit(text);
    if (!u) return '';
    return KNOWN_UNITS.has(u) || /[/*^°%]/.test(u) ? u : '';
}

/** The label of a value cell: the nearest text to its left in the row (stopping at a number), else the header above it. */
export function guessLabel(s: SheetData, r: number, c: number): Label | null {
    for (let cc = c - 1; cc >= 0 && cc >= c - 3; cc--) {
        const x = cellAt(s, r, cc);
        if (!x) continue;
        if (!isText(x) || x.f !== undefined) break;
        const text = String(x.v).trim();
        // A unit column between the label and the value: the label is the cell before it.
        const before = cellAt(s, r, cc - 1);
        if (cc === c - 1 && unitFromCell(text) && isText(before) && before.f === undefined) return { text: String(before.v).trim(), addr: a1(r, cc - 1), from: 'left' };
        return { text, addr: a1(r, cc), from: 'left' };
    }
    for (let rr = r - 1; rr >= 0 && rr >= r - 60; rr--) {
        const x = cellAt(s, rr, c);
        if (!x) continue;
        if (isText(x) && x.f === undefined) return { text: String(x.v).trim(), addr: a1(rr, c), from: 'above' };
    }
    return null;
}

/** The unit of a value cell: from its label's brackets, else a unit in the cell to its right (or to its left of the value). */
export function guessUnit(s: SheetData, r: number, c: number, label: Label | null): string {
    if (label) {
        const u = splitLabelUnit(label.text).unit;
        if (u) return u;
    }
    const right = cellAt(s, r, c + 1);
    if (isText(right) && right.f === undefined) {
        const u = unitFromCell(String(right.v));
        if (u) return u;
    }
    const left = cellAt(s, r, c - 1);
    if (isText(left) && left.f === undefined) {
        const u = unitFromCell(String(left.v));
        if (u) return u;
    }
    return '';
}

export interface Proposal { name: string; unit: string; label: string }

/** The name and unit proposed for a value cell; the name is made unique among `used`. */
export function proposeForCell(s: SheetData, r: number, c: number, used: Set<string>): Proposal {
    const label = guessLabel(s, r, c);
    const unit = guessUnit(s, r, c, label);
    const name = uniqueName(toSheetName(label?.text ?? '', `cell${a1(r, c)}`), used);
    return { name, unit, label: label ? splitLabelUnit(label.text).base : '' };
}

/** The name and unit proposed for a range: from the text left of its first row, else above its first column. */
export function proposeForRange(s: SheetData, r0: number, c0: number, used: Set<string>): Proposal {
    const label = guessLabel(s, r0, c0);
    const name = uniqueName(toSheetName(label?.text ?? '', 'table'), used);
    return { name, unit: label ? splitLabelUnit(label.text).unit : '', label: label ? splitLabelUnit(label.text).base : '' };
}

// ---------------------------------------------------------------------------------------------------------------------
// Lines

/** A number written the way a sheet takes it (no exponent), up to 15 significant digits. */
export function numStr(v: number): string {
    if (!Number.isFinite(v)) return '0';
    if (v === 0) return '0';
    let s = String(Number(v.toPrecision(15)));
    if (/e/i.test(s)) {
        const neg = s.startsWith('-');
        const [mant, expS] = (neg ? s.slice(1) : s).toLowerCase().split('e');
        const exp = Number(expS);
        const [ip, fp = ''] = mant.split('.');
        const digits = ip + fp;
        const point = ip.length + exp;
        if (point <= 0) s = `0.${'0'.repeat(-point)}${digits}`;
        else if (point >= digits.length) s = digits + '0'.repeat(point - digits.length);
        else s = `${digits.slice(0, point)}.${digits.slice(point)}`;
        s = (neg ? '-' : '') + s;
    }
    return s;
}

/** The number a cell stands for in a calculation: a number, or a boolean as 1 or 0; null for text, errors and empty cells. */
export function cellNumber(c: Cell | undefined): number | null {
    if (!c) return null;
    if (c.t === 'n') return c.v as number;
    if (c.t === 'b') return c.v ? 1 : 0;
    return null;
}

const withUnit = (n: string, unit: string): string => (unit ? `${n} ${unit}` : n);

/** A note after `|`: one line, without the characters a sheet reads as markup. */
export function noteText(s: string): string {
    return s.replace(/\s+/g, ' ').replace(/[|{}<>`]/g, '').replace(/->/g, '').trim().slice(0, 60);
}

const tail = (note: string | undefined): string => {
    const n = note ? noteText(note) : '';
    return n ? ` | ${n}` : '';
};

export function inputLine(name: string, value: number, unit: string, note?: string): string {
    return `${name}: ? ${withUnit(numStr(value), unit)}${tail(note)}`;
}

export function valueLine(name: string, value: number, unit: string, note?: string): string {
    return `${name}: ${withUnit(numStr(value), unit)}${tail(note)}`;
}

/** The quiet source line above imported lines: an HTML comment, which prose renders as nothing. */
export function provenance(fileName: string, sheetName: string, range: string): string {
    const sheet = /^[A-Za-z0-9_]+$/.test(sheetName) ? sheetName : `'${sheetName.replace(/'/g, "''")}'`;
    const text = `${fileName}, ${sheet}!${range}`.replace(/[\u0000-\u001f]/g, ' ').replace(/--+/g, '-').replace(/[<>]/g, '');
    return `<!-- ${text} -->`;
}

export interface Rect { r0: number; c0: number; r1: number; c1: number }

export const normRect = (a: { r: number; c: number }, b: { r: number; c: number }): Rect =>
    ({ r0: Math.min(a.r, b.r), c0: Math.min(a.c, b.c), r1: Math.max(a.r, b.r), c1: Math.max(a.c, b.c) });

export const MAX_TABLE_CELLS = 5000;

export interface Built {
    /** The lines to insert, without a trailing newline. */
    text: string;
    /** For marking the grid: the cells used and the name shown on each. */
    marks: { r: number; c: number; name: string }[];
    /** A short account for the status line. */
    summary: string;
    /** Rows left out of a labelled-rows import (no number in the value column). */
    skipped?: number;
}

/** A single cell as an input (`name: ? 6 m`); only a constant number is an input. */
export function buildInput(s: SheetData, file: string, r: number, c: number, name: string, unit: string, note?: string): Built {
    const cell = cellAt(s, r, c);
    const v = cellNumber(cell);
    if (!cell || v == null) throw new Error('Only a number can be an input.');
    if (cell.f !== undefined) throw new Error('This cell is a formula, so it is a result and not an input. Use "Add as value".');
    return { text: `${provenance(file, s.name, a1(r, c))}\n${inputLine(name, v, unit, note)}`, marks: [{ r, c, name }], summary: `Added input ${name}` };
}

/** A single cell as a value (`name: 12.5 kN`); a formula cell gives the value stored in the file. */
export function buildValue(s: SheetData, file: string, r: number, c: number, name: string, unit: string, note?: string): Built {
    const v = cellNumber(cellAt(s, r, c));
    if (v == null) throw new Error('Only a number can be added as a value.');
    return { text: `${provenance(file, s.name, a1(r, c))}\n${valueLine(name, v, unit, note)}`, marks: [{ r, c, name }], summary: `Added value ${name}` };
}

const mdCell = (c: Cell | undefined): string => displayValue(c).replace(/\|/g, '/').replace(/[\r\n]+/g, ' ').replace(/[`<>*_\\$]/g, (m) => '\\' + m).trim();

/**
 * A range as a table. All numbers: a vector (one row or column) or a matrix, named and with the unit on every element.
 * A header row of text over numbers is left out of the matrix and listed in the comment line. Anything else: a pipe table
 * of the cells as shown (text for the report, not for calculation).
 */
export function buildTable(s: SheetData, file: string, rect: Rect, name: string, unit: string): Built {
    const { r0, c0, r1, c1 } = rect;
    const rows = r1 - r0 + 1;
    const cols = c1 - c0 + 1;
    if (rows * cols > MAX_TABLE_CELLS) throw new Error(`The range has ${rows * cols} cells; a table is limited to ${MAX_TABLE_CELLS}. Select a smaller range.`);
    const range = rangeA1(r0, c0, r1, c1);
    const allNum = (ra: number, rb: number) => {
        let any = false;
        for (let r = ra; r <= rb; r++) for (let c = c0; c <= c1; c++) {
            const x = cellAt(s, r, c);
            if (!x) continue; // an empty cell reads as 0
            if (cellNumber(x) == null) return false;
            any = true;
        }
        return any;
    };
    const marks: Built['marks'] = [];
    const markAll = (ra: number) => { for (let r = ra; r <= r1; r++) for (let c = c0; c <= c1; c++) if (cellAt(s, r, c)) marks.push({ r, c, name }); };
    let bodyStart = r0;
    let headers = '';
    if (rows > 1 && !allNum(r0, r1)) {
        const headerRow = Array.from({ length: cols }, (_, i) => cellAt(s, r0, c0 + i));
        if (headerRow.every((x) => !x || x.t === 's') && allNum(r0 + 1, r1)) {
            bodyStart = r0 + 1;
            headers = headerRow.map((x) => (x ? String(x.v).trim() : '')).join(', ');
        }
    }
    if (allNum(bodyStart, r1)) {
        const num = (r: number, c: number) => withUnit(numStr(cellNumber(cellAt(s, r, c)) ?? 0), unit);
        const body: string[][] = [];
        for (let r = bodyStart; r <= r1; r++) body.push(Array.from({ length: cols }, (_, i) => num(r, c0 + i)));
        const lit = body.length === 1 ? `[${body[0].join(', ')}]` : cols === 1 ? `[${body.map((x) => x[0]).join('; ')}]` : `[${body.map((x) => x.join(', ')).join('; ')}]`;
        const src = headers ? `${range} (columns: ${headers})` : range;
        markAll(bodyStart);
        const kind = body.length === 1 || cols === 1 ? 'vector' : 'matrix';
        return { text: `${provenance(file, s.name, src)}\n${name}: ${lit}`, marks, summary: `Added ${kind} ${name} (${body.length} x ${cols})` };
    }
    // Text or mixed: a pipe table. The first row is its header.
    const out: string[] = [];
    for (let r = r0; r <= r1; r++) {
        out.push(`| ${Array.from({ length: cols }, (_, i) => mdCell(cellAt(s, r, c0 + i))).join(' | ')} |`);
        if (r === r0) out.push(`| ${Array.from({ length: cols }, () => '---').join(' | ')} |`);
    }
    markAll(r0);
    return { text: `${provenance(file, s.name, range)}\n\n${out.join('\n')}\n`, marks, summary: `Added a table of text (${rows} x ${cols}), shown in the report but not used in calculations` };
}

/**
 * A two-column range (label, value; a third column may be the unit) as one line per row. Rows whose value is not a number
 * are left out and counted. Names come from the labels and are unique in the sheet and among themselves.
 */
export function buildLabelled(s: SheetData, file: string, rect: Rect, used: Set<string>, asInput: boolean): Built {
    const { r0, c0, r1, c1 } = rect;
    if (c1 - c0 < 1) throw new Error('Select at least two columns: the labels, then the values.');
    const lines: string[] = [];
    const marks: Built['marks'] = [];
    let skipped = 0;
    const taken = new Set(used);
    for (let r = r0; r <= r1; r++) {
        const lab = cellAt(s, r, c0);
        const val = cellAt(s, r, c0 + 1);
        const v = cellNumber(val);
        if (!isText(lab) || v == null) { if (lab || val) skipped++; continue; }
        const { base, unit: labUnit } = splitLabelUnit(String(lab.v));
        let unit = labUnit;
        if (!unit && c1 >= c0 + 2) unit = unitFromCell(displayValue(cellAt(s, r, c0 + 2)));
        if (!unit && c1 === c0 + 1) {
            const right = cellAt(s, r, c0 + 2);
            if (isText(right) && right.f === undefined) unit = unitFromCell(String(right.v));
        }
        const name = uniqueName(toSheetName(String(lab.v), `row${r + 1}`), taken);
        taken.add(nameKey(name));
        const asIn = asInput && val!.f === undefined;
        lines.push(asIn ? inputLine(name, v, unit, base) : valueLine(name, v, unit, base));
        marks.push({ r, c: c0 + 1, name });
    }
    if (!lines.length) throw new Error('No row has a label and a number. Select the label column and the value column.');
    return { text: `${provenance(file, s.name, rangeA1(r0, c0, r1, c1))}\n${lines.join('\n')}`, marks, summary: `Added ${lines.length} line${lines.length === 1 ? '' : 's'}${skipped ? `, left out ${skipped} without a number` : ''}`, skipped };
}
