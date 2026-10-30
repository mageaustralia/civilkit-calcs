// Proves that the converted pieces of the interface (the ribbon, the Settings dialog, About, Keyboard shortcuts) render the same
// English text and attributes as before the strings moved into the translator.
//   node e2e/i18n-english-check.mjs --dist DIR            compare with e2e/i18n-english.baseline.json
//   node e2e/i18n-english-check.mjs --dist DIR --record   write the baseline (run on a build made BEFORE the conversion)
//   node e2e/i18n-english-check.mjs --url http://127.0.0.1:4180/   use a server that is already running
// Every text node, and the title, aria-label, placeholder, label and alt attributes and the option texts, are collected from each
// piece, in document order, with whitespace collapsed. The pieces are opened the way a person opens them (the ribbon tabs,
// the Settings button, the About and Shortcuts buttons).
import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFile, writeFile } from 'node:fs/promises';
import { existsSync, statSync } from 'node:fs';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const flag = (n) => process.argv.includes(n);
const BASELINE = join(here, 'i18n-english.baseline.json');
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.woff': 'font/woff', '.ttf': 'font/ttf', '.txt': 'text/plain', '.webmanifest': 'application/manifest+json' };

/** Serves a build folder on a free port; resolves { url, close }. */
export function serve(dist) {
    return new Promise((ok) => {
        const srv = createServer(async (req, res) => {
            try {
                let p = decodeURIComponent(new URL(req.url, 'http://x').pathname);
                if (p.endsWith('/')) p += 'index.html';
                const f = join(dist, p);
                if (!f.startsWith(dist) || !existsSync(f) || statSync(f).isDirectory()) { res.writeHead(404); res.end('not found'); return; }
                res.writeHead(200, { 'Content-Type': MIME[extname(f)] ?? 'application/octet-stream' });
                res.end(await readFile(f));
            } catch {
                res.writeHead(400); res.end('bad request');
            }
        });
        srv.listen(0, '127.0.0.1', () => ok({ url: `http://127.0.0.1:${srv.address().port}/`, close: () => srv.close() }));
    });
}

/** The words of a piece of the page: text nodes and the text-bearing attributes, in order, whitespace collapsed. */
const COLLECT = (arg) => {
    const { sel, skip } = typeof arg === 'string' ? { sel: arg, skip: '' } : arg;
    const root = document.querySelector(sel);
    if (!root) return null;
    const out = [];
    // The release number is masked: it changes at every release and is not a string of the interface.
    const norm = (s) => s.replace(/\s+/g, ' ').trim().replace(/\b\d{2}\.\d{1,2}\.\d+\b/g, '<version>');
    const walk = (n) => {
        if (n.nodeType === 3) { const t = norm(n.nodeValue); if (t) out.push(t); return; }
        if (n.nodeType !== 1) return;
        const tag = n.tagName;
        if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'svg' || tag === 'SVG') return;
        if (skip && n.matches(skip)) return;
        for (const a of ['title', 'aria-label', 'placeholder', 'label', 'alt']) { const v = n.getAttribute(a); if (v) out.push(`@${a}=${norm(v)}`); }
        for (const c of n.childNodes) walk(c);
    };
    walk(root);
    return out;
};

// Worksheets whose lines fail, each with a different message of the engine.
const ENGINE_SHEETS = [
    ['undefined name', 'a = nosuchvariable + 1\nb = 2'],
    ['units', 'x = 2 m + 3 s\ny = 1'],
    ['bracket', 'y = (1 + 2\nz = 3'],
    ['incomplete', 'z = 1 +\nq = 4'],
    ['several', 'a = 1\nb = a + c\nd = 2 m + 3 s\ne = sin(1; 2; 3)\nf = 5'],
];

