// The quick reference page, public/reference/index.html (/calcs/reference/): the sheet language on one printable sheet,
// with the functions, constants, operators and units of the engine. The line kinds are written here (docs/sheet-syntax.md
// is the full account); the function and unit lists come from the engine's own snippet table
// (calcpad-highlighter/src/snippets/snippets.json), so they cannot drift from what the Functions palette and the MCP
// reference offer. Written by make_learn_pages.mjs (and so checked by its --check); web/e2e/seo-check.mjs checks its head,
// breadcrumb and links, and web/e2e/reference-check.mjs checks that every example line compiles and runs.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { root } from './engine.mjs';
import { SITE, BASE, REPO, esc, head, header, crumbs, footer, breadcrumbs, SOFTWARE } from './render.mjs';

export const REFERENCE = `${SITE}/calcs/reference/`;
export const REFERENCE_TITLE = 'Engineering calculation sheet quick reference';
export const REFERENCE_DESCRIPTION = 'One printable page of the CivilKit Calcs sheet language: line kinds, inputs, plots, every function, constant and unit. Free, and it prints to PDF.';

const snippets = JSON.parse(readFileSync(join(root, 'calcpad-highlighter', 'src', 'snippets', 'snippets.json'), 'utf8'));

/** The example lines of the syntax tables: [what it is, what you write, what you get]. */
export const SYNTAX = [
    ['Lines of a sheet', [
        ['Heading', '# Beam check', 'A heading, level 1 to 6 with more #.'],
        ['Prose', 'Plain text, **bold**, *italic*, lists and tables.', 'Markdown. A line that reads as a calculation but is not one can start with a backslash.'],
        ['Formula in text', 'The moment is $M = wL^2/8$.', 'A formula typeset in the text.'],
        ['Quantity', 'L: 6 m | span', 'A name with a value and a unit; the text after | is a note.'],
        ['Calculation', 'M: w * L^2 / 8 -> kN*m', 'The working shown with the values put in, converted to kN·m.'],
        ['Several on a line', 'a: 1; b: 2 | note for b', 'Two quantities; the note belongs to the last.'],
        ['Your own function', 'f(x, y): sqrt(x^2 + y^2)', 'Used as f(3, 4) in later lines and in plots.'],
        ['A unit of your own', '#unit bag = 25 kg', 'Then 3 bag, or -> bag.'],
        ['Vector and matrix', 'v: [1, 2, 3]\nA: [1, 2; 3, 4]', 'Commas across, semicolons between rows. v.2 and A.(1, 2) pick an element.'],
        ['Condition', 'if u <= 1: "OK" else: "Fails"', 'A block with a body uses an indented body after if, else, for, while or repeat.'],
        ['Loop', 'for i in 1..5:\n    P: 10 kN * i', 'for x in v, for i in 0..10 step 2, while, repeat n, break and continue also work.'],
        ['Solve', 'x0: solve f(x) = 4 for x in 0..5', 'Root finding on a range.'],
        ['Integral and slope', 'A: integral of f(x) for x in 0..3\nk: slope of f(x) at x = 3', 'Numerical integral and derivative.'],
        ['Plot', 'plot sin(x) as "sine", cos(x) for x in 0..6', 'Curves on one plot; as "name" adds a legend entry.'],
        ['Page break', '<div style="page-break-after:always"></div>', 'Starts a new page in print, PDF and Word.'],
        ['Raw engine line', '>> #deg', 'A line passed to the Calcpad engine as written.'],
    ]],
    ['Inputs (a question mark makes the value a form field)', [
        ['Number', 'L: ? 6 m | span', 'A box with its unit; the value is the default.'],
        ['Slider', 'w: ? 12 kN/m [2 .. 30 step 0.5] | load', 'A box with a slider between the bounds.'],
        ['Drop-down', 'E: ? [Steel = 200 GPa, *Timber = 11 GPa]', 'The name takes the value of the chosen option; * marks the default.'],
        ['Radio buttons', 'k: ? (Low = 1, High = 2)', 'The same with round brackets.'],
        ['Checkbox', 'ribs: ? on', 'on is 1 and off is 0.'],
    ]],
];

