//! Vectors of texts (a column of `"OK"` / `"NG"` messages).
//!
//! The engine has no texts, and a worksheet text is held as a whole number, its code in a table of
//! the sheet's literals (see `translate::strings`). A vector whose elements are texts is therefore a
//! vector of codes. This pass runs on the tree before it is translated: a text assigned to an
//! element (in a program, or `v[k] := if(c; "OK"; "NG")`) becomes its code, and an element compared
//! with a text (`v[k] = "OK"`) is compared with that code, so the vector goes through the ordinary
//! number machinery of the translator. The names that hold such vectors are returned so that an
//! evaluation prints the texts and not the codes.

use crate::ir::*;
use crate::tables;
use std::collections::HashMap;

/// The table of text literals found, and the names that hold vectors of texts.
#[derive(Default)]
pub(crate) struct TextVecs {
    pub table: Vec<String>,
    /// The names that hold vectors of texts, with the codes their elements can have.
    pub names: HashMap<String, Vec<usize>>,
    current: Option<String>,
}

fn flat(n: &Name) -> String {
    tables::normalise_name(&n.flat())
}

impl TextVecs {
    /// The code of a text, and a note that the current definition can hold it.
    fn assigned(&mut self, s: &str) -> usize {
        let c = self.code(s);
        if let Some(n) = self.current.clone() {
            let d = self.names.entry(n).or_default();
            if !d.contains(&c) {
                d.push(c);
            }
        }
        c
    }

    fn code(&mut self, s: &str) -> usize {
        match self.table.iter().position(|t| t == s) {
            Some(i) => i + 1,
            None => {
                self.table.push(s.to_string());
                self.table.len()
            }
        }
    }
}

fn is_if(n: &Name, args: &[Expr]) -> bool {
    n.sub.is_none() && n.base == "if" && args.len() == 3
}

/// A text, or an `if` that chooses between texts (and possibly numbers).
fn textish(e: &Expr) -> bool {
    match e {
        Expr::Str(_) => true,
        Expr::Call(n, args) if is_if(n, args) => textish(&args[1]) || textish(&args[2]),
        // A program (a chain of `if`s) whose every value is a text.
        Expr::Program(stmts) => {
            let mut any = false;
            program_values_are_texts(stmts, &mut any) && any
        }
        _ => false,
    }
}

/// Whether every value a program gives (an expression statement or a `return`) is a text; `any`
/// says whether it has one.
fn program_values_are_texts(stmts: &[PStmt], any: &mut bool) -> bool {
    stmts.iter().all(|st| match st {
        PStmt::Expr(e) | PStmt::Return(e) => {
            *any = true;
            matches!(e, Expr::Str(_))
        }
        PStmt::If { branches, otherwise } => {
            branches.iter().all(|(_, b)| program_values_are_texts(b, any))
                && otherwise.as_ref().is_none_or(|o| program_values_are_texts(o, any))
        }
        PStmt::Local(PTarget::Var(_), _) => true,
        _ => false,
    })
}

