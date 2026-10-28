# Function and command reference

This page is generated from the editor's own snippet library (`book/gen-reference.py`), so it
lists what the Insert tab and the ribbon offer. In the Calcpad language, arguments are separated
with `;`, as shown here. In a sheet, write commas: `max(a, b)`.

Names are case sensitive. Greek letters, subscripts (`x_1`) and primes can be part of a name.

## Constants

`π` (also `pi`) and `e` are built in. The others are snippets that insert a definition line; until the line is in the worksheet the name means whatever it meant before (`g` is the unit gram).

| Form | Meaning |
| --- | --- |
| `π` | Pi - ratio of circumference to diameter |
| `pi` | Pi - ratio of circumference to diameter (spelled out) |
| `e` | Euler's number - base of natural logarithm |
| `φ = 1.618033988749894` | Golden ratio |
| `γ = 0.5772156649015328` | Euler-Mascheroni constant |
| `g = 9.80665m/s^2` | Standard gravitational acceleration |
| `G = 6.67430*10^-11*(m^3/(kg*s^2))` | Gravitational constant |
| `M_E = 5.9722*10^24*kg` | Earth mass |
| `M_S = 1.98847*10^30*kg` | Solar mass |
| `c = 299792458m/s` | Speed of light in vacuum |
| `h = 6.62607015*10^-34*J/Hz` | Planck constant |
| `μ_0 = 1.25663706212*10^-6*N/A^2` | Vacuum permeability |
| `ε_0 = 8.8541878128*10^-12*F/m` | Vacuum permittivity |
| `k_e = 8.9875517923*10^9*N*m^2/C^2` | Coulomb constant |
| `e = 1.602176634*10^-19*C` | Elementary charge |
| `m_e = 9.1093837015*10^-31*kg` | Electron mass |
| `m_p = 1.67262192369*10^-27*kg` | Proton mass |
| `m_n = 1.67492749804*10^-27*kg` | Neutron mass |
| `N_A = 6.02214076*10^23*mol^-1` | Avogadro constant |
| `σ = 5.670374419*10^-8*W*m^-2*K^-4` | Stefan-Boltzmann constant |
| `k_B = 1.380649*10^-23*J/K` | Boltzmann constant |
| `R = 8.31446261815324*(J/(mol*K))` | Molar gas constant |
| `F = 96485.33212331C/mol` | Faraday constant |
| `γ_c = 25kN/m^3` | Unit weight of concrete |
| `γ_s = 78.5kN/m^3` | Unit weight of steel |
| `γ_a = 27kN/m^3` | Unit weight of aluminium |
| `γ_g = 25kN/m^3` | Unit weight of glass |
| `γ_w = 10kN/m^3` | Unit weight of water |

## Operators

Where a typed alternative exists it is given; the editor converts it when a file is opened.

