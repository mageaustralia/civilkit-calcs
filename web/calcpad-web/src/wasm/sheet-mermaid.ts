// calcpad-rs: diagrams from ```mermaid fences. The Markdown renderer writes a fence as
//   <pre ...><code class="language-mermaid">source</code></pre>
// and this module swaps each one for the diagram as inline SVG, drawn by Mermaid (MIT). Mermaid is
// bundled with the app as its own chunks and imported the first time a page has such a fence, so a
// page without one loads nothing more, and the service worker's precache makes it work offline.
// Like the KaTeX formulas (sheet-tex.ts), the result is markup and styles only: the preview's
// content security policy is unchanged, and the diagram prints and exports with the page.
// Labels are SVG text, not HTML in a foreignObject, so the drawing stands alone outside a browser.
// A fence Mermaid cannot read stays as its source, with the reason under it.

type Mermaid = {
    initialize: (config: Record<string, unknown>) => void;
    render: (id: string, text: string) => Promise<{ svg: string }>;
};

let mermaid: Promise<Mermaid> | null = null;

function loadMermaid(): Promise<Mermaid> {
    mermaid ??= import('mermaid').then(m => {
        const mm = (m as unknown as { default: Mermaid }).default ?? (m as unknown as Mermaid);
        // 'strict' sanitises labels and turns off click handlers and scripts in a diagram. The
        // dagre layout (MIT): the elk one (EPL-2.0) is not shipped, see elk-stub.ts.
        mm.initialize({
            startOnLoad: false, securityLevel: 'strict', theme: 'neutral', layout: 'dagre', htmlLabels: false,
            flowchart: { htmlLabels: false }, suppressErrorRendering: true, fontFamily: 'system-ui, -apple-system, "Segoe UI", sans-serif',
        });
        return mm;
    });
    mermaid.catch(() => { mermaid = null; });
    return mermaid;
}

const FENCE = /<pre\b([^>]*)>\s*<code\b[^>]*\bclass="[^"]*\blanguage-mermaid\b[^"]*"[^>]*>([\s\S]*?)<\/code>([\s\S]*?)<\/pre>/g;

/** True if the page has a mermaid fence. */
export function hasMermaid(html: string): boolean {
    return html.includes('language-mermaid');
}

/** Code text as the renderer escaped it, back to the diagram source. */
export function decodeCode(s: string): string {
    return s.replace(/&#(\d+);/g, (_m, n: string) => String.fromCodePoint(Number(n)))
        .replace(/&#x([0-9a-f]+);/gi, (_m, n: string) => String.fromCodePoint(parseInt(n, 16)))
        .replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
}

function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** A short stable id for a diagram source (Mermaid scopes the SVG's styles by it). */
export function diagramId(src: string): string {
    let h = 2166136261;
    for (let i = 0; i < src.length; i++) h = Math.imul(h ^ src.charCodeAt(i), 16777619);
    return 'ck-mermaid-' + (h >>> 0).toString(36);
}

// Drawn diagrams by source, so a re-render after each edit only draws what changed.
const drawn = new Map<string, Promise<string>>();
const DRAWN_MAX = 64;

function draw(src: string): Promise<string> {
    let p = drawn.get(src);
    if (!p) {
        p = loadMermaid().then(m => m.render(diagramId(src), src)).then(r => r.svg);
        p.catch(() => drawn.delete(src));
        if (drawn.size >= DRAWN_MAX) drawn.delete(drawn.keys().next().value as string);
        drawn.set(src, p);
    }
    return p;
}

const STYLE = '<style id="cprs-mermaid">.sheet-mermaid{margin:.6em 0;padding:8px;text-align:center;background:#fff;border-radius:4px;overflow-x:auto}'
    + '.sheet-mermaid svg{max-width:100%;height:auto}'
    + '.sheet-mermaid-error{margin:0 0 .6em;color:#900;font:10pt sans-serif}'
    + '@media print{.sheet-mermaid{padding:0;break-inside:avoid}}</style>';

/** Replaces each mermaid fence with its diagram, and adds the stylesheet. */
export async function renderMermaid(html: string): Promise<string> {
    if (!hasMermaid(html)) return html;
    const fences: { attrs: string; src: string; tail: string }[] = [];
    html.replace(FENCE, (_m, attrs: string, code: string, tail: string) => { fences.push({ attrs, src: decodeCode(code).trim(), tail }); return ''; });
    if (!fences.length) return html;
    const out = await Promise.all(fences.map(async f => {
        try {
            const svg = await draw(f.src);
            return `<div${f.attrs.replace(/\bclass="/, 'class="sheet-mermaid ')}${/\bclass="/.test(f.attrs) ? '' : ' class="sheet-mermaid"'} role="img" aria-label="Diagram">${svg}${f.tail}</div>`;
        } catch (e) {
            const why = e instanceof Error ? e.message.split('\n')[0] : String(e);
            return `<pre${f.attrs}><code class="language-mermaid">${escapeHtml(f.src)}</code>${f.tail}</pre>`
                + `<p class="sheet-mermaid-error">Diagram not drawn: ${escapeHtml(why)}</p>`;
        }
    }));
    let i = 0;
    const body = html.replace(FENCE, () => out[i++]);
    const at = body.toLowerCase().lastIndexOf('</head>');
    return at < 0 ? STYLE + body : body.slice(0, at) + STYLE + body.slice(at);
}
