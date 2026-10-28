# Editing

There are two places to edit: the source on the left and, in the results, the page itself.

## The source editor

The editor colours a sheet by role (prose, quantity, block, raw Calcpad, plot) and completes the
names the sheet defines, Greek spellings, function names and block snippets. It underlines mistakes
and lists them in the **Errors** dialog; the status bar at the bottom shows the error, warning and
information counts.

| Action | How |
| --- | --- |
| Go to a name's definition | F12, or hover to see the defining line and its Calcpad spelling |
| Find references | Shift+F12, or the editor's context menu |
| Tidy the sheet | Shift+Alt+F (Format Document) |
| Wrap long lines | Alt+Z |
| Find | Ctrl+F (Cmd+F on a Mac) |
| Insert a symbol, function, unit or snippet | the **Insert** tab in the sidebar, or the ribbon |

The Insert tab has a search box that filters the whole library. In a sheet, entries that have no
sheet form (the data grid and the other `#UI` forms, `$Map`, directives) carry a `>>` mark and insert Calcpad text as `>>` lines.

### The text toolbar

Select some words in a line of prose, or click into one, and a small toolbar appears above the line:
**Style** (Normal, Heading 1, 2 or 3), **B**, *I*, **</>** for code, **Link**, a bulleted list and a
numbered list. It does not appear on quantity lines, in code fences or inside a `$$` formula, where
those would only add characters that mean something else. The same toolbar appears over the text in
the in-place rich-text editor.

Each button's tooltip shows the Markdown it types (`Bold: **text**`, `Link: [text](address)`,
`Numbered list: 1. item`), so you can learn the syntax and type it yourself next time.

* A button is a toggle: Bold on text that is already bold takes the `**` off again, and a list button
  on a list takes the markers off. A button that is on is shown pressed.
* With no selection, Bold wraps the word at the caret.
* **Link** wraps the selection as `[text](url)` and selects `url`, ready for the address.
* The edit is made in the source, in one undo step, and the editor keeps the focus.
* On a phone or tablet the floating bar stays away so it does not fight the system selection menu;
  the in-place editor keeps its fixed bar there.

### Name completion

Type a letter in a calculation and the editor offers the names the sheet has defined on the lines
above (with the defining line beside each, for example `w: 12 kN/m | load`) and the built-in
functions (`max`, `sqrt`, `sin` and the rest). Accepting a function writes the brackets and leaves the
caret between them. Nothing pops up while you write prose, in a note after `|`, in quoted text or in
a `$...$` formula. Ctrl+Space on an empty spot offers everything, including the Greek spellings and
the block snippets.

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

