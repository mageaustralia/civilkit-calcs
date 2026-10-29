# Input forms with `#UI`

<!-- description: Turn Calcpad assignments into input controls with #UI: text boxes, drop-downs, radio buttons, checkboxes and editable grids in Input mode. -->

`#UI` is the Calcpad language's own input directive. It turns the assignment that follows it into an
input control in Input mode: a text box, a drop-down, radio buttons, a checkbox or an editable grid.

In a **sheet**, most inputs do not need it. A line such as `L: ? 6 m | span` is an input, and the
sheet compiler writes the `#UI` line for you. See [Templates with inputs](../templates.md), which
also covers the Input mode, saving values and sharing a template. Write `#UI` yourself when:

* you work in a Calcpad worksheet (`.cpd`);
* you need a data grid, a vector or matrix input, or one of the entry options below, which the `?`
  form has no way to say.

In a sheet, a `#UI` line is a Calcpad line, so it starts with `>>`.

## The input line

```text
>> #UI 'Span, 'L = 6m
```

* `>>` hands the line to the engine as written (leave it out in a `.cpd` worksheet).
* `#UI` says "show the next assignment as an input control in Input mode".
* `'Span, 'L = 6m`: an optional label (`'Span, ` is text) and then the assignment. `L = 6m` is the
  name and the default value, with its unit.

In Preview and in a report this line shows as a plain line (`Span, L = 6 m`). In Input mode the
`6` becomes a text box with `m` beside it.

### What may be on the right of the `=`

| Right-hand side | Allowed | Example |
| --- | --- | --- |
| A number, with or without a unit | Yes | `#UI L = 10m`, `#UI n = 4`, `#UI q = 3kN/m` |
| A vector or matrix literal | Yes | `#UI v = [1; 2; 3]`, `#UI M = [1; 2 \| 3; 4]` |
| `vector(n)` or `matrix(r; c)` | Yes | `#UI Z = vector(5)` |
| An expression | Only with `allowExpression` | `#UI {"allowExpression": true} h = L/2` |
| Text | No | |

## A sheet with `#UI` lines

The beam check as a sheet whose inputs are `#UI` lines. Names such as `phi` or `Delta_T` work in a `#UI` line as in any other line (the first is shown as
`φ`). A dotted name does not: a dot is the index operator.

<!-- expect: Span, L = 6 m | M _star = w · L ^2 8 = 12 kN ∕ m · ( 6 m ) ^2 8 = 54 kN · m | = 0.163 | OK: the section passes. -->
```sheet
# Beam bending check

Fill in the inputs, then read the check below.

## Inputs
>> #UI 'Span, 'L = 6m
>> #UI 'Design load, 'w = 12kN/m
>> #UI 'Yield stress, 'f_y = 300MPa
>> #UI 'Section modulus, 'Z_e = 1230000mm^3
>> #UI 'Capacity factor, 'cf = 0.9

## Check
Mstar: w * L^2 / 8 -> kN*m | maximum moment
Mcap: cf * f_y * Z_e -> kN*m | design capacity
ratio: Mstar / Mcap | utilisation
if ratio <= 1: "OK: the section passes." else: "Fails: increase the section."
```

Change the span to 10 and the load to 20 in the form: the moment is 250 kN·m, the ratio 0.753 and
the message still reads OK.

## Controls

The `type` property picks the control. When it is left out, the engine picks one from the right-hand
side.

```text
>> #UI {"type": "dropdown", "keys": ["Soft clay", "Stiff clay", "Dense sand"], "values": ["75kPa", "150kPa", "300kPa"]} 'Soil allowable bearing, 'q_a = 150kPa
>> #UI {"type": "radio", "keys": ["Concrete", "Steel"], "values": ["25", "78.5"]} 'Column unit weight in kN/m3, 'unit_weight = 25
>> #UI {"type": "checkbox"} 'Include footing weight (1 = yes), 'add_weight = 1
```

* A drop-down or radio group shows the `keys` and works by the matching `values` entry. A value can
  carry its own unit, so a menu can switch units as well as magnitudes (`"values": ["1m", "3.281ft"]`).
* A checkbox is `1` when ticked and `0` when not.
* Use the `?` form of a sheet for these when you can: `E: ? [Steel = 200 GPa, *Timber = 11 GPa]`
  writes this JSON for you.

### A grid and a radio group together

<!-- expect: levels = n_rows(loads) = 3 | = 3240 kN | = 1260 kN | G _column = unit _weight -->
```sheet
# Column load take-down

A column carries the tributary area of each level. Choose the column material, set the sizes and fill in the floor loads, one row per level.

## Inputs
>> #UI 'Tributary area per level, 'A_t = 36m^2
>> #UI 'Column section area, 'A_c = 0.09m^2
>> #UI 'Storey height, 'H = 3.5m
>> #UI {"type": "radio", "keys": ["Concrete", "Steel"], "values": ["25", "78.5"]} 'Column unit weight in kN/m3, 'unit_weight = 25
>> #UI {"type": "datagrid", "columnHeaders": ["Dead", "Live"], "rowHeaders": ["Roof", "Level 2", "Level 1"]} loads = [20; 5 | 35; 15 | 35; 15]

## Loads at the base of the column
levels: n_rows(loads)
G_floors: sum(col(loads, 1)) * 1 kPa * A_t -> kN | dead load from the floors
Q_floors: sum(col(loads, 2)) * 1 kPa * A_t -> kN | live load from the floors
G_column: unit_weight * 1 kN/m^3 * A_c * H * levels -> kN | weight of the column
Nstar: 1.2 * (G_floors + G_column) + 1.5 * Q_floors -> kN | factored axial load
```

