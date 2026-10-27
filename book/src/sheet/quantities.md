# Quantities, units and notes

A quantity line defines a name:

```text
name: expression [-> unit] [| note]
name = expression
a: 1; b: 2 | note for b
```

The colon and the equals sign both work. The result shows what an equation line shows in
Calcpad: the name, the formula, the numbers put in, and the answer with its unit.

## Names

A name is made of letters, digits, `_` and primes. Greek spellings become Greek letters:

```sheet
phi: 0.9
sigma_c: 25 MPa
Delta_T: 40 K
phiMs: phi * 369 kN*m
Mstar: 54 kN*m
```

```text
φ = 0.9
σ_c = 25 MPa
Δ_T = 40 K
φMs = φ · 369 kN·m = 0.9 · 369 kN·m = 332.1 kN·m
M_star = 54 kN·m
```

* `phi` is φ, `sigma_c` is σ with a subscript c, `Delta_T` is Δ with a subscript T.
* A Greek spelling followed by a capital or a digit (`phiMs`, `sigma2`) is the letter and the
  rest.
* Capital Greek letters are written only where they differ from a Latin capital: `Gamma`,
  `Delta`, `Theta`, `Lambda`, `Xi`, `Pi`, `Sigma`, `Upsilon`, `Phi`, `Psi`, `Omega`.
* A name that ends in `star` after a capital or a digit (`Mstar`) is that name with the subscript
  `star`.

The same rule renames every use of a name, so you spell it one way throughout the sheet. Inside
`>>` lines and `calcpad` fences the sheet's spelling is used too: `phi` in a raw line is φ.

## Units

Write a unit after a number. The engine checks and combines them:

```sheet
a: 2 m; b: 350 mm
A: a * b -> m^2
v: 90 km/h -> m/s
t: 2 h + 30 min -> min
F: 10 N * 3 m -> J
```

```text
a = 2 m
b = 350 mm
A = a·b = 2 m · 350 mm = 0.7 m²
v = 90 km/h = 25 m/s
t = 2 h + 30 min = 150 min
F = 10 N · 3 m = 30 J
```

* `²` and `³` after a name or unit are `^2` and `^3`. `mm³` and `mm^3` are the same.
* A unit written after a number is never renamed, so `3 psi` is a pressure and `psi` alone would
  be the Greek letter ψ.
* Adding a length to a time is an error. Multiplying and dividing them is fine.
* Several units in a sum convert to the first: `5 m + 30 cm` is `5.3 m`.

The whole subject, with the unit list and custom units, is in [Units](../units.md).

## Converting the answer: `->`

`-> unit` converts the result. It must come last, before the note:

```sheet
Mstar: 12 kN/m * (6 m)^2 / 8 -> kN*m | maximum bending moment
Mnmm: Mstar -> N*mm
```

```text
M_star = 12 kN/m · (6 m)² / 8 = 54 kN·m    maximum bending moment
Mnmm = M_star = 54 kN·m = 54000000 N·mm
```

Without `->` the engine chooses the unit itself, from the units in the formula. Use `->` when you
want a particular unit in the result, and always when the result goes into a report.

## Notes

The first `|` outside brackets starts a note, a short comment shown beside the result in a muted
style. `||` is the logical *or*, not a note. A note can contain LaTeX (`| load at stage $i$`).

## Several per line

Separate quantities with `;`. A note belongs to the last one:

```sheet
a: 3; b: 4 | note for b
c: sqrt(a^2 + b^2)
```

```text
a = 3
b = 4    note for b
c = √(a² + b²) = √(3² + 4²) = 5
```

## Numbers and operators

* You can write `1.23e6` or `2.5e-3`. The engine has no exponent notation, so the compiler writes
  the plain number (`1230000`, `0.0025`) and that is what the result shows.
* Operators are `+ - * / ^ %` and the comparisons `< > <= >= == !=`, with `and` and `or`. The
  Unicode forms `≤ ≥ ≠ ≡ ∧ ∨ × · ÷ −` work too. In a condition a single `=` also means equal.
* Calls take commas: `max(a, b)`. The compiler writes the semicolon the engine wants.
* Use `abs(x)` for an absolute value, not bars.
* `π` (or `pi`) and `e` are built in. The other constants in the Insert tab (gravity, the speed of
  light, unit weights and so on) are snippets that insert a definition line such as
  `g = 9.80665m/s^2`; until you insert one, `g` is the unit gram. See the
  [reference](../reference/functions.md#constants).

## Vectors and matrices

```sheet
v: [1, 2, 3]
A: [2, -1; -1, 2]
```

`[1, 2, 3]` is a vector; `[1, 2; 3, 4]` is a matrix with columns separated by commas and rows by
semicolons. See [Vectors and matrices](../vectors-matrices.md).

## Inputs: a quantity the reader can change

Put a `?` in front of the value and the quantity becomes an input, with that value as its default:

```sheet
L: ? 6 m | span
w: ? 12 kN/m | design load
M: w * L^2 / 8 -> kN*m | maximum moment
```

In Input mode `L` and `w` are boxes. Everywhere else the lines read as the quantities they are. A
list of options (`[Steel = 200 GPa, Timber = 11 GPa]`, or round brackets for radio buttons) and
`on` or `off` (a checkbox) are inputs too. See [Templates with inputs](../templates.md).
