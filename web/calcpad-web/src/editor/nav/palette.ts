// Variable explorer quick recall: Ctrl+; (Cmd+; on a Mac) opens a search box over every variable,
// function and macro of the document with its last calculated value and unit. Type to filter
// (letters in order are enough: `sigc` finds `sigma_c`), Up and Down choose, Enter inserts the name at
// the caret (a function goes in as `name()` with the caret inside), Alt+Enter jumps to where it is
// defined, Esc closes. Built-in functions are listed too once something is typed. Tapping a row
// inserts it; the arrow button on a row goes to its definition.
//
// The box sits at the top of the window, so an on-screen keyboard never covers it.

import * as monaco from 'monaco-editor';
import { getActiveDocumentKey } from '../bridge';
import { builtins, signature } from '../completions';
import { extractFunctionName } from '../builtin-docs';
import { compileSheetNow } from '../../wasm/sheet';
import { buildIndex, isSheetModel } from './index-builder';
import type { NavCtx } from './index-builder';
import { cachedValues, formatValue, loadValues } from './values';
import type { VarValue } from './values';

export interface PaletteItem {
    name: string;
    kind: 'variable' | 'function' | 'macro' | 'built-in';
    /** Parameter list, `x; y`. */
    params?: string;
    expr?: string;
    /** 0 based definition position; line -1 when it is not in this document. */
    line: number;
    col: number;
    /** The name the engine reports a value under (Calcpad spelling). */
    valueKey: string;
    desc?: string;
}

/** The names of the open document, from the engine (Calcpad) or the sheet scan. */
export async function collectSymbols(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx): Promise<PaletteItem[]> {
    const model = editor.getModel();
    if (!model) return [];
    const out: PaletteItem[] = [];
    if (isSheetModel(model)) {
        const idx = await buildIndex(nav, model);
        if (!idx) return out;
        const c = compileSheetNow(model.getValue());
        const spelled = new Map((c?.defs ?? []).map(d => [d.name, d.calcpad]));
        for (const s of idx.syms.values()) {
            if (s.local || s.defLine < 0) continue;
            const text = model.getLineContent(s.defLine + 1);
            const m = /^\s*[\p{L}][\p{L}\p{N}_′″₀-₉]*(?:\(([^)]*)\))?\s*(?::|=(?!=))\s*(.*)$/u.exec(text);
            const params = m?.[1];
            const note = /(?<!\|)\|(?!\|)\s*(.*)$/.exec(m?.[2] ?? '')?.[1];
            out.push({
                name: s.name, kind: params !== undefined || s.kind === 'function' ? 'function' : 'variable',
                params: params?.trim(), expr: (m?.[2] ?? '').replace(/\s*\|.*$/, '').replace(/\s*->.*$/, '').trim(),
                line: s.defLine, col: s.defCol, valueKey: spelled.get(s.name) ?? s.name, desc: note,
            });
        }
    } else {
        const text = model.getValue();
        const ctx = nav.getFileContext ? await nav.getFileContext(text).catch(() => ({} as { sourceFilePath?: string })) : {};
        const defs = await nav.bridge.definitions.refreshDefinitions(text, getActiveDocumentKey(), ctx.sourceFilePath)
            ?? nav.bridge.definitions.getCachedDefinitions(getActiveDocumentKey());
        const seen = new Set<string>();
        const colOf = (n: number, name: string) => Math.max(0, (model.getLineContent(n + 1) ?? '').indexOf(name));
        const add = (p: PaletteItem) => { if (!seen.has(p.name)) { seen.add(p.name); out.push(p); } };
        for (const v of defs?.variables ?? []) {
            const local = v.source === 'local';
            add({ name: v.name, kind: 'variable', expr: v.expression, line: local ? v.lineNumber : -1, col: local ? colOf(v.lineNumber, v.name) : 0, valueKey: v.name, desc: v.description });
        }
        for (const f of defs?.functions ?? []) {
            const local = f.source === 'local';
            add({ name: f.name, kind: 'function', params: f.parameters.join('; '), expr: f.expression, line: local ? f.lineNumber : -1, col: local ? colOf(f.lineNumber, f.name) : 0, valueKey: f.name, desc: f.description });
        }
        for (const m of defs?.macros ?? []) {
            const local = m.source === 'local';
            add({ name: m.name, kind: 'macro', params: m.parameters.join('; '), line: local ? m.lineNumber : -1, col: local ? colOf(m.lineNumber, m.name) : 0, valueKey: m.name, desc: m.description });
        }
    }
    return out.sort((a, b) => (a.line < 0 ? 1e9 : a.line) - (b.line < 0 ? 1e9 : b.line));
}

