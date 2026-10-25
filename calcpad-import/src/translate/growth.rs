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
    out
}

impl Tr {
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
