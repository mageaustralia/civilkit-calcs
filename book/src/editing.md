# Editing

There are two places to edit: the source on the left and, in the results, the page itself.

## The source editor

The editor colours a sheet by role (prose, quantity, block, raw Calcpad, plot). It underlines
mistakes and lists them in the **Errors** dialog; the status bar at the bottom shows the error,
warning and information counts.

| Action | How |
| --- | --- |
| Complete a name, function, unit or keyword | just type; Tab accepts the highlighted entry, Ctrl+Space opens the list by hand |
| Rename a name everywhere | F2, or the editor's context menu, or the Variables tab (right-click a name) |
| Find a variable or function and its value | Ctrl+; (Cmd+; on a Mac) |
| Go to a name's definition | F12, or hover to see the defining line and its Calcpad spelling |
| Find references | Shift+F12, or the editor's context menu |
| Tidy the sheet | Shift+Alt+F (Format Document) |
| Wrap long lines | Alt+Z |
| Find | Ctrl+F (Cmd+F on a Mac) |
| Insert a symbol, function, unit or snippet | the **Insert** tab in the sidebar, or the ribbon |

### Completion

The list opens as you type, in a worksheet and in a sheet alike, and offers what fits where the
cursor is:

* **Names defined above the cursor**, with their last calculated value next to them
  (`load = 12 kN`). A name defined further down is not offered, and neither is the word you are
  typing. The parameters of the function you are writing on the current line are offered too.
  Greek letters, subscripts and primes work as in any name: typing `σ` finds `σ_c`.
* **Built-in functions**, with their parameters and a one-line description. Pick one and Tab fills the
  call with a placeholder for each argument. In a sheet the arguments are separated with commas, in
  a worksheet with `;`. The same list is what the [function reference](reference/functions.md) shows.
* **Units, only units, right after a number** (`5 k` offers `kN`, `kg`, `kA`), and after a unit and
  `*` or `/` (`5kN*`). A sheet offers units after `->` as well. A worksheet's own custom units are in the list.
* **`#` directives and `$` commands** at the start of a line of a worksheet (`#fo` offers `#for`).
* **Block keywords and format markers** at the start of a line of a sheet: `if`, `else`, `for`, `while`,
  `repeat`, `plot`, `>>`, code fences for `calcpad` and `python`, `$$`, the input forms (`input`, `dropdown`,
  `radio`, `checkbox`) and `<!--sheet-->` on the first line.

Nothing is offered where the text is not code: a text or comment line, after a quote mark, in a note
(after `|`), in a string, in a `#markdown`, `#html` or `#python` block, in sheet prose and
formulas, and after the `?` of an input. The calculated values come from one calculation of the
whole worksheet (Python blocks are not run for it), so the first list after an edit may show the
value from a moment ago; the list never waits more than a quarter of a second for it.

### Rename everywhere

Put the cursor on a variable, function or macro and press **F2** (also in the editor's context
menu, and in the Variables tab: right-click a name and choose **Rename Everywhere**). A small box
opens at the top of the editor. It says how many places and lines will change, marks every place in
the text and shows the first changed lines as they will read. Enter (or the **Rename** button) applies
it as **one undo step**; Esc cancels.

* **Scope is respected.** A function's parameters, and the name bound inside `$Sum{ ... }`, are renamed only
  on their own line; a macro's parameters only inside its `#def` block. Renaming a variable `a` leaves
  a parameter `a` alone, and the other way round. In a sheet the same holds for a function's parameters
  and the variable of a `plot` line.
* **Only code is renamed.** Comments and text lines, strings, `#markdown` and `#html` blocks, and in a sheet
  prose, notes after `|`, headings, `$...$` formulas, and the unit after a number (`3 psi` stays) are left as
  they are. A formula is typeset text, not a reference to a variable, so it is not renamed.
* **Inputs follow the name.** A `?` input (`L = ?`, `L: ? 6 m`) is an ordinary definition and is renamed with the name.
* **Python is left alone except for what it hands over.** The names a Python block passes to the worksheet with
  `export(name=...)` or `export("name", ...)` are the worksheet's variables, so they are renamed with
  the name (the box counts them). Any other Python code, which is not a reference to a worksheet variable,
  is never touched.
