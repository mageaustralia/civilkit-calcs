# Templates with inputs

Video: Templates with inputs (`videos/05-templates-with-inputs.mp4`)

A template is a calculation that you fill in rather than edit. You mark some quantities as
**inputs**, switch the results pane to **Input**, and those quantities become text boxes, drop-downs,
radio buttons, check boxes and editable grids. Type a new value and the whole sheet recalculates.
The same file is still an ordinary sheet: in Preview and in the report the input lines look like
any other line.

This chapter describes what works today. Inputs come from the Calcpad language (`#UI`), and a sheet
reaches them through `>>` lines, so a template is a sheet whose input lines start with `>>`.
Everything below was run in the page: the forms were opened, filled in and read back.

> There is no dedicated input syntax in the sheet format yet. If you see `>> #UI` lines that look
> heavy, that is why. The sheet's own lines (`name: value`) are always plain values and are not
> turned into form controls.

## The input line

```text
>> #UI 'Span, 'L = 6m
```

* `>>` hands the line to the engine as written.
* `#UI` says "show the next assignment as an input control in Input mode".
* `'Span, 'L = 6m` is an optional label (`'Span, ` is text) followed by the assignment.
  `L = 6m` is the name and the default value, with its unit.

In Preview and in a report, this line shows as a plain line (`Span, L = 6 m`). In Input mode the
`6` becomes a text box with `m` beside it.

### What may be on the right of the `=`

| Right-hand side | Allowed | Example |
| --- | --- | --- |
| A number, with or without a unit | Yes | `#UI L = 10m`, `#UI n = 4`, `#UI q = 3kN/m` |
| A vector or matrix literal | Yes | `#UI v = [1; 2; 3]`, `#UI M = [1; 2 \| 3; 4]` |
| `vector(n)` or `matrix(r; c)` | Yes | `#UI Z = vector(5)` |
| An expression | Only with `allowExpression` | `#UI {"allowExpression": true} h = L/2` |
| Text | No | |

Write the unit the Calcpad way: `mm^3`, not `mm³` (the `³` sign is an error in a `#UI` line). Write
`1230000mm^3`, not `1.23e6`, because exponent notation is not valid there either.

## Switching on Input mode

In the browser, a template opens like any sheet, as Preview. Switch to the form with the
**Input** button in the results toolbar (the toolbar shows Preview, Unwrapped, Input and Report),
or **Input mode** on the Input/Output tab, or **Input** on the Calculation tab. (The desktop app
opens a worksheet that has inputs straight into the form; the browser does not.)

In Input mode:

* The editor stays on the left, but the **Insert** panel says "Unavailable in input mode": it
  acts on the source, which the form does not edit.
* The results toolbar adds **Report**, **Print PDF**, **Save values** and **Exit input mode**
  buttons next to the four view buttons.
* Typing in a box and pressing Enter (or leaving the box) recalculates the whole sheet. The formula
  lines below the inputs show the new numbers as you go.
* **Report** opens the report beside the form with the values you have entered. **Print PDF**
  prints the same report.
* **Exit input mode** (or choosing another view) asks "Unsaved input values: save the values
  entered in the input form before exiting?" with Save, Don't Save and Cancel.

| View | Entered values applied | `#pre` shown | `#post` shown |
| --- | --- | --- | --- |
| Preview | No (the defaults), unless **Apply #UI Values in Preview** is on in Settings | Yes | Yes |
| Input | Yes | Yes | No |
| Report | Yes | No | Yes |

`#pre` and `#post` let one file serve both audiences: an instruction ("fill in the values below")
that only the person at the form sees, and a note ("values used in this report") that only the
report carries. In a sheet they go in a `calcpad` fence:

````text
```calcpad
#pre
'<p>Fill in the values below, then press Report.</p>
#end pre
```
````

### What does not work yet

* **Save values.** In a Calcpad worksheet this writes the entered values into a comment on the first
  line, so they come back the next time. In a sheet it puts that comment *above* the `<!--sheet-->`
  line, and a file that does not end in `.sheet` stops being read as a sheet. Until this is
  addressed, do not use Save values on a sheet. Choose **Don't Save**, or set the defaults in the
  `#UI` lines themselves, which is the reliable way to keep a value.
