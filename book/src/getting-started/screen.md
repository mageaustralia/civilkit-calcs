# The screen

<!-- description: A tour of the CivilKit Calcs window: the ribbon, the editor, the results, the sidebar, and how the layout changes on a phone. -->

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
| Math | Regions (math line, text, heading, image, plot, table), operators, Greek letters and symbols, programming (conditions, loops, macros), constants, numerical methods, units, clipboard. |
| Input/Output | Input fields: entry, slider, dropdown, radio, checkbox (the `?` form in a sheet, `#UI` in a worksheet), data grid; reading and writing files; breakpoints; the **Input form** view. |
| Functions | Trigonometric, hyperbolic, logarithmic, rounding, statistical, logical, complex and other functions; define a function; the **Symbolic** dialog. |
| Matrices/Tables | Vector and matrix literals, vector and matrix functions, determinant, inverse and transpose, tables. |
| Calculation | Calculate, auto-calculate, the four result views, settings, and the Variables and Errors lists. |
| Plots & Diagrams | Function, parametric, multiple and colour-map plots; plot settings; the **Figures** group: the **Diagrams** menu of starter diagrams (see [Diagrams](../sheet/diagrams.md)) and **Image**. |
| Math Formatting | Rounding, number format, angle units, output control (`#hide`, `#val`, `#nosub` and so on). |
| Text Formatting | Headings, bold, italic, colours, links, lists, rules, page break. In a sheet these write Markdown. |
| Document | Import worksheet, the tidy tools, header and footer, page break, the report, contents, metadata, and Save as, Print PDF, Word, Excel and Export. |
| Resources | Samples, library, New sheet, New Calcpad, Show compiled, Convert to Calcpad, themes, zoom, the sidebar and Settings, help. |

The tabs follow the work: write (Math to Matrices/Tables), calculate, plot, format, then the
document. Hover over a button for what it inserts or does; where it teaches syntax the tooltip shows
the typed form.

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
snippet (a click inserts it at the cursor), and tabs for the table of **Contents** and the
**Variables** of the worksheet. **Settings**, **Export**, **Errors** and **Prettify** open as
dialogs over the page. Hide the sidebar with the button at its top right, or Resources > Sidebar
(Ctrl+Shift+B in the desktop app).

## Settings

The gear at the right end of the top strip opens **Settings** (so do Calculation > Calculation settings and
Resources > Settings). It has five tabs:

| Tab | What is on it |
| --- | --- |
| General | Language, number format, and the appearance (colour theme, preview theme). |
| Calculation | Decimals, angle units, complex numbers and the other math settings, plot defaults, US or UK units. |
| Editor | Quick typing, comment format, formatting keys, auto-run, cursor sync, the linter. |
| Export | The PDF page: paper size, orientation, margins, header and footer. |
| Advanced | Script libraries, logging and limits, the server address, saved configurations. |

The search box above the tabs looks through all of them and lists each match under its tab's name; clear it to
get the tabs back. Settings opens on the tab you last used, until the browser tab is closed. Calculation settings
opens it on Calculation, and Document > Header and footer on Export. On the tab strip, the arrow keys, Home and End
move between the tabs.

## On a phone

At 760 pixels wide or less, one pane shows at a time, switched from a bar along the bottom:
**Edit**, **Results** and **Insert**. The ribbon starts folded; tap an open tab to fold it. See
[Editing](../editing.md) for touch editing.