/** Calls a function entry as a signature: sin(x), root(x; n). */
function signature(e) {
    const params = (e.Parameters ?? []).map((p) => p.Name);
    const insert = String(e.Insert ?? '');
    const label = String(e.Label ?? '').trim();
    let s = label && /\(/.test(label) ? label : insert;
    if (s.includes('§')) {
        if (params.length) {
            let k = 0;
            s = (s.match(/§/g) ?? []).length === 1 ? s.replace('§', params.join('; ')) : s.replace(/§/g, () => params[k++] ?? '…');
        } else {
            s = s.replace(/§/g, '…');
        }
    }
    return s.trim();
}

/** Groups a kind's entries by the part of their category after `strip`, in table order: Map(category -> entries). */
function byCategory(kind, strip) {
    const groups = new Map();
    for (const e of snippets[kind] ?? []) {
        const c = String(e.Category ?? kind).replace(strip, '') || kind;
        if (!groups.has(c)) groups.set(c, []);
        groups.get(c).push(e);
    }
    return groups;
}

const table = (rows, head) => `<table class="ref">
<thead><tr>${head.map((h) => `<th>${esc(h)}</th>`).join('')}</tr></thead>
<tbody>
${rows.map((r) => `<tr>${r.map((c, i) => (i === 1 ? `<td><code>${esc(c).replace(/\n/g, '<br>')}</code></td>` : `<td>${esc(c)}</td>`)).join('')}</tr>`).join('\n')}
</tbody>
</table>`;

const syntaxSection = () => SYNTAX.map(([title, rows]) => `<section>\n<h2>${esc(title)}</h2>\n${table(rows, ['Line', 'You write', 'You get'])}\n</section>`).join('\n');

const names = (list) => `<p class="names">${list.map((e) => `<code>${esc(signature(e))}</code>`).join(' ')}</p>`;

const functionsSection = () => {
    const parts = [];
    for (const [kind, strip] of [['Function', /^Functions\//], ['VectorFunction', /^Functions\/Vector\//], ['MatrixFunction', /^Functions\/Matrix\//]]) {
        const heading = kind === 'Function' ? 'Functions of numbers' : kind === 'VectorFunction' ? 'Vector functions' : 'Matrix functions';
        parts.push(`<h3>${heading}</h3>`);
        for (const [cat, list] of byCategory(kind, strip)) parts.push(`<h4>${esc(cat)}</h4>\n${names(list)}`);
    }
    return `<section class="wide">\n<h2>Functions</h2>\n<p>Arguments are separated by commas in a sheet (the engine's semicolon also works inside brackets). A function written with a unit-bearing argument returns its result with the right unit.</p>\n${parts.join('\n')}\n</section>`;
};

const constantsSection = () => `<section>
<h2>Constants and operators</h2>
${table((snippets.Constant ?? []).map((e) => [String(e.Description ?? ''), String(e.Insert)]), ['Constant', 'Symbol'])}
${table((snippets.Operator ?? []).map((e) => [String(e.Description ?? ''), String(e.Insert)]), ['Operator', 'Symbol'])}
<p>In a sheet: <code>+ - * / ^ %</code>, <code>&lt; &gt; &lt;= &gt;= == !=</code>, <code>and or</code>, and the forms <code>≤ ≥ ≠ ∧ ∨ × · ÷</code>. <code>²</code> and <code>³</code> after a name or unit mean <code>^2</code> and <code>^3</code>.</p>
</section>`;

const unitsSection = () => {
    const parts = [];
    for (const [cat, list] of byCategory('Unit', /^Units\//)) {
        parts.push(`<h4>${esc(cat.replace(/\//g, ': '))}</h4>\n<p class="names">${list.map((e) => `<code title="${esc(e.Description ?? '')}">${esc(e.Insert)}</code>`).join(' ')}</p>`);
    }
    return `<section class="wide">
<h2>Units</h2>
<p>Write a unit after a number (<code>300 MPa</code>, <code>9.81 m/s²</code>, <code>5 kN*m</code>). The engine combines and checks them, so a unit error is a mistake caught. Prefixes (k, M, G, m, µ, n and the rest of the SI set) apply to the metric units. Convert a result with <code>-> unit</code>, show a ratio as a percentage with <code>-> %</code>, and show a result in SI base units with <code>-> base</code>. Point at a unit for its meaning.</p>
${parts.join('\n')}
</section>`;
};

export function renderReferencePage() {
    const ld = {
        '@context': 'https://schema.org',
        '@graph': [
            breadcrumbs([['Quick reference', REFERENCE]], []),
            { '@type': 'WebPage', name: REFERENCE_TITLE, url: REFERENCE, description: REFERENCE_DESCRIPTION, isPartOf: { '@type': 'WebSite', name: 'CivilKit', url: `${SITE}/` }, about: SOFTWARE },
        ],
    };
    const STYLE = `<style>
.ref { width: 100%; border-collapse: collapse; font-size: 0.9rem; margin: 0.5rem 0 1rem; }
.ref th, .ref td { text-align: left; vertical-align: top; padding: 0.3rem 0.5rem; border-bottom: 1px solid var(--line); }
.ref code { white-space: pre-wrap; }
.names { line-height: 1.9; }
.names code { margin-right: 0.35rem; white-space: nowrap; }
h4 { margin: 1rem 0 0.2rem; font-size: 0.95rem; color: var(--dim); }
.print { margin: 0.5rem 0 1rem; }
.print button { font: inherit; padding: 0.5rem 1rem; border-radius: 6px; border: 2px solid var(--accent); background: transparent; color: var(--accent); cursor: pointer; }
@media print {
  body { font-size: 9pt; }
  main { max-width: none; padding: 0; }
  .print, .lead, .skip, header.site, footer.site, .crumbs { display: none; }
  section { break-inside: auto; }
  h2, h3, h4 { break-after: avoid; }
  .ref tr { break-inside: avoid; }
  .ref { font-size: 8.5pt; }
  .names { line-height: 1.55; columns: 2; }
}
</style>`;
    return `${head({ title: REFERENCE_TITLE, description: REFERENCE_DESCRIPTION, url: REFERENCE, ld }).replace('</head>', `${STYLE}\n</head>`)}
<body>
${header()}
<main id="main">
${crumbs([['Quick reference']])}
<h1>Sheet language quick reference</h1>
<div class="lead">
<p>Every kind of line a CivilKit Calcs sheet takes, its inputs, and the functions, constants, operators and units of the engine, on one page that prints well. Use <strong>Print</strong> and choose Save as PDF to keep a copy. The full account is in the <a href="/calcs/guide/">user guide</a>.</p>
</div>
<p class="print"><button type="button" id="print-ref" hidden>Print this page</button> <a class="button" href="/calcs/">Open Calcs</a></p>
${syntaxSection()}
${constantsSection()}
${functionsSection()}
${unitsSection()}
<section>
<h2>More</h2>
<ul>
<li><a href="/calcs/guide/">The user guide</a>, with a chapter for each of these.</li>
<li><a href="${BASE}">Calculators and guides</a>: ready-made templates that open as forms.</li>
<li><a href="${REPO}">Source on GitHub</a> (MIT licence).</li>
</ul>
</section>
</main>
${footer()}
<script>
var b = document.getElementById('print-ref');
if (b && window.print) { b.hidden = false; b.addEventListener('click', function () { window.print(); }); }
</script>
</body>
</html>
`;
}
