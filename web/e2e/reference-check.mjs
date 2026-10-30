// The quick reference page (public/reference/index.html, tools/learn/reference.mjs): every example line of its syntax
// tables compiles and runs on the real engine without an error, the function and unit lists are filled from the engine's
// own table, and the file on disk is what the generator writes.   node e2e/reference-check.mjs
// Needs public/wasm and a Cli binary as tools/make_learn_pages.mjs does (SHEET_WASM_DIR, CALCPAD_CLI).
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runSheet } from '../../tools/learn/engine.mjs';
import { SYNTAX, renderReferencePage } from '../../tools/learn/reference.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

// Names the examples use, defined above each one.
const PRELUDE = 'L: 6 m\nw: 12 kN/m\nu: 0.5\nf(x): x^2\nv: [1, 2, 3]\nn: 3\nx: 2\n';
for (const [, rows] of SYNTAX) {
    for (const [what, line] of rows) {
        if (/^(Heading|Prose|Formula in text|Page break|Raw engine line)$/.test(what)) continue;
        const run = runSheet(PRELUDE + line + '\n');
        check(run.errors.length === 0, `${what}: ${line.split('\n')[0]}${run.errors[0] ? ' -> ' + run.errors[0] : ''}`);
    }
}
// The prose kinds just have to compile.
for (const line of ['# Beam check', 'Plain text, **bold**, *italic*.', 'The moment is $M = wL^2/8$.', '<div style="page-break-after:always"></div>', '>> #deg']) {
    const run = runSheet(line + '\n');
    check(run.errors.length === 0, `compiles: ${line}`);
}

const html = readFileSync(join(here, '..', 'calcpad-web', 'public', 'reference', 'index.html'), 'utf8');
check(html === renderReferencePage(), 'the file on disk is what the generator writes');
for (const name of ['sin(', 'sqrt(', 'matrix(', 'vector(']) check(html.includes(`<code>${name}`), `the function list has ${name}`);
check(/<code title="[^"]*">kN<\/code>/.test(html) && /<code title="[^"]*">MPa<\/code>/.test(html), 'the unit list has kN and MPa');
check((html.match(/<h1>/g) ?? []).length === 1, 'one h1');
check(!/[–—]/.test(html), 'no en or em dash');
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall ok');
process.exit(fails.length ? 1 : 0);
