//! The expression tree and its canonical form.
//!
//! Every constructor (`Expr::add`, `Expr::mul`, `Expr::pow`, ...) returns an expression in canonical form, so
//! two expressions that are equal as polynomials-of-monomials compare equal with `==`:
//!
//! * sums and products are flattened and sorted;
//! * like terms are combined (`2*x + 3*x` is `5*x`), equal bases are combined (`x * x^2` is `x^3`);
//! * numbers are exact rationals, folded at construction;
//! * a quotient is a product with a negative exponent, so `1/(E*I)` is `E^-1 * I^-1`;
//! * rational powers of integers are written with prime radicands (`sqrt(8)` is `2*sqrt(2)`).
//!
//! Products are not distributed over sums by the constructors; `expand` does that.
//!
//! # Positivity convention
//!
//! Symbols are treated as positive real quantities when rational powers are combined:
//! `(a*b)^(1/2)` is `a^(1/2) * b^(1/2)` and `(x^2)^(1/2)` is `x`. A sum is only treated as positive when
//! every term is. Numeric evaluation at a negative symbol value can therefore differ from the original
//! text for expressions that contain such powers.

use crate::error::{CasError, CasResult};
use crate::rat::Rat;
use std::collections::{BTreeMap, BTreeSet};

/// A built-in function of one argument. `exp` is `E^x` and `sqrt` is a power, so neither is listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Func {
    /// Sine (radians).
    Sin,
    /// Cosine (radians).
    Cos,
    /// Natural logarithm.
    Ln,
}

/// A named constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Const {
    /// The ratio of a circle's circumference to its diameter.
    Pi,
    /// Euler's number.
    E,
}

/// A symbolic expression. Build values with the `Expr` constructors, which keep them canonical; building
/// the variants directly bypasses canonicalisation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expr {
    /// An exact rational number.
    Num(Rat),
    /// `pi` or `e`.
    Const(Const),
    /// A named quantity.
    Sym(String),
    /// A function applied to an argument.
    Func(Func, Box<Expr>),
    /// `base ^ exponent`.
    Pow(Box<Expr>, Box<Expr>),
    /// A sum of two or more terms.
    Add(Vec<Expr>),
    /// A product of two or more factors; a numeric coefficient, if not 1, comes first.
    Mul(Vec<Expr>),
}

fn zero() -> Expr {
    Expr::Num(Rat::ZERO)
}

fn one() -> Expr {
    Expr::Num(Rat::ONE)
}

fn raw_pow(b: Expr, e: Expr) -> Expr {
    Expr::Pow(Box::new(b), Box::new(e))
}

/// Trial-division factorisation of a positive integer. A cofactor left after the trial limit is returned
/// as one entry (it may be composite for very large inputs).
fn factorize(mut m: i128) -> Vec<(i128, u32)> {
    let mut out = Vec::new();
    let mut p: i128 = 2;
    while p <= 100_000 && p * p <= m {
        if m % p == 0 {
            let mut c = 0;
            while m % p == 0 {
                m /= p;
                c += 1;
            }
            out.push((p, c));
        }
        p += if p == 2 { 1 } else { 2 };
    }
    if m > 1 {
        out.push((m, 1));
    }
    out
}

/// `q^r` for exact rational `q` and rational `r`.
fn num_pow(q: Rat, r: Rat) -> CasResult<Expr> {
    if q.is_zero() {
        return if r.is_positive() { Ok(zero()) } else { Err(CasError::DivisionByZero) };
    }
    if q.is_one() {
        return Ok(one());
    }
    if r.is_integer() {
        let e = r.numer();
        return Ok(Expr::Num(q.pow_i(e)?));
    }
    if q.is_negative() {
        return Ok(raw_pow(Expr::Num(q), Expr::Num(r)));
    }
    let mut coef = Rat::ONE;
    let mut radicals = Vec::new();
    for (m, rr) in [(q.numer(), r), (q.denom(), r.neg()?)] {
        if m == 1 {
            continue;
        }
        for (p, e) in factorize(m) {
            let t = rr.mul(Rat::int(e as i64))?;
            let k = t.floor();
            let f = t.sub(Rat::new(k, 1)?)?;
            coef = coef.mul(Rat::new(p, 1)?.pow_i(k)?)?;
            if !f.is_zero() {
                radicals.push(raw_pow(Expr::Num(Rat::new(p, 1)?), Expr::Num(f)));
            }
        }
    }
    let mut items = vec![Expr::Num(coef)];
    items.extend(radicals);
    Expr::product(items)
}

