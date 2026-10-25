//! The translator: a [`Sheet`] in, Calcpad text and a report out. It does no arithmetic and
//! evaluates nothing; every number it writes is a literal that was in the source.
//!
//! Safety rule: a line of output is built only from (a) text that goes through the escaping in
//! [`text_line`], (b) names that passed [`valid_ident`], (c) numbers that parsed as finite
//! numbers, (d) entries of the mapping tables, or (e) a placeholder line that starts with
//! [`GAP_MARKER`]. Nothing else from the input reaches the output.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ir::*;
use crate::mathml::{self, MathMl};
use crate::picture::{self, SvgError};
use crate::report::{Code, ImportReport, Issue, KindStats, Severity};
use crate::tables::{self, Constant, FnRule};
use crate::{Asset, ImportOptions};

mod complex;
mod exits;
mod growth;
mod multi;
mod nested;
mod plot;
mod slices;
mod solve;
mod strings;
mod trycatch;

/// Every placeholder line starts with this (after the leading `'` that makes it a text line).
pub const GAP_MARKER: &str = "[import-gap";

/// Rows closer than this many points (by baseline) are one row for reading order.
pub const ROW_TOLERANCE: f64 = 4.0;

// Precedence levels of the rendered Calcpad text.
const P_LOGIC: u8 = 1;
const P_CMP: u8 = 2;
const P_ADD: u8 = 3;
const P_MUL: u8 = 4;
const P_NEG: u8 = 5;
const P_POW: u8 = 6;
const P_ATOM: u8 = 7;

struct R {
    s: String,
    prec: u8,
}

/// A program run symbolically: locals are substituted into what uses them, an `if` becomes the
/// engine's `if(c; a; b)` over the rest of the program, a `return` ends its path, and a `for`
/// over a short literal range is unrolled. The program is then one expression, with no
/// statements and no names of its own. `Err` says why a program is not that: a loop whose
/// iterations are not known, an element write, `break`/`continue`, or too many paths.
/// The marker for a path along which no statement gave a value (replaced before the result is used).
const NO_VALUE: &str = "\u{1}no-value";

/// The result of running a program symbolically: the expression, and whether some path of the
/// program gave no value (the source stops there) and was given the value of a neighbouring path.
fn symexec_program(stmts: &[PStmt], unroll: bool, known: &HashSet<String>) -> Result<(Expr, bool), String> {
    let mut budget = if unroll { 600usize } else { 600_000 };
    // A local that is only ever given texts (a message set by `if`s) is the empty text where no
    // `if` set it, so a path no condition covers has a value.
    let mut env: HashMap<String, Expr> = HashMap::new();
    for name in text_locals(stmts) {
        env.insert(name, Expr::Str(String::new()));
    }
    // A local the sheet has no name for, read on a path where no statement set it, has no value
    // there (the source stops).
    let mut assigned: HashSet<String> = HashSet::new();
    assigned_names(stmts, &mut assigned);
    for name in assigned {
        if !known.contains(&name) && !env.contains_key(&name) {
            env.insert(name, Expr::Name(Name::new(NO_VALUE)));
        }
    }
    let e = symexec(stmts, &env, None, &mut budget, unroll)?;
    let mut uncovered = false;
    match strip_no_value(e, &mut uncovered) {
        Some(e) if mentions(&e, NO_VALUE) => {
            Err("a local is used on a path where no statement gave it a value (the source would stop there)".into())
        }
        Some(e) => Ok((e, uncovered)),
        None => Err("the program does not end with a value".into()),
    }
}

/// What a path with no value gives where another path of the same program has one: the engine's
/// undefined (not a number), which `if` keeps away from the paths that have a value.
fn no_value() -> Expr {
    Expr::Un(UnOp::Sqrt, Box::new(Expr::Num("-1".into())))
}

/// Replaces the paths that gave no value in an `if` chain by the undefined value. `None` when no
/// path gave a value.
fn strip_no_value(e: Expr, uncovered: &mut bool) -> Option<Expr> {
    match e {
        Expr::Name(ref n) if n.base == NO_VALUE => None,
        Expr::Call(n, mut args) if n.base == "if" && n.sub.is_none() && args.len() == 3 => {
            let b = strip_no_value(args.pop()?, uncovered);
            let a = strip_no_value(args.pop()?, uncovered);
            let c = args.pop()?;
            match (a, b) {
                (Some(a), Some(b)) => Some(Expr::Call(n, vec![c, a, b])),
                // The source stops on a path that gave no value; the engine's undefined (a square root
                // of a negative number) is the same, and never a neighbouring branch's number.
                (Some(x), None) => {
                    *uncovered = true;
                    Some(Expr::Call(n, vec![c, x, no_value()]))
                }
                (None, Some(x)) => {
                    *uncovered = true;
                    Some(Expr::Call(n, vec![c, no_value(), x]))
                }
                (None, None) => None,
            }
        }
        other => Some(other),
    }
}

/// Locals of a program whose every assignment is a text literal.
fn text_locals(stmts: &[PStmt]) -> Vec<String> {
    fn walk(stmts: &[PStmt], text: &mut HashMap<String, bool>) {
        for st in stmts {
            match st {
                PStmt::Local(PTarget::Var(n), e) => {
                    let all = text.entry(tables::normalise_name(&n.flat())).or_insert(true);
                    *all &= matches!(e, Expr::Str(_));
                }
                PStmt::Local(PTarget::Element { base, .. }, _) => {
                    text.insert(tables::normalise_name(&base.flat()), false);
                }
                PStmt::For { var, body, .. } => {
                    text.insert(tables::normalise_name(&var.flat()), false);
                    walk(body, text);
                }
                PStmt::While { body, .. } => walk(body, text),
                PStmt::If { branches, otherwise } => {
                    for (_, b) in branches {
                        walk(b, text);
                    }
                    if let Some(o) = otherwise {
                        walk(o, text);
                    }
                }
                _ => {}
            }
        }
    }
    let mut text: HashMap<String, bool> = HashMap::new();
    walk(stmts, &mut text);
    let mut v: Vec<String> = text.into_iter().filter(|(_, t)| *t).map(|(n, _)| n).collect();
    v.sort();
    v
}

const LOOP_NEEDS: &str = "a loop whose number of iterations is not a short literal range cannot be one expression";

fn symexec(
    stmts: &[PStmt],
    env: &HashMap<String, Expr>,
    cur: Option<Expr>,
    budget: &mut usize,
    unroll: bool,
) -> Result<Expr, String> {
    if *budget == 0 {
        return Err("too many branches and loop iterations to write as one expression".into());
    }
    *budget -= 1;
    let mut env = env.clone();
    let mut cur = cur;
    for (k, st) in stmts.iter().enumerate() {
        let rest = &stmts[k + 1..];
        match st {
            PStmt::Local(PTarget::Var(n), e) => {
                let v = subst(e, &env).ok_or("a local's value would be captured by an integration variable")?;
                // The value of a program is that of the last statement run, an assignment included.
                cur = Some(v.clone());
                env.insert(tables::normalise_name(&n.flat()), v);
            }
            PStmt::Local(PTarget::Element { .. } | PTarget::Slice { .. }, _) => {
                return Err(
                    "an element assignment needs an array built by statements, which an expression cannot hold".into(),
                )
            }
            PStmt::Expr(e) if !rest.is_empty() => {
                // The value of a program is the last expression it evaluates: this one stands
                // unless a later statement gives another.
                cur = Some(
                    subst(e, &env)
                        .ok_or_else(|| "a local's value would be captured by an integration variable".to_string())?,
                );
            }
            PStmt::Expr(e) | PStmt::Return(e) => {
                return subst(e, &env)
                    .ok_or_else(|| "a local's value would be captured by an integration variable".to_string());
            }
            PStmt::If { branches, otherwise }
                if branches.iter().all(|(_, b)| matches!(b.as_slice(), [PStmt::Expr(_)]))
                    && otherwise
                        .as_ref()
                        .is_none_or(|o| matches!(o.as_slice(), [PStmt::Expr(_)])) =>
            {
                // Every branch only gives a value: the value so far is that of the branch taken,
                // or what it was. Nothing is copied for the rest of the program.
                let value_of = |b: &[PStmt]| match b {
                    [PStmt::Expr(e)] => subst(e, &env),
                    _ => None,
                };
                let mut value = match otherwise {
                    Some(o) => value_of(o),
                    None => Some(cur.clone().unwrap_or_else(|| Expr::Name(Name::new(NO_VALUE)))),
                }
                .ok_or("a local's value would be captured by an integration variable")?;
                for (c, b) in branches.iter().rev() {
                    let cond = subst(c, &env).ok_or("a condition would capture a local")?;
                    let v = value_of(b).ok_or("a local's value would be captured by an integration variable")?;
                    value = Expr::Call(Name::new("if"), vec![cond, v, value]);
                }
                cur = Some(value);
            }
            PStmt::If { branches, otherwise }
                if branches
                    .iter()
                    .map(|(_, b)| b)
                    .chain(otherwise.iter())
                    .all(|b| !b.is_empty() && b.iter().all(|x| matches!(x, PStmt::Local(PTarget::Var(_), _)))) =>
            {
                // Every branch only sets locals: each local becomes `if(c; value; ...)` over its
                // old value, so the statements after the `if` are read once, not once per branch.
                let none = || Expr::Name(Name::new(NO_VALUE));
                let apply = |body: &[PStmt]| -> Result<(HashMap<String, Expr>, Option<Expr>), String> {
                    let mut e = env.clone();
                    let mut last = None;
                    let mut changed: Vec<String> = Vec::new();
                    for x in body {
                        if let PStmt::Local(PTarget::Var(n), v) = x {
                            let v =
                                subst(v, &e).ok_or("a local's value would be captured by an integration variable")?;
                            let key = tables::normalise_name(&n.flat());
                            last = Some(v.clone());
                            e.insert(key.clone(), v);
                            changed.push(key);
                        }
                    }
                    let mut out = HashMap::new();
                    for k in changed {
                        out.insert(k.clone(), e[&k].clone());
                    }
                    Ok((out, last))
                };
                let mut sets: Vec<(Expr, HashMap<String, Expr>, Option<Expr>)> = Vec::new();
                for (c, b) in branches {
                    let cond = subst(c, &env).ok_or("a condition would capture a local")?;
                    let (vars, last) = apply(b)?;
                    sets.push((cond, vars, last));
                }
                let other = match otherwise {
                    Some(o) => Some(apply(o)?),
                    None => None,
                };
                let mut names: Vec<String> = sets
                    .iter()
                    .flat_map(|(_, v, _)| v.keys().cloned())
                    .chain(other.iter().flat_map(|(v, _)| v.keys().cloned()))
                    .collect();
                names.sort();
                names.dedup();
                for name in names {
                    let mut value = other
                        .as_ref()
                        .and_then(|(v, _)| v.get(&name).cloned())
                        .or_else(|| env.get(&name).cloned())
                        .unwrap_or_else(none);
                    for (c, v, _) in sets.iter().rev() {
                        let old = value.clone();
                        let taken = v
                            .get(&name)
                            .cloned()
                            .unwrap_or_else(|| env.get(&name).cloned().unwrap_or_else(none));
                        value = Expr::Call(Name::new("if"), vec![c.clone(), taken, old]);
                    }
                    env.insert(name, value);
                }
                {
                    // The value so far is the last value the branch that ran set, or what it was
                    // when no branch ran (a later `if` must not hide an earlier one that ran).
                    let mut value = match &other {
                        Some((_, last)) => last.clone().or_else(|| cur.clone()),
                        None => cur.clone(),
                    }
                    .unwrap_or_else(none);
                    for (c, _, last) in sets.iter().rev() {
                        let taken = last.clone().or_else(|| cur.clone()).unwrap_or_else(none);
                        value = Expr::Call(Name::new("if"), vec![c.clone(), taken, value]);
                    }
                    cur = Some(value);
                }
            }
            PStmt::If { branches, otherwise } => {
                let mut arms = Vec::new();
                for (c, body) in branches {
                    let cond = subst(c, &env).ok_or("a condition would capture a local")?;
                    let mut path: Vec<PStmt> = body.clone();
                    path.extend(rest.iter().cloned());
                    arms.push((cond, symexec(&path, &env, cur.clone(), budget, unroll)?));
                }
                let mut path: Vec<PStmt> = otherwise.clone().unwrap_or_default();
                path.extend(rest.iter().cloned());
                let mut value = symexec(&path, &env, cur.clone(), budget, unroll)?;
                for (cond, v) in arms.into_iter().rev() {
                    value = Expr::Call(Name::new("if"), vec![cond, v, value]);
                }
                return Ok(value);
            }
            PStmt::For { var, range, body } => {
                let Expr::Range {
                    first,
                    second: None,
                    last,
                } = range
                else {
                    return Err(LOOP_NEEDS.into());
                };
                let (Expr::Num(a), Expr::Num(z)) = (&**first, &**last) else {
                    return Err(LOOP_NEEDS.into());
                };
                let (Some(a), Some(z)) = (is_int_literal(a), is_int_literal(z)) else {
                    return Err(LOOP_NEEDS.into());
                };
                if !unroll || (z - a).abs() > 63 {
                    return Err(LOOP_NEEDS.into());
                }
                if body.iter().any(has_break) {
                    return Err("break or continue in a loop cannot be unrolled".into());
                }
                let values: Vec<i64> = if a <= z {
                    (a..=z).collect()
                } else {
                    (z..=a).rev().collect()
                };
                let mut path: Vec<PStmt> = Vec::new();
                for v in values {
                    path.push(PStmt::Local(PTarget::Var(var.clone()), Expr::Num(v.to_string())));
                    path.extend(body.iter().cloned());
                }
                path.extend(rest.iter().cloned());
                return symexec(&path, &env, cur, budget, unroll);
            }
            PStmt::While { .. } => return Err(LOOP_NEEDS.into()),
            PStmt::Break | PStmt::Continue => return Err("break or continue outside a loop".into()),
            PStmt::Unsupported(w) => return Err(format!("{w} is not read")),
        }
    }
    Ok(cur.unwrap_or_else(|| Expr::Name(Name::new(NO_VALUE))))
}

fn subst(e: &Expr, env: &HashMap<String, Expr>) -> Option<Expr> {
    let b = |x: &Expr| subst(x, env).map(Box::new);
    let without = |v: &str| -> HashMap<String, Expr> {
        env.iter()
            .filter(|(k, _)| k.as_str() != v)
            .map(|(k, x)| (k.clone(), x.clone()))
            .collect()
    };
    Some(match e {
        Expr::Name(n) => match env.get(&tables::normalise_name(&n.flat())) {
            Some(v) => v.clone(),
            None => e.clone(),
        },
        Expr::Num(_) | Expr::Str(_) | Expr::Unsupported { .. } => e.clone(),
        Expr::Un(o, x) => Expr::Un(*o, b(x)?),
        Expr::Bin(o, x, y) => Expr::Bin(*o, b(x)?, b(y)?),
        Expr::Call(f, args) => Expr::Call(f.clone(), args.iter().map(|a| subst(a, env)).collect::<Option<_>>()?),
        Expr::Postfix(u, x) => Expr::Postfix(u.clone(), b(x)?),
        Expr::Index(x, idx) => Expr::Index(b(x)?, idx.iter().map(|a| subst(a, env)).collect::<Option<_>>()?),
        Expr::Matrix { rows, cols, items } => Expr::Matrix {
            rows: *rows,
            cols: *cols,
            items: items.iter().map(|a| subst(a, env)).collect::<Option<_>>()?,
        },
        Expr::Range { first, second, last } => Expr::Range {
            first: b(first)?,
            second: match second {
                Some(s) => Some(b(s)?),
                None => None,
            },
            last: b(last)?,
        },
        Expr::Integral {
            var,
            body,
            lower,
            upper,
        } => {
            let v = tables::normalise_name(&var.flat());
            // A substituted value that mentions the integration variable would be captured by it.
            if env.values().any(|x| mentions(x, &v)) {
                return None;
            }
            Expr::Integral {
                var: var.clone(),
                body: Box::new(subst(body, &without(&v))?),
                lower: b(lower)?,
                upper: b(upper)?,
            }
        }
        Expr::Series {
            product,
            var,
            body,
            bounds,
        } => {
            let v = tables::normalise_name(&var.flat());
            if env.values().any(|x| mentions(x, &v)) {
                return None;
            }
            Expr::Series {
                product: *product,
                var: var.clone(),
                body: Box::new(subst(body, &without(&v))?),
                bounds: match bounds {
                    Some((lo, hi)) => Some((b(lo)?, b(hi)?)),
                    None => None,
                },
            }
        }
        Expr::Derivative { var, body } => {
            let v = tables::normalise_name(&var.flat());
            if env.values().any(|x| mentions(x, &v)) {
                return None;
            }
            Expr::Derivative {
                var: var.clone(),
                body: Box::new(subst(body, &without(&v))?),
            }
        }
        Expr::Program(_) => return None,
    })
}

/// The unit `deg` (or `°`): a display unit for an angle, which the source holds in radians.
fn is_degree(e: &Expr) -> bool {
    matches!(e, Expr::Name(n) if n.sub.is_none() && matches!(n.base.as_str(), "deg" | "°" | "degree"))
}

/// An expression made of numbers and arithmetic only (no name, so no unit).
fn numeric_only(e: &Expr) -> bool {
    match e {
        Expr::Num(_) => true,
        Expr::Un(UnOp::Neg, a) => numeric_only(a),
        Expr::Bin(BinOp::Mul | BinOp::Div | BinOp::Pow | BinOp::Add | BinOp::Sub, a, b) => {
            numeric_only(a) && numeric_only(b)
        }
        _ => false,
    }
}

fn mentions(e: &Expr, name: &str) -> bool {
    match e {
        Expr::Name(n) => tables::normalise_name(&n.flat()) == name,
        Expr::Un(_, a) | Expr::Postfix(_, a) => mentions(a, name),
        Expr::Bin(_, a, b) => mentions(a, name) || mentions(b, name),
        Expr::Call(f, args) => tables::normalise_name(&f.flat()) == name || args.iter().any(|a| mentions(a, name)),
        Expr::Index(b, idx) => mentions(b, name) || idx.iter().any(|a| mentions(a, name)),
        Expr::Matrix { items, .. } => items.iter().any(|a| mentions(a, name)),
        Expr::Range { first, second, last } => {
            mentions(first, name) || second.as_ref().is_some_and(|s| mentions(s, name)) || mentions(last, name)
        }
        Expr::Integral {
            var,
            body,
            lower,
            upper,
        } => {
            mentions(lower, name)
                || mentions(upper, name)
                || (tables::normalise_name(&var.flat()) != name && mentions(body, name))
        }
        Expr::Series { var, body, bounds, .. } => {
            bounds
                .as_ref()
                .is_some_and(|(lo, hi)| mentions(lo, name) || mentions(hi, name))
                || (tables::normalise_name(&var.flat()) != name && mentions(body, name))
        }
        Expr::Derivative { var, body } => tables::normalise_name(&var.flat()) != name && mentions(body, name),
        Expr::Num(_) | Expr::Str(_) | Expr::Unsupported { .. } => false,
        Expr::Program(_) => true,
    }
}

fn atom(s: impl Into<String>) -> R {
    R {
        s: s.into(),
        prec: P_ATOM,
    }
}

struct Pending {
    code: Code,
    detail: Option<String>,
    root: Option<Code>,
}

/// Sorts regions into the order the source evaluates them: top to bottom, then left to right,
/// where regions whose baselines are within [`ROW_TOLERANCE`] of the first region of a row share
/// that row. The baseline (not the top edge) is used because a tall region next to a short one
/// sits on the same line while their top edges differ.
pub fn reading_order(regions: &mut Vec<Region>) {
    let rows = reading_rows(std::mem::take(regions));
    regions.extend(rows.into_iter().flatten());
}

/// The same order as [`reading_order`], kept as the rows it was found in (each row left to right).
pub fn reading_rows(regions: Vec<Region>) -> Vec<Vec<Region>> {
    let key = |r: &Region| r.baseline.unwrap_or(r.top);
    let mut idx: Vec<usize> = (0..regions.len()).collect();
    idx.sort_by(|&a, &b| {
        key(&regions[a])
            .partial_cmp(&key(&regions[b]))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    // Group into rows, then order each row by left edge.
    let mut rows_idx: Vec<Vec<usize>> = Vec::new();
    let mut i = 0;
    while i < idx.len() {
        let anchor = key(&regions[idx[i]]);
        let mut j = i;
        while j < idx.len() && key(&regions[idx[j]]) - anchor <= ROW_TOLERANCE {
            j += 1;
        }
        let mut row: Vec<usize> = idx[i..j].to_vec();
        row.sort_by(|&a, &b| {
            regions[a]
                .left
                .partial_cmp(&regions[b].left)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        });
        rows_idx.push(row);
        i = j;
    }
    // A value written beside a label that wraps (the label's baseline is its first line, the
    // value sits on a later one) is on the label's row: it lies to the right of the whole row
    // and its baseline falls inside the label's height.
    let mut k = 0;
    while k + 1 < rows_idx.len() {
        let text = rows_idx[k]
            .iter()
            .filter(|&&a| matches!(regions[a].kind, RegionKind::Text(_)) && regions[a].width > 0.0)
            .max_by(|&&a, &&b| {
                regions[a]
                    .left
                    .partial_cmp(&regions[b].left)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();
        let merge = text.is_some_and(|t| {
            let t = &regions[t];
            let right_of_row = rows_idx[k]
                .iter()
                .map(|&a| regions[a].left + regions[a].width)
                .fold(f64::MIN, f64::max);
            rows_idx[k + 1].iter().all(|&b| {
                let r = &regions[b];
                !matches!(r.kind, RegionKind::Text(_)) && r.left >= right_of_row - 1.0 && key(r) <= t.top + t.height
            })
        });
        if merge {
            let next = rows_idx.remove(k + 1);
            rows_idx[k].extend(next);
            rows_idx[k].sort_by(|&a, &b| {
                regions[a]
                    .left
                    .partial_cmp(&regions[b].left)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.cmp(&b))
            });
        } else {
            k += 1;
        }
    }
    let mut old: Vec<Option<Region>> = regions.into_iter().map(Some).collect();
    rows_idx
        .into_iter()
        .map(|row| row.into_iter().filter_map(|src| old[src].take()).collect())
        .collect()
}

/// Calcpad identifiers: a letter, then letters, digits, `_` and primes.
pub fn valid_ident(s: &str) -> bool {
    let mut ch = s.chars();
    match ch.next() {
        Some(c) if c.is_alphabetic() => {}
        _ => return false,
    }
    ch.all(|c| c.is_alphanumeric() || c == '_' || c == '\u{2032}' || c == '\u{2033}')
}

fn is_int_literal(s: &str) -> Option<i64> {
    s.parse::<i64>().ok()
}

/// Calcpad does not read `1.5e-3`: rewrites a literal with an exponent as plain digits (exactly,
/// by moving the decimal point) or, for a large exponent, as `(m*10^e)`.
fn expand_literal(s: &str) -> Option<String> {
    let Some(epos) = s.find(['e', 'E']) else {
        return Some(s.to_string());
    };
    let (mant, exp) = (&s[..epos], &s[epos + 1..]);
    let exp: i32 = exp.trim_start_matches('+').parse().ok()?;
    let (neg, mant) = match mant.strip_prefix('-') {
        Some(m) => (true, m),
        None => (false, mant),
    };
    let (int, frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    if int.is_empty() && frac.is_empty() || !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let sign = if neg { "-" } else { "" };
    if exp.abs() > 20 {
        return Some(format!("({}{}*10^({}))", sign, mant, exp));
    }
    let digits = format!("{}{}", int, frac);
    let point = int.len() as i32 + exp; // position of the decimal point within `digits`
    let body = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
    } else {
        format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
    };
    // Tidy leading zeros of the integer part and trailing zeros of the fraction.
    let body = match body.split_once('.') {
        Some((i, f)) => {
            let i = i.trim_start_matches('0');
            let f = f.trim_end_matches('0');
            let i = if i.is_empty() { "0" } else { i };
            if f.is_empty() {
                i.to_string()
            } else {
                format!("{}.{}", i, f)
            }
        }
        None => {
            let b = body.trim_start_matches('0');
            if b.is_empty() {
                "0".to_string()
            } else {
                b.to_string()
            }
        }
    };
    Some(format!("{}{}", sign, body))
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(16)
        .collect()
}

/// Escapes one line of prose for a Calcpad text line (without the leading quote mark).
fn escape_text(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '\'' => o.push_str("''"),
            '"' => o.push_str("\"\""),
            c if c.is_control() => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

/// One line of an unconverted region as shown to the reader: `true` when it is Calcpad maths that
/// goes in a `#noc` block (typeset, never evaluated), `false` for plain text.
type GapLine = (bool, String);

/// A conservative check that a rendered expression is Calcpad the engine parses, so that a `#noc`
/// block never shows an error: only the characters and shapes the renderer produces for maths,
/// balanced brackets, no stand-in, no string, no `..`, no indexing of a call result.
fn plausible_calcpad(s: &str) -> bool {
    if s.trim().is_empty() || s.len() > 3000 || s.contains("..") || s.contains(").") || s.contains("].") {
        return false;
    }
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            c if c.is_alphanumeric() => {}
            ' ' | '_' | '.' | ',' | ';' | '+' | '-' | '*' | '/' | '^' | '|' | '=' | '<' | '>' | '≡' | '≠' | '≤'
            | '≥' | '∧' | '∨' | '⊕' | '!' | '$' | '@' | ':' | '%' | '′' | '°' => {}
            _ => return false,
        }
    }
    depth == 0
}

/// A range variable `i := a .. z`: its bounds as Calcpad text, and as integers when both
/// are literals and `a <= z`. `i := a, b .. z` has the step `b - a` as well.
#[derive(Debug, Clone, Default)]
struct RangeInfo {
    a: String,
    z: String,
    lit: Option<(i64, i64)>,
    step: Option<String>,
}

/// An enclosing `for` loop while a program is lowered: its variable, the expression for the
/// largest index it reaches (when that can be written ahead of the loop), and the output line the
/// loop starts at.
struct LoopCtx {
    var: String,
    size: Option<String>,
    start: usize,
}

/// Lines before the loops, one header per range, and the size each range reaches.
type RangeLoops = (Vec<String>, Vec<String>, Vec<String>, Vec<String>);

/// State while a function body is lowered to an expression block.
struct FnBlk {
    /// How often each name occurs in the whole body, and in the innermost loop body being lowered.
    total: HashMap<String, usize>,
    inside: Option<HashMap<String, usize>>,
    /// Source names known to hold an array (received, copied or created by the body).
    arrays: HashSet<String>,
    /// Locals that start as not a number (they may be read on a path that never set them).
    starts: Vec<String>,
    /// Names every arm of an `if`/`else` sets: they have a value after it, whichever arm ran.
    exhaustive: HashSet<String>,
    /// Arrays the body writes by element without having made them: the statement before which each is
    /// allocated, and why the others cannot be.
    allocs: Vec<(usize, String)>,
    no_alloc: HashMap<String, String>,
    writes: Vec<ElemWrite>,
    /// The result and done flags of an early `return`, when the body has one.
    ret: Option<exits::RetFlags>,
    /// The break and continue flags of the loops being lowered, innermost last.
    flags: Vec<exits::LoopFlags>,
    /// Arrays made at exactly the size their writes need (no write grows them).
    exact: HashSet<String>,
    /// Arrays that start with one zero where first written and grow with each write: the statement
    /// they are made before, the name and the number of indexes.
    grow_starts: Vec<(usize, String, usize)>,
    /// What the body does to each array by element.
    scan: HashMap<String, growth::WriteScan>,
}

/// What one dimension of an array needs: a literal index, or the end of a loop (plus an offset).
#[derive(Clone)]
enum Need {
    Lit(i64),
    Loop { first: Expr, last: Expr, off: i64 },
}

/// One element write to an array the function has not made.
struct ElemWrite {
    base: String,
    needs: Vec<Need>,
    conditional: bool,
    /// The statement of the function body it is in.
    at: usize,
}

/// The writes in `stmts` to arrays by element, with what each index needs. A write the sizes of which
/// cannot be known (an index that is not a loop variable or a literal, a write inside a `while`) is
/// reported against its array.
fn element_writes(
    stmts: &[PStmt],
    at: usize,
    conditional: bool,
    loops: &mut Vec<(String, Option<(Expr, Expr)>)>,
    in_while: bool,
    out: &mut Vec<ElemWrite>,
    bad: &mut HashMap<String, String>,
) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Element { base, index }, _) => {
                let b = tables::normalise_name(&base.flat());
                if in_while {
                    bad.insert(b, "it is written inside a while loop, whose length is not known".into());
                    continue;
                }
                let mut needs = Vec::new();
                for e in index {
                    let (name, off) = match e {
                        Expr::Num(s) => match is_int_literal(s) {
                            Some(v) => {
                                needs.push(Need::Lit(v));
                                continue;
                            }
                            None => {
                                bad.insert(b.clone(), "an index is not a whole number".into());
                                break;
                            }
                        },
                        Expr::Name(n) => (tables::normalise_name(&n.flat()), 0),
                        Expr::Bin(op @ (BinOp::Add | BinOp::Sub), l, r) => match (&**l, &**r) {
                            (Expr::Name(n), Expr::Num(c)) if is_int_literal(c).is_some() => {
                                let c = is_int_literal(c).unwrap_or(0);
                                (
                                    tables::normalise_name(&n.flat()),
                                    if *op == BinOp::Add { c } else { -c },
                                )
                            }
                            _ => {
                                bad.insert(b.clone(), "an index is not a loop variable or a number".into());
                                break;
                            }
                        },
                        _ => {
                            bad.insert(b.clone(), "an index is not a loop variable or a number".into());
                            break;
                        }
                    };
                    match loops.iter().rev().find(|(v, _)| *v == name) {
                        Some((_, Some((first, last)))) => needs.push(Need::Loop {
                            first: first.clone(),
                            last: last.clone(),
                            off,
                        }),
                        _ => {
                            bad.insert(
                                b.clone(),
                                format!("the index {name} is not a loop variable with known ends"),
                            );
                            break;
                        }
                    }
                }
                if needs.len() == index.len() && (1..=2).contains(&needs.len()) {
                    out.push(ElemWrite {
                        base: b,
                        needs,
                        conditional,
                        at,
                    });
                }
            }
            PStmt::For { var, range, body } => {
                let ends = match range {
                    Expr::Range {
                        first,
                        second: None,
                        last,
                    } => Some(((**first).clone(), (**last).clone())),
                    _ => None,
                };
                loops.push((tables::normalise_name(&var.flat()), ends));
                element_writes(body, at, conditional, loops, in_while, out, bad);
                loops.pop();
            }
            PStmt::While { body, .. } => element_writes(body, at, conditional, loops, true, out, bad),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    element_writes(b, at, true, loops, in_while, out, bad);
                }
                if let Some(o) = otherwise {
                    element_writes(o, at, true, loops, in_while, out, bad);
                }
            }
            _ => {}
        }
    }
}

