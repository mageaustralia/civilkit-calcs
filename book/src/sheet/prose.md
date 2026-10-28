# Prose, LaTeX and tables

Anything that is not one of the line kinds is Markdown prose. Use it to say what is being
checked, to state assumptions and to explain results.

## Markdown

Supported Markdown: headings (`#` to `######`), paragraphs, `**bold**`, `*italic*`, bulleted
(`-`) and numbered lists, pipe tables, links, `![alt](file)` images from the library, `` `code` ``
and fenced code blocks. Blank lines between sections are kept.

A plain paragraph, even over several lines, becomes one text line. Only prose that really
contains Markdown (lists, tables, quotes, emphasis, links, code) is passed to the engine's
Markdown renderer.

## LaTeX

`$...$` is an inline formula and `$$...$$` a display formula, in prose, in notes and in the text
of an `if` line. A `$$` alone on a line opens and closes a multi-line formula. `\$` is a dollar
sign, and a `$` before a digit ("$5") is one too.

```sheet
A pipe of diameter $d$ carries a flow $Q$. The mean velocity is

$$v = \frac{4 Q}{\pi d^2}$$

d: 0.2 m
Q: 0.05 m^3/s
v: 4 * Q / (π * d^2) -> m/s
```

```text
A pipe of diameter d carries a flow Q. The mean velocity is

        4Q
   v = ----
        πd²

d = 0.2 m
Q = 0.05 m³/s
v = 4·Q / (π·d²) = 4 · 0.05 m³/s / (3.14 · (0.2 m)²) = 1.59 m/s
```

The page typesets formulas with KaTeX, which is bundled with the app and loaded the first time a
sheet has a formula, so formulas work offline. A formula that cannot be typeset is shown as
its TeX source. In the LaTeX of a sheet, variable names take the same blue as in an equation. In
Word and Excel a formula appears as its TeX source in italics, not as Office Math.

## Tables

A pipe table is prose, with the engine's table styling:

```sheet
| Action     | Symbol | Value |
|------------|--------|-------|
| Permanent  | G      | 4.5 kPa |
| Imposed    | Q      | 3 kPa |

G: 4.5 kPa
Q: 3 kPa
w_uls: 1.2 * G + 1.5 * Q -> kPa | strength combination
```

```text
Action      Symbol   Value
Permanent   G        4.5 kPa
Imposed     Q        3 kPa

G = 4.5 kPa
Q = 3 kPa
w_uls = 1.2·G + 1.5·Q = 1.2 · 4.5 kPa + 1.5 · 3 kPa = 9.9 kPa    strength combination
```

The table is text. It does not feed the calculation, so type the numbers again in quantity lines,
or define the quantities first and mention them in the table text.

In the preview, click a column heading to sort the rows by it (again for descending, a third time
for the source order). It is a view only: the sheet, the printout and the exports keep the order
you wrote. See [Sorting a table](../editing.md#sorting-a-table).

## Forcing prose

A line starts with a backslash when it must not be read as a quantity:

```sheet
\L: 5 is shown as text
Note: see the drawing
```

`Note: see the drawing` is already prose, because "see the drawing" is not an expression.
