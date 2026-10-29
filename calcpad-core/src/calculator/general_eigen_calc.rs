//! `eigenvals`, `eigenvecs` and `eigen` of a full (not symmetric) matrix, and `ceigenvals` / `ceigenvecs`.
//!
//! A full matrix whose transpose is equal to it goes to the symmetric solver as before. Any other square matrix
//! goes to `matrix::general_eigen`. Vectors and matrices hold real values only, so a complex result is written the
//! way `fft` writes one: a real part row followed by an imaginary part row.
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::general_eigen::{eigen_pairs, EigenPair};
use crate::matrix::{symmetric_matrix, Matrix};
use crate::vector::Vector;

fn complex_eigenvalues() -> MathParserException {
    MathParserException::new(
        "The matrix has complex eigenvalues. Use ceigenvals and ceigenvecs for the real and imaginary parts.",
    )
}

/// A copy of `m` as a symmetric matrix if it equals its transpose (to a relative 1e-12), `None` if it does not.
pub fn as_symmetric(m: &Matrix) -> CResult<Option<Matrix>> {
    let n = m.row_count();
    if n != m.col_count() {
        return Err(exceptions::MatrixNotSquare());
    }
    let mut scale = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            scale = scale.max(m.get(i, j).d.abs());
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            let diff = RealValue::sub(&m.get(i, j), &m.get(j, i))?;
            if diff.d.abs() > 1e-12 * scale {
                return Ok(None);
            }
        }
    }
    let s = symmetric_matrix::new_checked(n)?;
    for i in 0..n {
        for j in i..n {
            s.set(i, j, m.get(i, j))?;
        }
    }
    Ok(Some(s))
}

/// The eigenpairs of square `m` with every element converted to the units of its first nonzero element.
fn pairs_of(m: &Matrix) -> CResult<(Vec<EigenPair>, OptUnit)> {
    let n = m.row_count();
    if n != m.col_count() {
        return Err(exceptions::MatrixNotSquare());
    }
    let mut units: OptUnit = None;
    let mut found = false;
    let mut grid = vec![vec![0.0; n]; n];
    for (i, row) in grid.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            let v = m.get(i, j);
            if v.d == 0.0 {
                continue;
            }
            if !found {
                units = v.units.clone();
                found = true;
            }
            *x = v.d * Unit::convert(&units, &v.units, '+')?;
            if !x.is_finite() {
                return Err(exceptions::ArgumentOutOfRange("eigen"));
            }
        }
    }
    let pairs = eigen_pairs(&grid).ok_or_else(|| exceptions::ArgumentOutOfRange("eigen"))?;
    Ok((pairs, units))
}

/// `eigenvals` (2), `eigenvecs` (3) or `eigen` (4) of a full matrix that is not symmetric. The eigenvalues must
/// all be real. `count` follows the symmetric functions: the first `count` in ascending order, the last `-count`
/// in descending order if negative, all if zero.
pub fn real_eigen(index: i64, m: &Matrix, count: i64) -> CResult<Value> {
    let (mut pairs, units) = pairs_of(m)?;
    if pairs.iter().any(|p| p.0 .1 != 0.0) {
        return Err(complex_eigenvalues());
    }
    let n = pairs.len();
    let mut count = count;
    if count < 0 {
        count = -count;
        pairs.reverse();
    }
    let count = if count == 0 || count as usize > n {
        n
    } else {
        count as usize
    };
    pairs.truncate(count);
    let value = |p: &EigenPair| RealValue::new(p.0 .0, units.clone());
    match index {
        2 => Ok(Value::Vector(Vector::from_values(pairs.iter().map(value).collect())?)),
        _ => {
            let j0 = if index == 4 { 1 } else { 0 };
            let r = Matrix::new(count, n + j0)?;
            for (i, p) in pairs.iter().enumerate() {
                if j0 == 1 {
                    r.set(i, 0, value(p))?;
                }
                for (j, c) in p.1.iter().enumerate() {
                    r.set(i, j + j0, RealValue::num(c.0))?;
                }
            }
            Ok(Value::Matrix(r))
        }
    }
}

/// `ceigenvals(M)`: a 2 x n matrix, real parts of the eigenvalues in row 1 and imaginary parts in row 2.
/// `ceigenvecs(M)`: a 2n x n matrix, eigenvector k as row 2k - 1 (real parts) and row 2k (imaginary parts).
/// Ascending by real part, then imaginary part.
pub fn complex_eigen(m: &Matrix, vectors: bool) -> CResult<Value> {
    let (pairs, units) = pairs_of(m)?;
    let n = pairs.len();
    if !vectors {
        let r = Matrix::new(2, n)?;
        for (k, p) in pairs.iter().enumerate() {
            r.set(0, k, RealValue::new(p.0 .0, units.clone()))?;
            r.set(1, k, RealValue::new(p.0 .1, units.clone()))?;
        }
        return Ok(Value::Matrix(r));
    }
    let r = Matrix::new(2 * n, n)?;
    for (k, p) in pairs.iter().enumerate() {
        for (j, c) in p.1.iter().enumerate() {
            r.set(2 * k, j, RealValue::num(c.0))?;
            r.set(2 * k + 1, j, RealValue::num(c.1))?;
        }
    }
    Ok(Value::Matrix(r))
}
