//! Text values used in checks (`"OK"` / `"NG"`), as the engine can run them.
//!
//! The engine has no string values. A text that a worksheet compares, selects with `if` or stores
//! in a name is therefore held as a whole number, its code in a table of the sheet's literals
//! (`"OK"` is 1, `"NG"` is 2, and so on). The computation is the same, because only equality of
//! two texts and the choice between texts occur in the checks of real worksheets. The page shows
//! the author's equation as written, and an evaluation of a text name prints the text of its
//! current code, selected by an `#if` chain over the texts that name can hold.
//!
//! What cannot be held this way stays a gap that says so: joining texts, taking a part of one,
//! text as a function argument, text read from a file.

use super::*;

impl Tr {
    /// The code of a text literal in the sheet's table (new literals get the next number).
    pub(super) fn str_code(&mut self, s: &str) -> usize {
        if let Some(i) = self.str_table.iter().position(|t| t == s) {
            return i + 1;
        }
        self.str_table.push(s.to_string());
        self.str_table.len()
    }

    /// Whether the expression is a text: a literal, a name or call known to hold one, or an `if`
    /// that chooses between texts.
    pub(super) fn is_stringy(&self, e: &Expr) -> bool {
        match e {
            Expr::Str(_) => true,
            Expr::Name(n) => self.name_is_text(&tables::normalise_name(&n.flat())),
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    return self.is_stringy(&args[1]) || self.is_stringy(&args[2]);
                }
                self.str_funcs.contains_key(&f) && self.funcs.contains_key(&f)
            }
            _ => false,
        }
    }

    /// Whether the name, read now, holds a text: a text argument of the function being translated,
    /// a local of the program being translated that only ever holds texts, or a text of the sheet
    /// (that no argument or local of the same name hides).
    pub(super) fn name_is_text(&self, f: &str) -> bool {
        if self.text_params.contains(f) {
            return true;
        }
        if self.locals.contains_key(f) {
            return self.str_locs.contains_key(f);
        }
        self.str_vars.contains_key(f) && !self.params.iter().any(|p| p == f)
    }

    /// The codes a text expression can have.
    pub(super) fn str_domain(&mut self, e: &Expr) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        match e {
            Expr::Str(s) => out.push(self.str_code(s)),
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                if self.text_params.contains(&f) {
                    out.push(TEXT_ARG);
                } else if self.locals.contains_key(&f) {
                    if let Some(d) = self.str_locs.get(&f) {
                        out.extend(d.iter().copied());
                    }
                } else if let Some(d) = self.str_vars.get(&f) {
                    out.extend(d.iter().copied());
                }
            }
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    out.extend(self.str_domain(&args[1]));
                    out.extend(self.str_domain(&args[2]));
                } else if let Some(d) = self.str_funcs.get(&f) {
                    out.extend(d.iter().copied().filter(|c| *c != MIXED_FN));
                }
            }
            _ => {}
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// A text expression as the engine's whole-number code. `None` when the expression is not a
    /// text this module can hold (the caller then translates it as it would any other).
    pub(super) fn str_expr(&mut self, e: &Expr) -> Option<R> {
        if self.readable || self.display || !self.is_stringy(e) {
            return None;
        }
        match e {
            Expr::Str(s) => Some(atom(self.str_code(s).to_string())),
            Expr::Name(_) => {
                self.str_allow = true;
                let r = self.expr(e);
                self.str_allow = false;
                Some(r)
            }
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    let c = self.expr(&args[0]).s;
                    let a = self.str_or_expr(&args[1]).s;
                    let b = self.str_or_expr(&args[2]).s;
                    return Some(atom(format!("if({c}; {a}; {b})")));
                }
                self.str_allow = true;
                let r = self.expr(e);
                self.str_allow = false;
                Some(r)
            }
            _ => None,
        }
    }

    /// The code of a text, or the ordinary translation of anything else.
    pub(super) fn str_or_expr(&mut self, e: &Expr) -> R {
        match self.str_expr(e) {
            Some(r) => r,
            None => self.expr(e),
        }
    }

    /// Whether a text expression is a text on some paths and a number on others.
    pub(super) fn is_mixed(&self, e: &Expr) -> bool {
        let (t, n) = self.text_number_leaves(e);
        t && n
    }

    /// Whether some leaf of the expression is a text, and whether some leaf is a number.
    fn text_number_leaves(&self, e: &Expr) -> (bool, bool) {
        match e {
            Expr::Str(_) => (true, false),
            Expr::Name(nm) => {
                let f = tables::normalise_name(&nm.flat());
                if self.text_params.contains(&f) || (self.locals.contains_key(&f) && self.str_locs.contains_key(&f)) {
                    (true, false)
                } else if self.str_vars.contains_key(&f) && !self.params.contains(&f) && !self.locals.contains_key(&f) {
                    (true, self.str_mixed.contains(&f))
                } else {
                    (false, true)
                }
            }
            Expr::Call(nm, args) => {
                let f = tables::normalise_name(&nm.flat());
                if nm.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    let (a, b) = (self.text_number_leaves(&args[1]), self.text_number_leaves(&args[2]));
                    (a.0 || b.0, a.1 || b.1)
                } else if self.str_funcs.contains_key(&f) && self.funcs.contains_key(&f) {
                    (true, self.str_funcs[&f].contains(&MIXED_FN))
                } else {
                    (false, true)
                }
            }
            _ => (false, true),
        }
    }

    /// Records that `name` now holds a text (with these possible codes) or no longer does.
    pub(super) fn set_str_var(&mut self, name: &str, domain: Option<Vec<usize>>) {
        match domain {
            Some(d) => {
                self.str_vars.insert(name.to_string(), d);
                if std::mem::take(&mut self.next_mixed) {
                    self.str_mixed.insert(name.to_string());
                } else {
                    self.str_mixed.remove(name);
                }
            }
            None => {
                self.str_vars.remove(name);
                self.str_mixed.remove(name);
                self.next_mixed = false;
            }
        }
    }

    /// The lines that print a vector of texts held as codes: the author's left side, then a column
    /// with the text of each element's code.
    pub(super) fn text_vec_lines(&self, head: Option<&str>, name: &str, shown: &str, domain: &[usize]) -> String {
        let all: Vec<usize> = (1..=self.str_table.len()).collect();
        let domain: &[usize] = if domain.is_empty() { &all } else { domain };
        let mut out = String::new();
        let mathml = head.is_some();
        match head {
            Some(h) => out.push_str(&format!(
                "'<div class=\"imp-eq\"><math class=\"imp-m\">{h}<mrow><mo>[</mo><mtable>\n"
            )),
            None => out.push_str(&format!("'<p><b>{}</b> =\n", crate::mathml::esc(shown))),
        }
        out.push_str(&format!("#for i_textvec = 1 : len({name})\n"));
        out.push_str(if mathml { "'<mtr><mtd>\n" } else { "'<br>\n" });
        for (k, &c) in domain.iter().enumerate() {
            let kw = if k == 0 { "#if" } else { "#else if" };
            let t = crate::mathml::esc(&self.str_table[c - 1]);
            let item = if mathml { format!("<ms>{t}</ms>") } else { t };
            out.push_str(&format!("{kw} {name}.(i_textvec) ≡ {c}\n'{item}\n"));
        }
        if !domain.is_empty() {
            out.push_str("#end if\n");
        }
        if mathml {
            out.push_str("'</mtd></mtr>\n#loop\n'</mtable><mo>]</mo></mrow></math></div>");
        } else {
            out.push_str("#loop\n'</p>");
        }
        out
    }

    /// The lines that print a text result: for each text the expression can have, the equation
    /// head and that text, under a test of the code.
    pub(super) fn str_result_lines(&mut self, head: &str, code: &str, domain: &[usize]) -> String {
        let mut out = String::new();
        for (k, &d) in domain.iter().enumerate() {
            let text = self.str_table[d - 1].clone();
            let kw = if k == 0 { "#if" } else { "#else if" };
            out.push_str(&format!(
                "{kw} {code} ≡ {d}\n'<div class=\"imp-eq\"><math class=\"imp-m\">{head}<ms>{}</ms></math></div>\n",
                crate::mathml::esc(&text)
            ));
        }
        out.push_str("#end if");
        out
    }
}