* The entered values are held in memory for the form. Reloading the page or closing the tab loses
  them.
* The input form itself cannot be exported to Word; the report can. Save a Word or PDF while the
  form is on, from the Document tab or the **Print PDF** button, and the entered values go in.

## Worked template 1: a beam check

The smallest useful template: four numbers and a check. Open a new sheet and type this.

```sheet
# Beam bending check

Fill in the inputs, then read the check below.

## Inputs
>> #UI 'Span, 'L = 6m
>> #UI 'Design load, 'w = 12kN/m
>> #UI 'Yield stress, 'f_y = 300MPa
>> #UI 'Section modulus, 'Z_e = 1230000mm^3
>> #UI 'Capacity factor, 'φ = 0.9

## Check
Mstar: w * L^2 / 8 -> kN*m | maximum moment
Mcap: phi * f_y * Z_e -> kN*m | design capacity
ratio: Mstar / Mcap | utilisation
if ratio <= 1: "OK: the section passes." else: "Fails: increase the section."
```

**In Preview and in the report** it reads as a normal calculation:

```text
Beam bending check
Fill in the inputs, then read the check below.

Inputs
Span, L = 6 m
Design load, w = 12 kN/m
Yield stress, f_y = 300 MPa
Section modulus, Z_e = 1230000 mm³
Capacity factor, φ = 0.9

Check
M_star = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m    maximum moment
Mcap = φ·f_y·Z_e = 0.9 · 300 MPa · 1230000 mm³ = 332.1 kN·m    design capacity
ratio = M_star / Mcap = 54 kN·m / 332.1 kN·m = 0.163    utilisation
OK: the section passes.
```

**In Input mode** the five inputs are yellow boxes: `Span, L = [ 6 ] m`, and so on, with the unit
next to the box. Change the span to 10 and the load to 20. The lines below update as soon as you press Enter:

```text
M_star = w·L²/8 = 20 kN/m · (10 m)² / 8 = 250 kN·m
ratio = M_star / Mcap = 250 kN·m / 332.1 kN·m = 0.753
OK: the section passes.
```

Things to notice:

* The name `φ` is typed as the Greek letter in the `#UI` line but as `phi` in the sheet line.
  A sheet renames `phi` to φ in its own lines, and it does not rename the name inside a `>>`
  `#UI` line. If the two spellings differ, the sheet line reports `Undefined variable: φ`.
  The same applies to any name a sheet spells specially (`Mstar`, `sigma_c`, `Delta_T`): in a `#UI`
  line write the spelled form (`M_star`, `σ_c`, `Δ_T`), or choose names that need no renaming
  (`f_y`, `Z_e`, `L`).
