import * as monaco from 'monaco-editor';
import { buildInsertSnippet, hasSnippetPlaceholders } from 'calcpad-frontend/text/snippet-insert';
import type { InsertItem } from 'calcpad-frontend/types/snippets';
import { getActiveDocumentKey, type EditorBridge } from './bridge';
import { parseDirectiveLine } from 'calcpad-frontend';
import { buildBuiltinDocMarkdown, extractFunctionName } from './builtin-docs';
import { compileSheetNow, loadSheet } from '../wasm/sheet';
import { toSheetExpr } from '../wasm/sheet-insert';
import { completionContext, type Syntax } from './nav/context';
import { scanSheet } from './nav/doc-index';
import { cachedValues, formatValue, loadValues } from './nav/values';
import type { VarValue } from './nav/values';
import type { FileContextProvider } from './diagnostics';

/**
 * One completion provider for both editors (Calcpad worksheets and sheets). What it offers depends
 * on where the cursor is (nav/context.ts): names defined ABOVE the cursor with their last calculated
 * value, the parameters of the function being written, built-in functions with signature and a
 * one-line description, units after a number, `#` directives and `$` commands in a Calcpad
 * worksheet, block keywords and format markers at the start of a sheet line. In a comment, a text
 * line, a string, a note, a Markdown/HTML/Python block or sheet prose it offers nothing.
 *
 * The built-in entries are the editor's snippet library (the data the Insert tab and the generated
 * function reference use), so what completes is what the reference lists.
 */
export function registerCompletionProvider(bridge: EditorBridge, getFileContext?: FileContextProvider): monaco.IDisposable {
    const make = (syntax: Syntax): monaco.languages.CompletionItemProvider => ({
        triggerCharacters: syntax === 'calcpad' ? ['#', '$', '.'] : ['>', '`', '<', '$', '-'],
        async provideCompletionItems(model, position) {
            const items = await complete(bridge, getFileContext, model, position, syntax);
            return { suggestions: items };
        },
    });
    const a = monaco.languages.registerCompletionItemProvider('calcpad', make('calcpad'));
    const b = monaco.languages.registerCompletionItemProvider('calcpad-sheet', make('sheet'));
    return { dispose() { a.dispose(); b.dispose(); } };
}

const K = monaco.languages.CompletionItemKind;
const SNIPPET = monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet;

interface Builtins {
    functions: InsertItem[];
    constants: InsertItem[];
    directives: InsertItem[];
    commands: InsertItem[];
    settings: InsertItem[];
    units: InsertItem[];
}

let builtinCache: { src: InsertItem[]; sets: Builtins } | null = null;

/** The snippet library sorted into the groups completion uses. */
export function builtins(bridge: EditorBridge): Builtins | null {
    if (!bridge.snippets.isLoaded()) return null;
    const src = bridge.snippets.getAllItems();
    if (builtinCache && builtinCache.src === src) return builtinCache.sets;
    const sets: Builtins = { functions: [], constants: [], directives: [], commands: [], settings: [], units: [] };
    for (const item of src) {
        switch (item.keywordType) {
            case 'Function': sets.functions.push(item); break;
            case 'Constant': sets.constants.push(item); break;
            case 'Keyword': case 'ControlBlockKeyword': case 'EndKeyword': sets.directives.push(item); break;
            case 'Command': sets.commands.push(item); break;
            case 'Setting': sets.settings.push(item); break;
            case 'Unit': sets.units.push(item); break;
            default: break;
        }
    }
    builtinCache = { src, sets };
    return sets;
}

const docCache = new WeakMap<InsertItem, string>();
function docOf(item: InsertItem): string {
    let d = docCache.get(item);
    if (d === undefined) { d = buildBuiltinDocMarkdown(item); docCache.set(item, d); }
    return d;
}

