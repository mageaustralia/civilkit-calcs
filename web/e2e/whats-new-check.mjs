// The "What's new" page: the generated file, its manifest, posters, changelog coverage, links and hygiene. No browser needed.
//   node e2e/whats-new-check.mjs [--dist DIR]
// (The page in a real browser, with and without the clip files, is checked by e2e/whats-new-browser-check.mjs.)
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { CANONICAL, GITHUB, PAGE_DIR, MANIFEST, RECENT, anchor, generate, parseChangelog, plainText, pngSize } from '../../tools/make_whats_new.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '../..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const dist = resolve(arg('--dist', join(repo, 'web', 'calcpad-web', 'dist')));
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const pagePath = join(PAGE_DIR, 'index.html');
check(existsSync(pagePath), 'public/whats-new/index.html exists');
const html = readFileSync(pagePath, 'utf8');
const manifest = JSON.parse(readFileSync(MANIFEST, 'utf8'));
const changelog = readFileSync(join(repo, 'CHANGELOG.md'), 'utf8');
const releases = parseChangelog(changelog).slice(0, RECENT);

// ---- the page is what the generator writes now ---------------------------------------------------------------
check(html === generate(), 'the page equals what tools/make_whats_new.mjs writes now');

// ---- head ------------------------------------------------------------------------------------------------------
const h1s = html.match(/<h1[ >]/g) ?? [];
check(h1s.length === 1, `one h1 (${h1s.length})`);
const title = /<title>([^<]*)<\/title>/.exec(html)?.[1] ?? '';
const desc = /<meta name="description" content="([^"]*)"/.exec(html)?.[1] ?? '';
check(title.length >= 20 && title.length <= 65, `title length ${title.length} is 20 to 65`);
check(desc.length >= 70 && desc.length <= 160, `description length ${desc.length} is 70 to 160`);
check(new RegExp(`<link rel="canonical" href="${CANONICAL}">`).test(html), `canonical is ${CANONICAL}`);
for (const p of ['og:type', 'og:title', 'og:description', 'og:url', 'og:image']) check(new RegExp(`<meta property="${p}" content="[^"]+"`).test(html), `Open Graph ${p}`);
const ldText = /<script type="application\/ld\+json">\n([^]*?)\n<\/script>/.exec(html)?.[1];
let ld = null;
try { ld = JSON.parse(ldText); } catch { /* reported below */ }
check(ld && ld['@type'] === 'CollectionPage' && ld.url === CANONICAL, 'JSON-LD parses as a CollectionPage with the page URL');
check(ld && ld.mainEntity.itemListElement.length === releases.length && ld.mainEntity.itemListElement.every((e, i) => e.url === `${CANONICAL}#${anchor(releases[i].version)}`), `JSON-LD lists the ${releases.length} releases with their anchors`);
check((html.match(/<script/g) ?? []).length === 2 && html.includes('<script src="whats-new.js" defer></script>'), 'the only scripts are the JSON-LD and the same-folder hover-to-play script');

