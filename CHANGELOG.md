# Changelog

All notable changes are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions are numbered YY.M.N with no zero padding (26.10.1 is the first release of October 2026; N counts the releases
of the month and restarts at 1), which is valid semver for Cargo and npm. Tags are `v26.10.2` and so on.
CalcpadCE's own version is tracked separately in NOTICE.

The app changes are listed under their YY.M.N version below. The three Rust library crates have their own entries
under "Crates" at the end, on plain 0.x semver, and are tagged `<crate-name>-v<version>`
(for example `civilkit-calcs-engine-v0.1.0`).

## [Unreleased]

### Fixed
- **Clicking a diagram** - Clicking a Mermaid diagram or a code block in the preview now selects the whole block in the editor, instead of opening the one-line editor on its first line.

## [26.10.13] - 2026-10-29

### Added
- **Quick convert** - Select an expression in text or in a calculation and press Ctrl+Shift+E (or use the editor menu, = Formula on the prose toolbar, or the button under the selection on touch screens) to turn it into a formula line: in text it becomes a line after the paragraph, in a calculation it is pulled out to a named line above. One undo step.
- **Percentage output** - `-> %` and `-> ‰` show a ratio as a percentage or per mille in a sheet; see the percentages section of the guide.
- **Units of the sheet's own** - A `#unit bag = 25 kg | note` line defines a unit for the rest of the sheet, usable after a number and after `->`. A bad name or value is an error on its line, and the editor colours `#unit` as a keyword.
- **Eigenvalues of any square matrix** - `eigenvals`, `eigenvecs` and `eigen` accept a matrix that is not symmetric, without copying it into `symmetric(n)`. Complex results come from the new `ceigenvals` and `ceigenvecs`, which return real and imaginary rows; the real functions refuse a matrix with complex eigenvalues and point to them.
- **Generalised eigenproblem** - `eigenvals(K; M)`, `eigenvecs(K; M)` and `eigen(K; M)` solve K x = λ M x for symmetric K and positive definite M. Eigenvalues come in ascending order, mode shapes are M-orthonormal, and a bad pair is refused with a message naming the problem.
- **Sortable tables** - Click a column heading of a sheet table in the preview to sort ascending, descending, then back to the source order. It is a view only: the source is unchanged and printing keeps the source order. Headings are buttons, so the keyboard and touch work.
- **Mermaid diagrams** - A fenced `mermaid` block in a sheet, or in a `#markdown` block of a worksheet, is drawn as a diagram in the preview, print and the HTML report; Word export shows the source. The diagram code loads only on a page that has one and works offline once visited. A fence that does not parse stays as code with the reason. In a fresh checkout, `npm install` is needed to get mermaid.
- **Page breaks** - Page break is in the + menu between blocks and in the ribbon for sheets, drawn in the preview as a dashed line (screen only), and Word export turns it into a real page break.
- **Unsaved changes** - The window title starts with a dot while the active document is unsaved, closing an unsaved tab on the web offers Save, Don't Save or cancel, and the browser asks before the page is left with unsaved work. Exports do not count as saves.
- **New samples** - Two Mermaid diagram sheets (design flow, load path) and Principal axes of a section.
- **Diagrams in the guide** - A Diagrams chapter shows how to draw flowcharts and other Mermaid diagrams in a sheet, with three examples that the guide check runs.
- **Sliders** - `L: ? 6 m [2 .. 12 step 0.5] | span` is a number input with bounds, drawn in Input mode as a slider with the number box beside it. It works by touch and with the keyboard, a typed value outside the bounds shows `must be from 2 to 12` beside the box, and the bounds are checked in the sheet; the engine's `#UI` takes `min`, `max` and `step` (D17 in DEVIATIONS.md).
- **SI unit prefixes** - `3 mN`, `5 µm`, `40 nF`, `1.2 kΩ`, `2 GPa` and the like all work (yocto `y` to yotta `Y`; micro as `μ`, `µ` or `u`), where before only prefixed names the unit list spelled out were units. Non-SI units stay unprefixed; `ks` is still the metric horsepower. See D16 in DEVIATIONS.md.
- **Folding sections and key results** - A heading that ends with `{collapse}` folds the section under it in the preview and the report until the reader clicks it, a line ending with `{key}` gets a star, and `{key results}` lists the key lines above it. Print, PDF and the exports show everything; the preview's own fold arrow on other headings is a separate view-only fold.
- **Editor navigation** - One completion provider serves worksheets and sheets (names above the cursor with their last value, the function being written, built-in functions, units, block keywords), nothing is offered in prose, notes or strings, and Rename Everywhere (F2) and a Variable palette (Ctrl+; or Cmd+;) work in both editors.
- **What's new page** - `whats-new/` beside the app shows, for each release, a few short looping clips of its headline features and then the changelog's Added, Changed and Fixed items. `tools/make_whats_new.mjs` writes it from `CHANGELOG.md`, and `web/e2e/whats-new-check.mjs` checks it.
- **Block grip menu** - A grip beside each block of the sheet preview opens a menu to move, duplicate, cut, copy, paste below, disable or enable and delete the block, with an Undo toast; Ctrl/Cmd+Shift+K deletes the block at the caret.
- **Heading collapse and drag reorder** - Headings in the sheet preview fold their section with an arrow (Collapse all and Expand all in the heading's menu), and dragging a grip moves a block, or a heading with its section.
- **Prose toolbar** - A floating bar over prose in the source and in-place editors (Style, bold, italic, code, link, lists) whose tooltips show the Markdown each button types.
- **Name completion in calculation lines** - Typing in a sheet calculation offers the names defined above and the built-in functions, and nothing while writing prose.

### Changed
- **Input form focus** - The input form keeps the focus on the control you were using when it re-runs the sheet (a slider, a drop-down, a radio button or a box), so the arrow keys keep moving a slider.
- **Find references** - In a sheet it uses the same scan as rename, so a parameter is a separate name and strings and units are not matches.
- **Borderless block grip** - The grip beside a block is plain dots with no border, with a soft highlight on hover.
- **Importer function list** - `ceigenvals` and `ceigenvecs` are listed as engine function names when a worksheet is imported.

### Fixed
- **Deleting or moving a block** - Hidden code (`>> #hide` lines and loop code) is now its own block, so deleting or moving the block above no longer takes it along.
- **Insert menu** - It fits the window and scrolls when the list is longer than the screen.
- **Folding headings** - The preview's own fold arrow leaves headings that end with `{collapse}` to their own marker.
- **What's new page** - Bold text that contains a code span renders as bold.

## [26.10.12] - 2026-10-28

### Added
- **User guide** - Getting started, the sheet format, units, plots, Python, templates, importing and exporting, at `/calcs/guide/`. Every example in it is run by `book/check-examples.sh`, and eight short how-to videos go with it.
- **On-screen math keyboard** - Tabbed keys and templates for touch screens, and a draggable floating panel on desktop (Keyboard in the status bar).
- **Insert strip** - A dashed line with a + appears between rendered blocks; click it to insert a math line, text, heading, list, table, plot, condition or Python block there.
- **Plot chips** - Log x, Log y, 1:1, Copy data, PNG and SVG under each plot in the preview.
- **Edit in place** - Click a block in the preview to edit it where it stands; click away to apply, Esc to cancel.
- **Sheet inputs** - A line such as `L: ? 6 m | span` becomes a number box, `E: ? [Steel = 200 GPa, *Timber = 11 GPa]` a drop-down (round brackets for radio buttons) and `ribs: ? on` a checkbox. They compile to the engine's `#UI` controls, so reports and Word and Excel exports work unchanged; see "Inputs and templates" in docs/sheet-syntax.md.
- **Templates samples** - Seven sheets that use inputs: beam check, cantilever, column slenderness, pad footing, pipe flow, retaining wall, pump and pipeline. Generic textbook formulas, no design standard.
- **Input buttons** - The ribbon's Entry, Dropdown, Radio and Checkbox buttons insert the sheet form.
- **Tennis ball flight sample** - Drag, Magnus lift, spin and serve trajectories; large matrices also get a taller box.
- **Suite bar** - On screens 900 px wide or more in the browser, a slim bar above the ribbon links to the other CivilKit apps.
- **Theme in the address** - `?theme=dark` or `?theme=light` sets the theme for that page load without changing the saved choice.
- **Input check** - `e2e/inputs-check.mjs` changes each kind of input in a template and checks the result, the saved values and the exports.

### Fixed
- **Units** - The dot operators and the micro sign `µ` are read correctly (`5 µm` failed before).
- **Radio and drop-down defaults** - A default with a divided or multiplied unit (`25 kN/m`, `N/mm^2`) now selects its option instead of opening empty.
- **Names from `>> #UI`** - A name defined by a raw `>> #UI` line can be used by later sheet lines (it failed with `Undefined φ`).
- **Save values** - The comment is now written under the `<!--sheet-->` marker, so the file stays a sheet.
- **Browser check** - `e2e/browser-check.mjs` follows the three-tab sidebar and the new dialogs.

### Changed
- **Three sidebar tabs** - Insert, TOC and Variables; Settings, Export, Errors and Prettify open as dialogs, and Prettify joins the Document ribbon.
- **Browser shortcuts** - In a browser the app leaves the browser's own keys alone (address bar, tabs, zoom, bookmarks, print, new window).
- **Shared suite theme** - Ribbon, panels, dialogs, editor, results and charts use the shared palette in light and dark, with a quieter status bar. Printed reports and exports keep their light styling.

## [26.10.11] - 2026-10-26

### Added
- **Longer time limit prompt** - When a Python block hits its 10 s limit, the page offers to rerun with a longer one (60 s per block, 180 s per calculation), remembered for the tab; endless blocks are still stopped.
- **Plots sample** - A plot gallery of four plots on a grid, covering implicit curves, functions, parametric curves, noise, markers, a legend and a second y axis.
- **3D plot samples** - Five samples (plate deflection, column interaction surface, space frame, mode shape explorer with a dropdown, footing reactions bar chart), each checked in light and dark themes.
- **Importer text values** - Programs and functions that set, return or take a text (`check <- "OK"`) and tables mixing text names with numbers now convert live; across a local set of 16 structural worksheets the live share rose from 82.7 % to 87.4 %.
- **Clearer `string-value` gaps** - The gap now names what stopped it, such as a text-and-number mix or the text function called, which were previously reported as `builtin-missing`.
- **Importer external libraries** - A definition blocked only by an external library call takes the value the source saved, shown frozen and labelled; the import dialog counts these and the batch example lists why functions did not convert.

### Changed
- **3D plot theme** - The walls of 3D axes now follow the light or dark theme.

### Fixed
- **Importer text-or-number results** - A function returning text for some arguments and a number for others now prints the number, and a text-or-number table copied with `B := A` no longer prints text codes.

## [26.10.10] - 2026-10-26

### Added
- **Phone layout** - At 760 px wide or less the app shows one pane at a time (Edit, Results, Insert) from a bottom bar, with a folded ribbon, finger-sized controls and keyboard and safe-area handling; desktop and tablet are unchanged.
- **Touch editing** - A half-second long press on a rendered line, paragraph or table opens the in-place editor, and the keyboard hint is hidden on touch screens.
- **Editor above the keyboard** - The in-place editor follows the window when an on-screen keyboard opens and keeps its buttons above the bottom bar.
- **Touch key row** - A scrolling row under the line editor inserts symbols (`^ * / ( ) = < > ≤ ≥ √ π °`) and common units at the cursor without closing the keyboard.
- **Phone results toolbar** - The toolbar drops its label, scrolls sideways instead of wrapping, and the plot hover toolbar is left out; pinch and drag still work.
- **`polyroots(v)`** - Returns the real roots of a polynomial in ascending order from a coefficient vector with the constant term first.
- **Sample: Laguerre polynomial roots** - Coefficients from a loop, all real roots from one `polyroots` call, a residual check and the curve.
- **Sample: Beam deflection derived symbolically** - Derives the midspan deflection of a simply supported beam with the Symbolic dialog, then checks it numerically against the curve.

### Changed
- **LaTeX variable colours** - Variables in sheet LaTeX are coloured like the engine's equations (blue in light theme, light blue in dark).

## [26.10.9] - 2026-10-26

### Added
- **Symbolic dialog** - On the Functions tab: simplify, expand, factor, differentiate, integrate, solve, substitute and evaluate, with typeset results you can copy or insert into a worksheet or sheet. It loads only when first used (`docs/caskit.md`).
- **Computer algebra library** - A new pure-Rust library for engineering formulas with exact rationals, simplify, expand, factor, diff, integrate and linear and quadratic solving; anything unsupported is an explicit error.

## [26.10.8] - 2026-10-25

### Added
- **Rich-text Markdown editing** - Double-click Markdown prose (a `#markdown` ... `#end markdown` block) to edit with bold, italic, headings, bullets and tables; Apply writes raw Markdown in one undo step, and unchanged paragraphs keep their exact source (`docs/rich-text-editing.md`).
- **Rich-text editing in sheets** - Double-clicking prose in a sheet opens the surrounding heading, paragraphs, list or table as one block; input lines, calculations and fences stay one-line edits.
- **Import options: Tidy and Markdown** - Tidy text and Text to Markdown (import dialog and ribbon) strip imported fonts, sizes and colours and turn prose into clean Markdown, leaving equations, pictures and visible text untouched (`docs/worksheet-import.md`).
- **Markdown tables** - Pipe tables are drawn with thin rules, padding and a shaded header in preview, print and DOCX export (deviation D14); layout tables, bordered tables and matrices are left alone.
- **New build notice** - A small box offers Reload or Later when a newer build is ready, never reloads by itself, and checks every 30 minutes and when the tab is shown again.
- **Importer early exits** - `return`, `break` and `continue` inside function bodies convert, and locals that may be unset on some path give undefined rather than zero.
- **Importer ranges and arrays** - Stepped ranges (`x := 0, 5 .. 100`) work as tables, plot domains and loops; arrays grown by element, element assignments on existing arrays, whole-column writes, and arrays of vectors (`V[k][j]`) all convert, and `[A, B] := program` defines each name.
- **Importer complex numbers** - Complex values are handled as real and imaginary pairs (`z__re`, `z__im`) with products, quotients, powers 2 to 4, `Re`, `Im`, `abs` and `conj`; a complex value where a real is needed is a clear gap.
- **Importer `try ... on error`** - Now looks through single-expression functions and common math functions, and functions whose value is an `if` or loop convert without a block.
- **Importer solve blocks** - `.mcdx` solve blocks with one or two unknowns convert to an iterating solver that keeps the result only if every equation agrees, and solve-block regions are reported individually; solvers with a saved value take it frozen and labelled.
- **Importer lazy `if`** - One-line definitions whose branches call functions or index arrays are evaluated in an expression block so the untaken branch cannot fail.
- **Importer `Ceil` and `Floor` with a multiple** - `Ceil(L, in)` rounds up to a multiple of the second argument, noted as `round-digits`.
- **Importer worksheet references** - A referenced worksheet found in the package is followed: its definitions are computed in a hidden block, closing many undefined-name gaps.
- **Importer text-or-number values** - Values that are a number on some paths and text on others compute where numeric, undefined where text, and print either; vectors of texts become vectors of codes, while vectors mixing texts and numbers stay gaps.
- **Importer controls and button values** - Combo, text and list boxes convert to their stored selection, scripted buttons and check boxes to their stored value (note `control-input`), and bare names or products with no result show as written (`inert-expression`).
- **Importer file output** - Calls that write files (`WRITEEXCEL`, `WRITEPRN`, `APPENDPRN`, `WRITEBMP`) show as written (`file-output`), `csort` and `rsort` map to `sort_rows` and `sort_cols`, and a `READPRN` with nothing saved stays a gap.
- **Importer plot lengths** - A plot of arrays of different lengths stops at the shorter array instead of reading past its end.
- **Importer readable names** - Names the engine cannot spell (dots, `#`, `/`, `:`, spaces, leading digit) get one readable spelling sheet-wide (`SF_crack.step2` becomes `SF_crack_step2`) while typeset equations keep the author's name, closing most `invalid-name` gaps.
- **Importer text checks** - Text values in checks (`if(c; "OK"; "NG")`, comparisons, names and functions holding text) compute, and evaluations show the text; text used as a number, joined or sliced is still a `string-value` gap.
- **Importer last-`if` programs** - Programs whose value is their last `if` convert, along with `alsoif`, `return x <- e`, chained comparisons (`a < b <= c`) and programs ending in an assignment, and paths where nothing ran are undefined with note `uncovered-path`.
- **Importer data-file tables** - Tables read from files the importer cannot see (`READEXCEL`, `READPRN`) take the saved value, frozen and labelled; unset paths stay undefined.
- **Importer collapsed regions** - Regions inside collapsed areas are calculated in file order, and names defined in hidden regions are defined before use.
- **Importer small additions** - `linterp(xs, ys, x)` maps to `interp(x; xs; ys)`, and `x%`, `nthRoot` with a degree, the units `plf`, `pcf`, `klf`, `kcf`, `pli` and `kips`, and sub and superscript text are supported.
- **`engine_check` example** - Takes region markers from the importer, prints engine error messages and draws plots as vector graphics; new helpers `run_cpd` and `dump_region`.
- **Engine statistics** - `median`, `stdev`, `stdevp`, `var`, `varp`, `cumsum`, `convolve`, `linspace`, `polyfit(x; y; n)` and `interp(x; xs; ys)` (error outside the table) are in autocomplete, help and the ribbon.
- **Importer x-y plots** - Plot regions convert to `$Plot{...}` with multiple traces, axis units, range variables, titles and labels; unsupported styles are dropped with a `plot-style` note and other plot kinds stay named gaps.
- **Importer function bodies** - Functions with run-time loop bounds, `while` loops and element writes to an argument copy convert to expression blocks, and ones that fill an array by element without making it create it first as zeros.
- **Engine nested loops (D12)** - A loop inside an expression block now updates the enclosing block's variables, so function bodies can accumulate.
- **`linterp(x; xs; ys)`** - Linear interpolation that extends along the first or last segment outside the table (`interp` errors there).
- **Importer program results** - Programs return their last statement's value, including sequential `if`s and trailing assignments; a path with no value is undefined, never zero.
- **Importer function mappings** - `matcol`/`matrow` become `col`/`row`, `nthRoot(n, x)` becomes `root(x; n)`, `%` is the percent unit, elements of non-names use `take(i; array)`, and sums with `lowerBound`/`upperBound` convert.
- **Importer division-safe `try`** - `try expr on error fallback` converts when only a divide by zero can fail, by testing every divisor; anything else is a gap that says so.
- **Importer gap reporting** - Element writes that depend on an earlier gap are reported against it, so `ranked_gaps()` points at the real cause.

### Changed
- **Importer program arrays** - An array a program writes by element is the program's own even if the worksheet has a same-named array.
- **Importer text-label rows** - A row of text labels written by element is held as codes, so printing the table gives the `string-value` gap instead of silently losing the header row.
- **Single unit after vectors** - A vector or matrix with one shared unit shows it once after the bracket (`s = [16; 12; 3] ft`); `#settings {"unitPerElement": true}` restores the old display.
- **Importer angles** - Angles are plain numbers (`deg` and `°` are pi/180, `rad` is 1), fixing unit errors and a factor-of-57.2958 display error.
- **Importer zero comparisons** - A bare `0` compared with a quantity takes its unit everywhere, and also in `if` branches, `max`, `min` and sums.
- **Importer unit overrides** - A number-only override such as `10^(-3)` divides the value, and a display-unit evaluation is `((expr)/unit)*unit` so mixed units like kip times length are accepted.
- **Importer list and text boxes** - A list box is its selected position from 0, and an empty text box read by `str2num` is 0.
- **Importer array values** - An array value is no longer put into one element of a vector; the region is an `element-assignment` gap with the reason.
- **Importer equation indent** - An equation's page indent shrinks before the equation does, so far-right matrices keep their width in narrow previews.
- **Nested `if` spacing** - Nested `if` results have more line height and room between braces.
- **Importer vector products** - The product of two column vectors is `dot(a; b)`, and arrays assigned from another array name are copied.
- **Importer function names** - A worksheet function named like an engine function (`Norm`, `Max`, `Sin`) gets an `_f` suffix.

## [26.10.7] - 2026-10-25

### Added
- **Typeset equations on import** - Imported equations are shown as proper maths with stacked fractions, bounded sums and integrals, subscripts and matrices, in light and dark themes. Set the importer option `mathml` to turn it off; Word export gets Office Math and Excel export gets readable text.
- **Import fidelity** - Imported sheets follow the original look: names, units (`100 ft`), vector arrows, indents, range tables, Greek text and stacked unit fractions all match the source.
- **Original equations for definitions** - A definition shows only the author's equation, and an evaluation shows the expression and its result, using the original names and the decimal places stored in the file.
- **Result display setting** - `#settings {"valMathml": true}` makes `#val` write a result as MathML with the unit once; it is off by default and used by the importer.
- **Unconverted regions** - A region that cannot convert is shown as the original typeset equation with a short reason, and it is never evaluated.
- **Frozen input values** - Numbers read from a saved input (spreadsheet) component now define their names, so regions that depend on them convert, and the dialog shows a count.
- **More maths converts** - `kN·m` units, integrals (`$Integral`), first derivatives (`$Slope`), sums and products over bounds or a range variable, and range-variable tables now convert.
- **Programs convert** - Local definitions, `for`, `while`, `if`/`else`, `break`/`continue` and final values become `#for`/`#while`/`#if` statements, or one expression when straight-line. A program that cannot convert faithfully stays a gap that says why.
- **More functions and arrays** - `lsolve`, `submatrix`, `augment`/`stack`, `vectorSum`, `last` and `vectorize` map to engine functions, and element-by-element arrays are allocated first.
- **Import `.mcdx` files** - File > Open, the Import button and drag and drop now accept `.mcdx`, with unsupported parts listed as skipped with a reason. Details in docs/worksheet-import.md.
- **Installable and offline** - The app can be added to a home screen or installed, and it opens and calculates without a network. Interactive charts need their script loaded once while online.
- **Python highlighting** - `#python` blocks and sheet python fences are coloured with the Python grammar.
- **Side-by-side regions** - Regions that share a row in the source are laid out side by side while keeping their evaluation order.
- **Cleaner loops and arrays** - Loop blocks no longer print raw values in the preview, and an array accumulated from 0 takes the units of the first value added.
- **Source text styling** - Paragraph styles, fonts, sizes, colours and alignment carry over, and picture regions now convert and keep their size.
- **Better conversion rate** - Two handbook worksheets went from 70% and 65% to 95% of regions converted, with values matching the saved printouts.

### Changed
- **Importer renamed** - The importer crate and its WebAssembly module now have format-neutral names (`calcpad-import`, `calcpad-import-wasm`), since they read both `.xmcd` and `.mcdx`.

## [26.10.6] - 2026-10-24

### Added
- **Charts from Python** - Python blocks can build interactive charts without HTML or JavaScript: `import plotly.graph_objects as go`, `go.Figure`, common traces, layout updates, `fig.show()` and `make_subplots`. Keywords pass straight through, anything unsupported fails by name, and the chart script loads only when a figure is present. See docs/python-blocks.md.
- **Beam Explorer sample** - "Beam Explorer: Plotly from Python" solves a three-span beam in Python and explores it with a load case menu, a load factor slider (0.50 to 2.00) and a point load position slider, and an end-to-end check validates its moments.

### Changed
- **Raw JS beam explorer kept** - The HTML and JavaScript version is now "Beam Explorer: Python Results in Raw JS Plotly", the script-route example, and the new sample comes first in the Python group.

## [26.10.5] - 2026-10-24

### Added
- **`#python` pair in sheets** - A sheet accepts `#python` ... `#end python` as a Python block, the same as a `python` fence.
- **Linear algebra in Python** - `numpy.linalg` and `scipy.linalg` calls such as `solve`, `inv`, `det`, `cholesky`, `eigh` and `svd` run on the engine, so Python and worksheet lines give one answer. Large matrix products are about 7 times faster, and a 120 by 120 solve takes 87 ms instead of 1.5 s. See docs/python-blocks.md.
- **Build script** - `tools/build_wasm.sh linalg` builds the new module.
- **Plane Truss sample** - "Plane Truss by the Stiffness Method" assembles and solves a truss in Python, draws forces and deflections, and checks results against hand values.

### Changed
- **Section sample moved** - "Built-up Section Properties" now lives in a new Sections group and uses vectors and `hprod` with no Python, so units carry through and the report shows every substitution.
- **Linear algebra note** - docs/python-blocks.md records why a smaller numpy-like module was measured and not shipped.

### Fixed
- **Exported names in sheets** - Names handed over with `export()` now use the sheet's spelling, so `export(sigma_c=2)` is found below the block as `σ_c`.

## [26.10.4] - 2026-10-23

### Added
- **Python blocks** - `#python` ... `#end python` blocks in a worksheet, or a ```python fence in a sheet, run before the engine. Output appears as text, figures as SVG, and `export(name=value)` hands numbers back as definitions. Runs are sandboxed with no network or file access, a 10 s limit and capped output; see docs/python-blocks.md.
- **Editor support for Python** - Python blocks get no worksheet errors, exported names are defined for the lines after, formatting leaves them alone, and the ribbon inserts a `python` fence.
- **Python samples** - Four samples: a beam explorer with an interactive chart, a beam moment diagram, a function plot and section properties. Math > Programming gains `#python` entries.

### Changed
- **Samples browser** - It shows the source of a Python sample instead of running it.
- **Faster heavy worksheets** - The heaviest worksheets calculate faster with identical output; Soil Stress and Deformations took 72 s of CPU time before and 33 s now.

## [26.10.3] - 2026-10-23

### Added
- **Sheet syntax** - The new default way to start a worksheet: Markdown prose with `name: expression -> unit | note` lines, LaTeX in prose, vectors, matrices, plots, and `for`, `while`, `repeat` and `if` blocks. A sheet starts with `<!--sheet-->` or is a `.sheet` file; see docs/sheet-syntax.md.
- **Sheet editor support** - Colours, name completions, Format Document, go to definition, find references, hover and semantic highlighting, with errors and lint markers reported on the sheet line.
- **Show compiled / Convert** - The Resources tab can show the classic-syntax version of a sheet or convert it.
- **Sheet-aware insert tools** - The ribbon, Text Formatting tab and sidebar Insert tab insert the sheet form of each item; items with no sheet form insert raw classic syntax as `>>` lines and are marked.
- **New sheet by default** - File > New and the New button open a sheet (Ctrl+N too in the desktop app), and a separate New worksheet entry opens the classic syntax.
- **Sheet samples** - Eight sheet samples sit at the top of the Samples dialog, including beam deflection and cantilever examples.
- **Changelog links** - A Changelog button on the Resources tab and a What's new link in About.

### Changed
- **Cleaner compiled output** - The compiled version of a sheet reads as if written by hand, with headings as heading lines, one text line per plain paragraph, and blank lines between sections kept.

## [26.10.2] - 2026-10-23

### Added
- **Import worksheet** - File menu, a Document-tab ribbon button and drag and drop bring in `.xmcd` worksheets, converted in the page into a new unsaved worksheet with pictures kept in the library under `/imported/<name>/`. It reports what converted, what became a static picture and what needs attention; `.mcdx` files are refused with a message.
- **Drop hint** - While files are dragged over the page, a full-window hint says whether a drop opens a worksheet (`.cpd`) or imports one (`.xmcd`).
- **Faster library writes** - Many files can be written with one store write and one sync, which matters when an import adds hundreds of pictures.
- **Licences** - THIRD_PARTY_LICENSES.md now lists the extra Rust crates the importer uses.

### Fixed
- **Relative includes** - Relative `#include` paths now resolve for worksheets that have a source path, such as the Frames samples.

## [26.10.1] - 2026-10-22

First public release, a derivative of an MIT-licensed project: its web frontend and worksheet language, with the engine, highlighter and Word and Excel export reimplemented in Rust.

### Added
- **Rust calculation engine** - Compiled to WebAssembly, so no calculation request leaves the page.
- **Editor services module** - A second WebAssembly module, loaded on first use, provides highlighting, linting, prettifier, definitions and the snippet library.
- **Word and Excel export** - Word export writes equations as Office Math; Excel export has a Report sheet plus Variables, Inputs and Plot data sheets, with native charts. Both are built in the page.
- **`#UI` input forms** - Select, radio and checkbox values carry into the report, and the preview has a dark theme.
- **Samples browser** - 101 example worksheets with their input fields rewritten as `#UI`.
- **Ribbon interface** - Ten tabs, palette popovers with a Common section above the full snippet library, and light and dark icons.
- **Model bindings** - `civilkit: bind` comments have a specification, parser and in-app display; connecting a model is not implemented yet.
- **Restricted scripts** - Worksheet scripts are limited to a library list under a Content Security Policy.
- **Version display** - The version (YY.M.N) shows in About, the status bar and `<meta name="version">`, and `tools/set_version.mjs` keeps every manifest in step.

### Changed
- **Product name** - The product has its own name, and the original project is credited in About, README, LICENSE and NOTICE.

### Known limits
- **Unsupported features** - Raster plots, `#ProjectPath` and `#LibraryPath` path roots, and compiled `.cpdz` worksheets are not supported.
- **Office testing** - Exports are untested in the Office apps; Excel files were only checked in other spreadsheet tools.

  LibreOffice.
- Deviations from CalcpadCE are listed in DEVIATIONS.md.

## Crates

Published on crates.io. The API follows the needs of the web app and may change between 0.x minor versions.

### civilkit-calcs-caskit 0.1.0 - unreleased

Not published yet. A pure-Rust computer algebra library for engineering formulas (see `caskit/README.md`); it has no dependencies and builds for `wasm32-unknown-unknown`.

### civilkit-calcs-engine 0.1.0, civilkit-calcs-highlighter 0.1.0, civilkit-calcs-docx 0.1.0 - 2026-10-22

First release of the three libraries, from the same commit as the 26.10.1 release.
- `civilkit-calcs-engine`: the calculation engine (a Rust port of the CalcpadCE worksheet engine).
- `civilkit-calcs-highlighter`: tokenizer, linter, content resolution, prettifier and snippet library. Depends on the engine.
- `civilkit-calcs-docx`: Word and Excel export. Depends on the engine.
- The Rust crate names are unchanged (`calcpad_core`, `calcpad_highlighter`, `calcpad_docx`).