/** Letters in order, scored: a prefix and word starts count most, a gap costs. -1 when it does not match. */
export function fuzzy(query: string, text: string): { score: number; hits: number[] } | null {
    if (!query) return { score: 0, hits: [] };
    const q = query.toLowerCase();
    const t = text.toLowerCase();
    const hits: number[] = [];
    let score = 0;
    let ti = 0;
    let last = -2;
    for (const ch of q) {
        const at = t.indexOf(ch, ti);
        if (at < 0) return null;
        hits.push(at);
        const boundary = at === 0 || /[_\s.′]/.test(t[at - 1]) || (text[at] !== text[at].toLowerCase() && text[at - 1] === text[at - 1]?.toLowerCase());
        score += (at === last + 1 ? 6 : 1) + (boundary ? 4 : 0) - Math.min(at - ti, 4) * 0.5;
        last = at;
        ti = at + 1;
    }
    if (t.startsWith(q)) score += 20;
    else if (t.includes(q)) score += 8;
    score -= text.length * 0.05;
    return { score, hits };
}

interface Row { item: PaletteItem; hits: number[]; score: number }

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, text?: string): HTMLElementTagNameMap[K] {
    const e = document.createElement(tag);
    e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
}

let current: { close(): void; editor: monaco.editor.IStandaloneCodeEditor } | null = null;

/** True while the palette is open. */
export function paletteIsOpen(): boolean {
    return current !== null;
}

export function closePalette(): void {
    current?.close();
}

