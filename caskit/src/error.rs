//! The error type shared by every operation. Operations that cannot give a right answer return one of
//! these instead of a wrong or half-simplified result.

use std::fmt;

/// Why an operation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CasError {
    /// The text could not be parsed. `pos` is a byte offset into the input.
    Parse { msg: String, pos: usize },
    /// A division by zero, or zero raised to a negative power.
    DivisionByZero,
    /// An exact rational number does not fit in 128 bits.
    Overflow,
    /// A function is outside its domain (for example `ln(0)`).
    Domain(String),
    /// The integrand is not in the supported forms.
    CannotIntegrate(String),
    /// The operation is outside what the library supports.
    Unsupported(String),
    /// A symbol had no value during numeric evaluation.
    UnboundSymbol(String),
    /// The equation holds for every value of the unknown.
    Identity,
    /// The equation holds for no value of the unknown.
    NoSolution,
    /// The only roots are complex numbers, which the library does not represent.
    ComplexRoots,
    /// The linear system has no unique solution (its determinant is zero).
    Singular,
    /// An expansion would produce more terms than the safety limit allows.
    TooLarge,
}

impl fmt::Display for CasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CasError::Parse { msg, pos } => write!(f, "parse error at {pos}: {msg}"),
            CasError::DivisionByZero => write!(f, "division by zero"),
            CasError::Overflow => write!(f, "exact number too large (does not fit in 128 bits)"),
            CasError::Domain(s) => write!(f, "domain error: {s}"),
            CasError::CannotIntegrate(s) => write!(f, "cannot integrate: {s}"),
            CasError::Unsupported(s) => write!(f, "unsupported: {s}"),
            CasError::UnboundSymbol(s) => write!(f, "no value for symbol {s}"),
            CasError::Identity => write!(f, "the equation is an identity (true for every value)"),
            CasError::NoSolution => write!(f, "the equation has no solution"),
            CasError::ComplexRoots => write!(f, "the roots are complex (not supported)"),
            CasError::Singular => write!(f, "the system is singular (no unique solution)"),
            CasError::TooLarge => write!(f, "expression too large to expand"),
        }
    }
}

impl std::error::Error for CasError {}

/// Result of a symbolic operation.
pub type CasResult<T> = Result<T, CasError>;
