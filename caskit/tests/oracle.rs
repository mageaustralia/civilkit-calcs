//! Property tests against an independent oracle: random expressions are evaluated directly in `f64` from
//! the generator's own tree, and every library result must agree with that number. The generator is
//! seeded, so a failure reproduces.

mod common;
use caskit::{CasError, Expr};
use common::{close, p, vars};
use std::collections::HashMap;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[derive(Clone, Debug)]
enum T {
    Int(i64),
    Frac(i64, i64),
    Var(usize),
    Add(Box<T>, Box<T>),
    Sub(Box<T>, Box<T>),
    Mul(Box<T>, Box<T>),
    /// a / (2 + b^2): the denominator is never zero
    Div(Box<T>, Box<T>),
    PowI(Box<T>, i32),
    /// sqrt(1 + a^2)
    Sqrt(Box<T>),
    Sin(Box<T>),
    Cos(Box<T>),
    /// exp(a / 8)
    Exp(Box<T>),
    /// ln(1 + a^2)
    Ln(Box<T>),
}

const NAMES: [&str; 3] = ["x", "y", "z"];

fn gen(r: &mut Rng, depth: u32) -> T {
    if depth == 0 || r.below(5) == 0 {
        return match r.below(3) {
            0 => T::Int(r.below(7) as i64 - 3),
            1 => T::Frac(r.below(9) as i64 + 1, r.below(5) as i64 + 2),
            _ => T::Var(r.below(3) as usize),
        };
    }
    let k = r.below(12);
    let a = Box::new(gen(r, depth - 1));
    let b = Box::new(gen(r, depth - 1));
    match k {
        0 | 1 => T::Add(a, b),
        2 => T::Sub(a, b),
        3 | 4 => T::Mul(a, b),
        5 => T::Div(a, b),
        6 => T::PowI(a, 2 + (depth % 2) as i32),
        7 => T::Sqrt(a),
        8 => T::Sin(a),
        9 => T::Cos(a),
        10 => T::Exp(a),
        _ => T::Ln(a),
    }
}

fn eval(t: &T, v: &[f64; 3]) -> f64 {
    match t {
        T::Int(n) => *n as f64,
        T::Frac(a, b) => *a as f64 / *b as f64,
        T::Var(i) => v[*i],
        T::Add(a, b) => eval(a, v) + eval(b, v),
        T::Sub(a, b) => eval(a, v) - eval(b, v),
        T::Mul(a, b) => eval(a, v) * eval(b, v),
        T::Div(a, b) => eval(a, v) / (2.0 + eval(b, v).powi(2)),
        T::PowI(a, n) => eval(a, v).powi(*n),
        T::Sqrt(a) => (1.0 + eval(a, v).powi(2)).sqrt(),
        T::Sin(a) => eval(a, v).sin(),
        T::Cos(a) => eval(a, v).cos(),
        T::Exp(a) => (eval(a, v) / 8.0).exp(),
        T::Ln(a) => (1.0 + eval(a, v).powi(2)).ln(),
    }
}

fn build(t: &T) -> Result<Expr, CasError> {
    Ok(match t {
        T::Int(n) => Expr::int(*n),
        T::Frac(a, b) => Expr::rational(*a as i128, *b as i128)?,
        T::Var(i) => Expr::sym(NAMES[*i]),
        T::Add(a, b) => Expr::add(build(a)?, build(b)?)?,
        T::Sub(a, b) => Expr::sub(build(a)?, build(b)?)?,
        T::Mul(a, b) => Expr::mul(build(a)?, build(b)?)?,
        T::Div(a, b) => {
            let den = Expr::add(Expr::int(2), Expr::pow(build(b)?, Expr::int(2))?)?;
            Expr::div(build(a)?, den)?
        }
        T::PowI(a, n) => Expr::pow(build(a)?, Expr::int(*n as i64))?,
        T::Sqrt(a) => Expr::sqrt(Expr::add(Expr::int(1), Expr::pow(build(a)?, Expr::int(2))?)?)?,
        T::Sin(a) => Expr::sin(build(a)?)?,
        T::Cos(a) => Expr::cos(build(a)?)?,
        T::Exp(a) => Expr::exp(Expr::div(build(a)?, Expr::int(8))?)?,
        T::Ln(a) => Expr::ln(Expr::add(Expr::int(1), Expr::pow(build(a)?, Expr::int(2))?)?)?,
    })
}

fn point(r: &mut Rng) -> ([f64; 3], HashMap<String, f64>) {
    let v = [0.6 + 1.9 * r.unit(), 0.6 + 1.9 * r.unit(), 0.6 + 1.9 * r.unit()];
    let m = vars(&[("x", v[0]), ("y", v[1]), ("z", v[2])]);
    (v, m)
}

fn agree(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()))
}

#[test]
fn build_simplify_expand_match_the_oracle() {
    let mut r = Rng(0x9E3779B97F4A7C15);
    let (mut checked, mut skipped) = (0, 0);
    for _ in 0..500 {
        let t = gen(&mut r, 4);
        let Ok(e) = build(&t) else {
            skipped += 1;
            continue;
        };
        let (Ok(simp), Ok(exp)) = (e.simplify(), e.expand()) else {
            skipped += 1;
            continue;
        };
        for _ in 0..3 {
            let (v, m) = point(&mut r);
            let want = eval(&t, &v);
            if !want.is_finite() || want.abs() > 1e7 {
                continue;
            }
            for (label, ex) in [("built", &e), ("simplified", &simp), ("expanded", &exp)] {
                let got = ex.eval(&m).unwrap();
                assert!(
                    agree(got, want, 1e-8),
                    "{label}: {t:?}\n  {ex}\n  got {got} want {want}"
                );
            }
            checked += 1;
        }
    }
    assert!(
        checked > 900,
        "only {checked} points checked ({skipped} expressions skipped)"
    );
    assert!(skipped < 50, "{skipped} expressions skipped");
}

