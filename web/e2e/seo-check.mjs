// The search-engine basics of every page Calcs publishes, from the generated files (no browser, no network):
//   node e2e/seo-check.mjs [--book DIR]      DIR is the built guide (default target/book; run book/build.sh first)
// - the embed catalogue (public/embed): the same head checks, breadcrumb CivilKit > Calcs > Embed, a snippet per sample;
// - landing pages and their hub (public/learn): title 15 to 60 and description 70 to 160 characters, both unique; one h1;
//   the canonical URL and og:url; JSON-LD that parses; a BreadcrumbList of CivilKit > Calcs (the product page) >
//   Calculators and guides > the page, and the same trail on the page; "CivilKit Calcs" links to the product page and the
//   app (/calcs/) only behind "Open" actions; each page links 3 to 5 sibling pages and the hub; the hub lists every page;
// - the guide (target/book): one h1 per page, one canonical at the extensionless address (the introduction and
//   index.html at /calcs/guide/), a description of its own of 70 to 160 characters, Open Graph tags, breadcrumb JSON-LD;
// - the app shell (index.html): description, canonical /calcs/, Open Graph and Twitter tags with absolute images,
//   SoftwareApplication JSON-LD, a noscript block with real links;
// - sitemap.xml and llms.txt: every URL is in its canonical form (no guide .html, folders with a slash) and resolves;
// - links: every internal link of the landing pages, the hub, What's new and the guide resolves to a file or a sample.
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, resolve, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import { guideChapters, guideUrl, guidePath, descriptionOf, GUIDE } from '../../tools/guide_url.mjs';
import { guidePages, finish } from '../../tools/guide_seo.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '..', '..');
const pub = join(repo, 'web', 'calcpad-web', 'public');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const book = resolve(arg('--book', join(repo, 'target', 'book')));
const SITE = 'https://civilkit.io';
const PRODUCT = `${SITE}/calc/`;
const HUB = `${SITE}/calcs/learn/`;
const APP = `${SITE}/calcs/`;

const fails = [];
let checks = 0;
const check = (ok, what) => { checks++; if (!ok) { fails.push(what); console.log(`FAIL ${what}`); } };
const unesc = (s) => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, '&');
const read = (p) => readFileSync(p, 'utf8');
const meta = (html, attr, name) => [...html.matchAll(new RegExp(`<meta ${attr}="${name}" content="([^"]*)"`, 'g'))].map((m) => unesc(m[1]));
const canonicals = (html) => [...html.matchAll(/<link rel="canonical" href="([^"]+)"/g)].map((m) => m[1]);
const h1s = (html) => html.match(/<h1[\s>]/g)?.length ?? 0;
const ldOf = (html, where) => [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)].map((m) => {
    try { return JSON.parse(m[1]); } catch (e) { check(false, `${where}: JSON-LD parses (${e.message})`); return {}; }
});
const nodes = (ld) => ld.flatMap((x) => x['@graph'] ?? [x]);
const titles = new Map();
const descriptions = new Map();
const unique = (map, value, where, what) => { check(!map.has(value), `${where}: ${what} is unique (also on ${map.get(value)})`); map.set(value, where); };

