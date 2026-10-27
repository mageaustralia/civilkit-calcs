// The text side of `#python` blocks (docs/python-blocks.md): finding the blocks in a worksheet, turning what a
// block produced into Calcpad lines, and mapping the source line numbers of the engine's answer back to the
// worksheet the user wrote. No I/O here; python-runner.ts runs the code and transport.ts drives it.
//
// The grammar is kept in step with calcpad-highlighter/src/python_blocks.rs (the editor side):
//   #python                a line holding only this (any case, surrounding blanks allowed) opens a block
//   #end python            a line holding only this closes it; no nesting; a block left open runs to the end

export interface PyBlock {
    /** 0-based line of `#python`. */
    start: number;
    /** 0-based line of `#end python`, or -1 when the block is left open. */
    end: number;
    /** The block's code lines, joined with \n. */
    code: string;
    /** 0-based line of the first code line. */
    first: number;
}

const OPEN = /^\s*#python\s*$/i;
const CLOSE = /^\s*#end\s+python\s*$/i;

/** Cheap test for the lazy load: does the text hold a `#python` line at all? */
export function hasPythonBlock(text: string): boolean {
    return /^[ \t]*#python[ \t]*\r?$/im.test(text);
}

export function findPythonBlocks(text: string): PyBlock[] {
    const lines = text.split('\n');
    const blocks: PyBlock[] = [];
    for (let i = 0; i < lines.length; i++) {
        if (!OPEN.test(lines[i])) continue;
        let end = -1;
        for (let j = i + 1; j < lines.length; j++) if (CLOSE.test(lines[j])) { end = j; break; }
        const stop = end < 0 ? lines.length : end;
        blocks.push({ start: i, end, code: lines.slice(i + 1, stop).join('\n').replace(/\r/g, ''), first: i + 1 });
        i = stop;
    }
    return blocks;
}

export type ExportValue = number | number[] | number[][];

export interface BlockError {
    type: string;
    message: string;
    /** 0-based worksheet line the error points at, when the traceback names a line of the block. */
    line: number | null;
}

export interface BlockResult {
    /** print() text and figures, in the order the script made them. */
    parts: ({ text: string } | { svg: string } | { plotly: string; title: string } | { panel: string })[];
    exports: { name: string; value: ExportValue; hidden?: boolean }[];
    error: BlockError | null;
    /** The output or the figure count hit its cap. */
    truncated: boolean;
    /** The run these results belong to (python-runner.ts); a panel's page quotes it when it asks for a recalculation. */
    runKey?: string;
}

const escapeHtml = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');

/** A Calcpad number: JS exponent notation becomes `m*10^e`, which the engine reads. */
export function formatNumber(n: number): string {
    const s = String(n);
    const m = /^(-?[\d.]+)e([+-]?\d+)$/i.exec(s);
    return m ? `${m[1]}*10^${Number(m[2])}` : s;
}

/** True for a name Calcpad accepts as a variable: letters (any script), digits and underscores, not starting with a digit or `_`. */
export function validName(name: string): boolean {
    return /^[\p{L}][\p{L}\p{N}_]*$/u.test(name);
}

/** `name = value`, `name = [a; b; c]` (vector), `name = [a; b| c; d]` (matrix). */
export function definitionLine(name: string, value: ExportValue): string {
    if (typeof value === 'number') return `${name} = ${formatNumber(value)}`;
    if (Array.isArray(value[0])) return `${name} = [${(value as number[][]).map((r) => r.map(formatNumber).join('; ')).join('| ')}]`;
    return `${name} = [${(value as number[]).map(formatNumber).join('; ')}]`;
}

/** Base64 of the text's UTF-8 bytes: nothing in it can be read as Calcpad or HTML syntax. */
function base64Utf8(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let bin = '';
    for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return btoa(bin);
}

const NOTE_STYLE = 'margin:4pt 0;padding:4pt 8pt;border-left:3pt solid #8a8a8a;background:rgba(128,128,128,.12);font-family:sans-serif;font-size:10pt';

/**
 * A Plotly figure from `fig.show()`: the container the page draws into (python-plotly.ts), and under it the note that stays
 * where the chart cannot be drawn (print, Word, Excel, no network). The figure JSON travels as base64 in an attribute.
 */
export function plotlyMarkup(figureJson: string, title: string): string {
    const name = title.replace(/<[^>]*>/g, '').trim();
    const note = `Interactive chart${name ? `: ${escapeHtml(name)}` : ''}. It is drawn in the browser and is not carried by print or export.`;
    return `<div class="cprs-plotly-wrap" style="margin:4pt 0"><div class="cprs-plotly" data-fig="${base64Utf8(figureJson)}"></div>`
        + `<p class="cprs-plotly-note" style="${NOTE_STYLE}" data-title="${escapeHtml(name)}">${note}</p></div>`;
}

