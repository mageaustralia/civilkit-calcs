# Conditions and loops

<!-- description: Conditions and loops in a sheet: one-line if checks that print OK or a message, blocks of lines, for and while loops, and their limits. -->

## One-line conditions

```sheet
utilisation: 0.82
if utilisation <= 1: "OK" else: "Fails"
```

The branches are a quoted text or quantities. The text may contain LaTeX. A one-line condition
can also define a quantity: `if a == 2: b: 1`.

## Blocks

A line ending in `:` that starts with `if`, `else if`, `else`, `for`, `while` or `repeat` opens a
block. The body is the lines indented more than the header. A tab counts as four spaces. Blank
lines inside are fine. A line indented no deeper than the header closes the block. A body can hold
anything: quantities, prose, formulas, `>>` lines, and nested blocks.

```sheet
n: 5
if n > 3:
    Large.
else if n > 1:
    Medium.
else:
    Small.
```

```text
n = 5
Large.
```

## Loops

```sheet
for i in 1..3:
    P: 10 kN * i | load at stage $i$
```

```text
P = 10 kN · i = 10 kN · 1 = 10 kN    load at stage 1
P = 10 kN · i = 10 kN · 2 = 20 kN    load at stage 2
P = 10 kN · i = 10 kN · 3 = 30 kN    load at stage 3
```

| Sheet | Meaning |
| --- | --- |
| `for i in a..b:` | `i` takes a, a+1, ... b. |
| `for i in a..b step s:` | A stepped range. The step may be negative. |
| `for x in v:` | Each element of the vector `v` (a name or a literal). |
| `while c:` | Repeat while the condition holds. |
| `repeat n:` | Repeat n times. |
| `break`, `continue` | Leave the loop, or skip to the next pass. `if c: break` works on one line. |

```sheet
for i in 0..10 step 5:
    s: i
```

```sheet
for x in [2, 4, 6]:
    y: x^2
```

```sheet
n: 3
while n > 0:
    n: n - 1
```

```sheet
total: 0
for i in 1..5:
    total: total + i
    if total > 5: break
```

The last example stops once the total passes 5; it prints the passes with total 1, 3 and 6.

A quantity line in a loop prints on every pass, exactly like a Calcpad loop. Use the loop to build
a vector or to accumulate; if you only need the final value, put the quantity line after the loop.

## Limits

* `repeat` needs a count; for anything else use `>>` lines.
* A loop over a vector cannot have a step.
* A header whose condition is not an expression (`If the load is large:`) is prose. The compiler
  warns if it starts with a keyword.
* An `else` with no matching `if` at the same indentation is read as text, with a warning.
