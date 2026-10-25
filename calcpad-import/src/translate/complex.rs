//! Complex numbers, as pairs of real ones.
//!
//! The engine's vectors and matrices hold real numbers only (its complex mode excludes arrays), so a
//! complex value is two real values: `z__re` is the real part and `z__im` the imaginary part, and an
//! array of complex numbers is two arrays of the same shape. Arithmetic on the pairs is written out
//! (`(a + bi)(c + di) = (ac - bd) + (ad + bc)i`), `Re`, `Im`, `abs` and `conj` take the parts, and a
//! complex value used where a real number is needed is a gap that says so: nothing is guessed.
//!
//! A definition whose value is complex becomes two definitions; a program that computes complex
//! values becomes a program that computes both parts, run once for each part of its result.

use super::*;

/// One part of a complex value; `None` is zero.
type Part = Option<Expr>;

/// The imaginary unit as the reader leaves it (an element the importer has no other use for).
pub(super) fn is_imag(e: &Expr) -> bool {
    matches!(e, Expr::Unsupported { detail: Some(d), .. } if d == "imag")
}

/// The name of the imaginary part of a complex name.
pub(super) fn im_name(n: &Name) -> Name {
    Name::new(&format!("{}__im", tables::normalise_name(&n.flat())))
}

/// The name of the real part of a complex name.
pub(super) fn re_name(n: &Name) -> Name {
    Name::new(&format!("{}__re", tables::normalise_name(&n.flat())))
}

fn num(s: &str) -> Expr {
    Expr::Num(s.to_string())
}
fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
    Expr::Bin(op, Box::new(a), Box::new(b))
}
fn neg(a: Expr) -> Expr {
    Expr::Un(UnOp::Neg, Box::new(a))
}
fn add(a: Part, b: Part) -> Part {
    match (a, b) {
        (None, x) | (x, None) => x,
        (Some(x), Some(y)) => Some(bin(BinOp::Add, x, y)),
    }
}
fn sub(a: Part, b: Part) -> Part {
    match (a, b) {
        (x, None) => x,
        (None, Some(y)) => Some(neg(y)),
        (Some(x), Some(y)) => Some(bin(BinOp::Sub, x, y)),
    }
}
fn mul(a: &Part, b: &Part) -> Part {
    match (a, b) {
        (Some(x), Some(y)) => Some(bin(BinOp::Mul, x.clone(), y.clone())),
        _ => None,
    }
}
fn orz(p: Part) -> Expr {
    p.unwrap_or_else(|| num("0"))
}

/// A complex name: a worksheet one, or a local of the program being split.
pub(super) struct Known<'a> {
    pub names: &'a HashSet<String>,
}