/** `(x; y)` for a built-in function. */
export function signature(item: InsertItem, sheet: boolean): string {
    const ps = item.parameters ?? [];
    return `(${ps.map(p => (p.isVariadic ? '...' : '') + p.name).join(sheet ? ', ' : '; ')})`;
}

const GREEK = ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta', 'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'pi', 'rho', 'sigma', 'tau', 'upsilon', 'phi', 'chi', 'psi', 'omega'];

interface Named { name: string; detail: string; value?: VarValue; line: number; doc: string; params?: string[]; kind: 'variable' | 'function' | 'macro' | 'param' }

function userItem(n: Named, range: monaco.IRange, sheet: boolean): monaco.languages.CompletionItem {
    const val = formatValue(n.value);
    const fn = n.kind === 'function' || n.kind === 'macro';
    const sig = fn ? `(${(n.params ?? []).join(sheet ? ', ' : '; ')})` : '';
    const sort = n.kind === 'param' ? '0' : n.kind === 'variable' ? '1' : '2';
    const insert = fn
        ? `${n.name}(${(n.params ?? []).map((p, i) => '${' + (i + 1) + ':' + p.replace(/[\\$}]/g, '\\$&') + '}').join(sheet ? ', ' : '; ')})`
        : n.name;
    return {
        // The value goes right after the name so a narrow list still shows it; the description is on the right.
        label: fn ? { label: n.name, detail: sig, description: n.detail } : { label: n.name, detail: val ? ` = ${val}` : '', description: val ? undefined : n.detail },
        kind: n.kind === 'param' ? K.TypeParameter : n.kind === 'variable' ? K.Variable : K.Function,
        insertText: insert,
        insertTextRules: fn ? SNIPPET : undefined,
        detail: n.kind === 'param' ? 'parameter' : `${n.kind}${n.line >= 0 ? `, line ${n.line + 1}` : ''}`,
        documentation: { value: n.doc },
        sortText: sort + n.name,
        filterText: n.name,
        range,
    };
}

function userDoc(name: string, expr: string | undefined, value: VarValue | undefined, sig = ''): string {
    let d = '```calcpad\n' + name + sig + (expr ? ' = ' + expr : '') + '\n```';
    const v = formatValue(value);
    if (v) d += `\n\nLast calculated value: \`${v}\``;
    return d;
}

/** The names defined above the cursor in a Calcpad worksheet, from the engine's definitions. */
function calcpadNames(bridge: EditorBridge, line0: number, values: Map<string, VarValue> | null): Named[] {
    const defs = bridge.definitions.getCachedDefinitions(getActiveDocumentKey());
    const out: Named[] = [];
    if (!defs) return out;
    const above = (src: string, n: number) => src !== 'local' || n < line0;
    const seen = new Set<string>();
    for (const m of defs.macros) {
        if (!above(m.source, m.lineNumber) || seen.has(m.name)) continue;
        seen.add(m.name);
        out.push({ name: m.name, kind: 'macro', detail: m.description || 'macro', line: m.source === 'local' ? m.lineNumber : -1, params: m.parameters, doc: userDoc(m.name, undefined, undefined, `(${m.parameters.join('; ')})`) });
    }
    for (const f of defs.functions) {
        if (!above(f.source, f.lineNumber) || seen.has(f.name)) continue;
        seen.add(f.name);
        out.push({ name: f.name, kind: 'function', detail: f.description || f.expression || 'function', line: f.source === 'local' ? f.lineNumber : -1, params: f.parameters.map(p => p.split(':')[0].trim()), doc: userDoc(f.name, f.expression, undefined, `(${f.parameters.join('; ')})`) });
    }
    for (const v of defs.variables) {
        if (!above(v.source, v.lineNumber) || seen.has(v.name)) continue;
        seen.add(v.name);
        const value = values?.get(v.name);
        out.push({ name: v.name, kind: 'variable', detail: v.description || (v.expression ? v.expression : 'variable'), value, line: v.source === 'local' ? v.lineNumber : -1, doc: userDoc(v.name, v.expression, value) });
    }
    return out;
}

