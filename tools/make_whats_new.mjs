#!/usr/bin/env node
// Writes the static "What's new" page, web/calcpad-web/public/whats-new/index.html.
//   node tools/make_whats_new.mjs            write the page
//   node tools/make_whats_new.mjs --check    fail if the file on disk differs from what would be written now
//
// Inputs: CHANGELOG.md (Keep a Changelog; "## [26.10.12] - 2026-10-28" starts a release) and
// web/calcpad-web/whats-new/clips.json (per version: id, title, blurb and recorder of each clip). The page covers the
// every release in the changelog, newest first: one section each, with the clips of that version and then the changelog's bullets as HTML.
// A clip's poster is public/whats-new/posters/<id>.png (committed); the clip files (clips/<id>.webm and .mp4) are
// not committed and the page looks right without them. See docs/DEPLOY.md.
// Clips recorded ahead of a release go under the key "Unreleased": they are left off the page until the release is cut,
// then the key is renamed to the new version (and their posters are committed with it).
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const PAGE_DIR = join(repo, 'web', 'calcpad-web', 'public', 'whats-new');
export const MANIFEST = join(repo, 'web', 'calcpad-web', 'whats-new', 'clips.json');
/** The clips.json key for clips of the next release, kept off the page until it has a changelog version. */
export const UNRELEASED = 'Unreleased';
export const SITE = 'https://civilkit.io/calcs/';
export const CANONICAL = `${SITE}whats-new/`;
export const GITHUB = 'https://github.com/mageaustralia/civilkit-calcs';
export const TITLE = "What's new in CivilKit Calcs: release notes with short clips";
export const DESCRIPTION = 'Short looping clips and release notes for every CivilKit Calcs release: sheets, Python blocks, Plotly charts, symbolic algebra, rich-text editing and more.';

export const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
export const anchor = (version) => `v${version.replace(/\./g, '-')}`;

