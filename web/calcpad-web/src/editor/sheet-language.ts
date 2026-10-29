import * as monaco from 'monaco-editor';
import { compileSheetNow, formatSheet, isSheetText, loadSheet } from '../wasm/sheet';
import type { SheetCompiled } from '../wasm/sheet';
import { occAt, scanSheet } from './nav/doc-index';

/** Monaco language id of a sheet (docs/sheet-syntax.md). Calcpad's own providers do not apply to it. */
export const SHEET_LANGUAGE = 'calcpad-sheet';

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
            [/\|\s.*?(?=\s*\{key\}\s*$|$)/, 'comment'],
            [/\{key\}\s*$|^\s*\{key results\}\s*$/, 'keyword'],
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

let registered = false;

/** Registers the sheet language and its tokenizer, formatter and navigation. Call once with the Calcpad language. */
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
        // The same scan rename uses (nav/doc-index.ts): code only, with a parameter's scope.
        provideReferences(model, position) {
            const c = compileSheetNow(model.getValue());
            if (!c) return [];
            const idx = scanSheet(model.getLinesContent(), c);
            const occ = occAt(idx, position.lineNumber - 1, position.column - 1);
            const sym = occ && idx.syms.get(occ.sym);
            return sym ? sym.occs.map(o => ({ uri: model.uri, range: new monaco.Range(o.line + 1, o.start + 1, o.line + 1, o.end + 1) })) : [];
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
    // Completion: one provider for both languages, in completions.ts.
}

/** Sets a model's language from its text: a sheet when the first line is the marker. */
export function syncModelLanguage(model: monaco.editor.ITextModel): void {
    const want = isSheetText(model.getValue()) ? SHEET_LANGUAGE : 'calcpad';
    if (model.getLanguageId() !== want) monaco.editor.setModelLanguage(model, want);
}
