// The prose toolbar in the source editor: it shows above the caret line or the selection when that is
// prose in a sheet (not a quantity or other math line, and not inside a fence or a display formula).
// The edit is made on the source text with executeEdits, so it is one undo step. Hidden on touch screens,
// where it would fight the system selection menu; there the in-place editor has its own bar.
import * as monaco from 'monaco-editor';
import { compileSheetNow } from '../wasm/sheet';
import { SHEET_LANGUAGE } from './sheet-language';
import { applyProse, proseState } from './prose-format';
import type { ProseAction } from './prose-format';
import { ProseToolbar, isTouchDevice } from './prose-toolbar';
import { canConvertToFormula, formulaButton } from './to-formula-monaco';

/** Which lines of a sheet are plain prose: kind `p` and outside code fences and `$$` display formulas. */
export function proseLines(lines: string[], kinds: string): boolean[] {
    const out: boolean[] = [];
    let fence = false;
    let math = false;
    for (let i = 0; i < lines.length; i++) {
        const t = lines[i].trim();
        if (!math && t.startsWith('```')) { fence = !fence; out.push(false); continue; }
        if (!fence && t === '$$') { math = !math; out.push(false); continue; }
        out.push(!fence && !math && kinds[i] === 'p');
    }
    return out;
}

export function attachProseToolbar(ed: monaco.editor.IStandaloneCodeEditor): void {
    if (isTouchDevice()) return;
    // The editor's own element is replaced when a model is set, so the bar lives in its container.
    const host = ed.getContainerDomNode();
    if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
    let bar: ProseToolbar | null = null;
    // "= Formula" (to-formula-monaco.ts), shown while the selection is an expression.
    let formula: HTMLButtonElement | null = null;
    let cacheVersion = -1;
    let cache: boolean[] = [];

    const prose = (model: monaco.editor.ITextModel): boolean[] | null => {
        if (model.getLanguageId() !== SHEET_LANGUAGE) return null;
        if (model.getVersionId() === cacheVersion) return cache;
        const c = compileSheetNow(model.getValue());
        if (!c) return null;
        cache = proseLines(model.getLinesContent(), c.kinds);
        cacheVersion = model.getVersionId();
        return cache;
    };

    /** The selection when the toolbar applies to it: 0-based lines and columns. */
    const target = () => {
        const model = ed.getModel();
        const s = ed.getSelection();
        if (!model || !s) return null;
        const flags = prose(model);
        if (!flags) return null;
        const l1 = s.startLineNumber - 1;
        const l2 = s.endLineNumber - 1 - (s.endColumn === 1 && s.endLineNumber > s.startLineNumber ? 1 : 0);
        for (let l = l1; l <= l2; l++) {
            if (l === l1 || l === l2) { if (!flags[l]) return null; } else if (!flags[l] && model.getLineContent(l + 1).trim() !== '') return null;
        }
        const c2 = l2 + 1 === s.endLineNumber ? s.endColumn - 1 : model.getLineMaxColumn(l2 + 1) - 1;
        return { model, l1, l2, c1: s.startColumn - 1, c2 };
    };

    const ensure = (): ProseToolbar => {
        if (bar) return bar;
        bar = new ProseToolbar(act => run(act));
        formula = formulaButton(ed);
        bar.el.appendChild(formula);
        host.appendChild(bar.el);
        return bar;
    };

    const update = (quiet: boolean) => {
        const s = ed.getSelection();
        // While typing, the bar stays out of the way unless text is selected.
        const t = s && ed.hasTextFocus() && !(quiet && s.isEmpty()) ? target() : null;
        if (!t || !s) { bar?.hide(); return; }
        const tb = ensure();
        const line = t.model.getLineContent(t.l1 + 1);
        tb.setState(proseState(line, t.c1, t.l1 === t.l2 ? t.c2 : line.length));
        if (formula) formula.hidden = s.isEmpty() || !canConvertToFormula(ed);
        tb.show();
        const pos = ed.getScrolledVisiblePosition({ lineNumber: s.startLineNumber, column: s.startColumn });
        if (!pos) { tb.hide(); return; }
        const w = tb.el.offsetWidth;
        const h = tb.el.offsetHeight;
        let top = pos.top - h - 4;
        if (top < 2) top = pos.top + pos.height + 4;
        const left = Math.max(4, Math.min(pos.left, host.clientWidth - w - 4));
        tb.el.style.top = top + 'px';
        tb.el.style.left = left + 'px';
    };

    const run = (action: ProseAction) => {
        const t = target();
        if (!t) return;
        const lines = t.model.getLinesContent().slice(t.l1, t.l2 + 1);
        const r = applyProse(lines, action, { l1: 0, c1: t.c1, l2: t.l2 - t.l1, c2: t.c2 });
        if (!r) return;
        const last = t.l2 + 1;
        ed.pushUndoStop();
        ed.executeEdits('prose-toolbar', [{
            range: new monaco.Range(t.l1 + 1, 1, last, t.model.getLineMaxColumn(last)),
            text: r.lines.join('\n'),
        }], [new monaco.Selection(t.l1 + 1 + r.l1, r.c1 + 1, t.l1 + 1 + r.l2, r.c2 + 1)]);
        ed.pushUndoStop();
        update(false);
    };

    ed.onDidChangeCursorSelection(e => update(e.reason !== monaco.editor.CursorChangeReason.Explicit));
    ed.onDidChangeModel(() => bar?.hide());
    ed.onDidBlurEditorText(() => bar?.hide());
    ed.onDidFocusEditorText(() => update(true));
    ed.onDidScrollChange(() => { if (bar?.shown) update(false); });
    ed.onDidChangeModelContent(() => { if (bar?.shown) update(true); });
    ed.onKeyDown(e => { if (e.keyCode === monaco.KeyCode.Escape) bar?.hide(); });
}
