// Finishes the built guide (mdbook build writes target/book) for search engines. mdBook gives every page the book's one
// description and no canonical URL, and its templates cannot compute either per page, so this step writes into each
// chapter page:
//   - a meta description of its own (tools/guide_url.mjs descriptionOf: the chapter's first sentence, or its
//     <!-- description: --> comment), with og:description and twitter:description;
//   - a canonical link and og:url at the extensionless address the host serves (/calcs/guide/units, /calcs/guide/sheet/;
//     the introduction and index.html are both /calcs/guide/);
//   - BreadcrumbList JSON-LD: CivilKit > Calcs (the product page) > User guide > the chapter;
//   - the book title in the menu bar as a <p>, not an <h1>, so the chapter's heading is the page's only h1.
// The static Open Graph tags and the site bar come from book/theme (head.hbs, header.hbs). Run it after every build:
//   book/build.sh                              mdbook build, then this
//   node tools/guide_seo.mjs [--dir target/book] [--check]
// --check exits 1 if a page is not finished (for example after a bare mdbook build). Running it twice changes nothing.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { root, guideChapters, guideUrl, descriptionOf, GUIDE } from './guide_url.mjs';

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const START = '<!-- civilkit seo -->';
const END = '<!-- /civilkit seo -->';

/** The pages to finish: [built file, chapter .md, chapter title]. index.html is mdBook's copy of the first chapter. */
export function guidePages() {
    const ch = guideChapters();
    const intro = ch.find((c) => c.file === 'introduction.md');
    return [['index.html', intro.file, intro.title], ...ch.map((c) => [c.file.replace(/\.md$/, '.html'), c.file, c.title])];
}

/** One page, finished. Idempotent: an earlier block is replaced. */
export function finish(html, md, title) {
    const url = guideUrl(md);
    const description = descriptionOf(readFileSync(join(root, 'book', 'src', md), 'utf8'));
    if (!description) throw new Error(`${md}: no description`);
    const crumbs = [['CivilKit', 'https://civilkit.io/'], ['Calcs', 'https://civilkit.io/calc/'], ['User guide', GUIDE]];
    if (url !== GUIDE) crumbs.push([title, url]);
    const ld = {
        '@context': 'https://schema.org',
        '@type': 'BreadcrumbList',
        itemListElement: crumbs.map(([name, item], i) => ({ '@type': 'ListItem', position: i + 1, name, item })),
    };
    const block = [
        START,
        `<link rel="canonical" href="${url}">`,
        `<meta property="og:url" content="${url}">`,
        `<meta property="og:description" content="${esc(description)}">`,
        `<meta name="twitter:description" content="${esc(description)}">`,
        `<script type="application/ld+json">${JSON.stringify(ld).replace(/</g, '\\u003c')}</script>`,
        END,
    ].join('\n        ');
    let out = html.replace(new RegExp(`\\s*${START}[\\s\\S]*?${END}`), '');
    if (!/<meta name="description" content="[^"]*">/.test(out)) throw new Error(`${md}: no meta description to replace`);
    out = out.replace(/<meta name="description" content="[^"]*">/, `<meta name="description" content="${esc(description)}">\n        ${block}`);
    out = out.replace(/<h1 class="menu-title">([\s\S]*?)<\/h1>/, '<p class="menu-title">$1</p>');
    return out;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(root, 'tools', 'guide_seo.mjs')) {
    const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
    const dir = resolve(arg('--dir', join(root, 'target', 'book')));
    const check = process.argv.includes('--check');
    let bad = 0, done = 0;
    for (const [file, md, title] of guidePages()) {
        const p = join(dir, file);
        if (!existsSync(p)) { console.error(`missing: ${p} (run mdbook build in book/ first)`); bad++; continue; }
        const html = readFileSync(p, 'utf8');
        const out = finish(html, md, title);
        if (check) { if (out !== html) { console.error(`not finished: ${file}`); bad++; } continue; }
        if (out !== html) writeFileSync(p, out);
        done++;
    }
    if (check) { console.log(bad ? `${bad} guide pages not finished: run node tools/guide_seo.mjs` : 'guide pages are finished'); process.exit(bad ? 1 : 0); }
    console.log(`finished ${done} guide pages in ${dir}`);
    if (bad) process.exit(1);
}
