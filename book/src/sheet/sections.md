# Folding sections and key results

A long calculation has a few numbers the reader came for and a lot of working that leads to them. Two markers
let the sheet say which is which, without changing a single result.

## A section that folds

End a heading with `{collapse}` and the section under it is folded away until the reader opens it:

<!-- expect: Beam moment | L = 6 m | M = 54 kN | utilisation = 0.3 -->
```sheet
# Beam moment

L: 6 m | span
w: 12 kN/m | design load

## Workings {collapse}
The moment at mid-span of a simply supported beam under a uniform load is $w L^2 / 8$.
M: w * L^2 / 8 -> kN*m | maximum bending moment
Ms: 180 kN*m | moment capacity

## Result
utilisation: M / Ms | bending utilisation
```

In the preview and the report the heading shows with a small arrow in front of it. Click the heading to open
the section and click it again to fold it. The lines under a folded heading are still calculated, so `M` is
there for the line after it. The section runs to the next heading of the same level or a higher one, so a
`###` under a `##` is part of it, and it ends at the end of the sheet if no such heading follows.

`{collapse open}` starts the section open. Use it for a section the reader should see at first and may fold away.

Only a heading at the top level of the sheet can fold. A `{collapse}` heading inside an `if`, `for`, `while` or
`repeat` block is ignored, with a warning.

When you print, or save as PDF, every folded section is opened, so the printed report is complete. The Word and
Excel exports include them too. There is no option to leave a folded section out of an export.

## Key lines

End a quantity or an input line with `{key}` to mark it as one of the numbers that matter:

<!-- expect: L = 6 m | M = 54 kN | utilisation = 0.3 -->
```sheet
L: ? 6 m | span {key}
w: 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum bending moment {key}
Ms: 180 kN*m
utilisation: M / Ms | bending utilisation {key}
```

A key line carries a star after it and is highlighted in the preview, the report and the printed page. In Word
and Excel the star is in the line, though the highlight is not. The marker goes last, after the note.

## A list of the key lines

A line that is just `{key results}` becomes a list of the key lines above it, with the values the engine
worked out and their notes:

<!-- expect: Key results | L = 6 m span | M = 54 kN | utilisation = 0.3 -->
```sheet
L: ? 6 m | span {key}
w: 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum bending moment {key}
Ms: 180 kN*m
utilisation: M / Ms | bending utilisation {key}

## Key results
{key results}
```

Put it after the lines it lists: a sheet is calculated from the top, so a key line written below the list has no
value yet and is not in it. A line inside an `if` or `for` block, and a function definition, can be marked but
are not listed.

In the editor, **Text formatting > Fold** inserts a folding heading, **Key results** inserts the heading and the
list, and the **+** between blocks of the preview offers both. Format Document writes the markers as one space
and the marker.
