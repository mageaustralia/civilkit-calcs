// Runs a sheet through the real engine for the landing pages: the sheet compiler (wasm) and then the Cli binary,
// the same path as book/check-examples.mjs. Used by make_learn_pages.mjs (to quote numbers) and by
// web/e2e/learn-check.mjs (to prove the quoted numbers still equal a fresh run).
import { readFileSync, writeFileSync, mkdtempSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
export const root = resolve(here, '..', '..');
export const samplesDir = join(root, 'web', 'calcpad-web', 'public', 'sheet-samples');
const wasmDir = resolve(process.env.SHEET_WASM_DIR ?? join(root, 'web', 'calcpad-web', 'public', 'wasm'));
const cli = process.env.CALCPAD_CLI ?? ['release', 'debug'].map((p) => join(root, 'target', p, 'Cli')).find(existsSync);
if (!existsSync(join(wasmDir, 'calcpad_sheet_wasm_bg.wasm'))) throw new Error(`No sheet module in ${wasmDir}. Run tools/build_wasm.sh sheet, or set SHEET_WASM_DIR.`);
if (!cli || !existsSync(cli)) throw new Error('No Cli binary. Run cargo build --release -p calcpad-cli, or set CALCPAD_CLI.');
const sheet = await import(join(wasmDir, 'calcpad_sheet_wasm.js'));
sheet.initSync({ module: readFileSync(join(wasmDir, 'calcpad_sheet_wasm_bg.wasm')) });

const tmp = mkdtempSync(join(tmpdir(), 'learn-'));
writeFileSync(join(tmp, 'Settings.xml'), '<Settings><Math><Decimals>2</Decimals></Math><Plot><VectorGraphics>true</VectorGraphics></Plot></Settings>');
process.env.CALCPAD_SETTINGS = join(tmp, 'Settings.xml');

/** Plain text of the engine's HTML, with sub/superscripts flattened and whitespace collapsed. */
export const text = (html) =>
    html.replace(/<(script|style)[\s\S]*?<\/\1>/g, '').replace(/<sup>([\s\S]*?)<\/sup>/g, '^$1').replace(/<sub>([\s\S]*?)<\/sub>/g, '_$1')
        .replace(/<svg[\s\S]*?<\/svg>/g, '[drawing]').replace(/<[^>]+>/g, ' ')
        .replace(/&nbsp;/g, ' ').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();

let counter = 0;
/** Compiles and runs a sheet; returns { html, text, errors }. */
export function runSheet(source) {
    const body = /^\s*<!--sheet-->/.test(source) ? source : '<!--sheet-->\n' + source;
    const c = JSON.parse(sheet.compile(body, false));
    const errors = c.diagnostics.filter((d) => d.severity === 'error').map((d) => `sheet line ${d.line + 1}: ${d.message}`);
    return runCpd(c.text, errors);
}

/** Runs Calcpad worksheet text (no sheet compile) through the Cli. */
export function runCpd(cpd, errors = []) {
    const n = ++counter;
    const f = join(tmp, `s${n}.cpd`);
    const out = join(tmp, `s${n}.html`);
    const c = { text: cpd };
    writeFileSync(f, c.text);
    execFileSync(cli, [f, out, '-s'], { stdio: ['ignore', 'pipe', 'pipe'] });
    const html = readFileSync(out, 'utf8');
    for (const e of html.matchAll(/<(?:p|span|div)[^>]*class="err"[^>]*>([\s\S]*?)<\/(?:p|span|div)>/g)) errors.push(text(e[1]).slice(0, 200));
    return { html, text: text(html), errors, calcpad: c.text };
}

const GREEK = { alpha: 'α', beta: 'β', gamma: 'γ', delta: 'δ', epsilon: 'ε', theta: 'θ', lambda: 'λ', mu: 'μ', nu: 'ν', rho: 'ρ', sigma: 'σ', tau: 'τ', phi: 'φ', omega: 'ω' };
/** A sheet name as the engine prints it: sigma -> σ, M_max stays M_max. */
export const printed = (name) => name.replace(/^([A-Za-z]+)/, (w) => GREEK[w] ?? w);
const SUP = { 2: '²', 3: '³', 4: '⁴', 5: '⁵', 6: '⁶', '-1': '⁻¹', '-2': '⁻²' };
/** "56.26 kN · m" -> "56.26 kN·m", "kg ∕ m ^3" -> "kg/m³". */
export const tidyValue = (v) => v.replace(/\s*·\s*/g, '·').replace(/\s*∕\s*/g, '/').replace(/\s*\^(-?\d)/g, (m, d) => SUP[d] ?? '^' + d).replace(/\s+(\d)$/, (m, d) => SUP[d] ?? '^' + d).trim();

/**
 * Every equation of a run as name -> value text: for each `<span class="eq">` the part before the first " = " is
 * the name and the part after the last " = " is the value with its unit. The last equation of a name wins.
 */
export function results(run) {
    const out = new Map();
    for (const m of run.html.matchAll(/<span class="eq">([\s\S]*?)<\/span>(?= <span class="sheet-note">| ?<\/p>)/g)) {
        const t = text(m[1]).replace(/\s+_/g, '_');
        const parts = t.split(' = ');
        if (parts.length < 2) continue;
        out.set(parts[0].trim(), tidyValue(parts[parts.length - 1]));
    }
    return out;
}

/** The value of a sheet name (written as in the sheet) from a run, or null. */
export function valueOf(run, name) {
    return results(run).get(printed(name)) ?? null;
}

export const readSample = (file) => readFileSync(join(samplesDir, file), 'utf8');
