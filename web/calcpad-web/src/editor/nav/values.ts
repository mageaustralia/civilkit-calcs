// What the variables of the open document come to, for completion details and the quick-recall
// palette. The engine reports a value only by running the worksheet, so this asks the web transport's
// `values` endpoint (one calculation with the variables recorded) and keeps the last answer per document.
// The answer is the value at the END of the worksheet, as the Excel export's Variables sheet shows it,
// and it can be a little behind the text: it is labelled as the last calculated value.
//
// Nothing here runs on every keystroke: a request is made when a completion list or the palette asks
// for values, never more often than every 1.5 s per document, and one at a time.

export interface VarValue {
    name: string;
    value: string;
    unit: string;
    input: boolean;
}

export type ValuesFetcher = (content: string, sourceFilePath?: string) => Promise<VarValue[] | null>;

interface Entry {
    content: string;
    at: number;
    byName: Map<string, VarValue>;
}

const MIN_INTERVAL_MS = 1500;
const cache = new Map<string, Entry>();
const inflight = new Map<string, Promise<Entry | null>>();
let fetcher: ValuesFetcher | null = null;

export function setValuesFetcher(f: ValuesFetcher | null): void {
    fetcher = f;
}

/** The last values calculated for a document, whatever its text was then. */
export function cachedValues(key: string): Map<string, VarValue> | null {
    return cache.get(key)?.byName ?? null;
}

/** Whether the cached values were calculated from exactly this text. */
export function valuesAreCurrent(key: string, content: string): boolean {
    return cache.get(key)?.content === content;
}

/**
 * Calculates the values when the cache is missing or behind the text (and the last run is not too
 * recent), and resolves with the cache either way. `maxWaitMs` bounds how long the caller waits; the
 * run carries on in the background.
 */
export async function loadValues(key: string, content: string, sourceFilePath: string | undefined, maxWaitMs = 0, force = false): Promise<Map<string, VarValue> | null> {
    const have = cache.get(key);
    const stale = !have || have.content !== content;
    if (stale && fetcher && !inflight.has(key) && (force || !have || Date.now() - have.at >= MIN_INTERVAL_MS)) {
        const f = fetcher;
        const run = (async (): Promise<Entry | null> => {
            try {
                const list = await f(content, sourceFilePath);
                if (!list) return null;
                const entry: Entry = { content, at: Date.now(), byName: new Map(list.map(v => [v.name, v])) };
                cache.set(key, entry);
                return entry;
            } catch {
                return null;
            } finally {
                inflight.delete(key);
            }
        })();
        inflight.set(key, run);
    }
    const pending = inflight.get(key);
    if (pending && maxWaitMs > 0) {
        await Promise.race([pending, new Promise(r => setTimeout(r, maxWaitMs))]);
    }
    return cachedValues(key);
}

/** A value for a list: `7`, `0.3`, `12.5 kN`, six significant figures at most. */
export function formatValue(v: VarValue | undefined): string {
    if (!v) return '';
    let text = v.value;
    if (/^-?\d+(\.\d+)?(e[+-]?\d+)?$/i.test(text)) {
        const n = Number(text);
        if (Number.isFinite(n)) text = String(+n.toPrecision(6));
    }
    return v.unit ? `${text} ${v.unit}` : text;
}
