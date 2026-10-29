// The embed catalogue page, public/embed/index.html (/calcs/embed/): what an embed is, a live example, and a copy and
// paste snippet for every sample a landing page uses. Written by make_learn_pages.mjs with the landing pages (and so
// checked by its --check); web/e2e/seo-check.mjs checks its head, breadcrumb and links. See docs/embedding.md.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { samplesDir } from './engine.mjs';
import { pages } from './pages.mjs';
import { SITE, PRODUCT, BASE, EMBED, EMBED_HEIGHT, REPO, esc, head, header, crumbs, footer, breadcrumbs, SOFTWARE, labelOf } from './render.mjs';

export const EMBED_TITLE = 'Embed a free engineering calculator on your site';
export const EMBED_DESCRIPTION = "Put a live engineering calculator on your own page with one iframe. Free, MIT licensed, no tracking, and it calculates in the reader's browser.";
const APP = `${SITE}/calcs/`;
const SANDBOX = 'allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox';

/** The samples of the landing pages, in page order, once each: { id, title, description, slug }. */
export function embedSamples() {
    const manifest = JSON.parse(readFileSync(join(samplesDir, 'manifest.json'), 'utf8'));
    const byId = new Map(manifest.samples.map((s) => [s.id, s]));
    const out = [];
    for (const p of pages) {
        if (!p.open || out.some((s) => s.id === p.open)) continue;
        const s = byId.get(p.open);
        if (!s) throw new Error(`embed page: no sample ${p.open} (page ${p.slug})`);
        out.push({ id: s.id, title: s.title.replace(/^Template:\s*/, ''), description: s.description, slug: p.slug });
    }
    return out;
}

/** The copy and paste iframe of one sample. */
export const snippet = (id, title) =>
    `<iframe src="${APP}?embed=1&sample=${id}" width="100%" height="${EMBED_HEIGHT}" style="border:1px solid #ccc" loading="lazy" title="${esc(title)}" allow="" sandbox="${SANDBOX}"></iframe>`;

const code = (text, id) => `<pre><code id="${id}">${esc(text)}</code></pre>\n<p class="copyrow"><button type="button" class="copy" data-copy="${id}" hidden>Copy</button></p>`;

// Progressive enhancement: without JavaScript the snippets are plain text to select; with it, each gets a Copy button.
const COPY_SCRIPT = `<script>
document.querySelectorAll('button.copy').forEach(function (b) {
  if (!navigator.clipboard) return;
  b.hidden = false;
  b.addEventListener('click', function () {
    var t = document.getElementById(b.getAttribute('data-copy')).textContent;
    navigator.clipboard.writeText(t).then(function () { b.textContent = 'Copied'; setTimeout(function () { b.textContent = 'Copy'; }, 1500); });
  });
});
</script>`;

const STYLE = `<style>
.copyrow { margin: -0.3rem 0 1rem; }
.copy { font: inherit; font-size: 0.875rem; padding: 0.3rem 0.9rem; border-radius: 6px; border: 2px solid var(--accent); background: transparent; color: var(--accent); cursor: pointer; }
.copy:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
iframe.embed { display: block; width: 100%; border: 1px solid var(--line); border-radius: 6px; margin: 0.75rem 0; }
.samples h3 { margin-top: 1.5rem; }
</style>`;

