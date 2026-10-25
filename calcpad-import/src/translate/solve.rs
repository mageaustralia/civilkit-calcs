//! Solve blocks: `Given` constraints and `Find(x)`, read from the regions of an `.mcdx` solve block.
//!
//! The source's solver looks for the solution nearest the guess (the value the unknown has when the
//! block starts). That is a secant iteration here, written as an expression block of the engine:
//! start at the guess, step by a millionth of it, and iterate `c = b - f(b)(b - a)/(f(b) - f(a))` until
//! the step is a trillionth of the value. Two unknowns with two equations are Newton's method with
//! a difference quotient for the Jacobian. A result is kept only when the equations hold at it
//! (their two sides agree to a hundred-millionth of their size); otherwise it is undefined, never a
//! number that is not a solution. The source stops at its own tolerance (0.001 by default), so its
//! last digits can differ from this exact root.

use super::*;

/// The residual of each equation in the unknowns, as functions of the engine.
struct Residual {
    /// `sb_f(x; ...)`: left side minus right side.
    f: String,
    /// `sb_l(x; ...)` and `sb_r(x; ...)`, for the check that the equation holds.
    l: String,
    r: String,
}

impl Tr {
    /// The text that defines the unknown(s) as the solution of the block's constraints.
    pub(super) fn solve_block(
        &mut self,
        target: &Target,
        value: &Expr,
        saved: Option<&Stmt>,
    ) -> Result<(String, Vec<String>), String> {
        // `Find(x)` or `Find(x) * unit`: a unit the result is multiplied by.
        let (call, factor): (&Expr, Option<&Expr>) = match value {
            Expr::Bin(BinOp::Mul, a, b) if matches!(&**a, Expr::Call(f, _) if f.base == "Find") => (a, Some(b)),
            Expr::Bin(BinOp::Mul, a, b) if matches!(&**b, Expr::Call(f, _) if f.base == "Find") => (b, Some(a)),
            other => (other, None),
        };
        let Expr::Call(f, args) = call else {
            return Err("the solver region is not a call of Find".into());
        };
        if f.base != "Find" {
            return Err(format!("{} is not read (only Find)", f.base));
        }
        let cons = std::mem::take(&mut self.solve_cons);
        let mut unknowns: Vec<String> = Vec::new();
        for a in args {
            let Expr::Name(n) = a else {
                return Err("Find of something that is not a name".into());
            };
            let flat = tables::normalise_name(&n.flat());
            if !valid_ident(&flat) {
                return Err("an unknown whose name cannot be spelled".into());
            }
            if !self.defined.contains(&flat) {
                return Err(format!("{flat} has no value to start from (the source needs a guess)"));
            }
            unknowns.push(flat);
        }
        let targets: Vec<String> = match target {
            Target::Var(n) => vec![tables::normalise_name(&n.flat())],
            Target::Multi(ns) => ns.iter().map(|n| tables::normalise_name(&n.flat())).collect(),
            _ => return Err("the solution is not assigned to a name".into()),
        };
        if factor.is_some() && unknowns.len() != 1 {
            return Err("a unit on the solutions of several unknowns".into());
        }
        if targets.len() != unknowns.len() || !(1..=2).contains(&unknowns.len()) {
            return Err("one or two unknowns are read".into());
        }
        if cons.len() != unknowns.len() {
            return Err(format!("{} equations for {} unknowns", cons.len(), unknowns.len()));
        }
        let mut eqs: Vec<(Expr, Expr)> = Vec::new();
        for c in &cons {
            match c {
                Expr::Bin(BinOp::Eq, a, b) => eqs.push(((**a).clone(), (**b).clone())),
                _ => return Err("a constraint that is not an equation".into()),
            }
        }
        self.prog_n += 1;
        let uid = self.prog_n;
        // Where solving here fails the check, the value the file saved for that unknown (frozen), when it
        // saved one; otherwise undefined.
        let fallbacks: Vec<String> = {
            let items: Option<Vec<Expr>> = match saved {
                Some(Stmt::Define { value, .. }) if factor.is_none() => match value {
                    Expr::Matrix { items, .. } if unknowns.len() > 1 && items.len() == unknowns.len() => {
                        Some(items.clone())
                    }
                    other if unknowns.len() == 1 => Some(vec![other.clone()]),
                    _ => None,
                },
                _ => None,
            };
            match items {
                Some(v) => {
                    self.issue(
                        Code::FrozenValue,
                        Some("used only where solving here does not verify".into()),
                    );
                    v.iter().map(|e| self.expr(e).s).collect()
                }
                None => vec!["sqrt(-1)".to_string(); unknowns.len()],
            }
        };
        // The functions: one per equation, in the unknowns as parameters.
        let saved_params = std::mem::replace(&mut self.params, unknowns.clone());
        let mut lines: Vec<String> = Vec::new();
        let mut res: Vec<Residual> = Vec::new();
        let plist = unknowns.join("; ");
        for (k, (l, r)) in eqs.iter().enumerate() {
            let (ls, rs) = (self.expr(l).s, self.expr(r).s);
            let (fl, fr, ff) = (
                format!("sb_l{uid}_{k}"),
                format!("sb_r{uid}_{k}"),
                format!("sb_f{uid}_{k}"),
            );
            lines.push(format!("{fl}({plist}) = {ls}"));
            lines.push(format!("{fr}({plist}) = {rs}"));
            lines.push(format!("{ff}({plist}) = {fl}({plist}) - {fr}({plist})"));
            res.push(Residual { f: ff, l: fl, r: fr });
        }
        self.params = saved_params;
        if unknowns.len() == 1 {
            let u = &unknowns[0];
            let ff = &res[0];
            let a = format!("sb_a{uid}");
            let b = format!("sb_b{uid}");
            let fa = format!("sb_fa{uid}");
            let fb = format!("sb_fb{uid}");
            let n = format!("sb_n{uid}");
            let block = format!(
                "$Block{{{a} = {u}; {b} = {a} + 0.000001*(abs({a}) + getunits({a})); {fa} = {f}({a}); {fb} = {f}({b}); {n} = 0; \
                 $While{{if({n} < 100; if(abs({b} - {a}) > 0.000000000001*(abs({b}) + getunits({b})); if({fb} ≠ {fa}; 1; 0); 0); 0); \
                 {a}2 = {b} - {fb}*({b} - {a})/({fb} - {fa}); {a} = {b}; {fa} = {fb}; {b} = {a}2; {fb} = {f}({b}); {n} = {n} + 1}}; \
                 if(abs({fb}) ≤ 0.00000001*(abs({l}({b})) + abs({r}({b}))); {b}; {fallback})}}",
                f = ff.f,
                l = ff.l,
                r = ff.r,
                fallback = fallbacks[0],
            );
            let times = match factor {
                Some(u) => format!("*({})", self.expr(u).s),
                None => String::new(),
            };
            lines.push(format!("{} = {block}{times}", targets[0]));
        } else {
            // Newton's method in two unknowns; each is solved for separately from the same start.
            for (which, t) in targets.iter().enumerate() {
                let (u1, u2) = (&unknowns[0], &unknowns[1]);
                let (f1, f2) = (&res[0], &res[1]);
                let id = |s: &str| format!("sb_{s}{uid}_{which}");
                let (a1, a2, n, ok) = (id("a1"), id("a2"), id("n"), id("ok"));
                let (v1, v2, h1, h2) = (id("v1"), id("v2"), id("h1"), id("h2"));
                let (j11, j12, j21, j22, det, d1, d2) = (
                    id("j11"),
                    id("j12"),
                    id("j21"),
                    id("j22"),
                    id("det"),
                    id("d1"),
                    id("d2"),
                );
                let call = |f: &str, x: &str, y: &str| format!("{f}({x}; {y})");
                let body = format!(
                    "{v1} = {f1v}; {v2} = {f2v}; \
                     {h1} = 0.000001*(abs({a1}) + getunits({a1})); {h2} = 0.000001*(abs({a2}) + getunits({a2})); \
                     {j11} = ({f1a} - {v1})/{h1}; {j12} = ({f1b} - {v1})/{h2}; \
                     {j21} = ({f2a} - {v2})/{h1}; {j22} = ({f2b} - {v2})/{h2}; \
                     {det} = {j11}*{j22} - {j12}*{j21}; \
                     {d1} = ({v1}*{j22} - {v2}*{j12})/{det}; {d2} = ({j11}*{v2} - {j21}*{v1})/{det}; \
                     {a1} = {a1} - {d1}; {a2} = {a2} - {d2}; \
                     {ok} = if(abs({d1}) ≤ 0.000000000001*(abs({a1}) + getunits({a1})); if(abs({d2}) ≤ 0.000000000001*(abs({a2}) + getunits({a2})); 1; 0); 0); \
                     {n} = {n} + 1",
                    f1v = call(&f1.f, &a1, &a2),
                    f2v = call(&f2.f, &a1, &a2),
                    f1a = call(&f1.f, &format!("{a1} + {h1}"), &a2),
                    f1b = call(&f1.f, &a1, &format!("{a2} + {h2}")),
                    f2a = call(&f2.f, &format!("{a1} + {h1}"), &a2),
                    f2b = call(&f2.f, &a1, &format!("{a2} + {h2}")),
                );
                let check = |r: &Residual| {
                    format!(
                        "abs({f}({a1}; {a2})) ≤ 0.00000001*(abs({l}({a1}; {a2})) + abs({r}({a1}; {a2})))",
                        f = r.f,
                        l = r.l,
                        r = r.r
                    )
                };
                let pick = if which == 0 { &a1 } else { &a2 };
                let block = format!(
                    "$Block{{{a1} = {u1}; {a2} = {u2}; {n} = 0; {ok} = 0; \
                     $While{{if({n} < 100; if({ok} ≡ 0; 1; 0); 0); {body}}}; \
                     if({c1}; if({c2}; {pick}; {fb}); {fb})}}",
                    c1 = check(f1),
                    c2 = check(f2),
                    fb = fallbacks[which],
                );
                lines.push(format!("{t} = {block}"));
            }
        }
        Ok((lines.join("\n"), targets))
    }
}
