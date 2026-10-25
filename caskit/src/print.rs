//! ASCII and LaTeX printers. The ASCII form parses back to the same expression.

use crate::expr::{Const, Expr, Func};
use crate::rat::Rat;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Style {
    Ascii,
    Latex,
}

impl Expr {
    /// ASCII text that `parse` reads back to an equal expression, for example `L^2*w/8`.
    pub fn to_ascii(&self) -> String {
        render(self, Style::Ascii, 0)
    }

    /// LaTeX source, for example `\frac{L^{2} w}{8}`.
    pub fn to_latex(&self) -> String {
        render(self, Style::Latex, 0)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_ascii())
    }
}

// precedence levels: 0 sum context, 1 product context, 2 power base / function-free atom context
fn paren(s: String, st: Style) -> String {
    match st {
        Style::Ascii => format!("({s})"),
        Style::Latex => format!("\\left({s}\\right)"),
    }
}

fn sort_key(e: &Expr) -> (u8, String) {
    match e {
        Expr::Const(Const::Pi) => (0, "pi".into()),
        Expr::Const(Const::E) => (0, "e".into()),
        Expr::Sym(s) => (1, format!("{}\u{0}{}", s.to_lowercase(), s)),
        Expr::Func(f, a) => (2, format!("{f:?}{}", a.to_ascii())),
        Expr::Pow(b, _) if **b == Expr::e() => (3, e.to_ascii()),
        Expr::Pow(b, _) => sort_key(b),
        other => (4, other.to_ascii()),
    }
}

fn degree(e: &Expr) -> f64 {
    let fs: &[Expr] = match e {
        Expr::Mul(v) => v,
        other => std::slice::from_ref(other),
    };
    fs.iter()
        .map(|f| match f {
            Expr::Sym(_) => 1.0,
            Expr::Pow(b, x) if matches!(**b, Expr::Sym(_)) => x.as_num().map(|r| r.to_f64()).unwrap_or(0.0),
            _ => 0.0,
        })
        .sum()
}

fn num_text(r: Rat, st: Style) -> String {
    if r.is_integer() {
        r.numer().to_string()
    } else if st == Style::Latex {
        let s = if r.is_negative() { "-" } else { "" };
        format!("{s}\\frac{{{}}}{{{}}}", r.numer().abs(), r.denom())
    } else {
        format!("{}/{}", r.numer(), r.denom())
    }
}

fn sym_text(name: &str, st: Style) -> String {
    if st == Style::Ascii {
        return name.to_string();
    }
    const GREEK: [&str; 17] = [
        "alpha", "beta", "gamma", "delta", "epsilon", "theta", "lambda", "mu", "nu", "rho", "sigma", "tau", "phi",
        "omega", "eta", "kappa", "zeta",
    ];
    let (base, sub) = match name.split_once('_') {
        Some((b, s)) => (b, Some(s.to_string())),
        None => {
            let cut = name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
            if cut > 0 && cut < name.len() {
                (&name[..cut], Some(name[cut..].to_string()))
            } else {
                (name, None)
            }
        }
    };
    let b = if GREEK.contains(&base) {
        format!("\\{base}")
    } else if base.chars().count() > 1 {
        format!("\\mathrm{{{base}}}")
    } else {
        base.to_string()
    };
    match sub {
        Some(s) => format!("{b}_{{{s}}}"),
        None => b,
    }
}

fn is_atomic(e: &Expr) -> bool {
    match e {
        Expr::Num(r) => r.is_integer() && !r.is_negative(),
        Expr::Const(_) | Expr::Sym(_) | Expr::Func(..) => true,
        Expr::Pow(b, x) => **b == Expr::e() || matches!(&**x, Expr::Num(r) if *r == Rat::new(1, 2).unwrap()),
        _ => false,
    }
}

fn render(e: &Expr, st: Style, prec: u8) -> String {
    match e {
        Expr::Num(r) => {
            let s = num_text(*r, st);
            if (r.is_negative() && prec > 0) || (!r.is_integer() && prec > 1) {
                paren(s, st)
            } else {
                s
            }
        }
        Expr::Const(Const::Pi) => if st == Style::Ascii { "pi" } else { "\\pi" }.to_string(),
        Expr::Const(Const::E) => "e".to_string(),
        Expr::Sym(s) => sym_text(s, st),
        Expr::Func(f, a) => {
            let inner = render(a, st, 0);
            match (st, f) {
                (Style::Ascii, Func::Sin) => format!("sin({inner})"),
                (Style::Ascii, Func::Cos) => format!("cos({inner})"),
                (Style::Ascii, Func::Ln) => format!("ln({inner})"),
                (Style::Latex, Func::Sin) => format!("\\sin\\left({inner}\\right)"),
                (Style::Latex, Func::Cos) => format!("\\cos\\left({inner}\\right)"),
                (Style::Latex, Func::Ln) => format!("\\ln\\left({inner}\\right)"),
            }
        }
        Expr::Pow(b, x) => {
            if matches!(&**x, Expr::Num(r) if r.is_negative()) {
                return render_mul(std::slice::from_ref(e), st, prec);
            }
            render_pow(b, x, st)
        }
        Expr::Add(ts) => {
            let mut ts = ts.clone();
            ts.sort_by(|a, b| degree(b).total_cmp(&degree(a)).then_with(|| a.cmp(b)));
            let mut out = String::new();
            for (i, t) in ts.iter().enumerate() {
                let s = render(t, st, 0);
                if i == 0 {
                    out.push_str(&s);
                } else if let Some(rest) = s.strip_prefix('-') {
                    out.push_str(" - ");
                    out.push_str(rest);
                } else {
                    out.push_str(" + ");
                    out.push_str(&s);
                }
            }
            if prec > 0 {
                paren(out, st)
            } else {
                out
            }
        }
        Expr::Mul(fs) => render_mul(fs, st, prec),
    }
}

