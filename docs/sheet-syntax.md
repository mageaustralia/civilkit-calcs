# Sheet syntax

A sheet is a friendlier way to write a worksheet: Markdown prose, plain `name: expression`
lines and LaTeX, compiled to the Calcpad worksheet language and run by the same engine. There
is no second engine. Real unit algebra, vectors and matrices, plots and the Word and Excel
exports all work, because the engine sees an ordinary Calcpad worksheet. The compiler is the
crate `civilkit-calcs-sheet` (`calcpad-sheet/`), a deterministic function from sheet text to
Calcpad text plus a source map. This is our addition and not part of CalcpadCE.

A file is a sheet when its first non-blank line is

```
<!--sheet-->
```

or when its name ends in `.sheet` (the editor adds the marker line when it opens one, so the
text and the editor mode always agree). **File > New** and the quick-access New button (and Ctrl+N in the desktop app) open a
new sheet; **New Calcpad worksheet** (second in the File menu, and on the Resources tab) opens the
advanced syntax. Use **Show compiled** to see the Calcpad a sheet becomes (its line numbers are the
sheet's lines) and **Convert to Calcpad** to replace the sheet with that text. Both give Calcpad
as a person would write it, so a sheet is also a way to learn the Calcpad language.

## Example

```
<!--sheet-->
# Beam bending check

A simply supported beam. The moment **comes out in kN·m**.

L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
Ms: 300 MPa * 1.23e6 mm³ -> kN*m
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

## Prose

Anything that is not one of the line kinds below is Markdown prose: `#` to `######` headings,
paragraphs, `**bold**`, `*italic*`, `-` and numbered lists, pipe tables, links,
`![alt](file)` images (library files, as in Calcpad), `` `code` `` and fenced code blocks. The
compiler writes prose the way a person would write it in Calcpad. A heading becomes a heading
line (`# Title` is `'<h1>Title</h1>`, and `### Heading` is `"Heading`); a paragraph of plain text,
even over several lines, becomes one text line (`'text`); only prose that really contains Markdown
(lists, tables, quotes, emphasis, links, code, backslashes) goes in a `#markdown ... #end markdown`
block, which the engine's own Markdown renderer handles. Blank lines between sections are kept.

A line that would read as a quantity but is prose (`Note: see the drawing`) is prose already,
because its right side is not an expression. To force prose, start the line with a backslash.

### LaTeX

`$...$` is an inline formula and `$$...$$` a display formula, in prose, in notes and in the
text of an `if` line. A `$$` alone on a line opens and closes a multi-line formula. `\$` is a
dollar sign, and a `$` before a digit ("$5") is one too. The preview renders formulas with
KaTeX, which is bundled with the app and loaded the first time a sheet has a formula; the
rendered page needs no script and the content security policy is unchanged. For Word and Excel
a formula is written as its TeX source in italics.

### Page breaks

A prose line holding Calcpad's page break, `<div style="page-break-after:always"></div>`, passes through as
the text line `'<div style="page-break-after:always"></div>`. Print PDF starts a new page after it, the Word
writer writes a Word page break, and the preview draws it as a dashed line labelled "Page break" (screen only).
The "+" menu between blocks and the ribbon's Page break button insert it. There is no other sheet form.

### Diagrams

A fenced code block tagged `mermaid` (in prose, or in a `#markdown` block of a Calcpad worksheet)
is drawn as a diagram. The app swaps the rendered fence for the SVG that Mermaid draws, with SVG
text labels; Mermaid is bundled as separate chunks, loaded the first time a page has a diagram,
and precached by the service worker. The page gets markup and styles only, so the content
security policy is unchanged, and the HTML and PDF reports carry the drawing. Word export shows
the source. Diagrams use the dagre layout (`layout: elk` is not shipped). A fence Mermaid cannot
parse stays as code, followed by the reason.

The user guide's chapter on diagrams (`book/src/sheet/diagrams.md`) is a gallery with one small
example of each kind of diagram the bundled Mermaid (12.1.0) draws, a list of the kinds it does not
have, and tips. The Samples dialog's **Plots and diagrams** group has fourteen sheets with a diagram
next to the calculation it describes (state, sequence, Gantt, pie, treemap, quadrant, XY, Sankey,
mind map, requirement, timeline and class diagrams as well as flow charts); `web/e2e/diagram-samples-check.mjs`
opens each one in the app and checks that it draws.

## Quantity lines

```
name: expression [-> unit] [| note]
name = expression
a: 1; b: 2 | note for b
f(x): x^2 + 1
```

* The line shows what a Calcpad equation line shows: the name, the expression, the values
  substituted and the result with its units. The note follows in a muted style (a `sheet-note`
  class, styled by the page). In the compiled view and in Convert to Calcpad a note is a plain
  Calcpad comment: `x = 5 kN 'the load`.
* **Names** are letters, digits, `_` and primes. Greek spellings become Greek letters:
  `phi` is φ, `sigma_c` is σ with subscript c, `Delta_T` is Δ with subscript T, and a Greek
  spelling followed by a capital or a digit (`phiMs`, `sigma2`) is the letter and the rest.
  Capitals are used only where they differ from a Latin capital (`Gamma`, `Delta`, `Theta`,
  `Lambda`, `Xi`, `Pi`, `Sigma`, `Upsilon`, `Phi`, `Psi`, `Omega`). A name ending in `star`
  after a capital or digit (`Mstar`) is that name with subscript `star`. The same rule renames
  every use, so a name is spelled one way throughout the sheet.
* **Units are real.** Write a unit after a number: `300 MPa`, `9.81 m/s²`, `5 kN*m`. The engine
  checks and combines them, so `fy * Ze` in MPa and mm³ is a moment. A unit written after a
  number is never renamed (`3 psi` is a pressure and `psi` alone is ψ).
* **Conversion:** `-> unit` converts the result: `Ms: fy * Ze -> kN*m` (Calcpad's `|kN*m`). It
  must come last, before any note.
  `-> %` (or `-> ‰`) shows a ratio as a percentage: `util: Ed / Rd -> %` gives `69.07 %`.
  `-> base` shows the result in SI base units (kg, m, s, A, mol, cd): `M: F * L -> base` with `F` in kN and `L` in m
  gives `10000 kg·m²/s²`. A value with no unit, or with a temperature or an angle in it, is left as it is. It compiles
  to the engine's `| base` (DEVIATIONS.md D20).
* **A unit of the sheet's own:** `#unit bag = 25 kg [| note]` defines the unit `bag` (the engine's
  custom unit `.bag = 25 kg`), which later lines use after a number (`3 bag`) and after `->`. The
  name is letters and `_` only; the value is an expression with units. A bad name or value is an
  error on its line, and redefining a built-in unit is the engine's error. The line is not renamed,
  and Format Document leaves it as written.
* **Notes:** the first `|` outside brackets starts the note. `||` is the logical or. Use
  `abs(x)` for an absolute value.
* **Several per line:** separate with `;`. A note belongs to the last one.
* **Numbers:** `1.23e6` is written as `1230000`, since the engine has no exponent notation.
* **Operators:** `+ - * / ^ %`, `< > <= >= == != and or`, and the Unicode forms `≤ ≥ ≠ ≡ ∧ ∨ × · ÷ −`.
  In a condition a single `=` also means equal. `²` and `³` after a name or unit are `^2` and `^3`.
* **Calls** take commas: `max(a, b)`; the compiler writes the engine's `;`. Calcpad's `;` is accepted
  as well inside parentheses, in a call (`f(1; 2)`, `max(3; 7)`) and in the parameter list of a definition
  (`f(x; y): x + y`). Decision: `;` keeps its meanings elsewhere (outside brackets it separates the
  statements of a line, inside `[...]` it separates rows); inside `(...)` it had none (a definition with it was
  read as prose, a call was an error), so there it is normalised to the comma form instead of being reported.
* **Solving, integrals and slopes:** the value of a quantity can be one of three numerical operations written
  like a plot line: `x0: solve f(x) = c for x in a..b` is `x0 = $Root{f(x) = c @ x = a : b}` (without `= c`
  it solves `f(x) = 0`; the engine evaluates `c` once, so a right side that uses the variable, as in
  `solve f(x) = g(x) for ...`, is moved to the left: `$Root{f(x) - (g(x)) @ ...}`), `A: integral of f(x) for x in a..b` is `$Integral{f(x) @ x = a : b}`, and
  `k: slope of f(x) at x = a` is `$Slope{f(x) @ x = a}`. The line takes `-> unit` and a note as any quantity
  line does; a bare `0` bound takes the other bound's unit (`0*(5 m)`), since the engine needs both in one
  unit. A line with the shape of one of them (a `..` range, or `at` and `=`) that does not parse is an error on
  its line; `Note: solve this by hand` stays prose. The operation must be the whole value (`2 * solve ...` is
  not read).
* **Vectors and matrices:** `[1; 2; 3]` and `[1, 2, 3]` are vectors. `[1, 2; 3, 4]` is a
  matrix: columns are separated by commas and rows by semicolons. (Calcpad writes
  `[1; 2 | 3; 4]`.) Rows must have equal length. Elements are expressions. An element is
  reached with `v.2` and `A.(1, 2)`.
* A quantity cannot have a bare unit after a bracket (`[1, 2] kN`); put the unit on the elements.
* **A unit the elements share is shown once.** When every element of a vector or matrix is a number
  with the same unit (`[16 ft; 12 ft]`), the result and the equation show it once after the closing
  bracket, `[16; 12] ft`, in the page and in the Word and Excel exports. Elements in another unit of
  the same dimension are converted to the first element's unit in a result; an element that is an
  expression, is unitless or has another dimension keeps its own unit. To write the unit on every
  element as before, put `#settings {"unitPerElement": true}` in the sheet.

## Conditions

```
if utilisation <= 1: "OK" else: "Fails"
if a == 2: b: 1
```

One line: the branches are a quoted text or quantities. For anything longer use a block.

## Blocks, loops and conditions with a body

A line ending in `:` that starts with `if`, `else if`, `else`, `for`, `while` or `repeat`
opens a block. The body is the lines indented more than the header (a tab counts as four
spaces); blank lines inside are fine; a line indented no deeper than the header closes the
block. A body can hold anything: quantities, prose, LaTeX, `>>` lines and nested blocks. A
line that mixes tabs and spaces in its indentation is an error on that line.

```
for i in 1..5:
    P: 10 kN * i | load at stage $i$
    if i >= 3:
        Heavy stage.
    else:
        Light stage.

for x in [1, 2, 3]:
    y: x^2
while n > 0:
    n: n - 1
```

| Sheet | Calcpad |
| --- | --- |
| `if c:` / `else if c:` / `else:` | `#if c` / `#else if c` / `#else`, closed by `#end if` |
| `for i in a..b:` | `#for i = a : b`, closed by `#loop` |
| `for i in a..b step s:` | an index loop of `floor((b - a)/s) + 1` passes, with hidden lines that set `i` (the step may be negative) |
| `break`, `continue` | `#break`, `#continue` (inside a loop; `if c: break` works on one line) |
| `for x in v:` | an index loop over the vector `v` (a name or a literal), hidden helper lines set `x` |
| `while c:` | `#while c` ... `#loop` |
| `repeat n:` | `#repeat n` ... `#loop` |

The step is written `step s` after the range (`for i in 0..10 step 2:`); a range without it counts
by one. Quantity lines in a loop give exactly the report a Calcpad loop gives
(a test compares them). A header whose condition is not an expression (`If the load is
large:`) is prose, with a warning if it starts with a keyword.

## Plots

```
plot M(x) for x in 0..L
plot sin(x), cos(x) for x in 0..2*π
plot y = f(x) for x in 0..10
```

becomes `$Plot{ M(x) @ x = 0 : L }`; several expressions are drawn together. An optional
`name =` before the expressions is dropped. A malformed plot line is reported in the report and
on its sheet line.

A curve takes a name for a legend with `as "name"`: `plot sin(x) as "sine", cos(x) as "cosine" for x in 0..6`
becomes `$Plot{ sin(x) "sine" & cos(x) "cosine" @ x = 0 : 6 }`. The engine draws a legend box (a swatch and
the name of each curve, in the top right of the plot area) inside the SVG when at least one curve has a name;
a curve without one is listed by its expression. Being in the SVG, the legend is in the preview, print, PDF and
the Word picture; the Excel chart takes the names as its series names. A name cannot contain quotes, `&`, `|`,
`@` or braces (the engine's `$Plot` delimiters); an empty name or a quoted text without `as` is an error on
the line. The quoted name in `$Plot` is an extension of this port (DEVIATIONS.md).

## Calcpad pass-through

Every engine feature stays reachable. (Number, choice and checkbox inputs have a sheet form of their own,
see "Inputs and templates"; the data grid and the other `#UI` forms are reached this way.) A line starting with `>>` is passed to the engine as
written, and so is everything between a `calcpad` fence:

````
>> #UI 'Span -'L = 6'm
```calcpad
#for i = 1 : n
    s = s + i*phi
#loop
```
````

A fence with the language `python` is a Python block (docs/python-blocks.md); so is Calcpad's own `#python` ... `#end python` pair written on lines of their own, which a sheet accepts as the same block. It compiles to `#python` ... `#end python`, its lines
are copied as written (no renaming, indentation kept), and what it prints, draws and `export`s becomes part of the sheet. Names it
exports can be used by the quantity lines after it, under the sheet's spelling of the name (`export(sigma_c=2)` is `σ_c` below the block, as `phi` is `φ`).
A block that does `import ui` can draw interactive panels: sliders, drop-downs and checkboxes whose moves run one Python function again and redraw its chart, table or text, without recalculating the sheet (the guide chapter "Interactive panels", book/src/python-panels.md).

Sheet lines and raw lines share one worksheet, so they alternate freely: a raw block reads
the variables defined by sheet lines above it and defines variables that sheet lines below it
use, and prose, quantities, LaTeX and raw Calcpad can come in any order. To make that work
without remembering renames, an identifier in a raw line that the sheet defines, or that a raw
line assigns (`sigma = 3`), is spelled the way the sheet spells it: `phi` in a raw line is
`φ`. The name a raw `#UI` line assigns (`>> #UI phi = 0.9`, also with options and a label in front of the name) counts as an assignment too. (A dot is the index operator, not part of a name.) (Text after a `'` or `"` in a raw line is left alone. `Show compiled` shows the result.)

A raw block inside a sheet block keeps the block's indentation rules like any other line.

## Importing spreadsheet cells

**Import spreadsheet** (Document tab, and the File menu) reads a `.xlsx`, `.xlsm`, `.csv` or `.tsv` file and writes
cells into the sheet as ordinary lines at the cursor: `name: ? 6 m` for a constant (an input), `name: 56.25 kN*m` for a
value, `name: [1, 2; 3, 4]` for a range of numbers, one quantity line per row for a label and value range, and a pipe table for
text. Each group has a source comment above it, `<!-- file.xlsx, Sheet1!B1 -->`, an HTML comment that prose passes through
and the page does not show. The values are copied: formulas are not recalculated (the value the file stored is used) and there is
no live link to the file. Names come from the label beside or above the cell and follow the rules under "Quantity lines".
The guide chapter "Importing a spreadsheet" (`book/src/importing-spreadsheets.md`) describes the panel;
`web/e2e/sheet-import-check.mjs` and `web/e2e/sheet-import-logic-check.mjs` test it.

## Inputs and templates

An input line makes a value the reader of the sheet can change. It is a quantity line whose right side
starts with `?`:

```
L: ? 6 m | span
E: ? [Steel = 200 GPa, *Timber = 11 GPa, Alloy = 70 GPa] | material
k: ? (Low = 1, High = 2) | load class
ribs: ? on | stiffeners fitted
w: ? 12 kN/m [2 .. 30 step 0.5] | load
```

These are, in order, a number with its unit and a note, a drop-down list (the option marked `*` is the one selected at
first), radio buttons (the same options in round brackets), a checkbox and a slider.

* **A number** is the default, written as a number with an optional unit (`6 m`, `12 kN/m`, `0.9`).
  It is not an expression: `2*3 m` is an error on its line. The unit is shown after the box. Units may use the usual spellings: `m³`, `N/mm²`, `kN·m`, `kg/m³`, `m·s⁻¹`, `°`, `°C`, `%`, `µm` (they are written to the engine as `m^3`, `N/mm^2`, `kN*m` and so on). The engine's own `#UI` line takes only the `^` and `*` spellings, so use those in a `>>` line.
* **A slider** is a number with bounds after it: `w: ? 12 kN/m [2 .. 30 step 0.5] | load`. The form draws a slider with
  the box beside it. The bounds are plain numbers in the unit of the default (here kN/m), `step` is optional (without it
  the slider moves in a step of 1, 2 or 5 times a power of ten that gives about a hundred positions), the minimum must be
  below the maximum, and the default must lie between them; otherwise the line is an error. The bounds also check the
  box: a typed value outside them is not used and the box shows `must be from 2 to 30` beside it. Moving the slider
  re-runs the sheet after a short pause (a quarter of a second), and it works by touch and with the arrow, Home,
  End and Page keys. The slider is drawn by the input form only; the report, the PDF and the exports show the
  number. Typing a value between the steps is fine (the step belongs to the slider).
* **A choice** is a list of `label = value`. The value is a number with an optional unit, as above, and
  it is what the name has when that option is chosen. Without a `*`, the first option is the one selected.
  A label with a comma, an equals sign or brackets is written in double quotes (`"Beam (A), long" = 1`);
  a label cannot hold `}`. A choice needs two options or more, and no two options may have the same value (an error on the line). The default is always one of the options, and the form opens on it whatever its unit.
* **A checkbox** is `on` or `off` (`true` and `false` also work) and the name is `1` or `0`: `if ribs == 1:`.
* **The note** after `|` is shown after a number, as on a quantity line. A drop-down, radio buttons and a checkbox
  show no name, so their note is the label in front of the control (the name is used when there is no note).
* In Input mode, consecutive input lines are drawn as one block: the names, the number boxes (and drop-downs), the sliders,
  the units and the notes line up in columns. The names are right aligned so the equals signs meet, a very long name is
  held to 40% of the width and wraps, and a text line, a heading or a blank line starts a new block. A single input line
  is drawn on its own. This is how the form looks only; the report and the exports are not affected.
* One input per line, and the name is a plain name (Greek spellings as always). `name = ? 6 m` works too.
  The name is defined like any quantity: later lines, `>>` lines and Go to definition use it.
* A choice gives one number. To choose among several properties at once (a material with its modulus, strength and
  density) give the choice an index and look the rest up in vectors defined after it. The templates do this:

```
mat: ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
E_tab: [200 GPa, 70 GPa, 11 GPa] | modulus of elasticity
E: E_tab.mat -> GPa
if mat == 2: "The material is aluminium."
```

A run of input lines is one **form**: a heading and the input lines under it make a group, drawn with a bar at its
left edge. In plain Calcpad (Convert to Calcpad, Show compiled) the bar is not written.

### What the compiler writes

An input line is compiled to the engine's own `#UI` control, so the input form, the report, the Word and Excel
exports and the `uiOverrides` the page keeps all work as they do for a Calcpad worksheet:

| Sheet | Calcpad |
| --- | --- |
| `L: ? 6 m \| span` | `#UI L = 6m 'span` |
| `E: ? [Steel = 200 GPa, *Timber = 11 GPa] \| material` | `#UI {"type":"dropdown","keys":["Steel","Timber"],"values":["200GPa","11GPa"]} 'material -'E = 11GPa` |
| `k: ? (Low = 1, High = 2)` | `#UI {"type":"radio","keys":["Low","High"],"values":["1","2"]} 'k -'k = 1` |
| `ribs: ? on` | `#UI {"type":"checkbox"} 'ribs -'ribs = 1` |
| `w: ? 12 kN/m [2 .. 30 step 0.5] \| load` | `#UI {"min":2,"max":30,"step":0.5} w = 12kN/m 'load` |

Use **Show compiled** to see it for a whole sheet. Bad input lines (no default, an expression for a default, a choice
with one option or with an option that has no value, an unclosed bracket, two inputs on one line) are errors on their line,
in the report and as editor markers, and say what to write. A line such as `Why: ? is it` is prose, with a warning.

### How the form behaves

* Open a template and choose **Input** in the results bar. The page shows the sheet with a box, list, button or
  checkbox in place of each input line, and the results below it recalculate when you change one (press Enter or leave
  the box). **Report** and **Print PDF** next to it use the entered values, as do the Word and Excel exports.
* The values are held in the page, so typing never changes the file. **Save values** writes them into the sheet as a
  comment line directly under `<!--sheet-->` (`'<!--{"uiOverrides":{"L:1":"8"}}-->`); the compiler passes that line
  through, and they are applied again the next time the sheet is opened in Input mode or as a Report (Preview shows the
  sheet's own defaults, unless *Apply #UI Values in Preview* is on in the settings). Edit the number in the sheet itself
  to change a default, or use **Update defaults** in the results bar, which writes the entered values into their lines
  (a drop-down or radio group moves its `*`, a checkbox becomes `on`/`off`), and **Reset**, which drops them. An input
  line whose entered value differs from its default carries a faint `← form: value` mark in the editor. The setting
  *Keep Input Form Values in the Source* (Settings, Editor) writes each entered value into its line as it is entered.
* Inputs hold numbers only (the engine's own limit), so there is no text box: a choice of names is a drop-down.
  A plain number input has no minimum or maximum; give it bounds as a slider (above) or check a range with a
  condition, `if L > 30 m: "Span out of range"`. A value outside the bounds of a slider that reaches the engine
  from saved values is an error on the input's line.
* The report shows the number a choice stands for, not its label (`mat = 2`). A line such as
  `if mat == 2: "The material is aluminium."` names it, and the templates do.

### Templates

The Samples dialog has a **Templates** group of seven sheets that use inputs: a simply supported beam check, a
cantilever with a tip load, a rectangular column (slenderness and buckling), a pad footing bearing pressure check, pipe flow
(Darcy-Weisbach), a retaining wall (sliding and overturning, with radio buttons for the backfill and a surcharge checkbox)
and a pump and pipeline operating point (a drop-down, radio buttons, a checkbox, a table of results and a plot of both
curves). They use generic textbook formulas and no design standard. Open one, press **Input**,
change a value. To make your own, write the inputs as above, put the results under them and save the file. Sharing a template is sharing the `.sheet` file (or the text of the sheet); anyone who opens it gets the
same form, and **Save values** keeps their numbers in their copy. A new input is one more line, a new option one more
entry in a list.

## Folding sections and key results

Two small markers shape how a long sheet reads without changing a single number.

```
## Workings {collapse}
...the long derivation...

## Results
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment {key}
utilisation: Mstar / (0.9 * Ms) | bending utilisation {key}

## Key results
{key results}
```

* **A folding section.** A heading that ends with `{collapse}` starts a section that is folded away in the preview
  and the report until the reader opens it (a click on the heading; a small arrow shows the state). The section
  runs to the next heading of the same level or a higher one (`#` is the highest), or to the end of the sheet.
  `{collapse open}` makes it open when the page is first drawn. Its lines are calculated whether it is folded or
  not, so a folded section can define names the rest of the sheet uses. A folding section can hold other headings
  (a `###` inside a `##`), and a deeper section may fold on its own. Only a heading at the top level of the sheet
  can fold: a `{collapse}` heading inside an `if`, `for`, `while` or `repeat` block is a warning and is ignored.
  This marker is part of the sheet and travels with the file. It is separate from the fold arrow the preview shows
  beside any heading (a view-only fold that is not saved in the sheet): a `{collapse}` heading folds by its marker
  alone and has no preview arrow, while the arrow works on every other heading. Folding an unmarked heading above a
  `{collapse}` section hides the whole section, and "Collapse all" in the heading menu leaves marked sections as the
  sheet wrote them.
* **A key line.** A quantity or input line that ends with `{key}` (after the note, if there is one) is a key line.
  It is marked with a star and highlighted in the preview and the report, and Print and PDF keep the highlight.
  Several statements on a line (`a: 1; b: 2 {key}`) are all key. It works the same on an input line
  (`L: ? 6 m | span {key}`).
* **A key results list.** A line that is just `{key results}` becomes a list of the key quantities above it, each
  with its name, value and unit as the engine shows them, and the note. It lists the top-level key lines written
  before it, once each, in the order written; a key line below the list is not in it (the page has no value for it
  yet), and a list with no key line above it is a warning. Put a heading in front of it as above. A key line
  inside a block, or on a function definition, is highlighted but not listed.
* **Print, PDF, Word and Excel.** The Print / PDF output opens every folded section, so the printed report is
  complete. The Word and Excel exports also include folded sections in full, and a key line carries its star
  in the line. Leaving folded sections out of an export is not offered.
* **Plain Calcpad.** Show compiled and Convert to Calcpad give plain text for these: a folding section is
  the heading in a `<details>` block of text lines and a key line has a star in its comment.
* The formatter writes `{collapse}`, `{collapse open}` and `{key}` as one space and the marker, and does not change
  anything else about these lines.

## Errors

The engine's errors, the report's "on line [n]" links, the error banner, the scroll anchors and
the editor's lint markers all report the sheet line, whatever kind of line it was (quantity,
raw, inside a block). The text of an engine message still quotes the compiled Calcpad line,
which is what Show compiled displays. The compiler's own problems (mixed indentation, an
`else` with no `if`, an empty block, an unclosed `$$`, a bad plot line) are markers on their
lines too. "Unused variable" hints are dropped for sheets, since every quantity is shown.

## Not supported, and the limits

* No `repeat` without a count (use `>>`); no step on a loop over a vector.
* No units on vector or matrix literals; no nested brackets; no symbolic algebra (the engine
  is numeric, as Calcpad is).
* Calcpad's own highlighting, symbol lookup and prettifier do not apply to a sheet; the sheet
  has its own (next section).
* A formula in Word or Excel is its TeX source, not Office Math.
* A name that is a Calcpad function (`gamma`, `beta`) as a variable keeps working only when
  written the Calcpad way; function calls are not renamed unless the sheet defines the function.
* `Convert to Calcpad` is one way: the Calcpad text is not turned back into a sheet.

## The editor

A sheet has its own colours, and these work on names:

* **Completion** while you type: the names the sheet defines above the cursor (with their last calculated
  value), the parameters of the function being written, built-in functions with their signature, units after a
  number or `->`, Greek spellings, and at the start of a line the block keywords, the `>>` marker, code fences,
  `$$`, the input forms and `<!--sheet-->`. Nothing is offered in prose, in a note after `|`, in a string,
  in a formula or in a Python block. Tab accepts.
* **Rename everywhere** (F2, the context menu, or the Variables tab): every use of a name in code, in one undo
  step, with a preview of the count and a refusal for a name that collides. A function's parameters and the variable
  of a `plot` line are renamed only on their own line. Prose, notes, strings, `$...$` formulas, the unit after a
  number and Python code are left alone, except the names a Python block `export`s, which are the sheet's
  variables. See "Rename everywhere" in the user guide's Editing chapter.
* **Variable palette** (Ctrl+;, Cmd+; on a Mac): a search box over every name with its value and unit; Enter inserts the name, Alt+Enter goes to its definition.
* **Format Document** (Shift+Alt+F) prettifies: one space around `:`, `|` and `->` and after
  `;`, a blank line before each heading, runs of blank lines collapsed, and the colons of a run
  of quantity lines aligned. Expressions, prose, `>>` lines, fences and formulas are not touched,
  and formatting twice changes nothing.
* **Go to definition** (F12), **find references** and **hover** (the defining line and the
  Calcpad spelling) work on the names a sheet defines; references are searched in code lines, not
  in prose, and a function's parameter is a separate name. Names are also coloured by semantic highlighting.
* **Text toolbar**: a floating bar over prose (Style, bold, italic, code, link, lists) whose tooltips show the Markdown
  each button types.
* **Show compiled** and **Convert to Calcpad**, above.
* **Rich-text editing**: click prose in the preview (a paragraph with its heading, a list, a
  table) to edit it as formatted text in place; input lines and calculations stay one-line edits.
  See `docs/rich-text-editing.md`.

Under each plot in the preview there is a row of chips. A figure drawn by a Python block gets Log x, Log y, 1:1,
Copy data, PNG and SVG (the 3D scenes get Copy data and PNG). A `plot` line gets Copy data, PNG and SVG; its data is
read back from the drawing through the axis labels, so it is the plotted values to about four figures. The chips are
only in the preview, not in print or export.

In a sheet, one click on a block in the preview edits it where it stands (a drag that selects text does not): the
editor opens over the block, clicking away applies the change, Esc throws it away, Enter (Ctrl+Enter for prose) applies
at once. On a touch screen a long press opens the editor and Apply and Cancel stay on screen.

## Ribbon and the Insert tab

Between the rendered blocks in the preview, a small **+** (a tab on the left edge on touch screens) opens a menu to insert a
block there: Math line, Text, Heading, List, Table, Plot, Condition or Python. The new block goes in front of the
next block's source line, indented like it, and its first word is left selected in the editor.

In a sheet the ribbon and the sidebar's Insert tab insert the sheet form: Heading is
`## Heading`, Text is a prose line, Table is a pipe table, Plot is a `plot` line, the
conditions and loops are the indented blocks, and the Text Formatting tab writes Markdown
(Bold `**x**`, Italic `*x*`, H1 to H5, Bullets and Numbered lists, Link, Code), a definition such as `c = 299792458m/s` is
`c: 299792458 m/s`. Functions, operators, Greek letters, units and constants are the same in
both syntaxes (with the argument and matrix punctuation above). Anything with no sheet form
(the data grid and the other `#UI` forms, `$Map`, directives, macros) inserts its Calcpad text as `>>` lines, and its
button carries a small `>>` mark and a tooltip saying so; the sidebar's Insert tab does the same.
Each open file has its own mode and the ribbon and the Insert tab follow the active tab.

`web/e2e/verify-ribbon-sheet.mjs` runs the sheet form of every ribbon button and palette entry
through the compiler and the engine.
