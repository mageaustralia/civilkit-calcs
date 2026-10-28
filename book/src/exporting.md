# Exporting

<p><strong>Video:</strong> Export a report to Word, Excel and PDF</p>
<video controls preload="none" poster="videos/08-export-word-excel-pdf.jpg" style="width:100%;max-width:960px" src="videos/08-export-word-excel-pdf.mp4"></video>

A calculated sheet leaves the page in four ways: a PDF through the print dialog, a Word document,
an Excel workbook and standalone HTML. All of them are built in the page, from the same calculated
report, so what you see is what you export. The Word and Excel writers load the first time you use
them and work offline.

The **Document** tab has buttons for **Print PDF**, **Word**, **Excel** and **Export...**. The
File menu has **Print PDF...**, **Export...** and **Export Excel...**. The **Export** dialog lists the
views you can export (Report, Preview, Input form, Unwrapped), each with its own **Save PDF**,
**Save Word** (not offered for the input form and the unwrapped listing) and **Save HTML** buttons,
and a **Plots** list that downloads each plot as an image, or all of them as a ZIP.

Which view to export: **Report** is the usual choice. It applies the values entered in an input form,
hides `#pre` text and shows `#post` text. Preview shows what the results pane shows.

## Page breaks

To start a new page at a point in the report, put a page break line there:

```sheet
# Loads

w: 12 kN/m | design load

<div style="page-break-after:always"></div>

# Bending

L: 6 m
M: w * L^2 / 8 -> kN*m
```

**Print PDF** and **Word** both start "Bending" on a new page. You do not need to type the line:

* In a sheet, the **+** between two blocks has **Page break** at the end of its menu.
* The ribbon has **Page break** on the **Document** tab (Page group) and the **Text Formatting** tab.
  It inserts the line at the cursor.

The preview draws the break as a dashed line labelled "Page break". The line is not printed and is
not in the Word file. A Calcpad worksheet (`.cpd`) writes the same thing as an HTML comment line,
`'<div style="page-break-after:always"></div>`, and the ribbon button inserts that form there.

Word also honours the other ways HTML forces a break on an element (`page-break-before: always`,
`break-before: page`, `break-after: page`). Excel has no pages, so the workbook ignores page breaks.

## Word

**Word** writes a `.docx` of the report:

* Equations are real Word equations (Office Math): fractions, subscripts, sums, matrices, units.
  In a test sheet with a handful of formulas the document held 21 equations.
* Headings, paragraphs, lists, links and tables keep their form. A Markdown pipe table gets rules.
* Plots and other drawings are pictures. A plot is stored as a PNG (the SVG is kept beside it, for
  versions of Word that read SVG).
* Pictures from the library are embedded.
* A formula typeset with LaTeX in a sheet appears as its TeX source in italics, not as a Word equation.
* An interactive Plotly chart cannot be carried. A paragraph stands where it was.
  A matplotlib figure from a Python block is carried as a picture.

The Word document has been checked in LibreOffice, not in Microsoft Word.

## Excel

**Excel** writes a workbook that mirrors the calculation:

| Sheet | Holds |
| --- | --- |
| Report | The report in document order. Headings and text in merged cells; every equation a row with its description, name, expression, expression with the numbers put in, value and unit. Text tables are cell ranges. |
| Variables | Every variable in the order it was first assigned, with its last value and unit. |
| One sheet for each vector or matrix | Its elements, a row per row, with the unit. A variable's row on Variables links to it. |
| Inputs | The variables set by input fields and `#UI` controls. |
| Plot data | The points of every plot, two columns per curve. |

A `plot` line or `$Plot` becomes a **native Excel XY chart** on the Report sheet at the place of the
plot, with its series pointing at the Plot data sheet, so you can edit the data and the chart redraws.
For a test sheet with one `plot` line the workbook held one chart and these sheets: Report,
Variables, K, F, u, Inputs and Plot data (K, F and u being a matrix and two vectors).

What the chart loses against the drawing: the faint fill under a curve, the exact fonts and grid
style. Colour maps and other drawings are pictures. The cells hold calculated values with the
expressions as text, not live Excel formulas.

The workbook opens in LibreOffice and a Python reader. It has not been tried in Microsoft Excel.

## PDF

**Print PDF** opens the browser's print dialog on the report. Choose "Save as PDF" as the destination.
The page size, orientation and margins come from the PDF settings (Settings, Header and footer).
The printed page carries the browser's own header and footer in place of title, date and page
numbers; turn "Headers and footers" off in the print dialog if you do not want them.

The printed page is the report: the buttons under plots are not printed, and an interactive chart is
replaced by a note.

## HTML

**Save HTML** writes a standalone page with the report, its styles, fonts and pictures. An exported
input form is static: it shows the controls, but nothing recalculates behind them.

## Files a worksheet writes

`#write` and `#append` save matrices to files. The browser keeps the file in the library and
downloads it when its content changes. By default this happens when you make a report; the Export
dialog's **Write files** choice and **Write to Disk** button change that.

## From the command line

The command-line front end of the engine converts a Calcpad worksheet to HTML, Word or Excel:
`Cli input.cpd output.xlsx`. It is built from the same code. A sheet has to be compiled first
(Convert to Calcpad, or the sheet compiler), because the command line reads Calcpad.
