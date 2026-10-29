// Builds the DocIndex of a Monaco model: the engine's tokens for a Calcpad worksheet, the
// compiler's line kinds for a sheet.

import type * as monaco from 'monaco-editor';
import { truncateBase64Content } from 'calcpad-frontend/services/base64-truncate';
import type { EditorBridge } from '../bridge';
import type { FileContextProvider } from '../diagnostics';
import { compileSheetNow, loadSheet } from '../../wasm/sheet';
import { SHEET_LANGUAGE } from '../sheet-language';
import { scanCalcpad, scanSheet } from './doc-index';
import type { DocIndex } from './doc-index';

export interface NavCtx {
    bridge: EditorBridge;
    getFileContext?: FileContextProvider;
}

export function isSheetModel(model: monaco.editor.ITextModel): boolean {
    return model.getLanguageId() === SHEET_LANGUAGE;
}

/** The index of the model's current text, or null when the engine could not be reached. */
export async function buildIndex(nav: NavCtx, model: monaco.editor.ITextModel): Promise<DocIndex | null> {
    const lines = model.getLinesContent();
    const text = lines.join('\n');
    if (isSheetModel(model)) {
        await loadSheet();
        const c = compileSheetNow(text);
        return c ? scanSheet(lines, c) : null;
    }
    const ctx = nav.getFileContext ? await nav.getFileContext(text).catch(() => ({} as { sourceFilePath?: string })) : {};
    const tokens = await nav.bridge.api.highlight(truncateBase64Content(text), true, ctx.sourceFilePath);
    return tokens ? scanCalcpad(lines, tokens) : null;
}