| Form | Meaning |
| --- | --- |
| `!` | Factorial |
| `^` | Exponent (power) |
| `/` | Division |
| `÷` | Division bar (inline) / slash (pro mode) |
| `\` | Integer division |
| `⦼` | Modulo (remainder) (type `%%`) |
| `*` | Multiplication |
| `-` | Subtraction / Negation |
| `+` | Addition |
| `≡` | Equal to (type `==`) |
| `≠` | Not equal to (type `!=`) |
| `<` | Less than |
| `>` | Greater than |
| `≤` | Less than or equal (type `<=`) |
| `≥` | Greater than or equal (type `>=`) |
| `∧` | Logical AND (type `&&`) |
| `∨` | Logical OR (type `\|\|`) |
| `⊕` | Logical XOR |
| `∠` | Phasor angle operator A∠φ (type `<<`) |
| `=` | Assignment |
| `←` | Outer scope assignment (type `<*`) |

## Functions

### Trigonometric

| Form | Meaning |
| --- | --- |
| `sin(x)` | Sine |
| `cos(x)` | Cosine |
| `tan(x)` | Tangent |
| `csc(x)` | Cosecant |
| `sec(x)` | Secant |
| `cot(x)` | Cotangent |

### Inverse Trigonometric

| Form | Meaning |
| --- | --- |
| `asin(x)` | Inverse sine (arc sine) |
| `acos(x)` | Inverse cosine (arc cosine) |
| `atan(x)` | Inverse tangent (arc tangent) |
| `atan2(a; b)` | Two-argument arc tangent |
| `acsc(x)` | Inverse cosecant |
| `asec(x)` | Inverse secant |
| `acot(x)` | Inverse cotangent |

### Hyperbolic

| Form | Meaning |
| --- | --- |
| `sinh(x)` | Hyperbolic sine |
| `cosh(x)` | Hyperbolic cosine |
| `tanh(x)` | Hyperbolic tangent |
| `csch(x)` | Hyperbolic cosecant |
| `sech(x)` | Hyperbolic secant |
| `coth(x)` | Hyperbolic cotangent |

### Inverse Hyperbolic

| Form | Meaning |
| --- | --- |
| `asinh(x)` | Inverse hyperbolic sine |
| `acosh(x)` | Inverse hyperbolic cosine |
| `atanh(x)` | Inverse hyperbolic tangent |
| `acsch(x)` | Inverse hyperbolic cosecant |
| `asech(x)` | Inverse hyperbolic secant |
| `acoth(x)` | Inverse hyperbolic cotangent |

### Logarithmic, Exponential and Roots

| Form | Meaning |
| --- | --- |
| `log(x)` | Decimal (base-10) logarithm |
| `ln(x)` | Natural logarithm |
| `log_2(x)` | Binary (base-2) logarithm |
| `exp(x)` | Exponential function (e^x) |
| `sqr(x)` | Square root |
| `sqrt(x)` | Square root |
| `cbrt(x)` | Cubic root |
| `root(a; b)` | N-th root |

### Rounding

| Form | Meaning |
| --- | --- |
| `round(x)` | Round to the nearest integer |
| `floor(x)` | Round down (towards -∞) |
| `ceiling(x)` | Round up (towards +∞) |
| `trunc(x)` | Truncate towards zero |

### Integer

| Form | Meaning |
| --- | --- |
| `mod(a; b)` | Remainder of integer division |
| `gcd(a; b)` | Greatest common divisor |
| `lcm(a; b)` | Least common multiple |
| `fact(x)` | Factorial |

### Complex

| Form | Meaning |
| --- | --- |
| `re(x)` | Real part of a complex number |
| `im(x)` | Imaginary part of a complex number |
| `abs(x)` | Absolute value / magnitude |
| `phase(x)` | Phase angle of a complex number |
| `conj(x)` | Complex conjugate |

### Aggregate and Interpolation

| Form | Meaning |
| --- | --- |
| `min(v₁; v₂; ...)` | Minimum of multiple values (scalars, vectors, or matrices) |
| `min(x)` | Minimum element of a vector |
| `min(x)` | Minimum element of a matrix |
| `max(v₁; v₂; ...)` | Maximum of multiple values (scalars, vectors, or matrices) |
| `max(x)` | Maximum element of a vector |
| `max(x)` | Maximum element of a matrix |
| `sum(v₁; v₂; ...)` | Sum of multiple values (scalars, vectors, or matrices) |
| `sum(x)` | Sum of all elements in a vector |
| `sum(x)` | Sum of all elements in a matrix |
| `sumsq(v₁; v₂; ...)` | Sum of squares of multiple values (scalars, vectors, or matrices) |
| `sumsq(x)` | Sum of squares of all elements in a vector |
| `sumsq(x)` | Sum of squares of all elements in a matrix |
| `srss(v₁; v₂; ...)` | Square root of sum of squares of multiple values (scalars, vectors, or matrices) |
| `srss(x)` | Square root of sum of squares of all elements in a vector |
| `srss(x)` | Square root of sum of squares of all elements in a matrix |
| `average(v₁; v₂; ...)` | Arithmetic mean of multiple values (scalars, vectors, or matrices) |
| `average(x)` | Arithmetic mean of all elements in a vector |
| `average(x)` | Arithmetic mean of all elements in a matrix |
| `product(v₁; v₂; ...)` | Product of multiple values (scalars, vectors, or matrices) |
| `product(x)` | Product of all elements in a vector |
| `product(x)` | Product of all elements in a matrix |
| `mean(v₁; v₂; ...)` | Geometric mean of multiple values (scalars, vectors, or matrices) |
| `mean(x)` | Geometric mean of all elements in a vector |
| `mean(x)` | Geometric mean of all elements in a matrix |
| `median(v₁; v₂; ...)` | Median of multiple values (scalars, vectors, or matrices) |
| `stdev(v₁; v₂; ...)` | Sample standard deviation (n - 1) of multiple values |
| `stdevp(v₁; v₂; ...)` | Population standard deviation (n) of multiple values |
| `var(v₁; v₂; ...)` | Sample variance (n - 1) of multiple values |
| `varp(v₁; v₂; ...)` | Population variance (n) of multiple values |
| `take(a; b)` | Returns the n-th element from a list of scalars |
| `take(a; b)` | Returns the n-th element from a vector |
| `take(a; b)` | Returns the n-th element from a matrix |
| `line(a; b)` | Linear interpolation with scalar data pairs |
| `line(a; b; c)` | Linear interpolation with x and y vectors |
| `line(a; b)` | Linear interpolation over a linearized matrix |
| `spline(a; b)` | Hermite spline interpolation with scalar data pairs |
| `spline(a; b; c)` | Hermite spline interpolation with x and y vectors |
| `spline(a; b)` | Hermite spline interpolation over a linearized matrix |

### Conditional and Logical

| Form | Meaning |
| --- | --- |
| `if(a; b; c)` | Conditional evaluation |
| `switch(c₁; v₁; c₂; v₂; …; def)` | Selective evaluation (multiple conditions) |
| `not(x)` | Logical NOT |
| `and(a; b)` | Logical AND of multiple boolean values |
| `or(a; b)` | Logical OR of multiple boolean values |
| `xor(a; b)` | Logical XOR of multiple boolean values |

### Other

| Form | Meaning |
| --- | --- |
| `sign(x)` | Sign of a number (-1, 0, or 1) |
| `random(x)` | Random number between 0 and x |
| `getunits(x)` | Gets the units of x (returns 1 if unitless) |
| `setunits(a; b)` | Sets the units to a scalar, vector, or matrix |
| `clrunits(x)` | Clears the units from a scalar, vector, or matrix |
| `hp(x)` | Converts to high-performance type |
| `ishp(x)` | Checks if the type is high-performance |
| `timer(x)` | Elapsed time in seconds |
| `mandelbrot(a; b)` | Mandelbrot set escape value |

## Vector functions

### Creational

| Form | Meaning |
| --- | --- |
| `vector(x)` | Creates an empty vector with length n |
| `vector_hp(x)` | Creates an empty high-performance vector with length n |
| `range(a; b; c)` | Creates a vector with values from x1 to xn with step s |
| `range_hp(a; b; c)` | Creates a high-performance vector from a range |
| `linspace(a; b; c)` | Vector of n equally spaced values from a to b |

### Structural

| Form | Meaning |
| --- | --- |
| `len(x)` | Returns the length of the vector |
| `size(x)` | Returns the actual size (index of last non-zero element) |
| `resize(a; b)` | Sets a new length for the vector |
| `fill(a; b)` | Fills the vector with a value |
| `join(x)` | Creates a vector by joining matrices, vectors, and scalars |
| `slice(a; b; c)` | Returns part of vector bounded by indexes i1 and i2 |
| `first(a; b)` | Returns the first n elements of the vector |
| `last(a; b)` | Returns the last n elements of the vector |
| `extract(a; b)` | Extracts elements from v whose indexes are in i |

### Data

| Form | Meaning |
| --- | --- |
| `sort(x)` | Sorts the vector in ascending order |
| `rsort(x)` | Sorts the vector in descending order |
| `order(x)` | Returns indexes in ascending order by element values |
| `revorder(x)` | Returns indexes in descending order by element values |
| `reverse(x)` | Returns vector with elements in reverse order |
| `count(a; b; c)` | Counts elements equal to x starting from index i |
| `search(a; b; c)` | Returns index of first element equal to x starting from i |
| `find(a; b; c)` | Returns indexes of elements equal to x after index i |
| `find_eq(a; b; c)` | Returns indexes of elements = x after index i |
| `find_ne(a; b; c)` | Returns indexes of elements != x after index i |
| `find_lt(a; b; c)` | Returns indexes of elements < x after index i |
| `find_le(a; b; c)` | Returns indexes of elements <= x after index i |
| `find_gt(a; b; c)` | Returns indexes of elements > x after index i |
| `find_ge(a; b; c)` | Returns indexes of elements >= x after index i |
| `lookup(a; b; c)` | Elements of a where corresponding elements of b equal x |
| `lookup_eq(a; b; c)` | Elements of a where b elements = x |
| `lookup_ne(a; b; c)` | Elements of a where b elements != x |
| `lookup_lt(a; b; c)` | Elements of a where b elements < x |
| `lookup_le(a; b; c)` | Elements of a where b elements <= x |
| `lookup_gt(a; b; c)` | Elements of a where b elements > x |
| `lookup_ge(a; b; c)` | Elements of a where b elements >= x |
| `polyfit(a; b; c)` | Least-squares polynomial fit |
| `interp(a; b; c)` | Linear interpolation in a table |
| `linterp(a; b; c)` | Linear interpolation in a table, continued outside it |

### Math

| Form | Meaning |
| --- | --- |
| `norm(x)` | L2 (Euclidean) norm of vector |
| `norm_1(x)` | L1 (Manhattan) norm of vector |
| `norm_2(x)` | L2 (Euclidean) norm of vector |
| `norm_e(x)` | L2 (Euclidean) norm of vector |
| `norm_p(a; b)` | Lp norm of vector |
| `norm_i(x)` | L-infinity norm of vector |
| `unit(x)` | Normalized vector (with L2 norm = 1) |
| `cumsum(x)` | Running sum of a vector |
| `convolve(a; b)` | Discrete convolution of two vectors |
| `dot(a; b)` | Scalar (dot) product of two vectors |
| `cross(a; b)` | Cross product of two vectors (length 2 or 3) |

## Matrix functions

Functions marked `_hp` use the high-performance storage; they behave like the ordinary ones.

### Creational

| Form | Meaning |
| --- | --- |
| `matrix(a; b)` | Creates an empty matrix with dimensions m x n |
| `identity(x)` | Creates an identity matrix with dimensions n x n |
| `diagonal(a; b)` | Creates a diagonal matrix n x n filled with value d |
| `column(a; b)` | Creates a column matrix m x 1 filled with value c |
| `utriang(x)` | Creates an upper triangular matrix n x n |
| `ltriang(x)` | Creates a lower triangular matrix n x n |
| `symmetric(x)` | Creates a symmetric matrix n x n |
| `matrix_hp(a; b)` | Creates a high-performance matrix m x n |
| `identity_hp(x)` | Creates a high-performance identity matrix n x n |
| `diagonal_hp(a; b)` | Creates a high-performance diagonal matrix n x n |
| `column_hp(a; b)` | Creates a high-performance column matrix m x 1 |
| `utriang_hp(x)` | Creates a high-performance upper triangular matrix |
| `ltriang_hp(x)` | Creates a high-performance lower triangular matrix |
| `symmetric_hp(x)` | Creates a high-performance symmetric matrix |
| `vec2diag(x)` | Creates a diagonal matrix from vector elements |
| `vec2row(x)` | Creates a row matrix from vector elements |
| `vec2col(x)` | Creates a column matrix from vector elements |
| `join_cols(a; b)` | Creates a matrix by joining column vectors |
| `join_rows(a; b)` | Creates a matrix by joining row vectors |
| `augment(a; b)` | Creates a matrix by appending matrices side by side |
| `stack(a; b)` | Creates a matrix by stacking matrices vertically |

### Structural

| Form | Meaning |
| --- | --- |
| `n_rows(x)` | Number of rows in matrix M |
| `n_cols(x)` | Number of columns in matrix M |
| `mresize(a; b; c)` | Sets new dimensions m and n for matrix M |
| `mfill(a; b)` | Fills the matrix M with value x |
| `fill_row(a; b; c)` | Fills the i-th row of matrix M with value x |
| `fill_col(a; b; c)` | Fills the j-th column of matrix M with value x |
| `copy(a; b; c; d)` | Copies all elements from A to B starting at indexes i, j |
| `add(a; b; c; d)` | Adds elements from A to B starting at indexes i, j |
| `row(a; b)` | Extracts the i-th row of matrix M as a vector |
| `col(a; b)` | Extracts the j-th column of matrix M as a vector |
| `extract_rows(a; b)` | Extracts rows from M whose indexes are in vector i |
| `extract_cols(a; b)` | Extracts columns from M whose indexes are in vector j |
| `diag2vec(x)` | Extracts diagonal elements of matrix M to a vector |
| `submatrix(a; b; c; d; e)` | Extracts submatrix bounded by rows i1-i2 and columns j1-j2 |

### Data

| Form | Meaning |
| --- | --- |
| `sort_cols(a; b)` | Sorts columns based on values in row i (ascending) |
| `rsort_cols(a; b)` | Sorts columns based on values in row i (descending) |
| `sort_rows(a; b)` | Sorts rows based on values in column j (ascending) |
| `rsort_rows(a; b)` | Sorts rows based on values in column j (descending) |
| `order_cols(a; b)` | Column indexes in ascending order by row i values |
| `revorder_cols(a; b)` | Column indexes in descending order by row i values |
| `order_rows(a; b)` | Row indexes in ascending order by column j values |
| `revorder_rows(a; b)` | Row indexes in descending order by column j values |
| `mcount(a; b)` | Number of occurrences of value x in matrix M |
| `msearch(a; b; c; d)` | Vector with indexes of first occurrence of x starting from i, j |
| `mfind(a; b)` | Indexes of all elements in M equal to x |
| `mfind_eq(a; b)` | Indexes of elements = x |
| `mfind_ne(a; b)` | Indexes of elements != x |
| `mfind_lt(a; b)` | Indexes of elements < x |
| `mfind_le(a; b)` | Indexes of elements <= x |
| `mfind_gt(a; b)` | Indexes of elements > x |
| `mfind_ge(a; b)` | Indexes of elements >= x |
| `hlookup(a; b; c; d)` | Values from row i2 where row i1 elements = x |
| `hlookup_eq(a; b; c; d)` | Values from row i2 where row i1 = x |
| `hlookup_ne(a; b; c; d)` | Values from row i2 where row i1 != x |
| `hlookup_lt(a; b; c; d)` | Values from row i2 where row i1 < x |
| `hlookup_le(a; b; c; d)` | Values from row i2 where row i1 <= x |
| `hlookup_gt(a; b; c; d)` | Values from row i2 where row i1 > x |
| `hlookup_ge(a; b; c; d)` | Values from row i2 where row i1 >= x |
| `vlookup(a; b; c; d)` | Values from column j2 where column j1 = x |
| `vlookup_eq(a; b; c; d)` | Values from column j2 where column j1 = x |
| `vlookup_ne(a; b; c; d)` | Values from column j2 where column j1 != x |
| `vlookup_lt(a; b; c; d)` | Values from column j2 where column j1 < x |
| `vlookup_le(a; b; c; d)` | Values from column j2 where column j1 <= x |
| `vlookup_gt(a; b; c; d)` | Values from column j2 where column j1 > x |
| `vlookup_ge(a; b; c; d)` | Values from column j2 where column j1 >= x |

### Math

| Form | Meaning |
| --- | --- |
| `hprod(a; b)` | Hadamard (element-wise) product of matrices A and B |
| `fprod(a; b)` | Frobenius product of matrices A and B |
| `kprod(a; b)` | Kronecker product of matrices A and B |
| `mnorm(x)` | L2 norm of matrix M |
| `mnorm_1(x)` | L1 norm of matrix M |
| `mnorm_2(x)` | L2 norm of matrix M |
| `mnorm_e(x)` | Frobenius norm of matrix M |
| `mnorm_i(x)` | L-infinity norm of matrix M |
| `cond(x)` | Condition number based on L2 norm |
| `cond_1(x)` | Condition number based on L1 norm |
| `cond_2(x)` | Condition number based on L2 norm |
| `cond_e(x)` | Condition number based on Frobenius norm |
| `cond_i(x)` | Condition number based on L-infinity norm |
| `det(x)` | Determinant of matrix M |
| `rank(x)` | Rank of matrix M |
| `trace(x)` | Trace of matrix M (sum of diagonal elements) |
| `transp(x)` | Transpose of matrix M |
| `adj(x)` | Adjugate of matrix M |
| `cofactor(x)` | Cofactor matrix of M |
| `polyroots(x)` | Real roots of a polynomial |
| `eigenvals(a; b)` | First ne eigenvalues of matrix M (or all if omitted), or of K x = λ M x |
| `eigenvecs(a; b)` | First ne eigenvectors of matrix M (or all if omitted), or of K x = λ M x |
| `eigen(a; b)` | First ne eigenvalues and eigenvectors (or all if omitted) |
| `ceigenvals(x)` | Complex eigenvalues of matrix M |
| `ceigenvecs(x)` | Complex eigenvectors of matrix M |
| `cholesky(x)` | Cholesky decomposition of symmetric positive-definite matrix |
| `lu(x)` | LU decomposition of matrix M |
| `qr(x)` | QR decomposition of matrix M |
| `svd(x)` | Singular value decomposition of M |
| `inverse(x)` | Inverse of matrix M |
| `lsolve(a; b)` | Solves Ax = b using LDLT (symmetric) or LU decomposition |
| `clsolve(a; b)` | Solves Ax = b using Cholesky decomposition |
| `slsolve(a; b)` | Solves Ax = b using preconditioned conjugate gradient |
| `msolve(a; b)` | Solves AX = B using LDLT or LU decomposition |
| `cmsolve(a; b)` | Solves AX = B using Cholesky decomposition |
| `smsolve(a; b)` | Solves AX = B using preconditioned conjugate gradient |
| `fft(x)` | Fast Fourier transform (1 row for real, 2 for complex) |
| `ift(x)` | Inverse Fourier transform (1 row for real, 2 for complex) |
| `take(a; b; c)` | Returns element of matrix M at indexes x and y |
| `line(a; b; c)` | Double linear interpolation from matrix M |
| `spline(a; b; c)` | Double Hermite spline interpolation from matrix M |

## Commands

Commands start with `$` and take their arguments in braces. `@` separates the expression from the variable and its interval `a : b`. In a sheet, write them on a `>>` line, except plots, which have the `plot` line.

| Form | Meaning |
| --- | --- |
| `$Root{a @ b = c : d}` | Root finding for f(x) = 0 |
| `$Root{a = b @ c = d : e}` | Root finding for f(x) = const |
| `$Find{a @ b = c : d}` | Find approximate solution (x not required to be precise) |
| `$Sup{a @ b = c : d}` | Find local maximum of a function |
| `$Inf{a @ b = c : d}` | Find local minimum of a function |
| `$Area{a @ b = c : d}` | Numerical integration (adaptive Gauss-Lobatto) |
| `$Integral{a @ b = c : d}` | Numerical integration (Tanh-Sinh) |
| `$Slope{a @ b = c}` | Numerical differentiation (slope at point) |
| `$Sum{a @ b = c : d}` | Iterative sum |
| `$Product{a @ b = c : d}` | Iterative product |
| `$Repeat{a @ b = c : d}` | Iterative expression block with counter |
| `$While{a; b}` | Iterative expression block with condition |
| `$Block{x}` | Multiline expression block |
| `$Inline{x}` | Inline expression block |
| `$Plot{a @ b = c : d}` | Simple function plot |
| `$Plot{a \| b @ c = d : e}` | Parametric plot |
| `$Plot{a & b @ c = d : e}` | Multiple function plot |
| `$Map{a @ b = c : d & e = f : g}` | 2D color map of a 3D surface |

## Directives

Directives start with `#`. The block forms (`#if` ... `#end if`) are listed once, by their opening word.

