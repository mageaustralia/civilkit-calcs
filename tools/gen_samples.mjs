// Builds the Samples browser's content from CalcpadCE's Examples (MIT): copies the chosen
// worksheets and every file they use (images, included sheets, Excel data) into
// web/calcpad-web/public/samples, keeping upstream's folder layout, and writes
// public/samples/manifest.json and public/samples/NOTICE.md.
//
//   node tools/gen_samples.mjs [--ref ../calcpad-ref]      (after tools/build_wasm.sh)
//
// Each worksheet is rendered once with the calcpad-wasm engine (the browser build's own
// engine, run here in Node). One that reports any error is left out of the manifest and
// listed in the output.
// The browser opens a sheet as the desktop app does, converting C-style operators ("%%").
import { replaceCStyleOperators } from '../web/calcpad-web/src/wasm/c-style.ts';
import { convertLegacyInputs } from '../web/calcpad-web/src/wasm/legacy-input.ts';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, posix, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const argRef = process.argv.indexOf('--ref');
const ref = resolve(argRef > 0 ? process.argv[argRef + 1] : join(repo, '..', 'calcpad-ref'));
const examples = join(ref, 'Examples');
const out = join(repo, 'web', 'calcpad-web', 'public', 'samples');

// Structural: everything. Maths & plotting: the Engineering folders a structural engineer uses.
const MATHS = ['Getting Started', 'Matrices', 'Numerical', 'Interpolation', 'Finite Elements', 'Elastic Analysis', 'Custom Units', 'Multiline Functions'];
const GROUPS = {
    Cantilevers: 'Beams', 'Continuous Beam': 'Beams', Deflections: 'Beams', 'Fully Restrained Beams': 'Beams',
    'Propped Cantilevers': 'Beams', 'Simply Supported Beams': 'Beams',
    'Reinforced Concrete': 'Reinforced Concrete', 'Reinforced Concrete - Plates': 'Reinforced Concrete',
    'Reinforced Concrete - Punching': 'Reinforced Concrete', 'Reinforced Concrete - Seismic Detailing': 'Reinforced Concrete',
    Steel: 'Steel', Loads: 'Loads', Frames: 'Frames', 'Structural Dynamics': 'Dynamics', Nonlinear: 'Nonlinear',
    'Soil Mechanics': 'Soil Mechanics',
};
// Display titles. Upstream's file names repeat across folders ("Concentrated Force" in four
// beam folders), carry typos ("Unifrom") or say little ("Frame"); the copies keep upstream's
// names and paths, only the title shown changes.
const SUPPORT_PREFIX = {
    Cantilevers: 'Cantilever', 'Fully Restrained Beams': 'Fully Restrained Beam',
    'Propped Cantilevers': 'Propped Cantilever', 'Simply Supported Beams': 'Simply Supported Beam',
    'Reinforced Concrete - Plates': 'Two-Way Slab Panel', 'Reinforced Concrete - Punching': 'Punching Shear',
};
const TITLES = {
    'Structural/Simply Supported Beams/Concentrated Forces.cpd': 'Simply Supported Beam: Several Concentrated Forces',
    'Structural/Continuous Beam/Continuous Beam.cpd': 'Continuous Beam: Force Method under Uniform Load',
    'Structural/Deflections/Cantilever with Concentrated Load.cpd': 'Deflection: Cantilever with a Tip Load',
    'Structural/Deflections/Cantilever with Unifrom Load.cpd': 'Deflection: Cantilever under Uniform Load',
    'Structural/Deflections/Simply Supported Beam with Concentrated Load.cpd': 'Deflection: Simply Supported Beam with a Mid-Span Load',
    'Structural/Deflections/Simply Supported Beam with Unifrom Load.cpd': 'Deflection: Simply Supported Beam under Uniform Load',
    'Structural/Reinforced Concrete - Plates/Fully Restrained Slab Panel.cpd': 'Two-Way Slab Panel: All Edges Restrained',
    'Structural/Reinforced Concrete - Plates/Simply Supported Slab Panel.cpd': 'Two-Way Slab Panel: All Edges Simply Supported',
    'Structural/Reinforced Concrete - Seismic Detailing/Beam Detailing for DCM.cpd': 'Seismic Detailing (DCM): Beam',
    'Structural/Reinforced Concrete - Seismic Detailing/Column Detailing for DCM.cpd': 'Seismic Detailing (DCM): Column',
    'Structural/Reinforced Concrete - Seismic Detailing/Shear Wall Detailing for DCM.cpd': 'Seismic Detailing (DCM): Shear Wall',
    'Structural/Steel/I-Section Properties and Drawing.cpd': 'I-Section Properties from a Catalogue, with Drawing',
    'Structural/Steel/I-Section Properties Excel.cpd': 'I-Section Properties from an Excel Catalogue',
    'Structural/Frames/Frame.cpd': 'Plane Frame Analysis: Prismatic Members',
    'Structural/Frames/Frame-Var.cpd': 'Plane Frame Analysis: Tapered Members',
    'Structural/Frames/Multi-Story Frame.cpd': 'Multi-Storey RC Frame Analysis',
    'Structural/Structural Dynamics/Beam Impact Analysis Animated.cpd': 'Drop-Weight Impact on an RC Beam (Animated)',
    'Structural/Structural Dynamics/Free Vibrations of Steel Pole.cpd': 'Free Vibrations of a Tapered Steel Pole',
    'Structural/Structural Dynamics/Free Vibrations of Steel Pole Animated.cpd': 'Free Vibrations of a Tapered Steel Pole (SVG Animation)',
    'Structural/Structural Dynamics/Free Vibrations of Steel Pole Plotly Animated.cpd': 'Free Vibrations of a Tapered Steel Pole (Plotly Animation)',
    'Structural/Nonlinear/Nonlinear Analysis of a Von-Mises Truss Animated.cpd': 'Von Mises Truss: Nonlinear Analysis (Animated)',
    'Structural/Nonlinear/Material and Geometric Nonlinear Analysis of Prestressed Asymmetric Biot Truss.cpd': 'Prestressed Biot Truss: Material and Geometric Nonlinearity',
    'Structural/Loads/Optimization of Foundation Pad Dimensions.cpd': 'Pad Footing: Optimised Plan Dimensions',
    'Structural/Loads/Snow Drift.cpd': 'Snow Drift at a Taller Building (EN 1991-1-3)',
    'Structural/Loads/Snow Load.cpd': 'Snow Load on Monopitch and Duopitch Roofs (EN 1991-1-3)',
    'Structural/Loads/Wind Load.cpd': 'Wind Load on a Duopitch Building (EN 1991-1-4)',
    'Engineering/Getting Started/Calculating.cpd': 'Getting Started: Calculating',
    'Engineering/Getting Started/Functions.cpd': 'Getting Started: Functions and Plots',
    'Engineering/Getting Started/Unit Conversion.cpd': 'Getting Started: Units and Conversion',
    'Engineering/Elastic Analysis/Deep Beam.cpd': 'Deep Beam: Elastic Series Solution',
    'Engineering/Elastic Analysis/Rectangular Slab.cpd': 'Rectangular Plate: Navier Solution',
    'Engineering/Interpolation/Interpolation.cpd': 'Interpolation with take, line and spline',
    'Engineering/Numerical/Newton\'s Method Animated.cpd': "Newton's Method (Animated)",
    'Engineering/Finite Elements/Deep Beam FEA.cpd': 'Deep Beam: Finite Element Analysis',
};
const DESCRIPTIONS = {
    'Engineering/Getting Started/Calculating.cpd': 'First steps: arithmetic, comparisons and calculations with variables.',
};

