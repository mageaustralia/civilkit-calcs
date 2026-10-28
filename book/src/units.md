# Units

<p><strong>Video:</strong> Units and results</p>
<video controls preload="none" poster="videos/02-units-and-results.jpg" style="width:100%;max-width:960px" src="videos/02-units-and-results.mp4"></video>

Units are part of the value. The engine carries them through every operation, converts between
them, and stops with an error when a formula does not add up dimensionally.

## Attaching units

Write the unit after a number, with or without a space: `15 kg`, `12kN/m`, `1230000 mm³`.

```sheet
a: 1.23 m + 35 cm + 12 mm -> cm
v: 10 m/s -> km/h
P: 2.5 kN / (50 mm * 200 mm) -> MPa
g0: 9.81 m/s²
m: 70 kg
W: m * g0 -> N
```

```text
a = 1.23 m + 35 cm + 12 mm = 159.2 cm
v = 10 m/s = 36 km/h
P = 2.5 kN / (50 mm · 200 mm) = 0.25 MPa
g0 = 9.81 m/s²
m = 70 kg
W = m·g0 = 70 kg · 9.81 m/s² = 686.7 N
```

Without `->`, a sum comes out in the first unit it meets. With `-> unit` the result is converted
to the unit you choose. In the Calcpad language the same conversion is a vertical bar:
`a = 1.23m + 35cm + 12mm | cm`.

Prefixes work on every unit that takes them: `kN` is 10³ N, `MPa` is 10⁶ Pa, `mm` is 10⁻³ m.

## Errors you want

```sheet
x: 6 m
t: 2 s
speed: x / t
```

```text
speed = x/t = 6 m / 2 s = 3 m/s
```

Dividing a length by a time is fine. Adding them is not:

```text
x: 6 m
t: 2 s
speed: x / t
len: x + t

Error in "len = x + t" on line [4]: Inconsistent units: "m + s".
```

That is the engine doing its job. A unit slip is the most common mistake in hand calculations, and
here it stops the sheet and names the line.

## Which units exist

The built-in list is long. The groups are:

| Group | Examples |
| --- | --- |
| Dimensionless | `%`, `‰`, `ppm`, angles `°`, `rad`, `grad`, `rev` |
| Mass | `g`, `kg`, `t`, `mg`, `lb`, `oz`, `slug` |
| Length | `m`, `km`, `mm`, `μm`, `in`, `ft`, `yd`, `mi`, `nmi` |
| Time and frequency | `s`, `ms`, `min`, `h`, `d`, `Hz`, `rpm` |
| Force | `N`, `kN`, `MN`, `kgf`, `lbf`, `kip` |
| Pressure | `Pa`, `kPa`, `MPa`, `bar`, `atm`, `psi`, `ksi` |
| Energy and power | `J`, `kJ`, `Wh`, `kWh`, `cal`, `BTU`, `W`, `kW`, `hp` |
| Area and volume | `a`, `ha`, `ac`, `L`, `mL`, `gal`, `bbl` |
| Temperature | `K`, `°C`, `°F`, `°R` |
| Electrical | `A`, `V`, `Ω`, `F`, `H`, `C`, `S`, `T`, `Wb` |
| Other | `mol`, `cd`, `lm`, `lx`, `Bq`, `Gy`, `Sv`, `kat` |

The Insert tab in the sidebar lists all 400 or so, grouped, and a click inserts one. The Math
tab has a **Units** palette organised by dimension.

### Angles

Angle units are accepted by trigonometric functions and override every other setting.
`sin(30 deg)` is 0.5 whatever the angle setting is. A bare number follows the setting: degrees by
default, changed on the Math Formatting tab (`#deg`, `#rad`, `#gra` in Calcpad). Inverse
trigonometric functions return a plain number unless you define `ReturnAngleUnits = 1`
(Calcpad line) first:

```cpd
a = asin(0.5)
ReturnAngleUnits = 1
b = asin(0.5)
```

```text
a = asin(0.5) = 30
ReturnAngleUnits = 1
b = asin(0.5) = 30°
```

### Temperatures

`20 °C` is a temperature; `300 K` is another, and the difference of two temperatures is a
difference:

```sheet
T1: 20 °C
T2: 300 K
d: T2 - T1
```

