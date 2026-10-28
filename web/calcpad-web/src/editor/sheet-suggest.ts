// When sheet completions appear. The provider in sheet-language.ts offers names and built-in functions
// only in math lines, never while typing prose; Ctrl+Space still offers everything.
import * as monaco from 'monaco-editor';
import { isSheetText } from '../wasm/sheet';
import { buildBuiltinDocMarkdown, extractFunctionName } from './builtin-docs';
import type { EditorBridge } from './bridge';

export interface BuiltinFunction { name: string; detail: string; doc: string }

let functionSource: () => BuiltinFunction[] = () => [];

/** The built-in function names come from the snippet service the Calcpad editor already uses. */
export function builtinFunctions(): BuiltinFunction[] {
    return functionSource();
}

const NAME = '[\\p{L}_][\\p{L}\\p{N}_′″]*';
const HEAD = new RegExp(`^\\s*${NAME}(\\([^)]*\\))?\\s*(:|=(?!=))\\s*`, 'u');
const KEYWORD = /^\s*(if|else if|while|for|repeat|plot)\s/;

/** Which lines are code fences or the inside of a `$$` display formula (no completions there). */
export function verbatimLines(lines: string[]): boolean[] {
    const out: boolean[] = [];
    let fence = false;
    let math = false;
    for (const l of lines) {
        const t = l.trim();
        const edge = (!math && t.startsWith('```')) || (!fence && t === '$$');
        if (edge) { if (t === '$$') math = !math; else fence = !fence; }
        out.push(edge || fence || math);
    }
    return out;
}

/**
 * Whether the text before the caret is math: a quantity, condition, loop or plot line, or a line
 * that is being typed as one. Prose, notes after `|`, quoted text, `$...$` formulas and raw `>>` lines are not.
 * `kind` is the compiler's letter for the line, `pre` the line up to the caret.
 */
export function inMathContext(kind: string | undefined, pre: string): boolean {
    if (/^\s*>>/.test(pre)) return false;
    const bare = pre.replace(/\|\|/g, '');
    if (bare.includes('|')) return false;
    if ((pre.match(/"/g)?.length ?? 0) % 2 === 1) return false;
    if ((pre.match(/(?<!\\)\$/g)?.length ?? 0) % 2 === 1) return false;
    if (kind && 'qckg'.includes(kind)) return true;
    if (KEYWORD.test(pre)) return true;
    const head = HEAD.exec(pre);
    // `Note: see the drawing` is prose; `x: 2 *` is a line being typed.
    return !!head && /[-+*/^(<>,;]|\d/.test(pre.slice(head[0].length));
}

/** Sheet models complete only from the provider, not from words of the document. */
export function attachSheetSuggest(ed: monaco.editor.IStandaloneCodeEditor, bridge: EditorBridge): void {
    functionSource = () => {
        if (!bridge.snippets.isLoaded()) return [];
        const seen = new Set<string>();
        const out: BuiltinFunction[] = [];
        for (const item of bridge.snippets.getAllItems()) {
            if (item.keywordType !== 'Function') continue;
            const name = extractFunctionName(item.tag);
            if (!name || seen.has(name)) continue;
            seen.add(name);
            out.push({ name, detail: item.description || item.label || name, doc: buildBuiltinDocMarkdown(item) });
        }
        return out;
    };
    const sync = () => {
        const m = ed.getModel();
        ed.updateOptions({ wordBasedSuggestions: m && isSheetText(m.getValue()) ? 'off' : 'matchingDocuments' });
    };
    ed.onDidChangeModel(sync);
    ed.onDidChangeModelLanguage(sync);
    sync();
}
