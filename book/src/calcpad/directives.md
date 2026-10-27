# Directives and commands

A directive starts with `#` and controls the flow of the worksheet. A command starts with `$` and
takes arguments in braces. In a sheet, both are written on `>>` lines (or inside a `calcpad`
fence), except the ones the sheet has its own form for (`if`, `for`, `while`, `repeat`, `plot`).

## Conditions

```cpd
M = 150kN*m
#if M < 100kN*m
    'OK
#else if M < 200kN*m
    'Check again
#else
    'Too large
#end if
```

The condition can be any expression that gives a number: zero is false. A condition that cannot
be evaluated (an undefined name, say) is an error, and the block is left unresolved: no branch
runs. CalcpadCE would silently take the `#else` branch; see
[Deviations from CalcpadCE](../appendix/deviations.md).

## Loops

```cpd
#repeat 3
    y = 2
#loop
```

```cpd
#for i = 1 : 4
    #if i ≡ 3
        #continue
    #end if
    s = i
#loop
```

```text
s = i = 1
s = i = 2
s = i = 4
```

Pass 3 is skipped by `#continue`.

```cpd
n = 0
#while n < 3
    n = n + 1
#loop
```

`#break` leaves the loop and `#continue` skips to the next pass. Every line in the loop prints on
every pass, so use `#hide` around the body when you only want the final value. The commands
`$Sum`, `$Product`, `$Repeat` and `$While` do the same without printing each pass.

## Macros and string variables

Macros are text substituted before the worksheet is calculated. A name ends with `$`.

```cpd
#def twice$(a$) = 2*a$
r = twice$(7)
```

```text
r = 2·7 = 14
```

A multi-line macro repeats a whole block with different values:

```cpd
#def beam$(span$; load$)
    M = load$*span$^2/8
    V = load$*span$/2
#end def
beam$(6m; 12kN/m)
```

```text
M = 12 kN/m · (6 m)² / 8 = 9 kN·m
V = 12 kN/m · 6 m / 2 = 36 kN
```

The macro text does not have to be valid Calcpad by itself. It is substituted first and parsed
after. **Unwrapped** in the results views shows the text after substitution, which is the place to
look when a macro misbehaves. A macro cannot contain another definition; it can use an earlier
one.

## Modules

`#include name.cpd` pulls in a file from the library (see [Saving](../getting-started/saving.md)).
Put shared definitions in a file with a `#global` section and keep the worksheet-specific part in
`#local`: only the global part is included when the file is imported.

## Constants

```cpd
#const g_n = 9.81m/s^2
F = 70kg*g_n|N
```

A `#const` cannot be changed later.

## Number format

`#round n` rounds output to n digits, `#round default` goes back; `#format spec` sets a custom
format string (for example `#format 0.00`). The **Decimals** setting sets the default.
`#settings {"decimals": 4}` overrides engine settings for the lines that follow.

## Text modes

`#md on` and `#md off` turn Markdown on and off in comment lines. `#markdown` ... `#end markdown`
and `#html` ... `#end html` switch to a block of Markdown or raw HTML, and `#cpd` goes back to
calculation inside one.

```cpd
#markdown
## Section

* one
* two
#end markdown
```

## Reading and writing data

`#read M from file.csv`, `#write M to file.csv` and `#append M to file.csv` move a matrix
between the worksheet and a text, CSV or Excel file in the library. Options choose the cell range
(`@R1C1:R2C2` for text, `@Sheet1!A1:B2` for Excel), the matrix type and the separator. The full
forms are in the [reference](../reference/functions.md#directives).

A worksheet that writes files saves them into the library and downloads them when their content
changes.

## Numerical commands

```cpd
#rad
f(x) = x^3 - 2*x - 5
r = $Root{f(x) @ x = 2 : 3}
A = $Area{sin(x) @ x = 0 : π}
m = $Slope{x^3 @ x = 2}
hi = $Sup{sin(x) @ x = 0 : 3}
S = $Sum{1/k^2 @ k = 1 : 100}
P = $Product{k @ k = 1 : 5}
```

```text
r = $Root{f(x) = 0; x ∈ [2; 3]} = 2.09
A = ∫ sin(x) dx from 0 to π = 2
m = d/dx x³ at x = 2 = 12
hi = $Sup{sin(x); x ∈ [0; 3]} = 1
S = Σ 1/k² for k = 1 to 100 = 1.63
P = Π k for k = 1 to 5 = 120
```

| Command | Meaning |
| --- | --- |
| `$Root{f(x) @ x = a : b}` | A root of f in the interval. `$Root{f(x) = c @ ...}` solves f(x) = c. |
| `$Find{f(x) @ x = a : b}` | An approximate solution, when the exact value is not needed. |
| `$Sup`, `$Inf` | Local maximum or minimum. |
| `$Area` | Numerical integral (adaptive Gauss-Lobatto). |
| `$Integral` | Numerical integral (tanh-sinh). |
| `$Slope{f(x) @ x = a}` | Derivative at a point. |
| `$Sum`, `$Product` | Iterative sum and product. |
| `$Repeat`, `$While`, `$Block`, `$Inline` | Iterative expression blocks. |
| `$Plot`, `$Map` | Plots; see [Plots](../plots.md). |

The relative precision of the numerical methods is the `Precision` setting (default 1e-12).

## Python

`#python` ... `#end python` is a Python block. In a sheet it is a fenced `python` block. See
[Python blocks](../python.md).

## Input forms

`#UI` turns an assignment into an input control. See [Input forms with `#UI`](ui-inputs.md). In a sheet, most inputs are written as `L: ? 6 m` lines instead; see [Templates with inputs](../templates.md).

## Not available in the browser

`#pause` and `#input` (step-by-step execution) are not supported in the web version, and neither
are raster (PNG) plots or compiled `.cpdz` worksheets. See
[Deviations from CalcpadCE](../appendix/deviations.md).
