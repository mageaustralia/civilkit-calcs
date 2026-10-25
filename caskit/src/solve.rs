//! Solving equations for one unknown, and small linear systems.
//!
//! Branch convention for rearranging a power: `var^(m/n) = V` (lowest terms, `n > 0`) gives
//! `var = V^(n/m)`; when `m` is even the negative of that value is also returned, listed second.
//! Symbols are positive quantities (see the notes on `Expr`), so `V^(n/m)` is the positive root.
//! For a quadratic with a symbolic discriminant both roots are returned, `-` branch of the square root
//! first; the discriminant may be negative for some values of the symbols. A fully numeric quadratic
//! returns its roots in ascending order, and a negative numeric discriminant is `ComplexRoots`.

use crate::error::{CasError, CasResult};
use crate::expr::Expr;
use crate::parse::Equation;
use crate::rat::Rat;

/// Solves `eq` for `var`. See the module notes for the supported equations and the order of the roots.
pub fn solve(eq: &Equation, var: &str) -> CasResult<Vec<Expr>> {
    solve_expr(&eq.residual()?, var)
}

/// Solves `expr = 0` for `var`.
pub fn solve_expr(expr: &Expr, var: &str) -> CasResult<Vec<Expr>> {
    let r = expr.expand()?;
    if !r.has_symbol(var) {
        return Err(if r.is_zero() {
            CasError::Identity
        } else {
            CasError::NoSolution
        });
    }
    let terms = r
        .var_terms(var)
        .ok_or_else(|| CasError::Unsupported(format!("the equation is not a polynomial or a single power of {var}")))?;
    let nonzero: Vec<Rat> = terms.keys().filter(|p| !p.is_zero()).cloned().collect();
    if nonzero.len() == 1 {
        let p = nonzero[0];
        let a = terms[&p].clone();
        let b = terms.get(&Rat::ZERO).cloned().unwrap_or_else(|| Expr::int(0));
        let v = Expr::neg(Expr::div(b, a)?)?.expand()?;
        return power_roots(&v, p);
    }
    if terms.keys().any(|p| !p.is_integer()) {
        return Err(CasError::Unsupported(
            "fractional powers of the unknown in a sum".into(),
        ));
    }
    let lo = *terms.keys().next().unwrap();
    let shift = if lo.is_negative() { -lo.numer() } else { 0 };
    let hi = terms.keys().next_back().unwrap().numer() + shift;
    let coef = |i: i128| -> Expr {
        Rat::new(i - shift, 1)
            .ok()
            .and_then(|k| terms.get(&k).cloned())
            .unwrap_or_else(|| Expr::int(0))
    };
    match hi {
        1 => Ok(vec![Expr::neg(Expr::div(coef(0), coef(1))?)?.expand()?]),
        2 => quadratic(&coef(2), &coef(1), &coef(0)),
        d => Err(CasError::Unsupported(format!(
            "equations of degree {d}; only degree 1 and 2 are solved"
        ))),
    }
}

/// Roots of `var^p = v`.
fn power_roots(v: &Expr, p: Rat) -> CasResult<Vec<Expr>> {
    let (m, n) = (p.numer(), p.denom());
    let q = Rat::new(n, m)?;
    let even_m = m % 2 == 0;
    let root = match v {
        Expr::Num(x) if x.is_zero() => {
            return if p.is_positive() {
                Ok(vec![Expr::int(0)])
            } else {
                Err(CasError::NoSolution)
            };
        }
        Expr::Num(x) if x.is_negative() => {
            if n % 2 == 0 {
                return Err(CasError::NoSolution);
            }
            if even_m {
                return Err(CasError::ComplexRoots);
            }
            Expr::neg(Expr::pow(Expr::Num(x.neg()?), Expr::Num(q))?)?
        }
        _ => Expr::pow(v.clone(), Expr::Num(q))?,
    };
    let root = root.expand()?;
    if even_m {
        Ok(vec![root.clone(), Expr::neg(root)?.expand()?])
    } else {
        Ok(vec![root])
    }
}

