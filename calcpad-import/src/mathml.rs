//! The author's equation as MathML, written straight from the parsed source tree.
//!
//! The translator computes in hidden Calcpad code; this module only decides how the source
//! equation looks on the page: stacked fractions, sums with bounds, subscripts, a vector arrow over
//! a whole product, a transpose superscript, `:=` for a definition. Every browser renders MathML
//! Core. A node this module has no spelling for makes the whole equation `None`, and the caller
//! falls back to the typeset Calcpad display.
//!
//! Nothing here evaluates anything, and nothing here is read back by the engine.

use crate::ir::{BinOp, Expr, Name, Target, UnOp};

/// Binding strength, weakest first. A fraction is closed on its own, so it counts as an atom
/// everywhere except under a power or a postfix operator.
const P_LOGIC: u8 = 1;
const P_CMP: u8 = 2;
const P_ADD: u8 = 3;
const P_MUL: u8 = 4;
const P_NEG: u8 = 5;
const P_POW: u8 = 6;
const P_ATOM: u8 = 8;

/// A piece of MathML and how tightly it binds.
struct M {
    s: String,
    prec: u8,
    /// A stacked fraction (needs brackets under a power or a postfix operator).
    frac: bool,
}

fn atom(s: String) -> M {
    M {
        s,
        prec: P_ATOM,
        frac: false,
    }
}

pub(crate) fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            // A quote character in a text line would switch the engine to maths.
            '\'' => o.push('\u{2032}'),
            '$' => o.push_str("&#36;"),
            '{' => o.push_str("&#123;"),
            '}' => o.push_str("&#125;"),
            '#' => o.push_str("&#35;"),
            c if c.is_control() => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

fn mo(s: &str) -> String {
    format!("<mo>{}</mo>", esc(s))
}

fn row(parts: &[&str]) -> String {
    format!("<mrow>{}</mrow>", parts.concat())
}

fn paren(inner: &str) -> String {
    format!("<mrow><mo>(</mo>{inner}<mo>)</mo></mrow>")
}

/// Writes the source's equations as MathML. `is_unit` says whether an identifier is read as a
/// unit (upright) rather than a variable (italic) where the equation sits.
pub struct MathMl<'a> {
    pub is_unit: &'a dyn Fn(&Name) -> bool,
    /// Whether the page defines this name as its own function (a user function is a variable name
    /// like any other and is set in italic; only the tool's own functions are upright).
    pub is_user_fn: &'a dyn Fn(&Name) -> bool,
    /// The author's spelling of a name the translator renamed to keep it apart from a unit.
    pub original: &'a dyn Fn(&str) -> Option<String>,
}

/// How an identifier is read: where it sits decides, not its spelling.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// A unit when the page says so, a variable otherwise.
    Auto,
    /// Always a variable (the target of a definition, a parameter).
    Var,
    /// A function name.
    Func,
    /// A unit label (upright).
    Unit,
}