/** The releases of a Keep a Changelog file, newest first: { version, date, sections: [{ name, bullets: [markdown] }] }. */
export function parseChangelog(text) {
    const releases = [];
    let release = null, section = null, bullet = null;
    const flush = () => { if (bullet != null && section) section.bullets.push(bullet.trim()); bullet = null; };
    for (const line of text.split(/\r?\n/)) {
        let m;
        if ((m = /^## \[([^\]]+)\](?: - (\d{4}-\d{2}-\d{2}))?/.exec(line))) {
            flush();
            section = null;
            release = /^\d+\.\d+\.\d+$/.test(m[1]) ? { version: m[1], date: m[2] ?? '', sections: [] } : null;
            if (release) releases.push(release);
        } else if (/^## /.test(line)) { flush(); release = null; section = null; }
        else if (release && (m = /^### (.+?)\s*$/.exec(line))) { flush(); section = { name: m[1], bullets: [] }; release.sections.push(section); }
        else if (release && section && /^- /.test(line)) { flush(); bullet = line.slice(2); }
        else if (bullet != null && /^\s+\S/.test(line)) bullet += ' ' + line.trim();
        else if (line.trim() === '') flush();
    }
    flush();
    return releases;
}

const CODE = /`([^`]+)`/g;
/** Inline markdown to HTML: code spans, [text](http...) links and **bold**; everything else is escaped text. */
export function inline(md) {
    // Code spans become placeholders first, so **bold** and links may contain them (**Names from `>> #UI`**).
    const codes = [];
    const text = md.replace(CODE, (_, c) => { codes.push(`<code>${esc(c)}</code>`); return `\u0000${codes.length - 1}\u0000`; });
    return esc(text)
        .replace(/\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g, (_, t, u) => `<a href="${u}">${t}</a>`)
        .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
        .replace(/\u0000(\d+)\u0000/g, (_, i) => codes[Number(i)]);
}

/** What a reader sees of an inline markdown string: the same text with the markup removed. */
export function plainText(md) {
    return md.replace(/`([^`]+)`/g, '$1').replace(/\[([^\]]+)\]\((https?:\/\/[^)\s]+)\)/g, '$1').replace(/\*\*([^*]+)\*\*/g, '$1').replace(/\s+/g, ' ').trim();
}

const SECTION_ORDER = ['Added', 'Changed', 'Deprecated', 'Removed', 'Fixed', 'Security'];
const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const longDate = (iso) => { const [y, m, d] = iso.split('-').map(Number); return `${d} ${MONTHS[m - 1]} ${y}`; };

/** Width and height of a PNG, from its header. */
export function pngSize(file) {
    const b = readFileSync(file);
    if (b.length < 24 || b.readUInt32BE(0) !== 0x89504e47) throw new Error(`${file} is not a PNG`);
    return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}

function clipHtml(c, first, pair) {
    const poster = join(PAGE_DIR, 'posters', `${c.id}.png`);
    if (!existsSync(poster)) throw new Error(`clip ${c.id} has no poster: ${poster} (node e2e/clips/record.mjs makes it)`);
    const { w, h } = pngSize(poster);
    const alt = `${c.title}: a still from the clip`;
    return `<figure class="clip${h > w ? ' tall' : ''}" id="clip-${esc(c.id)}">
<h3>${esc(c.title)}</h3>
<span class="frame" tabindex="0"><video muted loop playsinline preload="metadata" poster="posters/${esc(c.id)}.png" width="${w}" height="${h}" aria-label="${esc(c.title)}">
<source src="clips/${esc(c.id)}.webm" type="video/webm">
<source src="clips/${esc(c.id)}.mp4" type="video/mp4">
<img src="posters/${esc(c.id)}.png" width="${w}" height="${h}" alt="${esc(alt)}">
</video><img class="still" src="posters/${esc(c.id)}.png" width="${w}" height="${h}" alt="${esc(alt)}" loading="lazy"></span>
<figcaption>${esc(c.blurb)}</figcaption>
</figure>`;
}

function releaseHtml(r, clips, first) {
    const id = anchor(r.version);
    const pair = clips.length > 1;
    const sections = [...r.sections].sort((a, b) => {
        const ia = SECTION_ORDER.indexOf(a.name), ib = SECTION_ORDER.indexOf(b.name);
        return (ia < 0 ? 99 : ia) - (ib < 0 ? 99 : ib);
    });
    const notes = sections.filter((s) => s.bullets.length).map((s) => `<h4>${esc(s.name)}</h4>
<ul>
${s.bullets.map((b) => `<li>${inline(b)}</li>`).join('\n')}
</ul>`).join('\n');
    return `<section class="release" id="${id}" aria-labelledby="${id}-h">
<header>
<h2 id="${id}-h">${esc(r.version)}</h2>
${r.date ? `<time datetime="${esc(r.date)}">${esc(longDate(r.date))}</time>` : ''}
</header>
${clips.length ? `<div class="clips${pair ? ' pair' : ''}">
${clips.map((c, i) => clipHtml(c, first && i === 0, pair)).join('\n')}
</div>` : ''}
<div class="changes">
<h3>Release notes</h3>
${notes}
<p class="more"><a href="${GITHUB}/releases/tag/v${esc(r.version)}">Release notes on GitHub</a></p>
</div>
</section>`;
}

/** The whole page as a string. `changelog` is the text of CHANGELOG.md, `manifest` the parsed clips.json. */
export function buildPage(changelog, manifest) {
    const releases = parseChangelog(changelog);
    if (!releases.length) throw new Error('CHANGELOG.md has no releases');
    const shown = new Set(releases.map((r) => r.version));
    for (const v of Object.keys(manifest)) if (v !== UNRELEASED && !shown.has(v)) throw new Error(`clips.json has clips for ${v}, which is not a release in CHANGELOG.md (${[...shown].join(', ')})`);
    const newest = releases[0];
    const ogClip = (manifest[newest.version] ?? [])[0];
    const ogImage = ogClip ? `${CANONICAL}posters/${ogClip.id}.png` : `${SITE}icon-512.png`;
    const ld = {
        '@context': 'https://schema.org',
        '@type': 'CollectionPage',
        name: TITLE,
        url: CANONICAL,
        description: DESCRIPTION,
        isPartOf: { '@type': 'WebSite', name: 'CivilKit Calcs', url: SITE },
        mainEntity: {
            '@type': 'ItemList',
            itemListElement: releases.map((r, i) => ({ '@type': 'ListItem', position: i + 1, name: `CivilKit Calcs ${r.version}`, url: `${CANONICAL}#${anchor(r.version)}` })),
        },
    };
    const body = releases.map((r, i) => releaseHtml(r, manifest[r.version] ?? [], i === 0)).join('\n');
    return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(TITLE)}</title>
