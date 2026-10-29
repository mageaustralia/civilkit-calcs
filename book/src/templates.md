# Templates with inputs

<!-- description: Turn a calculation into a fill-in template: a question mark makes an input box, a drop-down, radio buttons or a checkbox, grouped as a form. -->

<p><strong>Video:</strong> Templates with inputs</p>
<video controls preload="none" poster="videos/05-templates-with-inputs.jpg" style="width:100%;max-width:960px" src="videos/05-templates-with-inputs.mp4"></video>

A template is a calculation that you fill in rather than edit. A few lines of the sheet are
**inputs**: a number with its unit, a choice from a list, a group of radio buttons, a checkbox.
Choose **Input** in the results bar and those lines become boxes, lists, buttons and ticks. Change
one and the whole sheet recalculates. The same file is still an ordinary sheet: in Preview and in
the report an input line reads like any other line.

A template is one `.sheet` file. There is nothing to install and nothing to configure: the inputs
are lines of text in the sheet.

## An input is a line with a question mark

A quantity line gives a name a value: `L: 6 m | span`. Put a `?` in front of the value and the
same line becomes an input, with that value as its default:

<!-- expect: L = 6 m | span | M = w · L ^2 8 = 12 kN ∕ m · ( 6 m ) ^2 8 = 54 kN · m -->
```sheet
# Beam moment

L: ? 6 m | span
w: ? 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum moment
```

Everything else about the line is as for a quantity: the name is a plain name (Greek spellings
work as usual), the unit is written after the number, and the text after `|` is a note. Later
lines use `L` and `w` like any other quantity. In Preview the sheet reads `L = 6 m    span`, with
the defaults. In Input mode `6` is a yellow box with `m` beside it and `span` after it. Change the
box to `8`, press Enter, and the moment follows.

The default is a number with an optional unit (`6 m`, `12 kN/m`, `0.9`, `1230000 mm³`). It is not
an expression: `2*3 m` is an error on its line, and the message says what to write. Units may be
written with the superscript signs (`mm³`) as everywhere else in a sheet. The name can be written
either way too: `phi: ? 0.9` is the input `φ`.

`name = ? 6 m` works as well as `name: ? 6 m`.

## The kinds of input

| You write | You get | The name is |
| --- | --- | --- |
| `L: ? 6 m \| span` | A box with the unit after it | The number typed, with the unit |
| `E: ? [Steel = 200 GPa, *Timber = 11 GPa] \| material` | A drop-down list | The value of the chosen option |
| `k: ? (Low = 1, High = 2) \| load class` | Radio buttons | The value of the chosen option |
| `ribs: ? on \| stiffeners fitted` | A checkbox | `1` when ticked, `0` when not |
| `L: ? 6 m [2 .. 12 step 0.5] \| span` | A slider with the box beside it | The number, kept between the bounds |

### Sliders

Put bounds after the default and the box gets a slider:

```sheet
# Beam moment

L: ? 6 m [2 .. 12 step 0.5] | span
w: ? 12 kN/m [2 .. 40 step 1] | design load
M: w * L^2 / 8 -> kN*m | maximum moment
```

`[2 .. 12 step 0.5]` is the smallest value, the largest value and, if you give one, the step the slider moves
in. The numbers are in the unit of the default (metres for `6 m`). The default must lie between the bounds.
In Input mode the slider sits next to the box: drag it, use the arrow keys, or type a number, and the sheet
recalculates a moment after you stop moving. A number typed outside the bounds is not used; the box turns red
and says `must be from 2 to 12`. Without `step` the slider takes about a hundred steps across its range; a typed
number does not have to land on a step. Preview, Report, PDF and the Word and Excel exports show the number, as
they do for a plain input, so a sheet with sliders reads the same when it is printed. Two of the sample
templates, the beam check and the cantilever, use sliders for their span, length and loads.

### Drop-downs and radio buttons

A choice is a list of `label = value` entries between square brackets (a drop-down) or round
brackets (radio buttons). The label is what the reader sees. The value is a number with an
optional unit, and it is what the name becomes when that option is chosen.

* The option marked with `*` is the one selected when the sheet opens. With no `*`, the first
  option is.
* A label that contains a comma, an equals sign or a bracket goes in double quotes:
  `"Beam (A), long" = 1`.
* A choice needs at least two options, and each option needs a value.
* A drop-down, radio buttons and a checkbox show no name. Their note (the text after `|`) becomes
  the label in front of the control. With no note the name is used.

### Checkboxes

`on` or `off` (`true` and `false` also work) sets the starting state, and the name is `1` or `0`.
That makes it a multiplier (`sw * rho * A`) or a test (`if ribs == 1:`).

### A choice that picks several things at once

A choice gives one number. When a material has a modulus, a strength and a density, let the
choice give an index and look the rest up in tables written after it. The template below does this
with `E_tab.mat`, the entry of `E_tab` that `mat` points at:

<!-- expect: E = ⃗ E _tab.2 = 70 GPa | The material is aluminium. -->
```sheet
mat: ? [Structural steel = 1, *Aluminium = 2, Timber = 3] | material
E_tab: [200 GPa, 70 GPa, 11 GPa] | modulus of elasticity
E: E_tab.mat -> GPa
if mat == 2: "The material is aluminium."
```

A new material is then one more option and one more entry in each table. The report shows the
number a choice stands for (`mat = 3`), so a line such as `if mat == 3: "The material is timber."`
tells the reader what it means. Most of the sample templates do this.

## Grouping the inputs as a form

A run of input lines is drawn as one form: a block with a bar down its left edge. Put the inputs
together under a heading, so that the heading titles the form. The run ends at the first line that
is not an input (a line of prose, a calculation). A blank line inside the run does not end it.

```sheet
## Inputs
L:    ? 6 m | span
w:    ? 12 kN/m | applied uniform load
mat:  ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
sw:   ? on | include the own weight of the beam
```

Preview and the report show the same block without the bar, and **Show compiled** shows plain
Calcpad (the bar is not written). **Format Document** (Shift+Alt+F) lines the colons up.

## Using the form

In the browser a template opens as Preview. Choose **Input** in the results bar (Preview,
Unwrapped, **Input**, Report), or **Input form** on the Input/Output tab. Then:

