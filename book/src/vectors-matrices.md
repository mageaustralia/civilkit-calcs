# Vectors and matrices

## Writing them

In a sheet, a bracket makes a vector or a matrix. Elements are separated by commas or semicolons
and rows by semicolons:

| You write | You get |
| --- | --- |
| `[1, 2, 3]` or `[1; 2; 3]` | a vector with three elements |
| `[1, 2; 3, 4]` | a 2 by 2 matrix: columns separated by commas, rows by semicolons |

Rows must have equal length and elements can be expressions. (In the Calcpad language the same
matrix is `[1; 2 | 3; 4]`, with `|` between rows.)

```sheet
v: [1, 2, 3]
A: [2, -1; -1, 2]
b: [1; 0]
x: lsolve(A, b)
d: det(A)
e: v.2
f: A.(1, 2)
```

```text
v = [1; 2; 3] = [1 2 3]
A = [2; -1 | -1; 2] = | 2 -1 |
                      | -1 2 |
b = [1; 0] = [1 0]
x = lsolve(A; b) = [0.667 0.333]
d = det(A) = 3
e = v.2 = 2
f = A.(1; 2) = -1
```

An element is reached with a dot: `v.2` is the second element of the vector `v`, and `A.(1, 2)` is
row 1, column 2 of `A`. Indexes start at 1.

Vector and matrix names show an arrow over a vector in the report.

## Arithmetic

```sheet
A: [1, 2; 3, 4]
B: A * A
C: A + 2
D: A - [1, 1; 1, 1]
At: transp(A)
v: [1; 2]
Av: A * v
r: rank(A)
tr: trace(A)
```

```text
B = A·A = | 7 10 |
          | 15 22 |
C = A + 2 = | 3 4 |
            | 5 6 |
Av = A·v = [5 11]
rank(A) = 2     trace(A) = 5
```

A matrix times a matrix is the matrix product. A matrix plus or minus a number applies to every
element. A power applies to every element of a vector (`x^2` above squares each element), and
`hprod(A, B)` is the element-by-element product of two matrices.

## Solving and decomposing

```sheet
K: [4, -2; -2, 3]
F: [10; 0]
u: lsolve(K, F)
```

```text
u = lsolve(K; F) = [3.75 2.5]
```

`lsolve(A, b)` solves A x = b with an LDLᵀ factorisation for a symmetric matrix and an LU
factorisation otherwise. `clsolve` uses Cholesky and `slsolve` an iterative method for large
sparse systems. `msolve` solves for several right-hand sides at once. `inverse`, `det`, `rank`,
`cond`, `lu`, `qr`, `svd`, `cholesky`, `eigenvals`, `eigenvecs` and `eigen` are there too.

The eigenvalue functions need a matrix of the *symmetric* type. A literal is a general matrix,
so copy it into a symmetric one first:

```sheet
K: [4, -2; -2, 3]
Ks: copy(K, symmetric(2), 1, 1)
lam: eigenvals(Ks)
V: eigenvecs(Ks)
```

```text
lam = eigenvals(Ks) = [1.44 5.56]
V = eigenvecs(Ks) = | -0.615 -0.788 |
                    | -0.788  0.615 |
```

Without the copy the engine reports `Matrix must be symmetric.`, even for a matrix that is
symmetric as written.

## Building vectors and matrices

```sheet
x: range(0, 10, 2.5)
y: x^2
s: sum(y)
mx: max(y)
w: linspace(0, 1, 5)
```

```text
x = range(0; 10; 2.5) = [0 2.5 5 7.5 10]
y = x ^ 2 = [0 6.25 25 56.25 100]
s = sum(y) = 187.5
mx = max(y) = 100
w = linspace(0; 1; 5) = [0 0.25 0.5 0.75 1]
```

Constructors: `vector(n)`, `matrix(m, n)`, `identity(n)`, `diagonal(n, d)`, `column(m, c)`,
`utriang(n)`, `ltriang(n)`, `symmetric(n)`, `join_cols`, `join_rows`, `augment`, `stack`.
Pieces: `row`, `col`, `submatrix`, `slice`, `first`, `last`, `extract`, `diag2vec`. Sorting and
searching: `sort`, `rsort`, `order`, `find`, `lookup`, `hlookup`, `vlookup`. Statistics and
interpolation: `average`, `median`, `stdev`, `var`, `take`, `line`, `spline`, `interp`, `linterp`,
`polyfit`.

```sheet
A2: [1, 2, 3; 4, 5, 6]
r1: row(A2, 1)
c2: col(A2, 2)
sm: submatrix(A2, 1, 2, 2, 3)
```

```text
r1 = row(A2; 1) = [1 2 3]
c2 = col(A2; 2) = [2 5]
sm = submatrix(A2; 1; 2; 2; 3) = | 2 3 |
                                 | 5 6 |
```

The whole list is in the [function reference](reference/functions.md). The large high-performance
variants (`matrix_hp` and so on) are for big numerical models: they store the numbers more
compactly and are faster, and they behave like the ordinary ones in a calculation.

## Units in vectors and matrices

A vector or matrix whose elements all have the same unit shows the unit once, after the closing
bracket:

```sheet
P: [16 ft; 12 ft]
Q: [3 m; 40 cm]
```

```text
P = [16; 12] ft = [16 12] ft
Q = [3 m; 40 cm] = [3 0.4] m
```

Elements in another unit of the same dimension are converted to the first element's unit in the
result. An element that is an expression, has no unit, or has another dimension keeps its own
unit. To write the unit on every element, put a `>>` line in the sheet:

```sheet
>> #settings {"unitPerElement": true}
P: [16 ft; 12 ft]
```

You cannot write a unit after a bracket (`[1, 2] kN`). Put it on the elements.

## Reading and writing files

`#read` brings a matrix in from a text, CSV or Excel file in the library, and `#write` and
`#append` save one. They are Calcpad directives, so in a sheet they are `>>` lines:

```text
>> #read M from loads.csv
>> #write M to results.csv
```

Add the files to the library first (File > Open, or drag them onto the page). The options for
cell ranges and separators are in the [directive list](reference/functions.md#directives).