/** Opens (or closes, when open) the palette over the editor's document. */
export async function togglePalette(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx): Promise<void> {
    if (current) { current.close(); return; }
    const model = editor.getModel();
    if (!model) return;

    const backdrop = el('div', 'ck-pal-backdrop');
    const dom = el('div', 'ck-nav ck-palette');
    dom.setAttribute('role', 'dialog');
    dom.setAttribute('aria-label', 'Variables and functions');
    const input = el('input', 'ck-nav-input ck-pal-input');
    input.type = 'text';
    input.placeholder = 'Find a variable or function';
    input.spellcheck = false;
    input.autocomplete = 'off';
    input.setAttribute('autocapitalize', 'off');
    input.setAttribute('role', 'combobox');
    input.setAttribute('aria-expanded', 'true');
    input.setAttribute('aria-controls', 'ck-pal-list');
    input.setAttribute('aria-label', 'Find a variable or function');
    const list = el('div', 'ck-pal-list');
    list.id = 'ck-pal-list';
    list.setAttribute('role', 'listbox');
    const foot = el('div', 'ck-pal-foot');
    dom.append(input, list, foot);
    backdrop.append(dom);
    document.body.append(backdrop);

    const sheet = isSheetModel(model);
    let items: PaletteItem[] = [];
    let rows: Row[] = [];
    let sel = 0;
    let values: Map<string, VarValue> | null = cachedValues(model.uri.toString());
    let calculating = false;
    let closed = false;

    const close = () => {
        if (closed) return;
        closed = true;
        backdrop.remove();
        current = null;
        editor.focus();
    };
    current = { close, editor };

    const builtinItems = (): PaletteItem[] => {
        const sets = builtins(nav.bridge);
        const out: PaletteItem[] = [];
        for (const f of sets?.functions ?? []) {
            const name = extractFunctionName(f.tag);
            if (!name) continue;
            out.push({ name, kind: 'built-in', params: signature(f, sheet).slice(1, -1), line: -1, col: 0, valueKey: '', desc: f.description });
        }
        return out;
    };
    let builtinsList: PaletteItem[] | null = null;

    const compute = () => {
        const q = input.value.trim();
        const pool = q ? [...items, ...(builtinsList ??= builtinItems())] : items;
        rows = [];
        for (const item of pool) {
            const m = fuzzy(q, item.name);
            if (m) rows.push({ item, hits: m.hits, score: m.score + (item.kind === 'built-in' ? -100 : 0) });
        }
        // Without a query: the order of the document. With one: best match first.
        if (q) rows.sort((a, b) => b.score - a.score);
        rows = rows.slice(0, 200);
        if (sel >= rows.length) sel = Math.max(0, rows.length - 1);
    };

    const markName = (name: string, hits: number[]): DocumentFragment => {
        const f = document.createDocumentFragment();
        const set = new Set(hits);
        let run = '';
        let on = false;
        const flush = () => {
            if (!run) return;
            if (on) { const m = document.createElement('mark'); m.textContent = run; f.append(m); } else f.append(run);
            run = '';
        };
        [...name].forEach((ch, i) => {
            const h = set.has(i);
            if (h !== on) { flush(); on = h; }
            run += ch;
        });
        flush();
        return f;
    };

    const render = () => {
        list.textContent = '';
        if (!rows.length) {
            list.append(el('div', 'ck-pal-empty', items.length || input.value ? 'Nothing matches.' : 'No variables or functions defined yet.'));
        }
        rows.forEach((r, i) => {
            const row = el('div', `ck-pal-row${i === sel ? ' ck-sel' : ''}`);
            row.id = `ck-pal-${i}`;
            row.setAttribute('role', 'option');
            row.setAttribute('aria-selected', i === sel ? 'true' : 'false');
            const main = el('div', 'ck-pal-main');
            const name = el('span', 'ck-pal-name');
            name.append(markName(r.item.name, r.hits));
            if (r.item.kind === 'function' || r.item.kind === 'macro' || r.item.kind === 'built-in') name.append(el('span', 'ck-pal-sig', `(${r.item.params ?? ''})`));
            const val = values?.get(r.item.valueKey);
            const valText = r.item.kind === 'variable' || r.item.kind === 'function' ? formatValue(val) : '';
            const v = el('span', 'ck-pal-val', r.item.kind === 'variable' ? (valText || (calculating ? '…' : '')) : '');
            if (val?.input) v.title = 'An input';
            main.append(name, v);
            const meta = el('div', 'ck-pal-meta');
            const where = r.item.kind === 'built-in' ? 'built in' : r.item.line >= 0 ? `${r.item.kind}, line ${r.item.line + 1}` : `${r.item.kind}, from an included file`;
            const text = r.item.desc || (r.item.expr ? `= ${r.item.expr}` : '');
            meta.textContent = text ? `${where} · ${text}` : where;
            const goBtn = el('button', 'ck-pal-go', '→');
            goBtn.type = 'button';
            goBtn.title = 'Go to definition';
            goBtn.setAttribute('aria-label', `Go to definition of ${r.item.name}`);
            goBtn.disabled = r.item.line < 0;
            row.append(main, meta, goBtn);
            row.addEventListener('mousedown', e => e.preventDefault());
            row.addEventListener('click', e => {
                if ((e.target as HTMLElement).closest('.ck-pal-go')) { jump(r.item); return; }
                if (e.altKey) jump(r.item); else insert(r.item);
            });
            list.append(row);
        });
        const active = list.querySelector('.ck-sel');
        active?.scrollIntoView({ block: 'nearest' });
        input.setAttribute('aria-activedescendant', rows.length ? `ck-pal-${sel}` : '');
        foot.textContent = calculating
            ? 'Calculating values…'
            : values ? 'Last calculated values · Enter inserts · Alt+Enter goes to the definition · Esc closes'
                : 'Enter inserts · Alt+Enter goes to the definition · Esc closes';
    };

    const insert = (item: PaletteItem) => {
        close();
        const sel0 = editor.getSelection();
        if (!sel0) return;
        const call = item.kind !== 'variable';
        const text = call ? `${item.name}()` : item.name;
        editor.executeEdits('calcpad-palette', [{ range: sel0, text }]);
        const end = editor.getModel()!.getPositionAt(editor.getModel()!.getOffsetAt(sel0.getStartPosition()) + (call ? text.length - 1 : text.length));
        editor.setPosition(end);
        editor.focus();
    };
    const jump = (item: PaletteItem) => {
        if (item.line < 0) return;
        close();
        editor.setPosition({ lineNumber: item.line + 1, column: item.col + 1 });
        editor.revealLineInCenter(item.line + 1);
        editor.focus();
    };

    input.addEventListener('input', () => { sel = 0; compute(); render(); });
    input.addEventListener('keydown', e => {
        if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); return; }
        if ((e.ctrlKey || e.metaKey) && e.key === ';') { e.preventDefault(); close(); return; }
        if (e.key === 'ArrowDown') { e.preventDefault(); if (rows.length) sel = (sel + 1) % rows.length; render(); return; }
        if (e.key === 'ArrowUp') { e.preventDefault(); if (rows.length) sel = (sel - 1 + rows.length) % rows.length; render(); return; }
        if (e.key === 'PageDown') { e.preventDefault(); sel = Math.min(rows.length - 1, sel + 6); render(); return; }
        if (e.key === 'PageUp') { e.preventDefault(); sel = Math.max(0, sel - 6); render(); return; }
        if (e.key === 'Enter') {
            e.preventDefault();
            const r = rows[sel];
            if (!r) return;
            if (e.altKey) jump(r.item); else insert(r.item);
        }
    });
    backdrop.addEventListener('mousedown', e => { if (e.target === backdrop) close(); });

    items = [];
    compute();
    render();
    input.focus();

    items = await collectSymbols(editor, nav);
    if (closed) return;
    compute();
    render();

    // Values: shown at once when there are earlier ones, refreshed when the text has moved on.
    const key = model.uri.toString();
    calculating = true;
    render();
    const text = model.getValue();
    const ctx = nav.getFileContext ? await nav.getFileContext(text).catch(() => ({} as { sourceFilePath?: string })) : {};
    const got = await loadValues(key, text, ctx.sourceFilePath, 4000, true);
    calculating = false;
    if (closed) return;
    values = got;
    render();
}
