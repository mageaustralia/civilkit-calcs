/**
 * Moves values entered in the input form into the defaults written in the source, and describes where the two
 * differ. Pure text work on the sheet's input lines (`L: ? 6 m`, a drop-down, radio buttons, a checkbox, a slider)
 * and on the numeric `#UI name = 6m` line; the caller supplies the overrides already matched to their source line.
 */

/** An entered value paired with the source line of its control (null when it applies to none). */
export interface DefaultRow {
    key: string;
    value: string;
    line: number | null;
}

export interface DefaultsResult {
    source: string;
    /** Keys whose value now stands in the source. */
    applied: string[];
    /** Keys left as entered values: a pass of a loop, a control the line could not be read for. */
    kept: string[];
}

/** A value that differs from the default written in the source, to show beside its line. */
export interface DefaultDifference {
    line: number;
    key: string;
    /** What the form holds, as it would be written in the source. */
    shown: string;
    /** What the source line says now. */
    written: string;
}

/** Keys the form adds to drive its radio buttons; they follow the control they belong to. */
const isHelperKey = (key: string): boolean => key.startsWith('map:');

/** Compares two option values (`200 GPa`, `200GPa`, `m³`, `m^3`) without regard to spacing and spelling. */
export function sameValue(a: string, b: string): boolean {
    const norm = (s: string): string => s
        .replace(/\s+/g, '').replace(/[·*]/g, '').replace(/\^/g, '')
        .replace(/²/g, '2').replace(/³/g, '3').replace(/⁻¹/g, '-1').toLowerCase();
    if (norm(a) === norm(b)) return true;
    const na = Number(a.trim()), nb = Number(b.trim());
    return a.trim() !== '' && b.trim() !== '' && Number.isFinite(na) && na === nb;
}

const NUMBER = /^-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/;

/** Splits `a = 1, *b = 2, "c, d" = 3` at the commas outside quotes. */
function splitOptions(list: string): string[] {
    const parts: string[] = [];
    let cur = '', quoted = false;
    for (const ch of list) {
        if (ch === '"') quoted = !quoted;
        if (ch === ',' && !quoted) { parts.push(cur); cur = ''; } else cur += ch;
    }
    parts.push(cur);
    return parts;
}

interface Option { text: string; value: string; marked: boolean }