// ---- what an address resolves to -------------------------------------------------------------------------------------
const manifest = JSON.parse(read(join(pub, 'sheet-samples', 'manifest.json')));
const sampleIds = new Set(manifest.samples.flatMap((s) => [s.id, s.id.replace(/^template-/, '')]));
const chapters = guideChapters();
const guideCanon = new Set(chapters.map((c) => guidePath(c.file)));
const videoNames = new Set(readdirSync(join(repo, 'book', 'videos')).filter((f) => /^\d\d-.*\.md$/.test(f)).flatMap((f) => [f.replace(/\.md$/, '.mp4'), f.replace(/\.md$/, '.jpg')]));
const isFile = (p) => existsSync(p) && statSync(p).isFile();
/** null if a site path under /calcs/ (with an optional query) resolves, else the reason. canonicalOnly: guide links must be extensionless. */
function resolvesPath(path, query, { canonicalOnly = true } = {}) {
    // The MCP endpoint is a Worker of its own at the root of the host, not a file of this build.
    if (path === '/mcp') return null;
    if (!path.startsWith('/calcs/')) return `not under /calcs/: ${path}`;
    const rel = path.slice('/calcs/'.length);
    if (rel === '') {
        if (!query) return null;
        const m = /^sample=(.+)$/.exec(query);
        if (m) return sampleIds.has(decodeURIComponent(m[1])) ? null : `no sample for ?${query}`;
        // An embed (docs/embedding.md): embed=1 or edit, a known sample and only the documented options.
        const q = new URLSearchParams(query);
        const known = ['embed', 'sample', 'theme', 'mode', 'hide', 'allowpython'];
        return /^(1|edit)$/.test(q.get('embed') ?? '') && sampleIds.has(q.get('sample') ?? '') && [...q.keys()].every((k) => known.includes(k)) ? null : `not an app or embed address: ?${query}`;
    }
    if (rel.startsWith('guide/')) {
        const g = rel.slice('guide/'.length);
        if (g.startsWith('videos/')) return videoNames.has(g.slice(7)) ? null : `no video ${path}`;
        if (guideCanon.has(g)) return null;
        if (!canonicalOnly && (isFile(join(book, g)) || isFile(join(book, g, 'index.html')))) return null;
        return `not a guide chapter at its canonical address: ${path}`;
    }
    if (rel.startsWith('whats-new/clips/')) return null; // the clips are copied in at deploy time (docs/DEPLOY.md), not committed
    const f = join(pub, rel.endsWith('/') ? rel + 'index.html' : rel);
    return isFile(f) ? null : `not in public/: ${path}`;
}
/** Checks every href and src of a page. `base` is the page's own site path (for relative links). */
function checkLinks(html, where, base, opts) {
    const ids = new Set([...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));
    let n = 0;
    for (const m of html.matchAll(/\s(?:href|src)="([^"]+)"/g)) {
        const raw = unesc(m[1]);
        if (/^(mailto:|data:|javascript:)/.test(raw)) continue;
        if (raw.startsWith('#')) { check(ids.has(raw.slice(1)) || raw === '#', `${where}: anchor ${raw} exists`); n++; continue; }
        let u;
        try { u = new URL(raw, `${SITE}${base}`); } catch { check(false, `${where}: link ${raw} parses`); continue; }
        if (u.origin !== SITE) { check(u.protocol === 'https:', `${where}: external link ${raw} is https`); continue; }
        n++;
        if (u.pathname === '/' || u.pathname === '/calc/') continue; // the suite home and the product page (another site)
        const r = resolvesPath(decodeURIComponent(u.pathname), u.search.slice(1), opts);
        check(!r, `${where}: ${raw}: ${r}`);
    }
    return n;
}

