// Convert a selection to a formula, in the source editor (the line arithmetic is to-formula.ts). Reached
// three ways: the editor's context menu and Ctrl+Shift+E (Cmd+Shift+E on a Mac), the "= Formula" button
// of the prose toolbar, and on a touch screen a small button under the selection. One edit, one undo step.
import * as monaco from 'monaco-editor';
import { compileSheetNow } from '../wasm/sheet';
import { SHEET_LANGUAGE } from './sheet-language';
import { proseLines } from './prose-toolbar-monaco';
import { isTouchDevice } from './prose-toolbar';
import { cpdPlace, sheetPlace, toFormula } from './to-formula';
import type { FormulaEdit, FormulaPlace } from './to-formula';

export const TO_FORMULA_ACTION = 'calcpad.toFormula';
export const TO_FORMULA_TIP = 'Turn the selected expression into a formula line (Ctrl+Shift+E). In text it goes on a line of its own after the paragraph; in a calculation it is pulled out to a named line above.';

let cacheModel: monaco.editor.ITextModel | null = null;
let cacheVersion = -1;
let cachePlace: ((l: number, c: number) => FormulaPlace) | null = null;

/** Where each line of the editor's document sits, or null for a file that is neither a sheet nor a worksheet. */
function placeOf(model: monaco.editor.ITextModel): ((l: number, c: number) => FormulaPlace) | null {
    if (model === cacheModel && model.getVersionId() === cacheVersion) return cachePlace;
    const lang = model.getLanguageId();
    const lines = model.getLinesContent();
    let place: ((l: number, c: number) => FormulaPlace) | null = null;
    if (lang === SHEET_LANGUAGE) {
        const c = compileSheetNow(model.getValue());
        if (c) {
            const prose = proseLines(lines, c.kinds);
            place = (l, col) => sheetPlace(lines[l] ?? '', c.kinds[l] ?? '', !!prose[l], col);
        }
    } else if (lang === 'calcpad') {
        place = (l, col) => cpdPlace(lines[l] ?? '', col);
    }
    cacheModel = model;
    cacheVersion = model.getVersionId();
    cachePlace = place;
    return place;
}

/** The edit for the editor's selection, or null when it does not apply. */
function plan(ed: monaco.editor.ICodeEditor): FormulaEdit | null {
    const model = ed.getModel();
    const s = ed.getSelection();
    if (!model || !s || s.isEmpty() || s.startLineNumber !== s.endLineNumber) return null;
    const place = placeOf(model);
    if (!place) return null;
    return toFormula(model.getLinesContent(), { line: s.startLineNumber - 1, c1: s.startColumn - 1, c2: s.endColumn - 1 }, place, model.getLanguageId() === SHEET_LANGUAGE);
}

/** True when the selection can be turned into a formula. */
export function canConvertToFormula(ed: monaco.editor.ICodeEditor): boolean {
    return plan(ed) !== null;
}

/** Makes the edit; when the selection is not an expression on one line, says so under the caret. */
export function convertSelectionToFormula(ed: monaco.editor.ICodeEditor): boolean {
    const model = ed.getModel();
    const e = plan(ed);
    if (!model || !e) {
        const pos = ed.getPosition();
        const msg = ed.getContribution('editor.contrib.messageController') as { showMessage?: (m: string, p: monaco.IPosition) => void } | null;
        if (pos) msg?.showMessage?.('Select an expression on one line, such as w*L^2/8, in text or in a calculation.', pos);
        return false;
    }
    ed.pushUndoStop();
    ed.executeEdits('to-formula', [{
        range: new monaco.Range(e.from + 1, 1, e.to + 1, model.getLineMaxColumn(e.to + 1)),
        text: e.text.join(model.getEOL()),
    }], e.sels.map(s => new monaco.Selection(s.line + 1, s.c1 + 1, s.line + 1, s.c2 + 1)));
    ed.pushUndoStop();
    ed.revealLineInCenterIfOutsideViewport(e.sels[0].line + 1);
    ed.focus();
    return true;
}

/** The "= Formula" button for a toolbar; it never takes the focus from the editor. */
export function formulaButton(ed: monaco.editor.ICodeEditor, label = '= Formula'): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'pt-formula';
    b.textContent = label;
    b.title = TO_FORMULA_TIP;
    b.setAttribute('aria-label', 'Convert the selection to a formula');
    b.addEventListener('mousedown', e => e.preventDefault());
    b.addEventListener('click', () => { convertSelectionToFormula(ed); });
    return b;
}

export function attachToFormula(ed: monaco.editor.IStandaloneCodeEditor): void {
    ed.addAction({
        id: TO_FORMULA_ACTION,
        label: 'Convert Selection to Formula',
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyE],
        precondition: 'editorHasSelection',
        contextMenuGroupId: '1_modification',
        contextMenuOrder: 0.5,
        run: e => { convertSelectionToFormula(e); },
    });
    if (!isTouchDevice()) return;

    // Touch: no hover, no context menu and no prose toolbar, so a button under the selection.
    const host = ed.getContainerDomNode();
    if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
    const chip = formulaButton(ed);
    chip.classList.add('to-formula-chip');
    chip.hidden = true;
    // A touch on the button must not move the caret in the editor underneath.
    chip.addEventListener('pointerdown', e => e.stopPropagation());
    chip.addEventListener('touchstart', e => e.stopPropagation(), { passive: true });
    host.appendChild(chip);
    const update = () => {
        const s = ed.getSelection();
        const pos = s && !s.isEmpty() && canConvertToFormula(ed)
            ? ed.getScrolledVisiblePosition({ lineNumber: s.endLineNumber, column: s.endColumn })
            : null;
        if (!pos) { chip.hidden = true; return; }
        chip.hidden = false;
        const top = pos.top + pos.height + 8;
        chip.style.top = Math.min(top, host.clientHeight - chip.offsetHeight - 4) + 'px';
        chip.style.left = Math.max(4, Math.min(pos.left - chip.offsetWidth / 2, host.clientWidth - chip.offsetWidth - 4)) + 'px';
    };
    ed.onDidChangeCursorSelection(update);
    ed.onDidScrollChange(() => { if (!chip.hidden) update(); });
    ed.onDidChangeModel(() => { chip.hidden = true; });
}
