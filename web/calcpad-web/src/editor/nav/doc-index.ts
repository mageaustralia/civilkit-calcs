// Where every name of a document is defined and used, with its scope. Rename and the quick-recall
// palette read it; completion reads it for sheets.
//
// A Calcpad worksheet is classified by the engine's own highlighter (the same tokens that colour the
// editor): a Variable token is a global name, a LocalVariable token belongs to the line it is on (a
// function's parameters, the bound name of `$Sum{...}`), a macro's parameters belong to its `#def`
// block, and comments, text lines, strings and the bodies of #markdown/#html blocks produce no
// name tokens at all. A sheet is scanned here, line by line, with the kinds the sheet compiler
// gives each line (prose and blank lines are never code). Python blocks are never renamed except
// for the names they `export`, which are Calcpad variables below the block.

import type { HighlightToken } from 'calcpad-frontend/types/api';
import type { SheetCompiled } from '../../wasm/sheet';
import { calcpadNames, pythonExportNames, sheetNames } from './lexer';
import type { NameTok } from './lexer';

export type SymKind = 'variable' | 'function' | 'macro';

export interface Occ {
    /** 0 based line and column range. */
    line: number;
    start: number;
    end: number;
    /** The symbol's name (a sheet's own spelling). */
    name: string;
    sym: string;
    /** Where the name is defined (an assignment, a parameter list, a loop header, an export). */
    def: boolean;
    /** Written as the key of a Python block's `export(...)`. */
    exported?: boolean;
}

export interface Sym {
    id: string;
    name: string;
    kind: SymKind;
    /** A function's or macro's parameter, or a name bound inside one expression: it is renamed only within `scope`. */
    local: boolean;
    /** The first and last 0 based line of a local name's scope. */
    scope?: [number, number];
    params?: string[];
    occs: Occ[];
    defLine: number;
    defCol: number;
}

export interface DocIndex {
    lang: 'calcpad' | 'sheet';
    syms: Map<string, Sym>;
    occs: Occ[];
}

class Builder {
    syms = new Map<string, Sym>();
    occs: Occ[] = [];

    add(name: string, kind: SymKind, line: number, start: number, end: number, def: boolean, local?: { id: string; scope: [number, number] }, exported = false): void {
        const id = local ? local.id : `g:${name}`;
        let s = this.syms.get(id);
        if (!s) {
            s = { id, name, kind, local: !!local, scope: local?.scope, occs: [], defLine: -1, defCol: -1 };
            this.syms.set(id, s);
        }
        // A name that is called as a function somewhere is a function; a macro stays a macro.
        if (s.kind === 'variable' && kind !== 'variable') s.kind = kind;
        const occ: Occ = { line, start, end, name, sym: id, def, exported: exported || undefined };
        s.occs.push(occ);
        this.occs.push(occ);
        if (def && s.defLine < 0) { s.defLine = line; s.defCol = start; }
    }

    done(lang: 'calcpad' | 'sheet'): DocIndex {
        return { lang, syms: this.syms, occs: this.occs };
    }
}

/** The occurrence the cursor is on or touches (0 based position). */
export function occAt(idx: DocIndex, line: number, col: number): Occ | undefined {
    let touch: Occ | undefined;
    for (const o of idx.occs) {
        if (o.line !== line) continue;
        if (col > o.start && col < o.end) return o;
        if (col === o.start || col === o.end) touch ??= o;
    }
    return touch;
}

// ---------------------------------------------------------------------------------------------
// Calcpad worksheets
// ---------------------------------------------------------------------------------------------

