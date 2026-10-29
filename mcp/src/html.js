// The engine answers with an HTML page (the same page the app shows). These helpers turn it into plain text and
// pick out the equation lines, without a DOM (Workers have none): the engine's markup is regular, so a small scanner
// over its tags is enough.

const ENTITIES = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: ' ', ensp: ' ', emsp: ' ', thinsp: ' ', hairsp: '', ThinSpace: ' ' };

export function decodeEntities(s) {
    return s.replace(/&(#x[0-9a-f]+|#\d+|[a-z]+);/gi, (m, e) => {
        if (e[0] === '#') {
            const n = e[1] === 'x' || e[1] === 'X' ? parseInt(e.slice(2), 16) : parseInt(e.slice(1), 10);
            return Number.isFinite(n) && n > 0 && n < 0x110000 ? String.fromCodePoint(n) : m;
        }
        return ENTITIES[e] ?? m;
    });
}

/** The index just past the element that opens at `start` (a `<tag ...>`), counting nested elements of the same tag. */
function elementEnd(html, start, tag) {
    const re = new RegExp(`<(/?)${tag}\\b[^>]*>`, 'gi');
    re.lastIndex = start;
    let depth = 0;
    for (let m; (m = re.exec(html));) {
        if (m[1]) depth--;
        else if (!m[0].endsWith('/>')) depth++;
        if (depth === 0) return re.lastIndex;
    }
    return html.length;
}

/** The top level `<span class="cls">` elements of `html`, as [start, end, inner] (inner without the outer tags). */
export function spansOfClass(html, cls) {
    const out = [];
    const open = new RegExp(`<span class="${cls}"[^>]*>`, 'g');
    for (let m; (m = open.exec(html));) {
        const end = elementEnd(html, m.index, 'span');
        out.push([m.index, end, html.slice(m.index + m[0].length, end - '</span>'.length)]);
        open.lastIndex = end;
    }
    return out;
}

/** Fractions (`dvc` with a `dvl` bar) become a/b, innermost first, with brackets where a side has spaces. */
function fractions(html) {
    const OPEN = /<span class="dvc(?:\s[^"]*)?">/g;
    const BAR = /<span class="dvl(?:\s[^"]*)?"><\/span>/;
    for (let guard = 0; guard < 10000; guard++) {
        const all = [...html.matchAll(OPEN)].map((m) => [m.index, m[0].length]);
        if (!all.length) return html;
        let done = false;
        for (const [i, len] of all.reverse()) {
            const end = elementEnd(html, i, 'span');
            const inner = html.slice(i + len, end - '</span>'.length);
            if (OPEN.test(inner)) { OPEN.lastIndex = 0; continue; }
            OPEN.lastIndex = 0;
            const bar = BAR.exec(inner);
            const wrap = (t) => {
                const s = t.trim();
                return /\s[^\s]/.test(s.replace(/<[^>]+>/g, '')) ? `(${s})` : s;
            };
            const text = bar ? `${wrap(inner.slice(0, bar.index))}/${wrap(inner.slice(bar.index + bar[0].length))}` : inner;
            html = html.slice(0, i) + text + html.slice(end);
            done = true;
            break;
        }
        if (!done) return html;
    }
    return html;
}

/** Matrices (`<span class="matrix">` of `tr`/`td` spans) become [a b; c d]. */
function matrices(html) {
    for (const [start, end, inner] of spansOfClass(html, 'matrix').reverse()) {
        const rows = spansOfClass(inner, 'tr').map(([, , row]) =>
            spansOfClass(row, 'td').map(([, , cell]) => cell.trim()).filter((c) => c !== '').join(' '));
        html = html.slice(0, start) + `[${rows.join('; ')}]` + html.slice(end);
    }
    return html;
}

/** One fragment of the page as text on one line: sub/superscripts as _x and ^x, fractions as a/b, entities decoded. */
export function inlineText(html) {
    let s = html
        .replace(/<span class="vec">[^<]*<\/span>/g, '')
        .replace(/<span class="sheet-tex" data-tex="([^"]*)"><\/span>/g, (m, t) => `$${t}$`)
        .replace(/<i>\s*·\s*<\/i>/g, '·')
        .replace(/<span class="r">√<\/span>/g, '√');
    s = fractions(matrices(s));
    s = s.replace(/<sub\b[^>]*>([\s\S]*?)<\/sub>/g, '_$1')
        .replace(/<sup\b[^>]*>([\s\S]*?)<\/sup>/g, '^$1')
        .replace(/<[^>]+>/g, '');
    return decodeEntities(s).replace(/\s*∕\s*/g, '/').replace(/⃗/g, '').replace(/\s+/g, ' ')
        .replace(/\( /g, '(').replace(/ \)/g, ')').replace(/\) \^/g, ')^').trim();
}

