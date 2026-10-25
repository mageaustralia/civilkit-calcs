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
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                self.str_vars.contains_key(&f) && !self.params.contains(&f) && !self.locals.contains_key(&f)
            }
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

    /// The codes a text expression can have.
    pub(super) fn str_domain(&mut self, e: &Expr) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        match e {
            Expr::Str(s) => out.push(self.str_code(s)),
            Expr::Name(n) => {
                let f = tables::normalise_name(&n.flat());
                if let Some(d) = self.str_vars.get(&f) {
                    out.extend(d.iter().copied());
                }
            }
            Expr::Call(n, args) => {
                let f = tables::normalise_name(&n.flat());
                if n.sub.is_none() && f == "if" && args.len() == 3 && !self.funcs.contains_key(&f) {
                    out.extend(self.str_domain(&args[1]));
                    out.extend(self.str_domain(&args[2]));
                } else if let Some(d) = self.str_funcs.get(&f) {
                    out.extend(d.iter().copied());
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
                if self.str_vars.contains_key(&f) && !self.params.contains(&f) && !self.locals.contains_key(&f) {
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
                    (true, false)
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