interface PanelWidgetSpec { kind: string; name: string; label: string; value: number | boolean; unit?: string; options?: string[] }
type PanelItem = { t: 'plotly'; fig: unknown } | { t: 'text'; s: string } | { t: 'table'; head: string[] | null; rows: string[][] };

/** What a widget shows as text: `L = 6 m`, `material = Steel`, `ribs = yes`. */
export function panelWidgetText(w: PanelWidgetSpec): string {
    if (w.kind === 'select') return `${w.label} = ${w.options?.[w.value as number] ?? w.value}`;
    if (w.kind === 'checkbox') return `${w.label} = ${w.value ? 'yes' : 'no'}`;
    return `${w.label} = ${w.value}${w.unit ? ` ${w.unit}` : ''}`;
}

/** One panel output as static markup: this is what print, Word and Excel carry, and what the page shows before it is scripted. */
function panelItemMarkup(it: PanelItem): string {
    if (it.t === 'text') return `<p class="cprs-panel-text" style="margin:4pt 0">${escapeHtml(it.s).replace(/\n/g, '&#10;')}</p>`;
    if (it.t === 'table') {
        const head = it.head ? `<thead><tr>${it.head.map((h) => `<th>${escapeHtml(h)}</th>`).join('')}</tr></thead>` : '';
        const rows = it.rows.map((r) => `<tr>${r.map((c) => `<td>${escapeHtml(c)}</td>`).join('')}</tr>`).join('');
        return `<table class="cprs-panel-table">${head}<tbody>${rows}</tbody></table>`;
    }
    const title = (it.fig as { layout?: { title?: unknown } })?.layout?.title;
    const t = typeof title === 'string' ? title : (title as { text?: unknown } | undefined)?.text;
    return plotlyMarkup(JSON.stringify(it.fig), typeof t === 'string' ? t : '');
}

/**
 * An interactive panel (public/py/lib/ui.py, docs/interactive-panels-design.md). The markup is the panel at the values it was
 * calculated with: an empty container for the widgets (python-panels.ts draws them), the first output as static markup, and a
 * note that print and export keep. The spec travels as base64, so nothing in it can be read as Calcpad or HTML.
 */
export function panelMarkup(panelJson: string, runKey: string): string {
    let spec: { id: string; widgets: PanelWidgetSpec[]; first: { out: PanelItem[] } };
    try { spec = JSON.parse(panelJson); } catch { return '<p><span class="err">Python error: the panel could not be read.</span></p>'; }
    const out = spec.first.out.map(panelItemMarkup).join('');
    const note = `Interactive panel ${escapeHtml(spec.id)}: ${spec.widgets.map((w) => escapeHtml(panelWidgetText(w))).join(', ')}. It is drawn in the browser; print and export show it at these values.`;
    const wire = JSON.stringify({ id: spec.id, widgets: spec.widgets, types: spec.first.out.map((o) => o.t) });
    return `<div class="cprs-panel" data-panel="${escapeHtml(spec.id)}" data-run="${escapeHtml(runKey)}" data-spec="${base64Utf8(wire)}" style="margin:6pt 0">`
        + `<div class="cprs-panel-ui"></div><div class="cprs-panel-out">${out}</div>`
        + `<p class="cprs-panel-note" style="${NOTE_STYLE}">${note}</p></div>`;
}

const PY_LINE = (n: number) => `@@PY:${n}@@`;

/** Replaces the line placeholders of Python error lines; `map` turns a line of the engine's text into the user's line. */
export function resolvePythonLines(html: string, map: (line: number) => number): string {
    return html.replace(/@@PY:(\d+)@@/g, (_m, n: string) => String(map(Number(n))));
}

const PRE_STYLE = 'font-family:monospace;white-space:pre-wrap;margin:4pt 0;padding:4pt 8pt;border-left:3pt solid #8a8a8a;background:rgba(128,128,128,.12)';

/**
 * The lines that stand in for a block in the text the engine reads: one text line holding the output (print text in a
 * `<pre>`, figures as inline SVG, an error), then one definition line per exported name. The text line uses `'`, so it
 * holds no single quote and no line break.
 */
