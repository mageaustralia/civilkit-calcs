// Moving, duplicating and deleting a rendered block of a SHEET, as edits to its source lines. Plain data in and out, with
// no imports, so a Node check can load it. `starts` are the 1-based source lines the preview reports for its top-level
// blocks; a block runs from its start to the last non-blank line before the next one.

/** Replaces source lines `from`..`to` (1-based, inclusive) with `text`; an empty `text` deletes those lines. */
export interface BlockEdit { from: number; to: number; text: string[] }

export type BlockOp = 'up' | 'down' | 'duplicate' | 'delete';

interface Span { start: number; end: number }

const blank = (s: string | undefined): boolean => s === undefined || s.trim() === '';

function spans(lines: string[], starts: number[]): Span[] {
    // A blank source line renders as an empty block of its own; it is spacing, not a block.
    const s = [...new Set(starts)].filter(n => Number.isInteger(n) && n >= 1 && n <= lines.length && !blank(lines[n - 1])).sort((a, b) => a - b);
    return s.map((start, i) => {
        let end = (i + 1 < s.length ? s[i + 1] : lines.length + 1) - 1;
        while (end > start && blank(lines[end - 1])) end--;
        return { start, end };
    });
}

/** The edit for `op` on the block that starts at `line`, or null when it does not apply (first block up, last down, unknown line). */
export function blockEdit(lines: string[], starts: number[], line: number, op: BlockOp): BlockEdit | null {
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
    const next = all[i + 1];
    if (next) return { from: cur.start, to: next.start - 1, text: [] };
    const prev = all[i - 1];
    return { from: prev ? prev.end + 1 : cur.start, to: lines.length, text: [] };
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
    return Math.max(Math.min(cur.start, lines.length - (cur.end - cur.start + 1)), 1);
}
