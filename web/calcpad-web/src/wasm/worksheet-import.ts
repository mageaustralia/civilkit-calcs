// calcpad-rs: "Import worksheet...". Reads a foreign `.xmcd` or `.mcdx` worksheet in the page, converts it with the
// importer module (import-worker.ts, its own wasm, loaded on first use), stores the picture files in the
// library under /imported/<name>/, opens the converted sheet as a new unsaved tab, and tells the user
// what converted and what did not. Everything runs locally; nothing is uploaded.

import type { FileLibrary } from './library';
import type { TabsLike } from './web-files';
import type { ImportResult } from './import-worker';

export interface ImportReportJson {
    regionsTotal: number;
    regionsConverted: number;
    regionsShowing: number;
    regionsNeedingAttention: number;
    /** Definitions whose value was taken from the saved data of an input component (frozen). */
    frozenValues?: number;
    kinds: Record<string, { total: number; converted: number; withNotes: number; attention: number; pictured: number }>;
    pictures: { svgOk: number; svgFailed: number; pngUsed: number; fallbackShown: number; savedResultAvailable: number; savedResultEmitted: number };
    ranked: { code: string; reason: string; direct: number; downstream: number; blocked: number; topDetails: [string, number][] }[];
    fatal: string | null;
    fatalMessage: string | null;
}

export interface ImportOptionsJson {
    pictures: boolean;
    savedResults: boolean;
    assetPrefix: string;
    convertMetafiles?: boolean;
    tidy?: boolean;
    markdown?: boolean;
}

/** Human wording for the gap codes (the report's own reasons are written for developers). */
const WORDING: Record<string, string> = {
    'addin-function': 'calls to a function from an add-in we cannot run',
    'reference-function': 'functions that live in other worksheets (not imported)',
    'builtin-missing': 'built-in functions the engine does not have yet',
    'ole-object': 'embedded objects',
    'string-value': 'text (string) values',
    'bare-expression': 'expressions that are shown but not calculated',
    'constraint-equation': 'equations written as constraints (shown, not solved)',
    'undefined-variable': 'names used before they are defined',
    'range-variable': 'range variables (i = 1..10) and what iterates over them',
    'program-block': 'multi-line programs',
    'solve-block': 'solve blocks',
    'symbolic-evaluation': 'symbolic evaluation',
    'calculus-operator': 'derivatives and integrals',
    'summation-operator': 'sums, products and limits',
    'vectorize-operator': 'the vectorize operator',
    'matrix-operator': 'matrix column, row and vector-sum operators',
    plot: 'plots',
    image: 'pictures without usable image data',
    component: 'data tables and scripted components',
    'worksheet-reference': 'references to other worksheets',
    'unmapped-unit': 'units without an equivalent',
    'unknown-element': 'constructs the importer does not know',
    'depends-on-gap': 'regions that use something that did not convert',
    'element-assignment': 'assignments to matrix elements',
    'disabled-region': 'regions that are switched off in the original',
    'header-footer': 'page headers and footers',
    'rich-text-element': 'tables, pictures and floating blocks inside text',
    'text-unreadable': 'text that could not be read from the file',
};

export function wording(code: string, reason: string): string {
    return WORDING[code] ?? reason;
}

/** A name safe to use as a library folder: letters, digits, `_`, `-`; anything else becomes `_`. */
export function slug(fileName: string): string {
    const stem = fileName.replace(/\.[^.]*$/, '');
    return stem.normalize('NFKD').replace(/[^A-Za-z0-9_-]+/g, '_').replace(/^_+|_+$/g, '').slice(0, 60) || 'worksheet';
}

interface Pending { resolve: (r: ImportResult) => void; reject: (e: Error & { trap?: boolean }) => void }

/** The importer worker, started on first use and replaced if the module traps. */
export class WorksheetImporter {
    private worker: Worker | null = null;
    private seq = 0;
    private pending = new Map<number, Pending>();

    constructor(private readonly base: string) {}