* Each input line is a control. A box holds the number only; the unit stays beside it, so a number
  cannot be typed in the wrong unit. A box accepts digits, a decimal point and a sign.
* Type a value and press Enter, or leave the box, and the sheet recalculates. Lists, buttons and
  the checkbox act at once.
* The **Insert** panel says "Unavailable in input mode": it acts on the source, which the form does
  not edit. Choose **Preview** to edit again.
* The bar has **Report** and **Print PDF**, which use the values you entered, **Save values** (it
  appears once you have changed a value) and **Exit input mode**.
* Leaving input mode with values you did not save asks "Unsaved input values: save the values
  entered in the input form before exiting?" with Save, Don't Save and Cancel.

| View | Entered values applied | `#pre` shown | `#post` shown |
| --- | --- | --- | --- |
| Preview | No (the defaults), unless **Apply #UI Values in Preview** is on in Settings | Yes | Yes |
| Input | Yes | Yes | No |
| Report | Yes | No | Yes |

The Word and Excel exports and **Print PDF** use the entered values. The Excel file lists the input
variables on its **Inputs** sheet. The form itself has no Word export; the report is the document. See [Exporting](exporting.md).

### Saving the values

The values you type are held in the page, so typing never changes the file. Choose **Save values**
to keep them. It writes one comment line directly under the `<!--sheet-->` line:

```text
<!--sheet-->
'<!--{"uiOverrides":{"L:1":"8"}}-->
```

The file is still a sheet. The next time it is opened and put in Input mode, or opened as a
report, the saved values are applied again. Preview still shows the defaults written in the sheet.
To change a default for good, edit the number in the sheet line.

A saved value belongs to the input's name. Rename an input and the value saved under the old name
is not used any more. Delete the comment line to go back to the defaults.

### What an input cannot do

* **Numbers only.** An input holds a number. There is no text box: a choice of names is a
  drop-down. There is no minimum or maximum either, so check a range in the sheet:
  `if L > 30 m: "Span out of range"`.
* **No expression as a default.** `2*3 m` is an error. Write the number, or compute the value in
  a line after the input.

### Known limits

Workarounds for behaviour that is to be fixed. This is the only list of them on this page.

* **No two options may have the same value.** `(Low = 1 m, High = 1 m)` is an error on its line,
  because the form could not tell which option is chosen. Give each option its own value.
* **A name cannot contain a dot.** A dot is the index operator (`E_tab.mat`), so `a.b: ? 5` is
  not an input.

## Worked template 1: a simply supported beam

Inputs: the span, the load, a section depth (drop-down), a material (drop-down), a deflection
limit (radio buttons) and whether the beam's own weight counts (checkbox). The beam is a doubly
symmetric I-beam described by one depth, so the section is a drop-down of three depths. This is
the sample **Template: simply supported beam check** in the Templates group of the Samples
browser, and the text below is the file as shipped.

The inputs, six lines:

```sheet
## Inputs
L:     ? 6 m | span
w:     ? 12 kN/m | applied uniform load
h:     ? [200 mm deep = 200 mm, *300 mm deep = 300 mm, 400 mm deep = 400 mm] | I-beam section
mat:   ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
limit: ? (L/250 = 250, L/360 = 360, L/500 = 500) | deflection limit
sw:    ? on | include the own weight of the beam
```

Note that `h` is a drop-down whose values are lengths: the choice sets the section depth directly.
The deflection limit is a number, 250, 360 or 500, and a later line divides the span by it
(`L / limit`).

<details>
<summary>The whole sheet</summary>

<!-- sample: Template simply supported beam.sheet -->
<!-- expect: M _max = w _tot · L ^2 8 = 12.5 kN ∕ m · ( 6 m ) ^2 8 = 56.26 kN · m | σ = M _max Z = 56.26 kN · m 691.76 cm 3 = 81.33 MPa | = 10.17 mm | = 0.493 | = 0.424 | Bending: OK | Deflection: OK -->
```sheet
<!--sheet-->
# Simply supported beam check

A template with inputs. Choose **Input** in the results bar to fill in the form; every change
recalculates the page. The beam is a doubly symmetric I-beam, so one depth $h$ describes it:
flange width $b = h/2$, flange thickness $t_f = h/20$ and web thickness $t_w = h/40$. The check
compares the bending stress $\sigma = M/Z$ with an allowable stress and the mid-span deflection
$\delta = 5 w L^4 / (384 E I)$ with a limit.

## Inputs
L:     ? 6 m [2 .. 12 step 0.5] | span
w:     ? 12 kN/m [2 .. 40 step 1] | applied uniform load
h:     ? [200 mm deep = 200 mm, *300 mm deep = 300 mm, 400 mm deep = 400 mm] | I-beam section
mat:   ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
limit: ? (L/250 = 250, L/360 = 360, L/500 = 500) | deflection limit
sw:    ? on | include the own weight of the beam

## Material properties
The input above picks one entry of each table, so a new material is one more entry in each.

E_tab:   [200 GPa, 70 GPa, 11 GPa] | modulus of elasticity
fa_tab:  [165 MPa, 100 MPa, 12 MPa] | allowable bending stress
rho_tab: [7850 kg/m^3, 2700 kg/m^3, 500 kg/m^3] | density
E:   E_tab.mat -> GPa
f_a: fa_tab.mat -> MPa
rho: rho_tab.mat -> kg/m^3

if mat == 1: "The material is structural steel."
if mat == 2: "The material is aluminium."
if mat == 3: "The material is timber."

## Section
b:   h / 2 | flange width
t_f: h / 20 | flange thickness
t_w: h / 40 | web thickness
A:   2 * b * t_f + (h - 2 * t_f) * t_w -> cm^2 | area
I:   (b * h^3 - (b - t_w) * (h - 2 * t_f)^3) / 12 -> cm^4 | second moment of area
Z:   2 * I / h -> cm^3 | elastic section modulus

## Load
g_acc: 9.81 m/s^2 | gravitational acceleration
w_sw:  sw * rho * A * g_acc -> kN/m | own weight, when included
w_tot: w + w_sw | total uniform load

## Results
M_max: w_tot * L^2 / 8 -> kN*m | maximum bending moment, at mid-span
V_max: w_tot * L / 2 -> kN | maximum shear force, at the supports
sigma: M_max / Z -> MPa | maximum bending stress
delta: 5 * w_tot * L^4 / (384 * E * I) -> mm | mid-span deflection
delta_lim: L / limit -> mm | deflection limit
util_b: sigma / f_a | bending utilisation
util_d: delta / delta_lim | deflection utilisation

if util_b <= 1: "Bending: OK, the stress is within the allowable stress." else: "Bending: the stress exceeds the allowable stress, choose a deeper section or a stronger material."
if util_d <= 1: "Deflection: OK, within the limit." else: "Deflection: over the limit, choose a deeper section or a stiffer material."

## Moment diagram
M(x): w_tot * x * (L - x) / 2 -> kN*m | bending moment along the span

plot M(x * 1 m) / (1 kN*m) for x in 0..L / (1 m)
```