/** The names a sheet defines above the cursor. */
function sheetNames(text: string, lines: string[], line0: number, values: Map<string, VarValue> | null): Named[] | null {
    const c = compileSheetNow(text);
    if (!c) { void loadSheet(); return null; }
    const idx = scanSheet(lines, c);
    const spelled = new Map(c.defs.map(d => [d.name, d.calcpad]));
    const out: Named[] = [];
    for (const s of idx.syms.values()) {
        if (s.local || s.defLine < 0 || s.defLine >= line0) continue;
        if (!spelled.has(s.name)) continue;
        const defText = lines[s.defLine] ?? '';
        const head = /^\s*[\p{L}][\p{L}\p{N}_′″₀-₉]*(?:\(([^)]*)\))?\s*(?::|=(?!=))\s*(.*)$/u.exec(defText);
        const params = head?.[1] !== undefined ? head[1].split(',').map(p => p.trim()).filter(Boolean) : undefined;
        const note = defText.indexOf('|') >= 0 ? defText.slice(defText.indexOf('|') + 1).trim() : '';
        const value = values?.get(spelled.get(s.name)!);
        const expr = head?.[2]?.replace(/\s*\|.*$/, '').replace(/\s*->.*$/, '');
        out.push({
            name: s.name, kind: params ? 'function' : 'variable', detail: note || expr || 'defined in this sheet', value,
            line: s.defLine, params, doc: userDoc(s.name, expr, value, params ? `(${params.join(', ')})` : ''),
        });
    }
    return out;
}

