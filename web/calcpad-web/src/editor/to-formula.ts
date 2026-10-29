// Convert a selection to a formula: the line arithmetic, with no editor in it (checked in Node by
// e2e/to-formula-check.mjs; the editor wiring is to-formula-monaco.ts).
//
// The selection is an expression written somewhere in the document, such as `w*L^2/8`:
//  - in text (sheet prose, a Calcpad text line or the text part of a line), it becomes a line of its
//    own after the paragraph: `res1: w*L^2/8` in a sheet, the plain calculation `w*L^2/8` in a Calcpad
//    worksheet. When the selection is the whole line, the line itself becomes the formula.
//  - in a calculation, it is pulled out to a named line above (`res1: w*L^2/8`, `res1 = w*L^2/8`)
//    and the selection is replaced by the name. Both copies of the name are left selected, so typing
//    a better name renames the two at once.
// Text written as `M = w*L^2/8` (or `M: ...` in a sheet) keeps its own name.

/** Where the selection sits: in text, in a calculation, or somewhere it does not apply (a fence, a directive). */
export type FormulaPlace = 'text' | 'code' | 'none';

/** Lines `from`..`to` (0 based, inclusive) are replaced by `text`; `sels` are in the new text, 0 based columns. */
export interface FormulaEdit {
    from: number;
    to: number;
    text: string[];
    sels: { line: number; c1: number; c2: number }[];
}

const NAME = /^[\p{L}_][\p{L}\p{N}_′″]*$/u;

/** The first `res1`, `res2`, ... that is not a word of the document. */
export function freshName(lines: string[], base = 'res'): string {
    const used = new Set<string>();
    for (const l of lines) for (const m of l.matchAll(/[\p{L}_][\p{L}\p{N}_′″]*/gu)) used.add(m[0]);
    let n = 1;
    while (used.has(base + n)) n++;
    return base + n;
}

/**
 * Could be an expression: a number or an operator, brackets that close, and not words in a row (`the moment`)
 * or marks no expression has. A number with a unit (`6 m`) is fine.
 */