Move the pointer between two rendered blocks and a dashed line appears with a **+** in the middle.
Click the **+** for a menu of blocks to insert there: Math line, Text, Heading, List, Table, Plot,
Condition, Python or Page break (see [Page breaks](exporting.md#page-breaks)). The new block goes in front of the next block's source line, indented like it,
and its first word is left selected in the editor so you can type over it. On a touch screen the
**+** stays hidden until you tap a block, and then shows only in the two gaps beside it.

### Cut, copy and paste blocks

The block menu (the **⋮** grip at the left of a block) has **Cut**, **Copy** and **Paste below**.
Copy puts the block's source text on the system clipboard, so you can also paste it into another
sheet or a chat. Cut does the same and then deletes the block, with an **Undo** toast. **Paste below**
inserts the clipboard text after the block, with one blank line between, as a single undo step. If
the browser will not let the page read the system clipboard, Paste uses the last block you copied
or cut in this window. The **+** menu between blocks also lists **Paste** whenever there is something
to paste.

**Ctrl+Shift+K** (**Cmd+Shift+K** on a Mac) deletes the block that holds the editor caret, with the
same Undo toast. In a file that is not a sheet it keeps its usual meaning, delete line.

### Turning a block off

**Disable** in the block menu comments a block out: it drops out of the preview and the calculation
but stays in the sheet. A sheet has no multi-line comment, so each line of the block is wrapped in
an HTML comment (the one comment the language has):

```sheet
a: 2 m
<!-- b: a * 3 -->
```

A disabled block is not drawn at all, so it has no grip. To get it back, put the caret in it and press
**Ctrl+Shift+/** (**Cmd+Shift+/** on a Mac), which toggles Disable and Enable, or press Undo
right after disabling. Limits: every line is wrapped on its own, a blank line inside the block
becomes an empty comment so the block stays in one piece, and a `-->` inside a line is written as
`--&gt;` while it is disabled and restored on Enable. Anything that used a disabled math line sees
an undefined name until it is enabled again.

### Collapsing sections

Hover a heading in a sheet and a small arrow shows under its grip. Click it and every block under
the heading, up to the next heading of the same or a higher level, folds away; the heading shows
how many blocks are hidden. Click the arrow again to open the section. A folded `##` section hides
only its own blocks, a folded `#` section hides its `##` sub-sections too. The grip menu of a
heading also has **Collapse all** and **Expand all**.

Folding is only a view: the sheet text and the results do not change, and printing and exporting
always include everything. Folds stay as you edit and when you switch between tabs. Headings are
matched by level, text and how many headings with the same text come before them.

### Sorting a table

Click a column heading of a table in the preview to sort its rows by that column: the first click
sorts ascending, the second descending, the third puts the rows back in the order of the source. An
arrow next to the heading shows which way it is sorted. A column of numbers sorts by value, even
with a unit after the number (`2.5 kN` comes before `10 kN`); any other column sorts as text, with
`A2` before `A10`. Empty cells stay at the bottom.

Sorting is only a view, like folding: the sheet text does not change, the sort stays as you edit,
and printing and exporting show the rows as they are written. The headings are buttons, so Tab
reaches them and Enter or Space sorts; on a phone, tap a heading. See
[Tables](sheet/prose.md#tables).

### Reordering blocks

Drag a block's grip up or down; a blue line shows where it will land, and the block goes there when
you let go. Dragging a heading takes its whole section along: the heading and the blocks up to the
next heading of the same or a higher level (folded blocks too). A section cannot be dropped inside
itself. The move is one step in the editor's history, and a toast offers **Undo**. A click on the
grip, or a movement of under 6 pixels, still opens its menu. On a touch screen, tap a block to show
its grip, then drag the grip.

### Plot chips

Under each plot in the preview there is a row of chips: Log x, Log y, 1:1 (equal axis scale),
Copy data, PNG and SVG. Copy data puts the plotted values on the clipboard as tab-separated text,
ready for a spreadsheet. The 3D plots have Copy data and PNG only.

### Sliders on inputs

Give a number input a range and Input mode shows a slider under its box:

```sheet
L: ? 6 m slider 2..12 step 0.5 | span
w: ? 12 kN/m slider 0..30 | design load
M: w * L^2 / 8 -> kN*m | maximum moment
```

Dragging the slider (or using the arrow keys, Home and End) fills the box and recalculates the
results while you drag; typing in the box moves the slider. The numbers are in the unit of the
default, the slider stops at the minimum and maximum, and `step` is the size of one stop (without it,
about a hundredth of the range). A line without `slider` has no slider. On a phone the track is
taller so a finger can use it. See [Templates with inputs](templates.md#sliders-for-number-inputs).

## The math keyboard

The **Keyboard** item in the status bar opens an on-screen math keyboard with tabs for numbers and
operators, functions, calculus and plots (sum, integral, root, parametric plot), matrices, capital
and lower-case letters, Greek letters and units. On a computer it is a panel you can drag by its title; on a touch screen it docks
at the bottom. A key or template inserts at the cursor of the editor, or of the in-place editor when
one is open. In a sheet, angles are degrees unless the sheet has a `#rad` line.

## Convert a selection to a formula

Select an expression you wrote in a sentence, say `w*L^2/8`, and press **Ctrl+Shift+E** (**Cmd+Shift+E** on a
Mac), or choose **Convert Selection to Formula** in the editor's right-click menu, or **= Formula** on the text
toolbar. In a sheet this

```sheet
w: 12 kN/m
L: 6 m

The moment at midspan is w*L^2/8.
```

becomes

<!-- expect: res1 = w · L ^2 8 | 54 kNm -->
```sheet
w: 12 kN/m
L: 6 m

The moment at midspan is w*L^2/8.
res1: w*L^2/8
```

The new line goes after the paragraph, and its name is selected: type a better one (`Mmid`) and it is
named. Where the selection lands:

* **In text** (prose, a note after `|`, a Calcpad text line or comment): a line of its own after the
  paragraph. In a sheet it is a named line, `res1: w*L^2/8`; in a Calcpad worksheet a plain calculation
  line, `w*L^2/8`. If the selection is the whole line, the line itself becomes the formula. A selection
  written with its name, `M = w*L^2/8`, keeps that name, and `$...$`, backticks and a full stop around it
  are dropped.
* **In a calculation**: the expression is pulled out to a named line above, and the selection becomes the
  name: `M: w*L^2/8 -> kN*m` turns into `res1: w*L^2/8` and `M: res1 -> kN*m`. Both copies of `res1` are
  selected, so typing a name renames the two at once.

The new name is the first of `res1`, `res2`, ... that the document does not use yet. The whole change is one
undo step. A selection that is not an expression on one line (plain words, a heading, a code fence, an
input's default) is left alone and the editor says so. On a phone or tablet a **= Formula** button shows
under the selection instead of the toolbar.

## The sidebar

The sidebar has three tabs: **Insert**, **TOC** (the outline of headings) and **Variables**.
Settings, Export, Errors and Prettify open as dialogs from the ribbon (and Errors from the status
bar); Esc closes them.

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