</details>

At the defaults (6 m, 12 kN/m, 300 mm deep, steel, L/250, own weight on):

| Quantity | Result |
| --- | --- |
| Own weight `w_sw` and total load `w_tot` | 0.502 kN/m and 12.5 kN/m |
| Section `A`, `I`, `Z` | 65.25 cm², 10376.4 cm⁴, 691.76 cm³ |
| Moment `M_max`, shear `V_max` | 56.26 kN·m, 37.51 kN |
| Stress `σ` against `f_a` | 81.33 MPa against 165 MPa, utilisation 0.493 |
| Deflection `δ` against `δ_lim` | 10.17 mm against 24 mm, utilisation 0.424 |
| Messages | Bending: OK. Deflection: OK. |

What changing each input does (one change at a time from the defaults, unless the row says
otherwise):

| Change | Result |
| --- | --- |
| Span 8 m | Moment 100.02 kN·m, stress 144.59 MPa (0.876), deflection 32.13 mm against a 32 mm limit. Bending: OK. Deflection: over the limit. |
| Span 8 m and 400 mm deep | Total load 12.89 kN/m, moment 103.15 kN·m, stress 62.9 MPa (0.381), deflection 10.48 mm (0.328). Both OK. |
| Timber | Moment 54.14 kN·m, stress 78.27 MPa against 12 MPa (6.52), deflection 177.89 mm (7.41). Both fail. |
| Limit L/500 and own weight off | Moment 54 kN·m, stress 78.06 MPa (0.473), deflection 9.76 mm against 12 mm (0.813). Both OK. |

The first row shows a deflection utilisation of `1` next to the message "over the limit". The
value is 1.004, and two decimals show it as `1`. The message tests the full number, not the
rounded one. For a closer look, raise the decimals in Settings.

The own-weight checkbox is a multiplier: `w_sw: sw * rho * A * g_acc`. With the box ticked it is
0.502 kN/m, unticked it is 0 and the total load is the applied load alone.

## Worked template 2: a pad footing under a load and a moment

Eight inputs: the column load and moment, the footing's width, length and thickness, the depth of
its base, a soil (a drop-down of allowable bearing pressures) and a checkbox for the weight of
the footing and the soil above it. This is **Template: pad footing bearing pressure**.

<details>
<summary>The whole sheet</summary>

<!-- sample: Template pad footing bearing.sheet -->
<!-- expect: P = N + W _f + W _s = 600 kN + 57.6 kN + 43.2 kN = 700.8 kN | q _max = q _avg · ( 1 + 6 · e L ) = 146 kPa · ( 1 + 6 · 0.0856 m 2.4 m ) = 177.25 kPa | = 0.886 | OK: the largest pressure is within -->
```sheet
<!--sheet-->
# Pad footing: bearing pressure check

A template with inputs. Choose **Input** in the results bar to fill in the form. A rectangular
pad footing $B \times L$ carries a vertical column load $N$ and a moment $M$ about the axis
parallel to $B$. The total vertical load $P$ adds the weight of the footing and of the soil above
it, the eccentricity is $e = M/P$ and, while $e \le L/6$ (no uplift), the largest bearing pressure is
$q_{max} = \frac{P}{B L}\left(1 + \frac{6e}{L}\right)$. Beyond that the pressure acts on part of the
base only, $q_{max} = \frac{2P}{3B(L/2 - e)}$. It is compared with an allowable bearing pressure.

## Inputs
N:    ? 600 kN | vertical column load
M:    ? 60 kN*m | column moment, in the direction of L
B:    ? 2 m | footing width
L:    ? 2.4 m | footing length, in the direction of the moment
t:    ? 0.5 m | footing thickness
D_f:  ? 1 m | depth of the base below the ground
q_a:  ? [Soft clay = 80 kPa, Firm clay = 150 kPa, *Medium dense sand = 200 kPa, Dense gravel = 300 kPa] | soil, allowable bearing pressure
wts:  ? on | include the weight of the footing and of the soil above it

## Constants
gamma_c: 24 kN/m^3 | unit weight of concrete
gamma_s: 18 kN/m^3 | unit weight of the soil above the footing

## Vertical load
W_f: wts * gamma_c * B * L * t -> kN | weight of the footing
W_s: wts * gamma_s * B * L * (D_f - t) -> kN | weight of the soil above the footing
P:   N + W_f + W_s -> kN | total vertical load on the soil

## Bearing pressure
e:   M / P -> m | eccentricity of the load
e_lim: L / 6 | the load stays inside the middle third while e is below this
q_avg: P / (B * L) -> kPa | average pressure

if e <= e_lim:
    q_max: q_avg * (1 + 6 * e / L) -> kPa | largest pressure
    q_min: q_avg * (1 - 6 * e / L) -> kPa | smallest pressure
    The whole base stays in contact with the soil.
else:
    q_max: 2 * P / (3 * B * (L / 2 - e)) -> kPa | largest pressure, partial contact
    q_min: 0 kPa | smallest pressure
    Part of the base lifts off the soil.

util: q_max / q_a | utilisation of the allowable bearing pressure
if util <= 1: "OK: the largest pressure is within the allowable bearing pressure." else: "Fails: the largest pressure exceeds the allowable bearing pressure. Enlarge the footing."
```

</details>

Two things in this sheet are new. The soil drop-down has an option marked `*`:

```text
q_a: ? [Soft clay = 80 kPa, Firm clay = 150 kPa, *Medium dense sand = 200 kPa, Dense gravel = 300 kPa] | soil, allowable bearing pressure
```