impl Known<'_> {
    fn is(&self, n: &Name) -> bool {
        self.names.contains(&tables::normalise_name(&n.flat()))
    }

    /// Whether a complex value or name occurs anywhere in `e`.
    pub fn contains(&self, e: &Expr) -> bool {
        match e {
            e if is_imag(e) => true,
            Expr::Name(n) => self.is(n),
            Expr::Num(_) | Expr::Str(_) | Expr::Unsupported { .. } => false,
            Expr::Un(_, a) | Expr::Postfix(_, a) => self.contains(a),
            Expr::Bin(_, a, b) => self.contains(a) || self.contains(b),
            Expr::Call(_, args) => args.iter().any(|a| self.contains(a)),
            Expr::Index(b, idx) => self.contains(b) || idx.iter().any(|a| self.contains(a)),
            Expr::Matrix { items, .. } => items.iter().any(|a| self.contains(a)),
            Expr::Range { first, second, last } => {
                self.contains(first) || second.as_ref().is_some_and(|s| self.contains(s)) || self.contains(last)
            }
            Expr::Integral { .. } | Expr::Series { .. } | Expr::Derivative { .. } | Expr::Program(_) => false,
        }
    }

    /// The real and imaginary parts of a complex-valued expression.
    pub fn parts(&self, e: &Expr) -> Result<(Part, Part), String> {
        if !self.contains(e) {
            return Ok((Some(e.clone()), None));
        }
        let wrapper = |name: &str| name.to_ascii_lowercase();
        Ok(match e {
            e if is_imag(e) => (None, Some(num("1"))),
            Expr::Name(n) => (Some(Expr::Name(re_name(n))), Some(Expr::Name(im_name(n)))),
            Expr::Index(b, idx) => match &**b {
                Expr::Name(n) if self.is(n) && !idx.iter().any(|i| self.contains(i)) => (
                    Some(Expr::Index(Box::new(Expr::Name(re_name(n))), idx.clone())),
                    Some(Expr::Index(Box::new(Expr::Name(im_name(n))), idx.clone())),
                ),
                _ => return Err("an element of a complex array computed from complex values".into()),
            },
            Expr::Un(UnOp::Neg, a) => {
                let (r, i) = self.parts(a)?;
                (r.map(neg), i.map(neg))
            }
            Expr::Bin(op @ (BinOp::Add | BinOp::Sub), a, b) => {
                let ((ar, ai), (br, bi)) = (self.parts(a)?, self.parts(b)?);
                if *op == BinOp::Add {
                    (add(ar, br), add(ai, bi))
                } else {
                    (sub(ar, br), sub(ai, bi))
                }
            }
            Expr::Bin(BinOp::Mul, a, b) => {
                let ((ar, ai), (br, bi)) = (self.parts(a)?, self.parts(b)?);
                (sub(mul(&ar, &br), mul(&ai, &bi)), add(mul(&ar, &bi), mul(&ai, &br)))
            }
            Expr::Bin(BinOp::Div, a, b) => {
                let (ar, ai) = self.parts(a)?;
                if !self.contains(b) {
                    let d = |p: Part| p.map(|x| bin(BinOp::Div, x, (**b).clone()));
                    (d(ar), d(ai))
                } else {
                    // a / b = a conj(b) / |b|^2
                    let (br, bi) = self.parts(b)?;
                    let nb = neg_part(bi.clone());
                    let (nr, ni) = (sub(mul(&ar, &br), mul(&ai, &nb)), add(mul(&ar, &nb), mul(&ai, &br)));
                    let sq = |p: &Part| mul(p, p);
                    let Some(den) = add(sq(&br), sq(&bi)) else {
                        return Err("a division by zero".into());
                    };
                    let d = |p: Part| p.map(|x| bin(BinOp::Div, x, den.clone()));
                    (d(nr), d(ni))
                }
            }
            Expr::Bin(BinOp::Pow, a, k) if matches!(&**k, Expr::Num(s) if matches!(is_int_literal(s), Some(2..=4))) => {
                let n = match &**k {
                    Expr::Num(s) => is_int_literal(s).unwrap_or(2),
                    _ => 2,
                };
                let (ar, ai) = self.parts(a)?;
                let (mut r, mut i) = (ar.clone(), ai.clone());
                for _ in 1..n {
                    let (nr, ni) = (sub(mul(&r, &ar), mul(&i, &ai)), add(mul(&r, &ai), mul(&i, &ar)));
                    r = nr;
                    i = ni;
                }
                (r, i)
            }
            Expr::Call(f, args) if args.len() == 1 && f.sub.is_none() && wrapper(&f.base) == "conj" => {
                let (r, i) = self.parts(&args[0])?;
                (r, i.map(neg))
            }
            // The real functions of a complex value: a real result.
            Expr::Call(f, args) if args.len() == 1 && f.sub.is_none() && wrapper(&f.base) == "re" => {
                (self.parts(&args[0])?.0, None)
            }
            Expr::Call(f, args) if args.len() == 1 && f.sub.is_none() && wrapper(&f.base) == "im" => {
                (self.parts(&args[0])?.1, None)
            }
            Expr::Call(f, args) if args.len() == 1 && f.sub.is_none() && wrapper(&f.base) == "abs" => {
                let (r, i) = self.parts(&args[0])?;
                let sq = |p: &Part| mul(p, p);
                let s = add(sq(&r), sq(&i)).unwrap_or_else(|| num("0"));
                (Some(Expr::Un(UnOp::Sqrt, Box::new(s))), None)
            }
            Expr::Un(UnOp::Abs, a) => {
                let (r, i) = self.parts(a)?;
                let sq = |p: &Part| mul(p, p);
                let s = add(sq(&r), sq(&i)).unwrap_or_else(|| num("0"));
                (Some(Expr::Un(UnOp::Sqrt, Box::new(s))), None)
            }
            _ => return Err("a complex value in an operation that is not written out for its parts".into()),
        })
    }

    /// `e` with every complex value that is only read through `Re`, `Im`, `abs` or `conj` replaced by
    /// the real expression for it; a complex value left over is an error.
    pub fn realify(&self, e: &Expr) -> Result<Expr, String> {
        if !self.contains(e) {
            return Ok(e.clone());
        }

        match self.parts(e) {
            Ok((r, None)) => return Ok(orz(r)),
            Ok((_, Some(_))) => {}
            // Not a complex-valued whole: look inside (a real expression of complex pieces).
            Err(_) => {}
        }
        let rec = |x: &Expr| self.realify(x);
        Ok(match e {
            Expr::Un(o, a) => Expr::Un(*o, Box::new(rec(a)?)),
            Expr::Postfix(u, a) => Expr::Postfix(u.clone(), Box::new(rec(a)?)),
            Expr::Bin(o, a, b) => Expr::Bin(*o, Box::new(rec(a)?), Box::new(rec(b)?)),
            Expr::Call(f, args) => Expr::Call(f.clone(), args.iter().map(rec).collect::<Result<_, _>>()?),
            Expr::Index(b, idx) => Expr::Index(Box::new(rec(b)?), idx.iter().map(rec).collect::<Result<_, _>>()?),
            Expr::Matrix { rows, cols, items } => Expr::Matrix {
                rows: *rows,
                cols: *cols,
                items: items.iter().map(rec).collect::<Result<_, _>>()?,
            },
            _ => return Err("a complex value where a real number is needed".into()),
        })
    }
}

