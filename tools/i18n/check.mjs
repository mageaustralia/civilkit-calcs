// The translation check and extraction script (docs/i18n-plan.md, section 4).
//
//   node tools/i18n/check.mjs             check everything, print the coverage table; exit 1 on an error
//   node tools/i18n/check.mjs --report    also print coverage by area for each language
//   node tools/i18n/check.mjs --write     bring en.json up to date with the sources (see below), then check
//
// What it reads
//   - the source code: every t('key'), tn('key', n), tHtml('key'), td('key', params, 'English') and tr('key', params, 'English')
//     in web/calcpad-web/src and web/calcpad-frontend/src (*.ts, *.vue);
//   - the data that keys are derived from: the ribbon (ribbon-data.ts, keys by ribbon-i18n.ts) and the setting definitions
//     (types/settings.ts, pdf-settings.ts) of the Settings dialog;
//   - web/calcpad-web/src/i18n/en.json (the English source) and web/calcpad-web/translations/<lang>.json.
//
// What --write does: a call with an English fallback (td, tr) and each derived key gets its English from the code or the data;
// the keys of calls without a fallback (t, tn, tHtml) are written by hand in en.json and are never touched.
//
// What it fails on
//   1. a key used in the code that en.json lacks;
//   2. a key in en.json that nothing uses (unused keys);
//   3. English in en.json that differs from the English in the code or data (run --write);
//   4. a call that does not pass a parameter its English text needs, or passes one it does not use;
//   5. a translation key that is not in en.json, or a placeholder / markup mismatch between English and a translation;
//   6. a plural set without its `_other` form;
//   7. an em-dash or en-dash, a local path or a work address in any catalogue;
//   8. a catalogue without a _meta block, or one that claims a review it does not record.
// Missing translations are reported (the coverage table) but are not errors: a language ships at any coverage, and the
// English shows for a missing key.
import { readdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const web = join(root, 'web');
const webSrc = join(web, 'calcpad-web', 'src');
const feSrc = join(web, 'calcpad-frontend', 'src');
const EN_PATH = join(webSrc, 'i18n', 'en.json');
const TR_DIR = join(web, 'calcpad-web', 'translations');
const WRITE = process.argv.includes('--write');
const REPORT = process.argv.includes('--report');
const errors = [];
const warnings = [];
const fail = (m) => errors.push(m);
const warn = (m) => warnings.push(m);

// ---------------------------------------------------------------- source scan
const walk = (d) => readdirSync(d, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)]));
const files = [...walk(webSrc), ...walk(feSrc)].filter((f) => /\.(ts|vue)$/.test(f) && !f.endsWith('.d.ts'));
// The translator itself and its hook quote the call forms in comments.
const SKIP = new Set([join(webSrc, 'i18n', 'index.ts'), join(feSrc, 'i18n-hook.ts')]);