/// Names a list of statements assigns whole (not by element).
fn whole_assigned(stmts: &[PStmt], out: &mut HashSet<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), _) => {
                out.insert(tables::normalise_name(&n.flat()));
            }
            PStmt::For { var, body, .. } => {
                out.insert(tables::normalise_name(&var.flat()));
                whole_assigned(body, out);
            }
            PStmt::While { body, .. } => whole_assigned(body, out),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    whole_assigned(b, out);
                }
                if let Some(o) = otherwise {
                    whole_assigned(o, out);
                }
            }
            _ => {}
        }
    }
}

impl FnBlk {
    /// The array is made with one zero where it is first written and grows with each write, when the
    /// source's own result does not depend on whether some write happens: one of its writes is not
    /// under an `if`, and every write has the same number of indexes.
    fn grow_from(&mut self, name: &str) {
        let why = match self.scan.get(name) {
            Some(info) if info.ranks.len() != 1 || !(1..=2).contains(&info.ranks[0]) => {
                "it is indexed with a different number of indexes"
            }
            // Written only under conditions, but never read on a path that did not write it: the
            // source's array exists wherever it is used.
            Some(info) if !info.uncond && !info.reads_after_writes => "it is written only under a condition",
            Some(info) => {
                self.grow_starts.push((info.first, name.to_string(), info.ranks[0]));
                return;
            }
            None => "it is not written",
        };
        self.no_alloc.entry(name.to_string()).or_insert_with(|| why.to_string());
    }

    /// A local set for the first time here: it must have a value on every path that reads it. The
    /// reason it does not, if it does not.
    fn first_set_problem(&self, flat: &str, guard: Option<&str>, loops: usize) -> Option<String> {
        if guard.is_some() && !(loops == 0 && self.exhaustive.contains(flat)) {
            return Some(format!(
                "{flat} is first set under a condition, so it has no value on the other path (the source would stop there, the engine would have no value)"
            ));
        }
        if loops > 0 {
            let confined = self
                .inside
                .as_ref()
                .is_some_and(|m| m.get(flat) == self.total.get(flat));
            if !confined {
                return Some(format!(
                    "{flat} is first set inside a loop and read outside it, so it has no value if the loop does not run"
                ));
            }
        }
        None
    }
}

/// The local that holds the value of a program that is written with statements.
const RESULT_LOCAL: &str = "val_of_program";

/// Does some path of the program end in a way whose value is not read: a loop or an element
/// assignment last, or an `if` with no `else` after a statement that is not a value (the value is
/// then that statement's when the condition fails)? An assignment as the last statement is read:
/// its value is the program's.
fn ends_without_value(stmts: &[PStmt]) -> bool {
    match stmts.last() {
        Some(PStmt::Local(PTarget::Element { .. }, _) | PStmt::For { .. } | PStmt::While { .. }) => true,
        Some(PStmt::If { branches, otherwise }) => {
            let before = stmts.len().checked_sub(2).map(|k| &stmts[k]);
            let falls_through = otherwise.is_none() && matches!(before, Some(PStmt::Local(..)));
            falls_through
                || branches.iter().any(|(_, b)| ends_without_value(b))
                || otherwise.as_ref().is_some_and(|o| ends_without_value(o))
        }
        _ => false,
    }
}

/// Does a program keep its value in a way a final `value` statement cannot show: it ends in a
/// statement that is not an expression, or has value statements before the end (the last one run
/// is the program's value)?
fn needs_result_form(stmts: &[PStmt]) -> bool {
    fn has_expr(list: &[PStmt]) -> bool {
        list.iter().any(|st| match st {
            PStmt::Expr(_) => true,
            PStmt::For { body, .. } | PStmt::While { body, .. } => has_expr(body),
            PStmt::If { branches, otherwise } => {
                branches.iter().any(|(_, b)| has_expr(b)) || otherwise.as_ref().is_some_and(|o| has_expr(o))
            }
            _ => false,
        })
    }
    match stmts.split_last() {
        Some((PStmt::Expr(_) | PStmt::Return(_), body)) => has_expr(body),
        Some(_) => true,
        None => false,
    }
}

/// The program with its value kept in a local: every value statement assigns it (so the last one
/// run wins, as in the source), an assignment or `return` as the last thing a path does assigns it
/// too, and the program ends by reading it. A path that never sets it leaves it without a value.
fn result_form(stmts: &[PStmt]) -> Vec<PStmt> {
    let res = Name::new(RESULT_LOCAL);
    fn walk(list: &[PStmt], tail: bool, res: &Name) -> Vec<PStmt> {
        let n = list.len();
        let mut out = Vec::new();
        for (k, st) in list.iter().enumerate() {
            let last = tail && k + 1 == n;
            match st {
                PStmt::Expr(e) => out.push(PStmt::Local(PTarget::Var(res.clone()), e.clone())),
                PStmt::Return(e) if last => out.push(PStmt::Local(PTarget::Var(res.clone()), e.clone())),
                PStmt::Local(PTarget::Var(v), _) if last => {
                    out.push(st.clone());
                    out.push(PStmt::Local(PTarget::Var(res.clone()), Expr::Name(v.clone())));
                }
                PStmt::For { var, range, body } => out.push(PStmt::For {
                    var: var.clone(),
                    range: range.clone(),
                    body: walk(body, false, res),
                }),
                PStmt::While { cond, body } => out.push(PStmt::While {
                    cond: cond.clone(),
                    body: walk(body, false, res),
                }),
                PStmt::If { branches, otherwise } => out.push(PStmt::If {
                    branches: branches.iter().map(|(c, b)| (c.clone(), walk(b, last, res))).collect(),
                    otherwise: otherwise.as_ref().map(|o| walk(o, last, res)),
                }),
                other => out.push(other.clone()),
            }
        }
        out
    }
    let mut out = walk(stmts, true, &res);
    out.push(PStmt::Expr(Expr::Name(res)));
    out
}

/// How often each identifier occurs in a list of statements.
fn name_counts(stmts: &[PStmt]) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    let mut c = stmts.to_vec();
    for st in c.iter_mut() {
        st.for_each_name_mut(&mut |n| *m.entry(tables::normalise_name(&n.flat())).or_insert(0) += 1);
    }
    m
}

/// The names a list of statements assigns to (a name or an element of it).
fn assigned_names(stmts: &[PStmt], out: &mut HashSet<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), _)
            | PStmt::Local(PTarget::Element { base: n, .. }, _)
            | PStmt::Local(PTarget::Slice { base: n, .. }, _) => {
                out.insert(tables::normalise_name(&n.flat()));
            }
            PStmt::For { body, .. } | PStmt::While { body, .. } => assigned_names(body, out),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    assigned_names(b, out);
                }
                if let Some(o) = otherwise {
                    assigned_names(o, out);
                }
            }
            _ => {}
        }
    }
}

/// A literal column vector (`n x 1`, n > 1), a row literal transposed, or a program that ends in one.
fn is_col_literal(e: &Expr) -> bool {
    match e {
        Expr::Matrix { rows, cols, .. } => *cols == 1 && *rows > 1,
        Expr::Un(UnOp::Transpose, a) => matches!(&**a, Expr::Matrix { rows: 1, cols, .. } if *cols > 1),
        Expr::Program(stmts) => matches!(stmts.last(), Some(PStmt::Expr(x) | PStmt::Return(x)) if is_col_literal(x)),
        _ => false,
    }
}

/// A matrix literal with several rows and several columns (or the transpose of one).
fn literal_matrix(e: &Expr) -> bool {
    match e {
        Expr::Matrix { rows, cols, .. } => *rows > 1 && *cols > 1,
        Expr::Un(UnOp::Transpose, a) => literal_matrix(a),
        _ => false,
    }
}

/// A program that returns a local it writes by element with two indexes.
fn returns_matrix(stmts: &[PStmt]) -> bool {
    fn writes_two(list: &[PStmt], name: &str) -> bool {
        list.iter().any(|st| match st {
            PStmt::Local(PTarget::Element { base, index }, _) => {
                index.len() == 2 && tables::normalise_name(&base.flat()) == name
            }
            PStmt::For { body, .. } | PStmt::While { body, .. } => writes_two(body, name),
            PStmt::If { branches, otherwise } => {
                branches.iter().any(|(_, b)| writes_two(b, name))
                    || otherwise.as_ref().is_some_and(|o| writes_two(o, name))
            }
            _ => false,
        })
    }
    match stmts.last() {
        Some(PStmt::Expr(Expr::Name(n)) | PStmt::Return(Expr::Name(n))) => {
            writes_two(stmts, &tables::normalise_name(&n.flat()))
        }
        _ => false,
    }
}

fn may_return(st: &PStmt) -> bool {
    match st {
        PStmt::Return(_) => true,
        PStmt::For { body, .. } | PStmt::While { body, .. } => body.iter().any(may_return),
        PStmt::If { branches, otherwise } => {
            branches.iter().any(|(_, b)| b.iter().any(may_return))
                || otherwise.as_ref().is_some_and(|o| o.iter().any(may_return))
        }
        _ => false,
    }
}

fn has_break(st: &PStmt) -> bool {
    match st {
        PStmt::Break | PStmt::Continue => true,
        PStmt::For { body, .. } | PStmt::While { body, .. } => body.iter().any(has_break),
        PStmt::If { branches, otherwise } => {
            branches.iter().any(|(_, b)| b.iter().any(has_break))
                || otherwise.as_ref().is_some_and(|o| o.iter().any(has_break))
        }
        _ => false,
    }
}

/// Names a program assigns or loops over: the only names whose value is not known before it runs.
fn program_locals(stmts: &[PStmt], out: &mut HashSet<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), _) => {
                out.insert(tables::normalise_name(&n.flat()));
            }
            PStmt::Local(PTarget::Element { base, .. }, _) => {
                out.insert(tables::normalise_name(&base.flat()));
            }
            PStmt::For { var, body, .. } => {
                out.insert(tables::normalise_name(&var.flat()));
                program_locals(body, out);
            }
            PStmt::While { body, .. } => program_locals(body, out),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    program_locals(b, out);
                }
                if let Some(o) = otherwise {
                    program_locals(o, out);
                }
            }
            _ => {}
        }
    }
}

/// Array names that the program sets to a literal 0 element by element.
fn zero_assigned(stmts: &[PStmt], out: &mut Vec<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Element { base, .. }, Expr::Num(s)) if s.trim().parse::<f64>() == Ok(0.0) => {
                let b = tables::normalise_name(&base.flat());
                if !out.contains(&b) {
                    out.push(b);
                }
            }
            PStmt::For { body, .. } | PStmt::While { body, .. } => zero_assigned(body, out),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    zero_assigned(b, out);
                }
                if let Some(o) = otherwise {
                    zero_assigned(o, out);
                }
            }
            _ => {}
        }
    }
}

