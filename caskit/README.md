# civilkit-calcs-caskit

A small, pure-Rust computer algebra library for engineering formulas. Exact rational numbers, canonical
expressions, differentiation, integration, factoring and solving, with no dependencies. The library name is
`caskit`.

It is the symbolic layer of [CivilKit Calcs](https://civilkit.io/calcs/), a browser-based engineering
calculation sheet. Source and issues: <https://github.com/mageaustralia/civilkit-calcs>. The crate is
usable on its own and compiles to WebAssembly (`wasm32-unknown-unknown`) without changes.

Status: 0.1, early. The API may change between 0.x minor versions.

## Quick start

```toml
[dependencies]
civilkit-calcs-caskit = "0.1"
```

```rust
use caskit::{parse, parse_equation, solve};
use std::collections::HashMap;

fn main() -> Result<(), caskit::CasError> {
    // Parse and simplify. Like terms and powers combine, 1/(E*I) is the product E^-1 * I^-1,
    // so this reduces to exactly zero.
    let e = parse("-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)")?.simplify()?;
    assert!(e.is_zero());

    // Differentiate and integrate.
    let f = parse("x^3 + sin(2*x)")?;
    assert_eq!(f.diff("x")?.simplify()?, parse("3*x^2 + 2*cos(2*x)")?.simplify()?);
    assert_eq!(parse("a*x^2")?.integrate("x")?, parse("a*x^3/3")?);

    // Solve and rearrange: M = w*L^2/8 for w.
    let w = solve(&parse_equation("M = w*L^2/8")?, "w")?;
    assert_eq!(w[0].to_ascii(), "8*M/L^2");

    // Evaluate numerically.
    let mut v = HashMap::new();
    v.insert("x".to_string(), 3.0);
    assert_eq!(parse("x^2 + 1")?.eval(&v)?, 10.0);

    // Print as ASCII or LaTeX.
    println!("{}", parse("w*L^2/8")?.to_latex()); // \frac{L^{2} w}{8}
    Ok(())
}
```

The simply supported beam, derived from `E*I*y'' = -(w*L*x/2 - w*x^2/2)` (`y` measured in the direction
of the load):

```rust
use caskit::{parse, solve_linear_system, Equation, Expr};

fn main() -> Result<(), caskit::CasError> {
    let ypp = parse("-(w*L*x/2 - w*x^2/2)/(E*I)")?;
    let yp = Expr::add(ypp.integrate("x")?, parse("C1")?)?;
    let y = Expr::add(yp.integrate("x")?, parse("C2")?)?;
    // y(0) = 0 and y(L) = 0
    let c = solve_linear_system(
        &[
            Equation::new(y.substitute("x", &parse("0")?)?, parse("0")?),
            Equation::new(y.substitute("x", &parse("L")?)?, parse("0")?),
        ],
        &["C1", "C2"],
    )?;
    let y = y.substitute_all(&[("C1", c[0].clone()), ("C2", c[1].clone())])?;
    let mid = y.substitute("x", &parse("L/2")?)?.simplify()?;
    assert_eq!(mid, parse("5*w*L^4/(384*E*I)")?.simplify()?);
    Ok(())
}
```

Runnable examples: `cargo run -p civilkit-calcs-caskit --example basics`, `--example beam_deflection` and
`--example solve_for`.

## What it does

* **Expressions.** Exact rationals (`i128`, overflow is an error), symbols, `pi`, `e`, `sin`, `cos`, `ln`,
  `exp`, `sqrt` (a power), sums, products, powers with rational or symbolic exponents. Constructors keep every
  expression canonical, so equal expressions compare equal with `==`.
* **Parse and print.** `parse("x^2 - 5*x + 6")`, `parse_equation("M = w*L^2/8")`; `to_ascii()` (parses back
  to the same expression) and `to_latex()`.
* **`simplify` / `expand`.** Distribute, collect like terms, combine powers, fold exact numbers
  (`sqrt(8)` is `2*sqrt(2)`) and exact values of `sin`/`cos` at multiples of pi/12 that are also multiples of
  pi/6 or pi/4.
* **`diff` / `diff_n`.** Sum, product, chain and power rules for every supported function, including
  variable exponents.
* **`integrate` / `integrate_definite`.** Constants, `x^n` (and `1/x` as `ln(x)`), powers of a linear form,
  `sin`, `cos`, `exp`, `ln` of a linear form, and `x^k` times `sin`, `cos` or `exp` of a linear form (by
  parts), with coefficients that contain other symbols.
* **`factor`.** Common numeric and symbolic factors of a sum; polynomials in one symbol with rational
  coefficients (degree 2 to 8) by rational roots; cancels common factors in a product with a quotient.
* **`solve`.** Linear and quadratic equations in the unknown with symbolic coefficients; rearrangement of a
  single power (`M = w*L^2/8` for `L` gives both branches); `solve_linear_system` for 1 to 4 unknowns.
* **`eval` / `substitute` / `substitute_all`.** Numeric evaluation with a variable map; exact substitution.

## Conventions

* Symbols are treated as positive real quantities when rational powers are combined: `sqrt(x^2)` is `x`,
  `sqrt(a*b)` is `sqrt(a)*sqrt(b)`. A sum is positive only when every term is.
* Solving `x^(m/n) = V` returns `V^(n/m)`, and also its negative when `m` is even (listed second). A
  numeric quadratic returns its roots in ascending order; a symbolic one returns the `-sqrt` root first.
* `ln(u)` as an antiderivative holds for `u > 0` (there is no absolute value).
* `e` and `pi` (lower case) are constants; `E`, `I`, `L` and so on are ordinary symbols.

## What it cannot do

Everything below is an explicit error or simply absent. Nothing is guessed.

* No complex numbers. Complex roots are `ComplexRoots`; `ln` of a non-positive number is a domain error;
  a negative number to a fractional power stays unevaluated.
* Functions: only `sin`, `cos`, `ln`, `exp` and `sqrt`. No `tan`, inverse or hyperbolic functions, `abs`,
  `min`, `max`, `floor` or piecewise definitions. No identity such as `sin^2 + cos^2 = 1`.
* `simplify` does not cancel a common factor between a numerator and a denominator that are sums
  (use `factor` for polynomials in one symbol), and does no partial fractions or trigonometric rewriting.
* `factor` does not split irreducible-over-the-rationals factors such as `x^4 + 4`, does not factor
  polynomials in several symbols (for example `a^2 - b^2`), and stops at degree 8.
* `integrate` returns `CannotIntegrate` for: powers of non-linear bases (`1/(x^2+1)`), `sin(x)^2`, products
  of two non-polynomial functions (`sin(x)*cos(x)`), `x*ln(x)`, `exp(x^2)`, a variable in an exponent,
  three or more factors that depend on the variable, and anything that needs a substitution. Definite
  integrals do not check the integrand for singularities between the limits.
* `solve` handles polynomials of degree 1 and 2 in the unknown (negative whole powers are cleared), and a
  single power of the unknown. Not supported: degree 3 or higher in a sum, fractional powers in a sum,
  `sin(x) = c`, `exp(x) = c`, and equations whose unknown sits inside a denominator sum. A system must be
  square, linear in the unknowns, with at most 4 unknowns (Cramer's rule with expanded determinants).
  Symbolic pivots and determinants are assumed non-zero; only a determinant that expands to exactly zero is
  reported as `Singular`.
* Exact numbers are `i128` rationals. A result that does not fit is `Overflow`. Expansion is capped
  (powers of sums up to 64, 20 000 terms) and reports `TooLarge`.
* Input syntax: no implicit multiplication (`2*x`, not `2x`), no derivative notation such as `y''`, no
  assignment or function definitions.

## Testing

The test suite compares every result with an independent numeric oracle: random expressions are evaluated
in `f64` straight from the generator and must agree with the built, simplified, expanded and differentiated
forms (derivatives against a central finite difference), and `integrate` followed by `diff` must return the
integrand.

## Licence

MIT.
