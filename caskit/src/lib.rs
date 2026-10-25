//! caskit: a small computer algebra library for engineering formulas.
//! (Crate documentation is written in the final pass.)
pub mod error;
pub mod expr;
pub mod factor;
pub mod integrate;
pub mod ops;
pub mod parse;
pub mod print;
pub mod rat;
pub mod solve;
pub use error::*;
pub use expr::*;
pub use parse::*;
pub use rat::Rat;
pub use solve::{solve, solve_expr, solve_linear_system};
