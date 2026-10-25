//! Substitution, numeric evaluation, expansion, simplification and differentiation.

use crate::error::{CasError, CasResult};
use crate::expr::{Const, Expr, Func};
use crate::rat::Rat;
use std::collections::{BTreeMap, HashMap};

/// The largest number of terms `expand` will build before giving up with `TooLarge`.
const EXPAND_LIMIT: usize = 20_000;

impl Expr {
    /// Replaces every occurrence of the symbol `var` by `with`, then re-canonicalises.
    pub fn substitute(&self, var: &str, with: &Expr) -> CasResult<Expr> {
        match self {
            Expr::Num(_) | Expr::Const(_) => Ok(self.clone()),
            Expr::Sym(s) => Ok(if s == var { with.clone() } else { self.clone() }),
            Expr::Func(f, a) => apply_func(*f, a.substitute(var, with)?),
            Expr::Pow(b, e) => Expr::pow(b.substitute(var, with)?, e.substitute(var, with)?),
            Expr::Add(v) => Expr::sum(v.iter().map(|t| t.substitute(var, with)).collect::<CasResult<_>>()?),
            Expr::Mul(v) => Expr::product(v.iter().map(|t| t.substitute(var, with)).collect::<CasResult<_>>()?),
        }
    }

    /// Replaces several symbols at once (the replacements are not applied to each other).
    pub fn substitute_all(&self, map: &[(&str, Expr)]) -> CasResult<Expr> {
        match self {
            Expr::Num(_) | Expr::Const(_) => Ok(self.clone()),
            Expr::Sym(s) => Ok(map.iter().find(|(n, _)| n == s).map(|(_, v)| v.clone()).unwrap_or_else(|| self.clone())),
            Expr::Func(f, a) => apply_func(*f, a.substitute_all(map)?),
            Expr::Pow(b, e) => Expr::pow(b.substitute_all(map)?, e.substitute_all(map)?),
            Expr::Add(v) => Expr::sum(v.iter().map(|t| t.substitute_all(map)).collect::<CasResult<_>>()?),
            Expr::Mul(v) => Expr::product(v.iter().map(|t| t.substitute_all(map)).collect::<CasResult<_>>()?),
        }
    }

    /// Evaluates numerically. Every symbol must have a value in `vars`.
    pub fn eval(&self, vars: &HashMap<String, f64>) -> CasResult<f64> {
        Ok(match self {
            Expr::Num(r) => r.to_f64(),
            Expr::Const(Const::Pi) => std::f64::consts::PI,
            Expr::Const(Const::E) => std::f64::consts::E,
            Expr::Sym(s) => *vars.get(s).ok_or_else(|| CasError::UnboundSymbol(s.clone()))?,
            Expr::Func(f, a) => {
                let x = a.eval(vars)?;
                match f {
                    Func::Sin => x.sin(),
                    Func::Cos => x.cos(),
                    Func::Ln => x.ln(),
                }
            }
            Expr::Pow(b, e) => {
                let (bv, ev) = (b.eval(vars)?, e.eval(vars)?);
                if **b == Expr::e() {
                    ev.exp()
                } else if bv < 0.0 && ev.fract() == 0.0 {
                    bv.powi(ev as i32)
                } else {
                    bv.powf(ev)
                }
            }
            Expr::Add(v) => {
                let mut s = 0.0;
                for t in v {
                    s += t.eval(vars)?;
                }
                s
            }
            Expr::Mul(v) => {
                let mut p = 1.0;
                for t in v {
                    p *= t.eval(vars)?;
                }
                p
            }
        })
    }

    /// Distributes products over sums and expands positive integer powers of sums, everywhere in the tree.
    pub fn expand(&self) -> CasResult<Expr> {
        match self {
            Expr::Num(_) | Expr::Const(_) | Expr::Sym(_) => Ok(self.clone()),
            Expr::Func(f, a) => apply_func(*f, a.expand()?),
            Expr::Add(v) => Expr::sum(v.iter().map(Expr::expand).collect::<CasResult<_>>()?),
            Expr::Mul(v) => {
                let mut acc = vec![Expr::int(1)];
                for f in v {
                    let ef = f.expand()?;
                    acc = distribute(&acc, &ef)?;
                }
                Expr::sum(acc)
            }
            Expr::Pow(b, x) => {
                let eb = b.expand()?;
                let ex = x.expand()?;
                if let (Expr::Add(_), Expr::Num(n)) = (&eb, &ex) {
                    if n.is_integer() && n.is_positive() {
                        let k = n.numer();
                        if k > 64 {
                            return Err(CasError::TooLarge);
                        }
                        let mut acc = vec![Expr::int(1)];
                        for _ in 0..k {
                            acc = distribute(&acc, &eb)?;
                        }
                        return Expr::sum(acc);
                    }
                }
                let p = Expr::pow(eb, ex)?;
                // the power may have turned into a product with sums in it
                if matches!(p, Expr::Mul(_) | Expr::Add(_)) {
                    p.expand()
                } else {
                    Ok(p)
                }
            }
        }
    }

    /// Expands and collects like terms. Equal expressions (as sums of monomials with rational powers)
    /// simplify to identical trees, so `simplify(a - b)` is `0` exactly when `a` and `b` are equal in that
    /// sense. It does not cancel common factors of sums in a denominator.
    pub fn simplify(&self) -> CasResult<Expr> {
        self.expand()
    }

