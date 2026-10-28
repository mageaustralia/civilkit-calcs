// Pure text logic of the prose formatting toolbar (see prose-toolbar.ts): each function takes the text of
// a prose line and a selection (0-based offsets into the line) and returns the new line and selection.
// No DOM and no Monaco, so `node e2e/prose-format-check.mjs` can test it.

export type ProseAction = 'normal' | 'h1' | 'h2' | 'h3' | 'bold' | 'italic' | 'code' | 'link' | 'bullet' | 'number';

/** One entry per button; the title names the Markdown the button types. */
export const PROSE_TIPS: Record<ProseAction, string> = {
    normal: 'Normal text',
    h1: 'Heading 1: # Title',
    h2: 'Heading 2: ## Title',
    h3: 'Heading 3: ### Title',
    bold: 'Bold: **text**',
    italic: 'Italic: *text*',
    code: 'Code: `text`',
    link: 'Link: [text](address)',
    bullet: 'Bulleted list: - item',
    number: 'Numbered list: 1. item',
};

export interface LineEdit { text: string; from: number; to: number }
export interface ProseState { heading: number; bold: boolean; italic: boolean; code: boolean; link: boolean; bullet: boolean; number: boolean }

const WORD = /[\p{L}\p{N}_]/u;
const HEADING = /^(\s{0,3})(#{1,6})\s+/;
const BULLET = /^(\s*)[-*+]\s+/;
const NUMBER = /^(\s*)\d+[.)]\s+/;

/** The selection, or the word around the caret when it is empty. */
function span(line: string, from: number, to: number): [number, number] {
    if (from !== to) return [from, to];
    let s = from;
    let e = to;
    while (s > 0 && WORD.test(line[s - 1])) s--;
    while (e < line.length && WORD.test(line[e])) e++;
    return [s, e];
}

function runBefore(line: string, i: number, ch: string): number {
    let n = 0;
    while (i - n > 0 && line[i - n - 1] === ch) n++;
    return n;
}
function runAfter(line: string, i: number, ch: string): number {
    let n = 0;
    while (i + n < line.length && line[i + n] === ch) n++;
    return n;
}

interface Core { start: number; end: number; before: number; after: number }

/** The text to format without the marker characters around it or inside the selection's edges. */
function core(line: string, from: number, to: number, ch: string): Core {
    let [s, e] = span(line, from, to);
    while (s < e && line[s] === ch) s++;
    while (e > s && line[e - 1] === ch) e--;
    while (s < e && /\s/.test(line[s])) s++;
    while (e > s && /\s/.test(line[e - 1])) e--;
    return { start: s, end: e, before: runBefore(line, s, ch), after: runAfter(line, e, ch) };
}

/** Bold, italic or code: wraps the selection (or the word at the caret), or unwraps it when it is already so. */
export function toggleWrap(line: string, from: number, to: number, marker: '**' | '*' | '`'): LineEdit {
    const ch = marker[0];
    const m = marker.length;
    const c = core(line, from, to, ch);
    if (c.start === c.end) {
        // Nothing to wrap: a pair of markers with the caret between.
        return { text: line.slice(0, from) + marker + marker + line.slice(from), from: from + m, to: from + m };
    }
    const n = Math.min(c.before, c.after);
    const has = ch === '`' ? n >= 1 : marker === '**' ? n >= 2 : n % 2 === 1;
    const head = line.slice(0, c.start - c.before);
    const tail = line.slice(c.end + c.after);
    const body = line.slice(c.start, c.end);
    const keep = (k: number) => ch.repeat(k);
    if (has) {
        const b = c.before - m;
        const a = c.after - m;
        return { text: head + keep(b) + body + keep(a) + tail, from: head.length + b, to: head.length + b + body.length };
    }
    const b = c.before + m;
    const a = c.after + m;
    return { text: head + keep(b) + body + keep(a) + tail, from: head.length + b, to: head.length + b + body.length };
}

const LINK = /\[([^\]]*)\]\(([^)]*)\)/g;

/** Link: wraps the selection as `[text](url)` with `url` selected, or removes the link the selection is in. */
export function toggleLink(line: string, from: number, to: number): LineEdit {
    for (const m of line.matchAll(LINK)) {
        const s = m.index!;
        const e = s + m[0].length;
        if (from >= s && to <= e) {
            return { text: line.slice(0, s) + m[1] + line.slice(e), from: s, to: s + m[1].length };
        }
    }
    const [s0, e0] = span(line, from, to);
    let s = s0;
    let e = e0;
    while (s < e && /\s/.test(line[s])) s++;
    while (e > s && /\s/.test(line[e - 1])) e--;
    const text = s === e ? 'text' : line.slice(s, e);
    const head = line.slice(0, s) + '[' + text + '](';
    return { text: head + 'url)' + line.slice(e), from: head.length, to: head.length + 3 };
}