/// The first value added to (or taken from) an element of `base` in program order, `a[i] <- a[i] + x`,
/// with the loop variables above the statement replaced by the first value of their range. `None`
/// when the array is never accumulated into. The value's units are then those of the first term.
fn first_accumulated(
    stmts: &[PStmt],
    base: &str,
    env: &mut HashMap<String, Expr>,
    locals: &HashSet<String>,
) -> Option<Result<Expr, String>> {
    let reads_base = |e: &Expr| matches!(e, Expr::Index(b, _) if matches!(&**b, Expr::Name(n) if tables::normalise_name(&n.flat()) == base));
    for st in stmts {
        let found = match st {
            PStmt::Local(PTarget::Element { base: b, .. }, e)
                if tables::normalise_name(&b.flat()) == base && mentions(e, base) =>
            {
                let term = match e {
                    Expr::Bin(BinOp::Add | BinOp::Sub, a, c) if reads_base(a) && !mentions(c, base) => Some(&**c),
                    Expr::Bin(BinOp::Add, a, c) if reads_base(c) && !mentions(a, base) => Some(&**a),
                    _ => None,
                };
                Some(match term {
                    None => Err(format!("{base} is added to in a form that is not a plain sum")),
                    Some(t) => match subst(t, env) {
                        None => Err("the first value added cannot be rewritten".to_string()),
                        Some(t) => {
                            let blockers: Vec<&String> = {
                                let mut v: Vec<&String> = locals.iter().filter(|l| mentions(&t, l)).collect();
                                v.sort();
                                v
                            };
                            if blockers.is_empty() {
                                Ok(t)
                            } else {
                                Err(format!(
                                    "the first value added to {base} reads {}, which the program sets, so its units are not known before the loops run",
                                    blockers.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                                ))
                            }
                        }
                    },
                })
            }
            PStmt::For { var, range, body } => {
                let v = tables::normalise_name(&var.flat());
                let prev = env.get(&v).cloned();
                if let Expr::Range { first, .. } = range {
                    env.insert(v.clone(), (**first).clone());
                }
                let r = first_accumulated(body, base, env, locals);
                match prev {
                    Some(p) => env.insert(v, p),
                    None => env.remove(&v),
                };
                r
            }
            PStmt::While { body, .. } => first_accumulated(body, base, env, locals),
            PStmt::If { branches, otherwise } => branches
                .iter()
                .map(|(_, b)| b.as_slice())
                .chain(otherwise.iter().map(|o| o.as_slice()))
                .find_map(|b| first_accumulated(b, base, env, locals)),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

struct Tr {
    origin: i64,
    defined: HashSet<String>,
    funcs: HashMap<String, usize>,
    /// The text literals of the sheet; the engine holds a text as its 1-based position here.
    str_table: Vec<String>,
    /// Names that hold a text now, with the codes they can have.
    str_vars: HashMap<String, Vec<usize>>,
    /// Functions that return a text, with the codes they can return.
    str_funcs: HashMap<String, Vec<usize>>,
    /// Functions whose value is an array (a matrix literal, or an `if` that gives one).
    array_funcs: HashSet<String>,
    /// Matrices that hold texts as codes among their numbers (labels of a data table), and whether
    /// the matrix literal being translated may do so.
    str_mats: HashSet<String>,
    str_mat_ok: bool,
    /// Text boxes whose stored text is a number: `str2num` of such a name is the name.
    num_texts: HashSet<String>,
    /// Text boxes that hold no text: `str2num` of one is 0 (the source has no number there).
    empty_texts: HashSet<String>,
    /// The value the file saved for a name where the sheet evaluates it (`name =`): for a name read
    /// from a data file, whose own definition keeps no result.
    saved_by_name: HashMap<String, Expr>,
    /// Set while the arguments of a function that reads or writes a file are translated: a file
    /// name is not a text value (the function is the gap).
    io_args: bool,
    /// Set while a text name or a call of a text function is translated as a text (a leaf of
    /// `str_expr`); anywhere else such a name is a gap, because its code is not a number.
    str_allow: bool,
    /// Names defined by regions that did not convert, with the code that ultimately blocks them.
    gap_names: HashMap<String, Code>,
    /// Defined names known to hold an array.
    arrays: HashSet<String>,
    /// Names (variables and functions) known to give a column vector, which the source multiplies as
    /// a dot product; the engine's `*` of two vectors is element by element.
    col_vecs: HashSet<String>,
    params: Vec<String>,
    /// While a plot is rendered: the plot variable, which an index that mentions it reads by interpolation.
    plot_param: Option<String>,
    /// Names the sheet shows in a unit somewhere (`f = ` with a unit): the unit the author reads the
    /// quantity in, used to put a plot's unitless axis in the source's base units.
    unit_hints: HashMap<String, Expr>,
    /// Range variables with literal integer bounds, usable as loop bounds of element assignments.
    ranges: HashMap<String, RangeInfo>,
    /// Names defined inside components that are not run (normalised).
    component_names: HashSet<String>,
    /// While a program is lowered: its local names and the names they were given in the output.
    locals: HashMap<String, String>,
    /// Output names of locals known to hold an array created in the program.
    local_arrays: HashSet<String>,
    /// Output names of local arrays whose elements were set to a unitless 0 literal.
    zero_started: HashSet<String>,
    /// Output names of function-body locals that may have no value on some path, with the flag that
    /// says whether one was set: a read is `if(flag; x; undefined)`.
    partial: HashMap<String, String>,
    /// A function the sheet defines more than once: the engine name of its current definition (the
    /// engine warns when a name is defined again, so each later definition has a name of its own).
    fn_alias: HashMap<String, String>,
    /// Arrays that hold an array in each element, kept as a matrix with one row per element.
    nested: HashSet<String>,
    /// The parameters and the value of each worksheet function that is one expression.
    fn_defs: HashMap<String, (Vec<String>, Expr)>,
    /// The sheet has complex numbers; the names that hold one (`name` is the real part and
    /// `name__im` the imaginary part), and the complex locals of the program being split.
    cx_any: bool,
    cplx: HashSet<String>,
    cx_locals: HashSet<String>,
    /// Arrays known to have several rows and several columns (a plot draws a trace per column).
    matrices: HashSet<String>,
    /// The complex program just split returns an array with several rows and columns.
    cx_matrix: bool,
    /// The equations of the solve block being read, until its solver region.
    solve_cons: Vec<Expr>,
    /// Output names of local arrays that grow as they are written (their size is not known ahead).
    growable: HashSet<String>,
    /// Lines a program lowered to statements starts with (the allocation of its growing arrays).
    prologue: Vec<String>,
    /// For a local array started from a literal 0 and later added to (source name): the first value
    /// added, with the loop variables above it replaced by their first values, or why that value's
    /// units cannot be known before the loops run.
    zero_units: HashMap<String, Result<Expr, String>>,
    /// Arrays made by assigning one element at a literal index (name, engine index of that element).
    elem_built: HashMap<String, usize>,
    prog_n: usize,
    /// The file's named paragraph styles, and the colours the text used.
    text_styles: HashMap<String, TextStyle>,
    colors: std::collections::BTreeSet<String>,
    /// Assignments to temporaries that an expression needs before the line it is on (the engine
    /// cannot index the result of a call: `t = f(x)` then `t.(1; 2)`).
    hoisted: Vec<String>,
    loop_stack: Vec<LoopCtx>,
    /// Inside an `if` or `while` of the program being lowered.
    cond_depth: usize,
    /// Result and done variables when the program returns early.
    ret: Option<(String, String)>,
    /// Rendering a gap region for the reader: pieces that cannot be converted keep their real
    /// names instead of a `?stand-in?`, and no issue is recorded.
    readable: bool,
    /// Set while rendering for the reader when something written cannot be Calcpad maths.
    readable_bad: bool,
    /// Rendering the author's own equation for display (never computed): the original names, the
    /// transpose as a superscript, a row as a row. Set only around `display_*`.
    display: bool,
    /// A math region was written to the page (the page then carries the maths styling).
    maths_shown: bool,
    /// Set while displaying when a name was written as the author's name instead of the engine's.
    display_renamed: bool,
    /// Engine name -> the author's name, for the names renamed to keep a variable apart from a unit.
    renamed: HashMap<String, String>,
    /// The decimal places the file shows results with, and the `#format` currently in force.
    default_precision: Option<i32>,
    /// The `#format` in force (`None`: the default).
    fmt_cur: Option<String>,
    cur: Vec<Pending>,
    uses_trig: bool,
    lines: Vec<String>,
    report: ImportReport,
    depth: usize,
    opts: ImportOptions,
    assets: Vec<Asset>,
    used_paths: HashSet<String>,
    asset_total: usize,
    has_references: bool,
    /// A result was written for the engine to typeset as MathML (the page then turns that on).
    uses_val_ml: bool,
    /// The typeset form of the unconverted region being shown, when it has one.
    pending_ml: Option<String>,
}

const MAX_RENDER_DEPTH: usize = 300;

impl Tr {
    fn issue(&mut self, code: Code, detail: Option<String>) {
        self.cur.push(Pending {
            code,
            detail,
            root: None,
        });
    }

    // ---- expressions ----------------------------------------------------------------------

    fn expr(&mut self, e: &Expr) -> R {
        if self.depth > MAX_RENDER_DEPTH {
            self.issue(Code::TooDeep, None);
            return atom("?too-deep?");
        }
        // Complex values are written out as real ones (a pair per value); one left over is a gap.
        if let Some(r) = self.realified(e) {
            return match r {
                Ok(real) => {
                    self.depth += 1;
                    let out = self.expr_inner(&real);
                    self.depth -= 1;
                    out
                }
                Err(why) => {
                    self.issue(Code::UnknownElement, Some(format!("complex number: {why}")));
                    atom("?complex?")
                }
            };
        }
        self.depth += 1;
        let r = self.expr_inner(e);
        self.depth -= 1;
        r
    }

    /// Renders `e` and wraps it in parentheses when it binds looser than `min`.
    fn operand(&mut self, e: &Expr, min: u8, allow_bare_neg: bool) -> String {
        let r = self.expr(e);
        let is_neg = matches!(e, Expr::Un(UnOp::Neg, _)) || matches!(e, Expr::Num(s) if s.starts_with('-'));
        if r.prec < min || (!allow_bare_neg && is_neg) {
            format!("({})", r.s)
        } else {
            r.s
        }
    }

    fn expr_inner(&mut self, e: &Expr) -> R {
        match e {
            Expr::Num(s) => self.number(s),
            Expr::Str(s) => {
                // Shown for reading, a text is the text; it is only a gap where it is computed.
                if !self.readable && !self.display && !self.io_args {
                    self.issue(Code::StringValue, None);
                }
                if self.readable || self.display {
                    atom(format!("\"{}\"", s.chars().take(60).collect::<String>()))
                } else {
                    atom("?string?")
                }
            }
            Expr::Name(n) => self.name_ref(n),
            Expr::Un(op, x) => self.unary(*op, x),
            Expr::Bin(op, a, b) => self.binary(*op, a, b),
            Expr::Call(n, args) => self.call(n, args),
            Expr::Postfix(unit, v) => {
                let u = self.unit_only(unit);
                let val = self.operand(v, P_MUL, true);
                R {
                    s: format!("{}*{}", val, u),
                    prec: P_MUL,
                }
            }
            Expr::Integral {
                var,
                body,
                lower,
                upper,
            } => self.integral(var, body, lower, upper),
            Expr::Series {
                product,
                var,
                body,
                bounds,
            } => self.series(*product, var, body, bounds.as_ref()),
            Expr::Program(stmts) if self.readable => {
                let t = self.program_outline(stmts);
                atom(format!("?program {{ {t} }}?"))
            }
            Expr::Program(_) => {
                self.issue(
                    Code::ProgramBlock,
                    Some(
                        "a program inside an expression, or a program that is not the whole value of a definition"
                            .into(),
                    ),
                );
                atom("?program?")
            }
            Expr::Derivative { var, body } => self.derivative(var, body),
            Expr::Index(base, idx) => self.index(base, idx),
            Expr::Matrix { rows, cols, items } => self.matrix(*rows, *cols, items),
            Expr::Range { first, second, last } => {
                // Render for the placeholder, record the gap.
                let a = self.expr(first).s;
                let b = second.as_ref().map(|s| self.expr(s).s);
                let c = self.expr(last).s;
                self.issue(Code::RangeVariable, None);
                let s = match (b, self.readable) {
                    (Some(b), true) => format!("range({a}; {c}; ({b}) - ({a}))"),
                    (None, true) => format!("range({a}; {c}; 1)"),
                    (Some(b), false) => format!("{}, {} .. {}", a, b, c),
                    (None, false) => format!("{} .. {}", a, c),
                };
                atom(s)
            }
            Expr::Unsupported { code, detail } => {
                self.issue(*code, detail.clone());
                if self.readable {
                    // The element's own name is the best a reader gets (`imag` is the imaginary unit).
                    return match detail.as_deref() {
                        Some("imag") => atom("i"),
                        Some(d) => atom(format!("?{d}?")),
                        None => atom(format!("?{}?", code.as_str())),
                    };
                }
                atom(format!("?{}?", code.as_str()))
            }
        }
    }

    /// The text for something that cannot be converted: its real name or call when rendering a
    /// gap for the reader, else the `?stand-in?`.
    fn stand_in(&mut self, readable: &str, placeholder: &str) -> R {
        if self.readable {
            // A name Calcpad cannot spell cannot be shown as maths.
            let head = readable.split('(').next().unwrap_or(readable);
            if !valid_ident(head) {
                self.readable_bad = true;
            }
            atom(readable.to_string())
        } else {
            atom(placeholder.to_string())
        }
    }

    fn number(&mut self, s: &str) -> R {
        let ok = !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'))
            && s.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false);
        let text = if ok { expand_literal(s) } else { None };
        match text {
            None => {
                self.issue(Code::BadNumber, None);
                atom("?number?")
            }
            Some(t) => {
                let prec = if t.starts_with('-') { P_NEG } else { P_ATOM };
                R { s: t, prec }
            }
        }
    }

    /// True when the expression holds an array: a name known to be one, a matrix literal, or a
    /// transpose or sum of those.
    fn holds_array(&self, e: &Expr) -> bool {
        match e {
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                self.arrays.contains(&f) || self.local_arrays.contains(&f)
            }
            Expr::Matrix { .. } => true,
            Expr::Call(f, _) => matches!(
                f.base.as_str(),
                "identity" | "matrix" | "augment" | "stack" | "lsolve" | "submatrix"
            ),
            Expr::Un(_, a) => self.holds_array(a),
            Expr::Bin(_, a, b) => self.holds_array(a) || self.holds_array(b),
            _ => false,
        }
    }

    /// True when the value of `e` is an array: a matrix literal, a function that makes one, a
    /// known array, or arithmetic on one.
    fn yields_array(&self, e: &Expr) -> bool {
        match e {
            Expr::Matrix { .. } => true,
            // An `if` gives an array when one of its branches does.
            Expr::Call(f, args)
                if f.base == "if" && f.sub.is_none() && args.len() == 3 && !self.funcs.contains_key("if") =>
            {
                self.yields_array(&args[1]) || self.yields_array(&args[2])
            }
            Expr::Call(f, _) => {
                matches!(
                    f.base.as_str(),
                    "identity" | "matrix" | "augment" | "stack" | "lsolve" | "submatrix"
                ) || self.array_funcs.contains(&tables::normalise_name(&f.flat()))
            }
            Expr::Name(_) | Expr::Un(..) | Expr::Bin(..) => self.holds_array(e),
            _ => false,
        }
    }

    /// True when `vectorize(e)` is just `e` in the engine.
    fn elementwise_ok(&self, e: &Expr) -> bool {
        match e {
            Expr::Bin(BinOp::Add | BinOp::Sub, a, b) => self.elementwise_ok(a) && self.elementwise_ok(b),
            Expr::Bin(BinOp::Mul, a, b) => {
                self.elementwise_ok(a) && self.elementwise_ok(b) && !(self.holds_array(a) && self.holds_array(b))
            }
            Expr::Bin(BinOp::Div, a, b) => self.elementwise_ok(a) && self.elementwise_ok(b) && !self.holds_array(b),
            Expr::Un(UnOp::Transpose, a) => self.elementwise_ok(a),
            Expr::Name(_) | Expr::Num(_) | Expr::Str(_) | Expr::Matrix { .. } | Expr::Postfix(..) => true,
            other => !self.holds_array(other),
        }
    }

    fn unary(&mut self, op: UnOp, x: &Expr) -> R {
        match op {
            UnOp::Neg => {
                let inner = self.operand(x, P_POW, true);
                R {
                    s: format!("-{}", inner),
                    prec: P_NEG,
                }
            }
            UnOp::Abs => {
                if let Expr::Name(n) = x {
                    if self.arrays.contains(&n.flat()) {
                        self.issue(Code::AbsOfArray, None);
                    }
                }
                let inner = self.expr(x).s;
                atom(format!("abs({})", inner))
            }
            UnOp::Sqrt => {
                let inner = self.expr(x).s;
                atom(format!("sqrt({})", inner))
            }
            UnOp::Factorial => {
                let inner = self.operand(x, P_ATOM, true);
                atom(format!("{}!", inner))
            }
            UnOp::Transpose => {
                // The engine keeps vectors and matrices apart (`len`, `max`, indexing with one
                // index and `vector(..)` want a vector), so a transposed row literal is the
                // column vector itself and a double transpose is the operand.
                if let Expr::Un(UnOp::Transpose, y) = x {
                    return self.expr(y);
                }
                if self.display {
                    // The author's form: the operand with a superscript T (the engine has no transpose
                    // operator, so this is typeset only, never computed).
                    let inner = self.operand(x, P_ATOM, true);
                    return R {
                        s: format!("{inner}^(T)"),
                        prec: P_POW,
                    };
                }
                if let Expr::Matrix { rows: 1, cols, items } = x {
                    if *cols > 1 {
                        return self.matrix(*cols, 1, items);
                    }
                }
                let inner = self.expr(x).s;
                atom(format!("transp({})", inner))
            }
            UnOp::Vectorize => {
                // The engine's `+`, `-`, scalar `*` and `/` already work element by element; an
                // element-by-element product or quotient of two arrays has no operator.
                if !self.elementwise_ok(x) {
                    self.issue(
                        Code::VectorizeOperator,
                        Some("an element-by-element product, quotient, power or call of arrays".into()),
                    );
                    return self.stand_in("vectorize(...)", "?vectorize?");
                }
                self.expr(x)
            }
            UnOp::Not => {
                let _ = self.expr(x);
                self.issue(Code::UnknownElement, Some("not".into()));
                atom("?not?")
            }
        }
    }

    fn binary(&mut self, op: BinOp, a: &Expr, b: &Expr) -> R {
        // `x < y <= z` is `x < y` and `y <= z`, not a comparison of a truth value.
        let ordering = |o: BinOp| matches!(o, BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge);
        if ordering(op) {
            if let Expr::Bin(op2, x, y) = a {
                if ordering(*op2) {
                    let left = Expr::Bin(*op2, x.clone(), y.clone());
                    let right = Expr::Bin(op, y.clone(), Box::new(b.clone()));
                    return self.binary(BinOp::And, &left, &right);
                }
            }
        }
        let (sym, lmin, rmin, prec, rneg): (&str, u8, u8, u8, bool) = match op {
            BinOp::Add => (" + ", P_ADD, P_ADD, P_ADD, false),
            BinOp::Sub => (" - ", P_ADD, P_MUL, P_ADD, false),
            BinOp::Mul => ("*", P_MUL, P_MUL, P_MUL, false),
            BinOp::Div => ("/", P_MUL, P_NEG, P_MUL, false),
            BinOp::Pow => ("^", P_ATOM, P_ATOM, P_POW, false),
            BinOp::Eq => (" ≡ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Ne => (" ≠ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Lt => (" < ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Gt => (" > ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Le => (" ≤ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::Ge => (" ≥ ", P_ADD, P_ADD, P_CMP, false),
            BinOp::And => (" ∧ ", P_CMP, P_CMP, P_LOGIC, false),
            BinOp::Or => (" ∨ ", P_CMP, P_CMP, P_LOGIC, false),
            BinOp::Xor => (" ⊕ ", P_CMP, P_CMP, P_LOGIC, false),
        };
        let _ = rneg;
        if op == BinOp::Mul && !self.display && !self.readable && self.is_col_vec(a) && self.is_col_vec(b) {
            // Two column vectors multiply as a dot product in the source.
            let l = self.expr(a).s;
            let r = self.expr(b).s;
            return atom(format!("dot({l}; {r})"));
        }
        if matches!(op, BinOp::Eq | BinOp::Ne)
            && (self.is_stringy(a) || self.is_stringy(b))
            && !self.readable
            && !self.display
        {
            // Two texts are equal when their codes are.
            if let (Some(l), Some(r)) = (self.str_expr(a), self.str_expr(b)) {
                return R {
                    s: format!("{}{}{}", l.s, sym, r.s),
                    prec,
                };
            }
        }
        let mut l = self.operand(a, lmin, true);
        // A product's right operand that is itself a division keeps its parentheses.
        let rmin = if op == BinOp::Mul && matches!(b, Expr::Bin(BinOp::Div, ..)) {
            P_NEG
        } else {
            rmin
        };
        let mut r = self.operand(b, rmin, false);
        // A bare 0 added to or taken from a quantity is a zero of its dimension.
        if matches!(op, BinOp::Add | BinOp::Sub) && !self.display && !self.readable {
            let zero = |e: &Expr| matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
            let plain = |e: &Expr| matches!(e, Expr::Num(_));
            if zero(a) && !plain(b) {
                l = format!("0*getunits({r})");
            } else if zero(b) && !plain(a) {
                r = format!("0*getunits({l})");
            }
        }
        // A comparison with a bare 0 is a comparison with a zero of the other side's dimension
        // (as in the source); the engine wants both sides in the same units.
        if (self.plot_param.is_some() || !self.holds_array(a) && !self.holds_array(b))
            && matches!(
                op,
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
            )
            && !self.display
            && !self.readable
        {
            let zero = |e: &Expr| matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
            if zero(b) && !zero(a) {
                r = format!("0*getunits({})", l);
            } else if zero(a) && !zero(b) {
                l = format!("0*getunits({})", r);
            }
        }
        R {
            s: format!("{}{}{}", l, sym, r),
            prec,
        }
    }

    /// True when `e` is statically a column vector: a literal, a name or call known to give one, or
    /// a scalar times one.
    fn is_col_vec(&self, e: &Expr) -> bool {
        match e {
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                !self.locals.contains_key(&f) && !self.params.contains(&f) && self.col_vecs.contains(&f)
            }
            Expr::Call(n, _) => {
                self.col_vecs.contains(&tables::normalise_name(&n.flat()))
                    && self.funcs.contains_key(&tables::normalise_name(&n.flat()))
            }
            Expr::Bin(BinOp::Mul, a, b) => {
                (self.is_col_vec(a) && !self.holds_array(b) && !self.yields_array(b))
                    || (self.is_col_vec(b) && !self.holds_array(a) && !self.yields_array(a))
            }
            Expr::Bin(BinOp::Div, a, b) => self.is_col_vec(a) && !self.holds_array(b),
            other => is_col_literal(other),
        }
    }

    fn render_args(&mut self, args: &[Expr]) -> Vec<String> {
        args.iter().map(|a| self.expr(a).s).collect()
    }

    /// The `#format` line that puts the number format in force (`None`: the default), or nothing when it already is.
    /// Numbers written in an equation shown for reading are rounded by the format like any other, so a
    /// displayed equation is always shown under the default one.
    fn format_line(&mut self, want: Option<String>) -> Option<String> {
        if self.fmt_cur == want {
            return None;
        }
        self.fmt_cur = want.clone();
        Some(match want {
            Some(f) => format!("#format {f}"),
            None => "#format default".to_string(),
        })
    }

    /// The author's name for an engine name (the same name unless it was renamed to stay apart from a unit).
    fn author_name(&self, flat: &str) -> String {
        self.renamed.get(flat).cloned().unwrap_or_else(|| flat.to_string())
    }

    /// Runs `f` in display mode (the author's own equation, typeset and never computed) with the
    /// issue list, parameters and temporaries left as they were. Returns what `f` made, and whether
    /// the rendering is clean: nothing the engine could not convert, nothing hoisted.
    fn displaying<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> (T, bool) {
        let saved_cur = std::mem::take(&mut self.cur);
        let saved_params = std::mem::take(&mut self.params);
        let saved_hoisted = std::mem::take(&mut self.hoisted);
        self.display = true;
        self.display_renamed = false;
        let out = f(self);
        let clean = self.cur.iter().all(|p| p.code.severity() != Severity::Gap) && self.hoisted.is_empty();
        self.display = false;
        self.params = saved_params;
        self.hoisted = saved_hoisted;
        self.cur = saved_cur;
        (out, clean)
    }

    /// `name = value` as the author wrote the definition, for a `#noc` block; `None` when it cannot
    /// be shown faithfully (then the computed form is shown as before).
    fn display_define(&mut self, target: &Target, value: &Expr) -> Option<String> {
        let (text, clean) = self.displaying(|t| {
            let head = match target {
                Target::Var(n) => t.author_name(&tables::normalise_name(&n.flat())),
                Target::Func(n, ps) => {
                    let plist: Vec<String> = ps.iter().map(|p| tables::normalise_name(&p.flat())).collect();
                    t.params = plist.clone();
                    let shown: Vec<String> = plist.iter().map(|p| t.author_name(p)).collect();
                    format!(
                        "{}({})",
                        t.author_name(&tables::normalise_name(&n.flat())),
                        shown.join("; ")
                    )
                }
                _ => return None,
            };
            Some(format!("{head} = {}", t.expr(value).s))
        });
        text.filter(|t| clean && plausible_calcpad(t))
    }

    /// `name.(index) = value` for the definition of one element or of every element over range variables.
    fn display_element(&mut self, base: &Name, index: &[Expr], value: &Expr) -> Option<String> {
        let (text, clean) = self.displaying(|t| {
            let idx: Vec<String> = index.iter().map(|i| t.expr(i).s).collect();
            let head = format!(
                "{}.({})",
                t.author_name(&tables::normalise_name(&base.flat())),
                idx.join("; ")
            );
            format!("{head} = {}", t.expr(value).s)
        });
        Some(text).filter(|t| clean && plausible_calcpad(t))
    }

    /// An evaluation's expression as the author wrote it, for a `#noc` block, and whether any name in
    /// it differs from the engine's.
    fn display_eval(&mut self, expr: &Expr, unit: Option<&Expr>) -> Option<(String, bool)> {
        let ((text, renamed), clean) = self.displaying(|t| {
            let e = t.expr(expr);
            let mut s = e.s.clone();
            if let Some(u) = unit {
                let inner = if e.prec >= P_ATOM { e.s } else { format!("({})", e.s) };
                let us = t.expr(u).s;
                s = if is_degree(u) {
                    format!("({inner}/(π/180))*deg")
                } else if numeric_only(u) {
                    format!("{inner}/({us})")
                } else {
                    format!("{inner}|{us}")
                };
            }
            (s, t.display_renamed)
        });
        (clean && plausible_calcpad(&text)).then_some((text, renamed))
    }

    /// The statement of an unconverted region as the reader should see it: Calcpad-style maths
    /// with the real names. Nothing here is recorded in the report.
    fn readable_stmt(&mut self, stmt: &Stmt) -> Option<Vec<GapLine>> {
        let saved_cur = std::mem::take(&mut self.cur);
        let saved_params = std::mem::take(&mut self.params);
        self.readable = true;
        self.readable_bad = false;
        let as_line = |t: String, bad: bool| -> GapLine { (!bad && plausible_calcpad(&t), t) };
        let lines: Option<Vec<GapLine>> = match stmt {
            Stmt::Define { target, value, .. } => {
                let names: Vec<String> = match target {
                    Target::Var(n) | Target::Func(n, _) => vec![tables::normalise_name(&n.flat())],
                    Target::Element { base, .. } => base.iter().map(|b| tables::normalise_name(&b.flat())).collect(),
                    Target::Multi(ns) => ns.iter().map(|n| tables::normalise_name(&n.flat())).collect(),
                    Target::Slice { base, .. } => vec![tables::normalise_name(&base.flat())],
                };
                // An element assignment is only valid for an array the engine holds; a function that
                // mentions itself is a circular reference to it.
                let head_bad = names.iter().any(|n| !valid_ident(n))
                    || matches!(target, Target::Element { .. } | Target::Slice { .. });
                let head = match target {
                    Target::Var(n) => tables::normalise_name(&n.flat()),
                    Target::Func(n, ps) => {
                        let plist: Vec<String> = ps.iter().map(|p| tables::normalise_name(&p.flat())).collect();
                        self.params = plist.clone();
                        format!("{}({})", tables::normalise_name(&n.flat()), plist.join("; "))
                    }
                    Target::Element { base, index } => {
                        let b = base
                            .as_ref()
                            .map(|b| tables::normalise_name(&b.flat()))
                            .unwrap_or_default();
                        let idx: Vec<String> = index.iter().map(|i| self.expr(i).s).collect();
                        format!("{}.({})", b, idx.join("; "))
                    }
                    Target::Multi(names) => {
                        let l: Vec<String> = names.iter().map(|n| tables::normalise_name(&n.flat())).collect();
                        format!("[{}]", l.join("; "))
                    }
                    Target::Slice { base, index, row } => {
                        let i = self.expr(index).s;
                        format!(
                            "{}({}; {})",
                            if *row { "row" } else { "col" },
                            tables::normalise_name(&base.flat()),
                            i
                        )
                    }
                };
                if let Expr::Program(stmts) = value {
                    let mut out = vec![(false, format!("{head} = program:"))];
                    self.program_lines(stmts, 1, &mut out);
                    Some(out)
                } else {
                    let rhs = self.expr(value).s;
                    let own_name = names.first().map(|n| format!("{n}(")).unwrap_or_default();
                    let unknown_fn = matches!(target, Target::Func(..))
                        && self.cur.iter().any(|p| {
                            matches!(
                                p.code,
                                Code::AddinFunction
                                    | Code::ReferenceFunction
                                    | Code::BuiltinMissing
                                    | Code::SolveBlock
                                    | Code::DependsOnGap
                                    | Code::UndefinedVariable
                            )
                        });
                    let recursive = unknown_fn || (matches!(target, Target::Func(..)) && rhs.contains(&own_name));
                    let bad = self.readable_bad || head_bad || recursive;
                    Some(vec![as_line(format!("{head} = {rhs}"), bad)])
                }
            }
            Stmt::Eval { expr, unit } => {
                let mut t = self.expr(expr).s;
                if let Some(u) = unit {
                    let us = self.expr(u).s;
                    t = if is_degree(u) {
                        format!("(({t})/(π/180))*deg")
                    } else if numeric_only(u) {
                        format!("({t})/({us})")
                    } else {
                        format!("{t}|{us}")
                    };
                }
                let bad = self.readable_bad;
                Some(vec![as_line(t, bad)])
            }
            Stmt::Bare(e) => {
                let mut t = self.expr(e).s;
                let mut bad = self.readable_bad;
                if matches!(e, Expr::Bin(BinOp::Eq, ..)) {
                    t = t.replace(" ≡ ", " = ");
                    // `f(x) = ...` on its own would be read as a definition of f.
                    if let Some((lhs, _)) = t.split_once(" = ") {
                        bad |= lhs.contains('(');
                    }
                }
                Some(vec![as_line(t, bad)])
            }
            Stmt::Unsupported { .. } => None,
        };
        self.readable = false;
        self.readable_bad = false;
        self.params = saved_params;
        self.cur = saved_cur;
        lines
    }

    /// The statements of an unconverted program as lines for the reader: assignments and the final
    /// value are equations, `for`, `while` and `if` are indented text.
    fn program_lines(&mut self, stmts: &[PStmt], depth: usize, out: &mut Vec<GapLine>) {
        let pad = "&emsp;".repeat(depth);
        for st in stmts {
            match st {
                PStmt::Local(PTarget::Var(n), e) => {
                    self.readable_bad = false;
                    let name = tables::normalise_name(&n.flat());
                    let t = format!("{} = {}", name, self.expr(e).s);
                    out.push((!self.readable_bad && valid_ident(&name) && plausible_calcpad(&t), t));
                }
                PStmt::Local(PTarget::Slice { base, index, row }, e) => {
                    let t = format!(
                        "{}({}; {}) = {}",
                        if *row { "row" } else { "col" },
                        tables::normalise_name(&base.flat()),
                        self.expr(index).s,
                        self.expr(e).s
                    );
                    out.push((false, t));
                }
                PStmt::Local(PTarget::Element { base, index }, e) => {
                    let idx: Vec<String> = index.iter().map(|i| self.expr(i).s).collect();
                    let t = format!(
                        "{}.({}) = {}",
                        tables::normalise_name(&base.flat()),
                        idx.join("; "),
                        self.expr(e).s
                    );
                    out.push((false, t));
                }
                PStmt::For { var, range, body } => {
                    let v = tables::normalise_name(&var.flat());
                    let r = match range {
                        Expr::Range {
                            first,
                            second: None,
                            last,
                        } => {
                            format!("{} to {}", self.expr(first).s, self.expr(last).s)
                        }
                        other => self.expr(other).s,
                    };
                    out.push((false, format!("{pad}for {v} = {r}:")));
                    self.params.push(v);
                    self.program_lines(body, depth + 1, out);
                    self.params.pop();
                }
                PStmt::While { cond, body } => {
                    let c = self.expr(cond).s;
                    out.push((false, format!("{pad}while {c}:")));
                    self.program_lines(body, depth + 1, out);
                }
                PStmt::If { branches, otherwise } => {
                    for (i, (c, b)) in branches.iter().enumerate() {
                        let cs = self.expr(c).s;
                        out.push((false, format!("{pad}{} {cs}:", if i == 0 { "if" } else { "else if" })));
                        self.program_lines(b, depth + 1, out);
                    }
                    if let Some(o) = otherwise {
                        out.push((false, format!("{pad}else:")));
                        self.program_lines(o, depth + 1, out);
                    }
                }
                PStmt::Return(e) => {
                    let t = self.expr(e).s;
                    out.push((false, format!("{pad}return {t}")));
                }
                PStmt::Break => out.push((false, format!("{pad}break"))),
                PStmt::Continue => out.push((false, format!("{pad}continue"))),
                PStmt::Expr(e) => {
                    self.readable_bad = false;
                    let t = self.expr(e).s;
                    out.push((!self.readable_bad && plausible_calcpad(&t), t));
                }
                PStmt::Unsupported(w) => out.push((false, format!("{pad}{w}"))),
            }
        }
    }

    /// A compact one-line outline of a program, for the reader of an unconverted region.
    fn program_outline(&mut self, stmts: &[PStmt]) -> String {
        let mut parts = Vec::new();
        for st in stmts {
            parts.push(match st {
                PStmt::Local(PTarget::Var(n), e) => {
                    format!("{} = {}", tables::normalise_name(&n.flat()), self.expr(e).s)
                }
                PStmt::Local(PTarget::Slice { base, index, row }, e) => {
                    format!(
                        "{}({}; {}) = {}",
                        if *row { "row" } else { "col" },
                        tables::normalise_name(&base.flat()),
                        self.expr(index).s,
                        self.expr(e).s
                    )
                }
                PStmt::Local(PTarget::Element { base, index }, e) => {
                    let idx: Vec<String> = index.iter().map(|i| self.expr(i).s).collect();
                    format!(
                        "{}.({}) = {}",
                        tables::normalise_name(&base.flat()),
                        idx.join("; "),
                        self.expr(e).s
                    )
                }
                PStmt::For { var, range, body } => {
                    let v = tables::normalise_name(&var.flat());
                    let r = self.expr(range).s;
                    self.params.push(v.clone());
                    let b = self.program_outline(body);
                    self.params.pop();
                    format!("for {v} in {r} {{ {b} }}")
                }
                PStmt::While { cond, body } => {
                    let c = self.expr(cond).s;
                    format!("while {c} {{ {} }}", self.program_outline(body))
                }
                PStmt::If { branches, otherwise } => {
                    let mut t = String::new();
                    for (i, (c, b)) in branches.iter().enumerate() {
                        let cs = self.expr(c).s;
                        t.push_str(&format!(
                            "{}{} {{ {} }}",
                            if i == 0 { "if " } else { " else if " },
                            cs,
                            self.program_outline(b)
                        ));
                    }
                    if let Some(o) = otherwise {
                        t.push_str(&format!(" else {{ {} }}", self.program_outline(o)));
                    }
                    t
                }
                PStmt::Return(e) => format!("return {}", self.expr(e).s),
                PStmt::Break => "break".to_string(),
                PStmt::Continue => "continue".to_string(),
                PStmt::Expr(e) => self.expr(e).s,
                PStmt::Unsupported(w) => w.clone(),
            });
        }
        parts.join("; ")
    }

    /// The loops that fill a new array `f` from range variables, preceded by its allocation.
    /// `idx` are the shifted index texts in the order of `rs`. A range with literal ascending
    /// bounds is a plain `#for`; any other counts steps and derives the value, so it runs down
    /// when the end is below the start as the source does. The array is as large as the largest
    /// index any range reaches, and an element no iteration writes stays 0 (the source would
    /// have no such element, or fill it with 0 below the first index written).
    fn fill_loops(
        &mut self,
        f: &str,
        rs: &[(String, RangeInfo)],
        idx: &[String],
        rhs: &str,
        pre: &[String],
    ) -> Result<String, String> {
        let (head, loops, sizes, steps) = self.range_loops(rs)?;
        // A hidden statement that fails leaves no trace of its own (in a loop it can leave the zeros
        // the array started with), so the loop writes a working array and counts the passes whose
        // value was computed; the real array is defined from it only if every pass was counted,
        // and otherwise the division fails, so the name is undefined where it is used.
        let (w, c) = (format!("{f}_w{}", self.prog_n), format!("{f}_c{}", self.prog_n));
        let alloc = if sizes.len() == 1 {
            format!("{w} = vector({})", sizes[0])
        } else {
            format!("{w} = matrix({}; {})", sizes[0], sizes[1])
        };
        let mut out = vec!["#hide".to_string()];
        out.extend(head);
        out.push(alloc);
        out.push(format!("{c} = 0"));
        for (d, l) in loops.iter().enumerate() {
            for line in l.split('\n') {
                out.push(format!("{}{}", "    ".repeat(d), line));
            }
        }
        let ind = "    ".repeat(loops.len());
        for p in pre {
            out.push(format!("{ind}{p}"));
        }
        out.push(format!("{ind}{w}.({}) = {rhs}", idx.join("; ")));
        // Multiplying by sign() keeps the count a plain number and makes it depend on the value written.
        out.push(format!("{ind}{c} = {c} + 1 + 0*sign({rhs})"));
        for d in (0..loops.len()).rev() {
            out.push(format!("{}#loop", "    ".repeat(d)));
        }
        out.push(format!("{f} = {w}*(1/({c} ≡ {}))", steps.join("*")));
        out.push("#end hide".to_string());
        Ok(out.join("\n"))
    }

    /// The loop headers for a set of range variables: lines before the loops (bounds taken once),
    /// one header per range (a plain `#for` for literal ascending bounds, else a step counter that
    /// derives the value so it runs down when the end is below the start), and the engine size each
    /// range reaches.
    fn range_loops(&mut self, rs: &[(String, RangeInfo)]) -> Result<RangeLoops, String> {
        let o = self.origin;
        let mut head = Vec::new();
        let mut loops = Vec::new();
        let mut sizes = Vec::new();
        let mut steps = Vec::new();
        self.prog_n += 1;
        for (iv, info) in rs {
            let (a, z) = (&info.a, &info.z);
            if let Some(st) = &info.step {
                // A step: the passes are floor((z - a) / step) + 1 (a little over, so that 0.1 steps
                // do not lose the last one), the variable a + k * step.
                let (lo, hi, sp, k) = (
                    format!("{iv}_a{}", self.prog_n),
                    format!("{iv}_z{}", self.prog_n),
                    format!("{iv}_s{}", self.prog_n),
                    format!("{iv}_k{}", self.prog_n),
                );
                head.push(format!("{lo} = {a}"));
                head.push(format!("{hi} = {z}"));
                head.push(format!("{sp} = {st}"));
                let count = format!("floor(({hi} - {lo})/{sp} + 0.000000001)");
                let last = format!("({lo} + {count}*{sp})");
                sizes.push(match o {
                    1 => format!("max({lo}; {last})"),
                    o if o > 1 => format!("max({lo}; {last}) - {}", o - 1),
                    o => format!("max({lo}; {last}) + {}", 1 - o),
                });
                steps.push(format!("({count} + 1)"));
                loops.push(format!("#for {k} = 0 : {count}\n{iv} = {lo} + {k}*{sp}"));
                continue;
            }
            match info.lit {
                Some((x, y)) => {
                    if x < o {
                        return Err("the range starts below the array origin".into());
                    }
                    sizes.push((y - o + 1).to_string());
                    steps.push((y - x + 1).to_string());
                    loops.push(format!("#for {iv} = {x} : {y}"));
                }
                None => {
                    let (lo, hi, k) = (
                        format!("{iv}_a{}", self.prog_n),
                        format!("{iv}_z{}", self.prog_n),
                        format!("{iv}_k{}", self.prog_n),
                    );
                    head.push(format!("{lo} = {a}"));
                    head.push(format!("{hi} = {z}"));
                    let size = match o {
                        1 => format!("max({lo}; {hi})"),
                        o if o > 1 => format!("max({lo}; {hi}) - {}", o - 1),
                        o => format!("max({lo}; {hi}) + {}", 1 - o),
                    };
                    sizes.push(size);
                    steps.push(format!("(abs({hi} - {lo}) + 1)"));
                    loops.push(format!(
                        "#for {k} = 0 : abs({hi} - {lo})\n{iv} = {lo} + {k}*if({hi} ≥ {lo}; 1; -1)"
                    ));
                }
            }
        }
        Ok((head, loops, sizes, steps))
    }

    /// The statements that write one element of an array that grows: before a write past the end,
    /// `resize` (or `mresize`) extends it with zeros, as the source does.
    fn growth_lines(mapped: &str, parts: &[String], pad: &str) -> Vec<String> {
        let mut out = Vec::new();
        if parts.len() == 1 {
            let i = &parts[0];
            out.push(format!("{pad}#if ({i}) > len({mapped})"));
            out.push(format!("{pad}    {mapped} = resize({mapped}; {i})"));
        } else {
            let (r, c) = (&parts[0], &parts[1]);
            out.push(format!(
                "{pad}#if max(({r}) - n_rows({mapped}); ({c}) - n_cols({mapped})) > 0"
            ));
            out.push(format!(
                "{pad}    {mapped} = mresize({mapped}; max({r}; n_rows({mapped})); max({c}; n_cols({mapped})))"
            ));
        }
        out.push(format!("{pad}#end if"));
        out
    }

    /// Whether an element assignment whose index is a computed expression (or whose array already
    /// has a value) can be written as loops over the range variables it reads: the ranges in the
    /// order they are first read, and whether the array is new. The array must be new, or hold an
    /// array; the value and the index must not read it.
    fn grow_fill_plan(
        &self,
        flat: &Option<String>,
        index: &[Expr],
        value: &Expr,
    ) -> Option<(Vec<(String, RangeInfo)>, bool)> {
        self.fill_plan(flat, index, value, false)
    }

    /// `grow_fill_plan`, or (`nested`) the plan for an array that holds an array in each element.
    fn fill_plan(
        &self,
        flat: &Option<String>,
        index: &[Expr],
        value: &Expr,
        nested: bool,
    ) -> Option<(Vec<(String, RangeInfo)>, bool)> {
        let f = flat.as_ref()?;
        if index.is_empty() || index.len() > 2 {
            return None;
        }
        // One element of a vector is a number: an array value is a matrix built row by row.
        if self.yields_array(value) != nested {
            return None;
        }
        let known = self.defined.contains(f) || self.funcs.contains_key(f) || self.gap_names.contains_key(f);
        let fresh = !known;
        // A new array has nothing to read yet (a read would silently give the zeros it starts with);
        // one that has a value may be read by its own update, in the order of the loop, as the
        // source reads it element by element.
        if fresh && (mentions(value, f) || index.iter().any(|e| mentions(e, f))) {
            return None;
        }
        let array = self.arrays.contains(f) && !self.funcs.contains_key(f) && !self.gap_names.contains_key(f);
        if !fresh && !array {
            return None;
        }
        let mut names: Vec<String> = Vec::new();
        for e in index.iter().chain(std::iter::once(value)) {
            let mut c = e.clone();
            c.for_each_name_mut(&mut |n| names.push(tables::normalise_name(&n.flat())));
        }
        let mut rs: Vec<(String, RangeInfo)> = Vec::new();
        for n in names {
            if rs.iter().any(|(m, _)| *m == n) || self.params.contains(&n) || self.locals.contains_key(&n) {
                continue;
            }
            if let Some(info) = self.ranges.get(&n) {
                rs.push((n, info.clone()));
            }
        }
        (!rs.is_empty() && rs.len() <= 3).then_some((rs, fresh))
    }

    /// Loops over the range variables an element assignment reads, writing the element at the
    /// computed index and growing the array when the index is past its end. A new array starts
    /// with one zero; one that has a value is written into.
    fn grow_fill_loops(
        &mut self,
        f: &str,
        rs: &[(String, RangeInfo)],
        idx: &[String],
        rhs: &str,
        pre: &[String],
        fresh: bool,
    ) -> Result<String, String> {
        let (head, loops, _, _) = self.range_loops(rs)?;
        let mut out = vec!["#hide".to_string()];
        out.extend(head);
        if fresh {
            out.push(if idx.len() == 1 {
                format!("{f} = vector(1)")
            } else {
                format!("{f} = matrix(1; 1)")
            });
        } else {
            // Writing into an array another name may share would change that name too.
            out.push(format!("{f} = {f}*1"));
        }
        for (d, l) in loops.iter().enumerate() {
            for line in l.split('\n') {
                out.push(format!("{}{}", "    ".repeat(d), line));
            }
        }
        let pad = "    ".repeat(loops.len());
        for p in pre {
            out.push(format!("{pad}{p}"));
        }
        out.extend(Self::growth_lines(f, idx, &pad));
        out.push(format!("{pad}{f}.({}) = {rhs}", idx.join("; ")));
        for d in (0..loops.len()).rev() {
            out.push(format!("{}#loop", "    ".repeat(d)));
        }
        out.push("#end hide".to_string());
        Ok(out.join("\n"))
    }

    /// Whether an expression reads any range variable of the sheet.
    fn reads_range(&self, e: &Expr) -> bool {
        self.ranges
            .keys()
            .any(|k| !self.params.contains(k) && !self.locals.contains_key(k) && mentions(e, k))
    }

    /// The one range variable an expression reads (not as a function argument or a bound name),
    /// when it reads exactly one.
    fn table_range(&self, e: &Expr) -> Option<(String, RangeInfo)> {
        let mut hit: Vec<(String, RangeInfo)> = self
            .ranges
            .iter()
            .filter(|(k, _)| !self.params.contains(k) && !self.locals.contains_key(*k) && mentions(e, k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        (hit.len() == 1).then(|| hit.remove(0))
    }

    /// Loops over a range variable, in the range's own order, collecting `rhs` into a new array
    /// that is then shown (`tail` is a `|unit` conversion).
    fn table_loops(&mut self, iv: &str, info: &RangeInfo, rhs: &str, pre: &[String], tail: &str) -> String {
        self.prog_n += 1;
        let n = self.prog_n;
        let (a, z) = (&info.a, &info.z);
        let (lo, hi, k, tbl) = (
            format!("{iv}_a{n}"),
            format!("{iv}_z{n}"),
            format!("{iv}_k{n}"),
            format!("tbl_{n}"),
        );
        let mut out = vec!["#hide".to_string(), format!("{lo} = {a}"), format!("{hi} = {z}")];
        match &info.step {
            Some(st) => {
                let sp = format!("{iv}_s{n}");
                let count = format!("floor(({hi} - {lo})/{sp} + 0.000000001)");
                out.push(format!("{sp} = {st}"));
                out.push(format!("{tbl} = vector({count} + 1)"));
                out.push(format!("#for {k} = 0 : {count}"));
                out.push(format!("{iv} = {lo} + {k}*{sp}"));
            }
            None => {
                out.push(format!("{tbl} = vector(abs({hi} - {lo}) + 1)"));
                out.push(format!("#for {k} = 0 : abs({hi} - {lo})"));
                out.push(format!("{iv} = {lo} + {k}*if({hi} ≥ {lo}; 1; -1)"));
            }
        }
        out.extend(pre.iter().map(|p| format!("    {p}")));
        out.push(format!("    {tbl}.({k} + 1) = {rhs}"));
        out.push("#loop".to_string());
        out.push("#end hide".to_string());
        out.push(format!("{tbl}{tail}"));
        out.join("\n")
    }

    /// A global definition (`x == 5`) holds from the top of the worksheet, wherever it is written.
    /// Those that are plain constants (numbers and units) are defined first, out of sight; the
    /// definition itself still appears, and is converted, where it stands.
    fn hoist_globals(&mut self, rows: &[Vec<Region>]) {
        fn constant(e: &Expr) -> bool {
            match e {
                Expr::Num(_) => true,
                Expr::Un(UnOp::Neg, a) => constant(a),
                Expr::Bin(op, a, b) => {
                    matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow)
                        && constant(a)
                        && constant(b)
                }
                Expr::Name(n) => n.sub.is_none() && tables::map_unit(&n.base).is_some(),
                Expr::Postfix(_, a) => constant(a),
                _ => false,
            }
        }
        let mut lines = Vec::new();
        for r in rows.iter().flatten() {
            let RegionKind::Math(m) = &r.kind else { continue };
            let Stmt::Define {
                target: Target::Var(n),
                value,
                global: true,
            } = &m.stmt
            else {
                continue;
            };
            let flat = tables::normalise_name(&n.flat());
            if m.disabled || !valid_ident(&flat) || flat == "ORIGIN" || !constant(value) {
                continue;
            }
            self.cur.clear();
            let rhs = self.expr(value);
            if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                lines.push(format!("{flat} = {}", rhs.s));
                self.defined.insert(flat);
            }
            self.cur.clear();
        }
        if !lines.is_empty() {
            self.lines.push("#hide".to_string());
            self.lines.extend(lines);
            self.lines.push("#show".to_string());
        }
    }

    /// Translates the regions row by row, in reading order. Regions that share a row are put side
    /// by side: a run of rows that each hold the same number of text regions (a legend of symbol and
    /// description) becomes a table, any other row with two or more regions that wrote something
    /// becomes a wrapping flex row with one block per region. The order the regions are translated
    /// in, and so the order the engine evaluates them in, is the reading order either way.
    fn layout_rows(&mut self, rows: &[Vec<Region>]) {
        self.hoist_globals(rows);
        struct Cell {
            text: bool,
            lines: Vec<String>,
        }
        impl Cell {
            /// Something shows: a cell of comments only does not take part in the layout.
            fn visible(&self) -> bool {
                self.lines.iter().any(|l| !l.starts_with("'<!--"))
            }
        }
        // A picture with text to its right that spans the picture's height is one block: the picture
        // on the left and those texts stacked beside it, whatever baselines the texts sit on.
        let mut beside: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
        let mut taken: HashSet<(usize, usize)> = HashSet::new();
        for (ri, row) in rows.iter().enumerate() {
            for (ci, p) in row.iter().enumerate() {
                if !matches!(p.kind, RegionKind::Picture(_))
                    || p.width <= 0.0
                    || p.height <= 0.0
                    || taken.contains(&(ri, ci))
                {
                    continue;
                }
                let bottom = p.top + p.height;
                let mut stack = Vec::new();
                for (rj, other) in rows.iter().enumerate().skip(ri) {
                    if other
                        .iter()
                        .all(|q| q.baseline.unwrap_or(q.top) > bottom + ROW_TOLERANCE)
                    {
                        break;
                    }
                    for (cj, q) in other.iter().enumerate() {
                        if (rj, cj) != (ri, ci)
                            && matches!(q.kind, RegionKind::Text(_))
                            && q.left >= p.left + p.width - 1.0
                            && q.top < bottom
                            && q.top + q.height > p.top
                            && !taken.contains(&(rj, cj))
                        {
                            stack.push((rj, cj));
                        }
                    }
                }
                if !stack.is_empty() {
                    taken.extend(stack.iter().copied());
                    beside.insert((ri, ci), stack);
                }
            }
        }
        let mut done: Vec<Vec<Cell>> = Vec::new();
        for row in rows {
            let mut cells = Vec::new();
            for r in row {
                let start = self.lines.len();
                self.region(r);
                let lines = self.lines.split_off(start);
                cells.push(Cell {
                    text: matches!(r.kind, RegionKind::Text(_)),
                    lines,
                });
            }
            done.push(cells);
        }
        for (&(ri, ci), stack) in &beside {
            let mut block = vec![
                "'<div class=\"imported-side\" style=\"display:flex;gap:1em;align-items:flex-start\">".to_string(),
                "'<div style=\"flex:none\">".to_string(),
            ];
            block.extend(done[ri][ci].lines.iter().cloned());
            block.push("'</div>".to_string());
            block.push("'<div>".to_string());
            for &(rj, cj) in stack {
                block.extend(std::mem::take(&mut done[rj][cj].lines));
            }
            block.push("'</div>".to_string());
            block.push("'</div>".to_string());
            done[ri][ci].lines = block;
        }
        // The axis titles of a plot are text regions beside it in the source (the y title to its left,
        // the x title just under it). Gathered with the plot they stay its labels, not loose lines.
        for (ri, row) in rows.iter().enumerate() {
            for (ci, p) in row.iter().enumerate() {
                if !matches!(p.kind, RegionKind::Plot(_))
                    || p.width <= 0.0
                    || p.height <= 0.0
                    || taken.contains(&(ri, ci))
                {
                    continue;
                }
                let bottom = p.top + p.height;
                let (mut ylab, mut xlab) = (Vec::new(), Vec::new());
                for (rj, other) in rows.iter().enumerate().skip(ri) {
                    for (cj, q) in other.iter().enumerate() {
                        if (rj, cj) == (ri, ci) || !matches!(q.kind, RegionKind::Text(_)) || taken.contains(&(rj, cj)) {
                            continue;
                        }
                        let right = q.left + q.width;
                        if right <= p.left + 1.0
                            && q.left >= p.left - 70.0
                            && q.top >= p.top - 6.0
                            && q.top + q.height <= bottom + 6.0
                        {
                            ylab.push((rj, cj));
                        } else if q.top >= bottom - 6.0
                            && q.top <= bottom + 24.0
                            && q.left >= p.left - 6.0
                            && right <= p.left + p.width + 6.0
                        {
                            xlab.push((rj, cj));
                        }
                    }
                }
                if ylab.is_empty() && xlab.is_empty() {
                    continue;
                }
                taken.extend(ylab.iter().chain(xlab.iter()).copied());
                let mut block = vec![
                    "'<div class=\"imported-plot\" style=\"display:inline-grid;grid-template-columns:auto auto;gap:.2em .6em;align-items:center\">".to_string(),
                    "'<div>".to_string(),
                ];
                for &(rj, cj) in &ylab {
                    block.extend(std::mem::take(&mut done[rj][cj].lines));
                }
                block.push("'</div>".to_string());
                block.push("'<div>".to_string());
                block.extend(std::mem::take(&mut done[ri][ci].lines));
                block.push("'</div>".to_string());
                block.push("'<div></div>".to_string());
                block.push("'<div style=\"text-align:center\">".to_string());
                for &(rj, cj) in &xlab {
                    block.extend(std::mem::take(&mut done[rj][cj].lines));
                }
                block.push("'</div>".to_string());
                block.push("'</div>".to_string());
                done[ri][ci].lines = block;
            }
        }
        // A row of equations (a table's columns) with a caption over each: the captions go with their
        // equations, so they stay column headings instead of lines above the whole row.
        let mut captioned: HashSet<usize> = HashSet::new();
        for (ri, row) in rows.iter().enumerate() {
            let mut caps: Vec<(usize, (usize, usize))> = Vec::new();
            for (ci, m) in row.iter().enumerate() {
                if !matches!(m.kind, RegionKind::Math(_)) || m.width <= 0.0 {
                    continue;
                }
                let mut best: Option<(f64, (usize, usize))> = None;
                for (rj, other) in rows.iter().enumerate().take(ri).skip(ri.saturating_sub(8)) {
                    for (cj, q) in other.iter().enumerate() {
                        let gap = m.top - (q.top + q.height);
                        if matches!(q.kind, RegionKind::Text(_))
                            && !taken.contains(&(rj, cj))
                            && (-3.0..=40.0).contains(&gap)
                            && q.left < m.left + m.width
                            && q.left + q.width > m.left - 20.0
                            && best.map_or(true, |(g, _)| gap < g)
                        {
                            best = Some((gap, (rj, cj)));
                        }
                    }
                }
                if let Some((_, at)) = best {
                    if !caps.iter().any(|(_, t)| *t == at) {
                        caps.push((ci, at));
                    }
                }
            }
            if caps.len() < 2 {
                continue;
            }
            captioned.insert(ri);
            for (ci, (rj, cj)) in caps {
                taken.insert((rj, cj));
                let mut lines = std::mem::take(&mut done[rj][cj].lines);
                lines.extend(std::mem::take(&mut done[ri][ci].lines));
                done[ri][ci].lines = lines;
            }
        }
        // Side by side only when the regions of the row do not overlap across (a region nested in a
        // text region sits inside it, not beside it).
        let apart: Vec<bool> = rows
            .iter()
            .map(|row| {
                row.windows(2)
                    .all(|w| w[0].width > 0.0 && w[1].left >= w[0].left + w[0].width - 1.0)
            })
            .collect();
        let page_left = rows.iter().flatten().map(|r| r.left).fold(f64::INFINITY, f64::min);
        let page_left = if page_left.is_finite() { page_left } else { 0.0 };
        let shown = |row: &Vec<Cell>| row.iter().filter(|c| c.visible()).count();
        let text_pair = |k: usize| {
            let row = &done[k];
            let n = shown(row);
            apart[k] && (2..=4).contains(&n) && row.iter().filter(|c| c.visible()).all(|c| c.text) && n == row.len()
        };
        let mut i = 0;
        while i < done.len() {
            if text_pair(i) {
                let n = done[i].len();
                let mut j = i;
                while j < done.len() && text_pair(j) && done[j].len() == n {
                    j += 1;
                }
                self.lines.push("'<table class=\"imported-grid\">".to_string());
                for row in &done[i..j] {
                    self.lines.push("'<tr>".to_string());
                    for c in row {
                        self.lines.push("'<td>".to_string());
                        self.lines.extend(c.lines.iter().cloned());
                        self.lines.push("'</td>".to_string());
                    }
                    self.lines.push("'</tr>".to_string());
                }
                self.lines.push("'</table>".to_string());
                i = j;
            } else if apart[i] && shown(&done[i]) >= 2 {
                for c in done[i].iter().filter(|c| !c.visible()) {
                    self.lines.extend(c.lines.iter().cloned());
                }
                // Columns with captions line up at the top, so each caption sits over its own equation.
                let align = if captioned.contains(&i) {
                    "flex-start"
                } else {
                    "baseline;align-items:last baseline"
                };
                self.lines.push(format!(
                    "'<div class=\"imported-row\" style=\"display:flex;flex-wrap:wrap;gap:0 2em;align-items:{align}\">"
                ));
                // Each block sits where the source puts it: the gap to the block before it, when the
                // file gives widths.
                let mut prev_right: Option<f64> = None;
                for (c, r) in done[i].iter().zip(rows[i].iter()).filter(|(c, _)| c.visible()) {
                    let style = match prev_right {
                        Some(pr) if r.left - pr > 16.0 => {
                            format!(" style=\"margin-left:{:.0}pt\"", (r.left - pr - 24.0).clamp(0.0, 600.0))
                        }
                        _ => String::new(),
                    };
                    prev_right = (r.width > 0.0).then_some(r.left + r.width);
                    self.lines.push(format!("'<div{style}>"));
                    self.lines.extend(c.lines.iter().cloned());
                    self.lines.push("'</div>".to_string());
                }
                self.lines.push("'</div>".to_string());
                i += 1;
            } else {
                for (c, r) in done[i].iter().zip(rows[i].iter()) {
                    // An equation keeps the indent the source gives it from the page margin.
                    let indent = r.left - page_left;
                    let typeset = c.lines.iter().any(|l| l.contains("class=\"imp-eq\""));
                    if matches!(r.kind, RegionKind::Math(_)) && typeset && indent >= 3.0 {
                        // The indent is the source's page position, held as a spacer (`--ind`) that shrinks before the
                        // equation does: in a pane narrower than the page a matrix keeps its full width.
                        let with = format!("class=\"imp-eq\" style=\"--ind:{indent:.0}pt\"");
                        self.lines
                            .extend(c.lines.iter().map(|l| l.replace("class=\"imp-eq\"", &with)));
                    } else {
                        self.lines.extend(c.lines.iter().cloned());
                    }
                }
                i += 1;
            }
        }
    }

    /// Lowers a program that is the value of `name := program` to Calcpad statements and the
    /// definition of `name`. Returns the text and whether the value is an array, or the reason
    /// the program is not read. Locals get names of their own (`x_p1`), so nothing leaks into the
    /// sheet's variables; an array is only written element by element if the program created it
    /// whole first, because the engine cannot grow one.
    fn lower_program(&mut self, stmts: &[PStmt], name: &str) -> Result<(String, bool), String> {
        let owned;
        let stmts = if needs_result_form(stmts) {
            if ends_without_value(stmts) {
                return Err("the program does not end with a value".into());
            }
            owned = result_form(stmts);
            &owned[..]
        } else {
            stmts
        };
        let Some((last, body)) = stmts.split_last() else {
            return Err("an empty program".into());
        };
        let tail = match last {
            PStmt::Expr(e) | PStmt::Return(e) => e,
            _ => return Err("the program does not end with a value".into()),
        };
        self.prog_n += 1;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_arrays = std::mem::take(&mut self.local_arrays);
        self.zero_started.clear();
        self.growable.clear();
        self.prologue.clear();
        let saved_zero_units = std::mem::take(&mut self.zero_units);
        {
            let mut locals = HashSet::new();
            program_locals(body, &mut locals);
            let mut zeroed = Vec::new();
            zero_assigned(body, &mut zeroed);
            for b in zeroed {
                if let Some(r) = first_accumulated(body, &b, &mut HashMap::new(), &locals) {
                    self.zero_units.insert(b, r);
                }
            }
        }
        let mut lines = Vec::new();
        // A `return` before the end is a result variable and a done flag, so nothing after it runs.
        let early = body.iter().any(may_return);
        let saved_ret = self.ret.take();
        if early {
            let (res_v, done_v) = (format!("ret_p{}", self.prog_n), format!("done_p{}", self.prog_n));
            lines.push(format!("{done_v} = 0"));
            self.ret = Some((res_v, done_v));
        }
        self.loop_stack.clear();
        self.cond_depth = 0;
        let res = self.lower_stmts(body, 0, false, &mut lines);
        let out = match res {
            Ok(()) => {
                let value = self.expr(tail);
                let is_array = !early
                    && matches!(tail, Expr::Name(n)
                        if self.locals.get(&tables::normalise_name(&n.flat())).is_some_and(|m| self.local_arrays.contains(m)));
                let def = if let Some((res_v, done_v)) = &self.ret {
                    lines.push(format!("#if {done_v} ≡ 0"));
                    lines.push(format!("    {res_v} = {}", value.s));
                    lines.push("#end if".to_string());
                    format!("{} = {}", name, res_v)
                } else {
                    format!("{} = {}", name, value.s)
                };
                lines.splice(0..0, std::mem::take(&mut self.prologue));
                if lines.is_empty() {
                    Ok((def, is_array))
                } else {
                    Ok((format!("#hide\n{}\n#end hide\n{}", lines.join("\n"), def), is_array))
                }
            }
            Err(e) => Err(e),
        };
        self.locals = saved_locals;
        self.local_arrays = saved_arrays;
        self.zero_units = saved_zero_units;
        self.ret = saved_ret;
        out
    }

    /// A function body that is statements, as one expression block: `$Block{ ...; value }`. A
    /// `for` is a `$Repeat` (counting steps and deriving the variable, so it runs down when the end
    /// is below the start as the source does), a `while` a `$While`, and an `if` sets guard
    /// variables (0 or 1, the condition evaluated once) that every assignment under it respects:
    /// `x = if(g; value; x)`. Locals have names of their own; an argument the function assigns to
    /// is copied first. A local first set under a condition, or in a loop and read outside it, has
    /// no value in the source on some paths and none here, so such a function is a gap, as are
    /// `break`, `continue`, an early `return` and an array the function grows.
    fn lower_fn_block(&mut self, stmts: &[PStmt]) -> Result<String, String> {
        let owned;
        let stmts = if needs_result_form(stmts) {
            if ends_without_value(stmts) {
                return Err("the program does not end with a value".into());
            }
            owned = result_form(stmts);
            &owned[..]
        } else {
            stmts
        };
        let Some((last, body)) = stmts.split_last() else {
            return Err("an empty program".into());
        };
        let tail = match last {
            PStmt::Expr(e) | PStmt::Return(e) => e,
            _ => return Err("the program does not end with a value".into()),
        };
        self.prog_n += 1;
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_partial = std::mem::take(&mut self.partial);
        let mut st = FnBlk {
            total: name_counts(stmts),
            arrays: HashSet::new(),
            inside: None,
            starts: Vec::new(),
            exhaustive: HashSet::new(),
            allocs: Vec::new(),
            no_alloc: HashMap::new(),
            writes: Vec::new(),
            ret: None,
            flags: Vec::new(),
            exact: HashSet::new(),
            grow_starts: Vec::new(),
            scan: growth::scan_element_writes(stmts),
        };
        let mut items: Vec<String> = Vec::new();
        let r = (|| -> Result<String, String> {
            // An argument the function assigns to is a local copy from the start.
            let mut written = HashSet::new();
            assigned_names(body, &mut written);
            let mut copies: Vec<String> = written.into_iter().filter(|w| self.params.contains(w)).collect();
            copies.sort();
            let copies_len = copies.len();
            for c in copies {
                if !valid_ident(&c) {
                    return Err(format!("argument name {c} that Calcpad cannot spell"));
                }
                let m = self.local_name(&c);
                // `*1` makes a new array: the engine's `B = A` would share it, and a write to B would change A.
                items.push(format!("{m} = {c}*1"));
                st.arrays.insert(c);
            }
            // A `return` before the end sets a result and a done flag (zero until set: a value
            // that is not a number cannot be held in a loop, and the result is read only once the
            // flag is set).
            if exits::exits_of(body).ret {
                let (res, done) = (format!("ret_p{}", self.prog_n), format!("done_p{}", self.prog_n));
                st.starts.push(format!("{res} = 0"));
                st.starts.push(format!("{done} = 0"));
                st.ret = Some(exits::RetFlags { res, done });
            }
            // Arrays the body fills by element without having made them are allocated, at the size
            // their writes need, just before the statement that writes them.
            let mut whole = HashSet::new();
            whole_assigned(body, &mut whole);
            let mut loops = Vec::new();
            for (k, s) in body.iter().enumerate() {
                let (mut w, mut bad) = (Vec::new(), HashMap::new());
                element_writes(std::slice::from_ref(s), k, false, &mut loops, false, &mut w, &mut bad);
                st.writes.extend(w);
                st.no_alloc.extend(bad);
            }
            let mut bases: Vec<String> = st.scan.keys().cloned().collect();
            bases.sort();
            for b in bases {
                if self.params.contains(&b) || st.arrays.contains(&b) {
                    continue;
                }
                // Also assigned whole somewhere: the array may exist already when it is first written
                // (checked as the statements are lowered), so it is made lazily and grows.
                if whole.contains(&b) {
                    st.grow_from(&b);
                    continue;
                }
                let mine: Vec<&ElemWrite> = st.writes.iter().filter(|w| w.base == b).collect();
                let mut places: Vec<usize> = mine.iter().map(|w| w.at).collect();
                places.sort();
                places.dedup();
                let literal_only = mine.iter().all(|w| w.needs.iter().all(|n| matches!(n, Need::Lit(_))));
                // Sized ahead when its writes are in one statement, or all at numbers (the size does
                // not depend on what runs between them).
                let slices = st.scan.get(&b).is_some_and(|i| i.has_slice);
                if !slices && !st.no_alloc.contains_key(&b) && !places.is_empty() && (places.len() == 1 || literal_only)
                {
                    st.allocs.push((places[0], b));
                    continue;
                }
                st.grow_from(&b);
            }
            for (k, stmt) in body.iter().enumerate() {
                let names: Vec<String> = st
                    .allocs
                    .iter()
                    .filter(|(at, _)| *at == k)
                    .map(|(_, b)| b.clone())
                    .collect();
                for b in names {
                    if let Err(why) = self.alloc_array(&b, stmt, &mut st, &mut items) {
                        st.no_alloc.insert(b.clone(), why);
                        st.grow_from(&b);
                    }
                }
                // Arrays that start with one zero and grow, made before the statement that first writes them.
                let due: Vec<(String, usize)> = st
                    .grow_starts
                    .iter()
                    .filter(|(at, b, _)| *at <= k && !st.arrays.contains(b))
                    .map(|(_, b, r)| (b.clone(), *r))
                    .collect();
                for (b, rank) in due {
                    let mapped = self.local_name(&b);
                    items.push(if rank == 1 {
                        format!("{mapped} = vector(1)")
                    } else {
                        format!("{mapped} = matrix(1; 1)")
                    });
                    st.arrays.insert(b);
                }
                self.lower_fn_stmts(std::slice::from_ref(stmt), None, 0, &mut st, &mut items)?;
            }
            let at = items.len().min(copies_len);
            items.splice(at..at, st.starts.drain(..));
            let mut value = self.expr(tail).s;
            if let Some(r) = &st.ret {
                value = format!("if({}; {}; {})", r.done, r.res, value);
            }
            Ok(if items.is_empty() {
                value
            } else {
                format!("$Block{{{}; {}}}", items.join("; "), value)
            })
        })();
        self.locals = saved_locals;
        self.partial = saved_partial;
        r
    }

    /// Makes an array that a statement of a function body fills by element: a vector or matrix of zeros
    /// as large as the largest index its writes reach (a loop's last value, or a literal), in front
    /// of that statement. The writes under a condition must not reach further than the ones that
    /// always run, since the source would then give a smaller array on some paths.
    fn alloc_array(&mut self, name: &str, stmt: &PStmt, st: &mut FnBlk, items: &mut Vec<String>) -> Result<(), String> {
        let writes: Vec<(Vec<Need>, bool)> = st
            .writes
            .iter()
            .filter(|w| w.base == name)
            .map(|w| (w.needs.clone(), w.conditional))
            .collect();
        let rank = writes.first().map(|w| w.0.len()).unwrap_or(0);
        if rank == 0 || writes.iter().any(|w| w.0.len() != rank) {
            return Err("it is indexed with a different number of indexes".into());
        }
        let mut changed = HashSet::new();
        whole_assigned(std::slice::from_ref(stmt), &mut changed);
        let delta = 1 - self.origin;
        let mut dims: Vec<String> = Vec::new();
        for d in 0..rank {
            let mut always: Vec<(String, Option<i64>)> = Vec::new();
            let mut sometimes: Vec<(String, Option<i64>)> = Vec::new();
            for (needs, conditional) in &writes {
                let r = match &needs[d] {
                    Need::Lit(c) => {
                        let v = c + delta;
                        if v < 1 {
                            return Err("an index is below the array origin".into());
                        }
                        (v.to_string(), Some(v))
                    }
                    Need::Loop { first, last, off } => {
                        for e in [first, last] {
                            if changed.iter().any(|a| mentions(e, a)) {
                                return Err("the ends of the loop read a name the loop changes".into());
                            }
                        }
                        let (a, z) = (self.expr(first).s, self.expr(last).s);
                        let k = off + delta;
                        match (is_int_literal(&a), is_int_literal(&z)) {
                            (Some(x), Some(y)) => {
                                let v = x.max(y) + k;
                                (v.to_string(), Some(v))
                            }
                            _ => (
                                match k {
                                    0 => format!("max({a}; {z})"),
                                    k if k > 0 => format!("max({a}; {z}) + {k}"),
                                    k => format!("max({a}; {z}) - {}", -k),
                                },
                                None,
                            ),
                        }
                    }
                };
                if *conditional {
                    sometimes.push(r);
                } else {
                    always.push(r);
                }
            }
            if always.is_empty() {
                return Err("it is written only under a condition".into());
            }
            let all_numbers = always.iter().all(|(_, n)| n.is_some());
            let size = if all_numbers {
                always.iter().filter_map(|(_, n)| *n).max().unwrap_or(1).to_string()
            } else {
                let mut texts: Vec<String> = always.iter().map(|(t, _)| t.clone()).collect();
                texts.dedup();
                if texts.len() == 1 {
                    texts.remove(0)
                } else {
                    format!("max({})", texts.join("; "))
                }
            };
            let biggest = always.iter().filter_map(|(_, n)| *n).max();
            for (t, n) in &sometimes {
                let covered = always.iter().any(|(u, _)| u == t)
                    || matches!((n, biggest, all_numbers), (Some(c), Some(m), true) if *c <= m);
                if !covered {
                    return Err("it is written under a condition at an index the other writes do not reach".into());
                }
            }
            dims.push(size);
        }
        let mapped = self.local_name(name);
        items.push(if rank == 1 {
            format!("{mapped} = vector({})", dims[0])
        } else {
            format!("{mapped} = matrix({}; {})", dims[0], dims[1])
        });
        st.arrays.insert(name.to_string());
        st.exact.insert(name.to_string());
        Ok(())
    }

    fn lower_fn_stmts(
        &mut self,
        stmts: &[PStmt],
        guard: Option<&str>,
        loops: usize,
        st: &mut FnBlk,
        items: &mut Vec<String>,
    ) -> Result<(), String> {
        if loops > 6 {
            return Err("loops nested too deeply".into());
        }
        // The guard changes along the list: after a statement that can end the pass, the loop or the
        // function, the statements that follow run only while no flag is set.
        let mut guard: Option<String> = guard.map(str::to_string);
        for stmt in stmts {
            let g = guard.as_deref();
            match stmt {
                PStmt::Local(PTarget::Var(n), e) => {
                    let flat = tables::normalise_name(&n.flat());
                    if !valid_ident(&flat) {
                        return Err(format!("local name {flat} that Calcpad cannot spell"));
                    }
                    let rhs = self.copy_if_array(e);
                    let mapped = match self.locals.get(&flat) {
                        Some(m) => m.clone(),
                        None => {
                            if let Some(why) = st.first_set_problem(&flat, g, loops) {
                                // The program's own result variable may be unset on a path (the source
                                // then has no value): it starts as not a number, which the engine shows
                                // as undefined. Outside a loop only: a not-a-number inside a loop body
                                // is an engine error even on the iterations that do not use it.
                                if flat == RESULT_LOCAL && loops == 0 {
                                    let mapped = self.local_name(&flat);
                                    st.starts.push(format!("{mapped} = sqrt(-1)"));
                                    mapped
                                } else if self.yields_array(e) || matches!(e, Expr::Matrix { .. }) {
                                    return Err(why);
                                } else {
                                    // Any other local that may have no value where it is read keeps a
                                    // flag that says whether it was set; it starts at zero (a number
                                    // can be held in a loop) and a read of it is not a number while
                                    // the flag is clear.
                                    let mapped = self.local_name(&flat);
                                    let flag = format!("{mapped}_set");
                                    st.starts.push(format!("{mapped} = 0"));
                                    st.starts.push(format!("{flag} = 0"));
                                    self.partial.insert(mapped.clone(), flag);
                                    mapped
                                }
                            } else {
                                let mapped = self.local_name(&flat);
                                if g.is_some() {
                                    // Set in every arm of an `if`: the first write reads the name's
                                    // earlier value, which only has to exist for the engine.
                                    st.starts.push(format!("{mapped} = sqrt(-1)"));
                                }
                                mapped
                            }
                        }
                    };
                    let is_array = self.fn_gives_array(e, &st.arrays);
                    if is_array {
                        st.arrays.insert(flat);
                    } else {
                        st.arrays.remove(&flat);
                    }
                    items.push(match g {
                        None => format!("{mapped} = {rhs}"),
                        Some(g) => format!("{mapped} = if({g}; {rhs}; {mapped})"),
                    });
                    if let Some(flag) = self.partial.get(&mapped) {
                        items.push(match g {
                            None => format!("{flag} = 1"),
                            Some(g) => format!("{flag} = if({g}; 1; {flag})"),
                        });
                    }
                }
                PStmt::Local(PTarget::Slice { base, index, row }, e) => {
                    let flat = tables::normalise_name(&base.flat());
                    let (Some(mapped), true) = (self.locals.get(&flat).cloned(), st.arrays.contains(&flat)) else {
                        return Err(format!(
                            "assignment to a column of {flat}, an array the function neither received nor created"
                        ));
                    };
                    let idx = self.index_expr(index);
                    let rhs = self.expr(e).s;
                    self.prog_n += 1;
                    let uid = self.prog_n;
                    let t = format!("sl_t{uid}");
                    items.push(match g {
                        None => format!("{t} = {rhs}"),
                        Some(g) => format!("{t} = if({g}; {rhs}; vector(1))"),
                    });
                    items.extend(self.slice_write(&mapped, &idx, *row, &t, uid, g));
                }
                PStmt::Local(PTarget::Element { base, index }, e) => {
                    let flat = tables::normalise_name(&base.flat());
                    let (Some(mapped), true) = (self.locals.get(&flat).cloned(), st.arrays.contains(&flat)) else {
                        return Err(match st.no_alloc.get(&flat) {
                            Some(why) => format!(
                                "element assignment to {flat}, an array the function neither received nor created, whose size is not known ahead: {why}"
                            ),
                            None => format!(
                                "element assignment to {flat}, an array the function neither received nor created (the source grows it, the engine cannot)"
                            ),
                        });
                    };
                    if index.is_empty() || index.len() > 2 {
                        return Err("an element index that is not one or two numbers".into());
                    }
                    let parts: Vec<String> = index.iter().map(|i| self.index_expr(i)).collect();
                    let rhs = self.expr(e).s;
                    if st.exact.contains(&flat) {
                        let at = format!("{mapped}.({})", parts.join("; "));
                        items.push(match g {
                            None => format!("{at} = {rhs}"),
                            Some(g) => format!("{at} = if({g}; {rhs}; {at})"),
                        });
                    } else {
                        items.extend(self.growing_write(&mapped, &parts, &rhs, g));
                    }
                }
                PStmt::For { var, range, body } => {
                    let Expr::Range {
                        first,
                        second: None,
                        last,
                    } = range
                    else {
                        return Err("a loop over something other than a plain range a .. b".into());
                    };
                    let v = tables::normalise_name(&var.flat());
                    if !valid_ident(&v) {
                        return Err(format!("loop variable {v} that Calcpad cannot spell"));
                    }
                    let a = self.expr(first).s;
                    let z = self.expr(last).s;
                    self.prog_n += 1;
                    let mapped = format!("{v}_p{}", self.prog_n);
                    let literal = |e: &Expr| match e {
                        Expr::Num(s) => is_int_literal(s),
                        _ => None,
                    };
                    let mut inner: Vec<String> = Vec::new();
                    let header = match (literal(first), literal(last)) {
                        (Some(x), Some(y)) if x <= y => format!("{mapped} = {x} : {y}"),
                        _ => {
                            let (lo, hi, k) = (format!("{mapped}_a"), format!("{mapped}_z"), format!("{mapped}_k"));
                            items.push(format!("{lo} = {a}"));
                            items.push(format!("{hi} = {z}"));
                            inner.push(format!("{mapped} = {lo} + {k}*if({hi} ≥ {lo}; 1; -1)"));
                            format!("{k} = 0 : abs({hi} - {lo})")
                        }
                    };
                    let (loop_guard, ex) = self.enter_loop(body, g, st, items, &mut inner);
                    let prev = self.locals.insert(v.clone(), mapped);
                    let outer = st.inside.replace(name_counts(body));
                    let r = self.lower_fn_stmts(body, loop_guard.as_deref(), loops + 1, st, &mut inner);
                    st.inside = outer;
                    if ex.brk || ex.cont {
                        st.flags.pop();
                    }
                    match prev {
                        Some(p) => self.locals.insert(v, p),
                        None => self.locals.remove(&v),
                    };
                    r?;
                    if inner.is_empty() {
                        return Err("an empty loop".into());
                    }
                    items.push(format!("$Repeat{{{} @ {header}}}", inner.join("; ")));
                }
                PStmt::While { cond, body } => {
                    let c = self.expr(cond).s;
                    // The test runs only while the loop is live: not after a `break`, nor a `return`.
                    let mut inner: Vec<String> = Vec::new();
                    let (loop_guard, ex) = self.enter_loop(body, g, st, items, &mut inner);
                    let stop = {
                        let mut flags = Vec::new();
                        if let Some(r) = &st.ret {
                            flags.push(r.done.clone());
                        }
                        if ex.brk {
                            if let Some(b) = st.flags.last().and_then(|f| f.brk.clone()) {
                                flags.push(b);
                            }
                        }
                        flags
                    };
                    let c = match exits::and_live(g, &stop) {
                        None => c,
                        Some(live) => format!("if({live}; {c}; 0)"),
                    };
                    let outer = st.inside.replace(name_counts(body));
                    inner.insert(0, c);
                    let r = self.lower_fn_stmts(body, loop_guard.as_deref(), loops + 1, st, &mut inner);
                    st.inside = outer;
                    if ex.brk || ex.cont {
                        st.flags.pop();
                    }
                    r?;
                    if inner.len() < 2 {
                        return Err("an empty loop".into());
                    }
                    items.push(format!("$While{{{}}}", inner.join("; ")));
                }
                PStmt::If { branches, otherwise } => {
                    self.prog_n += 1;
                    let n = self.prog_n;
                    if let (None, 0, Some(other)) = (g, loops, otherwise) {
                        // A name every arm assigns at its top level has a value after the `if`.
                        let sets = |b: &[PStmt]| -> HashSet<String> {
                            b.iter()
                                .filter_map(|x| match x {
                                    PStmt::Local(PTarget::Var(v), _) => Some(tables::normalise_name(&v.flat())),
                                    _ => None,
                                })
                                .collect()
                        };
                        let mut common = sets(other);
                        for (_, b) in branches {
                            let here = sets(b);
                            common.retain(|v| here.contains(v));
                        }
                        for v in common {
                            if !self.locals.contains_key(&v) {
                                st.exhaustive.insert(v);
                            }
                        }
                    }
                    let mut taken: Vec<String> = Vec::new();
                    let mut arms: Vec<(String, &[PStmt])> = Vec::new();
                    for (k, (c, b)) in branches.iter().enumerate() {
                        let cs = self.expr(c).s;
                        let test = format!("if({cs}; 1; 0)");
                        let test = if taken.is_empty() {
                            test
                        } else {
                            format!("if({}; 0; {test})", taken.join(" + "))
                        };
                        let test = match g {
                            None => test,
                            Some(g) => format!("if({g}; {test}; 0)"),
                        };
                        let a = format!("on_p{n}_{k}");
                        items.push(format!("{a} = {test}"));
                        taken.push(a.clone());
                        arms.push((a, b));
                    }
                    if let Some(b) = otherwise {
                        let test = format!("if({}; 0; 1)", taken.join(" + "));
                        let test = match g {
                            None => test,
                            Some(g) => format!("if({g}; {test}; 0)"),
                        };
                        let a = format!("on_p{n}_e");
                        items.push(format!("{a} = {test}"));
                        arms.push((a, b));
                    }
                    for (a, b) in arms {
                        self.lower_fn_stmts(b, Some(&a), loops, st, items)?;
                    }
                }
                PStmt::Break | PStmt::Continue => {
                    let is_break = matches!(stmt, PStmt::Break);
                    let flag = st
                        .flags
                        .last()
                        .and_then(|f| if is_break { f.brk.clone() } else { f.cont.clone() });
                    let Some(flag) = flag else {
                        return Err("break or continue outside a loop".into());
                    };
                    items.push(match g {
                        None => format!("{flag} = 1"),
                        Some(g) => format!("{flag} = if({g}; 1; {flag})"),
                    });
                }
                PStmt::Return(e) => {
                    let Some(r) = st.ret.clone() else {
                        return Err("a return before the end of the program (an early exit)".into());
                    };
                    if self.yields_array(e) {
                        return Err("an early return of an array".into());
                    }
                    let rhs = self.expr(e).s;
                    match g {
                        None => {
                            items.push(format!("{} = {rhs}", r.res));
                            items.push(format!("{} = 1", r.done));
                        }
                        Some(g) => {
                            items.push(format!("{} = if({g}; {rhs}; {})", r.res, r.res));
                            items.push(format!("{} = if({g}; 1; {})", r.done, r.done));
                        }
                    }
                }
                PStmt::Expr(_) => return Err("an expression statement before the end of the program".into()),
                PStmt::Unsupported(w) => return Err(format!("{w} is not read")),
            }
            // What the statement can end (the pass, the loop, the function) stops the rest of this list.
            let ex = exits::exits_of(std::slice::from_ref(stmt));
            if ex.any() {
                let mut flags = Vec::new();
                if ex.ret {
                    if let Some(r) = &st.ret {
                        flags.push(r.done.clone());
                    }
                }
                if let Some(f) = st.flags.last() {
                    if ex.brk {
                        flags.extend(f.brk.clone());
                    }
                    if ex.cont {
                        flags.extend(f.cont.clone());
                    }
                }
                if let Some(live) = exits::and_live(guard.as_deref(), &flags) {
                    self.prog_n += 1;
                    let name = format!("go_p{}", self.prog_n);
                    items.push(format!("{name} = {live}"));
                    guard = Some(name);
                }
            }
        }
        Ok(())
    }

    /// Sets up the flags of a loop about to be lowered: a `break` flag and a `continue` flag when its
    /// body has them (cleared before the loop, and the `continue` one at the start of every pass), and
    /// the guard of the body, which holds while the pass is live. Returns that guard and what the
    /// body can exit; when it has a `break` or `continue`, the flags are pushed (the caller pops them).
    fn enter_loop(
        &mut self,
        body: &[PStmt],
        outer: Option<&str>,
        st: &mut FnBlk,
        items: &mut Vec<String>,
        inner: &mut Vec<String>,
    ) -> (Option<String>, exits::Exits) {
        let ex = exits::exits_of(body);
        let mut flags = Vec::new();
        if ex.ret {
            if let Some(r) = &st.ret {
                flags.push(r.done.clone());
            }
        }
        if ex.brk || ex.cont {
            self.prog_n += 1;
            let n = self.prog_n;
            let mut lf = exits::LoopFlags::default();
            if ex.brk {
                let b = format!("brk_p{n}");
                items.push(format!("{b} = 0"));
                flags.push(b.clone());
                lf.brk = Some(b);
            }
            if ex.cont {
                let c = format!("cont_p{n}");
                inner.push(format!("{c} = 0"));
                lf.cont = Some(c);
            }
            st.flags.push(lf);
        }
        let guard = match exits::and_live(outer, &flags) {
            Some(live) if !flags.is_empty() => {
                self.prog_n += 1;
                let name = format!("go_p{}", self.prog_n);
                inner.push(format!("{name} = {live}"));
                Some(name)
            }
            other => other,
        };
        (guard, ex)
    }

    /// The value of `e`; an array that is just another array's name is copied (`*1`), because the
    /// engine's `B = A` makes B the same array as A, so writing an element of B would change A.
    fn copy_if_array(&mut self, e: &Expr) -> String {
        let s = self.expr(e).s;
        if matches!(e, Expr::Name(_)) && self.holds_array(e) {
            format!("{s}*1")
        } else {
            s
        }
    }

    fn local_name(&mut self, source: &str) -> String {
        let mapped = format!("{}_p{}", source, self.prog_n);
        self.locals.insert(source.to_string(), mapped.clone());
        mapped
    }

    fn lower_stmts(
        &mut self,
        stmts: &[PStmt],
        depth: usize,
        in_loop: bool,
        out: &mut Vec<String>,
    ) -> Result<(), String> {
        if depth > 12 {
            return Err("blocks nested too deeply".into());
        }
        let pad = "    ".repeat(depth);
        for (k, st) in stmts.iter().enumerate() {
            let before = out.len();
            let outer_hoisted = std::mem::take(&mut self.hoisted);
            let step = self.lower_stmt(st, depth, in_loop, out);
            let mine = std::mem::replace(&mut self.hoisted, outer_hoisted);
            step?;
            if !mine.is_empty() {
                let lines: Vec<String> = mine.into_iter().map(|l| format!("{pad}{l}")).collect();
                out.splice(before..before, lines);
            }
            // Whatever follows a statement that may have returned runs only if it did not.
            if k + 1 < stmts.len() && may_return(st) {
                if let Some((_, done_v)) = self.ret.clone() {
                    out.push(format!("{pad}#if {done_v} ≡ 0"));
                    self.lower_stmts(&stmts[k + 1..], depth + 1, in_loop, out)?;
                    out.push(format!("{pad}#end if"));
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn lower_stmt(&mut self, st: &PStmt, depth: usize, in_loop: bool, out: &mut Vec<String>) -> Result<(), String> {
        let pad = "    ".repeat(depth);
        {
            match st {
                PStmt::Local(PTarget::Var(n), e) => {
                    let flat = tables::normalise_name(&n.flat());
                    if !valid_ident(&flat) {
                        return Err(format!("local name {flat} that Calcpad cannot spell"));
                    }
                    // The value is read before the name is a local: `x <- x + 1` starts from the outer x.
                    let rhs = self.copy_if_array(e);
                    let is_array = matches!(e, Expr::Matrix { .. })
                        || matches!(e, Expr::Call(f, _) if matches!(f.base.as_str(), "identity" | "matrix" | "augment" | "stack" | "lsolve" | "submatrix"));
                    let mapped = match self.locals.get(&flat) {
                        Some(m) => m.clone(),
                        None => self.local_name(&flat),
                    };
                    if is_array {
                        self.local_arrays.insert(mapped.clone());
                    } else {
                        self.local_arrays.remove(&mapped);
                    }
                    out.push(format!("{pad}{mapped} = {rhs}"));
                }
                PStmt::Local(PTarget::Slice { .. }, _) => {
                    return Err("assignment to a whole column of an array in a worksheet-level program".into());
                }
                PStmt::Local(PTarget::Element { base, index }, e) => {
                    let flat = tables::normalise_name(&base.flat());
                    let Some(mapped) = self
                        .locals
                        .get(&flat)
                        .cloned()
                        .or_else(|| self.grow_local(&flat, index, out))
                        .or_else(|| self.growing_local(&flat, index.len()))
                    else {
                        return Err(format!(
                            "element assignment to {flat}, which the program did not create and whose size cannot be known ahead (the source grows it, the engine cannot)"
                        ));
                    };
                    if !self.local_arrays.contains(&mapped) {
                        return Err(format!(
                            "element assignment to {flat}, which grows element by element; the engine cannot grow an array"
                        ));
                    }
                    if index.is_empty() || index.len() > 2 {
                        return Err("an element index that is not one or two numbers".into());
                    }
                    // The source lets a literal 0 take the units of what is added to it; the engine does
                    // not. A 0 that is later added to gets the units of the first value added (`0*getunits(x)`
                    // is a zero in those units), so the sums are consistent; the units are read from the first
                    // term with the loop variables at their first values, which is only possible when that term
                    // reads nothing the program sets.
                    let is_zero = matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
                    let mut zero_rhs = None;
                    if is_zero {
                        self.zero_started.insert(mapped.clone());
                        match self.zero_units.get(&flat).cloned() {
                            Some(Ok(term)) => zero_rhs = Some(format!("0*getunits({})", self.expr(&term).s)),
                            Some(Err(why)) => {
                                return Err(format!(
                                    "{flat} starts from a unitless 0 and is then added to: the source lets 0 take the units of what is added, and here {why}"
                                ))
                            }
                            None => {}
                        }
                    } else if self.zero_started.contains(&mapped)
                        && mentions(e, &flat)
                        && !self.zero_units.contains_key(&flat)
                    {
                        return Err(format!(
                            "{flat} starts from a unitless 0 and is then added to: the source lets 0 take the units of what is added, the engine reports inconsistent units"
                        ));
                    }
                    if self.yields_array(e) {
                        return Err(format!(
                            "{flat} gets an array for one element (a matrix built row by row); the engine's vector element is a number"
                        ));
                    }
                    let parts: Vec<String> = index.iter().map(|i| self.index_expr(i)).collect();
                    let rhs = match zero_rhs {
                        Some(z) => z,
                        None => self.expr(e).s,
                    };
                    if self.growable.contains(&mapped) {
                        out.extend(Self::growth_lines(&mapped, &parts, &pad));
                    }
                    out.push(format!("{pad}{mapped}.({}) = {rhs}", parts.join("; ")));
                }
                PStmt::For { var, range, body } => {
                    let Expr::Range {
                        first,
                        second: None,
                        last,
                    } = range
                    else {
                        return Err("a loop over something other than a plain range a .. b".into());
                    };
                    let v = tables::normalise_name(&var.flat());
                    if !valid_ident(&v) {
                        return Err(format!("loop variable {v} that Calcpad cannot spell"));
                    }
                    let a = self.expr(first).s;
                    let z = self.expr(last).s;
                    let prev = self.locals.get(&v).cloned();
                    let mapped = format!("{}_p{}", v, self.prog_n);
                    let start = out.len();
                    let o = self.origin;
                    let depends_on_locals = self.locals.keys().any(|k| mentions(first, k) || mentions(last, k));
                    let size = if depends_on_locals || body.iter().any(has_break) {
                        None
                    } else {
                        match (&**first, &**last) {
                            (Expr::Num(x), Expr::Num(y)) => match (is_int_literal(x), is_int_literal(y)) {
                                (Some(x), Some(y)) if x <= y && x >= o => Some((y - o + 1).to_string()),
                                _ => None,
                            },
                            _ if o == 1 => Some(format!("max({a}; {z})")),
                            _ => Some(format!("max({a}; {z}) - {}", o - 1)),
                        }
                    };
                    let literal = |e: &Expr| match e {
                        Expr::Num(s) => is_int_literal(s),
                        _ => None,
                    };
                    match (literal(first), literal(last)) {
                        (Some(x), Some(y)) if x <= y => {
                            out.push(format!("{pad}#for {mapped} = {x} : {y}"));
                        }
                        _ => {
                            // The source counts down when the end is below the start; the engine's
                            // loop only counts up, so count steps and derive the value.
                            let (lo, hi, k) = (format!("{mapped}_a"), format!("{mapped}_z"), format!("{mapped}_k"));
                            out.push(format!("{pad}{lo} = {a}"));
                            out.push(format!("{pad}{hi} = {z}"));
                            out.push(format!("{pad}#for {k} = 0 : abs({hi} - {lo})"));
                            out.push(format!("{pad}    {mapped} = {lo} + {k}*if({hi} ≥ {lo}; 1; -1)"));
                        }
                    }
                    self.locals.insert(v.clone(), mapped);
                    if self.ret.is_some() && body.iter().any(may_return) {
                        let done_v = self.ret.as_ref().map(|r| r.1.clone()).unwrap_or_default();
                        out.push(format!("{pad}    #if {done_v} ≡ 1"));
                        out.push(format!("{pad}        #break"));
                        out.push(format!("{pad}    #end if"));
                    }
                    self.loop_stack.push(LoopCtx {
                        var: v.clone(),
                        size,
                        start,
                    });
                    let r = self.lower_stmts(body, depth + 1, true, out);
                    self.loop_stack.pop();
                    match prev {
                        Some(p) => self.locals.insert(v, p),
                        None => self.locals.remove(&v),
                    };
                    r?;
                    out.push(format!("{pad}#loop"));
                }
                PStmt::While { cond, body } => {
                    let c = self.expr(cond).s;
                    if !self.hoisted.is_empty() {
                        return Err("the result of a call is indexed in a loop condition".into());
                    }
                    out.push(format!("{pad}#while {c}"));
                    if self.ret.is_some() && body.iter().any(may_return) {
                        let done_v = self.ret.as_ref().map(|r| r.1.clone()).unwrap_or_default();
                        out.push(format!("{pad}    #if {done_v} ≡ 1"));
                        out.push(format!("{pad}        #break"));
                        out.push(format!("{pad}    #end if"));
                    }
                    self.cond_depth += 1;
                    let r = self.lower_stmts(body, depth + 1, true, out);
                    self.cond_depth -= 1;
                    r?;
                    out.push(format!("{pad}#loop"));
                }
                PStmt::If { branches, otherwise } => {
                    self.cond_depth += 1;
                    for (i, (c, b)) in branches.iter().enumerate() {
                        let cs = self.expr(c).s;
                        out.push(format!("{pad}{} {cs}", if i == 0 { "#if" } else { "#else if" }));
                        if b.is_empty() {
                            self.cond_depth -= 1;
                            return Err("an empty branch".into());
                        }
                        self.lower_stmts(b, depth + 1, in_loop, out)?;
                    }
                    if let Some(b) = otherwise {
                        if b.is_empty() {
                            self.cond_depth -= 1;
                            return Err("an empty branch".into());
                        }
                        out.push(format!("{pad}#else"));
                        self.lower_stmts(b, depth + 1, in_loop, out)?;
                    }
                    self.cond_depth -= 1;
                    out.push(format!("{pad}#end if"));
                }
                PStmt::Break if in_loop => out.push(format!("{pad}#break")),
                PStmt::Continue if in_loop => out.push(format!("{pad}#continue")),
                PStmt::Break | PStmt::Continue => return Err("break or continue outside a loop".into()),
                PStmt::Return(e) => {
                    let Some((res_v, done_v)) = self.ret.clone() else {
                        return Err("a return before the end of the program (an early exit)".into());
                    };
                    let v = self.expr(e).s;
                    out.push(format!("{pad}{res_v} = {v}"));
                    out.push(format!("{pad}{done_v} = 1"));
                    if in_loop {
                        out.push(format!("{pad}#break"));
                    }
                }
                PStmt::Expr(_) => return Err("an expression statement before the end of the program".into()),
                PStmt::Unsupported(w) => return Err(format!("{w} is not read")),
            }
        }
        Ok(())
    }

    /// An element write to an array the program has not created: if every index is the variable
    /// of an enclosing `for` loop (each different), the writes are unconditional, and the loops'
    /// bounds are known ahead of the first of those loops, the array is allocated just before that
    /// loop with the largest index each loop reaches. Returns the local's output name.
    fn grow_local(&mut self, flat: &str, index: &[Expr], out: &mut Vec<String>) -> Option<String> {
        if self.cond_depth > 0 || index.is_empty() || index.len() > 2 || !valid_ident(flat) {
            return None;
        }
        let mut sizes = Vec::new();
        let mut start = usize::MAX;
        for (d, e) in index.iter().enumerate() {
            let Expr::Name(n) = e else { return None };
            let v = tables::normalise_name(&n.flat());
            if index[..d]
                .iter()
                .any(|p| matches!(p, Expr::Name(q) if tables::normalise_name(&q.flat()) == v))
            {
                return None;
            }
            let ctx = self.loop_stack.iter().find(|c| c.var == v)?;
            sizes.push(ctx.size.clone()?);
            start = start.min(ctx.start);
        }
        let mapped = self.local_name(flat);
        let alloc = if sizes.len() == 1 {
            format!("{mapped} = vector({})", sizes[0])
        } else {
            format!("{mapped} = matrix({}; {})", sizes[0], sizes[1])
        };
        let pad = self
            .loop_stack
            .iter()
            .position(|c| c.start == start)
            .map(|p| "    ".repeat(p))
            .unwrap_or_default();
        // The nesting of the loop's line, not of the write, is the allocation's indent.
        out.insert(start, format!("{pad}{alloc}"));
        for c in self.loop_stack.iter_mut() {
            if c.start >= start {
                c.start += 1;
            }
        }
        self.local_arrays.insert(mapped.clone());
        Some(mapped)
    }

    /// An element write to an array the program has not created, whose size cannot be known
    /// ahead (a counter as the index, a write under a condition): the array is started with one
    /// zero at the top of the program and grown by `resize` before a write past its end, so it ends
    /// as large as the largest index written, as in the source. Only for a name the worksheet has
    /// no value of its own (a worksheet name of the same name is not where it starts).
    fn growing_local(&mut self, flat: &str, rank: usize) -> Option<String> {
        if rank == 0 || rank > 2 || !valid_ident(flat) {
            return None;
        }
        // A program's own array: a worksheet name of the same name is not its starting value (a stored
        // 2 x 19 result of a program that writes rows 1 and 2 of such a name shows it starts empty).
        if self.params.iter().any(|p| p == flat) {
            return None;
        }
        let mapped = self.local_name(flat);
        let alloc = if rank == 1 {
            format!("{mapped} = vector(1)")
        } else {
            format!("{mapped} = matrix(1; 1)")
        };
        self.prologue.push(alloc);
        self.local_arrays.insert(mapped.clone());
        self.growable.insert(mapped.clone());
        Some(mapped)
    }

    /// `$Integral{body @ x = a : b}`. The bound variable is local to the body; if the bounds
    /// mention the same name the reading is ambiguous and the region is a gap.
    fn integral(&mut self, var: &Name, body: &Expr, lower: &Expr, upper: &Expr) -> R {
        let v = tables::normalise_name(&var.flat());
        if !valid_ident(&v) {
            self.issue(Code::InvalidName, Some(v));
            return atom("?name?");
        }
        if mentions(lower, &v) || mentions(upper, &v) {
            self.issue(
                Code::CalculusOperator,
                Some("a bound uses the integration variable's name".into()),
            );
            if !self.readable {
                return atom("?calculus?");
            }
        }
        let mut lo = self.expr(lower).s;
        let mut hi = self.expr(upper).s;
        // A bare 0 is a zero of any dimension in the source; the engine wants both bounds in
        // the same units, so it is written as a zero of the other bound's dimension.
        let is_zero = |e: &Expr| matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
        if is_zero(lower) && !is_zero(upper) {
            lo = format!("0*({hi})");
        } else if is_zero(upper) && !is_zero(lower) {
            hi = format!("0*({lo})");
        }
        self.params.push(v.clone());
        let b = self.expr(body).s;
        self.params.pop();
        atom(format!("$Integral{{{b} @ {v} = {lo} : {hi}}}"))
    }

    /// `$Sum{body @ i = lo : hi}` (or `$Product`). Without bounds the operator runs over the range
    /// variable of the same name, whose bounds are those the earlier region gave it.
    fn series(&mut self, product: bool, var: &Name, body: &Expr, bounds: Option<&(Box<Expr>, Box<Expr>)>) -> R {
        let v = tables::normalise_name(&var.flat());
        if !valid_ident(&v) {
            self.issue(Code::InvalidName, Some(v));
            return atom("?name?");
        }
        let (lo, hi) = match bounds {
            Some((lo, hi)) => {
                if mentions(lo, &v) || mentions(hi, &v) {
                    self.issue(
                        Code::SummationOperator,
                        Some("a bound uses the summation variable's name".into()),
                    );
                    if !self.readable {
                        return atom("?summation?");
                    }
                }
                (self.expr(lo).s, self.expr(hi).s)
            }
            None => match self.ranges.get(&v).cloned() {
                Some(info) if !self.params.contains(&v) => (info.a, info.z),
                _ => {
                    self.issue(
                        Code::SummationOperator,
                        Some(format!("{v} is not a range variable with bounds")),
                    );
                    if !self.readable {
                        return atom("?summation?");
                    }
                    ("?".into(), "?".into())
                }
            },
        };
        self.params.push(v.clone());
        let b = self.expr(body).s;
        self.params.pop();
        let op = if product { "Product" } else { "Sum" };
        atom(format!("${op}{{{b} @ {v} = {lo} : {hi}}}"))
    }

    /// `$Slope{body @ x = x}`: the derivative at the value `x` has now. If `x` has no value in the
    /// sheet the source would not have calculated it either, so that is reported as such.
    fn derivative(&mut self, var: &Name, body: &Expr) -> R {
        let v = tables::normalise_name(&var.flat());
        if !valid_ident(&v) {
            self.issue(Code::InvalidName, Some(v));
            return atom("?name?");
        }
        if !self.defined.contains(&v) || self.params.contains(&v) {
            self.issue(
                Code::CalculusOperator,
                Some("derivative at a variable that has no value to evaluate it at".into()),
            );
            if !self.readable {
                return atom("?calculus?");
            }
        }
        let b = self.expr(body).s;
        atom(format!("$Slope{{{b} @ {v} = {v}}}"))
    }

    fn unit_only(&mut self, n: &Name) -> String {
        if n.sub.is_none() {
            if let Some(u) = tables::map_unit(&n.base) {
                return u;
            }
            if tables::is_unmapped_unit(&n.base) {
                self.issue(Code::UnmappedUnit, Some(n.base.clone()));
                return format!("?{}?", n.base);
            }
        }
        self.issue(Code::UnmappedUnit, Some(n.flat()));
        "?unit?".into()
    }

    fn name_ref(&mut self, n: &Name) -> R {
        if self.display {
            // A range variable is a name like any other on the page.
            let flat = tables::normalise_name(&n.flat());
            if self.ranges.contains_key(&flat) && !n.unit {
                return atom(self.author_name(&flat));
            }
        }
        let r = self.name_ref_engine(n);
        if self.display {
            if let Some(orig) = self.renamed.get(&r.s) {
                self.display_renamed = true;
                return atom(orig.clone());
            }
        }
        r
    }

    fn name_ref_engine(&mut self, n: &Name) -> R {
        let flat = tables::normalise_name(&n.flat());
        if self.str_vars.contains_key(&flat)
            && !self.str_allow
            && !self.readable
            && !self.display
            && !self.params.contains(&flat)
            && !self.locals.contains_key(&flat)
        {
            // A text held as a code is not a number.
            self.issue(Code::StringValue, Some(flat.clone()));
            return self.stand_in(&flat, "?string?");
        }
        // The file says this identifier is a unit: it is, whatever variables exist.
        if n.unit && n.sub.is_none() {
            if let Some(u) = tables::map_unit(&n.base) {
                return atom(u);
            }
        }
        if let Some(m) = self.locals.get(&flat) {
            if let Some(flag) = self.partial.get(m) {
                return atom(format!("if({flag}; {m}; sqrt(-1))"));
            }
            return atom(m.clone());
        }
        if self.ranges.contains_key(&flat) && !self.params.contains(&flat) {
            self.issue(
                Code::RangeVariable,
                Some(format!(
                    "{flat} is a range variable; it is only read as the loop of an element assignment"
                )),
            );
            return self.stand_in(&flat, "?range?");
        }
        if self.params.contains(&flat) || self.defined.contains(&flat) || self.funcs.contains_key(&flat) {
            if !valid_ident(&flat) {
                self.issue(Code::InvalidName, Some(flat.clone()));
                return self.stand_in(&flat, "?name?");
            }
            return atom(flat);
        }
        if let Some(root) = self.gap_names.get(&flat).copied() {
            self.cur.push(Pending {
                code: Code::DependsOnGap,
                detail: Some(flat.clone()),
                root: Some(root),
            });
            return self.stand_in(&flat, "?gap?");
        }
        if n.sub.is_none() {
            if tables::is_solve_name(&n.base) {
                self.issue(Code::SolveBlock, Some(n.base.clone()));
                return self.stand_in(&n.base, "?solve?");
            }
            if let Some(c) = tables::lookup_constant(&n.base) {
                return match c {
                    Constant::Text(t) => R {
                        s: t.to_string(),
                        prec: P_ATOM,
                    },
                    Constant::Origin => {
                        if self.origin != 0 {
                            self.issue(Code::ArrayOrigin, None);
                        }
                        atom(self.origin.to_string())
                    }
                };
            }
            if let Some(u) = tables::map_unit(&n.base) {
                return atom(u);
            }
            if tables::is_unmapped_unit(&n.base) {
                self.issue(Code::UnmappedUnit, Some(n.base.clone()));
                return self.stand_in(&n.base, &format!("?{}?", n.base));
            }
        }
        self.issue(Code::UndefinedVariable, Some(flat.clone()));
        self.stand_in(&flat, "?name?")
    }

    fn call(&mut self, n: &Name, args: &[Expr]) -> R {
        let flat = tables::normalise_name(&n.flat());
        let text_call_ok = std::mem::take(&mut self.str_allow);
        if self.str_funcs.contains_key(&flat) && !text_call_ok && !self.readable && !self.display {
            self.issue(Code::StringValue, Some(flat.clone()));
            return self.stand_in(&flat, "?string?");
        }
        // A variable applied like a function is a product.
        if (self.defined.contains(&flat) || self.params.contains(&flat)) && !self.funcs.contains_key(&flat) {
            if args.len() == 1 {
                self.issue(Code::CallAsMultiplication, None);
                let a = self.operand(&args[0], P_MUL, false);
                return R {
                    s: format!("{}*{}", flat, a),
                    prec: P_MUL,
                };
            }
            self.issue(Code::BadStructure, Some("variable applied to several arguments".into()));
            let shown = self.render_args(args);
            return self.stand_in(&format!("{}({})", flat, shown.join("; ")), "?call?");
        }
        // The number a text box holds as text is the number itself.
        if flat == "str2num" && n.sub.is_none() && !self.funcs.contains_key(&flat) && args.len() == 1 {
            if let Expr::Name(a) = &args[0] {
                let f = tables::normalise_name(&a.flat());
                if self.num_texts.contains(&f) {
                    return self.name_ref(a);
                }
                if self.empty_texts.contains(&f) {
                    // No text, no number: the source reads this only where a condition keeps it
                    // from being used.
                    self.issue(Code::ControlInput, Some("str2num of an empty text box is 0".into()));
                    return atom("0");
                }
            }
        }
        let mut rendered = if flat == "if" && n.sub.is_none() && args.len() == 3 && !self.funcs.contains_key(&flat) {
            // The branches may be texts (held as codes).
            let c = self.expr(&args[0]).s;
            let a = self.str_or_expr(&args[1]).s;
            let b = self.str_or_expr(&args[2]).s;
            vec![c, a, b]
        } else {
            // The names of the files a read or write names are not text values.
            let io = n.sub.is_none()
                && matches!(
                    flat.as_str(),
                    "READEXCEL" | "WRITEEXCEL" | "READPRN" | "WRITEPRN" | "APPENDPRN"
                );
            self.io_args = io;
            let r = self.render_args(args);
            self.io_args = false;
            r
        };
        // `stack` and `augment` take a number as a one-element array in the source; the engine wants arrays.
        if matches!(flat.as_str(), "stack" | "augment") && n.sub.is_none() && !self.funcs.contains_key(&flat) {
            for (a, r) in args.iter().zip(rendered.iter_mut()) {
                let scalar = match a {
                    Expr::Num(_) => true,
                    Expr::Name(m) => {
                        let f = tables::normalise_name(&m.flat());
                        self.defined.contains(&f)
                            && !self.arrays.contains(&f)
                            && !self.local_arrays.contains(&f)
                            && !self.str_mats.contains(&f)
                            && !self.col_vecs.contains(&f)
                            && !self.params.contains(&f)
                            && !self.locals.contains_key(&f)
                    }
                    _ => false,
                };
                if scalar {
                    *r = format!("[{r}]");
                }
            }
        }
        // In a plot, a bare 0 branch of `if` is a zero of the other branch's dimension (as in the
        // source); the engine wants both branches in the same units.
        // The same for `max` and `min`, and (outside a display) for the branches of `if` everywhere.
        let zero_ok = self.plot_param.is_some() || (!self.display && !self.readable);
        if zero_ok && matches!(flat.as_str(), "max" | "min") && n.sub.is_none() && !self.funcs.contains_key(&flat) {
            let zero = |e: &Expr| matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
            if let Some(k) = args.iter().position(|a| !zero(a)) {
                let other = rendered[k].clone();
                for (a, r) in args.iter().zip(rendered.iter_mut()) {
                    if zero(a) {
                        *r = format!("0*getunits({other})");
                    }
                }
            }
        }
        if zero_ok && flat == "if" && n.sub.is_none() && !self.funcs.contains_key(&flat) && args.len() == 3 {
            let zero = |e: &Expr| matches!(e, Expr::Num(s) if s.trim().parse::<f64>() == Ok(0.0));
            if zero(&args[1]) && !zero(&args[2]) {
                rendered[1] = format!("0*getunits({})", rendered[2]);
            } else if zero(&args[2]) && !zero(&args[1]) {
                rendered[2] = format!("0*getunits({})", rendered[1]);
            }
        }
        let shown_call = format!("{}({})", flat, rendered.join("; "));
        // `x / SIUnitsOf(x)` is the number of `x` in SI base units. The engine holds SI values but
        // has no function for their unit, only `getunits` for the unit a value is shown in.
        if flat == "SIUnitsOf" && n.sub.is_none() && !self.funcs.contains_key(&flat) && args.len() == 1 {
            self.issue(Code::SiUnitScale, None);
            return atom(format!("getunits({})", rendered[0]));
        }
        // `last(v)` is the index of the last element: the array origin plus the length less one.
        if flat == "last" && n.sub.is_none() && !self.funcs.contains_key(&flat) && args.len() == 1 {
            let o = self.origin - 1;
            let len = format!("len({})", rendered[0]);
            return match o {
                0 => atom(len),
                o if o > 0 => R {
                    s: format!("{len} + {o}"),
                    prec: P_ADD,
                },
                o => R {
                    s: format!("{len} - {}", -o),
                    prec: P_ADD,
                },
            };
        }
        if n.sub.is_none() && flat == "tryCatch" && args.len() == 2 && !self.funcs.contains_key(&flat) {
            return self.try_catch(&args[0], &rendered);
        }
        // `csort(A, j)` orders the rows by column j and `rsort(A, i)` the columns by row i (indexes counted
        // from the array origin); `linterp(vx, vy, x)` has the engine's `linterp(x; vx; vy)` argument order.
        if n.sub.is_none() && !self.funcs.contains_key(&flat) {
            match (flat.as_str(), args.len()) {
                ("csort" | "rsort", 2) => {
                    let at = self.index_expr(&args[1]);
                    let engine = if flat == "csort" { "sort_rows" } else { "sort_cols" };
                    return atom(format!("{engine}({}; {at})", rendered[0]));
                }
                ("nthRoot", 2) => {
                    return atom(format!("root({}; {})", rendered[1], rendered[0]));
                }
                ("linterp", 3) => {
                    // The engine wants vectors. A transposed row (a column matrix) is its first column.
                    let list = |t: &str| {
                        if t.starts_with("transp(") && t.ends_with(')') {
                            format!("col({t}; 1)")
                        } else {
                            t.to_string()
                        }
                    };
                    return atom(format!(
                        "linterp({}; {}; {})",
                        rendered[2],
                        list(&rendered[0]),
                        list(&rendered[1])
                    ));
                }
                _ => {}
            }
        }
        if let Some(&arity) = self.funcs.get(&flat) {
            if arity != args.len() {
                self.issue(Code::ArityMismatch, Some(flat.clone()));
                if self.readable {
                    self.readable_bad = true;
                }
            }
            return atom(format!("{}({})", self.fn_name(&flat), rendered.join("; ")));
        }
        if let Some(root) = self.gap_names.get(&flat).copied() {
            self.cur.push(Pending {
                code: Code::DependsOnGap,
                detail: Some(flat),
                root: Some(root),
            });
            return self.stand_in(&shown_call, "?gap?");
        }
        if tables::is_solve_name(&flat) {
            self.issue(Code::SolveBlock, Some(flat));
            return self.stand_in(&shown_call, "?solve?");
        }
        if n.sub.is_none() {
            if let Some((rule, lo, hi)) = tables::lookup_function(&flat) {
                if args.len() < lo || args.len() > hi {
                    self.issue(Code::ArityMismatch, Some(flat));
                    return self.stand_in(&shown_call, "?call?");
                }
                if rule == FnRule::Submatrix {
                    let mut parts = vec![rendered[0].clone()];
                    for a in &args[1..] {
                        parts.push(self.index_expr(a));
                    }
                    let text = format!("submatrix({})", parts.join("; "));
                    // One column: the source indexes it like a vector, and the engine's vector is a
                    // different thing from a one-column matrix.
                    let one_column = matches!((&args[3], &args[4]), (Expr::Num(a), Expr::Num(b)) if a == b);
                    return atom(if one_column { format!("col({text}; 1)") } else { text });
                }
                // `matcol(M, j)` and `matrow(M, i)` count from the sheet's array origin.
                if matches!(flat.as_str(), "matcol" | "matrow") {
                    let engine = if flat == "matcol" { "col" } else { "row" };
                    let at = self.index_expr(&args[1]);
                    return atom(format!("{engine}({}; {at})", rendered[0]));
                }
                return self.apply_rule(rule, &rendered);
            }
        }
        if n.sub.is_none() && tables::is_builtin_missing(&flat) {
            self.issue(Code::BuiltinMissing, Some(flat));
        } else if self.has_references {
            self.issue(Code::ReferenceFunction, Some(flat));
        } else {
            self.issue(Code::AddinFunction, Some(flat));
        }
        self.stand_in(&shown_call, "?call?")
    }

    /// `try body on error fallback`. The engine has no error trapping, so this is read for the one
    /// failure that arithmetic has, a division by zero (the engine gives infinity there): the body
    /// when every divisor is not zero, the fallback otherwise. A body that could fail in another way
    /// (a function call, an index, a power) is not converted.
    /// The engine's spelling of a worksheet function's name: the engine reads a function name
    /// without regard to case, so `Norm` or `Max` would be taken for its own `norm` and `max`.
    fn fn_name(&self, flat: &str) -> String {
        self.fn_alias
            .get(flat)
            .cloned()
            .unwrap_or_else(|| Self::engine_fn(flat))
    }

    fn engine_fn(flat: &str) -> String {
        if tables::is_engine_function(flat) {
            format!("{flat}_f")
        } else {
            flat.to_string()
        }
    }

    fn apply_rule(&mut self, rule: FnRule, a: &[String]) -> R {
        match rule {
            FnRule::Submatrix => atom(format!("submatrix({})", a.join("; "))),
            FnRule::Direct(name) => atom(format!("{}({})", name, a.join("; "))),
            FnRule::Trig(name) => {
                self.uses_trig = true;
                atom(format!("{}({})", name, a.join("; ")))
            }
            FnRule::Log => {
                if a.len() == 1 {
                    atom(format!("log({})", a[0]))
                } else {
                    self.issue(Code::LogBase, None);
                    atom(format!("(ln({})/ln({}))", a[0], a[1]))
                }
            }
            FnRule::Multiple(name) => {
                // Rounding to a multiple of the second argument (a unit, as a rule): the ratio is a plain
                // number, and the multiple has the unit of the second argument.
                self.issue(
                    Code::RoundDigits,
                    Some("rounded to a multiple of the second argument".into()),
                );
                atom(format!("({name}(({a})/({b}))*({b}))", a = a[0], b = a[1]))
            }
            FnRule::Round => {
                if a.len() == 1 {
                    atom(format!("round({})", a[0]))
                } else {
                    self.issue(Code::RoundDigits, None);
                    atom(format!("(round(({})*10^({}))/10^({}))", a[0], a[1], a[1]))
                }
            }
        }
    }

    /// An array index shifted from the sheet's origin to Calcpad's 1.
    fn index_expr(&mut self, e: &Expr) -> String {
        // The author's own index is shown as written; only the computed form is shifted to the engine's origin.
        let delta = if self.display { 0 } else { 1 - self.origin };
        if delta != 0 {
            self.issue(Code::ArrayOrigin, None);
        }
        if let Expr::Num(s) = e {
            if let Some(v) = is_int_literal(s) {
                return (v + delta).to_string();
            }
        }
        let r = self.operand(e, P_ADD, true);
        match delta {
            0 => r,
            d if d > 0 => format!("{} + {}", r, d),
            d => format!("{} - {}", r, -d),
        }
    }

    fn index(&mut self, base: &Expr, idx: &[Expr]) -> R {
        if self.readable {
            // The engine checks a typeset line against what it knows: indexing a name it holds as
            // a number, or the result of a call, is an error even in a `#noc` block.
            match base {
                Expr::Name(n) => {
                    let f = tables::normalise_name(&n.flat());
                    if self.defined.contains(&f) && !self.arrays.contains(&f) {
                        self.readable_bad = true;
                    }
                }
                _ => self.readable_bad = true,
            }
        }
        if let (Some(pp), Expr::Name(_), [i], false, false) =
            (self.plot_param.clone(), base, idx, self.readable, self.display)
        {
            if mentions(i, &pp) {
                // Plotting over the index: a fractional parameter reads between two elements.
                let at = self.index_expr(i);
                let arr = self.operand(base, P_ATOM, true);
                return atom(format!("line({}; {})", at, arr));
            }
        }
        // An element of an element: row and column of the matrix that holds the array of arrays.
        if let (Expr::Index(inner, outer), [j], false) = (base, idx, self.display) {
            if let (Expr::Name(n), [i]) = (&**inner, outer.as_slice()) {
                if self.nested.contains(&tables::normalise_name(&n.flat())) {
                    let m = self.operand(inner, P_ATOM, true);
                    let (a, b) = (self.index_expr(i), self.index_expr(j));
                    return atom(format!("{m}.({a}; {b})"));
                }
            }
        }
        if let (Expr::Name(n), [_], false) = (base, idx, self.display) {
            if self.nested.contains(&tables::normalise_name(&n.flat())) {
                self.issue(
                    Code::ElementAssignment,
                    Some("an element of an array of arrays read as a whole".into()),
                );
                return self.stand_in(&n.flat(), "?nested?");
            }
        }
        let b = self.operand(base, P_ATOM, true);
        if idx.is_empty() || idx.len() > 2 {
            self.issue(Code::BadStructure, Some("index list".into()));
            return atom("?index?");
        }
        let parts: Vec<String> = idx.iter().map(|i| self.index_expr(i)).collect();
        // Only a name can be followed by `.(i)`; the element of anything else (the result of a call,
        // a negated or scaled array) is `take(i; array)`, which needs no temporary.
        if !self.readable && !matches!(base, Expr::Name(_)) {
            return atom(format!("take({}; {})", parts.join("; "), b));
        }
        atom(format!("{}.({})", b, parts.join("; ")))
    }

    fn matrix(&mut self, rows: usize, cols: usize, items: &[Expr]) -> R {
        // The file stores items column by column.
        let cell: Vec<String> = items
            .iter()
            .map(|i| match i {
                Expr::Str(s) if self.str_mat_ok && !self.readable && !self.display => self.str_code(s).to_string(),
                _ => self.expr(i).s,
            })
            .collect();
        let at = |r: usize, c: usize| cell[c * rows + r].clone();
        if rows * cols > 1_000_000 {
            self.issue(Code::BadStructure, Some("matrix too large".into()));
            return atom("?matrix?");
        }
        if cols == 1 && rows == 1 {
            return atom(at(0, 0));
        }
        if cols == 1 {
            let v: Vec<String> = (0..rows).map(|r| at(r, 0)).collect();
            // Displayed, a column has one item per row (`|` starts a row).
            let sep = if self.display { " | " } else { "; " };
            return atom(format!("[{}]", v.join(sep)));
        }
        if rows == 1 {
            let v: Vec<String> = (0..cols).map(|c| at(0, c)).collect();
            if self.display {
                // Displayed, a row is the items side by side.
                return atom(format!("[{}]", v.join("; ")));
            }
            return atom(format!("transp([{}])", v.join("|")));
        }
        let body: Vec<String> = (0..rows)
            .map(|r| (0..cols).map(|c| at(r, c)).collect::<Vec<_>>().join("; "))
            .collect();
        atom(format!("[{}]", body.join("|")))
    }

    // ---- regions ---------------------------------------------------------------------------

    fn flush_issues(&mut self, id: &str, line: u32, severity_filter: Option<Severity>) {
        let pend = std::mem::take(&mut self.cur);
        // A name used many times in one region (a long program) is reported once.
        let mut seen: HashSet<(Code, Option<String>)> = HashSet::new();
        for p in pend {
            if !seen.insert((p.code, p.detail.clone())) {
                continue;
            }
            let sev = p.code.severity();
            if let Some(f) = severity_filter {
                if sev != f {
                    continue;
                }
            }
            self.report.issues.push(Issue {
                code: p.code,
                severity: sev,
                detail: p.detail,
                region_id: Some(id.to_string()),
                line: if line > 0 { Some(line) } else { None },
                reason: p.code.reason().to_string(),
                root: p.root,
            });
        }
    }

    fn bump(map: &mut BTreeMap<String, KindStats>, key: &str, converted: bool, notes: bool) {
        Self::bump_p(map, key, converted, notes, false)
    }

    fn bump_p(map: &mut BTreeMap<String, KindStats>, key: &str, converted: bool, notes: bool, pictured: bool) {
        let s = map.entry(key.to_string()).or_default();
        s.total += 1;
        if converted {
            s.converted += 1;
            if notes {
                s.with_notes += 1;
            }
        } else {
            s.attention += 1;
            if pictured {
                s.pictured += 1;
            }
        }
    }

    /// The output for a region that did not convert. The machine-readable marker is an HTML
    /// comment (with the full reason) followed by a small muted note; the original maths, when it
    /// can be written, follows in a `#noc` block, so it shows as a typeset equation and is never
    /// evaluated. Lines that cannot be Calcpad maths are shown as code text.
    fn placeholder(&mut self, id: &str, codes: &[Code], body: Option<&[GapLine]>) {
        let mut names: Vec<&str> = Vec::new();
        for c in codes {
            if !names.contains(&c.as_str()) {
                names.push(c.as_str());
            }
        }
        let reason = escape_text(codes[0].reason()).replace("--", "- -");
        // The codes are for the report and the hover text; the page shows a short, muted note.
        let note = match body {
            Some(_) => "not calculated".to_string(),
            None => format!("{} (not converted)", escape_text(codes[0].reason())),
        };
        self.lines.push(format!(
            "'<!-- {} R{} {}] {} --><small class=\"imp-gap\" style=\"font-size:8pt;font-style:italic;opacity:.55\" title=\"{}: {}\">{}</small>",
            GAP_MARKER,
            sanitize_id(id),
            names.join(","),
            reason,
            names.join(", "),
            reason.replace('"', "&quot;"),
            note
        ));
        let Some(lines) = body else { return };
        if let Some(ml) = self.pending_ml.take() {
            self.lines.push(ml);
            return;
        }
        let mut in_noc = false;
        for (eq, text) in lines {
            let one_line: String = text.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
            if *eq {
                if !in_noc {
                    if let Some(l) = self.format_line(None) {
                        self.lines.push(l);
                    }
                    self.lines.push("#noc".to_string());
                    in_noc = true;
                }
                self.lines.push(one_line);
            } else {
                if in_noc {
                    self.lines.push("#end noc".to_string());
                    in_noc = false;
                }
                // Source that is not maths is not shown: the reader sees the note, never code.
            }
        }
        if in_noc {
            self.lines.push("#end noc".to_string());
        }
    }

    /// Registers a binary asset and returns its library path, or `None` over the size budget.
    fn add_asset(&mut self, id: &str, ext: &str, mime: &'static str, bytes: Vec<u8>) -> Option<String> {
        if self.asset_total.saturating_add(bytes.len()) > self.opts.max_total_asset_bytes {
            return None;
        }
        let base = format!("{}/R{}", self.opts.asset_prefix, sanitize_id(id));
        let mut path = format!("{}.{}", base, ext);
        let mut n = 2;
        while self.used_paths.contains(&path) {
            path = format!("{}-{}.{}", base, n, ext);
            n += 1;
        }
        self.used_paths.insert(path.clone());
        self.asset_total += bytes.len();
        self.assets.push(Asset {
            path: path.clone(),
            mime,
            bytes,
        });
        Some(path)
    }

    fn img_line(path: &str, r: &Region) -> String {
        let style = if r.width > 0.0 {
            format!("width:{:.1}pt;max-width:100%;", r.width.min(900.0))
        } else {
            "max-width:100%;".to_string()
        };
        format!(
            "'<img style=\"{}\" src=\"{}\" alt=\"R{}\">",
            style,
            path,
            sanitize_id(&r.id)
        )
    }

    /// Output for a region that did not convert: a static picture of the original when the option is
    /// on and the file has one, otherwise the placeholder line (plus a labelled saved result when
    /// that option is on). Returns true when a picture was written.
    fn show_gap(&mut self, r: &Region, codes: &[Code], body: Option<&[GapLine]>, saved: Option<&SavedResult>) -> bool {
        if saved.is_some() {
            self.report.pictures.saved_result_available += 1;
        }
        if self.opts.pictures {
            if let Some(png) = &r.rendering {
                let len = png.len();
                if let Some(path) = self.add_asset(&r.id, "png", "image/png", png.clone()) {
                    // One line, so the caption stays under its picture.
                    self.lines.push(format!(
                        "{}<br/><i>{} (R{} {})</i>",
                        Self::img_line(&path, r),
                        picture::PICTURE_CAPTION,
                        sanitize_id(&r.id),
                        codes[0].as_str()
                    ));
                    self.report.pictures.fallback_shown += 1;
                    self.report.pictures.png_bytes += len;
                    self.issue(Code::StaticPicture, None);
                    return true;
                }
            }
        }
        self.placeholder(&r.id, codes, body);
        if self.opts.saved_results {
            if let Some(sr) = saved {
                let unit = sr.unit.as_ref().map(|u| format!(" {}", u)).unwrap_or_default();
                self.lines.push(format!(
                    "'<i>saved result from the original, not live (R{}): {}{}</i>",
                    sanitize_id(&r.id),
                    escape_text(&sr.value),
                    escape_text(&unit)
                ));
                self.report.pictures.saved_result_emitted += 1;
            }
        }
        false
    }

    fn simple_gap(&mut self, key: &str, r: &Region, code: Code, detail: Option<String>) {
        self.issue(code, detail);
        let pictured = self.show_gap(r, &[code], None, None);
        self.flush_issues(&r.id, r.line, None);
        Self::bump_p(&mut self.report.kinds, key, false, false, pictured);
    }

    fn picture(&mut self, r: &Region, p: &Picture) {
        if !self.opts.pictures {
            self.simple_gap("image", r, Code::Image, None);
            return;
        }
        let mut note_fallback = false;
        let metafile = if self.opts.convert_metafiles {
            p.metafile.as_ref()
        } else {
            None
        };
        if let Some(mf) = metafile {
            match picture::metafile_to_svg(mf, self.opts.svg_limits) {
                Ok(svg) => {
                    let len = svg.len();
                    if let Some(path) = self.add_asset(&r.id, "svg", "image/svg+xml", svg.into_bytes()) {
                        self.lines.push(Self::img_line(&path, r));
                        self.report.pictures.svg_ok += 1;
                        self.report.pictures.svg_bytes += len;
                        Self::bump(&mut self.report.kinds, "image", true, false);
                        return;
                    }
                    note_fallback = true;
                }
                Err(e) => {
                    note_fallback = true;
                    let why = match e {
                        SvgError::InputTooLarge => "input too large",
                        SvgError::Failed => "conversion failed",
                        SvgError::Aborted => "conversion aborted",
                        SvgError::OutputTooLarge => "output too large",
                        SvgError::Unsafe => "unsafe content",
                    };
                    self.cur.push(Pending {
                        code: Code::SvgFallback,
                        detail: Some(why.to_string()),
                        root: None,
                    });
                }
            }
            self.report.pictures.svg_failed += 1;
        }
        let png = p.png.clone().or_else(|| r.rendering.clone());
        if let Some(png) = png {
            let len = png.len();
            if let Some(path) = self.add_asset(&r.id, "png", "image/png", png) {
                self.lines.push(Self::img_line(&path, r));
                self.report.pictures.png_used += 1;
                self.report.pictures.png_bytes += len;
                if note_fallback && self.cur.is_empty() {
                    self.issue(Code::SvgFallback, None);
                }
                self.flush_issues(&r.id, r.line, None);
                Self::bump_p(&mut self.report.kinds, "image", true, true, false);
                return;
            }
        }
        // Neither usable: the placeholder.
        self.cur.retain(|p| p.code != Code::SvgFallback);
        self.simple_gap("image", r, Code::Image, None);
    }

    fn region(&mut self, r: &Region) {
        match &r.kind {
            RegionKind::PageBreak => self.report.page_breaks += 1,
            RegionKind::Text(paras) => {
                for p in paras {
                    if let Some(l) = text_line(p, &self.text_styles, &mut self.colors) {
                        self.lines.push(l);
                    }
                }
                Self::bump(&mut self.report.kinds, "text", true, false);
            }
            RegionKind::Math(m) => self.math(r, m),
            RegionKind::Plot(p) => self.plot(r, p),
            RegionKind::Picture(p) => self.picture(r, p),
            RegionKind::Ole => self.simple_gap("ole", r, Code::OleObject, None),
            RegionKind::Component => self.simple_gap("component", r, Code::Component, None),
            RegionKind::Reference => self.simple_gap("reference", r, Code::WorksheetReference, None),
            RegionKind::Other(tag) if matches!(tag.as_str(), "header" | "footer") => {
                self.simple_gap("other", r, Code::HeaderFooter, Some(tag.clone()))
            }
            RegionKind::Other(tag) if tag == "textUnreadable" => self.simple_gap("text", r, Code::TextUnreadable, None),
            RegionKind::Other(tag) if tag.starts_with("xaml") => {
                self.simple_gap("other", r, Code::RichTextElement, Some(tag[4..].to_string()))
            }
            RegionKind::Other(tag) => {
                let d = Some(
                    tag.chars()
                        .filter(|c| c.is_ascii_alphanumeric())
                        .take(32)
                        .collect::<String>(),
                );
                self.simple_gap("other", r, Code::OtherRegion, d)
            }
        }
    }

    /// The lines a converted math region puts on the page. A plain definition is computed out of
    /// sight and shows the author's equation only (a definition shows no result in the source). An
    /// evaluation shows the expression and its result at the precision the file used; when the
    /// engine would show a renamed variable, the author's expression is typeset first.
    fn present(
        &mut self,
        m: &MathRegion,
        shape: &str,
        plain_def: bool,
        elem_def: bool,
        multi_def: bool,
        line: String,
    ) -> String {
        // A range definition is only the loop of the element assignments that follow; the source
        // shows it as an equation, so it is typeset here.
        if line.starts_with("'<!-- range ") {
            if let Stmt::Define { target, value, global } = &m.stmt {
                if let Some(typeset) = self.typeset_define(target, value, *global) {
                    return format!("{line}\n{typeset}");
                }
            }
            return line;
        }
        if multi_def {
            if let Stmt::Define { target, value, global } = &m.stmt {
                if let Some(typeset) = self.typeset_define(target, value, *global) {
                    return format!("#hide\n{line}\n#end hide\n{typeset}");
                }
            }
            return line;
        }
        if plain_def {
            let line = self.lazy_if(line);
            if let Stmt::Define { target, value, global } = &m.stmt {
                if let Some(shown) = self.display_define(target, value) {
                    if let Some(typeset) = self.typeset_define(target, value, *global) {
                        return format!("#hide\n{line}\n#end hide\n{typeset}");
                    }
                    let fmt = self.format_line(None).map(|l| format!("{l}\n")).unwrap_or_default();
                    return format!("{fmt}#hide\n{line}\n#end hide\n#noc\n{shown}\n#end noc");
                }
                // A program that became one expression: the engine's echo of that expression would
                // evaluate every branch of its `if`s to show the substituted values (and fail on the
                // ones that do not apply), so only the value of the name is shown.
                if let (Target::Var(n), Expr::Program(_), false) = (target, value, line.contains('\n')) {
                    return format!("#hide\n{line}\n#end hide\n{}", tables::normalise_name(&n.flat()));
                }
            }
            return line;
        }
        if elem_def {
            if let Stmt::Define {
                target: target @ Target::Element { base: Some(b), index },
                value,
                global,
            } = &m.stmt
            {
                if let Some(shown) = self.display_element(b, index, value) {
                    // The whole computation goes out of sight, the author's equation is shown.
                    let hidden = match line.split_once("\n#show\n") {
                        Some((alloc, assign)) if line.starts_with("#hide\n") => {
                            format!("{alloc}\n#show\n#hide\n{assign}\n#end hide")
                        }
                        _ => line,
                    };
                    if let Some(typeset) = self.typeset_define(target, value, *global) {
                        return format!("{hidden}\n{typeset}");
                    }
                    let fmt = self.format_line(None).map(|l| format!("{l}\n")).unwrap_or_default();
                    return format!("{fmt}{hidden}\n#noc\n{shown}\n#end noc");
                }
            }
            return line;
        }
        if shape != "evaluation" {
            return line;
        }
        let mut out = String::new();
        // The file's decimal places (three when it says nothing, the tool's own default), as a number
        // format: fixed decimals without trailing zeros. Display only: every value is computed in full.
        let places = m.precision.or(self.default_precision).unwrap_or(3);
        let fmt = if places == 0 {
            "0".to_string()
        } else {
            format!("0.{}", "#".repeat(places as usize))
        };
        let mut scroll = false;
        // An expression of a range variable: the loop that collects the values stays out of sight,
        // and the author's left side is shown with the collected column as its value.
        if let Stmt::Eval { expr, .. } = &m.stmt {
            if self.table_range(expr).is_some() {
                if let Some((hide, last)) = line.rsplit_once('\n').filter(|(h, _)| h.ends_with("#end hide")) {
                    if let Some(head) = self.typeset_eval_head(expr) {
                        self.uses_val_ml = true;
                        return format!(
                            "{hide}\n#round {places}\n'<div class=\"imp-eq\"><math class=\"imp-m\">{head}\n#val\n{last}\n#end val\n'</math></div>\n#round 12"
                        );
                    }
                }
            }
        }
        if let Stmt::Eval { expr, unit } = &m.stmt {
            let shown = self.display_eval(expr, unit.as_ref());
            // The author's left side typeset, and the engine's own value after it (with its unit once).
            if let (Some(_), false) = (&shown, line.contains('\n')) {
                if let Some(head) = self.typeset_eval_head(expr) {
                    self.uses_val_ml = true;
                    return format!(
                        "#round {places}\n'<div class=\"imp-eq\"><math class=\"imp-m\">{head}\n#val\n{line}\n#end val\n'</math></div>\n#round 12"
                    );
                }
            }
            if let Some((shown, renamed)) = shown {
                if renamed {
                    if let Some(l) = self.format_line(None) {
                        out.push_str(&format!("{l}\n"));
                    }
                    out.push_str(&format!("#noc\n{shown}\n#end noc\n"));
                }
            }
            // A matrix result is one row of the page in the source, and scrolls there when long.
            scroll = self.yields_array(expr);
        }
        if let Some(l) = self.format_line(Some(fmt)) {
            out.push_str(&format!("{l}\n"));
        }
        if scroll {
            out.push_str(&format!("'<div class=\"imp-scroll\">\n{line}\n'</div>"));
        } else {
            out.push_str(&line);
        }
        out
    }

    /// A definition on one line, `name = ... if(...) ...`, whose branches call a function or index an array:
    /// the engine evaluates both branches of an `if` at the top of a sheet when it can fold the
    /// condition, and a branch the source never takes can fail. Inside an expression block the `if` is
    /// evaluated lazily, so the right side goes into one.
    fn lazy_if(&self, line: String) -> String {
        if line.contains('\n') || !line.contains("if(") || line.contains("$Block{") {
            return line;
        }
        let Some((name, rhs)) = line.split_once(" = ") else {
            return line;
        };
        let calls_user = self.funcs.keys().any(|f| rhs.contains(&format!("{f}(")));
        if !(calls_user || rhs.contains(".(")) || !valid_ident(name.trim()) {
            return line;
        }
        format!("{} = $Block{{{}}}", name.trim(), rhs)
    }

    /// Whether an identifier is read as a unit label where the equation sits.
    fn is_unit_name(&self, n: &Name) -> bool {
        if n.sub.is_some() || tables::map_unit(&n.base).is_none() {
            return false;
        }
        if n.unit {
            return true;
        }
        if n.var {
            return false;
        }
        // A one-letter name the file does not label as a unit is a variable (an unknown of a
        // solve block); longer unlabelled names such as `ft` are units where nothing defines them.
        if n.base.chars().count() == 1 {
            return false;
        }
        let flat = tables::normalise_name(&n.flat());
        !(self.defined.contains(&flat)
            || self.params.contains(&flat)
            || self.funcs.contains_key(&flat)
            || self.ranges.contains_key(&flat)
            || self.locals.contains_key(&flat)
            || self.gap_names.contains_key(&flat))
    }

    /// A definition as MathML, as an output line; `None` when the option is off or the equation has a
    /// part this module does not typeset.
    fn typeset_define(&self, target: &Target, value: &Expr, global: bool) -> Option<String> {
        if !self.opts.mathml {
            return None;
        }
        let is_unit = |n: &Name| self.is_unit_name(n);
        let is_user_fn = |n: &Name| self.funcs.contains_key(&tables::normalise_name(&n.flat()));
        let original = |b: &str| self.renamed.get(b).cloned();
        let ml = MathMl {
            is_unit: &is_unit,
            is_user_fn: &is_user_fn,
            original: &original,
        };
        ml.define(target, value, global).map(|m| mathml::line(&m))
    }

    /// An unconverted region's statement as MathML (a constraint, a check, an evaluation or a
    /// definition that the engine could not run), as an output line.
    fn typeset_gap(&self, stmt: &Stmt) -> Option<String> {
        if !self.opts.mathml {
            return None;
        }
        let is_unit = |n: &Name| self.is_unit_name(n);
        let is_user_fn = |n: &Name| self.funcs.contains_key(&tables::normalise_name(&n.flat()));
        let original = |b: &str| self.renamed.get(b).cloned();
        let ml = MathMl {
            is_unit: &is_unit,
            is_user_fn: &is_user_fn,
            original: &original,
        };
        let m = match stmt {
            Stmt::Bare(e) => ml.bare(e)?,
            // The result was not computed: a dash shows that, where an empty `x =` reads as a broken line.
            Stmt::Eval { expr, .. } => mathml::wrap(&format!(
                "{}<mtext class=\"imp-nv\" style=\"opacity:.55\">\u{2009}\u{2014}</mtext>",
                ml.eval_head(expr)?
            )),
            Stmt::Define { target, value, global } => ml.define(target, value, *global)?,
            _ => return None,
        };
        Some(mathml::line(&m))
    }

    /// The left side of an evaluation (`expr =`) as MathML.
    fn typeset_eval_head(&self, expr: &Expr) -> Option<String> {
        if !self.opts.mathml {
            return None;
        }
        let is_unit = |n: &Name| self.is_unit_name(n);
        let is_user_fn = |n: &Name| self.funcs.contains_key(&tables::normalise_name(&n.flat()));
        let original = |b: &str| self.renamed.get(b).cloned();
        let ml = MathMl {
            is_unit: &is_unit,
            is_user_fn: &is_user_fn,
            original: &original,
        };
        ml.eval_head(expr)
    }

    fn math(&mut self, r: &Region, m: &MathRegion) {
        self.cur.clear();
        self.params.clear();
        self.depth = 0;
        if m.disabled {
            self.simple_gap("math", r, Code::DisabledRegion, None);
            Self::bump(&mut self.report.math_shapes, "other", false, false);
            return;
        }
        let mut emitted: Option<String> = None;
        let mut attempt: Option<String> = None;
        let mut define_name: Option<(String, Option<usize>, bool)> = None; // name, arity, is array
                                                                           // Names a gap redefines without our knowing their value (they must not read as stale).
        let mut poison: Vec<String> = Vec::new();
        let mut origin_assign: Option<i64> = None;
        let mut range_def: Option<(String, RangeInfo)> = None;
        let mut multi_names: Vec<String> = Vec::new();
        let mut plain_def = false;
        let mut elem_def = false;
        // What a definition does to the texts the sheet holds: a name now holding one (with its
        // codes) or no longer holding one, and a function that returns one.
        let mut str_var_def: Option<(String, Option<Vec<usize>>)> = None;
        let mut str_fn_def: Option<(String, Vec<usize>)> = None;
        let mut fn_gives_array = false;
        let mut str_mat_def: Option<String> = None;
        let mut matrix_def = false;
        let mut str_multi_def: Vec<(String, Option<Vec<usize>>)> = Vec::new();
        // An evaluation of a text: the expression that gives its code and the texts it can be.
        let mut str_eval: Option<(String, Vec<usize>)> = None;
        let shape: &str;

        // A definition with a complex value is two definitions (the real and the imaginary part).
        let cx_result = match &m.stmt {
            Stmt::Define {
                target: Target::Var(n),
                value,
                ..
            } if self.cx_any => self.complex_define(n, value),
            _ => None,
        };

        match &m.stmt {
            // The solver of a solve block: the unknowns become the solution of its equations.
            Stmt::Define { target, value, .. } if m.solve.as_deref() == Some("solver") => {
                shape = "definition";
                match self.solve_block(target, value) {
                    Ok((text, names)) => {
                        let gap = self.cur.iter().any(|p| p.code.severity() == Severity::Gap);
                        if !gap {
                            emitted = Some(text);
                        } else {
                            attempt = Some(text);
                        }
                        if let [one] = names.as_slice() {
                            plain_def = !gap;
                            define_name = Some((one.clone(), None, false));
                        } else {
                            multi_names = if gap { Vec::new() } else { names.clone() };
                            poison.extend(names);
                        }
                    }
                    Err(why) => {
                        self.issue(Code::SolveBlock, Some(why));
                        match target {
                            Target::Var(n) => {
                                define_name = Some((tables::normalise_name(&n.flat()), None, false));
                            }
                            Target::Multi(ns) => {
                                poison.extend(ns.iter().map(|n| tables::normalise_name(&n.flat())));
                            }
                            _ => {}
                        }
                    }
                }
            }
            // An equation of a solve block: shown, collected for the solver that follows.
            Stmt::Bare(e) if m.solve.as_deref() == Some("constraint") => {
                shape = "expression";
                self.solve_cons.push(e.clone());
                emitted = Some(match self.typeset_gap(&m.stmt) {
                    Some(t) => t,
                    None => {
                        let saved = std::mem::take(&mut self.cur);
                        self.readable = true;
                        let text = self.expr(e).s.replace(" ≡ ", " = ");
                        self.readable = false;
                        self.cur = saved;
                        format!("#noc\n{text}\n#end noc")
                    }
                });
            }
            Stmt::Define {
                target: Target::Var(n), ..
            } if cx_result.is_some() => {
                shape = "definition";
                let flat = tables::normalise_name(&n.flat());
                match cx_result {
                    Some(Ok((text, names))) => {
                        matrix_def = self.cx_matrix;
                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                            emitted = Some(text);
                            multi_names = names;
                        } else {
                            attempt = Some(text);
                        }
                    }
                    Some(Err(why)) => {
                        self.issue(Code::UnknownElement, Some(format!("complex number: {why}")));
                        define_name = Some((flat, None, false));
                    }
                    None => {}
                }
            }
            Stmt::Define { target, value, global } => {
                shape = if matches!(target, Target::Func(..)) {
                    "function"
                } else {
                    "definition"
                };
                // A program that is only local definitions and a final value is that value with the
                // locals substituted. Other programs are statements, possible for a variable only.
                let inlined: Option<Expr>;
                let mut statements: Option<&[PStmt]> = None;
                let mut fn_program: Option<&[PStmt]> = None;
                let stub: Expr;
                let value: &Expr = match value {
                    Expr::Program(stmts) => match symexec_program(stmts, matches!(target, Target::Func(..)), &{
                        let mut known = self.defined.clone();
                        known.extend(self.ranges.keys().cloned());
                        if let Target::Func(_, ps) = target {
                            known.extend(ps.iter().map(|p| tables::normalise_name(&p.flat())));
                        }
                        known
                    }) {
                        Ok((e, uncovered)) => {
                            if uncovered {
                                self.issue(Code::UncoveredPath, None);
                            }
                            inlined = Some(e);
                            inlined.as_ref().unwrap()
                        }
                        Err(_) if matches!(target, Target::Var(_)) => {
                            statements = Some(stmts);
                            value
                        }
                        Err(why) => {
                            // Not one expression: the statements become an expression block.
                            fn_program = Some(stmts);
                            stub = Expr::Unsupported {
                                code: Code::ProgramBlock,
                                detail: Some(format!("in a function body: {why}")),
                            };
                            &stub
                        }
                    },
                    other => other,
                };
                match target {
                    Target::Var(n) if statements.is_some() => {
                        let flat = tables::normalise_name(&n.flat());
                        if !valid_ident(&flat) {
                            self.issue(Code::InvalidName, Some(flat.clone()));
                        }
                        match self.lower_program(statements.unwrap_or_default(), &flat) {
                            Ok((text, is_array)) => {
                                matrix_def = returns_matrix(statements.unwrap_or_default());
                                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                    emitted = Some(text);
                                } else {
                                    attempt = Some(text);
                                }
                                define_name = Some((flat, None, is_array));
                            }
                            Err(why) => {
                                self.issue(Code::ProgramBlock, Some(why));
                                define_name = Some((flat, None, false));
                            }
                        }
                        if *global {
                            self.issue(Code::GlobalDefinition, None);
                        }
                    }
                    Target::Var(n) => {
                        let flat = tables::normalise_name(&n.flat());
                        if !valid_ident(&flat) {
                            self.issue(Code::InvalidName, Some(flat.clone()));
                        }
                        if flat == "ORIGIN" {
                            if let Expr::Num(s) = value {
                                if let Some(v) = is_int_literal(s) {
                                    origin_assign = Some(v);
                                }
                            }
                        }
                        if let (Expr::Range { first, second, last }, true) = (value, valid_ident(&flat)) {
                            // The bounds are rendered now, as the source reads them now; a range whose
                            // bounds do not convert is a gap like any other region.
                            let mark = self.cur.len();
                            let a = self.expr(first).s;
                            let z = self.expr(last).s;
                            let step = second.as_ref().map(|b| format!("(({}) - ({a}))", self.expr(b).s));
                            let bad = self.cur[mark..].iter().any(|p| p.code.severity() == Severity::Gap);
                            self.cur.truncate(mark);
                            // A step of zero never reaches the end: not a range.
                            let zero_step = match (&**first, second.as_deref()) {
                                (Expr::Num(a), Some(Expr::Num(b))) => {
                                    matches!((a.trim().parse::<f64>(), b.trim().parse::<f64>()), (Ok(x), Ok(y)) if x == y)
                                }
                                _ => false,
                            };
                            if !bad && !zero_step {
                                let lit = match (&**first, &**last, second) {
                                    (Expr::Num(x), Expr::Num(y), None) => {
                                        match (is_int_literal(x), is_int_literal(y)) {
                                            (Some(x), Some(y)) if x <= y => Some((x, y)),
                                            _ => None,
                                        }
                                    }
                                    _ => None,
                                };
                                range_def = Some((flat.clone(), RangeInfo { a, z, lit, step }));
                            }
                        }
                        let text_def = range_def.is_none() && self.is_stringy(value);
                        let text_codes = if text_def { self.str_domain(value) } else { Vec::new() };
                        let labels = matches!(value, Expr::Matrix { items, .. } if items.iter().any(|i| matches!(i, Expr::Str(_))));
                        self.str_mat_ok = labels;
                        let rhs = if range_def.is_some() {
                            atom("")
                        } else {
                            self.str_or_expr(value)
                        };
                        self.str_mat_ok = false;
                        if labels {
                            str_mat_def = Some(flat.clone());
                        }
                        str_var_def = Some((flat.clone(), text_def.then_some(text_codes)));
                        let is_array = self.yields_array(value);
                        matrix_def = literal_matrix(value);
                        if let Some((_, info)) = &range_def {
                            let (a, z) = (&info.a, &info.z);
                            self.issue(Code::RangeLoop, Some(format!("{flat} = {a} .. {z}")));
                            emitted = Some(format!(
                                "'<!-- range {flat} = {a} .. {z}: used as the loop of the element assignments that follow -->"
                            ));
                        } else if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                            let alias = matches!(value, Expr::Name(_)) && self.holds_array(value);
                            emitted = Some(format!("{} = {}{}", flat, rhs.s, if alias { "*1" } else { "" }));
                            plain_def = true;
                        } else {
                            attempt = Some(format!("{} = {}", flat, rhs.s));
                        }
                        self.elem_built.remove(&flat);
                        define_name = Some((flat, None, is_array));
                        if *global {
                            self.issue(Code::GlobalDefinition, None);
                        }
                    }
                    Target::Func(n, params) => {
                        let flat = tables::normalise_name(&n.flat());
                        if !valid_ident(&flat) {
                            self.issue(Code::InvalidName, Some(flat.clone()));
                        }
                        if self.funcs.contains_key(&flat) {
                            // Defined before: this definition is a function of its own for the engine.
                            self.prog_n += 1;
                            let k = self.prog_n;
                            self.fn_alias
                                .insert(flat.clone(), format!("{}_v{k}", Self::engine_fn(&flat)));
                        }
                        let mut plist = Vec::new();
                        for p in params {
                            let pf = tables::normalise_name(&p.flat());
                            if !valid_ident(&pf) {
                                self.issue(Code::InvalidName, Some(pf.clone()));
                            }
                            plist.push(pf);
                        }
                        self.params = plist.clone();
                        fn_gives_array = gives_array(value);
                        let text_fn = fn_program.is_none() && self.is_stringy(value);
                        if text_fn {
                            let d = self.str_domain(value);
                            str_fn_def = Some((flat.clone(), d));
                        }
                        if fn_program.is_none() {
                            self.fn_defs.insert(flat.clone(), (plist.clone(), value.clone()));
                        } else {
                            self.fn_defs.remove(&flat);
                        }
                        let rhs = match fn_program {
                            Some(stmts) => match self.lower_fn_block(stmts) {
                                Ok(t) => atom(t),
                                Err(why) => {
                                    self.issue(Code::ProgramBlock, Some(format!("in a function body: {why}")));
                                    atom("0")
                                }
                            },
                            None => self.str_or_expr(value),
                        };
                        self.params.clear();
                        let head = format!("{}({})", self.fn_name(&flat), plist.join("; "));
                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                            emitted = Some(format!("{} = {}", head, rhs.s));
                            plain_def = true;
                        } else {
                            attempt = Some(format!("{} = {}", head, rhs.s));
                        }
                        define_name = Some((flat, Some(plist.len()), false));
                        if *global {
                            self.issue(Code::GlobalDefinition, None);
                        }
                    }
                    Target::Multi(names) => {
                        let flats: Vec<String> = names.iter().map(|b| tables::normalise_name(&b.flat())).collect();
                        let distinct = flats.iter().enumerate().all(|(k, f)| !flats[..k].contains(f));
                        match value {
                            // `[A, B] := [1, 2]` defines each name from the item in its place.
                            Expr::Matrix { items, .. }
                                if items.len() == flats.len() && distinct && flats.iter().all(|f| valid_ident(f)) =>
                            {
                                let mut out = Vec::new();
                                let mut texts: Vec<(String, Option<Vec<usize>>)> = Vec::new();
                                for (f, it) in flats.iter().zip(items) {
                                    let text = self.is_stringy(it);
                                    let codes = if text { Some(self.str_domain(it)) } else { None };
                                    let rhs = self.str_or_expr(it);
                                    out.push(format!("{f} = {}", rhs.s));
                                    texts.push((f.clone(), codes));
                                }
                                str_multi_def = texts;
                                let text = out.join("\n");
                                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                    emitted = Some(text);
                                    multi_names = flats.clone();
                                } else {
                                    attempt = Some(text);
                                }
                            }
                            _ if fn_program.is_some() && distinct && flats.iter().all(|f| valid_ident(f)) => {
                                match self.multi_from_program(fn_program.unwrap_or_default(), &flats) {
                                    Ok(parts) => {
                                        let text = parts.join("\n");
                                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                            emitted = Some(text);
                                            multi_names = flats.clone();
                                        } else {
                                            attempt = Some(text);
                                        }
                                    }
                                    Err(why) => self.issue(Code::ProgramBlock, Some(why)),
                                }
                            }
                            _ => self.issue(
                                Code::BadStructure,
                                Some("matrix of names as a definition target".into()),
                            ),
                        }
                        poison.extend(
                            names
                                .iter()
                                .map(|b| tables::normalise_name(&b.flat()))
                                .filter(|f| valid_ident(f)),
                        );
                    }
                    Target::Slice { base, index, row } => {
                        let flat = tables::normalise_name(&base.flat());
                        if valid_ident(&flat) {
                            define_name = Some((flat.clone(), None, true));
                        }
                        match self.slice_assign(base, index, *row, value) {
                            Ok((text, labels)) => {
                                if labels {
                                    str_mat_def = Some(flat.clone());
                                }
                                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                    emitted = Some(text);
                                } else {
                                    attempt = Some(text);
                                }
                            }
                            Err(why) => self.issue(Code::ElementAssignment, Some(why)),
                        }
                    }
                    Target::Element { base, index } => {
                        let flat = base
                            .as_ref()
                            .map(|b| tables::normalise_name(&b.flat()))
                            .filter(|f| valid_ident(f));
                        // The array is no longer what an earlier definition said: later uses of its
                        // name must not read the stale value (or, for a name such as `L`, a unit).
                        if let Some(f) = &flat {
                            define_name = Some((f.clone(), None, true));
                        }
                        // `v[i] := expr(i)` (or `M[i, j]` with two range variables) for an array that has
                        // no value yet fills a zero array whose size is the largest index the ranges
                        // reach: a loop over the range, preceded by `vector(n)` or `matrix(m; n)`. The
                        // size is an expression the engine evaluates, so run-time bounds work. Anything
                        // else about an element assignment is reported.
                        let ranges: Option<Vec<(String, RangeInfo)>> = index
                            .iter()
                            .map(|e| match e {
                                Expr::Name(i) => {
                                    let f = tables::normalise_name(&i.flat());
                                    self.ranges.get(&f).map(|b| (f, b.clone()))
                                }
                                _ => None,
                            })
                            .collect();
                        let distinct = ranges.as_ref().is_some_and(|r| {
                            r.iter()
                                .enumerate()
                                .all(|(k, (n, _))| r[..k].iter().all(|(m, _)| m != n))
                        });
                        let why = match (&flat, &ranges) {
                            (None, _) => Some("the target is not a plain indexed name"),
                            (Some(_), None) => Some("the index is not made of range variables"),
                            (Some(_), Some(_)) if self.yields_array(value) => {
                                Some("the value is an array, which one element of a vector cannot hold")
                            }
                            (Some(_), Some(r)) if r.is_empty() || r.len() > 2 || !distinct => {
                                Some("the index is not one or two different range variables")
                            }
                            (Some(f), Some(r))
                                if (self.defined.contains(f)
                                    || self.funcs.contains_key(f)
                                    || self.gap_names.contains_key(f))
                                    && !(self.elem_built.get(f) == Some(&1)
                                        && r.len() == 1
                                        && !self.funcs.contains_key(f)
                                        && !self.gap_names.contains_key(f)
                                        && r[0].1.a == self.origin.to_string()) =>
                            {
                                Some("the array already has a value, so growing or overwriting it element by element is not read")
                            }
                            (Some(f), Some(_)) if mentions(value, f) => Some("the value reads the array it is filling"),
                            _ => None,
                        };
                        let lits: Option<Vec<i64>> = index
                            .iter()
                            .map(|e| match e {
                                Expr::Num(s) => is_int_literal(s),
                                _ => None,
                            })
                            .collect();
                        let fresh = flat.as_ref().is_some_and(|f| {
                            !self.defined.contains(f) && !self.funcs.contains_key(f) && !self.gap_names.contains_key(f)
                        });
                        // One element of a vector that has a value, at an index that is not a loop: the
                        // vector grows (zeros) when the index is past its end, then the element is set.
                        let grow = match (&flat, &ranges) {
                            (Some(f), None)
                                if (index.len() == 1 || index.len() == 2)
                                    && self.arrays.contains(f)
                                    && index.iter().chain(std::iter::once(value)).all(|e| !self.reads_range(e)) =>
                            {
                                Some(f.clone())
                            }
                            _ => None,
                        };
                        if let Some(f) = grow {
                            let idx: Vec<String> = index.iter().map(|e| self.index_expr(e)).collect();
                            let rhs = self.expr(value);
                            let pre = std::mem::take(&mut self.hoisted);
                            let at = idx.join("; ");
                            if !pre.is_empty() {
                                self.issue(
                                    Code::ElementAssignment,
                                    Some("the value indexes the result of a call".to_string()),
                                );
                            } else if self.yields_array(value) {
                                self.issue(
                                    Code::ElementAssignment,
                                    Some(
                                        "the value is an array, which one element of an array cannot hold".to_string(),
                                    ),
                                );
                            } else if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                // The new size is taken from the old one and the index before either changes.
                                let grown = if idx.len() == 1 {
                                    format!("{f} = resize({f}; max(len({f}); {}))", idx[0])
                                } else {
                                    format!(
                                        "{f} = mresize({f}; max(n_rows({f}); {}); max(n_cols({f}); {}))",
                                        idx[0], idx[1]
                                    )
                                };
                                emitted = Some(format!("#hide\n{grown}\n#show\n{f}.({at}) = {}", rhs.s));
                                elem_def = true;
                                self.elem_built.remove(&f);
                            } else {
                                attempt = Some(format!("{f}.({at}) = {}", rhs.s));
                            }
                        } else {
                            let literal = match (&flat, &lits) {
                                (Some(f), Some(l))
                                    if fresh && (l.len() == 1 || l.len() == 2) && !mentions(value, f) =>
                                {
                                    Some((f.clone(), l.clone()))
                                }
                                _ => None,
                            };
                            if let Some((f, l)) = literal {
                                // One element of an array that has no value yet: the array is created
                                // (zeros) just large enough, without being shown, and the element set.
                                let idx: Vec<String> = index.iter().map(|e| self.index_expr(e)).collect();
                                let rhs = self.expr(value);
                                let pre = std::mem::take(&mut self.hoisted);
                                let ok = idx.iter().all(|s| s.parse::<i64>().is_ok_and(|v| v >= 1));
                                if !ok || !pre.is_empty() {
                                    self.issue(
                                        Code::ElementAssignment,
                                        Some("the index is below the array origin".to_string()),
                                    );
                                } else if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                    let alloc = if idx.len() == 1 {
                                        format!("{f} = vector({})", idx[0])
                                    } else {
                                        format!("{f} = matrix({}; {})", idx[0], idx[1])
                                    };
                                    emitted =
                                        Some(format!("#hide\n{alloc}\n#show\n{f}.({}) = {}", idx.join("; "), rhs.s));
                                    elem_def = true;
                                    if idx.len() == 1 {
                                        self.elem_built.insert(f.clone(), idx[0].parse().unwrap_or(0));
                                    }
                                } else {
                                    attempt = Some(format!("{f}.({}) = {}", idx.join("; "), rhs.s));
                                }
                                let _ = l;
                            } else {
                                let plan = if why.is_some() {
                                    self.grow_fill_plan(&flat, index, value)
                                } else {
                                    None
                                };
                                // An array an earlier element assignment could not set is a gap, and
                                // this write depends on it: that is where the cause is, not here.
                                let mut gap_root = flat
                                    .as_ref()
                                    .and_then(|f| self.gap_names.get(f).map(|c| (f.clone(), *c)));
                                if gap_root.is_none() {
                                    // An index that reads a name a gap defines (a range variable whose
                                    // bounds did not convert, a counter) is the same dependency.
                                    for e in index {
                                        e.clone().for_each_name_mut(&mut |n| {
                                            let f = tables::normalise_name(&n.flat());
                                            if gap_root.is_none() {
                                                if let Some(c) = self.gap_names.get(&f) {
                                                    gap_root = Some((f, *c));
                                                }
                                            }
                                        });
                                    }
                                }
                                // An array in each element (`v[k] := f(k)` with a vector value) is a matrix with
                                // one row per element.
                                let nested_plan = match &flat {
                                    Some(f) if index.len() == 1 && self.yields_array(value) => self
                                        .fill_plan(&flat, index, value, true)
                                        .filter(|(_, fresh)| *fresh || self.nested.contains(f)),
                                    _ => None,
                                };
                                match (why, flat, ranges) {
                                    (_, Some(f), _) if nested_plan.is_some() => {
                                        let (rs, fresh) = nested_plan.unwrap_or_default();
                                        for (iv, _) in &rs {
                                            self.params.push(iv.clone());
                                        }
                                        let rhs = self.expr(value);
                                        let idx: Vec<String> = index.iter().map(|e| self.index_expr(e)).collect();
                                        for _ in &rs {
                                            self.params.pop();
                                        }
                                        let pre = std::mem::take(&mut self.hoisted);
                                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                            match self.nested_fill_loops(&f, &rs, &idx[0], &rhs.s, &pre, fresh) {
                                                Ok(t) => {
                                                    self.elem_built.remove(&f);
                                                    self.nested.insert(f.clone());
                                                    emitted = Some(t);
                                                    elem_def = true;
                                                    self.issue(
                                                        Code::RangeLoop,
                                                        Some(format!(
                                                            "element assignment over {}: each element is an array, kept as a row of a matrix",
                                                            rs.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
                                                        )),
                                                    );
                                                }
                                                Err(w) => self.issue(Code::ElementAssignment, Some(w)),
                                            }
                                        } else {
                                            attempt = Some(format!("{f}.({}) = {}", idx.join("; "), rhs.s));
                                        }
                                    }
                                    (Some(_), Some(f), _) if plan.is_some() => {
                                        let (rs, fresh) = plan.unwrap_or_default();
                                        for (iv, _) in &rs {
                                            self.params.push(iv.clone());
                                        }
                                        let rhs = self.expr(value);
                                        let idx: Vec<String> = index.iter().map(|e| self.index_expr(e)).collect();
                                        for _ in &rs {
                                            self.params.pop();
                                        }
                                        let pre = std::mem::take(&mut self.hoisted);
                                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                            match self.grow_fill_loops(&f, &rs, &idx, &rhs.s, &pre, fresh) {
                                                Ok(t) => {
                                                    self.elem_built.remove(&f);
                                                    emitted = Some(t);
                                                    elem_def = true;
                                                    self.issue(
                                                        Code::RangeLoop,
                                                        Some(format!(
                                                            "element assignment over {}: written in a loop, the array grows to the largest index written",
                                                            rs.iter().map(|r| r.0.as_str()).collect::<Vec<_>>().join(", ")
                                                        )),
                                                    );
                                                }
                                                Err(w) => self.issue(Code::ElementAssignment, Some(w)),
                                            }
                                        } else {
                                            attempt = Some(format!("{f}.({}) = {}", idx.join("; "), rhs.s));
                                        }
                                    }
                                    (None, Some(f), Some(rs)) => {
                                        for (iv, _) in &rs {
                                            self.params.push(iv.clone());
                                        }
                                        let rhs = self.expr(value);
                                        let idx: Vec<String> = rs
                                            .iter()
                                            .map(|(iv, _)| self.index_expr(&Expr::Name(Name::new(iv))))
                                            .collect();
                                        for _ in &rs {
                                            self.params.pop();
                                        }
                                        let pre = std::mem::take(&mut self.hoisted);
                                        if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                                            match self.fill_loops(&f, &rs, &idx, &rhs.s, &pre) {
                                                Ok(t) => {
                                                    self.elem_built.remove(&f);
                                                    emitted = Some(t);
                                                    elem_def = true;
                                                }
                                                Err(w) => self.issue(Code::ElementAssignment, Some(w)),
                                            }
                                        } else {
                                            attempt = Some(format!("{f}.({}) = {}", idx.join("; "), rhs.s));
                                        }
                                    }
                                    (Some(_), ..) if gap_root.is_some() => {
                                        if let Some((f, root)) = gap_root {
                                            self.cur.push(Pending {
                                                code: Code::DependsOnGap,
                                                detail: Some(f),
                                                root: Some(root),
                                            });
                                        }
                                    }
                                    (Some(w), ..) => self.issue(Code::ElementAssignment, Some(w.to_string())),
                                    (None, ..) => {}
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Eval { expr, unit } if self.table_range(expr).is_some() => {
                // An expression of a range variable shows one value per step of the range: the
                // values are collected in an array by a loop and the array is shown.
                shape = "evaluation";
                let (iv, info) = self.table_range(expr).unwrap_or_default();
                self.params.push(iv.clone());
                let e = self.expr(expr);
                self.params.pop();
                let pre = std::mem::take(&mut self.hoisted);
                let mut tail = String::new();
                if let Some(u) = unit {
                    let ur = self.expr(u);
                    tail = format!("|{}", ur.s);
                    self.issue(Code::UnitOverride, None);
                }
                self.issue(
                    Code::RangeLoop,
                    Some(format!("{iv} = {} .. {}: shown as a table", info.a, info.z)),
                );
                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                    emitted = Some(self.table_loops(&iv, &info, &e.s, &pre, &tail));
                }
            }
            Stmt::Eval { expr, unit: None } if self.is_stringy(expr) && !self.readable => {
                shape = "evaluation";
                let domain = self.str_domain(expr);
                let code = self.str_or_expr(expr).s;
                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                    str_eval = Some((code.clone(), domain));
                    emitted = Some(code);
                } else {
                    attempt = Some(code);
                }
            }
            Stmt::Eval { expr, unit } => {
                shape = "evaluation";
                if let Expr::Name(n) = expr {
                    // Its labels are held as codes; printing the matrix would print the codes.
                    if self.str_mats.contains(&tables::normalise_name(&n.flat())) {
                        self.issue(
                            Code::StringValue,
                            Some("a matrix of texts and numbers is not printed".into()),
                        );
                    }
                }
                let e = self.expr(expr);
                let mut text = e.s.clone();
                if let Some(u) = unit {
                    let ur = self.expr(u);
                    // A unit that cancels (ft/ft) is a plain number: the engine refuses it as a target.
                    if !unit_cancels(&ur.s) {
                        text = if e.prec >= P_ATOM { e.s } else { format!("({})", e.s) };
                        // A "unit" that is only a number (a power of ten) shows the value in multiples of it.
                        // An angle is a plain number (radians) in the source; shown in degrees it is that
                        // number over pi/180, and the unit it carries is the engine's degree.
                        text = if is_degree(u) {
                            format!("({}/(π/180))*deg", text)
                        } else if numeric_only(u) {
                            format!("{}/({})", text, ur.s)
                        } else if matches!(ur.s.as_str(), "°C" | "°F") {
                            format!("{}|{}", text, ur.s)
                        } else {
                            // The source takes the unit as the part of the result's unit to show first
                            // and keeps the rest (`kip` for a moment shows kip times length); the
                            // engine's `|` refuses a unit of another dimension.
                            let u = if is_whole(&ur.s) {
                                ur.s.clone()
                            } else {
                                format!("({})", ur.s)
                            };
                            format!("({}/{})*{}", text, u, u)
                        };
                        self.issue(Code::UnitOverride, None);
                    }
                }
                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                    emitted = Some(text);
                } else {
                    attempt = Some(text);
                }
            }
            Stmt::Bare(Expr::Str(text)) => {
                // A text on its own is a label the author put in a math region.
                shape = "expression";
                let t: String = text
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(200)
                    .collect::<String>()
                    .replace('\'', "\u{2019}");
                let t = mathml::esc(&t);
                emitted = Some(if self.opts.mathml {
                    format!("'<div class=\"imp-eq\"><math class=\"imp-m\"><ms>{t}</ms></math></div>")
                } else {
                    format!("'<p><i>{t}</i></p>")
                });
            }
            Stmt::Bare(expr) if matches!(expr, Expr::Bin(BinOp::Eq, ..)) => {
                // An equation on its own line is a constraint (a solve block's equation, or a check):
                // it is shown with its real names, not reported name by name.
                shape = "expression";
                self.issue(Code::ConstraintEquation, None);
                let saved = std::mem::take(&mut self.cur);
                self.readable = true;
                let text = self.expr(expr).s.replace(" ≡ ", " = ");
                self.readable = false;
                self.cur = saved;
                attempt = Some(text);
            }
            Stmt::Bare(expr) => {
                shape = "expression";
                let e = self.expr(expr);
                self.issue(Code::BareExpression, None);
                attempt = Some(e.s);
            }
            Stmt::Unsupported { code, detail } => {
                shape = "other";
                self.issue(*code, detail.clone());
            }
        }

        if m.frozen && emitted.is_some() {
            // Data read from the workbook an input component saved.
            self.issue(Code::FrozenValue, None);
            self.report.frozen_values += 1;
            let note = format!(
                "'<small><i>R{}: value taken from the saved workbook of an input component (frozen, not live)</i></small>",
                sanitize_id(&r.id)
            );
            emitted = emitted.map(|e| format!("{note}\n{e}"));
            plain_def = false;
        }

        // Temporaries the line needs first (a call result that is indexed).
        let hoisted = std::mem::take(&mut self.hoisted);
        if !hoisted.is_empty() {
            if matches!(
                &m.stmt,
                Stmt::Define {
                    target: Target::Func(..),
                    ..
                }
            ) {
                self.issue(
                    Code::BadStructure,
                    Some("the result of a call is indexed inside a function body".into()),
                );
            } else if let Some(e) = emitted.take() {
                emitted = Some(format!("{}\n{}", hoisted.join("\n"), e));
            }
        }

        // A definition that only fails because it reads a name an input component defines is given
        // the value the file saved for it, as a literal, and says so.
        let saved_here: Option<Expr> = match &m.stmt {
            Stmt::Define {
                target: Target::Var(n), ..
            } => m
                .saved_expr
                .clone()
                .or_else(|| self.saved_by_name.get(&tables::normalise_name(&n.flat())).cloned()),
            _ => None,
        };
        if let (
            Stmt::Define {
                target: Target::Var(n), ..
            },
            Some(saved),
        ) = (&m.stmt, &saved_here)
        {
            let flat = tables::normalise_name(&n.flat());
            let blockers: Vec<&Pending> = self.cur.iter().filter(|p| p.code.severity() == Severity::Gap).collect();
            // A name an input component defines, or a read of a data file the importer cannot see.
            let only_inputs = !blockers.is_empty()
                && blockers.iter().all(|p| {
                    (p.code == Code::UndefinedVariable
                        && p.detail.as_ref().is_some_and(|d| self.component_names.contains(d)))
                        || (p.code == Code::BuiltinMissing
                            && p.detail
                                .as_deref()
                                .is_some_and(|d| matches!(d, "READEXCEL" | "READPRN" | "vec2str")))
                });
            if only_inputs && valid_ident(&flat) {
                let stash = std::mem::take(&mut self.cur);
                let (saved, empty_cells) = zero_empty_cells(saved);
                let saved = &saved;
                self.str_mat_ok = true;
                let v = self.str_or_expr(saved);
                self.str_mat_ok = false;
                if self.cur.iter().all(|p| p.code.severity() != Severity::Gap) {
                    self.cur.clear();
                    self.issue(Code::FrozenValue, Some(flat.clone()));
                    self.report.frozen_values += 1;
                    let is_array = matches!(saved, Expr::Matrix { .. })
                        || matches!(saved, Expr::Bin(BinOp::Mul, a, _) if matches!(**a, Expr::Matrix { .. }));
                    let cells = if empty_cells > 0 {
                        format!("; {empty_cells} empty cells read as 0")
                    } else {
                        String::new()
                    };
                    emitted = Some(format!(
                        "'<small><i>R{}: value taken from the saved result of an input component or a data file (frozen, not live{cells})</i></small>\n{} = {}",
                        sanitize_id(&r.id),
                        flat,
                        v.s
                    ));
                    attempt = None;
                    plain_def = false;
                    str_var_def = None;
                    str_mat_def = None;
                    if matches!(saved, Expr::Str(_)) {
                        let d = self.str_domain(saved);
                        str_var_def = Some((flat.clone(), Some(d)));
                    } else if matches!(saved, Expr::Matrix { items, .. } if items.iter().any(|i| matches!(i, Expr::Str(_))))
                    {
                        str_mat_def = Some(flat.clone());
                    }
                    define_name = Some((flat, None, is_array));
                } else {
                    self.cur = stash;
                }
            }
        }

        if let Some(c) = &m.control {
            self.issue(Code::ControlInput, Some(c.kind.clone()));
        }
        let gap_codes: Vec<Code> = self
            .cur
            .iter()
            .filter(|p| p.code.severity() == Severity::Gap)
            .map(|p| p.code)
            .collect();
        let has_notes = self.cur.iter().any(|p| p.code.severity() == Severity::Note);
        let converted = gap_codes.is_empty();
        let mut pictured = false;

        if converted {
            if let Some(v) = origin_assign {
                self.origin = v;
                self.issue(Code::OriginAssigned, None);
                self.lines
                    .push(format!("'<!-- ORIGIN assigned {} in the source -->", v));
                // The source shows the definition; the engine has no ORIGIN, so it is a typeset line only.
                if let Stmt::Define { target, value, global } = &m.stmt {
                    if let Some(typeset) = self.typeset_define(target, value, *global) {
                        self.lines.push(typeset);
                    }
                }
            } else if let Some(line) = emitted.take() {
                self.maths_shown = true;
                let multi_def = !multi_names.is_empty();
                let control_note = m.control.as_ref().map(|c| {
                    let pick = match (&c.chosen, c.options) {
                        (Some(t), 0) => format!("stored text \"{t}\""),
                        (Some(t), n) => format!("stored selection \"{t}\" of {n} options"),
                        (None, _) => "stored value".to_string(),
                    };
                    format!(
                        "'<small><i>R{}: {} input, {pick}; the options and the script behind the control are not carried</i></small>",
                        sanitize_id(&r.id),
                        c.kind
                    )
                });
                let text = match &str_eval {
                    Some((code, domain)) => {
                        let head = match &m.stmt {
                            Stmt::Eval { expr, .. } => self.typeset_eval_head(expr),
                            _ => None,
                        };
                        self.str_result_lines(head.as_deref().unwrap_or(""), code, domain)
                    }
                    None => self.present(m, shape, plain_def, elem_def, multi_def, line),
                };
                let text = match control_note {
                    Some(n) => format!("{n}\n{text}"),
                    None => text,
                };
                self.lines.push(text);
            }
            for name in &multi_names {
                if matrix_def {
                    self.matrices.insert(name.clone());
                } else {
                    self.matrices.remove(name);
                }
                self.ranges.remove(name);
                self.gap_names.remove(name);
                self.funcs.remove(name);
                self.arrays.remove(name);
                self.elem_built.remove(name);
                self.defined.insert(name.clone());
            }
            for (n, d) in std::mem::take(&mut str_multi_def) {
                self.str_funcs.remove(&n);
                self.set_str_var(&n, d);
            }
            if let Some((name, info)) = &range_def {
                self.defined.remove(name);
                self.funcs.remove(name);
                self.arrays.remove(name);
                self.gap_names.remove(name);
                self.ranges.insert(name.clone(), info.clone());
            } else if let Some((name, arity, is_array)) = define_name {
                self.ranges.remove(&name);
                self.gap_names.remove(&name);
                self.col_vecs.remove(&name);
                if matches!(&m.stmt, Stmt::Define { target: Target::Var(_) | Target::Func(..), value, .. } if is_col_literal(value))
                {
                    self.col_vecs.insert(name.clone());
                }
                match arity {
                    Some(a) => {
                        self.funcs.insert(name.clone(), a);
                        self.defined.remove(&name);
                        self.array_funcs.remove(&name);
                        if fn_gives_array {
                            self.array_funcs.insert(name.clone());
                        }
                    }
                    None => {
                        if name != "ORIGIN" {
                            self.defined.insert(name.clone());
                        }
                        self.funcs.remove(&name);
                    }
                }
                if is_array {
                    self.arrays.insert(name.clone());
                } else {
                    self.arrays.remove(&name);
                }
                if matrix_def {
                    self.matrices.insert(name.clone());
                } else {
                    self.matrices.remove(&name);
                }
                // Texts: the name holds one now, or no longer does.
                self.str_vars.remove(&name);
                self.str_funcs.remove(&name);
                self.str_mats.remove(&name);
                self.num_texts.remove(&name);
                self.empty_texts.remove(&name);
                if let Some(c) = &m.control {
                    if c.kind == "text box" && matches!(&c.value, Expr::Str(t) if t.trim().is_empty()) {
                        self.empty_texts.insert(name.clone());
                    }
                    if c.kind == "text box" && matches!(c.value, Expr::Num(_)) {
                        self.num_texts.insert(name.clone());
                    }
                }
                if let Some(n) = str_mat_def.take() {
                    self.str_mats.insert(n);
                }
                if let Some((n, d)) = str_var_def.take() {
                    self.set_str_var(&n, d);
                }
                if let Some((n, d)) = str_fn_def.take() {
                    self.str_funcs.insert(n, d);
                }
            }
        } else {
            // The root cause this region passes on to the regions that use its name.
            let root = self
                .cur
                .iter()
                .find(|p| p.code == Code::DependsOnGap)
                .and_then(|p| p.root)
                .or_else(|| gap_codes.iter().copied().find(|c| *c != Code::DependsOnGap))
                .unwrap_or(Code::DependsOnGap);
            for p in poison {
                self.defined.remove(&p);
                self.funcs.remove(&p);
                self.arrays.remove(&p);
                self.gap_names.insert(p, root);
            }
            if let Some((name, arity, _)) = define_name {
                self.str_vars.remove(&name);
                self.str_funcs.remove(&name);
                self.col_vecs.remove(&name);
                self.ranges.remove(&name);
                self.defined.remove(&name);
                self.funcs.remove(&name);
                self.arrays.remove(&name);
                let _ = arity;
                self.gap_names.insert(name, root);
            }
            let mut shown: Option<Vec<GapLine>> = self
                .readable_stmt(&m.stmt)
                .or_else(|| attempt.map(|a| vec![(false, a)]));
            self.pending_ml = self.typeset_gap(&m.stmt);
            if self.pending_ml.is_some() {
                shown = Some(Vec::new());
            }
            pictured = self.show_gap(r, &gap_codes, shown.as_deref(), m.saved_result.as_ref());
        }
        self.flush_issues(&r.id, r.line, None);
        Self::bump_p(&mut self.report.kinds, "math", converted, has_notes, pictured);
        Self::bump_p(&mut self.report.math_shapes, shape, converted, has_notes, pictured);
    }
}

