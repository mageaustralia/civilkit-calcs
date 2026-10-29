// A small BM25 index for the guide sections and the function reference. Built once per isolate, on first use.

export function tokens(text) {
    return String(text).toLowerCase().normalize('NFKC').match(/[\p{L}\p{N}_$#]+/gu) ?? [];
}

/** docs: [{ id, fields: [[text, weight], ...] }]. Returns search(query, limit) -> [{ id, score }]. */
export function bm25(docs, { k1 = 1.2, b = 0.75 } = {}) {
    const tf = [];
    const df = new Map();
    let total = 0;
    for (const d of docs) {
        const counts = new Map();
        let len = 0;
        for (const [text, weight] of d.fields) {
            for (const t of tokens(text)) {
                counts.set(t, (counts.get(t) ?? 0) + weight);
                len += weight;
            }
        }
        for (const t of counts.keys()) df.set(t, (df.get(t) ?? 0) + 1);
        tf.push({ id: d.id, counts, len });
        total += len;
    }
    const avg = total / Math.max(1, docs.length);
    const n = docs.length;
    return function search(query, limit = 10) {
        const q = [...new Set(tokens(query))];
        if (!q.length) return [];
        const scored = [];
        for (const d of tf) {
            let s = 0;
            let hits = 0;
            for (const t of q) {
                const f = d.counts.get(t);
                if (!f) continue;
                hits++;
                const idf = Math.log(1 + (n - df.get(t) + 0.5) / (df.get(t) + 0.5));
                s += (idf * f * (k1 + 1)) / (f + k1 * (1 - b + (b * d.len) / avg));
            }
            // Documents that hold every query word rank first.
            if (s > 0) scored.push({ id: d.id, score: s * (1 + hits / q.length) });
        }
        scored.sort((a, c) => c.score - a.score || (a.id < c.id ? -1 : 1));
        return scored.slice(0, limit);
    };
}

/** A short extract of `text` around the first query word, on one line. */
export function snippet(text, query, size = 240) {
    const flat = String(text).replace(/```[\s\S]*?```/g, ' ').replace(/[#>*`|_]+/g, ' ').replace(/\[([^\]]*)\]\([^)]*\)/g, '$1').replace(/\s+/g, ' ').trim();
    const lower = flat.toLowerCase();
    let at = -1;
    for (const t of tokens(query)) {
        const i = lower.indexOf(t);
        if (i >= 0 && (at < 0 || i < at)) at = i;
    }
    const start = Math.max(0, (at < 0 ? 0 : at) - Math.floor(size / 3));
    const cut = flat.slice(start, start + size);
    return (start > 0 ? '...' : '') + cut + (start + size < flat.length ? '...' : '');
}
