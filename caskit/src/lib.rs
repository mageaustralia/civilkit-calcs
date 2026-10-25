//! # caskit
//!
//! A small, pure-Rust computer algebra library for engineering formulas, with no dependencies. It is the
//! symbolic layer built for [CivilKit Calcs](https://civilkit.io/calcs/) (source:
//! <https://github.com/mageaustralia/civilkit-calcs>), and it works as a standalone crate.
//!
//! Numbers are exact rationals on `i128`; a number that does not fit is an error (`CasError::Overflow`),
//! never a wrong value. Every expression is kept in a canonical form, so equal expressions compare equal
//! with `==`, and `1/(E*I)` is the product `E^-1 * I^-1`.
//!
//! ## Quick start
//!
//! ```
//! use caskit::{parse, solve, parse_equation};
//!
//! // parse and simplify: like terms and powers combine, so this reduces to exactly zero
//! let e = parse("-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)")?.simplify()?;
//! assert!(e.is_zero());
//!
//! // differentiate and integrate
//! let f = parse("x^3 + sin(2*x)")?;
//! assert_eq!(f.diff("x")?.simplify()?, parse("3*x^2 + 2*cos(2*x)")?.simplify()?);
//! assert_eq!(parse("a*x^2")?.integrate("x")?, parse("a*x^3/3")?);
//!
//! // solve and rearrange: M = w*L^2/8 for w
//! let w = solve(&parse_equation("M = w*L^2/8")?, "w")?;
//! assert_eq!(w[0].to_ascii(), "8*M/L^2");
//!
//! // evaluate numerically
//! let mut v = std::collections::HashMap::new();
//! v.insert("x".to_string(), 3.0);
//! assert_eq!(parse("x^2 + 1")?.eval(&v)?, 10.0);
//! # Ok::<(), caskit::CasError>(())
//! ```
//!
//! ## Simply supported beam in a few lines
//!
//! ```
//! use caskit::{parse, solve_linear_system, Equation, Expr};
//!
//! // E*I*y'' = -(w*L*x/2 - w*x^2/2); y is measured in the direction of the load
//! let ypp = parse("-(w*L*x/2 - w*x^2/2)/(E*I)")?;
//! let yp = Expr::add(ypp.integrate("x")?, parse("C1")?)?;
//! let y = Expr::add(yp.integrate("x")?, parse("C2")?)?;
//! // y(0) = 0 and y(L) = 0 give C1 and C2
//! let c = solve_linear_system(
//!     &[
//!         Equation::new(y.substitute("x", &parse("0")?)?, parse("0")?),
//!         Equation::new(y.substitute("x", &parse("L")?)?, parse("0")?),
//!     ],
//!     &["C1", "C2"],
//! )?;
//! let y = y.substitute_all(&[("C1", c[0].clone()), ("C2", c[1].clone())])?;
//! let mid = y.substitute("x", &parse("L/2")?)?.simplify()?;
//! assert_eq!(mid, parse("5*w*L^4/(384*E*I)")?.simplify()?);
//! # Ok::<(), caskit::CasError>(())
//! ```
//!
//! ## Conventions
//!
//! * Symbols are positive real quantities when rational powers are combined (`sqrt(x^2)` is `x`), see
//!   [`Expr`]. There are no complex numbers; a complex root is `CasError::ComplexRoots`.
//! * `simplify` expands and collects. It does not cancel a common factor of a numerator and a
//!   denominator that are sums; `factor` does, for polynomials in one symbol.
//! * Anything outside the supported forms is an explicit error, listed in the README.
//!
//! The library does not use floating point except in `Expr::eval` and `Rat::to_f64`.

#![warn(missing_docs)]

mod error;
mod expr;
mod factor;
mod integrate;
mod ops;
mod parse;
mod print;
mod rat;
pub mod solve;

pub use error::{CasError, CasResult};
pub use expr::{Const, Expr, Func};
pub use parse::{parse, parse_equation, Equation};
pub use rat::Rat;
pub use solve::{solve, solve_expr, solve_linear_system};