    private start(): Worker {
        if (this.worker) return this.worker;
        const w = new Worker(new URL('./import-worker.ts', import.meta.url), { type: 'module' });
        w.onmessage = (e: MessageEvent<{ id: number; result?: ImportResult; error?: string; trap?: boolean }>) => {
            const p = this.pending.get(e.data.id);
            if (!p) return;
            this.pending.delete(e.data.id);
            if (e.data.error !== undefined) p.reject(Object.assign(new Error(e.data.error), { trap: !!e.data.trap }));
            else p.resolve(e.data.result!);
        };
        w.onerror = (e: ErrorEvent) => {
            const err = Object.assign(new Error(`the importer could not load: ${e.message || 'unknown error'}`), { trap: true });
            for (const p of this.pending.values()) p.reject(err);
            this.pending.clear();
            this.reset();
        };
        this.worker = w;
        return w;
    }

    private reset(): void {
        this.worker?.terminate();
        this.worker = null;
    }

    private post(bytes: Uint8Array, options: ImportOptionsJson, extra: { op?: 'tidy' | 'markdown'; text?: string } = {}): Promise<ImportResult> {
        return new Promise((resolve, reject) => {
            const id = ++this.seq;
            this.pending.set(id, { resolve, reject });
            this.start().postMessage({ id, base: this.base, bytes, options: JSON.stringify(options), ...extra });
        });
    }

    /** Tidies Calcpad text, or also writes its simple paragraphs as Markdown. */
    async transform(op: 'tidy' | 'markdown', text: string): Promise<string> {
        return (await this.post(new Uint8Array(0), {} as ImportOptionsJson, { op, text })).calcpad;
    }

    /**
     * Converts `bytes`. If the module traps (a hostile or odd metafile), the worker is replaced and the
     * import is retried once with vector pictures off, so the raster pictures are used instead.
     */
    async run(bytes: Uint8Array, options: ImportOptionsJson): Promise<{ result: ImportResult; report: ImportReportJson; retried: boolean }> {
        let retried = false;
        let result: ImportResult;
        try {
            result = await this.post(bytes, options);
        } catch (e) {
            if (!(e as { trap?: boolean }).trap) throw e;
            this.reset();
            retried = true;
            result = await this.post(bytes, { ...options, convertMetafiles: false });
        }
        return { result, report: JSON.parse(result.report) as ImportReportJson, retried };
    }
}

// ---------------------------------------------------------------------------------------------------

export interface ImportHost {
    library: FileLibrary;
    tabs: () => TabsLike & {
        newUntitled(content?: string, title?: string): string;
        activeModel: { getValue(): string; setValue?(v: string): void } | null;
    };
    log: (level: 'info' | 'warn' | 'error', message: string) => void;
}

interface Session {
    name: string;
    prefix: string;
    bytes: Uint8Array;
    tabId: string;
    pictures: boolean;
    savedResults: boolean;
    tidy: boolean;
    markdown: boolean;
    paths: Set<string>;
}

/** Counts for the summary line. */
export function summarise(rep: ImportReportJson): { live: number; total: number; pictures: number; attention: number } {
    const images = rep.kinds.image?.converted ?? 0;
    return {
        live: rep.regionsConverted - images,
        total: rep.regionsTotal,
        pictures: rep.pictures.fallbackShown + images,
        attention: rep.regionsNeedingAttention,
    };
}

export class WorksheetImport {
    private readonly importer: WorksheetImporter;
    private session: Session | null = null;
    private busy = false;

    constructor(private readonly host: ImportHost) {
        this.importer = new WorksheetImporter(new URL(import.meta.env.BASE_URL || './', document.baseURI).href);
    }

    /** True for the file names this importer handles (or refuses with a message). */
    static handles(name: string): boolean {
        return /\.(xmcd|mcdx)$/i.test(name);
    }