fn split_term(t: Expr) -> (Rat, Expr) {
    match t {
        Expr::Num(r) => (r, one()),
        Expr::Mul(mut fs) => {
            if let Some(Expr::Num(c)) = fs.first() {
                let c = *c;
                fs.remove(0);
                if fs.len() == 1 {
                    (c, fs.pop().unwrap())
                } else {
                    (c, Expr::Mul(fs))
                }
            } else {
                (Rat::ONE, Expr::Mul(fs))
            }
        }
        other => (Rat::ONE, other),
    }
}

fn scale(c: Rat, rest: Expr) -> Expr {
    if rest == one() {
        return Expr::Num(c);
    }
    if c.is_one() {
        return rest;
    }
    match rest {
        Expr::Mul(mut fs) => {
            fs.insert(0, Expr::Num(c));
            Expr::Mul(fs)
        }
        other => Expr::Mul(vec![Expr::Num(c), other]),
    }
}

impl Expr {
    /// The integer `n`.
    pub fn int(n: i64) -> Expr {
        Expr::Num(Rat::int(n))
    }

    /// The rational `n/d`.
    pub fn rational(n: i128, d: i128) -> CasResult<Expr> {
        Ok(Expr::Num(Rat::new(n, d)?))
    }

    /// A symbol.
    pub fn sym(name: &str) -> Expr {
        Expr::Sym(name.to_string())
    }

    /// The constant pi.
    pub fn pi() -> Expr {
        Expr::Const(Const::Pi)
    }

    /// The constant e.
    pub fn e() -> Expr {
        Expr::Const(Const::E)
    }

    /// The number, if this is one.
    pub fn as_num(&self) -> Option<Rat> {
        match self {
            Expr::Num(r) => Some(*r),
            _ => None,
        }
    }

    /// True for the number 0.
    pub fn is_zero(&self) -> bool {
        matches!(self, Expr::Num(r) if r.is_zero())
    }

    /// True for the number 1.
    pub fn is_one(&self) -> bool {
        matches!(self, Expr::Num(r) if r.is_one())
    }

    /// True when the symbol `name` occurs anywhere in the expression.
    pub fn has_symbol(&self, name: &str) -> bool {
        match self {
            Expr::Num(_) | Expr::Const(_) => false,
            Expr::Sym(s) => s == name,
            Expr::Func(_, a) => a.has_symbol(name),
            Expr::Pow(b, e) => b.has_symbol(name) || e.has_symbol(name),
            Expr::Add(v) | Expr::Mul(v) => v.iter().any(|t| t.has_symbol(name)),
        }
    }

    /// The names of all symbols, sorted.
    pub fn symbols(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        self.collect_symbols(&mut out);
        out
    }

    fn collect_symbols(&self, out: &mut BTreeSet<String>) {
        match self {
            Expr::Num(_) | Expr::Const(_) => {}
            Expr::Sym(s) => {
                out.insert(s.clone());
            }
            Expr::Func(_, a) => a.collect_symbols(out),
            Expr::Pow(b, e) => {
                b.collect_symbols(out);
                e.collect_symbols(out);
            }
            Expr::Add(v) | Expr::Mul(v) => v.iter().for_each(|t| t.collect_symbols(out)),
        }
    }

    /// True when the expression is positive under the library's convention (see the module notes).
    pub fn is_positive(&self) -> bool {
        match self {
            Expr::Num(r) => r.is_positive(),
            Expr::Const(_) | Expr::Sym(_) => true,
            Expr::Pow(b, _) => b.is_positive(),
            Expr::Add(v) | Expr::Mul(v) => v.iter().all(Expr::is_positive),
            Expr::Func(..) => false,
        }
    }

    // ---- constructors -------------------------------------------------------------------------

    /// `a + b`.
    pub fn add(a: Expr, b: Expr) -> CasResult<Expr> {
        Expr::sum(vec![a, b])
    }

    /// `a - b`.
    pub fn sub(a: Expr, b: Expr) -> CasResult<Expr> {
        Expr::sum(vec![a, Expr::neg(b)?])
    }

    /// `a * b`.
    pub fn mul(a: Expr, b: Expr) -> CasResult<Expr> {
        Expr::product(vec![a, b])
    }

    /// `a / b`; a divisor that is zero is `DivisionByZero`.
    pub fn div(a: Expr, b: Expr) -> CasResult<Expr> {
        Expr::product(vec![a, Expr::pow(b, Expr::int(-1))?])
    }