so it opens on Medium dense sand, the third option. And the check has two branches. An `if` block
may hold quantity lines and prose, and the branch that applies is the one shown:

```text
if e <= e_lim:
    q_max: q_avg * (1 + 6 * e / L) -> kPa | largest pressure
    ...
    The whole base stays in contact with the soil.
else:
    q_max: 2 * P / (3 * B * (L / 2 - e)) -> kPa | largest pressure, partial contact
    ...
    Part of the base lifts off the soil.
```

At the defaults (600 kN, 60 kN·m, 2 m by 2.4 m, 0.5 m thick, base 1 m down, medium dense sand,
weights on):

| Quantity | Result |
| --- | --- |
| Footing weight `W_f`, soil weight `W_s` | 57.6 kN, 43.2 kN |
| Total vertical load `P` | 700.8 kN |
| Eccentricity `e` against `e_lim` | 0.0856 m against 0.4 m, so the whole base is in contact |
| Average, largest, smallest pressure | 146 kPa, 177.25 kPa, 114.75 kPa |
| Utilisation | 0.886. OK. |

| Change | Result |
| --- | --- |
| Soft clay | Largest pressure unchanged, 177.25 kPa against 80 kPa: utilisation 2.22. Fails. |
| Moment 200 kN·m | `e` 0.285 m, largest 250.17 kPa, smallest 41.83 kPa, utilisation 1.25. Fails. |
| Moment 400 kN·m | `e` 0.571 m is beyond 0.4 m: "Part of the base lifts off the soil." Largest 371.25 kPa, smallest 0 kPa, utilisation 1.86. Fails. |
| Weights off (checkbox) | `P` is 600 kN, `e` 0.1 m, largest 156.25 kPa, utilisation 0.781. OK. |

## Worked template 3: a rectangular column

Seven inputs: the section, the length, the end conditions (a drop-down whose values are the
effective length factors), a material (drop-down), the axial load and a factor of safety. This is
**Template: rectangular column slenderness**. The sheet picks the buckling formula by slenderness:
a short column crushes, an intermediate one uses the parabolic formula, a slender one the Euler
formula.

Here one choice is the number the sheet needs, with no table in between:

```text
k:    ? [Pinned at both ends = 1, Fixed and pinned = 0.7, Fixed at both ends = 0.5, Fixed and free = 2] | end conditions
```

<details>
<summary>The whole sheet</summary>

<!-- sample: Template column slenderness.sheet -->
<!-- expect: = 138.56 | The column is slender: it buckles elastically and the Euler formula applies. | P _allow = P _cr FS = 616.85 kN 2 = 308.43 kN | = 0.324 | OK: the applied load is within the allowed load. -->
```sheet
<!--sheet-->
# Rectangular column: slenderness and buckling

A template with inputs. Choose **Input** in the results bar to fill in the form. A solid
rectangular column of length $L$ with an effective length factor $k$ has a slenderness
$\lambda = k L / r$, where $r = b_{min}/\sqrt{12}$ is the smallest radius of gyration. Above the
transition slenderness $\lambda_c = \pi \sqrt{2E/f_y}$ the column buckles elastically (Euler),
$\sigma_{cr} = \pi^2 E / \lambda^2$. Below it the parabolic (Johnson) formula applies,
$\sigma_{cr} = f_y \left(1 - f_y \lambda^2 / (4 \pi^2 E)\right)$. The load that is allowed is
$\sigma_{cr} A$ divided by a factor of safety.

## Inputs
b:    ? 60 mm | section width
d:    ? 100 mm | section depth
L:    ? 2.4 m | column length
k:    ? [Pinned at both ends = 1, Fixed and pinned = 0.7, Fixed at both ends = 0.5, Fixed and free = 2] | end conditions
mat:  ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
N:    ? 100 kN | axial load
FS:   ? 2 | factor of safety

## Material properties
E_tab:  [200 GPa, 70 GPa, 11 GPa] | modulus of elasticity
fy_tab: [250 MPa, 160 MPa, 24 MPa] | yield strength, or crushing strength for timber
E:  E_tab.mat -> GPa
f_y: fy_tab.mat -> MPa

if mat == 1: "The material is structural steel."
if mat == 2: "The material is aluminium."
if mat == 3: "The material is timber."

## Section
A:   b * d -> cm^2 | area
b_min: min(b, d) | smaller side
I_min: max(b, d) * b_min^3 / 12 -> cm^4 | smaller second moment of area
r:   b_min / sqrt(12) -> mm | smallest radius of gyration

## Slenderness
L_e:      k * L -> m | effective length
lambda:   L_e / r | slenderness ratio
lambda_c: π * sqrt(2 * E / f_y) | transition slenderness

if lambda <= 50:
    The column is short: crushing is likely to govern rather than buckling.
else if lambda <= lambda_c:
    The column is intermediate: it buckles inelastically and the parabolic formula applies.
else:
    The column is slender: it buckles elastically and the Euler formula applies.

## Buckling resistance
sigma_E: π^2 * E / lambda^2 -> MPa | elastic (Euler) buckling stress
P_E:     π^2 * E * I_min / L_e^2 -> kN | elastic (Euler) buckling load

if lambda <= lambda_c:
    sigma_cr: f_y * (1 - f_y * lambda^2 / (4 * π^2 * E)) -> MPa | critical stress, parabolic formula
else:
    sigma_cr: sigma_E | critical stress, Euler formula

P_cr:    sigma_cr * A -> kN | critical load
P_allow: P_cr / FS -> kN | allowed load
util:    N / P_allow | utilisation

if util <= 1: "OK: the applied load is within the allowed load." else: "Fails: the applied load exceeds the allowed load. Enlarge the section, shorten the column or fix its ends."
```

</details>

At the defaults (60 mm by 100 mm, 2.4 m, pinned at both ends, steel, 100 kN, factor of safety 2):

| Quantity | Result |
| --- | --- |
| Area, smaller second moment, radius of gyration | 60 cm², 180 cm⁴, 17.32 mm |
| Effective length `L_e` | 2.4 m |
| Slenderness `λ` against `λ_c` | 138.56 against 125.66: "The column is slender" |
| Critical stress (Euler) and load | 102.81 MPa, 616.85 kN |
| Allowed load | 308.43 kN |
| Utilisation | 0.324. OK. |