The grid is a small spreadsheet of three rows and two columns with its own headers. Its default
(`[20; 5 | 35; 15 | 35; 15]`, with `|` between rows) fills the cells. Click a cell, type a number
and press Enter, and every line that uses `loads` follows. At the defaults:

```text
levels = n_rows(loads) = 3
G_floors = sum(col(loads; 1)) · 1 kPa · A_t = ... = 3240 kN    dead load from the floors
Q_floors = sum(col(loads; 2)) · 1 kPa · A_t = ... = 1260 kN    live load from the floors
G_column = unit_weight · 1 kN/m³ · A_c · H · levels = 25 · 1 kN/m³ · 0.09 m² · 3.5 m · 3 = 23.62 kN
N_star = 1.2·(G_floors + G_column) + 1.5·Q_floors = 1.2 · (3240 kN + 23.62 kN) + 1.5 · 1260 kN = 5806.35 kN
```

Choose **Steel**: `unit_weight` becomes 78.5, `G_column` 74.18 kN and `N_star` 5867.02 kN.

* A grid holds numbers only. To give them units, multiply in the sheet (`* 1 kPa` above) or write
  the units in the headers (`"Dead (kPa)"`).
* The grid size comes from the default value (or the `rows` and `columns` properties). Rows and
  columns cannot be added in the form.
* One `#UI` line per name is the stable arrangement. Rename an input and a value saved under the old
  name is not used any more.

## Entry options

A JSON object between `#UI` and the label sets properties:

```text
>> #UI {"forceUnits": false} 'Line load, 'q = 3kN/m
>> #UI {"allowExpression": true} 'Half span, 'h = L/2
```

* `forceUnits: false` puts the unit inside the box (`3kN/m`), so the user can type another unit.
* `allowExpression: true` accepts a formula as typed, and keeps it a formula across edits. Nothing
  is validated as you type: an entry the engine cannot read leaves the control in place and shows
  the error on the line.

Both were run in the form: the first box held `3kN/m` and `5kN/m` gave a moment of 22.5 kN·m; the
second held `L/2`.

### All the properties

| Property | Applies to | Meaning |
| --- | --- | --- |
| `type` | all | `entry`, `datagrid`, `dropdown`, `radio`, `checkbox`. Found from the right-hand side when omitted. |
| `forceUnits` | entry, grid | Default `true`: the unit stays in the document and only the number is editable. |
| `allowExpression` | entry, grid | Default `false`. Accepts any expression. Implies `forceUnits: false`. |
| `keys`, `values` | dropdown, radio | The labels shown and the values used. Both are required and must be the same length. |
| `rows`, `columns` | grid | The grid size. A literal is cut or padded with zeros to fit. |
| `rowHeaders`, `columnHeaders` | grid | Header labels. |
| `width`, `rowHeaderWidth`, `columnWidths` | grid | Sizes in pixels, or a percentage for `width`. |
| `style`, `reportStyle` | all | A CSS class for the control, or for its line in the report. |

The ribbon's Input/Output tab inserts a starting line for each control. In a sheet, Entry,
Dropdown, Radio and Checkbox insert the `?` form. Data grid and **More input forms** (the labelled,
vector, matrix and formula forms) insert `>> #UI` lines.

## The views, `#pre` and `#post`

`#pre` and `#post` let one file serve two audiences: an instruction ("fill in the values below")
that only the person at the form sees, and a note ("values used in this report") that only the
report carries. See the table of views in [Templates with inputs](../templates.md#using-the-form).
In a sheet they go in a `calcpad` fence:

````text
```calcpad
#pre
'<p>Fill in the values below, then press Report.</p>
#end pre
```
````

## In a Calcpad worksheet

A worksheet takes the lines as they are and has no renaming issue. The beam check as a `.cpd`:

```cpd
"Beam bending check
#UI 'Span, 'L = 6m
#UI 'Design load, 'w = 12kN/m
#UI 'Yield stress, 'f_y = 300MPa
#UI 'Section modulus, 'Z_e = 1230000mm^3
#UI 'Capacity factor, 'φ = 0.9
M_star = w*L^2/8|kN*m
M_cap = φ*f_y*Z_e|kN*m
ratio = M_star/M_cap
#if ratio ≤ 1
    'OK
#else
    'Fails
#end if
```

In a worksheet, **Save values** writes the entered values into a comment on the first line, and the
browser opens the form only when you press **Input**. The Excel export has an **Inputs** sheet
listing the variables set by input fields.

## Known limits

Workarounds for behaviour that is to be fixed. This is the only list of them on this page.

* **Superscript units and exponent notation in a `#UI` line.** The engine's own `#UI` line takes
  only the `^` and `*` spellings. `1230000mm³` is an error there (`Error parsing "mm³" as
  units`), and so is `1.23e6`. Write `1230000mm^3`. A `?` line has no such limit: it takes `mm³`,
  `kN·m`, `N/mm²`, `°` and `%`, and writes the engine's spelling for you.
* **A default should be one of the options.** In a raw line the default of a drop-down or radio
  group should be one of its `values`. A default that matches none still shows the first label
  while the calculation keeps using the default, so the control and the numbers disagree. (The
  `?` form cannot do this: its default is always one of the options.) A match is by value, so a
  default such as `25kN/m` finds the option written `25 kN/m`.
* **Two options cannot have the same value** in a `?` line. It is an error on that line.
