# Introduction

CivilKit Calcs is an engineering calculation sheet that runs in your browser. You write a
calculation as readable text, with quantities, units, formulas and notes, and the page works it
out and shows a formatted report. You can plot the results, export the report to Word, Excel or
PDF, and keep working with no network connection.

Everything is calculated on your own computer. The engine is compiled to WebAssembly and runs
inside the page, so a calculation is never sent to a server.

## What you get

* **Real units.** `6 m`, `12 kN/m`, `300 MPa`. The engine multiplies, divides and converts
  them, and it refuses to add a length to a time. A moment comes out in kilonewton metres
  because the units said so, not because you typed a label.
* **Two ways to write.** The *sheet* format is Markdown prose with `name: value` lines. The
  *Calcpad worksheet language* (`.cpd`) is the full language of CalcpadCE, with macros, input
  forms and every directive. Both run on the same engine, and a sheet can show you the Calcpad
  it becomes.
* **Vectors, matrices and a large function library.** Linear solves, eigenvalues, interpolation,
  numerical integration, root finding and more.
* **Plots.** A one-line `plot` command, the Calcpad `$Plot` and `$Map` commands, and a Python
  block that draws with a matplotlib-style or Plotly-style interface, including 3D charts.
* **Python blocks.** Small Python programs inside a sheet, run in the page, with results handed
  back to the sheet as ordinary quantities.
* **Templates with inputs.** Turn a calculation into a fill-in form.
* **Symbolic maths.** Rearrange a formula, differentiate, integrate or solve exactly.
* **A worksheet importer.** Bring in worksheets from another calculation tool.
* **Export.** Word with real equations, Excel with native charts, and PDF through the print
  dialog.
* **Free and open source.** The source is on GitHub under the MIT licence.

## Where to start

If you have never used it, open [Your first sheet](getting-started/first-sheet.md) and follow it
through once. It takes about five minutes. After that, read [The sheet format](sheet/index.md),
which describes everything you can write on a sheet line. The later chapters are independent: read
the ones you need.

Each chapter that matches a short how-to video carries a line starting with "Video:". The
videos are recorded from the scripts in the `book/videos` folder of the source.

## Credit

CivilKit Calcs is built on [CalcpadCE](https://github.com/imartincei/CalcpadCE), the continuation
of Calcpad by Nedelcho Ganchovski (Proektsoft) and contributors. The worksheet language, the
editor support and the sample worksheets all come from that work. See
[Credits and licences](appendix/credits.md).

## A note on the examples

Every `sheet` and `cpd` example in this guide was run through the real engine when the guide was
built (`book/check-examples.sh`). The results shown beside them are what the engine printed, with
the default setting of two decimals. The Python examples need the browser, so they are checked
for syntax by the script and were run in the page by hand.
