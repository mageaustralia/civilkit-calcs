// Name scanning for the editor's navigation features (rename, completion context, quick recall).
// Plain functions with no Monaco import, so they can be reasoned about on their own.
//
// A sheet line is scanned for the names it USES as code: strings, notes after `|`, the unit written
// after a number and the target of `->` are skipped, and so is prose. The unit rule is the
// compiler's (calcpad-sheet/src/expr.rs): a name right after a number is a unit, and a unit
// continues through `*` and `/` written without spaces and through `^` with a number.

/** A character a name may contain (letters, digits, `_`, primes and subscripts; not raised digits). */
export function isNameChar(c: string): boolean {
    return /[\p{L}\p{N}_′″‴⁗]/u.test(c) && !/[⁰-⁹²³¹]/u.test(c);
}

/** A character a name may start with. */
export function isNameStart(c: string): boolean {
    return /\p{L}/u.test(c);
}

export interface NameTok { text: string; start: number; end: number }

/** The words a sheet reads as syntax, never as names. */
export const SHEET_KEYWORDS = new Set([
    'if', 'else', 'for', 'while', 'repeat', 'in', 'step', 'plot', 'and', 'or', 'break', 'continue',
    'on', 'off', 'true', 'false',
]);

/** Where a line's code ends: the start of a note (`|` that is not `||`) or the `->` of a unit conversion. */
function codeEnd(text: string, note: boolean): number {
    let inStr = false;
    for (let i = 0; i < text.length; i++) {
        const c = text[i];
        if (c === '"') { inStr = !inStr; continue; }
        if (inStr) continue;
        if (c === '-' && text[i + 1] === '>') return i;
        if (note && c === '|') {
            if (text[i + 1] === '|') { i++; continue; }
            return i;
        }
    }
    return text.length;
}

/**
 * The names a sheet quantity, block-header, condition or plot line uses. `from` is where scanning
 * starts (after `>> ` on a raw line). Returns tokens that are not strings, units or keywords.
 */
export function sheetNames(text: string, opts: { note: boolean; from?: number } = { note: false }): NameTok[] {
    const end = codeEnd(text, opts.note);
    const out: NameTok[] = [];
    let i = opts.from ?? 0;
    let prevNum: boolean = false;
    let inUnit: boolean = false;
    let chain: boolean = false;
    let caret: boolean = false;
    let space: boolean = false;
    while (i < end) {
        const c = text[i];
        if (c === ' ' || c === '\t') { space = true; i++; continue; }
        // A number: digits, a fraction, an exponent. A raised digit after a name is not one.
        if (/\d/.test(c)) {
            const m = /^\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/.exec(text.slice(i, end));
            if (m) {
                const unitish: boolean = inUnit && caret;
                i += m[0].length;
                prevNum = !unitish;
                inUnit = unitish;
                chain = false;
                caret = false;
                space = false;
                continue;
            }
        }
        if (c === '"') {
            const close = text.indexOf('"', i + 1);
            i = close < 0 || close >= end ? end : close + 1;
            prevNum = false; inUnit = false; chain = false; caret = false; space = false;
            continue;
        }
        if (isNameStart(c)) {
            let j = i + 1;
            while (j < end && isNameChar(text[j])) j++;
            const word = text.slice(i, j);
            const unit = prevNum || (chain && !space);
            if (unit) {
                inUnit = true; prevNum = false; chain = false; caret = false;
            } else {
                if (!SHEET_KEYWORDS.has(word) || text[j] === '(') out.push({ text: word, start: i, end: j });
                inUnit = false; prevNum = false; chain = false; caret = false;
            }
            i = j;
            space = false;
            continue;
        }
        // Operators: `*` and `/` without spaces continue a unit; `^` and a signed exponent too.
        if ((c === '*' || c === '/' || c === '·' || c === '×' || c === '÷') && inUnit && !space) {
            chain = true; prevNum = false; caret = false;
        } else if (c === '^' && inUnit) {
            caret = true; chain = false; prevNum = false;
        } else if ((c === '-' || c === '−') && inUnit && caret) {
            prevNum = false;
        } else {
            prevNum = false; inUnit = false; chain = false; caret = false;
        }
        space = false;
        i++;
    }
    return out;
}