| Change | Result |
| --- | --- |
| Fixed at both ends | `L_e` 1.2 m, `λ` 69.28: "The column is intermediate", parabolic formula, critical stress 212 MPa, allowed load 636.01 kN, utilisation 0.157. |
| Fixed at both ends and 300 kN | Utilisation 0.472. OK. |
| Timber | `λ_c` drops to 95.12, still slender, critical stress 5.65 MPa, allowed load 16.96 kN, utilisation 5.9. Fails. |

## Worked template 4: pipe flow

Eight inputs, with a flow rate in L/s, a pipe diameter and length, a rise in level, a sum of loss
coefficients, a pipe material (a drop-down of roughnesses in mm), a water temperature (radio
buttons that pick table entries) and a checkbox for the fittings. This is **Template: pipe flow,
Darcy-Weisbach**. The flow is laminar or turbulent by its Reynolds number, and the sheet uses the
matching friction factor.

<details>
<summary>The whole sheet</summary>

<!-- sample: Template pipe flow Darcy.sheet -->
<!-- expect: The water is at 5 degrees C. | V = Q A = 25 L ∕ s 176.71 cm 2 = 1.41 m ∕ s | The flow is turbulent. | h _L = h _f + h _m = 3.79 m + 0.255 m = 4.05 m | H = dz + h _L = 5 m + 4.05 m = 9.05 m | OK: the velocity is within the usual limit of 3 m/s. -->
```sheet
<!--sheet-->
# Pipe flow: friction loss and pump head

A template with inputs. Choose **Input** in the results bar to fill in the form. Water flows
through a full circular pipe of diameter $D$ and length $L_p$. The Darcy-Weisbach equation gives the
friction head loss $h_f = f \frac{L_p}{D}\frac{V^2}{2g}$. The friction factor is $f = 64/Re$ in laminar
flow and, in turbulent flow, the explicit Swamee-Jain formula
$f = 0.25 / \left[\log_{10}\left(\frac{\varepsilon}{3.7 D} + \frac{5.74}{Re^{0.9}}\right)\right]^2$.
Fittings add $h_m = K V^2 / 2g$.

## Inputs
Q:    ? 25 L/s | flow rate
D:    ? 150 mm | inside diameter of the pipe
L_p:  ? 300 m | pipe length
dz:   ? 5 m | rise in level from the inlet to the outlet
K_m:  ? 2.5 | sum of the loss coefficients of the fittings
eps:  ? [Drawn plastic = 0.0015 mm, *Commercial steel = 0.045 mm, Cast iron = 0.26 mm, Concrete = 1 mm] | pipe material, roughness
temp: ? (5 degrees C = 1, 20 degrees C = 2, 40 degrees C = 3) | water temperature
minor: ? on | include the fittings

## Water properties
rho_tab: [1000 kg/m^3, 998 kg/m^3, 992 kg/m^3] | density at 5, 20 and 40 degrees C
nu_tab:  [0.00000152 m^2/s, 0.00000100 m^2/s, 0.00000066 m^2/s] | kinematic viscosity at 5, 20 and 40 degrees C
rho: rho_tab.temp -> kg/m^3
nu:  nu_tab.temp -> m^2/s
g_acc: 9.81 m/s^2 | gravitational acceleration

if temp == 1: "The water is at 5 degrees C."
if temp == 2: "The water is at 20 degrees C."
if temp == 3: "The water is at 40 degrees C."

## Flow
A:  π * D^2 / 4 -> cm^2 | flow area
V:  Q / A -> m/s | mean velocity
Re: V * D / nu | Reynolds number
rr: eps / D | relative roughness

if Re < 2300:
    The flow is laminar.
    f: 64 / Re | friction factor, laminar
else:
    The flow is turbulent.
    f: 0.25 / (log(rr / 3.7 + 5.74 / Re^0.9))^2 | friction factor, Swamee-Jain

## Head loss
h_f: f * (L_p / D) * V^2 / (2 * g_acc) -> m | friction head loss
h_m: minor * K_m * V^2 / (2 * g_acc) -> m | head loss in the fittings, when included
h_L: h_f + h_m -> m | total head loss
H:   dz + h_L -> m | head the pump must supply
dp:  rho * g_acc * h_L -> kPa | pressure drop from losses
P_h: rho * g_acc * Q * H -> kW | hydraulic power

if V <= 3 m/s: "OK: the velocity is within the usual limit of 3 m/s." else: "High velocity: more than 3 m/s, consider a larger pipe."
```

</details>

Because the first radio option is selected when none is marked, the water starts at 5 degrees C.
At the defaults (25 L/s, 150 mm, 300 m, 5 m rise, `K` 2.5, commercial steel, 5 degrees C,
fittings on):

| Quantity | Result |
| --- | --- |
| Velocity, Reynolds number | 1.41 m/s, 139610: "The flow is turbulent." |
| Friction factor | 0.0186 |
| Friction loss, fittings loss, total | 3.79 m, 0.255 m, 4.05 m |
| Pump head `H` | 9.05 m |
| Pressure drop, hydraulic power | 39.72 kPa, 2.22 kW |
| Message | OK: the velocity is within the usual limit |

| Change | Result |
| --- | --- |
| Water at 20 degrees C (second radio button) | Total loss 3.86 m, `H` 8.86 m, pressure drop 37.8 kPa, power 2.17 kW. |
| Fittings off | Total loss 3.79 m, `H` 8.79 m, power 2.16 kW. |
| Diameter 200 mm | Velocity 0.796 m/s, total loss 1 m, `H` 6 m, power 1.47 kW. |
| Flow 60 L/s | Velocity 3.4 m/s: "High velocity: more than 3 m/s, consider a larger pipe." |
| Flow 0.2 L/s | Reynolds number 1116.88: "The flow is laminar", friction factor 0.0573. |

## Worked template 5: a cantilever with a tip load

The fifth sample, **Template: cantilever with a tip load**, is the beam template's close relative:
length, tip load, a rectangular section, a material and a deflection limit, with the own-weight
checkbox. It ends with a moment diagram.

<details>
<summary>The whole sheet</summary>