// ---- landing pages and the hub ---------------------------------------------------------------------------------------------
const { pages } = await import('../../tools/learn/pages.mjs');
const slugs = new Set(pages.map((p) => p.slug));
const learnPages = [...pages.map((p) => ({ file: join(pub, 'learn', p.slug, 'index.html'), url: `${HUB}${p.slug}/`, page: p })), { file: join(pub, 'learn', 'index.html'), url: HUB, page: null }];
let links = 0;
for (const { file, url, page } of learnPages) {
    const where = page ? page.slug : 'learn hub';
    if (!isFile(file)) { check(false, `${where}: ${file} exists (run node tools/make_learn_pages.mjs)`); continue; }
    const html = read(file);
    const title = unesc(/<title>([^<]*)<\/title>/.exec(html)?.[1] ?? '');
    const desc = meta(html, 'name', 'description');
    check(title.length >= 15 && title.length <= 60, `${where}: title of ${title.length} characters is 15 to 60`);
    check(desc.length === 1 && desc[0].length >= 70 && desc[0].length <= 160, `${where}: one description of 70 to 160 characters (${desc[0]?.length})`);
    unique(titles, title, where, 'title');
    unique(descriptions, desc[0], where, 'description');
    check(h1s(html) === 1, `${where}: one h1`);
    check(JSON.stringify(canonicals(html)) === JSON.stringify([url]), `${where}: one canonical, ${url}`);
    check(meta(html, 'property', 'og:url')[0] === url && !!meta(html, 'property', 'og:title')[0] && /^https:\/\//.test(meta(html, 'property', 'og:image')[0] ?? ''), `${where}: og:url, og:title and an absolute og:image`);
    const ld = nodes(ldOf(html, where));
    const bc = ld.find((n) => n['@type'] === 'BreadcrumbList')?.itemListElement ?? [];
    const want = page ? [['CivilKit', `${SITE}/`], ['Calcs', PRODUCT], ['Calculators and guides', HUB], [page.label, url]] : [['CivilKit', `${SITE}/`], ['Calcs', PRODUCT], ['Calculators and guides', HUB]];
    check(JSON.stringify(bc.map((i) => [i.name, i.item])) === JSON.stringify(want) && bc.every((i, k) => i.position === k + 1), `${where}: BreadcrumbList is ${want.map((w) => w[0]).join(' > ')}`);
    const crumb = /<nav class="crumbs"[^>]*>([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? '';
    check(crumb.includes(`<a href="${SITE}/">CivilKit</a>`) && crumb.includes(`<a href="${PRODUCT}">Calcs</a>`) && (page ? crumb.includes('<a href="/calcs/learn/">Calculators and guides</a>') : true), `${where}: the visible breadcrumb links CivilKit, Calcs (the product page) and the hub`);
    check([...html.matchAll(/<a [^>]*href="([^"]+)"[^>]*>CivilKit Calcs<\/a>/g)].every((m) => m[1] === PRODUCT), `${where}: every "CivilKit Calcs" link goes to the product page`);
    const toApp = [...html.matchAll(/<a [^>]*href="\/calcs\/(\?[^"]*)?"[^>]*>([^<]*)<\/a>/g)].map((m) => m[2]);
    check(toApp.length > 0 && toApp.every((t) => /^Open\b/.test(t)), `${where}: links to the app are "Open" actions (${[...new Set(toApp)].join(', ')})`);
    for (const n of ld.filter((x) => x['@type'] === 'SoftwareApplication')) check(n.url === PRODUCT, `${where}: the SoftwareApplication url is the product page`);
    if (page) {
        check(page.related.length >= 3 && page.related.length <= 5 && page.related.every((s) => slugs.has(s) && s !== page.slug), `${where}: 3 to 5 related sibling pages (${page.related.length})`);
        check(html.includes('href="/calcs/learn/"'), `${where}: links to the hub`);
        if (page.open) check(html.includes(`href="/calcs/?sample=${page.open}"`), `${where}: a deep link that opens its sheet`);
    } else {
        for (const p of pages) check(html.includes(`<a href="/calcs/learn/${p.slug}/">`), `hub lists ${p.slug}`);
        const list = ld.find((n) => n['@type'] === 'CollectionPage')?.mainEntity?.itemListElement ?? [];
        check(list.length === pages.length && list.every((i) => slugs.has(i.url.slice(HUB.length, -1))), `hub: the ItemList has the ${pages.length} pages`);
    }
    links += checkLinks(html, where, `/calcs/learn/${page ? page.slug + '/' : ''}`);
}

// ---- the embed catalogue (tools/learn/embed.mjs) ----------------------------------------------------------------------------
{
    const where = 'embed page';
    const url = `${SITE}/calcs/embed/`;
    const file = join(pub, 'embed', 'index.html');
    if (!isFile(file)) check(false, `${where}: ${file} exists (run node tools/make_learn_pages.mjs)`);
    else {
        const html = read(file);
        const title = unesc(/<title>([^<]*)<\/title>/.exec(html)?.[1] ?? '');
        const desc = meta(html, 'name', 'description');
        check(title.length >= 15 && title.length <= 60, `${where}: title of ${title.length} characters is 15 to 60`);
        check(desc.length === 1 && desc[0].length >= 70 && desc[0].length <= 160, `${where}: one description of 70 to 160 characters (${desc[0]?.length})`);
        unique(titles, title, where, 'title');
        unique(descriptions, desc[0], where, 'description');
        check(h1s(html) === 1, `${where}: one h1`);
        check(JSON.stringify(canonicals(html)) === JSON.stringify([url]), `${where}: one canonical, ${url}`);
        check(meta(html, 'property', 'og:url')[0] === url && !!meta(html, 'property', 'og:title')[0] && /^https:\/\//.test(meta(html, 'property', 'og:image')[0] ?? ''), `${where}: og:url, og:title and an absolute og:image`);
        const bc = nodes(ldOf(html, where)).find((n) => n['@type'] === 'BreadcrumbList')?.itemListElement ?? [];
        const want = [['CivilKit', `${SITE}/`], ['Calcs', PRODUCT], ['Embed', url]];
        check(JSON.stringify(bc.map((i) => [i.name, i.item])) === JSON.stringify(want), `${where}: BreadcrumbList is CivilKit > Calcs > Embed`);
        const crumb = /<nav class="crumbs"[^>]*>([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? '';
        check(crumb.includes(`<a href="${SITE}/">CivilKit</a>`) && crumb.includes(`<a href="${PRODUCT}">Calcs</a>`) && crumb.includes('<span>Embed</span>'), `${where}: the visible breadcrumb CivilKit / Calcs / Embed`);
        check(html.includes('href="/calcs/learn/"') && read(join(pub, 'learn', 'index.html')).includes('href="/calcs/embed/"'), `${where}: links the hub, and the hub links it`);
        for (const p of pages.filter((x) => x.open)) check(html.includes(`?embed=1&amp;sample=${p.open}`), `${where}: a snippet for ${p.open}`);
        links += checkLinks(html, where, '/calcs/embed/');
        // The snippets are text: every embed address in them resolves too.
        for (const m of html.matchAll(/https:\/\/civilkit\.io(\/calcs\/[^\s"&]*)\?([^\s"]*?)&quot;/g)) {
            const r = resolvesPath(m[1], unesc(m[2]));
            check(!r, `${where}: snippet ${m[1]}?${unesc(m[2])}: ${r}`);
        }
    }
}

// ---- the quick reference (tools/learn/reference.mjs) ----------------------------------------------------------------------
{
    const where = 'reference page';
    const url = `${SITE}/calcs/reference/`;
    const file = join(pub, 'reference', 'index.html');
    if (!isFile(file)) check(false, `${where}: ${file} exists (run node tools/make_learn_pages.mjs)`);
    else {
        const html = read(file);
        const title = unesc(/<title>([^<]*)<\/title>/.exec(html)?.[1] ?? '');
        const desc = meta(html, 'name', 'description');
        check(title.length >= 15 && title.length <= 60, `${where}: title of ${title.length} characters is 15 to 60`);
        check(desc.length === 1 && desc[0].length >= 70 && desc[0].length <= 160, `${where}: one description of 70 to 160 characters (${desc[0]?.length})`);
        unique(titles, title, where, 'title');
        unique(descriptions, desc[0], where, 'description');
        check(h1s(html) === 1, `${where}: one h1`);
        check(JSON.stringify(canonicals(html)) === JSON.stringify([url]), `${where}: one canonical, ${url}`);
        check(meta(html, 'property', 'og:url')[0] === url && !!meta(html, 'property', 'og:title')[0] && /^https:\/\//.test(meta(html, 'property', 'og:image')[0] ?? ''), `${where}: og:url, og:title and an absolute og:image`);
        const bc = nodes(ldOf(html, where)).find((n) => n['@type'] === 'BreadcrumbList')?.itemListElement ?? [];
        const want = [['CivilKit', `${SITE}/`], ['Calcs', PRODUCT], ['Quick reference', url]];
        check(JSON.stringify(bc.map((i) => [i.name, i.item])) === JSON.stringify(want), `${where}: BreadcrumbList is CivilKit > Calcs > Quick reference`);
        check(html.includes('href="/calcs/guide/"') && read(join(pub, 'learn', 'index.html')).includes('href="/calcs/reference/"'), `${where}: links the guide, and the hub links it`);
        links += checkLinks(html, where, '/calcs/reference/');
    }
}

// ---- What's new ----------------------------------------------------------------------------------------------------------------------------
{
    const html = read(join(pub, 'whats-new', 'index.html'));
    unique(titles, unesc(/<title>([^<]*)<\/title>/.exec(html)?.[1] ?? ''), "what's new", 'title');
    unique(descriptions, meta(html, 'name', 'description')[0], "what's new", 'description');
    check(/<a class="brand" href="https:\/\/civilkit\.io\/calc\/">CivilKit Calcs<\/a>/.test(html), "what's new: the brand links to the product page");
    check(html.includes('href="../learn/"'), "what's new: links to the hub");
    links += checkLinks(html, "what's new", '/calcs/whats-new/');
}

// ---- the app shell ------------------------------------------------------------------------------------------------------------------------
{
    const html = read(join(repo, 'web', 'calcpad-web', 'index.html'));
    const where = 'app shell';
    const title = unesc(/<title>([^<]*)<\/title>/.exec(html)?.[1] ?? '');
    const desc = meta(html, 'name', 'description')[0] ?? '';
    check(title.length >= 15 && title.length <= 60, `${where}: title of ${title.length} characters`);
    check(desc.length >= 70 && desc.length <= 160, `${where}: description of ${desc.length} characters`);
    unique(titles, title, where, 'title');
    unique(descriptions, desc, where, 'description');
    check(JSON.stringify(canonicals(html)) === JSON.stringify([APP]), `${where}: canonical is ${APP} (covers every ?sample= link)`);
    for (const p of ['og:title', 'og:description', 'og:url', 'og:type', 'og:image']) check(!!meta(html, 'property', p)[0], `${where}: ${p}`);
    for (const p of ['twitter:card', 'twitter:image']) check(!!meta(html, 'name', p)[0], `${where}: ${p}`);
    check([...meta(html, 'property', 'og:image'), ...meta(html, 'name', 'twitter:image')].every((u) => u.startsWith(`${SITE}/`)), `${where}: images are absolute`);
    const app = nodes(ldOf(html, where)).find((n) => n['@type'] === 'SoftwareApplication');
    check(!!app && app.url === APP && app.operatingSystem === 'Any' && app.applicationCategory === 'EngineeringApplication' && app.offers?.price === '0' && /mit/.test(app.license ?? '') && !!app.softwareVersion && /^https:\/\//.test(app.screenshot ?? ''), `${where}: SoftwareApplication JSON-LD (free, MIT, any OS, version, screenshot)`);
    const ns = /<noscript>([\s\S]*?)<\/noscript>/.exec(html)?.[1] ?? '';
    check([`${SITE}/calcs/guide/`, HUB, `${SITE}/calcs/whats-new/`, PRODUCT].every((u) => ns.includes(`href="${u}"`)), `${where}: the noscript block links the guide, the hub, What's new and the product page`);
    links += checkLinks(ns, where, '/calcs/');
    const vite = read(join(repo, 'web', 'calcpad-web', 'vite.config.ts'));
    check(/replaceAll\('__APP_VERSION__'/.test(vite), `${where}: the build fills every __APP_VERSION__ (meta and JSON-LD)`);
}

// ---- the guide ---------------------------------------------------------------------------------------------------------------------------------
if (!isFile(join(book, 'index.html'))) check(false, `the built guide is in ${book} (run book/build.sh, or pass --book)`);
else {
    const guideDesc = new Map();
    for (const [file, md, title] of guidePages()) {
        const where = `guide/${file}`;
        const p = join(book, file);
        if (!isFile(p)) { check(false, `${where} exists`); continue; }
        const html = read(p);
        check(finish(html, md, title) === html, `${where}: finished by tools/guide_seo.mjs (book/build.sh)`);
        check(h1s(html) === 1, `${where}: one h1 (${h1s(html)})`);
        const url = guideUrl(md);
        check(JSON.stringify(canonicals(html)) === JSON.stringify([url]), `${where}: one canonical, ${url}`);
        check(!url.endsWith('.html') && (url === GUIDE || !/\/index$/.test(url)), `${where}: the canonical is extensionless`);
        const desc = meta(html, 'name', 'description');
        check(desc.length === 1 && desc[0].length >= 70 && desc[0].length <= 160, `${where}: one description of 70 to 160 characters (${desc[0]?.length})`);
        check(desc[0] === descriptionOf(read(join(repo, 'book', 'src', md))), `${where}: the description is the chapter's own`);
        if (file !== 'introduction.html') { unique(guideDesc, desc[0], where, 'description'); unique(descriptions, desc[0], where, 'description'); }
        check(meta(html, 'property', 'og:url')[0] === url && meta(html, 'property', 'og:type')[0] === 'article' && /^https:\/\//.test(meta(html, 'property', 'og:image')[0] ?? '') && !!meta(html, 'property', 'og:title')[0] && meta(html, 'property', 'og:description')[0] === desc[0], `${where}: Open Graph title, description, url, type article and an absolute image`);
        const bc = nodes(ldOf(html, where)).find((n) => n['@type'] === 'BreadcrumbList')?.itemListElement ?? [];
        check(bc[0]?.item === `${SITE}/` && bc[1]?.item === PRODUCT && bc[2]?.item === GUIDE && bc[bc.length - 1]?.item === url, `${where}: BreadcrumbList CivilKit > Calcs > User guide${url === GUIDE ? '' : ' > chapter'}`);
        check(/<p class="menu-title">/.test(html) && html.includes(`href="${PRODUCT}"`) && html.includes(`href="${HUB}"`) && html.includes(`href="${APP}"`), `${where}: site bar links the product page, the hub and the app`);
        const rel = file.includes('/') ? `/calcs/guide/${file.slice(0, file.lastIndexOf('/') + 1)}` : '/calcs/guide/';
        // Inside the guide, mdBook's own relative .html links are fine (the host redirects them); links out must resolve.
        links += checkLinks(html.replace(/<script[\s\S]*?<\/script>/g, ''), where, rel, { canonicalOnly: false });
    }
    const intro = read(join(repo, 'book', 'src', 'introduction.md'));
    check(/Try it\./.test(intro) && intro.includes(HUB) && intro.includes(APP) && intro.includes(PRODUCT), 'guide introduction: a "Try it" callout with the app, the hub and the product page');
}

// ---- sitemap.xml and llms.txt -------------------------------------------------------------------------------------------------------------------------
const toPath = (u) => { const x = new URL(u); return [decodeURIComponent(x.pathname), x.search.slice(1)]; };
{
    const sm = read(join(pub, 'sitemap.xml'));
    const locs = [...sm.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
    check(locs.length === new Set(locs).size, 'sitemap: no duplicate URLs');
    check(!locs.some((u) => u.endsWith('.html')), 'sitemap: no .html URL');
    for (const u of locs) {
        check(u.startsWith(APP), `sitemap: ${u} is under ${APP}`);
        const [p, q] = toPath(u);
        check(!q, `sitemap: ${u} has no query`);
        const r = resolvesPath(p, q);
        check(!r, `sitemap: ${u}: ${r}`);
        if (p.startsWith('/calcs/learn/') || p === '/calcs/whats-new' ) check(p.endsWith('/'), `sitemap: ${u} ends with a slash`);
    }
    for (const c of chapters) check(c.file === 'introduction.md' ? locs.includes(GUIDE) : locs.includes(guideUrl(c.file)), `sitemap lists guide ${c.file} at ${guideUrl(c.file)}`);
    for (const x of [HUB, APP, `${SITE}/calcs/embed/`, `${SITE}/calcs/reference/`, `${SITE}/calcs/whats-new/`, ...pages.map((p) => `${HUB}${p.slug}/`)]) check(locs.includes(x), `sitemap lists ${x}`);
}
for (const f of ['llms.txt', 'llms-full.txt']) {
    const t = read(join(pub, f));
    const urls = [...t.matchAll(/https:\/\/civilkit\.io\/[^\s)<>"`]*/g)].map((m) => m[0].replace(/[.,:;]+$/, ''));
    check(!urls.some((u) => /\/calcs\/guide\/.*\.html/.test(u)), `${f}: no guide .html URL`);
    for (const u of new Set(urls)) {
        const [p, q] = toPath(u);
        if (p === '/' || p === '/calc/') continue;
        const r = resolvesPath(p, q);
        check(!r, `${f}: ${u}: ${r}`);
    }
    if (f === 'llms.txt') check(t.includes(`(${HUB})`) && t.includes(`(${PRODUCT})`), 'llms.txt links the hub and the product page');
}

console.log(`\n${checks} checks, ${links} internal links followed`);
if (fails.length) { console.log(`${fails.length} FAILED`); process.exit(1); }
console.log('seo check passed');
