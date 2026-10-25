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

/// The regions of the solve block `c`, or `None` when its solution was not saved in a form that
/// can be read (every region of the solver must hold one).
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
    let mut solved = false;
    for (cat, unit, regions) in parts {
        for mut r in regions {
            let RegionKind::Math(m) = &mut r.kind else {
                out.push(r);
                continue;
            };
            match cat.as_deref() {
                Some("solver") => {
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
                    m.stmt = Stmt::Define {
                        target: target.clone(),
                        value,
                        global: false,
                    };
                    m.frozen = true;
                    m.solve = Some(SolveRole::Solution);
                    solved = true;
                }
                Some("constraint") => m.solve = Some(SolveRole::Constraint),
                _ => {}
            }
            out.push(r);
        }
    }
    solved.then_some(out)
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
