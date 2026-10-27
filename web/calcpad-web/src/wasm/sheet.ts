// calcpad-rs: sheet syntax (our addition, not part of CalcpadCE). A sheet is Markdown-like text
// that compiles to the Calcpad worksheet language, so the one engine runs both. The compiler is
// the Rust crate civilkit-calcs-sheet in its own small wasm (about 40 KB gzipped), loaded on
// first use. See docs/sheet-syntax.md.
//
// A file is a sheet when its first non-blank line is the marker below, or when its path ends in
// ".sheet" (the editor adds the marker when such a file is opened, so the text and the language
// mode always agree). The compile happens in the engine call path (transport.ts), for Preview,
// Report, Print PDF, Word and Excel, and for the linter; errors and markers are mapped back to
// the sheet lines through the compiler's source map.

export const SHEET_MARKER = '<!--sheet-->';

export interface SheetDiagnostic { line: number; severity: 'error' | 'warning'; message: string }
export interface SheetDef { name: string; calcpad: string; line: number }
export interface SheetCompiled {
    /** The Calcpad worksheet. */
    text: string;
    /** For each line of `text` (0 based), the sheet line (0 based) it came from. */
    map: number[];
    diagnostics: SheetDiagnostic[];
    /** The output carries formula placeholders (rendered by sheet-tex.ts). */
    latex: boolean;
    defs: SheetDef[];
    /** One letter per sheet line: b blank, p prose, q quantity, k block, r raw Calcpad, g plot, c condition. */
    kinds: string;
}

interface SheetModule {
    default: (input?: unknown) => Promise<unknown>;
    compile: (src: string, plainTex: boolean) => string;
    format: (src: string) => string;
}

/** The first non-blank line is the marker. Needs no wasm. */
export function isSheetText(content: string): boolean {
    for (const line of content.split('\n')) {
        if (line.trim() === '') continue;
        return line.trim().toLowerCase() === SHEET_MARKER;
    }
    return false;
}

/** A request is a sheet by its content or by the ".sheet" extension of its path. */
export function isSheetRequest(content: string, path?: string | null): boolean {
    return isSheetText(content) || /\.sheet$/i.test(path ?? '');
}

/** A sheet file's text as the editor holds it: the marker is added when missing. */
export function withMarker(content: string): string {
    return isSheetText(content) ? content : `${SHEET_MARKER}\n${content}`;
}

let module_: Promise<SheetModule> | null = null;
let ready: SheetModule | null = null;