fn neg_part(p: Part) -> Part {
    p.map(neg)
}

impl Tr {
    /// The set of complex names in force (worksheet ones and the locals of the program being split).
    fn complex_names(&self) -> HashSet<String> {
        self.cplx.union(&self.cx_locals).cloned().collect()
    }

    /// `e` with its complex values rewritten as real expressions, when the sheet has any.
    pub(super) fn realified(&mut self, e: &Expr) -> Option<Result<Expr, String>> {
        if !self.cx_any {
            return None;
        }
        let names = self.complex_names();
        let k = Known { names: &names };
        k.contains(e).then(|| k.realify(e))
    }

    /// `name := value` with a complex value: two definitions, the real and the imaginary part. `None`
    /// when the value is not complex (the definition is an ordinary one).
    pub(super) fn complex_define(&mut self, n: &Name, value: &Expr) -> Option<Result<(String, Vec<String>), String>> {
        if !self.cx_any {
            return None;
        }
        let flat = tables::normalise_name(&n.flat());
        self.cx_matrix = false;
        let names = self.complex_names();
        let k = Known { names: &names };
        if !k.contains(value) && !matches!(value, Expr::Program(_)) {
            return None;
        }
        let (re_text, im_text) = match value {
            Expr::Program(stmts) => {
                let split = match self.split_program(stmts) {
                    Ok(Some(s)) => s,
                    Ok(None) => return None,
                    Err(why) => return Some(Err(why)),
                };
                // The split program has no complex value left: the locals are real arrays now.
                self.cx_locals.clear();
                self.cx_matrix = returns_matrix(stmts);
                let (mut re_prog, mut im_prog) = (split.body.clone(), split.body);
                re_prog.push(PStmt::Expr(split.re));
                im_prog.push(PStmt::Expr(split.im));
                let a = self.lower_program(&re_prog, &format!("{flat}__re"));
                let b = self.lower_program(&im_prog, &format!("{flat}__im"));
                self.cx_locals.clear();
                match (a, b) {
                    (Ok((a, _)), Ok((b, _))) => (a, b),
                    (Err(e), _) | (_, Err(e)) => return Some(Err(e)),
                }
            }
            _ => {
                let (re, im) = match k.parts(value) {
                    Ok(p) => p,
                    Err(why) => return Some(Err(why)),
                };
                if im.is_none() {
                    return None;
                }
                let re = self.expr(&orz(re)).s;
                let im = self.expr(&orz(im)).s;
                (format!("{flat}__re = {re}"), format!("{flat}__im = {im}"))
            }
        };
        self.cplx.insert(flat.clone());
        if self.cx_matrix {
            self.matrices.insert(flat.clone());
        } else {
            self.matrices.remove(&flat);
        }
        Some(Ok((
            format!("{re_text}\n{im_text}"),
            vec![format!("{flat}__re"), format!("{flat}__im")],
        )))
    }

