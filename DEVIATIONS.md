# Deliberate deviations from CalcpadCE

The port reproduces CalcpadCE output exactly unless a behaviour is listed here. Each entry
names the behaviour, why it differs, the test that pins it, and the conformance stubs
(`*.html.stub`) whose expected output it changes.

## D1. A failing `#if` / `#else if` condition runs no branch

**CalcpadCE:** when the condition expression throws (for example a misspelled variable,
`#if v > v_typo`), `ExpressionParser.ParseTokens` catches the error and `ParseLine` then
calls `Condition.Check(_parser.Result)`. `MathParser.Parse` reset the result to zero, so
the condition is treated as false and the `#else` branch runs. In calculation mode the
condition line is not part of the output (`isOutput` is false when the keyword length is
non-zero), so the error is not shown at all: the worksheet silently takes the `#else`
branch. With

```
#if v > v_typo
'NOT OK
#else
'OK
#end if
```

CalcpadCE prints `OK` and nothing else.

**calcpad-rs:** the block is left unresolved. Neither the branch of the failing condition
nor any later `#else if` / `#else` of the same chain runs (`Condition.check_failed` marks
the chain as found and false), and an error paragraph is written in place of the block:

```
Error in "v > v_typo" on line [n]: Undefined variable or units: "v_typo". The condition
could not be evaluated, so the block is unresolved and none of its branches were run.
```

Valid conditions, `#while` conditions and condition errors outside calculation mode are
unchanged.

**Tests:** `calcpad-core/tests/deviations.rs`
(`d1_failed_if_condition_runs_neither_branch`,
`d1_failed_else_if_condition_skips_the_rest_of_the_chain`,
`valid_conditions_are_unchanged`).

**Stubs changed:** none. One out-of-scope worksheet reaches the path, because of an
earlier out-of-scope failure: `Examples/Isaiah/UI Mode Test.cpd` (`useSteel` is defined by
an unsupported `#UI` line, so `#if useSteel ≡ 1` fails). The vector case listed in phase 1
(`Rayleigh Quotient Iteration.cpd`) now evaluates and passes.

## D2. Plot grid loops stop after 100,000 lines

**CalcpadCE:** the grid loops of `$Plot`/`$Map` step from the lower to the upper limit;
a NaN step never terminates (the worksheet hangs).

**calcpad-rs:** the loops stop after 100,000 iterations. Finite grids have far fewer lines,
so the output is unchanged for every terminating case.

**Tests:** none needed for finite cases; the guard only replaces a hang.

**Stubs changed:** none.

## D3. Worksheet scripts run only from a library list (browser build)

CalcpadCE passes HTML in comments through to the output, including `<script>`, and runs every
script a worksheet carries, from any address. The browser build gives each rendered page a
Content-Security-Policy instead (`web/calcpad-web/src/wasm/script-policy.ts`):

* `script-src` allows inline code (worksheet glue) and only the exact file URLs on the library
  list, never a whole host. Each listed `<script src>` gets `integrity=` (sha384) and
  `crossorigin="anonymous"`; module files are checked through an import map.
* A `<script src>` that is not on the list is not loaded and is replaced by a note naming its
  URL; anything else the policy stops while the page runs (a module import, a script added
  later, an image from another site) is reported by a note as well. There is no prompt.
* `connect-src blob:` and `form-action 'none'`: a script cannot send data anywhere. `blob:`
  is allowed because it is data the page made itself (the DXF examples hand a drawing built in
  the page to dxf-viewer that way); it cannot reach another host.
* Images, fonts and styles come only from `data:`, `blob:`, inline styles and the app's origin.

The list starts with the eight libraries the upstream examples use (jQuery 3.6.3, D3 3.5.17,
Plotly 2.26.0, three.js 0.145 with OrbitControls, three.js 0.160.0 as a module, dxf-viewer
1.0.46 with its module dependencies, dxf-writer 1.18.4, model-viewer 3.0.1), hashed by
`tools/gen_library_defaults.mjs` from the files as served. Settings > Libraries removes and
restores entries and adds new ones after showing the fetched file's size and hash. Effects on
the upstream examples: `Toc.cpd`'s relative `./toc.js` is not a library and is blocked, the
texture image in `Parametric 3D Drawing of RC Beam.cpd` comes from another site and is
blocked, and `Bolt 3D Model.cpd` cannot fetch its `.glb` model.

