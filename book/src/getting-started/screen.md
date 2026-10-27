# The screen

The window has five parts.

```text
+-------------------------------------------------------------+
| (suite bar, wide browser windows only)                      |
| File  New Open Save Undo Redo                               |
| Math | Input/Output | Functions | ... | Resources           |   ribbon
|  [groups of buttons for the selected tab]                   |
+----------------------------+-----------------------+--------+
| tab: Beam check            |  Results              |Sidebar |
|                            |  Preview Unwrapped    | Insert |
|  editor                    |  Input Report         |  ...   |
|                            |                       |        |
+----------------------------+-----------------------+--------+
```

## The ribbon

A quick-access strip at the top holds **New**, **Open**, **Save**, **Undo** and **Redo**. Under it
are ten tabs. In a sheet the buttons insert the sheet form of each item, and a button that has no
sheet form inserts Calcpad text as `>>` lines and carries a small `>>` mark.

| Tab | What is on it |
| --- | --- |
| Math | Operators, Greek letters and symbols, programming (conditions, loops, macros), constants, numerical methods, units. |
| Input/Output | Input fields: entry, dropdown, radio, checkbox (the `?` form in a sheet, `#UI` in a worksheet), data grid; reading and writing files; the Input mode button. |
| Functions | Trigonometric, hyperbolic, logarithmic, rounding, statistical, logical, complex and other functions; define a function; the **Symbolic** dialog. |
| Matrices/Tables | Vector and matrix literals, vector and matrix functions, linear algebra, tables, data grid. |
| Plots | Function, parametric, multiple and colour-map plots; plot settings. |
| Math Formatting | Rounding, number format, angle units, output control (`#hide`, `#val`, `#nosub` and so on). |
| Text Formatting | Headings, bold, italic, colours, links, lists, rules, page break. In a sheet these write Markdown. |
| Calculation | Calculate, auto-calculate, the four result views, settings, variables, errors. |
| Document | Print PDF, Word, Excel, Export, Import worksheet, Save As, page break, contents, metadata. |
| Resources | Samples, library, New sheet, New Calcpad, Show compiled, Convert to Calcpad, help, themes, zoom. |

## The editor

The editor is on the left, one tab per open file. It colours the text, completes names, Greek
spellings and function names, and underlines mistakes. Hover over a name to see where it is
defined; F12 jumps to the definition. Format Document (Shift+Alt+F) tidies a sheet. See
[Editing](../editing.md).

## The results

The right-hand pane shows the calculated worksheet. The Calculation tab chooses between four
views:

* **Preview**: the document as written, with every `#pre` and `#post` part. This is the usual
  view while you work.
* **Unwrapped**: the source listing with macros and includes resolved. Useful for finding errors
  in macros.
* **Input**: the fill-in form of a sheet or worksheet that has inputs (`L: ? 6 m` in a sheet, `#UI`
  in a worksheet). See [Templates with inputs](../templates.md).
* **Report**: the print layout, as it will come out in a PDF or Word file.

Results update as you type; **Auto-calculate** can be turned off on the Calculation tab, and
then **Calculate** (Ctrl+Alt+X, and F5 in the desktop app) runs the sheet on demand.

Under a plot there is a row of small buttons (log axes, 1:1, copy data, save as PNG or SVG).
These are in the preview only and are not part of a printout or export.

## The sidebar

The sidebar holds the **Insert** tab, a searchable library of every symbol, function, unit and
snippet (a click inserts it at the cursor), and tabs for the table of contents (TOC) and the
variables of the worksheet. **Settings**, **Export**, **Errors** and **Prettify** open as
dialogs over the page. Hide the sidebar with Ctrl+Shift+B.

## On a phone

At 760 pixels wide or less, one pane shows at a time, switched from a bar along the bottom:
**Edit**, **Results** and **Insert**. The ribbon starts folded; tap an open tab to fold it. See
[Editing](../editing.md) for touch editing.