export function renderEmbedPage() {
    const samples = embedSamples();
    const first = samples[0];
    const ld = {
        '@context': 'https://schema.org',
        '@graph': [
            breadcrumbs([['Embed', EMBED]], []),
            { '@type': 'WebPage', name: EMBED_TITLE, url: EMBED, description: EMBED_DESCRIPTION, isPartOf: { '@type': 'WebSite', name: 'CivilKit', url: `${SITE}/` }, about: SOFTWARE },
        ],
    };
    const list = samples.map((s) => `<h3 id="s-${s.id}">${esc(s.title)}</h3>
<p>${esc(s.description)} About it: <a href="${BASE}${s.slug}/">${esc(labelOf(s.slug))}</a>.</p>
${code(snippet(s.id, `${s.title}: CivilKit Calcs`), `code-${s.id}`)}`).join('\n');
    return `${head({ title: EMBED_TITLE, description: EMBED_DESCRIPTION, url: EMBED, ld }).replace('</head>', `${STYLE}\n</head>`)}
<body>
${header()}
<main id="main">
${crumbs([['Embed']])}
<h1>Embed a live engineering calculator</h1>
<div class="lead">
<p>An embed is a small window onto a CivilKit Calcs sheet that sits on your own web page: the reader changes the inputs and the results recalculate on the spot, with the workings shown. You add it with one line of HTML. It is free, MIT licensed and has no tracking: the calculation runs in the reader's browser and nothing is sent to a server.</p>
</div>
<p class="cta"><a class="button" href="#samples">Pick a calculator</a> <a class="button ghost" href="/calcs/guide/embedding">Read the embedding guide</a></p>
<section>
<h2>A live example</h2>
<p>This is the simply supported beam template, embedded exactly as the snippet below embeds it. Drag the span slider or type a load.</p>
<iframe class="embed" src="/calcs/?embed=1&amp;sample=${esc(first.id)}" width="100%" height="${EMBED_HEIGHT}" loading="lazy" title="${esc(first.title)}: live sheet" sandbox="${SANDBOX}"></iframe>
<p>The link at its top right, Open in CivilKit Calcs, opens the same sheet in the full app in a new tab, where the reader can edit it, save it and export a report.</p>
</section>
<section>
<h2>How to add one</h2>
<p>Copy the snippet of a calculator below into your page's HTML. That is all: no account, no key, no script of your own.</p>
${code(snippet(first.id, `${first.title}: CivilKit Calcs`), 'code-example')}
<p>Or let a small script place the frames and fit their height to the content: put a placeholder where each calculator goes and load the script once (about 3 KB, no dependencies).</p>
${code(`<div data-civilkit-sample="${first.id}" data-height="${EMBED_HEIGHT}" data-theme="auto"></div>\n<script src="${APP}embed.js" async></script>`, 'code-script')}
<h3>Options</h3>
<p>Add these to the address of the frame (or as data attributes of the placeholder):</p>
<ul>
<li><code>theme=light</code>, <code>theme=dark</code> or <code>theme=auto</code> (the default follows the reader's system setting).</li>
<li><code>mode=input</code> (only the form), <code>mode=results</code> (only the workings) or <code>mode=both</code> (the default: the form beside or above the results).</li>
<li><code>hide=title</code> leaves out the sheet's title.</li>
<li><code>embed=edit</code> in place of <code>embed=1</code> adds a small text editor under the results, for readers who want to change the sheet itself.</li>
</ul>
</section>
<section>
<h2>Sandboxing</h2>
<p>The snippets carry <code>sandbox="${SANDBOX}"</code>. It keeps the frame away from your page (no top navigation, no forms on your origin) while letting the sheet calculate and the Open in CivilKit Calcs link open a normal tab. The frame asks for no browser permissions (<code>allow=""</code>). Python blocks never run in an embed unless the address says <code>allowpython=1</code>, and even then the reader is asked first.</p>
</section>
<section>
<h2>Talking to the frame</h2>
<p>A page can set inputs and read results with <code>postMessage</code>. The frame answers only its own parent page and checks every value.</p>
<ul>
<li>Send <code>{type: 'civilkit-set', values: {L: 8}}</code> to set inputs by name. A number must be finite and within the slider's range; a choice must be one of its options. Anything else is refused with <code>{type: 'civilkit-error', errors: [...]}</code>.</li>
<li>After each calculation the frame sends <code>{type: 'civilkit-results', values: {name: {value, unit, text}}}</code>.</li>
<li>It also sends <code>{type: 'civilkit-ready', inputs: [...]}</code> with the inputs and their ranges, and <code>{type: 'civilkit-embed-height', height}</code> whenever its height changes.</li>
</ul>
<p>Check that a message comes from <code>${SITE}</code> before you use it. The <a href="/calcs/guide/embedding">embedding guide</a> has a complete example.</p>
</section>
<section class="samples" id="samples">
<h2>Calculators to embed</h2>
<p>Each of these is a sheet from a <a href="${BASE}">calculator page</a>. Its numbers are generic textbook calculations, not design-code checks.</p>
${list}
</section>
<section>
<h2>Free, open and private</h2>
<p>CivilKit Calcs is free and open source under the MIT licence, so you may embed it anywhere, commercial sites included. The embed sets no cookies, stores nothing, loads no analytics and sends nothing anywhere: the engine runs in the reader's browser. Every embed links back to the full app with Open in CivilKit Calcs. You can also host your own copy from the <a href="${REPO}">source on GitHub</a>.</p>
</section>
</main>
${footer()}
${COPY_SCRIPT}
</body>
</html>
`;
}