/** $...$ maths in a description as readable text: subscripts and common symbols, no TeX. */
function plainMath(tex) {
    const SYMBOLS = {
        varphi: 'φ', phi: 'φ', pi: 'π', mu: 'μ', tau: 'τ', sigma: 'σ', alpha: 'α', beta: 'β', gamma: 'γ', delta: 'δ',
        Delta: 'Δ', lambda: 'λ', theta: 'θ', varepsilon: 'ε', epsilon: 'ε', eta: 'η', rho: 'ρ', omega: 'ω', xi: 'ξ',
        nu: 'ν', kappa: 'κ', psi: 'ψ', chi: 'χ', zeta: 'ζ', times: '×', cdot: '·', sqrt: '√', int: '∫', sum: 'Σ',
        le: '≤', ge: '≥', leq: '≤', geq: '≥', approx: '≈', infty: '∞', pm: '±', partial: '∂', ell: 'l',
    };
    return tex
        .replace(/\\(?:,|;|!|quad|qquad| )/g, ' ')
        .replace(/\\(?:mathrm|text|mathbf|mathit|operatorname)\{([^}]*)\}/g, '$1')
        .replace(/\\frac\{([^}]*)\}\{([^}]*)\}/g, '($1)/($2)')
        .replace(/\\([A-Za-z]+)/g, (m, w) => SYMBOLS[w] ?? w)
        .replace(/_\{([^}]*)\}/g, '_$1').replace(/\^\{([^}]*)\}/g, '^$1')
        .replace(/[{}]/g, '');
}

