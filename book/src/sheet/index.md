# The sheet format

<!-- description: A sheet is a calculation written the way you would explain it: Markdown prose, plain name: value lines with real units, and LaTeX formulas. -->

A **sheet** is a calculation written the way you would explain it to a colleague: Markdown
prose, plain `name: value` lines, and LaTeX where you want a typeset formula. The sheet compiler
turns it into the Calcpad worksheet language and the engine runs that. There is no second engine:
units, vectors, matrices, plots, and the Word and Excel exports all work because the engine sees
an ordinary Calcpad worksheet.

A file is a sheet when its first non-blank line is `<!--sheet-->`, or when its name ends in
`.sheet`. The editor adds the marker line for you.

## A whole sheet

```sheet
# Beam bending check

A simply supported beam. The moment **comes out in kN·m**.

L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
Ms: 300 MPa * 1230000 mm³ -> kN*m
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

The result:

```text
Beam bending check
A simply supported beam. The moment comes out in kN·m.

L = 6 m    span
w = 12 kN/m    design load
M_star = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m    maximum bending moment
Ms = 300 MPa · 1230000 mm³ = 369 kN·m
utilisation = M_star / (0.9·Ms) = 54 kN·m / (0.9 · 369 kN·m) = 0.163
OK
```

## The line kinds

Every line of a sheet is one of these.

| Line | Looks like | Chapter |
| --- | --- | --- |
| Prose | `A simply supported beam.` or `## Heading` | [Prose, LaTeX and tables](prose.md) |
| Quantity | `L: 6 m \| span` | [Quantities and notes](quantities.md) |
| Input | `L: ? 6 m \| span` | [Templates with inputs](../templates.md) |
| Section marker | `## Workings {collapse}`, `M: ... {key}`, `{key results}` | [Folding sections and key results](sections.md) |
| Function | `f(x): x^2 + 1` | [Functions](functions.md) |
| Condition | `if u <= 1: "OK" else: "Fails"` | [Conditions and loops](flow.md) |
| Block | `for i in 1..5:` with an indented body | [Conditions and loops](flow.md) |
| Plot | `plot M(x) for x in 0..L` | [Plots](../plots.md) |
| Python | a fenced `python` block | [Python blocks](../python.md) |
| Diagram | a fenced `mermaid` block | [Diagrams](diagrams.md) |
| Calcpad | a line starting with `>>`, or a `calcpad` fence | [Mixing in Calcpad](passthrough.md) |

A line that looks like a quantity but is really prose (`Note: see the drawing`) is prose, because
its right-hand side is not an expression. To force prose, start the line with a backslash.

## A page to keep open

The [quick reference](https://civilkit.io/calcs/reference/) puts the sheet language on one page that prints:
every line kind and input, the functions, constants, operators and units of the engine. It has a Print button,
and it is also what to hand to a colleague who has not used a sheet before.

## What a sheet cannot do

* There is no symbolic algebra in the sheet itself. The engine is numeric. Use the
  [Symbolic dialog](../symbolic.md) to derive a formula and paste it in.
* A vector or matrix literal cannot carry a unit after the bracket (`[1, 2] kN`). Put the unit on
  the elements.
* A formula is typeset in the page, but in Word and Excel it appears as its TeX source in
  italics.
* **Convert to Calcpad** works one way. Calcpad text is not turned back into a sheet.

Everything else in the Calcpad language is reachable by passing lines through. See
[Mixing in Calcpad](passthrough.md).