### Program Flow Control

| Form | Meaning |
| --- | --- |
| `#if` | If condition |
| `#else if condition` | Else If clause |
| `#else` | Else clause |

### Iteration Blocks

| Form | Meaning |
| --- | --- |
| `#repeat` | Repeat loop start |
| `#for` | For loop start |
| `#while` | While loop start |
| `#loop` | End of loop block |
| `#break` | Break out of current loop |
| `#continue` | Continue to next iteration |

### Modules and Macros

| Form | Meaning |
| --- | --- |
| `#include filename` | Include external file (module). Path is relative to the current file or the library path. |
| `#ProjectPath path` | Declares the folder {project} stands for in #include, #read, #write and image paths below this line. |
| `#LibraryPath path` | Declares the folder {library} stands for in #include, #read, #write and image paths below this line. |
| `#local` | Start local section (not included when file is imported) |
| `#global` | Start global section (included when file is imported) |
| `#def` | Define macro or string variable |
| `#def name$ = expression` | Inline string variable definition |
| `#def name$(param$) = expression` | Inline macro with parameters |

### External Data

| Form | Meaning |
| --- | --- |
| `#read M from filename` | Read matrix from text/CSV or Excel file |
| `#read M from filename.csv@R1C1:R2C2 TYPE=R SEP=','` | Read matrix from a CSV/text file with all options. @R1C1:R2C2 = cell range (row, column). TYPE: R=Raw (default), D=Diagonal, C=Column, L=Lower triangular, U=Upper triangular, S=Symmetric, V=Vector. SEP: column separator character (default ','). |
| `#read M from filename.xlsx@Sheet1!A1:B2 TYPE=R` | Read matrix from an Excel file (.xlsx/.xlsm) with all options. @Sheet!A1:B2 = sheet name and cell range. TYPE: R=Raw (default), D=Diagonal, C=Column, L=Lower triangular, U=Upper triangular, S=Symmetric, V=Vector. |
| `#write M to filename` | Write matrix to text/CSV or Excel file |
| `#write M to filename.csv@R1C1:R2C2 TYPE=N SEP=','` | Write matrix to a CSV/text file with all options. @R1C1:R2C2 = cell range (row, column). TYPE: Y=Compact (transpose special matrix types), N=Normal (default). SEP: column separator character (default ','). |
| `#write M to filename.xlsx@Sheet1!A1:B2 TYPE=N` | Write matrix to an Excel file (.xlsx/.xlsm) with all options. @Sheet!A1:B2 = sheet name and cell range. TYPE: Y=Compact (transpose special matrix types), N=Normal (default). |
| `#append M to filename` | Append matrix to text/CSV or Excel file |
| `#append M to filename.csv@R1C1:R2C2 TYPE=N SEP=','` | Append matrix to a CSV/text file with all options. @R1C1:R2C2 = cell range (row, column). TYPE: Y=Compact (transpose special matrix types), N=Normal (default). SEP: column separator character (default ','). |
| `#append M to filename.xlsx@Sheet1!A1:B2 TYPE=N` | Append matrix to an Excel file (.xlsx/.xlsm) with all options. @Sheet!A1:B2 = sheet name and cell range. TYPE: Y=Compact (transpose special matrix types), N=Normal (default). |

