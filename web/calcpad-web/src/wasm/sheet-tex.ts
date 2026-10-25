// calcpad-rs: LaTeX in sheet prose. The compiler leaves placeholders
//   <span class="sheet-tex" data-tex="..."></span>   (and a <div> with data-display="1")
// in the report; this module fills them with KaTeX (MIT), bundled with the app and loaded the first
// time a report contains one. The result is markup and styles only, no script, so the preview's
// content security policy is untouched: the stylesheet is inlined, and the fonts a formula
// uses go in as data: URLs (the policy allows font-src data:). The bundle is also what makes the
// report print and save without the page.

type Katex = { renderToString: (tex: string, opts: Record<string, unknown>) => string };

let katex: Promise<Katex> | null = null;
let rules: Promise<string> | null = null;

function loadKatex(): Promise<Katex> {
    katex ??= import('katex').then(m => (m as unknown as { default: Katex }).default ?? (m as unknown as Katex));
    return katex;
}

/** KaTeX's stylesheet without its @font-face rules (those are written per page). */
function loadRules(): Promise<string> {
    rules ??= import('katex/dist/katex.min.css?raw').then(m => (m.default as string).replace(/@font-face\s*\{[^}]*\}/g, ''));
    return rules;
}

/** Each font file, the page text that shows it is needed, and how to fetch it. */
const FONTS: { name: string; family: string; weight: string; style: string; when: RegExp; url: () => Promise<{ default: string }> }[] = [
    { name: 'Main-Regular', family: 'KaTeX_Main', weight: 'normal', style: 'normal', when: /./, url: () => import('katex/dist/fonts/KaTeX_Main-Regular.woff2?url') },
    { name: 'Main-Bold', family: 'KaTeX_Main', weight: 'bold', style: 'normal', when: /mathbf|textbf|boldsymbol/, url: () => import('katex/dist/fonts/KaTeX_Main-Bold.woff2?url') },
    { name: 'Main-Italic', family: 'KaTeX_Main', weight: 'normal', style: 'italic', when: /mathit|textit|emph/, url: () => import('katex/dist/fonts/KaTeX_Main-Italic.woff2?url') },
    { name: 'Main-BoldItalic', family: 'KaTeX_Main', weight: 'bold', style: 'italic', when: /mathbf.*mathit|boldsymbol/, url: () => import('katex/dist/fonts/KaTeX_Main-BoldItalic.woff2?url') },
    { name: 'Math-Italic', family: 'KaTeX_Math', weight: 'normal', style: 'italic', when: /mathnormal|mathdefault/, url: () => import('katex/dist/fonts/KaTeX_Math-Italic.woff2?url') },
    { name: 'Math-BoldItalic', family: 'KaTeX_Math', weight: 'bold', style: 'italic', when: /boldsymbol/, url: () => import('katex/dist/fonts/KaTeX_Math-BoldItalic.woff2?url') },
    { name: 'AMS-Regular', family: 'KaTeX_AMS', weight: 'normal', style: 'normal', when: /amsrm|mathbb/, url: () => import('katex/dist/fonts/KaTeX_AMS-Regular.woff2?url') },
    { name: 'Caligraphic-Regular', family: 'KaTeX_Caligraphic', weight: 'normal', style: 'normal', when: /mathcal/, url: () => import('katex/dist/fonts/KaTeX_Caligraphic-Regular.woff2?url') },
    { name: 'Fraktur-Regular', family: 'KaTeX_Fraktur', weight: 'normal', style: 'normal', when: /mathfrak/, url: () => import('katex/dist/fonts/KaTeX_Fraktur-Regular.woff2?url') },
    { name: 'SansSerif-Regular', family: 'KaTeX_SansSerif', weight: 'normal', style: 'normal', when: /mathsf|textsf/, url: () => import('katex/dist/fonts/KaTeX_SansSerif-Regular.woff2?url') },
    { name: 'Script-Regular', family: 'KaTeX_Script', weight: 'normal', style: 'normal', when: /mathscr/, url: () => import('katex/dist/fonts/KaTeX_Script-Regular.woff2?url') },
    { name: 'Size1-Regular', family: 'KaTeX_Size1', weight: 'normal', style: 'normal', when: /size1|op-symbol small-op/, url: () => import('katex/dist/fonts/KaTeX_Size1-Regular.woff2?url') },
    { name: 'Size2-Regular', family: 'KaTeX_Size2', weight: 'normal', style: 'normal', when: /size2|op-symbol large-op/, url: () => import('katex/dist/fonts/KaTeX_Size2-Regular.woff2?url') },
    { name: 'Size3-Regular', family: 'KaTeX_Size3', weight: 'normal', style: 'normal', when: /size3/, url: () => import('katex/dist/fonts/KaTeX_Size3-Regular.woff2?url') },
    { name: 'Size4-Regular', family: 'KaTeX_Size4', weight: 'normal', style: 'normal', when: /size4/, url: () => import('katex/dist/fonts/KaTeX_Size4-Regular.woff2?url') },
    { name: 'Typewriter-Regular', family: 'KaTeX_Typewriter', weight: 'normal', style: 'normal', when: /mathtt|texttt/, url: () => import('katex/dist/fonts/KaTeX_Typewriter-Regular.woff2?url') },
];