const GROUP_ORDER = ['Beams', 'Reinforced Concrete', 'Steel', 'Loads', 'Frames', 'Dynamics', 'Nonlinear', 'Soil Mechanics', 'Sections', 'Maths & plotting', 'Python'];

const cpdFiles = (dir) => readdirSync(dir).filter((f) => f.endsWith('.cpd')).map((f) => join(dir, f));
const folders = [
    ...readdirSync(join(examples, 'Structural')).filter((d) => statSync(join(examples, 'Structural', d)).isDirectory()).map((d) => ['Structural', d]),
    ...MATHS.map((d) => ['Engineering', d]),
];

/** Design codes cited in the text, normalised ("BS EN 1992-1-1" and "EN 1992-1-1" are one tag). */
function designCodes(text) {
    const found = new Set();
    for (const m of text.matchAll(/\b(?:BS |DIN |prEN )?EN ?(199[0-9])((?:-[0-9]+){0,2})/g)) found.add(`EN ${m[1]}${m[2]}`);
    for (const m of text.matchAll(/\bEurocode ?([0-9])\b/g)) found.add(`EN 199${m[1]}`);
    for (const [re, fmt] of [
        [/\bACI ?(318|350|440)\b/g, (m) => `ACI ${m[1]}`], [/\bAISC ?(360|341)\b/g, (m) => `AISC ${m[1]}`],
        [/\bASCE ?(7)\b/g, (m) => `ASCE ${m[1]}`],
        [/\bNZS ?(3101|3404|1170)\b/g, (m) => `NZS ${m[1]}`], [/\bBS ?(8110|5950|6399)\b/g, (m) => `BS ${m[1]}`],
        [/\bIS ?(456|800|875|1893)\b/g, (m) => `IS ${m[1]}`],
    ]) for (const m of text.matchAll(re)) found.add(fmt(m));
    // A part citation implies its document ("EN 1992-1-1" also tags "EN 1992").
    for (const c of [...found]) { const doc = /^EN \d{4}/.exec(c); if (doc) found.add(doc[0]); }
    return [...found].sort();
}