    /// `-a`.
    pub fn neg(a: Expr) -> CasResult<Expr> {
        Expr::product(vec![Expr::int(-1), a])
    }

    /// `1/a`.
    pub fn recip(a: Expr) -> CasResult<Expr> {
        Expr::pow(a, Expr::int(-1))
    }

    /// `sqrt(a)`, the power `a^(1/2)`.
    pub fn sqrt(a: Expr) -> CasResult<Expr> {
        Expr::pow(a, Expr::rational(1, 2)?)
    }

    /// `exp(a)`, the power `E^a`.
    pub fn exp(a: Expr) -> CasResult<Expr> {
        Expr::pow(Expr::e(), a)
    }

    /// The canonical sum of the terms.
    pub fn sum(terms: Vec<Expr>) -> CasResult<Expr> {
        let mut flat = Vec::with_capacity(terms.len());
        for t in terms {
            match t {
                Expr::Add(v) => flat.extend(v),
                other => flat.push(other),
            }
        }
        let mut map: BTreeMap<Expr, Rat> = BTreeMap::new();
        for t in flat {
            let (c, rest) = split_term(t);
            let slot = map.entry(rest).or_insert(Rat::ZERO);
            *slot = slot.add(c)?;
        }
        let mut out: Vec<Expr> =
            map.into_iter().filter(|(_, c)| !c.is_zero()).map(|(rest, c)| scale(c, rest)).collect();
        out.sort();
        Ok(match out.len() {
            0 => zero(),
            1 => out.pop().unwrap(),
            _ => Expr::Add(out),
        })
    }

    /// The canonical product of the factors.
    pub fn product(items: Vec<Expr>) -> CasResult<Expr> {
        let mut items = items;
        for _ in 0..16 {
            let mut coef = Rat::ONE;
            let mut flat = Vec::new();
            for it in items {
                match it {
                    Expr::Num(r) => coef = coef.mul(r)?,
                    Expr::Mul(fs) => {
                        for f in fs {
                            match f {
                                Expr::Num(r) => coef = coef.mul(r)?,
                                other => flat.push(other),
                            }
                        }
                    }
                    other => flat.push(other),
                }
            }
            if coef.is_zero() {
                return Ok(zero());
            }
            let mut groups: BTreeMap<Expr, Vec<(Expr, Expr)>> = BTreeMap::new();
            for f in flat {
                let (b, e) = match &f {
                    Expr::Pow(b, e) => ((**b).clone(), (**e).clone()),
                    _ => (f.clone(), one()),
                };
                groups.entry(b).or_default().push((e, f));
            }
            let merged = groups.values().any(|g| g.len() > 1);
            if !merged {
                let mut fs: Vec<Expr> = groups.into_values().flatten().map(|(_, f)| f).collect();
                fs.sort();
                return Ok(match (fs.len(), coef.is_one()) {
                    (0, _) => Expr::Num(coef),
                    (1, true) => fs.pop().unwrap(),
                    _ => {
                        if !coef.is_one() {
                            fs.insert(0, Expr::Num(coef));
                        }
                        Expr::Mul(fs)
                    }
                });
            }
            let mut next = vec![Expr::Num(coef)];
            for (base, g) in groups {
                if g.len() == 1 {
                    next.push(g.into_iter().next().unwrap().1);
                } else {
                    let total = Expr::sum(g.into_iter().map(|(e, _)| e).collect())?;
                    next.push(Expr::pow(base, total)?);
                }
            }
            items = next;
        }
        Err(CasError::Unsupported("product did not reach a canonical form".into()))
    }

