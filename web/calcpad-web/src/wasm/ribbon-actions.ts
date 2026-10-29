// calcpad-rs: what the ribbon's `action` buttons do. Each key is an `action` id from ribbon-data.ts;
// ids with a colon (`mode:report`, `tab:insert`, `theme:Dark`) share one handler and take the part
// after the colon as their argument.
import type { WebFiles } from './web-files';
import { getResolvedAppTheme } from '../editor/app-theme';
import { convertToCalcpad, newSheet, showCompiled } from './sheet-ui';
import { sheetInsert } from './sheet-insert';
import { diagramEdit } from './diagram-starters.ts';
import { SymbolicDialog } from './symbolic';

/** The editor's file is a sheet (its Monaco language, which follows the marker line). */
export const isSheetEditor = (editor: any): boolean => editor?.getModel?.()?.getLanguageId?.() === 'calcpad-sheet';

export interface RibbonCtx {
    files: WebFiles;
    /** The mounted App.vue instance (its exposed API). */
    app: any;
    /** The message bridge (settings, theme, exports). */
    bridge: any;
    /** The active Monaco editor, if any. */
    editor: () => any;
    /** Opens a tab of the sidebar. */
    showSidebarTab: (id: string) => void;
    /** The element a dropdown should hang from. */
    anchor: HTMLElement;
}

const UPSTREAM_URL = 'https://github.com/imartincei/CalcpadCE';
const PROJECT_URL = 'https://github.com/mageaustralia/civilkit-calcs';
const DOCS_URL = UPSTREAM_URL;
const CHANGELOG_URL = `${PROJECT_URL}/blob/main/CHANGELOG.md`;
const WHATS_NEW_URL = 'https://civilkit.io/calcs/whats-new/';
const LEARN_URL = 'https://civilkit.io/calcs/learn/engineering-calculation-software-free/';
const APP_VERSION = import.meta.env.VITE_APP_VERSION ?? '';
const UPSTREAM_VERSION = import.meta.env.VITE_UPSTREAM_VERSION ?? '';
const mod = /Mac|iPhone|iPad/.test(navigator.platform) ? 'Cmd' : 'Ctrl';

const edit = (c: RibbonCtx, cmd: string) => {
    const e = c.editor();
    e?.focus();
    e?.trigger('ribbon', cmd, null);
};
const zoom = (c: RibbonCtx, delta: number) => {
    const e = c.editor();
    if (!e) return;
    const cur = e.getOption((window as any).monaco.editor.EditorOption.fontSize) as number;
    e.updateOptions({ fontSize: Math.max(8, Math.min(40, cur + delta)) });
};
/** A small modal dialog: a title, paragraphs (a string with a URL becomes a link) and a Close button. Esc and a click outside close it. */
function dialog(title: string, paragraphs: string[]): void {
    document.getElementById('cprs-dialog')?.remove();
    const back = document.createElement('div');
    back.id = 'cprs-dialog';
    back.style.cssText = 'position:fixed;inset:0;z-index:2000;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)';
    const box = document.createElement('div');
    box.setAttribute('role', 'dialog');
    box.setAttribute('aria-label', title);
    box.style.cssText = 'max-width:min(520px,92vw);max-height:80vh;overflow:auto;padding:18px 22px;border-radius:6px;font:13px/1.5 system-ui,sans-serif;' +
        'background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)';
    const h = document.createElement('h3');
    h.textContent = title;
    h.style.cssText = 'margin:0 0 10px;font-size:16px';
    box.append(h);
    for (const p of paragraphs) {
        const el = document.createElement('p');
        el.style.margin = '6px 0';
        const md = /^(.*?)\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)(.*)$/.exec(p);
        const m = md ? null : /^(.*?)(https?:\/\/\S+)$/.exec(p);
        if (md) {
            el.append(md[1]);
            const a = document.createElement('a');
            a.href = md[3];
            a.textContent = md[2];
            a.target = '_blank';
            a.rel = 'noopener';
            a.style.color = 'var(--vscode-textLink-foreground,#0060c0)';
            el.append(a, md[4]);
        } else if (m) {
            el.append(m[1]);
            const a = document.createElement('a');
            a.href = m[2];
            a.textContent = m[2];
            a.target = '_blank';
            a.rel = 'noopener';
            a.style.color = 'var(--vscode-textLink-foreground,#0060c0)';
            el.append(a);
        } else el.textContent = p;
        box.append(el);
    }
    const close = document.createElement('button');
    close.textContent = 'Close';
    close.style.cssText = 'margin-top:12px;padding:4px 14px;font:inherit;cursor:pointer';
    const done = () => { back.remove(); document.removeEventListener('keydown', onKey, true); };
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.stopPropagation(); done(); } };
    close.onclick = done;
    back.onmousedown = (e) => { if (e.target === back) done(); };
    document.addEventListener('keydown', onKey, true);
    box.append(close);
    back.append(box);
    document.body.append(back);
    close.focus();
}