/// A stored table with its empty cells (stored as NaN) as 0, and how many there were.
fn zero_empty_cells(e: &Expr) -> (Expr, usize) {
    match e {
        Expr::Num(s) if s == "NaN" => (Expr::Num("0".into()), 1),
        Expr::Matrix { rows, cols, items } => {
            let mut n = 0;
            let items = items
                .iter()
                .map(|i| {
                    let (x, k) = zero_empty_cells(i);
                    n += k;
                    x
                })
                .collect();
            (
                Expr::Matrix {
                    rows: *rows,
                    cols: *cols,
                    items,
                },
                n,
            )
        }
        Expr::Bin(op, a, b) => {
            let (a, n) = zero_empty_cells(a);
            (Expr::Bin(*op, Box::new(a), b.clone()), n)
        }
        other => (other.clone(), 0),
    }
}

/// True when a function body gives an array: a matrix literal, or an `if` with such a branch.
fn gives_array(e: &Expr) -> bool {
    match e {
        Expr::Matrix { .. } => true,
        Expr::Call(n, args) if n.base == "if" && n.sub.is_none() && args.len() == 3 => {
            gives_array(&args[1]) || gives_array(&args[2])
        }
        _ => false,
    }
}

/// True when the expression text is one operand: a plain unit name or one parenthesised whole.
fn is_whole(s: &str) -> bool {
    if s.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '°' | '%' | '\u{2032}' | '\u{2033}'))
    {
        return true;
    }
    if !(s.starts_with('(') && s.ends_with(')')) {
        return false;
    }
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && i != s.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

