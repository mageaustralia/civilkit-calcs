//! Whole columns and rows of an array as assignment targets: `M<j> := v`, written `matcol(M, j) := v`
//! in the file. The column is written element by element, after the array has grown to hold it.

use super::*;

impl Tr {
    /// The statements that set column (or row) `idx` of `m` to the vector in `t`, after the array has
    /// grown to hold it, under the guard (if any). `uid` makes the counter unique.
    pub(super) fn slice_write(
        &self,
        m: &str,
        idx: &str,
        row: bool,
        t: &str,
        uid: usize,
        guard: Option<&str>,
    ) -> Vec<String> {
        let k = format!("sl_k{uid}");
        // A column is a vector; a row is a one-row matrix.
        let (n, item) = if row {
            (format!("n_cols({t})"), format!("{t}.(1; {k})"))
        } else {
            (format!("len({t})"), format!("{t}.({k})"))
        };
        let grown = if row {
            format!("mresize({m}; max(n_rows({m}); {idx}); max(n_cols({m}); {n}))")
        } else {
            format!("mresize({m}; max(n_rows({m}); {n}); max(n_cols({m}); {idx}))")
        };
        let at = |r: &str, c: &str| format!("{m}.({r}; {c})");
        match guard {
            None => {
                let target = if row { at(idx, &k) } else { at(&k, idx) };
                vec![
                    format!("{m} = {grown}"),
                    format!("$Repeat{{{target} = {item} @ {k} = 1 : {n}}}"),
                ]
            }
            Some(g) => {
                let (gi, gk) = (format!("if({g}; {idx}; 1)"), format!("if({g}; {k}; 1)"));
                let target = if row { at(&gi, &gk) } else { at(&gk, &gi) };
                vec![
                    format!("{m} = if({g}; {grown}; {m})"),
                    format!("$Repeat{{{target} = if({g}; {item}; {m}.(1; 1)) @ {k} = 1 : {n}}}"),
                ]
            }
        }
    }

    /// `M<j> := value` at the sheet: the array is copied (a name that shares it keeps its values),
    /// grown to hold the column, and the column written. Fresh arrays start from one zero.
    pub(super) fn slice_assign(
        &mut self,
        base: &Name,
        index: &Expr,
        row: bool,
        value: &Expr,
    ) -> Result<(String, bool), String> {
        let f = tables::normalise_name(&base.flat());
        if !valid_ident(&f) {
            return Err("the array's name cannot be spelled".into());
        }
        let known = self.defined.contains(&f) || self.funcs.contains_key(&f) || self.gap_names.contains_key(&f);
        if known && !self.arrays.contains(&f) {
            return Err("the name holds a value that is not known to be an array".into());
        }
        if self.reads_range(index) || self.reads_range(value) {
            return Err("a range variable in a whole-column assignment".into());
        }
        let idx = self.index_expr(index);
        // Text labels written into a row or column of a table are held as codes, as in a definition.
        let labels = matches!(value, Expr::Matrix { items, .. } if items.iter().any(|i| matches!(i, Expr::Str(_))));
        self.str_mat_ok = labels;
        let rhs = if labels {
            self.str_or_expr(value).s
        } else {
            self.expr(value).s
        };
        self.str_mat_ok = false;
        let pre = std::mem::take(&mut self.hoisted);
        self.prog_n += 1;
        let uid = self.prog_n;
        let t = format!("sl_t{uid}");
        let mut out = vec!["#hide".to_string()];
        out.extend(pre);
        out.push(format!("{t} = {rhs}"));
        out.push(if known {
            format!("{f} = {f}*1")
        } else {
            format!("{f} = matrix(1; 1)")
        });
        // The same statements as in a function body, with the loop as a `#for`.
        let k = format!("sl_k{uid}");
        let n = if row {
            format!("n_cols({t})")
        } else {
            format!("len({t})")
        };
        let (rows, cols) = if row {
            (format!("max(n_rows({f}); {idx})"), format!("max(n_cols({f}); {n})"))
        } else {
            (format!("max(n_rows({f}); {n})"), format!("max(n_cols({f}); {idx})"))
        };
        out.push(format!("{f} = mresize({f}; {rows}; {cols})"));
        out.push(format!("#for {k} = 1 : {n}"));
        out.push(if row {
            format!("    {f}.({idx}; {k}) = {t}.(1; {k})")
        } else {
            format!("    {f}.({k}; {idx}) = {t}.({k})")
        });
        out.push("#loop".to_string());
        out.push("#end hide".to_string());
        Ok((out.join("\n"), labels))
    }
}