export function replacementLines(res: BlockResult, blockLine: number): string[] {
    const html: string[] = [];
    for (const p of res.parts) {
        if ('text' in p) {
            if (p.text.length) html.push(`<pre class="cprs-py-out" style="${PRE_STYLE}">${escapeHtml(p.text).replace(/\n/g, '&#10;')}</pre>`);
        } else if ('panel' in p) html.push(panelMarkup(p.panel, res.runKey ?? ''));
        else if ('plotly' in p) html.push(plotlyMarkup(p.plotly, p.title));
        else html.push(`<div class="cprs-py-fig" style="margin:4pt 0">${p.svg}</div>`);
    }
    if (res.truncated) html.push('<p><i>Python output was cut at its size limit.</i></p>');
    const lines: string[] = [];
    if (res.error) {
        // The line is written as a placeholder: it is a line of the text given to the engine, and resolvePythonLines turns it into
        // the user's own line (the sheet line, in a sheet) once the page is back.
        const where = res.error.line != null ? ` on line ${PY_LINE(res.error.line + 1)}` : ` in the block at line ${PY_LINE(blockLine + 1)}`;
        html.push(`<p><span class="err">Python error${where}: ${escapeHtml(res.error.type ? `${res.error.type}: ` : '')}${escapeHtml(res.error.message)}</span></p>`);
    }
    if (html.length) lines.push(`'${html.join('')}`);
    if (!res.error) {
        // export_data: defined but not shown (large arrays, data for a chart).
        const hidden = res.exports.filter((e) => e.hidden);
        if (hidden.length) lines.push('#hide', ...hidden.map((e) => definitionLine(e.name, e.value)), '#show');
        for (const e of res.exports) if (!e.hidden) lines.push(definitionLine(e.name, e.value));
    }
    return lines;
}

export interface Substitution {
    /** The worksheet with each block replaced by its results. */
    content: string;
    /** For each line of `content` (0-based), the 0-based worksheet line it stands for. */
    map: number[];
}

export function substitute(content: string, blocks: PyBlock[], results: BlockResult[]): Substitution {
    const lines = content.split('\n');
    const out: string[] = [];
    const map: number[] = [];
    let at = 0;
    blocks.forEach((b, k) => {
        for (; at < b.start; at++) { out.push(lines[at]); map.push(at); }
        for (const l of replacementLines(results[k], b.start)) { out.push(l); map.push(b.start); }
        at = (b.end < 0 ? lines.length : b.end + 1);
    });
    for (; at < lines.length; at++) { out.push(lines[at]); map.push(at); }
    return { content: out.join('\n'), map };
}

/**
 * The engine numbers lines of the text it was given; the editor needs the worksheet's own. Rewrites the line numbers of
 * the page (`data-source-line`, `data-ui-line`, `data-line`, error links) and of the error list. A line number the map
 * does not know (past the end, from an included file) is left as it is.
 */
export function remapPage(html: string, map: number[]): string {
    const m = (s: string) => { const i = Number(s) - 1; return i >= 0 && i < map.length ? String(map[i] + 1) : s; };
    return html
        .replace(/(data-source-line="|data-ui-line=")(\d+)(")/g, (_a, p, n, q) => p + m(n) + q)
        .replace(/(<a\b[^>]*?\bdata-text=")(\d+)("[^>]*>)(\d+)(<\/a>)/g, (_a, p, n, q, v, e) => p + m(n) + q + m(v) + e)
        .replace(/(<a\b[^>]*?\bhref="#line-)(\d+)("[^>]*?\bdata-text=")(\d+)("[^>]*>)(\d+)(<\/a>)/g, (_a, p, n, q, d, r, v, e) => p + m(n) + q + m(d) + r + m(v) + e)
        .replace(/(\bdata-line=")(\d+)("[^>]*>)(\d+)(<\/span>)/g, (_a, p, n, q, v, e) => p + m(n) + q + m(v) + e);
}

export interface EngineError { message?: string; sourceLine?: number; outputLine?: number; source?: string; [k: string]: unknown }

export function remapErrors(errors: EngineError[], map: number[]): EngineError[] {
    return errors.map((e) => {
        const i = (e.sourceLine ?? 0) - 1;
        return i >= 0 && i < map.length ? { ...e, sourceLine: map[i] + 1 } : e;
    });
}

/** The entries a block's error adds to the engine's error list (lines are 1-based there). */
export function pythonErrors(blocks: PyBlock[], results: BlockResult[]): EngineError[] {
    const out: EngineError[] = [];
    results.forEach((r, k) => {
        if (!r.error) return;
        out.push({
            message: `Python ${r.error.type ? `${r.error.type}: ` : ''}${r.error.message}`,
            sourceLine: (r.error.line ?? blocks[k].start) + 1,
            outputLine: 0,
            source: 'Python',
        });
    });
    return out;
}

/** A MicroPython traceback read for the worksheet: the exception, and the innermost frame that is the block's own code. */
export function readTraceback(text: string): { type: string; message: string; line: number | null } {
    const lines = String(text ?? '').replace(/\s+$/, '').split(/\r?\n/);
    const last = lines[lines.length - 1] ?? '';
    const m = /^(\w+(?:\.\w+)*): ?(.*)$/.exec(last);
    let line: number | null = null;
    for (const l of lines) {
        const f = /^\s*File "<python>", line (\d+)/.exec(l);
        if (f) line = Number(f[1]) - 1;
    }
    return { type: m ? m[1] : '', message: m ? m[2] : last, line };
}
