# Vectors and matrices

<!-- description: Vectors and matrices in a sheet: write them with brackets, do arithmetic, solve and decompose, find eigenvalues and read or write data files. -->

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

## Eigenvalues and eigenvectors

```sheet
K: [4, -2; -2, 3]
lam: eigenvals(K)
V: eigenvecs(K)
E: eigen(K)
```

```text
lam = eigenvals(K) = [1.44 5.56]
V = eigenvecs(K) = | -0.615 -0.788 |
                   | -0.788  0.615 |
E = eigen(K) = | 1.44 -0.615 -0.788 |
               | 5.56 -0.788  0.615 |
```

`eigenvals(M)` returns the eigenvalues of a square matrix in ascending order.
`eigenvecs(M)` returns one unit eigenvector per **row**, in the same order, and `eigen(M)` gives
both at once with the eigenvalue in the first column. A second argument keeps only some of them:
`eigenvals(M, 2)` is the two smallest and `eigenvals(M, -1)` the largest. Units carry over, so a
stress tensor in MPa has principal stresses in MPa.

The matrix does not have to be symmetric:

<!-- expect: eigenvals(A) = [2 5] | eigenvals(A; -1) = [5] -->
```sheet
A: [4, 1; 2, 3]
lam: eigenvals(A)
V: eigenvecs(A)
lmax: eigenvals(A, -1)
```

```text
lam = eigenvals(A) = [2 5]
V = eigenvecs(A) = | -0.447 0.894 |
                   |  0.707 0.707 |
lmax = eigenvals(A; -1) = [5]
```

A symmetric matrix uses the symmetric solver. Any other square matrix is reduced to Hessenberg form
and solved with the shifted QR iteration; each eigenvector is scaled to unit length with its largest
component positive.

### Complex eigenvalues

A real matrix that is not symmetric can have complex eigenvalues, which a vector cannot hold.
`eigenvals` then stops with `The matrix has complex eigenvalues.` Use `ceigenvals` and
`ceigenvecs`, which write each complex number as a real part and an imaginary part, the same
layout `fft` uses:

<!-- expect: ceigenvals(R) = 1 1 -2 2 -->
```sheet
R: [1, -2; 2, 1]
C: ceigenvals(R)
W: ceigenvecs(R)
```

```text
C = ceigenvals(R) = |  1 1 |
                    | -2 2 |
W = ceigenvecs(R) = | 0.707     0 |
                    |     0 0.707 |
                    | 0.707     0 |
                    |     0 -0.707 |
```

Column k of `ceigenvals` is eigenvalue k: row 1 is the real part and row 2 the imaginary part,
so the eigenvalues above are 1 - 2i and 1 + 2i. In `ceigenvecs` eigenvector k is row 2k - 1
(real parts) plus row 2k (imaginary parts): the first one is [0.707, 0.707i]. They are sorted by
real part, then imaginary part.

For the polar form, magnitude and angle, use the two rows:

```sheet
R: [1, -2; 2, 1]
C: ceigenvals(R)
mag: sqrt(row(C, 1)^2 + row(C, 2)^2)
ang: atan2(row(C, 1), row(C, 2))
```

```text
mag = [2.24 2.24]
ang = atan2(row(C; 1); row(C; 2)) = [-63.43 63.43]
```

In a Calcpad document with complex numbers switched on in Settings, `#phasor` shows every
complex result as magnitude∠angle, and `#complex` switches back to a + bi.

### Natural frequencies: K x = λ M x

Give `eigenvals`, `eigenvecs` or `eigen` a second matrix and they solve the generalized problem
K x = λ M x. With a stiffness matrix K and a mass matrix M, each λ is ω², the square of a natural
circular frequency:

<!-- expect: eigenvals(K; M) = [585.79 3414.21] | = 3.85 Hz | row(Φ; 1)) = 1 -->
```sheet
k: 20000 kN/m
m: 10 t
K: [2*k, -k; -k, k]
M: [2*m, 0 t; 0 t, m]
λ: eigenvals(K, M)
f_1: sqrt(λ.1)/(2*π) -> Hz
Phi: eigenvecs(K, M)
m_1: dot(row(Phi, 1), M*row(Phi, 1))
```

```text
λ = eigenvals(K; M) = [585.79 3414.21] s⁻²
f_1 = 3.85 Hz
Φ = eigenvecs(K; M) = |  0.158 0.224 | t^-0.5
                      | -0.158 0.224 |
m_1 = 1
```

K must be symmetric and M symmetric positive definite, both square and the same size; anything
else stops with a message that says which matrix is wrong. The eigenvalues come in ascending order
with the units of K/M (kN/m over t is s⁻²). Each mode shape is a row of `eigenvecs(K, M)`, scaled so
that xᵀ M x = 1 and turned so that its largest component is positive; its units are those of
M^-0.5, so the modal mass `m_1` above is exactly 1. Divide a row by one of its components to scale
it to 1 at that point instead. Internally M is factored by Cholesky (M = L Lᵀ), the problem is
turned into the symmetric one of L⁻¹ K L⁻ᵀ, solved with the symmetric solver, and transformed back.
To keep only the lowest modes, take them from the result: `first(eigenvals(K, M), 3)`.

The samples *Principal stresses* (a plane and a 3D stress tensor, checked with Mohr's circle) and
*Shear frame natural frequencies* (the frequencies and mode shapes of a two storey frame from
`eigenvals(K, M)`) use these functions.

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

**Related calculators:** [Principal stresses](https://civilkit.io/calcs/learn/principal-stress-calculator/), [Principal axes of a section](https://civilkit.io/calcs/learn/section-principal-axes-calculator/), [Shear frame frequencies](https://civilkit.io/calcs/learn/shear-frame-natural-frequencies/). More pages are in [Calculators and guides](https://civilkit.io/calcs/learn/).
