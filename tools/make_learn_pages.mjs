// Generates the ten landing pages (web/calcpad-web/public/learn/<slug>/index.html), public/sitemap.xml and public/robots.txt.
//   node tools/make_learn_pages.mjs            write the files
//   node tools/make_learn_pages.mjs --check    exit 1 if a file on disk differs from what would be written
// Every number on a task page comes from a real engine run (tools/learn/engine.mjs: the sheet compiler wasm and the Cli
// binary, as in book/check-examples.mjs). Needs public/wasm (tools/build_wasm.sh) and target/release/Cli.
import { mkdirSync, writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { root } from './learn/engine.mjs';
import { pages } from './learn/pages.mjs';
import { renderPage, runAll, urlOf, SITE } from './learn/render.mjs';

const pub = join(root, 'web', 'calcpad-web', 'public');

/** All the files this script owns, as { path (under public/), content }. */
export function outputs() {
    const files = [];
    for (const page of pages) files.push({ path: `learn/${page.slug}/index.html`, content: renderPage(page, runAll(page)) });
    // The guide chapters, from the book's table of contents (mdBook writes chapter.md as chapter.html).
    const summary = readFileSync(join(root, 'book', 'src', 'SUMMARY.md'), 'utf8');
    const chapters = [...summary.matchAll(/\]\(([^)]+\.md)\)/g)].map((m) => m[1]).filter((f) => f !== 'introduction.md').map((f) => f.replace(/\.md$/, '.html'));
    const urls = [`${SITE}/calcs/`, `${SITE}/calcs/guide/`, ...chapters.map((c) => `${SITE}/calcs/guide/${c}`), ...pages.map((p) => urlOf(p.slug))];
    files.push({
        path: 'sitemap.xml',
        content: `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((u) => `  <url><loc>${u}</loc></url>`).join('\n')}\n</urlset>\n`,
    });
    // Served at /calcs/robots.txt, which crawlers ignore (they read only /robots.txt of the host). It documents the sitemap;
    // the host's root robots.txt is owned elsewhere and should list the same sitemap, which should also be submitted in Search Console.
    files.push({ path: 'robots.txt', content: `User-agent: *\nAllow: /\n\nSitemap: ${SITE}/calcs/sitemap.xml\n` });
    return files;
}

const check = process.argv.includes('--check');
let stale = 0;
for (const f of outputs()) {
    const full = join(pub, f.path);
    if (check) {
        if (!existsSync(full) || readFileSync(full, 'utf8') !== f.content) { console.log(`stale: ${f.path}`); stale++; }
        continue;
    }
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, f.content);
    console.log(`wrote ${f.path}`);
}
if (check) { console.log(stale ? `${stale} stale` : 'generated files are up to date'); process.exit(stale ? 1 : 0); }
