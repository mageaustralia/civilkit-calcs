// The data of the ten landing pages (public/learn/<slug>/index.html). Prose is written here; every number that comes
// from a calculation is a [[run:name]] placeholder that make_learn_pages.mjs fills from a real engine run and
// web/e2e/learn-check.mjs re-checks. No design standards are named: these are generic textbook calculations.

const V = {
    first: { file: '01-first-calculation.mp4', title: 'Your first calculation', chapter: 'getting-started/first-sheet.html' },
    plots: { file: '03-plots-in-sheets.mp4', title: 'Plots in the sheet format', chapter: 'plots.html' },
    templates: { file: '05-templates-with-inputs.mp4', title: 'Templates with inputs', chapter: 'templates.html' },
    importing: { file: '06-import-a-worksheet.mp4', title: 'Import an existing worksheet', chapter: 'importing.html' },
    exporting: { file: '08-export-word-excel-pdf.mp4', title: 'Export a report to Word, Excel and PDF', chapter: 'exporting.html' },
};

/** A two-column table of "label, engine value" rows for one run. */
const tbl = (run, rows, head = ['Quantity', 'Result']) =>
    `<table>\n<thead><tr><th>${head[0]}</th><th>${head[1]}</th></tr></thead>\n<tbody>\n${rows.map(([l, k]) => `<tr><td>${l}</td><td>[[${run}:${k}]]</td></tr>`).join('\n')}\n</tbody>\n</table>`;

const notCode = (what) => `<div class="note">
<p><strong>This is a generic textbook calculation, not a design-code check.</strong> ${what} It does not apply load factors, resistance factors or any limit from a design standard. If you need a code check, use the formulas your standard requires, and treat this sheet as a way to see the mechanics and to check hand calculations.</p>
</div>`;

const taskSteps = (what) => [
    ['Open the template', 'Choose <strong>Open this template</strong> on this page. The sheet opens in a new tab of CivilKit Calcs, with the inputs set to the values used on this page.'],
    ['Switch to Input mode', 'In the results bar choose <strong>Input</strong>. The input lines become boxes, lists, buttons and ticks.'],
    ['Enter your values', `Change the inputs to your own. ${what} Every change recalculates the whole page.`],
    ['Read and export the result', 'Read the results and the OK or not OK lines under them. Use <strong>Report</strong>, then Print PDF, Word or Excel, to keep the calculation with its workings.'],
];

export const pages = [];

/* ------------------------------------------------------------------------------------------------ 1 */
pages.push({
    slug: 'mathcad-alternative',
    title: 'Free Mathcad alternative that runs in your browser',
    description: 'A free, open source engineering calculation sheet for the browser: real units, plots, Word and Excel export, offline use, and a Mathcad worksheet importer.',
    h1: 'A free Mathcad alternative that runs in your browser',
    schema: ['software'],
    video: V.first,
    related: ['mathcad-worksheet-converter', 'calcpad-online', 'excel-alternative-for-engineering-calculations', 'engineering-calculation-software-free'],
    runs: {
        m: { sheet: `# Beam moment

L: 6 m | span
w: 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum moment
` },
    },
    lead: `<p>If you search for a Mathcad alternative you probably want four things: formulas that read like maths, real units that are checked, a report you can hand to someone, and no licence to renew. CivilKit Calcs is a calculation sheet that does those four in a browser tab. It is free, open source (MIT) and it keeps working offline once it has loaded.</p>
<p>It is not a clone. It works differently from Mathcad in a few ways that matter, and this page says which, so you can decide before you spend an afternoon on it.</p>`,
    sections: [
        ['What a calculation looks like', `<p>You write the calculation as plain text. A quantity has a name, a value or formula, an optional unit to show it in, and a note after a bar:</p>
{{code:m}}
<p>The page shows the formulas typeset as maths, with your numbers substituted and the result in the unit you asked for. Here the moment comes out as [[m:M]], because the engine multiplies and divides the units rather than trusting a label. A length added to a time is refused with a message, not silently summed.</p>
<p>Text between the calculations is Markdown, and formulas in LaTeX are typeset too. There is a plot command, input forms (a number box, a drop-down, radio buttons, a checkbox), vectors and matrices, a function library for solving, interpolation and integration, small Python blocks, and a symbolic dialog for exact algebra.</p>`],
        ['What you get that Mathcad users usually ask for', `<ul>
<li><strong>Real units.</strong> Entered as <code>12 kN/m</code> or <code>300 MPa</code>, converted and combined by the engine.</li>
<li><strong>A report.</strong> Print to PDF, or export Word (with real Word equations) or Excel (with the workings and native charts).</li>
<li><strong>Offline.</strong> The page can be installed as an app and calculates without a connection.</li>
<li><strong>Privacy.</strong> The engine is compiled to WebAssembly and runs in the page. A calculation is not sent to a server.</li>
<li><strong>No cost, no account.</strong> The source is public and the licence is MIT.</li>
</ul>`],
        ['Where it is different', `<p>A worksheet here is a document that runs from top to bottom, like a page of text with calculations in it. Mathcad lets you place regions anywhere on a canvas. If you rely on that free layout, expect to re-flow the page. You also write formulas as text lines and see them typeset in the result, rather than building them in an equation editor, although there is an on-screen maths keyboard for touch screens.</p>
<p>The symbolic side is smaller. The symbolic dialog does exact rational algebra with documented limits: it simplifies, expands, factors, differentiates, integrates a small class of integrands and solves equations of degree 1 and 2, and it has no complex numbers. If your worksheets lean on a full computer algebra system, this is not a replacement for it.</p>
<p>The product is young. Check anything you will rely on against a hand calculation or a result you already trust, as you would with any new tool.</p>`],
        ['Bringing your worksheets across', `<p>The page can read Mathcad worksheet files (<code>.xmcd</code> and <code>.mcdx</code>) and write them out as a new sheet that its own engine then calculates. It translates; it does not evaluate. Text, definitions, evaluations, functions, units, vectors and matrices, sums, integrals, programs made of loops and conditions, simple x-y plots, solve blocks of one or two unknowns and pictures convert. Regions it cannot convert (add-in functions, embedded objects, symbolic evaluation, some plot types) are shown as a picture of the original and counted in a summary, so you see what needs rebuilding.</p>
<p>Read <a href="/calcs/learn/mathcad-worksheet-converter/">how the converter works and where it stops</a> before you trust a converted file. Plan on checking it against the source, line by line.</p>`],
        ['When Mathcad is still the better choice', `<p>Keep it if your worksheets call add-in libraries, embed other objects, depend on symbolic evaluation, or sit in a team library that a whole office edits with the same tool. The importer reports those as gaps, and no import will make them work. A good trial is to import one real worksheet, then compare every result with the original.</p>`],
    ],
    faq: [
        ['Is CivilKit Calcs really free?', 'Yes. It is open source under the MIT licence and free to use, with no account. The source is on GitHub.'],
        ['Can it open Mathcad worksheets?', 'It can import <code>.xmcd</code> and <code>.mcdx</code> files through File, then Import worksheet. Much converts and the rest is flagged. It is a translator, so check an imported worksheet against the original before relying on it.'],
        ['Will it give the same numbers as Mathcad?', 'For the same formulas and units it should agree to display precision, but that is something to verify on your own worksheets rather than assume. The importer copies every number from the source file and the engine recomputes the results.'],
        ['Does it work offline?', 'Yes. After the first visit the page can be installed and used with no connection. Interactive Plotly charts need their script loaded once while online.'],
        ['Is it affiliated with the makers of Mathcad?', 'No. Mathcad is a product of its owner and is named here only so that people searching for an alternative can find this page.'],
    ],
    steps: null,
});

