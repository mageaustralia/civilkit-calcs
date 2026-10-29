// The shared template for the landing pages under public/learn/. Pure functions: a page's data (pages.mjs) and the
// values of its engine runs go in, one HTML string comes out. make_learn_pages.mjs writes the files and
// web/e2e/learn-check.mjs renders again and compares, so a page cannot drift from its data or from the engine.
import { runSheet, runCpd, readSample, valueOf } from './engine.mjs';
import { pages } from './pages.mjs';
import { guidePath, chapterTitle } from '../guide_url.mjs';

export const SITE = 'https://civilkit.io';
export const BASE = '/calcs/learn/';
export const urlOf = (slug) => `${SITE}${BASE}${slug}/`;
/** The hub that lists every landing page. */
export const HUB = `${SITE}${BASE}`;
/** The product page of CivilKit Calcs (what "CivilKit Calcs" links to); the app itself (/calcs/) is only for "Open Calcs". */
export const PRODUCT = `${SITE}/calc/`;
/** The site path of a guide chapter, in its canonical extensionless form: 'units.md' -> /calcs/guide/units. */
export const guideHref = (md) => `/calcs/guide/${guidePath(md)}`;
export const REPO = 'https://github.com/mageaustralia/civilkit-calcs';
export const CALCPADCE = 'https://github.com/imartincei/CalcpadCE';

export const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const strip = (html) => html.replace(/<[^>]+>/g, '').replace(/&nbsp;/g, ' ').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();

/** The source text of one run of a page: a sample file with some input lines replaced, an inline sheet or Calcpad text. */
export function sourceOf(run) {
    if (run.sample) {
        let t = readSample(run.sample);
        // A Python fence runs only in the browser; the engine numbers quoted from such a sheet come from its other lines.
        if (run.dropPython) t = t.replace(/^```python[^\n]*\n[\s\S]*?^```[ \t]*\n?/gm, '');
        for (const [from, to] of run.replace ?? []) {
            if (!t.includes(from)) throw new Error(`${run.sample}: no text ${from}`);
            t = t.replace(from, to);
        }
        if (run.append) t = t.trimEnd() + '\n\n' + run.append;
        for (const [name, line] of Object.entries(run.set ?? {})) {
            const re = new RegExp(`^${name}\\s*:\\s*\\?.*$`, 'm');
            if (!re.test(t)) throw new Error(`${run.sample}: no input line for ${name}`);
            t = t.replace(re, line);
        }
        return t;
    }
    return run.sheet ?? run.cpd;
}

/** Runs every run of a page. Returns { id: engineRun }. A run with errors stops the build. */
export function runAll(page) {
    const out = {};
    for (const [id, run] of Object.entries(page.runs ?? {})) {
        const r = run.cpd ? runCpd(sourceOf(run)) : runSheet(sourceOf(run));
        if (r.errors.length) throw new Error(`${page.slug} run ${id}: ${r.errors.join('; ')}`);
        out[id] = r;
    }
    return out;
}

/** Replaces [[run:name]] with the engine's value in a marked span and {{code:run}} with the run's source. */
function fill(html, page, runs) {
    return html
        .replace(/\[\[(\w+):([^\]]+)\]\]/g, (m, id, key) => {
            const r = runs[id];
            if (!r) throw new Error(`${page.slug}: no run ${id}`);
            const v = valueOf(r, key);
            if (v == null) throw new Error(`${page.slug}: run ${id} has no value ${key}`);
            return `<span class="v" data-run="${id}" data-key="${esc(key)}">${esc(v)}</span>`;
        })
        .replace(/\{\{code:(\w+)\}\}/g, (m, id) => {
            const run = page.runs[id];
            if (!run) throw new Error(`${page.slug}: no run ${id}`);
            return `<pre><code>${esc(sourceOf(run).replace(/^<!--sheet-->\n/, '').trimEnd())}</code></pre>`;
        });
}

