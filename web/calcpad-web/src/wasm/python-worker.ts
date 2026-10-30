/// <reference lib="webworker" />
// calcpad-rs: runs the Python of a worksheet's `#python` blocks on MicroPython, off the main thread.
//
//   in   { type: 'start', token, base }                       a fresh interpreter (one per calculation)
//        { type: 'run', token, id, source, first }            one block, in the interpreter's session
//   out  { type: 'ready', token } after 'start'
//        { type: 'result', token, id, result: BlockResult }
//        { type: 'boot-error', token, error }
//
// The runtime and the stand-ins load on the first 'start' (nothing before that): micropython.mjs and .wasm and the
// files listed in py/files.json, all from the app's own origin. This worker never talks to the network again after
// that load: its fetch is replaced by one that serves the wasm bytes it already holds, the other ways out (XMLHttpRequest,
// WebSocket, importScripts, IndexedDB and the rest) are removed, and the interpreter's own `js` and `jsffi` modules are
// refused by an import guard (public/py/lib/calcpad_py.py). A run that does not finish is killed by the page.
// Sandbox limits are in docs/python-blocks.md.

import { renderFigure } from './python-chart';
import { readTraceback, type BlockResult } from './python-blocks';

type StartMsg = { type: 'start'; token: string; base: string; seed?: string };
type PanelMsg = { type: 'panel'; token: string; id: number; payload: string };
type RunMsg = { type: 'run'; token: string; id: number; source: string; first: number };

const OUTPUT_MAX = 100_000;      // characters of print output kept per worksheet run
const FIGURES_MAX = 20;          // figures per worksheet run
const SVG_MAX = 3_000_000;       // characters of SVG per worksheet run
const PLOTLY_MAX = 4_000_000;    // characters of Plotly figure JSON per worksheet run
const PANEL_MAX = 2_000_000;     // characters of one panel update (a figure and its data)
const EXPORTS_MAX = 500;         // exported names per worksheet run
const HEAP = 32 * 1024 * 1024;   // the interpreter's heap
// The interpreter's Python stack, in words (MicroPython's own default is 2048). A list literal holds every element on this stack
// while it is built, so the default stopped a literal of 2,043 numbers with "pystack exhausted"; 16384 takes 16,378.
// It is not raised further: MicroPython counts a code block's stack in 16 bits, so a literal past 65,535 elements wraps that
// count and can write past the stack instead of failing cleanly. Literals that long are refused before they run (LITERAL_MAX).
const PYSTACK = 16384;
const LITERAL_MAX = 16000;
/** The hint added to a "pystack exhausted" error: a long list literal is the usual cause. */
const PYSTACK_HINT = 'long list literals are limited: build the list in a loop or parse a text string, '
    + 'for example [float(v) for v in "1.5 2.5 3.5".split()]';

/** The request and answer format of the linear algebra module (calcpad-linalg-wasm); must equal its `abi()`. */
const LINALG_ABI = 'calcpad-linalg-2';

interface Loaded {
    realFetch: typeof fetch;
    base: string;
    loadMicroPython: (o: Record<string, unknown>) => Promise<any>;
    wasm: Uint8Array;
    wasmUrl: string;
    files: [string, string][];
    /** Library files fetched only for a block that imports them (py/lib/plotly/, py/lib/ui.py), as paths under py/lib. */
    lazy: string[];
    lazyText: Promise<[string, string][]> | null;
}

let loaded: Promise<Loaded> | null = null;
let lastBase = '';
// The engine's dense linear algebra (numpy.linalg, scipy.linalg), loaded the first time a block mentions it.
let linalgCall: ((request: string) => string) | null = null;
let linalgError: string | null = null;
let linalgLoading: Promise<void> | null = null;

async function loadLinalg(l: Loaded): Promise<void> {
    try {
        const mod = (await import(/* @vite-ignore */ `${l.base}wasm/calcpad_linalg_wasm.js`)) as {
            default: (o: { module_or_path: ArrayBuffer }) => Promise<unknown>;
            abi: () => string;
            linalg: (request: string) => string;
        };
        // 'no-cache' revalidates, so the wasm is never an older build than the glue just imported.
        const r = await l.realFetch(`${l.base}wasm/calcpad_linalg_wasm_bg.wasm`, { cache: 'no-cache' });
        if (!r.ok) throw new Error(`calcpad_linalg_wasm_bg.wasm: ${r.status}`);
        await mod.default({ module_or_path: await r.arrayBuffer() });
        if (mod.abi() !== LINALG_ABI) throw new Error(`the linear algebra module is ${mod.abi()}, this page needs ${LINALG_ABI} (reload the page)`);
        linalgCall = mod.linalg;
    } catch (e) {
        linalgError = e instanceof Error ? e.message : String(e);
    }
}

