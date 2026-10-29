# Number format (localisation)

Users whose language writes 1,5 instead of 1.5, or groups digits as 1.000,5 or 1 000,5, choose that in
Settings > Number format. The code is `web/calcpad-web/src/wasm/number-format.ts`; the check is
`web/e2e/locale-check.mjs`.

## Design

1. **The source stays canonical.** A worksheet or sheet always uses a decimal point and the argument separators
   it has today, so files stay portable and the engine and its tests are unchanged. Saved input values
   (`uiOverrides`) are canonical too.
2. **The web layer translates at the edge.** The in-page transport (`transport.ts`) rewrites the numbers of the
   rendered HTML before it is shown, printed, saved or handed to the Word and Excel writers. Nothing in the engine
   knows about the format.
3. **Setting.** Automatic (from `navigator.language`), Point (1,234.5), Comma (1.234,5) or Space and comma
   (1 234,5), plus a digit grouping toggle (off by default, so nothing changes for existing users). It is kept in
   `localStorage` under `calcpad-number-format`. The link `?numfmt=auto|point|comma|space` (and `&numgroup=1|0`)
   applies a format to that page only and stores nothing.
4. **Display.** Only text inside an equation span (class `eq`) changes, and a bare number the value-only mode writes
   into the page. Names (`var`), subscripts, exponents, units, error text, scripts, styles, SVG plots, code
   listings and attributes are copied byte for byte, so only digits and separators can differ. Authored prose,
   headings and Markdown tables keep the numbers as typed (they are the author's text, and the table sort reads them).
5. **Input boxes.** In a comma format a number box shows 6,5 and accepts both 6,5 and 6.5. A shim runs ahead of the
   form's own script and wraps each box's `value`: the form reads and posts canonical text (6.5), the box shows the
   format. Boxes show the decimal mark only, no digit groups, so a point typed in a box is always a decimal point.
   The range message ("must be from 0,5 to 2,5") follows the format. Expression boxes (`allowExpression`) are not changed.
6. **Exports.** HTML and PDF (print) use the same rendered page. Word follows the format in its text and in the
   equation runs outside subscripts and exponents. Excel's Report sheet text follows it; the Variables, Vectors and
   Inputs sheets keep real numeric cells.

## Not covered

- **Pasting into the editor.** The source is canonical and `,` is an argument separator, so a pasted 1,5 cannot be
  told apart from two arguments. Left out on purpose.
- **Plots and SVG axis labels, 3D views, Python output, spreadsheet-style grids (`datagrid`)** keep the decimal point.
- **Prose tables in the preview** keep their typed numbers.
