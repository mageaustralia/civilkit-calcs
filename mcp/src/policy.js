// What a public, unauthenticated server refuses to run. The engine cannot be interrupted once it runs (it is
// synchronous wasm, and a Worker's clock does not advance while it runs), so work that can be bounded before the run
// is bounded here, and the rest is bounded by the Worker's CPU limit (wrangler.toml, limits.cpu_ms) and by the
// engine's memory cap (scripts/build.mjs). See docs/mcp.md, "Limits".
//
// The checks run on the Calcpad text the engine will see (a sheet is checked after it is compiled), line by line.

export const LIMITS = {
    /** Characters of a sheet or worksheet. */
    sourceChars: 64 * 1024,
    /** Characters of an expression (convert, check). */
    expressionChars: 4 * 1024,
    /** Passes of loops with literal bounds, nested loops multiplied. */
    loopPasses: 200_000,
    /** Passes of one $Sum, $Product, $Repeat, ... with literal bounds. */
    inlineLoopPasses: 1_000_000,
    /** Elements of a vector or matrix made with a literal size. */
    elements: 2_000_000,
    /** PlotWidth and PlotHeight. */
    plotPixels: 2000,
    /** Lines of the Calcpad text. */
    lines: 5000,
};

const NUM = String.raw`[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?`;
const literal = (s) => (new RegExp(`^\\s*${NUM}\\s*$`).test(s) ? Number(s) : null);

const SIZED = /\b(vector|matrix|identity|diagonal|column|utriang|ltriang|symmetric)(?:_hp)?\s*\(\s*([^;()]*)(?:;\s*([^;()]*))?/gi;

/**
 * Problems with `text`, each { line (1 based, in `text`), message }. An empty list means the text may run.
 */
export function checkSource(text) {
    const problems = [];
    const lines = text.split('\n');
    if (lines.length > LIMITS.lines) {
        problems.push({ line: null, message: `The worksheet has ${lines.length} lines; this server runs at most ${LIMITS.lines}.` });
        return problems;
    }
    const stack = []; // passes of the open loops (null when the count is not a literal)
    const work = () => stack.reduce((a, n) => a * (n ?? 1), 1);
    lines.forEach((raw, i) => {
        const line = i + 1;
        const t = raw.trim();
        const add = (message) => problems.push({ line, message });
        // Comments and text lines ('... and "...) are not run. Their text is still checked below for nothing.
        if (/^['"]/.test(t)) return;
        const kw = /^#([a-z]+)\b/i.exec(t)?.[1]?.toLowerCase();
        if (kw === 'python') add('Python blocks are not run by this server (they run only in the app, in the browser). Remove the block or open the sheet in the app.');
        if (kw === 'include' || kw === 'read' || kw === 'write' || kw === 'append') add(`#${kw} needs files, and this server has no file access. Put the values in the sheet itself.`);
        if (kw === 'for') {
            const m = /^#for\s+[^=]+=\s*(.+?)\s*:\s*(.+?)\s*$/i.exec(t);
            const a = m ? literal(m[1]) : null;
            const b = m ? literal(m[2]) : null;
            stack.push(a !== null && b !== null ? Math.abs(Math.floor(b) - Math.ceil(a)) + 1 : null);
        } else if (kw === 'repeat') {
            const n = literal(t.slice('#repeat'.length));
            stack.push(n !== null ? Math.max(0, n) : null);
        } else if (kw === 'while') {
            stack.push(null);
        } else if (kw === 'loop') {
            stack.pop();
        }
        if ((kw === 'for' || kw === 'repeat') && work() > LIMITS.loopPasses) {
            add(`These loops make ${work().toLocaleString('en')} passes; this server allows ${LIMITS.loopPasses.toLocaleString('en')}. Use a smaller range, or open the sheet in the app.`);
        }
        // $Sum{...@ k = a : b}, $Repeat, $Product, $Map/$Plot ranges are drawn at a fixed resolution and not counted.
        for (const m of t.matchAll(new RegExp(`\\$(Sum|Product|Repeat|While|Block|Inline)\\s*\\{[^@}]*@\\s*[^=]+=\\s*(${NUM})\\s*:\\s*(${NUM})`, 'gi'))) {
            const n = Math.abs(Number(m[3]) - Number(m[2])) + 1;
            if (n > LIMITS.inlineLoopPasses) add(`$${m[1]} over ${n.toLocaleString('en')} values; this server allows ${LIMITS.inlineLoopPasses.toLocaleString('en')}.`);
        }
        for (const m of t.matchAll(SIZED)) {
            const a = literal(m[2] ?? '');
            const b = m[3] !== undefined ? literal(m[3]) : null;
            const fn = m[1].toLowerCase();
            const n = a === null ? 0 : fn === 'matrix' ? a * (b ?? 1) : fn === 'vector' || fn === 'column' ? a : a * a;
            if (n > LIMITS.elements) add(`${m[1]}(...) makes ${n.toLocaleString('en')} elements; this server allows ${LIMITS.elements.toLocaleString('en')}.`);
        }
        const plot = /^(PlotWidth|PlotHeight)\s*=\s*(.+)$/i.exec(t);
        if (plot && (literal(plot[2]) ?? 0) > LIMITS.plotPixels) add(`${plot[1]} above ${LIMITS.plotPixels} is not allowed on this server.`);
    });
    return problems;
}