/** The embed catalogue page (tools/learn/embed.mjs). */
export const EMBED = `${SITE}/calcs/embed/`;
const NAV = [[PRODUCT, 'Calcs home'], [BASE, 'Calculators and guides'], ['/calcs/guide/', 'User guide'], ['/calcs/whats-new/', "What's new"], ['/calcs/', 'Open Calcs']];
const LABEL = Object.fromEntries(pages.map((p) => [p.slug, p.label]));
export const labelOf = (slug) => LABEL[slug];

function jsonld(page, runs, faq) {
    const url = urlOf(page.slug);
    const graph = [];
    graph.push(breadcrumbs([[LABEL[page.slug], url]]));
    if (page.schema?.includes('software')) graph.push(SOFTWARE);
    if (page.steps) {
        graph.push({
            '@type': 'HowTo',
            name: page.howto,
            description: page.description,
            step: page.steps.map(([name, textHtml], i) => ({ '@type': 'HowToStep', position: i + 1, name, text: strip(fill(textHtml, page, runs)) })),
        });
    }
    graph.push({
        '@type': 'FAQPage',
        mainEntity: faq.map(([q, a]) => ({ '@type': 'Question', name: q, acceptedAnswer: { '@type': 'Answer', text: strip(a) } })),
    });
    return { '@context': 'https://schema.org', '@graph': graph };
}

/** The BreadcrumbList of a Calcs content page: CivilKit > Calcs (the product page) > Calculators and guides > the page. */
export function breadcrumbs(tail, middle = [['Calculators and guides', HUB]]) {
    const items = [['CivilKit', `${SITE}/`], ['Calcs', PRODUCT], ...middle, ...tail].filter(([, u], i, a) => a.findIndex(([, v]) => v === u) === i);
    return { '@type': 'BreadcrumbList', itemListElement: items.map(([name, item], i) => ({ '@type': 'ListItem', position: i + 1, name, item })) };
}

/** The product node, shared by the comparison pages and the hub. */
export const SOFTWARE = {
    '@type': 'SoftwareApplication',
    name: 'CivilKit Calcs',
    url: PRODUCT,
    description: 'A free, open source engineering calculation sheet that runs in the browser. Real units, plots, input forms, Word, Excel and PDF export, offline use.',
    applicationCategory: 'EngineeringApplication',
    applicationSubCategory: 'Engineering calculation',
    operatingSystem: 'Any',
    browserRequirements: 'A current web browser with WebAssembly',
    isAccessibleForFree: true,
    license: 'https://opensource.org/license/mit',
    offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
};

/** The page head shared by the landing pages, the hub and the embed page. */
export function head({ title, description, url, ld }) {
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<meta name="robots" content="index, follow, max-image-preview:large">
<meta name="color-scheme" content="light dark">
<link rel="canonical" href="${url}">
<link rel="stylesheet" href="${BASE}learn.css">
<link rel="icon" href="/calcs/icon-192.png" sizes="192x192">
<meta property="og:type" content="website">
<meta property="og:site_name" content="CivilKit Calcs">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(description)}">
<meta property="og:url" content="${url}">
<meta property="og:image" content="${SITE}/calcs/icon-512.png">
<meta name="twitter:card" content="summary">
<meta name="twitter:title" content="${esc(title)}">
<meta name="twitter:description" content="${esc(description)}">
<meta name="twitter:image" content="${SITE}/calcs/icon-512.png">
<script type="application/ld+json">
${JSON.stringify(ld, null, 1).replace(/</g, '\\u003c')}
</script>
</head>`;
}

export const header = () => `<a class="skip" href="#main">Skip to the content</a>
<header class="site">
<a class="brand" href="${PRODUCT}">CivilKit Calcs</a>
<nav aria-label="Site">
${NAV.map(([h, t]) => `<a href="${h}">${esc(t)}</a>`).join('\n')}
</nav>
</header>`;

/** The breadcrumb trail shown on the page; the last entry is the current page (no link). */
export const crumbs = (trail) => `<nav class="crumbs" aria-label="Breadcrumb"><a href="${SITE}/">CivilKit</a> / <a href="${PRODUCT}">Calcs</a> / ${trail.map(([t, h]) => (h ? `<a href="${h}">${esc(t)}</a>` : `<span>${esc(t)}</span>`)).join(' / ')}</nav>`;

export const footer = () => `<footer class="site">
<p><a href="${SITE}/">CivilKit</a> · <a href="${PRODUCT}">CivilKit Calcs</a> · <a href="${BASE}">Calculators and guides</a> · <a href="/calcs/guide/">User guide</a> · <a href="/calcs/whats-new/">What's new</a> · <a href="${REPO}">Source on GitHub</a></p>
<p>CivilKit Calcs is free and open source (MIT licence). It runs in your browser: your calculations are not sent to a server. It is built on <a href="${CALCPADCE}">CalcpadCE</a> by Nedelcho Ganchovski (Proektsoft) and contributors.</p>
<p>Product names on this page belong to their owners. CivilKit Calcs is not affiliated with or endorsed by them.</p>
</footer>`;

/**
 * The "Try it" section of a page with a sample: the sample live in an embed (docs/embedding.md). The frame loads lazily
 * and has a fixed height, so the page stays fast and does not shift; the link under it works without the frame.
 */
export function tryIt(page) {
    if (!page.open) return '';
    return `