/** The leading comment of a worksheet, as one line of plain text. */
function description(text) {
    const m = /^'\s*<!--([\s\S]*?)-->/m.exec(text);
    if (!m) return '';
    return m[1].split('\n').map((l) => l.replace(/^'/, '')).join(' ')
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1').replace(/<[^>]+>/g, '')
        .replace(/\$([^$]+)\$/g, (x, tex) => plainMath(tex))
        .replace(/`/g, '').replace(/\*\*?([^*]+)\*\*?/g, '$1').replace(/\s+/g, ' ').trim();
}

/** Files a worksheet uses, as written in it: images, included sheets, #read sources. */
function references(text) {
    const refs = new Set();
    for (const m of text.matchAll(/\bsrc\s*=\s*["']([^"']+)["']/g)) refs.add(m[1]);
    for (const m of text.matchAll(/\]\(([^)\s]+\.(?:png|jpe?g|gif|svg|webp))\)/gi)) refs.add(m[1]);
    for (const m of text.matchAll(/^#include\s+(.+?)\s*$/gm)) refs.add(m[1]);
    for (const m of text.matchAll(/^#read\s+\S+\s+from\s+(.+?\.\w+)(?:[@!:\s].*)?$/gmi)) refs.add(m[1]);
    return [...refs].filter((r) => !/^[a-z]+:/i.test(r) && !r.startsWith('//') && !r.startsWith('#'));
}

/** The library path of a reference made from an untitled sheet: rooted at "/", ".." stops there. */
function libraryKey(ref) {
    const parts = [];
    for (const p of ref.replace(/\\/g, '/').split('/')) {
        if (p === '' || p === '.') continue;
        if (p === '..') parts.pop(); else parts.push(p);
    }
    return '/' + parts.join('/');
}

// ---- the engine, for the render check ----------------------------------------------------
const wasmDir = join(repo, 'web', 'calcpad-web', 'public', 'wasm');
const engine = await import(join(wasmDir, 'calcpad_wasm.js'));
engine.initSync({ module: readFileSync(join(wasmDir, 'calcpad_wasm_bg.wasm')) });

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
const copied = new Set();
// Worksheets whose deprecated `name = ? {default}` fields were rewritten as #UI declarations
// (src/wasm/legacy-input.ts): absolute path -> new text. Only after the render check below
// found the result unchanged.
const converted = new Map();
const copy = (abs) => {
    const rel = relative(examples, abs);
    if (copied.has(rel)) return posix.join('samples', rel.split('\\').join('/'));
    mkdirSync(dirname(join(out, rel)), { recursive: true });
    if (converted.has(abs)) writeFileSync(join(out, rel), converted.get(abs));
    else copyFileSync(abs, join(out, rel));
    copied.add(rel);
    return posix.join('samples', rel.split('\\').join('/'));
};

// ---- the #UI conversion's bookkeeping and equivalence test -------------------------------
const legacy = { converted: [], kept: [], partial: [] };
/** The page's content with the input widget markup dropped: <u class="input-N">v</u> is v (fields left alone still have it). */
function renderedBody(html) {
    let b = html.slice(html.lastIndexOf('<body'));
    b = b.replace(/<u class="input-\d+">([^<]*)<\/u>/g, '$1');
    return b.replace(/\s+/g, ' ').replace(/ (?=<\/)/g, '');
}

/** The source lines whose rendered paragraphs differ between two pages. */
function differingLines(a, b) {
    const byLine = (html) => {
        const m = new Map();
        for (const part of html.split(/(?=<p id="line-\d+" data-source-line="\d+")/)) {
            const n = /^<p id="line-\d+" data-source-line="(\d+)"/.exec(part);
            if (n) m.set(Number(n[1]), (m.get(Number(n[1])) ?? '') + part);
        }
        return m;
    };
    const x = byLine(a);
    const y = byLine(b);
    return [...new Set([...x.keys(), ...y.keys()])].filter((l) => x.get(l) !== y.get(l));
}

/** Where two rendered pages first differ, for the report. */
function firstDifference(a, b) {
    let i = 0;
    while (i < a.length && i < b.length && a[i] === b[i]) i++;
    return `...${a.slice(Math.max(0, i - 40), i + 80)} <> ${b.slice(Math.max(0, i - 40), i + 80)}`;
}

const samples = [];
const excluded = [];
const notice = [];
for (const [top, folder] of folders) {
    const dir = join(examples, top, folder);
    const files = cpdFiles(dir);
    // A sheet another sheet in its folder includes is a library (svg_drawing.cpd), not a sample.
    const included = new Set(files.flatMap((f) => [...readFileSync(f, 'utf8').matchAll(/^#include\s+(.+?)\s*$/gm)].map((m) => resolve(dir, m[1]))));
    for (const file of files.sort()) {
        if (included.has(file)) continue;
        const text = readFileSync(file, 'utf8').replace(/^\uFEFF/, '');
        const rel = relative(examples, file).split('\\').join('/');
        const title = TITLES[rel] ?? (SUPPORT_PREFIX[folder] ? `${SUPPORT_PREFIX[folder]}: ${basename(file, '.cpd')}` : basename(file, '.cpd'));
        // Assets, following included sheets for their own references.
        const assets = new Map();
        const missing = [];
        const queue = [[text, dir]];
        while (queue.length) {
            const [t, d] = queue.pop();
            for (const r of references(t)) {
                const abs = resolve(d, r);
                const key = libraryKey(r);
                if (assets.has(key)) continue;
                if (!existsSync(abs) || !statSync(abs).isFile()) { missing.push(r); continue; }
                assets.set(key, abs);
                if (abs.endsWith('.cpd')) queue.push([readFileSync(abs, 'utf8'), dirname(abs)]);
            }
        }
        // Render check: the engine with the sheet's files, as the browser has them.
        for (const [key, abs] of assets) engine.set_file_bytes(key, readFileSync(abs));
        const res = JSON.parse(engine.convert(JSON.stringify({ content: replaceCStyleOperators(text) }), false));
        engine.clear_files();
        const errors = res.errors ?? [];
        if (res.status || errors.length) {
            excluded.push({ file: rel, reason: res.error ?? `${errors.length} error(s), first: line ${errors[0].sourceLine}: ${errors[0].message}` });
            continue;
        }
        // Deprecated `name = ? {default}` fields (linter CPD-3419) become #UI declarations. The
        // converted sheet (and the sheets it includes) must render exactly as the original, with
        // only the input widget markup dropped; otherwise the sheet is kept as upstream wrote it.
        let uiText = text;
        {
            const plan = new Map();
            let fields = 0;
            let skipped = 0;
            // A converted line goes back to what upstream wrote when the engine rejects it (a #UI
            // line that also assigns an expression: "#UI does not support expressions") or when
            // its rendered paragraph differs (an input shows the digits as typed, "0.60", where a
            // value is shown as the number, "0.6"). The sheet is then converted again without it.
            const keep = new Set();
            let r;
            let after;
            for (let round = 0; round < 30; round++) {
                r = convertLegacyInputs(readFileSync(file, 'utf8'), keep);
                if (!r.converted) { after = null; break; }
                for (const [key, abs] of assets) engine.set_file_bytes(key, readFileSync(abs));
                after = JSON.parse(engine.convert(JSON.stringify({ content: replaceCStyleOperators(r.text.replace(/^\uFEFF/, '')) }), false));
                engine.clear_files();
                const errorLines = (after.errors ?? []).filter((e) => e.sourceLine > 0).map((e) => e.sourceLine);
                const bad = after.status ? [] : errorLines.length ? errorLines : differingLines(renderedBody(res.html), renderedBody(after.html));
                const fresh = bad.filter((l) => !keep.has(l));
                if (!fresh.length) break;
                for (const l of fresh) keep.add(l);
            }
            skipped += r.skipped;
            if (r.converted) { plan.set(file, r.text); fields += r.converted; }
            for (const abs of [...assets.values()].filter((a) => a.endsWith('.cpd'))) {
                const ri = convertLegacyInputs(readFileSync(abs, 'utf8'));
                skipped += ri.skipped;
                if (ri.converted) { plan.set(abs, ri.text); fields += ri.converted; }
            }
            if (fields) {
                const bytes = (abs) => plan.has(abs) ? Buffer.from(plan.get(abs), 'utf8') : readFileSync(abs);
                for (const [key, abs] of assets) engine.set_file_bytes(key, bytes(abs));
                const convertedMain = (plan.get(file) ?? readFileSync(file, 'utf8')).replace(/^\uFEFF/, '');
                const after = JSON.parse(engine.convert(JSON.stringify({ content: replaceCStyleOperators(convertedMain) }), false));
                engine.clear_files();
                const same = !after.status && !(after.errors ?? []).length && renderedBody(res.html) === renderedBody(after.html);
                if (same) {
                    for (const [abs, t] of plan) converted.set(abs, t);
                    uiText = convertedMain;
                    legacy.converted.push({ file: rel, fields, skipped });
                } else {
                    legacy.kept.push({ file: rel, fields, reason: after.status || (after.errors ?? []).length ? `the converted sheet reports errors: ${after.error ?? after.errors[0].message}` : `the rendered result differs: ${firstDifference(renderedBody(res.html), renderedBody(after.html))}` });
                }
            } else if (skipped) {
                legacy.partial.push({ file: rel, skipped });
            }
        }
        const id = rel.replace(/\.cpd$/, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
        samples.push({
            id,
            title,
            group: top === 'Engineering' ? 'Maths & plotting' : (GROUPS[folder] ?? folder),
            folder,
            file: copy(file),
            upstream: `Examples/${rel}`,
            description: DESCRIPTIONS[rel] ?? description(text),
            codes: designCodes(text),
            ui: /^#UI\b/m.test(uiText),
            scripts: /<script\b/i.test(text),
            read: /^#read\b/mi.test(text),
            assets: [...assets].map(([key, abs]) => ({ key, url: copy(abs) })),
            ...(missing.length ? { missingAssets: missing } : {}),
        });
        notice.push(`- \`Examples/${rel}\``);
    }
}
// Worksheets written for this app (tools/own_samples/<group>/), not copied from upstream. A sheet with a #python block
// cannot be rendered by the engine alone (the page runs the block first), so it is not render-checked here;
// e2e/python-check.mjs runs every one of them in the browser.
const OWN = join(repo, 'tools', 'own_samples');
const OWN_TITLES = {
    'Beam explorer with Plotly': ['Beam Explorer: Plotly from Python', 'A three-span beam solved in Python (stiffness matrix, numpy) for several load cases and a moving point load; the same block draws the results with the Plotly API (go.Figure, make_subplots) and drives them with Plotly\'s own menu and sliders, with no HTML or JavaScript, and a Calcpad design check follows.'],
    'Continuous beam explorer': ['Beam Explorer: Python Results in Raw JS Plotly', 'The same three-span beam, with the chart written as raw HTML and JavaScript that reads the exported results (a load case menu, sliders that recompute in the browser, hover values). Kept as the example of the raw script route; the Plotly API sample needs none of it.'],
    'Beam moment diagram with matplotlib': ['Beam Moment Diagram with matplotlib', 'A #python block draws the moment diagram of a simply supported beam with numpy and matplotlib and exports the maximum moment; the Calcpad lines after it check that moment against a capacity.'],
    'Plane truss by the stiffness method': ['Plane Truss by the Stiffness Method', 'A statically determinate plane truss is assembled and solved in a #python block (global stiffness matrix in loops, np.linalg.solve), the member forces and reactions are recovered and drawn with matplotlib, and the Calcpad lines after the block check the governing members.'],
    'Function plot with matplotlib': ['Function Plot with matplotlib', 'A #python block plots a damped oscillation and a bar chart side by side with matplotlib and exports the peak value.'],
    'Built-up section properties': ['Built-up Section Properties', 'Vectors and hprod give the centroid and second moment of area of a built-up section with units carried through, and a bending stress follows.'],
};
if (existsSync(OWN)) {
    for (const folder of readdirSync(OWN).filter((d) => statSync(join(OWN, d)).isDirectory()).sort()) {
        for (const [name, [title, desc]] of Object.entries(OWN_TITLES)) {
            const src = join(OWN, folder, `${name}.cpd`);
            if (!existsSync(src)) continue;
            const text = readFileSync(src, 'utf8');
            mkdirSync(join(out, folder), { recursive: true });
            writeFileSync(join(out, folder, `${name}.cpd`), text);
            samples.push({
                id: `${folder}-${name}`.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, ''),
                title, group: folder, folder, file: posix.join('samples', folder, `${name}.cpd`), upstream: `own/${folder}/${name}.cpd`,
                description: desc, codes: [], ui: false, scripts: /<script\b/i.test(text), read: false, python: /^#python\s*$/mi.test(text), assets: [],
            });
        }
    }
}
const seen = new Map();
for (const s of samples) {
    if (seen.has(s.title)) throw new Error(`Two samples have the title "${s.title}": ${seen.get(s.title)} and ${s.upstream}`);
    seen.set(s.title, s.upstream);
}
for (const rel of [...Object.keys(TITLES), ...Object.keys(DESCRIPTIONS)]) {
    if (!samples.some((s) => s.upstream === `Examples/${rel}`) && !excluded.some((e) => e.file === rel)) throw new Error(`Override for a sheet that is not a sample: ${rel}`);
}
samples.sort((a, b) => GROUP_ORDER.indexOf(a.group) - GROUP_ORDER.indexOf(b.group) || a.folder.localeCompare(b.folder) || a.title.localeCompare(b.title));
writeFileSync(join(out, 'manifest.json'), JSON.stringify({ groups: GROUP_ORDER.filter((g) => samples.some((s) => s.group === g)), samples }, null, 1) + '\n');
writeFileSync(join(out, 'NOTICE.md'), [
    '# Samples',
    '',
    'These worksheets, and the images, included sheets and data files they use, are copied',
    'from the CalcpadCE repository (https://github.com/imartincei/CalcpadCE, commit',
    '0cf438f, folder `Examples/`), MIT licence, Copyright (c) 2026 CalcpadCE Contributors.',
    'The copies keep upstream\'s folder layout under this folder.',
    '',
    'The copies differ from upstream in one way: a deprecated stored input field',
    '`name = ? {default}` (CalcpadCE\'s linter warns CPD-3419) is written as the `#UI`',
    'declaration its migration guide gives, `#UI name = default`, keeping the name, the value',
    'and the unit. tools/gen_samples.mjs does it (src/wasm/legacy-input.ts) and only keeps a',
    'converted sheet when the engine renders it exactly like the original, apart from the',
    'input widget markup (DEVIATIONS.md, D8). Worksheets:',
    '',
    ...notice,
    '',
].join('\n'));

const byGroup = {};
for (const s of samples) byGroup[s.group] = (byGroup[s.group] ?? 0) + 1;
console.log(`samples: ${samples.length}`, byGroup);
console.log(`files copied: ${copied.size}`);
console.log(`codes: ${[...new Set(samples.flatMap((s) => s.codes))].sort().join(', ')}`);
console.log(`excluded (${excluded.length}):`);
for (const e of excluded) console.log(`  ${e.file}: ${e.reason}`);
const missing = samples.filter((s) => s.missingAssets);
if (missing.length) for (const s of missing) console.log(`  missing assets in ${s.upstream}: ${s.missingAssets.join(', ')}`);

console.log(`#UI conversion: ${legacy.converted.length} sheets converted (${legacy.converted.reduce((n, c) => n + c.fields, 0)} fields), ${legacy.kept.length} kept as upstream wrote them, ${legacy.partial.length} with fields left alone`);
for (const k of legacy.kept) console.log(`  kept ${k.file}: ${k.reason}`);
for (const k of legacy.partial) console.log(`  fields left alone in ${k.file}: ${k.skipped}`);
for (const k of legacy.converted.filter((c) => c.skipped)) console.log(`  converted ${k.file}, fields left alone: ${k.skipped}`);
