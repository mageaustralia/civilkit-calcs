# Importing worksheets

Video: Import an existing worksheet (`videos/06-import-a-worksheet.mp4`)

The worksheet importer converts worksheet files from a spreadsheet-like calculation format, with
text, equations, plots and pictures placed freely on a page (files ending `.xmcd` and `.mcdx`), into
a new Calcpad worksheet that this engine calculates. It evaluates nothing itself: it only translates,
and then the one engine runs the result. Every number in the output is a number that was in the
source file.

**Nothing is uploaded.** The file is read and converted in the page, by a module that loads on first use.

## How to import

Any one of:

* **File > Import worksheet...**
* the **Import worksheet** button on the Document tab
* **File > Open...** and pick the file
* drag the file onto the page (a dashed hint shows while you drag)

The converted worksheet opens as a new unsaved tab named after the file, with the extension `.cpd`.
Picture files from the source go into the library under `/imported/<file name>/`. Importing the same
file name again replaces that folder.

A dialog then says how the import went, for example:

```text
Imported tank.xmcd: 18 of 21 regions converted live, 2 shown as static pictures, 1 need attention.
```

followed by a list of what did not convert, in plain words, by the number of regions affected
(for example "calls to a function from an add-in we cannot run" or "embedded objects"). Two toggles
re-run the import in the same tab:

* **Pictures of unconverted regions** (on by default): a region that cannot be converted is shown as a
  picture of the original, so the page still reads like the source.
* **Show saved results as static values** (off by default): places the values the source file saved
  where it could not compute them.

The dialog's last line repeats that the file was read in this page and was not uploaded.

## What converts

The importer follows the author's page: it reads regions in reading order and writes Calcpad lines.

| In the source | In the result |
| --- | --- |
| Text, with heading styles, bold, italic, underline, sub and superscript, paragraph fonts, sizes and colours | Text lines. Fonts and sizes can be tidied away (see below). |
| Definitions (`x := ...`), evaluations (`x =`), functions | Calcpad definitions and equation lines, with the source's names, subscripts and Greek letters. |
| Units | The same units, converted to this engine's names. A unit the engine does not know is reported. |
| Vectors, matrices and indexing | Vectors and matrices. The array origin is shifted to 1, with a note. |
| Sums, products, integrals, derivatives (supported forms) | The engine's `$Sum`, `$Product`, `$Area` and slope commands. |
| Programs made of loops and conditions, early exits, grown arrays | Expression blocks. |
| Plots (x-y plots) | `$Plot` lines. A plot's colours, markers and axis ranges may not carry over (a note says so). |
| Solve blocks of one or two unknowns | Solved here by iteration, and kept only if the equations hold at the answer; otherwise the value the file saved. |
| Statistics functions, complex numbers, simple text values (codes) | Converted. |
| Referenced worksheets | Their definitions are read in, when the file holds a copy. |
| Pictures | Embedded in the library. |

The author's equations are shown as the author wrote them (stacked fractions, sums with bounds,
subscripts, a transpose superscript), typeset as maths, and computed out of sight. The page reads
like the source, and the Word export turns the typeset equations into Word equations.

## What is frozen

Some values cannot be computed here, so the importer takes the number the source file *saved* and
shows it as a literal marked **frozen**:

* values that came from an input component (a combo box, a list box, a text box): the stored
  selection becomes the value, and it does not follow a change of the input;
* a call into a function from an external library (an add-in) that is not available here, when the
  file saved a value;
* a table read from a data file, as the table the worksheet saved;
* a solve block whose iteration does not verify.

The dialog counts frozen values and says so.

## What does not convert

Each gap is counted under a stable code and replaced by a placeholder: a typeset, not-calculated
equation or a static picture of the original. The main ones:

* range variables that fall outside the supported uses, and programs outside the supported subset;
* solve blocks of more than two unknowns, other solvers, symbolic evaluation;
* derivative and integral forms the importer does not map, and some summation forms;
* plots that are not x-y plots, or that store binary items;
* embedded objects, components, worksheet references with no copy, add-in functions with no saved value;
* text values that are joined, measured or formatted (`concat` and similar);
* a region the source has switched off, and some page headers and footers.

A region that depends on a gap is reported as downstream of it, and the dialog ranks the missing
features by the number of regions each blocks. A calculation that depends on an unconverted region
shows that region's names as undefined; read the dialog list first.

> The importer is a work in progress. Plan on checking an imported worksheet against the source,
> line by line, before relying on it, as you would with a hand conversion.

## After the import

* **Tidy text** (Document tab) removes imported fonts, sizes and colours from paragraphs and turns
  import markers into short notes. Equations are left alone.
* **Text to Markdown** tidies and then writes runs of simple paragraphs as Markdown, which you can then
  edit as rich text by double-clicking. See [Editing](editing.md).
* **Save**. The `.cpd` refers to the pictures by library path (`imported/<name>/R12.svg`). Pictures
  stay as long as the browser's library does. A `.cpd` opened later from disk shows its pictures
  only while the library holds them; export to HTML or Word first for a self-contained file.

## Limits

Input over 64 MiB, XML nested deeper than 256, a DTD, and pictures over 16 MiB each or 192 MiB in
total are refused or skipped. Errors are messages, not crashes: "... is not a worksheet this importer
reads", "not well-formed XML", "the file is a zip package that cannot be read".