/// A font family as a CSS stack: the file's name first, then the metric-compatible and generic fallbacks
/// a browser on any platform has. Only plain name characters are kept.
fn font_stack(name: &str) -> String {
    // No quote characters anywhere: a `'` inside a text line would switch the engine to maths. A family
    // name is written as the plain run of words CSS allows, and one that is not (a word that starts with
    // a digit) is left out.
    let clean: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .collect();
    let clean = if clean
        .split_whitespace()
        .all(|w| !w.starts_with(|c: char| c.is_ascii_digit() || c == '-'))
    {
        clean.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        String::new()
    };
    let clean = clean.as_str();
    let lower = clean.to_ascii_lowercase();
    let fallback = match lower.as_str() {
        "times new roman" | "times" | "cambria" | "georgia" | "garamond" | "book antiqua" | "palatino linotype" => {
            "Times New Roman,Times,Liberation Serif,serif"
        }
        "arial" | "helvetica" | "arial narrow" | "calibri" | "verdana" | "tahoma" | "segoe ui" | "trebuchet ms" => {
            "Arial,Helvetica,Liberation Sans,sans-serif"
        }
        "courier new" | "courier" | "consolas" | "lucida console" => "Courier New,Courier,Liberation Mono,monospace",
        _ => "sans-serif",
    };
    if clean.is_empty() {
        return fallback.to_string();
    }
    let first = clean.to_string();
    if fallback.split(',').next() == Some(first.as_str()) || fallback.contains(&first) {
        fallback.to_string()
    } else {
        format!("{first},{fallback}")
    }
}

