# Video 3: Plots in the sheet format

* File to record: `03-plots-in-sheets.mp4`
* Length: about 80 seconds
* Book page: [Plots](../src/plots.md)
* Screen: as in video 1.

## Before recording

1. New sheet, with the example lines deleted.
2. Know the one trap: angles are in degrees by default, so a sine plot needs a radians line first.
   The video shows it.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | Empty sheet. Caption: "One line, one plot". | None. | S1 |
| 0:10 to 0:28 | The inputs and the moment function appear, then the `plot` line is typed. A parabola appears under the formula. | Type block A. | S2 |
| 0:28 to 0:42 | Add a second plot of the shear force. The two diagrams sit one above the other. | Type block B. | S3 |
| 0:42 to 0:58 | A sine and a cosine on one plot. First without the radians line (a flat line), then with it. | Type block C, then add the first line. | S4 |
| 0:58 to 1:10 | Under the plot sits a row of small buttons: copy data, PNG and SVG. Click **Copy data**. | Point at the buttons and click. | S5 |
| 1:10 to 1:20 | Show the Document tab, **Excel**. Caption: "plots become real Excel charts". | Click Excel. | S6 |

## What to type

Block A:

```text
L: 6
w: 12
M(x): w * x * (L - x) / 2
plot M(x) for x in 0..L
```

Block B:

```text
V(x): w * (L / 2 - x)
plot V(x) for x in 0..L
```

Block C:

```text
plot sin(x), cos(x) for x in 0..2*π
```

then, above it:

```text
>> #rad
```

The complete sheet, as it is checked against the engine:

```sheet
L: 6
w: 12
M(x): w * x * (L - x) / 2
plot M(x) for x in 0..L
V(x): w * (L / 2 - x)
plot V(x) for x in 0..L
>> #rad
plot sin(x), cos(x) for x in 0..2*π
```

## Narration

S1: Plots in a sheet take one line. No other program is needed.

S2: Define a function of x. Then write the word plot, the function, the word for, the variable, and the range with two dots. The curve is drawn right under your formula, and it follows every change you make to the inputs.

S3: Add another for the shear force. Each plot line makes its own drawing, so one sheet can hold a whole set of diagrams.

S4: Several functions can share one plot. Separate them with commas. If the sine looks flat, the angles are in degrees, which is the default. Switch to radians with one line, and the wave appears.

S5: Under each plot, small buttons copy the data, or save a picture as a P N G or an S V G file.

S6: And when you export to Excel, a plot becomes a real chart that you can edit.

## Full text for the voice (one block)

Plots in a sheet take one line. No other program is needed. Define a function of x. Then write the word plot, the function, the word for, the variable, and the range with two dots. The curve is drawn right under your formula, and it follows every change you make to the inputs. Add another for the shear force. Each plot line makes its own drawing, so one sheet can hold a whole set of diagrams. Several functions can share one plot. Separate them with commas. If the sine looks flat, the angles are in degrees, which is the default. Switch to radians with one line, and the wave appears. Under each plot, small buttons copy the data, or save a picture as a P N G or an S V G file. And when you export to Excel, a plot becomes a real chart that you can edit.

## Light marketing beat

* Scene 6: plots become native Excel charts.

## Check after rendering

* The flat sine is shown for two seconds before the radians line is added.
* The row of buttons is only in the preview, not in the print or the export. Record it in the preview.
