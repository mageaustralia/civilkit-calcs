# Rich-text editing of prose

Prose written as Markdown can be edited as formatted text instead of source. Double-click a paragraph,
heading, list or table in the preview and an editor opens in its place; the rest of the page moves down to
make room and follows scrolling. Apply (or Ctrl+Enter) writes the text back to the source in one undo
step, and Esc or Cancel leaves it alone. Double-clicking an equation, an input line or anything else
still opens the one-line source editor.

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