### Read Only

| Form | Meaning |
| --- | --- |
| `#const` | Define a constant (readonly) variable or function |
| `#const name = value` | Define a constant variable |
| `#const f(x) = expression` | Define a constant function |

### Output Control

| Form | Meaning |
| --- | --- |
| `#show` | Show the output contents (default). Optionally takes a condition, e.g. '#show x == 5'. Applies to the rest of the document, or up to a matching #end show. |
| `#hide` | Hide the output contents. Optionally takes a condition, e.g. '#hide x == 5'. Applies to the rest of the document, or up to a matching #end hide. |
| `#pre` | Show contents on screen only - hidden when printing/exporting to PDF. Optionally takes a condition. Applies to the rest of the document, or up to a matching #end pre. |
| `#post` | Show contents in the preview and when printing/exporting to PDF - hidden in UI mode. Optionally takes a condition. Applies to the rest of the document, or up to a matching #end post. |
| `#val` | Show only the result, without the equation. Optionally takes a condition. Applies to the rest of the document, or up to a matching #end val. |
| `#equ` | Show complete equations and results (default). Optionally takes a condition. Applies to the rest of the document, or up to a matching #end equ. |
| `#noc` | Show equations without results (no calculations). Optionally takes a condition. Applies to the rest of the document, or up to a matching #end noc. |
| `#varsub` | Show equations with both variable names and their substituted values (default). Optionally takes a condition. Applies to the rest of the document, or up to a matching #end varsub. |
| `#nosub` | Show equations with variable names only, without substituted values. Optionally takes a condition. Applies to the rest of the document, or up to a matching #end nosub. |
| `#novar` | Show equations with substituted values only, without variable names. Optionally takes a condition. Applies to the rest of the document, or up to a matching #end novar. |
| `#split` | Split long equations after the "=" symbol onto a new indented line. Applies to the rest of the document; takes no condition and has no #end form - switch back with #wrap. |
| `#wrap` | Wrap long equations at the end of the line (default). Applies to the rest of the document; takes no condition and has no #end form - switch back with #split. |
| `#format spec` | Specify custom format string |
| `#format default` | Restore default formatting |
| `#phasor` | Set complex number output to polar phasor (A angle phi) |
| `#complex` | Set complex number output to cartesian algebraic (a + ib) |
| `#md on` | Enable markdown in comments |
| `#md off` | Disable markdown in comments |
| `#html` | Output the following lines as raw HTML. Only #cpd, #markdown and the #end forms are allowed inside; macros still expand. Optionally takes a condition, e.g. '#html x > 3', that skips the block when false. Applies up to a matching #end html, or until the next mode directive. |
| `#markdown` | Render the following lines as Markdown, including tables, lists and code blocks. Only #cpd, #html and the #end forms are allowed inside; macros still expand. Optionally takes a condition, e.g. '#markdown x > 3', that skips the block when false. Applies up to a matching #end markdown, or until the next mode directive. |
| `#cpd` | Parse the following lines as Calcpad (the default mode), e.g. to calculate inside an #html block. Optionally takes a condition, e.g. '#cpd x > 3', that skips the block when false. Applies up to a matching #end cpd, or until the next mode directive. |

