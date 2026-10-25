# caskit

`caskit/` is a small pure-Rust computer algebra library, published as the crate `civilkit-calcs-caskit`
(library name `caskit`). The engine (`calcpad-core`) is numeric only; caskit is the symbolic layer for
formulas that must stay exact and in terms of symbols: rearranging a formula, integrating a bending moment
twice, solving for the constants of integration.

Status: phase 1. The crate is a workspace member and has tests, but it is not released or published. The web
app uses it through the Symbolic dialog (see "In the app" below). Nothing in the engine or the other crates
depends on it.

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

The crate has no dependencies and no platform calls, and builds for `wasm32-unknown-unknown`. The web app
loads it through `calcpad-caskit-wasm`, a thin wasm-bindgen module with one entry point,
`run(op, expr, args_json) -> json`, built by `tools/build_wasm.sh caskit` at the `wasm-size` profile
(290 KB raw, 122 KB gzipped). It runs in a worker (`symbolic-worker.ts`) and is requested only when the
Symbolic dialog first runs something. The module answers `{"ok": true, "ascii", "latex", "items"}` or
`{"ok": false, "error"}` with the library's own message; a release wasm build aborts on a panic instead of
unwinding, so the boundary relies on the library returning errors, which it does for every documented limit.

## In the app

**Functions > Symbolic** opens the Symbolic dialog.

1. Type an expression, or an equation with `=`, in the box. Write `*` for every product, `^` for powers; the
   functions are `sin`, `cos`, `ln`, `exp`, `sqrt` and the constants are `pi` and `e`. Enter runs; Shift+Enter
   starts a new line.
2. Choose the operation. **Simplify**, **Expand** and **Factor** need nothing more. **Differentiate** takes the
   variable and an order. **Integrate** takes the variable and, for a definite integral, both limits (each an
   expression such as `0` or `L/2`). **Solve for** takes one variable for an equation (a quadratic or a single
   power gives both roots), or several variables separated by commas for a linear system whose equations are
   separated by `;` or new lines. **Substitute** takes `x = 2, b = 3*a` and returns the exact result.
   **Evaluate** takes a number for every variable and returns a decimal number to 12 digits.
3. The result is shown typeset (KaTeX, loaded with the result; the text form is shown if it cannot load) and
   as text, with a **Copy** button for the text.
4. **Insert into worksheet** writes the result at the cursor of the open worksheet.
   * In a **Calcpad worksheet**: a calculation line `name = expression` (caskit's text is already Calcpad
     expression syntax; only a negative exponent gets brackets), or a comment line with the result. Two roots of
     an equation are numbered `L_1`, `L_2`.
   * In a **sheet**: a display formula block (`$$ ... $$`, typeset in the preview) or a quantity line
     `name: expression`.

   A calculation or quantity line is calculated by the engine, so every symbol in it needs a value in the
   worksheet; use the formula block or the comment to keep a result in terms of symbols.

A refusal is shown in words with the library's message (for example "Cannot integrate: power of a non-linear
base."); nothing is guessed. The limits are those of the library, listed above.