/// Whether the statements give a local only texts as elements and end with that local.
fn ends_with_text_vector(stmts: &[PStmt]) -> bool {
    /// For each local, whether every element assigned to it is a text.
    fn assigns(stmts: &[PStmt], out: &mut HashMap<String, bool>) {
        for st in stmts {
            match st {
                PStmt::Local(PTarget::Element { base, .. }, e) => {
                    let all = out.entry(flat(base)).or_insert(true);
                    *all &= textish(e);
                }
                PStmt::For { body, .. } | PStmt::While { body, .. } => assigns(body, out),
                PStmt::If { branches, otherwise } => {
                    for (_, b) in branches {
                        assigns(b, out);
                    }
                    if let Some(o) = otherwise {
                        assigns(o, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut locals = HashMap::new();
    assigns(stmts, &mut locals);
    match stmts.last() {
        Some(PStmt::Expr(Expr::Name(n))) | Some(PStmt::Return(Expr::Name(n))) => {
            locals.get(&flat(n)).copied().unwrap_or(false)
        }
        _ => false,
    }
}

pub(crate) fn rewrite(sheet: &mut Sheet) -> TextVecs {
    let mut tv = TextVecs::default();
    // Which names hold vectors of texts.
    loop {
        let before = tv.names.len();
        for r in &sheet.regions {
            let RegionKind::Math(m) = &r.kind else { continue };
            let Stmt::Define { target, value, .. } = &m.stmt else {
                continue;
            };
            match (target, value) {
                (Target::Var(n), Expr::Program(stmts)) if ends_with_text_vector(stmts) => {
                    tv.names.entry(flat(n)).or_default();
                }
                (Target::Var(n), Expr::Name(a)) if tv.names.contains_key(&flat(a)) => {
                    tv.names.entry(flat(n)).or_default();
                }
                (Target::Element { base: Some(n), .. }, v) if textish(v) => {
                    tv.names.entry(flat(n)).or_default();
                }
                _ => {}
            }
        }
        if tv.names.len() == before {
            break;
        }
    }
    if tv.names.is_empty() {
        return tv;
    }
    for r in sheet.regions.iter_mut() {
        if let RegionKind::Math(m) = &mut r.kind {
            match &mut m.stmt {
                Stmt::Define { target, value, .. } => {
                    tv.current = match target {
                        Target::Var(n) | Target::Element { base: Some(n), .. } => Some(flat(n)),
                        _ => None,
                    };
                    if let Target::Element { index, .. } = target {
                        index.iter_mut().for_each(|e| walk(e, &mut tv));
                    }
                    if matches!(target, Target::Element { .. }) && textish(value) {
                        codes(value, &mut tv);
                    }
                    walk(value, &mut tv);
                    if let (Target::Var(n), Expr::Name(a)) = (&*target, &*value) {
                        if let Some(d) = tv.names.get(&flat(a)).cloned() {
                            if tv.names.contains_key(&flat(n)) {
                                tv.names.insert(flat(n), d);
                            }
                        }
                    }
                    tv.current = None;
                }
                Stmt::Eval { expr, unit } => {
                    walk(expr, &mut tv);
                    if let Some(u) = unit {
                        walk(u, &mut tv);
                    }
                }
                Stmt::Bare(e) => walk(e, &mut tv),
                Stmt::Unsupported { .. } => {}
            }
        }
    }
    tv
}

/// Texts in a chosen value become their codes.
fn codes(e: &mut Expr, tv: &mut TextVecs) {
    match e {
        Expr::Str(s) => {
            let c = tv.assigned(s);
            *e = Expr::Num(c.to_string());
        }
        Expr::Call(n, args) if is_if(n, args) => {
            let (a, b) = args.split_at_mut(2);
            codes(&mut a[1], tv);
            codes(&mut b[0], tv);
        }
        Expr::Program(stmts) => codes_in(stmts, tv),
        _ => {}
    }
}

fn codes_in(stmts: &mut [PStmt], tv: &mut TextVecs) {
    for st in stmts {
        match st {
            PStmt::Expr(e) | PStmt::Return(e) => codes(e, tv),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    codes_in(b, tv);
                }
                if let Some(o) = otherwise {
                    codes_in(o, tv);
                }
            }
            _ => {}
        }
    }
}

fn is_text_element(e: &Expr, tv: &TextVecs) -> bool {
    matches!(e, Expr::Index(b, _) if matches!(&**b, Expr::Name(n) if tv.names.contains_key(&flat(n))))
}

fn walk_stmts(stmts: &mut [PStmt], tv: &mut TextVecs, text: bool) {
    for st in stmts {
        match st {
            PStmt::Local(t, e) => {
                if let PTarget::Element { index, .. } = t {
                    index.iter_mut().for_each(|x| walk(x, tv));
                    if text && textish(e) {
                        codes(e, tv);
                    }
                }
                walk(e, tv);
            }
            PStmt::For { range, body, .. } => {
                walk(range, tv);
                walk_stmts(body, tv, text);
            }
            PStmt::While { cond, body } => {
                walk(cond, tv);
                walk_stmts(body, tv, text);
            }
            PStmt::If { branches, otherwise } => {
                for (c, b) in branches {
                    walk(c, tv);
                    walk_stmts(b, tv, text);
                }
                if let Some(o) = otherwise {
                    walk_stmts(o, tv, text);
                }
            }
            PStmt::Return(e) | PStmt::Expr(e) => walk(e, tv),
            _ => {}
        }
    }
}

/// Visits every sub-expression; an element compared with a text is compared with its code.
fn walk(e: &mut Expr, tv: &mut TextVecs) {
    match e {
        Expr::Bin(op, a, b) => {
            walk(a, tv);
            walk(b, tv);
            if matches!(op, BinOp::Eq | BinOp::Ne) {
                if is_text_element(a, tv) {
                    if let Expr::Str(s) = &**b {
                        let c = tv.code(s);
                        **b = Expr::Num(c.to_string());
                    }
                } else if is_text_element(b, tv) {
                    if let Expr::Str(s) = &**a {
                        let c = tv.code(s);
                        **a = Expr::Num(c.to_string());
                    }
                }
            }
        }
        Expr::Un(_, a) | Expr::Postfix(_, a) => walk(a, tv),
        Expr::Call(_, args) => args.iter_mut().for_each(|a| walk(a, tv)),
        Expr::Index(b, idx) => {
            walk(b, tv);
            idx.iter_mut().for_each(|a| walk(a, tv));
        }
        Expr::Matrix { items, .. } => items.iter_mut().for_each(|a| walk(a, tv)),
        Expr::Range { first, second, last } => {
            walk(first, tv);
            if let Some(s) = second {
                walk(s, tv);
            }
            walk(last, tv);
        }
        Expr::Program(stmts) => {
            let text = ends_with_text_vector(stmts);
            walk_stmts(stmts, tv, text)
        }
        _ => {}
    }
}
