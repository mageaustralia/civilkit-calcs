// Moving, duplicating and deleting a rendered block of a SHEET, as edits to its source lines. Plain data in and out, with
// no imports, so a Node check can load it. `starts` are the 1-based source lines the preview reports for its top-level
// blocks; a block runs from its start to the last non-blank line before the next one.

/** Replaces source lines `from`..`to` (1-based, inclusive) with `text`; an empty `text` deletes those lines. */
export interface BlockEdit { from: number; to: number; text: string[] }

export type BlockOp = 'up' | 'down' | 'duplicate' | 'delete' | 'copy' | 'cut' | 'paste' | 'disable' | 'enable';

interface Span { start: number; end: number }

const blank = (s: string | undefined): boolean => s === undefined || s.trim() === '';

function spans(lines: string[], starts: number[]): Span[] {
    // A blank source line renders as an empty block of its own; it is spacing, not a block.
    // A run of disabled lines is one block, whether or not the preview reported its lines (it draws nothing for them).
    const off = (n: number): boolean => n >= 1 && n <= lines.length && blockDisabled(lines, n);
    const runs = lines.map((_l, i) => i + 1).filter(n => off(n) && !off(n - 1));
    const s = [...new Set([...starts, ...runs])].filter(n => Number.isInteger(n) && n >= 1 && n <= lines.length && !blank(lines[n - 1]) && !(off(n) && off(n - 1))).sort((a, b) => a - b);
    return s.map((start, i) => {
        let end = (i + 1 < s.length ? s[i + 1] : lines.length + 1) - 1;
        while (end > start && blank(lines[end - 1])) end--;
        return { start, end };
    });
}

/** The edit for `op` on the block that starts at `line`, or null when it does not apply (first block up, last down, unknown line). */
export function blockEdit(lines: string[], starts: number[], line: number, op: BlockOp, clip = ''): BlockEdit | null {
    if (op === 'enable') return toggleEdit(lines, line, false);
    const all = spans(lines, starts);
    const i = all.findIndex(b => b.start === line);
    if (i < 0) return null;
    const cur = all[i];
    const body = (b: Span): string[] => lines.slice(b.start - 1, b.end);
    const gap = (a: Span, b: Span): string[] => Array<string>(Math.max(b.start - a.end - 1, 0)).fill('');
    if (op === 'up' || op === 'down') {
        const other = all[op === 'up' ? i - 1 : i + 1];
        if (!other) return null;
        const [first, second] = op === 'up' ? [other, cur] : [cur, other];
        return { from: first.start, to: second.end, text: [...body(second), ...gap(first, second), ...body(first)] };
    }
    if (op === 'duplicate') {
        // A blank line keeps the copy from running into its original as one paragraph or list.
        return { from: cur.end, to: cur.end, text: [lines[cur.end - 1], '', ...body(cur)] };
    }
    if (op === 'copy') return null;
    if (op === 'disable') return toggleEdit(lines, cur.start, true, cur.end);
    if (op === 'paste') {
        const add = clip.split(/\r?\n/);
        while (add.length && blank(add[0])) add.shift();
        while (add.length && blank(add[add.length - 1])) add.pop();
        return add.length ? { from: cur.end, to: cur.end, text: [lines[cur.end - 1], '', ...add] } : null;
    }
    const next = all[i + 1];
    if (next) return { from: cur.start, to: next.start - 1, text: [] };
    const prev = all[i - 1];
    return { from: prev ? prev.end + 1 : cur.start, to: lines.length, text: [] };
}

/** The source text of the block that starts at `line`, or null for an unknown line (Copy and Cut). */
export function blockText(lines: string[], starts: number[], line: number): string | null {
    const b = spans(lines, starts).find(x => x.start === line);
    return b ? lines.slice(b.start - 1, b.end).join('\n') : null;
}

// A disabled line is wrapped in an HTML comment, the one comment the sheet language has: the preview and the calculation
// skip it and it stays in the text. Each line is wrapped on its own, because a comment that spans lines swallows the
// blocks after it. A blank line inside the block becomes an empty comment so the block stays one run. Indentation stays
// outside the comment, so a disabled `if` body keeps its place. `-->` inside a line would end the comment early, so it is
// written as `--&gt;` and turned back on Enable.
const OFF = /^(\s*)<!-- (.*) -->$/;
const offLine = (l: string): string => {
    if (blank(l)) return '<!-- -->';
    const ind = /^\s*/.exec(l)![0];
    return `${ind}<!-- ${l.slice(ind.length).replace(/-->/g, '--&gt;')} -->`;
};
const onLine = (l: string): string => (l === '<!-- -->' ? '' : l.replace(OFF, (_m, ind: string, body: string) => ind + body.replace(/--&gt;/g, '-->')));