<section class="try" id="try-it">
<h2>Try it</h2>
<iframe class="embed" src="/calcs/?embed=1&amp;sample=${esc(page.open)}" width="100%" height="${EMBED_HEIGHT}" loading="lazy" title="${esc(LABEL[page.slug])}: live sheet" sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"></iframe>
<p class="fallback"><a href="/calcs/?sample=${esc(page.open)}">Open the sheet in Calcs</a> or <a href="/calcs/embed/">embed it</a>.</p>
</section>`;
}
/** The height of an embedded sheet on the landing pages, in pixels (fixed, so nothing moves while it loads). */
export const EMBED_HEIGHT = 560;

/** One complete page. `runs` is runAll(page). */
export function renderPage(page, runs) {
    const url = urlOf(page.slug);
    const faq = page.faq.map(([q, a]) => [q, fill(a, page, runs)]);
    const video0 = page.video ? `<figure class="video">
<video controls preload="none" width="960" height="540" aria-label="${esc(page.video.title)}" src="/calcs/guide/videos/${page.video.file}"></video>
<figcaption>Video: ${esc(page.video.title)} (about ${page.video.seconds ?? 85} seconds). Read the chapter <a href="${guideHref(page.video.chapter)}">${esc(chapterTitle(page.video.chapter))}</a>.</figcaption>
</figure>` : '';
    const body = page.sections.map(([h, html], i) => `<section>\n<h2>${esc(h)}</h2>\n${fill(html, page, runs)}\n</section>${i === 0 ? '\n' + video0 + tryIt(page) : ''}`).join('\n');
    const openLabel = page.openLabel ?? (page.open?.startsWith('template-') ? 'Open this template' : 'Open this sheet in Calcs');
    const cta = page.open
        ? `<p class="cta"><a class="button" href="/calcs/?sample=${esc(page.open)}">${esc(openLabel)}</a> <a class="button ghost" href="/calcs/">Open a blank sheet</a></p>`
        : `<p class="cta"><a class="button" href="/calcs/">${esc(page.cta ?? 'Open Calcs')}</a> <a class="button ghost" href="/calcs/guide/">Read the guide</a></p>`;
    const related = [...page.related.map((s) => `<li><a href="${BASE}${s}/">${esc(LABEL[s])}</a></li>`), `<li><a href="${BASE}">All calculators and guides</a></li>`].join('\n');
    const faqHtml = faq.map(([q, a]) => `<h3>${esc(q)}</h3>\n<p>${a}</p>`).join('\n');
    return `${head({ title: page.title, description: page.description, url, ld: jsonld(page, runs, faq) })}
