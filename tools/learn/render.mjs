// The shared template for the landing pages under public/learn/. Pure functions: a page's data (pages.mjs) and the
// values of its engine runs go in, one HTML string comes out. make_learn_pages.mjs writes the files and
// web/e2e/learn-check.mjs renders again and compares, so a page cannot drift from its data or from the engine.
import { runSheet, runCpd, readSample, valueOf } from './engine.mjs';

export const SITE = 'https://civilkit.io';
export const BASE = '/calcs/learn/';
export const urlOf = (slug) => `${SITE}${BASE}${slug}/`;
export const REPO = 'https://github.com/mageaustralia/civilkit-calcs';
export const CALCPADCE = 'https://github.com/imartincei/CalcpadCE';

export const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const strip = (html) => html.replace(/<[^>]+>/g, '').replace(/&nbsp;/g, ' ').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();

/** The source text of one run of a page: a sample file with some input lines replaced, an inline sheet or Calcpad text. */
export function sourceOf(run) {
    if (run.sample) {
        let t = readSample(run.sample);
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

const NAV = [['/calcs/', 'Open the app'], ['/calcs/guide/', 'User guide'], [`${BASE}engineering-calculation-software-free/`, 'All pages']];
const LABEL = {
    'mathcad-alternative': 'Mathcad alternative',
    'calcpad-online': 'Calcpad online',
    'excel-alternative-for-engineering-calculations': 'Excel alternative',
    'engineering-calculation-software-free': 'Free calculation software',
    'simply-supported-beam-calculator': 'Simply supported beam',
    'retaining-wall-stability-calculator': 'Retaining wall stability',
    'pad-footing-bearing-pressure-calculator': 'Pad footing bearing pressure',
    'pipe-flow-darcy-weisbach-calculator': 'Pipe flow, Darcy-Weisbach',
    'pump-operating-point-calculator': 'Pump operating point',
    'mathcad-worksheet-converter': 'Mathcad worksheet converter',
};
export const labelOf = (slug) => LABEL[slug];

function jsonld(page, runs, faq) {
    const url = urlOf(page.slug);
    const graph = [];
    graph.push({
        '@type': 'BreadcrumbList',
        itemListElement: [
            { '@type': 'ListItem', position: 1, name: 'CivilKit Calcs', item: `${SITE}/calcs/` },
            { '@type': 'ListItem', position: 2, name: LABEL[page.slug], item: url },
        ],
    });
    if (page.schema?.includes('software')) {
        graph.push({
            '@type': 'SoftwareApplication',
            name: 'CivilKit Calcs',
            url: `${SITE}/calcs/`,
            description: 'A free, open source engineering calculation sheet that runs in the browser. Real units, plots, input forms, Word, Excel and PDF export, offline use.',
            applicationCategory: 'BusinessApplication',
            applicationSubCategory: 'Engineering calculation',
            operatingSystem: 'Any operating system with a modern web browser',
            isAccessibleForFree: true,
            license: 'https://opensource.org/license/mit',
            offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
        });
    }
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
    return JSON.stringify({ '@context': 'https://schema.org', '@graph': graph }, null, 1).replace(/</g, '\\u003c');
}

/** One complete page. `runs` is runAll(page). */
export function renderPage(page, runs) {
    const url = urlOf(page.slug);
    const faq = page.faq.map(([q, a]) => [q, fill(a, page, runs)]);
    const video0 = page.video ? `<figure class="video">
<video controls preload="none" width="960" height="540" aria-label="${esc(page.video.title)}" src="/calcs/guide/videos/${page.video.file}"></video>
<figcaption>Video: ${esc(page.video.title)} (about 85 seconds). <a href="/calcs/guide/${page.video.chapter}">Read the chapter</a>.</figcaption>
</figure>` : '';
    const body = page.sections.map(([h, html], i) => `<section>\n<h2>${esc(h)}</h2>\n${fill(html, page, runs)}\n</section>${i === 0 ? '\n' + video0 : ''}`).join('\n');
    const cta = page.open
        ? `<p class="cta"><a class="button" href="/calcs/?sample=${esc(page.open)}">Open this template</a> <a class="button ghost" href="/calcs/">Open a blank sheet</a></p>`
        : `<p class="cta"><a class="button" href="/calcs/">${esc(page.cta ?? 'Open CivilKit Calcs')}</a> <a class="button ghost" href="/calcs/guide/">Read the guide</a></p>`;
    const related = page.related.map((s) => `<li><a href="${BASE}${s}/">${esc(LABEL[s])}</a></li>`).join('\n');
    const nav = NAV.map(([h, t]) => `<a href="${h}">${esc(t)}</a>`).join('\n');
    const faqHtml = faq.map(([q, a]) => `<h3>${esc(q)}</h3>\n<p>${a}</p>`).join('\n');
    const title = page.title;
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(page.description)}">
<meta name="robots" content="index, follow, max-image-preview:large">
<meta name="color-scheme" content="light dark">
<link rel="canonical" href="${url}">
<link rel="stylesheet" href="${BASE}learn.css">
<link rel="icon" href="/calcs/icon-192.png" sizes="192x192">
<meta property="og:type" content="website">
<meta property="og:site_name" content="CivilKit Calcs">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(page.description)}">
<meta property="og:url" content="${url}">
<meta property="og:image" content="${SITE}/calcs/icon-512.png">
<meta name="twitter:card" content="summary">
<meta name="twitter:title" content="${esc(title)}">
<meta name="twitter:description" content="${esc(page.description)}">
<meta name="twitter:image" content="${SITE}/calcs/icon-512.png">
<script type="application/ld+json">
${jsonld(page, runs, faq)}
</script>
</head>
<body>
<a class="skip" href="#main">Skip to the content</a>
<header class="site">
<a class="brand" href="/calcs/">CivilKit Calcs</a>
<nav aria-label="Site">
${nav}
</nav>
</header>
<main id="main">
<nav class="crumbs" aria-label="Breadcrumb"><a href="/calcs/">CivilKit Calcs</a> / <span>${esc(LABEL[page.slug])}</span></nav>
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
<footer class="site">
<p>CivilKit Calcs is free and open source (MIT licence). It runs in your browser: your calculations are not sent to a server. Source: <a href="${REPO}">${REPO.replace('https://', '')}</a>. It is built on <a href="${CALCPADCE}">CalcpadCE</a> by Nedelcho Ganchovski (Proektsoft) and contributors.</p>
<p>Product names on this page belong to their owners. CivilKit Calcs is not affiliated with or endorsed by them.</p>
</footer>
</body>
</html>
`;
}
