# Number format

Some languages write 1,5 where others write 1.5, and group digits as 1.000,5 or 1 000,5. CivilKit Calcs lets you
choose how numbers are **shown and typed**. Your worksheets do not change: a sheet always keeps the decimal point,
so a file reads the same on every machine.

## Choosing a format

Open **Settings** (the gear at the right end of the top strip). **Number format** is on the **General** tab, under
Language.

| Choice | Looks like |
|---|---|
| Automatic | follows the language of your browser |
| Point | 1,234.5 |
| Comma | 1.234,5 |
| Space and comma | 1 234,5 |

**Group digits** adds the thousands separator; it is off by default. The choice is remembered in this browser.
Add `?numfmt=comma` to the address to try a format on one page without storing it (`point`, `comma`, `space` or
`auto`, and `&numgroup=1` for digit groups).

## What changes

Results in the preview, the report, print and PDF, and the HTML export, plus the equations and text of the Word
export and the Report sheet of the Excel export. Excel's variable sheets keep real numbers. Variable names,
subscripts, exponents and units are never touched, and neither is text you wrote: a sentence that says
"use 1.5 m" stays as typed.

```sheet
L: 6.5 m
w: 12.5 kN/m
M: w * L^2 / 8 -> kN*m
```

```text
L = 6.5 m
w = 12.5 kN/m
M = w·L² / 8 = 12.5 kN/m · (6.5 m)² / 8 = 66.02 kN·m
```

In the Comma format that reads `L = 6,5 m` and `M = ... = 66,02 kN·m`, while the sheet itself still says `6.5`.

## Typing numbers

In a comma format an input box shows 6,5 and accepts both 6,5 and 6.5. The value the engine and the saved
values use is always 6.5. A slider moves the box as before, and a message such as "must be from 0,5 to 2,5" uses
your format. In an input box a point is always a decimal point, so write 1234,5 or 1 234,5 rather than 1.234.

## What does not change

Pasting into the editor is not converted, because a comma in a formula separates arguments. Plot axes, 3D views,
Python output and spreadsheet-style grids keep the decimal point.
