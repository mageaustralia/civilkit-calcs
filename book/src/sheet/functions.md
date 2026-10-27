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
