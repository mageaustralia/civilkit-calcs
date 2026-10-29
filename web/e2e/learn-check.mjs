// Checks the static landing pages under /calcs/learn/ in a build (no browser needed).
//   node e2e/learn-check.mjs [--dist DIR]        (after: cd web && CALCS_BASE=/calcs/ npm run build)
//   - each landing page is in the build, has one h1, a title of 15 to 60 and a description of 70 to 159 characters,
//     the right canonical URL, Open Graph and Twitter tags, JSON-LD that parses (breadcrumb, FAQ matching the visible
//     questions, software application where it belongs), 500 to 1100 words of content and no script besides JSON-LD;
//   - every /calcs/ link and src resolves: to a file of the build, to a guide chapter or video of the book, or to a sample
//     id the ?sample= link opens;
//   - no design standard, internal path, work address or em-dash anywhere in the pages, the stylesheet, the sitemap;
//   - every quoted number (span.v) equals a fresh run of the engine, the code blocks are the shipped samples, and
//     the page on disk equals what tools/make_learn_pages.mjs would write now;
//   - the sitemap lists every page with absolute URLs, robots.txt points to it, and the service worker's precache
//     leaves learn/ out.
import { existsSync, readFileSync, statSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pages } from '../../tools/learn/pages.mjs';
import { renderPage, runAll, urlOf, SITE } from '../../tools/learn/render.mjs';
import { valueOf, root } from '../../tools/learn/engine.mjs';
import { guidePath } from '../../tools/guide_url.mjs';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(web, 'calcpad-web', 'dist')));
const failures = [];
let checks = 0;
const check = (ok, what) => { checks++; if (!ok) { failures.push(what); console.log(`FAIL ${what}`); } };
const unesc = (s) => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&amp;/g, '&');

// The terms are assembled from fragments so this file itself carries none of them (public-repo hygiene).
const J = (...p) => p.join('');
const FORBIDDEN = [
    [new RegExp(String.raw`\b${J('A', 'S')} ?(${J('41', '00')}|${J('36', '00')}|${J('11', '70')})\b`, 'i'), 'standard number'],
    [new RegExp(String.raw`\b${J('A', 'S')} ?\/ ?${J('N', 'ZS')}\b`, 'i'), 'standard family'],
    [new RegExp(String.raw`\b(${J('AS', 'CE')}|${J('AI', 'SC')}|${J('N', 'ZS')})\b`), 'standards body'],
    [new RegExp(J('euro', 'code'), 'i'), 'design code name'],
    [new RegExp(String.raw`\b${J('A', 'CI')} ?${J('31', '8')}\b`), 'standard number'],
    [new RegExp(String.raw`\b${J('B', 'S')} ?(${J('81', '10')}|${J('59', '50')})\b`), 'standard number'],
    [new RegExp(J('SG', 'Rebuild'), 'i'), 'internal project'],
    [new RegExp(String.raw`\/${J('Us', 'ers')}\/`), 'local path'], [new RegExp(String.raw`\/${J('Vol', 'umes')}\/`), 'local path'],
    [new RegExp(String.raw`\/${J('pri', 'vate')}\/${J('t', 'mp')}`), 'local path'],
    [new RegExp(J('mageaustralia', String.raw`\.com`), 'i'), 'work domain'],
    [/\u2014/, 'em-dash'], [/\u2013/, 'en-dash'],
];

const manifest = JSON.parse(readFileSync(join(web, 'calcpad-web', 'public', 'sheet-samples', 'manifest.json'), 'utf8'));
const sampleIds = new Set(manifest.samples.flatMap((s) => [s.id, s.id.replace(/^template-/, '')]));
const chapters = new Set();
const walk = (d) => readdirSync(d, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)]));
// Guide links use the canonical extensionless form (/calcs/guide/units, /calcs/guide/sheet/); the .html form only redirects.
for (const f of walk(join(root, 'book', 'src'))) if (f.endsWith('.md')) chapters.add(guidePath(f.slice(join(root, 'book', 'src').length + 1)));
const videoNames = new Set(readdirSync(join(root, 'book', 'videos')).filter((f) => /^\d\d-.*\.md$/.test(f)).map((f) => f.replace(/\.md$/, '.mp4')));