### UI Inputs

| Form | Meaning |
| --- | --- |
| `#UI L = 10m` | Expose the following assignment as an input control in UI mode. Ignored when rendering a report. |
| `#UI {"type": "datagrid", "rows": 2, "columns": 3, "columnHeaders": ["a", "b", "c"]} M = [1; 2; 3 \| 4; 5; 6]` | Editable grid for a vector or matrix. Rows and columns are auto-detected from the right hand side when omitted. |
| `#UI v = [1; 2; 3]` | Editable grid whose size is auto-detected from the vector or matrix assigned to it. |
| `#UI {"type": "datagrid", "width": "100%", "rowHeaderWidth": 120, "columnWidths": [2, 1, 1]} M = matrix(2; 3)` | Sized grid. 'width' is the total in pixels or "100%" to fill the line; 'columnWidths' are shared out in proportion. |
| `#UI {"forceUnits": false} q = 3kN/m` | Put the unit in the input box, so the whole value - number and unit - is editable. |
| `#UI {"allowExpression": true} k = 2*L` | Accept any expression, written to the document as typed. The line's own errors report what it cannot read. |
| `#UI {"type": "dropdown", "keys": ["Low", "High"], "values": ["1", "2"]} x = 1` | Drop-down list. 'keys' are the labels shown, 'values' are substituted into the calculation. |
| `#UI {"type": "radio", "keys": ["Steel", "Concrete"], "values": ["200GPa", "25GPa"]} E = 200GPa` | Radio button group. 'keys' are the labels shown, 'values' are substituted into the calculation. |
| `#UI {"type": "checkbox"} b = 1` | Checkbox toggling the value between 1 and 0. |

