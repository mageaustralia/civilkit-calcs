//! `eigenvals(K; M)`, `eigenvecs(K; M)` and `eigen(K; M)`: the generalized symmetric eigenproblem K x = λ M x.
//!
//! K must be symmetric and M symmetric positive definite (a stiffness and a mass matrix give λ = ω²). M is split
//! as L Lᵀ by Cholesky, the problem becomes the standard symmetric one C y = λ y with C = L⁻¹ K L⁻ᵀ, which the
//! symmetric solver of the engine handles, and x = L⁻ᵀ y. Each x is M-orthonormal (xᵀ M x = 1) and turned so that
//! its component of largest magnitude is positive. Ascending order, as for one matrix.
//!
//! Units: every element of K is converted to the units of its first nonzero element, and likewise for M. λ gets
//! the units of K / M and the components of x the units of M^-0.5, so that xᵀ M x = 1 holds in the sheet.
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::{symmetric_matrix, Matrix};
use crate::vector::Vector;

/// The elements of square `m` as numbers in the units of its first nonzero element.
fn grid_of(m: &Matrix) -> CResult<(Vec<Vec<f64>>, OptUnit)> {
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
    Ok((grid, units))
}

fn is_symmetric(a: &[Vec<f64>]) -> bool {
    let scale = a.iter().flatten().fold(0.0f64, |s, x| s.max(x.abs()));
    (0..a.len()).all(|i| (i + 1..a.len()).all(|j| (a[i][j] - a[j][i]).abs() <= 1e-12 * scale))
}

/// Lower triangular L with L Lᵀ = a, `None` unless `a` is positive definite. A pivot that is not clearly above
/// rounding (relative to the largest diagonal element) counts as zero, so a massless degree of freedom is caught.
fn cholesky(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = a.len();
    let big = (0..n).fold(0.0f64, |s, i| s.max(a[i][i].abs()));
    let tiny = 1e-13 * big * n as f64;
    let mut l = vec![vec![0.0; n]; n];
    for j in 0..n {
        let s = a[j][j] - (0..j).map(|k| l[j][k] * l[j][k]).sum::<f64>();
        if s <= tiny {
            return None;
        }
        let d = s.sqrt();
        l[j][j] = d;
        for i in j + 1..n {
            l[i][j] = (a[i][j] - (0..j).map(|k| l[i][k] * l[j][k]).sum::<f64>()) / d;
        }
    }
    Some(l)
}

/// Solves L y = b in place (forward substitution).
fn forward(l: &[Vec<f64>], b: &mut [f64]) {
    for i in 0..b.len() {
        b[i] = (b[i] - (0..i).map(|k| l[i][k] * b[k]).sum::<f64>()) / l[i][i];
    }
}

/// Solves Lᵀ x = b in place (back substitution).
fn backward(l: &[Vec<f64>], b: &mut [f64]) {
    for i in (0..b.len()).rev() {
        b[i] = (b[i] - (i + 1..b.len()).map(|k| l[k][i] * b[k]).sum::<f64>()) / l[i][i];
    }
}

fn not_spd() -> MathParserException {
    MathParserException::new("The second matrix (M) must be symmetric and positive definite.")
}

