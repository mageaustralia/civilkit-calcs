# Video 7: Symbolic maths for a beam deflection

* File to record: `07-symbolic-beam-deflection.mp4`
* Length: about 90 seconds
* Book page: [Symbolic maths](../src/symbolic.md)
* Screen: as in video 1. The dialog is centred, so zoom the video in on it during each scene.

## Before recording

1. New sheet. Type the title line `# Beam deflection derived symbolically` and a sentence under it:
   `Midspan deflection of a simply supported beam, derived with the Symbolic dialog.`
2. Leave the cursor on a new line at the end, so each **Insert into worksheet** lands there.
3. Open **Functions** on the ribbon, to be ready for the **Symbolic** button.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | The sheet with its title. Caption: "From a formula to a number". | None. | S1 |
| 0:10 to 0:28 | Click **Symbolic**. The dialog opens. Operation: Integrate. Variable: x. Expression: box 1. The typeset result appears. Click **Insert into worksheet**: a display formula lands in the sheet. | Type box 1 below. | S2 |
| 0:28 to 0:42 | Same again with box 2. The result gains a term with the constant C1. | Type box 2. | S3 |
| 0:42 to 0:58 | Operation: Solve for. Variables: C1, C2. The equations from box 3. The answer: C1 is L cubed w over twenty-four E I, and C2 is zero. | Type box 3. | S4 |
| 0:58 to 1:12 | Operation: Substitute. Values: x = L/2. Expression from box 4. The result: five L to the fourth w over three hundred and eighty-four E I. Insert it. | Type box 4. | S5 |
| 1:12 to 1:30 | Close the dialog. Type the numbers block. The result shows four point zero five millimetres from the formula and from the curve, and a ratio of one. End card: "Exact algebra, in your browser". | Type the numbers block. | S6 |

## What to type in the dialog

| Box | Operation | Variable or values | Expression box |
| --- | --- | --- | --- |
| 1 | Integrate | variable `x` | `-(w*L*x/2 - w*x^2/2)/(E*I)` |
| 2 | Integrate | variable `x` | `-(w*L*x^2/4 - w*x^3/6)/(E*I) + C1` |
| 3 | Solve for | variables `C1, C2` | `C2 = 0; -L^4*w/(12*E*I) + L^4*w/(24*E*I) + C1*L + C2 = 0` |
| 4 | Substitute | values `x = L/2` | `-L*w*x^3/(12*E*I) + w*x^4/(24*E*I) + L^3*w*x/(24*E*I)` |

Results the dialog gives:

```text
box 1:  -L*w*x^2/(4*E*I) + w*x^3/(6*E*I)
box 2:  -L*w*x^3/(12*E*I) + w*x^4/(24*E*I) + C1*x
box 3:  C1 = L^3*w/(24*E*I); C2 = 0
box 4:  5*L^4*w/(384*E*I)
```

## What to type in the sheet

After the dialog has put the formulas in, add the numbers:

```sheet
# Beam deflection derived symbolically

Midspan deflection of a simply supported beam, derived with the Symbolic dialog.

$$y\left(\frac{L}{2}\right) = \frac{5 L^4 w}{384 E I}$$

w: 12 kN/m | design load
L: 6 m | span
E: 200 GPa | modulus of elasticity
I: 250000000 mm^4 | second moment of area
delta: 5 * w * L^4 / (384 * E * I) -> mm | midspan deflection from the formula

y(x): -L * w * x^3 / (12 * E * I) + w * x^4 / (24 * E * I) + L^3 * w * x / (24 * E * I) | the curve with its constants
y_mid: y(L / 2) -> mm | the curve at midspan
ratio: y_mid / delta | 1 when the formula and the curve agree
```

The results at the end:

```text
δ = 5·w·L⁴/(384·E·I) = ... = 4.05 mm    midspan deflection from the formula
y_mid = y(L/2) = y(6 m / 2) = 4.05 mm    the curve at midspan
ratio = y_mid / δ = 4.05 mm / 4.05 mm = 1
```

## Narration

S1: A formula in, a number out. In between, the algebra. Here is how to derive the midspan deflection of a simply supported beam.

S2: Open the Symbolic dialog. Integrate the bending equation with respect to x. The answer is exact, and typeset. Insert it into the sheet.

S3: Integrate again, and add the constant of integration by hand. The result now has a term in C one.

S4: Now solve for the constants. The supports give two equations. The constant C two is zero, and C one is L cubed w over twenty-four E I.

S5: Substitute L over two for x. The famous result appears. Five L to the fourth w, over three hundred and eighty-four E I. Insert it too.

S6: Back in the sheet, give it numbers. The formula and the curve agree, at four point zero five millimetres. The algebra is exact, and it runs in your browser.

## Full text for the voice (one block)

A formula in, a number out. In between, the algebra. Here is how to derive the midspan deflection of a simply supported beam. Open the Symbolic dialog. Integrate the bending equation with respect to x. The answer is exact, and typeset. Insert it into the sheet. Integrate again, and add the constant of integration by hand. The result now has a term in C one. Now solve for the constants. The supports give two equations. The constant C two is zero, and C one is L cubed w over twenty-four E I. Substitute L over two for x. The famous result appears. Five L to the fourth w, over three hundred and eighty-four E I. Insert it too. Back in the sheet, give it numbers. The formula and the curve agree, at four point zero five millimetres. The algebra is exact, and it runs in your browser.

## Light marketing beat

* Scene 6: exact algebra, in the browser.

## Check after rendering

* Say "C one" and "C two" as the two constants appear.
* If the dialog shows the typeset result slowly the first time (KaTeX loads), cut the wait.