### Settings/Rounding

| Form | Meaning |
| --- | --- |
| `#round digits` | Round output to n digits after decimal point |
| `#round default` | Restore rounding to default settings |

### Settings/Overrides

| Form | Meaning |
| --- | --- |
| `#settings {"decimals": 4, "substitute": false}` | Override engine settings for subsequent lines via JSON. Use the Properties panel to see the available settings and edit them. |

### Breakpoints

| Form | Meaning |
| --- | --- |
| `#pause` | Pause calculation and wait for user to resume. Not supported in Calcpad.Web. |
| `#input` | Render input form and wait for user input. Not supported in Calcpad.Web. |

### Settings/Angle Units

| Form | Meaning |
| --- | --- |
| `#deg` | Set angle units to degrees |
| `#rad` | Set angle units to radians |
| `#gra` | Set angle units to gradians |

## Worksheet settings

Written as an assignment on a line of their own. In a sheet, use a `>>` line.

| Form | Meaning |
| --- | --- |
| `PlotHeight = 400` | Height of plot area in pixels |
| `PlotWidth = 800` | Width of plot area in pixels |
| `PlotSVG = 1` | Draw plots in SVG (1) or PNG (0) format |
| `PlotAdaptive = 1` | Use adaptive mesh (1) or uniform (0) |
| `PlotStep = 0` | Mesh size for map plotting |
| `PlotPalette = 0` | Color palette number (0-9) for surface plots |
| `PlotShadows = 1` | Draw surface plots with shadows |
| `PlotSmooth = 1` | Smooth gradient (1) or isobands (0) for surface plots |
| `PlotLightDir = 0` | Direction to light source (0-7) clockwise |
| `Precision = 1e-12` | Relative precision for numerical methods (10^-2 to 10^-16) |
| `Tol = 1e-8` | Target tolerance for iterative PCG solver |

