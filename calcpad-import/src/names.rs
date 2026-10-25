//! Names the engine cannot spell.
//!
//! A worksheet may name a variable `SF_crack.step2`, `U/R_twist` or `Rod#_top`. The engine
//! reads a name as letters, digits and underscores starting with a letter, so each such name is
//! given one engine spelling for the whole sheet (`SF_crack_step2`, `U_R_twist`,
//! `RodNum_top`), chosen once per distinct original and applied everywhere the name appears: the
//! definition, every use, function names and parameters, plot expressions and input components.
//! The typeset equations show the original (see [`crate::mathml`]), so the reader sees the
//! author's name and the hidden computation uses the engine's.
use crate::ir::*;
use crate::tables;
use std::collections::{HashMap, HashSet};

/// True when the engine can spell this name as it is.
fn spellable(flat: &str) -> bool {
    crate::translate::valid_ident(&tables::normalise_name(flat))
}

/// The engine spelling of one name, before collisions are looked at: every character the engine
/// cannot take becomes an underscore (`#` reads "number", `%` "pct"), runs of underscores
/// collapse, and a name that starts with a non-letter gets a leading `v`.
pub fn engine_spelling(orig: &str) -> String {
    let mut out = String::new();
    for c in tables::normalise_name(orig).chars() {
        if c.is_alphanumeric() || c == '_' || c == '\u{2032}' || c == '\u{2033}' {
            out.push(c);
        } else if c == '#' {
            out.push_str(if out.is_empty() { "Num_" } else { "Num" });
        } else if c == '%' {
            out.push_str("pct");
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    let mut out = out.trim_matches('_').to_string();
    match out.chars().next() {
        None => out = "x".into(),
        Some(c) if !c.is_alphabetic() => out.insert(0, 'v'),
        _ => {}
    }
    out
}

/// True when an engine spelling would be read as something else: a unit, a constant or one of the
/// engine's functions.
fn clashes(s: &str) -> bool {
    tables::map_unit(s).is_some() || tables::lookup_constant(s).is_some() || tables::is_engine_function(s)
}

/// Visits every identifier that has to be spelled: names read or bound, the names of called
/// functions and of defined functions, and the expressions on plot axes.
fn walk_sheet(sheet: &mut Sheet, f: &mut dyn FnMut(&mut Name)) {
    for n in sheet.component_names.iter_mut() {
        f(n);
    }
    for r in sheet.regions.iter_mut() {
        match &mut r.kind {
            RegionKind::Math(m) => walk_stmt(&mut m.stmt, f),
            RegionKind::Plot(PlotData::Xy(p)) => {
                for ax in [&mut p.x, &mut p.y] {
                    for e in ax.exprs.iter_mut().flatten() {
                        walk_expr(e, f);
                    }
                    for e in ax.units.iter_mut().flatten() {
                        walk_expr(e, f);
                    }
                    for e in [&mut ax.from, &mut ax.to].into_iter().flatten() {
                        walk_expr(e, f);
                    }
                }
            }
            _ => {}
        }
    }
}

fn walk_stmt(s: &mut Stmt, f: &mut dyn FnMut(&mut Name)) {
    match s {
        Stmt::Define { target, value, .. } => {
            match target {
                Target::Var(n) => f(n),
                Target::Func(n, params) => {
                    f(n);
                    params.iter_mut().for_each(&mut *f);
                }
                Target::Element { base, index } => {
                    if let Some(b) = base {
                        f(b);
                    }
                    index.iter_mut().for_each(|e| walk_expr(e, f));
                }
                Target::Multi(ns) => ns.iter_mut().for_each(&mut *f),
                Target::Slice { base, index, .. } => {
                    f(base);
                    walk_expr(index, f);
                }
            }
            walk_expr(value, f);
        }
        Stmt::Eval { expr, unit } => {
            walk_expr(expr, f);
            if let Some(u) = unit {
                walk_expr(u, f);
            }
        }
        Stmt::Bare(e) => walk_expr(e, f),
        Stmt::Unsupported { .. } => {}
    }
}

fn walk_expr(e: &mut Expr, f: &mut dyn FnMut(&mut Name)) {
    match e {
        Expr::Name(n) => f(n),
        Expr::Num(_) | Expr::Str(_) | Expr::Unsupported { .. } => {}
        Expr::Un(_, a) | Expr::Postfix(_, a) => walk_expr(a, f),
        Expr::Bin(_, a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Expr::Call(n, args) => {
            f(n);
            args.iter_mut().for_each(|a| walk_expr(a, f));
        }
        Expr::Index(b, idx) => {
            walk_expr(b, f);
            idx.iter_mut().for_each(|a| walk_expr(a, f));
        }
        Expr::Matrix { items, .. } => items.iter_mut().for_each(|a| walk_expr(a, f)),
        Expr::Range { first, second, last } => {
            walk_expr(first, f);
            if let Some(s) = second {
                walk_expr(s, f);
            }
            walk_expr(last, f);
        }
        Expr::Integral {
            var,
            body,
            lower,
            upper,
        } => {
            f(var);
            walk_expr(body, f);
            walk_expr(lower, f);
            walk_expr(upper, f);
        }
        Expr::Series { var, body, bounds, .. } => {
            f(var);
            walk_expr(body, f);
            if let Some((lo, hi)) = bounds {
                walk_expr(lo, f);
                walk_expr(hi, f);
            }
        }
        Expr::Derivative { var, body } => {
            f(var);
            walk_expr(body, f);
        }
        Expr::Program(stmts) => stmts.iter_mut().for_each(|s| walk_pstmt(s, f)),
    }
}

fn walk_pstmt(s: &mut PStmt, f: &mut dyn FnMut(&mut Name)) {
    match s {
        PStmt::Local(t, e) => {
            match t {
                PTarget::Var(n) => f(n),
                PTarget::Element { base, index } => {
                    f(base);
                    index.iter_mut().for_each(|a| walk_expr(a, f));
                }
                PTarget::Slice { base, index, .. } => {
                    f(base);
                    walk_expr(index, f);
                }
            }
            walk_expr(e, f);
        }
        PStmt::For { var, range, body } => {
            f(var);
            walk_expr(range, f);
            body.iter_mut().for_each(|s| walk_pstmt(s, f));
        }
        PStmt::While { cond, body } => {
            walk_expr(cond, f);
            body.iter_mut().for_each(|s| walk_pstmt(s, f));
        }
        PStmt::If { branches, otherwise } => {
            for (c, b) in branches.iter_mut() {
                walk_expr(c, f);
                b.iter_mut().for_each(|s| walk_pstmt(s, f));
            }
            if let Some(o) = otherwise {
                o.iter_mut().for_each(|s| walk_pstmt(s, f));
            }
        }
        PStmt::Return(e) | PStmt::Expr(e) => walk_expr(e, f),
        PStmt::Break | PStmt::Continue | PStmt::Unsupported(_) => {}
    }
}

/// Gives every name the engine cannot spell its one engine spelling, in place. Returns the engine
/// spelling -> the author's name, for the typeset display.
///
/// A name that carries a superscript caret is left alone (it is how the reader marks text that
/// cannot be a name), as is a name the file labels as a unit. The spelling is unique in the sheet:
/// when it equals another name in use, or would be read as a unit, constant or engine function, a
/// number or `_v` is added.
pub fn rename_invalid(sheet: &mut Sheet) -> HashMap<String, String> {
    // Pass 1: the distinct originals in order of first appearance, and the spellable names in use.
    let mut order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut taken: HashSet<String> = HashSet::new();
    walk_sheet(sheet, &mut |n: &mut Name| {
        let flat = n.flat();
        if n.unit || flat.contains('^') || tables::map_unit(&flat).is_some() {
            return;
        }
        if spellable(&flat) {
            taken.insert(tables::normalise_name(&flat));
        } else if seen.insert(flat.clone()) {
            order.push(flat);
        }
    });
    let mut map: HashMap<String, String> = HashMap::new();
    let mut shown: HashMap<String, String> = HashMap::new();
    for orig in order {
        let mut cand = engine_spelling(&orig);
        if clashes(&cand) {
            cand.push_str("_v");
        }
        let stem = cand.clone();
        let mut k = 2;
        while taken.contains(&cand) {
            cand = format!("{stem}_{k}");
            k += 1;
        }
        taken.insert(cand.clone());
        // What the page shows is the author's name; characters that mean something in markup or in
        // the engine's text are left out of it.
        let shown_name: String = orig
            .chars()
            .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | '&' | '"' | '\\' | '\''))
            .collect();
        shown.insert(
            cand.clone(),
            if shown_name.is_empty() {
                cand.clone()
            } else {
                shown_name
            },
        );
        map.insert(orig, cand);
    }
    if map.is_empty() {
        return shown;
    }
    // Pass 2: apply.
    walk_sheet(sheet, &mut |n: &mut Name| {
        if n.unit || tables::map_unit(&n.flat()).is_some() {
            return;
        }
        if let Some(new) = map.get(&n.flat()) {
            n.base = new.clone();
            n.sub = None;
        }
    });
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings() {
        assert_eq!(engine_spelling("SF_crack.step2"), "SF_crack_step2");
        assert_eq!(engine_spelling("U/R_twist"), "U_R_twist");
        assert_eq!(engine_spelling("Rod#_top"), "RodNum_top");
        assert_eq!(engine_spelling("#count"), "Num_count");
        assert_eq!(engine_spelling("U/R:Test_x.1"), "U_R_Test_x_1");
        assert_eq!(engine_spelling("2nd.pass"), "v2nd_pass");
        assert_eq!(engine_spelling("\u{394}%"), "\u{394}pct");
    }
}
