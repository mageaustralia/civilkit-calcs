// calcpad-rs: runs the `#python` blocks of a worksheet in python-worker.ts (docs/python-blocks.md).
// The worker, the MicroPython runtime and the stand-in modules load on the first call, never before.
// One worker serves every calculation; each calculation gets a fresh interpreter, and its blocks share one session.
// A block that does not finish within BLOCK_MS, or a calculation within TOTAL_MS, kills the worker.

import { validName, type BlockResult, type PyBlock } from './python-blocks';

export const BLOCK_MS = 10_000;
export const TOTAL_MS = 30_000;
/** The limits after the user answers "Run again with a longer limit" (python-consent.ts), for the rest of the tab. */
export const LONG_BLOCK_MS = 60_000;
export const LONG_TOTAL_MS = 180_000;

const fail = (b: PyBlock, type: string, message: string): BlockResult => ({
    parts: [], exports: [], truncated: false, error: { type, message, line: b.start },
});

type Pending = { resolve: (v: any) => void; reject: (e: Error) => void };

class Session {
    private worker: Worker;
    private token = crypto.randomUUID();
    private seq = 0;
    private pending = new Map<string, Pending>();

    constructor(private readonly base: string) {
        this.worker = new Worker(new URL('./python-worker.ts', import.meta.url), { type: 'module' });
        this.worker.onmessage = (e: MessageEvent) => {
            const m = e.data;
            if (!m || m.token !== this.token) return;
            const key = m.type === 'ready' ? 'start' : String(m.id);
            const p = this.pending.get(key);
            if (!p) return;
            this.pending.delete(key);
            if (m.type === 'boot-error') p.reject(new Error(m.error));
            else p.resolve(m.type === 'result' ? m.result : undefined);
        };
        this.worker.onerror = (e) => this.kill(new Error(e.message || 'the Python worker failed to load'));
    }

    private ask<T>(key: string, msg: Record<string, unknown>, ms: number): Promise<T> {
        return new Promise<T>((resolve, reject) => {
            const timer = setTimeout(() => this.kill(new Error(`timeout:${ms}`)), ms);
            this.pending.set(key, { resolve: (v) => { clearTimeout(timer); resolve(v); }, reject: (e) => { clearTimeout(timer); reject(e); } });
            this.worker.postMessage({ ...msg, token: this.token });
        });
    }

    start(): Promise<void> { return this.ask('start', { type: 'start', base: this.base }, BLOCK_MS * 3); }

    run(b: PyBlock, ms: number): Promise<BlockResult> {
        const id = ++this.seq;
        return this.ask(String(id), { type: 'run', id, source: b.code, first: b.first }, ms);
    }

    kill(reason: Error): void {
        this.worker.terminate();
        for (const p of this.pending.values()) p.reject(reason);
        this.pending.clear();
    }
}

export class PythonRunner {
    private session: Session | null = null;
    private cache = new Map<string, BlockResult[]>();
    private longer = false;
    private inflight: { key: string; promise: Promise<BlockResult[]>; session: Session } | null = null;

    constructor(private readonly base: string) {}

    /** True once the worker has been started (the lazy-load check of the e2e reads this). */
    get started(): boolean { return this.session !== null; }

    /** True after the user chose a longer limit for this tab. */
    get longLimit(): boolean { return this.longer; }
    useLongLimit(): void { this.longer = true; this.cache.clear(); }

    run(blocks: PyBlock[]): Promise<BlockResult[]> {
        const key = JSON.stringify(blocks.map((b) => [b.first, b.code]));
        const hit = this.cache.get(key);
        if (hit) return Promise.resolve(hit);
        if (this.inflight?.key === key) return this.inflight.promise;
        // A different worksheet text supersedes the run in progress.
        if (this.inflight) { this.inflight.session.kill(new Error('superseded')); this.session = null; }
        const session = this.session ??= new Session(this.base);
        const promise = this.runAll(session, blocks).then((r) => {
            if (this.cache.size > 8) this.cache.delete(this.cache.keys().next().value as string);
            this.cache.set(key, r);
            return r;
        }).finally(() => { if (this.inflight?.promise === promise) this.inflight = null; });
        this.inflight = { key, promise, session };
        return promise;
    }

    private async runAll(session: Session, blocks: PyBlock[]): Promise<BlockResult[]> {
        const out: BlockResult[] = [];
        const t0 = performance.now();
        const blockMs = this.longer ? LONG_BLOCK_MS : BLOCK_MS;
        const totalMs = this.longer ? LONG_TOTAL_MS : TOTAL_MS;
        try {
            await session.start();
            for (const b of blocks) {
                if (b.end < 0) { out.push(fail(b, 'SyntaxError', 'the block has no "#end python" line')); continue; }
                if (performance.now() - t0 > totalMs) { out.push(fail(b, 'TimeoutError', 'skipped: the worksheet has used its Python time budget')); continue; }
                const r = await session.run(b, blockMs);
                for (const e of r.exports) {
                    if (!validName(e.name)) { r.error = { type: 'ValueError', message: `export: "${e.name}" is not a valid Calcpad name`, line: b.start }; break; }
                }
                out.push(r);
            }
        } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            if (msg === 'superseded') throw e;
            this.session = null;
            const timeout = /^timeout:/.test(msg);
            const text = timeout ? `the script ran longer than ${blockMs / 1000} s and was stopped` : msg;
            const done = out.length;
            while (out.length < blocks.length) {
                const b = blocks[out.length];
                out.push(out.length === done ? fail(b, timeout ? 'TimeoutError' : 'Error', text) : fail(b, 'Error', 'skipped: an earlier block failed to run'));
            }
        }
        return out;
    }
}