Because the added `integrity`/`crossorigin` attributes are part of the page (Save HTML keeps
them too), `3D Plot with Plotly.cpd` in the browser sample no longer matches its stub byte for
byte: the browser comparison reads 27 of 30, the third difference being those two attributes.

The print frame has no scripts at all: the page is stripped of script, frame, object and
embed elements, `on*` attributes and `javascript:` URLs, and the frame is sandboxed with
`allow-same-origin allow-modals` (`print.ts`).

## D4. Model binding comments are marked in the browser output (addition)

A comment line `'<!--civilkit: bind <variable> = <path> [@ <selector>] -->` before an input
is a model binding (docs/model-bindings.md). The browser build (calcpad-wasm) adds a "model"
tag with the binding path at the end of the bound input's line, or the error for a malformed
binding (`calcpad_core::model_bindings`). Upstream passes the comment through unseen, and so
does the CLI here, so CLI output and the stub comparison are unchanged. No model is connected
yet: the typed values are used.

## D5. The browser converts C-style operators when it opens a sheet

The desktop app and the CLI read a worksheet file through `CalcpadReader`, which turns the
C-style shortcuts outside comments into Calcpad's operators: `==` `!=` `>=` `<=` into
≡ ≠ ≥ ≤, `%%` `&&` `||` `<<` into ⦼ ∧ ∨ ∠ and `<*` into ←. Upstream's server API does
not, so a sheet written with them (Simply Supported Beams/Concentrated Forces.cpd uses `%%`)
fails there. The browser build is the desktop app here, so it converts a sheet when it opens
one (File > Open, drag and drop, recent files, Samples), in the editor text as the desktop
does. Included files are not converted, as in the CLI. The conversion is
`calcpad_core::extension_methods::replace_c_style_operators` (moved from the CLI), with a
TypeScript copy in the browser (`c-style.ts`); `web/e2e/c-style-check.mjs` holds the two equal
over every line of the upstream repository.

## D6. Word export: no upstream oracle, and the differences from Calcpad.OpenXml

`calcpad-docx` ports the Word path of Calcpad.OpenXml (OpenXmlWriter, TableBuilder,
StyleDefinitionsWriter, ImageWriter/ImageProcessor, CssParser) and calcpad-core ports
Output/XmlWriter.cs for the equations (`ExpressionParser::capture_open_xml`). The package is
written with calcpad-core's own zip and XML code, not the OpenXml SDK.

There is no upstream oracle. The live API at api.calcpad-ce.org answers `/convert` but has no
`/docx` (404) and ignores an `outputFormat`; .NET is not available here. The port is checked by:
tests that follow the C# rules (`calcpad-core/tests/open_xml.rs`, `calcpad-docx/tests/docx.rs`,
mutation-checked); `tools/docx_check.py`, which writes every sample as .docx, checks the XML,
counts the equations against the HTML and has LibreOffice open each document; and the browser
check (Save Word).

Differences from upstream's writer, each deliberate:

* **SVG images** (plots, drawings) are rasterised to PNG with resvg at twice their size and the
  SVG is kept beside the PNG as an `svgBlip`, so Word versions without SVG support show the
  PNG. Upstream embeds the SVG alone. A plot's label alignment, which Calcpad sets with
  `.plot text.end` style rules that SVG renderers in word processors ignore, is written as
  `text-anchor` attributes. The browser draws SVG text with DejaVu Sans; the CLI uses the
  installed fonts.
* **Ordered lists are numbered.** Upstream gives each list the highest numbering instance + 1
  without creating it, so Word shows no numbers; each ordered list gets its own instance here.
* **A unit power with a format** (`#format F2`, then `m^2`): upstream's FixPowerFormatting for
  Office Math keeps the number's first character in front of the reformatted number ("22");
  the number is replaced whole.
* **Valid nesting.** A paragraph that the HTML puts inside another (an image in a text line)
  goes into it inline instead of as a nested `<w:p>`, and a table cell always ends with a
  paragraph.