/** Loads the linear algebra module before a block that may use it runs (Python calls it synchronously, so it must be there).
 *  A block that multiplies large matrices (`@`, dot, matmul) loads it too, because the product is done by the engine then. */
async function ensureLinalg(source: string): Promise<void> {
    if (linalgCall || linalgError || !/linalg|scipy|matmul|\bdot\b|@/.test(source)) return;
    const l = await load(lastBase);
    linalgLoading ??= loadLinalg(l);
    await linalgLoading;
}

let session: { mp: any; state: SessionState; written: Set<string>; made: Set<string> } | null = null;

interface SessionState {
    parts: BlockResult['parts'];
    exports: BlockResult['exports'];
    truncated: boolean;
    chars: number;
    svgChars: number;
    plotlyChars: number;
    figures: number;
    hostError: string | null;
    seed: string;
    panelOut: string | null;
}

const send = self.postMessage.bind(self);

function load(base: string): Promise<Loaded> {
    if (!loaded) {
        loaded = (async () => {
            const realFetch = self.fetch.bind(self);
            const wasmUrl = new URL(`${base}py/micropython/micropython.wasm`, self.location.href).href;
            const [mod, wasmBuf, list] = await Promise.all([
                import(/* @vite-ignore */ `${base}py/micropython/micropython.mjs`),
                realFetch(wasmUrl).then((r) => { if (!r.ok) throw new Error(`micropython.wasm: ${r.status}`); return r.arrayBuffer(); }),
                realFetch(`${base}py/files.json`).then((r) => { if (!r.ok) throw new Error(`files.json: ${r.status}`); return r.json() as Promise<string[]>; }),
            ]);
            const isLazy = (f: string) => f.startsWith('plotly/') || f === 'ui.py';
            const lazy = list.filter(isLazy);
            const files = await Promise.all(list.filter((f) => !isLazy(f)).map(async (f): Promise<[string, string]> => {
                const r = await realFetch(`${base}py/lib/${f}`);
                if (!r.ok) throw new Error(`${f}: ${r.status}`);
                return [f, await r.text()];
            }));
            const wasm = new Uint8Array(wasmBuf);
            // From here on the only thing this worker can fetch is the interpreter's own wasm, from memory.
            Object.defineProperty(self, 'fetch', {
                configurable: true, writable: true,
                value: (input: unknown) => {
                    const url = String((input as { url?: string })?.url ?? input);
                    if (url === wasmUrl || url.endsWith('micropython.wasm')) {
                        return Promise.resolve(new Response(wasm.slice(), { headers: { 'Content-Type': 'application/wasm' } }));
                    }
                    return Promise.reject(new TypeError('network access is not available to a worksheet script'));
                },
            });
            return { realFetch, base, loadMicroPython: mod.loadMicroPython, wasm, wasmUrl, files, lazy, lazyText: null };
        })();
        loaded.catch(() => { loaded = null; });
    }
    return loaded;
}

/** Removes the ways a script could reach the network or storage if it ever got hold of the global object. */
function lockDown(): void {
    const g = self as unknown as Record<string, unknown>;
    for (const k of ['XMLHttpRequest', 'WebSocket', 'EventSource', 'indexedDB', 'caches', 'BroadcastChannel',
        'Worker', 'SharedWorker', 'WebTransport', 'RTCPeerConnection']) {
        try { Object.defineProperty(g, k, { configurable: true, writable: true, value: undefined }); } catch { /* not removable */ }
    }
    // Emscripten's loader tells a worker from a shell by `typeof importScripts == "function"`, and a new interpreter starts
    // for every calculation, so the name stays and the function refuses.
    try {
        Object.defineProperty(g, 'importScripts', { configurable: true, writable: true, value: () => { throw new Error('not available to a worksheet script'); } });
    } catch { /* not replaceable */ }
}

/** Fetches the plotly shim (once per page) and writes it into the interpreter, for a block that mentions plotly. */
async function ensurePlotly(source: string): Promise<void> {
    if (!session || !/plotly|\bui\b/.test(source)) return;
    const l = await load(lastBase);
    l.lazyText ??= Promise.all(l.lazy.map(async (f): Promise<[string, string]> => {
        const r = await l.realFetch(`${l.base}py/lib/${f}`);
        if (!r.ok) throw new Error(`${f}: ${r.status}`);
        return [f, await r.text()];
    }));
    const files = (await l.lazyText).filter(([f]) => (f === 'ui.py' ? /\bui\b/.test(source) : /plotly/.test(source)));
    const { mp, made } = session;
    for (const [path, src] of files) {
        if (session.written.has(path)) continue;
        session.written.add(path);
        const full = `/lib/${path}`;
        let p = '';
        for (const part of full.slice(0, full.lastIndexOf('/')).split('/').filter(Boolean)) {
            p += `/${part}`;
            if (made.has(p)) continue;
            made.add(p);
            try { mp.FS.mkdir(p); } catch { /* exists */ }
        }
        mp.FS.writeFile(full, src);
    }
}