fn quadratic(a: &Expr, b: &Expr, c: &Expr) -> CasResult<Vec<Expr>> {
    if c.is_zero() {
        return Ok(vec![
            Expr::int(0),
            Expr::neg(Expr::div(b.clone(), a.clone())?)?.expand()?,
        ]);
    }
    let disc = Expr::sub(
        Expr::pow(b.clone(), Expr::int(2))?,
        Expr::product(vec![Expr::int(4), a.clone(), c.clone()])?,
    )?
    .expand()?;
    let two_a = Expr::mul(Expr::int(2), a.clone())?;
    if let Expr::Num(d) = &disc {
        if d.is_negative() {
            return Err(CasError::ComplexRoots);
        }
        if d.is_zero() {
            return Ok(vec![Expr::div(Expr::neg(b.clone())?, two_a)?.expand()?]);
        }
    }
    let s = Expr::sqrt(disc)?;
    let minus = Expr::div(Expr::sub(Expr::neg(b.clone())?, s.clone())?, two_a.clone())?.expand()?;
    let plus = Expr::div(Expr::add(Expr::neg(b.clone())?, s)?, two_a)?.expand()?;
    if let (Expr::Num(x), Expr::Num(y)) = (&minus, &plus) {
        if x > y {
            return Ok(vec![plus, minus]);
        }
    }
    Ok(vec![minus, plus])
}

/// Solves a square linear system with symbolic coefficients (1 to 4 unknowns) by Cramer's rule with
/// expanded determinants. The coefficients of the unknowns must not contain any unknown. A determinant
/// that is not provably non-zero is assumed to be non-zero; one that expands to exactly zero is `Singular`.
pub fn solve_linear_system(eqs: &[Equation], vars: &[&str]) -> CasResult<Vec<Expr>> {
    let n = vars.len();
    if n == 0 || n > 4 {
        return Err(CasError::Unsupported("a linear system needs 1 to 4 unknowns".into()));
    }
    if eqs.len() != n {
        return Err(CasError::Unsupported(
            "the number of equations must equal the number of unknowns".into(),
        ));
    }
    let zeros: Vec<(&str, Expr)> = vars.iter().map(|v| (*v, Expr::int(0))).collect();
    let mut a: Vec<Vec<Expr>> = Vec::new();
    let mut rhs: Vec<Expr> = Vec::new();
    for eq in eqs {
        let r = eq.residual()?.expand()?;
        let mut row = Vec::new();
        for v in vars {
            let c = r.diff(v)?.expand()?;
            if vars.iter().any(|w| c.has_symbol(w)) {
                return Err(CasError::Unsupported("the system is not linear in the unknowns".into()));
            }
            row.push(c);
        }
        a.push(row);
        rhs.push(Expr::neg(r.substitute_all(&zeros)?)?.expand()?);
    }
    let det = determinant(&a)?;
    if det.is_zero() {
        return Err(CasError::Singular);
    }
    let inv = Expr::recip(det)?;
    let mut out = Vec::new();
    for j in 0..n {
        let mut m = a.clone();
        for i in 0..n {
            m[i][j] = rhs[i].clone();
        }
        out.push(Expr::mul(determinant(&m)?, inv.clone())?.expand()?);
    }
    Ok(out)
}

fn determinant(m: &[Vec<Expr>]) -> CasResult<Expr> {
    let n = m.len();
    if n == 1 {
        return Ok(m[0][0].clone());
    }
    let mut terms = Vec::new();
    for j in 0..n {
        if m[0][j].is_zero() {
            continue;
        }
        let minor: Vec<Vec<Expr>> = m[1..]
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|(k, _)| *k != j)
                    .map(|(_, e)| e.clone())
                    .collect()
            })
            .collect();
        let t = Expr::mul(m[0][j].clone(), determinant(&minor)?)?.expand()?;
        terms.push(if j % 2 == 0 { t } else { Expr::neg(t)? });
    }
    Expr::sum(terms)?.expand()
}
