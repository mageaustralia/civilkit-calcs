//! Factoring: common factors of a sum, and polynomials in one symbol with rational coefficients by
//! rational roots.

use crate::error::CasResult;
use crate::expr::{split_term, Expr};
use crate::rat::Rat;
use std::collections::BTreeMap;

const MAX_DEGREE: i128 = 8;

fn gcd_u(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Expr {
    /// Factors the expression: pulls out the common numeric and symbolic factor of every sum, and splits a
    /// polynomial in one symbol (degree 2 to 8, rational coefficients) into the factors that its rational
    /// roots give. A remaining factor with no rational root is left as a sum. The product of the result,
    /// expanded, is the expanded input.
    pub fn factor(&self) -> CasResult<Expr> {
        match self {
            Expr::Mul(fs) => Expr::product(fs.iter().map(Expr::factor).collect::<CasResult<_>>()?),
            Expr::Pow(b, x) => Expr::pow(b.factor()?, (**x).clone()),
            Expr::Add(_) => factor_sum(&self.expand()?),
            other => Ok(other.clone()),
        }
    }
}

fn factor_pairs(rest: &Expr) -> Vec<(Expr, Expr)> {
    let fs: Vec<Expr> = match rest {
        Expr::Num(_) => vec![],
        Expr::Mul(v) => v.clone(),
        other => vec![other.clone()],
    };
    fs.into_iter()
        .map(|f| match f {
            Expr::Pow(b, e) => (*b, *e),
            other => (other, Expr::int(1)),
        })
        .collect()
}

fn factor_sum(sum: &Expr) -> CasResult<Expr> {
    let terms = match sum {
        Expr::Add(v) => v.clone(),
        other => return Ok(other.clone()),
    };
    // numeric part
    let mut split = Vec::new();
    for t in &terms {
        split.push(split_term(t.clone()));
    }
    let mut gn: u128 = 0;
    let mut ld: i128 = 1;
    for (c, _) in &split {
        gn = gcd_u(gn, c.numer().unsigned_abs());
        let d = c.denom();
        let g = gcd_u(ld as u128, d as u128) as i128;
        ld = (ld / g).checked_mul(d).ok_or(crate::CasError::Overflow)?;
    }
    let mut g = Rat::new(gn as i128, ld)?;
    if split.iter().all(|(c, _)| c.is_negative()) {
        g = g.neg()?;
    }
    // symbolic part: bases common to every term, with the smallest exponent
    let mut common: Option<BTreeMap<Expr, Expr>> = None;
    for (_, rest) in &split {
        let m: BTreeMap<Expr, Expr> = factor_pairs(rest).into_iter().collect();
        common = Some(match common {
            None => m,
            Some(prev) => prev
                .into_iter()
                .filter_map(|(b, e)| {
                    let o = m.get(&b)?;
                    match (&e, o) {
                        (Expr::Num(x), Expr::Num(y)) => Some((b, Expr::Num(*x.min(y)))),
                        _ if &e == o => Some((b, e)),
                        _ => None,
                    }
                })
                .collect(),
        });
    }
    let mut cf = vec![];
    for (b, e) in common.unwrap_or_default() {
        if !e.is_zero() {
            cf.push(Expr::pow(b, e)?);
        }
    }
    let cfp = Expr::product(cf)?;
    let inv = Expr::mul(Expr::Num(g.recip()?), Expr::recip(cfp.clone())?)?;
    let rem_terms: Vec<Expr> = terms
        .iter()
        .map(|t| Expr::mul(t.clone(), inv.clone()))
        .collect::<CasResult<_>>()?;
    let rem = Expr::sum(rem_terms)?;
    let rem_f = factor_poly_expr(&rem)?.unwrap_or(rem);
    Expr::product(vec![Expr::Num(g), cfp, rem_f])
}

/// If `e` is a polynomial in one symbol with rational coefficients, of degree 2..=8, returns it factored.
fn factor_poly_expr(e: &Expr) -> CasResult<Option<Expr>> {
    let syms = e.symbols();
    if syms.len() != 1 {
        return Ok(None);
    }
    let var = syms.iter().next().unwrap().clone();
    let Some(terms) = e.var_terms(&var) else {
        return Ok(None);
    };
    let mut deg: i128 = 0;
    for (p, c) in &terms {
        if !p.is_integer() || p.is_negative() || c.as_num().is_none() {
            return Ok(None);
        }
        deg = deg.max(p.numer());
    }
    if !(2..=MAX_DEGREE).contains(&deg) {
        return Ok(None);
    }
    let mut coeffs = vec![Rat::ZERO; deg as usize + 1];
    for (p, c) in &terms {
        coeffs[p.numer() as usize] = c.as_num().unwrap();
    }
    factor_poly(&coeffs, &var)
}

fn divisors(n: u128) -> Option<Vec<u128>> {
    if n == 0 || n > 1_000_000_000_000 {
        return None;
    }
    let mut out = Vec::new();
    let mut i: u128 = 1;
    while i * i <= n {
        if n.is_multiple_of(i) {
            out.push(i);
            if i * i != n {
                out.push(n / i);
            }
        }
        i += 1;
    }
    Some(out)
}

/// Sum of `a_i p^i q^(n-i)`, i.e. `q^n * P(p/q)`, or `None` on overflow.
fn eval_scaled(a: &[i128], p: i128, q: i128) -> Option<i128> {
    let n = a.len() - 1;
    let mut total: i128 = 0;
    for (i, &ai) in a.iter().enumerate() {
        let mut term = ai;
        for _ in 0..i {
            term = term.checked_mul(p)?;
        }
        for _ in 0..(n - i) {
            term = term.checked_mul(q)?;
        }
        total = total.checked_add(term)?;
    }
    Some(total)
}

fn find_root(a: &[i128]) -> Option<(i128, i128)> {
    if a[0] == 0 {
        return Some((0, 1));
    }
    let ps = divisors(a[0].unsigned_abs())?;
    let qs = divisors(a[a.len() - 1].unsigned_abs())?;
    for &q in &qs {
        for &p in &ps {
            if gcd_u(p, q) != 1 {
                continue;
            }
            for sign in [1i128, -1] {
                let (pp, qq) = (sign * p as i128, q as i128);
                if eval_scaled(a, pp, qq) == Some(0) {
                    return Some((pp, qq));
                }
            }
        }
    }
    None
}

/// Divides `P` (ascending integer coefficients) by `(q x - p)`; `None` when not exact or on overflow.
fn divide_linear(a: &[i128], p: i128, q: i128) -> Option<Vec<i128>> {
    let n = a.len() - 1;
    let mut b = vec![0i128; n];
    let mut carry: i128 = 0; // p * b_i from the previous step
    for i in (1..=n).rev() {
        let num = a[i].checked_add(carry)?;
        if num % q != 0 {
            return None;
        }
        b[i - 1] = num / q;
        carry = p.checked_mul(b[i - 1])?;
    }
    if a[0].checked_add(carry)? != 0 {
        return None;
    }
    Some(b)
}

fn factor_poly(coeffs: &[Rat], var: &str) -> CasResult<Option<Expr>> {
    let mut l: i128 = 1;
    for c in coeffs {
        let g = gcd_u(l as u128, c.denom() as u128) as i128;
        l = (l / g).checked_mul(c.denom()).ok_or(crate::CasError::Overflow)?;
    }
    let mut a: Vec<i128> = Vec::new();
    for c in coeffs {
        match c.numer().checked_mul(l / c.denom()) {
            Some(v) => a.push(v),
            None => return Ok(None),
        }
    }
    let content = a.iter().fold(0u128, |g, &v| gcd_u(g, v.unsigned_abs())) as i128;
    for v in a.iter_mut() {
        *v /= content;
    }
    let k = Rat::new(content, l)?;
    let x = Expr::sym(var);
    let mut factors = vec![Expr::Num(k)];
    let mut found = false;
    while a.len() > 1 {
        let Some((p, q)) = find_root(&a) else { break };
        let Some(b) = divide_linear(&a, p, q) else { break };
        found = true;
        factors.push(Expr::sum(vec![
            Expr::mul(Expr::Num(Rat::new(q, 1)?), x.clone())?,
            Expr::Num(Rat::new(-p, 1)?),
        ])?);
        a = b;
    }
    if !found {
        return Ok(None);
    }
    if a.len() > 1 {
        let mut terms = Vec::new();
        for (i, &c) in a.iter().enumerate() {
            terms.push(Expr::mul(
                Expr::Num(Rat::new(c, 1)?),
                Expr::pow(x.clone(), Expr::int(i as i64))?,
            )?);
        }
        factors.push(Expr::sum(terms)?);
    } else {
        factors.push(Expr::Num(Rat::new(a[0], 1)?));
    }
    Ok(Some(Expr::product(factors)?))
}