export async function snapshot(url) {
    const browser = await chromium.launch();
    const ctx = await browser.newContext({ viewport: { width: 1500, height: 1000 }, locale: 'en-US' });
    // The shortcut hints say Cmd on a Mac and Ctrl elsewhere: pin one platform so the snapshot is the same on every machine.
    await ctx.addInitScript(() => Object.defineProperty(Navigator.prototype, 'platform', { get: () => 'Win32' }));
    const page = await ctx.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.waitForSelector('.rb-tab');
    const grab = (sel) => page.evaluate(COLLECT, sel);
    const out = { ribbon: {}, dialogs: {} };
    const tabs = await page.locator('[data-ribbon-tab]').evaluateAll((els) => els.map((e) => e.getAttribute('data-ribbon-tab')));
    for (const id of tabs) {
        await page.locator(`[data-ribbon-tab="${id}"]`).click();
        await page.waitForSelector(`[data-ribbon-panel="${id}"]`);
        out.ribbon[id] = await grab('.rb');
    }
    // The popover of one palette button (heading texts, the filter box).
    await page.locator('[data-ribbon-tab="math"]').click();
    const firstPalette = page.locator('[data-ribbon-panel] .rb-caret').first();
    await firstPalette.click();
    await page.waitForSelector('.rb-pop');
    await page.waitForTimeout(400);
    out.dialogs.firstPalette = (await grab('.rb-pop')).filter((s) => !/^@title=/.test(s)).slice(0, 12);
    await page.keyboard.press('Escape');
    // Help buttons: About and Shortcuts.
    const tabOf = async (btn) => { for (const id of tabs) { await page.locator(`[data-ribbon-tab="${id}"]`).click(); if (await page.locator(`[data-ribbon-btn="${btn}"]`).first().isVisible()) return id; } return null; };
    for (const [name, btn] of [['about', 'x-about'], ['shortcuts', 'x-keys']]) {
        await tabOf(btn);
        await page.locator(`[data-ribbon-btn="${btn}"]`).first().click();
        await page.waitForSelector('#cprs-dialog');
        out.dialogs[name] = await grab('#cprs-dialog');
        await page.keyboard.press('Escape');
        await page.waitForTimeout(150);
    }
    out.dialogs.sidebarTabs = await grab('.calcpad-vue-ui .tab-container');
    // Settings: every section open, so that collapsed bodies are in the page (they are v-show, so their text is in the DOM either way).
    await tabOf('k-settings');
    await page.locator('[data-ribbon-btn="k-settings"]').first().click();
    await page.waitForSelector('.tab-content.as-dialog .settings-tab');
    await page.waitForTimeout(300);
    // The Language section is new (it did not exist before the strings moved), so it is left out of the comparison.
    out.dialogs.settings = await page.evaluate(COLLECT, { sel: '.tab-content.as-dialog .settings-tab', skip: '[data-settings-section="language"]' });
    // The Settings search placeholder and the "no results" text.
    await page.locator('.tab-content.as-dialog .search-input').fill('zzzzqq');
    await page.waitForTimeout(150);
    out.dialogs.settingsHead = await grab('.tab-content.as-dialog .cpd-dialog-head');
    out.dialogs.settingsNoResults = await grab('.tab-content.as-dialog .no-results');
    // The engine's messages: the page and the error list for worksheets that fail, as the transport returns them. The error entries
    // may carry a `code` and `args` (the engine's message codes); the comparison leaves those two fields out.
    out.engine = await page.evaluate(async (sheets) => {
        const res = [];
        for (const [name, content] of sheets) {
            for (const unwrap of [false, true]) {
                const r = await fetch(`/api/calcpad/convert${unwrap ? '?unwrap=true' : ''}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
                const errors = JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') || '[]')).map((e) => { delete e.code; delete e.args; return e; });
                const html = (await r.text()).split(location.origin).join('<origin>');
                const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(html)))).map((b) => b.toString(16).padStart(2, '0')).join('');
                // The whole page is large (it carries the page styles); keep its hash and the error markup, which is what the codes touch.
                const err = html.match(/<(span|p|b) class="err[^"]*"[\s\S]*?<\/\1>/g) ?? [];
                res.push({ name: name + (unwrap ? ' (unwrapped)' : ''), status: r.status, length: html.length, sha256: digest, err, errors });
            }
        }
        return res;
    }, ENGINE_SHEETS);
    await browser.close();
    out.pageErrors = errors;
    return out;
}

async function main() {
    let { url } = { url: arg('--url') };
    let srv = null;
    if (!url) {
        const dist = resolve(arg('--dist', join(here, '..', 'calcpad-web', 'dist')));
        srv = await serve(dist);
        url = srv.url;
    }
    const lang = arg('--lang');
    const snap = await snapshot(lang ? `${url}?lang=${lang}` : url);
    srv?.close();
    if (flag('--record')) {
        await writeFile(BASELINE, JSON.stringify(snap, null, 1) + '\n');
        const n = Object.values(snap.ribbon).reduce((a, v) => a + v.length, 0) + Object.values(snap.dialogs).reduce((a, v) => a + v.length, 0);
        console.log(`recorded ${n} strings to ${BASELINE}`);
        return;
    }
    const base = JSON.parse(await readFile(BASELINE, 'utf8'));
    let bad = 0, total = 0;
    const cmp = (name, a, b) => {
        const A = a ?? [], B = b ?? [];
        total += Math.max(A.length, B.length);
        if (JSON.stringify(A) === JSON.stringify(B)) { console.log(`ok   ${name} (${A.length} strings)`); return; }
        bad++;
        console.log(`FAIL ${name}`);
        const set = new Set(B);
        const cut = (x) => (x.length > 300 ? x.slice(0, 300) + '...' : x);
        for (const s of A.filter((x) => !set.has(x)).slice(0, 8)) console.log(`   baseline only: ${cut(s)}`);
        const set2 = new Set(A);
        for (const s of B.filter((x) => !set2.has(x)).slice(0, 8)) console.log(`   now only:      ${cut(s)}`);
        if (A.length === B.length && A.every((x) => set.has(x))) console.log('   same strings, different order');
    };
    for (const id of Object.keys(base.ribbon)) cmp(`ribbon tab ${id}`, base.ribbon[id], snap.ribbon[id]);
    for (const k of Object.keys(base.dialogs)) cmp(`piece ${k}`, base.dialogs[k], snap.dialogs[k]);
    cmp('engine pages and error lists', (base.engine ?? []).map((e) => JSON.stringify(e)), (snap.engine ?? []).map((e) => JSON.stringify(e)));
    if (snap.pageErrors.length) { bad++; console.log(`FAIL page errors: ${snap.pageErrors[0]}`); }
    console.log(bad ? `${bad} FAIL` : `all ok (${total} strings compared)`);
    process.exit(bad ? 1 : 0);
}
if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
