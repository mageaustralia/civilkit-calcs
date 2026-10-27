// Runs every ```sheet and ```cpd block of the book through the real engine.
//   node check-examples.mjs [--show] [file.md ...]
// A sheet is compiled by the sheet wasm module, then the Calcpad text is run by the Cli binary.
// ```python blocks are only compiled with CPython to catch syntax errors; they run in the page (check-python.mjs).
// A block tagged `sheet python` is a sheet with Python fences: it is compiled and the Python is syntax checked.
// An example fails when the sheet compiler reports an error or the engine writes an error into the page.
// --show prints the text of each result (used to copy real output into the book).
// Environment: SHEET_WASM_DIR (default ../web/calcpad-web/public/wasm), CALCPAD_CLI (default ../target/debug/Cli
// or ../target/release/Cli, whichever exists).
import { readFileSync, writeFileSync, readdirSync, statSync, mkdtempSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const show = args.includes('--show');
const files = args.filter((a) => !a.startsWith('--')).map((f) => resolve(process.env.ORIG_PWD ?? process.cwd(), f));

const wasmDir = resolve(process.env.SHEET_WASM_DIR ?? join(here, '..', 'web', 'calcpad-web', 'public', 'wasm'));
const cli = process.env.CALCPAD_CLI ?? ['debug', 'release'].map((p) => join(here, '..', 'target', p, 'Cli')).find(existsSync);
if (!existsSync(join(wasmDir, 'calcpad_sheet_wasm_bg.wasm'))) {
    console.error(`No sheet module in ${wasmDir}. Run tools/build_wasm.sh sheet, or set SHEET_WASM_DIR.`);
    process.exit(2);
}
if (!cli || !existsSync(cli)) {
    console.error('No Cli binary. Run cargo build -p calcpad-cli, or set CALCPAD_CLI.');
    process.exit(2);
}
const sheet = await import(join(wasmDir, 'calcpad_sheet_wasm.js'));
sheet.initSync({ module: readFileSync(join(wasmDir, 'calcpad_sheet_wasm_bg.wasm')) });

function walk(dir) {
    return readdirSync(dir).flatMap((n) => {
        const p = join(dir, n);
        return statSync(p).isDirectory() ? walk(p) : p.endsWith('.md') ? [p] : [];
    });
}
const mds = files.length ? files.flatMap((f) => (statSync(f).isDirectory() ? walk(f) : [f])) : walk(join(here, 'src')).concat(walk(join(here, 'videos')));

const tmp = mkdtempSync(join(tmpdir(), 'book-check-'));
// The page draws plots as SVG; the Cli needs to be told to (it has no raster plots).
writeFileSync(join(tmp, 'Settings.xml'), '<Settings><Math><Decimals>2</Decimals></Math><Plot><VectorGraphics>true</VectorGraphics></Plot></Settings>');
process.env.CALCPAD_SETTINGS = join(tmp, 'Settings.xml');
const text = (html) =>
    html.replace(/<(script|style)[\s\S]*?<\/\1>/g, '').replace(/<sup>([\s\S]*?)<\/sup>/g, '^$1').replace(/<sub>([\s\S]*?)<\/sub>/g, '_$1').replace(/<svg[\s\S]*?<\/svg>/g, '[drawing]').replace(/<[^>]+>/g, ' ')
        .replace(/&nbsp;/g, ' ').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&').replace(/\s+/g, ' ').trim();

let n = 0, bad = 0, sheets = 0, cpds = 0, pys = 0;
for (const md of mds) {
    const src = readFileSync(md, 'utf8').split('\n');
    for (let i = 0; i < src.length; i++) {
        const m = /^(`{3,})(sheet python|sheet|cpd|python)\s*$/.exec(src[i]);
        if (!m) continue;
        const start = i + 1;
        let j = start;
        while (j < src.length && !src[j].startsWith(m[1])) j++;
        let body = src.slice(start, j).join('\n') + '\n';
        i = j;
        n++;
        const where = `${md.replace(here + '/', '')}:${start}`;
        let cpd = body;
        const problems = [];
        if (m[2] === 'sheet python') {
            // A sheet with Python fences: the sheet must compile, and each fence must be valid Python.
            // The engine run needs the Python runtime of the page, see check-python.mjs.
            sheets++;
            if (!/^\s*<!--sheet-->/.test(body)) body = '<!--sheet-->\n' + body;
            const c = JSON.parse(sheet.compile(body, false));
            for (const d of c.diagnostics) if (d.severity === 'error') problems.push(`sheet line ${d.line + 1}: ${d.message}`);
            for (const f of body.matchAll(/^```python\n([\s\S]*?)^```/gm)) {
                try {
                    execFileSync('python3', ['-I', '-c', 'import sys; compile(sys.stdin.read(), "block", "exec")'], { input: f[1], stdio: ['pipe', 'pipe', 'pipe'] });
                } catch (e) {
                    problems.push(`python syntax: ${String(e.stderr).trim().split('\n').slice(-3).join(' ')}`);
                }
            }
            if (problems.length) { bad++; console.log(`FAIL ${where}`); for (const p of problems) console.log(`     ${p}`); }
            continue;
        }
        if (m[2] === 'python') {
            // Python runs in the page (MicroPython); here only the syntax is checked.
            pys++;
            try {
                execFileSync('python3', ['-I', '-c', 'import sys; compile(sys.stdin.read(), "block", "exec")'], { input: body, stdio: ['pipe', 'pipe', 'pipe'] });
            } catch (e) {
                problems.push(`python syntax: ${String(e.stderr).trim().split('\n').slice(-3).join(' ')}`);
            }
            if (problems.length) { bad++; console.log(`FAIL ${where}`); for (const p of problems) console.log(`     ${p}`); }
            continue;
        }
        if (m[2] === 'sheet') {
            sheets++;
            if (!/^\s*<!--sheet-->/.test(body)) body = '<!--sheet-->\n' + body;
            const c = JSON.parse(sheet.compile(body, false));
            for (const d of c.diagnostics) if (d.severity === 'error') problems.push(`sheet line ${d.line + 1}: ${d.message}`);
            cpd = c.text;
        } else cpds++;
        const f = join(tmp, `ex${n}.cpd`);
        writeFileSync(f, cpd);
        const out = join(tmp, `ex${n}.html`);
        try {
            execFileSync(cli, [f, out, '-s'], { stdio: ['ignore', 'pipe', 'pipe'] });
            const html = readFileSync(out, 'utf8');
            for (const e of html.matchAll(/<(?:p|span|div)[^>]*class="err"[^>]*>([\s\S]*?)<\/(?:p|span|div)>/g)) problems.push(text(e[1]).slice(0, 200));
            if (show) console.log(`--- ${where}\n${text(html)}\n`);
        } catch (e) {
            problems.push(`engine failed: ${e.message.split('\n')[0]}`);
        }
        if (problems.length) {
            bad++;
            console.log(`FAIL ${where}`);
            for (const p of problems) console.log(`     ${p}`);
        }
    }
}
console.log(`${n} examples (${sheets} sheet, ${cpds} cpd, ${pys} python syntax only): ${n - bad} ok, ${bad} failed`);
process.exit(bad ? 1 : 0);
