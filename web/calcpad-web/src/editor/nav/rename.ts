// Rename a variable, function or macro everywhere it is used in the document (F2, the editor's
// context menu, or the Variables tab's context menu).
//
// What is renamed: every use of the name in code, with scope. A function's parameters and the name
// bound inside `$Sum{...}` are renamed only on their own line, a macro's parameters only inside
// its `#def` block, so a parameter `a` is not confused with a variable `a`. In a sheet the unit
// after a number (`3 psi`), strings, notes after `|`, prose, formulas and fenced code other than
// raw Calcpad are left alone.
//
// What is not: comments and text lines, strings, Markdown/HTML blocks and sheet prose (including
// `$...$` formulas, which are typeset text, not a reference to a variable). A Python block is
// text to the renamer too, with one exception: the names it hands to the worksheet with
// `export(name=...)` / `export("name", ...)` ARE the sheet's variables, so they are renamed with
// it. A `?` input is an ordinary definition (`L = ?`, `L: ? 6 m`) and is renamed with its name.
//
// The rename is one edit (one undo step). It shows how many places change, and refuses a name that
// is not valid, is a keyword or built-in, is already in use, or would be captured by (or capture) a
// parameter of the same name. A name defined in an `#include`d file is refused: only this
// document is edited.

import * as monaco from 'monaco-editor';
import type { InsertItem } from 'calcpad-frontend/types/snippets';
import { getActiveDocumentKey } from '../bridge';
import { occAt } from './doc-index';
import type { DocIndex, Occ, Sym } from './doc-index';
import { buildIndex, isSheetModel } from './index-builder';
import type { NavCtx } from './index-builder';
import { SHEET_KEYWORDS, canonicalSheetName } from './lexer';
import { extractFunctionName } from '../builtin-docs';

export interface RenamePlan {
    sym: Sym;
    occs: Occ[];
    lines: number;
    exported: number;
    versionId: number;
    index: DocIndex;
    sheet: boolean;
    /** Names defined in #included files (Calcpad), which this document cannot rename. */
    external: Set<string>;
    reserved: Set<string>;
    units: Set<string>;
}

export interface RenameRefusal { error: string }

const VALID = /^[\p{L}][\p{L}\p{N}_′″‴⁗₀-₉]*$/u;

function reservedNames(nav: NavCtx, sheet: boolean): { reserved: Set<string>; units: Set<string> } {
    const reserved = new Set<string>();
    const units = new Set<string>();
    if (sheet) for (const k of SHEET_KEYWORDS) reserved.add(k);
    if (nav.bridge.snippets.isLoaded()) {
        for (const it of nav.bridge.snippets.getAllItems() as InsertItem[]) {
            if (it.keywordType === 'Function') { const n = extractFunctionName(it.tag); if (n) reserved.add(n); }
            else if (it.keywordType === 'Constant') reserved.add(it.tag);
            else if (it.keywordType === 'Unit') units.add(it.tag);
        }
    }
    return { reserved, units };
}

/** Works out what renaming the name at a 0 based position would change. */
export async function planRename(nav: NavCtx, model: monaco.editor.ITextModel, line0: number, col0: number, byName?: string): Promise<RenamePlan | RenameRefusal> {
    const versionId = model.getVersionId();
    const index = await buildIndex(nav, model);
    if (!index) return { error: 'The editor could not read this document just now. Try again in a moment.' };
    if (model.getVersionId() !== versionId) return { error: 'The document changed while it was being read. Try again.' };
    let occ: Occ | undefined;
    if (byName !== undefined) {
        const syms = [...index.syms.values()].filter(s => !s.local && (s.name === byName || canonicalSheetName(s.name) === byName));
        const s = syms.find(x => x.defLine >= 0) ?? syms[0];
        occ = s?.occs.find(o => o.def) ?? s?.occs[0];
    } else {
        occ = occAt(index, line0, col0);
    }
    if (!occ) return { error: 'Put the cursor on the name of a variable, function or macro to rename it.' };
    const sym = index.syms.get(occ.sym)!;
    const sheet = isSheetModel(model);
    const defs = sheet ? undefined : nav.bridge.definitions.getCachedDefinitions(getActiveDocumentKey());
    const external = new Set<string>();
    for (const list of [defs?.variables, defs?.functions, defs?.macros, defs?.customUnits] as ({ name: string; source: string }[] | undefined)[]) {
        for (const d of list ?? []) if (d.source !== 'local') external.add(d.name);
    }
    if (!sym.local && external.has(sym.name) && !sym.occs.some(o => o.def)) {
        return { error: `"${sym.name}" is defined in an included file, so it cannot be renamed from here.` };
    }
    if (!sym.local && external.has(sym.name)) {
        return { error: `"${sym.name}" is also defined in an included file, so it cannot be renamed from here.` };
    }
    if (!sym.local && sym.defLine < 0) {
        return { error: `"${sym.name}" is not defined in this document (it is built in, or comes from somewhere this editor cannot change).` };
    }
    const { reserved, units } = reservedNames(nav, sheet);
    return {
        sym, occs: [...sym.occs].sort((a, b) => a.line - b.line || a.start - b.start),
        lines: new Set(sym.occs.map(o => o.line)).size,
        exported: sym.occs.filter(o => o.exported).length,
        versionId, index, sheet, external, reserved, units,
    };
}