* The unit stays beside the box and only the number is editable, which is why the entered `6` can
  never be a length in the wrong unit. To let the user type the unit too, see
  [Entry options](#entry-options).
* A box accepts digits, a decimal point and a sign, and rejects other characters as you type.

## Worked template 2: a footing check with a soil menu

Adds a drop-down and a check box.

```sheet
# Footing bearing check

A square pad footing on a chosen soil. Pick the soil, fill in the load and read the check.

## Inputs
>> #UI 'Column load, 'N = 800kN
>> #UI 'Footing width, 'B = 2m
>> #UI 'Footing depth, 'D = 500mm
>> #UI {"type": "dropdown", "keys": ["Soft clay", "Stiff clay", "Dense sand"], "values": ["75kPa", "150kPa", "300kPa"]} 'Soil allowable bearing, 'q_a = 150kPa
>> #UI {"type": "checkbox"} 'Include footing weight (1 = yes), 'add_weight = 1

## Check
W_f: B^2 * D * 24 kN/m³ * add_weight -> kN | footing self weight
q: (N + W_f) / B^2 -> kPa | bearing pressure
ratio: q / q_a | utilisation
if ratio <= 1: "OK: the footing passes." else: "Fails: widen the footing."
```

At the defaults the report reads:

```text
W_f = B²·D·24 kN/m³·add_weight = (2 m)² · 500 mm · 24 kN/m³ · 1 = 48 kN    footing self weight
q = (N + W_f) / B² = (800 kN + 48 kN) / (2 m)² = 212 kPa    bearing pressure
ratio = q / q_a = 212 kPa / 150 kPa = 1.41    utilisation
Fails: widen the footing.
```

In Input mode, choose **Dense sand** in the menu. The labels ("Soft clay", "Stiff clay", "Dense
sand") are the `keys`; the engine substitutes the matching `values` entry (`300kPa`) into `q_a`:

```text
q = (800 kN + 48 kN) / (2 m)² = 212 kPa
ratio = 212 kPa / 300 kPa = 0.707
OK: the footing passes.
```

Now untick the check box and the footing weight drops out: `W_f` is 0 kN, `q` is 200 kPa and the
ratio 0.667. A check box is `1` when ticked and `0` when not, so the sheet uses it as a multiplier.

Notes on the controls:

* A drop-down is chosen by its labels but works by its values. A value can carry its own unit, so a
  menu can switch units as well as magnitudes (`"values": ["1m", "3.281ft"]`).
* The default (`q_a = 150kPa`) must be one of the `values`, so that the menu opens on the matching
  label. A default that matches none still shows the first label, while the calculation keeps using
  the default until a choice is made, so the menu and the numbers disagree.
* Use a check box for a yes or no that multiplies, or tests (`if add_weight == 1: ...`).

## Worked template 3: a column take-down with a grid and a radio group

Adds an editable table and a radio group.

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
values (`[20; 5 | 35; 15 | 35; 15]`, with `|` between rows) fill the cells. Click a cell, type a
number and press Enter, and every line that uses `loads` follows. At the defaults:

```text
levels = n_rows(loads) = 3
G_floors = sum(col(loads; 1)) · 1 kPa · A_t = ... = 3240 kN    dead load from the floors
Q_floors = sum(col(loads; 2)) · 1 kPa · A_t = ... = 1260 kN    live load from the floors
G_column = unit_weight · 1 kN/m³ · A_c · H · levels = 25 · 1 kN/m³ · 0.09 m² · 3.5 m · 3 = 23.62 kN
N_star = 1.2·(G_floors + G_column) + 1.5·Q_floors = 1.2 · (3240 kN + 23.62 kN) + 1.5 · 1260 kN = 5806.35 kN
```

Choose **Steel** in the radio group: `unit_weight` becomes 78.5, `G_column` 74.18 kN and `N_star`
5867.02 kN.

Notes on the controls:

* A grid is a plain-number grid. The cells hold numbers only; to give them units, multiply in
  the sheet (`* 1 kPa` above) or write the units in the headers (`"Dead (kPa)"`).
* The grid size comes from the default value (or the `rows` and `columns` properties). Rows and
  columns cannot be added in the form.
* A radio group is like a drop-down, laid out as buttons. The group opens with the button whose
  value equals the default. That match works for a plain number or a number with a simple unit
  (`25`, `200GPa`, `78.5kN`). A value with a divided unit such as `25kN/m` is not pre-selected, so
  the template keeps the numbers plain and puts the unit in the sheet line.
* Rename a grid or any input and a value saved under the old name is orphaned. One `#UI` line
  per name is the stable arrangement.

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

The ribbon's Input/Output tab inserts a starting line for each control (Entry, Dropdown, Radio,
Checkbox, Data grid, and **More input forms** for the labelled, vector, matrix and formula
forms). In a sheet these insert `>> #UI ...` lines.

## Rules of thumb

* Put inputs together at the top under a heading, so the form reads like a form.
* Keep each input to one `#UI` line and one name.
* Use the unit in the default value (`6m`, not `6`); the unit then stays with the number.
* Use names that need no renaming, or the spelled form in the `#UI` line (see template 1).
* Test the template in Report mode with the defaults: that is what a reader without the form sees.
* To hand a template to someone, send the `.sheet` file. They open it, press Input, fill it in and
  print.

## The same template in the Calcpad language

A Calcpad worksheet has no renaming issue and takes the lines as they are. Template 1 as a `.cpd`:

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

In a Calcpad worksheet, Save values works, and the browser still needs you to switch to Input.
The Excel export has an **Inputs** sheet listing the variables set by input fields.

## Model bindings

A `'<!--civilkit: bind L = path -->` comment before an input can tie it to a quantity in a
structural model. The specification and the in-app marker exist; no model is connected yet, so
typed values are used.
