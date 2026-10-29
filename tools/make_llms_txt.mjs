// Writes web/calcpad-web/public/llms.txt (an index of the guide for language models and other tools) and llms-full.txt (the
// guide itself as one Markdown file).
//   node tools/make_llms_txt.mjs            write the files
//   node tools/make_llms_txt.mjs --check    exit 1 if a file on disk differs from what would be written
// Both come from book/src (SUMMARY.md gives the order and the section names), docs/sheet-syntax.md and the landing pages in
// public/learn, so they follow the guide with nothing to keep in step by hand. They are served at /calcs/llms.txt and
// /calcs/llms-full.txt; a host that wants /llms.txt at its root can copy them there.
import { readFileSync, writeFileSync, existsSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const pub = join(root, 'web', 'calcpad-web', 'public');
const SITE = 'https://civilkit.io/calcs/';
const GITHUB = 'https://github.com/mageaustralia/civilkit-calcs';

const read = (p) => readFileSync(join(root, p), 'utf8');

/** The first sentence of the first prose paragraph after the chapter's heading, flattened to one line. */
function summaryOf(md) {
    const lines = md.split(/\r?\n/);
    const paras = [];
    let cur = [], fence = false;
    for (const raw of lines.slice(Math.max(lines.findIndex((l) => /^#\s/.test(l)) + 1, 0))) {
        const l = raw.trim();
        if (/^(```|~~~)/.test(l)) { fence = !fence; if (cur.length) { paras.push(cur); cur = []; } continue; }
        if (fence) continue;
        if (!l) { if (cur.length) { paras.push(cur); cur = []; } continue; }
        if (/^(#|>|[-*|]|!\[|<)/.test(l) || /^\d+\./.test(l)) { if (cur.length) { paras.push(cur); cur = []; } continue; }
        cur.push(l);
    }
    if (cur.length) paras.push(cur);
    for (const p of paras) {
        const text = p.join(' ').replace(/\[([^\]]+)\]\([^)]*\)/g, '$1').replace(/[`*_]/g, '').replace(/\s+/g, ' ').trim();
        if (!/^[A-Z0-9]/.test(text) || !text.includes(' ')) continue;
        const m = /^(.+?[.!?])(\s|$)/.exec(text);
        const first = (m ? m[1] : text.replace(/:$/, '.')).slice(0, 240);
        return first.replace(/:$/, '.');
    }
    return '';
}

const titleOf = (md, fallback) => (/^#\s+(.+)$/m.exec(md)?.[1] ?? fallback).trim();

/** The chapters of the guide in order: { section, title, file, url, md }. */
function chapters() {
    const out = [];
    let section = 'Guide';
    for (const line of read('book/src/SUMMARY.md').split(/\r?\n/)) {
        const h = /^#\s+(.+)$/.exec(line);
        if (h && h[1] !== 'Summary') { section = h[1].trim(); continue; }
        const m = /^\s*[-*]?\s*\[([^\]]+)\]\(([^)]+\.md)\)/.exec(line);
        if (!m) continue;
        const md = read(`book/src/${m[2]}`);
        out.push({ section, title: m[1], file: m[2], url: `${SITE}guide/${m[2].replace(/\.md$/, '.html')}`, md });
    }
    return out;
}

/** The landing pages under public/learn: { title, url, description }. */
function landingPages() {
    const dir = join(pub, 'learn');
    if (!existsSync(dir)) return [];
    return readdirSync(dir, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name).sort().flatMap((slug) => {
        const f = join(dir, slug, 'index.html');
        if (!existsSync(f)) return [];
        const html = readFileSync(f, 'utf8');
        const t = /<title>([^<]+)<\/title>/.exec(html)?.[1];
        const d = /<meta name="description" content="([^"]*)"/.exec(html)?.[1] ?? '';
        return t ? [{ title: t.replace(/&amp;/g, '&'), url: `${SITE}learn/${slug}/`, description: d.replace(/&amp;/g, '&') }] : [];
    });
}

export function outputs() {
    const ch = chapters();
    const sections = [...new Set(ch.map((c) => c.section))];
    const index = [];
    index.push('# CivilKit Calcs');
    index.push('');
    index.push('> CivilKit Calcs is a free, browser-based engineering calculation sheet. It runs entirely in the browser (no server, works offline) on a Rust port of the CalcpadCE engine. A sheet is plain text: Markdown prose with `name: expression` lines that carry real units, so results come out in the right units. It also reads and writes Calcpad worksheets, and exports to HTML, PDF, Word and Excel.');
    index.push('');
    index.push('Key facts for tools and assistants:');
    index.push('');
    index.push('- A sheet starts with `<!--sheet-->`. A quantity line is `name: expression -> unit | note`, for example `Mstar: w * L^2 / 8 -> kN*m | design moment`.');
    index.push('- An input line is `L: ? 6 m` (a number box in Input mode); `L: ? 6 m [2 .. 12 step 0.5]` adds a slider.');
    index.push('- Units are checked: a stress times a section modulus comes out as a moment. `-> %` shows a ratio as a percentage.');
    index.push('- Diagrams are Mermaid fences written in the sheet; plots, Python blocks and symbolic algebra are built in.');
    index.push('');
    index.push(`The whole guide in one file: ${SITE}llms-full.txt`);
    index.push('');
    for (const s of sections) {
        index.push(`## ${s}`);
        index.push('');
        for (const c of ch.filter((x) => x.section === s)) {
            const d = summaryOf(c.md);
            index.push(`- [${c.title}](${c.url})${d ? `: ${d}` : ''}`);
        }
        index.push('');
    }
    const learn = landingPages();
    if (learn.length) {
        index.push('## Task pages');
        index.push('');
        for (const p of learn) index.push(`- [${p.title}](${p.url})${p.description ? `: ${p.description}` : ''}`);
        index.push('');
    }
    index.push('## Optional');
    index.push('');
    index.push(`- [App](${SITE}): open the calculator`);
    index.push(`- [What's new](${SITE}whats-new/): release notes with short clips`);
    index.push(`- [Source code and issues](${GITHUB})`);
    index.push('');

    const full = [];
    full.push('# CivilKit Calcs: the guide in one file');
    full.push('');
    full.push(`Source: ${SITE}guide/ . Each chapter below starts with its address.`);
    full.push('');
    for (const c of ch) {
        full.push('---');
        full.push('');
        full.push(`Chapter: ${c.title}`);
        full.push(`URL: ${c.url}`);
        full.push('');
        full.push(c.md.trim());
        full.push('');
    }
    const syntax = read('docs/sheet-syntax.md').trim();
    full.push('---');
    full.push('');
    full.push('Reference: the sheet syntax');
    full.push(`Source: ${GITHUB}/blob/main/docs/sheet-syntax.md`);
    full.push('');
    full.push(syntax);
    full.push('');

    return [
        { path: 'llms.txt', content: index.join('\n') },
        { path: 'llms-full.txt', content: full.join('\n') },
    ];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
    const check = process.argv.includes('--check');
    let bad = 0;
    for (const f of outputs()) {
        const p = join(pub, f.path);
        const now = existsSync(p) ? readFileSync(p, 'utf8') : null;
        if (check) { if (now !== f.content) { console.error(`out of date: public/${f.path}`); bad++; } }
        else { writeFileSync(p, f.content); console.log(`wrote public/${f.path} (${f.content.length} bytes)`); }
    }
    if (check) { if (bad) process.exit(1); console.log('llms files are up to date'); }
}