<body>
${header()}
<main id="main">
${crumbs([['Calculators and guides', BASE], [LABEL[page.slug]]])}
<h1>${esc(page.h1)}</h1>
<div class="lead">
${fill(page.lead, page, runs)}
</div>
${cta}
${body}
<section class="faq">
<h2>Questions</h2>
${faqHtml}
</section>
<section class="related">
<h2>Related pages</h2>
<ul>
${related}
</ul>
</section>
${cta}
</main>
${footer()}
</body>
</html>
`;
}

/** The groups of the hub, in order: [group id, heading, intro, [area, subheading]...]. */
export const GROUPS = [
    ['calculator', 'Calculators', 'Each calculator is a sheet that opens in the app as a form, with a worked example whose numbers come from the engine. They are generic textbook calculations, not design-code checks.',
        [['structural', 'Structural'], ['geotechnical', 'Geotechnical'], ['hydraulics', 'Hydraulics'], ['materials', 'Materials']]],
    ['alternative', 'Alternatives and migration', 'For engineers comparing tools, or bringing worksheets across from another one.', []],
    ['guide', 'Guides', 'How-to pages on what a calculation sheet can do. The full manual is the <a href="/calcs/guide/">user guide</a>, and every release is on <a href="/calcs/whats-new/">What\'s new</a>.', []],
];
export const HUB_TITLE = 'Engineering calculators and guides: CivilKit Calcs';
export const HUB_DESCRIPTION = 'Free engineering calculators that open as live sheets in your browser, pages for switching from other tools, and guides to units, import and export.';

/** The hub page, /calcs/learn/: every landing page grouped by purpose, with its description. */
export function renderHub() {
    const listed = [];
    const item = (p) => { listed.push(p); return `<li><a href="${BASE}${p.slug}/">${esc(p.label)}</a>: ${esc(p.description)}</li>`; };
    const sections = GROUPS.map(([id, heading, intro, areas]) => {
        const inGroup = pages.filter((p) => p.group === id);
        if (!inGroup.length) return '';
        const lists = areas.length
            ? areas.map(([a, sub]) => { const ps = inGroup.filter((p) => p.area === a); return ps.length ? `<h3>${esc(sub)}</h3>\n<ul>\n${ps.map(item).join('\n')}\n</ul>` : ''; }).filter(Boolean).join('\n')
            : `<ul>\n${inGroup.map(item).join('\n')}\n</ul>`;
        return `<section id="${id}">\n<h2>${esc(heading)}</h2>\n<p>${intro}</p>\n${lists}\n</section>`;
    }).filter(Boolean).join('\n');
    const missing = pages.filter((p) => !listed.includes(p));
    if (missing.length) throw new Error(`hub: no group for ${missing.map((p) => p.slug).join(', ')}`);
    const ld = {
        '@context': 'https://schema.org',
        '@graph': [
            breadcrumbs([], [['Calculators and guides', HUB]]),
            {
                '@type': 'CollectionPage',
                name: 'Calculators and guides',
                url: HUB,
                description: HUB_DESCRIPTION,
                isPartOf: { '@type': 'WebSite', name: 'CivilKit', url: `${SITE}/` },
                about: SOFTWARE,
                mainEntity: {
                    '@type': 'ItemList',
                    numberOfItems: listed.length,
                    itemListElement: listed.map((p, i) => ({ '@type': 'ListItem', position: i + 1, name: p.label, url: urlOf(p.slug) })),
                },
            },
        ],
    };
    return `${head({ title: HUB_TITLE, description: HUB_DESCRIPTION, url: HUB, ld })}
<body>
${header()}
<main id="main">
${crumbs([['Calculators and guides']])}
<h1>Calculators and guides</h1>
<div class="lead">
<p>CivilKit Calcs is a free calculation sheet for engineers that runs in your browser and keeps working offline. These pages show what it does on real problems: calculators that open as fill-in forms, notes for people coming from another tool, and short guides. Every number quoted on them comes from a run of the engine.</p>
</div>
<p class="cta"><a class="button" href="/calcs/">Open Calcs</a> <a class="button ghost" href="/calcs/guide/">Read the user guide</a></p>
${sections}
<section id="embed">
<h2>Embed a calculator</h2>
<p>The calculators can also live on your own site: a free, copy and paste frame that calculates in the reader's browser. See <a href="/calcs/embed/">Embed a calculator</a>.</p>
</section>
</main>
${footer()}
</body>
</html>
`;
}
