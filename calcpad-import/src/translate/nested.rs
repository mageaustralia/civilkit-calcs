//! Arrays that hold an array in each element (`v[k] := f(k)` where `f` gives a vector). The engine
//! has no nested arrays; when every element is a vector the array is a matrix with one row per
//! element, and `v[k][j]` is the matrix element at row k, column j.

use super::*;

impl Tr {
    /// Loops over the range variables the assignment reads, writing the vector the value gives as
    /// row `idx` of a matrix that grows to hold it.
    pub(super) fn nested_fill_loops(
        &mut self,
        f: &str,
        rs: &[(String, RangeInfo)],
        idx: &str,
        rhs: &str,
        pre: &[String],
        fresh: bool,
    ) -> Result<String, String> {
        let (head, loops, _, _) = self.range_loops(rs)?;
        self.prog_n += 1;
        let (t, j) = (format!("nv_t{}", self.prog_n), format!("nv_j{}", self.prog_n));
        let mut out = vec!["#hide".to_string()];
        out.extend(head);
        out.push(if fresh {
            format!("{f} = matrix(1; 1)")
        } else {
            // Writing into an array another name may share would change that name too.
            format!("{f} = {f}*1")
        });
        for (d, l) in loops.iter().enumerate() {
            for line in l.split('\n') {
                out.push(format!("{}{}", "    ".repeat(d), line));
            }
        }
        let pad = "    ".repeat(loops.len());
        for p in pre {
            out.push(format!("{pad}{p}"));
        }
        out.push(format!("{pad}{t} = {rhs}"));
        out.push(format!(
            "{pad}{f} = mresize({f}; max(n_rows({f}); {idx}); max(n_cols({f}); len({t})))"
        ));
        out.push(format!("{pad}#for {j} = 1 : len({t})"));
        out.push(format!("{pad}    {f}.({idx}; {j}) = {t}.({j})"));
        out.push(format!("{pad}#loop"));
        for d in (0..loops.len()).rev() {
            out.push(format!("{}#loop", "    ".repeat(d)));
        }
        out.push("#end hide".to_string());
        Ok(out.join("\n"))
    }
}