const fontData = new Map<string, Promise<string>>();

function dataUrl(name: string, url: () => Promise<{ default: string }>): Promise<string> {
    let p = fontData.get(name);
    if (!p) {
        p = (async () => {
            const res = await fetch((await url()).default);
            const bytes = new Uint8Array(await res.arrayBuffer());
            let bin = '';
            for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
            return `data:font/woff2;base64,${btoa(bin)}`;
        })();
        p.catch(() => fontData.delete(name));
        fontData.set(name, p);
    }
    return p;
}

/** A numeric character reference list (the compiler's encoding) back to text. */
function decode(s: string): string {
    return s.replace(/&#(\d+);/g, (_m, n: string) => String.fromCodePoint(Number(n))).replace(/&quot;/g, '"').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
}

const PLACEHOLDER = /<(span|div)\b([^>]*\bclass="sheet-tex"[^>]*)>\s*<\/\1>/g;

/** True if the page has formula placeholders. */
export function hasTex(html: string): boolean {
    return html.includes('class="sheet-tex"');
}

/** Fills the placeholders with KaTeX output and adds the stylesheet. */
export async function renderTex(html: string): Promise<string> {
    if (!hasTex(html)) return html;
    const k = await loadKatex();
    const out = html.replace(PLACEHOLDER, (_m, tag: string, attrs: string) => {
        const tex = /data-tex="([^"]*)"/.exec(attrs)?.[1] ?? '';
        const display = /data-display="1"/.test(attrs);
        const markup = k.renderToString(decode(tex), { displayMode: display, throwOnError: false, output: 'htmlAndMathml', strict: 'ignore' });
        return `<${tag}${attrs}>${markup}</${tag}>`;
    });
    const css = await loadRules();
    let faces = '';
    for (const f of FONTS) {
        if (!f.when.test(out)) continue;
        try {
            faces += `@font-face{font-family:${f.family};font-weight:${f.weight};font-style:${f.style};src:url(${await dataUrl(f.name, f.url)}) format('woff2')}`;
        } catch { /* the font falls back to a system one */ }
    }
    const style = `<style id="cprs-katex">${faces}${css}`
        + '.sheet-tex[data-display="1"]{margin:.6em 0;text-align:center}.dark-theme .katex{color:#d4d4d4}.sheet-note .katex{color:inherit}'
        // Variables (KaTeX's italic math letters) in the colour the engine gives a variable in an equation.
        + '.katex .mathnormal{color:#06d}.dark-theme .katex .mathnormal{color:#569cd6}.sheet-note .katex .mathnormal{color:inherit}</style>';
    const i = out.toLowerCase().lastIndexOf('</head>');
    return i < 0 ? style + out : out.slice(0, i) + style + out.slice(i);
}
