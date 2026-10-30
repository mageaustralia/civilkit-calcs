// Number format: how numbers are SHOWN and TYPED (docs/localisation.md).
//
// Worksheet and sheet sources, saved input values and the engine all stay canonical: a decimal
// point, and `;` or `,` between arguments exactly as written. This module turns that canonical
// text into the reader's format on the way to the screen, the report and the exports, and turns
// what the reader types into an input box back into canonical text.

export type NumberFormatChoice = 'auto' | 'point' | 'comma' | 'space';

export interface NumberFormat {
    /** The decimal mark shown. */
    decimal: '.' | ',';
    /** The digit group mark ('' for none). */
    group: string;
}

export interface NumberFormatSetting {
    choice: NumberFormatChoice;
    grouping: boolean;
}

export const CHOICES: readonly NumberFormatChoice[] = ['auto', 'point', 'comma', 'space'];
export const DEFAULT_SETTING: NumberFormatSetting = { choice: 'auto', grouping: false };

/** The non-breaking space that separates digit groups in the space format. */
export const NBSP = '\u00a0';

/**
 * The pure text functions, as one self-contained function so the same source runs on this page
 * and, through `Function.prototype.toString`, inside the result frame (see numberFormatShim).
 * Nothing in here may refer to anything outside it.
 */
export function numberFormatCore(fmt: { decimal: string; group: string }) {
    const NB = '\u00a0';

    /** "1234.5" to the display text of `fmt` (digit groups only when `fmt.group` is set). */
    function formatCanonical(text: string, grouped: boolean): string {
        const dot = text.indexOf('.');
        let int = dot < 0 ? text : text.slice(0, dot);
        const frac = dot < 0 ? '' : text.slice(dot + 1);
        if (grouped && fmt.group && int.length > 3) {
            let out = '';
            for (let i = 0; i < int.length; i++) {
                if (i > 0 && (int.length - i) % 3 === 0) out += fmt.group;
                out += int[i];
            }
            int = out;
        }
        return dot < 0 ? int : int + fmt.decimal + frac;
    }

    /**
     * Every number in `text` (a stretch of a result: digits with an optional decimal point).
     * Names, subscripts and anything glued to a letter or another number are left alone.
     */
    function localiseText(text: string, grouped: boolean): string {
        if (fmt.decimal === '.' && !(grouped && fmt.group)) return text;
        return text.replace(/(^|[^\w.,])(\d+\.\d+|\d+|\.\d+)(?![\w]|\.\d)/g, (_m, pre: string, num: string) => {
            if (num[0] === '.') return pre + fmt.decimal + num.slice(1);
            return pre + formatCanonical(num, grouped);
        });
    }

    /** Typed or pasted text in a number box to canonical text: "6,5" and "6.5" both give "6.5". */
    function canonicalNumber(text: string): string {
        if (fmt.decimal !== ',') return text;
        // A decimal mark typed first, or alone, is on its way to a number.
        const bare = /^([-+\u2212]?)[.,]$/.exec(text);
        if (bare) return bare[1] + '.';
        const m = /^([-+\u2212]?)([0-9][0-9.,\u00a0\u202f ]*|[.,][0-9]+)([\s\S]*)$/.exec(text);
        if (!m) return text;
        let num = m[2];
        let tail = m[3];
        // Spaces between the number and a unit belong to the tail.
        const trail = /[\u00a0\u202f ]+$/.exec(num);
        if (trail) {
            num = num.slice(0, num.length - trail[0].length);
            tail = trail[0] + tail;
        }
        num = num.replace(/[\u00a0\u202f ]/g, '');
        const lastComma = num.lastIndexOf(',');
        const lastDot = num.lastIndexOf('.');
        if (lastComma >= 0 && lastDot >= 0) {
            // Both: the later one is the decimal mark, the other groups digits.
            const dec = Math.max(lastComma, lastDot);
            num = num.slice(0, dec).replace(/[.,]/g, '') + '.' + num.slice(dec + 1);
        } else if (lastComma < 0 && num.indexOf('.') !== lastDot) {
            // Several points and no comma can only be digit groups.
            num = num.replace(/\./g, '');
        } else if (lastComma >= 0) {
            // Commas only: one is the decimal mark; several can only be digit groups.
            num = num.indexOf(',') === lastComma ? num.replace(',', '.') : num.replace(/,/g, '');
        }
        return m[1] + num + tail;
    }

    /** Canonical text in a number box to what the box shows: the decimal mark only, no groups. */
    function localNumber(text: string): string {
        if (fmt.decimal !== ',') return text;
        const m = /^([-+\u2212]?)(\d*)\.(\d*)([\s\S]*)$/.exec(text);
        if (!m || (m[2] === '' && m[3] === '')) return text;
        return m[1] + m[2] + ',' + m[3] + m[4];
    }

    return { formatCanonical, localiseText, canonicalNumber, localNumber, NB };
}