<!-- sample: Template cantilever tip load.sheet -->
<!-- expect: M _max = P · L + w · L ^2 2 = 8 kN · 2.5 m + 1.54 kN ∕ m · ( 2.5 m ) ^2 2 = 24.81 kN · m | = 3.69 mm | Bending: OK | Deflection: OK -->
```sheet
<!--sheet-->
# Cantilever with a load at the tip

A template with inputs. Choose **Input** in the results bar to fill in the form. A cantilever of
length $L$ with a solid rectangular section $b \times d$ carries a point load $P$ at the free end
and, if wanted, its own weight $w$. At the fixed end $M = P L + w L^2 / 2$ and
$V = P + w L$, and the tip deflection is $\delta = P L^3 / (3 E I) + w L^4 / (8 E I)$.

## Inputs
L:     ? 2.5 m [0.5 .. 6 step 0.1] | length
P:     ? 8 kN [1 .. 100 step 1] | point load at the free end
b:     ? 100 mm | section width
d:     ? 200 mm | section depth
mat:   ? [Structural steel = 1, Aluminium = 2, Timber = 3] | material
limit: ? (L/90 = 90, L/180 = 180, L/360 = 360) | tip deflection limit
sw:    ? on | include the own weight of the beam

## Material properties
E_tab:   [200 GPa, 70 GPa, 11 GPa] | modulus of elasticity
fa_tab:  [165 MPa, 100 MPa, 12 MPa] | allowable bending stress
rho_tab: [7850 kg/m^3, 2700 kg/m^3, 500 kg/m^3] | density
E:   E_tab.mat -> GPa
f_a: fa_tab.mat -> MPa
rho: rho_tab.mat -> kg/m^3

if mat == 1: "The material is structural steel."
if mat == 2: "The material is aluminium."
if mat == 3: "The material is timber."

## Section
A: b * d -> cm^2 | area
I: b * d^3 / 12 -> cm^4 | second moment of area
Z: b * d^2 / 6 -> cm^3 | elastic section modulus

## Load
g_acc: 9.81 m/s^2 | gravitational acceleration
w:     sw * rho * A * g_acc -> kN/m | own weight, when included

## Results at the fixed end and at the tip
M_max: P * L + w * L^2 / 2 -> kN*m | maximum bending moment, at the fixed end
V_max: P + w * L -> kN | maximum shear force, at the fixed end
sigma: M_max / Z -> MPa | maximum bending stress
delta: P * L^3 / (3 * E * I) + w * L^4 / (8 * E * I) -> mm | tip deflection
theta: P * L^2 / (2 * E * I) + w * L^3 / (6 * E * I) | tip rotation, in radians
delta_lim: L / limit -> mm | tip deflection limit
util_b: sigma / f_a | bending utilisation
util_d: delta / delta_lim | deflection utilisation

if util_b <= 1: "Bending: OK, the stress is within the allowable stress." else: "Bending: the stress exceeds the allowable stress, enlarge the section or change the material."
if util_d <= 1: "Deflection: OK, within the limit." else: "Deflection: over the limit, enlarge the section or change the material."

## Bending moment diagram
M(x): P * (L - x) + w * (L - x)^2 / 2 -> kN*m | bending moment at a distance x from the fixed end

plot M(x * 1 m) / (1 kN*m) for x in 0..L / (1 m)
```

</details>

At the defaults (2.5 m, 8 kN, 100 mm by 200 mm, steel, L/90, own weight on) the moment at the
fixed end is 24.81 kN·m, the shear 11.85 kN, the bending stress 37.22 MPa (0.226) and the tip
deflection 3.69 mm against a 27.78 mm limit (0.133). Both checks pass. Choose timber and the stress
is 30.46 MPa against 12 MPa (2.54) and the tip deflection 57.47 mm (2.07): both fail.

## Worked template 6: a retaining wall

The sixth sample, **Template: retaining wall stability**, checks a cantilever retaining wall (a stem
on a base slab, one metre of wall) for overturning, sliding and bearing pressure under the base.
Its inputs are the geometry, a surcharge on the fill, and the backfill:

```sheet
## Inputs
H_s:   ? 3 m | height of the stem above the base slab
B:     ? 2.4 m | width of the base slab
t_b:   ? 0.4 m | thickness of the base slab
t_s:   ? 0.4 m | thickness of the stem
L_t:   ? 0.6 m | length of the toe, in front of the stem
soil:  ? (Loose sand = 28 °, *Medium sand = 32 °, Dense sand = 36 °) | backfill, friction angle
q_s:   ? 10 kPa | surcharge on the fill, when it is on
sur:   ? off | surcharge on the fill
```

Two things are new here. The radio group `soil` chooses a **friction angle** directly: its values
are angles (`28 °`), and a later line uses the chosen angle as a quantity (`phi: soil`). The
values carry the degree sign, which an input accepts like any other unit. The checkbox `sur` is
used as a multiplier, `q: sur * q_s`: ticked it is 1 and the surcharge counts, unticked it is 0.

<details>
<summary>The whole sheet</summary>

