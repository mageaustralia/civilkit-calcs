// Sheet models complete only from the provider in completions.ts (names, functions, units, block keywords), never from
// the words of the document. The editor option is switched per model, because a Calcpad worksheet keeps word-based
// suggestions.
import * as monaco from 'monaco-editor';
import { isSheetText } from '../wasm/sheet';

export function attachSheetSuggest(ed: monaco.editor.IStandaloneCodeEditor): void {
    const sync = () => {
        const m = ed.getModel();
        ed.updateOptions({ wordBasedSuggestions: m && isSheetText(m.getValue()) ? 'off' : 'matchingDocuments' });
    };
    ed.onDidChangeModel(sync);
    ed.onDidChangeModelLanguage(sync);
    sync();
}
