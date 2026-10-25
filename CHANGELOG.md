# Changelog

All notable changes are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions are numbered YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are `v26.10.2` and so on.
CalcpadCE's own version is tracked separately in NOTICE.

The app changes are listed under their YY.M.N version below. The three Rust library crates have their own entries
under "Crates" at the end, on plain 0.x semver, and are tagged `<crate-name>-v<version>`
(for example `civilkit-calcs-engine-v0.1.0`).

## [Unreleased]
### Added
- Engine: statistics and data functions. `median`, `stdev` (sample, n - 1), `stdevp` (population, n), `var` and `varp` take any mix of scalars, vectors and matrices like `sum` does; units are converted to the first value's unit, a median or standard deviation keeps it and a variance has its square. A sample statistic of fewer than two values and any statistic of an empty vector is an error, not a number. `cumsum(v)` (running sum), `convolve(a; b)`, `linspace(a; b; n)` (both ends included), `polyfit(x; y; n)` (least squares through the engine's `lsolve`, coefficients in ascending powers, coefficient k carries the unit y / x^k) and `interp(x; xs; ys)` (linear interpolation in a table with increasing `xs`; a value outside the table is an error, never an extrapolation). They are in the editor's autocomplete and help, and the ribbon has the statistics and the vector ones.
- Import worksheet: x-y plots convert. A plot region becomes a `$Plot{...}` command from the expressions stored on its axes: several traces (the last expression of an axis serves the traces that have none of their own, so a single x against the data is a vertical marker line), the unit typed beside an axis, a range variable of the sheet as the plot variable with indexed arrays read by interpolation (`line(i + 1; x)`), a name nothing defines as the plot variable over the typed domain or the axis range, and whole arrays against each other over the element number. A title and axis labels, when stored, become text lines. The engine's own colours, markers, line styles and axis ranges are used; columns and waterfalls are drawn as lines and log axes as linear, and a `plot-style` note says which were dropped. Other kinds of plot and the older binary plot item stay gaps that name the kind. On the local beam worksheet both plots convert (regions needing attention 14 of 105 to 12), the engine draws them without error; the truss plots still wait on the arrays they plot, and the single `.xmcd` plot is a binary item that is not read.
- Import worksheet: arrays grown element by element now convert. A program that writes `s[j]` with a counter, under a condition, or at an index it computes (a filter that keeps some rows, a reduced matrix built from a mask, `K[ii, jj] <- ...`) starts the array with one zero and extends it with `resize` / `mresize` before a write past its end, so it ends as large as the largest index written. At the sheet level, an element assignment whose index is an expression of range variables (`V[2 (i - 1) + j] := M[i, j]`) is a loop that grows a new array or writes into one that already holds an array (copied first, so a name that shares it keeps its values).
- Import worksheet: a function whose body has loops with run-time bounds, `while` loops, and element writes to a copy of its argument is converted to an expression block. A `for` counts down when its end is below its start, as in the source; an `if` / `else if` / `else` is evaluated once and guards each assignment under it. A function that cannot be converted faithfully (`break`, `continue`, early `return`, a local with no value on some path, an array it grows) stays a gap that says why.
- Engine (deviation D12): a loop nested in an expression block (`$Repeat` or `$While` inside `$Block`) updates the variables of the block around it instead of making private ones, so a function body can accumulate in a loop. Worksheet-level blocks and loop counters are unchanged. DEVIATIONS.md, `calcpad-core/tests/deviations.rs`.
- Engine: `linterp(x; xs; ys)`, linear interpolation in a table with increasing `xs` that continues along the first or last segment outside the table (`interp` is an error there). `x` is converted to the unit of `xs`, the result has the unit of `ys`, and the table needs two points. It is in the editor's autocomplete.
- Import worksheet: programs return the value of the last statement they run, an assignment included. Sequential `if`s (`if a then X`, `if b then Y`), an `if` chain whose arms end in values or assignments, a nested program and a program that ends in an assignment now convert, to one expression when there are no loops and to a statement block when there are. A path that runs no statement with a value is the engine's undefined, never a zero. A program that ends in a loop or an element assignment stays a gap, since the value of that last statement is not read. A name every arm of an `if`/`else` sets has a value after it. The package format's `alsoif` is read as `else if`.
- Import worksheet: `matcol` and `matrow` become `col` and `row` (index shifted from the array origin), `csort` and `rsort` become `sort_rows` and `sort_cols`, `linterp(vx, vy, x)` becomes the engine's `linterp(x; vx; vy)`, `nthRoot(n, x)` becomes `root(x; n)` (the degree comes first in the file), and `%` is the percent unit. The element of anything that is not a name (a call result, a negated array) is `take(i; array)`, which also works in a function body where the temporary it replaces could not be placed. A sum whose limits are `lowerBound` and `upperBound` elements converts.
- Import worksheet: `try expr on error fallback` converts when `expr` can fail only by dividing by zero (names, numbers, sums, products, indexes of names, non-negative integer powers): the engine gives infinity there, so every divisor is tested, `if(d*0 ≠ d; expr; fallback)`. A body with a function call or anything else that could fail in another way is a gap that says so.
- Import worksheet: an element write to an array that an earlier write could not set, or at an index that reads a name a gap defines, is reported as depending on that gap, so `ranked_gaps()` points at the real cause instead of at the element assignments that follow it.