<!-- sample: Template retaining wall stability.sheet -->
<!-- expect: FS _o = M _r M _o = 179.21 kN · m 36.23 kN · m = 4.95 | Overturning: OK. | FS _s = R _h P _h = 49.77 kN 31.97 kN = 1.56 | Sliding: OK. | The resultant is inside the middle third -->
```sheet
<!--sheet-->
# Retaining wall: sliding and overturning

A template with inputs. Choose **Input** in the results bar to fill in the form. A cantilever
retaining wall of concrete (a stem on a base slab) holds back a level fill; the sheet takes a strip of wall one metre long.
The active earth pressure is $K_a = \tan^2(45^\circ - \phi/2)$, acting on the vertical plane through
the heel, so the thrust is $P_a = \tfrac12 K_a \gamma H^2$ at $H/3$ above the underside of the base. A
surcharge $q$ on the fill adds $K_a q H$ at $H/2$ and its weight on the heel. The wall slides on a
base friction of $\tan(2\phi/3)$, and overturns about the toe. The soil in front of the toe is ignored.

## Inputs
H_s:   ? 3 m | height of the stem above the base slab
B:     ? 2.4 m | width of the base slab
t_b:   ? 0.4 m | thickness of the base slab
t_s:   ? 0.4 m | thickness of the stem
L_t:   ? 0.6 m | length of the toe, in front of the stem
soil:  ? (Loose sand = 28 °, *Medium sand = 32 °, Dense sand = 36 °) | backfill, friction angle
q_s:   ? 10 kPa | surcharge on the fill, when it is on
sur:   ? off | surcharge on the fill

## Constants
gamma_c: 24 kN/m^3 | unit weight of concrete
gamma_s: 18 kN/m^3 | unit weight of the fill
b_w:     1 m | length of wall in the strip
FS_min:  1.5 | required factor of safety, for sliding and for overturning

## Geometry
phi: soil | friction angle of the fill
H:   H_s + t_b -> m | height of the thrust plane, above the underside of the base
L_h: B - L_t - t_s -> m | length of the heel, behind the stem
q:   sur * q_s -> kPa | surcharge in use

## Earth pressure
K_a: tan(45 ° - phi / 2)^2 | active pressure coefficient
P_s: K_a * gamma_s * H^2 / 2 * b_w -> kN | thrust of the fill
P_q: K_a * q * H * b_w -> kN | thrust of the surcharge
P_h: P_s + P_q -> kN | total horizontal thrust

## Weights and their lever arms about the toe
W_stem: gamma_c * t_s * H_s * b_w -> kN | stem
x_stem: L_t + t_s / 2 -> m | its lever arm
W_base: gamma_c * B * t_b * b_w -> kN | base slab
x_base: B / 2 -> m | its lever arm
W_fill: gamma_s * L_h * H_s * b_w -> kN | fill on the heel
W_sur:  q * L_h * b_w -> kN | surcharge on the heel
x_heel: B - L_h / 2 -> m | lever arm of the fill and the surcharge
W: W_stem + W_base + W_fill + W_sur -> kN | total vertical load

## Overturning
M_o: P_s * H / 3 + P_q * H / 2 -> kN*m | overturning moment about the toe
M_r: W_stem * x_stem + W_base * x_base + (W_fill + W_sur) * x_heel -> kN*m | resisting moment about the toe
FS_o: M_r / M_o | factor of safety against overturning
if FS_o >= FS_min: "Overturning: OK." else: "Overturning: the factor of safety is too low, widen the base."

## Sliding
mu:   tan(2 * phi / 3) | friction coefficient of the base on the ground
R_h:  mu * W -> kN | sliding resistance
FS_s: R_h / P_h | factor of safety against sliding
if FS_s >= FS_min: "Sliding: OK." else: "Sliding: the factor of safety is too low, widen the base or make it thicker."

## Bearing pressure under the base
x_r: (M_r - M_o) / W -> m | where the resultant crosses the base, from the toe
e:   B / 2 - x_r -> m | eccentricity from the middle of the base
q_max: W / (B * b_w) * (1 + 6 * e / B) -> kPa | pressure at the toe
q_min: W / (B * b_w) * (1 - 6 * e / B) -> kPa | pressure at the heel
if abs(e) <= B / 6: "The resultant is inside the middle third: the whole base is in compression." else: "The resultant is outside the middle third: part of the base lifts off, and the pressures above no longer apply."
```

</details>

At the defaults (medium sand, no surcharge) the horizontal thrust is 31.97 kN and the vertical
load 127.44 kN:

| Quantity | Result |
| --- | --- |
| Active pressure coefficient `K_a` | 0.307 |
| Overturning moment, resisting moment | 36.23 kN·m, 179.21 kN·m |
| Factor of safety against overturning `FS_o` | 4.95: "Overturning: OK." |
| Sliding resistance, friction coefficient `mu` | 49.77 kN, 0.391 |
| Factor of safety against sliding `FS_s` | 1.56: "Sliding: OK." |
| Pressure at the toe, at the heel | 63.46 kPa, 42.74 kPa |
| Message | The resultant is inside the middle third. |

| Change | Result |
| --- | --- |
| Surcharge ticked | `FS_o` 3.76, `FS_s` 1.3: "Sliding: the factor of safety is too low, widen the base or make it thicker." |
| Loose sand | `FS_o` 4.21, `FS_s` 1.15: sliding fails. |
| Dense sand | `FS_o` 5.85, `FS_s` 2.1. Both pass. |
| Base 3 m wide | `FS_o` 7.79, `FS_s` 2.02. Both pass. |

## Worked template 7: a pump and pipeline operating point

The seventh sample, **Template: pump operating point**, finds where a pump works. A pump lifts
water through a pipeline to a higher tank. The pump gives less head as the flow grows, the
pipeline needs more, and the pump runs where the two curves cross. The inputs are a choice of
three pumps (radio buttons), the pipe diameter (a drop-down whose values are lengths), the pipe
length, the static lift, and a checkbox for an aged, rougher pipe:

```sheet
## Inputs
pump: ? (Small = 1, *Medium = 2, Large = 3) | pump
D:    ? [100 mm = 100 mm, *150 mm = 150 mm, 200 mm = 200 mm] | pipe inside diameter
L_p:  ? 400 m | pipe length
H_st: ? 15 m | static lift, from the water surface to the tank
aged: ? off | aged pipe (rougher: the friction factor is 0.030, not 0.018)
```

The pump is an index into two tables (`H0_tab`, `Qmax_tab`), as in the first template. What is new
is the output. The sheet solves the crossing in closed form, tabulates both curves with a vector
of flows and `augment`, and draws them with `plot`, so the form changes the numbers, the table and
the chart together.

<details>
<summary>The whole sheet</summary>

