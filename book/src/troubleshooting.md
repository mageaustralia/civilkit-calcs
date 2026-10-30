# Troubleshooting and FAQ

## Reading an error

An error appears in the results where the line would, as a red paragraph, in the **Errors**
dialog, and as an underline in the editor. For a sheet, the line number is the *sheet* line.
Click the line number in the report to jump to the source.

| Message | Cause and fix |
| --- | --- |
| `Undefined variable or units: "x"` | The name is not defined above this line, or it is misspelled, or a unit is misspelled. Names are case sensitive. A sheet renames some names (`phi` is `φ`); a `>>` line may use either spelling. |
| `Inconsistent units: "m + s"` | You added or compared quantities of different dimensions. Fix the formula. |
| `Invalid units: "maximum"` (or `Invalid symbol: "#"` on a heading) | The text is being read as a Calcpad worksheet, not as a sheet, so `\|` means a unit conversion and `#` a directive. The `<!--sheet-->` marker is missing from the first line. Put it back. |
| `Cannot rewrite existing units: kip` | A custom unit (`.kip = ...`) has the name of a built-in unit. Choose another name. |
| `The matrix has complex eigenvalues.` | `eigenvals`, `eigenvecs` and `eigen` return real values only. Use `ceigenvals` and `ceigenvecs`. See [Vectors and matrices](vectors-matrices.md). |
| `Inconsistent units: "unitless" and "m"` in a plot | The plot variable carried a unit. Write `plot f(x * 1 m) for x in 0..6`. See [Plots](plots.md). |
| `#UI does not support expressions unless 'allowExpression' is true.` | The value in a `#UI` line is not a plain number. Use `allowExpression` or move the formula into a sheet line. In a sheet, an input line with `?` takes a number only: `2*3 m` is an error on its line. See [Input forms with #UI](calcpad/ui-inputs.md) and [Templates](templates.md). |
| `The condition could not be evaluated, so the block is unresolved...` | An `#if` or `if` condition has an error (usually an undefined name). The block runs no branch. |
| `Python was not run` | You answered "Not now" to the Python question. The refusal is remembered until the Python code changes; edit it to be asked again. |
| `Sheet syntax: ...` in red | The sheet compiler found a bad `plot` line, an unclosed `$$` or similar. The marker is on that line. |

## Things that go quietly wrong

* **Angles.** The default is degrees. `sin(π)` is not zero in degrees mode. Put `>> #rad` in the sheet
  when you work in radians and pass `π` or radian values. A plot of `sin(x)` over `0..2*π` in degrees mode
  is nearly flat.
* **`g` is a gram.** Only `π` and `e` are built in. Insert the gravity constant from the Insert tab
  (`g = 9.80665m/s^2`) before you use `g` as an acceleration.
* **A variable called `m`, `s` or `L`.** After a number `m` is still the metre. A variable called `m`
  is a variable when it stands alone. In Python `export` names, avoid names that are units.
* **Rounding.** The result shows two decimals by default (Settings, Calculation, Decimals), and a number below
  one keeps three significant figures (`0.163`, `0.0856`). The number inside is full precision: a
  utilisation of 1.004 is shown as `1`, and a check on it (`if util <= 1`) still sees 1.004.
* **Every line of a sheet shows an error such as `Invalid symbol: "#"`.** The first line of the
  file is not `<!--sheet-->`. Older versions wrote the comment of **Save values** above that line;
  the current one writes it under the marker. Delete the stray first line, or put the marker back
  as the first line. See [Templates](templates.md#saving-the-values).
* **A drop-down opens showing the wrong option.** In a raw `>> #UI` line the default matches none of
  the `values`, so the first label is shown while the calculation uses the default. Make the
  default one of the values. A `?` line cannot do this. See the known limits in
  [Input forms with #UI](calcpad/ui-inputs.md#known-limits).
* **My saved values are not used.** A saved value belongs to the input's name. If the input was
  renamed, the old value is ignored. Preview always shows the defaults; use Input or Report.

## Frequently asked questions

**Is anything sent to a server?** No calculation is. The page is static files and the engine runs in
your browser. The only network requests are loading the app, and loading Plotly for an interactive
chart.

**Does it work offline?** Yes, after one visit. See [Saving](getting-started/saving.md).

**Can I use my own Python packages?** No. A small numpy, matplotlib and Plotly stand-in is built in,
and what is missing raises a named error. See [Python blocks](python.md).

**How do I share a calculation?** Send the `.sheet` or `.cpd` file, or export Word, Excel or PDF.

**Can I move a sheet to the Calcpad language?** Yes, with Convert to Calcpad. The other direction
does not exist.

**Why did the interactive chart disappear from my PDF?** Print and export cannot run Plotly. Draw a
static copy with matplotlib as well.

**Is it free?** Yes. It is open source under the MIT licence, and the page runs from static
files. See [Credits and licences](appendix/credits.md).

**Where do I report a problem?** On the project's issue tracker at
https://github.com/mageaustralia/civilkit-calcs . Say what you typed, what you saw and what you
expected.