const VOID = new Set([
    'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr',
]);
/** Elements whose text is never a result: names, subscripts, exponents, units, code, graphics. */
const SKIP = new Set(['var', 'sub', 'sup', 'i', 'script', 'style', 'textarea', 'svg', 'select', 'option', 'title', 'math', 'canvas']);
const RAW = new Set(['script', 'style', 'textarea', 'title']);

// A comment or doctype (no groups), or a tag: closing slash, name, attributes (quoted values may hold '>').
const TAG = /<!--[\s\S]*?-->|<![^>]*>|<(\/?)([a-zA-Z][\w:-]*)((?:[^>"']|"[^"]*"|'[^']*')*)>/g;

/**
 * The report's HTML in the reader's format. Only numbers in result text change: the text of an
 * equation span (class "eq"), and a bare number the value-only mode writes straight into the
 * page. Authored text, tables, names, units, exponents, attributes, scripts, styles and
 * graphics are copied through byte for byte, so nothing but digits and separators can differ.
 */
export function localiseHtml(html: string, fmt: NumberFormat, grouped: boolean): string {
    const core = numberFormatCore(fmt);
    if (fmt.decimal === '.' && !(grouped && fmt.group)) return html;
    let out = '';
    let last = 0;
    // The open elements: [name, isEq, isSkip, isErr].
    const stack: [string, boolean, boolean, boolean][] = [];
    let eq = 0;
    let skip = 0;
    const text = (from: number, to: number): void => {
        const chunk = html.slice(from, to);
        if (!chunk) return;
        if (skip > 0) { out += chunk; return; }
        if (eq > 0) {
            // Entities (&nbsp; &#8722;) hold digits that are not numbers; they pass through whole.
            out += chunk.split(/(&#?\w+;)/).map((p, k) => (k % 2 ? p : core.localiseText(p, grouped))).join('');
            return;
        }
        // A value-only number sits straight in the body, outside every element but the body.
        if ((stack.length === 0 || stack[stack.length - 1][0] === 'body') && /^\s*[-+\u2212]?(?:\d+\.\d+|\d+|\.\d+)\s*$/.test(chunk)) {
            out += chunk.replace(/(\d+\.\d+|\d+|\.\d+)/, n => (n[0] === '.' ? fmt.decimal + n.slice(1) : core.formatCanonical(n, grouped)));
            return;
        }
        out += chunk;
    };
    TAG.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = TAG.exec(html))) {
        const start = m.index;
        text(last, start);
        out += m[0];
        last = start + m[0].length;
        if (m[2] === undefined) continue;
        const closing = m[1] === '/';
        const name = m[2].toLowerCase();
        const selfClosed = m[3].endsWith('/');
        if (closing) {
            for (let i = stack.length - 1; i >= 0; i--) {
                if (stack[i][0] === name) {
                    for (let j = stack.length - 1; j >= i; j--) {
                        const [, e, s] = stack[j];
                        if (e) eq--;
                        if (s) skip--;
                    }
                    stack.length = i;
                    break;
                }
            }
            continue;
        }
        if (RAW.has(name) && !selfClosed) {
            // Everything up to the closing tag is code or data.
            const re = new RegExp(`</${name}\\s*>`, 'i');
            const rest = html.slice(last);
            const hit = re.exec(rest);
            const end = hit ? last + hit.index : html.length;
            out += html.slice(last, end);
            last = end;
            TAG.lastIndex = end;
            continue;
        }
        if (VOID.has(name) || selfClosed) continue;
        const isEq = name === 'span' && /\bclass\s*=\s*(?:"[^"]*\beq\b[^"]*"|'[^']*\beq\b[^']*')/.test(m[3]);
        const isSkip = SKIP.has(name) || (/\bclass\s*=\s*(?:"[^"]*\b(?:err|vec)\b[^"]*")/.test(m[3]));
        stack.push([name, isEq, isSkip, false]);
        if (isEq) eq++;
        if (isSkip) skip++;
    }
    text(last, html.length);
    return out;
}

/** The numbers of Office Math (Word): the text of runs that are not subscripts or exponents. */
export function localiseOmml(xml: string, fmt: NumberFormat, grouped: boolean): string {
    if (fmt.decimal === '.' && !(grouped && fmt.group)) return xml;
    const core = numberFormatCore(fmt);
    let depth = 0;
    return xml.replace(/<(\/?)m:(sub|sup)\b[^>]*>|(<m:t(?:\s[^>]*)?>)([^<]*)(<\/m:t>)/g, (m, close: string | undefined, _n: string | undefined, open?: string, body?: string, end?: string) => {
        if (open === undefined) {
            if (m.endsWith('/>')) return m;
            depth += close ? -1 : 1;
            if (depth < 0) depth = 0;
            return m;
        }
        return depth > 0 ? m : open + core.localiseText(body ?? '', grouped) + (end ?? '');
    });
}

/**
 * The script that goes into a result frame ahead of the form's own script. It makes the number
 * boxes of an input form speak the reader's format while the form's script keeps seeing, and
 * posting, canonical text: each box's value is read through canonicalNumber and written through
 * localNumber. Only boxes that take a number (not whole expressions) are changed.
 */
export function numberFormatShim(fmt: NumberFormat): string {
    const f = JSON.stringify({ decimal: fmt.decimal, group: fmt.group });
    return '<script>(' + numberFormatShimMain.toString() + ')(' + f + ',' + numberFormatCore.toString() + ');</script>\n';
}

/** Runs in the result frame (stringified by numberFormatShim). */
function numberFormatShimMain(fmt: { decimal: string; group: string }, makeCore: (f: { decimal: string; group: string }) => ReturnType<typeof numberFormatCore>) {
    const core = makeCore(fmt);
    const desc = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
    if (!desc || !desc.get || !desc.set) return;
    const get = desc.get;
    const set = desc.set;
    document.querySelectorAll('input.calcpad-ui-input').forEach(function (el) {
        const input = el as HTMLInputElement;
        if (input.getAttribute('data-ui-allow-expression') === '1') return;
        Object.defineProperty(input, 'value', {
            configurable: true,
            get() { return core.canonicalNumber(get.call(this)); },
            set(v: unknown) { set.call(this, core.localNumber(String(v))); },
        });
        set.call(input, core.localNumber(get.call(input)));
    });
    // The form's own range message ("must be from 2 to 12") is written in canonical numbers.
    const fix = function (note: Element): void {
        const t = note.textContent || '';
        const l = core.localiseText(t, false);
        if (l !== t) note.textContent = l;
    };
    // The note is made by the form's script after this one, and rewritten as the value changes.
    const obs = new MutationObserver(function () {
        document.querySelectorAll('.calcpad-ui-error').forEach(fix);
    });
    obs.observe(document.body, { childList: true, characterData: true, subtree: true });
}

/** Where the form's own script begins in a page the engine wrote (it is inlined, not a file). */
const FORM_SCRIPT_MARK = 'The document is sandboxed inside the host, so the parent is the only way out';

/** Puts the shim ahead of the form's script, if the page has one. */
export function withNumberFormatShim(html: string, fmt: NumberFormat): string {
    if (fmt.decimal !== ',') return html;
    const at = html.indexOf(FORM_SCRIPT_MARK);
    if (at < 0) return html;
    const open = html.lastIndexOf('<script', at);
    if (open < 0) return html;
    return html.slice(0, open) + numberFormatShim(fmt) + html.slice(open);
}

// ---- the setting -------------------------------------------------------------------------------

const KEY = 'calcpad-number-format';
const listeners: (() => void)[] = [];
let memory: NumberFormatSetting | null = null;

function isChoice(v: unknown): v is NumberFormatChoice {
    return typeof v === 'string' && (CHOICES as readonly string[]).includes(v);
}

/** What the page's link asks for (`?numfmt=comma`, `&numgroup=1`), or null. Not stored. */
export function urlOverride(search = typeof location === 'undefined' ? '' : location.search): Partial<NumberFormatSetting> | null {
    let p: URLSearchParams;
    try { p = new URLSearchParams(search); } catch { return null; }
    const choice = p.get('numfmt');
    const group = p.get('numgroup');
    const out: Partial<NumberFormatSetting> = {};
    if (isChoice(choice)) out.choice = choice;
    if (group === '1' || group === '0') out.grouping = group === '1';
    return Object.keys(out).length ? out : null;
}

function read(): NumberFormatSetting {
    try {
        const v = JSON.parse(localStorage.getItem(KEY) ?? 'null');
        if (v && isChoice(v.choice)) return { choice: v.choice, grouping: v.grouping === true };
    } catch {
        // Storage unavailable or corrupt: the default holds.
    }
    return { ...DEFAULT_SETTING };
}

/** The stored setting (without the link's override). */
export function storedSetting(): NumberFormatSetting {
    return memory ?? read();
}

/** The setting in force: the stored one, with the link's override on top. */
export function currentSetting(): NumberFormatSetting {
    return { ...storedSetting(), ...(urlOverride() ?? {}) };
}

export function setSetting(s: NumberFormatSetting): void {
    memory = { choice: s.choice, grouping: s.grouping };
    try {
        localStorage.setItem(KEY, JSON.stringify(memory));
    } catch {
        // The choice holds for this page only.
    }
    for (const l of listeners) l();
}

export function onNumberFormatChanged(listener: () => void): () => void {
    listeners.push(listener);
    return () => {
        const i = listeners.indexOf(listener);
        if (i >= 0) listeners.splice(i, 1);
    };
}

/** The separators a locale uses (Automatic), by what the browser's own number formatting does. */
export function formatForLocale(lang: string): Exclude<NumberFormatChoice, 'auto'> {
    try {
        const parts = new Intl.NumberFormat(lang).formatToParts(1234567.5);
        const decimal = parts.find(p => p.type === 'decimal')?.value;
        const group = parts.find(p => p.type === 'group')?.value;
        if (decimal === ',') return group && /^[\s\u00a0\u202f\u2009]$/.test(group) ? 'space' : 'comma';
    } catch {
        // An unknown tag: the point format.
    }
    return 'point';
}

export function languageNow(): string {
    return typeof navigator === 'undefined' ? 'en-US' : (navigator.language || 'en-US');
}

export function resolve(choice: NumberFormatChoice, lang = languageNow()): NumberFormat {
    const c = choice === 'auto' ? formatForLocale(lang) : choice;
    if (c === 'comma') return { decimal: ',', group: '.' };
    if (c === 'space') return { decimal: ',', group: NBSP };
    return { decimal: '.', group: ',' };
}

/** The format and grouping in force now. */
export function activeFormat(): { fmt: NumberFormat; grouped: boolean } {
    const s = currentSetting();
    return { fmt: resolve(s.choice), grouped: s.grouping };
}

/** A sample number in a format, for the Settings page. */
export function sample(fmt: NumberFormat, grouped: boolean): string {
    return numberFormatCore(fmt).formatCanonical('1234567.5', grouped);
}