let symbolic: SymbolicDialog | null = null;

type Handler = (c: RibbonCtx, arg: string) => void;
export const ACTIONS: Record<string, Handler> = {
    newsheet: c => newSheet(c.files.o.tabs()),
    newcalcpad: c => c.files.newWorksheet(),
    changelog: () => void window.open(CHANGELOG_URL, '_blank', 'noopener'),
    showcompiled: c => void showCompiled(c.editor()),
    convertsheet: c => void convertToCalcpad(c.editor()),
    open: c => void c.files.open(),
    save: c => void c.files.save(),
    saveas: c => void c.files.save(true),
    filemenu: c => c.files.toggleMenu(c.anchor),
    samples: c => c.files.o.openSamples?.(),
    importWorksheet: c => void c.files.o.importer?.pick(),
    tidyText: c => void c.files.o.importer?.transformActive('tidy'),
    markdownText: c => void c.files.o.importer?.transformActive('markdown'),
    symbolic: c => {
        symbolic ??= new SymbolicDialog({ editor: () => c.editor(), isSheet: isSheetEditor, insert: insertSnippet });
        symbolic.open();
    },
    pdf: c => c.files.o.print(),
    word: c => c.files.o.exportDocx?.(),
    excel: c => c.files.o.exportXlsx?.(),
    exportTab: c => c.files.o.openExport?.(),
    undo: c => edit(c, 'undo'),
    redo: c => edit(c, 'redo'),
    find: c => edit(c, 'actions.find'),
    cut: c => edit(c, 'editor.action.clipboardCutAction'),
    copy: c => edit(c, 'editor.action.clipboardCopyAction'),
    paste: c => {
        const e = c.editor();
        e?.focus();
        void navigator.clipboard?.readText().then(t => {
            if (t) e?.executeEdits('ribbon', [{ range: e.getSelection(), text: t, forceMoveMarkers: true }]);
        }).catch(() => e?.trigger('ribbon', 'editor.action.clipboardPasteAction', null));
    },
    calc: c => c.app.onRunRequest?.(),
    mode: (c, m) => void c.app.setResultMode(m),
    togglePreview: c => c.app.togglePreview?.(),
    sidebar: c => c.app.toggleSidebar?.(),
    tab: (c, id) => c.showSidebarTab(id),
    settings: c => c.showSidebarTab('settings'),
    theme: (c, t) => c.bridge?.handleMessage({ type: 'updateColorTheme', theme: t }),
    zoomIn: c => zoom(c, 1),
    zoomOut: c => zoom(c, -1),
    autocalc: c => c.bridge?.setExtraSetting('autoRun', c.bridge.getExtraSetting('autoRun') === 'false' ? 'true' : 'false'),
    docs: () => void window.open(DOCS_URL, '_blank', 'noopener'),
    // The browser keeps Ctrl+N, Ctrl+O, Ctrl+P, Ctrl+T and F5 for itself, so they are not listed (main.ts).
    shortcuts: () => dialog('Keyboard shortcuts', [
        `${mod}+S save, ${mod}+Shift+S save as`,
        `${mod}+Alt+X calculate (in the editor)`,
        `${mod}+F find, ${mod}+Z undo, ${mod}+Shift+Z redo`,
        'Alt+Z wrap long lines',
        'F2 rename the name under the cursor everywhere in the document',
        `${mod}+; find a variable or function (Enter inserts it, Alt+Enter goes to its definition)`,
        'F12 go to the definition, Shift+F12 find references',
        `${mod}+Shift+/ disable or enable the block at the cursor`,
    ]),
    about: () => dialog(`CivilKit Calcs ${APP_VERSION}`, [
        `Built on CalcpadCE ${UPSTREAM_VERSION} (MIT), by Nedelcho Ganchovski (Proektsoft) and contributors. The engine is reimplemented in Rust and runs in your browser.`,
        `CalcpadCE: ${UPSTREAM_URL}`,
        `This project: ${PROJECT_URL}`,
        `What's new: [the changelog](${CHANGELOG_URL})`,
        `Short clips of each release: [What's new](${WHATS_NEW_URL})`,
        `Calculators and guides: [ready-made calculators](${LEARN_URL})`,
        'MIT licence. Not affiliated with the CalcpadCE authors.',
    ]),
};