/* ------------------------------------------------------------------------------------------------ 2 */
pages.push({
    slug: 'calcpad-online',
    title: 'Calcpad online: run Calcpad worksheets in a browser',
    description: 'CivilKit Calcs runs the Calcpad worksheet language in your browser on a Rust engine compiled to WebAssembly. Free, open source, installable and offline.',
    h1: 'Calcpad online: the Calcpad language in your browser',
    schema: ['software'],
    video: V.first,
    related: ['mathcad-alternative', 'excel-alternative-for-engineering-calculations', 'engineering-calculation-software-free', 'simply-supported-beam-calculator'],
    runs: {
        t: { cpd: `'Right triangle
a = 3m
b = 4m
c = sqrt(a^2 + b^2)
` },
    },
    lead: `<p>Calcpad is a calculator and report writer by Nedelcho Ganchovski (Proektsoft) in which you write equations with units and get a formatted document. Its community edition, CalcpadCE, is MIT licensed. CivilKit Calcs is a derivative of CalcpadCE: the same worksheet language, with the calculation engine rewritten in Rust and compiled to WebAssembly, so it runs in a browser tab with nothing to install.</p>
<p>It is not affiliated with the Calcpad authors. Their work is the foundation, and the credit and licence travel with the code.</p>`,
    sections: [
        ['Run a Calcpad worksheet in the browser', `<p>Open the app, paste a worksheet or use File, then Open, and the page calculates it. A worksheet like this one is ordinary Calcpad:</p>
{{code:t}}
<p>It gives [[t:c]] for the hypotenuse, with the units carried through. Everything the language offers is meant to work: units, vectors and matrices, <code>$Plot</code> and <code>$Map</code> plots, macros, <code>#if</code> and <code>#while</code> blocks, includes, input fields and the large function library.</p>`],
        ['What was rebuilt, and what was kept', `<ul>
<li><strong>The engine</strong> is a Rust port of the CalcpadCE calculation engine, compiled to WebAssembly. The page answers the calls the original made to its server itself, so no server is needed.</li>
<li><strong>The editor and report view</strong> are CalcpadCE's web frontend, with changes.</li>
<li><strong>Word and Excel export</strong> are built in the page. Equations become Word equations. The Excel workbook has the workings, variables and native charts.</li>
<li><strong>Installable and offline.</strong> A service worker caches the build so the page opens and calculates without a connection.</li>
</ul>`],
        ['How close to CalcpadCE is it?', `<p>The Rust engine is tested against CalcpadCE's own test classes, transcribed to Rust, and against the expected-output files of its worksheets. Where it deliberately behaves differently, the difference is listed one by one with the test that pins it, in a file of deviations in the repository. One example: when an <code>#if</code> condition cannot be evaluated, the block is left unresolved with an error message, instead of silently running the <code>#else</code> branch.</p>
<p>Not supported: raster plots from the original's graphics library, the project and library path roots, and compiled <code>.cpdz</code> worksheets. The project is early, so if you depend on an unusual corner of the language, run your worksheet and compare the output before you switch.</p>`],
        ['What it adds: the sheet format and templates', `<p>On top of the Calcpad language there is a lighter way to write, called the sheet format: Markdown prose, <code>name: expression -> unit | note</code> lines and LaTeX. A sheet compiles to Calcpad and runs on the same engine, and you can always see the Calcpad it becomes with Show compiled.</p>
<p>A <code>?</code> in front of a value makes it an input, so a sheet becomes a fill-in template. See the <a href="/calcs/learn/simply-supported-beam-calculator/">beam calculator</a> for a worked one.</p>`],
        ['Credit and licence', `<p>Calcpad was created by Nedelcho Ganchovski (Proektsoft); CalcpadCE is its MIT licensed continuation, at <a href="https://github.com/imartincei/CalcpadCE">github.com/imartincei/CalcpadCE</a>. CivilKit Calcs is MIT licensed too, and the upstream copyright notices are kept in the repository. If Calcpad is useful to you, look at the original project as well.</p>`],
    ],
    faq: [
        ['Is this the official Calcpad?', 'No. It is an independent derivative of CalcpadCE, the MIT licensed community edition, with the engine rewritten in Rust. It is not made or endorsed by the Calcpad authors.'],
        ['Do I have to install anything?', 'No. Open the page in a current browser. You can install it as an app from the browser if you want it on the desktop or a phone.'],
        ['Can I open my existing .cpd files?', 'Yes, with File then Open or by pasting the text. Check the result against what you expect, because the engine is a port and a few behaviours are deliberately different. The deviations file lists them.'],
        ['Does my worksheet leave my computer?', 'No. The engine runs in the page. There is no calculation server.'],
        ['Does it work offline?', 'Yes, after a first visit. Plotly charts, which load a script from a CDN, need to have been loaded once while online.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 3 */
pages.push({
    slug: 'excel-alternative-for-engineering-calculations',
    title: 'Excel alternative for engineering calculations',
    description: 'Write engineering calculations with units and readable formulas instead of cell references, see the workings, and export to Word, PDF and Excel.',
    h1: 'An Excel alternative for engineering calculations',
    schema: ['software'],
    video: V.exporting,
    related: ['mathcad-alternative', 'calcpad-online', 'simply-supported-beam-calculator', 'engineering-calculation-software-free'],
    runs: {
        s: { sheet: `# Bending stress

M: 54 kN*m | design moment
Z: 691.76 cm^3 | section modulus
sigma: M / Z -> MPa | bending stress
` },
    },
    lead: `<p>Spreadsheets are the default tool for a quick engineering calculation, and for good reason: they are everywhere and everyone can open one. They are also a poor place to show workings. The formula sits behind a cell, the units are in a header, and a reviewer has to click through references to see what was done.</p>
<p>CivilKit Calcs keeps the calculation as a readable document instead. This page compares the two honestly, including where a spreadsheet is still the better tool.</p>`,
    sections: [
        ['The same calculation, written out', `<p>In a spreadsheet, a bending stress is a number in a cell whose formula reads something like <code>=B4/B5</code>, with the unit conversion done by hand. In a sheet it is:</p>
{{code:s}}
<p>The report shows the formula, the numbers substituted and the result: [[s:sigma]]. The engine converted kN·m divided by cm³ into MPa because you asked for MPa. A reviewer reads the page top to bottom and does not need to open anything.</p>`],
        ['Where a calculation sheet is stronger', `<ul>
<li><strong>Units are part of the value.</strong> A spreadsheet stores a bare number, so a kN in one cell and a N in another add without complaint. Here the engine tracks units, converts them, and refuses to add a length to a time.</li>
<li><strong>Formulas are readable.</strong> Names such as <code>M</code> and <code>Z</code> instead of <code>B4</code>, and the typeset formula appears in the output.</li>
<li><strong>The workings are the output.</strong> Each result shows its formula and its substituted numbers, ready to print.</li>
<li><strong>Inputs are explicit.</strong> A template marks its inputs with a question mark and presents them as a form. The cells you may change are not mixed with the cells you must not.</li>
<li><strong>Branching reads like text.</strong> An <code>if</code> line prints a plain sentence such as "OK" or "choose a deeper section".</li>
</ul>`],
        ['Where a spreadsheet is still better', `<p>Use a spreadsheet for tables of many similar rows, quick what-if columns, data you will sort or filter, pivot tables, and anything shared with people who only have Excel. A calculation sheet is for a calculation with a story: inputs, steps and checks that you want to read and file. Many engineers use both.</p>`],
        ['Getting the results into Excel', `<p>Export gives you a workbook with the report in document order. Each equation is a row with its description, name, expression, the expression with numbers put in, value and unit. There are separate sheets for variables, for each vector or matrix, for the inputs and for plot data, and plots become native Excel charts that you can edit.</p>
<p>Be clear about what you get: the cells hold calculated values with the expressions as text. They are not live Excel formulas, so changing a cell in the workbook does not recalculate the rest. The workbook opens in LibreOffice and a Python reader. We have not tried it in Microsoft Excel, so check it opens as you expect.</p>
<p>If you want a document rather than a workbook, export Word, with real equations, or print the report to PDF.</p>`],
        ['Moving a spreadsheet calculation over', `<p>Take the cell that holds the answer, and work backwards. Write each input as a line with its unit, then each formula as a line using those names. Keep the same numbers and check the result against the spreadsheet. A good first target is a calculation you repeat often, because you can turn its inputs into a template and reuse it as a form.</p>`],
    ],
    faq: [
        ['Can it replace Excel?', 'Not as a general tool. It replaces the spreadsheets you use to show a single calculation with its workings. For large tables of data, sorting and pivots, Excel remains the right tool.'],
        ['Does the Excel export keep my formulas live?', 'No. The workbook holds calculated values and the expressions as text. It also has native charts. Do the recalculation in the sheet, not in the workbook.'],
        ['Does it check units?', 'Yes. Units are part of every quantity, so multiplication and division combine them, results convert to the unit you ask for, and adding incompatible units is an error.'],
        ['Can I import data from a spreadsheet?', 'A worksheet can read a matrix from a file kept in the browser library with the <code>#read</code> command; see the user guide. For a first calculation, typing the few input values is simpler.'],
        ['Does it need an internet connection?', 'No. After the first visit it works offline, and nothing you calculate is sent to a server.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 4 hub */
pages.push({
    slug: 'engineering-calculation-software-free',
    title: 'Free engineering calculation software in the browser',
    description: 'A free browser tool for engineering calculations with units: templates for beams, walls, footings, pipes and pumps, plots, and Word, Excel and PDF export.',
    h1: 'Free engineering calculation software in your browser',
    schema: ['software'],
    video: V.first,
    related: ['mathcad-alternative', 'calcpad-online', 'excel-alternative-for-engineering-calculations', 'mathcad-worksheet-converter'],
    runs: {
        m: { sheet: `# Beam moment

L: 6 m | span
w: 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum moment
` },
    },
    lead: `<p>CivilKit Calcs is a calculation sheet for engineers that runs in a browser tab. You write quantities with units and formulas as readable text; the page works them out, typesets them, plots them if you ask, and prints or exports a report. It is free and open source, it works offline once loaded, and no calculation is sent to a server.</p>`,
    sections: [
        ['What it is for', `<p>It suits the calculations an engineer does many times a year and wants to read back later: a member check, a bearing pressure, a head loss, a quick plot of a function. Each calculation is a document with its inputs, steps and checks in order, so the person who reviews it can follow it.</p>
{{code:m}}
<p>That is a complete sheet. It reports a maximum moment of [[m:M]]. The units were multiplied and converted by the engine.</p>`],
        ['Ready-made calculators', `<p>These templates are sheets with inputs. Choose <strong>Open this template</strong> on any page and the sheet opens in the app as a form. All of them are generic textbook calculations, not design-code checks, and each page says what it assumes.</p>
<ul>
<li><a href="/calcs/learn/simply-supported-beam-calculator/">Simply supported beam calculator</a>: moment, shear, bending stress and deflection of an I-beam, with a moment diagram.</li>
<li><a href="/calcs/learn/retaining-wall-stability-calculator/">Retaining wall stability calculator</a>: earth thrust, overturning, sliding and base pressure for a cantilever wall.</li>
<li><a href="/calcs/learn/pad-footing-bearing-pressure-calculator/">Pad footing bearing pressure calculator</a>: eccentricity, largest and smallest soil pressure, uplift.</li>
<li><a href="/calcs/learn/pipe-flow-darcy-weisbach-calculator/">Pipe flow calculator, Darcy-Weisbach</a>: velocity, Reynolds number, friction factor, head loss and pump power.</li>
<li><a href="/calcs/learn/pump-operating-point-calculator/">Pump operating point calculator</a>: where a pump curve meets a pipeline curve, with both plotted.</li>
</ul>
<p>The app has two more templates, a cantilever with a tip load and a rectangular column slenderness check, and more than a hundred sample worksheets from the CalcpadCE project.</p>`],
        ['If you are coming from another tool', `<ul>
<li><a href="/calcs/learn/mathcad-alternative/">Mathcad alternative</a>, and how to <a href="/calcs/learn/mathcad-worksheet-converter/">convert Mathcad worksheets</a>.</li>
<li><a href="/calcs/learn/calcpad-online/">Calcpad online</a>, if you already use the Calcpad language.</li>
<li><a href="/calcs/learn/excel-alternative-for-engineering-calculations/">Excel alternative</a> for calculations that need their workings shown.</li>
</ul>`],
        ['What is in the box', `<ul>
<li>Real unit algebra, vectors and matrices, a large function library.</li>
<li>Plots from one line of a sheet, and Python blocks for Plotly-style and matplotlib-style charts.</li>
<li>Input forms from a <code>?</code> in front of a value: boxes, drop-downs, radio buttons and checkboxes.</li>
<li>A symbolic dialog for exact algebra, with documented limits.</li>
<li>Export to Word (with real equations), Excel (with workings and native charts), PDF through the print dialog, and HTML.</li>
<li>A worksheet importer for Mathcad files.</li>
<li>A <a href="/calcs/guide/">user guide</a> with short how-to videos.</li>
</ul>`],
        ['What it is not', `<p>It is not a finite element package, a structural design suite or a drawing tool. It does not carry design standards: the checks you write, and the ones in the templates, are as correct as the formulas in them. The project is early, so check any result you will rely on against a method you trust.</p>
<p>It is built on <a href="https://github.com/imartincei/CalcpadCE">CalcpadCE</a>, the MIT licensed Calcpad by Nedelcho Ganchovski (Proektsoft) and contributors, with a Rust engine compiled to WebAssembly.</p>`],
    ],
    faq: [
        ['Is it free for commercial use?', 'Yes. It is MIT licensed and free to use, with no account.'],
        ['What do I need to run it?', 'A current browser. You can install it as an app and use it offline after the first visit.'],
        ['Is my work stored on a server?', 'No. Calculations run in your browser and files are kept in its local storage or saved to your computer.'],
        ['Does it include design codes?', 'No. The templates are generic textbook calculations. You can write any formulas into a sheet, but the checks are yours.'],
        ['Where do I start?', 'Open a template from this page, then read the guide chapter "Your first sheet in five minutes".'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 5 beam */
const BEAM = 'Template simply supported beam.sheet';
pages.push({
    slug: 'simply-supported-beam-calculator',
    title: 'Simply supported beam calculator with units',
    description: 'Free beam calculator: moment, shear, bending stress and deflection of a simply supported I-beam under a uniform load, with a moment diagram. Opens as a form.',
    h1: 'Simply supported beam calculator',
    schema: [],
    open: 'template-simply-supported-beam',
    video: V.templates,
    related: ['retaining-wall-stability-calculator', 'pad-footing-bearing-pressure-calculator', 'engineering-calculation-software-free', 'excel-alternative-for-engineering-calculations'],
    howto: 'How to check a simply supported beam under a uniform load',
    runs: {
        base: { sample: BEAM },
        shallow: { sample: BEAM, set: { h: 'h:     ? [*200 mm deep = 200 mm, 300 mm deep = 300 mm, 400 mm deep = 400 mm] | I-beam section' } },
        alu: { sample: BEAM, set: { mat: 'mat:   ? [Structural steel = 1, *Aluminium = 2, Timber = 3] | material' } },
    },
    lead: `<p>This template checks a simply supported beam carrying a uniform load: the maximum bending moment and shear, the bending stress, and the mid-span deflection, each compared with a limit. The section is a doubly symmetric I-beam described by one depth, and the material comes from a drop-down. It opens as a fill-in form, and every change recalculates the page.</p>`,
    sections: [
        ['What it computes', `<p>You give the span, the uniform load, the section depth, the material, a deflection limit, and whether to include the beam's own weight. The sheet derives the section properties from the depth, adds the own weight to the load, and reports the bending moment, shear force, stress, deflection and two utilisations (stress over allowable stress, deflection over its limit), with a plain sentence under each saying whether it passes. A moment diagram is plotted along the span.</p>`],
        ['The formulas', `<ul>
<li>Section from the depth h: flange width b = h / 2, flange thickness t_f = h / 20, web thickness t_w = h / 40.</li>
<li>Area A = 2 b t_f + (h - 2 t_f) t_w. Second moment of area I = (b h^3 - (b - t_w)(h - 2 t_f)^3) / 12. Section modulus Z = 2 I / h.</li>
<li>Own weight w_sw = density x A x g, total load w_tot = w + w_sw.</li>
<li>Maximum moment M = w_tot L^2 / 8 at mid-span. Maximum shear V = w_tot L / 2 at the supports.</li>
<li>Bending stress sigma = M / Z. Deflection delta = 5 w_tot L^4 / (384 E I). Limit = L / 250, 360 or 500.</li>
<li>Utilisations: sigma / allowable stress, and delta / limit.</li>
</ul>
<p>The materials table holds three entries (structural steel, aluminium and timber) with a modulus, an allowable bending stress and a density. A new material is one more entry in each list.</p>`],
        ['Worked example', `<p>Defaults: span 6 m, load 12 kN/m, a 300 mm I-beam in structural steel (E = 200 GPa, allowable stress 165 MPa), a limit of L/250, own weight included. These are the values you get when you open the template. Results from the engine:</p>
${tbl('base', [['Area A', 'A'], ['Second moment of area I', 'I'], ['Section modulus Z', 'Z'], ['Own weight w_sw', 'w_sw'], ['Total load w_tot', 'w_tot'], ['Maximum moment M', 'M_max'], ['Maximum shear V', 'V_max'], ['Bending stress', 'sigma'], ['Mid-span deflection', 'delta'], ['Deflection limit', 'delta_lim'], ['Bending utilisation', 'util_b'], ['Deflection utilisation', 'util_d']])}
<p>Both utilisations are below 1, so the sheet prints "Bending: OK" and "Deflection: OK". Stress uses about [[base:util_b]] of the allowable stress and deflection about [[base:util_d]] of its limit.</p>`],
        ['Change one input and watch what governs', `<p>With the 200 mm section instead, the total load is [[shallow:w_tot]], the stress becomes [[shallow:sigma]] and the deflection [[shallow:delta]], giving utilisations of [[shallow:util_b]] for bending and [[shallow:util_d]] for deflection. Both fail, and the sheet says to choose a deeper section.</p>
<p>Back at 300 mm, switching the material to aluminium gives a stress of [[alu:sigma]] (utilisation [[alu:util_b]], which passes) but a deflection of [[alu:delta]] against a limit of [[alu:delta_lim]] (utilisation [[alu:util_d]], which fails). Stiffness, not strength, decides it. That is the kind of thing a form makes quick to see.</p>`],
        ['What this is, and is not', notCode('The beam is one doubly symmetric I-section with proportions fixed by the depth, loaded with one uniform load on a single span, and compared with an allowable stress you choose from the table. There is no lateral buckling check, no shear or web check and no load combination.')],
        ['The sheet source', `<p>This is the template exactly as it ships. The lines with a question mark are the inputs; everything else is calculation and text. You can copy it into the app, or open the template and edit it.</p>
{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('The span, load, section, material, limit and own weight are the six inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('The span, load, section, material, limit and own weight are the six inputs.'),
    faq: [
        ['Which beam formulas does the calculator use?', 'Maximum moment wL^2/8, maximum shear wL/2 and mid-span deflection 5wL^4/(384EI), all for a simply supported span with a uniform load.'],
        ['Can I use my own section?', 'Yes. Edit the section lines in the sheet and enter A, I and Z yourself, or replace the I-beam proportions. The rest of the sheet uses those names.'],
        ['Does it check lateral buckling or shear capacity?', 'No. It checks bending stress and deflection against allowable values, nothing else. It is a textbook calculation, not a design-code check.'],
        ['Can I save the result?', 'Yes. Use Report, then Print PDF, Word or Excel. The Word and Excel files use the values you entered.'],
        ['Can I add a point load?', 'You can add lines for it in the sheet, because it is ordinary text. The template as shipped has the uniform load only.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 6 retaining wall */
const WALL = 'Template retaining wall stability.sheet';
pages.push({
    slug: 'retaining-wall-stability-calculator',
    title: 'Retaining wall stability calculator',
    description: 'Free retaining wall calculator: active earth thrust, overturning and sliding safety factors and base pressure for a cantilever wall, with units.',
    h1: 'Retaining wall stability calculator',
    schema: [],
    open: 'template-retaining-wall-stability',
    video: V.templates,
    related: ['pad-footing-bearing-pressure-calculator', 'simply-supported-beam-calculator', 'engineering-calculation-software-free', 'pump-operating-point-calculator'],
    howto: 'How to check the stability of a cantilever retaining wall',
    runs: {
        base: { sample: WALL },
        sur: { sample: WALL, set: { sur: 'sur:   ? on | surcharge on the fill' } },
    },
    lead: `<p>This template checks a cantilever retaining wall (a stem on a base slab) that holds back level fill. For a one metre strip of wall it finds the active earth thrust, the factor of safety against overturning about the toe, the factor of safety against sliding, and the soil pressure under the base. The backfill type comes from radio buttons and a checkbox adds a surcharge on the fill.</p>`,
    sections: [
        ['What it computes', `<p>Inputs are the stem height, base width, base thickness, stem thickness, toe length, the backfill (loose, medium or dense sand, given as a friction angle) and an optional surcharge. The sheet works out the weights of the stem, base and fill on the heel, the thrust of the fill and surcharge, the overturning and resisting moments about the toe, and the pressure under the base. Two sentences under the results say whether overturning and sliding meet the required factor of safety of 1.5, which is a constant in the sheet that you can change.</p>`],
        ['The formulas', `<ul>
<li>Active pressure coefficient K_a = tan^2(45 degrees - phi / 2).</li>
<li>Thrust of the fill P_s = 1/2 K_a gamma H^2, acting H/3 above the underside of the base. H is the stem height plus the base thickness, because the thrust acts on the vertical plane through the heel.</li>
<li>Surcharge q adds a thrust P_q = K_a q H at H/2, and its weight on the heel.</li>
<li>Overturning: M_o = P_s H/3 + P_q H/2 about the toe, resisted by the weights times their lever arms. FS_o = M_r / M_o.</li>
<li>Sliding: friction coefficient mu = tan(2 phi / 3), resistance mu W, FS_s = mu W / P_h.</li>
<li>Base pressure: the resultant crosses the base at x_r = (M_r - M_o) / W from the toe, eccentricity e = B/2 - x_r, and q = W/B (1 plus or minus 6e/B).</li>
</ul>`],
        ['Worked example', `<p>Defaults: a 3 m stem on a 2.4 m by 0.4 m base, a 0.4 m stem, a 0.6 m toe, medium sand (32 degrees), no surcharge, concrete at 24 kN/m^3 and fill at 18 kN/m^3. Engine results:</p>
${tbl('base', [['Active coefficient K_a', 'K_a'], ['Thrust of the fill P_s', 'P_s'], ['Total vertical load W', 'W'], ['Overturning moment M_o', 'M_o'], ['Resisting moment M_r', 'M_r'], ['Factor of safety, overturning', 'FS_o'], ['Friction coefficient', 'mu'], ['Sliding resistance', 'R_h'], ['Factor of safety, sliding', 'FS_s'], ['Eccentricity of the resultant', 'e'], ['Pressure at the toe', 'q_max'], ['Pressure at the heel', 'q_min']])}
<p>Overturning is comfortable at [[base:FS_o]]. Sliding is closer, at [[base:FS_s]] against 1.5 required. The resultant is within the middle third of the base, so the whole base is in compression and the toe and heel pressures above apply.</p>`],
        ['Turn on the surcharge', `<p>Ticking the surcharge box (10 kPa on the fill) raises the horizontal thrust to [[sur:P_h]]. Overturning drops to [[sur:FS_o]], and sliding to [[sur:FS_s]], which is below 1.5, so the sheet now prints that the factor of safety is too low and suggests widening or thickening the base. The toe pressure rises to [[sur:q_max]]. Sliding governs, and the sheet shows it immediately.</p>`],
        ['What this is, and is not', notCode('The wall is a stem on a base slab, the fill is level, dry and granular, the back of the wall is treated as vertical with no wall friction, and the soil in front of the toe is ignored. There is no water, no seismic load, no passive resistance, no bearing capacity check and no check of the wall as a structure. The factor of safety of 1.5 is a value in the sheet, not a requirement of any standard.')],
        ['The sheet source', `{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('The wall dimensions, the backfill and the surcharge are the inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('The wall dimensions, the backfill and the surcharge are the inputs.'),
    faq: [
        ['Which earth pressure theory does it use?', 'Rankine active pressure on the vertical plane through the heel, with the coefficient tan^2(45 degrees - phi/2).'],
        ['What factor of safety does it require?', 'The sheet compares overturning and sliding with 1.5, which is a constant at the top of the sheet. Change it to your own value.'],
        ['Does it check bearing capacity?', 'No. It gives the pressure under the base and tells you whether the resultant is in the middle third. Compare the pressure with a bearing value you trust.'],
        ['Can I model sloping fill or groundwater?', 'Not as shipped. The sheet is plain text, so you can add the extra forces, but the template assumes level, dry fill.'],
        ['Is it a design-code check?', 'No. It is a textbook stability calculation and it names no standard.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 7 pad footing */
const PAD = 'Template pad footing bearing.sheet';
pages.push({
    slug: 'pad-footing-bearing-pressure-calculator',
    title: 'Pad footing bearing pressure calculator',
    description: 'Free pad footing calculator: bearing pressure under a column load and moment, eccentricity, uplift and the check against an allowable soil pressure.',
    h1: 'Pad footing bearing pressure calculator',
    schema: [],
    open: 'template-pad-footing-bearing',
    video: V.templates,
    related: ['retaining-wall-stability-calculator', 'simply-supported-beam-calculator', 'engineering-calculation-software-free', 'excel-alternative-for-engineering-calculations'],
    howto: 'How to check the bearing pressure under a pad footing',
    runs: {
        base: { sample: PAD },
        moment: { sample: PAD, set: { M: 'M:    ? 250 kN*m | column moment, in the direction of L' } },
    },
    lead: `<p>A rectangular pad footing carries a vertical column load and a moment. This template finds the soil pressure under it: the total load including the footing and the soil above it, the eccentricity of the load, the largest and smallest bearing pressure, whether part of the base lifts off, and the check against an allowable pressure chosen from a list of soils.</p>`,
    sections: [
        ['What it computes', `<p>The inputs are the column load N, the moment M in the direction of the footing length, the footing width B, length L and thickness t, the depth of the base below the ground, a soil from a drop-down (each with an allowable bearing pressure) and a checkbox to include the weight of the footing and of the soil above it. The sheet reports the total load P, the eccentricity, the average, largest and smallest pressures and the utilisation of the allowable pressure, and prints whether the base stays in full contact.</p>`],
        ['The formulas', `<ul>
<li>Total vertical load P = N + weight of the footing (24 kN/m^3 concrete) + weight of the soil above it (18 kN/m^3).</li>
<li>Eccentricity e = M / P. The load stays in the middle third of the footing while e is no more than L / 6.</li>
<li>Average pressure P / (B L). While e is within L / 6, the largest and smallest pressures are q = P / (B L) x (1 plus or minus 6 e / L).</li>
<li>Beyond L / 6 the base is in partial contact and the largest pressure is q_max = 2 P / (3 B (L/2 - e)), with zero at the other edge.</li>
<li>Utilisation = q_max / allowable bearing pressure.</li>
</ul>`],
        ['Worked example', `<p>Defaults: N = 600 kN, M = 60 kN·m, a 2 m by 2.4 m footing 0.5 m thick, base 1 m below ground, medium dense sand (allowable pressure 200 kPa), weights included. Engine results:</p>
${tbl('base', [['Weight of the footing', 'W_f'], ['Weight of the soil above', 'W_s'], ['Total load P', 'P'], ['Eccentricity e', 'e'], ['Middle-third limit L/6', 'e_lim'], ['Average pressure', 'q_avg'], ['Largest pressure', 'q_max'], ['Smallest pressure', 'q_min'], ['Utilisation', 'util']])}
<p>The eccentricity of [[base:e]] is well inside the limit of [[base:e_lim]], so the whole base is in contact with the soil. The largest pressure is [[base:q_max]], which uses [[base:util]] of the allowable pressure, so the sheet prints OK.</p>`],
        ['When the moment grows', `<p>With the moment raised to 250 kN·m the eccentricity becomes [[moment:e]], close to the middle-third limit of [[moment:e_lim]]. The largest pressure rises to [[moment:q_max]] while the smallest falls to [[moment:q_min]], and the utilisation of [[moment:util]] fails the check. The sheet says to enlarge the footing. A larger moment, or a smaller footing, takes the load outside the middle third and the formula switches to partial contact, with the other edge at zero.</p>`],
        ['What this is, and is not', notCode('The footing is a rectangle with a uniform soil, the moment acts about one axis, and the pressure is assumed to vary linearly across the base. The allowable bearing pressures in the drop-down are round example values, not site data. There is no bearing capacity calculation, no settlement, no sliding check and no structural check of the footing.')],
        ['The sheet source', `{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('Use your own loads, footing size and allowable pressure; to use a value that is not in the soil list, edit the list in the sheet.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('Use your own loads, footing size and allowable pressure; to use a value that is not in the soil list, edit the list in the sheet.'),
    faq: [
        ['What is the middle third rule?', 'While the eccentricity of the load is no more than one sixth of the footing length, the resultant lies in the middle third and the whole base stays in compression. Beyond that, part of the base lifts off.'],
        ['Can I enter my own allowable bearing pressure?', 'Edit the soil list in the sheet and add an entry with your value, or replace the drop-down with a number input.'],
        ['Does it handle moments about both axes?', 'No. The moment acts in the direction of the length. For biaxial loading you would add a second eccentricity to the sheet.'],
        ['Does it check bearing capacity or settlement?', 'No. It gives contact pressure and compares it with an allowable value you supply.'],
        ['Is it a design-code check?', 'No. It is a generic textbook calculation and uses no standard.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 8 pipe flow */
const PIPE = 'Template pipe flow Darcy.sheet';
pages.push({
    slug: 'pipe-flow-darcy-weisbach-calculator',
    title: 'Pipe flow calculator: Darcy-Weisbach head loss',
    description: 'Free pipe friction loss calculator: velocity, Reynolds number, friction factor, Darcy-Weisbach head loss, pump head and power for water in a full pipe.',
    h1: 'Pipe flow calculator with Darcy-Weisbach head loss',
    schema: [],
    open: 'template-pipe-flow-darcy',
    video: V.templates,
    related: ['pump-operating-point-calculator', 'simply-supported-beam-calculator', 'engineering-calculation-software-free', 'excel-alternative-for-engineering-calculations'],
    howto: 'How to calculate friction head loss in a full pipe',
    runs: {
        base: { sample: PIPE },
        flow: { sample: PIPE, set: { Q: 'Q:    ? 40 L/s | flow rate' } },
        small: { sample: PIPE, set: { D: 'D:    ? 100 mm | inside diameter of the pipe' } },
    },
    lead: `<p>This template finds the friction loss for water flowing in a full circular pipe, then the head a pump must supply and the hydraulic power. You enter the flow, diameter, length, rise in level, a sum of fitting loss coefficients, the pipe material (as a roughness) and the water temperature, and the sheet works through velocity, Reynolds number, friction factor and head loss.</p>`,
    sections: [
        ['What it computes', `<p>From the flow rate and the bore it gets the mean velocity and the Reynolds number. It picks a friction factor for laminar or turbulent flow, then applies the Darcy-Weisbach equation for the friction loss and adds the loss in fittings. The head to be supplied is the rise in level plus the total loss. It also reports the pressure drop due to losses and the hydraulic power of the water, and flags a velocity above 3 m/s.</p>`],
        ['The formulas', `<ul>
<li>Flow area A = pi D^2 / 4. Velocity V = Q / A. Reynolds number Re = V D / nu, with the kinematic viscosity nu looked up for 5, 20 or 40 degrees C.</li>
<li>Laminar flow (Re below 2300): friction factor f = 64 / Re.</li>
<li>Turbulent flow: the explicit Swamee-Jain formula f = 0.25 / [log10(eps / (3.7 D) + 5.74 / Re^0.9)]^2, where eps is the pipe roughness.</li>
<li>Friction head loss h_f = f (L / D) V^2 / (2 g). Fittings h_m = K V^2 / (2 g).</li>
<li>Head to supply H = rise + h_f + h_m. Pressure drop = rho g (h_f + h_m). Hydraulic power = rho g Q H.</li>
</ul>`],
        ['Worked example', `<p>Defaults: 25 L/s of water at 5 degrees C through 300 m of 150 mm commercial steel pipe (roughness 0.045 mm), a rise of 5 m and fittings with a total K of 2.5. Engine results:</p>
${tbl('base', [['Flow area', 'A'], ['Velocity', 'V'], ['Reynolds number', 'Re'], ['Relative roughness', 'rr'], ['Friction factor', 'f'], ['Friction head loss', 'h_f'], ['Head loss in fittings', 'h_m'], ['Total head loss', 'h_L'], ['Head the pump must supply', 'H'], ['Pressure drop from losses', 'dp'], ['Hydraulic power', 'P_h']])}
<p>The flow is turbulent (Re is [[base:Re]]) and the velocity of [[base:V]] is within the 3 m/s guide, so the sheet prints OK.</p>`],
        ['Change the flow or the bore', `<p>Raising the flow to 40 L/s lifts the velocity to [[flow:V]], the total loss to [[flow:h_L]] and the head to supply to [[flow:H]], with a hydraulic power of [[flow:P_h]]. Head loss rises roughly with the square of the flow, which is why the loss more than doubles when the flow grows by 60 percent.</p>
<p>Keeping 25 L/s but dropping the pipe to 100 mm gives [[small:V]], which is above 3 m/s, so the sheet warns about it. The loss is then [[small:h_L]] and the head to supply [[small:H]]. A modest cut in diameter has a large effect, because the velocity scales with the inverse square of the diameter.</p>`],
        ['What this is, and is not', notCode('The fluid is water at one of three temperatures from a short table, the pipe is full and of one diameter, and the fittings are lumped into one coefficient. The friction factor uses an explicit approximation, and the sheet switches between laminar and turbulent formulas at Re 2300, with no treatment of the transition zone. Pipe roughness values are typical figures for new pipe; real pipe ages.')],
        ['The sheet source', `{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('Flow, diameter, length, rise, fittings, material and temperature are the inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('Flow, diameter, length, rise, fittings, material and temperature are the inputs.'),
    faq: [
        ['Which friction factor formula is used?', 'For turbulent flow, the explicit Swamee-Jain approximation of the Colebrook equation. For laminar flow, 64 / Re.'],
        ['Can I use a fluid other than water?', 'Edit the density and viscosity tables in the sheet. They hold three entries for water at 5, 20 and 40 degrees C.'],
        ['Does it handle different diameters in series?', 'Not as shipped. It models one diameter. You can add sections as further lines in the sheet.'],
        ['How accurate is the head loss?', 'It is as accurate as the roughness, the fitting coefficients and the correlation you give it. Treat it as an estimate and compare with a method you trust for a critical line.'],
        ['What does the 3 m/s warning mean?', 'It is a rule of thumb in the sheet for noise and wear, not a limit from a standard. Change it to suit your project.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 9 pump */
const PUMP = 'Template pump operating point.sheet';
pages.push({
    slug: 'pump-operating-point-calculator',
    title: 'Pump operating point calculator: pump vs system curve',
    description: 'Free calculator for the operating point of a pump on a pipeline: where the pump curve meets the system curve, with a table of heads and both curves plotted.',
    h1: 'Pump operating point calculator',
    schema: [],
    open: 'template-pump-operating-point',
    video: V.plots,
    related: ['pipe-flow-darcy-weisbach-calculator', 'retaining-wall-stability-calculator', 'engineering-calculation-software-free', 'simply-supported-beam-calculator'],
    howto: 'How to find the operating point of a pump on a pipeline',
    runs: {
        base: { sample: PUMP },
        aged: { sample: PUMP, set: { aged: 'aged: ? on | aged pipe (rougher: the friction factor is 0.030, not 0.018)' } },
        lift: { sample: PUMP, set: { H_st: 'H_st: ? 35 m | static lift, from the water surface to the tank' } },
    },
    lead: `<p>A pump on a pipeline runs where the head it can deliver equals the head the pipeline demands. This template finds that point for a pump lifting water to a higher tank, tabulates both heads against flow, and plots the two curves so you can see where they cross. Pump size, pipe diameter, length, static lift and pipe condition are inputs.</p>`,
    sections: [
        ['What it computes', `<p>The pump is described by a shut-off head and a free delivery, which the sheet turns into a falling parabola of head against flow. The pipeline needs the static lift plus a friction loss that grows with the square of the flow. The operating point is the flow at which the two heads are equal, and the sheet solves that directly. It then reports the flow, the head, the pipe velocity (with a warning above 3 m/s) and a table and plot of both curves.</p>`],
        ['The formulas', `<ul>
<li>Pump curve: H_p = H_0 (1 - (Q / Q_max)^2), with a shut-off head H_0 and a free delivery Q_max.</li>
<li>System curve: H_s = H_st + K Q^2, where H_st is the static lift and K = f (L / D) / (2 g A^2), with a constant friction factor f of 0.018, or 0.030 for an aged pipe.</li>
<li>The curves cross where H_0 - H_0 Q^2 / Q_max^2 = H_st + K Q^2, so Q^2 = (H_0 - H_st) / (K + H_0 / Q_max^2).</li>
<li>If H_0 is not above the static lift, the pump cannot lift the water and the sheet says so.</li>
</ul>`],
        ['Worked example', `<p>Defaults: the medium pump (40 m shut-off head, 75 L/s free delivery), a 150 mm pipe 400 m long, a static lift of 15 m and a new pipe. Engine results:</p>
${tbl('base', [['Friction head per flow squared K', 'K'], ['Head above the static lift', 'H_net'], ['Flow at the operating point', 'Q_op'], ['Head at the operating point', 'H_op'], ['Velocity in the pipe', 'v']])}
<p>The pump settles at [[base:Q_op]] and [[base:H_op]]. The velocity of [[base:v]] is within 3 m/s.</p>`],
        ['Move the operating point', `<p>Marking the pipe as aged raises the friction factor to 0.030, which increases K to [[aged:K]] and moves the operating point to [[aged:Q_op]] at [[aged:H_op]]. A rougher pipe means less flow from the same pump.</p>
<p>Raising the static lift to 35 m instead drops the flow to [[lift:Q_op]], with the pump working harder at [[lift:H_op]]. The table in the sheet shows the heads at flows from 0 to 60 L/s, and the plot shows the crossing.</p>`],
        ['What this is, and is not', notCode('The pump curves are quadratic models with round example numbers for a small, medium and large pump, not manufacturer data. The friction factor is a constant, not recalculated from the flow, and the pipeline is one diameter with no minor losses. There is no efficiency, power, NPSH or cavitation check, and no speed or parallel pump operation. To use a real pump, replace the shut-off head and free delivery with values that fit its curve.')],
        ['The sheet source', `{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('Pump size, pipe diameter, length, static lift and the aged-pipe box are the inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('Pump size, pipe diameter, length, static lift and the aged-pipe box are the inputs.'),
    faq: [
        ['What is the operating point of a pump?', 'The flow and head at which the pump curve crosses the system curve of the pipeline. The pump cannot run anywhere else on that pipeline.'],
        ['Can I use my own pump curve?', 'The template models the pump with two numbers, a shut-off head and a free delivery. Replace them with values that fit your pump, or replace the pump curve line with your own formula.'],
        ['Why is the friction factor constant?', 'To keep the sheet simple and solvable in closed form. The pipe flow template calculates it from the Reynolds number if you want that.'],
        ['Does it give efficiency or power?', 'No. Add a line for power using the flow and head if you know the efficiency.'],
        ['Is it suitable for selecting a pump?', 'Use it to understand the system. Select a pump from the manufacturer\'s published curves.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 10 converter */
pages.push({
    slug: 'mathcad-worksheet-converter',
    title: 'Mathcad worksheet converter: import .xmcd and .mcdx',
    description: 'Import Mathcad .xmcd and .mcdx worksheets into a browser calculation sheet. What converts, what shows as a gap, and why your file is not uploaded.',
    h1: 'Convert Mathcad worksheets (.xmcd and .mcdx) in your browser',
    schema: [],
    video: V.importing,
    related: ['mathcad-alternative', 'calcpad-online', 'engineering-calculation-software-free', 'excel-alternative-for-engineering-calculations'],
    howto: 'How to import a Mathcad worksheet into CivilKit Calcs',
    runs: {},
    lead: `<p>CivilKit Calcs can read Mathcad worksheet files (<code>.xmcd</code> and <code>.mcdx</code>) and write them out as a new worksheet that its own engine calculates. The conversion runs in your browser, so the file is not uploaded. This page says what to expect, including what does not convert.</p>
<p>The importer is a work in progress. It is a translator, not a guarantee. Plan on checking every imported worksheet against the original, line by line.</p>`,
    sections: [
        ['How to import a worksheet', `<ol>
<li>Open <a href="/calcs/">CivilKit Calcs</a>.</li>
<li>Choose <strong>File, then Import worksheet</strong>, or the <strong>Import worksheet</strong> button on the Document tab. You can also use File, then Open, or drag the file onto the page.</li>
<li>The converted worksheet opens in a new unsaved tab named after the file, with the extension <code>.cpd</code>.</li>
<li>A dialog tells you how the import went, for example "18 of 21 regions converted live, 2 shown as static pictures, 1 need attention", with the reasons in plain words.</li>
<li>Read the list of gaps first, then compare the results with the original worksheet.</li>
</ol>`],
        ['What converts', `<table>
<thead><tr><th>In the source file</th><th>In the result</th></tr></thead>
<tbody>
<tr><td>Text, with headings, bold, italic, sub and superscripts</td><td>Text lines. Fonts, sizes and colours can be tidied away afterwards.</td></tr>
<tr><td>Definitions, evaluations and functions</td><td>Definitions and equation lines with the same names, subscripts and Greek letters.</td></tr>
<tr><td>Units</td><td>The same units under this engine's names. A unit it does not know is reported.</td></tr>
<tr><td>Vectors, matrices and indexing</td><td>Vectors and matrices. The array origin is shifted to 1, with a note.</td></tr>
<tr><td>Sums, products, integrals, derivatives (supported forms)</td><td>The engine's sum, product, area and slope commands.</td></tr>
<tr><td>Programs of loops, conditions and early exits</td><td>Expression blocks.</td></tr>
<tr><td>x-y plots</td><td>Plot lines. Colours, markers and axis ranges may not carry over.</td></tr>
<tr><td>Solve blocks of one or two unknowns</td><td>Solved here by iteration, and kept only if the equations hold at the answer.</td></tr>
<tr><td>Statistics functions, complex numbers, simple text values</td><td>Converted.</td></tr>
<tr><td>Referenced worksheets</td><td>Their definitions are read in when the file holds a copy.</td></tr>
<tr><td>Pictures</td><td>Embedded in the browser library.</td></tr>
</tbody>
</table>
<p>The equations are shown as the author wrote them, with stacked fractions and bounded sums, and computed out of sight. Every number in the output was in the source file; the importer evaluates nothing itself and the engine then runs the result.</p>`],
        ['What is frozen', `<p>Some values cannot be computed here, so the importer takes the number the source file saved and shows it as a literal marked frozen. That applies to values from input components such as a list box or text box, calls into add-in functions that are not available when the file saved a value, tables read from data files, and solve blocks whose iteration does not verify. A frozen value does not follow a change of its inputs. The dialog counts them.</p>`],
        ['What does not convert', `<ul>
<li>Range variables outside the supported uses, and programs outside the supported subset.</li>
<li>Solve blocks of more than two unknowns, other solvers and symbolic evaluation.</li>
<li>Derivative, integral and summation forms the importer does not map.</li>
<li>Plots that are not x-y plots.</li>
<li>Embedded objects, components, worksheet references with no copy, and add-in functions with no saved value.</li>
<li>Text values that are joined, measured or formatted.</li>
<li>Regions the source has switched off, and some page headers and footers.</li>
</ul>
<p>Each gap is replaced by a placeholder: a typeset, not-calculated equation, or a picture of the original region. A calculation that depends on an unconverted region shows its names as undefined. The dialog ranks the missing features by how many regions each one blocks, so you can see the worst first.</p>`],
        ['Is the file uploaded?', `<p>No. The file is read and converted inside the page, by a module that loads on first use and runs in a web worker. We checked the import code: it hands the file's bytes to the worker and gets text and pictures back by message, with no network request carrying the file. The dialog also ends with a line saying the file was read in this page and was not uploaded. Pictures from the file are stored in the browser's library under the name of the file.</p>
<p>Limits: input over 64 MiB, XML nested deeper than 256 levels, a document type declaration, and pictures over 16 MiB each or 192 MiB in total are refused or skipped. Problems come back as messages, not crashes.</p>`],
        ['After the import', `<ul>
<li><strong>Tidy text</strong> on the Document tab removes imported fonts, sizes and colours from paragraphs. <strong>Text to Markdown</strong> then writes runs of simple paragraphs as Markdown you can edit.</li>
<li>Save the worksheet. It refers to the pictures by their library path, so export to HTML or Word first if you need a self-contained file.</li>
<li>Compare the numbers. A good habit is to put the original's results next to the imported ones, and open the dialog's list of gaps beside them.</li>
</ul>
<p>If the result is a mess, the original is untouched. Nothing is changed in your file.</p>`],
    ],
    steps: [
        ['Open the importer', 'In CivilKit Calcs choose File, then Import worksheet, or drag the .xmcd or .mcdx file onto the page.'],
        ['Read the summary', 'The dialog says how many regions converted live, how many are shown as pictures and how many need attention, with the main reasons.'],
        ['Check the result', 'Compare the converted worksheet with the original line by line before relying on it.'],
        ['Tidy and save', 'Use Tidy text if you want the imported fonts removed, then save the worksheet.'],
    ],
    faq: [
        ['Which file types can it import?', 'The <code>.xmcd</code> and <code>.mcdx</code> worksheet formats. Other file types are rejected with a message.'],
        ['Will every worksheet convert completely?', 'No. Worksheets that use add-ins, embedded objects, symbolic evaluation or unsupported plots will have gaps, and the importer says where.'],
        ['Does it keep my original file?', 'Yes. It reads the file and does not change it.'],
        ['Is my worksheet sent to a server?', 'No. It is converted in your browser. We checked the import code for network requests that carry the file and found none.'],
        ['Do the numbers match the original?', 'Constants are copied from the source file and the engine recomputes the rest. Check on your own files: do not assume they match.'],
    ],
});