/** Loads the compiler wasm (once). */
export function loadSheet(): Promise<SheetModule> {
    module_ ??= (async () => {
        const base = new URL(import.meta.env.BASE_URL || './', document.baseURI).href;
        const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_sheet_wasm.js`)) as SheetModule;
        await mod.default({ module_or_path: `${base}wasm/calcpad_sheet_wasm_bg.wasm` });
        ready = mod;
        return mod;
    })();
    module_.catch(() => { module_ = null; });
    return module_;
}

function parse(json: string): SheetCompiled {
    return JSON.parse(json) as SheetCompiled;
}

/** Compiles a sheet. `plainTex` writes formulas as italic TeX source (Word, Excel, Convert to Calcpad). */
export async function compileSheet(src: string, plainTex = false): Promise<SheetCompiled> {
    const mod = await loadSheet();
    return parse(mod.compile(src, plainTex));
}

/** The same, once the compiler is loaded; null before that. */
export function compileSheetNow(src: string, plainTex = false): SheetCompiled | null {
    return ready ? parse(ready.compile(src, plainTex)) : null;
}

/** The sheet line (1 based) of an output line (1 based); the line itself when out of range. */
export function sheetLine(c: SheetCompiled, outputLine: number): number {
    const m = c.map[outputLine - 1];
    return m === undefined ? outputLine : m + 1;
}

/**
 * Rewrites the line numbers the engine put in a report (source anchors, error links, the
 * error header) from output lines to sheet lines.
 */
export function remapHtml(html: string, c: SheetCompiled): string {
    const one = (n: string) => String(sheetLine(c, Number(n)));
    return html
        .replace(/(data-source-line=")(\d+)(")/g, (_m, a: string, n: string, b: string) => a + one(n) + b)
        .replace(/(\bid="line-)(\d+)(")/g, (_m, a: string, n: string, b: string) => a + one(n) + b)
        .replace(/(\bdata-line=")(\d+)(")/g, (_m, a: string, n: string, b: string) => a + one(n) + b)
        .replace(/(data-text=")(\d+)(">)(\d+)(<)/g, (_m, a: string, n: string, b: string, t: string) => a + one(n) + b + one(t) + '<')
        .replace(/(data-error="[^"]*">)(\d+)(<)/g, (_m, a: string, n: string) => a + one(n) + '<');
}

/** The engine's error list with sourceLine in sheet lines (outputLine is kept). */
export function remapErrors<T extends { sourceLine?: number }>(errors: T[], c: SheetCompiled): T[] {
    return errors.map(e => (typeof e.sourceLine === 'number' ? { ...e, sourceLine: sheetLine(c, e.sourceLine) } : e));
}

/** The prettified sheet (spacing around `:` `|` `->`, blank lines before headings, aligned colons). */
export async function formatSheet(src: string): Promise<string> {
    const mod = await loadSheet();
    return mod.format(src);
}

/** The look of a quantity's note and of a run of input lines (`sheet-form`): classes in the compiled text, styled here instead of inline. */
export const SHEET_CSS = '<style id="cprs-sheet">.sheet-note{color:#5f6a77;font-style:italic;margin-left:.6em}.sheet-form{border-left:3px solid #8aa4c8;padding:.15em 0 .15em .8em;margin:.5em 0}@media screen{.dark-theme .sheet-note{color:#7c8896}.dark-theme .sheet-form{border-left-color:#4f6a8f}}</style>';

/**
 * The look of a key line (`sheet-key`, the star the compiler puts after a line ending in `{key}`), of the key results list
 * and of a folding section (`sheet-collapse`, a heading marked `{collapse}`). Added to a page only when it has one of them,
 * so a sheet that uses neither is drawn exactly as before. The key rules use `:has()`, which every current browser
 * supports; where it is missing the star still marks the line.
 */
export const SHEET_FEATURE_CSS = '<style id="cprs-sheet-features">'
    + '.sheet-key{color:#b7791f;font-weight:700;margin-left:.5em;cursor:help}'
    + 'p:has(.sheet-key){background:rgba(255,193,7,.17);border-left:3px solid #e0a800;padding:.1em .5em .1em .6em;margin-left:-.6em;border-radius:0 3px 3px 0}'
    + '.sheet-key-results{border:1px solid #d6dbe3;border-radius:6px;padding:.3em .9em;margin:.5em 0;background:rgba(0,0,0,.02)}'
    + '.sheet-key-results p:has(.sheet-key){background:none;border-left:0;padding:0;margin-left:0}'
    + '.sheet-collapse{margin:.5em 0}'
    + '.sheet-collapse>summary{cursor:pointer;display:flex;align-items:baseline;gap:.45em;list-style:none;border-radius:4px}'
    + '.sheet-collapse>summary::-webkit-details-marker{display:none}'
    + '.sheet-collapse>summary::before{content:"\\25B8";color:#5f6a77;font-size:1.2em;line-height:1;flex:none;transition:transform .12s}'
    + '.sheet-collapse[open]>summary::before{transform:rotate(90deg)}'
    + '@media (hover:hover){.sheet-collapse>summary:hover{background:rgba(95,106,119,.12)}}'
    + '.sheet-collapse>summary:focus-visible{outline:2px solid #4da3ff;outline-offset:2px}'
    + '.sheet-collapse>summary>h1,.sheet-collapse>summary>h2,.sheet-collapse>summary>h3,.sheet-collapse>summary>h4,.sheet-collapse>summary>h5,.sheet-collapse>summary>h6{display:inline-block;margin:.25em 0}'
    + '@media screen{'
    + '.dark-theme .sheet-key{color:#f2c14e}.dark-theme p:has(.sheet-key){background:rgba(242,193,78,.14);border-left-color:#f2c14e}'
    + '.dark-theme .sheet-key-results{border-color:#2a3038;background:rgba(255,255,255,.03)}'
    + '.dark-theme .sheet-collapse>summary::before{color:#9aa5b4}}'
    + '@media screen and (hover:hover){.dark-theme .sheet-collapse>summary:hover{background:rgba(154,165,180,.14)}}'
    + '@media print{.sheet-key{color:#000}p:has(.sheet-key){background:#fff3cd;border-left:3px solid #c99700;-webkit-print-color-adjust:exact;print-color-adjust:exact}'
    + '.sheet-collapse>summary::before{content:none}}'
    + '</style>';

/** Adds SHEET_CSS (and SHEET_FEATURE_CSS, if the page has key lines or folding sections) to a page's head. */
export function withSheetCss(html: string): string {
    const css = SHEET_CSS + (/class="sheet-(key|collapse)/.test(html) ? SHEET_FEATURE_CSS : '');
    const i = html.toLowerCase().lastIndexOf('</head>');
    return i < 0 ? css + html : html.slice(0, i) + css + html.slice(i);
}