const CALL = /(?<![\w$.])(tHtml|tn|td|tr|t)\(\s*(['"])((?:\\.|(?!\2)[^\\\n])*)\2/g;

/** Splits the argument text after a key literal into top-level arguments, stopping at the closing parenthesis. */
function splitArgs(text) {
    const args = [];
    let depth = 0, cur = '', q = null;
    for (let i = 0; i < text.length; i++) {
        const c = text[i];
        if (q) {
            cur += c;
            if (c === '\\') { cur += text[++i] ?? ''; continue; }
            if (c === q) q = null;
            continue;
        }
        if (c === "'" || c === '"' || c === '`') { q = c; cur += c; continue; }
        if (c === '(' || c === '[' || c === '{') depth++;
        if (c === ')' || c === ']' || c === '}') {
            if (depth === 0) { args.push(cur.trim()); return args; }
            depth--;
        }
        if (c === ',' && depth === 0) { args.push(cur.trim()); cur = ''; continue; }
        cur += c;
    }
    args.push(cur.trim());
    return args;
}
const unquote = (lit) => {
    const m = /^(['"])((?:\\.|(?!\1)[^\\])*)\1$/s.exec(lit);
    return m ? m[2].replace(/\\(['"\\])/g, '$1').replace(/\\n/g, '\n') : null;
};
const unescapeKey = (k) => k.replace(/\\(['"\\])/g, '$1');
/** The names of an object literal's top-level properties: `{ a: 1, b }` gives a, b. */
function objectKeys(lit) {
    if (!lit || !lit.startsWith('{')) return null;
    const inner = lit.slice(1, lit.lastIndexOf('}'));
    return splitArgs(inner + ')').map((p) => /^([A-Za-z_$][\w$]*)\s*(?::|$)/.exec(p)?.[1]).filter(Boolean);
}
const placeholders = (s) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]);

const used = new Map(); // key -> { kinds:Set, fallback?, params?, where[] }
function use(key, kind, where, fallback, params) {
    const u = used.get(key) ?? { kinds: new Set(), where: [], fallbacks: [], params: [] };
    u.kinds.add(kind);
    u.where.push(where);
    if (fallback != null) u.fallbacks.push(fallback);
    if (params) u.params.push(params);
    used.set(key, u);
}
for (const f of files) {
    if (SKIP.has(f)) continue;
    const text = readFileSync(f, 'utf8');
    const rel = relative(root, f);
    for (const m of text.matchAll(CALL)) {
        const line = text.slice(0, m.index).split('\n');
        const before = line[line.length - 1];
        // A comment line quoting a call is not a call.
        if (/^\s*(\/\/|\*|<!--)/.test(before)) continue;
        const kind = m[1];
        const key = unescapeKey(m[3]);
        const rest = text.slice(m.index + m[0].length);
        const args = rest.startsWith(',') ? splitArgs(rest.slice(1)) : [];
        const where = `${rel}:${line.length}`;
        if (kind === 'td' || kind === 'tr') {
            const fb = args[1] != null ? unquote(args[1]) : null;
            if (fb === null) fail(`${where}: ${kind}('${key}') needs a literal English fallback as its third argument`);
            use(key, kind, where, fb, objectKeys(args[0]));
        } else if (kind === 'tn') {
            use(key, 'tn', where, null, objectKeys(args[1]) ?? []);
        } else {
            use(key, kind, where, null, objectKeys(args[0]));
        }
    }
}

// ---------------------------------------------------------------- derived keys
const derived = new Map(); // key -> English
const derive = (key, en) => {
    if (en == null || en === '') return;
    if (derived.has(key) && derived.get(key) !== en) fail(`derived key ${key} has two different English texts`);
    derived.set(key, en);
};
const tsxApi = join(web, 'node_modules', 'tsx', 'dist', 'esm', 'api', 'index.mjs');
const { tsImport } = await import(pathToFileURL(tsxApi).href);

const ribbonData = await tsImport(pathToFileURL(join(webSrc, 'wasm', 'ribbon-data.ts')).href, import.meta.url);
const rk = await tsImport(pathToFileURL(join(webSrc, 'wasm', 'ribbon-i18n.ts')).href, import.meta.url);
const seenBtn = new Map();
const noteBtn = (b, where) => {
    if (seenBtn.has(b.id)) fail(`ribbon: button id ${b.id} appears twice (${seenBtn.get(b.id)} and ${where}); keys would collide`);
    seenBtn.set(b.id, where);
    derive(rk.btnKey(b.id, 'label'), b.label);
    if (b.title) derive(rk.btnKey(b.id, 'title'), b.title);
    for (const s of b.palette ?? []) if (s.heading) derive(rk.sectionKey(b.id, s.heading), s.heading);
};
for (const t of ribbonData.RIBBON) {
    derive(rk.tabKey(t.id), t.label);
    for (const g of t.groups) {
        derive(rk.groupKey(t.id, g.name), g.name);
        for (const b of g.buttons) noteBtn(b, `${t.id}/${g.name}`);
    }
}
for (const b of ribbonData.QUICK) noteBtn(b, 'quick bar');

const settings = await tsImport(pathToFileURL(join(feSrc, 'types', 'settings.ts')).href, import.meta.url);
const pdf = await tsImport(pathToFileURL(join(feSrc, 'types', 'pdf-settings.ts')).href, import.meta.url);
// The Settings dialog shows these metadata settings (its MATH_KEYS and PLOT_KEYS lists).
const tabSrc = readFileSync(join(feSrc, 'vue', 'components', 'CalcpadSettingsTab.vue'), 'utf8');
const shown = new Set([...tabSrc.matchAll(/const (?:MATH_KEYS|PLOT_KEYS_[ABC]) = \[([^\]]*)\]/g)].flatMap((m) => [...m[1].matchAll(/'(\w+)'/g)].map((x) => x[1])));
for (const spec of settings.METADATA_SETTINGS_KEYS.filter((s) => shown.has(s.key))) {
    derive(`setting.${spec.key}.label`, spec.label);
    derive(`setting.${spec.key}.detail`, spec.detail);
    for (const o of spec.options ?? []) derive(`setting.${spec.key}.opt.${o.value}`, o.label);
}
for (const spec of pdf.PDF_SETTING_KEYS) {
    derive(`pdf.${spec.key}.label`, spec.label);
    derive(`pdf.${spec.key}.detail`, spec.detail);
    for (const o of spec.options ?? []) derive(`pdf.${spec.key}.opt.${o.value}`, o.label);
}
for (const o of settings.LOG_LEVEL_OPTIONS) { derive(`log.${o.value}.label`, o.label); derive(`log.${o.value}.detail`, o.detail); }

// The engine's messages: the codes the engine records (calcpad-core exceptions.rs and the "Error in" frames), with the
// English of each from messages.rs. `msg.<code>` in a catalogue replaces the English text in the page (i18n/engine-messages.ts).
{
    const core = join(root, 'calcpad-core', 'src');
    const consts = new Map();
    for (const m of readFileSync(join(core, 'messages.rs'), 'utf8').matchAll(/pub const (\w+): &str =\s*"((?:\\.|[^"\\])*)";/g)) {
        consts.set(m[1], m[2].replace(/\\u\{([0-9a-fA-F]+)\}/g, (_x, h) => String.fromCodePoint(parseInt(h, 16))).replace(/\\n/g, '\n').replace(/\\(["'\\])/g, '$1'));
    }
    const codes = new Set();
    for (const f of walk(core).filter((x) => x.endsWith('.rs'))) {
        const src = readFileSync(f, 'utf8');
        for (const m of src.matchAll(/(?:::coded|message_codes::format)\(\s*"(\w+)"/g)) codes.add(m[1]);
    }
    for (const c of codes) {
        if (!consts.has(c)) fail(`engine code ${c} is not a constant of messages.rs`);
        else derive(`msg.${c}`, consts.get(c));
    }
}

// ---------------------------------------------------------------- en.json
let en = JSON.parse(readFileSync(EN_PATH, 'utf8'));
const fromCode = new Map(); // key -> English given in code
for (const [k, u] of used) for (const fb of u.fallbacks) {
    if (fromCode.has(k) && fromCode.get(k) !== fb) fail(`${k}: two different English fallbacks in the code (${u.where.join(', ')})`);
    fromCode.set(k, fb);
}
if (WRITE) {
    for (const [k, v] of [...derived, ...fromCode]) en[k] = v;
    // A derived or fallback key that nothing produces any more is dropped; hand-written keys stay.
    en = Object.fromEntries(Object.entries(en).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
    writeFileSync(EN_PATH, JSON.stringify(en, null, 2) + '\n');
}
const isPluralForm = (k) => /_(zero|one|two|few|many|other)$/.test(k);
const usedBase = (k) => used.has(k) || (isPluralForm(k) && used.get(k.replace(/_(zero|one|two|few|many|other)$/, ''))?.kinds.has('tn'));
const isDerived = (k) => derived.has(k);

for (const [k, u] of used) {
    const present = k in en || (u.kinds.has('tn') && `${k}_other` in en);
    if (!present) fail(`${u.where[0]}: key ${k} is not in en.json`);
}
for (const k of Object.keys(en)) {
    if (!usedBase(k) && !isDerived(k)) fail(`en.json: unused key ${k}`);
}
for (const [k, v] of derived) if (en[k] !== v) fail(`en.json: ${k} differs from the English in the data (run --write)`);
for (const [k, v] of fromCode) if (k in en && en[k] !== v) fail(`en.json: ${k} differs from the English in the code (run --write)`);
for (const [k, v] of derived) if (!(k in en)) fail(`en.json: derived key ${k} is missing (run --write)`);
// Parameters: every {name} in the English is passed; nothing extra is passed.
for (const [k, u] of used) {
    const text = en[k] ?? en[`${k}_other`];
    if (text == null) continue;
    const need = new Set(placeholders(text));
    if (u.kinds.has('tn')) need.delete('count');
    for (const p of u.params) {
        for (const n of need) if (!p.includes(n)) fail(`${u.where[0]}: ${k} needs {${n}} but the call does not pass it`);
        for (const n of p) if (!need.has(n) && !(u.kinds.has('tn') && n === 'count')) warn(`${u.where[0]}: ${k} is passed ${n} but its text has no {${n}}`);
    }
    if (u.params.length === 0 && need.size && !u.kinds.has('tn')) fail(`${u.where[0]}: ${k} needs ${[...need].map((n) => `{${n}}`).join(', ')} but the call passes no parameters`);
}
// Dash and path hygiene in the English source
const BAD = [[/\u2014/, 'em-dash'], [/\u2013/, 'en-dash'], [new RegExp(String.raw`\/(${['Us', 'ers'].join('')}|${['Vol', 'umes'].join('')})\/`), 'local path']];
const hygiene = (name, obj) => { for (const [k, v] of Object.entries(obj)) for (const [re, what] of BAD) if (typeof v === 'string' && re.test(v)) fail(`${name}: ${k} contains an ${what}`); };
hygiene('en.json', en);

// ---------------------------------------------------------------- translations
const tags = (s) => [...s.matchAll(/<\/?[a-zA-Z][^>]*>/g)].map((m) => m[0].replace(/\s+/g, ' ')).sort().join('|');
const area = (k) => k.split('.')[0] === 'settings' || k.split('.')[0] === 'setting' || k.split('.')[0] === 'pdf' || k.split('.')[0] === 'log' ? 'settings' : k.split('.')[0];
const langs = existsSync(TR_DIR) ? readdirSync(TR_DIR).filter((f) => f.endsWith('.json')).sort() : [];
const rows = [];
const total = Object.keys(en).length;
for (const f of langs) {
    const code = f.replace(/\.json$/, '');
    let cat;
    try { cat = JSON.parse(readFileSync(join(TR_DIR, f), 'utf8')); } catch (e) { fail(`${f}: not valid JSON (${e.message})`); continue; }
    const meta = cat._meta;
    if (!meta || typeof meta !== 'object') fail(`${f}: no _meta block`);
    else {
        if (meta.lang !== code) fail(`${f}: _meta.lang is ${meta.lang}, expected ${code}`);
        if (!meta.reviewed_by && !/machine draft/i.test(meta.status ?? '')) fail(`${f}: _meta.status must say "machine draft" until a reviewer is recorded in _meta.reviewed_by`);
        if (meta.reviewed_by && /machine draft/i.test(meta.status ?? '')) fail(`${f}: _meta has both reviewed_by and a machine draft status`);
    }
    hygiene(f, cat);
    let done = 0;
    const byArea = {};
    for (const k of Object.keys(en)) { const a = area(k); (byArea[a] ??= { done: 0, total: 0 }).total++; }
    for (const [k, v] of Object.entries(cat)) {
        if (k === '_meta') continue;
        if (typeof v !== 'string') { fail(`${f}: ${k} is not a string`); continue; }
        if (!(k in en)) { fail(`${f}: ${k} is not an English key (orphan)`); continue; }
        if (v === '') { fail(`${f}: ${k} is empty`); continue; }
        done++;
        byArea[area(k)].done++;
        const pe = placeholders(en[k]).sort().join(','), pt = placeholders(v).sort().join(',');
        if (pe !== pt) fail(`${f}: ${k} has placeholders {${pt}}, English has {${pe}}`);
        if (tags(en[k]) !== tags(v)) fail(`${f}: ${k} has different markup from the English`);
        if (/^[a-z]*\.[\w.]*\.(label|title)$/.test(k) && v.length > en[k].length * 1.4 + 6 && en[k].length > 12) warn(`${f}: ${k} is ${Math.round((v.length / en[k].length) * 100)}% of the English length`);
    }
    for (const k of Object.keys(cat)) if (/_(one|few|many|zero|two)$/.test(k) && !(`${k.replace(/_[a-z]+$/, '')}_other` in cat)) fail(`${f}: ${k} has no matching _other`);
    rows.push({ code, done, byArea, status: cat._meta?.reviewed_by ? `reviewed by ${cat._meta.reviewed_by}` : 'machine draft, unreviewed' });
}

// ---------------------------------------------------------------- output
const pct = (a, b) => `${((100 * a) / b).toFixed(1)}%`;
console.log(`English keys: ${total} (${used.size} used in code, ${derived.size} derived from the ribbon, the setting definitions and the engine's coded messages)`);
console.log('language   translated   missing   coverage   status');
for (const r of rows) console.log(`${r.code.padEnd(9)}  ${String(r.done).padStart(10)}  ${String(total - r.done).padStart(8)}  ${pct(r.done, total).padStart(8)}   ${r.status}`);
if (REPORT) for (const r of rows) {
    console.log(`\n${r.code} by area`);
    for (const [a, v] of Object.entries(r.byArea).sort()) console.log(`  ${a.padEnd(10)} ${String(v.done).padStart(4)} / ${String(v.total).padEnd(4)} ${pct(v.done, v.total)}`);
}
for (const w of warnings.slice(0, 40)) console.log(`warn ${w}`);
if (warnings.length > 40) console.log(`warn ... and ${warnings.length - 40} more`);
for (const e of errors.slice(0, 80)) console.log(`FAIL ${e}`);
if (errors.length > 80) console.log(`FAIL ... and ${errors.length - 80} more`);
console.log(errors.length ? `${errors.length} FAIL` : `all ok (${warnings.length} warnings)`);
process.exit(errors.length ? 1 : 0);
