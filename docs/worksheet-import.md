# Importing foreign worksheets (spike)

Crate: `calcpad-import` (package `civilkit-calcs-import`, not published). It reads the
`.xmcd` and `.mcdx` worksheet files of a commercial calculation tool, the spreadsheet-like worksheet format with
text, equations and plots placed freely on a page, and writes **Calcpad worksheet text** for the existing
engine. It evaluates nothing itself: one engine, one language. Every number in the output is a literal
that was in the source.

It is a spike. It answers two questions: how much of real files converts, and which missing features
come first. The two file formats share one translator; see [The .mcdx front door](#the-mcdx-front-door).

## In the app

`File > Import worksheet...`, the **Import worksheet** button on the Document tab, `File > Open...` with a `.xmcd` or
`.mcdx` file, or dropping one anywhere on the page (a dashed hint shows while a file is dragged over it).

1. The file is read in the page and converted by the importer module (`calcpad_import_wasm`, in its own web
   worker, loaded on first use; the engine module is not involved). **Nothing is uploaded**: the file never leaves
   the browser.
2. The picture files go into the library under `/imported/<file name>/` (letters, digits, `_`, `-`; importing the
   same file name again replaces that folder).
3. The converted worksheet opens as a new unsaved tab named `<file name>.cpd`.
4. A dialog says how many regions converted live, how many are shown as static pictures and how many need
   attention, and lists the top reasons in plain words ("calls to a function from an add-in we cannot run",
   "embedded objects", ...). Two toggles re-run the import in the same tab: **Pictures of unconverted regions**
   (default on) and **Show saved results as static values** (default off).

Errors are messages, never crashes: a file that is not `.xmcd` ("... is not a worksheet this importer reads"),
malformed XML or a DTD ("not well-formed XML"), text that is not UTF-8, and a `.mcdx` zip that cannot be read
("the file is a zip package that cannot be read").

**Pictures and Save.** The `.cpd` text refers to pictures by library path (`<img src="imported/<name>/R12.svg">`).
The report, the preview and the HTML, Word and Excel exports resolve those paths against the library and inline
the pictures (a test checks the rendered page and the Word package). Save writes the `.cpd` and keeps the pictures in
the library, which is stored in the browser: the pictures stay available as long as the library does. A `.cpd`
opened later from disk shows its pictures only while the library still holds them (it does not embed them);
exporting to HTML or Word first gives a self-contained file.

**Limits.** The module is built for the browser, which has no threads and no unwinding. Metafile conversion is bound
by a record-chain check (a file with more than 200 000 records, or a record chain that does not add up, is refused
and the raster picture used) plus the size caps below. If the module traps anyway, the page starts a fresh worker and
retries once with vector pictures off, so raster pictures are used instead. Input over 64 MiB, XML nested deeper than
256, a DTD, pictures over 16 MiB each or 192 MiB in all are refused or skipped as described under Security.

Size of the module: 1,316,704 bytes raw, 534,343 bytes gzipped (`tools/build_wasm.sh import`); the `.mcdx` reader, the translator features, the workbook reader and the equation display added about 100 KB.

## Use

```text
cargo run -p civilkit-calcs-import --example import -- <file.xmcd>   # text on stdout, report on stderr
```

```rust
let r = calcpad_import::import_xmcd(&xml);      // r.calcpad: String, r.report: ImportReport
println!("{} of {} regions need attention",
         r.report.regions_needing_attention(), r.report.regions_total());
```

The report has, per region kind (`text`, `math`, `plot`, `image`, `ole`, `component`, `reference`,
`other`), the totals, how many converted, how many converted with a note, and how many need attention.
Page breaks are layout and are not counted. `report.issues` lists every issue with a stable code, a
short reason, the `region-id` and the source line. `report.ranked_gaps()` ranks the missing features by
the number of regions they block.

### Text, pictures and layout

- **Text styles.** The named paragraph styles of an `.xmcd` file (font family, size in points, weight, slant, underline, colour,
  alignment) and the per-run font and colour elements are read. Each styled paragraph is written as `<p style="...">` (a heading
  style stays `<h1>`/`<h2>`) with the family as a CSS stack (the file's name, then metric-compatible and generic fallbacks, with
  no quote characters because a `'` inside a text line would switch the engine to maths), the size in `pt`, and a run that differs
  from its paragraph's style gets a `<span>` of its own. Alignment other than left, the left margin and the first-line indent are
  kept. A `.mcdx` file carries these on the XAML elements (`FontFamily`, `FontSize` in pixels, `Foreground`, `TextAlignment`,
  `TextIndent`, `Margin`) and has no named style table here. Maths keeps the app's own styling. The file's alignment is what is
  written: the source of the seismic sample says `left` everywhere, so the importer does not justify text the original page shows justified.
- **Colours and the dark page.** A colour becomes a class (`imp-c-rrggbb`) whose rules are written once, in a `<style>` line at the
  top: the source colour on a light page and, under `.dark-theme`, the same hue lightened to about 70 % lightness with its
  saturation held between 0.35 and 0.6, so dark red and teal headings stay readable on the dark preview. A very light colour is
  darkened on the light page. Black, white and greys carry no hue and are left to the app's text colour.
- **Pictures.** A `<png>` or `<metafile>` written straight into a region is a picture region like any other: it keeps its size and
  position and carries no caption. The italic "static picture of the original" caption is only for a picture that stands in for
  something that did not convert (maths, plots, components).
- **Picture beside text.** A picture with text regions to its right that overlap its vertical extent is one flex block: the picture on the left and
  those texts stacked beside it in reading order, whatever baselines the texts sit on (a title picture with a chapter line and a
  title line). Text below the picture is not pulled in.

### Equations as the author wrote them

### MathML display

By default the author's equation is written as MathML (`<math>` inside `<div class="imp-eq">`), built from the parsed source
tree by `calcpad-import/src/mathml.rs`: stacked fractions, `munderover` sums and products with bounds, integrals,
subscripts (`C_(v,i)`, `V_1`), a vector arrow over the whole product, a transpose superscript, bracketed matrices,
`:=` for a definition (`≡` for a global one) and `=` for an evaluation, original names (`SIUnitsOf`, a variable renamed
for the engine shows under its own name). One rule sets names: the tool's own functions (`sin`, `if`, `SIUnitsOf`) are upright;
every other name is an italic variable, a function the page defines included, so `V_AB(x) := ...` and a later call
`V_AB(L_AB)` look the same. A one-letter name the file does not label as a unit is a variable. Units are bold, blue
italic (a number before its unit has no dot, `100 ft`), and the keywords `if` and `otherwise` and the `=` of a check are
bold; these use a text face because the math face has no bold weight. The vector arrow is drawn by the stylesheet as a line
and a head across the whole operand (the `mo` stays for the Office Math export). Text colours follow the theme: units
have a light and a dark blue, everything else is `currentColor`. The computation is the unchanged hidden code.

An expression of a range variable (`s_i / ft =`) is typeset the same way: the author's left side, then the collected
column of values as a bracketed table in the file's decimal places. A quotient of units in a result is a stacked
fraction and a fractional power is a small fraction (`ft^(3/4)`).

An evaluation is `<math>` with the left side typeset, then the engine's value through `#val`: the page switches on
`#settings {"valMathml": true}`, an opt-in engine setting under which `#val` writes the value as MathML (bracketed table,
unit once after it, the file's decimal places through `#round`). Nothing in the engine changes unless the setting is on.
A region whose equation has a node without a MathML spelling (a program body, for example) keeps the typeset-Calcpad
display below. The editor accepts `valMathml` as a known setting, and the sheet carries a `LintIgnore` comment for the
unused-name hint (a name used only by a picture or an unconverted region is not a finding). `ImportOptions::mathml = false` (the wasm option `mathml`) uses it everywhere.

Exports: the Word export converts each `<math>` to Office Math (fractions, sub and superscripts, sums, brackets,
matrices, accents) and the Excel export writes a linear text form with the matrix cells apart. Print and PDF use the
browser's own rendering. Source that is not maths (a program body, an unreadable string) is never shown as code; the
region keeps its quiet "not calculated" note.

A converted math region is shown as the author's equation and computed out of sight, so the page reads like the source.

- **A definition** (`:=`) is computed inside `#hide` ... `#end hide` and shown as a typeset `#noc` block that carries the
  source's own names, subscripts, operators and matrix orientation: `s = [16; 12; 13.5]^(T)*ft` (a row transposed, a
  superscript T, a dot product), with **no result after it**, as in the source. The computation is still in the text, so the
  engine records its errors as before. An element definition over a range variable (`h[i] := ...`) is shown the same way
  (`h.(i) = ...`, indexes as written, no shift to the engine's origin). A definition that cannot be written faithfully
  (a program block, a range, a call result that is indexed, a gap) keeps the earlier form, computed and visible.
- **An evaluation** (`=`) is the expression and its result, to the precision the file used (below), under a document-wide
  `#nosub` so the substituted step the engine normally prints is left out, as in the source. When a name in it was renamed
  (a variable that has the name of a unit, `s_v` for `s`), the author's expression is typeset first in a `#noc` block and
  the engine's result line follows, so the original name is on the page and the engine name is too. A result that is an
  array is wrapped in a scrolling box (`imp-scroll`) instead of wrapping over several lines.
- **Names without the engine's marks.** The engine draws an arrow over the name of a vector, writes a matrix name in bold
  and marks a power of an array with a small circle. The source has none of these, and they cannot be turned off in a
  `#noc` block (they follow what the name holds), so the page carries a small `<style>` that hides them
  (`.vec`, `b>var`, `sup>small.nth`). The same rules hide them in the worksheet's own results.
- **Precision.** The file stores the decimal places per result (`precision`, with a document default). The importer writes
  a `#format 0.###` (as many `#` as places) before the results that use it, only when it changes. `#format` is display
  only: every value is computed in full (a test checks that `2/3` shown with three places still gives 1000 when
  multiplied by 1500). It is used instead of `#round` because `#round` keeps one more digit for values below 1
  (`0.6667` for three places) and the source shows `0.667`. Numbers written inside a typeset equation follow the format
  in force too, so a shown equation is always preceded by `#format default` (and a `#round 12` at the top of the sheet
  keeps the literals as typed). Trailing zeros and the source's exponent threshold are not reproduced.

Differences that remain: the engine spells some units differently (`s` for `sec`), prints a unit after every element of a
vector (the source prints it once after the brackets), writes a transposed result as `transp(h)` (the engine has no
transpose operator), shows `≡` for an equality and typesets a row of a matrix literal with `;` and a column with `|`.

### The placeholder rule

A region that does not convert is never dropped and never guessed. Its original maths is shown, as a
typeset equation with the real names, and is never evaluated:

```text
'<!-- [import-gap R41 undefined-variable,depends-on-gap] name used before it is defined ... --><small>not calculated: undefined-variable, depends-on-gap</small>
#noc
A = submatrix(N; 1; n_n; 3; 3)*cm^2
#end noc
```

The machine-readable marker (`[import-gap`, the region id, the code or codes, the full reason) is an HTML
comment, followed by a small muted note; the equation is the visible thing. `#noc` renders a line without
calculating it, so a name the sheet does not define, or a function the engine does not have, raises no error
and nothing cascades. What cannot be Calcpad maths (a string, an indexed call result, an element assignment, a
function with an unresolved call, a name Calcpad cannot spell) is shown as code text instead, so a block never
makes the engine report an error. A program is shown as its statements: assignments as equations, `for`, `while`
and `if` as indented text. A region with no maths (an embedded object, a plot) is a comment and a one-line note
with the reason. Input text reaches the output only through the same doors as before: escaped prose, names that
pass the Calcpad identifier check, and numbers that parse as finite numbers. A test plants a payload in every
kind of unconverted region and checks that it only ever appears inside a marked gap.

## What converts

| Source construct | Calcpad output |
| --- | --- |
| Text region | One `'` text line per paragraph. Styles `Heading 1..3` become `<h1>..<h3>`; bold, italic, underline, sub and superscript are kept; colour and font are dropped. Text is HTML-escaped and quote marks doubled, so prose cannot start a directive. Empty paragraphs are skipped. |
| Math nested inside a text region | Becomes its own region, ordered by its own position. |
| `x := expr` | `x = expr` |
| `x ≡ expr` (global) | `x = expr` at its reading-order position, with note `global-definition`. A global whose value is only numbers and units is also defined first, in a `#hide` block at the top, because a global holds wherever it is written (a use above it reads it). Other globals are not hoisted. |
| `expr =` | the bare expression |
| `x := expr =` | `x = expr` (a display unit on the result is dropped) |
| `expr = ` with a display unit | `((expr)/unit)*unit`, note `unit-override`. The source takes the unit as the part of the result's unit to show first and keeps the rest (`kip` on a moment shows kip times length); the engine's `\|` conversion refuses a unit of another dimension. A temperature unit keeps `\|` because of its offset. |
| `f(a, b) := expr` | `f(a; b) = expr` |
| `+ - * /`, powers, unary minus, factorial, comparison, and/or/xor | the same operators, with parentheses added from the tree, never from the source |
| `sqrt`, `\|x\|` | `sqrt(x)`, `abs(x)` (note `abs-of-array` when the operand is a known matrix or vector) |
| `x_i`, `ρ_tp` (literal subscripts), Greek names | `x_i`, `ρ_tp` |
| A name the engine cannot spell (dots, `#`, `/`, `:`, spaces, a leading digit: `SF_crack.step2`, `U/R_twist`, `Rod#_top`) | One engine spelling for the whole sheet, `SF_crack_step2`, `U_R_twist`, `RodNum_top`: every other character becomes an underscore, `#` reads "Num", `%` "pct", a leading digit gets a `v`, and runs of underscores collapse. The spelling is chosen once per distinct original (in order of first appearance) and applied to definitions, uses, function names and parameters, program locals, plot expressions and input components alike (`importer::names`). When it equals a name already in the sheet, or would be read as a unit, a constant or an engine function, `_2`, `_3` (or `_v`) is added. The typeset equations show the original: `SF` with the subscript `crack.step2`. A name that carries a superscript caret (text the reader cannot take for a name) stays an `invalid-name` gap. |
| Text values (`"OK"`, `"NG"`) | Held as whole numbers, the code of the literal in a table of the sheet: `Check := if(x >= 2; "OK"; "NG")` is `Check = if(x ≥ 2; 1; 2)`; two texts are equal when their codes are; a function that returns one works the same. The page shows the author's equation and, for an evaluation of a text, the text of the current code, chosen by an `#if` chain over the texts that name can hold. Which names hold texts is tracked in reading order, so a name that is a number before and a text after (a selection index replaced by its label) is read at each point as what it is there. A matrix literal that mixes text labels with numbers keeps its numbers and holds the labels as codes (a printed evaluation of such a matrix is the `string-value` gap). A text used as a number, joined, sliced, or given to a function (a file name) is the `string-value` gap. |
| Input controls: combo box, text box, list box | The control's stored value is the input: `x := <control>` becomes `x = <stored value>`. A combo box gives the value of the selected row (times the control's scale factor, a unit as a rule), a text box its text (a number when the text is one, otherwise a text value as above; `str2num` of a number-text box is the number), a list box the position of the selected item, counted from 0 (the worksheets compare it with whole numbers). A note line above the equation says what was taken (`combo box input, stored selection "..." of 3 options`) and the note `control-input` is added; the options, the script behind the control and its links to other names are not carried. A button or check box stores a value too (a press counter, a folder text, one value per name it defines): a definition from it takes that value with the note `control-input`, and a button on its own is its label in square brackets. A button that saved nothing stays an `unknown-element` gap. |
| A value that is a number on some paths and a text on others (`Psi := if(c >= 8d; 1.0; if(c < 3d; "Testing required"; 0.7 + ...))`) | The text is a code as above, so the value is held as a number or a code. Used in arithmetic it is `if(Psi ≡ code; sqrt(-1); Psi)`: where it holds the text the source stops at that use, so the result is undefined (not a number), never a number made from the code. Evaluating it prints the number, or the text when it holds one. |
| A vector of texts (`msg[i] <- "OK"` in a program, `v[k] := if(c; "OK"; "No Good")`, an element compared with a text) | The vector holds the codes; an element compared with a text is compared with its code; an evaluation prints a column of the texts. Only for a vector whose elements are all texts: a vector that mixes texts and numbers stays a `string-value` gap, because a code cannot be told from a number. |
| A name or an expression on its own with no result saved (`Width` beside an input, `20 d`, a table of numbers) | Shown as written, with the note `inert-expression`: the source computes nothing there. A comparison on its own, one with a gap of its own (a solve keyword, an element the importer does not know) and one with a saved result stay `bare-expression` gaps. |
| `.mcdx`: a reference to another worksheet, when the package holds a copy of it | The definitions of the copy (its math regions; ids `i<n>_<id>`) are read in at the reference and computed, in a hidden block, with no equation shown; the reference is one line of text. Evaluations, text, plots and pictures of the copy are not read, and a copy that holds references is not followed. A reference with no copy stays `worksheet-reference`. |
| `.mcdx`: a solve block (guess values, constraints, `Find`/`Minerr`) whose solver saved a value | Not solved. The guesses stay definitions, the constraints are shown as written (note `solve-constraint`), and the unknowns take the values the solver returned, frozen and labelled (note `frozen-value`), in the unit the result was shown in. A solver that saved no value (the solve failed in the source) leaves the block an `other-region` gap. |
| `WRITEEXCEL`, `WRITEPRN`, `APPENDPRN`, `WRITEBMP` as a definition or an evaluation | A write to a file cannot be done here and nothing the sheet computes depends on it: the call is shown as written with the note `file-output`, and a name it defines stays undefined. The file names in the call are not text values. |
| `csort(A, j)`, `rsort(A, i)` | `sort_rows(A; j)` and `sort_cols(A; i)`, the index counted from the sheet's array origin. Rows with equal keys may come out in another order than in the source. |
| A program whose value is its last `if` (`if a {x}`, then `if b {y}`, no `else`), with `alsoif` as `else if`, and `return v <- e` | The value of a program is the last expression it evaluates, so the chain is `if(b; y; if(a; x; ...))`: a later `if` that runs wins. A path along which no `if` ran has no value in the source (it stops there); the engine takes the value of the neighbouring branch there and the region carries the note `uncovered-path`. A local that is only ever given texts (a message set by `if`s) is the empty text on such a path instead. |
| A program that ends with an assignment (`Fc <- Fc*ksi`), or a block inside a block | The assignment gives its value. A block in a block runs in place. A run of `if`s that only give values is one chain however many there are (no path is copied for the statements after it). |
| `T := READEXCEL(...)` or `READPRN(...)`: a table read from a data file | The importer cannot see the file, so the table is the value the worksheet saved where it shows it (`T =` further down), as a literal, with the note `frozen-value` and a line that says so. A heading becomes a text code and an empty cell (stored as NaN) is read as 0, which the line counts. The file names in these calls are not text values: a read with nothing saved for it is the `builtin-missing` gap. |
| A local read on a path where no statement set it (`val` after `if n = 1 {val <- 1}`) | No value there (the source stops): that path is undefined (the engine's not-a-number), never a neighbouring branch's value, note `uncovered-path`. Only for a name the sheet has no value for; a local that shadows a name the sheet defines is not treated this way. |
| A chained comparison `a < b <= c` | `(a < b) ∧ (b ≤ c)` |
| A text on its own in a math region | A quiet line showing the text (a label the author put in the worksheet) |
| `x%` | `x*%` (the percent sign is a unit) |
| `nthRoot` with a degree (`nthRoot(3; x)`) | `x^(1/3)`; the degree is the first operand, as the empty first slot of the square root shows |
| `plf`, `pcf`, `klf`, `kcf`, `pli`, `kips` | `(lbf/ft)`, `(lbf/ft^3)`, `(kip/ft)`, `(kip/ft^3)`, `(lbf/in)`, `kip` |
| A bare `0` against a quantity with units: `if(c; 0; x)`, `max(0; x)`, `x ≡ 0`, `0 - x` | `0*getunits(x)`: the source lets a literal zero take the units of the other operand, the engine wants them equal. `getunits` reads the unit without evaluating the value, so a guard such as `if(s = 0; 0; m/s)` stays safe. |
| Vector, matrix literal | `[a; b; c]`, `[a; b\|c; d]`, a row vector as `transp([a\|b\|c])`. The file stores matrices column by column; the importer reads them that way. The engine keeps vectors and matrices apart (`len`, `max` and one-index access want a vector), so the transpose of a row literal is written as the column vector `[a; b; c]` and a double transpose is the operand. |
| Product of two column vectors (`u(i) * Θ(i)`, a scalar times either) | `dot(a; b)`: the source multiplies two column vectors as a dot product, the engine's `*` of two vectors is element by element. Applied when both operands are statically column vectors (a column literal, a transposed row literal, or a name or function the importer saw defined as one); two arrays whose shape is not known keep `*`. |
| `[A, B] := [1, 2]` (a matrix of names) | One definition per name, each from the item in its place; the matrix must be a literal of the same size. |
| `v[i]`, `M[i, j]` | `v.(i + 1)`, `M.(i + 1; j + 1)` for an array origin of 0; unchanged for 1; a literal index is folded. Note `array-origin`. |
| `ORIGIN := n` (integer literal) | Later indexes use `n`. Note `origin-assigned`. |
| Units | See below. A product written with a middle dot (`kN·m`, `N·m`) is a unit when every factor is: `(kN*m)`. |
| Integral `∫ f dx` from a to b | `$Integral{f @ x = a : b}`. A bound written as a bare 0 becomes `0*(other bound)` so both bounds carry the same units. The bound variable is local to the body. A bound that uses the variable's own name, or an operand layout that is not `lambda`, `lowerBound`, `upperBound`, stays a `calculus-operator` gap with the reason. |
| Range `i := a .. b` (integer literals, a <= b) with `v[i] := expr(i)` | The range line becomes a comment (note `range-loop`) and records the bounds. The element assignment becomes `#hide`, `v = vector(n)`, `#for i = a : b`, `v.(i) = expr`, `#loop`, `#end hide`: the vector is zero-filled, which is what the source has for the elements below `a`, and `expr` may carry units. `#for` and `vector(n)` were chosen over `range(...)` (a vector, not a loop variable) because an expression of `i` is evaluated one `i` at a time in the loop, as in the source, whereas Calcpad vectors do not combine element by element. Only this shape is read for a range: the array must have no value yet, the value must not read the array, the indexes must be one or two different bare range variables (two fill a matrix with `matrix(m; n)`), and the first bound must not be below the array origin. The bounds are expressions the engine evaluates, so run-time bounds work; a range whose end is below its start counts down as the source does (a counted loop that derives the variable). The array is as large as the largest index any range reaches, and an element no iteration writes stays 0 where the source has no such element. Other reads of a range variable (outside a sum, a table or an element assignment) are a `range-variable` gap that says so. A range with a step (`x := 0, 5 .. 100`) counts `floor((b - a) / step + a little)` passes from `a` in steps of `second - first`, up or down; it is a table, a plot domain or a loop like any other range. A step of zero is not a range. |
| Sum or product `Σ f(k)` with bounds, or over a range variable | `$Sum{f @ k = lo : hi}` / `$Product{...}`. Without bounds the operator runs over the range variable of the bound name, using the bounds that variable was given (the index of an array shifts with the origin as everywhere else). The bound name is local to the body, so a sum over `i` inside an element assignment over `i` is read correctly. A bound that uses the bound name's own spelling, or a name that is not a range variable with bounds, is a `summation-operator` gap that says so. Only the sum and the product are read; limits stay gaps. The `product` operator is read in the same shape as the sum (seen in files for the sum only). |
| An evaluation that reads a range variable (`i =`, `h[i =`, `2 i =`) | The values for every step of the range are collected in a new vector by a counted loop (`#hide`, `tbl_n = vector(...)`, `#for`, `#loop`, `#end hide`) and the vector is shown (with its `\|unit` if one was asked for). One range variable only; the range runs from its first to its last value in either direction. Note `range-loop`. |
| `v[k] := x` on a vector that has no value yet, `k` a literal | The vector is created without being shown, `vector(k)` (zeros below `k`), and the element is set and shown. A later fill loop over a range that starts at the array origin replaces a vector that was made this way from its first element only (anything else is the `element-assignment` gap, since a fill does not keep older elements). |
| `v[e] := x` on a vector that has a value, `e` not a range variable | The vector is grown with `resize(v; max(len(v); idx))` (zeros) when `e` is past its end, then the element is set and shown. |
| `v[e] := x` or `M[e1, e2] := x` where an index is an expression of range variables (`Vv[2 (i - 1) + j] := M[i, j]`), for a new array or for one that already holds an array | The range variables the index and the value read (up to three) become loops, in the order they are first read, and each pass writes the element at the computed index, extending the array with zeros (`resize`, `mresize`) when the index is past its end. A new array starts as one zero; an array that has a value is first copied (`Y = Y*1`) so a name that shares it keeps its values, and then written into. Note `range-loop`. The value and the index must not read the array when it is new; an array that has a value may be read by its own update (`v[i] := v[i - 1] + v[i]`), in the order of the loop, as the source reads it. |
| `M[r, 3] := expr` over a range variable `r` (a range and a literal index), or `M[3, 3] := x` past the end of a matrix | The same loops, with the literal as the other index; a write past the end of an array that has a value extends it with zeros (`mresize` for a matrix). |
| `v[k] := f(k)` where `f` gives a vector | An array whose elements are vectors (a nested array) is kept as a matrix with one row per element, grown to the longest vector (`nested_fill_loops`), and `v[k][j]` is the matrix element at row `k`, column `j`. The engine has no nested arrays; this is exact when every element is a vector, which is how the worksheets use it. An element read as a whole (`v[k]` alone) is an `element-assignment` gap. |
| `matcol(M, j) := v`, `matrow(M, i) := v` | The array is copied (a name that shares it keeps its values), grown to hold the column and the column written element by element; a name that has no value starts as one zero. A text label row written into a table (`FIB[0] := ["Case", "Ht"]`) is held as codes, as in a definition. |
| `[A, B] := program` (the program ends in a matrix of its locals) | One definition per name: the program is run once for each name, with that local as its value (same numbers, run twice). |
| A complex number (`a + b i`, `Re`, `Im`, `abs`, `conj`) | A pair of real numbers: `z__re` and `z__im`. A definition with a complex value is two definitions; `(a + bi)(c + di)`, quotients, powers 2 to 4, `Re(z)`, `Im(z)`, `abs(z)` and `conj(z)` are written out for the parts; an array of complex numbers is two arrays of the same shape; a program that computes them is run once for each part. A complex value where a real number is needed (a bare evaluation of it, or a function of it that is not written out) is an `unknown-element` gap that says so. The engine's own complex mode excludes vectors and matrices, which is why a pair of real arrays is used. A plot of an array with several columns (one trace per column in the source) is a `plot` gap. |
| `vectorSum(v)`, `last(v)`, `length(v)` | `sum(v)`; `len(v) + origin - 1`; `len(v)`. |
| `vectorize(e)` | `e` itself when `+`, `-`, a scalar product or a division by a scalar are all it contains (the engine does those element by element). An element-by-element product or quotient of two arrays, or a power or call over an array, has no operator: `vectorize-operator` gap. |
| `SIUnitsOf(x)` | `getunits(x)`, note `si-unit-scale`. The engine has no function for the SI unit of a quantity, only for the unit it is held in, so `x / SIUnitsOf(x)` is the number in the unit `x` has, which is the number in SI units only when that unit is an SI one or the scale cancels in the result. |
| Rows of the worksheet that hold more than one region | See Layout. |
| Program that is local definitions (each name assigned once) and a final value | The final value with the locals substituted, so `f(x) := program` is an ordinary function. Refused (gap) if a substituted value would be captured by an integration variable. |
| A function body that fills an array by element without having made it (`for i in 1 .. n: v[i] <- ...`) | The array is made first, as zeros of the size its writes reach: a loop variable index takes the larger end of the loop (shifted from the array origin), a literal index its own value, `v[i + 1]` one more, and a two-index write takes each dimension the same way (`matrix(max(1; n); 2)`). When the writes are in one statement, or all at numbers, and the ends of the loops do not read a name the loop changes, and a write under a condition does not reach past the writes that always run, the array is made at that size; otherwise it starts with one zero and grows with each write (below). |
| What a program returns | The value of the last statement it runs: an expression, or the value an assignment gives. So `if c then X` followed by `if d then Y` is `Y` when both hold and `X` when only the first does, and an `if` that sets a local in each of its arms returns that local. A path that runs no statement with a value returns nothing: where another path has one the result is the engine's undefined (`sqrt(-1)`, shown as undefined), never a zero, and where no path has one the program is a gap. A program inside a program (a block in a branch) is read as if its statements stood where it does, since locals are not scoped to it. When the value is not a final expression the statements keep it in a local (`val_of_program`) that every value statement assigns, and the program ends by reading it. A program that ends in a loop, or in an element assignment, or in an `if` with no `else` after an assignment, has a last statement whose value is not read and stays a gap. |
| Package files: `alsoif` | Read as `elseif` (the first true test wins). |
| `matcol(M, j)`, `matrow(M, i)` | `col(M; j)`, `row(M; i)`, the index shifted from the array origin. |
| `csort(A, j)`, `rsort(A, i)`, `linterp(vx, vy, x)` | `sort_rows(A; j)` (the rows ordered by column j), `sort_cols(A; i)` (the columns ordered by row i), and the engine's `linterp(x; vx; vy)` (linear interpolation that continues along the end segments outside the table; `interp` is an error there). |
| `nthRoot(n, x)`, `p%` | `root(x; n)` (the degree first; a real root of a negative number for an odd degree) and `p` of the `%` unit. |
| The element of anything that is not a name (`lsolve(A, b)[k]`, `(-v)[k]`) | `take(k; array)`, which needs no temporary and works in a function body. |
| `try expr on error fallback` | When `expr` is built from names, numbers, sums, products, differences, negation, absolute value, indexes of names and non-negative integer powers, the only way it fails is a division by zero, and the engine gives infinity there instead of an error: it becomes `if(d*0 ≠ d; expr; fallback)` over every divisor `d` (a divisor with a unit keeps it, so the zero test is between like units). A body with a call of a worksheet function whose value is one expression is looked through (its definition is substituted, so a division inside is tested too, and only on the branch of an `if` that runs); `max`, `min`, `floor`, `ceil`, `round`, `trunc`, `sign`, `abs`, `sin`, `cos` and `atan` cannot fail. Any other call or operation that could fail differently is a `program-block` gap that says so. |
| Angles: `30 deg`, `deg`, `°`, `rad`, and `x =` with the display unit `deg` | An angle is a plain number in the source: `deg` and `°` are `(π/180)` and `rad` is `1`, so they add to, compare with and multiply unitless numbers (the engine's angle units refuse to). A result shown in `deg` is `((x)/(π/180))*deg`: the number in degrees with the engine's degree unit. A definition that carries the display unit (`a := .01 = deg`) is shown as the definition only, as for other definitions, so its value stays in radians. |
| A comparison with a bare `0` (`c = 0`, `x < 0`) | The zero takes the other side's unit, `c ≡ 0*getunits(c)`, as in the source where a bare 0 matches any dimension; the engine refuses to compare a quantity with a unitless 0. |
| Program with loops, conditions or reassigned locals, as the value of a variable `name := program` | `#hide`, the statements, `#end hide`, then `name = value`. `#hide` keeps the loop from printing every intermediate value as plain text (`#val` did). The engine still evaluates the block and records any error raised inside it, so the editor lists it with its line, but the error text is not printed inline in the preview or the printed report; a test checks that the error is recorded. A local array that starts from a literal 0 and is then added to (`K[i,j] <- 0`, later `K[i,j] <- K[i,j] + x`) is started from `0*getunits(x)`, where `x` is the first value added with the loop variables at their first values, so the sums have consistent units; when that value reads something the program itself sets, its units are not known before the loops run and the program stays a `program-block` gap that says so. `for` over a range becomes `#for` (literal ascending bounds) or a counted loop that derives the variable from a start and an end taken once, so it counts down when the end is below the start as the source does; `while` becomes `#while`; `if`/`elseif`/`else` become `#if`/`#else if`/`#else`/`#end if`; `break` and `continue` are the same in both. Every local gets a name of its own (`x_p1`), so nothing leaks into the sheet: `x <- 5` inside a program never changes the sheet's `x`, and the right side of a first assignment still reads the outer `x`. Arrays are written element by element when the program created them whole first (`v <- [..]`), or when every index is the variable of an enclosing `for` loop, the write is outside any condition and loop bounds are known ahead (they mention no local of the program): the array is then allocated just before the outermost of those loops, as large as the largest index each loop reaches. An array whose size is not known ahead (indexed by a counter, written under a condition, a loop bound that is a local) is started with one zero at the top of the program and extended before each write past its end (`resize`, `mresize`), so it ends as large as the largest index written, as in the source; the program's array is its own even when the worksheet has a name of the same name (a stored 2 x 19 result of a real program that writes rows 1 and 2 of such a name shows it does not start from the worksheet's array). An array that is just another array's name is copied (`*1`), because the engine's `B = A` shares the array. A `return` before the end sets a result variable and a done flag; the rest of the block is guarded by the flag and loops break out. |
| A function defined again (`kb(x)` twice, with different bodies) | Each definition after the first gets an engine name of its own (`kb_v12`) and the calls after it use that one, since the engine warns about a name defined twice and the source means the latest definition from there on. |
| A program that is not one expression in a function body `f(x) := program` | First the program is run symbolically: locals are substituted, an `if` becomes `if(c; a; b)` over the rest of the program, a `return` ends its path, and a `for` over a literal range of at most 64 values is unrolled. What that cannot do (a loop with run-time bounds, a `while`, an element assignment) becomes one expression block `$Block{...; value}` (engine deviation D12: a loop nested in a block updates the block's variables). A `for` is a `$Repeat` that counts steps and derives the variable, so it runs down when the end is below the start as the source does; a `while` is a `$While`; an `if` sets 0/1 guard variables (the condition evaluated once, branches chained) and every assignment under it is `x = if(g; value; x)`; an argument the function assigns to is copied first (`M_p1 = M*1`) so the caller's array is not changed. Locals have names of their own. A `return` before the end sets a result and a done flag (zero until set: a not-a-number cannot be held in an engine loop), `break` and `continue` set the flag of the loop they leave or the pass they end, and after a statement that can set a flag the rest of its list runs only while no flag is set (a guard variable, as the arms of an `if` have); a `while` tests the flags before its condition and a `for` runs its remaining passes with every statement guarded off. A local that no statement sets on some path (first set under a condition or inside a loop, then read after it) has a companion flag that says whether it was set and starts at zero; a read of it is `if(flag; x; undefined)`, so where nothing set it the result is the engine's undefined, as the source has no value there. An array the function grows by element (written in several statements, in a `while`, at a computed index, or past the end of an argument) starts with one zero where it is first written (an argument is copied) and each write first extends it to hold the index (`resize`, `mresize`), so it ends as large as the largest index written; a write under an `if` extends it only when it runs (the index is 1 otherwise). A whole column or row (`matcol(M, j) <- v`) is written element by element after the array has grown to hold it. A function whose body cannot be that is a `program-block` gap with the reason: a local that is an array first set under a condition; an array written only under a condition (the source has none when the condition never holds); a loop over something other than a plain range. A worksheet function named like an engine function (`Norm`, `Max`: the engine reads function names without regard to case) is written `Norm_f`. |
| `lsolve(A, b)`, `augment` and `stack` with any number of arguments, `submatrix(A, ir, jr, ic, jc)` | The engine functions of the same name; the four `submatrix` indexes are shifted from the array origin. |
| First derivative `d/dx f` | `$Slope{f @ x = x}`, taken at the current value of `x`. If `x` has no value in the sheet the source could not calculate it either: `calculus-operator` gap with that reason. Higher derivatives are not read. |
| Trigonometric functions | The output starts with `#rad` (the source default is radians); note `angle-mode`. |
| Numbers with an exponent (`1.5e-3`) | Plain digits (`0.0015`), by moving the decimal point; `(m*10^(e))` beyond 20 places. Calcpad does not read `e` notation. |
| A variable applied like a function (`k(2)` where `k` is a variable) | `k*2`, note `call-as-multiplication` |

### Reading order

The source evaluates regions top to bottom, then left to right. Regions are sorted by the row they
sit on, then by their left edge; position is used only for this and then dropped.

* The row key is the region's **baseline** (`align-y`) when the file gives one, otherwise its top.
  A tall region next to a short one shares the short one's baseline while its top edge is higher, so
  sorting by top alone puts a use before the definition to its left. Real files do this.
* Regions whose keys are within 4 points of the first region of a row are one row.
* **`.mcdx` regions with no size** (the ones inside a collapsed area, about half of the regions of a large design
  program) keep the position of the area's header, which is not where they are calculated: the file lists regions in
  calculation order. A region with no size takes the largest top seen so far among the regions before it in the
  file, so the order it is listed in is the order it is calculated in; a region that has a size and is listed after
  a region lower on the page follows that region. On design programs of this kind a name read before the region that
  defines it is rare in file order and common in page order.

### Layout

The rows found for the reading order are kept. Regions that share a row, do not overlap across
and wrote something are put side by side; the order they are translated in, and so evaluated in,
is unchanged (left to right within a row).

* A run of consecutive rows that each hold 2 to 4 text regions (a legend of symbol and description)
  is one `<table class="imported-grid">`, one `<tr>` per row, one `<td>` per region.
* Any other row with two or more regions is a wrapping flex row, `<div class="imported-row">`, one
  `<div>` per region. A region that only wrote a comment (a range line) takes no part.
* Regions of a row sit where the file puts them: each block after the first is moved right by the gap the file leaves
  between the regions (when it gives widths), and the blocks of a row share their last baseline, so a value written
  beside a label that wraps lines up with the label's last line. A value to the right of a text region whose baseline
  falls inside the text's height is on the text's row even when the baselines differ.
* An equation on its own is indented by its distance from the page margin (the smallest `left` of the sheet), in points.
* A region that starts inside the previous one (math nested in a text region) is stacked, not placed beside it.

The Calcpad lines of a region are inside the cell, so equations are typeset by the engine in the cell.

### Array origin

The `array-origin` setting is read from the file. The default in the source is 0 and Calcpad counts
from 1, so every index is shifted (`v[i]` becomes `v.(i + 1)`). A literal index is folded.

### Functions

The mapping table lives in one place, `src/tables.rs` (`FUNCTIONS`). Calls with the wrong number of
arguments are `arity-mismatch` gaps. Caveats are notes: `log(x, b)` becomes `(ln(x)/ln(b))`,
`round(x, n)` becomes `(round((x)*10^(n))/10^(n))`.

| Source | Calcpad |
| --- | --- |
| sin cos tan sec csc cot asin acos atan atan2 acot asec acsc | same name (angle mode: see above) |
| sinh cosh tanh sech csch coth asinh acosh atanh | same name |
| ln, exp, sqrt, abs, sign, signum, floor, trunc, mod, min, max, gcd, lcm, if, identity, matrix, det, rank, augment, stack, sort, reverse, mean | same name (`signum` becomes `sign`) |
| log(x) | `log(x)` (base 10) |
| ceil | `ceiling` |
| Ceil(x, y), Floor(x, y) | `ceiling((x)/(y))*(y)`, `floor((x)/(y))*(y)`: the smallest multiple of `y` not below `x` (the largest not above it), `y` a unit as a rule (`Ceil(L, in)` rounds a length up to whole inches); note `round-digits`. Read from how the worksheets use it (rounding a length to a unit); a stored result that is already a whole multiple does not tell the two readings apart |
| Re, Im, arg | `re`, `im`, `phase` |
| rnd | `random` |
| linterp(xs, ys, x) | `interp(x; xs; ys)`: piecewise linear in the table; a value outside the table is an engine error where the source extrapolates |
| rows, cols, length | `n_rows`, `n_cols`, `len` |
| norm2 | `norm_2` |

A call to a function that is not defined in the sheet is one of three codes, ranked separately:

* `builtin-missing`: a built-in of the source tool that the engine lacks (`stdev`, `lookup`, `erf`,
  `cumsum`, ...). The table is `BUILTIN_MISSING` in `src/tables.rs`; together with `FUNCTIONS` it is the
  list of built-in names the importer knows.
* `reference-function`: any other undefined name, in a sheet that has worksheet-reference regions, which may
  define it.
* `addin-function`: any other undefined name, in a sheet with no reference regions. The file has no
  structural marker for an add-in (the files only mention add-ins in prose), so this is a heuristic:
  not built in, not defined, nothing to define it.

Solve-block names (`Given`, `Find`, ...) are `solve-block`, except in a solve block of an `.mcdx` file that has the shape below.

### Solve blocks

A solve block of an `.mcdx` file (guess values, equations, and `x := Find(x)`) is read from its regions and converted when it has one or two unknowns, as many equations (`a = b`, no inequalities), a value for each unknown before the block (its guess) and nothing but `Find` (optionally times a unit). The equations are shown as typeset equations. The solver region becomes a definition of the unknown that runs a secant iteration from the guess (Newton's method with a difference quotient for two unknowns), written as an expression block of the engine and ending at a step of a trillionth of the value, so it finds the solution nearest the guess as the source's solver does. The result is kept only when the two sides of every equation agree to a hundred-millionth at it; otherwise it is the engine's undefined, never the last iterate. The source stops at its own tolerance (0.001 by default), so the last digits of its answer can differ from this exact root; on the local worksheets the answers agree to 13 digits. A solve block of any other shape (more unknowns, inequalities, another solver, no guess, an unknown it cannot spell) is a `solve-block` gap that says why. Older `.xmcd` files, where `Given` and `Find` are plain names, stay `solve-block` gaps.

### Plots

An x-y plot (the `xyPlot` of an `.mcdx` file) becomes one `$Plot{...}` command of the engine, from the expressions the file stores on its two axes.

* **Traces.** Trace `k` pairs the k-th x expression with the k-th y expression; when one axis has fewer expressions than there are traces, its last one is used for the rest. A single x value against a vector of y values is a vertical line across the data, which is how the source draws a marker at a support. The unit typed beside an axis expression is the unit of the axis: every expression of that axis is divided by it.
* **The plot variable.** The engine plots expressions of one variable. A range variable of the sheet used in the expressions (`x[i]`, `V(x[i])` over `i = 0 .. 300`) runs over its range, and an index that mentions it reads the array by linear interpolation (`line(min(i + 1; len(x)); x)`), so a fractional value of the variable falls between two elements. A single name that nothing defines (`f(x)` against `x`) runs over the domain the author typed, or over the range the axis was drawn with. Whole arrays against each other (`Re(S)` against `Im(S)`) run over the element number (`line(i_plot; (a))`); when the traces differ in length the longest sets the range and a shorter array stays on its last point (`line(min(i_plot; len(a)); (a))`), as does an array read past its end by the range variable. Two range variables are not guessed between.
* **Text.** A title and axis labels, when the file stores them, are written as centred text lines around the plot.
* **Not carried (`plot-style` note, with the list in the detail).** The engine draws its own colours, markers, line styles and axis ranges, so the file's are dropped; columns and waterfalls are drawn as lines and a logarithmic axis is drawn linear.
* **Gaps.** A plot whose expressions use a name that did not convert (or is undefined) is a gap with that name as the reason, like a math region. A plot of an array the importer knows has several rows and columns (the source draws one trace per column) is a gap that says so. A plot of another kind (surface, polar, contour, 3-D) is `plot` with the element name as the detail. A plot in the older `.xmcd` file is an opaque binary item (a serialised object tree, not XML); it is `plot` with that reason. An `.xmcd` plot written as XML is read the same way as an `.mcdx` one.

On the local worksheets this reads the two plots of a beam worksheet (three traces each; the engine draws both with no error). Two of the four plots of a truss worksheet draw the complex coordinates of its members as `Re` against `Im` of a matrix with two rows: the source draws one trace per column, the engine plots one vector against another, so they are `plot` gaps that say so (the arrays themselves convert, see complex numbers under What converts). Axis labels and titles were empty in every plot of the files available, so the reading of a stored label is tested on synthetic XML only.

### Units

A name is a unit when it is not defined as a variable or function earlier in reading order and it is in
the table. Units are written with `*` (`1200*psi`). A name that is neither defined nor in a table is
`undefined-variable`, so an undefined variable whose name spells a unit (`m`, `h`, `T`) is read as the
unit: this cannot be told apart without types.

Same spelling in both: m km dm cm mm μm nm pm in ft yd mi nmi th ly AU, g kg mg μg lb oz slug gr t kt,
s ms μs ns min h d w y Hz kHz MHz GHz rpm, A mA μA kA V mV kV Ω kΩ F μF H mH C S T Wb, K °C °F °R mol cd
lm lx, N kN MN GN daN kNm Nm lbf kgf kip dyn, Pa kPa MPa GPa hPa μPa bar mbar atm psi ksi psf ksf mmHg
inHg Torr, J kJ MJ GJ mJ cal kcal BTU erg eV Wh kWh MWh W kW MW GW mW hp, L mL μL ha, rad deg ° grad rev %,
mph knot. The micro sign (U+00B5) is read as Greek mu (U+03BC).

Renamed: sec to s, hr and hour to h, day to d, yr to y, gm to g, tonne to t, lbm and pound to lb, liter
and litre to L, ml to mL, gal to gal_US, qt to qt_US, pt to pt_US, mile to mi, torr to Torr, dyne to dyn,
ohm to Ω, tonf to tonf_US, ton to ton_US, mil to th, acre to ac, nmile to nmi, degC/degF/degR to
°C/°F/°R, foot to ft, inch to in, International_BTU to BTU, pounds_per_sq_inch to psi, and the spelled-out
SI names (meter, gram, kilogram, second, minute, newton, pascal, joule, watt, volt, ampere, kelvin,
hertz, coulomb, farad, henry, tesla, weber, siemens, radian, percent).

Known to the source but missing in Calcpad (`unmapped-unit`): kmol, mmol, lbmol, gmol, kgmol, angstrom,
gal_imp, dB, Bq, Gy, Sv.

A test checks that every Calcpad unit and function name in the tables exists in the engine.

The built-in gravitational acceleration `g` becomes `(9.80665*m/s^2)`, because Calcpad's `g` is the gram.

## Pictures and fallbacks

Options (`ImportOptions`, `import_xmcd_opts`): `pictures` (default on), `saved_results` (default off),
`asset_prefix` (default `imported`), reader `limits`, `svg_limits`, `max_total_asset_bytes`.

`ImportResult.assets` is a list of `Asset { path, mime, bytes }`. The Calcpad text refers to each by exactly
its `path` in `<img src="...">`, relative to the library root with `/` separators (for example
`imported/R1795.svg`). The caller stores each asset in the page's file library under that path: the web
app resolves a relative `<img src>` against the library (root `/`) and inlines it, the same way the bundled
samples show their images. Give every imported worksheet its own `asset_prefix`; the prefix is cleaned to
letters, digits, `_ - . /` and cannot climb out of the library. Region ids are made unique within a prefix.
The batch runner writes the assets under the output folder as `<worksheet name>/R<id>.svg|png`.

* **Picture regions.** A picture region carries a metafile and/or a PNG. The metafile (base64, gzip when the item
  says so, WMF or EMF) is converted to SVG with the `emf-core` crate and written as `R<id>.svg`; the
  text gets `'<img style="width:Npt;max-width:100%;" src="..." alt="R<id>">`. If the conversion fails, is
  refused by a guard, or the SVG is too large, the PNG (the picture's own `png` item, else the region's saved
  rendering) is used instead, with note `svg-fallback`. If neither exists the region keeps its placeholder
  (`image`). A shown picture counts as converted.
* **Fallback pictures.** For every region that does not convert (math, embedded object, plot, component,
  reference) and has a saved rendering (a PNG), the output shows that picture, followed on the same text line
  by a caption: `static picture of the original, not live (R1795 ole-object)`. The grey `[import-gap` line
  is not written for such a region; the report keeps the gap codes and adds the note `static-picture`. The
  region still counts as needing attention (it is not live); `report.regions_showing()` counts converted plus
  pictured regions and `KindStats.pictured` the pictured ones. With `pictures` off there are no assets and
  every unconverted region is a placeholder line, as before.
* **Saved results.** An evaluation stores its last calculated result in the file. A single number (with a
  unit) is read; strings, matrices and anything else are ignored. For an unconverted math region that has
  one, `report.pictures.saved_result_available` counts it. With `saved_results` on, and no picture shown for
  the region, the output adds `'<i>saved result from the original, not live (R12): 571.9 International_BTU/(pound)</i>`
  after the placeholder line. Units are written as the source spells them; they are not converted. Default off.

Guards (every byte here is untrusted): base64 text over 32 MiB is refused; a gzip item may not inflate past
16 MiB (`max_item_bytes`) and all items together are capped at 192 MiB; a metafile over 4 MiB is not converted; the
converter runs on its own thread with a 32 MiB stack and a 10 s limit (a panic is caught, a run past the limit is
abandoned and its thread left to finish, so a hostile file costs a stuck thread, not the caller); an SVG over
3 MiB, or one containing script, foreign content, entity declarations or event attributes, is refused and the PNG is
used; a worksheet may produce at most 128 MiB of assets. Tests cut a hand-made metafile short at every
length, corrupt it at every byte and feed 300 random buffers.

## What does not convert

Reported with a code, counted, and replaced by a placeholder. Codes are stable; the UI and the batch
summary key on them.

| Code | Meaning |
| --- | --- |
| `range-variable` | `i := 1..10` with bounds that do not convert or a step of zero, and a read of a range variable that is not a sum, a table or an element assignment (more than one range variable in one evaluation, a range variable in a function body). The engine has no range variable; the supported uses are listed under What converts. Names it defines block later regions (`depends-on-gap`, counted as downstream of this code). |
| `program-block` | Programs outside the supported subset, each with its reason: a function array local that has no value on some path, an array written only under a condition, a `try` whose body could fail other than by dividing by zero, a loop over something other than a plain range, a program that ends in a loop or an element assignment |
| `solve-block` | Given/Find and relatives that do not meet the conditions under Solve blocks (the detail says which) |
| `symbolic-evaluation` | Symbolic evaluation |
| `calculus-operator` | Derivative and integral operators that do not meet the conditions under What converts (the detail names which); other derivative forms |
| `summation-operator`, `vectorize-operator`, `matrix-operator` | A sum or product that does not meet the conditions under What converts, limits; vectorize over an element-by-element product or quotient of arrays; column/row, dot, cross and magnitude operators |
| `string-value` | A text the engine cannot hold as a code: used as a number, joined, sliced, given to a function (a file name), or a matrix of texts and numbers that is printed |
| `bare-expression` | A comparison on its own, or an expression with a gap of its own or a saved result, with no definition or evaluation operator |
| `constraint-equation` | An equation on its own (`f_B = c*L + d`): a solve block's equation or a check. Shown with its real names, not solved |
| `element-assignment`, `disabled-region` | `v[i] := x`; a region switched off in the source |
| `plot` | A plot that is not an x-y plot, is stored as a binary item, has an empty slot on an axis, or uses a name that did not convert (see Plots) |
| `ole-object`, `component`, `worksheet-reference`, `other-region` | Counted only, never converted; shown as a static picture when the file has one (see Pictures) |
| `plot-style` | Note: a converted plot whose colours, markers, line styles, axis ranges, columns or log axes the engine does not draw |
| `header-footer`, `rich-text-element`, `text-unreadable` | `.mcdx` only: a page header or footer; an element of a text region that would hide content (a table, a picture, a floating block; the element name is the detail); a text region whose prose could not be read from the package |
| `image` | A picture region with nothing usable (no metafile, no raster picture) |
| `unknown-element` | An element name the reader does not know (the name is the detail). `imag`, the imaginary unit, is read (see What converts: complex numbers); a complex value used where a real number is needed is reported with this code and the reason |
| `unmapped-unit`, `builtin-missing`, `addin-function`, `reference-function`, `undefined-variable` | See above |
| `depends-on-gap` | Uses a name that an earlier unconverted region defines; attributed to the root cause in `ranked_gaps()` |
| `invalid-name`, `bad-number`, `arity-mismatch`, `bad-structure`, `too-deep` | Structure the importer will not guess |

Notes (the region converted, with a caveat): `global-definition`, `angle-mode`, `array-origin`,
`origin-assigned`, `log-base`, `round-digits`, `abs-of-array`, `call-as-multiplication`, `unit-override`, `control-input`, `uncovered-path`, `si-unit-scale`, `range-loop`, `inert-expression`, `solve-constraint`, `file-output`.
Whole-file problems: `xml-error`, `not-a-worksheet`, `input-too-large`, `unsupported-container` (a zip that is damaged,
encrypted or uses a zip feature the reader refuses).

### Element names: seen in real files, or assumed

The reader was written against 31 real `.xmcd` files and then tested on synthetic snippets. These
element names were **seen in real files**: `id` (with `subscript`), `real`, `str`, `apply` (with
`fixity="postfix"`), `plus`, `minus`, `neg`, `mult`, `div`, `pow`, `equal`, `lessThan`, `absval`,
`parens`, `sequence`, `define`, `globalDefine`, `localDefine`, `function`, `boundVars`, `lambda`,
`matrix` (with `rows`, `cols`), `indexer`, `range`, `eval` (with `unitOverride` and a cached `result`),
`provenance`, `derivative`, `integral`, `degree`, `program`, `ifThen`, `for`, `otherwise`, `matcol`,
`vectorSum`. These are **assumed** from the schema family and only exercised by synthetic tests:
`sqrt`, `factorial`, `transpose`, `notEqual`, `greaterThan`, `lessOrEqual`, `greaterOrEqual`, `and`,
`or`, `xor`, `not`, `symEval`. A wrong guess fails safe (an `unknown-element` gap), never as a wrong
number. `nthRoot` with an empty degree slot is the square root; with a degree it takes the degree first, then the radicand (a worksheet that writes a cube root puts the 3 before the quotient it roots), and `percent` is the postfix `%`.

The cached `result` of an evaluation is ignored. Comparing the engine's value with it would be a
strong check and is the obvious next step for validating numbers.

## Batch runner

```text
cargo build --release -p calcpad-cli                       # the engine command line
cargo run --release -p civilkit-calcs-import --example batch -- \
    <dir with .xmcd files> <output dir outside the repo> target/release/Cli
```

For each file it imports, writes `<name>.cpd` to the output directory, runs the engine on it
(`Cli x.cpd x.html -b -s`), and counts the engine's `Error in "..."` messages in the HTML. It prints
and saves `summary.txt`: a per-file and total table (regions, text and math counts, converted share
of all regions and of math regions, regions needing attention, engine errors), the regions by kind and
by math shape, the ranked missing features with their most frequent details, the notes, and the engine
errors grouped by message. Top-level directories whose name starts with `build`, `doc` or `includes`
are skipped. It only reads the inputs. Input files are never copied into this repository.

### What the spike found (31 worksheets of a thermodynamic property handbook)

* 1315 content regions: 562 text (all converted), 578 math, 132 embedded objects, 28 pictures, 11 plots,
  2 components, 2 worksheet references. 48 page breaks are not counted.
* 79 of 578 math regions convert (13.7 %). Live-convertible regions overall: 669 of 1315 (50.9 %), 641
  (48.7 %) when the 28 pictures are not counted. With pictures on, every region shows something: the 669 live
  plus 646 static pictures of the original. Engine errors after import: 0 in all 31 files, because anything
  that would fail is a placeholder or a picture.
* Pictures: 13 of 13 metafiles became an SVG (1.4 MB in total), 15 pictures came from a PNG item, and 646
  unconverted regions got their saved rendering (1.3 MB of PNG). Saved scalar results exist on 26 unconverted
  math regions.
* The corpus is dominated by calls into an add-in (the sheets are wrappers around a native library), so
  the top gaps are `addin-function` (408 regions including 82 downstream), embedded objects (132),
  strings that feed those calls (107 including downstream), `bare-expression` (94, documentation
  that shows call syntax), `reference-function` (18) and, in this corpus, no `builtin-missing` at all. Range variables, programs and calculus together are under 30 regions here.
  A corpus of ordinary structural or mechanical sheets will rank differently; this one does not tell
  how common range variables are in the wild.

## Security

The XML is untrusted. A DTD is refused (no entity definitions, so no entity expansion and no external
entities); the input size (64 MiB), node count (4 million) and element nesting (256, checked by a
pre-scan before the parser runs) are capped; expression and region nesting is followed only 64 levels
deep (deeper becomes a `too-deep` gap); an n-ary operator with more than 100 operands is a `too-deep`
gap so that its left-leaning tree stays shallow. Tests cover malformed input, a billion-laughs DTD, an
external entity, 200 000 nested elements, 200 nested expressions, a 5000-term sum and odd attribute values.
The crate forbids `unsafe`. Dependencies: `roxmltree` (MIT OR Apache-2.0), `miniz_oxide` (MIT OR Zlib OR
Apache-2.0) and `emf-core` 0.1.1 (MIT, default features off, so no `tracing`), which brings `wmf-core` (MIT),
`snafu` and `snafu-derive` (MIT OR Apache-2.0), `strum` and `strum_macros` (MIT), `heck` and `embedded-io`
(MIT OR Apache-2.0), `base64` (MIT OR Apache-2.0), `codepage` (Apache-2.0 OR MIT), `encoding_rs`
((Apache-2.0 OR MIT) AND BSD-3-Clause), `multiversion_no_op`, `scopeguard`, `simdutf8`, `unicode-segmentation`
and `unicode-width` (all MIT or Apache-2.0 alternatives). Nothing non-permissive; `THIRD_PARTY_LICENSES.md`
is regenerated. The importer never executes anything from the file.

A `.mcdx` is a zip, which adds its own guards (`src/zip.rs`): the entry count (20 000) and the declared size of
each entry are capped before anything is allocated; inflate stops at the declared size, so an entry that lies
about it fails instead of growing; the total of decoded bytes is capped (192 MiB, the same budget as pictures);
encryption, zip64 and compression methods other than stored and deflate are refused; entries are looked up by
name inside the archive and never written to disk, and a relationship target is folded (`..` cannot climb out).
The CRC is not checked. A flow document is a second, nested zip and gets the same treatment. The XML parts obey
the same DTD, depth and node limits as an `.xmcd`.

## The .mcdx front door

`.mcdx` is the newer container of the same worksheet language. `import_bytes` looks at the first bytes: a zip
signature goes to `mcdx::read` (also callable as `import_mcdx`), anything else is read as the plain XML. Both
produce the same `ir::Sheet`, so the translator, the report and the placeholder rule are shared. The mcdx
reader is `src/mcdx.rs`; the zip walker `src/zip.rs`; the math reader of `xmcd.rs` is reused (namespaces are matched
by family, so `math30` and `math50` are both `math`).

What the reader does with the package:

* The worksheet part is found through the package relationships (any part called `worksheet.xml` as a fallback).
  Its siblings are read relative to it: `result.xml` (saved results, joined to a math region by `resultRef`),
  `settings/calculation.xml` (the array origin; 0 when absent) and the header and footer parts.
* **Text.** A text region names a XAML flow document through a relationship. That part is a nested zip holding
  `Xaml/Document.xaml`. Older files carry the flow document inline, as a child of the text element; both forms are read. Paragraphs, runs, bold, italic, underline, sub and superscript, line breaks, hyperlinks
  (the text stays, the address does not), bulleted lists and decimal lists (numbered "1. ", "2. ") convert.
  Colours, fonts and sizes are dropped, as for `.xmcd`. Math nested in a text region becomes its own region at the
  text's position, as before.
* **Math.** The tree is the same family as `.xmcd`. What differs: a value followed by its unit is a `scale`
  application (read as a product); an identifier's subscript is a styled span inside it (`T` + subscript `max`
  reads as `T_max`); a call with one argument has no `sequence` wrapper; an empty slot is a `placeholder`
  (an empty unit override means "no unit"; a placeholder anywhere else is an `unknown-element` gap). Matrices are
  stored column by column, as in `.xmcd` (read off a 3 x 2 matrix in the sample files, a column of names then a column of numbers).
* **Programs** (`if`, `for`, `while`, `return`, try/catch, local definitions) convert as in `.xmcd` where the
  rules above allow, and are `program-block` gaps otherwise; `alsoif` is `elseif`; `summation` is read like the
  `.xmcd` sum, with its limits as `lowerBound` and `upperBound` elements.
* **Collapsible areas** only group regions: their regions are read as if on the page, positioned relative to
  the area; regions with no size (inside a closed area) are placed by the file's order (see the reading order of
  regions with no size).
* **Pictures** with a PNG convert like `.xmcd` pictures (the PNG is stored in the library). The package carries no
  cached rendering of a region, so an unconverted region is a placeholder line, never a static picture.

Skipped, with a code and a visible placeholder line each (nothing is dropped silently):

| Part | Code |
| --- | --- |
| Embedded object of another application (`oleObject`) | `ole-object` |
| Spreadsheet and chart components, with the regions inside them | `component` |
| Reference to another worksheet | `worksheet-reference` (and `reference-function` for calls it may define) |
| Plot | `plot` (x-y plots convert, see Plots) |
| Page header and footer (when they hold regions) | `header-footer` |
| Table, picture or floating block inside a text region | `rich-text-element` (the element name is the detail) |
| A text region whose flow document is missing or unreadable | `text-unreadable` |
| Any other region kind | `other-region` |
| Calls into an external library | `addin-function` (as in `.xmcd`) |

Two wrong-number traps are closed for both formats: after an element assignment (`v[i] := x`) or a definition
whose target is a matrix of names, the names involved are marked as defined by an unconverted region, so a later
use is a `depends-on-gap` instead of the old value (or, for a name such as `L` that is also a unit, the unit).

Spreadsheet components: a component defines names from ranges of a workbook it saves in the package
(`Name := excel["Sheet1!C6:H16"]`). The workbook is read (`src/xls.rs`: a compound file, then the cell records of
the sheet) and, when every cell of the range is a number (a cell nothing was written to reads as 0), the name is
defined from them as a literal matrix, with a visible note and the report counts it under `frozenValues`: the
value is what the file held, it does not follow a change of the input, and the import dialog says so. A range with
text in it, a sheet that is not there or a file that is not a workbook leaves the component a gap. A definition
that only fails on such a name takes the value the file saved for it, when that is complete (a number or a matrix
of numbers with a unit of mapped names). The component itself stays a `component` gap, as does everything the
component writes back.

A name the file labels as a unit in one place and defines as a variable in another (a matrix `N` next to forces in
`N`) is renamed `N_v` everywhere it is the variable; the engine would otherwise read every `N` as the variable.
Indexing the result of a call goes through a temporary (`ix_n = f(x)` on the line before). A one-column `submatrix` is
`col(submatrix(...); 1)`: the source indexes a column like a vector, the engine's vector is a different thing.

Equality is written with the engine's sign (`≡`); `==` is a pair of assignments to it. A literal 0 in the source
takes the units of what it is added to or compared with; the engine reports inconsistent units, so an array
that starts from 0 and is accumulated into is not converted (the reason says so).

Not read: the unit system setting, strings-use-origin and the other calculation settings; the regions inside a
component (they belong to the skipped component); a definition whose target is a matrix of names
(`bad-structure`).

`cargo run --release -p civilkit-calcs-import --example engine_check -- file.mcdx out` imports a file, runs
the result through the engine and writes the text, the engine's HTML and a list of saved results, so converted
lines can be compared with the numbers the original stored.

Checked against seven worksheets of a thermodynamic property handbook, which are mostly text and calls into an
external library, so they exercise the reader and the skip reports far more than the maths. Their counts are kept
out of the repository with the files. The synthetic packages in `tests/mcdx.rs` cover assignment, arithmetic with
units, a function, a matrix, a program, a range variable, text, areas, skipped parts, saved results and zip abuse;
each math case is built once and imported through both front doors, and the two must agree.

## Not covered

Range variables (top engine gap to close, see the ranked list), programs, solve blocks, symbolic
evaluation, plots other than x-y plots, pictures, embedded objects, data tables and components, strings, derivative and integral
operators, worksheet references and includes, text colours and fonts, tables of results. Types
are not tracked, so `|x|` of a vector or matrix is only noted when the operand was defined from a matrix
literal.
