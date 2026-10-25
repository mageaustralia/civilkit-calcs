// Prose written as Markdown in a worksheet, and how a rich-text editor reads and writes it.
//
// Two forms exist:
//   - a `#markdown` ... `#end markdown` block: the lines between are one raw Markdown document (pipe
//     tables, lists, `f~c~` subscripts), the readable form that Tidy / Markdown imports write;
//   - an older `#md on` ... `#md off` run of `'` comment lines, where every line is a document of its
//     own (so a paragraph is always one line and there can be no tables).
// The helpers find the block around a source line and map it to one Markdown document and back.

export interface MdBlock {
    kind: 'block' | 'comment';
    /** 1-based line numbers of the opening and closing marker lines. */
    on: number;
    off: number;
    /** The lines between them: raw for a block, without the leading apostrophe for comment lines. */
    body: string[];
}

/** The Markdown block that holds `line` (1-based), or null if it is not inside one. */
export function findMdBlock(lines: string[], line: number): MdBlock | null {
    if (line < 1 || line > lines.length) return null;
    return findRaw(lines, line) ?? findComments(lines, line);
}

function findRaw(lines: string[], line: number): MdBlock | null {
    let on = -1;
    for (let i = line - 1; i >= 0; i--) {
        const t = lines[i].trim();
        if (/^#end\s+markdown\b/i.test(t)) return null;
        if (/^#markdown\b/i.test(t)) { on = i; break; }
    }
    if (on < 0) return null;
    for (let i = on + 1; i < lines.length; i++) {
        const t = lines[i].trim();
        if (/^#end\s+markdown\b/i.test(t)) {
            if (i < line - 1 && i + 1 < line) return null;
            return { kind: 'block', on: on + 1, off: i + 1, body: lines.slice(on + 1, i) };
        }
        if (/^#markdown\b/i.test(t)) return null;
    }
    return null;
}

function findComments(lines: string[], line: number): MdBlock | null {
    let on = -1;
    for (let i = line - 1; i >= 0; i--) {
        const t = lines[i].trim();
        if (t === '#md off') return null;
        if (/^#md on\b/.test(t)) { on = i; break; }
    }
    if (on < 0) return null;
    let off = -1;
    for (let i = on + 1; i < lines.length; i++) {
        const t = lines[i].trim();
        if (t === '#md off') { off = i; break; }
        if (/^#md on\b/.test(t)) return null;
    }
    if (off < 0 || off < line - 1) return null;
    const body: string[] = [];
    for (let i = on + 1; i < off; i++) {
        // Anything but a plain comment line (a directive, an equation) is not prose this editor can own.
        if (!lines[i].startsWith("'")) return null;
        body.push(lines[i].slice(1));
    }
    if (body.length === 0) return null;
    return { kind: 'comment', on: on + 1, off: off + 1, body };
}

// The engine writes subscript as `~x~`, superscript as `^x^` and underline as `++x++`; the editor
// knows them as the HTML tags (a single `~` would be strikethrough to it). These are the two ways.
const SUB = /(?<![\\~])~([^\s~]+)~(?!~)/g;
const SUP = /(?<![\\^])\^([^\s^]+)\^(?!\^)/g;
const INS = /(?<![\\+])\+\+([^\s+](?:[^+]*[^\s+])?)\+\+(?!\+)/g;

export function toEditor(md: string): string {
    return md.replace(SUB, '<sub>$1</sub>').replace(SUP, '<sup>$1</sup>').replace(INS, '<ins>$1</ins>');
}

export function fromEditor(md: string): string {
    return md
        // The editor marks every column left-aligned (`:-`); plain `---` says the same and reads better.
        .replace(/^\s*\|(?:\s*:?-+:?\s*\|)+\s*$/gm, row => row.replace(/\|(\s*):-+(\s*)(?=\|)/g, (_m, a, b) => `|${a}---${b}`))
        .replace(/<sub>([^\s<]+)<\/sub>/g, '~$1~')
        .replace(/<sup>([^\s<]+)<\/sup>/g, '^$1^')
        .replace(/<ins>([^<]+)<\/ins>/g, '++$1++');
}

/** One Markdown document for the editor. Comment lines are separate documents: a blank line between. */
export function toMarkdown(block: MdBlock): string {
    return toEditor(block.kind === 'comment' ? block.body.join('\n\n') : block.body.join('\n'));
}

/** Splits lines into top-level chunks at blank lines, keeping a fenced code block whole. */
function chunks(lines: string[]): string[] {
    const out: string[] = [];
    let cur: string[] = [];
    let fence = false;
    for (const l of lines) {
        if (/^\s*(```|~~~)/.test(l)) fence = !fence;
        if (!fence && l.trim() === '') {
            if (cur.length) out.push(cur.join('\n'));
            cur = [];
        } else {
            cur.push(l);
        }
    }
    if (cur.length) out.push(cur.join('\n'));
    return out;
}

/**
 * The lines to put between the markers for an edited document. `baseline` is what the editor
 * serialised from the untouched text and `edited` what it serialises now; a chunk the editor did not
 * change keeps its original source lines, so opening and applying without a change rewrites nothing.
 * Returns null if the edit has something a one-line comment cannot hold (a table, a code fence, a list).
 */
export function fromMarkdown(block: MdBlock, baseline: string, edited: string): string[] | null {
    const base = fromEditor(baseline).replace(/\r\n?/g, '\n');
    const next = fromEditor(edited).replace(/\r\n?/g, '\n');
    const original = block.body;
    if (next === base) return original;
    const nextChunks = chunks(next.split('\n'));
    const baseChunks = chunks(base.split('\n'));
    if (block.kind === 'comment') {
        // Each source line is its own document: only paragraphs and headings fit on one.
        const isTable = (c: string) => /^\s*\|?\s*:?-{3,}/.test(c.split('\n')[1] ?? '');
        if (nextChunks.some(c => /^(```|~~~)/.test(c) || isTable(c))) return null;
        const lines = nextChunks.map(c => c.replace(/\s*\n\s*/g, ' '));
        const flat = baseChunks.map(c => c.replace(/\s*\n\s*/g, ' '));
        if (flat.length === original.length && flat.length === lines.length) {
            return lines.map((p, i) => (p === flat[i] ? original[i] : p));
        }
        return lines;
    }
    const origChunks = chunks(original);
    if (origChunks.length === baseChunks.length && baseChunks.length === nextChunks.length) {
        const merged = nextChunks.map((c, i) => (c === baseChunks[i] ? origChunks[i] : c));
        return merged.join('\n\n').split('\n');
    }
    return nextChunks.join('\n\n').split('\n');
}