fn pt(v: f64) -> String {
    format!("{}", (v * 100.0).round() / 100.0)
}

/// `#rrggbb` as hue, saturation and lightness in 0..=1.
fn to_hsl(hex: &str) -> (f64, f64, f64) {
    let c = |i: usize| f64::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)) / 255.0;
    let (r, g, b) = (c(1), c(3), c(5));
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let l = (mx + mn) / 2.0;
    if (mx - mn).abs() < 1e-9 {
        return (0.0, 0.0, l);
    }
    let d = mx - mn;
    let s = if l > 0.5 { d / (2.0 - mx - mn) } else { d / (mx + mn) };
    let h = if (mx - r).abs() < 1e-9 {
        ((g - b) / d).rem_euclid(6.0)
    } else if (mx - g).abs() < 1e-9 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, s, l)
}

fn from_hsl(h: f64, s: f64, l: f64) -> String {
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let ch = |t: f64| {
        let t = t.rem_euclid(1.0);
        let v = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (v * 255.0).round() as u8
    };
    format!("#{:02x}{:02x}{:02x}", ch(h + 1.0 / 3.0), ch(h), ch(h - 1.0 / 3.0))
}

/// The colour a source colour is shown in on a light page and on a dark one. A dark colour is
/// lightened on the dark page (keeping its hue) and a very light one is darkened on the light page.
/// Black, white and greys carry no hue and are left to the app's own text colour (`None`).
fn theme_colors(hex: &str) -> Option<(String, String)> {
    let (h, s, l) = to_hsl(hex);
    if s < 0.1 && (l < 0.2 || l > 0.9) {
        return None;
    }
    let light = if l > 0.85 {
        from_hsl(h, s, 0.35)
    } else {
        hex.to_string()
    };
    let dark = if l < 0.65 {
        from_hsl(h, s.clamp(0.35, 0.6), 0.72)
    } else {
        hex.to_string()
    };
    Some((light, dark))
}