/** The report as plain text: one line per paragraph, heading or table row; drawings and plots as placeholders. */
export function pageText(html) {
    let body = html.replace(/<(script|style|template)\b[\s\S]*?<\/\1>/gi, '');
    body = /<body\b[^>]*>([\s\S]*)<\/body>/i.exec(body)?.[1] ?? body;
    body = body
        .replace(/<span class="sheet-note">/g, '<span>| ')
        .replace(/<svg\b[^>]*class="plot"[\s\S]*?<\/svg>/gi, '\n[plot]\n')
        .replace(/<svg\b[\s\S]*?<\/svg>/gi, ' [drawing] ')
        .replace(/<img\b[^>]*class="plot"[^>]*>/gi, '\n[plot]\n')
        .replace(/<img\b[^>]*>/gi, ' [image] ')
        .replace(/<div class="errorHeader">[\s\S]*?<\/div>/g, '');
    // Block boundaries become a separator (NUL); each piece is then made text once.
    const SEP = '\u0000';
    body = body.replace(/\u0000/g, '')
        .replace(/<(h[1-6])\b[^>]*>/gi, (m, h) => `${SEP}${'#'.repeat(Number(h[1]))} `).replace(/<\/h[1-6]>/gi, SEP)
        .replace(/<tr\b[^>]*>/gi, `${SEP}| `).replace(/<\/t[hd]>/gi, ' | ').replace(/<\/tr>/gi, SEP)
        .replace(/<li\b[^>]*>/gi, `${SEP}- `).replace(/<\/li>/gi, SEP)
        .replace(/<\/?(p|div|ul|ol|table|thead|tbody|tfoot|blockquote|pre|details|summary)\b[^>]*>/gi, SEP)
        .replace(/<br\s*\/?>/gi, SEP)
        .replace(/\n\[plot\]\n/g, `${SEP}[plot]${SEP}`);
    const lines = body.split(SEP).map((l) => inlineText(l));
    const out = [];
    for (const l of lines) {
        if (l === '' && (out.length === 0 || out[out.length - 1] === '')) continue;
        out.push(l);
    }
    return out.join('\n').trim();
}

/**
 * The equation lines of the page: for each element with a data-source-line, each top level `eq` span as
 * { outputLine, text, lhs, note }. `note` is the sheet note (sheet-note span) or the rest of the paragraph's text.
 */
export function equationLines(html) {
    const lines = [];
    const re = /<p\b[^>]*\bdata-source-line="(\d+)"[^>]*>/g;
    for (let m; (m = re.exec(html));) {
        const end = elementEnd(html, m.index, 'p');
        const inner = html.slice(m.index + m[0].length, end - 4);
        re.lastIndex = end;
        const eqs = spansOfClass(inner, 'eq');
        if (!eqs.length) continue;
        const note = spansOfClass(inner, 'sheet-note').map(([, , t]) => inlineText(t)).join(' ');
        let rest = inner;
        for (const [s, e] of [...eqs].reverse()) rest = rest.slice(0, s) + ' ' + rest.slice(e);
        const comment = note || inlineText(rest.replace(/<span class="sheet-(key|note)"[\s\S]*?<\/span>/g, '')).replace(/^[;,\s]+|[;,\s]+$/g, '');
        for (const [, , eq] of eqs) {
            const text = inlineText(eq);
            const lhs = /^([^=]+?)\s*=/.exec(text)?.[1]?.trim() ?? '';
            lines.push({ outputLine: Number(m[1]), text, lhs, note: comment });
        }
    }
    return lines;
}

/** The error messages the page shows (`err` elements), as text. */
export function pageErrors(html) {
    return [...html.matchAll(/<(p|span)\b[^>]*class="err"[^>]*data-source-line="(\d+)"[^>]*>([\s\S]*?)<\/\1>/g)]
        .map((m) => ({ outputLine: Number(m[2]), message: inlineText(m[3]) }));
}