/** Normal text or a heading of the given level (0 to 3); a list marker is dropped. */
export function setStyle(line: string, level: number): LineEdit {
    const body = line.replace(HEADING, '').replace(BULLET, '').replace(NUMBER, '');
    const prefix = level > 0 ? '#'.repeat(level) + ' ' : '';
    const text = prefix + body;
    return { text, from: text.length, to: text.length };
}

/**
 * Bulleted or numbered list over a run of lines: when every non-blank line already is that list
 * the markers are removed, otherwise each non-blank line becomes an item (numbered from 1 for a
 * numbered list; a heading marker is dropped).
 */
export function toggleList(lines: string[], kind: 'bullet' | 'number'): string[] {
    const re = kind === 'bullet' ? BULLET : NUMBER;
    const items = lines.filter(l => l.trim() !== '');
    const all = items.length > 0 && items.every(l => re.test(l));
    let i = 0;
    return lines.map(l => {
        if (l.trim() === '' && lines.length > 1) return l;
        const indent = /^\s*/.exec(l)![0];
        if (all) return l.replace(re, '$1');
        i++;
        const body = l.replace(HEADING, '').replace(BULLET, '').replace(NUMBER, '').trimStart();
        return indent + (kind === 'bullet' ? '- ' : `${i}. `) + body;
    });
}

/** What the line already is, for the buttons' pressed state. */
export function proseState(line: string, from: number, to: number): ProseState {
    const h = HEADING.exec(line);
    const st = (marker: '**' | '*' | '`') => {
        const ch = marker[0];
        const c = core(line, from, to, ch);
        if (c.start === c.end) return false;
        const n = Math.min(c.before, c.after);
        return ch === '`' ? n >= 1 : marker === '**' ? n >= 2 : n % 2 === 1;
    };
    let link = false;
    for (const m of line.matchAll(LINK)) {
        const s = m.index!;
        if (from >= s && to <= s + m[0].length) link = true;
    }
    return { heading: h ? h[2].length : 0, bold: st('**'), italic: st('*'), code: st('`'), link, bullet: BULLET.test(line), number: NUMBER.test(line) };
}

/**
 * Applies a toolbar action to lines[first..last] with a selection given as (line, column) pairs
 * (0-based). Returns the new lines of that range and the new selection, or null when nothing applies.
 */
export function applyProse(
    lines: string[],
    action: ProseAction,
    sel: { l1: number; c1: number; l2: number; c2: number },
): { lines: string[]; l1: number; c1: number; l2: number; c2: number } | null {
    const { l1, c1, l2, c2 } = sel;
    const out = lines.slice();
    const skip = (l: number) => l1 !== l2 && out[l].trim() === '';
    const res = { lines: out, l1, c1, l2, c2 };
    const wrap = (marker: '**' | '*' | '`') => {
        for (let l = l1; l <= l2; l++) {
            if (skip(l)) continue;
            const r = toggleWrap(out[l], l === l1 ? c1 : 0, l === l2 ? c2 : out[l].length, marker);
            out[l] = r.text;
            if (l === l1) res.c1 = r.from;
            if (l === l2) res.c2 = r.to;
        }
    };
    switch (action) {
        case 'bold': wrap('**'); break;
        case 'italic': wrap('*'); break;
        case 'code': wrap('`'); break;
        case 'link': {
            if (l1 !== l2) return null;
            const e = toggleLink(out[l1], c1, c2);
            out[l1] = e.text;
            res.c1 = e.from;
            res.c2 = e.to;
            break;
        }
        case 'normal': case 'h1': case 'h2': case 'h3': {
            const level = action === 'normal' ? 0 : Number(action[1]);
            for (let l = l1; l <= l2; l++) {
                if (skip(l)) continue;
                const was = out[l].length;
                out[l] = setStyle(out[l], level).text;
                if (l === l1) res.c1 = Math.max(0, c1 + out[l].length - was);
                if (l === l2) res.c2 = Math.max(0, c2 + out[l].length - was);
            }
            break;
        }
        case 'bullet': case 'number': {
            const next = toggleList(out.slice(l1, l2 + 1), action);
            for (let l = l1; l <= l2; l++) out[l] = next[l - l1];
            res.c1 = Math.max(0, c1 + next[0].length - lines[l1].length);
            res.c2 = Math.max(0, c2 + next[next.length - 1].length - lines[l2].length);
            break;
        }
    }
    return res;
}