* **A name that would collide is refused**, with the reason, and **Rename** stays off: a name that is not valid
  (it starts with a letter and holds letters, digits, `_` and primes), a keyword or built-in function or constant,
  a name already in use in the document or in an included file, and a name a parameter on a changed line would
  capture (or that would hide a variable used inside the function). A name that is also a unit is allowed, with a note.
* A name defined in an `#include`d file cannot be renamed from here (only this document is edited).

### The variable palette

**Ctrl+;** (Cmd+; on a Mac) opens a search box over every variable, function and macro of the document, each with
its last calculated value and unit and the line it is defined on. The key is not used by the browsers or by the
editor itself. Type to filter: letters in order are enough (`sc` finds `sigma_c`), and built-in functions are included
once you type something. **Up** and **Down** choose, **Enter** inserts the name at the caret (a function goes in as
`name()` with the caret between the brackets), **Alt+Enter** (or the arrow button on the row) goes to where it is
defined, **Esc** or the same key closes it. A tap on a row inserts it, so it works on a touch screen. The Variables tab
in the sidebar has the same names, with a search box that matches the name, the definition, the parameters
and the note.

The Insert tab has a search box that filters the whole library. In a sheet, entries that have no
sheet form (the data grid and the other `#UI` forms, `$Map`, directives) carry a `>>` mark and insert Calcpad text as `>>` lines.

## In-place editing in the results

Edit a sheet *where it stands*, in the preview.

* **Click** a block (a quantity line, a plot line, a table, a formula, a paragraph) and an editor opens
  over it. Dragging to select text does not open it.
* **Click away** applies the change. **Esc** throws it away. **Enter** applies at once (Ctrl+Enter
  for prose, where Enter makes a new line).
* Input and calculation lines are one-line edits. A paragraph, a heading, a list or a table opens
  as **rich text**: a small toolbar over the text with bold, italic, heading levels, bullets, a
  table and its row and column controls. A line under the box says "Ctrl+Enter or click away to
  apply · Esc to cancel".
* Everything you apply is written back to the source in one undo step. A paragraph or table you open
  and apply without changing keeps its exact original lines.

In a Calcpad worksheet, prose written as Markdown (a `#markdown` ... `#end markdown` block) is edited
the same way, by **double-clicking** it. The worksheet importer's Tidy and Markdown options write
this form (see [Importing worksheets](importing.md)).

What the rich-text editor writes is plain Markdown: `**bold**`, `_italic_`, `~x~` subscript, `^x^`
superscript, `++x++` underline, `-` lists and pipe tables (no merged cells; a cell is one line).
It loads the first time you use it.

### Insert between blocks

Between the rendered blocks there is a small **+**. It opens a menu to insert a block at that place:
Math line, Text, Heading, List, Table, Plot, Condition or Python. The new block goes in front of the
next block's source line, indented like it, and its first word is left selected in the editor so
you can type over it. On a touch screen the **+** appears as a tab on the left edge, around the
block you tapped.

## Phones and tablets

At 760 pixels wide or less, the app shows one pane at a time, switched from a bar along the bottom:
**Edit**, **Results** and **Insert**. The ribbon starts folded; a tap on its open tab folds it again.

* A **long press** (half a second) on a rendered line, paragraph or table opens the same in-place
  editor as a click. **Apply** and **Cancel** stay on screen.
* The in-place editor stays visible when the on-screen keyboard opens, and its buttons stay above
  the bottom bar.
* A row of keys under the line editor offers what a phone keyboard hides: `^ * / ( ) = < > ≤ ≥ √ π ° | _ , ;`
  and the common units (`mm`, `m`, `kN`, `MPa`, `kN·m`). A tap inserts at the cursor and keeps the
  keyboard open. The row scrolls sideways.
* A plot's hover toolbar is left out on a phone; pinch and drag still work.
* Python can need several times longer on a phone. The page offers the longer time limit when a
  block runs out of its ten seconds. See [Python blocks](python.md).

The app can be installed to the home screen and works offline. See
[Saving, opening and working offline](getting-started/saving.md).