fn render_pow(b: &Expr, x: &Expr, st: Style) -> String {
    if *b == Expr::e() {
        let inner = render(x, st, 0);
        return match st {
            Style::Ascii => format!("exp({inner})"),
            Style::Latex => format!("e^{{{inner}}}"),
        };
    }
    if let Expr::Num(r) = x {
        if *r == Rat::new(1, 2).unwrap() {
            let inner = render(b, st, 0);
            return match st {
                Style::Ascii => format!("sqrt({inner})"),
                Style::Latex => format!("\\sqrt{{{inner}}}"),
            };
        }
        if st == Style::Latex && r.numer() == 1 && r.denom() > 2 {
            return format!("\\sqrt[{}]{{{}}}", r.denom(), render(b, st, 0));
        }
    }
    let base = if is_atomic(b) {
        render(b, st, 2)
    } else {
        paren(render(b, st, 0), st)
    };
    match st {
        Style::Ascii => match x {
            Expr::Num(r) if r.is_integer() && !r.is_negative() => format!("{base}^{r}"),
            _ => format!("{base}^({})", render(x, st, 0)),
        },
        Style::Latex => format!("{base}^{{{}}}", render(x, st, 0)),
    }
}

fn render_mul(fs: &[Expr], st: Style, prec: u8) -> String {
    let mut coef = Rat::ONE;
    let mut nums: Vec<&Expr> = Vec::new();
    let mut dens: Vec<Expr> = Vec::new();
    for f in fs {
        match f {
            Expr::Num(r) => coef = coef.mul(*r).unwrap_or(coef),
            Expr::Pow(b, x) if matches!(&**x, Expr::Num(r) if r.is_negative()) => {
                let r = x.as_num().unwrap();
                let pos = r.neg().unwrap_or(r);
                let d = if pos.is_one() {
                    (**b).clone()
                } else {
                    Expr::Pow(b.clone(), Box::new(Expr::Num(pos)))
                };
                dens.push(d);
            }
            other => nums.push(other),
        }
    }
    nums.sort_by_key(|e| sort_key(e));
    dens.sort_by_key(sort_key);
    let mut ns: Vec<String> = nums.iter().map(|e| render(e, st, 1)).collect();
    let mut ds: Vec<String> = dens
        .iter()
        .map(|e| match e {
            Expr::Pow(b, x) => render_pow_wrapped(b, x, st),
            other => render(other, st, 2),
        })
        .collect();
    let n_abs = coef.numer().abs();
    if n_abs != 1 || ns.is_empty() {
        ns.insert(0, n_abs.to_string());
    }
    if coef.denom() != 1 {
        ds.insert(0, coef.denom().to_string());
    }
    let sign = if coef.is_negative() { "-" } else { "" };
    let join = |items: &[String]| -> String {
        match st {
            Style::Ascii => items.join("*"),
            Style::Latex => {
                let mut out = String::new();
                for (i, s) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(if s.starts_with(|c: char| c.is_ascii_digit()) {
                            " \\cdot "
                        } else {
                            " "
                        });
                    }
                    out.push_str(s);
                }
                out
            }
        }
    };
    let body = if ds.is_empty() {
        join(&ns)
    } else {
        match st {
            Style::Ascii => {
                let den = if ds.len() == 1 {
                    ds[0].clone()
                } else {
                    format!("({})", join(&ds))
                };
                format!("{}/{}", join(&ns), den)
            }
            Style::Latex => format!("\\frac{{{}}}{{{}}}", join(&ns), join(&ds)),
        }
    };
    let out = format!("{sign}{body}");
    if prec >= 2 || (prec == 1 && !sign.is_empty()) {
        paren(out, st)
    } else {
        out
    }
}

fn render_pow_wrapped(b: &Expr, x: &Expr, st: Style) -> String {
    render_pow(b, x, st)
}