/// All eigenpairs (λ, x) of K x = λ M x in plain numbers, ascending, x M-orthonormal with its largest component
/// positive. `tol` is passed on to the symmetric solver.
pub fn generalized_pairs(k: &[Vec<f64>], m: &[Vec<f64>], tol: f64) -> CResult<Vec<(f64, Vec<f64>)>> {
    let n = k.len();
    if m.len() != n {
        return Err(exceptions::MatrixDimensions());
    }
    if !is_symmetric(k) {
        return Err(MathParserException::new("The first matrix (K) must be symmetric."));
    }
    if !is_symmetric(m) {
        return Err(not_spd());
    }
    let l = cholesky(m).ok_or_else(not_spd)?;
    // C = L⁻¹ K L⁻ᵀ: first X = L⁻¹ K column by column, then the rows of C solve L cᵢ = (row i of X)ᵀ.
    let mut x = vec![vec![0.0; n]; n];
    for j in 0..n {
        let mut col: Vec<f64> = (0..n).map(|i| k[i][j]).collect();
        forward(&l, &mut col);
        for i in 0..n {
            x[i][j] = col[i];
        }
    }
    for row in x.iter_mut() {
        forward(&l, row);
    }
    let c = symmetric_matrix::new_checked(n)?;
    for i in 0..n {
        for j in i..n {
            c.set(i, j, RealValue::num(0.5 * (x[i][j] + x[j][i])))?;
        }
    }
    let e = c.eigen(0, tol)?;
    let mut pairs = Vec::with_capacity(n);
    for r in 0..e.row_count() {
        let mut v: Vec<f64> = (0..n).map(|j| e.get(r, j + 1).d).collect();
        backward(&l, &mut v);
        let mass: f64 = (0..n).map(|i| v[i] * (0..n).map(|j| m[i][j] * v[j]).sum::<f64>()).sum();
        let mut big = 0.0f64;
        for &c in v.iter() {
            if c.abs() > big.abs() * (1.0 + 1e-12) {
                big = c;
            }
        }
        let s = if big < 0.0 { -1.0 } else { 1.0 } / mass.sqrt();
        for c in v.iter_mut() {
            *c *= s;
        }
        pairs.push((e.get(r, 0).d, v));
    }
    Ok(pairs)
}