/// The domain entry of a text that is an argument of the function being translated: its value
/// is whatever the caller passes, so it has no code of its own.
pub(super) const TEXT_ARG: usize = usize::MAX;

/// Kept in the codes of a function that gives a number for some arguments and a text for others.
pub(super) const MIXED_FN: usize = usize::MAX - 1;

/// Whether the expression has a text literal as a value it can give (itself, or a branch of an `if`).
pub(super) fn has_text_leaf(e: &Expr) -> bool {
    match e {
        Expr::Str(_) => true,
        Expr::Call(n, args) if n.sub.is_none() && n.base == "if" && args.len() == 3 => {
            has_text_leaf(&args[1]) || has_text_leaf(&args[2])
        }
        _ => false,
    }
}

/// Every assignment of a name in the statements (not through an element or a slice), and the names
/// that are assigned some other way (by element, by slice, or as a loop variable).
fn assignments<'a>(stmts: &'a [PStmt], out: &mut Vec<(String, &'a Expr)>, other: &mut HashSet<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), e) => out.push((tables::normalise_name(&n.flat()), e)),
            PStmt::Local(PTarget::Element { base, .. } | PTarget::Slice { base, .. }, _) => {
                other.insert(tables::normalise_name(&base.flat()));
            }
            PStmt::For { var, body, .. } => {
                other.insert(tables::normalise_name(&var.flat()));
                assignments(body, out, other);
            }
            PStmt::While { body, .. } => assignments(body, out, other),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    assignments(b, out, other);
                }
                if let Some(o) = otherwise {
                    assignments(o, out, other);
                }
            }
            _ => {}
        }
    }
}

