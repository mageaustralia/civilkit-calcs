# Plots

Video: Plots in the sheet format (`videos/03-plots-in-sheets.mp4`)

There are three ways to draw: the `plot` line of the sheet format, the Calcpad `$Plot` and `$Map`
commands, and Python blocks, which are covered in [Python blocks](python.md). All three can sit in
the same sheet.

## The `plot` line

```sheet
L: 6
w: 12
M(x): w * x * (L - x) / 2
plot M(x) for x in 0..L
```

The line is `plot expression for variable in from..to`. It becomes `$Plot{ M(x) @ x = 0 : L }`. The
expression can be a function you defined, or any expression of the variable:

```sheet
>> #rad
plot sin(x), cos(x) for x in 0..2*π
```

Several expressions, separated by commas, are drawn together, each in its own colour. An optional
`y =` before the expressions is dropped (`plot y = f(x) for x in 0..10`). A malformed `plot` line
is reported on its line in the report and in the editor.

The plot is an SVG drawing. It is sharp at any zoom, it prints well, and its data is exported to
Excel as a native chart.

### Radians, not degrees

Angles are in degrees by default. `sin(x)` over `0..2*π` is then the sine of the first six
degrees, which is a nearly flat line. Put `>> #rad` in the sheet before the plot, as above, or
plot over degrees (`0..360`).

### Plotting a function with units

The plotted variable must be a plain number. If `L` is `6 m`, then `for x in 0..L` stops with
`Inconsistent units`. Give the variable the unit inside the expression instead:

```sheet
L: 6 m
w: 12 kN/m
M(x): w * x * (L - x) / 2
plot M(x * 1 m) for x in 0..6
```

The axes are labelled with the variable names, and the end points of the curve are marked with
their values.

## Other kinds of plot: `$Plot` and `$Map`

A sheet reaches the rest of the plotting commands through `>>` lines.

```sheet
>> #rad
>> $Plot{cos(t) | sin(t) @ t = 0 : 2*π}
>> $Plot{cos(t) | sin(t) & 2*cos(t) | 2*sin(t) @ t = 0 : 2*π}
>> $Map{x*y @ x = -1 : 1 & y = -1 : 1}
```

| Form | Draws |
| --- | --- |
| `$Plot{y(x) @ x = a : b}` | A function of one variable. |
| `$Plot{y1(x) & y2(x) @ x = a : b}` | Several functions on one plot. |
| `$Plot{x(t) \| y(t) @ t = a : b}` | A parametric curve. |
| `$Plot{x1(t) \| y1(t) & x2(t) \| y2(t) @ t = a : b}` | Several parametric curves. |
| `$Map{f(x; y) @ x = a : b & y = c : d}` | A colour map of a surface. |

`$Plot` must be the first thing on its line. Anything after the closing brace is ignored. Plotting
supports real numbers only.

## Size and style

Set these by assigning them on a line of their own (in a sheet, a `>>` line), before the plot:

```sheet
L: 6
w: 12
M(x): w * x * (L - x) / 2
>> PlotWidth = 600
>> PlotHeight = 300
plot M(x) for x in 0..L
```

| Setting | Meaning |
| --- | --- |
| `PlotWidth`, `PlotHeight` | Size of the plot area in pixels. |
| `PlotSVG` | 1 for vector (SVG), 0 for raster. The web version draws SVG only. |
| `PlotAdaptive` | 1 to concentrate sample points where the curve bends (the default). Turn it off for rapidly oscillating functions, which can lose peaks. |
| `PlotStep` | Mesh size for colour maps. |
| `PlotPalette` | Colour palette 0 to 9 for colour maps. |
| `PlotSmooth` | 1 for a smooth gradient, 0 for isobands. |
| `PlotShadows` | 1 to shade colour maps. |
| `PlotLightDir` | Direction of the light, 0 to 7 clockwise from north. |

The same settings are in the Settings dialog, which sets the defaults for every worksheet. The
Plots tab of the ribbon has buttons for each form and for these lines.

## The buttons under a plot

In the preview, a plot has a row of small buttons. They are not part of a printout or an export.

| Plot made by | Buttons |
| --- | --- |
| A `plot` line or `$Plot` | Copy data, PNG, SVG |
| Python (matplotlib style) | Log x, Log y, 1:1, Copy data, PNG, SVG |
| Python (3D scene) | Copy data, PNG |

**Copy data** puts the plotted points on the clipboard. For a `plot` line the data is read back
from the drawing, through the axis labels, so it is correct to about four figures. **PNG** and
**SVG** save the picture.

## Plots in exports

* **Word**: plots are pictures. The SVG is rasterised for Word and kept beside the picture.
* **Excel**: a `$Plot` or `plot` becomes a native, editable XY chart with its data on a "Plot data"
  sheet. A colour map becomes a picture.
* **PDF**: the printed page has the plot as drawn.

See [Exporting](exporting.md).

## Python plots

For interactive charts, subplots, 3D surfaces and plots of computed arrays, use a Python block.
See [Python blocks](python.md).