/// Whether a unit expression is a ratio of one unit to itself (`ft/ft`), which has no units left.
fn unit_cancels(u: &str) -> bool {
    let strip = |x: &str| {
        x.trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim()
            .to_string()
    };
    let mut parts = u.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), None) => !strip(a).is_empty() && strip(a) == strip(b),
        _ => false,
    }
}

/// The class a colour is given; its rules are written once at the top of the output.
fn color_class(hex: &str, colors: &mut std::collections::BTreeSet<String>) -> Option<String> {
    theme_colors(hex)?;
    colors.insert(hex.to_string());
    Some(format!("imp-c-{}", &hex[1..]))
}

fn style_block(colors: &std::collections::BTreeSet<String>, maths: bool) -> Option<String> {
    // The source draws a vector's name plain (no arrow, no bold) and a transpose as a plain
    // superscript; the engine marks array names and powers of arrays, which the page leaves out.
    if colors.is_empty() && !maths {
        return None;
    }
    let mut css = String::new();
    if maths {
        css.push_str(
            ".vec{display:none}b>var{font-weight:normal}sup>small.nth:first-child{display:none}.imp-scroll{overflow-x:auto;white-space:nowrap}\
.imp-eq{display:flex;overflow-x:auto;overflow-y:hidden;scrollbar-width:thin;padding:.35em 0;margin:.15em 0}.imp-eq::before{content:\"\";flex:0 1 var(--ind,0px);min-width:0}.imp-eq>math{flex:none}.imp-m{font-size:1.15em;math-style:normal;font-family:Cambria Math,STIX Two Math,Latin Modern Math,Noto Sans Math,math}\
.imp-m mi.v{font-style:italic}\
.imp-m .kw,.imp-m .dot,.imp-m .beq,.imp-m .ru{font-family:STIX Two Text,Times New Roman,Times,serif}\
.imp-m .kw,.imp-m .dot,.imp-m .beq,.imp-m .ru{font-weight:bold}.imp-m .ru{font-style:italic}\
.imp-m .ru{color:rgb(28,61,153)}.dark-theme .imp-m .ru{color:rgb(143,176,255)}\
.imp-m munderover>:nth-child(2),.imp-m munder>:nth-child(2){margin-top:-.12em}.imp-m munderover>:nth-child(3){margin-bottom:-.12em}\
.imp-m mover.va>mo{visibility:hidden}.imp-m mover.va{background:linear-gradient(currentColor,currentColor) 0 .2em/100% 1.3px no-repeat,\
linear-gradient(to top right,transparent calc(50% - .65px),currentColor calc(50% - .65px) calc(50% + .65px),transparent calc(50% + .65px)) right .08em/.22em .12em no-repeat,\
linear-gradient(to bottom right,transparent calc(50% - .65px),currentColor calc(50% - .65px) calc(50% + .65px),transparent calc(50% + .65px)) right .2em/.22em .12em no-repeat}",
        );
    }
    for c in colors {
        if let Some((light, dark)) = theme_colors(c) {
            let cls = format!(".imp-c-{}", &c[1..]);
            css.push_str(&format!("{cls}{{color:{light}}}.dark-theme {cls}{{color:{dark}}}"));
        }
    }
    Some(format!("'<style>{css}</style>"))
}

