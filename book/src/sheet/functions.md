# Functions

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

Calls take commas in a sheet; the compiler writes the engine's `;`. Functions can use any
quantity defined above them, and may be used in plots (`plot g(t) for t in 0..5`).

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

## Numerical commands

The `$Root`, `$Area`, `$Sum`, `$Find` and similar commands are Calcpad commands. In a sheet
they go through `>>`:

```sheet
>> f(x) = x^2 - 2
>> x_0 = $Root{f(x) @ x = 0 : 2}
>> I = $Area{x^2 @ x = 0 : 3}
```

The result of the first is 1.41, of the second 9. See [The Calcpad language](../calcpad/index.md).