/** The names on a raw Calcpad line (a `>>` line or a calcpad fence): comments, directives, commands and units are skipped. */
export function calcpadNames(text: string, from = 0): NameTok[] {
    const out: NameTok[] = [];
    let i = from;
    let prevNum = false;
    while (i < text.length) {
        const c = text[i];
        if (c === "'" || c === '"') break;
        if (c === ' ' || c === '\t') { i++; continue; }
        if (/\d/.test(c)) {
            const m = /^\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/.exec(text.slice(i));
            i += m ? m[0].length : 1;
            prevNum = true;
            continue;
        }
        if (c === '#' || c === '$') {
            let j = i + 1;
            while (j < text.length && /[\p{L}]/u.test(text[j])) j++;
            i = j;
            prevNum = false;
            continue;
        }
        if (isNameStart(c)) {
            let j = i + 1;
            while (j < text.length && isNameChar(text[j])) j++;
            if (text[j] === '$') j++;
            if (!prevNum) out.push({ text: text.slice(i, j), start: i, end: j });
            prevNum = false;
            i = j;
            continue;
        }
        prevNum = false;
        i++;
    }
    return out;
}

/** The names a Python block hands to the worksheet with `export(...)` and `export_data(...)`. */
export function pythonExportNames(lines: string[], first: number, last: number): { line: number; tok: NameTok }[] {
    const out: { line: number; tok: NameTok }[] = [];
    let depth = 0;
    let prev = '';
    let firstArg = false;
    for (let ln = first; ln <= last; ln++) {
        const text = lines[ln];
        let i = 0;
        while (i < text.length) {
            const c = text[i];
            if (c === '#') break;
            if (c === '"' || c === "'") {
                let j = i + 1;
                while (j < text.length && text[j] !== c) j += text[j] === '\\' ? 2 : 1;
                const inner = text.slice(i + 1, Math.min(j, text.length));
                if (depth === 1 && firstArg && prev === '(' && j < text.length && /^[\p{L}][\p{L}\p{N}_]*$/u.test(inner)) {
                    out.push({ line: ln, tok: { text: inner, start: i + 1, end: i + 1 + inner.length } });
                }
                firstArg = false;
                prev = c;
                i = Math.min(j + 1, text.length);
                continue;
            }
            if (/[\p{L}_]/u.test(c)) {
                let j = i + 1;
                while (j < text.length && /[\p{L}\p{N}_]/u.test(text[j])) j++;
                const word = text.slice(i, j);
                if (depth === 0 && (word === 'export' || word === 'export_data') && /^\s*\(/.test(text.slice(j)) && !/[\w.]/.test(text[i - 1] ?? '')) {
                    // The call opens at the next `(`.
                    const open = text.indexOf('(', j);
                    depth = 1; prev = '('; firstArg = true;
                    i = open + 1;
                    continue;
                }
                if (depth === 1 && (prev === '(' || prev === ',') && /^\s*=(?!=)/.test(text.slice(j))) {
                    out.push({ line: ln, tok: { text: word, start: i, end: j } });
                }
                prev = 'w';
                firstArg = false;
                i = j;
                continue;
            }
            if (c === '(' || c === '[' || c === '{') { if (depth > 0) depth++; prev = c; firstArg = false; }
            else if (c === ')' || c === ']' || c === '}') { if (depth > 0) depth--; prev = c; firstArg = false; }
            else if (c === ',') { prev = ','; firstArg = false; }
            else if (!/\s/.test(c)) { prev = c; firstArg = false; }
            i++;
        }
    }
    return out;
}

/** The common Greek spellings, for comparing a typed name with the sheet's spelling of one (phi is the letter). */
const GREEK_SPELLINGS: Record<string, string> = {
    alpha: 'α', beta: 'β', gamma: 'γ', delta: 'δ', epsilon: 'ε', zeta: 'ζ', eta: 'η',
    theta: 'θ', iota: 'ι', kappa: 'κ', lambda: 'λ', mu: 'μ', nu: 'ν', xi: 'ξ',
    pi: 'π', rho: 'ρ', sigma: 'σ', tau: 'τ', upsilon: 'υ', phi: 'φ', chi: 'χ',
    psi: 'ψ', omega: 'ω',
};

/** A sheet name with each whole Greek spelling between underscores written as its letter: `sigma_c` is `σ_c`. */
export function canonicalSheetName(name: string): string {
    return name.split('_').map(seg => GREEK_SPELLINGS[seg] ?? seg).join('_');
}
