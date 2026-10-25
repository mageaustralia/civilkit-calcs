//! `[a, b] := program`: several names defined at once from a program that ends in a matrix of its
//! locals (`return [shape, hole]`). Each name is defined by running the program with that local as
//! its value; the program is run once per name, which is slower but gives the same numbers.

use super::*;

/// The locals a program returns as `[a, b, ...]` (a row, a column or its transpose), when it does.
fn returned_locals(stmts: &[PStmt]) -> Option<Vec<Name>> {
    let Some(PStmt::Expr(last) | PStmt::Return(last)) = stmts.last() else {
        return None;
    };
    let m = match last {
        Expr::Un(UnOp::Transpose, inner) => &**inner,
        other => other,
    };
    let Expr::Matrix { rows, cols, items } = m else {
        return None;
    };
    if *rows != 1 && *cols != 1 {
        return None;
    }
    items
        .iter()
        .map(|i| match i {
            Expr::Name(n) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

impl Tr {
    /// The text that defines each of `flats` from the program, or why that cannot be done.
    pub(super) fn multi_from_program(&mut self, stmts: &[PStmt], flats: &[String]) -> Result<Vec<String>, String> {
        let locals = returned_locals(stmts).ok_or("a program that does not end in a matrix of its locals")?;
        if locals.len() != flats.len() {
            return Err("the program returns a different number of values than there are names".into());
        }
        let mut out = Vec::new();
        for (flat, local) in flats.iter().zip(&locals) {
            let mut one: Vec<PStmt> = stmts[..stmts.len() - 1].to_vec();
            one.push(PStmt::Expr(Expr::Name(local.clone())));
            let (text, _) = self.lower_program(&one, flat)?;
            out.push(text);
        }
        Ok(out)
    }
}