/** Disables the lines `from`..`to`, or (enable) the run of disabled lines around `from`. Null when there is nothing to change. */
function toggleEdit(lines: string[], from: number, off: boolean, to = from): BlockEdit | null {
    const isOff = (l: string | undefined): boolean => l !== undefined && (l === '<!-- -->' || OFF.test(l));
    if (off) {
        const body = lines.slice(from - 1, to);
        return body.length && !body.every(isOff) ? { from, to, text: body.map(offLine) } : null;
    }
    if (!isOff(lines[from - 1])) return null;
    let a = from, b = from;
    while (a > 1 && isOff(lines[a - 2])) a--;
    while (b < lines.length && isOff(lines[b])) b++;
    return { from: a, to: b, text: lines.slice(a - 1, b).map(onLine) };
}

/** True when the block that starts at `line` is disabled. */
export function blockDisabled(lines: string[], line: number): boolean {
    const l = lines[line - 1];
    return l !== undefined && (l === '<!-- -->' || OFF.test(l));
}

/** The start line of the block that holds `line` (a caret), or null on spacing below the last line of a block. */
export function blockHolding(lines: string[], starts: number[], line: number): number | null {
    const all = spans(lines, starts);
    const b = [...all].reverse().find(x => x.start <= line);
    return b && line <= b.end ? b.start : null;
}

/** Block starts read from the text alone, for when no preview has reported them: a run of lines between blank lines, a fence or $$ pair kept whole. */
export function textStarts(lines: string[]): number[] {
    const out: number[] = [];
    let fence = false, prevBlank = true;
    lines.forEach((l, i) => {
        const t = l.trim();
        if (!fence && t !== '' && prevBlank) out.push(i + 1);
        if (/^(```|~~~)/.test(t) || t === '$$') fence = !fence;
        prevBlank = t === '' && !fence;
    });
    return out;
}

/** The block's start line after the edit, so the caret can go to the moved or copied block. */
export function blockLineAfter(lines: string[], starts: number[], line: number, op: BlockOp): number {
    const all = spans(lines, starts);
    const i = all.findIndex(b => b.start === line);
    if (i < 0) return line;
    const cur = all[i];
    if (op === 'up' && all[i - 1]) return all[i - 1].start;
    if (op === 'down' && all[i + 1]) {
        const n = all[i + 1];
        return cur.start + (n.end - n.start + 1) + Math.max(n.start - cur.end - 1, 0);
    }
    if (op === 'duplicate') return cur.end + 2;
    if (op === 'paste') return cur.end + 2;
    if (op === 'copy' || op === 'disable' || op === 'enable') return cur.start;
    return Math.max(Math.min(cur.start, lines.length - (cur.end - cur.start + 1)), 1);
}

/** A move edit, with the start line the moved block (or section) has afterwards. */
export interface MoveEdit extends BlockEdit { at: number }

/**
 * The edit that moves the block starting at `line` to just before the block starting at `beforeLine` (0 or omitted: to the
 * end). `lastLine` is the start of the last block that goes with it (a heading drags its section); it defaults to `line`.
 * Null when it does not apply: unknown lines, or a target inside or right beside the moved blocks.
 */
export function moveBlockTo(lines: string[], starts: number[], line: number, beforeLine?: number, lastLine?: number): MoveEdit | null {
    const all = spans(lines, starts);
    const i = all.findIndex(b => b.start === line);
    const j = lastLine === undefined ? i : all.findIndex(b => b.start === lastLine);
    if (i < 0 || j < i) return null;
    const t = !beforeLine ? all.length : all.findIndex(b => b.start === beforeLine);
    if (t < 0 || (t >= i && t <= j + 1)) return null;
    const body = (a: number, b: number): string[] => lines.slice(all[a].start - 1, all[b].end);
    const moved = body(i, j);
    if (t > j) {
        // Down: what lay between goes first, the blocks follow it, with the gap they had before.
        const gap = Array<string>(Math.max(all[j + 1].start - all[j].end - 1, 0)).fill('');
        const mid = body(j + 1, t - 1);
        return { from: all[i].start, to: all[t - 1].end, text: [...mid, ...gap, ...moved], at: all[i].start + mid.length + gap.length };
    }
    const gap = Array<string>(Math.max(all[i].start - all[i - 1].end - 1, 0)).fill('');
    return { from: all[t].start, to: all[j].end, text: [...moved, ...gap, ...body(t, i - 1)], at: all[t].start };
}
