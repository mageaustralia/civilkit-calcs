/// <reference lib="webworker" />
// calcpad-rs: runs the calcpad-import-wasm module (the foreign worksheet importer) off the main
// thread. It is a separate wasm, loaded the first time a worksheet is imported; the engine module is
// not involved. The importer reports bad input in its result and does not throw, so an exception here
// means the module trapped: the caller then terminates this worker, starts a fresh one and retries
// with vector pictures off (`convertMetafiles: false`).
//
//   { id, base, bytes, options }  ->  { id, result: ImportResult }  or  { id, error, trap: true }

export {};

export interface ImportAsset { path: string; mime: string; bytes: Uint8Array }
export interface ImportResult { calcpad: string; report: string; assets: ImportAsset[] }

interface ImportOut {
    calcpad: string;
    report: string;
    asset_count(): number;
    asset_path(i: number): string;
    asset_mime(i: number): string;
    asset_bytes(i: number): Uint8Array;
    free(): void;
}

interface Importer {
    default: (input?: unknown) => Promise<unknown>;
    import_worksheet: (bytes: Uint8Array, optionsJson: string) => ImportOut;
    tidy_text: (src: string) => string;
    markdown_text: (src: string) => string;
}

let module_: Promise<Importer> | null = null;

function load(base: string): Promise<Importer> {
    if (!module_) {
        module_ = (async () => {
            const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_import_wasm.js`)) as Importer;
            await mod.default({ module_or_path: `${base}wasm/calcpad_import_wasm_bg.wasm` });
            return mod;
        })();
        module_.catch(() => { module_ = null; });
    }
    return module_;
}

self.onmessage = async (e: MessageEvent<{ id: number; base: string; bytes: Uint8Array; options: string; op?: 'tidy' | 'markdown'; text?: string }>) => {
    const { id, base, bytes, options, op, text } = e.data;
    try {
        const mod = await load(base);
        if (op) {
            // Text in, text out: the same clean-up the import options apply, for a worksheet already open.
            const out = op === 'markdown' ? mod.markdown_text(text ?? '') : mod.tidy_text(text ?? '');
            self.postMessage({ id, result: { calcpad: out, report: '{}', assets: [] } });
            return;
        }
        const out = mod.import_worksheet(bytes, options);
        const assets: ImportAsset[] = [];
        for (let i = 0, n = out.asset_count(); i < n; i++) {
            assets.push({ path: out.asset_path(i), mime: out.asset_mime(i), bytes: out.asset_bytes(i) });
        }
        const result: ImportResult = { calcpad: out.calcpad, report: out.report, assets };
        out.free();
        (self as unknown as Worker).postMessage({ id, result }, assets.map(a => a.bytes.buffer));
    } catch (err) {
        self.postMessage({ id, error: err instanceof Error ? err.message : String(err), trap: true });
    }
};
