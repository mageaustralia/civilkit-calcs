# Functions

<!-- description: Define your own functions in a sheet, call the built-in function library, and use numerical commands such as root finding and integration. -->

## Your own functions

A line with an argument list defines a function:

```sheet
f(x, y): sqrt(x^2 + y^2)
g(t): 2 * t
r: f(3, 4) + g(1)
```

```text
f(x; y) = √(x² + y²)
g(t) = 2·t
r = f(3; 4) + g(1) = 7
```

Calls take commas in a sheet; the compiler writes the engine's `;`. Calcpad's own `;` works too,
in a definition and in a call (`f(x; y): x + y`, `max(3; 7)`): inside brackets it means the same as
a comma. Functions can use any quantity defined above them, and may be used in plots
(`plot g(t) for t in 0..5`).

### Keeping functions for reuse

A function defined in a sheet belongs to that sheet. To use it in other sheets, put its definition
in a `.cpd` file and add the file to the library (**File** > Library > Add files to the library,
or drop it on the page). Any sheet then pulls it in with an include line:

```text
>> #include beam_functions.cpd
```

The file holds Calcpad text, so its arguments are separated by `;`:

```text
delta(w; L; E; I) = 5*w*L^4/(384*E*I)
```

Two things to watch in definitions that integrate over time or space:

* The limits of `$Area`, `$Sum` and the other numerical commands must have the same unit.
  `$Area{F(t) @ t = 0ms : 2s}` is fine; a bare `0` against `2s` is an error.
* An angular frequency is a number per second. Write `20 / 1 s`, not `20 rad/s`.

Copying the definition lines into each sheet also works, and keeps the sheet self-contained when you
send it to someone who does not have your library.

## Built-in functions

The engine has more than 200 functions: trigonometric, hyperbolic, logarithmic, rounding,
statistical, interpolation, complex numbers, and a large set for vectors and matrices. The
[function reference](../reference/functions.md) lists them all, with the argument forms, and the
Insert tab in the sidebar finds them as you type.

A few that are used constantly:

```sheet
a: sqrt(2)
b: round(3.14159)
c: max(3, 7, 5)
d: sum([1, 2, 3])
e: interp(2.5, [1, 2, 3], [10, 20, 30])
```

A function whose name is also a variable name needs the Calcpad spelling (a name like `gamma`
or `beta` used as a variable only keeps working if you write it the Calcpad way). Function calls
are not renamed unless the sheet defines the function.

## Solving, integrating and slopes

Three numerical operations have a sheet form of their own, written like a `plot` line:

<!-- expect: = 1.41 the positive root | d x = 9 | x = 3 = 6 -->
```sheet
f(x): x^2
x0: solve f(x) = 2 for x in 0..2 | the positive root
A: integral of f(x) for x in 0..3
k: slope of f(x) at x = 3
```

* `solve f(x) = c for x in a..b` finds the `x` between `a` and `b` where both sides are equal
  (Calcpad's `$Root{f(x) = c @ x = a : b}`). Without `= c` it solves `f(x) = 0`. Both sides may
  use the variable (`solve f(x) = g(x) for x in 0..5` finds where two curves cross). The difference
  of the sides must change sign once across the range.
* `integral of f(x) for x in a..b` is the numerical integral (`$Integral`).
* `slope of f(x) at x = a` is the derivative at a point (`$Slope`).

`root(27, 3)` is something else: the cube root of 27. The bounds may carry units, and a bare `0`
takes the unit of the other bound, so `solve x^2 = 4 m^2 for x in 0..5 m -> mm` gives 2000 mm. A
line that starts like one of these but is malformed (`for 2 in 0..5`, two `=` signs) is an error on
its line.

## Numerical commands

The `$Root`, `$Area`, `$Sum`, `$Find` and similar commands are Calcpad commands. Apart from the three
above, in a sheet they go through `>>`:

```sheet
>> f(x) = x^2 - 2
>> x_0 = $Root{f(x) @ x = 0 : 2}
>> I = $Area{x^2 @ x = 0 : 3}
```

The result of the first is 1.41, of the second 9. See [The Calcpad language](../calcpad/index.md).