function parseOption(raw: string): Option | null {
    const m = /^(\s*)(\*?)(\s*)((?:"[^"]*"|[^=])*?)=(.*)$/.exec(raw);
    if (!m) return null;
    return { text: raw, value: m[5].trim(), marked: m[2] === '*' };
}

/** Moves the `*` of a choice to the option whose value is `value`; null when no option has it. */
function moveMarker(list: string, value: string): string | null {
    const raws = splitOptions(list);
    const opts = raws.map(parseOption);
    if (opts.some((o) => !o)) return null;
    const pick = (opts as Option[]).findIndex((o) => sameValue(o.value, value));
    if (pick < 0) return null;
    // Already the default: the marked option, or the first when none is marked.
    const marked = (opts as Option[]).findIndex((o) => o.marked);
    if ((marked < 0 ? 0 : marked) === pick) return list;
    return raws.map((raw, i) => {
        const bare = raw.replace(/^(\s*)\*\s*/, '$1');
        if (i !== pick) return bare;
        const lead = /^\s*/.exec(bare)![0];
        return `${lead}*${bare.slice(lead.length)}`;
    }).join(',');
}

/** The line with its default replaced by `value`, or null when this line cannot take it. */
export function withDefault(line: string, value: string): string | null {
    // A sheet input: name, colon or equals, a question mark, then the default.
    const input = /^(\s*[^\s:=?|][^:=?|]*?\s*[:=]\s*\?\s*)(.*)$/.exec(line);
    if (input) {
        const [, head, rest] = input;
        if (rest.startsWith('[') || rest.startsWith('(')) {
            const close = rest.startsWith('[') ? ']' : ')';
            // The option list is the first bracketed group; a slider's bounds are not one (they follow a number).
            const end = rest.indexOf(close);
            if (end < 0) return null;
            const moved = moveMarker(rest.slice(1, end), value);
            return moved === null ? null : `${head}${rest[0]}${moved}${rest.slice(end)}`;
        }
        const word = /^(on|off|true|false)\b/i.exec(rest);
        if (word) {
            const on = value.trim() === '1' || /^(on|true)$/i.test(value.trim());
            const off = value.trim() === '0' || /^(off|false)$/i.test(value.trim());
            if (!on && !off) return null;
            const long = /^(true|false)$/i.test(word[1]);
            return `${head}${on ? (long ? 'true' : 'on') : (long ? 'false' : 'off')}${rest.slice(word[1].length)}`;
        }
        const num = /^(-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)/.exec(rest);
        if (num && NUMBER.test(value.trim())) return `${head}${value.trim()}${rest.slice(num[1].length)}`;
        return null;
    }
    // The engine's own line: `#UI name = 6m 'note`, numeric entry boxes only.
    const ui = /^(\s*#UI\s+(?:\{[^}]*\}\s*)?[^=\s]+\s*=\s*)(-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)(.*)$/i.exec(line);
    if (ui && !/"type"\s*:\s*"(dropdown|radio|checkbox)"/.test(line) && NUMBER.test(value.trim()))
        return `${ui[1]}${value.trim()}${ui[3]}`;
    return null;
}

/** Whether an entered value belongs to the first (or only) pass of its control, the one a default stands for. */
const isFirstPass = (key: string): boolean => {
    const parts = key.split(':');
    return parts.length < 3 || parts[2] === '1';
};

/** Writes each entered value into the default of its line. Lines that cannot take one keep the value as entered. */
export function applyOverridesToDefaults(source: string, rows: DefaultRow[]): DefaultsResult {
    const eol = source.includes('\r\n') ? '\r\n' : '\n';
    const lines = source.split(/\r?\n/);
    const applied: string[] = [];
    const kept: string[] = [];
    const done = new Set<number>();
    for (const row of rows) {
        if (isHelperKey(row.key)) continue;
        if (row.line === null || !isFirstPass(row.key) || done.has(row.line)) { kept.push(row.key); continue; }
        const current = lines[row.line - 1];
        const next = current === undefined ? null : withDefault(current, row.value);
        if (next === null) { kept.push(row.key); continue; }
        lines[row.line - 1] = next;
        done.add(row.line);
        applied.push(row.key);
    }
    return { source: lines.join(eol), applied, kept };
}

/** Lines whose entered value differs from the default written there. */
export function differences(source: string, rows: DefaultRow[]): DefaultDifference[] {
    const lines = source.split(/\r?\n/);
    const out: DefaultDifference[] = [];
    for (const row of rows) {
        if (isHelperKey(row.key) || row.line === null || !isFirstPass(row.key)) continue;
        const current = lines[row.line - 1];
        if (current === undefined) continue;
        const next = withDefault(current, row.value);
        if (next === null || next === current) continue;
        out.push({ line: row.line, key: row.key, shown: row.value, written: current.trim() });
    }
    return out;
}

/** The pattern of a line that declares the input `name`: `name: ? ...`, `name = ? ...` or `#UI name = ...`. */
function declaration(name: string): RegExp[] {
    const own = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return [new RegExp(`^\\s*${own}\\s*[:=]\\s*\\?`), new RegExp(`^\\s*#UI\\s+(?:\\{[^}]*\\}\\s*)?${own}\\s*=`, 'i')];
}

/** The ordinal-th input line of the sheet that declares `name`, 1-based, or null. */
function findInputLine(lines: string[], name: string, ordinal: number): number | null {
    const patterns = declaration(name);
    let seen = 0;
    for (let i = 0; i < lines.length; i++) {
        if (patterns.some((re) => re.test(lines[i])) && ++seen === ordinal) return i + 1;
    }
    return null;
}

/**
 * Gives a line to each row that has none, by looking for the input of that name in the text, and checks the line a row
 * has: lines reported by a form drawn before an edit may have moved, so a line that does not declare the name is looked
 * for again. A name the text does not spell the same way (a Greek letter written out) keeps the line it was given.
 */
export function locateRows(source: string, rows: DefaultRow[]): DefaultRow[] {
    const lines = source.split(/\r?\n/);
    return rows.map((row) => {
        if (isHelperKey(row.key)) return row;
        const [name, ordinal] = row.key.split(':');
        if (!name) return row;
        if (row.line !== null) {
            const text = lines[row.line - 1];
            if (text !== undefined && (declaration(name).some((re) => re.test(text)) || /[^\x00-\x7f]/.test(name))) return row;
        }
        const line = findInputLine(lines, name, Number(ordinal) || 1);
        return line === null ? { ...row, line: row.line !== null && /[^\x00-\x7f]/.test(name) ? row.line : null } : { ...row, line };
    });
}