<!-- sample: Template pump operating point.sheet -->
<!-- expect: The pump is the medium one. | Q _op = &emsp;&thinsp; max ( H _net; 0 m ) K + H _0 Q _max^2 = &emsp;&thinsp; max ( 25 m ; 0 m ) 7834.25 s 2 ∕ m 5 + 40 m ( 75 L ∕ s ) ^2 = 40.9 L ∕ s | H _op = H _0 · ( 1 − ( Q _op Q _max ) 2 ) = 40 m · ( 1 − ( 40.9 L ∕ s 75 L ∕ s ) 2 ) = 28.1 m | The velocity is within 3 m/s. -->
```sheet
<!--sheet-->
# Pump and pipeline: operating point

A template with inputs. Choose **Input** in the results bar to fill in the form. A pump lifts water
through a pipeline to a higher tank. The pump delivers the head $H_p = H_0 \left(1 - (Q/Q_{max})^2\right)$
at the flow $Q$ (a shut-off head $H_0$ and a free delivery $Q_{max}$). The pipeline needs the head
$H_s = H_{st} + f \frac{L}{D}\frac{v^2}{2g}$, the static lift plus the friction loss with a constant friction
factor $f$. The pump runs where the two curves cross. The table and the plot follow the form.

## Inputs
pump: ? (Small = 1, *Medium = 2, Large = 3) | pump
D:    ? [100 mm = 100 mm, *150 mm = 150 mm, 200 mm = 200 mm] | pipe inside diameter
L_p:  ? 400 m | pipe length
H_st: ? 15 m | static lift, from the water surface to the tank
aged: ? off | aged pipe (rougher: the friction factor is 0.030, not 0.018)

## Pump curves
The input above picks one entry of each list.

H0_tab:   [30 m, 40 m, 50 m] | shut-off head
Qmax_tab: [60 L/s, 75 L/s, 90 L/s] | free delivery
H_0:   H0_tab.pump -> m
Q_max: Qmax_tab.pump -> L/s
if pump == 1: "The pump is the small one."
if pump == 2: "The pump is the medium one."
if pump == 3: "The pump is the large one."

## Pipeline
g_acc: 9.81 m/s^2 | gravitational acceleration
f: 0.018 + aged * 0.012 | friction factor
A: pi * D^2 / 4 -> m^2 | flow area
K: f * L_p / D / (2 * g_acc * A^2) -> s^2/m^5 | friction head per flow squared

## Operating point
The two curves cross where $H_0 - H_0 Q^2/Q_{max}^2 = H_{st} + K Q^2$, so
$Q^2 = (H_0 - H_{st}) / (K + H_0/Q_{max}^2)$.

H_net: H_0 - H_st -> m | head the pump has above the static lift
if H_net <= 0 m: "The pump cannot lift the water to the tank: choose a larger pump or lower the static lift."
Q_op: sqrt(max(H_net, 0 m) / (K + H_0 / Q_max^2)) -> L/s | flow at the operating point
H_op: H_0 * (1 - (Q_op / Q_max)^2) -> m | head at the operating point
v:    Q_op / A -> m/s | velocity in the pipe
if v > 3 m/s: "The velocity is above 3 m/s: expect noise and wear, use a larger pipe." else: "The velocity is within 3 m/s."

## Table
Q_v:  [0, 10, 20, 30, 40, 50, 60] * 1 L/s | flows in the table
H_pv: H_0 * (1 - (Q_v / Q_max)^2) -> m | pump head
H_sv: H_st + K * Q_v^2 -> m | head the pipeline needs
T: augment(Q_v / (1 L/s), H_pv / (1 m), H_sv / (1 m)) | columns: flow in L/s, pump head in m, pipeline head in m

## Curves
H_p(Q): H_0 * (1 - (Q / Q_max)^2) -> m | pump head at the flow Q
H_s(Q): H_st + K * Q^2 -> m | pipeline head at the flow Q

plot H_p(x * 1 L/s) / (1 m), H_s(x * 1 L/s) / (1 m) for x in 0..60
```

</details>

At the defaults (medium pump, 150 mm, 400 m, 15 m lift, new pipe) the pump runs at 40.9 L/s and
28.1 m of head, with 2.31 m/s in the pipe. The table has seven rows, from 0 to 60 L/s: at 40 L/s
the pump gives 28.62 m and the pipeline needs 27.53 m, and at 50 L/s the pump gives 22.22 m
against 34.59 m, so the crossing lies between them.

| Change | Result |
| --- | --- |
| Small pump | 30.46 L/s at 22.27 m |
| Large pump | 49.99 L/s at 34.58 m, 2.83 m/s |
| Aged pipe | 35.21 L/s at 31.19 m |
| Diameter 100 mm | 19.37 L/s at 37.33 m, 2.47 m/s |
| Static lift 35 m | 18.29 L/s at 37.62 m |
| Small pump and static lift 35 m | "The pump cannot lift the water to the tank: choose a larger pump or lower the static lift." The flow is 0. |

## Share a template

A template is one text file. Save it (Ctrl+S) as a `.sheet` file and send that file, or paste the
text of the sheet. Whoever opens it gets the same form, in any browser, with nothing to install.
Once the app has been visited it runs offline too (see
[Saving, opening and working offline](getting-started/saving.md)). Nothing is sent to a server:
the numbers are worked out on the reader's own computer.

* The values you saved with **Save values** travel with the file, so a template can be sent already
  filled in. Send the file without that comment line to send the defaults.
* Each person who opens it and saves values keeps their own numbers in their own copy.

## Writing your own

1. Write the calculation as an ordinary sheet and check it.
2. Turn the quantities someone will want to change into inputs: put `?` in front of the value.
3. Put them together under a heading (`## Inputs`), so the form reads like a form.
4. Put the results below the inputs, with a note on every line.
5. Where a result depends on a choice, use an index and tables (worked template 1), or let the
   choice be the number itself (worked template 3).
6. Add a message for each check (`if util <= 1: "OK" else: "Fails"`), so that the report says what
   the numbers mean.
7. Test it in Input mode, then in Report with the defaults. That is what a reader without the form
   sees.

The ribbon's Input/Output tab inserts a starting line for each kind: **Entry**, **Dropdown**,
**Radio** and **Checkbox** write the `?` form in a sheet. **Data grid** and **More input forms**
(labelled, vector, matrix and formula inputs) have no sheet form and insert `#UI` lines, marked
with `>>`. See [Input forms with `#UI`](calcpad/ui-inputs.md).

## Model bindings

A `'<!--civilkit: bind L = path -->` comment before an input can tie it to a quantity in a
structural model. The specification and the in-app marker exist; no model is connected yet, so
typed values are used.

**Related calculators:** [Simply supported beam](https://civilkit.io/calcs/learn/simply-supported-beam-calculator/), [Retaining wall stability](https://civilkit.io/calcs/learn/retaining-wall-stability-calculator/), [Pad footing bearing pressure](https://civilkit.io/calcs/learn/pad-footing-bearing-pressure-calculator/), [Pipe flow, Darcy-Weisbach](https://civilkit.io/calcs/learn/pipe-flow-darcy-weisbach-calculator/), [Pump operating point](https://civilkit.io/calcs/learn/pump-operating-point-calculator/). More pages are in [Calculators and guides](https://civilkit.io/calcs/learn/).
