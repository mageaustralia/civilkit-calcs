# The Calcpad language

The sheet format is a front end. Underneath, every sheet becomes a **Calcpad worksheet**, the
language of CalcpadCE, and the engine runs that. You can also write Calcpad directly: choose
**File > New Calcpad worksheet** (the second item of the File menu, also on the Resources tab), or
open a `.cpd` file.

Use the Calcpad language when you need something a sheet does not offer on its own: macros that
repeat a block with different arguments, input forms with drop-downs and grids, output control
(hide a block, show only results), the numerical commands, reading and writing data files, or
when you are working with worksheets from the CalcpadCE community.

## How a Calcpad worksheet reads

A worksheet is a list of lines. Each line is one of these.

| First character | Kind | Example |
| --- | --- | --- |
| `"` | Heading, a comment shown as a heading | `"Beam check` |
| `'` | Text, a comment shown as written (HTML allowed) | `'Span -'L = 6m` |
| `#` | A directive | `#if`, `#for`, `#def`, `#hide` |
| `$` | A command | `$Plot{x^2 @ x = 0 : 5}` |
| anything else | A calculation | `M = w*L^2/8\|kN*m` |

A single quote also switches between text and calculation inside a line, which is how labels
sit in front of an equation: `'Span -'L = 6m` shows the text "Span -" and then the equation.

```cpd
"Beam check
'Span -'L = 6m
'Load -'w = 12kN/m
'Moment -'M = w*L^2/8|kN*m
#if M < 100kN*m
    'OK
#else
    'Too large
#end if
```

```text
Beam check
Span - L = 6 m
Load - w = 12 kN/m
Moment - M = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m
OK
```

## Sheet and Calcpad side by side

| Sheet | Calcpad |
| --- | --- |
| `L: 6 m \| span` | `L = 6m 'span` |
| `M: w*L^2/8 -> kN*m` | `M = w*L^2/8\|kN*m` |
| `# Heading` | `'<h1>Heading</h1>` |
| `Plain paragraph` | `'Plain paragraph` |
| `if u <= 1: "OK" else: "Fails"` | `#if u ≤ 1` ... `#else` ... `#end if` |
| `for i in 1..5:` + indented body | `#for i = 1 : 5` ... `#loop` |
| `f(x, y): x + y` | `f(x; y) = x + y` |
| `[1, 2; 3, 4]` | `[1; 2 \| 3; 4]` |
| `plot f(x) for x in 0..5` | `$Plot{f(x) @ x = 0 : 5}` |
| `phi`, `Mstar` | `φ`, `M_star` |

Function arguments are separated with `;` in Calcpad, matrix rows with `|`, and a unit
conversion is a vertical bar. A sheet writes commas, `->` and brackets, and the compiler does the
translation. To see it for any sheet, use **Show compiled**.

## Operators

`^` power, `*` multiplication, `/` division, `\` integer division, `⦼` modulo (type `%%`), `!`
factorial, `+ - `, comparisons `≡ ≠ < > ≤ ≥` (type `== != <= >=`), logical `∧ ∨ ⊕` (type `&& || `),
`=` assignment, `←` assignment to an outer scope. The editor converts the typed forms when you
open a file. The complete table is in the [reference](../reference/functions.md#operators).

## Angles

Angles are in **degrees** unless you say otherwise. `#rad` switches the rest of the worksheet to
radians, `#gra` to gradians and `#deg` back. This matters whenever you pass π or a plain number to
a trigonometric function. See [Troubleshooting](../troubleshooting.md).

## Output control

Directives shape what the report shows without changing the calculation.

```cpd
#hide
k = 3
#show
a = k*2
#val
b = k*3
#equ
#nosub
c = k*4
#varsub
```

```text
a = k·2 = 3 · 2 = 6
9
c = k·4 = 12
```

`#hide` ... `#show` calculates but does not print, `#val` prints only the value, `#nosub` prints
the formula without the numbers put in, `#noc` the formula without the result and `#novar` only
the substituted numbers. `#pre` and `#post` split a document into the part that is filled in on
screen and the part that appears in the report. The complete list is in
[Directives](directives.md).

## What the next pages cover

* [Directives and commands](directives.md): conditions, loops, macros, modules, constants,
  settings, files, numerical commands.
* [Templates with inputs](../templates.md): `#UI` forms.
