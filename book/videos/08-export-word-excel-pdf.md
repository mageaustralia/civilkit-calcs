# Video 8: Export a report to Word, Excel and PDF

* File to record: `08-export-word-excel-pdf.mp4`
* Length: about 85 seconds
* Book page: [Exporting](../src/exporting.md)
* Screen: as in video 1. The exported files open in other windows (Word or a compatible word
  processor, a spreadsheet program, a PDF viewer). Record those windows full screen.

## Before recording

1. Paste the sheet below into a new tab. It has values, a table, a vector, a matrix and a plot, so
   that every part of the exports has something to show.
2. Choose a folder for the downloads and clear it.

```sheet
# Floor beam report

A simply supported beam under a uniform load.

| Action | Symbol | Value |
|--------|--------|-------|
| Permanent | G | 4.5 kPa |
| Imposed | Q | 3 kPa |

L: 6 m | span
spacing: 2.4 m | beam spacing
G: 4.5 kPa
Q: 3 kPa
w: (1.2 * G + 1.5 * Q) * spacing -> kN/m | line load on one beam
Mstar: w * L^2 / 8 -> kN*m | maximum moment
K: [4, -2; -2, 3]
F: [10; 0]
u: lsolve(K, F)
M(x): w * x * (L - x) / 2
plot M(x * 1 m) for x in 0..6
```

What it calculates: the line load is 23.76 kilonewtons per metre, the moment 106.92 kilonewton metres,
and the solve gives 3.75 and 2.5.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | The finished report in the results pane. Caption: "One report, three formats". | None. | S1 |
| 0:10 to 0:30 | Open the **Document** tab. Click **Word**. The file downloads. Open it. Show the equations: they are real Word equations. Click one. Scroll to the plot and the table. | Document tab, Word. | S2 |
| 0:30 to 0:55 | Click **Excel**. Open the workbook. Show the sheets along the bottom: Report, Variables, K, F, u, Inputs and Plot data. On the Report sheet, point at the chart. Click a number in the Plot data sheet, change it, and watch the chart move. | Document tab, Excel. | S3 |
| 0:55 to 1:12 | Click **Print PDF**. The print dialog opens. Choose Save as PDF, and turn off headers and footers. Save. Open the PDF. | Document tab, Print PDF. | S4 |
| 1:12 to 1:25 | The three files side by side. End card: "No server, no upload". | None. | S5 |

## Narration

S1: Once a calculation is done, you can send it out in three formats.

S2: Word first. The equations come across as real Word equations, so you can edit them. Headings, tables and the plot come across too.

S3: Excel next. The report is the first sheet, one row per equation. The variables, each vector and each matrix get their own sheets. And the plot is a native chart, drawn from the data in the last sheet. Change a number, and the chart follows.

S4: For P D F, press Print. Choose Save as P D F in the print dialog, and turn off the browser's headers and footers if you do not want them.

S5: All three are made in your browser, from the same calculated report. Nothing is uploaded.

## Full text for the voice (one block)

Once a calculation is done, you can send it out in three formats. Word first. The equations come across as real Word equations, so you can edit them. Headings, tables and the plot come across too. Excel next. The report is the first sheet, one row per equation. The variables, each vector and each matrix get their own sheets. And the plot is a native chart, drawn from the data in the last sheet. Change a number, and the chart follows. For P D F, press Print. Choose Save as P D F in the print dialog, and turn off the browser's headers and footers if you do not want them. All three are made in your browser, from the same calculated report. Nothing is uploaded.

## Light marketing beat

* Scene 5: made in the browser, nothing is uploaded.

## Check after rendering

* Say the numbers only if they are on screen.
* The Excel scene needs a spreadsheet program that can edit chart data. Check it before recording.