### Changed
- Import worksheet: a unit override that is only a number (`10^(-3)`) divides the value instead of being read as a unit, which the engine rejected.
- Import worksheet: an equation's indent from the page margin is now a spacer that shrinks before the equation does, so in a preview narrower than the page a matrix placed far to the right keeps its full width instead of being squeezed into a thin scroll box.
- A nested `if` result is spaced out: more line height and room between the nested braces, so the lines and braces no longer touch or overlap.
- Import worksheet: the product of two column vectors is `dot(a; b)` (the source takes the scalar product; the engine's `*` of two vectors is element by element), when both are known to be column vectors. An array assigned from another array's name is copied (`*1`), because the engine's `B = A` shares the array.
- Import worksheet: a worksheet function named like an engine function (`Norm`, `Max`, `Sin`: the engine reads function names without regard to case) is written with a `_f` suffix instead of failing in the engine. `tables::ENGINE_FUNCTIONS` lists the names; a test checks it against the engine's tables, so a new engine function must be added to it.

## [26.10.7] - 2026-10-25
### Added
- Import worksheet, fidelity pass: names follow one rule (the tool's own functions upright, the page's own functions italic, so `V_AB(x)` is italic at its definition and its calls), units are bold blue italic in equations and results (light and dark blues, no dot after a number: `100 ft`), `if` and the check `=` are bold, the vector arrow is drawn across the whole operand, sum limits sit closer, equations keep their indent from the page margin, a value beside a wrapping label lines up with its last line and keeps its offset, range-variable tables are typeset with their column of values, a quotient of units in a result is a stacked fraction (`ft^(3/4)` as a small fraction), text in the symbol font reads as Greek (`Σ`), and indentation between formatted runs no longer splits a word. The editor accepts `valMathml` in `#settings` (no warning on an imported sheet) and imported sheets no longer show the unused-name hint.
- Import worksheet: the author's equations are typeset as MathML from the parsed source (stacked fractions, sums and integrals with bounds, subscripts such as `C_(v,i)`, a vector arrow over a whole product, a transpose superscript, bracketed matrices, `:=` for a definition and `=` for an evaluation, original names such as `SIUnitsOf`), in light and dark themes. The computation is the same hidden code as before. An evaluation shows the author's left side and the engine's value; a vector or matrix result is written `[a b c] unit` with the unit once and scrolls instead of clipping. A construct without a MathML spelling falls back to the typeset-Calcpad display; the new importer option `mathml` (default on) turns MathML off. Unconverted regions that are equations (constraints, integrals) are typeset the same way, and no region shows Calcpad source as code text. The Word export turns the equations into Office Math and the Excel export writes them as readable text.
- Engine: an opt-in `#settings {"valMathml": true}` makes `#val` write a result as MathML with the unit once; off by default and used only by the importer.
- Import worksheet: a region that does not convert is now shown as the original equation, typeset, with its real names (in a `#noc` block, so it is never evaluated and raises no error), with the reason in a small note, instead of a line of prose with stand-ins. An equation on its own is reported as a constraint. The numbers an input (spreadsheet) component read from its saved workbook define the names it defines, with a note and a count in the dialog ("values frozen from input components"); regions that depend on them now convert. Arrays built element by element are allocated first, functions with conditions and early returns become one expression, and a variable named like a unit is renamed so the unit keeps its name.
- Import worksheet: units written with a middle dot (`kN·m`) are read as units; integrals become `$Integral` and first derivatives `$Slope`, with units and bounds checked against the engine; a range variable with literal bounds and the element assignments that fill an array from it become a `#for` loop over a zero vector.
- Import worksheet: programs made of local definitions, `for`, `while`, `if`/`else`, `break`/`continue` and a final value become `#for`/`#while`/`#if` statements with private local names, or a single expression when they are straight-line; `lsolve`, `submatrix` and `augment`/`stack` with several arguments map to the engine's functions. A program that cannot be converted faithfully stays a gap that says why.
- Import worksheet reads `.mcdx` files (the zipped container of the same worksheet language): File > Open, the Import button and drag and drop accept them. Text comes from the package's flow documents, maths goes through the same translator as `.xmcd`, and embedded objects, spreadsheet and chart components, worksheet references, page headers and footers and tables or pictures inside text are listed as skipped with a reason and a placeholder line. The importer module grows by about 120 KB (1.32 MB, 0.53 MB gzipped). Details in docs/worksheet-import.md.
- Installable and offline: a web app manifest, icons and a service worker that caches the whole build (about 19 MB, 417 files). "Add to home screen" or "Install" works on phones and desktops, and the app opens and calculates without a network. Plotly charts need the CDN script to have been loaded once while online. `web/e2e/pwa-check.mjs` checks it.
- `#python` blocks and sheet python fences are coloured with the Python grammar.
- Import worksheet: sums and products (`$Sum`, `$Product`) with bounds or over a range variable; an expression of a range variable is shown as a table (a loop collects the steps in a vector); a transposed row literal is a column vector the engine can count and index; `vectorSum`, `last`, `vectorize` over scalar arithmetic and `[A, B] := [1, 2]` convert; an element assigned to a vector that has no value (or past its end) creates or grows it; constant global definitions hold from the top of the sheet; a variable named like a unit under another spelling is renamed; primes in names work. Two worksheets of a structural handbook went from 70 % and 65 % of regions converted to 95 % with no engine errors; the computed values match the saved reference printouts.
- Import worksheet: regions that share a row are laid out side by side (a table for rows of symbol and description text, a wrapping row for everything else) while keeping the order they are evaluated in.
- Import worksheet: loop blocks no longer print raw engine values as plain text in the preview (they are hidden; errors inside are still recorded by the engine). An array started from 0 and then accumulated into takes the units of the first value added, which converts the stiffness-assembly style of program.
- Import worksheet: a definition is computed out of sight and shown as the author's equation only (original names and subscripts, a row transposed with a superscript T, a dot product, no result after it, as in the source); an evaluation shows the expression and its result without the substituted step; a renamed variable is shown under its original name; array arrows and bold matrix names are hidden by a small style; a long array result scrolls instead of wrapping. Results use the decimal places stored in the file (`#format 0.###`, display only, the engine still computes in full).
- Import worksheet: text keeps the source's look: paragraph styles and per-run font, size, colour and alignment are read and written as inline CSS (headings stay headings, colours get a lightened variant on the dark preview), plain picture regions are quiet and keep their size, and a picture with text to its right is one row. Picture regions that were reported as an unknown region kind now convert.

### Changed
- The worksheet importer crate and its WebAssembly module are renamed to be format-neutral, since they read both `.xmcd` and `.mcdx`: `calcpad-import` (package `civilkit-calcs-import`, not published) and `calcpad-import-wasm` (`calcpad_import_wasm.js` / `_bg.wasm`). Earlier entries below use the old names.

## [26.10.6] - 2026-10-24

### Added
- The Plotly API in Python blocks, so an interactive chart needs no HTML and no JavaScript: `import plotly.graph_objects as go` (also
  `plotly.graph_objs`), `go.Figure`, the common trace classes (`Scatter`, `Bar`, `Heatmap`, `Scatter3d` and others), `add_trace`,
  `update_layout`, `update_xaxes`, `update_yaxes`, `update_traces`, `add_shape`, `add_vline`, `add_hline`, `fig.show()`, and
  `plotly.subplots.make_subplots` (rows, cols, shared axes, subplot titles). It is a thin pass-through: every keyword is written into the
  figure JSON as given and Plotly.js validates it; `line_color`-style names expand to nested dicts, and anything the shim does not provide
  (another trace type, `write_html`, `fig.data`, a named template, a `fig.show` option) fails by name. Menus and sliders are Plotly's own
  `updatemenus` and `sliders` with precomputed steps. The page draws the figure with Plotly.js from its CDN, loaded only when a figure is
  present through the library list entry that already holds the pinned URL and integrity hash (not bundled, policy unchanged), with a light and
  a dark theme; without a connection a note says so, and print, Word and Excel carry a paragraph naming the chart. The 24 KB shim is fetched
  only by a block that mentions plotly. docs/python-blocks.md.
- A "Beam Explorer: Plotly from Python" sample (also as a sheet): the same three-span beam, solved in Python and explored with a native load case
  menu, a load factor slider (0.50 to 2.00 in steps of 0.05) and a point load position slider, with an envelope band and support lines, in 190 kB of figure data.
  `web/e2e/python-check.mjs` checks its plotted hogging moments against a three-moment solution, and that the real menu and sliders change the plotted data.

### Changed
- The raw HTML and JavaScript beam explorer is kept as "Beam Explorer: Python Results in Raw JS Plotly" (the example of the script route; it
  can combine controls and compute a readout, which Plotly's own controls cannot), and the Plotly API sample comes first in the Python group.

## [26.10.5] - 2026-10-24

### Added
- A sheet accepts Calcpad's own `#python` ... `#end python` pair as a Python block, the same block as a `python` fence (a pasted block no longer turns into sheet lines).
- `numpy.linalg` and `scipy.linalg` in Python blocks, computed by the Calcpad engine (the code behind `lsolve`, `inverse`, `det`,
  `cholesky`, `eigenvals`, `svd` in Calcpad lines), so Python and Calcpad lines give one answer: `solve`, `inv`, `det`,
  `cholesky`, `eigh`, `eigvalsh`, `svd`, `matrix_rank`, `cond`, `pinv`, `norm`; `scipy.linalg.solve`, `cholesky`, `cho_factor`,
  `cho_solve`, `solve_triangular`. A singular matrix is a `LinAlgError` on its Python line; unsupported calls (`lstsq`, `qr`,
  `eig` of a non-symmetric matrix) say so in a `NotImplementedError`. A new `calcpad-linalg-wasm` module (345 KB, 138 KB
  gzipped) is downloaded only by a block that mentions linalg, scipy, dot, matmul or `@`. Large float matrix products
  (about 46 by 46 and up) go to the engine as well, with identical values and about 7 times faster.
  Solving a 120 by 120 system takes 87 ms instead of 1.5 s in a hand-written Python loop. docs/python-blocks.md.
- `tools/build_wasm.sh linalg` builds the new module.
- A "Plane Truss by the Stiffness Method" sample in the Python group: a statically determinate truss assembled in loops and solved with `np.linalg.solve`, member forces and reactions recovered and drawn with matplotlib (tension and compression coloured, deflected shape), and a Calcpad member check on the exported maxima. `web/e2e/python-check.mjs` checks its member forces, reactions and deflections against hand values (method of joints, virtual work), and now also covers a bare `#python` block in a sheet, exported names with an underscore or a capital in a sheet, the section properties calculation, and the Sections group in the Samples browser.

### Changed
- The "Section Properties from numpy Arrays" sample is now "Built-up Section Properties" in a new Sections group, written with vectors and `hprod` and no Python: the units carry through and the report shows every substitution. The Python group keeps the samples where Python is the better tool.
- docs/python-blocks.md records why ulab (a numpy-like MicroPython module) was measured and not shipped: it is 10 times faster
  than the engine for dense solves, but a second, different numpy (2-D only, no `@` or `.sum()`) and 430 KB more wasm.

### Fixed
- In a sheet, the names a Python block hands over with `export()` use the sheet's spelling, so `export(sigma_c=2)`, `phi` and `Mstar` are found by the lines below the block as `σ_c`, `φ` and `M_star`.

## [26.10.4] - 2026-10-23

### Added
- Python blocks. A worksheet can hold `#python` ... `#end python` blocks, and a sheet a fenced ```python block, that run on
  MicroPython in a worker before the engine and become part of the worksheet: `print` output as text, matplotlib figures as
  inline SVG, and numbers handed back with `export(name=value)` as Calcpad definitions (a number, a vector, a matrix).
  `export_data` defines values without printing them, for a chart to read. The runtime (a 0.4 MB wasm, MIT licensed,
  with small numpy and matplotlib.pyplot stand-ins, about 240 KB gzipped in all) loads only when a worksheet has a block.
  The first run in each tab asks "This worksheet runs Python code"; a script has no network or file access, runs at most 10 s
  and has capped output. Preview, Report, Print PDF, Word and Excel carry the results. Python cannot read Calcpad variables
  yet. Design, sandbox limits and a compatibility table: docs/python-blocks.md.
- The editor treats a block as Python: no Calcpad errors on its lines, names it exports are defined for the lines after it,
  the prettifier and Format Document leave it alone. In a sheet the ribbon inserts a `python` fence.
- Four Python samples: "Beam Explorer: Python Solve, Plotly Charts" (a three-span beam solved by the stiffness method in
  numpy, with an interactive Plotly chart driven by the exported results; also as a sheet), a beam moment diagram, a function
  plot, and section properties from numpy arrays. Math > Programming has `#python` entries.

### Changed
- The Samples browser shows the source of a Python sample instead of running it.
- Faster calculation of the heaviest worksheets, with identical output. Soil Stress and Deformations runs on 37 %
  of the instructions in the command line tool (72 s of CPU time became 33 s on a busy machine) and in 60 % of the
  CPU time in the browser; Rectangular Slab FEA on 67 % of the instructions in the command line tool. Units are one
  allocation instead of about six, the command line tool uses a small-block allocator, and the result cache of
  custom functions hashes with a multiply. The engine wasm is 2 KB smaller.

## [26.10.3] - 2026-10-23

### Added
- Sheet syntax, now the default way to start a worksheet: Markdown prose, `name: expression -> unit | note` lines,
  LaTeX in prose (`$...$`, `$$...$$`, rendered with KaTeX, bundled and loaded on first use), vectors and matrices,
  conditions, plots, indented `for` (with `step`), `while`, `repeat` and `if` / `else if` / `else` blocks with
  `break` and `continue`, and `>>` lines and `calcpad` fences that pass raw Calcpad through. It compiles to the
  Calcpad language and runs on the same engine, so units, vectors, plots and the Word and Excel exports work.
  A sheet starts with `<!--sheet-->` or is a `.sheet` file. The compiler is a new crate (`calcpad-sheet`, not
  published) in its own WebAssembly module (84 KB, 39 KB gzipped, loaded on first use). Grammar: docs/sheet-syntax.md.
- Editor support for sheets: colours, name completions, Format Document, go to definition, find references, hover,
  semantic highlighting of names; engine errors, report links and lint markers report the sheet line.
- Show compiled Calcpad and Convert to Calcpad (Resources tab) for a sheet.
- In a sheet the ribbon, the Text Formatting tab (Markdown bold, italic, headings, lists, links, code) and the
  sidebar Insert tab insert the sheet form of each item; items with no sheet form insert raw Calcpad as `>>` lines
  and are marked.
- File > New and the quick-access New button open a new sheet (Ctrl+N too in the desktop app); New Calcpad worksheet opens the Calcpad
  syntax. The first tab of a fresh start is a sheet that mentions both.
- Eight sheet samples at the top of the Samples dialog, including sheet versions of the bundled beam deflection
  and cantilever examples.
- Changelog button on the Resources tab and a What's new link in About (GitHub, new tab).
- Checks: `web/e2e/sheet-check.mjs` and `web/e2e/verify-ribbon-sheet.mjs` (every ribbon insert, in sheet form,
  through the compiler and the engine).

### Changed
- The compiled Calcpad of a sheet reads as hand-written: headings are heading lines, a paragraph of plain text is
  one text line, Markdown blocks are used only for prose that needs them (lists, tables, emphasis, links, code),
  notes are plain comments in the compiled view (a style class in the page), and blank lines between sections
  are kept.

## [26.10.2] - 2026-10-23

### Added
- Import worksheet: File menu, a Document-tab ribbon button and drag and drop bring in a worksheet of the
  spreadsheet-like format of another calculation tool (`.xmcd`). It is read and converted in the page by a third
  WebAssembly module (loaded on first use, 1.1 MB, 0.46 MB gzipped; the engine module is unchanged), opens as a
  new unsaved worksheet, keeps its pictures in the library under `/imported/<name>/`, and reports what converted
  live, what is shown as a static picture and what needs attention, with toggles for the pictures and for saved
  results. Prime `.mcdx` files are refused with a message.
- While files are dragged over the page, a full-window hint says whether a drop opens a worksheet (`.cpd`) or imports
  one (`.xmcd`).
- Library: several files can be written with a single store write and one sync (an import brings in hundreds of
  pictures).
- `calcpad-import-xmcd` (not published): the converter behind Import worksheet, with a batch runner that reports
  what converted over a folder of worksheets; mapping tables and limits are in docs/worksheet-import.md.
- THIRD_PARTY_LICENSES.md lists the 16 additional Rust crates the importer pulls in (94 in the build graph).

### Fixed
- Relative `#include` paths resolve for a worksheet that has a source path (the Frames samples): the active sheet
  is given a path in the editor's requests.

## [26.10.1] - 2026-10-22

First public release, named CivilKit Calcs. It is a derivative of CalcpadCE (MIT): its web frontend and
worksheet language, with the engine, highlighter and Word and Excel export reimplemented in Rust.

### Added
- Rust calculation engine compiled to WebAssembly, in place of the .NET engine and server; no calculation
  request leaves the page.
- A second WebAssembly module for editor services (highlighting, linting, prettifier, definitions, the 1166-snippet
  library), loaded on first use.
- Word export (equations as Office Math) and Excel export (a Report sheet with text, equations, tables, pictures and
  native charts, plus Variables, Inputs and Plot data sheets), both built in the page.
- `#UI` input forms with select, radio and checkbox values carried into the report; dark theme for the preview.
- Samples browser with 101 worksheets from CalcpadCE's examples, their input fields rewritten as `#UI`.
- Ribbon interface with ten tabs, palette popovers with a Common section above the full snippet library, and icons in
  light and dark.
- Model bindings (`civilkit: bind` comments): specification, parser and in-app display; connecting a model is not
  implemented yet.
- Worksheet scripts limited to a library list under a Content Security Policy.
- The version (YY.M.N) shown in the page (About, status bar, `<meta name="version">`) and returned by the engine's
  `version()`; `tools/set_version.mjs` keeps every manifest in step.

### Changed
- The product is named CivilKit Calcs; CalcpadCE is credited in the About text, README, LICENSE and NOTICE.

### Known limits
- Raster (SkiaSharp) plots, `#ProjectPath`/`#LibraryPath` path roots and compiled `.cpdz` worksheets are not supported.
- Exports are untested in Microsoft Office: Excel files were checked in openpyxl and LibreOffice, Word files in
  LibreOffice.
- Deviations from CalcpadCE are listed in DEVIATIONS.md.

## Crates

Published on crates.io. The API follows the needs of the web app and may change between 0.x minor versions.

### civilkit-calcs-engine 0.1.0, civilkit-calcs-highlighter 0.1.0, civilkit-calcs-docx 0.1.0 - 2026-10-22

First release of the three libraries, from the same commit as the 26.10.1 release.
- `civilkit-calcs-engine`: the calculation engine (a Rust port of the CalcpadCE worksheet engine).
- `civilkit-calcs-highlighter`: tokenizer, linter, content resolution, prettifier and snippet library. Depends on the engine.
- `civilkit-calcs-docx`: Word and Excel export. Depends on the engine.
- The Rust crate names are unchanged (`calcpad_core`, `calcpad_highlighter`, `calcpad_docx`).
