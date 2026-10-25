//! Solve blocks whose solution the file saved.
//!
//! A solve block (guess values, constraints and a solver such as `Find`) is not solved here. When
//! the file saved what the solver returned, the unknowns take those values, frozen, as the unknowns
//! of an input component do; the guesses stay definitions and the constraints are shown as written.
//! A block with no saved solution stays a region the importer does not know.

use crate::ir::*;
use crate::mcdx::Ctx;
use crate::xmcd::{is_ml, is_ws, local, ml_kids, read_expr, unwrap_provenance};
use roxmltree::Node;

/// The regions of the solve block `c`, or `None` when it has no solver region. A guess value is an
/// ordinary definition. The solver keeps its definition (solved here when the block has the shape that
/// is, see `translate::solve`) and carries, when the file saved what the solver returned, the
/// definition of the unknowns at those values for the blocks that are not solved here.
pub(crate) fn read_solve_block(ctx: &Ctx, c: Node, at: (f64, f64), depth: usize) -> Option<Vec<Region>> {
    let nested = c.children().find(|k| is_ws(*k, "regions"))?;
    let mut parts: Vec<(Option<String>, Option<Expr>, Vec<Region>)> = Vec::new();
    for n in nested.children().filter(|k| is_ws(*k, "region")) {
        let mut tmp = Vec::new();
        ctx.collect_region(n, depth + 1, Some(at), &mut tmp);
        parts.push((
            n.attribute("solve-block-category").map(str::to_string),
            override_unit(n, ctx),
            tmp,
        ));
    }
    let mut out: Vec<Region> = Vec::new();
    let mut solver = false;
    let mut shown_in_unit = false;
    let (mut equations, mut relations_only_eq, mut unknowns) = (0usize, true, 0usize);
    for (cat, unit, regions) in parts {
        for mut r in regions {
            let RegionKind::Math(m) = &mut r.kind else {
                out.push(r);
                continue;
            };
            match cat.as_deref() {
                Some("solver") => {
                    solver = true;
                    unknowns = match &m.stmt {
                        Stmt::Define {
                            target: Target::Var(_), ..
                        } => 1,
                        Stmt::Define {
                            target: Target::Multi(ns),
                            ..
                        } => ns.len(),
                        _ => 0,
                    };
                    let fallback = (|| {
                        let mut value = m.saved_expr.clone()?;
                        // A result shown in a unit of its own is stored as the number in that unit.
                        if !has_unit(&value) {
                            if let Some(u) = unit.clone() {
                                value = times_unit(value, u);
                            }
                        }
                        let Stmt::Define { target, .. } = &m.stmt else {
                            return None;
                        };
                        let value = match target {
                            Target::Multi(names) => spread(value, names.len())?,
                            Target::Var(_) => value,
                            _ => return None,
                        };
                        Some(Box::new(Stmt::Define {
                            target: target.clone(),
                            value,
                            global: false,
                        }))
                    })();
                    if unit.is_some() {
                        // A result shown in a unit of its own: the file's saved value (the number in that unit)
                        // is the answer; solving here would not know what dimension the number has.
                        shown_in_unit = true;
                        match fallback {
                            Some(f) => {
                                m.stmt = *f;
                                m.frozen = true;
                                m.solve = Some(SolveRole::Solution);
                            }
                            None => {
                                m.stmt = Stmt::Unsupported {
                                    code: crate::report::Code::SolveBlock,
                                    detail: Some("the result is shown in a unit and the file saved no solution".into()),
                                };
                                m.solve = None;
                            }
                        }
                    } else {
                        m.solve = Some(SolveRole::Solver(fallback));
                    }
                }
                Some("constraint") => {
                    equations += 1;
                    relations_only_eq &= matches!(&m.stmt, Stmt::Bare(Expr::Bin(BinOp::Eq, ..)));
                    m.solve = Some(SolveRole::Constraint { live: false });
                }
                _ => {}
            }
            out.push(r);
        }
    }
    // A block of the shape that is solved here: as many equations as unknowns, one or two, all equations.
    let live = !shown_in_unit && relations_only_eq && equations == unknowns && (1..=2).contains(&unknowns);
    for r in out.iter_mut() {
        if let RegionKind::Math(m) = &mut r.kind {
            if let Some(SolveRole::Constraint { live: l }) = &mut m.solve {
                *l = live;
            }
        }
    }
    solver.then_some(out)
}

/// Whether a stored value carries its unit (a number or a matrix times a unit).
fn has_unit(e: &Expr) -> bool {
    matches!(e, Expr::Bin(BinOp::Mul, ..))
}

fn times_unit(value: Expr, u: Expr) -> Expr {
    Expr::Bin(BinOp::Mul, Box::new(value), Box::new(u))
}

/// The unit a solver region shows its result in (`x := Find(..) =` with a unit typed after it).
fn override_unit(region: Node, ctx: &Ctx) -> Option<Expr> {
    let ev = region.descendants().find(|d| is_ml(*d) && local(*d) == "eval")?;
    let slot = ml_kids(ev).into_iter().find(|k| local(*k) == "unitOverride")?;
    let u = ml_kids(slot).into_iter().find(|u| local(*u) != "placeholder")?;
    Some(read_expr(unwrap_provenance(u), ctx.lim, 0))
}

/// A column of `n` values with one unit (`[a; b] * unit`) as the column of `a * unit; b * unit`.
fn spread(value: Expr, n: usize) -> Option<Expr> {
    match value {
        Expr::Matrix { rows, cols, items } if items.len() == n && (rows == 1 || cols == 1) => {
            Some(Expr::Matrix { rows, cols, items })
        }
        Expr::Bin(BinOp::Mul, a, u) => match *a {
            Expr::Matrix { rows, cols, items } if items.len() == n && (rows == 1 || cols == 1) => {
                let items = items
                    .into_iter()
                    .map(|i| Expr::Bin(BinOp::Mul, Box::new(i), u.clone()))
                    .collect();
                Some(Expr::Matrix { rows, cols, items })
            }
            _ => None,
        },
        _ => None,
    }
}
