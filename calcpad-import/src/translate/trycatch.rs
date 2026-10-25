//! `try expr on error fallback`: the engine has no way to catch an error, so the conversion tests
//! beforehand that `expr` cannot fail, and gives `fallback` when the test does not hold. That is
//! faithful only when `expr` can fail in ways that can be tested: a division by zero. The body may
//! use worksheet functions whose own body is one expression (their definition is substituted, so a
//! division inside is tested too, and only on the branch of an `if` that runs).

use super::*;

/// The deepest chain of worksheet functions calling one another that is looked through.
const MAX_CALL_DEPTH: usize = 8;

impl Tr {
    /// The tests that must all hold for `e` to evaluate without failing, or why that cannot be tested.
    pub(super) fn safe_tests(&mut self, e: &Expr, depth: usize) -> Result<Vec<String>, String> {
        match e {
            Expr::Num(_) | Expr::Name(_) => Ok(Vec::new()),
            Expr::Un(UnOp::Neg | UnOp::Abs, a) | Expr::Postfix(_, a) => self.safe_tests(a, depth),
            Expr::Bin(
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Eq
                | BinOp::Ne
                | BinOp::Lt
                | BinOp::Gt
                | BinOp::Le
                | BinOp::Ge
                | BinOp::And
                | BinOp::Or
                | BinOp::Xor,
                a,
                b,
            ) => {
                let mut t = self.safe_tests(a, depth)?;
                t.extend(self.safe_tests(b, depth)?);
                Ok(t)
            }
            Expr::Bin(BinOp::Div, a, b) => {
                let mut t = self.safe_tests(a, depth)?;
                t.extend(self.safe_tests(b, depth)?);
                if !matches!(&**b, Expr::Num(s) if s.trim().parse::<f64>().is_ok_and(|v| v != 0.0)) {
                    let d = self.expr(b).s;
                    // A zero keeps its unit when multiplied by 0, so the comparison is between like units.
                    t.push(format!("({d})*0 ≠ ({d})"));
                }
                Ok(t)
            }
            Expr::Bin(BinOp::Pow, a, b) if matches!(&**b, Expr::Num(s) if is_int_literal(s).is_some_and(|v| v >= 0)) => {
                self.safe_tests(a, depth)
            }
            Expr::Index(base, idx) if matches!(&**base, Expr::Name(_)) => {
                let mut t = Vec::new();
                for i in idx {
                    t.extend(self.safe_tests(i, depth)?);
                }
                Ok(t)
            }
            Expr::Call(f, args) if f.sub.is_none() && f.base == "if" && args.len() == 3 => {
                let mut t = self.safe_tests(&args[0], depth)?;
                let (a, b) = (self.safe_tests(&args[1], depth)?, self.safe_tests(&args[2], depth)?);
                if !a.is_empty() || !b.is_empty() {
                    // A division on the branch that does not run is not a failure.
                    let c = self.expr(&args[0]).s;
                    let all = |v: &[String]| if v.is_empty() { "1".to_string() } else { v.join(" ∧ ") };
                    t.push(format!("if({c}; {}; {})", all(&a), all(&b)));
                }
                Ok(t)
            }
            // Functions the source defines for every number (no domain to fail on).
            Expr::Call(f, args)
                if f.sub.is_none()
                    && !self.funcs.contains_key(&tables::normalise_name(&f.flat()))
                    && matches!(
                        f.base.as_str(),
                        "max"
                            | "min"
                            | "floor"
                            | "ceil"
                            | "ceiling"
                            | "round"
                            | "trunc"
                            | "sign"
                            | "abs"
                            | "sin"
                            | "cos"
                            | "atan"
                    ) =>
            {
                let mut t = Vec::new();
                for a in args {
                    t.extend(self.safe_tests(a, depth)?);
                }
                Ok(t)
            }
            Expr::Call(f, args) => {
                let flat = tables::normalise_name(&f.flat());
                let def = match (self.fn_defs.get(&flat), self.funcs.get(&flat)) {
                    (Some((params, body)), Some(&arity)) if arity == args.len() && params.len() == arity => {
                        Some((params.clone(), body.clone()))
                    }
                    _ => None,
                };
                let Some((params, body)) = def else {
                    return Err("the body calls a function, which could fail in ways other than a division".into());
                };
                if depth >= MAX_CALL_DEPTH {
                    return Err("the body calls worksheet functions nested too deeply to test".into());
                }
                let mut t = Vec::new();
                for a in args {
                    t.extend(self.safe_tests(a, depth)?);
                }
                let env: HashMap<String, Expr> = params.into_iter().zip(args.iter().cloned()).collect();
                let Some(inlined) = subst(&body, &env) else {
                    return Err("the body calls a function that cannot be looked through".into());
                };
                t.extend(self.safe_tests(&inlined, depth + 1)?);
                Ok(t)
            }
            _ => Err("the body could fail in ways other than a division".into()),
        }
    }

    pub(super) fn try_catch(&mut self, body: &Expr, rendered: &[String]) -> R {
        let tests = match self.safe_tests(body, 0) {
            Ok(t) => t,
            Err(why) => {
                self.issue(Code::ProgramBlock, Some(format!("tryCatch: {why}")));
                if self.readable {
                    self.readable_bad = true;
                }
                return self.stand_in(&format!("tryCatch({}; {})", rendered[0], rendered[1]), "?try?");
            }
        };
        let mut unique: Vec<String> = Vec::new();
        for t in tests {
            if !unique.contains(&t) {
                unique.push(t);
            }
        }
        if unique.is_empty() {
            return R {
                s: rendered[0].clone(),
                prec: P_ATOM,
            };
        }
        atom(format!("if({}; {}; {})", unique.join(" ∧ "), rendered[0], rendered[1]))
    }
}
