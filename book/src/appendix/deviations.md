# Appendix: deviations from CalcpadCE

CivilKit Calcs reproduces CalcpadCE's output exactly unless a behaviour is listed in the project's
`DEVIATIONS.md`. Each entry there names the behaviour, why it differs and the test that pins it. This
page is a summary for users.

## Behaviour that differs

| | CalcpadCE | CivilKit Calcs |
| --- | --- | --- |
| A failing `#if` condition | A condition that throws (a misspelled name) is false, so the `#else` branch runs without a word. | The block is left unresolved: no branch runs, and an error paragraph says why. |
| Plot loops | A plot grid with a NaN step never ends. | Grid loops stop after 100,000 lines. Finite grids are unchanged. |
| Scripts in a worksheet | Any `<script>` in a worksheet runs from any address. | Scripts run only from a library list (Settings, Libraries), under a content security policy, with exact file addresses and integrity hashes. A script that is not on the list is replaced by a note naming its address. Scripts cannot send data anywhere. The print frame has no scripts. |
| C-style operators | The desktop app converts `==`, `!=`, `>=`, `<=`, `%%`, `&&`, `\|\|`, `<<`, `<*` when it reads a file. | The browser does the same when it opens a worksheet. |
| Input fields | Many examples use the old `name = ? {default}` form. | The bundled samples are rewritten with `#UI`, and the editor offers a "Convert to #UI" quick fix. |
| Select, radio and checkbox | A select or radio that names a text box does nothing in the web front end. | It fills the text boxes it names and recalculates; the choice is carried into the Report, PDF and Word. |
| Radio and drop-down defaults | A radio group or drop-down marks its default by comparing text, so a default with a divided unit such as `25kN/m` finds no option and opens with nothing chosen (a drop-down shows its first label). | The default is found by value: the division sign, middle dot, raised digits and `^` are compared as plain marks, and `1.0` equals `1`. |
| Nested blocks | A loop nested in an expression block cannot update the enclosing block's variable. | It can. |
| Units in arrays | A vector or matrix writes the unit on every element. | It writes the unit once after the bracket (`[16; 12] ft`). `#settings {"unitPerElement": true}` restores the old display. |
| Eigenvalues | `eigenvals`, `eigenvecs` and `eigen` take only a matrix of the symmetric type. | They take any square matrix; a symmetric literal no longer needs `copy(A, symmetric(n), 1, 1)`. `ceigenvals` and `ceigenvecs` return complex eigenvalues and eigenvectors. |
| Markdown tables in Word | A table without the `bordered` class has no rules. | A Markdown pipe table is ruled. |
| Word export | SVG pictures are embedded alone. Numbered lists lose their numbers. | An SVG is also written as a PNG for readers that lack SVG. Ordered lists are numbered. Unknown elements keep their content. |
| Print header and footer | The server prints title, date and page numbers. | The browser prints; its own header and footer apply. |
| `#write` | Overwrites on disk. | Keeps the file in the library and downloads it when its content changes. |

## Additions

* The **sheet format** (Markdown prose, `name: value` lines, LaTeX): a front end to the Calcpad
  language that is not part of CalcpadCE.
* The **Excel export**, with native charts.
* **Python blocks**, with matplotlib-style and Plotly-style plotting.
* The **Symbolic** dialog.
* The **worksheet importer**.
* **Rich-text editing** of Markdown prose in the results.
* **Sheet inputs**: `L: ? 6 m` and its drop-down, radio and checkbox forms, which compile to `#UI`, and sliders with bounds (`L: ? 6 m [2 .. 12 step 0.5]`), for which `#UI` takes `min`, `max` and `step`.
* **SI prefixes on every SI unit** (`3 mN`, `5 µm`, `40 nF`): CalcpadCE lists the prefixed units one by one and lacks many.
* **Folding sections and key results** in the sheet format (`## Workings {collapse}`, `{key}`, `{key results}`).
* **Model bindings**: a comment that ties an input to a quantity of a structural model, with a marker
  in the report. No model is connected yet.
* A **ribbon** interface, a phone layout and an installable, offline app.

## Not supported

Raster (PNG) plots (the web version draws SVG), the `#ProjectPath` and `#LibraryPath` roots (they are
accepted and ignored), compiled `.cpdz` worksheets, and step-by-step execution (`#pause`, `#input`).

## Numerical differences

The platform's maths library supplies `sin`, `exp`, `pow` and friends, where CalcpadCE uses the .NET
runtime's, so the last printed digit of a result can differ. `#write` line endings are `\n`.

The full text, with the tests, is in
[DEVIATIONS.md](https://github.com/mageaustralia/civilkit-calcs/blob/main/DEVIATIONS.md).
