// The canonical address of a guide chapter. The host serves book/src/units.md (mdBook writes units.html) at the
// extensionless /calcs/guide/units and redirects the .html form to it, so everything that links to the guide (sitemap,
// llms.txt, landing pages, the guide's own canonical tags) uses the form below. Directory chapters (sheet/index.md) are
// served at their folder, and the introduction is the guide's front page.
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const GUIDE = 'https://civilkit.io/calcs/guide/';

/** 'units.md' -> 'units', 'sheet/index.md' -> 'sheet/', 'introduction.md' -> '' (relative to /calcs/guide/). */
export function guidePath(md) {
    const f = md.replace(/^\.?\//, '').replace(/\.(md|html)$/, '');
    if (f === 'introduction' || f === 'index') return '';
    return f.endsWith('/index') ? f.slice(0, -'index'.length) : f;
}

/** The absolute canonical URL of a chapter. */
export const guideUrl = (md) => GUIDE + guidePath(md);

/** The chapters of the guide in SUMMARY.md order: { section, title, file }. */
export function guideChapters() {
    const out = [];
    let section = 'Guide';
    for (const line of readFileSync(join(root, 'book', 'src', 'SUMMARY.md'), 'utf8').split(/\r?\n/)) {
        const h = /^#\s+(.+)$/.exec(line);
        if (h && h[1] !== 'Summary') { section = h[1].trim(); continue; }
        const m = /^\s*[-*]?\s*\[([^\]]+)\]\(([^)]+\.md)\)/.exec(line);
        if (m) out.push({ section, title: m[1], file: m[2] });
    }
    return out;
}

/** The title of a chapter as the table of contents names it. */
export function chapterTitle(md) {
    const c = guideChapters().find((x) => x.file === md);
    if (!c) throw new Error(`no guide chapter ${md}`);
    return c.title;
}

/**
 * The first sentence of the first prose paragraph after the chapter's heading, flattened to one line (no Markdown).
 * Used for the llms.txt index and for each guide page's meta description.
 */
export function summaryOf(md) {
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

/**
 * The meta description of a guide page: a `<!-- description: ... -->` comment in the chapter if it has one, otherwise
 * its first sentence (summaryOf). A chapter whose first sentence is shorter than 70 or longer than 160 characters
 * carries the comment (web/e2e/seo-check.mjs fails otherwise).
 */
export function descriptionOf(md) {
    const own = /<!--\s*description:\s*([\s\S]*?)\s*-->/.exec(md);
    return own ? own[1].replace(/\s+/g, ' ').trim() : summaryOf(md);
}