export function isRefusal(x: RenamePlan | RenameRefusal): x is RenameRefusal {
    return (x as RenameRefusal).error !== undefined;
}

/** Whether `newName` is acceptable. `error` blocks the rename; `warning` does not. */
export function checkNewName(plan: RenamePlan, newName: string): { error?: string; warning?: string } {
    const { sym } = plan;
    if (newName === '') return { error: 'Type the new name.' };
    if (newName === sym.name) return { error: 'That is the current name.' };
    // A macro, and a macro's parameters, are written with a trailing $ (or, as old worksheets have them, without).
    const wantDollar = sym.name.endsWith('$');
    const body = newName.endsWith('$') ? newName.slice(0, -1) : newName;
    if (!VALID.test(body)) return { error: 'A name starts with a letter and holds letters, digits, _ and primes (x_1, \u03c3_c, M\u2032).' };
    if (newName.endsWith('$') && !wantDollar) return { error: 'Only a macro or a macro parameter has a name ending with $.' };
    if (wantDollar && !newName.endsWith('$')) return { error: 'The name ends with $ (macros and their parameters do).' };
    if (plan.reserved.has(newName)) return { error: `"${newName}" is a keyword or built-in name.` };
    const canon = plan.sheet ? canonicalSheetName(newName) : newName;
    const same = (n: string) => (plan.sheet ? canonicalSheetName(n) : n) === canon;
    const lineOf = (s: Sym) => (s.defLine >= 0 ? ` (line ${s.defLine + 1})` : '');
    if (plan.external.has(newName)) return { error: `"${newName}" is already defined in an included file.` };
    const planLines = new Set(plan.occs.map(o => o.line));
    for (const other of plan.index.syms.values()) {
        if (other.id === sym.id || !same(other.name)) continue;
        if (!sym.local && !other.local) return { error: `"${other.name}" is already used${lineOf(other)}. Renaming would merge the two.` };
        if (!sym.local && other.local) {
            const hit = other.scope && [...planLines].find(l => l >= other.scope![0] && l <= other.scope![1]);
            if (hit !== undefined) return { error: `A parameter named "${other.name}" on line ${hit + 1} would capture it.` };
        } else if (sym.local && !other.local) {
            const hit = other.occs.find(o => o.line >= sym.scope![0] && o.line <= sym.scope![1]);
            if (hit) return { error: `"${other.name}" is used on line ${hit.line + 1}; the parameter would hide it.` };
        } else if (sym.local && other.local && other.scope && sym.scope && other.scope[0] <= sym.scope[1] && other.scope[1] >= sym.scope[0]) {
            return { error: `"${other.name}" is already a parameter there.` };
        }
    }
    if (plan.units.has(newName)) return { warning: `"${newName}" is also a unit name; after a number it still reads as the unit.` };
    return {};
}