/// One output line for a paragraph, or `None` if it holds no text. `styles` are the file's named
/// paragraph styles; the colours used are collected in `colors`.
fn text_line(
    p: &Paragraph,
    styles: &HashMap<String, TextStyle>,
    colors: &mut std::collections::BTreeSet<String>,
) -> Option<String> {
    let st = styles.get(&p.style).cloned().unwrap_or_default();
    let heading = match p.style.as_str() {
        "Heading 1" => Some(1),
        "Heading 2" => Some(2),
        "Heading 3" => Some(3),
        _ => None,
    };
    // Merge runs of equal formatting (whitespace-only runs take the formatting of their
    // neighbour), then collapse repeated spaces across run boundaries.
    enum Seg {
        Text(Run),
        Br,
    }
    let mut segs: Vec<Seg> = Vec::new();
    for piece in &p.pieces {
        match piece {
            Piece::Break => segs.push(Seg::Br),
            Piece::Run(r) => {
                if r.text.is_empty() {
                    continue;
                }
                let mut r = r.clone();
                if r.text.trim().is_empty() {
                    if let Some(Seg::Text(prev)) = segs.last() {
                        r = Run {
                            text: r.text,
                            ..prev.clone()
                        };
                    } else {
                        r = Run {
                            text: r.text,
                            ..Run::default()
                        };
                    }
                }
                match segs.last_mut() {
                    Some(Seg::Text(prev))
                        if (prev.bold, prev.italic, prev.underline, prev.sub, prev.sup)
                            == (r.bold, r.italic, r.underline, r.sub, r.sup)
                            && (&prev.font, prev.size.map(f64::to_bits), &prev.color)
                                == (&r.font, r.size.map(f64::to_bits), &r.color) =>
                    {
                        prev.text.push_str(&r.text);
                    }
                    _ => segs.push(Seg::Text(r)),
                }
            }
        }
    }
    let mut prev_space = true;
    let mut any = false;
    for seg in segs.iter_mut() {
        match seg {
            Seg::Br => prev_space = true,
            Seg::Text(r) => {
                let mut o = String::with_capacity(r.text.len());
                for c in r.text.chars() {
                    if c == ' ' {
                        if !prev_space {
                            o.push(' ');
                        }
                        prev_space = true;
                    } else {
                        o.push(c);
                        prev_space = false;
                        any = true;
                    }
                }
                r.text = o;
            }
        }
    }
    if !any {
        return None;
    }
    // Trim a trailing space before each break and at the end.
    let n = segs.len();
    for i in 0..n {
        let at_end = i + 1 == n || matches!(segs[i + 1], Seg::Br);
        if at_end {
            if let Seg::Text(r) = &mut segs[i] {
                while r.text.ends_with(' ') {
                    r.text.pop();
                }
            }
        }
    }
    let mut html = String::new();
    for seg in &segs {
        match seg {
            Seg::Br => html.push_str("<br/>"),
            Seg::Text(r) => {
                if r.text.is_empty() {
                    continue;
                }
                let mut s = escape_text(&r.text);
                if heading.is_none() {
                    if r.bold {
                        s = format!("<b>{}</b>", s);
                    }
                    if r.italic {
                        s = format!("<i>{}</i>", s);
                    }
                    if r.underline {
                        s = format!("<u>{}</u>", s);
                    }
                }
                if r.sub {
                    s = format!("<sub>{}</sub>", s);
                }
                if r.sup {
                    s = format!("<sup>{}</sup>", s);
                }
                // Only what differs from the paragraph's own style gets a span of its own.
                let mut css = Vec::new();
                if r.font.is_some() && r.font != st.font {
                    css.push(format!("font-family:{}", font_stack(r.font.as_deref().unwrap_or(""))));
                }
                if r.size.is_some() && r.size != st.size {
                    css.push(format!("font-size:{}pt", pt(r.size.unwrap_or(0.0))));
                }
                let cls = match &r.color {
                    Some(c) if r.color != st.color => color_class(c, colors),
                    _ => None,
                };
                if !css.is_empty() || cls.is_some() {
                    let class = cls.map(|c| format!(" class=\"{c}\"")).unwrap_or_default();
                    let style = if css.is_empty() {
                        String::new()
                    } else {
                        format!(" style=\"{}\"", css.join(";"))
                    };
                    s = format!("<span{class}{style}>{s}</span>");
                }
                html.push_str(&s);
            }
        }
    }
    // The paragraph's own style: font, size, weight and colour; alignment and margins on a block around it.
    let mut css = Vec::new();
    if let Some(f) = &st.font {
        css.push(format!("font-family:{}", font_stack(f)));
    }
    if let Some(z) = st.size {
        css.push(format!("font-size:{}pt", pt(z)));
    }
    if st.bold && heading.is_none() {
        css.push("font-weight:bold".to_string());
    }
    if st.italic && heading.is_none() {
        css.push("font-style:italic".to_string());
    }
    if st.underline {
        css.push("text-decoration:underline".to_string());
    }
    let cls = st.color.as_deref().and_then(|c| color_class(c, colors));
    let class = cls.map(|c| format!(" class=\"{c}\"")).unwrap_or_default();
    let mut block = Vec::new();
    match p.align.as_deref().or(st.align.as_deref()) {
        Some(a) if a != "left" => block.push(format!("text-align:{a}")),
        _ => {}
    }
    if p.margin_left > 0.0 {
        block.push(format!("padding-left:{}pt", pt(p.margin_left)));
    }
    if p.text_indent.abs() > 0.0 {
        block.push(format!("text-indent:{}pt", pt(p.text_indent)));
    }
    Some(match heading {
        Some(n) => {
            let style = join_style(&css, &block);
            format!("'<h{n}{class}{style}>{html}</h{n}>")
        }
        None => {
            let inner = if p.style == "List" {
                format!("&bull; {}", html)
            } else {
                html
            };
            if class.is_empty() && css.is_empty() && block.is_empty() && !inner.starts_with('<') {
                format!("'{inner}")
            } else {
                // An HTML line gets no paragraph of its own from the engine, so it is one here.
                let style = join_style(&css, &block);
                format!("'<p{class}{style}>{inner}</p>")
            }
        }
    })
}

fn join_style(a: &[String], b: &[String]) -> String {
    let all: Vec<&str> = a.iter().chain(b.iter()).map(String::as_str).collect();
    if all.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", all.join(";"))
    }
}

/// Translates a sheet. See the crate documentation.
/// A name the file labels as a unit in one place and defines as a variable in another (a matrix
/// `N` next to forces in `N`): the engine would read every `N` as the variable, so the variable is
/// renamed `N_v` everywhere and the unit keeps its name.
fn rename_unit_clashes(sheet: &mut Sheet) -> HashMap<String, String> {
    let mut units: HashSet<String> = HashSet::new();
    let mut vars: HashSet<String> = HashSet::new();
    let mut scan = |stmt: &mut Stmt| {
        // Names read or bound anywhere; the labelled ones are units, the targets are variables.
        stmt.for_each_name_mut(&mut |n: &mut Name| {
            if n.unit && n.sub.is_none() && tables::map_unit(&n.base).is_some() {
                units.insert(n.base.clone());
            }
            // A unit the engine spells differently (`sec` is `s`) takes that spelling from any
            // variable of the same name, labelled or not.
            if n.sub.is_none() {
                if let Some(m) = tables::map_unit(&n.base) {
                    if m != n.base {
                        units.insert(m);
                    }
                }
            }
        });
        if let Stmt::Define { target, .. } = stmt {
            match target {
                Target::Var(n) if !n.unit && n.sub.is_none() => {
                    vars.insert(n.base.clone());
                }
                Target::Element { base: Some(b), .. } if b.sub.is_none() => {
                    vars.insert(b.base.clone());
                }
                Target::Multi(ns) => {
                    for n in ns.iter().filter(|n| n.sub.is_none()) {
                        vars.insert(n.base.clone());
                    }
                }
                _ => {}
            }
        }
    };
    for r in sheet.regions.iter_mut() {
        if let RegionKind::Math(m) = &mut r.kind {
            scan(&mut m.stmt);
        }
    }
    let clash: HashSet<String> = units.intersection(&vars).cloned().collect();
    let renamed: HashMap<String, String> = clash.iter().map(|c| (format!("{c}_v"), c.clone())).collect();
    if clash.is_empty() {
        return renamed;
    }
    for r in sheet.regions.iter_mut() {
        if let RegionKind::Math(m) = &mut r.kind {
            m.stmt.for_each_name_mut(&mut |n: &mut Name| {
                if !n.unit && n.sub.is_none() && clash.contains(&n.base) {
                    n.base = format!("{}_v", n.base);
                }
            });
        }
    }
    renamed
}

pub fn translate(mut sheet: Sheet, opts: &ImportOptions) -> (String, ImportReport, Vec<Asset>) {
    let mut renamed = rename_unit_clashes(&mut sheet);
    renamed.extend(crate::names::rename_invalid(&mut sheet));
    let rows = reading_rows(std::mem::take(&mut sheet.regions));
    let has_references = rows.iter().flatten().any(|r| matches!(r.kind, RegionKind::Reference));
    let cx_any = rows.iter().flatten().any(|r| {
        matches!(r.kind, RegionKind::Math(_) | RegionKind::Plot(_)) && format!("{:?}", r.kind).contains("\"imag\"")
    });
    let mut tr = Tr {
        origin: sheet.origin,
        defined: HashSet::new(),
        funcs: HashMap::new(),
        str_table: Vec::new(),
        str_vars: HashMap::new(),
        str_funcs: HashMap::new(),
        array_funcs: HashSet::new(),
        str_mats: HashSet::new(),
        str_mat_ok: false,
        num_texts: HashSet::new(),
        empty_texts: HashSet::new(),
        str_allow: false,
        io_args: false,
        saved_by_name: HashMap::new(),
        gap_names: HashMap::new(),
        arrays: HashSet::new(),
        params: Vec::new(),
        plot_param: None,
        unit_hints: HashMap::new(),
        ranges: HashMap::new(),
        component_names: sheet
            .component_names
            .iter()
            .map(|n| tables::normalise_name(&n.flat()))
            .collect(),
        locals: HashMap::new(),
        local_arrays: HashSet::new(),
        zero_started: HashSet::new(),
        partial: HashMap::new(),
        fn_alias: HashMap::new(),
        nested: HashSet::new(),
        fn_defs: HashMap::new(),
        cx_any,
        cplx: HashSet::new(),
        cx_locals: HashSet::new(),
        matrices: HashSet::new(),
        cx_matrix: false,
        solve_cons: Vec::new(),
        col_vecs: HashSet::new(),
        growable: HashSet::new(),
        prologue: Vec::new(),
        zero_units: HashMap::new(),
        elem_built: HashMap::new(),
        prog_n: 0,
        text_styles: std::mem::take(&mut sheet.text_styles),
        colors: Default::default(),
        hoisted: Vec::new(),
        loop_stack: Vec::new(),
        cond_depth: 0,
        ret: None,
        readable: false,
        readable_bad: false,
        display: false,
        maths_shown: false,
        display_renamed: false,
        renamed,
        default_precision: sheet.default_precision,
        fmt_cur: None,
        cur: Vec::new(),
        uses_trig: false,
        lines: Vec::new(),
        report: ImportReport {
            origin: sheet.origin,
            ..Default::default()
        },
        depth: 0,
        opts: opts.clone(),
        assets: Vec::new(),
        used_paths: HashSet::new(),
        asset_total: 0,
        has_references,
        uses_val_ml: false,
        pending_ml: None,
    };
    for r in rows.iter().flatten() {
        if let RegionKind::Math(m) = &r.kind {
            if let (
                Stmt::Eval {
                    expr: Expr::Name(n), ..
                },
                Some(v),
            ) = (&m.stmt, &m.saved_expr)
            {
                tr.saved_by_name
                    .entry(tables::normalise_name(&n.flat()))
                    .or_insert_with(|| v.clone());
            }
            if let Stmt::Eval {
                expr: Expr::Name(n),
                unit: Some(u),
            } = &m.stmt
            {
                tr.unit_hints
                    .entry(tables::normalise_name(&n.flat()))
                    .or_insert_with(|| u.clone());
            }
        }
    }
    tr.layout_rows(&rows);
    let mut out = String::new();
    out.push_str(&format!(
        "'<!-- Imported from a foreign worksheet: {} of {} regions need attention. Lines that start with {} mark what did not convert. -->\n",
        tr.report.regions_needing_attention(),
        tr.report.regions_total(),
        GAP_MARKER
    ));
    // The definitions are the author's; a name used only by a region shown as a picture or by an
    // unconverted one is not an editor finding.
    if tr.maths_shown {
        out.push_str("'<!--{\"LintIgnore\": [\"CPD-3312\"]}-->\n");
    }
    if let Some(block) = style_block(&tr.colors, tr.maths_shown) {
        out.push_str(&block);
        out.push('\n');
    }
    // Results are shown as the source shows them: the expression and its value, no substituted step.
    if tr.maths_shown {
        // Equations shown for reading carry the numbers as written (see `format_line`); results are
        // shown under a number format set where they are.
        out.push_str("#nosub\n#round 12\n");
        if tr.uses_val_ml {
            out.push_str("#settings {\"valMathml\": true}\n");
        }
    }
    if tr.uses_trig {
        out.push_str("#rad\n");
        tr.report.issues.insert(
            0,
            Issue {
                code: Code::AngleMode,
                severity: Severity::Note,
                detail: None,
                region_id: None,
                line: None,
                reason: Code::AngleMode.reason().to_string(),
                root: None,
            },
        );
    }
    for l in &tr.lines {
        out.push_str(l);
        out.push('\n');
    }
    (out, tr.report, tr.assets)
}
