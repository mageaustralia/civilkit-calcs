# Importing a spreadsheet

<!-- description: Bring cells from an Excel or CSV file into a sheet as inputs, values, tables and labelled rows. Values are copied as plain lines, not recalculated or linked. -->

**Import spreadsheet** reads an Excel or CSV file and lets you pick cells from it and add them to a
sheet as named quantities and tables. Use it for a schedule of loads, a list of member sizes or a
column of test results that already lives in a spreadsheet.

**What it does and does not do.** The values are *copied* into the sheet as ordinary lines, so the
sheet carries them and a report prints without the file. Formulas in the file are **not**
recalculated and there is **no live link** back to the file: a formula cell gives the value the
file stored for it (spreadsheet programs save one), and the calculation is done by your sheet from
there. If the file changes, import the cells again. **Nothing is uploaded**; the file is read in the
page.

## How to import

Any one of:

* the **Import spreadsheet** button on the Document tab (next to Import worksheet)
* **File > Import spreadsheet...**

Choose a `.xlsx`, `.xlsm`, `.csv` or `.tsv` file. A panel opens over the page. It is not modal: you can
click in the editor to move the cursor between additions, and **Close** (or Esc) puts it away.

Open or create a **sheet** first. The lines are sheet lines, so adding into a Calcpad worksheet is
refused with a message.

## The panel

* **Tabs** along the top, one for each sheet of the workbook (a CSV has one).
* A **read-only grid** with column letters and row numbers. Formula cells are grey and italic, and
  hover (or the cell's tooltip) shows the formula text. Constant cells are plain.
* **Click a cell**, or **drag over a range**, to select it. On a touch screen tap a cell, switch on
  **Select range**, then tap the far corner.
* An **action bar** under the grid shows the selection, a proposed **name** and **unit** you can edit,
  and the actions that fit the selection.
* Cells you have added turn **green** with their name, so you can see what is already used.

| Selection | Action | Writes |
| --- | --- | --- |
| A constant number | **Add as input** | `span: ? 6 m` (the reader can change it) |
| A number or formula cell | **Add as value** | `Mstar: 56.25 kN*m` (the stored result) |
| A range of numbers | **Add as table** | a vector `[1; 2; 3]` or a matrix `[1, 2; 3, 4]` |
| A range with text | **Add as table** | a table of text for the report (not used in calculations) |
| Two columns, label then value | **Add labelled rows** | one quantity line for each row |

A formula cell cannot be an input, because its value is a result. A text cell has no number to add.

The lines go at the editor cursor: on the cursor's line when it is blank, otherwise on the lines after
it. Above them goes a short source comment, for example `<!-- loads.xlsx, Loads!B1 -->`. It is an HTML
comment, so it shows in the editor and prints as nothing in the report.

## Names and units

The name is guessed from the **label** to the left of the cell, or the **header** above it, and turned
into a sheet name (see [Quantities](sheet/quantities.md)): several words become camel case
(`Design load` is `designLoad`), Greek letters become their spellings (`φ factor` is `phiFactor`), a
leading digit gets an `n`, and a name that is a keyword gets a `_`. If the sheet already has the name,
a number is added (`span2`). Edit the name before you add if you want another.

The unit is taken from the label's brackets (`Length (m)`, `Load [kN/m]`, `Area (mm²)` is `mm^2`), or
from a unit in the cell beside the value (a unit column), when it looks like a unit. Words in
brackets such as `(optional)` are not taken. A wrong or empty unit is easy to fix in the name and
unit boxes, or in the sheet afterwards.

The label becomes the note after the value (`span: 6 m | Span`).

## Tables

A range of numbers becomes a vector (one row or column) or a matrix, with the unit on every
element. If the first row is text over numbers, it is treated as a header: the matrix holds the
numbers below, and the header names go in the source comment. Empty cells in the numbers read as 0.
A table is limited to 5,000 cells. Reach an element with `A.(row, column)` as for any matrix.

A range that has text in it is added as a plain table of the cells as shown, for the report only.

## Labelled rows

Select a range of two columns: the labels, then the values (a third column may hold the units).
Each row with a label and a number becomes one line, named from its label. Rows whose value is not
a number are left out, and the status line says how many. Tick **Rows as inputs** to make the
constant rows inputs.

## Files and limits

* **Excel**: `.xlsx` and `.xlsm`. Shared and inline strings, numbers, booleans and error values are
  read. Dates are shown as their serial numbers. Merged cells are ignored (each cell is shown
  where it is stored). Macros are never run. The old `.xls` format and password-protected
  workbooks are refused with a message; save as `.xlsx` or `.csv`.
* **CSV and TSV**: quotes (and doubled quotes), line breaks inside quotes, a comma, semicolon or
  tab delimiter found by itself, and a decimal comma when the delimiter is not a comma (`6,5`).
* Files over 5 MB, and workbooks with more than 200,000 cells, are refused. Save the part you
  need as a CSV.
* A damaged file shows a message and changes nothing.
* The grid shows 200 rows at a time (a button shows more) and the first 100 columns.
* Text cells cannot be used as values in calculations; only numbers and booleans (as 1 or 0).
