import * as monaco from 'monaco-editor';
import { compileSheetNow, formatSheet, isSheetText, loadSheet } from '../wasm/sheet';
import type { SheetCompiled } from '../wasm/sheet';
import { builtinFunctions, inMathContext, verbatimLines } from './sheet-suggest';

/** Monaco language id of a sheet (docs/sheet-syntax.md). Calcpad's own providers do not apply to it. */
export const SHEET_LANGUAGE = 'calcpad-sheet';

const GREEK = ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta', 'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'pi', 'rho', 'sigma', 'tau', 'upsilon', 'phi', 'chi', 'psi', 'omega'];

const configuration: monaco.languages.LanguageConfiguration = {
    brackets: [['(', ')'], ['[', ']'], ['{', '}']],
    autoClosingPairs: [
        { open: '(', close: ')' },
        { open: '[', close: ']' },
        { open: '{', close: '}' },
        { open: '"', close: '"', notIn: ['string'] },
        { open: '`', close: '`' },
    ],
    surroundingPairs: [{ open: '*', close: '*' }, { open: '`', close: '`' }, { open: '$', close: '$' }, { open: '(', close: ')' }, { open: '[', close: ']' }],
    onEnterRules: [
        // A block header continues one level in.
        { beforeText: /^\s*(if|else if|else|while|repeat|for)\b.*:\s*$/, action: { indentAction: monaco.languages.IndentAction.Indent } },
    ],
};

const ID = '[A-Za-z_\\u0370-\\u03ff][A-Za-z0-9_\\u0370-\\u03ff′″]*';