async function complete(
    bridge: EditorBridge,
    getFileContext: FileContextProvider | undefined,
    model: monaco.editor.ITextModel,
    position: monaco.Position,
    syntax: Syntax,
): Promise<monaco.languages.CompletionItem[]> {
    const lineToCursor = model.getLineContent(position.lineNumber).substring(0, position.column - 1);
    // Inside a #include/#read/#write path only the file suggestions apply.
    if (syntax === 'calcpad' && parseDirectiveLine(lineToCursor)) return [];
    const lines = model.getLinesContent();
    const text = lines.join('\n');
    const compiled = syntax === 'sheet' ? compileSheetNow(text) : null;
    if (syntax === 'sheet' && !compiled) void loadSheet();
    const line0 = position.lineNumber - 1;
    const col0 = position.column - 1;
    const ctx = completionContext(lines, line0, col0, syntax, compiled);
    if (ctx.kind === 'none') return [];
    const sheet = syntax === 'sheet';
    const sets = builtins(bridge);

    if (ctx.kind === 'marker') {
        const range = { startLineNumber: position.lineNumber, endLineNumber: position.lineNumber, startColumn: ctx.start + 1, endColumn: position.column };
        return markerItems(range, line0 === 0);
    }
    const range: monaco.IRange = {
        startLineNumber: position.lineNumber, endLineNumber: position.lineNumber,
        startColumn: ctx.start + 1, endColumn: position.column,
    };
    const out: monaco.languages.CompletionItem[] = [];

    // Units after a number.
    if (ctx.unit) {
        for (const u of sets?.units ?? []) {
            out.push({ label: { label: u.tag, description: u.description }, kind: K.Unit, insertText: u.tag, detail: u.categoryPath ?? 'unit', documentation: { value: `**${u.description}**` }, sortText: '0' + u.tag, filterText: u.tag, range });
        }
        const defs = syntax === 'calcpad' ? bridge.definitions.getCachedDefinitions(getActiveDocumentKey()) : undefined;
        for (const u of defs?.customUnits ?? []) {
            out.push({ label: { label: u.name, description: u.expression ? `= ${u.expression}` : 'custom unit' }, kind: K.Unit, insertText: u.name, detail: 'custom unit', sortText: '0' + u.name, filterText: u.name, range });
        }
        return out;
    }

    // Values: the last calculated ones are used when there are any; a calculation is started
    // (or waited for, briefly) when the text has moved on.
    const key = model.uri.toString();
    const ctxFile = getFileContext ? await getFileContext(text).catch(() => ({} as { sourceFilePath?: string })) : {};
    const values = await loadValues(key, text, ctxFile.sourceFilePath, 250) ?? cachedValues(key);

    // The parameters of the function being written on this line.
    const before = lines[line0].slice(0, col0);
    const hm = sheet
        ? /^\s*[\p{L}][\p{L}\p{N}_′″₀-₉]*\(([^)]*)\)\s*(?::|=(?!=))/u.exec(before)
        : /^\s*[\p{L}][\p{L}\p{N}_′″₀-₉]*\(([^)]*)\)\s*=(?!=)/u.exec(before);
    if (hm) {
        for (const p of hm[1].split(sheet ? ',' : ';').map(x => x.trim().split(':')[0].replace(/[$?]$/, '')).filter(Boolean)) {
            out.push(userItem({ name: p, kind: 'param', detail: 'parameter', line: line0, doc: `Parameter of the function on this line.` }, range, sheet));
        }
    }

    const names = sheet ? sheetNames(text, lines, line0, values) ?? [] : calcpadNames(bridge, line0, values);
    if (!ctx.directive) {
        for (const n of names) out.push(userItem(n, range, sheet));
    }

    if (sets && !ctx.directive) {
        for (const f of sets.functions) {
            const name = extractFunctionName(f.tag);
            if (!name) continue;
            const snip = hasSnippetPlaceholders(f) ? buildInsertSnippet(f) : f.tag;
            out.push({
                label: { label: name, detail: signature(f, sheet), description: f.description },
                kind: K.Function,
                insertText: sheet ? toSheetExpr(snip) : snip,
                insertTextRules: hasSnippetPlaceholders(f) ? SNIPPET : undefined,
                detail: f.categoryPath ?? 'function',
                documentation: { value: docOf(f) },
                sortText: '3' + name,
                filterText: name,
                range,
            });
        }
        for (const c of sets.constants) {
            out.push({ label: { label: c.tag, description: c.description }, kind: K.Constant, insertText: c.tag, detail: 'constant', documentation: { value: `**${c.description}**` }, sortText: '4' + c.tag, filterText: c.tag, range });
        }
    }
    if (sets && syntax === 'calcpad') {
        if (ctx.directive || ctx.lineStart) {
            for (const d of sets.directives) {
                if (!d.tag.startsWith('#')) continue;
                out.push(directiveItem(d, range));
            }
        }
        if (ctx.directive || ctx.lineStart) {
            for (const d of sets.commands) out.push(directiveItem(d, range));
        }
        if (ctx.lineStart && !ctx.directive) {
            for (const s of sets.settings) out.push({ label: { label: s.tag.split(/\s*=/)[0], description: s.description }, kind: K.Property, insertText: s.tag, detail: s.categoryPath ?? 'setting', sortText: '7' + s.tag, filterText: s.tag.split(/\s*=/)[0], range });
        }
    }
    if (sheet && !ctx.directive) {
        for (const g of GREEK) {
            out.push({ label: { label: g, description: 'Greek letter' }, kind: K.Text, insertText: g, detail: 'Greek letter', sortText: '8' + g, range });
        }
    }
    return out;
}

