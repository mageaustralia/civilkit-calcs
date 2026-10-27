//! Expressions: sheet spelling to Calcpad spelling.
//!
//! The grammar is deliberately small. An expression is numbers (an exponent is expanded, because
//! the engine has no `e` notation), names, operators, function calls with comma separated
//! arguments, `[...]` vectors and matrices, and units written after a number. Anything outside
//! that returns an error, which the caller reads as "this line is prose".

use crate::names;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(String),
    Ident(String),
    Str(String),
    Sup(String),
    Op(String),
    Lp,
    Rp,
    Lb,
    Rb,
    Comma,
    Semi,
    Dot,
}

/// How an expression is read.
pub struct Ctx<'a> {
    /// Names the sheet defines, in the sheet's spelling (used to rename function calls).
    pub defined: &'a HashSet<String>,
    /// A condition: a single `=` means equality.
    pub cond: bool,
}

const SUPERSCRIPTS: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹⁻⁺";

fn sup_char(c: char) -> Option<char> {
    let i = SUPERSCRIPTS.chars().position(|s| s == c)?;
    Some(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '+'][i])
}

fn tokenize(s: &str) -> Result<Vec<(Tok, bool)>, String> {
    let cs: Vec<char> = s.chars().collect();
    let mut out: Vec<(Tok, bool)> = Vec::new();
    let mut i = 0;
    let mut space = false;
    while i < cs.len() {
        // Other spellings of the multiplication dot are the engine's middle dot.
        let c = if matches!(cs[i], '\u{22C5}' | '\u{2219}' | '\u{2022}') { '\u{b7}' } else { cs[i] };
        if c.is_whitespace() {
            space = true;
            i += 1;
            continue;
        }
        let after_operand = matches!(out.last(), Some((Tok::Ident(_) | Tok::Rp | Tok::Rb, _)));
        let tok;
        if c.is_ascii_digit() || (c == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit()) && !after_operand) {
            let st = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            if i < cs.len() && cs[i] == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') {
                let mut j = i + 1;
                if j < cs.len() && (cs[j] == '+' || cs[j] == '-') {
                    j += 1;
                }
                if j < cs.len() && cs[j].is_ascii_digit() {
                    while j < cs.len() && cs[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            tok = Tok::Num(cs[st..i].iter().collect());
        } else if c == '°' {
            // A degree sign, alone or leading a unit name (°C, °F).
            let st = i;
            i += 1;
            while i < cs.len() && names::is_name_char(cs[i]) {
                i += 1;
            }
            tok = Tok::Ident(cs[st..i].iter().collect());
        } else if names::is_name_start(c) || c == '_' {
            let st = i;
            while i < cs.len() && names::is_name_char(cs[i]) {
                i += 1;
            }
            // The micro sign is the engine's Greek mu.
            tok = Tok::Ident(cs[st..i].iter().collect::<String>().replace('\u{b5}', "\u{3bc}"));
        } else if sup_char(c).is_some() {
            let mut t = String::new();
            while i < cs.len() {
                match sup_char(cs[i]) {
                    Some(d) => t.push(d),
                    None => break,
                }
                i += 1;
            }
            tok = Tok::Sup(t);
        } else if c == '"' {
            let st = i + 1;
            i += 1;
            while i < cs.len() && cs[i] != '"' {
                i += 1;
            }
            if i >= cs.len() {
                return Err("unterminated string".into());
            }
            tok = Tok::Str(cs[st..i].iter().collect());
            i += 1;
        } else {
            i += 1;
            let two: String = cs[i - 1..(i + 1).min(cs.len())].iter().collect();
            tok = match two.as_str() {
                "<=" | ">=" | "!=" | "==" | "&&" | "||" | "->" => {
                    i += 1;
                    Tok::Op(two)
                }
                _ => match c {
                    '(' => Tok::Lp,
                    ')' => Tok::Rp,
                    '[' => Tok::Lb,
                    ']' => Tok::Rb,
                    ',' => Tok::Comma,
                    ';' => Tok::Semi,
                    '.' => Tok::Dot,
                    '+' | '-' | '*' | '/' | '^' | '%' | '<' | '>' | '!' | '=' | '≤' | '≥' | '≠' | '≡' | '∧' | '∨'
                    | '×' | '·' | '÷' | '−' | '⦼' | '\\' | '∠' | '⊕' => Tok::Op(c.to_string()),
                    _ => return Err(format!("unexpected character '{c}'")),
                },
            };
        }
        out.push((tok, space));
        space = false;
    }
    Ok(out)
}

/// Writes a number the engine reads: an exponent is expanded into digits.
fn number(raw: &str) -> String {
    let Some(epos) = raw.find(['e', 'E']) else {
        return raw.to_string();
    };
    let (mant, exp) = (&raw[..epos], &raw[epos + 1..]);
    let Ok(e) = exp.parse::<i32>() else {
        return raw.to_string();
    };
    if e.abs() > 18 {
        return format!("{mant}*10^{e}");
    }
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let mut digits = format!("{int}{frac}");
    let point = int.len() as i32 + e;
    let s = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        digits.push_str(&"0".repeat(point as usize - digits.len()));
        digits
    } else {
        format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
    };
    let s = s.trim_start_matches('0');
    let s = if s.starts_with('.') || s.is_empty() {
        format!("0{s}")
    } else {
        s.to_string()
    };
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[derive(Clone, Copy, PartialEq)]
enum K {
    Num,
    Unit,
    Ident,
    Close,
    Op,
    Open,
    Dot,
    Sup,
    Start,
}

fn operand(k: K) -> bool {
    matches!(k, K::Num | K::Unit | K::Ident | K::Close | K::Sup)
}

fn matching_rb(t: &[(Tok, bool)], open: usize) -> Option<usize> {
    let mut d = 0;
    for (i, (tok, _)) in t.iter().enumerate().skip(open) {
        match tok {
            Tok::Lb => d += 1,
            Tok::Rb => {
                d -= 1;
                if d == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn matrix(inner: &[(Tok, bool)], ctx: &Ctx) -> Result<String, String> {
    let mut rows: Vec<Vec<&[(Tok, bool)]>> = vec![vec![]];
    let mut start = 0;
    let mut depth = 0i32;
    for (i, (tok, _)) in inner.iter().enumerate() {
        match tok {
            Tok::Lp => depth += 1,
            Tok::Rp => depth -= 1,
            Tok::Lb | Tok::Rb => return Err("brackets cannot be nested".into()),
            Tok::Comma if depth == 0 => {
                rows.last_mut().unwrap().push(&inner[start..i]);
                start = i + 1;
            }
            Tok::Semi if depth == 0 => {
                rows.last_mut().unwrap().push(&inner[start..i]);
                rows.push(vec![]);
                start = i + 1;
            }
            _ => {}
        }
    }
    rows.last_mut().unwrap().push(&inner[start..]);
    let mut text: Vec<Vec<String>> = Vec::new();
    for r in &rows {
        let mut row = Vec::new();
        for cell in r {
            if cell.is_empty() {
                return Err("empty element".into());
            }
            row.push(slice(cell, ctx)?);
        }
        text.push(row);
    }
    let all: Vec<String>;
    if text.len() == 1 || text.iter().all(|r| r.len() == 1) {
        all = text.into_iter().flatten().collect();
        return Ok(format!("[{}]", all.join("; ")));
    }
    let w = text[0].len();
    if text.iter().any(|r| r.len() != w) {
        return Err("the rows of a matrix must have the same length".into());
    }
    Ok(format!(
        "[{}]",
        text.iter().map(|r| r.join("; ")).collect::<Vec<_>>().join(" | ")
    ))
}

fn slice(t: &[(Tok, bool)], ctx: &Ctx) -> Result<String, String> {
    let mut out = String::new();
    let mut prev = K::Start;
    let mut paren = 0i32;
    // State of a unit written after a number (`kN*m^2`): its names are never renamed.
    let (mut in_unit, mut chain, mut caret) = (false, false, false);
    let mut i = 0;
    while i < t.len() {
        let (tok, space) = (&t[i].0, t[i].1);
        let next_is_lp = matches!(t.get(i + 1), Some((Tok::Lp, _)));
        let piece: String;
        let kind;
        let (mut unitish, mut new_chain, mut new_caret) = (false, false, false);
        match tok {
            Tok::Num(n) => {
                if operand(prev) {
                    return Err("two values in a row".into());
                }
                piece = number(n);
                kind = K::Num;
                unitish = in_unit && caret;
            }
            Tok::Ident(name) if (name == "and" || name == "or") && !next_is_lp => {
                if !operand(prev) {
                    return Err(format!("'{name}' needs a value before it"));
                }
                piece = if name == "and" { "∧" } else { "∨" }.into();
                kind = K::Op;
            }
            Tok::Ident(name) => {
                if prev == K::Num || (chain && !space) {
                    piece = name.clone();
                    kind = K::Unit;
                    unitish = true;
                } else {
                    if operand(prev) {
                        return Err("two values in a row".into());
                    }
                    piece = if next_is_lp && !ctx.defined.contains(name) {
                        name.clone()
                    } else {
                        names::rename(name)
                    };
                    kind = K::Ident;
                }
            }
            Tok::Str(_) => return Err("text is not an expression".into()),
            Tok::Sup(s) => {
                if !operand(prev) {
                    return Err("exponent without a value".into());
                }
                piece = format!("^{s}");
                kind = K::Sup;
                unitish = in_unit;
            }
            Tok::Op(o) => {
                kind = K::Op;
                piece = match o.as_str() {
                    "<=" => "≤",
                    ">=" => "≥",
                    "!=" => "≠",
                    "==" => "≡",
                    "&&" => "∧",
                    "||" => "∨",
                    "·" | "×" => "*",
                    "÷" => "/",
                    "−" => "-",
                    "->" => return Err("'->' must come last".into()),
                    "=" if ctx.cond => "≡",
                    "=" => return Err("'=' is not allowed here".into()),
                    other => other,
                }
                .into();
                match o.as_str() {
                    "*" | "/" | "·" | "×" | "÷" if in_unit && !space => {
                        unitish = true;
                        new_chain = true;
                    }
                    "^" if in_unit => {
                        unitish = true;
                        new_caret = true;
                    }
                    "-" | "−" if in_unit && caret => {
                        unitish = true;
                        new_caret = true;
                    }
                    _ => {}
                }
            }
            Tok::Lp => {
                if operand(prev) && prev != K::Ident {
                    return Err("a value cannot be followed by '('".into());
                }
                paren += 1;
                piece = "(".into();
                kind = K::Open;
            }
            Tok::Rp => {
                paren -= 1;
                if paren < 0 {
                    return Err("unbalanced ')'".into());
                }
                piece = ")".into();
                kind = K::Close;
            }
            Tok::Lb => {
                if operand(prev) {
                    return Err("a value cannot be followed by '['".into());
                }
                let Some(close) = matching_rb(t, i) else {
                    return Err("unbalanced '['".into());
                };
                piece = matrix(&t[i + 1..close], ctx)?;
                i = close;
                kind = K::Close;
            }
            Tok::Rb => return Err("unbalanced ']'".into()),
            Tok::Comma => {
                if paren == 0 {
                    return Err("unexpected ','".into());
                }
                piece = ";".into();
                kind = K::Op;
            }
            Tok::Semi => return Err("unexpected ';'".into()),
            Tok::Dot => {
                if !matches!(prev, K::Ident | K::Close) {
                    return Err("unexpected '.'".into());
                }
                piece = ".".into();
                kind = K::Dot;
            }
        }
        in_unit = unitish;
        chain = new_chain;
        caret = new_caret;
        if space && !out.is_empty() && piece != ")" && !out.ends_with('(') && kind != K::Dot && prev != K::Dot {
            out.push(' ');
        }
        out.push_str(&piece);
        prev = kind;
        i += 1;
    }
    if paren != 0 {
        return Err("unbalanced '('".into());
    }
    if out.trim().is_empty() {
        return Err("empty expression".into());
    }
    Ok(out)
}

/// Translates an expression. `Err` means the text is not an expression of the sheet grammar.
pub fn translate(s: &str, ctx: &Ctx) -> Result<String, String> {
    slice(&tokenize(s)?, ctx)
}

/// Translates a unit: names, `*`, `/`, `^`, numbers and parentheses; names are never renamed.
pub fn translate_unit(s: &str) -> Result<String, String> {
    let toks = tokenize(s)?;
    let mut out = String::new();
    for (tok, _) in &toks {
        match tok {
            Tok::Ident(n) => out.push_str(n),
            Tok::Num(n) => out.push_str(n),
            Tok::Sup(s) => {
                out.push('^');
                out.push_str(s);
            }
            Tok::Lp => out.push('('),
            Tok::Rp => out.push(')'),
            Tok::Op(o) => out.push_str(match o.as_str() {
                "·" | "×" | "*" => "*",
                "÷" | "/" => "/",
                "^" => "^",
                "-" | "−" => "-",
                _ => return Err(format!("'{o}' is not part of a unit")),
            }),
            _ => return Err("not a unit".into()),
        }
    }
    if out.is_empty() {
        return Err("empty unit".into());
    }
    Ok(out)
}

/// Byte positions of the characters of `s` that are outside quotes and brackets.
pub fn top_level(s: &str) -> Vec<(usize, char)> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut quote = false;
    for (i, c) in s.char_indices() {
        if c == '"' {
            quote = !quote;
            continue;
        }
        if quote {
            continue;
        }
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            _ if depth == 0 => out.push((i, c)),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Result<String, String> {
        let d = HashSet::new();
        translate(
            s,
            &Ctx {
                defined: &d,
                cond: false,
            },
        )
    }

    #[test]
    fn numbers_and_units() {
        assert_eq!(t("1.23e6 mm³").unwrap(), "1230000 mm^3");
        assert_eq!(t("300 MPa").unwrap(), "300 MPa");
        assert_eq!(t("2.5e-3").unwrap(), "0.0025");
        assert_eq!(t("5 kN·m").unwrap(), "5 kN*m");
        assert_eq!(t("fy * Ze").unwrap(), "fy * Ze");
        assert_eq!(t("3 psi + 2 psi").unwrap(), "3 psi + 2 psi");
        assert_eq!(t("phi * psi").unwrap(), "φ * ψ");
        assert_eq!(t("9.81 m/s²").unwrap(), "9.81 m/s^2");
    }

    #[test]
    fn calls_and_matrices() {
        assert_eq!(t("max(a, b)").unwrap(), "max(a; b)");
        assert_eq!(t("[1; 2; 3]").unwrap(), "[1; 2; 3]");
        assert_eq!(t("[1, 2, 3]").unwrap(), "[1; 2; 3]");
        assert_eq!(t("[1, 2; 3, 4]").unwrap(), "[1; 2 | 3; 4]");
        assert!(t("[1, 2; 3]").is_err());
        assert_eq!(t("v.2").unwrap(), "v.2");
    }

    #[test]
    fn prose_is_rejected() {
        assert!(t("see below").is_err());
        assert!(t("5 kN applied here").is_err());
        assert!(t("a = 3").is_err());
    }

    #[test]
    fn operators() {
        let d = HashSet::new();
        let c = Ctx {
            defined: &d,
            cond: true,
        };
        assert_eq!(translate("a <= 1 and b != 2", &c).unwrap(), "a ≤ 1 ∧ b ≠ 2");
        assert_eq!(translate("a = 2", &c).unwrap(), "a ≡ 2");
    }
}