    /// A program that computes complex values, as a program that computes both parts: complex locals
    /// become two locals, a write of a complex value into an array two writes. The parts of the value
    /// it returns come back with the statements. `None` when nothing in it is complex.
    pub(super) fn split_program(&mut self, stmts: &[PStmt]) -> Result<Option<SplitProgram>, String> {
        let Some((last, body)) = stmts.split_last() else {
            return Ok(None);
        };
        let tail = match last {
            PStmt::Expr(e) | PStmt::Return(e) => e,
            _ => return Ok(None),
        };
        // Which locals are complex: any that is given a complex value (found by passes until stable,
        // since a loop can read a local that a later statement makes complex).
        self.cx_locals.clear();
        for _ in 0..4 {
            let before = self.cx_locals.len();
            let names = self.complex_names();
            let k = Known { names: &names };
            let mut found = HashSet::new();
            find_complex_locals(stmts, &k, &mut found);
            self.cx_locals.extend(found);
            if self.cx_locals.len() == before {
                break;
            }
        }
        let names = self.complex_names();
        let k = Known { names: &names };
        let (re, im) = match k.parts(tail) {
            Ok(p) => p,
            Err(why) => return Err(why),
        };
        if im.is_none() && self.cx_locals.is_empty() {
            return Ok(None);
        }
        let body = split_stmts(body, &k)?;
        Ok(Some(SplitProgram {
            body,
            re: orz(re),
            im: orz(im),
        }))
    }
}

/// A program split into the statements that compute both parts and the two parts of its value.
pub(super) struct SplitProgram {
    pub body: Vec<PStmt>,
    pub re: Expr,
    pub im: Expr,
}

/// The locals the statements give a complex value (a whole local, or an element of an array).
fn find_complex_locals(stmts: &[PStmt], k: &Known, out: &mut HashSet<String>) {
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), e) | PStmt::Local(PTarget::Element { base: n, .. }, e) => {
                if k.contains(e) && k.parts(e).is_ok_and(|(_, i)| i.is_some()) {
                    out.insert(tables::normalise_name(&n.flat()));
                }
            }
            PStmt::For { body, .. } | PStmt::While { body, .. } => find_complex_locals(body, k, out),
            PStmt::If { branches, otherwise } => {
                for (_, b) in branches {
                    find_complex_locals(b, k, out);
                }
                if let Some(o) = otherwise {
                    find_complex_locals(o, k, out);
                }
            }
            _ => {}
        }
    }
}

fn split_stmts(stmts: &[PStmt], k: &Known) -> Result<Vec<PStmt>, String> {
    let mut out = Vec::new();
    for st in stmts {
        match st {
            PStmt::Local(PTarget::Var(n), e) if k.is(n) => {
                let (r, i) = k.parts(e)?;
                out.push(PStmt::Local(PTarget::Var(re_name(n)), orz(r)));
                out.push(PStmt::Local(PTarget::Var(im_name(n)), orz(i)));
            }
            PStmt::Local(PTarget::Element { base, index }, e) if k.is(base) => {
                let index: Vec<Expr> = index.iter().map(|i| k.realify(i)).collect::<Result<_, _>>()?;
                let (r, i) = k.parts(e)?;
                out.push(PStmt::Local(
                    PTarget::Element {
                        base: re_name(base),
                        index: index.clone(),
                    },
                    orz(r),
                ));
                out.push(PStmt::Local(
                    PTarget::Element {
                        base: im_name(base),
                        index,
                    },
                    orz(i),
                ));
            }
            PStmt::Local(PTarget::Var(n), e) => out.push(PStmt::Local(PTarget::Var(n.clone()), k.realify(e)?)),
            PStmt::Local(PTarget::Element { base, index }, e) => out.push(PStmt::Local(
                PTarget::Element {
                    base: base.clone(),
                    index: index.iter().map(|i| k.realify(i)).collect::<Result<_, _>>()?,
                },
                k.realify(e)?,
            )),
            PStmt::Local(PTarget::Slice { .. }, _) => out.push(st.clone()),
            PStmt::For { var, range, body } => out.push(PStmt::For {
                var: var.clone(),
                range: k.realify(range)?,
                body: split_stmts(body, k)?,
            }),
            PStmt::While { cond, body } => out.push(PStmt::While {
                cond: k.realify(cond)?,
                body: split_stmts(body, k)?,
            }),
            PStmt::If { branches, otherwise } => out.push(PStmt::If {
                branches: branches
                    .iter()
                    .map(|(c, b)| Ok((k.realify(c)?, split_stmts(b, k)?)))
                    .collect::<Result<_, String>>()?,
                otherwise: match otherwise {
                    Some(o) => Some(split_stmts(o, k)?),
                    None => None,
                },
            }),
            PStmt::Return(e) | PStmt::Expr(e) => {
                if k.contains(e) {
                    return Err("a complex value returned from inside a program".into());
                }
                out.push(st.clone());
            }
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}