* **Unknown elements keep their content.** Upstream drops any element it has no case for
  together with everything in it, so a `<blockquote>` (Numerical Differentiation puts its
  error lines in one) or an `<abbr>` vanished from the document; here such an element passes
  its content on. Scripts, styles, form buttons and other non-content elements are still left
  out. The HTML is read as browsers read it in one more respect: a `<select>` closes a
  `<select>` left open (the I-Section Properties samples do this), where HtmlAgilityPack would
  take the rest of the form into the first list.
* **Images** come from data: URLs (the browser turns library images into these) or, in the
  CLI, files next to the worksheet. Web addresses are not fetched (upstream downloads them
  with a 2 s timeout); a missing image is reported in the document as upstream does.
* **No validation pass.** Upstream runs the SDK's validator and returns its messages; the
  checks above stand in for it.
* The Office Math is written without the indentation upstream's verbatim strings carry.

## D7. Excel export of the worksheet (addition)

CalcpadCE has no such export. File > Export Excel (and `Cli input.cpd output.xlsx`) saves the
calculated worksheet as a workbook. The report is calculated as for print, with the input values
and `#UI` entries the sheet holds. The variable sheets are made by the engine
(`openxml::variables_export::sheets`, recorded by `ExpressionParser::capture_values`); the whole
workbook is written by calcpad-docx (`xlsx.rs` the package writer, `xlsx_report.rs` the report),
which the page loads on demand like the Word export.