#[test]
fn simplify_is_idempotent_on_random_expressions() {
    let mut r = Rng(42);
    let mut n = 0;
    for _ in 0..300 {
        let t = gen(&mut r, 3);
        let Ok(e) = build(&t) else { continue };
        let Ok(once) = e.simplify() else { continue };
        let twice = once.simplify().unwrap();
        assert_eq!(once, twice, "{t:?}");
        n += 1;
    }
    assert!(n > 250);
}

#[test]
fn printing_round_trips_on_random_expressions() {
    let mut r = Rng(7);
    let mut n = 0;
    for _ in 0..300 {
        let t = gen(&mut r, 4);
        let Ok(e) = build(&t) else { continue };
        let Ok(s) = e.simplify() else { continue };
        for ex in [&e, &s] {
            let text = ex.to_ascii();
            assert_eq!(p(&text), *ex, "text {text}");
        }
        n += 1;
    }
    assert!(n > 250);
}

#[test]
fn derivative_matches_central_difference() {
    let mut r = Rng(0xDEADBEEF);
    let mut checked = 0;
    for _ in 0..400 {
        let t = gen(&mut r, 3);
        let Ok(e) = build(&t) else { continue };
        let var = NAMES[r.below(3) as usize];
        let Ok(d) = e.diff(var) else { continue };
        let Ok(d) = d.simplify() else { continue };
        for _ in 0..3 {
            let (_, m) = point(&mut r);
            let f = |delta: f64| {
                let mut mm = m.clone();
                *mm.get_mut(var).unwrap() += delta;
                e.eval(&mm).unwrap()
            };
            let h = 1e-5;
            let fd = (f(h) - f(-h)) / (2.0 * h);
            let got = d.eval(&m).unwrap();
            if !fd.is_finite() || fd.abs() > 1e5 {
                continue;
            }
            assert!(
                agree(got, fd, 1e-5),
                "d/d{var} of {e}\n  derivative {d}\n  got {got} finite difference {fd}"
            );
            checked += 1;
        }
    }
    assert!(checked > 600, "only {checked} points");
}

fn random_integrand(r: &mut Rng) -> String {
    let coefs = ["1", "3", "-2", "1/2", "a", "b", "a*b", "a/b", "-3*a/4"];
    let kinds = ["sin", "cos", "exp"];
    let args = ["x", "2*x", "3*x", "x/2", "a*x", "a*x+b", "2*x+1", "-x"];
    let mut terms = Vec::new();
    for _ in 0..(1 + r.below(3)) {
        let c = coefs[r.below(coefs.len() as u64) as usize];
        let term = match r.below(6) {
            0 => format!(
                "{c}*x^{}",
                ["(-3)", "(-2)", "(-1)", "0", "1", "2", "3", "(1/2)", "(3/2)"][r.below(9) as usize]
            ),
            1 => format!(
                "{c}*{}({})",
                kinds[r.below(3) as usize],
                args[r.below(args.len() as u64) as usize]
            ),
            2 => format!(
                "{c}*x^{}*{}({})",
                1 + r.below(3),
                kinds[r.below(3) as usize],
                ["x", "2*x", "a*x", "x/2"][r.below(4) as usize]
            ),
            3 => format!(
                "{c}*(2*x+1)^{}",
                ["(-2)", "(-1)", "2", "(1/2)", "(-3/2)"][r.below(5) as usize]
            ),
            4 => format!("{c}*ln({})", ["x", "2*x", "a*x+b"][r.below(3) as usize]),
            _ => format!("{c}*(a*x+b)^{}", ["(-1)", "3", "(-2)"][r.below(3) as usize]),
        };
        terms.push(term);
    }
    terms.join(" + ")
}

#[test]
fn integrate_then_differentiate_returns_the_integrand() {
    let mut r = Rng(0xC0FFEE);
    for _ in 0..300 {
        let src = random_integrand(&mut r);
        let f = p(&src);
        let anti = f.integrate("x").unwrap_or_else(|e| panic!("{src}: {e}"));
        let back = anti.diff("x").unwrap();
        for _ in 0..3 {
            let m = vars(&[
                ("x", 0.7 + 1.8 * r.unit()),
                ("a", 0.5 + 1.5 * r.unit()),
                ("b", 0.5 + 1.5 * r.unit()),
            ]);
            let (want, got) = (f.eval(&m).unwrap(), back.eval(&m).unwrap());
            assert!(
                close(want, got) || agree(want, got, 1e-8),
                "{src}\n  antiderivative {anti}\n  {got} vs {want}"
            );
        }
    }
}

#[test]
fn definite_integral_matches_numeric_quadrature() {
    let mut r = Rng(99);
    for _ in 0..60 {
        let src = random_integrand(&mut r);
        let f = p(&src);
        // keep the integrand regular on [1, 2]: skip anything with a pole in a*x+b, which is positive there
        let exact = f.integrate_definite("x", &p("1"), &p("2")).unwrap();
        let m = vars(&[("a", 0.9 + r.unit()), ("b", 0.9 + r.unit())]);
        let exact_v = exact.eval(&m).unwrap();
        // composite Simpson with 2000 panels
        let n = 2000;
        let h = 1.0 / n as f64;
        let mut sum = 0.0;
        for i in 0..=n {
            let mut mm = m.clone();
            mm.insert("x".into(), 1.0 + i as f64 * h);
            let w = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            sum += w * f.eval(&mm).unwrap();
        }
        let approx = sum * h / 3.0;
        assert!(
            agree(exact_v, approx, 1e-7),
            "{src}: exact {exact_v} quadrature {approx}"
        );
    }
}
