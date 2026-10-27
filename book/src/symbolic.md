# Symbolic maths

<p><strong>Video:</strong> Symbolic maths for a beam deflection</p>
<video controls preload="none" style="width:100%;max-width:960px" src="videos/07-symbolic-beam-deflection.mp4"></video>

The calculation engine is numeric: it gives you numbers. The **Symbolic** dialog gives you
formulas. It rearranges an equation, differentiates, integrates, solves for the constants of
integration and substitutes values, all *exactly*, with fractions kept as fractions and symbols kept
as symbols. Then you put the result into the sheet and let the engine check it with numbers.

Open it from **Functions > Symbolic** on the ribbon. The module loads the first time you run
something, and it works offline.

## Using the dialog

1. Type an expression, or an equation with `=`. Write `*` for every product and `^` for powers.
   The functions are `sin`, `cos`, `ln`, `exp` and `sqrt`; the constants are `pi` and `e`.
   Enter runs it; Shift+Enter starts a new line.
2. Choose the operation:

| Operation | What it needs | What you get |
| --- | --- | --- |
| Simplify | the expression | like terms collected, powers combined |
| Expand | the expression | products and powers of sums multiplied out |
| Factor | the expression | common factors, and polynomials in one variable |
| Differentiate | a variable, and an order (1 by default) | the derivative |
| Integrate | a variable, and for a definite integral both limits | an antiderivative (with no constant), or the value between the limits |
| Solve for | one variable for one equation (write `=`), or several variables separated by commas for a linear system whose equations are separated by `;` or new lines | the solutions; a quadratic gives both roots |
| Substitute | values such as `x = 2, b = 3*a` | the exact result |
| Evaluate | a number for every variable | a decimal number to 12 digits |

3. The result is typeset (with KaTeX) and also shown as text, with a **Copy** button.
4. **Insert into worksheet** writes the result at the cursor:
   * in a **sheet**: a display formula block (`$$ ... $$`, typeset in the preview) or a quantity line
     `name: expression`;
   * in a **Calcpad worksheet**: a calculation line `name = expression`, or a comment line.

   A quantity line is calculated by the engine, so every symbol in it needs a value in the sheet.
   Use the formula block to keep a result in terms of symbols.

## A worked derivation: the deflection of a simply supported beam

The beam has span L, uniform load w and stiffness EI. The bending moment at x is
M(x) = w L x / 2 - w x² / 2, and the deflection y, taken in the direction of the load, satisfies
EI y'' = -M(x). These are the dialog's results for each step (the inputs are in the first column).

| Step | Operation and input | Result |
| --- | --- | --- |
| 1 | **Integrate** `-(w*L*x/2 - w*x^2/2)/(E*I)` for `x` | `-L*w*x^2/(4*E*I) + w*x^3/(6*E*I)` |
| 2 | Add a constant `C1` yourself and **Integrate** again: `-(w*L*x^2/4 - w*x^3/6)/(E*I) + C1` for `x` | `-L*w*x^3/(12*E*I) + w*x^4/(24*E*I) + C1*x` |
| 3 | Add `C2`. The supports give y(0) = 0 and y(L) = 0, so **Solve for** `C1, C2` the equations `C2 = 0; -L^4*w/(12*E*I) + L^4*w/(24*E*I) + C1*L + C2 = 0` | `C1 = L^3*w/(24*E*I); C2 = 0` |
| 4 | **Substitute** `x = L/2` into the curve `-L*w*x^3/(12*E*I) + w*x^4/(24*E*I) + L^3*w*x/(24*E*I)` | `5*L^4*w/(384*E*I)` |

The midspan deflection is 5 w L⁴ / (384 E I). That is the formula to put in the sheet. The dialog
can also **differentiate** it (`5*L^3*w/(96*E*I)` with respect to L), or **evaluate** it for given
numbers: with `w = 12`, `L = 6`, `E = 200` and `I = 250` the result is `0.00405` (in consistent
units of your choice).

## Putting the result into a sheet

In the sheet, write the derivation as prose and display formulas, then give the engine the numbers
and check the formula against the curve:

```sheet
## Step 4: midspan

Substituting $x = L/2$ into the curve and simplifying gives

$$y\left(\frac{L}{2}\right) = \frac{5 L^4 w}{384 E I}$$

## Numbers

w: 12 kN/m | design load
L: 6 m | span
E: 200 GPa | modulus of elasticity
I: 250000000 mm^4 | second moment of area
delta: 5 * w * L^4 / (384 * E * I) -> mm | midspan deflection from the formula

y(x): -L * w * x^3 / (12 * E * I) + w * x^4 / (24 * E * I) + L^3 * w * x / (24 * E * I) | the curve with its constants
y_mid: y(L / 2) -> mm | the curve at midspan
ratio: y_mid / delta | 1 when the formula and the curve agree
```

```text
w = 12 kN/m    design load
L = 6 m    span
E = 200 GPa    modulus of elasticity
I = 250000000 mm⁴    second moment of area
δ = 5·w·L⁴/(384·E·I) = 5 · 12 kN/m · (6 m)⁴ / (384 · 200 GPa · 250000000 mm⁴) = 4.05 mm
y_mid = y(L/2) = y(6 m / 2) = 4.05 mm    the curve at midspan
ratio = y_mid / δ = 4.05 mm / 4.05 mm = 1
```

The two routes agree: the exact formula from the dialog and the curve with its constants give the
same 4.05 mm. The sample **Beam deflection derived symbolically** (Samples, group Sheet syntax)
is this sheet in full, with the deflection curve plotted:

```sheet
L: 6 m
w: 12 kN/m
E: 200 GPa
I: 250000000 mm^4
y(x): -L * w * x^3 / (12 * E * I) + w * x^4 / (24 * E * I) + L^3 * w * x / (24 * E * I)
plot -y(x * 1 m) / (1 mm) for x in 0..6
```

(The plot divides by `1 mm` to give the plot a plain number, and flips the sign so the beam sags
downward on the page.)

## Limits

The algebra is exact rational arithmetic, so a result that does not fit in 128 bits is an error
rather than a rounded guess, and a refusal is shown in words with the library's own message
("Cannot integrate: power of a non-linear base."). Nothing is guessed. The limits:

* no complex numbers;
* only `sin`, `cos`, `ln`, `exp` and `sqrt`;
* integration covers a small class of integrands;
* solving covers a single equation of degree 1 or 2, and linear systems of up to four unknowns;
* symbols are treated as positive real numbers, so `sqrt(x^2)` is `x`.

For anything else, use the numeric tools: `$Root`, `$Area`, `$Slope`, `lsolve` and friends.