/** Applies the rename as one edit. Returns an error text, or null on success. */
export function applyRename(editor: monaco.editor.IStandaloneCodeEditor, plan: RenamePlan, newName: string): string | null {
    const model = editor.getModel();
    if (!model) return 'There is no document.';
    if (model.getVersionId() !== plan.versionId) return 'The document changed after the rename was prepared. Open the rename again.';
    const check = checkNewName(plan, newName);
    if (check.error) return check.error;
    const edits = plan.occs.map(o => ({ range: new monaco.Range(o.line + 1, o.start + 1, o.line + 1, o.end + 1), text: newName }));
    editor.pushUndoStop();
    editor.executeEdits('calcpad-rename', edits);
    editor.pushUndoStop();
    return null;
}

/** The changed lines as they will read, for the preview (first `max`). */
export function previewLines(model: monaco.editor.ITextModel, plan: RenamePlan, newName: string, max = 6): { line: number; before: string; after: string }[] {
    const byLine = new Map<number, Occ[]>();
    for (const o of plan.occs) {
        let l = byLine.get(o.line);
        if (!l) byLine.set(o.line, l = []);
        l.push(o);
    }
    const out: { line: number; before: string; after: string }[] = [];
    for (const [line, occs] of [...byLine].sort((a, b) => a[0] - b[0]).slice(0, max)) {
        const text = model.getLineContent(line + 1);
        let after = text;
        for (const o of [...occs].sort((a, b) => b.start - a.start)) after = after.slice(0, o.start) + newName + after.slice(o.end);
        out.push({ line: line + 1, before: text.trim(), after: after.trim() });
    }
    return out;
}

// ---------------------------------------------------------------------------------------------
// The rename box
// ---------------------------------------------------------------------------------------------

interface Box {
    id: string;
    dom: HTMLElement;
    close(): void;
}

const boxes = new WeakMap<monaco.editor.ICodeEditor, Box>();

function el<K extends keyof HTMLElementTagNameMap>(tag: K, cls: string, text?: string): HTMLElementTagNameMap[K] {
    const e = document.createElement(tag);
    e.className = cls;
    if (text !== undefined) e.textContent = text;
    return e;
}

function plural(n: number, one: string, many: string): string {
    return `${n} ${n === 1 ? one : many}`;
}