export function looksLikeExpr(s: string): boolean {
    if (!/[\p{N}+\-*/^()√·×÷%<>=≤≥]/u.test(s) || /[#?@&{}\\]/.test(s)) return false;
    if (/(?<![\p{L}\p{N}_.])\p{L}[\p{L}\p{N}_′]*\s+\p{L}/u.test(s)) return false;
    return balanced(s);
}

function balanced(s: string): boolean {
    const open = '([{', close = ')]}';
    const st: string[] = [];
    for (const ch of s) {
        if (open.includes(ch)) st.push(close[open.indexOf(ch)]);
        else if (close.includes(ch) && st.pop() !== ch) return false;
    }
    return st.length === 0;
}

/**
 * The expression in a selection of text: inline-formula `$`s and code backticks around it, and a trailing
 * `=` or full stop, are dropped. `M = expr` (or `M: expr` in a sheet) gives the name too. Null when it is
 * not one line of something that could be an expression.
 */
export function selectedExpr(text: string, sheet: boolean): { name: string | null; expr: string } | null {
    let t = text.trim().replace(/[.,;]$/, '');
    if (!t || t.includes('\n')) return null;
    for (const q of ['$$', '$', '`']) {
        if (t.length > 2 * q.length && t.startsWith(q) && t.endsWith(q)) { t = t.slice(q.length, -q.length).trim(); break; }
    }
    t = t.replace(/\s*=\s*$/, '').replace(/[.,;]$/, '').trim();
    let name: string | null = null;
    const m = (sheet ? /^([^\s:=]+)\s*(?::|=(?!=))\s*(.+)$/ : /^([^\s=]+)\s*=(?!=)\s*(.+)$/).exec(t);
    if (m && NAME.test(m[1])) { name = m[1]; t = m[2].trim(); }
    // Something to calculate.
    if (!t || !/[\p{L}\p{N}]/u.test(t) || !looksLikeExpr(t)) return null;
    return { name, expr: t };
}

/** In a Calcpad line, whether column `c` is inside text: after an odd number of `'` or `"` marks. */
export function cpdInText(line: string, c: number): boolean {
    let inside = false;
    let q = '';
    for (let i = 0; i < c && i < line.length; i++) {
        const ch = line[i];
        if (!inside && (ch === "'" || ch === '"')) { inside = true; q = ch; } else if (inside && ch === q) inside = false;
    }
    return inside;
}

/** Where a selection starting at `c` on `line` of a Calcpad worksheet sits. */
export function cpdPlace(line: string, c: number): FormulaPlace {
    const t = line.trim();
    if (cpdInText(line, c)) return 'text';
    // The directives that hold an expression can have one pulled out above them; the others cannot.
    if (t.startsWith('#')) return /^#(if|else\s+if|while|repeat|for)\b/i.test(t) ? 'code' : 'none';
    return 'code';
}

/**
 * Where a selection starting at `c` on a sheet line sits, from the compiler's kind of the line (`kinds` in
 * sheet.ts) and whether it is prose: a note after `|` and a quoted text count as text.
 */
export function sheetPlace(line: string, kind: string, prose: boolean, c: number): FormulaPlace {
    if (prose) return 'text';
    if (!kind || !'qckg'.includes(kind)) return 'none';
    const before = line.slice(0, c).replace(/\|\|/g, '');
    if (before.includes('|') || (before.match(/"/g)?.length ?? 0) % 2 === 1) return 'text';
    return 'code';
}

/**
 * The edit for a selection of one line (`line`, columns `c1`..`c2`, 0 based). `place(l)` says where line
 * `l` sits, the selection starting at column `c` (`sheetPlace`, `cpdPlace`); `sheet` picks the sheet form of the new line.
 */
export function toFormula(
    lines: string[], sel: { line: number; c1: number; c2: number }, place: (l: number, c: number) => FormulaPlace, sheet: boolean,
): FormulaEdit | null {
    const line = lines[sel.line];
    if (line === undefined || sel.c2 <= sel.c1) return null;
    const where = place(sel.line, sel.c1);
    if (where === 'none') return null;
    const raw = line.slice(sel.c1, sel.c2);
    const indent = /^\s*/.exec(line)![0];
    if (where === 'text') {
        const e = selectedExpr(raw, sheet);
        if (!e) return null;
        const name = e.name ?? (sheet ? freshName(lines) : null);
        const body = name ? (sheet ? `${name}: ${e.expr}` : `${name} = ${e.expr}`) : e.expr;
        const newLine = indent + body;
        // The selection on the new line: the name, so a better one can be typed, or the whole calculation.
        const sel0 = name && !e.name ? { c1: indent.length, c2: indent.length + name.length } : { c1: indent.length, c2: newLine.length };
        // The whole text of the line (past a heading, list or text mark): the line becomes the formula.
        const content = line.trim().replace(sheet ? /^(#{1,6}\s+|[-*+]\s+|\d+[.)]\s+|>\s*)/ : /^['"]/, '').replace(/['"]$/, '').trim();
        if (content === raw.trim()) return { from: sel.line, to: sel.line, text: [newLine], sels: [{ line: sel.line, ...sel0 }] };
        // Otherwise after the paragraph: a sheet's run of text lines (a heading is one on its own); a Calcpad text line is one line.
        let end = sel.line;
        if (sheet && !/^\s*#/.test(line)) {
            while (end + 1 < lines.length && lines[end + 1].trim() !== '' && place(end + 1, 0) === 'text' && !/^\s*#/.test(lines[end + 1])) end++;
        }
        return { from: end, to: end, text: [lines[end], newLine], sels: [{ line: end + 1, ...sel0 }] };
    }
    // In a calculation: pulled out to a line above, the selection becomes the name.
    const expr = raw.trim();
    if (!expr || !/[\p{L}\p{N}]/u.test(expr) || !looksLikeExpr(expr)) return null;
    // Not across a note, a unit conversion or a text part, and not the name being defined or an input's default.
    if (/\|(?!\|)|->|['"]/.test(expr.replace(/\|\|/g, ''))) return null;
    const def = sheet ? /^\s*[^\s:=]+(?:\([^)]*\))?\s*(?::|=(?!=))/.exec(line) : /^\s*[^\s=]+(?:\([^)]*\))?\s*=(?!=)/.exec(line);
    if (def && sel.c1 < def[0].length) return null;
    if (def && /^\s*\?/.test(line.slice(def[0].length))) return null;
    if (def && line.slice(def[0].length).trim() === expr) return null;
    const name = freshName(lines);
    const lead = raw.length - raw.trimStart().length;
    const a = sel.c1 + lead;
    const b = a + expr.length;
    const above = indent + (sheet ? `${name}: ${expr}` : `${name} = ${expr}`);
    const changed = line.slice(0, a) + name + line.slice(b);
    return {
        from: sel.line, to: sel.line, text: [above, changed],
        sels: [{ line: sel.line, c1: indent.length, c2: indent.length + name.length }, { line: sel.line + 1, c1: a, c2: a + name.length }],
    };
}