function pythonBlocksOf(lines: string[]): [number, number][] {
    const out: [number, number][] = [];
    let open = -1;
    for (let i = 0; i < lines.length; i++) {
        const t = lines[i].trim().toLowerCase();
        if (open < 0 && t === '#python') open = i + 1;
        else if (open >= 0 && /^#end\s+python\b/.test(t)) { out.push([open, i - 1]); open = -1; }
    }
    if (open >= 0) out.push([open, lines.length - 1]);
    return out;
}

/** Scans a Calcpad worksheet from the highlighter's tokens (requested with their text). */
export function scanCalcpad(lines: string[], tokens: HighlightToken[]): DocIndex {
    const b = new Builder();
    const byLine = new Map<number, HighlightToken[]>();
    for (const t of tokens) {
        let l = byLine.get(t.line);
        if (!l) byLine.set(t.line, l = []);
        l.push(t);
    }
    for (const l of byLine.values()) l.sort((a, c) => a.column - c.column);
    const text = (t: HighlightToken | undefined) => t?.text ?? '';

    // Pass 1: user functions (`f(x) = ...` at the start of a line) and macro blocks.
    const userFuncs = new Set<string>();
    const funcLines = new Set<number>();
    interface MacroDef { name: string; line: number; last: number; params: Set<string> }
    const macros: MacroDef[] = [];
    for (const [line, toks] of byLine) {
        const t0 = toks[0];
        if (t0.type === 'Function' && text(toks[1]) === '(' && toks.some((x, k) => x.type === 'Operator' && x.text === '=' && text(toks[k - 1]) === ')')) {
            userFuncs.add(t0.text ?? '');
            funcLines.add(line);
        }
        const k0 = toks.findIndex(x => x.type === 'Keyword' && /^#def$/i.test(x.text ?? ''));
        if (k0 >= 0) {
            const nameTok = toks[k0 + 1];
            if (nameTok && (nameTok.type === 'Macro' || nameTok.type === 'Variable')) {
                const params = new Set<string>();
                let open = false;
                for (let k = k0 + 2; k < toks.length; k++) {
                    const x = toks[k];
                    if (x.type === 'Bracket' && x.text === '(') open = true;
                    else if (x.type === 'Bracket' && x.text === ')') open = false;
                    else if (open && (x.type === 'MacroParameter' || x.type === 'Macro' || x.type === 'Variable')) params.add(x.text ?? '');
                }
                const inline = toks.some((x, k) => k > k0 + 1 && x.type === 'Operator' && x.text === '=' && (text(toks[k - 1]) === ')' || k === k0 + 2));
                let last = line;
                if (!inline) {
                    last = lines.length - 1;
                    for (const [l2, t2] of byLine) {
                        if (l2 > line && t2.some(x => (x.type === 'EndKeyword' || x.type === 'Keyword') && /^#end\s+def$/i.test(x.text ?? ''))) { last = Math.min(last, l2); }
                    }
                }
                macros.push({ name: nameTok.text ?? '', line, last, params });
            }
        }
    }
    const macroNames = new Set(macros.map(m => m.name));
    const macroScope = (line: number, name: string) => macros.find(m => line >= m.line && line <= m.last && m.params.has(name));

    // Pass 2: every name.
    for (const [line, toks] of [...byLine].sort((a, c) => a[0] - c[0])) {
        for (let k = 0; k < toks.length; k++) {
            const t = toks[k];
            const name = t.text ?? '';
            if (!name) continue;
            const end = t.column + t.length;
            const next = toks[k + 1];
            const assign = next?.type === 'Operator' && next.text === '=' && !(toks[k + 2]?.type === 'Operator' && toks[k + 2].text === '=' && toks[k + 2].column === next.column + 1);
            switch (t.type) {
                case 'Variable': {
                    if (macroNames.has(name)) {
                        b.add(name, 'macro', line, t.column, end, false);
                    } else {
                        b.add(name, 'variable', line, t.column, end, assign);
                    }
                    break;
                }
                case 'Function':
                    if (userFuncs.has(name)) b.add(name, 'function', line, t.column, end, funcLines.has(line) && k === 0);
                    break;
                case 'LocalVariable':
                    b.add(name, 'variable', line, t.column, end, false, { id: `l${line}:${name}`, scope: [line, line] });
                    break;
                case 'MacroParameter':
                case 'Macro': {
                    const m = macroScope(line, name);
                    if (m && !(t.type === 'Macro' && m.name === name)) {
                        b.add(name, 'variable', line, t.column, end, line === m.line && k > 0, { id: `m${m.line}:${name}`, scope: [m.line, m.last] });
                    } else if (macroNames.has(name)) {
                        b.add(name, 'macro', line, t.column, end, macros.some(d => d.name === name && d.line === line));
                    }
                    break;
                }
                default:
                    break;
            }
        }
    }
    // Names a Python block exports are Calcpad variables below it.
    for (const [first, last] of pythonBlocksOf(lines)) {
        for (const e of pythonExportNames(lines, first, last)) b.add(e.tok.text, 'variable', e.line, e.tok.start, e.tok.end, true, undefined, true);
    }
    // Parameters get their parameter list.
    for (const s of b.syms.values()) {
        if (s.kind === 'function') {
            // `f(a; b) = ...`: the local names on the defining line, in order.
            const defOcc = s.occs.find(o => o.def);
            if (defOcc) {
                s.params = [...b.syms.values()].filter(p => p.local && p.scope![0] === defOcc.line).sort((x, y) => x.occs[0].start - y.occs[0].start).map(p => p.name);
            }
        }
    }
    return b.done('calcpad');
}

// ---------------------------------------------------------------------------------------------
// Sheets
// ---------------------------------------------------------------------------------------------

/** The start offsets of the statements of a quantity line (split at `;` outside brackets and strings). */
function statementStarts(text: string): number[] {
    const out = [0];
    let depth = 0;
    let inStr = false;
    for (let i = 0; i < text.length; i++) {
        const c = text[i];
        if (c === '"') inStr = !inStr;
        if (inStr) continue;
        if (c === '(' || c === '[') depth++;
        else if (c === ')' || c === ']') depth = Math.max(0, depth - 1);
        else if (c === '|' && text[i + 1] !== '|' && text[i - 1] !== '|') break;
        else if (c === ';' && depth === 0) out.push(i + 1);
    }
    return out;
}

const STMT_HEAD = /^(\s*)([\p{L}][\p{L}\p{N}_′″‴⁗₀-₉]*)(?:\(([^)]*)\))?\s*(?::|=(?!=))/u;

/** Scans a sheet with the compiler's line kinds and definitions. */
export function scanSheet(lines: string[], c: SheetCompiled): DocIndex {
    const b = new Builder();
    const defNames = new Set(c.defs.map(d => d.name));
    const spelled = new Map(c.defs.map(d => [d.calcpad, d.name]));
    const resolve = (w: string): string | undefined => (defNames.has(w) ? w : spelled.get(w));
    // Names a `for` header binds are variables like any other.
    const loopVars = new Set<string>();
    for (let i = 0; i < lines.length; i++) {
        if ((c.kinds[i] ?? '') === 'k') {
            const m = /^\s*for\s+([\p{L}][\p{L}\p{N}_′″]*)\s+in\b/u.exec(lines[i]);
            if (m) loopVars.add(m[1]);
        }
    }
    const funcs = new Set<string>();
    for (const l of lines) {
        const m = STMT_HEAD.exec(l);
        if (m && m[3] !== undefined) funcs.add(m[2]);
    }

    // A first pass over the fences: Python blocks, whose exported names are sheet variables.
    const pyRanges: [number, number][] = [];
    {
        let f0: string | null = null;
        let from = 0;
        for (let i = 0; i < lines.length; i++) {
            const m = /^\s*```\s*([\w-]*)\s*$/.exec(lines[i]);
            if (f0 !== null) {
                if (m && m[1] === '') { if (f0 === 'python') pyRanges.push([from + 1, i - 1]); f0 = null; }
            } else if (m) { f0 = m[1].toLowerCase(); from = i; }
        }
    }
    const exportNames = new Set<string>();
    for (const [first, last] of pyRanges) for (const e of pythonExportNames(lines, first, last)) exportNames.add(e.tok.text);
    const known = (w: string): string | undefined => resolve(w) ?? (loopVars.has(w) || exportNames.has(w) ? w : undefined);
    let fence: string | null = null;
    let inMath = false;
    for (let i = 0; i < lines.length; i++) {
        const text = lines[i];
        const f = /^\s*```\s*([\w-]*)\s*$/.exec(text);
        if (fence !== null) {
            if (f && f[1] === '') {
                fence = null;
                continue;
            }
            if (fence === 'calcpad') {
                for (const t of calcpadNames(text)) emitRaw(i, t);
            }
            continue;
        }
        if (f) { fence = f[1].toLowerCase(); continue; }
        if (/^\s*\$\$\s*$/.test(text)) { inMath = !inMath; continue; }
        if (inMath) continue;
        const kind = c.kinds[i] ?? 'b';
        if (kind === 'r') {
            const from = /^\s*>>\s?/.exec(text)?.[0].length ?? 0;
            for (const t of calcpadNames(text, from)) emitRaw(i, t);
            continue;
        }
        if (!'qkcg'.includes(kind)) continue;
        // A one-line input: the right side after `?` is a default, a list of labels and a note, never names.
        const isInput = kind === 'q' && /^\s*[\p{L}][\p{L}\p{N}_′″]*\s*(?::|=(?!=))\s*\?/u.test(text);
        const toks = isInput ? sheetNames(text.slice(0, text.indexOf('?')), { note: false }) : sheetNames(text, { note: kind === 'q' });
        const starts = kind === 'q' ? statementStarts(text) : [0];
        // The names a statement's parameter list binds, for that statement only.
        const locals: { names: Set<string>; from: number; to: number; id: string }[] = [];
        if (kind === 'q') {
            starts.forEach((s, k) => {
                const m = STMT_HEAD.exec(text.slice(s));
                if (m && m[3] !== undefined) {
                    const names = new Set(m[3].split(',').map(x => x.trim()).filter(Boolean));
                    locals.push({ names, from: s, to: starts[k + 1] ?? text.length, id: `l${i}.${s}` });
                }
            });
        }
        let plotVar: string | null = null;
        if (kind === 'g') plotVar = /\bfor\s+([\p{L}][\p{L}\p{N}_′″]*)\s+in\b/u.exec(text)?.[1] ?? null;
        for (const t of toks) {
            const local = locals.find(l => t.start >= l.from && t.start < l.to && l.names.has(t.text));
            if (local) {
                b.add(t.text, 'variable', i, t.start, t.end, false, { id: `${local.id}:${t.text}`, scope: [i, i] });
                continue;
            }
            if (plotVar && t.text === plotVar) {
                b.add(t.text, 'variable', i, t.start, t.end, false, { id: `l${i}:${t.text}`, scope: [i, i] });
                continue;
            }
            const name = known(t.text);
            if (!name) continue;
            const stmtStart = starts.filter(s => s <= t.start).pop() ?? 0;
            const head = kind === 'q' && t.start === stmtStart + (STMT_HEAD.exec(text.slice(stmtStart))?.[1].length ?? 0) && STMT_HEAD.test(text.slice(stmtStart));
            const forVar = kind === 'k' && loopVars.has(name) && new RegExp(`^\\s*for\\s+${t.text}\\s+in\\b`, 'u').test(text) && t.start === text.indexOf(t.text);
            b.add(name, funcs.has(name) ? 'function' : 'variable', i, t.start, t.end, head || forVar);
        }
    }
    for (const [first, last] of pyRanges) {
        for (const e of pythonExportNames(lines, first, last)) {
            const name = known(e.tok.text) ?? e.tok.text;
            b.add(name, 'variable', e.line, e.tok.start, e.tok.end, true, undefined, true);
        }
    }
    function emitRaw(line: number, t: NameTok): void {
        const name = known(t.text);
        if (name) b.add(name, 'variable', line, t.start, t.end, false);
    }
    for (const s of b.syms.values()) if (funcs.has(s.name) && !s.local) s.kind = 'function';
    return b.done('sheet');
}