/// Calls `f` on every expression of the statements, and of the expressions inside them.
fn visit_stmts(stmts: &[PStmt], f: &mut dyn FnMut(&Expr)) {
    for st in stmts {
        match st {
            PStmt::Local(t, e) => {
                match t {
                    PTarget::Element { index, .. } => index.iter().for_each(|i| visit_expr(i, f)),
                    PTarget::Slice { index, .. } => visit_expr(index, f),
                    PTarget::Var(_) => {}
                }
                visit_expr(e, f);
            }
            PStmt::For { range, body, .. } => {
                visit_expr(range, f);
                visit_stmts(body, f);
            }
            PStmt::While { cond, body } => {
                visit_expr(cond, f);
                visit_stmts(body, f);
            }
            PStmt::If { branches, otherwise } => {
                for (c, b) in branches {
                    visit_expr(c, f);
                    visit_stmts(b, f);
                }
                if let Some(o) = otherwise {
                    visit_stmts(o, f);
                }
            }
            PStmt::Return(e) | PStmt::Expr(e) => visit_expr(e, f),
            PStmt::Break | PStmt::Continue | PStmt::Unsupported(_) => {}
        }
    }
}

fn visit_expr(e: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Un(_, a) | Expr::Postfix(_, a) => visit_expr(a, f),
        Expr::Bin(_, a, b) => {
            visit_expr(a, f);
            visit_expr(b, f);
        }
        Expr::Call(_, args) => args.iter().for_each(|a| visit_expr(a, f)),
        Expr::Index(b, idx) => {
            visit_expr(b, f);
            idx.iter().for_each(|a| visit_expr(a, f));
        }
        Expr::Matrix { items, .. } => items.iter().for_each(|a| visit_expr(a, f)),
        Expr::Range { first, second, last } => {
            visit_expr(first, f);
            if let Some(s) = second {
                visit_expr(s, f);
            }
            visit_expr(last, f);
        }
        Expr::Integral { body, lower, upper, .. } => {
            visit_expr(body, f);
            visit_expr(lower, f);
            visit_expr(upper, f);
        }
        Expr::Series { body, bounds, .. } => {
            visit_expr(body, f);
            if let Some((lo, hi)) = bounds {
                visit_expr(lo, f);
                visit_expr(hi, f);
            }
        }
        Expr::Derivative { body, .. } => visit_expr(body, f),
        Expr::Program(stmts) => visit_stmts(stmts, f),
        Expr::Num(_) | Expr::Str(_) | Expr::Name(_) | Expr::Unsupported { .. } => {}
    }
}

