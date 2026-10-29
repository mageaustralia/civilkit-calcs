// What kind of place the cursor is in, for completion: code (names, functions, units), a line
// start (keywords and markers), or somewhere that is not code at all (a comment or text line, a
// string, a note, a Markdown or HTML block, a Python block, sheet prose). Nothing is suggested
// in the last group. The checks are on the text, so they work before the engine has seen a
// keystroke; for a sheet the compiler's line kinds back them up.

import type { SheetCompiled } from '../../wasm/sheet';

export type Syntax = 'calcpad' | 'sheet';

export type CompletionContext =
    | { kind: 'none' }
    /** A bare word at the start of a line: a block keyword or a format marker in a sheet. */
    | { kind: 'marker'; start: number; text: string }
    | {
        kind: 'code';
        syntax: Syntax;
        /** After a number (or in a unit chain, or after `->`): only units. */
        unit: boolean;
        /** Where the word being typed starts (0 based column) and the word. */
        start: number;
        word: string;
        /** The word is the first thing on its line. */
        lineStart: boolean;
        /** A Calcpad `#` directive or `$` command is being typed. */
        directive: boolean;
    };

const WORD = /[#$]?[\p{L}_°µΩ%][\p{L}\p{N}_′″°µΩ]*$|[#$]$/u;
const UNIT_WORD = '[\\p{L}\\u00b5\\u03a9\\u00b0%_][\\p{L}\\p{N}_\\u00b0\\u00b5\\u03a9]*';
// A number, then a unit chain written without spaces, ending in an operator right before the word: `5 kN*`, `3m/`.
const UNIT_CHAIN = new RegExp(`(?:^|[^\\p{L}\\p{N}_.])\\d[\\d.]*(?:[eE][+-]?\\d+)?\\s*${UNIT_WORD}(?:\\^-?\\d+)?(?:[*/\\u00b7]${UNIT_WORD}(?:\\^-?\\d+)?)*[*/\\u00b7]$`, 'u');
const AFTER_NUMBER = /(?:^|[^\p{L}\p{N}_.′″])\d[\d.]*(?:[eE][+-]?\d+)?\s*$/u;

function quotesOpen(s: string): boolean {
    let open = false;
    for (const c of s) if (c === '"') open = !open;
    return open;
}

function stripStrings(s: string): string {
    return s.replace(/"[^"]*"?/g, '""');
}

/** The text looks like part of an expression rather than a sentence. */
export function looksLikeExpression(rhs: string): boolean {
    const t = stripStrings(rhs).trim();
    if (!t) return true;
    // Two words in a row (outside a number-and-unit) read as prose: "see the".
    return !/(?<![\d.])[\p{L}_][\p{L}\p{N}_]*\s+(?!and\b|or\b|in\b|step\b|if\b|else\b|for\b)[\p{L}_][\p{L}\p{N}_]{1,}/u.test(t.replace(/\d[\d.]*\s*[\p{L}µΩ°%]+/gu, '0'));
}

function unitContext(codeBefore: string): boolean {
    if (UNIT_CHAIN.test(codeBefore)) return true;
    if (AFTER_NUMBER.test(codeBefore)) return true;
    // A conversion target after `->` (sheet) or `|` outside brackets (Calcpad).
    if (/->\s*$/.test(codeBefore)) return true;
    if (/->[^|]*[*/·^]$/.test(codeBefore)) return true;
    return false;
}

/** Calcpad: the line is inside a Markdown, HTML or Python block opened above it. */
export function calcpadBlockAt(lines: string[], lineIdx: number): 'md' | 'html' | 'python' | null {
    let block: 'md' | 'html' | 'python' | null = null;
    for (let i = 0; i < lineIdx; i++) {
        const t = lines[i];
        if (block === null) {
            if (/^\s*#markdown\b/i.test(t)) block = 'md';
            else if (/^\s*#html\b/i.test(t)) block = 'html';
            else if (/^\s*#python\s*$/i.test(t)) block = 'python';
        } else if (
            (block === 'md' && /^\s*#end\s+markdown\b/i.test(t))
            || (block === 'html' && /^\s*#end\s+html\b/i.test(t))
            || (block === 'python' && /^\s*#end\s+python\b/i.test(t))
        ) {
            block = null;
        }
    }
    return block;
}

function calcpadContext(lines: string[], lineIdx: number, col: number): CompletionContext {
    const text = lines[lineIdx];
    const before = text.slice(0, col);
    const inBlock = calcpadBlockAt(lines, lineIdx);
    if (inBlock) return { kind: 'none' };
    const t = before.trimStart();
    // A text line (`'` or `"`) and anything after a quote mark on a line is text.
    if (/^['"]/.test(t)) return { kind: 'none' };
    if (/['"]/.test(before)) return { kind: 'none' };
    const wm = WORD.exec(before);
    const word = wm?.[0] ?? '';
    const start = col - word.length;
    const head = before.slice(0, start);
    const directive = /^[#$]/.test(word);
    // A `#` directive is only a directive as the first word of a line.
    if (word.startsWith('#') && head.trim() !== '') return { kind: 'none' };
    return {
        kind: 'code', syntax: 'calcpad', start, word,
        unit: !directive && unitContext(head),
        lineStart: head.trim() === '', directive,
    };
}

const BLOCK_KEYWORD = /^\s*(?:else\s+if|if|while|for|repeat|plot)\s/;
const MARKER_LINE = /^\s*(?:>{1,2}|<!?-{0,2}[\w-]*>?|`{1,3}[\w-]*|\${1,2})$/;
// `name:`, `name = `, `f(x):` at the start of a line.
const HEAD = /^(\s*)([\p{L}][\p{L}\p{N}_′″₀-₉]*)(\([^)]*\))?\s*(:|=(?!=))/u;

function sheetContext(lines: string[], lineIdx: number, col: number, c: SheetCompiled | null): CompletionContext {
    const text = lines[lineIdx];
    const before = text.slice(0, col);
    // Fences and display formulas opened above.
    let fence: string | null = null;
    let math = false;
    for (let i = 0; i < lineIdx; i++) {
        const f = /^\s*```\s*([\w-]*)\s*$/.exec(lines[i]);
        if (fence !== null) { if (f && f[1] === '') fence = null; continue; }
        if (f) { fence = f[1].toLowerCase(); continue; }
        if (/^\s*\$\$\s*$/.test(lines[i])) math = !math;
    }
    const lead = /^\s*/.exec(before)![0].length;
    if (MARKER_LINE.test(before) && !(fence !== null && !/^\s*`/.test(before))) {
        return { kind: 'marker', start: lead, text: before.slice(lead) };
    }
    if (math) return { kind: 'none' };
    if (fence !== null) {
        if (fence !== 'calcpad') return { kind: 'none' };
        return calcpadContext([...lines.slice(0, lineIdx), text], lineIdx, col);
    }
    const kind = c?.kinds[lineIdx] ?? 'p';
    let from = 0;
    let rhs = before;
    let isCode = false;
    let raw = false;
    const rawM = /^\s*>>\s?/.exec(before);
    if (rawM) {
        isCode = true;
        raw = true;
        from = rawM[0].length;
        rhs = before.slice(from);
    } else if (BLOCK_KEYWORD.test(before)) {
        isCode = true;
        const km = BLOCK_KEYWORD.exec(before)![0];
        from = km.length;
        rhs = before.slice(from);
    } else {
        const h = HEAD.exec(before);
        if (h) {
            from = h[0].length;
            rhs = before.slice(from);
            if (/^\s*\?/.test(rhs)) return { kind: 'none' };
            isCode = 'qkcgr'.includes(kind) || looksLikeExpression(rhs);
        }
    }
    if (raw) return calcpadContext([...lines.slice(0, lineIdx), text.slice(0, col)], lineIdx, col);
    if (!isCode) {
        // A bare word at the start of a line: a block keyword or marker, not a name.
        if (/^\s*[\p{L}]*$/u.test(before) && before.trim() !== '') {
            return { kind: 'marker', start: lead, text: before.slice(lead) };
        }
        return { kind: 'none' };
    }
    if (quotesOpen(rhs)) return { kind: 'none' };
    const bare = stripStrings(rhs);
    // The note of a quantity line is prose.
    if (kind !== 'k' && !BLOCK_KEYWORD.test(before) && /(?<!\|)\|(?!\|)/.test(bare)) return { kind: 'none' };
    const wm = /[\p{L}_°µΩ%][\p{L}\p{N}_′″°µΩ]*$/u.exec(before);
    const word = wm && before.length - wm[0].length >= from ? wm[0] : '';
    const start = col - word.length;
    const head = before.slice(from, start);
    return {
        kind: 'code', syntax: 'sheet', start, word,
        unit: unitContext(head),
        lineStart: false, directive: false,
    };
}

/** The completion context at a 0 based position. */
export function completionContext(lines: string[], lineIdx: number, col: number, syntax: Syntax, compiled: SheetCompiled | null): CompletionContext {
    return syntax === 'sheet' ? sheetContext(lines, lineIdx, col, compiled) : calcpadContext(lines, lineIdx, col);
}
