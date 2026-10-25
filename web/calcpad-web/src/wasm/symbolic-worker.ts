/// <reference lib="webworker" />
// calcpad-rs: runs the calcpad-caskit-wasm module (the symbolic algebra of the Symbolic dialog) off the main
// thread. It is a separate wasm, downloaded the first time the dialog computes something. The module answers
// every request with JSON text and does not throw for bad input; an exception here means it trapped, and the
// caller then terminates this worker and starts a fresh one.
//
//   { id, base, op, expr, args }  ->  { id, answer: string }  or  { id, error }

export {};

/** The request and answer format of the module; must equal its `abi()`. */
const ABI = 'calcpad-caskit-1';

interface Caskit {
    default: (input?: unknown) => Promise<unknown>;
    abi: () => string;
    run: (op: string, expr: string, args: string) => string;
}

let module_: Promise<Caskit> | null = null;

function load(base: string): Promise<Caskit> {
    if (!module_) {
        module_ = (async () => {
            const mod = (await import(/* @vite-ignore */ `${base}wasm/calcpad_caskit_wasm.js`)) as Caskit;
            await mod.default({ module_or_path: `${base}wasm/calcpad_caskit_wasm_bg.wasm` });
            if (mod.abi() !== ABI) throw new Error('the symbolic algebra module does not match this page; reload the page');
            return mod;
        })();
        module_.catch(() => { module_ = null; });
    }
    return module_;
}

self.onmessage = async (e: MessageEvent<{ id: number; base: string; op: string; expr: string; args: string }>) => {
    const { id, base, op, expr, args } = e.data;
    try {
        const mod = await load(base);
        self.postMessage({ id, answer: mod.run(op, expr, args) });
    } catch (err) {
        self.postMessage({ id, error: err instanceof Error ? err.message : String(err) });
    }
};