/// `eigenvals` (2), `eigenvecs` (3) or `eigen` (4) of the pair (K, M). Same layout as for one matrix: a vector of
/// eigenvalues, one eigenvector per row, or the eigenvalue in column 1 followed by the eigenvector.
pub fn generalized_eigen(index: i64, k: &Matrix, m: &Matrix, tol: f64) -> CResult<Value> {
    let (kg, ku) = grid_of(k)?;
    let (mg, mu) = grid_of(m)?;
    let pairs = generalized_pairs(&kg, &mg, tol)?;
    let n = pairs.len();
    let unit_m = RealValue::new(1.0, mu.clone());
    let lambda = |l: f64| RealValue::div_op(&RealValue::new(l, ku.clone()), &unit_m);
    let xu: OptUnit = mu.as_ref().and_then(|u| u.pow(-0.5));
    match index {
        2 => Ok(Value::Vector(Vector::from_values(pairs.iter().map(|p| lambda(p.0)).collect())?)),
        _ => {
            let j0 = if index == 4 { 1 } else { 0 };
            let r = Matrix::new(n, n + j0)?;
            for (i, p) in pairs.iter().enumerate() {
                if j0 == 1 {
                    r.set(i, 0, lambda(p.0))?;
                }
                for (j, c) in p.1.iter().enumerate() {
                    r.set(i, j + j0, RealValue::new(*c, xu.clone()))?;
                }
            }
            Ok(Value::Matrix(r))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-14;

    fn mul(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
        a.iter().map(|r| r.iter().zip(x).map(|(p, q)| p * q).sum()).collect()
    }

    /// max over the pairs of |K x - λ M x| / (|K| |x|), and max |xᵢᵀ M xⱼ - δᵢⱼ|.
    fn residuals(k: &[Vec<f64>], m: &[Vec<f64>], p: &[(f64, Vec<f64>)]) -> (f64, f64) {
        let kn = k.iter().flatten().fold(0.0f64, |s, x| s.max(x.abs())).max(1e-300);
        let mut res = 0.0f64;
        for (l, x) in p {
            let (kx, mx) = (mul(k, x), mul(m, x));
            let xn = x.iter().fold(0.0f64, |s, c| s.max(c.abs()));
            for i in 0..x.len() {
                res = res.max((kx[i] - l * mx[i]).abs() / (kn * xn));
            }
        }
        let mut orth = 0.0f64;
        for (i, (_, xi)) in p.iter().enumerate() {
            for (j, (_, xj)) in p.iter().enumerate() {
                let d: f64 = xi.iter().zip(mul(m, xj)).map(|(a, b)| a * b).sum();
                orth = orth.max((d - if i == j { 1.0 } else { 0.0 }).abs());
            }
        }
        (res, orth)
    }

    #[test]
    fn spring_mass_chain_matches_closed_form() {
        // Three equal masses m on equal springs k, fixed at one end, free at the other:
        // ω² = 4 k/m sin²((2r - 1) π / (2 (2n + 1))), r = 1..n.
        let (kk, mm) = (1000.0, 2.0);
        let k = vec![vec![2.0 * kk, -kk, 0.0], vec![-kk, 2.0 * kk, -kk], vec![0.0, -kk, kk]];
        let m = vec![vec![mm, 0.0, 0.0], vec![0.0, mm, 0.0], vec![0.0, 0.0, mm]];
        let p = generalized_pairs(&k, &m, TOL).unwrap();
        for (r, q) in p.iter().enumerate() {
            let s = ((2 * r + 1) as f64 * std::f64::consts::PI / 14.0).sin();
            let want = 4.0 * kk / mm * s * s;
            assert!((q.0 - want).abs() < 1e-10 * want, "{} vs {want}", q.0);
            // Largest component positive.
            let big = q.1.iter().cloned().fold(0.0f64, |a, c| if c.abs() > a.abs() { c } else { a });
            assert!(big > 0.0);
        }
        // The first mode moves every mass the same way, more towards the free end.
        let x = &p[0].1;
        assert!(x[0] > 0.0 && x[1] > x[0] && x[2] > x[1]);
        let (res, orth) = residuals(&k, &m, &p);
        assert!(res < 1e-12 && orth < 1e-12, "{res} {orth}");
    }

    #[test]
    fn random_spd_pairs_satisfy_definition() {
        let mut seed = 2024u64;
        let mut rnd = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
        };
        for n in 1..10 {
            let a: Vec<Vec<f64>> = (0..n).map(|_| (0..n).map(|_| rnd()).collect()).collect();
            let b: Vec<Vec<f64>> = (0..n).map(|_| (0..n).map(|_| rnd()).collect()).collect();
            // K symmetric (indefinite), M = B Bᵀ + n I (positive definite, not diagonal).
            let k: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| a[i][j] + a[j][i]).collect()).collect();
            let m: Vec<Vec<f64>> = (0..n)
                .map(|i| {
                    (0..n)
                        .map(|j| (0..n).map(|q| b[i][q] * b[j][q]).sum::<f64>() + if i == j { n as f64 } else { 0.0 })
                        .collect()
                })
                .collect();
            let p = generalized_pairs(&k, &m, TOL).unwrap();
            assert_eq!(p.len(), n);
            assert!(p.windows(2).all(|w| w[0].0 <= w[1].0), "ascending");
            let (res, orth) = residuals(&k, &m, &p);
            assert!(res < 1e-11 && orth < 1e-11, "n = {n}: {res} {orth}");
        }
    }

    #[test]
    fn identity_mass_gives_the_standard_problem() {
        let k = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let m = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let p = generalized_pairs(&k, &m, TOL).unwrap();
        assert!((p[0].0 - 1.0).abs() < 1e-12 && (p[1].0 - 3.0).abs() < 1e-12);
    }

    #[test]
    fn bad_input_is_refused_with_a_clear_message() {
        let k = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let msg = |r: CResult<Vec<(f64, Vec<f64>)>>| r.err().map(|e| e.message).unwrap_or_default();
        // Not positive definite, singular (massless degree of freedom), not symmetric.
        assert!(msg(generalized_pairs(&k, &[vec![1.0, 2.0], vec![2.0, 1.0]], TOL)).contains("positive definite"));
        assert!(msg(generalized_pairs(&k, &[vec![1.0, 0.0], vec![0.0, 0.0]], TOL)).contains("positive definite"));
        assert!(msg(generalized_pairs(&k, &[vec![1.0, 0.5], vec![0.0, 1.0]], TOL)).contains("positive definite"));
        assert!(msg(generalized_pairs(&[vec![2.0, 1.0], vec![0.0, 2.0]], &k, TOL)).contains("(K) must be symmetric"));
        assert!(generalized_pairs(&k, &[vec![1.0]], TOL).is_err());
    }
}