    /// The derivative with respect to `var`, not expanded.
    pub fn diff(&self, var: &str) -> CasResult<Expr> {
        match self {
            Expr::Num(_) | Expr::Const(_) => Ok(Expr::int(0)),
            Expr::Sym(s) => Ok(Expr::int(if s == var { 1 } else { 0 })),
            Expr::Add(v) => Expr::sum(v.iter().map(|t| t.diff(var)).collect::<CasResult<_>>()?),
            Expr::Mul(v) => {
                let mut terms = Vec::new();
                for i in 0..v.len() {
                    let d = v[i].diff(var)?;
                    if d.is_zero() {
                        continue;
                    }
                    let mut fs: Vec<Expr> = v.clone();
                    fs[i] = d;
                    terms.push(Expr::product(fs)?);
                }
                Expr::sum(terms)
            }
            Expr::Pow(b, e) => {
                if !e.has_symbol(var) {
                    let db = b.diff(var)?;
                    if db.is_zero() {
                        return Ok(Expr::int(0));
                    }
                    let em1 = Expr::sub((**e).clone(), Expr::int(1))?;
                    return Expr::product(vec![(**e).clone(), Expr::pow((**b).clone(), em1)?, db]);
                }
                // general power: b^e * (e' ln b + e b'/b)
                let de = e.diff(var)?;
                let db = b.diff(var)?;
                let t1 = Expr::mul(de, Expr::ln((**b).clone())?)?;
                let t2 = Expr::product(vec![(**e).clone(), db, Expr::recip((**b).clone())?])?;
                Expr::mul(self.clone(), Expr::add(t1, t2)?)
            }
            Expr::Func(f, a) => {
                let da = a.diff(var)?;
                if da.is_zero() {
                    return Ok(Expr::int(0));
                }
                match f {
                    Func::Sin => Expr::mul(Expr::cos((**a).clone())?, da),
                    Func::Cos => Expr::product(vec![Expr::int(-1), Expr::sin((**a).clone())?, da]),
                    Func::Ln => Expr::div(da, (**a).clone()),
                }
            }
        }
    }

    /// The `n`-th derivative.
    pub fn diff_n(&self, var: &str, n: usize) -> CasResult<Expr> {
        let mut e = self.clone();
        for _ in 0..n {
            e = e.diff(var)?;
        }
        Ok(e)
    }

    /// Groups the terms of the (expanded) expression by the power of `var`. Returns `None` unless every
    /// term is `coefficient * var^p` with a rational `p` and a coefficient that does not contain `var`.
    pub fn var_terms(&self, var: &str) -> Option<BTreeMap<Rat, Expr>> {
        let ex = self.expand().ok()?;
        let terms: Vec<Expr> = match ex {
            Expr::Add(v) => v,
            other => vec![other],
        };
        let mut out: BTreeMap<Rat, Vec<Expr>> = BTreeMap::new();
        for t in terms {
            let factors: Vec<Expr> = match t {
                Expr::Mul(fs) => fs,
                other => vec![other],
            };
            let mut p = Rat::ZERO;
            let mut free = Vec::new();
            for f in factors {
                if !f.has_symbol(var) {
                    free.push(f);
                    continue;
                }
                match &f {
                    Expr::Sym(s) if s == var => p = p.add(Rat::ONE).ok()?,
                    Expr::Pow(b, e) if matches!(&**b, Expr::Sym(s) if s == var) => match &**e {
                        Expr::Num(r) => p = p.add(*r).ok()?,
                        _ => return None,
                    },
                    _ => return None,
                }
            }
            out.entry(p).or_default().push(Expr::product(free).ok()?);
        }
        let mut res = BTreeMap::new();
        for (p, cs) in out {
            let c = Expr::sum(cs).ok()?;
            if !c.is_zero() {
                res.insert(p, c);
            }
        }
        Some(res)
    }

    /// For an expression of degree exactly one in `var`, returns `(a, b)` with `self = a*var + b`.
    pub fn linear_in(&self, var: &str) -> Option<(Expr, Expr)> {
        let t = self.var_terms(var)?;
        if t.keys().any(|p| *p != Rat::ZERO && *p != Rat::ONE) {
            return None;
        }
        let a = t.get(&Rat::ONE)?.clone();
        let b = t.get(&Rat::ZERO).cloned().unwrap_or_else(|| Expr::int(0));
        Some((a, b))
    }
}

fn apply_func(f: Func, a: Expr) -> CasResult<Expr> {
    match f {
        Func::Sin => Expr::sin(a),
        Func::Cos => Expr::cos(a),
        Func::Ln => Expr::ln(a),
    }
}

fn distribute(acc: &[Expr], f: &Expr) -> CasResult<Vec<Expr>> {
    let fterms: Vec<Expr> = match f {
        Expr::Add(ts) => ts.clone(),
        other => vec![other.clone()],
    };
    if acc.len().saturating_mul(fterms.len()) > EXPAND_LIMIT {
        return Err(CasError::TooLarge);
    }
    let mut out = Vec::with_capacity(acc.len() * fterms.len());
    for a in acc {
        for t in &fterms {
            let p = Expr::mul(a.clone(), t.clone())?;
            match p {
                Expr::Add(v) => out.extend(v),
                other => out.push(other),
            }
        }
    }
    // keep the working list canonical and small
    let s = Expr::sum(out)?;
    Ok(match s {
        Expr::Add(v) => v,
        other if other.is_zero() => vec![],
        other => vec![other],
    })
}