```text
d = T2 − T1 = 300 K − 20 °C = 280 K
```

### US and UK names

A few bare names mean different things in the US and UK systems (`gal`, `ton`, `pt`, `qt`, `bbl`,
`cwt`, `therm`). The setting **Non-metric units** picks one. Write the explicit suffix
(`gal_US`, `gal_UK`) when a sheet must not depend on the setting.

## Percentages

A ratio has no unit, so it comes out as a plain number. Convert it with `-> %` to show it as a
percentage, and write `%` after a number to enter one. `‰` (per mille) works the same way.

<!-- expect: 355.1 kN = 0.691 | util = ratio = 0.691 = 69.07 % | f = Ed · r = 245.25 kN · 4 % = 9.81 kN -->
```sheet
Ed: 245.25 kN
Rd: 355.1 kN
ratio: Ed / Rd
util: ratio -> %
r: 4%
f: Ed * r
slope: 1 / 40 -> ‰
```

```text
ratio = Ed / Rd = 245.25 kN / 355.1 kN = 0.691
util = ratio = 0.691 = 69.07 %
r = 4 %
f = Ed · r = 245.25 kN · 4 % = 9.81 kN
slope = 1 / 40 = 25 ‰
```

The value itself does not change, only the way it is shown: `util` is the same number as
`ratio`, and `r` is 0.04 in every formula. A result in percent is rounded to the decimals set on the
Math Formatting tab like any other result, and the rounded figure typed back in (`69.07%`) is the
same ratio to that many digits. An input works the same way: `rate: ? 12.5 %` shows a box holding
12.5 with `%` after it, and the name is 0.125 in the formulas.

A percentage times a plain number is a plain number (`4 % · 2` is `0.08`); add `-> %` to the line
to keep it as a percentage.

## Your own units

A sheet defines a unit of its own with a `#unit` line: the name, `=`, and its value in units
the engine knows or in units defined above it. Later lines use it after a number and after `->`,
like any other unit.

<!-- expect: bag = 25 kg cement bag | load = 3 pallet = 3 t | n = 1000 kg = 40 bag -->
```sheet
#unit bag = 25 kg | cement bag
#unit pallet = 40 bag
load: 3 pallet -> t
n: 1000 kg -> bag
rate: 6 bag / 1 m³ -> kg/m³
```

```text
bag = 25 kg   cement bag
pallet = 40 bag
load = 3 pallet = 3 t
n = 1000 kg = 40 bag
rate = 6 bag / 1 m³ = 150 kg/m³
```

A unit name is letters and `_` only (`bag`, `ton_short`), with no digits; a name with a digit is an
error on its line. The value can use the usual spellings (`m²`, `kN·m`) and can have a note after
`|`. A built-in unit cannot be redefined: `#unit slug = 14.6 kg` is an error, because `slug` is
already a unit. In the Calcpad language, and in a `>>` line of a sheet, the same unit is a line
with a dot in front of the name: `.bag = 25kg`.

Currency and information units are defined as dimensionless units, the first as 1 and the others
as multiples:

```sheet
#unit USD = 1
#unit AUD = 0.65 USD
price: 200 AUD -> USD
```

```text
USD = 1
AUD = 0.65 USD
price = 200 AUD = 130 USD
```

Two families of dimensionless custom units in one sheet share a dimension, so do not mix them.

## Units and names

A word that follows a number immediately is a unit (`2 m`). A standalone word is a variable if you
defined one with that name, and a unit otherwise. Put a dot in front to force a unit even when a
variable has the same name. In a sheet this rarely matters, but it explains one surprise: if you
name a variable `m` and later write `5 m`, the `m` after the number is still the unit.

## Removing and reading units

The functions `getunits(x)`, `clrunits(x)` and `setunits(x, u)` read, strip and attach units. They
are in the [function reference](reference/functions.md).

```sheet
q: setunits(5, kN)
F: 5 kN
n: F / 1 kN
```

```text
q = setunits(5; kN) = 5 kN
F = 5 kN
n = F / 1 kN = 5 kN / 1 kN = 5
```

Use `clrunits` sparingly. A value without a unit is a value the engine can no longer check.

## Units in vectors and matrices

A vector or matrix whose elements all carry the same unit shows the unit once after the
bracket. See [Vectors and matrices](vectors-matrices.md).
