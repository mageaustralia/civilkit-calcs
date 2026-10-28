# Your first sheet in five minutes

<p><strong>Video:</strong> Your first calculation in 5 minutes</p>
<video controls preload="none" poster="../videos/01-first-calculation.jpg" style="width:100%;max-width:960px" src="../videos/01-first-calculation.mp4"></video>

This walk-through builds a bending check for a simply supported beam. You will type a few lines,
see units work, add a pass or fail message and change a number.

## 1. Open a new sheet

Open https://civilkit.io/calcs/ . The first tab is a short welcome sheet. To start your own, use
**File > New sheet**, or the **New** button at the top left. A sheet opens with a few lines of
example text. Select all of it and delete it. The editor is on the left and the result is on the
right.

> A sheet is a plain text file. Its first line is `<!--sheet-->`, which the editor adds for you.

## 2. Type the inputs

Type these lines. Each one is a *quantity line*: a name, a colon, a value with a unit, and after
a bar `|` a note.

```sheet
# Beam check

A simply supported beam under a uniform load.

L: 6 m | span
w: 12 kN/m | design load
```

The result pane shows the heading, the sentence, and two lines like these:

```text
L = 6 m    span
w = 12 kN/m    design load
```

The result updates as you type. If it does not, press **Calculate** on the Calculation tab, or Ctrl+Alt+X.

## 3. Calculate with units

Add a line below the inputs:

```sheet
L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
```

The result shows the formula, the numbers put in, and the answer:

```text
M⋆ = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m    maximum bending moment
```

Three things happened:

* `Mstar` was written as M with a subscript "star". A name that ends in `star` after a capital
  letter becomes a subscript. Greek names work the same way: `phi` is φ, `sigma_c` is σ with a
  subscript c.
* `^` is a power. `L^2` is L squared.
* `-> kN*m` asked for the answer in kilonewton metres. Without it the engine picks a unit
  itself, which here is also kN·m. Write `->` when you want to be sure which unit you get, or a
  different one (`-> N*mm`).

## 4. Add a capacity and a check

Now add the section capacity and a utilisation ratio. This is the complete sheet:

```sheet
# Beam check

A simply supported beam under a uniform load. The moment **comes out in kN·m** because the
engine tracks the units.

L: 6 m | span
w: 12 kN/m | design load
Mstar: w * L^2 / 8 -> kN*m | maximum bending moment
fy: 300 MPa | yield stress
Ze: 1230000 mm³ | section modulus
Ms: fy * Ze -> kN*m | moment capacity
utilisation: Mstar / (0.9 * Ms)
if utilisation <= 1: "OK" else: "Fails"
```

The result ends with:

```text
Ms = fy·Ze = 300 MPa · 1230000 mm³ = 369 kN·m    moment capacity
utilisation = M⋆ / (0.9·Ms) = 54 kN·m / (0.9 · 369 kN·m) = 0.163
OK
```

Look at the `Ms` line. A stress in megapascals times a volume in cubic millimetres came out as a
moment in kilonewton metres. You did not convert anything. The `utilisation` line divides a
moment by a moment, so the units cancel and it is a plain number.

The last line is a one-line condition. It prints the text `OK` when the ratio is at most 1 and
`Fails` otherwise.

## 5. Change a number

Change `w: 12 kN/m` to `w: 80 kN/m`. The moment becomes 360 kN·m, the ratio 1.08, and the last
line reads `Fails`. Everything after the changed line recalculates as you type.

## 6. See what the sheet compiles to

A sheet is turned into the Calcpad worksheet language, and the engine runs that. To see the
translation, open the **Resources** tab and choose **Show compiled**. For the sheet above it is:

```text
'<h1>Beam check</h1>

#markdown
A simply supported beam under a uniform load. The moment **comes out in kN·m** because the
engine tracks the units.
#end markdown

L = 6 m
w = 12 kN/m
M_star = w * L^2 / 8|kN*m
fy = 300 MPa
Ze = 1230000 mm^3
Ms = fy * Ze|kN*m
utilisation = M_star / (0.9 * Ms)
#if utilisation ≤ 1
'OK
#else
'Fails
#end if
```

(In the compiled view each note is kept as a comment at the end of its line.) You can read it as
a short lesson in the Calcpad language: `|kN*m` is the conversion, `#if` ... `#end if` is the
condition. The language is covered in [The Calcpad language](../calcpad/index.md). **Convert to
Calcpad**, next to Show compiled, replaces the sheet with that text, for when you want to leave
the sheet format for good.

## 7. Save it

Press Ctrl+S (Cmd+S on a Mac). In browsers that can write files, you choose a place and the sheet
is saved as a `.sheet` file. In other browsers the file downloads. See
[Saving, opening and working offline](saving.md).

## Where next

* All the line kinds: [The sheet format](../sheet/index.md).
* Why the units came out right: [Units](../units.md).
* Draw the moment along the beam: [Plots](../plots.md).
* Turn the sheet into a form you fill in: [Templates with inputs](../templates.md).
