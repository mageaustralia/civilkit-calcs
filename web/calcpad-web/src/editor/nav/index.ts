// Editor navigation: rename everywhere (F2) and the variable explorer palette (Ctrl+;), added to
// each editor, plus the entry points the Variables tab uses.

import * as monaco from 'monaco-editor';
import { closeRename, openRename } from './rename';
import { buildIndex } from './index-builder';
import type { NavCtx } from './index-builder';
import { canonicalSheetName } from './lexer';
import { closePalette, togglePalette } from './palette';

export type { NavCtx };

/**
 * The palette's key. Ctrl+; (Cmd+; on a Mac) is free in the browsers and in Monaco: the editor
 * uses Ctrl+Space, Ctrl+Shift+Space and Ctrl+. for its own lists, and a browser keeps Ctrl+L,
 * D, R, N, P, T, W, Q, J, 0-9, plus and minus.
 */
export const PALETTE_KEYBINDING = monaco.KeyMod.CtrlCmd | monaco.KeyCode.Semicolon;

export function attachEditorNav(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx): monaco.IDisposable[] {
    return [
        editor.addAction({
            id: 'calcpad.renameEverywhere',
            label: 'Rename Everywhere...',
            keybindings: [monaco.KeyCode.F2],
            contextMenuGroupId: '1_modification',
            contextMenuOrder: 1.1,
            run: () => { void openRename(editor, nav); },
        }),
        editor.addAction({
            id: 'calcpad.quickRecall',
            label: 'Find Variable or Function...',
            keybindings: [PALETTE_KEYBINDING],
            contextMenuGroupId: 'navigation',
            contextMenuOrder: 1.8,
            run: () => { void togglePalette(editor, nav); },
        }),
        editor.onDidChangeModel(() => { closeRename(editor); closePalette(); }),
    ];
}

/** The Variables tab asks for a rename of a name (Calcpad spelling) in the active editor. */
export function renameSymbolByName(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx, name: string): Promise<void> {
    return openRename(editor, nav, name);
}

/** The Variables tab asks to go to the definition of a name. Returns false when it is not in the document. */
export async function gotoSymbolByName(editor: monaco.editor.IStandaloneCodeEditor, nav: NavCtx, name: string): Promise<boolean> {
    const model = editor.getModel();
    if (!model) return false;
    const idx = await buildIndex(nav, model);
    if (!idx) return false;
    const syms = [...idx.syms.values()].filter(s => !s.local && s.defLine >= 0 && (s.name === name || canonicalSheetName(s.name) === name));
    const s = syms[0];
    if (!s) return false;
    editor.setPosition({ lineNumber: s.defLine + 1, column: s.defCol + 1 });
    editor.revealLineInCenter(s.defLine + 1);
    editor.focus();
    return true;
}