impl MathMl<'_> {
    /// `name := value`, `f(x, y) := value` or `a_(b,i) := value`, as a whole `<math>` element.
    pub fn define(&self, target: &Target, value: &Expr, global: bool) -> Option<String> {
        let head = match target {
            Target::Var(n) => self.name(n, Kind::Var),
            Target::Func(n, params) => {
                let ps: Vec<String> = params.iter().map(|p| self.name(p, Kind::Var)).collect();
                format!("{}{}", self.name(n, Kind::Var), paren(&ps.join("<mo>,</mo>")))
            }
            Target::Element { base: Some(b), index } => self.subscripted(b, index, Kind::Var)?,
            Target::Multi(names) if !names.is_empty() && names.len() <= 40 => {
                let mut t = String::from("<mrow><mo>[</mo><mtable>");
                for n in names {
                    t.push_str(&format!("<mtr><mtd>{}</mtd></mtr>", self.name(n, Kind::Var)));
                }
                t.push_str("</mtable><mo>]</mo></mrow>");
                t
            }
            _ => return None,
        };
        let op = if global { "\u{2261}" } else { ":=" };
        let rhs = self.expr(value)?;
        Some(wrap(&format!("{head}{}{}", mo(op), rhs.s)))
    }

    /// An expression on its own (a constraint or a check), as a whole `<math>` element.
    pub fn bare(&self, expr: &Expr) -> Option<String> {
        Some(wrap(&self.expr(expr)?.s))
    }

    /// The left side of an evaluation, `expr =`, without the result; the caller adds the value.
    pub fn eval_head(&self, expr: &Expr) -> Option<String> {
        let e = self.expr(expr)?;
        Some(format!("{}{}", e.s, mo("=")))
    }

    fn subscripted(&self, base: &Name, index: &[Expr], kind: Kind) -> Option<String> {
        let mut subs: Vec<String> = Vec::new();
        if let Some(s) = base.sub.as_deref().filter(|s| !s.is_empty()) {
            subs.push(sub_text(s));
        }
        for i in index {
            subs.push(self.expr(i)?.s);
        }
        let plain = Name {
            base: base.base.clone(),
            sub: None,
            unit: base.unit,
            var: base.var,
        };
        Some(format!(
            "<msub>{}{}</msub>",
            self.name(&plain, kind),
            row(&[&subs.join("<mo>,</mo>")])
        ))
    }

    /// An identifier: base, trailing primes, literal subscript.
    fn name(&self, n: &Name, kind: Kind) -> String {
        let unit = kind == Kind::Unit || (kind == Kind::Auto && (self.is_unit)(n));
        let as_function = kind == Kind::Func;
        let mut shown = (self.original)(&n.base).unwrap_or_else(|| n.base.clone());
        // A name the engine cannot spell is shown as the author wrote it, subscript included.
        let mut own_sub: Option<String> = None;
        if n.sub.is_none() {
            if let Some((b, s)) = shown.split_once('_') {
                if !b.is_empty() && !s.is_empty() {
                    own_sub = Some(s.to_string());
                    shown = b.to_string();
                }
            }
        }
        let (stem, primes) = split_primes(&shown);
        let mut base = if unit {
            format!("<mi mathvariant=\"normal\" class=\"ru\">{}</mi>", esc(&stem))
        } else {
            ident(&stem, as_function)
        };
        if as_function && (stem == "if" || stem == "otherwise") {
            base = "<mi mathvariant=\"normal\" class=\"kw\">".to_string() + &esc(&stem) + "</mi>";
        }
        if primes > 0 {
            base = format!("<msup>{base}<mo>{}</mo></msup>", "\u{2032}".repeat(primes));
        }
        match n.sub.as_deref().filter(|s| !s.is_empty()).or(own_sub.as_deref()) {
            Some(s) => format!("<msub>{base}{}</msub>", row(&[&sub_text(s)])),
            None => base,
        }
    }

    /// How the name of a called function is set: one rule everywhere. The tool's own functions
    /// (and the `if` keyword) are upright; any other name, the page's own functions included,
    /// is an italic variable name.
    fn call_kind(&self, n: &Name) -> Kind {
        if (self.is_user_fn)(n) {
            return Kind::Var;
        }
        let flat = crate::tables::normalise_name(&n.flat());
        let builtin = n.sub.is_none()
            && (matches!(n.base.as_str(), "if" | "otherwise" | "SIUnitsOf")
                || crate::tables::lookup_function(&flat).is_some()
                || crate::tables::is_builtin_missing(&flat)
                || crate::tables::is_solve_name(&flat));
        if builtin {
            Kind::Func
        } else {
            Kind::Var
        }
    }

    fn expr(&self, e: &Expr) -> Option<M> {
        Some(match e {
            Expr::Num(s) => {
                let t = s.trim();
                match t.strip_prefix('-') {
                    Some(rest) => M {
                        s: format!("{}<mn>{}</mn>", mo("\u{2212}"), esc(rest)),
                        prec: P_NEG,
                        frac: false,
                    },
                    None => atom(format!("<mn>{}</mn>", esc(t))),
                }
            }
            Expr::Str(s) => atom(format!("<ms>{}</ms>", esc(s))),
            Expr::Name(n) => atom(self.name(n, Kind::Auto)),
            Expr::Un(op, a) => self.unary(*op, a)?,
            Expr::Bin(op, a, b) => self.binary(*op, a, b)?,
            Expr::Call(n, args) => {
                let mut parts = Vec::new();
                for a in args {
                    parts.push(self.expr(a)?.s);
                }
                atom(format!(
                    "{}{}",
                    self.name(n, self.call_kind(n)),
                    paren(&parts.join("<mo>,</mo>"))
                ))
            }
            Expr::Postfix(unit, a) => {
                let v = self.expr(a)?;
                let v = self.group_unless(v, P_POW);
                M {
                    s: format!("{v}<mspace width=\"0.2em\"></mspace>{}", self.name(unit, Kind::Unit)),
                    prec: P_MUL,
                    frac: false,
                }
            }
            Expr::Index(base, idx) => {
                let mut subs = Vec::new();
                for i in idx {
                    subs.push(self.expr(i)?.s);
                }
                let subs = row(&[&subs.join("<mo>,</mo>")]);
                match &**base {
                    Expr::Name(n) if !(self.is_unit)(n) => {
                        return Some(atom(self.subscripted_name(n, &subs)));
                    }
                    other => {
                        let b = self.expr(other)?;
                        let b = self.group_unless(b, P_ATOM);
                        atom(format!("<msub>{b}{subs}</msub>"))
                    }
                }
            }
            Expr::Matrix { rows, cols, items } => {
                if *rows == 0 || *cols == 0 || items.len() != rows * cols || rows * cols > 400 {
                    return None;
                }
                let mut t = String::from("<mrow><mo>[</mo><mtable>");
                for r in 0..*rows {
                    t.push_str("<mtr>");
                    for c in 0..*cols {
                        t.push_str(&format!("<mtd>{}</mtd>", self.expr(&items[c * rows + r])?.s));
                    }
                    t.push_str("</mtr>");
                }
                t.push_str("</mtable><mo>]</mo></mrow>");
                atom(t)
            }
            Expr::Range { first, second, last } => {
                let mut s = self.expr(first)?.s;
                if let Some(sec) = second {
                    s = format!("{s}<mo>,</mo>{}", self.expr(sec)?.s);
                }
                M {
                    s: format!("{s}<mo>..</mo>{}", self.expr(last)?.s),
                    prec: P_LOGIC,
                    frac: false,
                }
            }
            Expr::Series {
                product,
                var,
                body,
                bounds,
            } => {
                let sign = if *product { "\u{220F}" } else { "\u{2211}" };
                let body = self.expr(body)?;
                let v = self.name(var, Kind::Var);
                let op = match bounds {
                    Some((lo, hi)) => format!(
                        "<munderover style=\"math-style:normal\"><mo largeop=\"true\">{sign}</mo>{}{}</munderover>",
                        row(&[&v, &mo("="), &self.expr(lo)?.s]),
                        row(&[&self.expr(hi)?.s]),
                    ),
                    None => format!("<munder style=\"math-style:normal\"><mo largeop=\"true\">{sign}</mo>{v}</munder>"),
                };
                M {
                    s: format!("{op}{}", paren(&body.s)),
                    prec: P_MUL,
                    frac: false,
                }
            }
            Expr::Integral {
                var,
                body,
                lower,
                upper,
            } => {
                let body = self.expr(body)?;
                M {
                    s: format!(
                        "<msubsup style=\"math-style:normal\"><mo largeop=\"true\">\u{222B}</mo>{}{}</msubsup>{}<mi>d</mi>{}",
                        row(&[&self.expr(lower)?.s]),
                        row(&[&self.expr(upper)?.s]),
                        if body.prec > P_ADD { body.s.clone() } else { paren(&body.s) },
                        self.name(var, Kind::Var)
                    ),
                    prec: P_MUL,
                    frac: false,
                }
            }
            Expr::Derivative { var, body } => {
                let body = self.expr(body)?;
                M {
                    s: format!(
                        "<mfrac><mi>d</mi>{}</mfrac>{}",
                        row(&["<mi>d</mi>", &self.name(var, Kind::Var)]),
                        paren(&body.s)
                    ),
                    prec: P_MUL,
                    frac: false,
                }
            }
            Expr::Program(_) | Expr::Unsupported { .. } => return None,
        })
    }

    /// `name_(literal sub, index)`: the index joins the literal subscript.
    fn subscripted_name(&self, n: &Name, idx: &str) -> String {
        let plain = Name {
            base: n.base.clone(),
            sub: None,
            unit: n.unit,
            var: n.var,
        };
        let lit = match n.sub.as_deref().filter(|s| !s.is_empty()) {
            Some(s) => format!("{}<mo>,</mo>", sub_text(s)),
            None => String::new(),
        };
        let subs = if lit.is_empty() {
            idx.to_string()
        } else {
            row(&[&lit, idx])
        };
        format!("<msub>{}{subs}</msub>", self.name(&plain, Kind::Var))
    }

    /// An expression made only of units (and the numbers of their powers).
    fn is_unit_expr(&self, e: &Expr) -> bool {
        match e {
            Expr::Name(n) => (self.is_unit)(n),
            Expr::Num(_) => false,
            Expr::Bin(BinOp::Mul | BinOp::Div, a, b) => self.is_unit_expr(a) && self.is_unit_expr(b),
            Expr::Bin(BinOp::Pow, a, b) => self.is_unit_expr(a) && matches!(**b, Expr::Num(_) | Expr::Un(UnOp::Neg, _)),
            _ => false,
        }
    }

    fn group_unless(&self, m: M, min: u8) -> String {
        if m.prec >= min && !(m.frac && min >= P_POW) {
            m.s
        } else {
            paren(&m.s)
        }
    }

    fn unary(&self, op: UnOp, a: &Expr) -> Option<M> {
        let x = self.expr(a)?;
        Some(match op {
            UnOp::Neg => M {
                s: format!("{}{}", mo("\u{2212}"), self.group_unless(x, P_NEG)),
                prec: P_NEG,
                frac: false,
            },
            UnOp::Not => M {
                s: format!("{}{}", mo("\u{00AC}"), self.group_unless(x, P_NEG)),
                prec: P_NEG,
                frac: false,
            },
            UnOp::Abs => atom(format!("<mrow><mo>|</mo>{}<mo>|</mo></mrow>", x.s)),
            UnOp::Sqrt => atom(format!("<msqrt>{}</msqrt>", x.s)),
            UnOp::Factorial => atom(format!("{}{}", self.group_unless(x, P_ATOM), mo("!"))),
            UnOp::Transpose => {
                // A transposed matrix literal is written in brackets, as `(M)^T`.
                let base = if matches!(a, Expr::Matrix { .. }) {
                    paren(&x.s)
                } else {
                    self.group_unless(x, P_ATOM)
                };
                atom(format!("<msup>{base}<mi mathvariant=\"normal\">T</mi></msup>"))
            }
            // The arrow spans the whole operand, and the operand needs no brackets for it.
            // The page draws the arrow itself (class `va`: a line and a head across the whole operand,
            // which no font stretches reliably); the `mo` stays for the Office Math export.
            UnOp::Vectorize => atom(format!(
                "<mover class=\"va\" accent=\"true\"><mrow>{}</mrow><mo stretchy=\"true\">\u{2192}</mo></mover>",
                x.s
            )),
        })
    }

    fn binary(&self, op: BinOp, a: &Expr, b: &Expr) -> Option<M> {
        if op == BinOp::Div {
            let (x, y) = (self.expr(a)?, self.expr(b)?);
            return Some(M {
                s: format!("<mfrac>{}{}</mfrac>", row(&[&x.s]), row(&[&y.s])),
                prec: P_ATOM,
                frac: true,
            });
        }
        if op == BinOp::Pow {
            let (x, y) = (self.expr(a)?, self.expr(b)?);
            return Some(M {
                s: format!("<msup>{}{}</msup>", self.group_unless(x, P_ATOM), row(&[&y.s])),
                prec: P_POW,
                frac: false,
            });
        }
        let (x, y) = (self.expr(a)?, self.expr(b)?);
        // A number written before its unit is read `100 ft`: no multiplication dot.
        if op == BinOp::Mul && matches!(a, Expr::Num(_)) && self.is_unit_expr(b) {
            return Some(M {
                s: format!(
                    "{}<mspace width=\"0.2em\"></mspace>{}",
                    x.s,
                    self.group_unless(y, P_MUL + 1)
                ),
                prec: P_MUL,
                frac: false,
            });
        }
        let (sym, prec, class) = match op {
            BinOp::Add => ("+", P_ADD, ""),
            BinOp::Sub => ("\u{2212}", P_ADD, ""),
            BinOp::Mul => ("\u{22C5}", P_MUL, " class=\"dot\""),
            BinOp::Eq => ("=", P_CMP, " class=\"beq\""),
            BinOp::Ne => ("\u{2260}", P_CMP, ""),
            BinOp::Lt => ("<", P_CMP, ""),
            BinOp::Gt => (">", P_CMP, ""),
            BinOp::Le => ("\u{2264}", P_CMP, ""),
            BinOp::Ge => ("\u{2265}", P_CMP, ""),
            BinOp::And => ("\u{2227}", P_LOGIC, ""),
            BinOp::Or => ("\u{2228}", P_LOGIC, ""),
            BinOp::Xor => ("\u{2295}", P_LOGIC, ""),
            BinOp::Div | BinOp::Pow => return None,
        };
        // Left operands bind from their own strength; a right operand of equal strength needs
        // brackets under `-` (a - (b - c)) and a negative or lower one under a product.
        let left = self.group_unless(x, prec);
        let right_min = if matches!(op, BinOp::Sub) || prec == P_MUL {
            prec + 1
        } else {
            prec
        };
        let right = self.group_unless(y, right_min);
        Some(M {
            s: format!("{left}<mo{class}>{}</mo>{right}", esc(sym)),
            prec,
            frac: false,
        })
    }
}

/// One identifier as a token: a unit or function name is upright, a single letter is left to the
/// renderer's italic, a longer variable name is italic by class.
fn ident(s: &str, upright: bool) -> String {
    let text = esc(s);
    if upright {
        format!("<mi mathvariant=\"normal\">{text}</mi>")
    } else if s.chars().count() > 1 {
        format!("<mi class=\"v\">{text}</mi>")
    } else {
        format!("<mi>{text}</mi>")
    }
}

fn sub_text(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_digit()) {
        format!("<mn>{}</mn>", esc(s))
    } else {
        ident(s, false)
    }
}

/// `T'` is `T` with one prime.
fn split_primes(base: &str) -> (String, usize) {
    let stem = base.trim_end_matches(['\'', '\u{2032}']);
    (stem.to_string(), base.chars().count() - stem.chars().count())
}

/// The element a typeset equation sits in. `class` is styled by the page (colours follow the page).
pub fn wrap(inner: &str) -> String {
    format!("<math class=\"imp-m\">{inner}</math>")
}

/// A whole output line for a typeset equation (a text line the engine passes through as HTML).
pub fn line(math: &str) -> String {
    format!("'<div class=\"imp-eq\">{math}</div>")
}
