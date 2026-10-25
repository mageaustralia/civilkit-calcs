//! Early exits in function bodies: `return` before the end, `break` and `continue`.
//!
//! A function body is one expression block, with no jump. An exit is a flag: `return e` sets the
//! result and a done flag, `break` sets the flag of the loop it leaves, `continue` the flag of the
//! pass it ends. After a statement that can set a flag, the rest of its list runs only while no flag
//! is set (a guard variable, as the arms of an `if` have), a `while` tests the flags before its
//! condition, and a `for` runs its remaining passes with every statement guarded off.

use super::*;

/// What a list of statements can do to the flow around it.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub(super) struct Exits {
    /// A `return` anywhere inside (a loop does not stop one).
    pub ret: bool,
    /// A `break` that leaves the loop enclosing this list (not one in a loop inside it).
    pub brk: bool,
    /// A `continue` of the loop enclosing this list.
    pub cont: bool,
}

impl Exits {
    pub fn any(&self) -> bool {
        self.ret || self.brk || self.cont
    }
}

pub(super) fn exits_of(list: &[PStmt]) -> Exits {
    let mut e = Exits::default();
    walk(list, &mut e, false);
    e
}

fn walk(list: &[PStmt], e: &mut Exits, inner: bool) {
    for st in list {
        match st {
            PStmt::Return(_) => e.ret = true,
            PStmt::Break => e.brk |= !inner,
            PStmt::Continue => e.cont |= !inner,
            PStmt::For { body, .. } | PStmt::While { body, .. } => walk(body, e, true),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    walk(b, e, inner);
                }
                if let Some(o) = otherwise {
                    walk(o, e, inner);
                }
            }
            _ => {}
        }
    }
}

/// The flags of the loop being lowered: the names of the break and continue flags it has.
#[derive(Clone, Default, Debug)]
pub(super) struct LoopFlags {
    pub brk: Option<String>,
    pub cont: Option<String>,
}

/// The flags of the function: the result and the done flag of an early `return`.
#[derive(Clone, Debug)]
pub(super) struct RetFlags {
    pub res: String,
    pub done: String,
}

/// The test `if(a + b > 0; 0; 1)` that is 1 while none of the flags is set.
pub(super) fn live_test(flags: &[String]) -> Option<String> {
    (!flags.is_empty()).then(|| format!("if({} > 0; 0; 1)", flags.join(" + ")))
}

/// A guard that holds when `outer` holds and none of `flags` is set.
pub(super) fn and_live(outer: Option<&str>, flags: &[String]) -> Option<String> {
    match (outer, live_test(flags)) {
        (None, None) => None,
        (None, Some(l)) => Some(l),
        (Some(g), None) => Some(g.to_string()),
        (Some(g), Some(l)) => Some(format!("if({g}; {l}; 0)")),
    }
}