export const ACTIVE: Record<string, (c: RibbonCtx, arg: string) => boolean> = {
    mode: (c, m) => c.app.getResultMode?.() === m,
    togglePreview: c => !!c.app.isPreviewVisible?.(),
    theme: (_c, t) => getResolvedAppTheme() === t.toLowerCase(),
    autocalc: c => c.bridge?.getExtraSetting('autoRun') !== 'false',
};

export function runAction(id: string, c: RibbonCtx): void {
    const i = id.indexOf(':');
    const key = i < 0 ? id : id.slice(0, i);
    ACTIONS[key]?.(c, i < 0 ? '' : id.slice(i + 1));
}
export function isActive(id: string, c: RibbonCtx): boolean {
    const i = id.indexOf(':');
    const key = i < 0 ? id : id.slice(0, i);
    try {
        return !!ACTIVE[key]?.(c, i < 0 ? '' : id.slice(i + 1));
    } catch {
        return false;
    }
}

/**
 * Inserts text at the editor cursor. A `line` snippet goes on its own line (after the current one when
 * that has content); `{sel}` in the text stands for the selection, or the word "text" when nothing is selected.
 */
export function insertSnippet(editor: any, text: string, line: boolean, sheet?: string): boolean {
    const model = editor?.getModel();
    if (!model) return false;
    // In a sheet each button inserts its sheet form (sheet-insert.ts).
    if (isSheetEditor(editor)) {
        const i = sheetInsert(text, line, sheet);
        text = i.text;
        line = i.line;
    }
    const sel = editor.getSelection();
    const selected = model.getValueInRange(sel);
    const body = text.replace(/\{sel\}/g, selected || 'text');
    if (line) {
        const ln = sel.endLineNumber;
        const blank = model.getLineContent(ln).trim() === '';
        const col = model.getLineMaxColumn(ln);
        const at = blank ? { startLineNumber: ln, startColumn: 1, endLineNumber: ln, endColumn: col } : { startLineNumber: ln, startColumn: col, endLineNumber: ln, endColumn: col };
        editor.executeEdits('ribbon', [{ range: at, text: (blank ? '' : '\n') + body, forceMoveMarkers: true }]);
    } else {
        editor.executeEdits('ribbon', [{ range: sel, text: body, forceMoveMarkers: true }]);
    }
    editor.focus();
    return true;
}

/**
 * Inserts a starter diagram (the Diagrams menu) as a block of its own at the caret's block boundary: a ```mermaid
 * fence in a sheet, the fence in a #markdown block in a worksheet (diagram-starters.ts). One undo step; the
 * diagram's source is left selected, ready to edit.
 */
export function insertDiagram(editor: any, source: string): boolean {
    const model = editor?.getModel();
    if (!model) return false;
    const e = diagramEdit(model.getLinesContent(), editor.getPosition()?.lineNumber ?? 1, source, isSheetEditor(editor));
    const count = model.getLineCount();
    let range, text;
    if (e.replace) {
        range = { startLineNumber: e.before, startColumn: 1, endLineNumber: e.before, endColumn: model.getLineMaxColumn(e.before) };
        text = e.lines.join('\n');
    } else if (e.before > count) {
        const col = model.getLineMaxColumn(count);
        range = { startLineNumber: count, startColumn: col, endLineNumber: count, endColumn: col };
        text = '\n' + e.lines.join('\n');
    } else {
        range = { startLineNumber: e.before, startColumn: 1, endLineNumber: e.before, endColumn: 1 };
        text = e.lines.join('\n') + '\n';
    }
    editor.pushUndoStop();
    editor.executeEdits('ribbon-diagram', [{ range, text }]);
    editor.pushUndoStop();
    editor.setSelection({ startLineNumber: e.selectFrom, startColumn: 1, endLineNumber: e.selectTo, endColumn: model.getLineMaxColumn(e.selectTo) });
    editor.revealLinesInCenterIfOutsideViewport(e.selectFrom, e.selectTo);
    editor.focus();
    return true;
}
