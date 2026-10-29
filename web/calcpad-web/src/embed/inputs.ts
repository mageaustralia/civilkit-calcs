// The inputs of a worksheet for the embed (docs/embedding.md): read from the `#UI` lines of the Calcpad text (a sheet's
// `name: ? ...` lines compile to them), so the embed can draw its own form and check what a host sends. Pure functions,
// no DOM: web/e2e/embed-check.mjs drives them through the page.
import { findPythonBlocks } from '../wasm/python-blocks';

export type InputKind = 'number' | 'select' | 'radio' | 'check';

export interface EmbedOption { label: string; value: string }

export interface EmbedInput {
    /** The quantity name, as written in the worksheet. */
    name: string;
    kind: InputKind;
    /** The note or label of the line (plain text). */
    label: string;
    /** The default: the number for a number input, the option value for a choice, '0' or '1' for a check box. */
    value: string;
    /** The unit written after the default number ('' for none). */
    unit: string;
    min?: number;
    max?: number;
    step?: number;
    options?: EmbedOption[];
}

const ENTITIES: Record<string, string> = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ' };
/** Markup made plain: tags dropped, the common entities decoded. */
export const plain = (html: string): string =>
    html.replace(/<[^>]*>/g, '')
        .replace(/&(#x[0-9a-f]+|#\d+|\w+);/gi, (m, e: string) => e[0] === '#'
            ? String.fromCodePoint(e[1] === 'x' || e[1] === 'X' ? parseInt(e.slice(2), 16) : Number(e.slice(1)))
            : ENTITIES[e.toLowerCase()] ?? m)
        .replace(/\s+/g, ' ').trim();

/** The JSON object at the start of `s` (the `#UI` options) and the rest, or null when `s` does not start with one. */
function leadingJson(s: string): { json: Record<string, unknown>; rest: string } | null {
    if (!s.startsWith('{')) return null;
    for (let i = s.indexOf('}'); i > 0; i = s.indexOf('}', i + 1)) {
        try {
            const json = JSON.parse(s.slice(0, i + 1)) as unknown;
            if (json && typeof json === 'object' && !Array.isArray(json)) return { json: json as Record<string, unknown>, rest: s.slice(i + 1).trim() };
        } catch { /* a brace inside a string: try the next one */ }
    }
    return null;
}

const NUMBER = /^([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)\s*(.*)$/;
const finite = (v: unknown): number | undefined => (typeof v === 'number' && Number.isFinite(v) ? v : undefined);
const strings = (v: unknown): string[] | null => (Array.isArray(v) && v.every((x) => typeof x === 'string') ? (v as string[]) : null);
/** "300 mm" and "300mm" are the same option value. */
export const sameValue = (a: string, b: string): boolean => a.replace(/\s+/g, '') === b.replace(/\s+/g, '');

/** One `#UI` line, or null when it is not one the embed can draw (a data grid, an expression for a default). */
export function parseUiLine(line: string): EmbedInput | null {
    const m = /^\s*#UI\b\s*(.*)$/.exec(line);
    if (!m) return null;
    let rest = m[1];
    let opts: Record<string, unknown> = {};
    const lead = leadingJson(rest);
    if (lead) { opts = lead.json; rest = lead.rest; }
    // A label before the name: 'I-beam section -'h = 300mm
    let label = '';
    const lab = /^'([^']*?)\s*-'\s*(.*)$/.exec(rest);
    if (lab) { label = plain(lab[1]); rest = lab[2]; }
    const eq = /^([A-Za-zͰ-Ͽ][\wͰ-Ͽ,′″‴]*)\s*=\s*([^']*?)\s*(?:'(.*))?$/.exec(rest);
    if (!eq) return null;
    const [, name, rhs, note] = eq;
    if (note && !label) label = plain(note);
    const type = typeof opts.type === 'string' ? opts.type : '';
    if (type === 'dropdown' || type === 'radio') {
        const keys = strings(opts.keys);
        const values = strings(opts.values);
        if (!keys || !values || keys.length !== values.length || !keys.length) return null;
        const options = keys.map((k, i) => ({ label: plain(k), value: values[i] }));
        const chosen = options.find((o) => sameValue(o.value, rhs)) ?? options[0];
        return { name, kind: type === 'dropdown' ? 'select' : 'radio', label, value: chosen.value, unit: '', options };
    }
    if (type === 'checkbox') {
        const n = Number(rhs);
        return { name, kind: 'check', label, value: Number.isFinite(n) && n !== 0 ? '1' : '0', unit: '' };
    }
    if (type && type !== 'slider' && type !== 'input') return null;
    const num = NUMBER.exec(rhs);
    if (!num || !Number.isFinite(Number(num[1]))) return null;
    const input: EmbedInput = { name, kind: 'number', label, value: num[1], unit: num[2].trim() };
    const min = finite(opts.min), max = finite(opts.max), step = finite(opts.step);
    if (min !== undefined && max !== undefined && min < max) { input.min = min; input.max = max; }
    if (step !== undefined && step > 0) input.step = step;
    return input;
}

/** Every input of a Calcpad text, first declaration of a name only. */
export function parseInputs(calcpad: string): EmbedInput[] {
    const out: EmbedInput[] = [];
    const seen = new Set<string>();
    for (const line of calcpad.split('\n')) {
        const i = parseUiLine(line);
        if (i && !seen.has(i.name)) { seen.add(i.name); out.push(i); }
    }
    return out;
}

/**
 * For a sheet without any `?` input: its plain number lines (`L: 9.8 m | span`) become inputs, so an embed of a worked
 * example can still be tried. Only top level lines outside a fence, and only a number with an optional unit.
 */
export function promoteNumberLines(sheet: string): string {
    if (/^[^\s#|][^:\n]*:\s*\?/m.test(sheet)) return sheet;
    let fence = false;
    return sheet.split('\n').map((line) => {
        if (/^\s*```/.test(line)) { fence = !fence; return line; }
        if (fence) return line;
        const m = /^([A-Za-zͰ-Ͽ][\wͰ-Ͽ]*)(\s*):(\s*)([-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?)(\s*[A-Za-z°µμΩ%][A-Za-z0-9°µμΩ%_*/^]*)?(\s*(?:\|.*)?)$/.exec(line);
        if (!m) return line;
        const [, name, sp1, sp2, num, unit = '', tail] = m;
        return `${name}${sp1}:${sp2}? ${num}${unit}${tail}`;
    }).join('\n');
}

export type Checked = { ok: true; value: string } | { ok: false; error: string };

/**
 * A value a host sent for `input`, checked: a number must be finite and within the slider bounds, a choice one of the
 * option values (or labels), a check box a boolean or 0 or 1. The result is the override text the engine takes.
 */
export function checkValue(input: EmbedInput, raw: unknown): Checked {
    if (input.kind === 'number') {
        const n = typeof raw === 'number' ? raw : typeof raw === 'string' && raw.trim() !== '' ? Number(raw) : NaN;
        if (!Number.isFinite(n)) return { ok: false, error: `${input.name}: not a finite number` };
        if (input.min !== undefined && input.max !== undefined && (n < input.min || n > input.max)) {
            return { ok: false, error: `${input.name}: must be from ${input.min} to ${input.max}` };
        }
        return { ok: true, value: String(n) };
    }
    if (input.kind === 'check') {
        if (raw === true || raw === 1 || raw === '1') return { ok: true, value: '1' };
        if (raw === false || raw === 0 || raw === '0') return { ok: true, value: '0' };
        return { ok: false, error: `${input.name}: must be true or false` };
    }
    if (typeof raw !== 'string' && typeof raw !== 'number') return { ok: false, error: `${input.name}: must be one of the options` };
    const text = String(raw);
    const o = input.options?.find((x) => sameValue(x.value, text) || x.label === text);
    return o ? { ok: true, value: o.value } : { ok: false, error: `${input.name}: must be one of ${input.options?.map((x) => x.value).join(', ')}` };
}

const FENCE = /^```python[^\n]*\n([\s\S]*?)^```[ \t]*(?:\n|$)/gm;

/** Python fences of a sheet (```python ... ```) and `#python` blocks of a worksheet, as their source text. */
export function pythonBlocks(text: string): string[] {
    const out = [...text.matchAll(FENCE)].map((m) => m[1].trimEnd());
    return [...out, ...findPythonBlocks(text.replace(FENCE, '')).map((b) => b.code.trimEnd())];
}

/** The text with its Python blocks left out (an embed does not run them unless asked to). */
export function withoutPython(text: string): string {
    const lines = text.replace(FENCE, '').split('\n');
    for (const b of findPythonBlocks(lines.join('\n')).reverse()) lines.splice(b.start, (b.end < 0 ? lines.length : b.end + 1) - b.start);
    return lines.join('\n');
}