    /** The File menu / ribbon command: pick a file, then import it. */
    async pick(): Promise<void> {
        const files = await new Promise<File[]>((resolve) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.accept = '.xmcd,.mcdx';
            input.style.display = 'none';
            input.setAttribute('data-cprs-import-input', '');
            input.onchange = () => { resolve(Array.from(input.files ?? [])); input.remove(); };
            input.addEventListener('cancel', () => { resolve([]); input.remove(); });
            document.body.appendChild(input);
            input.click();
        });
        for (const f of files) await this.importFile(f);
    }

    async importFile(file: File): Promise<void> {
        const name = file.name;
        if (!/\.(xmcd|mcdx)$/i.test(name)) {
            this.message('Import worksheet', `${name} is not a worksheet this importer reads. Choose a .xmcd or .mcdx file.`);
            return;
        }
        if (this.busy) {
            this.message('Import worksheet', 'An import is already running.');
            return;
        }
        this.busy = true;
        try {
            const bytes = new Uint8Array(await file.arrayBuffer());
            const session: Session = { name, prefix: `imported/${slug(name)}`, bytes, tabId: '', pictures: true, savedResults: false, tidy: false, markdown: false, paths: new Set() };
            const rep = await this.convert(session, true);
            if (rep) this.showResult(session, rep);
        } catch (e) {
            this.message('Import failed', `${name}: ${e instanceof Error ? e.message : String(e)}`);
        } finally {
            this.busy = false;
        }
    }

    /** The Import group's Tidy / Markdown commands: rewrite the active worksheet's text (one undo step). */
    async transformActive(op: 'tidy' | 'markdown'): Promise<void> {
        const ed = (window as unknown as { calcpadActiveEditor?: {
            getModel(): { getValue(): string; getFullModelRange(): unknown } | null;
            executeEdits(source: string, edits: { range: unknown; text: string }[]): boolean;
            pushUndoStop(): void;
        } }).calcpadActiveEditor;
        const model = ed?.getModel();
        if (!ed || !model) return;
        try {
            const next = await this.importer.transform(op, model.getValue());
            if (next === model.getValue()) {
                this.message(op === 'tidy' ? 'Tidy text' : 'Text to Markdown', 'Nothing to change in this worksheet.');
                return;
            }
            ed.pushUndoStop();
            ed.executeEdits(op, [{ range: model.getFullModelRange(), text: next }]);
            ed.pushUndoStop();
            this.host.log('info', op === 'tidy' ? 'Tidied the worksheet text.' : 'Wrote the worksheet text as Markdown.');
        } catch (e) {
            this.message('Import failed', e instanceof Error ? e.message : String(e));
        }
    }

    /** Runs the importer and applies the result; returns the report, or null if the file was refused. */
    private async convert(s: Session, first: boolean): Promise<ImportReportJson | null> {
        const { result, report, retried } = await this.importer.run(s.bytes, {
            pictures: s.pictures,
            savedResults: s.savedResults,
            tidy: s.tidy,
            markdown: s.markdown,
            assetPrefix: s.prefix,
        });
        if (report.fatal) {
            this.message('Import failed', `${s.name}: ${report.fatalMessage ?? 'the file could not be read.'}`);
            return null;
        }
        const next = new Set(result.assets.map(a => `/${a.path}`));
        const stale = [...s.paths].filter(p => !next.has(p));
        await this.host.library.putMany(result.assets.map(a => ({ path: a.path, data: a.bytes })), stale);
        s.paths = next;
        const tabs = this.host.tabs();
        if (first) {
            s.tabId = tabs.newUntitled(result.calcpad, `${s.name.replace(/\.(xmcd|mcdx)$/i, '')}.cpd`);
        } else {
            tabs.activate(s.tabId);
            tabs.activeModel?.setValue?.(result.calcpad);
        }
        if (retried) this.host.log('warn', 'The importer restarted once and showed vector pictures as raster pictures.');
        this.host.log('info', `Imported ${s.name}: ${report.regionsConverted} of ${report.regionsTotal} regions converted.`);
        return report;
    }

    // ----- dialogs ---------------------------------------------------------------------------------

    private shell(title: string): { back: HTMLDivElement; box: HTMLDivElement; close: () => void } {
        document.getElementById('cprs-import-dialog')?.remove();
        const back = document.createElement('div');
        back.id = 'cprs-import-dialog';
        back.style.cssText = 'position:fixed;inset:0;z-index:2000;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)';
        const box = document.createElement('div');
        box.setAttribute('role', 'dialog');
        box.setAttribute('aria-label', title);
        box.style.cssText = 'width:min(560px,92vw);max-height:84vh;overflow:auto;padding:18px 22px;border-radius:6px;font:13px/1.5 system-ui,sans-serif;' +
            'background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)';
        const h = document.createElement('h3');
        h.textContent = title;
        h.style.cssText = 'margin:0 0 10px;font-size:16px';
        box.append(h);
        const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.stopPropagation(); close(); } };
        const close = () => { back.remove(); document.removeEventListener('keydown', onKey, true); };
        back.onmousedown = (e) => { if (e.target === back) close(); };
        document.addEventListener('keydown', onKey, true);
        back.append(box);
        document.body.append(back);
        return { back, box, close };
    }

    private button(label: string, onclick: () => void): HTMLButtonElement {
        const b = document.createElement('button');
        b.textContent = label;
        b.style.cssText = 'margin-top:12px;padding:4px 14px;font:inherit;cursor:pointer';
        b.onclick = onclick;
        return b;
    }

    message(title: string, text: string): void {
        const { box, close } = this.shell(title);
        const p = document.createElement('p');
        p.textContent = text;
        p.setAttribute('data-cprs-import-message', '');
        p.style.margin = '6px 0';
        box.append(p);
        const b = this.button('Close', close);
        box.append(b);
        b.focus();
    }

    private showResult(s: Session, rep: ImportReportJson): void {
        const { box, close } = this.shell('Import worksheet');
        const sum = summarise(rep);
        const line = document.createElement('p');
        line.setAttribute('data-cprs-import-summary', '');
        line.style.margin = '6px 0';
        line.textContent = `Imported ${s.name}: ${sum.live} of ${sum.total} regions converted live, ${sum.pictures} shown as static pictures, ${sum.attention} need attention.`;
        box.append(line);

        if ((rep.frozenValues ?? 0) > 0) {
            const fz = document.createElement('p');
            fz.setAttribute('data-cprs-import-frozen', '');
            fz.style.margin = '6px 0';
            fz.textContent = `${rep.frozenValues} ${rep.frozenValues === 1 ? 'value' : 'values'} frozen from input components: taken from what the file saved, so they do not follow a change of the input.`;
            box.append(fz);
        }

        const gaps = rep.ranked.filter(r => r.code !== 'depends-on-gap' && r.blocked > 0).slice(0, 5);
        if (gaps.length) {
            const head = document.createElement('p');
            head.textContent = 'What did not convert, by regions affected:';
            head.style.cssText = 'margin:10px 0 2px';
            box.append(head);
            const ul = document.createElement('ul');
            ul.setAttribute('data-cprs-import-gaps', '');
            ul.style.cssText = 'margin:0;padding-left:20px';
            for (const g of gaps) {
                const li = document.createElement('li');
                li.textContent = `${g.blocked} ${wording(g.code, g.reason)}`;
                li.dataset.code = g.code;
                ul.append(li);
            }
            box.append(ul);
        }

        const toggles = document.createElement('div');
        toggles.style.cssText = 'margin-top:12px;display:flex;flex-direction:column;gap:4px';
        const toggle = (label: string, checked: boolean, attr: string, set: (v: boolean) => void) => {
            const l = document.createElement('label');
            l.style.cssText = 'display:flex;gap:8px;align-items:center;cursor:pointer';
            const c = document.createElement('input');
            c.type = 'checkbox';
            c.checked = checked;
            c.setAttribute(attr, '');
            c.onchange = async () => {
                set(c.checked);
                c.disabled = true;
                try {
                    const next = await this.convert(s, false);
                    if (next) { close(); this.showResult(s, next); }
                } catch (e) {
                    this.message('Import failed', e instanceof Error ? e.message : String(e));
                }
            };
            l.append(c, label);
            toggles.append(l);
        };
        toggle('Pictures of unconverted regions', s.pictures, 'data-cprs-import-pictures', v => { s.pictures = v; });
        toggle('Show saved results as static values', s.savedResults, 'data-cprs-import-saved', v => { s.savedResults = v; });
        toggle('Tidy text (plain paragraphs, no imported fonts or colours)', s.tidy || s.markdown, 'data-cprs-import-tidy', v => { s.tidy = v; if (!v) s.markdown = false; });
        toggle('Text as Markdown (#md blocks; includes tidy)', s.markdown, 'data-cprs-import-markdown', v => { s.markdown = v; if (v) s.tidy = true; });
        box.append(toggles);

        const note = document.createElement('p');
        note.style.cssText = 'margin:10px 0 0;opacity:.75;font-size:12px';
        note.textContent = 'The file was read in this page and was not uploaded. Pictures are stored in the library under ' + `/${s.prefix}/.`;
        box.append(note);
        const b = this.button('Close', close);
        box.append(b);
        b.focus();
    }
}
