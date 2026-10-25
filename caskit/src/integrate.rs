//! Symbolic integration of a deliberately small class of integrands. Anything outside it is an error,
//! never a guess.
//!
//! Supported (after expansion, term by term, with coefficients free of the variable):
//! constants, `x^n` for rational `n` (`n = -1` gives `ln(x)`), powers of a linear form `(a*x+b)^n`,
//! `sin`, `cos`, `exp` and `ln` of a linear form, and `x^k * F` for a whole number `k >= 1` and `F` one of
//! `sin`, `cos`, `exp` of a linear form (by parts). `ln(u)` stands for the antiderivative on `u > 0`.

use crate::error::{CasError, CasResult};
use crate::expr::{Expr, Func};
use crate::rat::Rat;

fn cannot(what: &str) -> CasError {
    CasError::CannotIntegrate(what.to_string())
}

impl Expr {
    /// An antiderivative with respect to `var`, without a constant of integration. Terms that are
    /// logarithms of `x` are valid for `x > 0` only. Returns `CannotIntegrate` outside the supported forms.
    pub fn integrate(&self, var: &str) -> CasResult<Expr> {
        let ex = self.expand()?;
        let terms: Vec<Expr> = match ex {
            Expr::Add(v) => v,
            other => vec![other],
        };
        let mut out = Vec::with_capacity(terms.len());
        for t in terms {
            out.push(integrate_term(&t, var)?);
        }
        Expr::sum(out)?.expand()
    }

    /// The definite integral from `a` to `b`, by substituting the limits into the antiderivative.
    /// It does not check the integrand for singularities between the limits.
    pub fn integrate_definite(&self, var: &str, a: &Expr, b: &Expr) -> CasResult<Expr> {
        let f = self.integrate(var)?;
        Expr::sub(f.substitute(var, b)?, f.substitute(var, a)?)?.simplify()
    }
}

fn integrate_term(t: &Expr, var: &str) -> CasResult<Expr> {
    let factors: Vec<Expr> = match t {
        Expr::Mul(fs) => fs.clone(),
        other => vec![other.clone()],
    };
    let (dep, free): (Vec<Expr>, Vec<Expr>) = factors.into_iter().partition(|f| f.has_symbol(var));
    let coef = Expr::product(free)?;
    if dep.is_empty() {
        return Expr::mul(coef, Expr::sym(var));
    }
    let anti = match dep.len() {
        1 => antiderivative(&dep[0], var)?,
        2 => by_parts(&dep, var)?,
        _ => return Err(cannot("product of several functions of the variable")),
    };
    Expr::mul(coef, anti)
}

/// `x^k` with whole `k >= 1`.
fn var_power(f: &Expr, var: &str) -> Option<i64> {
    match f {
        Expr::Sym(s) if s == var => Some(1),
        Expr::Pow(b, e) if matches!(&**b, Expr::Sym(s) if s == var) => match &**e {
            Expr::Num(r) if r.is_integer() && r.is_positive() && r.numer() < 1000 => Some(r.numer() as i64),
            _ => None,
        },
        _ => None,
    }
}

fn by_parts(dep: &[Expr], var: &str) -> CasResult<Expr> {
    let (pi, k) = if let Some(k) = var_power(&dep[0], var) {
        (0, k)
    } else if let Some(k) = var_power(&dep[1], var) {
        (1, k)
    } else {
        return Err(cannot("product of two functions of the variable"));
    };
    let f = &dep[1 - pi];
    let is_transcendental =
        matches!(f, Expr::Func(Func::Sin | Func::Cos, _)) || matches!(f, Expr::Pow(b, _) if **b == Expr::e());
    if !is_transcendental {
        return Err(cannot("polynomial times a function other than sin, cos or exp"));
    }
    let g = antiderivative(f, var)?;
    let x = Expr::sym(var);
    let first = Expr::mul(Expr::pow(x.clone(), Expr::int(k))?, g.clone())?;
    let inner = Expr::product(vec![Expr::int(k), Expr::pow(x, Expr::int(k - 1))?, g])?;
    Expr::sub(first, inner.integrate(var)?)
}

fn antiderivative(f: &Expr, var: &str) -> CasResult<Expr> {
    let x = Expr::sym(var);
    match f {
        Expr::Sym(s) if s == var => Expr::div(Expr::pow(x, Expr::int(2))?, Expr::int(2)),
        Expr::Pow(b, e) if **b == Expr::e() => {
            let (a, _) = e.linear_in(var).ok_or_else(|| cannot("exp of a non-linear argument"))?;
            Expr::div(f.clone(), a)
        }
        Expr::Pow(b, e) if !e.has_symbol(var) => {
            let n = match &**e {
                Expr::Num(n) => *n,
                _ => return Err(cannot("symbolic exponent")),
            };
            let (a, _) = b.linear_in(var).ok_or_else(|| cannot("power of a non-linear base"))?;
            if n == Rat::int(-1) {
                Expr::div(Expr::ln((**b).clone())?, a)
            } else {
                let n1 = n.add(Rat::ONE)?;
                let p = Expr::pow((**b).clone(), Expr::Num(n1))?;
                Expr::div(p, Expr::mul(a, Expr::Num(n1))?)
            }
        }
        Expr::Pow(..) => Err(cannot("variable in the exponent")),
        Expr::Func(func, arg) => {
            let (a, _) = arg
                .linear_in(var)
                .ok_or_else(|| cannot("function of a non-linear argument"))?;
            let inner = match func {
                Func::Sin => Expr::neg(Expr::cos((**arg).clone())?)?,
                Func::Cos => Expr::sin((**arg).clone())?,
                Func::Ln => Expr::sub(Expr::mul((**arg).clone(), Expr::ln((**arg).clone())?)?, (**arg).clone())?,
            };
            Expr::div(inner, a)
        }
        _ => Err(cannot("unsupported form")),
    }
}