## Number format strings

Used after `#format`, or at the end of an equation to format one result.

| Form | Meaning |
| --- | --- |
| `:e` | Exponential notation, default precision |
| `:e3` | Exponential notation with 3 mantissa digits |
| `:f` | Fixed-point notation, default precision |
| `:f3` | Fixed-point notation with 3 decimal digits |
| `:g` | General notation, default precision |
| `:g3` | General notation with 3 significant digits |
| `:n` | Number with digit grouping, default precision |
| `:n3` | Number with digit grouping and 3 decimal digits |
| `:c` | Currency notation, default precision |
| `:c3` | Currency notation with 3 decimal digits |
| `:0.000` | Custom format - fixed decimal places, zero-padded |
| `:#,#.00` | Custom format - digit grouping with optional leading digits |
| `:0.00e+00` | Custom format - explicit exponential notation |

## Units

The predefined units by group. Prefixes (k, M, G, m, μ and so on) are listed where they apply. The names, not the descriptions, are what you type.

* **Dimensionless**: `%`, `‰`, `‱`, `pcm`, `ppm`, `ppb`, `ppt`, `ppq`
* **Angles**: `°`, `′`, `″`, `deg`, `rad`, `grad`, `rev`
* **Metric/Mass**: `g`, `dag`, `hg`, `kg`, `t`, `kt`, `Mt`, `Gt`, `dg`, `cg`, `mg`, `μg`, `ng`, `pg`, `Da`, `u`
* **Metric/Length**: `m`, `km`, `dm`, `cm`, `mm`, `μm`, `nm`, `pm`, `AU`, `ly`
* **Metric/Area**: `a`, `daa`, `ha`
* **Metric/Volume**: `L`, `daL`, `hL`, `dL`, `cL`, `mL`, `μL`, `nL`, `pL`
* **Time**: `s`, `ms`, `μs`, `ns`, `ps`, `min`, `h`, `d`, `w`, `y`
* **Frequency**: `Hz`, `kHz`, `MHz`, `GHz`, `THz`, `mHz`, `μHz`, `nHz`, `pHz`, `rpm`
* **Speed**: `kmh`
* **Temperature**: `°C`, `Δ°C`, `K`
* **Metric/Force**: `N`, `daN`, `hN`, `kN`, `MN`, `GN`, `TN`, `gf`, `kgf`, `tf`, `dyn`
* **Metric/Moment**: `Nm`, `kNm`
* **Metric/Pressure**: `Pa`, `daPa`, `hPa`, `kPa`, `MPa`, `GPa`, `TPa`, `dPa`, `cPa`, `mPa`, `μPa`, `nPa`, `pPa`, `bar`, `mbar`, `μbar`, `atm`, `at`, `Torr`, `mmHg`
* **Metric/Viscosity**: `P`, `cP`, `St`, `cSt`
* **Metric/Energy**: `J`, `kJ`, `MJ`, `GJ`, `TJ`, `mJ`, `μJ`, `nJ`, `pJ`, `Wh`, `kWh`, `MWh`, `GWh`, `TWh`, `mWh`, `μWh`, `nWh`, `pWh`, `VAh`, `kVAh`, `MVAh`, `GVAh`, `VARh`, `kVARh`, `MVARh`, `GVARh`, `eV`, `keV`, `MeV`, `GeV`, `TeV`, `PeV`, `EeV`, `cal`, `kcal`, `erg`
* **Metric/Power**: `W`, `kW`, `MW`, `GW`, `TW`, `mW`, `μW`, `nW`, `pW`, `VA`, `kVA`, `MVA`, `GVA`, `TVA`, `mVA`, `μVA`, `nVA`, `pVA`, `VAR`, `kVAR`, `MVAR`, `GVAR`, `TVAR`, `mVAR`, `μVAR`, `nVAR`, `pVAR`
* **Electrical/Current**: `A`, `kA`, `MA`, `GA`, `TA`, `mA`, `μA`, `nA`, `pA`
* **Electrical/Charge**: `C`, `kC`, `MC`, `GC`, `TC`, `mC`, `μC`, `nC`, `pC`, `Ah`, `mAh`
* **Electrical/Potential**: `V`, `kV`, `MV`, `GV`, `TV`, `mV`, `μV`, `nV`, `pV`
* **Electrical/Capacitance**: `F`, `kF`, `MF`, `GF`, `TF`, `mF`, `μF`, `nF`, `pF`
* **Electrical/Resistance**: `Ω`, `kΩ`, `MΩ`, `GΩ`, `TΩ`, `mΩ`, `μΩ`, `nΩ`, `pΩ`
* **Electrical/Conductance**: `S`, `kS`, `MS`, `GS`, `TS`, `mS`, `μS`, `nS`, `pS`, `℧`, `k℧`, `M℧`, `G℧`, `T℧`, `m℧`, `μ℧`, `n℧`, `p℧`
* **Electrical/Magnetic Flux**: `Wb`, `kWb`, `MWb`, `GWb`, `TWb`, `mWb`, `μWb`, `nWb`, `pWb`
* **Electrical/Magnetic Flux Density**: `T`, `kT`, `MT`, `GT`, `TT`, `mT`, `μT`, `nT`, `pT`
* **Electrical/Inductance**: `H`, `kH`, `MH`, `GH`, `TH`, `mH`, `μH`, `nH`, `pH`
* **Luminous**: `cd`, `lm`, `lx`
* **Radioactivity**: `Bq`, `kBq`, `MBq`, `GBq`, `TBq`, `mBq`, `μBq`, `nBq`, `pBq`, `Ci`, `Rd`
* **Dose/Absorbed**: `Gy`, `kGy`, `MGy`, `GGy`, `TGy`, `mGy`, `μGy`, `nGy`, `pGy`
* **Dose/Equivalent**: `Sv`, `kSv`, `MSv`, `GSv`, `TSv`, `mSv`, `μSv`, `nSv`, `pSv`
* **Catalytic Activity**: `kat`
* **Amount of Substance**: `mol`
* **Imperial/Mass**: `gr`, `dr`, `oz`, `lb`, `lbm`, `lb_m`, `kipm`, `kip_m`, `klb`, `st`, `qr`, `cwt`, `cwt_US`, `cwt_UK`, `ton`, `ton_US`, `ton_UK`, `slug`
* **Imperial/Length**: `th`, `in`, `ft`, `yd`, `ch`, `fur`, `mi`, `ftm`, `ftm_UK`, `ftm_US`, `cable`, `cable_UK`, `cable_US`, `nmi`, `li`, `rod`, `pole`, `perch`, `lea`
* **Imperial/Speed**: `mph`, `knot`
* **Imperial/Temperature**: `°F`, `Δ°F`, `°R`
* **Imperial/Area**: `rood`, `ac`
* **Imperial/Volume**: `fl_dr_UK`, `fl_dr_US`, `fl_oz`, `fl_oz_UK`, `fl_oz_US`, `gi`, `gi_UK`, `gi_US`, `pt`, `pt_UK`, `pt_US`, `pt_dry`, `qt`, `qt_UK`, `qt_US`, `qt_dry`, `gal`, `gal_UK`, `gal_US`, `gal_dry`, `bbl`, `bbl_UK`, `bbl_US`, `bbl_dry`, `pk_UK`, `pk_US`, `bu_UK`, `bu_US`
* **Imperial/Force**: `ozf`, `oz_f`, `lbf`, `lb_f`, `kip`, `kipf`, `kip_f`, `tonf`, `tonf_US`, `tonf_UK`, `pdl`
* **Imperial/Pressure**: `osi`, `osf`, `psi`, `psf`, `ksi`, `ksf`, `tsi`, `tsf`, `inHg`
* **Imperial/Energy**: `BTU`, `therm`, `therm_US`, `therm_UK`, `quad`
* **Imperial/Power**: `hp`, `hp_E`, `hp_S`, `hp_M`, `ks`
