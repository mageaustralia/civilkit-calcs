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
Condition or Python. The new block goes in front of the next block's source line, indented like it,
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