/** Opens the rename box on the name at the cursor (or on `byName`, from the Variables tab). */
export async function openRename(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx, byName?: string): Promise<void> {
    boxes.get(editor)?.close();
    const model = editor.getModel();
    const pos = editor.getPosition();
    if (!model || !pos) return;
    const plan = await planRename(nav, model, pos.lineNumber - 1, pos.column - 1, byName);

    const dom = el('div', 'ck-nav ck-rename');
    dom.setAttribute('role', 'dialog');
    dom.setAttribute('aria-label', 'Rename everywhere');
    const id = `calcpad.rename.${Math.random().toString(36).slice(2)}`;
    let decorations: string[] = [];
    let closed = false;
    const close = () => {
        if (closed) return;
        closed = true;
        decorations = editor.deltaDecorations(decorations, []);
        editor.removeOverlayWidget(widget);
        boxes.delete(editor);
        editor.focus();
    };
    const widget: monaco.editor.IOverlayWidget = {
        getId: () => id,
        getDomNode: () => dom,
        getPosition: () => ({ preference: monaco.editor.OverlayWidgetPositionPreference.TOP_RIGHT_CORNER }),
    };
    boxes.set(editor, { id, dom, close });
    editor.addOverlayWidget(widget);

    const key = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); } };
    dom.addEventListener('keydown', key);

    if (isRefusal(plan)) {
        dom.append(el('div', 'ck-nav-title', 'Rename'), el('div', 'ck-nav-msg ck-nav-err', plan.error));
        const row = el('div', 'ck-nav-actions');
        const ok = el('button', 'ck-nav-btn', 'Close');
        ok.type = 'button';
        ok.addEventListener('click', close);
        row.append(ok);
        dom.append(row);
        ok.focus();
        return;
    }

    const kindLabel = plan.sym.local ? (plan.sym.id.startsWith('m') ? 'macro parameter' : 'local name') : plan.sym.kind;
    const title = el('div', 'ck-nav-title');
    title.append('Rename ', el('code', 'ck-nav-name', plan.sym.name), ' ', el('span', 'ck-nav-kind', kindLabel));
    const input = el('input', 'ck-nav-input');
    input.type = 'text';
    input.value = plan.sym.name;
    input.spellcheck = false;
    input.autocomplete = 'off';
    input.setAttribute('autocapitalize', 'off');
    input.setAttribute('aria-label', 'New name');
    const count = el('div', 'ck-nav-count');
    count.setAttribute('aria-live', 'polite');
    const msg = el('div', 'ck-nav-msg');
    msg.setAttribute('role', 'status');
    const list = el('ul', 'ck-nav-preview');
    const scopeNote = plan.sym.local
        ? `Only in its own ${plan.sym.id.startsWith('m') ? 'macro' : 'function'} (line ${plan.sym.scope![0] + 1}${plan.sym.scope![1] !== plan.sym.scope![0] ? ` to ${plan.sym.scope![1] + 1}` : ''}).`
        : 'Everywhere in this document, in code only: not in text, comments, strings or prose.';
    const note = el('div', 'ck-nav-note', scopeNote);
    const row = el('div', 'ck-nav-actions');
    const go = el('button', 'ck-nav-btn ck-nav-primary', 'Rename');
    go.type = 'button';
    const cancel = el('button', 'ck-nav-btn', 'Cancel');
    cancel.type = 'button';
    row.append(go, cancel);
    dom.append(title, input, count, msg, list, note, row);

    const hits = (): monaco.editor.IModelDeltaDecoration[] => plan.occs.map(o => ({
        range: new monaco.Range(o.line + 1, o.start + 1, o.line + 1, o.end + 1),
        options: { className: 'ck-rename-hit', stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges },
    }) as monaco.editor.IModelDeltaDecoration);

    const refresh = () => {
        const name = input.value.trim();
        const check = name === plan.sym.name ? { error: undefined } : checkNewName(plan, name);
        const changing = name !== plan.sym.name;
        count.textContent = `${plural(plan.occs.length, 'place', 'places')} on ${plural(plan.lines, 'line', 'lines')}`
            + (plan.exported ? `, ${plural(plan.exported, 'Python export name', 'Python export names')}` : '');
        msg.textContent = changing ? (check.error ?? check.warning ?? '') : '';
        msg.className = `ck-nav-msg${check.error && changing ? ' ck-nav-err' : check.warning ? ' ck-nav-warn' : ''}`;
        go.disabled = !changing || !!check.error;
        list.textContent = '';
        if (changing && !check.error) {
            for (const l of previewLines(model, plan, name)) {
                const li = el('li', 'ck-nav-prev');
                li.append(el('span', 'ck-nav-ln', String(l.line)), el('span', 'ck-nav-after', l.after));
                list.append(li);
            }
            if (plan.lines > 6) list.append(el('li', 'ck-nav-more', `and ${plan.lines - 6} more lines`));
        }
    };
    const apply = () => {
        const err = applyRename(editor, plan, input.value.trim());
        if (err) { msg.textContent = err; msg.className = 'ck-nav-msg ck-nav-err'; return; }
        close();
    };
    input.addEventListener('input', refresh);
    input.addEventListener('keydown', e => {
        if (e.key === 'Enter') { e.preventDefault(); if (!go.disabled) apply(); }
    });
    go.addEventListener('click', apply);
    cancel.addEventListener('click', close);
    decorations = editor.deltaDecorations(decorations, hits());
    if (byName !== undefined && plan.sym.defLine >= 0) editor.revealLineInCenterIfOutsideViewport(plan.sym.defLine + 1);
    refresh();
    input.focus();
    input.select();
    // A click outside the box ends it (not the buttons inside).
    const away = (e: FocusEvent) => {
        const to = e.relatedTarget as Node | null;
        if (to && dom.contains(to)) return;
        setTimeout(() => { if (!closed && !dom.contains(document.activeElement)) close(); }, 0);
    };
    dom.addEventListener('focusout', away);
    // Safari does not focus a button that is clicked, so keep the focus where it is.
    dom.addEventListener('mousedown', e => { if ((e.target as HTMLElement).tagName === 'BUTTON') e.preventDefault(); });
}

/** Closes an open rename box, if any. */
export function closeRename(editor: monaco.editor.ICodeEditor): void {
    boxes.get(editor)?.close();
}
