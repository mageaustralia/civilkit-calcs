// The data of the ten landing pages (public/learn/<slug>/index.html). Prose is written here; every number that comes
// from a calculation is a [[run:name]] placeholder that make_learn_pages.mjs fills from a real engine run and
// web/e2e/learn-check.mjs re-checks. No design standards are named: these are generic textbook calculations.

const V = {
    first: { file: '01-first-calculation.mp4', title: 'Your first calculation', chapter: 'getting-started/first-sheet.md' },
    plots: { file: '03-plots-in-sheets.mp4', title: 'Plots in the sheet format', chapter: 'plots.md' },
    templates: { file: '05-templates-with-inputs.mp4', title: 'Templates with inputs', chapter: 'templates.md' },
    importing: { file: '06-import-a-worksheet.mp4', title: 'Import an existing worksheet', chapter: 'importing.md' },
    exporting: { file: '08-export-word-excel-pdf.mp4', title: 'Export a report to Word, Excel and PDF', chapter: 'exporting.md' },
    units: { file: '02-units-and-results.mp4', title: 'Units and results', chapter: 'units.md', seconds: 85 },
    symbolic: { file: '07-symbolic-beam-deflection.mp4', title: 'Symbolic maths for a beam deflection', chapter: 'symbolic.md', seconds: 90 },
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
    label: 'Mathcad alternative',
    group: 'alternative',
    title: 'Free Mathcad alternative that runs in your browser',
    description: 'A free, open source engineering calculation sheet for the browser: real units, plots, Word and Excel export, offline use, and a Mathcad worksheet importer.',
    h1: 'A free Mathcad alternative that runs in your browser',
    schema: ['software'],
    video: V.first,
    related: ['mathcad-worksheet-converter', 'calcpad-online', 'excel-alternative-for-engineering-calculations', 'export-calculations-to-word-excel-pdf', 'engineering-calculation-software-free'],
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
    label: 'Calcpad worksheets in the browser',
    group: 'alternative',
    title: 'Open Calcpad .cpd files in your browser, offline',
    description: 'Open and run Calcpad .cpd worksheets in your browser, offline, on an open source Rust engine. Edit, plot, import and export to Word, Excel or PDF.',
    h1: 'Run Calcpad worksheets in your browser, offline',
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
    lead: `<p>Calcpad is a calculator and report writer by Nedelcho Ganchovski (Proektsoft): you write equations with units and get a formatted document. Its authors offer an official online version on their own site, and that is the place to go for Calcpad itself. This page is about something else. CivilKit Calcs is an independent, open source derivative of CalcpadCE, the MIT licensed community edition, and it opens your <code>.cpd</code> worksheets in a browser tab and runs them there, with no server and, once it has loaded, no connection.</p>
<p>The worksheet language is the same. The calculation engine was rewritten in Rust and compiled to WebAssembly, so the arithmetic happens on your own computer. It is not affiliated with the Calcpad authors: their work is the foundation, and the credit and licence travel with the code.</p>`,
    sections: [
        ['Open a .cpd worksheet', `<p>Open the app and use File, then Open, drag a <code>.cpd</code> file onto the page, or paste the text into a new Calcpad tab. The page calculates it straight away. A worksheet like this one is ordinary Calcpad:</p>
{{code:t}}
<p>It gives [[t:c]] for the hypotenuse, with the units carried through. Everything the language offers is meant to work: units, vectors and matrices, <code>$Plot</code> and <code>$Map</code> plots, macros, <code>#if</code> and <code>#while</code> blocks, includes, input fields and the large function library.</p>`],
        ['Offline, and on your own computer', `<p>The first visit loads the app; after that a service worker keeps a copy, so the page opens and calculates with no network at all. You can install it from the browser as an app on a desktop or a phone. Worksheets are kept in the browser's own library or saved to your disk as ordinary <code>.cpd</code> text files, and nothing you calculate is sent anywhere. That makes it a practical way to read and check a colleague's worksheet on a machine where you cannot install software.</p>`],
        ['What was rebuilt, and what was kept', `<ul>
<li><strong>The engine</strong> is a Rust port of the CalcpadCE calculation engine, compiled to WebAssembly. The page answers the calls the original made to its server itself, so no server is needed.</li>
<li><strong>The editor and report view</strong> are CalcpadCE's web frontend, with changes.</li>
<li><strong>Word and Excel export</strong> are built in the page. Equations become Word equations. The Excel workbook has the workings, variables and native charts.</li>
<li><strong>Installable and offline.</strong> A service worker caches the build so the page opens and calculates without a connection.</li>
</ul>`],
        ['How close to CalcpadCE is it?', `<p>The Rust engine is tested against CalcpadCE's own test classes, transcribed to Rust, and against the expected-output files of its worksheets. Where it deliberately behaves differently, the difference is listed one by one with the test that pins it, in a file of deviations in the repository. One example: when an <code>#if</code> condition cannot be evaluated, the block is left unresolved with an error message, instead of silently running the <code>#else</code> branch.</p>
<p>Not supported: raster plots from the original's graphics library, the project and library path roots, and compiled <code>.cpdz</code> worksheets. The project is early, so if you depend on an unusual corner of the language, run your worksheet and compare the output before you switch.</p>`],
        ['Beyond the worksheet: sheets, imports and exports', `<p>On top of the Calcpad language there is a lighter way to write, called the sheet format: Markdown prose, <code>name: expression -> unit | note</code> lines and LaTeX. A sheet compiles to Calcpad and runs on the same engine, and you can always see the Calcpad it becomes with Show compiled. A <code>?</code> in front of a value makes it an input, so a sheet becomes a fill-in template; the <a href="/calcs/learn/simply-supported-beam-calculator/">beam calculator</a> is a worked one.</p>
<p>The same page can <a href="/calcs/learn/mathcad-worksheet-converter/">import worksheets from another calculation tool</a> and export any worksheet to Word with real equations, to Excel with native charts, or to PDF.</p>`],
        ['Credit and licence', `<p>Calcpad was created by Nedelcho Ganchovski (Proektsoft); CalcpadCE is its MIT licensed continuation, at <a href="https://github.com/imartincei/CalcpadCE">github.com/imartincei/CalcpadCE</a>. CivilKit Calcs is MIT licensed too, and the upstream copyright notices are kept in the repository. If Calcpad is useful to you, look at the original project as well.</p>`],
    ],
    faq: [
        ['Is this the official Calcpad online version?', 'No. The Calcpad authors publish their own online version. This is an independent derivative of CalcpadCE, the MIT licensed community edition, with the engine rewritten in Rust. It is not made or endorsed by the Calcpad authors.'],
        ['Do I have to install anything?', 'No. Open the page in a current browser. You can install it as an app from the browser if you want it on the desktop or a phone.'],
        ['Can I open my existing .cpd files?', 'Yes, with File then Open or by pasting the text. Check the result against what you expect, because the engine is a port and a few behaviours are deliberately different. The deviations file lists them.'],
        ['Does my worksheet leave my computer?', 'No. The engine runs in the page. There is no calculation server.'],
        ['Does it work offline?', 'Yes, after a first visit. Plotly charts, which load a script from a CDN, need to have been loaded once while online.'],
        ['Can it open compiled .cpdz files?', 'No. Compiled <code>.cpdz</code> worksheets are not supported. Open the <code>.cpd</code> source instead.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 3 */
pages.push({
    slug: 'excel-alternative-for-engineering-calculations',
    label: 'Excel alternative',
    group: 'alternative',
    title: 'Excel alternative for engineering calculations',
    description: 'Write engineering calculations with units and readable formulas instead of cell references, see the workings, and export to Word, PDF and Excel.',
    h1: 'An Excel alternative for engineering calculations',
    schema: ['software'],
    video: V.exporting,
    related: ['mathcad-alternative', 'export-calculations-to-word-excel-pdf', 'engineering-calculations-with-units', 'simply-supported-beam-calculator', 'engineering-calculation-software-free'],
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
    label: 'Free calculation software',
    group: 'alternative',
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
<p>More calculators, from a <a href="/calcs/learn/cantilever-beam-calculator/">cantilever</a> and <a href="/calcs/learn/column-buckling-calculator/">column buckling</a> to <a href="/calcs/learn/principal-stress-calculator/">principal stresses</a> and <a href="/calcs/learn/concrete-strength-gain-calculator/">concrete strength gain</a>, are listed in <a href="/calcs/learn/">Calculators and guides</a>. The app also has more than a hundred sample worksheets from the CalcpadCE project.</p>`],
        ['If you are coming from another tool', `<ul>
<li><a href="/calcs/learn/mathcad-alternative/">Mathcad alternative</a>, and how to <a href="/calcs/learn/mathcad-worksheet-converter/">open Mathcad files without Mathcad</a>.</li>
<li><a href="/calcs/learn/calcpad-online/">Calcpad worksheets in the browser</a>, if you already use the Calcpad language.</li>
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
    label: 'Simply supported beam',
    group: 'calculator',
    area: 'structural',
    title: 'Simply supported beam calculator with units',
    description: 'Free beam calculator: moment, shear, bending stress and deflection of a simply supported I-beam under a uniform load, with a moment diagram. Opens as a form.',
    h1: 'Simply supported beam calculator',
    schema: [],
    open: 'template-simply-supported-beam',
    video: V.templates,
    related: ['cantilever-beam-calculator', 'bending-moment-shear-force-diagram', 'beam-deflection-formula-derivation', 'column-buckling-calculator', 'engineering-calculation-software-free'],
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
        ['The sheet source', `<p>This is the template exactly as it ships. The lines with a question mark are the inputs, and the span and load show as sliders in Input mode; everything else is calculation and text. You can copy it into the app, or open the template and edit it.</p>
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
    label: 'Retaining wall stability',
    group: 'calculator',
    area: 'geotechnical',
    title: 'Retaining wall stability calculator',
    description: 'Free retaining wall calculator: active earth thrust, overturning and sliding safety factors and base pressure for a cantilever wall, with units.',
    h1: 'Retaining wall stability calculator',
    schema: [],
    open: 'template-retaining-wall-stability',
    video: V.templates,
    related: ['pad-footing-bearing-pressure-calculator', 'concrete-strength-gain-calculator', 'gable-frame-thrust-calculator', 'simply-supported-beam-calculator', 'engineering-calculation-software-free'],
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
    label: 'Pad footing bearing pressure',
    group: 'calculator',
    area: 'geotechnical',
    title: 'Pad footing bearing pressure calculator',
    description: 'Free pad footing calculator: bearing pressure under a column load and moment, eccentricity, uplift and the check against an allowable soil pressure.',
    h1: 'Pad footing bearing pressure calculator',
    schema: [],
    open: 'template-pad-footing-bearing',
    video: V.templates,
    related: ['retaining-wall-stability-calculator', 'column-buckling-calculator', 'concrete-strength-gain-calculator', 'simply-supported-beam-calculator', 'engineering-calculation-software-free'],
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
    label: 'Pipe flow, Darcy-Weisbach',
    group: 'calculator',
    area: 'hydraulics',
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
    label: 'Pump operating point',
    group: 'calculator',
    area: 'hydraulics',
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
    label: 'Open Mathcad files',
    group: 'alternative',
    title: 'Open .xmcd and .mcdx files without Mathcad, free',
    description: 'Open Mathcad .xmcd and .mcdx worksheets without Mathcad: they convert to a live calculation sheet in your browser, and nothing is uploaded.',
    h1: 'Open Mathcad .xmcd and .mcdx files without Mathcad',
    schema: [],
    video: V.importing,
    related: ['mathcad-alternative', 'calcpad-online', 'engineering-calculation-software-free', 'excel-alternative-for-engineering-calculations'],
    howto: 'How to open a Mathcad worksheet without Mathcad',
    runs: {},
    lead: `<p>No Mathcad licence, and a worksheet you need to read? CivilKit Calcs opens Mathcad worksheet files (<code>.xmcd</code> and <code>.mcdx</code>) and converts them into a live calculation sheet in your browser, which its own engine then calculates. Nothing is uploaded: the conversion runs inside the page, and it is free. This page says what to expect, including what does not convert.</p>
<p>The importer is a work in progress. It is a translator, not a guarantee. Plan on checking every imported worksheet against the original, line by line.</p>`,
    sections: [
        ['How to import a worksheet', `<ol>
<li><a href="/calcs/">Open CivilKit Calcs</a> in a new tab.</li>
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
        ['Do I need Mathcad installed?', 'No. The file is read and converted by CivilKit Calcs itself, in the browser. You do not need Mathcad or any other program.'],
        ['Which file types can it import?', 'The <code>.xmcd</code> and <code>.mcdx</code> worksheet formats. Other file types are rejected with a message.'],
        ['Will every worksheet convert completely?', 'No. Worksheets that use add-ins, embedded objects, symbolic evaluation or unsupported plots will have gaps, and the importer says where.'],
        ['Does it keep my original file?', 'Yes. It reads the file and does not change it.'],
        ['Is my worksheet sent to a server?', 'No. It is converted in your browser. We checked the import code for network requests that carry the file and found none.'],
        ['Do the numbers match the original?', 'Constants are copied from the source file and the engine recomputes the rest. Check on your own files: do not assume they match.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 11 cantilever */
const CANT = 'Template cantilever tip load.sheet';
pages.push({
    slug: 'cantilever-beam-calculator',
    label: 'Cantilever beam',
    group: 'calculator',
    area: 'structural',
    title: 'Cantilever beam calculator: tip load and deflection',
    description: 'Free cantilever calculator with units: moment, shear, bending stress, tip deflection and rotation for a point load at the free end, with a moment diagram.',
    h1: 'Cantilever beam calculator with a tip load',
    schema: [],
    open: 'template-cantilever-tip-load',
    video: V.templates,
    related: ['simply-supported-beam-calculator', 'column-buckling-calculator', 'bending-moment-shear-force-diagram', 'beam-deflection-formula-derivation', 'engineering-calculations-with-units'],
    howto: 'How to check a cantilever with a point load at the free end',
    runs: {
        base: { sample: CANT },
        long: { sample: CANT, set: { L: 'L:     ? 4 m [0.5 .. 6 step 0.1] | length' } },
        timber: { sample: CANT, set: { mat: 'mat:   ? [Structural steel = 1, Aluminium = 2, *Timber = 3] | material' } },
    },
    lead: `<p>This template checks a cantilever: a beam fixed at one end and free at the other, with a point load at the free end and, if you want it, the beam's own weight. It reports the moment and shear at the support, the bending stress, the tip deflection and rotation, and compares the stress and the deflection with limits. The section is a solid rectangle and the material comes from a drop-down. It opens as a fill-in form, with the length and the load as sliders.</p>`,
    sections: [
        ['What it computes', `<p>You give the length, the tip load, the width and depth of the section, the material, a deflection limit (the length over 90, 180 or 360) and whether to include the own weight. The sheet works out the section properties, adds the own weight as a uniform load, and reports the moment and shear at the fixed end, the bending stress, the tip deflection and rotation, and two utilisations, with a plain sentence under each saying whether it passes. A moment diagram is plotted along the length.</p>`],
        ['The formulas', `<ul>
<li>Section: A = b d, I = b d^3 / 12, Z = b d^2 / 6.</li>
<li>Own weight w = density x A x g, when the box is ticked.</li>
<li>Moment at the fixed end M = P L + w L^2 / 2. Shear at the fixed end V = P + w L.</li>
<li>Bending stress sigma = M / Z.</li>
<li>Tip deflection delta = P L^3 / (3 E I) + w L^4 / (8 E I). Tip rotation theta = P L^2 / (2 E I) + w L^3 / (6 E I), in radians.</li>
<li>Limit = L / 90, 180 or 360. Utilisations: sigma / allowable stress, and delta / limit.</li>
</ul>
<p>The materials table holds structural steel, aluminium and timber, each with a modulus, an allowable bending stress and a density. A new material is one more entry in each list.</p>`],
        ['Worked example', `<p>Defaults: a 2.5 m cantilever carrying 8 kN at the tip, a 100 mm by 200 mm section in structural steel (E = 200 GPa, allowable stress 165 MPa), a limit of L/90, own weight included. These are the values you get when you open the template. Results from the engine:</p>
${tbl('base', [['Area A', 'A'], ['Second moment of area I', 'I'], ['Section modulus Z', 'Z'], ['Own weight w', 'w'], ['Moment at the fixed end', 'M_max'], ['Shear at the fixed end', 'V_max'], ['Bending stress', 'sigma'], ['Tip deflection', 'delta'], ['Tip rotation (radians)', 'theta'], ['Deflection limit', 'delta_lim'], ['Bending utilisation', 'util_b'], ['Deflection utilisation', 'util_d']])}
<p>Both utilisations are well below 1, so the sheet prints "Bending: OK" and "Deflection: OK".</p>`],
        ['Change the length or the material', `<p>With the length raised to 4 m the moment grows to [[long:M_max]] and the tip deflection to [[long:delta]]. Deflection grows with the cube of the length for the point load and with the fourth power for the own weight, so 60 percent more length gives more than four times the deflection. Both checks still pass, at [[long:util_b]] for bending and [[long:util_d]] for deflection.</p>
<p>Back at 2.5 m, switching the material to timber gives a stress of [[timber:sigma]], a little lower than for steel because timber is lighter, against an allowable stress of only 12 MPa. The tip deflection is [[timber:delta]] against a limit of [[timber:delta_lim]]. The utilisations of [[timber:util_b]] and [[timber:util_d]] both fail, and the sheet says to enlarge the section or change the material.</p>`],
        ['What this is, and is not', notCode('The cantilever is fully fixed at the support, the section is a solid rectangle, the load is one point load at the free end plus the optional own weight, and stress and deflection are compared with values you choose. There is no lateral torsional buckling, shear or connection check, and no load combination.')],
        ['The sheet source', `<p>This is the template exactly as it ships. The lines with a question mark are the inputs; the length and the load show as sliders in Input mode.</p>
{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('Length, load, section width and depth, material, deflection limit and own weight are the inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('Length, load, section width and depth, material, deflection limit and own weight are the inputs.'),
    faq: [
        ['What is the maximum moment in a cantilever with a tip load?', 'P L at the fixed end, where P is the load and L the length. With an own weight of w per metre it is P L + w L^2 / 2. The moment falls to zero at the free end.'],
        ['What is the tip deflection formula?', 'P L^3 / (3 E I) for a point load at the free end, plus w L^4 / (8 E I) for a uniform load such as the own weight.'],
        ['Can I use an I-section or a hollow section?', 'Yes, by editing the section lines: replace A, I and Z with the values of your section. The rest of the sheet uses those names.'],
        ['Does it check the support or the connection?', 'No. It checks bending stress and deflection only. It is a textbook calculation, not a design-code check.'],
        ['Can I save the result?', 'Yes. Use Report, then Print PDF, Word or Excel. The files use the values you entered.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 12 column */
const COL = 'Template column slenderness.sheet';
const colK = (pick) => `k:    ? [${[['Pinned at both ends', 1], ['Fixed and pinned', 0.7], ['Fixed at both ends', 0.5], ['Fixed and free', 2]].map(([n, v]) => `${n === pick ? '*' : ''}${n} = ${v}`).join(', ')}] | end conditions`;
pages.push({
    slug: 'column-buckling-calculator',
    label: 'Column buckling',
    group: 'calculator',
    area: 'structural',
    title: 'Column buckling calculator: slenderness and Euler load',
    description: 'Slenderness and buckling of a solid rectangular column: end conditions, the Euler or parabolic critical stress and the allowed load, as a form you can edit.',
    h1: 'Column buckling and slenderness calculator',
    schema: [],
    open: 'template-column-slenderness',
    video: V.templates,
    related: ['cantilever-beam-calculator', 'section-principal-axes-calculator', 'simply-supported-beam-calculator', 'pad-footing-bearing-pressure-calculator'],
    howto: 'How to check a rectangular column for buckling',
    runs: {
        base: { sample: COL },
        fixed: { sample: COL, set: { k: colK('Fixed at both ends') } },
        free: { sample: COL, set: { k: colK('Fixed and free') } },
    },
    lead: `<p>This template checks a solid rectangular column in compression for buckling. From the size, the length and the end conditions it works out the slenderness, decides whether the column buckles elastically or inelastically, and gives the critical stress, the critical load and the load allowed with a factor of safety. The end conditions and the material are drop-downs, so the effect of fixing an end is one click away.</p>`,
    sections: [
        ['What it computes', `<p>The inputs are the width and depth of the section, the length, the end conditions (pinned at both ends, fixed and pinned, fixed at both ends, or fixed and free), the material, the axial load and a factor of safety. The sheet reports the effective length, the smallest radius of gyration, the slenderness ratio and the transition slenderness, a sentence saying whether the column is short, intermediate or slender, the Euler stress and load, the critical stress and load, the allowed load and the utilisation.</p>`],
        ['The formulas', `<ul>
<li>Effective length L_e = k L, with k = 1, 0.7, 0.5 or 2 for the four end conditions.</li>
<li>Smallest radius of gyration r = b_min / sqrt(12) for a solid rectangle, where b_min is the smaller side.</li>
<li>Slenderness lambda = L_e / r. Transition slenderness lambda_c = pi sqrt(2 E / f_y).</li>
<li>Above lambda_c the column buckles elastically (Euler): sigma_cr = pi^2 E / lambda^2, and the Euler load is P_E = pi^2 E I_min / L_e^2.</li>
<li>At or below lambda_c the parabolic (Johnson) formula applies: sigma_cr = f_y (1 - f_y lambda^2 / (4 pi^2 E)).</li>
<li>Critical load P_cr = sigma_cr A, allowed load P_allow = P_cr / FS, utilisation = N / P_allow.</li>
</ul>
<p>The two curves meet at lambda_c, where both give half the yield stress, so the critical stress never jumps. A slenderness of 50 or less is reported as short, where crushing is likely to govern.</p>`],
        ['Worked example', `<p>Defaults: a 60 mm by 100 mm steel bar (E = 200 GPa, yield stress 250 MPa), 2.4 m long and pinned at both ends, carrying 100 kN with a factor of safety of 2. Results from the engine:</p>
${tbl('base', [['Area A', 'A'], ['Smaller second moment of area', 'I_min'], ['Radius of gyration r', 'r'], ['Effective length', 'L_e'], ['Slenderness', 'lambda'], ['Transition slenderness', 'lambda_c'], ['Euler stress', 'sigma_E'], ['Critical stress', 'sigma_cr'], ['Critical load', 'P_cr'], ['Allowed load', 'P_allow'], ['Utilisation', 'util']])}
<p>The slenderness of [[base:lambda]] is above the transition value of [[base:lambda_c]], so the column is slender and the Euler formula governs. The allowed load of [[base:P_allow]] is well above the 100 kN applied, and the sheet prints OK.</p>`],
        ['Change the end conditions', `<p>Fixing both ends halves the effective length to [[fixed:L_e]]. The slenderness drops to [[fixed:lambda]], below the transition, so the sheet switches to the parabolic formula: the critical stress is [[fixed:sigma_cr]] instead of the Euler value of [[fixed:sigma_E]], which would be above the yield stress. The allowed load rises to [[fixed:P_allow]].</p>
<p>A column fixed at the base and free at the top, like a post or a flagpole, doubles the effective length to [[free:L_e]] instead. The slenderness becomes [[free:lambda]], the allowed load falls to [[free:P_allow]], and the utilisation of [[free:util]] fails, so the sheet suggests a bigger section, a shorter column or fixed ends.</p>`],
        ['What this is, and is not', notCode('The column is a solid rectangle, straight and loaded through its centroid, and it buckles about its weaker axis. The material values are round examples. There is no allowance for initial crookedness, eccentric load, residual stress or bending combined with compression, and the parabolic curve is a classical textbook transition, not a column curve from any standard.')],
        ['The sheet source', `<p>This is the template exactly as it ships. The lines with a question mark are the inputs.</p>
{{code:base}}`],
        ['How to use it', `<ol>
${taskSteps('Size, length, end conditions, material, axial load and factor of safety are the inputs.').map(([n, t]) => `<li><strong>${n}.</strong> ${t}</li>`).join('\n')}
</ol>`],
    ],
    steps: taskSteps('Size, length, end conditions, material, axial load and factor of safety are the inputs.'),
    faq: [
        ['What is the Euler buckling load?', 'The axial load at which a perfectly straight, elastic column buckles: P_E = pi^2 E I / L_e^2, with L_e the effective length. Real columns fail below it, which is why the sheet divides by a factor of safety and uses the parabolic formula for stockier columns.'],
        ['Which effective length factors does it use?', '1 for pinned at both ends, 0.7 for fixed and pinned, 0.5 for fixed at both ends and 2 for fixed at the base and free at the top. These are the theoretical values; edit the list if you use others.'],
        ['Why does it switch between two formulas?', 'For a stocky column the Euler stress would exceed the yield stress. Below the transition slenderness the parabolic formula gives a critical stress that never exceeds yield and meets the Euler curve at half the yield stress.'],
        ['Can I use a hollow or I-section?', 'Edit the section lines: give A, the smaller second moment of area and the radius of gyration of your section. The rest of the sheet uses those names.'],
        ['Is it a design-code check?', 'No. It is a textbook calculation and names no standard.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 13 deflection derivation */
pages.push({
    slug: 'beam-deflection-formula-derivation',
    label: 'Beam deflection formula, derived',
    group: 'guide',
    title: 'Beam deflection formula 5wL^4/384EI, derived',
    description: 'Derive the midspan deflection of a simply supported beam under a uniform load step by step with symbolic integration, then check it against a plotted curve.',
    h1: 'Deriving the simply supported beam deflection formula',
    schema: [],
    open: 'sheet-beam-deflection-symbolic',
    video: V.symbolic,
    related: ['simply-supported-beam-calculator', 'cantilever-beam-calculator', 'bending-moment-shear-force-diagram', 'engineering-calculations-with-units'],
    runs: {
        base: { sample: 'Beam deflection derived symbolically.sheet' },
    },
    lead: `<p>Every structural engineer knows that a simply supported beam under a uniform load deflects 5 w L^4 / (384 E I) at midspan. This page derives that formula from the bending equation in four exact steps with the Symbolic dialog of CivilKit Calcs, then puts numbers in and lets the engine check that the formula and the derived deflection curve agree.</p>`,
    sections: [
        ['The starting point', `<p>The beam has span L, uniform load w and bending stiffness E I. Taking moments about a support, the bending moment at a distance x is M(x) = w L x / 2 - w x^2 / 2. With the deflection y measured in the direction of the load, the bending equation is E I y'' = -M(x). Integrating twice gives the deflection curve, and the two supports fix the two constants of integration.</p>`],
        ['The four steps', `<ol>
<li><strong>Integrate</strong> -(w L x / 2 - w x^2 / 2) / (E I) with respect to x. The dialog returns the slope, -L w x^2 / (4 E I) + w x^3 / (6 E I), with no constant: add C1 yourself.</li>
<li><strong>Integrate</strong> again, giving -L w x^3 / (12 E I) + w x^4 / (24 E I) + C1 x, and add C2.</li>
<li>The supports give y(0) = 0 and y(L) = 0. <strong>Solve</strong> those two equations for C1 and C2: C1 = L^3 w / (24 E I) and C2 = 0.</li>
<li><strong>Substitute</strong> x = L / 2 into the curve and simplify: 5 L^4 w / (384 E I).</li>
</ol>
<p>Each result is exact: fractions stay fractions and symbols stay symbols, so there is no rounding to hide a slip. The dialog also differentiates and evaluates, so the slope at a support or the deflection for given numbers can be checked on the spot.</p>`],
        ['Numbers, and a check', `<p>The sheet then puts in w = 12 kN/m, L = 6 m, E = 200 GPa and I = 250,000,000 mm^4, and computes the deflection twice: once from the formula, once from the curve of step 3 evaluated at midspan. Results from the engine:</p>
${tbl('base', [['Midspan deflection from the formula', 'delta'], ['The derived curve at midspan', 'y_mid'], ['Ratio of the two', 'ratio']])}
<p>The ratio is [[base:ratio]], so the derivation and the textbook formula agree. The sheet also plots the curve along the span, with its sign flipped so the beam sags downward on the page.</p>`],
        ['What the dialog can and cannot do', `<p>The symbolic side is deliberately small. It simplifies, expands, factors, differentiates, integrates a small class of integrands, solves one equation of degree 1 or 2 or a linear system of up to four unknowns, and substitutes. It knows sin, cos, ln, exp and sqrt, has no complex numbers and treats symbols as positive real numbers. A result that does not fit its exact arithmetic is refused with a message rather than rounded.</p>
<p>The same four steps apply to any load whose bending moment is a polynomial in x: change M(x) and the support conditions. A cantilever fixed at x = 0, for example, has y(0) = 0 and y'(0) = 0. For anything beyond the dialog, use the numerical tools of the engine: root finding, numerical integration and linear solvers.</p>`],
        ['The sheet source', `<p>The sample sheet holds the steps as typeset formulas and the check as quantity lines. Open it with the button on this page to see it calculated.</p>
{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['Where does 5/384 come from?', 'From integrating the bending equation twice and applying the support conditions. The curve is y(x) = w x (L^3 - 2 L x^2 + x^3) / (24 E I), and at x = L/2 that is 5 w L^4 / (384 E I).'],
        ['What is the slope at the supports?', 'w L^3 / (24 E I), the value of C1: the derivative of the curve at x = 0.'],
        ['Does the Symbolic dialog need an internet connection?', 'No. It loads the first time you run it and then works offline, like the rest of the app.'],
        ['Can I put the derived formula into a calculation?', 'Yes. Insert into worksheet writes the result as a typeset formula or as a quantity line, which the engine calculates once every symbol has a value.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 14 moment and shear diagrams */
pages.push({
    slug: 'bending-moment-shear-force-diagram',
    label: 'Moment and shear diagrams',
    group: 'calculator',
    area: 'structural',
    title: 'Bending moment and shear force diagrams, plotted',
    description: 'Plot bending moment and shear force along a simply supported beam from a moment function, worked out at sixths of the span in a loop, with real units.',
    h1: 'Bending moment and shear force diagrams',
    schema: [],
    open: 'diagram-moment-chart',
    video: { ...V.plots, seconds: 80 },
    related: ['simply-supported-beam-calculator', 'cantilever-beam-calculator', 'beam-deflection-formula-derivation', 'gable-frame-thrust-calculator'],
    runs: {
        base: { sample: 'Diagram moment and shear chart.sheet' },
        pts: { sheet: `# Moment and shear at points

L: 6 m
w: 12 kN/m
M(x): w * x * (L - x) / 2
V(x): w * (L / 2 - x)
M_0: M(0 m) -> kN*m
M_1: M(1 m) -> kN*m
M_2: M(2 m) -> kN*m
M_3: M(3 m) -> kN*m
V_0: V(0 m) -> kN
V_1: V(1 m) -> kN
V_2: V(2 m) -> kN
V_3: V(3 m) -> kN
` },
        plot: { sample: 'Plot.sheet' },
    },
    lead: `<p>A bending moment diagram and a shear force diagram show how the internal forces vary along a beam. In a calculation sheet you write the moment and the shear as functions of the position x, and the page tabulates them and draws them. This page does it for a simply supported beam under a uniform load in two ways: a loop that works out values at sixths of the span with their units, and a plot line that draws the curve.</p>`,
    sections: [
        ['The functions', `<p>For a simply supported span L under a uniform load w:</p>
<ul>
<li>Bending moment M(x) = w x (L - x) / 2: a parabola, zero at both supports and largest at midspan, where it is w L^2 / 8.</li>
<li>Shear force V(x) = w (L / 2 - x): a straight line from w L / 2 at the left support to minus that at the right.</li>
</ul>
<p>The shear is the slope of the moment, so it passes through zero exactly where the moment peaks.</p>`],
        ['Values along the span', `<p>With L = 6 m and w = 12 kN/m, the engine gives these values for the first half of the span. The second half mirrors it: the moment is symmetric and the shear changes sign.</p>
<table>
<thead><tr><th>x</th><th>Bending moment M</th><th>Shear force V</th></tr></thead>
<tbody>
<tr><td>0 m (support)</td><td>[[pts:M_0]]</td><td>[[pts:V_0]]</td></tr>
<tr><td>1 m</td><td>[[pts:M_1]]</td><td>[[pts:V_1]]</td></tr>
<tr><td>2 m</td><td>[[pts:M_2]]</td><td>[[pts:V_2]]</td></tr>
<tr><td>3 m (midspan)</td><td>[[pts:M_3]]</td><td>[[pts:V_3]]</td></tr>
</tbody>
</table>
<p>The sample sheet writes the two functions once and works through the span in a loop, printing the moment and the shear at each sixth with units, then draws the moments as an XY chart and plots the moment curve from the function:</p>
{{code:base}}
<p>The bars and the line of the XY chart are typed into the diagram text, so they do not follow a change of span or load. The loop values and the plot do, because they are calculated from the functions.</p>`],
        ['A plot from one line', `<p>The shortest form is a plot line. This sheet uses plain numbers rather than units, defines the moment and plots it over the span:</p>
{{code:plot}}
<p>It reports a peak moment of [[plot:Mmax]] (in kN·m, because the numbers are in kN and m) and draws the parabola. Plots are drawn in the page, so changing L or w redraws the curve at once.</p>`],
        ['Other beams and loads', `<p>Change the functions to change the beam. A cantilever with a point load P at its free end has M(x) = P (L - x), measured from the fixed end; the <a href="/calcs/learn/cantilever-beam-calculator/">cantilever calculator</a> plots exactly that. Several loads on one span add up: write the moment of each and plot their sum. A point load part way along the span needs a moment function that changes at the load, which a sheet can write with a condition.</p>
<p>These sheets draw the diagrams of the functions you write; they do not analyse a structure. For a continuous beam or a frame, find the moments with an analysis first. The samples include a three-span beam solved by the stiffness method in a Python block, with its diagrams drawn by an interactive chart.</p>`],
    ],
    steps: null,
    faq: [
        ['What is the maximum bending moment of a simply supported beam with a uniform load?', 'w L^2 / 8 at midspan. For a 6 m span under 12 kN/m that is [[pts:M_3]].'],
        ['Where is the shear force zero?', 'At midspan for a symmetric load, which is where the moment is largest. In general the moment peaks where the shear changes sign.'],
        ['Can I plot the shear as well as the moment?', 'Yes. Add a second plot line for V(x) under the first.'],
        ['Do the diagrams follow a change of span or load?', 'The loop values and the plot do, because they are calculated from the functions. The bars of the XY chart in the sample are typed in, so update them by hand or leave the chart out.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 15 principal stresses */
pages.push({
    slug: 'principal-stress-calculator',
    label: 'Principal stresses',
    group: 'calculator',
    area: 'structural',
    title: 'Principal stress calculator for 2D and 3D stress',
    description: "Principal stresses and directions of a plane or 3D stress tensor from eigenvalues and eigenvectors, checked against Mohr's circle, with units.",
    h1: 'Principal stress calculator',
    schema: [],
    open: 'sheet-principal-stresses',
    related: ['section-principal-axes-calculator', 'shear-frame-natural-frequencies', 'engineering-calculations-with-units', 'cantilever-beam-calculator'],
    runs: {
        base: { sample: 'Principal stresses.sheet', append: 's_a: sigma_3D.1 | smallest\ns_b: sigma_3D.2 | middle\ns_c: sigma_3D.3 | largest\n' },
    },
    lead: `<p>The principal stresses at a point are the normal stresses on the planes where the shear stress vanishes. Mathematically they are the eigenvalues of the stress tensor, and the principal directions are its eigenvectors. This sheet finds both with one function call each, for a plane stress state and for a general 3D one, with the stresses in MPa, and checks the plane result against Mohr's circle.</p>`,
    sections: [
        ['Plane stress', `<p>The stress tensor is written as a 2 by 2 matrix with units: S = [sigma_x, tau_xy; tau_xy, sigma_y]. The function <code>eigenvals(S)</code> returns the principal stresses in ascending order, so the major one is the second. <code>eigenvecs(S)</code> returns unit vectors along the principal directions, one per row in the same order. For sigma_x = 80 MPa, sigma_y = -40 MPa and tau_xy = 25 MPa, the engine gives:</p>
${tbl('base', [['Major principal stress', 'sigma_1'], ['Minor principal stress', 'sigma_2'], ['Maximum in-plane shear stress', 'tau_max'], ['Angle of the major direction from x (degrees)', 'theta_p']])}`],
        ["The Mohr's circle check", `<p>The same results by the classical construction: the circle's centre is the mean normal stress (sigma_x + sigma_y) / 2, its radius is R = sqrt(((sigma_x - sigma_y) / 2)^2 + tau_xy^2), the principal stresses are the centre plus and minus the radius, and the angle is atan(2 tau_xy / (sigma_x - sigma_y)) / 2.</p>
${tbl('base', [['Centre of the circle', 'sigma_avg'], ['Radius R', 'R'], ['Major principal stress from the circle', 'sigma_1c'], ['Minor principal stress from the circle', 'sigma_2c'], ['Angle from the circle (degrees)', 'theta_pc']])}
<p>The eigenvalue and the circle agree to the decimals shown, and the maximum shear stress is the radius of the circle.</p>`],
        ['A 3D stress state', `<p>For a general stress state the tensor is 3 by 3, and the same two calls work. With T = [60, 20, -10; 20, -30, 15; -10, 15, 40] MPa the three principal stresses are [[base:s_c]], [[base:s_b]] and [[base:s_a]]. Mohr's circle has no simple 3D form, so the check here is an invariant: the trace of the tensor, [[base:I_1]], equals the sum of the principal stresses, [[base:I_1c]], as it must for any rotation of the axes.</p>`],
        ['Why eigenvalues give principal stresses', `<p>Rotating the axes changes the components of the stress tensor but not the state of stress. In the principal axes the tensor is diagonal: no shear, only normal stresses. Finding those axes is the eigenvalue problem of the matrix. Because the stress tensor is symmetric, its eigenvalues are real and its eigenvectors are at right angles to each other, which is why the principal planes are always perpendicular.</p>`],
        ['The sheet source', `<p>The sample as it ships (this page adds three lines that pick out the 3D principal stresses):</p>
{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['What are principal stresses?', 'The normal stresses on the planes through a point where the shear stress is zero. There are two in plane stress and three in 3D, and they are the largest and smallest normal stresses at that point.'],
        ['How is the maximum shear stress found?', "In plane stress it is half the difference of the principal stresses, the radius of Mohr's circle. Here that is [[base:tau_max]]."],
        ['Does it handle units?', 'Yes. The tensor is entered in MPa and the principal stresses come back in MPa. Enter the components in kPa or psi and the results follow.'],
        ['Which way is the angle measured?', 'From the x axis to the major principal direction, in degrees with the default angle setting. A direction has no sign, so an eigenvector and its negative describe the same axis.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 16 principal axes */
pages.push({
    slug: 'section-principal-axes-calculator',
    label: 'Principal axes of a section',
    group: 'calculator',
    area: 'structural',
    title: 'Principal axes of an unsymmetric section, calculated',
    description: 'Strong and weak axes of an unsymmetric cross-section from the second moments of area by eigenvalues, checked against the closed-form result.',
    h1: 'Principal axes of a cross-section',
    schema: [],
    open: 'sheet-principal-axes',
    related: ['principal-stress-calculator', 'column-buckling-calculator', 'cantilever-beam-calculator', 'simply-supported-beam-calculator'],
    runs: {
        base: { sample: 'Principal axes of a section.sheet', append: 'I_sum: I_1 + I_2 -> mm^4 | the trace after rotation\nI_xy_sum: Ixx + Iyy -> mm^4 | the trace before rotation\n' },
    },
    lead: `<p>For a symmetric section the principal axes are its axes of symmetry. For an angle, a Z-section or any other unsymmetric shape they are rotated, and a beam loaded about its geometric axes bends about both. This sheet finds the strong and weak axes from the three second moments of area, Ixx, Iyy and Ixy, as the eigenvalues and eigenvectors of a 2 by 2 matrix, and checks them against the closed-form result.</p>`,
    sections: [
        ['The idea', `<p>The second moments of area about the centroidal x and y axes form a symmetric matrix, [Ixx, Ixy; Ixy, Iyy]. Its eigenvalues are the principal second moments: the largest (the strong axis) and the smallest (the weak axis) values over every rotation of the axes. Its eigenvectors point along those axes. It is the same mathematics as <a href="/calcs/learn/principal-stress-calculator/">principal stresses</a>, which is why the two sheets look alike.</p>`],
        ['Worked example', `<p>The sample takes Ixx = 8.2 x 10^6 mm^4, Iyy = 3.1 x 10^6 mm^4 and a product of inertia Ixy = -2.4 x 10^6 mm^4. Results from the engine:</p>
${tbl('base', [['Strong axis, I1', 'I_1'], ['Weak axis, I2', 'I_2'], ['Angle from the eigenvector (degrees)', 'theta_p'], ['Mean of Ixx and Iyy', 'I_avg'], ['Radius R', 'R'], ['I1 by the closed form', 'I_1c'], ['I2 by the closed form', 'I_2c']])}
<p>The closed form is the mean plus and minus R, with R = sqrt(((Ixx - Iyy) / 2)^2 + Ixy^2), and it agrees with the eigenvalues. A second check: the sum I1 + I2 is [[base:I_sum]], equal to Ixx + Iyy, [[base:I_xy_sum]], because the trace of the matrix does not change when the axes rotate.</p>`],
        ['Signs and conventions', `<p>The sign of the product of inertia depends on the quadrants the section occupies and on the convention your textbook uses for it, and the matrix can be written with +Ixy or -Ixy. The principal values are the same either way; only the sign of the angle changes. Check the direction of the strong axis on a sketch of the section before you use it.</p>`],
        ['Using the result', `<p>Resolve a bending moment into components about the two principal axes: each component bends the section about its own axis only, and the stresses from the two add. For buckling, the weak axis governs: a column check should use I2, not the smaller of Ixx and Iyy, which for an unsymmetric section is larger than I2. The <a href="/calcs/learn/column-buckling-calculator/">column buckling calculator</a> uses the smaller second moment of a rectangle; for another shape, give it I2.</p>`],
        ['The sheet source', `<p>The sample as it ships (this page adds two lines for the trace check):</p>
{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['What is the product of inertia?', 'Ixy is the integral of x y over the area, about the centroid. It is zero when either axis is an axis of symmetry, which is why symmetric sections need no rotation.'],
        ['Why do I1 and I2 add up to Ixx + Iyy?', 'The trace of the matrix is invariant under rotation of the axes. It is a quick check on any result.'],
        ['Can it compute Ixx, Iyy and Ixy from a shape?', 'Not this sheet: it starts from the three values. Work them out from the parts of the section, each part with its own values plus the parallel axis terms, in lines above it, or take them from a section table.'],
        ['Which axis buckles first?', 'The weak principal axis, with the smaller principal second moment I2.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 17 shear frame */
pages.push({
    slug: 'shear-frame-natural-frequencies',
    label: 'Shear frame frequencies',
    group: 'calculator',
    area: 'structural',
    title: 'Shear frame natural frequencies and mode shapes',
    description: 'Natural frequencies and mode shapes of a two storey shear frame from the generalised eigenvalue problem K phi = w^2 M phi, worked with units.',
    h1: 'Natural frequencies of a shear frame',
    schema: [],
    open: 'sheet-shear-frame-frequencies',
    related: ['principal-stress-calculator', 'section-principal-axes-calculator', 'gable-frame-thrust-calculator', 'engineering-calculations-with-units'],
    runs: {
        base: { sample: 'Shear frame natural frequencies.sheet', append: 'r_1: phi_1.1 | first floor, mode 1\nr_2: phi_2.1 | first floor, mode 2\n' },
    },
    lead: `<p>A shear frame is the simplest model of a building swaying sideways: rigid floors, each with its mass, joined by storeys that each have a lateral stiffness. Its natural frequencies and mode shapes come from the generalised eigenvalue problem K phi = omega^2 M phi. This sheet solves it for a two storey frame in one function call, with the masses in tonnes and the stiffnesses in kN/m, and checks that the modes are orthogonal.</p>`,
    sections: [
        ['The model', `<p>The first floor has a mass of 30 t and the roof 20 t. The lower storey has a lateral stiffness of 40,000 kN/m and the upper one 30,000 kN/m. The stiffness matrix is K = [k1 + k2, -k2; -k2, k2] and the mass matrix is diagonal. With two arguments, <code>eigenvals(K, M)</code> solves the generalised problem directly: K must be symmetric and M symmetric positive definite. Each eigenvalue is omega^2 in the units of K over M, so kN/m divided by tonnes comes out per second squared without any conversion by hand.</p>`],
        ['Frequencies', `<p>The natural frequency is f = omega / (2 pi) and the period is its inverse. Results from the engine:</p>
${tbl('base', [['First natural frequency', 'f_1'], ['Second natural frequency', 'f_2'], ['Fundamental period', 'T_1']])}`],
        ['Mode shapes', `<p><code>eigenvecs(K, M)</code> returns one mode per row, in the order of the frequencies, each scaled so that phi^T M phi = 1. The sheet scales them again so the roof moves 1. In the first mode the first floor then moves [[base:r_1]] in the same direction: the whole frame sways one way. In the second it moves [[base:r_2]]: the floors move against each other.</p>
<p>The modes are orthogonal through the mass matrix, so the product phi_1^T M phi_2 is zero; the sheet prints [[base:check]]. That is a useful check that the matrices were entered correctly.</p>`],
        ['More storeys', `<p>Each extra floor adds a row and a column to K and M: K stays tridiagonal, with k_i + k_(i+1) on the diagonal and -k_(i+1) beside it, and M stays diagonal. The same two calls then give all the frequencies and modes. For many storeys, build the matrices in a loop rather than by hand.</p>`],
        ['What this is, and is not', notCode('The floors are rigid, the masses are lumped at the floors, there is no damping, and the storey stiffness is a number you supply (for example 12 E I / h^3 for each column fixed at both ends). There is no response spectrum, time history or seismic load calculation.')],
        ['The sheet source', `<p>The sample as it ships (this page adds two lines that pick out the first floor of each mode):</p>
{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['What is a shear frame?', 'A frame model in which the floors are rigid and the columns bend in double curvature, so each storey acts as a lateral spring between two masses.'],
        ['Why are there as many frequencies as storeys?', 'Each floor has one degree of freedom, its sideways movement, so a two storey frame has two modes.'],
        ['What units does it need?', 'Any consistent ones, because the engine converts. Masses in tonnes and stiffnesses in kN/m give eigenvalues per second squared, and frequencies in Hz after dividing omega by 2 pi.'],
        ['Can it include damping or a time history?', 'Not this sheet. It finds frequencies and mode shapes only.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 18 gable frame */
pages.push({
    slug: 'gable-frame-thrust-calculator',
    label: 'Gable frame thrust',
    group: 'calculator',
    area: 'structural',
    title: 'Gable frame thrust: pinned vs rigid ridge',
    description: 'Horizontal thrust of a symmetric pitched roof frame for a pinned ridge and a rigid ridge, and why the common shortcut overstates it.',
    h1: 'Horizontal thrust of a gable frame',
    schema: [],
    open: 'sheet-gable-frame-thrust',
    related: ['bending-moment-shear-force-diagram', 'shear-frame-natural-frequencies', 'simply-supported-beam-calculator', 'retaining-wall-stability-calculator'],
    runs: {
        base: { sample: 'Gable frame thrust.sheet' },
    },
    lead: `<p>A pitched roof frame pushes its supports outwards. How hard depends on the ridge: a pin carries no moment, a continuous ridge carries some, and the thrust differs between the two. This sheet works out the vertical reactions and the horizontal thrust for both cases from the roof loads, and shows why a common hand shortcut overstates the thrust.</p>`,
    sections: [
        ['Loads on plan', `<p>The sample frame spans 9.8 m with a rise of 2.8 m to the ridge and a roof pitch of 30 degrees. It carries a strip of roof 4.35 m wide: a dead load of 1.2 kPa measured on the slope and a live load of 0.7 kPa on plan. The dead load is converted to plan area by dividing by the cosine of the pitch, then both become a line load on the frame. Results from the engine:</p>
${tbl('base', [['Dead load on plan', 'g'], ['Line load on the frame', 'w'], ['Vertical reaction at each support', 'V']])}`],
        ['Three thrusts', `<ul>
<li><strong>The shortcut</strong> H = V a / h, with a the horizontal run of one rafter (half the span) and h the rise. It treats the whole reaction as acting at the ridge and ignores the load on the rafter.</li>
<li><strong>Pinned ridge</strong>: moments about the ridge of one half give H = (V a - w a^2 / 2) / h, which is the same as w L^2 / (8 h).</li>
<li><strong>Rigid ridge</strong>, pinned feet and the same stiffness along both rafters: H = 5 w a^2 / (8 h).</li>
</ul>
${tbl('base', [['Shortcut, V a / h', 'H_hand'], ['Pinned ridge', 'H_pin'], ['Pinned ridge, in one line', 'H_pin_check'], ['Rigid ridge', 'H_rigid'], ['Hogging moment at a rigid ridge', 'M_ridge'], ['Rigid over pinned', 'ratio']])}
<p>The shortcut gives exactly twice the pinned value, for any span and any uniform load, because it leaves out the moment of the load on the rafter. The rigid ridge gives five quarters of the pinned value, here a ratio of [[base:ratio]]: the ridge moment it carries pushes the feet further apart.</p>`],
        ['Which one to use', `<p>The ridge as built decides. A joint with little moment capacity behaves close to a pin; a welded or haunched ridge is closer to the rigid case. When a frame model gives the rigid value but the joint is really a pin, release the moment at the ridge node in the model and the thrust drops to the pinned value. The member self weight is not in the loads above, so a model that includes it will show a little more, about two percent for this frame.</p>`],
        ['What this is, and is not', notCode('The frame is symmetric, with pinned feet and a uniform load on plan. The rigid case assumes the same stiffness along both rafters and ignores axial and shear deformation, and the self weight of the members is left out. There is no wind, no unbalanced load, no tie and no member check.')],
        ['The sheet source', `<p>The sample, with its decision diagram:</p>
{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['Why does a pitched roof push its supports out?', 'The rafters lean on each other at the ridge. Under vertical load each rafter tends to rotate about its foot, and unless the feet are tied the supports must resist a horizontal thrust.'],
        ['Is the V a / h shortcut safe?', 'It overstates the thrust, at twice the pinned-ridge value for a uniform load, so it errs on the high side for the supports. It can still mislead a model check or a tie design. Use the formula for the joint you have.'],
        ['How much larger is the thrust with a rigid ridge?', 'With pinned feet and uniform stiffness, five quarters of the pinned value: here [[base:H_rigid]] against [[base:H_pin]].'],
        ['Does it include wind?', 'No. It covers a uniform vertical load only.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 19 concrete strength */
const CONC = 'Diagram concrete strength timeline.sheet';
pages.push({
    slug: 'concrete-strength-gain-calculator',
    label: 'Concrete strength gain',
    group: 'calculator',
    area: 'materials',
    title: 'Concrete strength gain calculator, 1 to 28 days',
    description: 'Strength of concrete at ages from 1 to 28 days from a common strength gain formula, listed on a timeline and plotted. Edit the 28-day strength.',
    h1: 'Concrete strength gain with age',
    schema: [],
    open: 'diagram-concrete-strength',
    related: ['pad-footing-bearing-pressure-calculator', 'retaining-wall-stability-calculator', 'column-buckling-calculator', 'engineering-calculations-with-units'],
    runs: {
        base: { sample: CONC },
        c40: { sample: CONC, replace: [['f_28: 32 MPa', 'f_28: 40 MPa']] },
    },
    lead: `<p>Concrete gains strength quickly in its first days and slowly after that. Site decisions such as stripping edge forms, stressing a slab or removing props depend on the strength at an early age. This sheet estimates it at 1, 3, 7, 14 and 28 days from a common formula for moist-cured concrete, lists the ages on a timeline and plots the curve.</p>`,
    sections: [
        ['The formula', `<p>The strength at an age of t days is f(t) = f_28 t / (4 + 0.85 t), where f_28 is the specified strength at 28 days. The two constants set how fast the strength rises. At 28 days the formula gives 28 / 27.8, about 1.007 times f_28, so the 28 day line reads a little above the specified strength, and the curve keeps rising slowly after that.</p>
<p>The constants describe moist-cured concrete with a normal rate of strength gain. Other cements and other curing have other constants: put yours in the formula line.</p>`],
        ['Worked example', `<p>For a mix with f_28 = 32 MPa the engine gives:</p>
${tbl('base', [['1 day', 'f_1'], ['3 days', 'f_3'], ['7 days', 'f_7'], ['14 days', 'f_14'], ['28 days', 'f_28c']], ['Age', 'Estimated strength'])}
<p>About 70 percent of the specified strength is reached at 7 days and close to 90 percent at 14.</p>`],
        ['A stronger mix', `<p>With f_28 = 40 MPa the values scale in proportion: [[c40:f_3]] at 3 days, [[c40:f_7]] at 7 days and [[c40:f_28c]] at 28. The formula is linear in f_28, so every age keeps the same fraction of the specified strength whatever the mix.</p>`],
        ['The timeline and the plot', `<p>The sample lists the ages on a timeline diagram with an example of what each strength might allow on site: stripping edge forms, light foot traffic, stressing, removing props. Those strengths are typed into the diagram text, so they do not follow a change of f_28; edit the timeline when you change the mix. The plot line under it draws the formula itself, so it always matches the numbers above.</p>`],
        ['What this is, and is not', notCode('The formula is an empirical estimate of the mean strength gain of moist-cured concrete at normal temperatures, not a prediction for your mix: cement type, temperature, curing and admixtures all change the rate, and the activities on the timeline are examples, not limits. Decisions such as stripping or stressing should rest on test results and on the strength your specification requires.')],
        ['The sheet source', `{{code:base}}`],
    ],
    steps: null,
    faq: [
        ['How strong is concrete after 7 days?', 'By this formula about 70 percent of the 28 day strength: [[base:f_7]] for a 32 MPa mix.'],
        ['Why is the 28 day value slightly above the specified strength?', 'The formula gives 28 / (4 + 0.85 x 28), about 1.007 times f_28, at 28 days. It is a fitted curve, not a definition.'],
        ['Can I use different constants?', 'Yes. Edit the f(t) line. The two constants control how quickly the strength rises.'],
        ['Does it account for temperature?', 'No. It assumes normal curing temperatures. Cold weather slows the gain considerably.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 20 export */
const BEND = 'Beam bending.sheet';
pages.push({
    slug: 'export-calculations-to-word-excel-pdf',
    label: 'Export to Word, Excel and PDF',
    group: 'guide',
    title: 'Export engineering calculations to Word, Excel and PDF',
    description: 'Turn a calculation into a Word report with real equations, an Excel workbook with native charts, or a PDF, from the browser. What each format keeps.',
    h1: 'Export calculations to Word, Excel and PDF',
    schema: [],
    open: 'sheet-beam-bending',
    video: V.exporting,
    related: ['excel-alternative-for-engineering-calculations', 'engineering-calculations-with-units', 'mathcad-alternative', 'simply-supported-beam-calculator'],
    runs: {
        base: { sample: BEND },
    },
    lead: `<p>A calculation is only finished when someone else can read it. CivilKit Calcs turns a calculated sheet into a Word document with real equations, an Excel workbook with the workings and native charts, a PDF through the print dialog, or a standalone HTML page. Every file is written in the browser, from the same calculated report you see on screen, and it works offline.</p>`,
    sections: [
        ['Where the exports are', `<p>The <strong>Document</strong> tab has buttons for Print PDF, Word, Excel and Export. The File menu has Print PDF, Export and Export Excel. The Export dialog lists the views you can export (Report, Preview, Input form, Unwrapped), each with its own Save PDF, Save Word and Save HTML buttons, and a list of the plots that downloads each one as an image, or all of them as a ZIP. Report is the usual choice: it applies the values entered in an input form.</p>`],
        ['An example to try', `<p>The beam bending sample is a short check with units:</p>
{{code:base}}
<p>The report shows a moment of [[base:M_star]], a design capacity of [[base:φMs]] and a utilisation of [[base:utilisation]], with the formulas, the substituted numbers and an OK line. Open it with the button on this page and export it to each format to compare.</p>`],
        ['Word', `<ul>
<li>Equations are real Word equations (Office Math): fractions, subscripts, sums, matrices and units, editable in the document.</li>
<li>Headings, paragraphs, lists, links and tables keep their form.</li>
<li>Plots and other drawings are pictures, stored as PNG with the SVG kept beside it.</li>
<li>A formula typeset with LaTeX in a sheet appears as its TeX source in italics, not as a Word equation.</li>
<li>An interactive chart cannot be carried, and a paragraph stands in its place. A matplotlib-style figure from a Python block is carried as a picture.</li>
</ul>
<p>The Word file has been checked in LibreOffice, not in Microsoft Word.</p>`],
        ['Excel', `<p>The workbook mirrors the calculation. The Report sheet holds the report in document order, with every equation as a row: description, name, expression, expression with the numbers put in, value and unit. A Variables sheet lists every variable with its last value and unit, each vector or matrix gets a sheet of its own, an Inputs sheet holds the values from input fields, and a Plot data sheet holds the points of every plot. Each plot becomes a native XY chart on the Report sheet, pointing at the plot data, so you can edit it.</p>
<p>The cells hold calculated values with the expressions as text, not live formulas, so changing a cell does not recalculate the rest. The workbook opens in LibreOffice and a Python reader; it has not been tried in Microsoft Excel.</p>`],
        ['PDF and HTML', `<p><strong>Print PDF</strong> opens the browser's print dialog on the report: choose Save as PDF. The page size, orientation and margins come from the PDF settings. A page break line starts a new page, in the PDF and in Word; the + menu between blocks and the Document tab both insert one.</p>
<p><strong>Save HTML</strong> writes a standalone page with the report, its styles, fonts and pictures. An exported input form is static: it shows the controls, but nothing recalculates behind them.</p>`],
    ],
    steps: null,
    faq: [
        ['Are the Word equations editable?', 'Yes. They are native Word equations, not pictures. A formula typed in LaTeX in a sheet is the exception: it appears as its TeX source.'],
        ['Does the Excel workbook recalculate?', 'No. The cells hold calculated values with the expressions as text. Change the inputs in the sheet and export again.'],
        ['Is my calculation uploaded to make the files?', 'No. The Word and Excel writers run in the page and work offline once they have loaded.'],
        ['Which view should I export?', 'Report, usually. It applies the values entered in an input form.'],
    ],
});

/* ------------------------------------------------------------------------------------------------ 21 units */
pages.push({
    slug: 'engineering-calculations-with-units',
    label: 'Calculations with units',
    group: 'guide',
    title: 'Engineering calculations with units, SI and US',
    description: 'Write calculations where every quantity carries a unit: SI and US customary units, prefixes, unit errors that catch mistakes, and your own units.',
    h1: 'Engineering calculations with real units',
    schema: [],
    open: 'sheet-beam-bending',
    video: V.units,
    related: ['export-calculations-to-word-excel-pdf', 'excel-alternative-for-engineering-calculations', 'principal-stress-calculator', 'simply-supported-beam-calculator'],
    runs: {
        conv: { sheet: `# Converting units

a: 1.23 m + 35 cm + 12 mm -> cm
v: 10 m/s -> km/h
P: 2.5 kN / (50 mm * 200 mm) -> MPa
` },
        us: { sheet: `# US customary units

F: 10 kip -> kN
p: 2000 psi -> MPa
H: 20 ft -> m
` },
        beam: { sample: BEND },
        pct: { sheet: `# A ratio as a percentage

Ed: 245.25 kN
Rd: 355.1 kN
ratio: Ed / Rd
util: ratio -> %
` },
        own: { sheet: `# Units of your own

#unit bag = 25 kg | cement bag
#unit pallet = 40 bag
load: 3 pallet -> t
n: 1000 kg -> bag
` },
    },
    lead: `<p>A spreadsheet stores a bare number and leaves the unit in a column heading. In a CivilKit Calcs sheet every quantity carries its unit: the engine multiplies and divides units, converts a result to the unit you ask for, and stops with a message when a formula adds a length to a time. This page shows how that works, with SI and US customary units, prefixes, percentages and units of your own.</p>`,
    sections: [
        ['Attaching and converting units', `<p>Write the unit after the number, with or without a space. An arrow converts the result to the unit you choose:</p>
{{code:conv}}
<p>The engine gives [[conv:a]], [[conv:v]] and [[conv:P]]. Without the arrow, a sum comes out in the first unit it meets. Any SI prefix works on any SI unit that takes one, so kN, MPa, mm and µm need no definition.</p>`],
        ['US customary units', `<p>US customary units are built in and mix freely with SI ones:</p>
{{code:us}}
<p>The results are [[us:F]], [[us:p]] and [[us:H]]. A few bare names mean different things in the US and UK systems, such as gal and ton; a setting picks one, and an explicit suffix such as gal_US makes a sheet independent of it.</p>`],
        ['Errors that catch mistakes', `<p>Multiplying and dividing different units is normal: a uniform load in kN/m times a span squared gives a moment. In the beam bending sample, 12 kN/m on a 6 m span comes out as [[beam:M_star]], and the yield stress times the section modulus as a design capacity of [[beam:φMs]], both in kN·m because the units said so. Adding units that do not match is a mistake, and the engine stops at it. A sheet with <code>len: x + t</code>, where x is in metres and t in seconds, reports:</p>
<pre><code>Inconsistent units: "m + s".</code></pre>
<p>A unit slip is one of the most common errors in hand calculations. Here it stops the sheet and names the line.</p>`],
        ['Percentages and units of your own', `<p>A ratio has no unit and comes out as a plain number. An arrow to % shows it as a percentage without changing its value: a utilisation of 245.25 kN over 355.1 kN shows as [[pct:util]].</p>
<p>A <code>#unit</code> line defines a unit of your own from units the engine knows:</p>
{{code:own}}
<p>Three pallets come out as [[own:load]] and a tonne of cement as [[own:n]]. Currency can be defined the same way, as a dimensionless unit.</p>`],
        ['Where to find the units', `<p>There are about 400 built-in units: lengths, masses, forces, pressures, energy and power, temperatures, angles and electrical units among them. The Insert tab in the sidebar lists them by group, and the Math tab has a Units palette organised by dimension. The <a href="/calcs/guide/units">Units chapter of the guide</a> has the full rules, including temperatures and angles.</p>`],
    ],
    steps: null,
    faq: [
        ['Does it convert between SI and US units?', 'Yes. Write the unit after the number and ask for the result in any compatible unit with an arrow. For example 2000 psi comes out as [[us:p]].'],
        ['What happens if the units do not match?', 'The line stops with an error that names the units it could not combine, such as "Inconsistent units". Multiplying and dividing different units is fine.'],
        ['Can I define my own units?', 'Yes, with a #unit line: a name, an equals sign and its value in known units.'],
        ['How are ratios shown?', 'As plain numbers. Add an arrow to % to show a ratio as a percentage; the value itself does not change.'],
    ],
});
