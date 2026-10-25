//! Arrays a function body grows by writing elements: the array starts with one zero (or is the
//! argument the function received) and each write first extends it to hold the index, as the source
//! does, so the array ends as large as the largest index written.
//!
//! The expression block has no conditional statements: a write under a guard `g` extends the array
//! only when `g` holds and, when it does not, writes the array's first element back to itself.

use super::*;

/// What the statements of a function body do to one array by element.
#[derive(Debug, Default, Clone)]
pub(super) struct WriteScan {
    /// The statement of the body (by position) that first writes it.
    pub first: usize,
    /// The number of indexes each write has.
    pub ranks: Vec<usize>,
    /// Some write is not under an `if`.
    pub uncond: bool,
    /// Some write is a whole column or row, which the sizes of the element writes do not cover.
    pub has_slice: bool,
    /// Every read of the array comes after a write that ran on the way to it, so a path that never
    /// writes the array never reads it.
    pub reads_after_writes: bool,
}

/// The arrays a body writes by element: where, with how many indexes, and whether always.
pub(super) fn scan_element_writes(body: &[PStmt]) -> HashMap<String, WriteScan> {
    fn walk(list: &[PStmt], at: usize, cond: bool, out: &mut HashMap<String, WriteScan>) {
        for st in list {
            match st {
                PStmt::Local(PTarget::Element { base, index }, _) => {
                    let e = out
                        .entry(tables::normalise_name(&base.flat()))
                        .or_insert_with(|| WriteScan {
                            first: at,
                            ..WriteScan::default()
                        });
                    if !e.ranks.contains(&index.len()) {
                        e.ranks.push(index.len());
                    }
                    e.uncond |= !cond;
                }
                PStmt::Local(PTarget::Slice { base, .. }, _) => {
                    let e = out
                        .entry(tables::normalise_name(&base.flat()))
                        .or_insert_with(|| WriteScan {
                            first: at,
                            ..WriteScan::default()
                        });
                    if !e.ranks.contains(&2) {
                        e.ranks.push(2);
                    }
                    e.uncond |= !cond;
                    e.has_slice = true;
                }
                PStmt::For { body, .. } | PStmt::While { body, .. } => walk(body, at, cond, out),
                PStmt::If { branches, otherwise } => {
                    for (_, b) in branches {
                        walk(b, at, true, out);
                    }
                    if let Some(o) = otherwise {
                        walk(o, at, true, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = HashMap::new();
    for (k, st) in body.iter().enumerate() {
        walk(std::slice::from_ref(st), k, false, &mut out);
    }
    // A write under an `if` with an `else` counts as always run when every arm writes the array.
    for (name, info) in out.iter_mut() {
        info.uncond = info.uncond || body.iter().any(|st| writes_always(st, name));
        info.reads_after_writes = reads_follow_writes(body, name, false);
    }
    out
}

/// Names read by a statement (not the array a target writes), anywhere inside it.
fn reads(st: &PStmt, name: &str) -> bool {
    let has = |e: &Expr| {
        let mut c = e.clone();
        let mut found = false;
        c.for_each_name_mut(&mut |n| found |= tables::normalise_name(&n.flat()) == name);
        found
    };
    match st {
        PStmt::Local(PTarget::Var(_), e) => has(e),
        PStmt::Local(PTarget::Element { index, .. }, e) => has(e) || index.iter().any(has),
        PStmt::Local(PTarget::Slice { index, .. }, e) => has(e) || has(index),
        PStmt::For { range, body, .. } => has(range) || body.iter().any(|s| reads(s, name)),
        PStmt::While { cond, body } => has(cond) || body.iter().any(|s| reads(s, name)),
        PStmt::If { branches, otherwise } => {
            branches.iter().any(|(c, b)| has(c) || b.iter().any(|s| reads(s, name)))
                || otherwise.iter().any(|b| b.iter().any(|s| reads(s, name)))
        }
        PStmt::Return(e) | PStmt::Expr(e) => has(e),
        PStmt::Break | PStmt::Continue | PStmt::Unsupported(_) => false,
    }
}

/// Whether every read of `name` in the list has a write before it on the way (`written`: one already
/// ran). A write counts in the list it is in; an `if` arm that writes does not make the array written
/// after the `if` unless every arm does.
fn reads_follow_writes(list: &[PStmt], name: &str, mut written: bool) -> bool {
    for st in list {
        match st {
            PStmt::If { branches, otherwise } => {
                for (c, b) in branches {
                    let cond_reads = reads(&PStmt::Expr(c.clone()), name);
                    if cond_reads && !written {
                        return false;
                    }
                    if !reads_follow_writes(b, name, written) {
                        return false;
                    }
                }
                if let Some(o) = otherwise {
                    if !reads_follow_writes(o, name, written) {
                        return false;
                    }
                }
                written = written || writes_always(st, name);
            }
            PStmt::For { body, .. } | PStmt::While { body, .. } => {
                let header_reads = match st {
                    PStmt::For { range, .. } => reads(&PStmt::Expr(range.clone()), name),
                    PStmt::While { cond, .. } => reads(&PStmt::Expr(cond.clone()), name),
                    _ => false,
                };
                if header_reads && !written {
                    return false;
                }
                if !reads_follow_writes(body, name, written) {
                    return false;
                }
                written = written || writes_always(st, name);
            }
            PStmt::Local(PTarget::Element { base, .. } | PTarget::Slice { base, .. }, _) => {
                // The value is evaluated before the write.
                if reads(st, name) && !written {
                    return false;
                }
                if tables::normalise_name(&base.flat()) == name {
                    written = true;
                }
            }
            other => {
                if reads(other, name) && !written {
                    return false;
                }
            }
        }
    }
    true
}

/// Whether a statement writes the array on every path through it.
fn writes_always(st: &PStmt, name: &str) -> bool {
    match st {
        PStmt::Local(PTarget::Element { base, .. } | PTarget::Slice { base, .. }, _) => {
            tables::normalise_name(&base.flat()) == name
        }
        PStmt::For { body, .. } | PStmt::While { body, .. } => body.iter().any(|s| writes_always(s, name)),
        PStmt::If { branches, otherwise } => {
            otherwise.is_some()
                && branches.iter().all(|(_, b)| b.iter().any(|s| writes_always(s, name)))
                && otherwise.iter().all(|b| b.iter().any(|s| writes_always(s, name)))
        }
        _ => false,
    }
}

impl Tr {
    /// Whether the value of `e` in a function body is an array: a matrix literal, a function that makes
    /// one, arithmetic on an array the body holds (`arrays`: source names), or an `if` of arrays.
    pub(super) fn fn_gives_array(&self, e: &Expr, arrays: &HashSet<String>) -> bool {
        match e {
            Expr::Matrix { .. } => true,
            Expr::Name(n) => arrays.contains(&tables::normalise_name(&n.flat())),
            Expr::Call(f, args) if f.base == "if" && f.sub.is_none() && args.len() == 3 => {
                self.fn_gives_array(&args[1], arrays) || self.fn_gives_array(&args[2], arrays)
            }
            Expr::Call(f, _) => {
                matches!(
                    f.base.as_str(),
                    "identity" | "matrix" | "augment" | "stack" | "lsolve" | "submatrix"
                ) || self.array_funcs.contains(&tables::normalise_name(&f.flat()))
            }
            Expr::Un(UnOp::Neg | UnOp::Transpose, a) => self.fn_gives_array(a, arrays),
            Expr::Bin(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, a, b) => {
                self.fn_gives_array(a, arrays) || self.fn_gives_array(b, arrays)
            }
            _ => false,
        }
    }

    /// The statements that write one element of a growing array, under the guard (if any).
    pub(super) fn growing_write(&self, mapped: &str, parts: &[String], rhs: &str, guard: Option<&str>) -> Vec<String> {
        let grown = if parts.len() == 1 {
            format!("resize({mapped}; max(len({mapped}); {}))", parts[0])
        } else {
            format!(
                "mresize({mapped}; max(n_rows({mapped}); {}); max(n_cols({mapped}); {}))",
                parts[0], parts[1]
            )
        };
        match guard {
            None => vec![
                format!("{mapped} = {grown}"),
                format!("{mapped}.({}) = {rhs}", parts.join("; ")),
            ],
            Some(g) => {
                let safe: Vec<String> = parts.iter().map(|p| format!("if({g}; {p}; 1)")).collect();
                let first = if parts.len() == 1 {
                    format!("{mapped}.(1)")
                } else {
                    format!("{mapped}.(1; 1)")
                };
                vec![
                    format!("{mapped} = if({g}; {grown}; {mapped})"),
                    format!("{mapped}.({}) = if({g}; {rhs}; {first})", safe.join("; ")),
                ]
            }
        }
    }
}