async function start(msg: StartMsg): Promise<void> {
    lastBase = msg.base;
    const l = await load(msg.base);
    const state: SessionState = { parts: [], exports: [], truncated: false, chars: 0, svgChars: 0, plotlyChars: 0, figures: 0, hostError: null, seed: msg.seed || '{}', panelOut: null };
    let pend: number[] = [];
    const dec = new TextDecoder();
    const text = (s: string) => {
        if (!s || state.truncated) return;
        if (state.chars + s.length > OUTPUT_MAX) { s = s.slice(0, OUTPUT_MAX - state.chars); state.truncated = true; }
        state.chars += s.length;
        const last = state.parts[state.parts.length - 1];
        if (last && 'text' in last) last.text += s; else state.parts.push({ text: s });
    };
    const flush = () => { if (pend.length) { text(dec.decode(Uint8Array.from(pend))); pend = []; } };
    const write = (bytes: Uint8Array) => {
        if (state.truncated) return;
        for (const c of bytes) { pend.push(c); if (c === 10) flush(); }
        if (pend.length > 65536) flush();
    };
    const host = {
        figure(specText: string) {
            flush();
            if (state.figures >= FIGURES_MAX) { state.truncated = true; return; }
            try {
                const svg = renderFigure(String(specText).replace(/-?\bInfinity\b/g, 'null').replace(/\bNaN\b/g, 'null'));
                state.svgChars += svg.length;
                if (state.svgChars > SVG_MAX) { state.truncated = true; return; }
                state.figures++;
                state.parts.push({ svg });
            } catch (e) {
                state.hostError ??= `figure could not be drawn: ${e instanceof Error ? e.message : String(e)}`;
            }
        },
        plotly(specText: string) {
            flush();
            const spec = String(specText);
            if (state.figures >= FIGURES_MAX || state.plotlyChars + spec.length > PLOTLY_MAX) { state.truncated = true; return; }
            let title = '';
            try {
                const t = (JSON.parse(spec) as { layout?: { title?: unknown } }).layout?.title;
                const text = typeof t === 'string' ? t : (t as { text?: unknown } | undefined)?.text;
                title = typeof text === 'string' ? text : '';
            } catch {
                state.hostError ??= 'plotly figure could not be read';
                return;
            }
            state.plotlyChars += spec.length;
            state.figures++;
            state.parts.push({ plotly: spec, title });
        },
        panel(t: string) {
            flush();
            const spec = String(t);
            if (state.figures >= FIGURES_MAX || state.plotlyChars + spec.length > PLOTLY_MAX) { state.truncated = true; return; }
            state.plotlyChars += spec.length;
            state.figures++;
            state.parts.push({ panel: spec });
        },
        panelOut(t: string) {
            const out = String(t);
            state.panelOut = out.length > PANEL_MAX ? JSON.stringify({ error: `the panel's answer is larger than ${PANEL_MAX / 1_000_000} MB` }) : out;
        },
        seed(): string { return state.seed; },
        linalg(t: string): string {
            if (linalgCall) {
                try { return linalgCall(String(t)); } catch (e) { return JSON.stringify({ ok: false, error: `linear algebra failed: ${e instanceof Error ? e.message : String(e)}` }); }
            }
            return JSON.stringify({ ok: false, error: linalgError ? `the linear algebra module did not load: ${linalgError}` : 'the linear algebra module is not loaded (write numpy.linalg or scipy.linalg in the block itself)' });
        },
        export(t: string) {
            if (state.exports.length >= EXPORTS_MAX) { state.hostError ??= `a worksheet may export at most ${EXPORTS_MAX} names`; return; }
            try { state.exports.push(JSON.parse(String(t))); } catch { state.hostError ??= 'export: unreadable value'; }
        },
    };
    const mp = await l.loadMicroPython({ pystack: PYSTACK, heapsize: HEAP, stdout: write, stderr: write, linebuffer: false, url: l.wasmUrl });
    mp.registerJsModule('_cp_raw', host);
    const made = new Set<string>();
    const mkdirs = (dir: string) => {
        let p = '';
        for (const part of dir.split('/').filter(Boolean)) {
            p += `/${part}`;
            if (made.has(p)) continue;
            made.add(p);
            try { mp.FS.mkdir(p); } catch { /* exists */ }
        }
    };
    for (const [path, src] of l.files) {
        const full = `/lib/${path}`;
        mkdirs(full.slice(0, full.lastIndexOf('/')));
        mp.FS.writeFile(full, src);
    }
    mp.runPython('import _cp_raw\nimport calcpad_py\ncalcpad_py.install(_cp_raw)\n_cp_run = calcpad_py.run\ndel _cp_raw\ndel calcpad_py\n');
    (state as SessionState & { flush?: () => void }).flush = flush;
    session = { mp, state, written: new Set<string>(), made };
    lockDown();
}