// ---- the releases shown are the newest six ------------------------------------------------------------------------
check(releases.length === RECENT, `the changelog has ${RECENT} releases to show`);
const sections = [...html.matchAll(/<section class="release" id="([^"]+)"[^>]*>([^]*?)<\/section>/g)];
check(sections.length === releases.length && sections.every((m, i) => m[1] === anchor(releases[i].version)), `one section per release, newest first (${sections.map((m) => m[1]).join(', ')})`);
const ids = [...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]);
check(new Set(ids).size === ids.length, `all ${ids.length} ids are unique`);
const hrefs = [...html.matchAll(/<a [^>]*href="([^"]+)"/g)].map((m) => m[1]);
check(hrefs.filter((h) => h.startsWith('#')).every((h) => ids.includes(h.slice(1))), 'every #link has a target');
const okHref = (h) => h.startsWith('#') || ['../', '../guide/', './'].includes(h) || h.startsWith(`${GITHUB}/`) || h === GITHUB;
check(hrefs.every(okHref), `every link is an anchor, an app path or a project link${hrefs.filter((h) => !okHref(h)).map((h) => ` (${h})`).join('')}`);
for (const r of releases) check(html.includes(`href="${GITHUB}/releases/tag/v${r.version}"`), `${r.version}: a "Release notes on GitHub" link to its tag`);
check(!/https?:\/\/(?!civilkit\.io\/|github\.com\/mageaustralia\/civilkit-calcs|schema\.org|opensource\.org)[^"\s<]+/.test(html.replace(/<script[^]*?<\/script>/, '')), 'no link to any other site');

// ---- books balance: every changelog bullet of the covered releases is on the page, once ----------------------------------------
const decode = (s) => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, '&');
const strip = (s) => decode(s.replace(/<[^>]+>/g, '')).replace(/\s+/g, ' ').trim();
let want = 0, got = 0, balanced = true;
for (const [i, r] of releases.entries()) {
    const body = sections[i]?.[2] ?? '';
    const lis = [...body.matchAll(/<li>([^]*?)<\/li>/g)].map((m) => strip(m[1]));
    const bullets = r.sections.flatMap((s) => s.bullets).map(plainText);
    want += bullets.length; got += lis.length;
    const sortedA = [...lis].sort(), sortedB = [...bullets].sort();
    const same = lis.length === bullets.length && sortedA.every((x, k) => x === sortedB[k]);
    if (!same) { balanced = false; console.log(`     ${r.version}: ${bullets.length} bullets in the changelog, ${lis.length} on the page`); for (const b of bullets) if (!lis.includes(b)) console.log(`       missing: ${b.slice(0, 90)}`); }
    // headings: one h4 per non-empty changelog section
    const names = r.sections.filter((s) => s.bullets.length).map((s) => s.name).sort();
    const heads = [...body.matchAll(/<h4>([^<]+)<\/h4>/g)].map((m) => decode(m[1])).sort();
    if (JSON.stringify(names) !== JSON.stringify(heads)) { balanced = false; console.log(`     ${r.version}: sections ${names} vs ${heads}`); }
}
check(balanced && want === got && want > 0, `every changelog bullet of the ${releases.length} releases is on the page: ${want} bullets, ${got} list items`);
check(releases.every((r) => r.sections.every((s) => s.bullets.every((b) => !/^\s*$/.test(b)))), 'no empty bullet');

// ---- clips: manifest, posters, recorders, markup --------------------------------------------------------------------------------
const clips = Object.entries(manifest).flatMap(([version, list]) => list.map((c) => ({ ...c, version })));
const clipIds = clips.map((c) => c.id);
check(new Set(clipIds).size === clipIds.length, `${clips.length} clips with unique ids`);
check(Object.keys(manifest).every((v) => releases.some((r) => r.version === v)), 'every manifest version is one of the releases shown');
const recorders = readdirSync(join(here, 'clips')).filter((f) => /^\d\d-.*\.mjs$/.test(f));
for (const c of clips) {
    check(/^[a-z0-9]+(-[a-z0-9]+)*$/.test(c.id) && c.title?.length >= 5 && c.blurb?.length >= 40 && c.blurb.length <= 400, `${c.id}: id, title and a blurb of 40 to 400 characters`);
    const poster = join(PAGE_DIR, 'posters', `${c.id}.png`);
    check(existsSync(poster) && statSync(poster).size <= 60 * 1000, `${c.id}: committed poster is a PNG of at most 60 KB (${existsSync(poster) ? (statSync(poster).size / 1000).toFixed(0) + ' KB' : 'missing'})`);
    if (existsSync(poster)) { const { w, h } = pngSize(poster); check(w >= 300 && h >= 300, `${c.id}: poster is ${w} x ${h}`); }
    const rec = recorders.find((f) => f === `${c.recorder}.mjs`);
    check(!!rec, `${c.id}: recorder ${c.recorder}.mjs exists`);
    if (rec) check(new RegExp(`id: '${c.id}'`).test(readFileSync(join(here, 'clips', rec), 'utf8')), `${c.id}: the recorder records this id`);
    const block = new RegExp(`<figure class="clip[^"]*" id="clip-${c.id}">[^]*?</figure>`).exec(html)?.[0] ?? '';
    check(!!block && block.includes(`poster="posters/${c.id}.png"`) && block.includes(`src="clips/${c.id}.webm"`) && block.includes(`src="clips/${c.id}.mp4"`), `${c.id}: on the page with its poster and both sources`);
    check(/<video muted loop playsinline preload="metadata"/.test(block) && !block.includes('autoplay'), `${c.id}: video is muted loop playsinline, preloads metadata and does not autoplay (hover to play)`);
    check(sections.find((m) => m[1] === anchor(c.version))?.[2].includes(`id="clip-${c.id}"`) ?? false, `${c.id}: shown under ${c.version}`);
}
check([...html.matchAll(/<figure class="clip/g)].length === clips.length, 'the page has exactly the manifest clips');
check(recorders.every((f) => clips.some((c) => `${c.recorder}.mjs` === f)), 'every recorder is in the manifest');

// ---- the build -----------------------------------------------------------------------------------------------------------------------------------
if (existsSync(join(dist, 'index.html'))) {
    check(existsSync(join(dist, 'whats-new', 'index.html')), 'the build has whats-new/index.html');
    check(existsSync(join(dist, 'whats-new', 'whats-new.css')), 'the build has whats-new/whats-new.css');
    const sw = existsSync(join(dist, 'sw.js')) ? readFileSync(join(dist, 'sw.js'), 'utf8') : '';
    const files = JSON.parse(/const FILES = (\[.*\]);/.exec(sw)?.[1] ?? '[]');
    check(files.length > 20 && !files.some((f) => f.startsWith('whats-new/')), `the service worker precache (${files.length} files) leaves out whats-new/`);
} else console.log('skip  no build in', dist, '(pass --dist after npm run build)');

// ---- hygiene -----------------------------------------------------------------------------------------------------------------------------------------------
const files = [pagePath, join(PAGE_DIR, 'whats-new.css'), MANIFEST, join(repo, 'tools', 'make_whats_new.mjs'), fileURLToPath(import.meta.url), ...readdirSync(join(here, 'clips')).map((f) => join(here, 'clips', f)), join(here, 'whats-new-browser-check.mjs')].filter(existsSync);
// The words are assembled from pieces so this file does not contain them itself.
const w = (...parts) => parts.join('');
const BAD = [
    [new RegExp('\\u2014'), 'an em-dash'],
    [/\/Users\/|\/Volumes\/|\/private\/tmp|\/home\/[a-z]/, 'a local path'],
    [new RegExp(`${w('space', 'gass')}|${w('sg', 'rebuild')}|${w('fem', '-core')}|${w('civilkit', '-studio')}`, 'i'), 'an internal project name'],
    [new RegExp(`@${w('mageaustralia', '\\.com\\.au')}|${w('mcamp', 'bell')}`, 'i'), 'a work email'],
    [new RegExp(`${w('math', 'cad')}|${w('mat', 'lab')}|${w('mathe', 'matica')}|\\b${w('ma', 'ple')}\\b|${w('s', 'math')}`, 'i'), 'a competitor name'],
    [new RegExp(`\\bAS ?\\d{4}|\\b${w('NZ', 'S')}|\\b${w('AI', 'SC')}|\\b${w('AS', 'CE')}\\b|\\b${w('AC', 'I ?318')}|\\b${w('E', 'N ?199')}\\d|\\bBS ?\\d{4}|${w('[Ee]uro', 'code')}`), 'a design standard'],
];
for (const f of files) for (const [re, what] of BAD) {
    const text = readFileSync(f, 'utf8');
    const m = re.exec(text);
    if (m) check(false, `${f.replace(repo + '/', '')} has ${what}: ${m[0]}`);
}
check(true, `hygiene: ${files.length} files have no em-dash, local path, internal name, work email, competitor name or design standard`);

console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
