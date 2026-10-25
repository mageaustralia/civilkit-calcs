# caskit

`caskit/` is a small pure-Rust computer algebra library, published as the crate `civilkit-calcs-caskit`
(library name `caskit`). The engine (`calcpad-core`) is numeric only; caskit is the symbolic layer for
formulas that must stay exact and in terms of symbols: rearranging a formula, integrating a bending moment
twice, solving for the constants of integration.

Status: phase 1. The crate is a workspace member and has tests, but it is not released or published, and
the web app does not load it yet. Nothing in the engine or the other crates depends on it.

## Design

* **Exact numbers.** Rationals on `i128`. Overflow is an error (`CasError::Overflow`), so a number is never
  silently wrong.
* **Canonical form.** Every constructor returns a canonical expression: sums and products are flattened and
  sorted, like terms and equal bases are combined, a quotient is a negative power. Two equal expressions
  compare equal with `==`. In particular `1/(E*I)` is `E^-1 * I^-1`, which is what lets
  `-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)` reduce to exactly 0.
* **Positive symbols.** Rational powers combine as if symbols were positive reals (`sqrt(x^2)` is `x`). A
  sum is not assumed positive unless every term is.
* **Errors over guesses.** Integration, factoring and solving cover a stated class and return an error
  for anything else (`CannotIntegrate`, `Unsupported`, `ComplexRoots`, `Singular`, ...).

## Layout

| File | Content |
| --- | --- |
| `src/rat.rs` | checked `i128` rationals |
| `src/expr.rs` | the expression tree, canonical constructors, exact trigonometric values |
| `src/ops.rs` | substitute, evaluate, expand, simplify, differentiate, polynomial helpers |
| `src/parse.rs`, `src/print.rs` | text to expression, ASCII and LaTeX output |
| `src/integrate.rs`, `src/factor.rs`, `src/solve.rs` | integration, factoring, equations and linear systems |
| `tests/` | unit tests, the beam derivations, the numeric oracle property tests |
| `examples/` | `basics`, `beam_deflection`, `solve_for` |

## Verification

`tests/oracle.rs` generates random expressions with a seeded generator and compares every result with an
`f64` evaluation of the generator's own tree: build, simplify and expand must agree with it, derivatives must
match a central finite difference, `integrate` then `diff` must return the integrand, and definite
integrals must match composite Simpson quadrature. `tests/beam.rs` derives the deflection of a simply
supported beam under a uniform load (mid-span `5*w*L^4/(384*E*I)`) and of a cantilever with a tip load
(`P*L^3/(3*E*I)`) end to end.

## Limits

The full list is in `caskit/README.md`. The main ones: no complex numbers, only `sin`, `cos`, `ln`, `exp`
and `sqrt`, no cancellation of sums between numerator and denominator in `simplify`, integration of a small
class of integrands, and solving of degree 1 and 2 only.

## Browser build

The crate has no dependencies and no platform calls, and builds for `wasm32-unknown-unknown`. A
wasm-bindgen module loaded on demand (like the linear algebra module) is a later step.
