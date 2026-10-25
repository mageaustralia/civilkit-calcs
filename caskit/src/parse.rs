//! Text to expression. Grammar (lowest precedence first): `+ -`, `* /`, unary `-`/`+`, `^` (right
//! associative; the exponent may start with `-`), then numbers, names, calls and parentheses.
//! There is no implicit multiplication. `pi` and `e` (lower case) are constants; the functions are
//! `sin`, `cos`, `ln`, `exp` and `sqrt`. Decimal numbers are read exactly (`0.1` is 1/10).

use crate::error::{CasError, CasResult};
use crate::expr::Expr;
use crate::rat::Rat;

/// An equation `lhs = rhs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Equation {
    /// Left-hand side.
    pub lhs: Expr,
    /// Right-hand side.
    pub rhs: Expr,
}

impl Equation {
    /// Builds an equation from two expressions.
    pub fn new(lhs: Expr, rhs: Expr) -> Equation {
        Equation { lhs, rhs }
    }

    /// Parses `"lhs = rhs"`.
    pub fn parse(s: &str) -> CasResult<Equation> {
        parse_equation(s)
    }

    /// `lhs - rhs`, the expression that is zero at a solution.
    pub fn residual(&self) -> CasResult<Expr> {
        Expr::sub(self.lhs.clone(), self.rhs.clone())
    }
}

/// Parses an expression.
pub fn parse(s: &str) -> CasResult<Expr> {
    let mut p = Parser {
        src: s.as_bytes(),
        pos: 0,
    };
    let e = p.expr()?;
    p.skip_ws();
    if p.pos != p.src.len() {
        return Err(p.err("unexpected input"));
    }
    Ok(e)
}

/// Parses `"lhs = rhs"` (exactly one `=`).
pub fn parse_equation(s: &str) -> CasResult<Equation> {
    let mut parts = s.split('=');
    let (l, r) = (parts.next().unwrap_or(""), parts.next());
    match (r, parts.next()) {
        (Some(r), None) => Ok(Equation {
            lhs: parse(l)?,
            rhs: parse(r)?,
        }),
        (None, _) => Err(CasError::Parse {
            msg: "an equation needs an '='".into(),
            pos: s.len(),
        }),
        _ => Err(CasError::Parse {
            msg: "more than one '='".into(),
            pos: s.len(),
        }),
    }
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn err(&self, msg: &str) -> CasError {
        CasError::Parse {
            msg: msg.to_string(),
            pos: self.pos,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.src.get(self.pos).copied()
    }

    fn expr(&mut self) -> CasResult<Expr> {
        let mut terms = vec![self.term()?];
        loop {
            match self.peek() {
                Some(b'+') => {
                    self.pos += 1;
                    terms.push(self.term()?);
                }
                Some(b'-') => {
                    self.pos += 1;
                    let t = self.term()?;
                    terms.push(Expr::neg(t)?);
                }
                _ => break,
            }
        }
        Expr::sum(terms)
    }

    fn term(&mut self) -> CasResult<Expr> {
        let mut acc = self.unary()?;
        loop {
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    let r = self.unary()?;
                    acc = Expr::mul(acc, r)?;
                }
                Some(b'/') => {
                    self.pos += 1;
                    let r = self.unary()?;
                    acc = Expr::div(acc, r)?;
                }
                _ => break,
            }
        }
        Ok(acc)
    }

    fn unary(&mut self) -> CasResult<Expr> {
        match self.peek() {
            Some(b'-') => {
                self.pos += 1;
                Expr::neg(self.unary()?)
            }
            Some(b'+') => {
                self.pos += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> CasResult<Expr> {
        let base = self.atom()?;
        if self.peek() == Some(b'^') {
            self.pos += 1;
            let exp = self.unary()?;
            return Expr::pow(base, exp);
        }
        Ok(base)
    }

    fn atom(&mut self) -> CasResult<Expr> {
        match self.peek() {
            None => Err(self.err("unexpected end of input")),
            Some(b'(') => {
                self.pos += 1;
                let e = self.expr()?;
                if self.peek() != Some(b')') {
                    return Err(self.err("expected ')'"));
                }
                self.pos += 1;
                Ok(e)
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.number(),
            Some(c) if c.is_ascii_alphabetic() || c == b'_' => self.name(),
            Some(_) => Err(self.err("unexpected character")),
        }
    }

    fn number(&mut self) -> CasResult<Expr> {
        let start = self.pos;
        let mut digits: i128 = 0;
        let mut scale: u32 = 0;
        let mut seen_dot = false;
        let mut any = false;
        while let Some(&c) = self.src.get(self.pos) {
            if c.is_ascii_digit() {
                digits = digits
                    .checked_mul(10)
                    .and_then(|d| d.checked_add((c - b'0') as i128))
                    .ok_or(CasError::Overflow)?;
                if seen_dot {
                    scale += 1;
                }
                any = true;
            } else if c == b'.' && !seen_dot {
                seen_dot = true;
            } else {
                break;
            }
            self.pos += 1;
        }
        if !any {
            self.pos = start;
            return Err(self.err("expected a number"));
        }
        let mut exp10: i32 = 0;
        if matches!(self.src.get(self.pos), Some(b'e') | Some(b'E')) {
            let mut j = self.pos + 1;
            let mut neg = false;
            if matches!(self.src.get(j), Some(b'+') | Some(b'-')) {
                neg = self.src[j] == b'-';
                j += 1;
            }
            if self.src.get(j).is_some_and(|c| c.is_ascii_digit()) {
                let mut v: i32 = 0;
                while let Some(&c) = self.src.get(j) {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    v = v.saturating_mul(10).saturating_add((c - b'0') as i32);
                    j += 1;
                }
                if v > 30 {
                    return Err(self.err("exponent too large"));
                }
                exp10 = if neg { -v } else { v };
                self.pos = j;
            }
        }
        let mut r = Rat::new(digits, 1)?;
        let net = exp10 - scale as i32;
        let ten = Rat::int(10);
        r = r.mul(ten.pow_i(net as i128)?)?;
        Ok(Expr::Num(r))
    }

    fn name(&mut self) -> CasResult<Expr> {
        let start = self.pos;
        while let Some(&c) = self.src.get(self.pos) {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let name = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("");
        if self.peek() == Some(b'(') {
            self.pos += 1;
            let arg = self.expr()?;
            if self.peek() != Some(b')') {
                return Err(self.err("expected ')' after the function argument"));
            }
            self.pos += 1;
            return match name {
                "sin" => Expr::sin(arg),
                "cos" => Expr::cos(arg),
                "ln" => Expr::ln(arg),
                "exp" => Expr::exp(arg),
                "sqrt" => Expr::sqrt(arg),
                _ => {
                    self.pos = start;
                    Err(self.err(&format!("unknown function {name}")))
                }
            };
        }
        Ok(match name {
            "pi" => Expr::pi(),
            "e" => Expr::e(),
            _ => Expr::sym(name),
        })
    }
}