<meta name="description" content="${esc(DESCRIPTION)}">
<meta name="robots" content="index, follow, max-image-preview:large">
<meta name="color-scheme" content="light dark">
<link rel="canonical" href="${CANONICAL}">
<link rel="stylesheet" href="whats-new.css">
<script src="whats-new.js" defer></script>
<link rel="icon" href="../icon-192.png" sizes="192x192">
<meta property="og:type" content="website">
<meta property="og:site_name" content="CivilKit Calcs">
<meta property="og:title" content="${esc(TITLE)}">
<meta property="og:description" content="${esc(DESCRIPTION)}">
<meta property="og:url" content="${CANONICAL}">
<meta property="og:image" content="${esc(ogImage)}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="${esc(TITLE)}">
<meta name="twitter:description" content="${esc(DESCRIPTION)}">
<meta name="twitter:image" content="${esc(ogImage)}">
<script type="application/ld+json">
${JSON.stringify(ld, null, 1)}
</script>
</head>
<body>
<a class="skip" href="#main">Skip to the releases</a>
<header class="site">
<a class="brand" href="../">CivilKit Calcs</a>
<nav aria-label="Site">
<a href="../">Open the app</a>
<a href="../guide/">User guide</a>
<a href="./" aria-current="page">What's new</a>
</nav>
</header>
<main id="main">
<h1>What's new in CivilKit Calcs</h1>
<p class="lead">A short clip of the headline features of each release, then everything the release changed. The clips are a few seconds long, silent and loop; they were recorded from the app itself.</p>
<div class="cta">
<a class="button" href="../">Open CivilKit Calcs</a>
<a class="button ghost" href="${GITHUB}/blob/main/CHANGELOG.md">Full changelog</a>
</div>
<nav aria-label="Releases">
<ul class="jump">
${releases.map((r) => `<li><a href="#${anchor(r.version)}">${esc(r.version)}</a></li>`).join('\n')}
</ul>
</nav>
${body}
<p class="more">The same notes are in the <a href="${GITHUB}/blob/main/CHANGELOG.md">changelog</a> and on the <a href="${GITHUB}/releases">releases page</a>.</p>
</main>
<footer class="site">
<p>CivilKit Calcs is free and open source (MIT). <a href="${GITHUB}">Source on GitHub</a>.</p>
</footer>
</body>
</html>
`;
}

export function generate() {
    return buildPage(readFileSync(join(repo, 'CHANGELOG.md'), 'utf8'), JSON.parse(readFileSync(MANIFEST, 'utf8')));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
    const out = join(PAGE_DIR, 'index.html');
    const html = generate();
    if (process.argv.includes('--check')) {
        const have = existsSync(out) ? readFileSync(out, 'utf8') : '';
        if (have !== html) { console.error('public/whats-new/index.html is out of date: run node tools/make_whats_new.mjs'); process.exit(1); }
        console.log('whats-new/index.html is up to date');
    } else {
        writeFileSync(out, html);
        console.log(`wrote ${out}`);
    }
}
