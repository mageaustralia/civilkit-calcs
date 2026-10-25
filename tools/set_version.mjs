#!/usr/bin/env node
// One source of truth for the version: [workspace.package] version in the root Cargo.toml.
//   node tools/set_version.mjs 26.10.2      sets it everywhere (Cargo.toml, Cargo.lock, every package.json and the npm lock)
//   node tools/set_version.mjs --check      fails if any manifest disagrees with Cargo.toml (run by e2e/ribbon-check.mjs)
// The scheme is YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
// of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are v26.10.1.
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const rel = (p) => join(root, p);
const SEMVER = /^(\d{2})\.([1-9]|1[0-2])\.([1-9]\d*)$/;
// The three library crates (civilkit-calcs-engine, -highlighter, -docx) are published to crates.io on their own 0.x.y
// semver and are not touched here; the app and its unpublished crates carry the YY.M.N version.
const CRATES = ['calcpad-cli', 'calcpad-wasm', 'calcpad-docx-wasm', 'calcpad-highlighter-wasm', 'civilkit-calcs-import', 'calcpad-import-wasm', 'civilkit-calcs-sheet', 'calcpad-sheet-wasm', 'calcpad-linalg-wasm', 'calcpad-caskit-wasm'];
const PACKAGES = ['web/package.json', 'web/calcpad-web/package.json', 'web/calcpad-frontend/package.json'];

const cargoToml = readFileSync(rel('Cargo.toml'), 'utf8');
const tomlRe = /(\[workspace\.package\][^\[]*?\nversion\s*=\s*")([^"]+)(")/;
const current = tomlRe.exec(cargoToml)?.[2];
if (!current) { console.error('no [workspace.package] version in Cargo.toml'); process.exit(2); }

/** Every place that carries the version, as [label, value]. */
function found() {
    const out = [['Cargo.toml', current]];
    const lock = readFileSync(rel('Cargo.lock'), 'utf8');
    for (const c of CRATES) {
        const m = new RegExp(`name = "${c}"\\nversion = "([^"]+)"`).exec(lock);
        out.push([`Cargo.lock ${c}`, m ? m[1] : '(missing)']);
    }
    for (const p of PACKAGES) out.push([p, JSON.parse(readFileSync(rel(p), 'utf8')).version ?? '(none)']);
    if (existsSync(rel('web/package-lock.json'))) {
        const lock = JSON.parse(readFileSync(rel('web/package-lock.json'), 'utf8'));
        out.push(['web/package-lock.json', lock.version ?? '(none)']);
        for (const k of ['', 'calcpad-frontend', 'calcpad-web']) out.push([`web/package-lock.json packages["${k}"]`, lock.packages?.[k]?.version ?? '(none)']);
    }
    return out;
}

const arg = process.argv[2];
if (arg === '--check') {
    const bad = found().filter(([, v]) => v !== current);
    if (bad.length) { console.error(`version mismatch (Cargo.toml says ${current}):\n` + bad.map(([l, v]) => `  ${l}: ${v}`).join('\n')); process.exit(1); }
    if (!SEMVER.test(current)) { console.error(`${current} is not YY.M.N without zero padding`); process.exit(1); }
    console.log(`all manifests agree: ${current}`);
} else if (arg && SEMVER.test(arg)) {
    writeFileSync(rel('Cargo.toml'), cargoToml.replace(tomlRe, `$1${arg}$3`));
    let lock = readFileSync(rel('Cargo.lock'), 'utf8');
    for (const c of CRATES) lock = lock.replace(new RegExp(`(name = "${c}"\\nversion = ")[^"]+(")`), `$1${arg}$2`);
    writeFileSync(rel('Cargo.lock'), lock);
    for (const p of PACKAGES) {
        const text = readFileSync(rel(p), 'utf8');
        writeFileSync(rel(p), text.replace(/("version"\s*:\s*")[^"]+(")/, `$1${arg}$2`));
    }
    if (existsSync(rel('web/package-lock.json'))) {
        const lock = JSON.parse(readFileSync(rel('web/package-lock.json'), 'utf8'));
        lock.version = arg;
        for (const k of ['', 'calcpad-frontend', 'calcpad-web']) if (lock.packages?.[k]) lock.packages[k].version = arg;
        writeFileSync(rel('web/package-lock.json'), JSON.stringify(lock, null, 2) + '\n');
    }
    console.log(`version set to ${arg}; rebuild the wasm (tools/build_wasm.sh) so version() reports it`);
} else {
    console.error('usage: node tools/set_version.mjs <YY.M.N> | --check');
    process.exit(2);
}