/// The arguments (of the given names) that the expression compares with a text.
pub(super) fn params_compared_with_text(value: &Expr, params: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut look = |e: &Expr| {
        if let Expr::Bin(BinOp::Eq | BinOp::Ne, a, b) = e {
            for (x, y) in [(a, b), (b, a)] {
                if let (Expr::Name(n), Expr::Str(_)) = (&**x, &**y) {
                    let f = tables::normalise_name(&n.flat());
                    if params.contains(&f) {
                        out.insert(f);
                    }
                }
            }
        }
    };
    visit_expr(value, &mut look);
    out
}

/// A text statement in a program that is not its value: a note the author left among the
/// statements (`"place holder"`). When the program's last statement gives the value, such a
/// statement is overwritten and nothing reads it, so it is left out. A statement alone in its
/// block stays (it may be the value of its branch).
pub(super) fn without_note_texts(stmts: &[PStmt]) -> Vec<PStmt> {
    fn is_note(st: &PStmt) -> bool {
        matches!(st, PStmt::Expr(Expr::Str(_)))
    }
    fn clean(list: &[PStmt]) -> Vec<PStmt> {
        let kept: Vec<&PStmt> = list.iter().filter(|s| !is_note(s)).collect();
        let source: Vec<&PStmt> = if kept.is_empty() { list.iter().collect() } else { kept };
        source
            .into_iter()
            .map(|st| match st {
                PStmt::For { var, range, body } => PStmt::For {
                    var: var.clone(),
                    range: range.clone(),
                    body: clean(body),
                },
                PStmt::While { cond, body } => PStmt::While {
                    cond: cond.clone(),
                    body: clean(body),
                },
                PStmt::If { branches, otherwise } => PStmt::If {
                    branches: branches.iter().map(|(c, b)| (c.clone(), clean(b))).collect(),
                    otherwise: otherwise.as_ref().map(|o| clean(o)),
                },
                other => other.clone(),
            })
            .collect()
    }
    // The last statement run gives the value: a last statement that is a value or an assignment
    // overwrites every note before it.
    let overwritten = matches!(
        stmts.last(),
        Some(PStmt::Expr(e)) if !matches!(e, Expr::Str(_))
    ) || matches!(stmts.last(), Some(PStmt::Return(_) | PStmt::Local(PTarget::Var(_), _)));
    if overwritten && stmts.iter().any(contains_note) {
        clean(stmts)
    } else {
        stmts.to_vec()
    }
}

fn contains_note(st: &PStmt) -> bool {
    match st {
        PStmt::Expr(Expr::Str(_)) => true,
        PStmt::For { body, .. } | PStmt::While { body, .. } => body.iter().any(contains_note),
        PStmt::If { branches, otherwise } => {
            branches.iter().any(|(_, b)| b.iter().any(contains_note))
                || otherwise.as_ref().is_some_and(|o| o.iter().any(contains_note))
        }
        _ => false,
    }
}

