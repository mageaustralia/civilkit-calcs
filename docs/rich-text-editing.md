# Rich-text editing of prose

Prose written as Markdown can be edited as formatted text instead of source. In a sheet, click a
paragraph, heading, list or table in the preview and an editor opens in its place; the rest of the
page moves down to make room and follows scrolling. Clicking away applies the change (Ctrl+Enter
applies at once, Enter makes a new line), and Esc leaves the text alone. A drag that selects text
does not open the editor. Everything you apply is written back to the source in one undo step. On a
touch screen a long press opens the editor, and Apply and Cancel stay on screen.

In a Calcpad worksheet the same editor opens on a **double-click** on the Markdown text, and Apply
(or Ctrl+Enter) or Cancel close it. An equation, an input line or anything else opens the one-line
source editor instead, on the same gesture (one click in a sheet, a double-click in a worksheet).

## What is editable

* In a Calcpad worksheet: a `#markdown` ... `#end markdown` block. The lines between are one raw
  Markdown document, so headings, paragraphs separated by blank lines, lists and pipe tables all work.
  This is the form the import options Tidy and Markdown write (`docs/worksheet-import.md`).
* In a sheet (`<!--sheet-->`): the run of prose lines around the line you clicked. A sheet needs no
  markers, because it is Markdown already. Do not add `#markdown` lines to a sheet: the compiler wraps
  its prose in its own blocks and nested markers leave stray apostrophes.
* Older worksheets with `#md on` ... `#md off` and one comment line per paragraph still open. There
  every line is a Markdown document of its own, so a paragraph is one line and a table or code block
  cannot be written; the editor says so instead of writing something broken.

## What the editor writes

A paragraph or table you did not change keeps its exact original lines, so opening the editor and
applying without a change leaves the file as it was. Changed text is written as plain Markdown:

```text
## Design basis

The **load** is _applied_ at f~c~ = 4 ksi, x^2^ and ++underlined++ text.

| Member | Load | Span |
| ------ | ---- | ---- |
| AB     | 5 kN | 3 m  |
```

Subscript is `~x~`, superscript `^x^` and underline `++x++` (the engine's Markdown); a subscript or
superscript with a space in it stays as the `<sub>` or `<sup>` tag. The editor shows these three as
their tags while you edit, and writes them back as the marks above.

## Limits

* Tables are plain pipe tables: no merged cells, and a cell is one line.
* A formula written with `$$...$$` inside a paragraph is plain text to the editor; if you edit that
  paragraph, check that its underscores and asterisks survived.
* The editor is loaded the first time you use it (about 140 KB gzipped).