/** Does an internal URL path (as written in a page) resolve? */
function resolves(href) {
    const [pathAndQuery] = href.split('#');
    const [path, query] = pathAndQuery.split('?');
    if (!path.startsWith('/calcs/')) return `not under /calcs/: ${href}`;
    const rel = path.slice('/calcs/'.length);
    if (rel === '' && query) {
        const m = /^sample=(.+)$/.exec(query);
        return m && sampleIds.has(decodeURIComponent(m[1])) ? null : `no sample for ${href}`;
    }
    if (rel.startsWith('guide/')) {
        const g = rel.slice('guide/'.length);
        if (g === '') return null;
        if (g.startsWith('videos/')) return videoNames.has(g.slice(7)) ? null : `no video ${href}`;
        return chapters.has(g) ? null : `no guide chapter at the canonical address ${href}`;
    }
    const file = join(dist, rel.endsWith('/') || rel === '' ? rel + 'index.html' : rel);
    return existsSync(file) && statSync(file).isFile() ? null : `not in the build: ${href}`;
}

let words = [];
let spans = 0;
let links = 0;
let renders = 0;
for (const page of pages) {
    const file = join(dist, 'learn', page.slug, 'index.html');
    const where = page.slug;
    if (!existsSync(file)) { check(false, `${where}: missing from the build (${file})`); continue; }
    const html = readFileSync(file, 'utf8');

    const h1s = [...html.matchAll(/<h1[\s>]/g)].length;
    check(h1s === 1, `${where}: exactly one h1 (found ${h1s})`);
    const title = unesc(/<title>([\s\S]*?)<\/title>/.exec(html)?.[1] ?? '');
    check(title.length >= 15 && title.length <= 60, `${where}: title length ${title.length} is 15 to 60`);
    const desc = unesc(/<meta name="description" content="([^"]*)"/.exec(html)?.[1] ?? '');
    check(desc.length >= 70 && desc.length <= 159, `${where}: description length ${desc.length} is 70 to 159`);
    check(new RegExp(`<link rel="canonical" href="${urlOf(page.slug).replace(/[.]/g, '\\.')}">`).test(html), `${where}: canonical is ${urlOf(page.slug)}`);
    for (const t of ['og:title', 'og:description', 'og:url', 'og:type', 'og:image']) check(new RegExp(`<meta property="${t}" content="[^"]+"`).test(html), `${where}: ${t}`);
    for (const t of ['twitter:card', 'twitter:title', 'twitter:description']) check(new RegExp(`<meta name="${t}" content="[^"]+"`).test(html), `${where}: ${t}`);
    check(/<meta property="og:url" content="([^"]+)"/.exec(html)?.[1] === urlOf(page.slug), `${where}: og:url equals the canonical`);
    check(/<meta name="viewport" content="width=device-width/.test(html) && /<html lang="en">/.test(html), `${where}: viewport and lang`);

    // JSON-LD
    const blocks = [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)];
    check(blocks.length === 1, `${where}: one JSON-LD block (found ${blocks.length})`);
    check([...html.matchAll(/<script\b/g)].length === blocks.length, `${where}: no script other than JSON-LD`);
    let graph = [];
    try { graph = JSON.parse(blocks[0][1])['@graph']; } catch (e) { check(false, `${where}: JSON-LD parses (${e.message})`); }
    const types = graph.map((g) => g['@type']);
    check(types.includes('BreadcrumbList') && types.includes('FAQPage'), `${where}: JSON-LD has BreadcrumbList and FAQPage (${types.join(', ')})`);
    check(types.includes('SoftwareApplication') === !!page.schema?.includes('software'), `${where}: SoftwareApplication only where intended`);
    check(types.includes('HowTo') === !!page.steps, `${where}: HowTo where steps are given`);
    const visibleQs = [...html.matchAll(/<h3>([\s\S]*?)<\/h3>/g)].map((m) => unesc(m[1]));
    const ldQs = (graph.find((g) => g['@type'] === 'FAQPage')?.mainEntity ?? []).map((q) => q.name);
    check(visibleQs.length >= 4 && JSON.stringify(visibleQs) === JSON.stringify(ldQs), `${where}: FAQ JSON-LD equals the ${visibleQs.length} visible questions`);

    // Words of content (code blocks left out)
    const main = /<main[\s\S]*<\/main>/.exec(html)?.[0] ?? '';
    const w = main.replace(/<pre>[\s\S]*?<\/pre>/g, ' ').replace(/<[^>]+>/g, ' ').split(/\s+/).filter(Boolean).length;
    words.push([page.slug, w]);
    check(w >= 500 && w <= 1100, `${where}: ${w} words of content is 500 to 1100`);

    // Links and ids
    const ids = new Set([...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
    for (const m of html.matchAll(/\s(?:href|src)="([^"]+)"/g)) {
        const u = unesc(m[1]);
        links++;
        if (u.startsWith('#')) check(ids.has(u.slice(1)), `${where}: anchor ${u} exists`);
        else if (u.startsWith('/')) { const r = resolves(u); check(!r, `${where}: ${r ?? u}`); }
        else check(/^https:\/\//.test(u), `${where}: link ${u} is absolute https or internal`);
    }
    const internal = page.related.every((s) => html.includes(`href="/calcs/learn/${s}/"`));
    check(internal && html.includes('href="/calcs/"') && html.includes('href="/calcs/guide/"'), `${where}: links to its related pages, the app and the guide`);
    if (page.open) check(html.includes(`href="/calcs/?sample=${page.open}"`), `${where}: Open this template deep link`);
    if (page.video) check(html.includes(`<video controls preload="none"`) && html.includes(`/calcs/guide/videos/${page.video.file}`), `${where}: video element`);

    // Forbidden terms
    for (const [re, name] of FORBIDDEN) check(!re.test(html), `${where}: no ${name}`);

    // Numbers equal a fresh run; the page equals a fresh render
    const runs = runAll(page);
    const found = [...html.matchAll(/<span class="v" data-run="(\w+)" data-key="([^"]+)">([^<]*)<\/span>/g)];
    spans += found.length;
    if (['simply-supported-beam-calculator', 'retaining-wall-stability-calculator', 'pad-footing-bearing-pressure-calculator', 'pipe-flow-darcy-weisbach-calculator', 'pump-operating-point-calculator'].includes(page.slug)) {
        check(found.length >= 12, `${where}: quotes at least 12 engine values (${found.length})`);
    }
    for (const [, id, key, val] of found) check(runs[id] && unesc(val) === valueOf(runs[id], unesc(key)), `${where}: ${id}:${key} = "${unesc(val)}" equals the engine (${runs[id] ? valueOf(runs[id], unesc(key)) : 'no run'})`);
    renders++;
    check(renderPage(page, runs) === html, `${where}: equals what tools/make_learn_pages.mjs writes now`);
}

// Stylesheet, sitemap, robots, service worker
const cssFile = join(dist, 'learn', 'learn.css');
check(existsSync(cssFile), 'learn.css is in the build');
if (existsSync(cssFile)) for (const [re, name] of FORBIDDEN) check(!re.test(readFileSync(cssFile, 'utf8')), `learn.css: no ${name}`);
const sm = existsSync(join(dist, 'sitemap.xml')) ? readFileSync(join(dist, 'sitemap.xml'), 'utf8') : '';
const locs = [...sm.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
check(locs.length > 10 && locs.every((l) => l.startsWith(`${SITE}/calcs/`)), `sitemap has ${locs.length} absolute URLs under ${SITE}/calcs/`);
for (const p of pages) check(locs.includes(urlOf(p.slug)), `sitemap lists ${p.slug}`);
check(new Set(locs).size === locs.length, 'sitemap has no duplicates');
for (const [re, name] of FORBIDDEN) check(!re.test(sm), `sitemap: no ${name}`);
const robots = existsSync(join(dist, 'robots.txt')) ? readFileSync(join(dist, 'robots.txt'), 'utf8') : '';
check(/^User-agent: \*$/m.test(robots) && /^Allow: \/$/m.test(robots) && robots.includes(`Sitemap: ${SITE}/calcs/sitemap.xml`), 'robots.txt allows all and names the sitemap');
const swFile = join(dist, 'sw.js');
if (existsSync(swFile)) {
    const files = JSON.parse(/const FILES = (\[.*\]);/.exec(readFileSync(swFile, 'utf8'))[1]);
    check(!files.some((f) => f.startsWith('learn/') || f === 'sitemap.xml' || f === 'robots.txt'), `service worker precache (${files.length} files) leaves out learn/, sitemap.xml and robots.txt`);
    check(files.includes('./') && files.some((f) => f.startsWith('assets/')), 'service worker precache still holds the app');
}

console.log(`\n${checks} checks over ${pages.length} pages: ${words.map(([s, n]) => `${s} ${n}w`).join(', ')}`);
console.log(`${links} links resolved, ${spans} quoted engine values compared, ${renders} pages re-rendered`);
if (failures.length) { console.log(`${failures.length} failed`); process.exit(1); }
console.log('all passed');