* **Report** (first sheet): the rendered page in document order. Headings are bold and larger,
  text paragraphs merged text cells. An equation is a row: description (the short label before it
  in the text, "Yield strength -"), name, the expression, the expression with the values put in,
  the value (a number cell, `m·10ⁿ` read as a number) and its unit; a vector or matrix value
  links to its sheet. A table that holds only text is a cell range (header cells bold, borders,
  numbers as numbers, colspan/rowspan merged); a table that lays out other content (headings,
  pictures, equations) is read cell by cell, in order. A `$Plot` is a native, editable Excel
  chart (see below); other inline SVG drawings, `$Map` colour maps and anything else the engine
  gave no data for are pictures, rendered to PNG at twice their size by the code of the Word
  export, and images
  (including those from the page's file library) too, each at its place in the text with the rows
  it needs left free. A select shows the chosen option; hidden elements are skipped. Printing is
  landscape, fitted to the width.
* **Charts:** the plotter records what it calculated for each `$Plot` (`plotter::capture`:
  the points of every curve, the colour of its `SeriesN` class, the axis limits as drawn, the
  unit of each axis, whether a curve is a single point). The workbook gets one XY scatter chart
  (`xl/charts/chartN.xml`, straight lines, no markers; a dot for a single point) per plot, placed at
  the plot's position on the Report sheet in the size of the SVG, instead of the picture
  (no picture next to it: the chart replaces it, so the two cannot disagree). The data is on a
  last sheet, "Plot data" (two columns per curve, headers in rows 1 and 2, values from row 3), the
  series refer to those ranges and carry a cache of the values, so editing a cell redraws the
  chart. Axis limits are fixed to those of the SVG, titles are the plot variable and the curve's
  expression with their units, a legend appears when there is more than one curve. What a chart
  loses against the SVG: the faint fill under a `y = f(x)` curve, the exact fonts, the grid
  style. The matching of plots to drawings is by order: the n-th `<svg class="plot">` of the page
  takes the n-th captured plot; when there are fewer captured plots, the rest stay pictures.
  The chart XML is written in ECMA-376 schema order (Excel asks to repair a part otherwise) and is
  checked for that order by `tools/xlsx_check.py` and a unit test; real Excel was not available,
  so it is verified with openpyxl and LibreOffice only.
* **Variables:** every variable in the order it was first assigned, with its last value, as name,
  value, unit. A vector or matrix names the sheet that holds it.
* **One sheet per vector or matrix:** the elements, each row with its unit.
* **Inputs:** the variables set by input fields (`?`) and `#UI` controls.

Units: a value carries a unit only when the calculation gave it one. Many sheets declare a
variable as `h = ? {550}'mm` or `#UI 'Height -'h = 550'mm`: the unit is a comment after the value,
so `h` is unitless (confirmed on the I-section sheet, where every dimension was). When the text
that follows an equation's unitless number starts with a valid unit (checked with the unit
parser: "mm", "kN/m", "mm" followed by an exponent), the Report row gets it in the Unit column
and the Variables sheet in its Unit column with a Note ("unit read from the text after the
value"). A word such as "with" is not a unit. Numbers are numeric cells at full precision; a
complex value is text ("3 + 2i"). It is checked by `calcpad-core/tests/excel_export.rs`,
`calcpad-docx/tests/xlsx_report.rs`, `tools/xlsx_check.py` (every sample through the CLI, read
with openpyxl, and printed by LibreOffice) and the browser check.

Not done: live formulas (the cells hold the
calculated values and the expressions as text), and conditional output of `#if` blocks other
than what the page shows.

Found on the way: the workbook writer ordered a row's cells by comparing their references as
text, so a 27th column ("AA1") went in front of "B1" and the sheet was invalid. It now compares
column numbers, which also fixes `#write` past column Z.

## D8. The bundled samples declare their inputs with #UI (modification of upstream's examples)

**CalcpadCE:** many of its example worksheets write inputs as stored fields,
`name = ? {default}`. Its linter warns about them (CPD-3419, "declare the input with #UI
instead"), and the migration guide says to write `#UI name = default`.

**calcpad-rs:** the copies in `web/calcpad-web/public/samples` are written with `#UI`. The
rewrite is `web/calcpad-web/src/wasm/legacy-input.ts`, run by `tools/gen_samples.mjs`, and
the same function is the editor's "Convert to #UI" quick fix on that warning. A field is
rewritten when it is a plain number (no exponent, which `#UI` rejects) assigned with `=` to a
name; `? {` and `}` go, the default, the name and the unit written after the brace stay, and
`#UI ` is put in front of the line (labels before and between fields are what `#UI` lines
allow, docs/new-ui-mode.md). Lines that are another directive, or hold an expression
assignment next to the field (`#UI` rejects those), keep the stored field.

The generator renders the original and the converted sheet with the engine (included sheets
too) and keeps a line converted only when its rendered paragraph is the same once the input
widget markup (`<u class="input-N">v</u>`) is dropped. A line goes back to upstream's text when
the converted sheet reports an error or the paragraph differs. An input shows its digits as
typed (`0.60`), a `#UI` value is shown as a number (`0.6`), which is why two lines of the
seismic detailing sheets stay as upstream wrote them. The sheets with `#UI` lines open in the
input form (Settings > Open #UI Documents in Input Mode) and carry the "input form" tag in the
Samples browser. Result: 61 of the 101 samples converted (574 fields); 23 fields in 14 sheets
remain.

## D9. Dark preview: form controls, drawings and light cards stay readable

**CalcpadCE:** a worksheet's own HTML is styled for a white page. The steel I-section samples
write a select as `color:black; border:none; background:none; appearance:none`, draw SVG with
black text and strokes, and one puts its inputs on a `#f0f0ee` card.

**calcpad-rs:** in the dark Preview and Input views only (the rule is `@media screen` under the
page's `dark-theme` class, so the print frame, the exports and the light theme are unchanged) the
page carries a stylesheet (`PREVIEW_DARK` in `src/wasm/transport.ts`): inline SVG and `$Plot`
images sit on a white card and other images get a white background; select, input and textarea
get a readable colour, a border, a hover and focus outline, and a select the native arrow
(`!important`, since the inline style would win; the #UI controls keep their own look); the
`html` canvas gets the page background; and an element whose own inline style has a light
background is marked `cprs-light-card` by a small inline script, inside which the text takes
the light theme's colours. Checked by the browser check (computed contrast of the select, the
SVG card, the light card, the light theme unchanged).

## D10. select, radio and checkbox elements fill the text boxes they name

**CalcpadCE:** docs/programming.md ("Advanced UI with Html and CSS") says a select, radio button
or checkbox whose `name` or `data-target` equals the `id` of an element fills the text boxes
inside it with the semicolon separated parts of its value, in order. Neither CalcpadCE's web
frontend nor Calcpad.Server's pages do it, so in the browser build changing such a select did
nothing.

**calcpad-rs:** the input form's script (`calcpad-wasm/assets/ui-preview-script.html`) does it,
for the `calcpad-ui-input` boxes (the #UI fields, which the sample conversion D8 produced for
these sheets), and as an addition the change recalculates at once: the new values go to the
host in one `uiValuesChange` message (one render), through the existing postMessage channel, so
the sandbox and CSP are unchanged. A part that is not a number (the field's own rule) is
skipped. A checkbox gives its `value` when ticked and `0` when not (the documentation does not
say what unticked means). The choice itself is stored with the entered values as a `map:` entry
(`map:<target>|select` the chosen option's value, `map:<target>|radio`, `map:<target>|checkbox|<n>`
true or false), so it is saved in the document's `uiOverrides` comment like them, and the engine
(`calcpad-wasm/src/mapped_choices.rs`) marks the matching `selected` / `checked` in every render
that applies the entered values: the input form, the Report, the PDF and the Word export's HTML.
The Preview ignores entered values by design, so it shows the defaults unless Settings > Apply
#UI Values in Preview is on. Only the Input view maps; `e2e/select-map-check.mjs` clicks
through it on the I-section, catalogue (IPE 200 R in the Report), Excel catalogue, Wind Load,
seismic detailing, radio and checkbox cases.

## D11. Sheet syntax (addition)

A second way to write a worksheet, `<!--sheet-->` on the first line or a `.sheet` file: Markdown
prose, `name: expression -> unit | note` quantity lines, LaTeX, indented blocks, plots, and
`>>` pass-through lines. It is not part of CalcpadCE. It is a front end: `calcpad-sheet`
compiles it to a Calcpad worksheet (with a map from output lines to sheet lines) and the one
engine runs that, so nothing in the engine, the CLI or the stub comparison changes, and a
worksheet that is not a sheet is untouched. The compile happens in the page's request adapter
(`transport.ts`) for Preview, Report, Print, Word, Excel and the linter; the answers' line
numbers are mapped back. LaTeX is rendered by KaTeX in the page (bundled, MIT), not by the
engine. Grammar and limits: [docs/sheet-syntax.md](docs/sheet-syntax.md). Tests: the crate's unit
tests (`cargo test -p civilkit-calcs-sheet`, which render every sample through the engine and
check hand-computed values), `web/e2e/sheet-check.mjs` and `web/e2e/verify-ribbon-sheet.mjs`.

## D18. Eigenvalues of any square matrix, and complex eigenvalues (addition)

**CalcpadCE:** `eigenvals`, `eigenvecs` and `eigen` take only a matrix of the symmetric (or diagonal) type. A
literal such as `[2; 1|1; 2]` is a full matrix, so it fails with `Matrix must be symmetric.` until it is copied
into `symmetric(n)`, and a matrix that is not symmetric has no eigen function at all.

**Here:** a full matrix that equals its transpose (to a relative 1e-12) is copied to the symmetric type and
solved as before. Any other square matrix is reduced to Hessenberg form and solved with the shifted double QR
iteration; when all its eigenvalues are real the three functions return them as for a symmetric matrix
(ascending, the same `ne` argument, one unit eigenvector per row, scaled so its largest component is positive).
When some are complex they stop with `The matrix has complex eigenvalues.` Two new functions return the complex
case in the `fft` layout: `ceigenvals(M)` a 2 x n matrix (real parts, imaginary parts) and `ceigenvecs(M)` a
2n x n matrix (eigenvector k is rows 2k - 1 and 2k). Units of the matrix carry over to the eigenvalues.

**Why:** a literal stress tensor or `inverse(M)*K` of a dynamics problem is the common input, and the copy into a
symmetric matrix was a stumbling block.

**Tests:** `calcpad-core/src/matrix/general_eigen.rs` (residuals of random and defective matrices),
`calcpad-core/tests/general_eigen_functions.rs`, `web/e2e/eigen-check.mjs`.

## Not deviations (scope and known differences)

These are not deliberate behaviour changes, but they can make output differ:

* **Out of scope** (raster `$Plot` output with `VectorGraphics=false`
  or `ImagePath` set): the engine reports `... is not supported by calcpad-rs yet ...` as an
  ordinary expression error and records the feature so the harness classifies the worksheet
  as OUT-OF-SCOPE. `#ProjectPath`/`#LibraryPath` roots are accepted and ignored.
* **`#write` line endings:** CSV rows end with `\n` (.NET on Linux and macOS); Calcpad on
  Windows writes `\r\n`.
* **Excel workbooks written by `#write`** are built by `calcpad-core/src/openxml` rather than
  the DocumentFormat.OpenXml SDK. They hold the same cells (numbers as numbers, other text as
  inline strings) and read back the same, but the package parts are laid out differently;
  writing over a cell also drops what it held before (the SDK keeps an old value element
  beside a new inline string).
* **Browser build, `#write`:** a written file is kept in the page's file library and
  downloaded when its content differs from the library copy, rather than overwritten on disk
  on every render. Writes run only where upstream's write mode allows them (by default, the
  report).
* **Browser build, PDF:** printed by the browser (`@page` size, orientation and margins from
  the PDF settings); Calcpad.Server's header and footer (title, date, page numbers) are
  replaced by the browser's own print header and footer.
* **`$Map` PNG encoding:** CalcpadCE encodes the colour map with SkiaSharp (libpng and
  Chromium's zlib, level 4). calcpad-rs computes identical pixels and writes the same chunk
  layout, but its deflate stream differs, so the base64 text differs. `tools/compare_png.py`
  and `tools/classify_failures.py` report these worksheets as PNG-PIXEL (pixel-equal).
* **Floating point library:** `f64::sin`, `exp`, `pow` and friends come from the platform
  libm (macOS here); CalcpadCE uses the .NET runtime's. Last digit differences are absorbed
  by the harness except where a stub prints an exact integer (the reconciliation ignores
  integers): `Tests/Others/Integration tests.cpd` (`I_191`, classified NOISE) and
  `Examples/Engineering/Getting Started/Functions.cpd`, where `pow(1.4999999999999998, 2)`
  is one ulp off on macOS, the plot minimum becomes exactly 1.75 and one extra grid line is
  drawn (a structural difference, so it stays FAIL).
* **Browser build follows Calcpad.Server, the stubs follow Calcpad.Cli:** the CLI reads a
  worksheet through `CalcpadReader.Read`, whose `EnumerateLines` drops the final line break,
  while the server (and so `calcpad-wasm`) gives the editor text to the macro parser as it
  is. A worksheet ending in several blank lines therefore renders one more empty `<p>` in
  the browser than in its stub (`Tests/HP/Matrices/Functions/eigen functions.cpd`,
  `Tests/Matrices/Operators/scalar-operator-matrix/diagonal.cpd` in the browser sample).
  Removing one trailing newline gives the stub's output.
* **Browser build, report fonts:** Calcpad.Server embeds its bundled fonts in every page as
  data URLs (`BundledFonts`). `calcpad-wasm` does not compile them in (they would add about
  640 KB to the module); the page fetches them once from `report-fonts/` and passes the same
  `<style>` and `window.__calcpadFonts` markup with each request, so the HTML matches.
* **Browser build, code view:** the unwrapped/macro-error listing is coloured by the
  highlighter wasm, which the engine worker loads on demand (the engine asks through
  `globalThis.calcpadTokenizeListing`; without it the listing is plain, which is
  Calcpad.Server's own fallback when the tokenizer fails, and the answer says
  `needsHighlighter` so the worker loads the module and converts again).

## Calcpad.Highlighter (calcpad-highlighter)

The port reproduces the upstream tokens, diagnostics and definitions; the list below is where
it differs. Tests: `calcpad-highlighter/tests` and the unit tests in `calcpad-highlighter/src`.

* **H1. Columns are Unicode scalar values inside, UTF-16 units on the wire.** C# strings are
  UTF-16, so a character outside the BMP is two columns upstream. The port scans `&[char]`,
  and `api` converts every column and length it returns to UTF-16 (and the cursor column of
  `symbol-at-position` back). `tokenize_listing`, which only the engine reads, stays in scalar
  values. Pinned by `tests/api.rs` and the browser check.
* **H2. .NET character classes.** `char.IsLetter`/`IsDigit` are approximated from Rust's
  `is_alphabetic` and a short table of Nd ranges (`text.rs`); OrdinalIgnoreCase is invariant
  simple upper-casing. They differ from .NET only for exotic scripts.
* **H3. Regexes** use the Rust `regex` crate (no lookaround; only the patterns the highlighter
  applies are compiled). `IgnoreCase` is Unicode simple folding; `$` is written `\n?\z` where
  .NET `$` also matches before a final newline.
* **H4. JSON.** `System.Text.Json` is `serde_json` with `preserve_order`; nesting deeper than
  64 is rejected like `JsonDocument`, but serde_json also rejects lone surrogate escapes and
  numbers such as `1e999`, and collapses duplicate property names. `preserve_order` is
  switched on for the workspace by feature unification, so `calcpad-core`'s `serde_json`
  also keeps insertion order (its tests are unchanged).
* **H5. Uncaught C# exceptions return gracefully.** An empty document to
  `ContentResolver.GetStagedContent` (C# `ArgumentException`) is one empty line; a metadata
  comment with a non-string array item (`InvalidOperationException`) is not metadata;
  `Substring` past the end of a line in the semantic, naming and usage validators, an empty
  macro parameter name in `StringBuilder.Replace`, and the other spots listed in the code as
  "C# would throw" are handled instead of crashing.
* **H6. Disk access** goes through `IncludeProvider` (`NoDisk` by default, `NativeDisk` for
  native hosts, `MemoryFiles` in the browser). `Path.GetFullPath`, `Combine` and
  `GetDirectoryName` are reimplemented on strings (Unix rules, Windows rules when a path
  starts with a drive letter or `\\`); SHA-256 for `IncludedFileHashes` is in-tree.
* **H7. Set and dictionary order.** .NET `FrozenSet`/`FrozenDictionary` enumeration order is
  unspecified; the port uses declaration order where the C# code iterates (`CommandBlocks`,
  `KeywordToType` for the `#else if` versus `#else` prefix match, `commands_with_brace`).
  `Dictionary<,>`/`HashSet<>` enumerate in insertion order (`text::StrMap`/`StrSet`), also
  reproducing slot reuse in `LintIgnoreRegionParser`. `List.Sort` (unstable above 16
  elements) is a stable sort for the macro parameters.
* **H8. Not ported:** the `ContentResolutionCache` (every request resolves again, a
  one-document editor re-resolves in milliseconds), `CalcpadLinter.LintAsync` (a `Task.Run`
  wrapper), the console warning for invalid script/style/svg tag nesting, request
  cancellation (a superseded request is dropped by the page), and the C# `Console` output.
* **H9. Snippet text.** The 24 snippet descriptions that contained an em-dash use a spaced
  hyphen; everything else in `snippets.json` is the C# data in C# order. `AllSnippets` is a
  `FrozenSet` upstream (unspecified order); the port keeps registry order. The `snippets` JSON
  mimics the default `System.Text.Json` encoder escaping.
* **H10. Browser wiring.** The editor's `highlight`, `lint`, `definitions`,
  `symbol-at-position`, `prettify` and `snippets` requests are answered by the highlighter
  wasm with the server's JSON shapes and status codes (400 for an empty document, with
  `{"status", "error"}` carrying them across the worker). `sourceFilePath` is looked up in the
  page's file library like the engine's `#include`. The `HtmlComment` test that renders a
  worksheet through the Calcpad.Core parsers (`SettingsDirective_AppliesFourDecimals...`) is
  `#[ignore]`d in the highlighter crate; the Core behaviour is covered by the engine tests.
* **H11. A relative `#include` needs a source path; the browser gives one.** Upstream resolves a
  relative include against the directory of `sourceFilePath` and, when there is none, against the
  server's working directory (`Path.GetFullPath`). `ContentResolver` does the same here, so a sheet
  without a path finds no included file and reports every macro and variable an include defines as
  undefined (CPD-3303, CPD-3304 and the like). The page has no working directory, and its library keeps
  files at `/<name>`, so the web message bridge (`buildFileContext`) sends the active sheet's library
  path as `sourceFilePath`, and `/<title>.cpd` for an untitled sheet, which is where the engine looks
  too. Before this the page sent none, and the three Frames samples showed 40 undefined-macro markers.
  Checked by `web/e2e/browser-check.mjs` (the samples section) and `web/e2e/samples-sweep.mjs`, which
  compares every bundled sample's editor diagnostics and engine errors with the command line's on the same
  file with its includes on disk.
* **H12. `valMathml` is a known `#settings` key.** The engine's opt-in `#settings {"valMathml": true}`
  (an extension, see the engine's `MathSettings`) is accepted by the editor's `#settings` check as a
  boolean, so the line an imported worksheet carries raises no warning. Any other unknown key still warns
  (CPD-3413). An imported worksheet also carries a `LintIgnore` comment for the unused-name hint
  (CPD-3312): its definitions are the author's, and a name used only by a region shown as a picture or
  an unconverted one is not an editor finding.

* **H13. `unitPerElement` is a known `#settings` key.** The engine's `#settings {"unitPerElement": true}`
  (an extension, see the engine's `MathSettings`) is accepted by the editor's `#settings` check as a
  boolean; any other unknown key still warns (CPD-3413).

## D12. A nested block assigns the enclosing block's variable (modification)

**CalcpadCE:** every `$While`, `$Repeat` or `$Block` keeps the names it assigns to itself. A loop
nested in an expression block therefore cannot update a variable of the block around it: with
`f(n) = $Block{s = 0; $Repeat{s = s + k @ k = 1 : n}; s}` the result is 0 (the loop sums into a
private `s`), so a function body cannot hold a loop that accumulates.

**calcpad-rs:** when a block assigns a name that an enclosing block (not the worksheet) already
holds, it updates that variable. A name the enclosing blocks do not hold is still private to the
block that assigns it, a loop counter is still the loop's own, and a block at the worksheet level
still cannot assign a worksheet variable. The same function returns 15.

**Why:** the worksheet importer turns the functions of a worksheet whose bodies loop over a run-time
number of items (`for i in 1 .. rows(A)`) into expression blocks.

**Tests:** `calcpad-core/tests/deviations.rs` (`d12_*`).

## D13. A unit the elements of a vector or matrix share is written once (modification)

**CalcpadCE:** a vector or matrix of numbers that each carry a unit writes the unit on every element in the
equation as typed and as substituted (`[16 ft; 12 ft; 3 ft]`); only a result held as a single-unit array
(`[16 12 3] ft`) writes it once.

**Here:** when every element is a number with the same unit (a literal `16*ft`, a substituted variable, a
negated one), or every row is, the vector or matrix writes the unit once after the closing bracket
(`[16; 12; 3] ft`), in the HTML, text and Word (OMML) output. An element that is an expression (`2*a`), a
unitless element, or a different unit keeps the unit on each element. A result whose elements are in
different units of one dimension is written in the first element's unit. `#settings {"unitPerElement": true}`
restores the unit on every element.

**Why:** the unit once is how engineers write a vector of one quantity, and it is what the worksheet importer
produces for a result.

## D14. A Markdown pipe table is ruled in Word (modification)

**CalcpadCE:** a table is drawn with borders in Word only when it has the class `bordered`; the HTML a
Markdown pipe table gives has no class, so it is written without rules.

**Here:** a table that has a `<thead>` (what a Markdown pipe table gives) is written with the bordered table
style, as it is drawn with rules in the preview. A table without a header section is unchanged.

**Why:** a table without rules is hard to read, and the worksheet importer's Markdown option and the sheet
syntax both produce pipe tables.

**Tests:** `calcpad-docx/tests/docx.rs` (`a_markdown_table_is_ruled_and_a_plain_table_is_not`).

## D15. A radio group or drop-down finds its default by value, not by text (modification)

**CalcpadCE:** a `#UI` radio group or drop-down marks as chosen the option whose value text equals the text of the
variable as the report shows it. A default such as `25kN/m` is shown as `25kN∕m` (or a stacked fraction), `kN*m` as
`kN·m` and `m^3` with a raised 3, so the texts differ and no option is chosen; a drop-down then shows its first label
while the sheet calculates with the default.

**Here:** the shown text and the option value are compared with the unit marks made plain (the division sign, the
middle dot, raised digits and `^`), and the stacked-fraction bar counts as a slash. The numbers are compared by value
(`1.0` is `1`), and a value with more digits than the report shows matches its rounded form.

**Why:** a template whose default has a divided or multiplied unit opened with nothing chosen.

**Tests:** `calcpad-sheet/src/tests.rs` (`a_radio_group_or_drop_down_opens_on_its_default_whatever_the_unit`).