function directiveItem(d: InsertItem, range: monaco.IRange): monaco.languages.CompletionItem {
    const asSnippet = hasSnippetPlaceholders(d);
    const label = d.tag.split('\n')[0].replace(/§/g, '').replace(/\s+/g, ' ').trim();
    const word = /^[#$]\w*(?:\s+\w+)?/.exec(label)?.[0] ?? label;
    return {
        label: { label: word, description: d.description },
        kind: d.tag.startsWith('$') ? K.Function : K.Keyword,
        insertText: asSnippet ? buildInsertSnippet(d) : d.tag,
        insertTextRules: asSnippet ? SNIPPET : undefined,
        detail: d.categoryPath ?? 'directive',
        documentation: { value: docOf(d) },
        sortText: '6' + d.tag,
        filterText: word,
        range,
    };
}

/** Block keywords and format markers at the start of a sheet line. */
function markerItems(range: monaco.IRange, first: boolean): monaco.languages.CompletionItem[] {
    const m = (label: string, text: string, detail: string, doc: string, kind: monaco.languages.CompletionItemKind = K.Snippet, filter = label): monaco.languages.CompletionItem => ({
        label: { label, description: detail }, kind, insertText: text, insertTextRules: SNIPPET, detail, documentation: { value: doc }, sortText: '0' + label, filterText: filter, range,
    });
    const items = [
        m('if', 'if ${1:condition}:\n\t$0', 'conditional block', 'A block that runs when the condition holds. One line: `if a > 1: "OK" else: "No"`.', K.Keyword),
        m('else if', 'else if ${1:condition}:\n\t$0', 'another condition', 'The next condition of an `if` block.', K.Keyword),
        m('else', 'else:\n\t$0', 'else branch', 'The branch of an `if` block that runs when no condition held.', K.Keyword),
        m('for', 'for ${1:i} in ${2:1}..${3:10}:\n\t$0', 'loop over a range', 'Repeats the indented body. `for i in 1..5 step 2:` counts by 2; `for x in v:` runs over a vector.', K.Keyword),
        m('while', 'while ${1:condition}:\n\t$0', 'while loop', 'Repeats the indented body while the condition holds.', K.Keyword),
        m('repeat', 'repeat ${1:n}:\n\t$0', 'repeat n times', 'Repeats the indented body n times.', K.Keyword),
        m('plot', 'plot ${1:f(x)} for ${2:x} in ${3:0}..${4:10}', 'plot', 'Draws one or more expressions over a range: `plot sin(x), cos(x) for x in 0..2*pi`.', K.Keyword),
        m('>>', '>> ', 'raw Calcpad line', 'A line passed to the engine as written, so every Calcpad feature stays reachable.', K.Operator),
        m('```calcpad', '```calcpad\n$0\n```', 'raw Calcpad block', 'Lines between the fences are passed to the engine as written.', K.Snippet),
        m('```python', '```python\n$0\n```', 'Python block', 'A Python block: what it prints and draws joins the sheet, and `export(name=value)` defines a sheet variable.', K.Snippet),
        m('$$', '$$\n$0\n$$', 'display formula', 'A LaTeX formula on lines of its own.', K.Snippet),
        m('input', '${1:name}: ? ${2:6} ${3:m} | ${4:note}', 'number input', 'An input a reader of the sheet can change: a number with its unit and a note.', K.Snippet),
        m('dropdown', '${1:name}: ? [${2:Steel} = ${3:200}, *${4:Timber} = ${5:11}] | ${6:note}', 'drop-down input', 'A list of `label = value`; the option marked `*` starts selected.', K.Snippet),
        m('radio', '${1:name}: ? (${2:Low} = ${3:1}, ${4:High} = ${5:2}) | ${6:note}', 'radio input', 'The same options in round brackets give radio buttons.', K.Snippet),
        m('checkbox', '${1:name}: ? ${2:on} | ${3:note}', 'checkbox input', '`on` or `off`; the name is 1 or 0.', K.Snippet),
        m('break', 'break', 'leave the loop', 'Leaves the enclosing loop.', K.Keyword),
        m('continue', 'continue', 'next pass', 'Goes on with the next pass of the enclosing loop.', K.Keyword),
    ];
    if (first) items.unshift(m('<!--sheet-->', '<!--sheet-->', 'sheet marker', 'The first line of a sheet; it makes the file a sheet rather than a Calcpad worksheet.', K.Keyword));
    return items;
}