    /// `base ^ exp`, canonical. `0^negative` is `DivisionByZero`; a negative number to a fractional power
    /// is kept unevaluated (the library has no complex numbers).
    pub fn pow(base: Expr, exp: Expr) -> CasResult<Expr> {
        if let Expr::Num(r) = &exp {
            if r.is_zero() {
                return Ok(one());
            }
            if r.is_one() {
                return Ok(base);
            }
        }
        if base.is_one() {
            return Ok(one());
        }
        if let (Expr::Num(q), Expr::Num(r)) = (&base, &exp) {
            return num_pow(*q, *r);
        }
        if base == Expr::e() {
            if let Expr::Func(Func::Ln, u) = &exp {
                return Ok((**u).clone());
            }
        }
        let int_exp = matches!(&exp, Expr::Num(r) if r.is_integer());
        match &base {
            Expr::Pow(b2, e2) if int_exp || b2.is_positive() => {
                let ne = Expr::mul((**e2).clone(), exp)?;
                return Expr::pow((**b2).clone(), ne);
            }
            Expr::Mul(fs) => {
                if int_exp {
                    let parts: CasResult<Vec<Expr>> =
                        fs.iter().map(|f| Expr::pow(f.clone(), exp.clone())).collect();
                    return Expr::product(parts?);
                }
                let (pos, rest): (Vec<Expr>, Vec<Expr>) = fs.iter().cloned().partition(Expr::is_positive);
                if !pos.is_empty() {
                    let mut parts: Vec<Expr> =
                        pos.into_iter().map(|f| Expr::pow(f, exp.clone())).collect::<CasResult<_>>()?;
                    if !rest.is_empty() {
                        parts.push(raw_pow(Expr::product(rest)?, exp));
                    }
                    return Expr::product(parts);
                }
            }
            _ => {}
        }
        Ok(raw_pow(base, exp))
    }

    /// `sin(a)`, with exact values at rational multiples of pi (multiples of pi/12 that are also multiples
    /// of pi/6 or pi/4).
    pub fn sin(a: Expr) -> CasResult<Expr> {
        Expr::trig(Func::Sin, a)
    }

    /// `cos(a)`, with the same exact values as `sin`.
    pub fn cos(a: Expr) -> CasResult<Expr> {
        Expr::trig(Func::Cos, a)
    }

    fn trig(f: Func, a: Expr) -> CasResult<Expr> {
        if a.is_zero() {
            return Ok(if f == Func::Sin { zero() } else { one() });
        }
        let (c, rest) = split_term(a.clone());
        if c.is_negative() {
            let pos = scale(c.neg()?, rest);
            let inner = Expr::trig(f, pos)?;
            return if f == Func::Sin { Expr::neg(inner) } else { Ok(inner) };
        }
        // rational multiple of pi?
        if rest == Expr::pi() {
            if let Some(v) = trig_pi_multiple(f, c)? {
                return Ok(v);
            }
        }
        Ok(Expr::Func(f, Box::new(a)))
    }

    /// `ln(a)`. `ln(0)` and the logarithm of a negative number are domain errors.
    pub fn ln(a: Expr) -> CasResult<Expr> {
        match &a {
            Expr::Num(r) if !r.is_positive() => {
                return Err(CasError::Domain("ln of a non-positive number".into()));
            }
            Expr::Num(r) if r.is_one() => return Ok(zero()),
            Expr::Const(Const::E) => return Ok(one()),
            Expr::Pow(b, e) if **b == Expr::e() => return Ok((**e).clone()),
            Expr::Pow(b, e) if b.is_positive() && matches!(**b, Expr::Sym(_)) => {
                if let Expr::Num(_) = **e {
                    return Expr::mul((**e).clone(), Expr::ln((**b).clone())?);
                }
            }
            _ => {}
        }
        Ok(Expr::Func(Func::Ln, Box::new(a)))
    }
}

/// Exact `sin`/`cos` of `c*pi` for the angles whose value is a short closed form.
fn trig_pi_multiple(f: Func, c: Rat) -> CasResult<Option<Expr>> {
    // reduce c to [0, 2)
    let two = Rat::int(2);
    let k = c.div(two)?.floor();
    let c = c.sub(two.mul(Rat::new(k, 1)?)?)?;
    // c*12 must be an integer m
    let m12 = c.mul(Rat::int(12))?;
    if !m12.is_integer() {
        return Ok(None);
    }
    let m = m12.numer() as i64; // 0..24
    // sin(m*pi/12): known for m even (multiples of pi/6) and m multiple of 3 (multiples of pi/4)
    let m = if f == Func::Cos { (m + 6).rem_euclid(24) } else { m };
    let half = Expr::rational(1, 2)?;
    let s3 = Expr::mul(half.clone(), Expr::sqrt(Expr::int(3))?)?;
    let s2 = Expr::mul(half.clone(), Expr::sqrt(Expr::int(2))?)?;
    let neg = |e: Expr| Expr::neg(e);
    let v = match m {
        0 | 12 => zero(),
        2 | 10 => half,
        14 | 22 => neg(half)?,
        4 | 8 => s3,
        16 | 20 => neg(s3)?,
        6 => one(),
        18 => Expr::int(-1),
        3 | 9 => s2,
        15 | 21 => neg(s2)?,
        _ => return Ok(None),
    };
    Ok(Some(v))
}