/** The number of elements of the longest bracketed literal in `src` (a rough scan: strings and comments are skipped). */
function longestLiteral(src: string): { n: number; line: number } {
    const stack: number[] = [];
    let best = { n: 0, line: 0 };
    let line = 1;
    let start = 1;
    const starts: number[] = [];
    for (let i = 0; i < src.length; i++) {
        const c = src[i];
        if (c === '\n') line++;
        else if (c === '#') { while (i + 1 < src.length && src[i + 1] !== '\n') i++; }
        else if (c === '"' || c === "'") {
            const q = src.startsWith(c.repeat(3), i) ? c.repeat(3) : c;
            const end = src.indexOf(q, i + q.length);
            const stop = end < 0 ? src.length : end + q.length - 1;
            for (let k = i; k < stop; k++) if (src[k] === '\n') line++;
            i = stop;
        } else if (c === '[' || c === '(' || c === '{') { stack.push(1); starts.push(line); }
        else if (c === ']' || c === ')' || c === '}') {
            const n = stack.pop() ?? 0;
            start = starts.pop() ?? line;
            if (n > best.n) best = { n, line: start };
        } else if (c === ',' && stack.length) stack[stack.length - 1]++;
    }
    return best;
}

function run(msg: RunMsg): BlockResult {
    if (!session) throw new Error('no interpreter');
    // A literal that long is not compiled at all: the block raises the error on the literal's line instead, through the
    // same path (and line numbering) as any other Python error.
    let source = msg.source;
    const big = longestLiteral(source);
    if (big.n > LITERAL_MAX) {
        const text = `a literal of ${big.n} elements is too long (at most ${LITERAL_MAX}): ${PYSTACK_HINT}`;
        source = '\n'.repeat(big.line - 1) + `raise RuntimeError(${JSON.stringify(text)})\n`;
    }
    const { mp, state } = session;
    state.parts = [];
    state.exports = [];
    state.hostError = null;
    const before = state.truncated;
    let error: BlockResult['error'] = null;
    mp.globals.set('_cp_src', '\n'.repeat(msg.first) + source);
    try {
        mp.runPython('_cp_run(_cp_src)');
    } catch (e) {
        const t = readTraceback(String((e as Error)?.message ?? e));
        const deep = /pystack exhausted/.test(t.message);
        error = { type: t.type, message: deep ? `${t.message} (${PYSTACK_HINT})` : t.message, line: t.line };
    }
    (state as SessionState & { flush?: () => void }).flush?.();
    if (!error && state.hostError) error = { type: 'Error', message: state.hostError, line: null };
    return { parts: state.parts, exports: state.exports, error, truncated: state.truncated && !before ? true : state.truncated };
}

/** One widget change: runs the panel's function again in the live interpreter and returns its answer as JSON text. */
function panel(msg: PanelMsg): string {
    if (!session) return JSON.stringify({ error: 'the Python session is gone' });
    const { mp, state } = session;
    state.panelOut = null;
    state.parts = [];
    state.exports = [];
    mp.globals.set('_cp_pev', msg.payload);
    try {
        mp.runPython('import ui\nui._event(_cp_pev)');
    } catch (e) {
        const t = readTraceback(String((e as Error)?.message ?? e));
        return JSON.stringify({ error: `${t.type ? `${t.type}: ` : ''}${t.message}` });
    }
    (state as SessionState & { flush?: () => void }).flush?.();
    state.parts = [];   // print() inside a panel function is dropped; only the return value is shown
    return state.panelOut ?? JSON.stringify({ error: 'the panel gave no answer' });
}

self.onmessage = async (e: MessageEvent<StartMsg | RunMsg | PanelMsg>) => {
    const m = e.data;
    try {
        if (m.type === 'start') {
            await start(m);
            send({ type: 'ready', token: m.token });
        } else if (m.type === 'panel') {
            send({ type: 'result', token: m.token, id: m.id, result: panel(m) });
        } else if (m.type === 'run') {
            await ensureLinalg(m.source);
            await ensurePlotly(m.source);
            send({ type: 'result', token: m.token, id: m.id, result: run(m) });
        }
    } catch (err) {
        send({ type: 'boot-error', token: m.token, id: (m as RunMsg | PanelMsg).id, error: err instanceof Error ? err.message : String(err) });
    }
};