const monarch: monaco.languages.IMonarchLanguage = {
    defaultToken: '',
    tokenPostfix: '.sheet',
    unicode: true,
    tokenizer: {
        root: [
            [/<!--sheet-->/, 'comment'],
            [/^\s*>>.*$/, 'type'],
            [/^\s*#unit(?=\s)/, 'keyword'],
            [/^\s*```\s*python\s*$/, { token: 'type', next: '@pyFence', nextEmbedded: 'python' }],
            [/^\s*```\s*calcpad\s*$/, { token: 'type', next: '@calcpadFence' }],
            [/^\s*```.*$/, { token: 'string', next: '@codeFence' }],
            [/^\s*#{1,6}\s.*$/, 'keyword.heading'],
            [/^\s*\$\$\s*$/, { token: 'string.math', next: '@displayMath' }],
            [/\$\$.+?\$\$/, 'string.math'],
            [/\$[^\s$][^$]*\$/, 'string.math'],
            [/^\s*(if|else if|else|while|repeat|for|plot)\b/, 'keyword'],
            [/\b(in)\b/, 'keyword'],
            [new RegExp(`^(\\s*)(${ID}(?:\\([^)]*\\))?)(\\s*)(:|=(?!=))`), ['', 'variable.name', '', 'delimiter']],
            [/\|\s.*$/, 'comment'],
            [/->/, 'operator'],
            [/\*\*[^*]+\*\*/, 'strong'],
            [/\*[^*\s][^*]*\*/, 'emphasis'],
            [/`[^`]*`/, 'string'],
            [/"[^"]*"/, 'string'],
            [/\d+(\.\d+)?([eE][+-]?\d+)?/, 'number'],
        ],
        calcpadFence: [[/^\s*```\s*$/, { token: 'type', next: '@pop' }], [/.*/, 'type']],
        pyFence: [[/^\s*```\s*$/, { token: 'type', next: '@pop', nextEmbedded: '@pop' }], [/.+/, '']],
        codeFence: [[/^\s*```\s*$/, { token: 'string', next: '@pop' }], [/.*/, 'string']],
        displayMath: [[/^\s*\$\$\s*$/, { token: 'string.math', next: '@pop' }], [/.*/, 'string.math']],
    },
};

const NAME_CHARS = '[\\p{L}\\p{N}_\u2032\u2033]';

/** The word under the cursor that is a name this sheet defines, with its definition. */
function definitionAt(model: monaco.editor.ITextModel, position: monaco.Position, c: SheetCompiled | null) {
    const w = model.getWordAtPosition(position);
    const d = w && c?.defs.find(x => x.name === w.word);
    return w && d ? { word: w, def: d } : null;
}

/** Where the names are used: code lines (not prose), outside a quantity's note. */
function usages(model: monaco.editor.ITextModel, c: SheetCompiled, name: string): monaco.Range[] {
    const re = new RegExp(`(?<!${NAME_CHARS})${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}(?!${NAME_CHARS})`, 'gu');
    const out: monaco.Range[] = [];
    for (let i = 0; i < model.getLineCount(); i++) {
        const k = c.kinds[i];
        if (!k || 'bp'.includes(k)) continue;
        let text = model.getLineContent(i + 1);
        const note = k === 'q' ? text.search(/(?<!\|)\|(?!\|)/) : -1;
        if (note >= 0) text = text.slice(0, note);
        for (const m of text.matchAll(re)) {
            out.push(new monaco.Range(i + 1, m.index! + 1, i + 1, m.index! + 1 + name.length));
        }
    }
    return out;
}

let registered = false;

/** Registers the sheet language, its tokenizer and completions. Call once with the Calcpad language. */
export function registerSheetLanguage(): void {
    if (registered) return;
    registered = true;
    monaco.languages.register({ id: SHEET_LANGUAGE, extensions: ['.sheet'] });
    monaco.languages.setLanguageConfiguration(SHEET_LANGUAGE, configuration);
    monaco.languages.setMonarchTokensProvider(SHEET_LANGUAGE, monarch);
    monaco.languages.registerDocumentFormattingEditProvider(SHEET_LANGUAGE, {
        async provideDocumentFormattingEdits(model) {
            const text = await formatSheet(model.getValue());
            return text === model.getValue() ? [] : [{ range: model.getFullModelRange(), text }];
        },
    });
    monaco.languages.registerDefinitionProvider(SHEET_LANGUAGE, {
        provideDefinition(model, position) {
            const hit = definitionAt(model, position, compileSheetNow(model.getValue()));
            if (!hit) return null;
            const line = hit.def.line + 1;
            const col = Math.max(1, model.getLineContent(line).indexOf(hit.def.name) + 1);
            return { uri: model.uri, range: new monaco.Range(line, col, line, col + hit.def.name.length) };
        },
    });
    monaco.languages.registerReferenceProvider(SHEET_LANGUAGE, {
        provideReferences(model, position) {
            const c = compileSheetNow(model.getValue());
            const hit = definitionAt(model, position, c);
            return hit && c ? usages(model, c, hit.def.name).map(range => ({ uri: model.uri, range })) : [];
        },
    });
    monaco.languages.registerHoverProvider(SHEET_LANGUAGE, {
        provideHover(model, position) {
            const hit = definitionAt(model, position, compileSheetNow(model.getValue()));
            if (!hit) return null;
            const text = model.getLineContent(hit.def.line + 1).trim();
            const lines = ['```\n' + text + '\n```', `Defined on line ${hit.def.line + 1}.`];
            if (hit.def.calcpad !== hit.def.name) lines.push(`Calcpad name: \`${hit.def.calcpad}\``);
            return { range: new monaco.Range(position.lineNumber, hit.word.startColumn, position.lineNumber, hit.word.endColumn), contents: lines.map(value => ({ value })) };
        },
    });
    monaco.languages.registerDocumentSemanticTokensProvider(SHEET_LANGUAGE, {
        getLegend: () => ({ tokenTypes: ['variable'], tokenModifiers: ['declaration'] }),
        provideDocumentSemanticTokens(model) {
            const c = compileSheetNow(model.getValue());
            const data: number[] = [];
            if (!c) {
                void loadSheet();
                return { data: new Uint32Array(0) };
            }
            let prevLine = 0;
            let prevCol = 0;
            const names = new Map(c.defs.map(d => [d.name, d]));
            for (let i = 0; i < model.getLineCount(); i++) {
                const k = c.kinds[i];
                if (!k || 'bp'.includes(k)) continue;
                let text = model.getLineContent(i + 1);
                const note = k === 'q' ? text.search(/(?<!\|)\|(?!\|)/) : -1;
                if (note >= 0) text = text.slice(0, note);
                for (const m of text.matchAll(new RegExp(`(?<!${NAME_CHARS})[\\p{L}_]${NAME_CHARS}*`, 'gu'))) {
                    const d = names.get(m[0]);
                    if (!d) continue;
                    const col = m.index!;
                    data.push(i - prevLine, i === prevLine ? col - prevCol : col, m[0].length, 0, d.line === i && col === text.search(/\S/) ? 1 : 0);
                    prevLine = i;
                    prevCol = col;
                }
            }
            return { data: new Uint32Array(data) };
        },
        releaseDocumentSemanticTokens() {},
    });
    monaco.languages.registerCompletionItemProvider(SHEET_LANGUAGE, {
        provideCompletionItems(model, position) {
            const word = model.getWordUntilPosition(position);
            const range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: word.startColumn, endColumn: word.endColumn };
            const out: monaco.languages.CompletionItem[] = [];
            const compiled = compileSheetNow(model.getValue());
            if (!compiled) void loadSheet();
            // Typing a word offers names and functions in math lines only; Ctrl+Space with no word typed offers all.
            const asked = word.word === '';
            const i = position.lineNumber - 1;
            const lines = model.getLinesContent();
            if (!asked) {
                if (!/^[\p{L}_]/u.test(word.word) || verbatimLines(lines.slice(0, i + 1))[i]) return { suggestions: [] };
                if (!inMathContext(compiled?.kinds[i], lines[i].slice(0, position.column - 1))) return { suggestions: [] };
            }
            for (const d of compiled?.defs ?? []) {
                if (!asked && d.line >= i) continue;
                out.push({
                    label: d.name, kind: monaco.languages.CompletionItemKind.Variable, insertText: d.name, range, sortText: '0' + d.name,
                    detail: d.calcpad === d.name ? 'defined in this sheet' : `Calcpad name: ${d.calcpad}`,
                    documentation: { value: '```\n' + (lines[d.line] ?? '').trim() + '\n```' },
                });
            }
            for (const f of builtinFunctions()) {
                out.push({
                    label: f.name, kind: monaco.languages.CompletionItemKind.Function, insertText: f.name + '($0)', range, sortText: '2' + f.name,
                    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
                    detail: f.detail, documentation: { value: f.doc },
                });
            }
            for (const g of GREEK) {
                out.push({ label: g, kind: monaco.languages.CompletionItemKind.Text, insertText: g, range, detail: 'Greek letter' });
            }
            const snip = (label: string, text: string, detail: string) => out.push({
                label, kind: monaco.languages.CompletionItemKind.Snippet, insertText: text, range, detail,
                insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            });
            snip('if', 'if ${1:condition}:\n\t$0', 'conditional block');
            snip('else', 'else:\n\t$0', 'else branch');
            snip('for', 'for ${1:i} in ${2:1}..${3:10}:\n\t$0', 'loop over a range');
            snip('while', 'while ${1:condition}:\n\t$0', 'while loop');
            snip('plot', 'plot ${1:f(x)} for ${2:x} in ${3:0}..${4:10}', 'plot');
            return { suggestions: out };
        },
    });
}

/** Sets a model's language from its text: a sheet when the first line is the marker. */
export function syncModelLanguage(model: monaco.editor.ITextModel): void {
    const want = isSheetText(model.getValue()) ? SHEET_LANGUAGE : 'calcpad';
    if (model.getLanguageId() !== want) monaco.editor.setModelLanguage(model, want);
}