impl Tr {
    /// Whether the expression gives a text on every path, given the locals taken to be texts.
    fn text_like(&self, e: &Expr, locs: &HashSet<String>, assigned: &HashSet<String>) -> bool {
        match e {
            Expr::Str(_) => true,
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                if assigned.contains(&f) {
                    locs.contains(&f)
                } else {
                    self.text_params.contains(&f)
                        || (self.str_vars.contains_key(&f) && !self.str_mixed.contains(&f) && !self.params.contains(&f))
                }
            }
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    self.text_like(&args[1], locs, assigned) && self.text_like(&args[2], locs, assigned)
                } else {
                    self.funcs.contains_key(&f) && self.str_funcs.get(&f).is_some_and(|d| !d.contains(&MIXED_FN))
                }
            }
            _ => false,
        }
    }

    /// Whether the expression is a text that does not depend on an unproven local: a literal, a
    /// text of the sheet, or a local already proven.
    fn text_rooted(&self, e: &Expr, proven: &HashSet<String>, assigned: &HashSet<String>) -> bool {
        match e {
            Expr::Str(_) => true,
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                if assigned.contains(&f) {
                    proven.contains(&f)
                } else {
                    true
                }
            }
            Expr::Call(n, args) if n.sub.is_none() && n.base == "if" && args.len() == 3 => {
                self.text_rooted(&args[1], proven, assigned) || self.text_rooted(&args[2], proven, assigned)
            }
            _ => true,
        }
    }

    /// The codes a text expression can have, with the locals' domains so far.
    fn text_codes(&mut self, e: &Expr, doms: &HashMap<String, Vec<usize>>, assigned: &HashSet<String>) -> Vec<usize> {
        match e {
            Expr::Str(s) => vec![self.str_code(s)],
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                if assigned.contains(&f) {
                    doms.get(&f).cloned().unwrap_or_default()
                } else if self.text_params.contains(&f) {
                    vec![TEXT_ARG]
                } else {
                    self.str_vars.get(&f).cloned().unwrap_or_default()
                }
            }
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    let mut a = self.text_codes(&args[1], doms, assigned);
                    a.extend(self.text_codes(&args[2], doms, assigned));
                    a
                } else {
                    self.str_funcs
                        .get(&f)
                        .map(|d| d.iter().copied().filter(|c| *c != MIXED_FN).collect())
                        .unwrap_or_default()
                }
            }
            _ => Vec::new(),
        }
    }

    /// The locals of a program that only ever hold texts, with the codes each can have. A local that
    /// holds a text on one path and a number on another is not one (a code cannot be told from the
    /// number), nor is a local that is also written by element, by slice or as a loop variable.
    pub(super) fn scan_text_locals(&mut self, stmts: &[PStmt], skip: &[String]) -> HashMap<String, Vec<usize>> {
        let mut list: Vec<(String, &Expr)> = Vec::new();
        let mut other: HashSet<String> = HashSet::new();
        assignments(stmts, &mut list, &mut other);
        let assigned: HashSet<String> = list.iter().map(|(n, _)| n.clone()).collect();
        // The greatest set of locals whose every assignment is a text, given that set.
        let mut locs: HashSet<String> = assigned
            .iter()
            .filter(|n| !other.contains(*n) && !skip.contains(n))
            .cloned()
            .collect();
        loop {
            let bad: Vec<String> = list
                .iter()
                .filter(|(n, e)| locs.contains(n) && !self.text_like(e, &locs, &assigned))
                .map(|(n, _)| n.clone())
                .collect();
            if bad.is_empty() {
                break;
            }
            for b in bad {
                locs.remove(&b);
            }
        }
        // Only those that get a text from something real: a literal, a text of the sheet.
        let mut proven: HashSet<String> = HashSet::new();
        loop {
            let add: Vec<String> = list
                .iter()
                .filter(|(n, e)| locs.contains(n) && !proven.contains(n) && self.text_rooted(e, &proven, &assigned))
                .map(|(n, _)| n.clone())
                .collect();
            if add.is_empty() {
                break;
            }
            proven.extend(add);
        }
        // The codes each can have (a local read from another has that one's codes).
        let mut doms: HashMap<String, Vec<usize>> = proven.iter().map(|n| (n.clone(), Vec::new())).collect();
        for _ in 0..=proven.len() {
            let mut changed = false;
            for (n, e) in &list {
                if !proven.contains(n) {
                    continue;
                }
                let add = self.text_codes(e, &doms, &assigned);
                let d = doms.entry(n.clone()).or_default();
                for c in add {
                    if !d.contains(&c) {
                        d.push(c);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for d in doms.values_mut() {
            d.sort_unstable();
            d.dedup();
        }
        doms
    }

    /// The value assigned to a local where the value is a text. `None` when it is not one (the
    /// ordinary translation applies). A text given to a local that also holds a number on some path
    /// is a gap that says so.
    pub(super) fn text_rhs(&mut self, flat: &str, e: &Expr) -> Option<String> {
        if self.readable || self.display {
            return None;
        }
        if self.str_locs.contains_key(flat) {
            return self.is_stringy(e).then(|| self.str_or_expr(e).s);
        }
        if has_text_leaf(e) {
            let what = if flat == RESULT_LOCAL {
                "the program gives a text on some paths and a number on others".to_string()
            } else {
                format!("{flat} holds a text on one path of the program and a number on another")
            };
            self.issue(Code::StringValue, Some(what));
            self.io_args = true;
            let r = self.expr(e);
            self.io_args = false;
            return Some(r.s);
        }
        None
    }

    /// The value assigned to an element of an array in a program. A text there is a gap that says
    /// what is not read: a vector of texts is held only when it is the program's own value and
    /// every element is a text.
    pub(super) fn element_rhs(&mut self, base: &str, e: &Expr) -> String {
        if !self.readable && !self.display && has_text_leaf(e) {
            self.issue(
                Code::StringValue,
                Some(format!(
                    "a text assigned to an element of {base}, which is not the program's value as a vector of texts (a vector that mixes texts and numbers cannot be held)"
                )),
            );
            self.io_args = true;
            let r = self.expr(e).s;
            self.io_args = false;
            return r;
        }
        self.expr(e).s
    }

    /// The value a program gives, and whether it is a text (then its codes are left in
    /// `prog_text`). A value that is a text on some paths and a number on others is a gap.
    pub(super) fn program_value(&mut self, tail: &Expr) -> R {
        self.prog_text = None;
        if self.readable || self.display || !self.is_stringy(tail) {
            return self.expr(tail);
        }
        if self.is_mixed(tail) {
            self.issue(
                Code::StringValue,
                Some("the program gives a text on some paths and a number on others".into()),
            );
            return self.expr(tail);
        }
        let d = self.str_domain(tail);
        self.prog_text = Some(d);
        self.str_or_expr(tail)
    }
}

/// What one cell of a table that mixes texts and numbers holds.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Cell {
    Num,
    /// A text, with the codes it can have.
    Text(Vec<usize>),
    /// Something that may be either (a name that holds a text or a number).
    Unknown,
}

impl Tr {
    /// The cells of a matrix literal (or its transpose), by row and column.
    pub(super) fn text_cells(&mut self, e: &Expr) -> Option<Vec<Vec<Cell>>> {
        match e {
            Expr::Un(UnOp::Transpose, a) => {
                let m = self.text_cells(a)?;
                let (r, c) = (m.len(), m.first().map_or(0, |row| row.len()));
                Some((0..c).map(|j| (0..r).map(|i| m[i][j].clone()).collect()).collect())
            }
            Expr::Matrix { rows, cols, items } if items.len() == rows * cols && *rows > 0 && *cols > 0 => {
                let mut cell = Vec::new();
                for it in items {
                    cell.push(match it {
                        Expr::Str(s) => Cell::Text(vec![self.str_code(s)]),
                        Expr::Name(n) => {
                            let f = tables::normalise_name(&n.flat());
                            if self.is_stringy(it) {
                                if self.str_mixed.contains(&f) {
                                    Cell::Unknown
                                } else {
                                    Cell::Text(self.str_domain(it))
                                }
                            } else {
                                Cell::Num
                            }
                        }
                        _ => Cell::Num,
                    });
                }
                // The file stores items column by column.
                Some(
                    (0..*rows)
                        .map(|r| (0..*cols).map(|c| cell[c * rows + r].clone()).collect())
                        .collect(),
                )
            }
            _ => None,
        }
    }

    /// The lines that print a table of texts and numbers: a text cell is its text, a number cell
    /// the number as the engine shows it.
    pub(super) fn mixed_table_lines(&self, head: &str, name: &str, cells: &[Vec<Cell>]) -> String {
        let ats: Vec<Vec<String>> = cells
            .iter()
            .enumerate()
            .map(|(r, row)| {
                (0..row.len())
                    .map(|c| {
                        // A single column is a vector for the engine: one index.
                        if row.len() == 1 {
                            format!("{name}.({})", r + 1)
                        } else {
                            format!("{name}.({}; {})", r + 1, c + 1)
                        }
                    })
                    .collect()
            })
            .collect();
        self.table_lines(head, cells, &ats)
    }

    /// The same for a table of the cells' own expressions (a matrix written out).
    pub(super) fn table_lines(&self, head: &str, cells: &[Vec<Cell>], ats: &[Vec<String>]) -> String {
        let mut out = format!("'<div class=\"imp-eq\"><math class=\"imp-m\">{head}<mrow><mo>[</mo><mtable>\n");
        for (r, row) in cells.iter().enumerate() {
            out.push_str("'<mtr>\n");
            for (c, cell) in row.iter().enumerate() {
                let at = ats[r][c].clone();
                match cell {
                    Cell::Text(codes) if codes.len() == 1 => {
                        let t = crate::mathml::esc(&self.str_table[codes[0] - 1]);
                        out.push_str(&format!("'<mtd><ms>{t}</ms></mtd>\n"));
                    }
                    Cell::Text(codes) => {
                        out.push_str("'<mtd>\n");
                        for (k, &d) in codes.iter().enumerate() {
                            let kw = if k == 0 { "#if" } else { "#else if" };
                            let t = crate::mathml::esc(&self.str_table[d - 1]);
                            out.push_str(&format!("{kw} {at} ≡ {d}\n'<ms>{t}</ms>\n"));
                        }
                        out.push_str("#end if\n'</mtd>\n");
                    }
                    _ => out.push_str(&format!("'<mtd><mtext>\n#val\n{at}\n#end val\n'</mtext></mtd>\n")),
                }
            }
            out.push_str("'</mtr>\n");
        }
        out.push_str("'</mtable><mo>]</mo></mrow></math></div>");
        out
    }

    /// An element of a table that mixes texts and numbers, written with literal indexes at a cell that
    /// holds a text, would be read as the number that is its code: a gap.
    pub(super) fn check_table_index(&mut self, base: &Expr, idx: &[Expr]) {
        let Expr::Name(n) = base else { return };
        let f = tables::normalise_name(&n.flat());
        let Some(cells) = self.str_mat_cells.get(&f) else {
            return;
        };
        let lit = |e: &Expr| match e {
            Expr::Num(s) => s.trim().parse::<i64>().ok().map(|v| v - self.origin),
            _ => None,
        };
        let one_col = cells.first().is_some_and(|row| row.len() == 1);
        let (r, c) = match idx {
            [i, j] => match (lit(i), lit(j)) {
                (Some(r), Some(c)) => (r, c),
                _ => return,
            },
            // One index into a single column (or a single row): the element in order.
            [i] => match lit(i) {
                Some(k) if one_col => (k, 0),
                Some(k) if cells.len() == 1 => (0, k),
                _ => return,
            },
            _ => return,
        };
        let cell = usize::try_from(r)
            .ok()
            .and_then(|r| cells.get(r))
            .and_then(|row| usize::try_from(c).ok().and_then(|c| row.get(c)));
        if matches!(cell, Some(Cell::Text(_) | Cell::Unknown)) {
            self.issue(
                Code::StringValue,
                Some(format!(
                    "an element of {f}, a table of texts and numbers, that holds a text"
                )),
            );
        }
    }
}
