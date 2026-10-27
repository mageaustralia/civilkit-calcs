//! The prettifier of a sheet: spacing around `:`, `|` and `->`, one blank line before a heading,
//! colons aligned in a run of quantity lines. Only lines the compiler reads as quantities are
//! touched; prose, raw Calcpad, fences and formulas are left as written. Idempotent.

use crate::{find_sep, heading, is_quantity_line, split_line};

struct Q {
    indent: String,
    name: String,
    rest: String,
}

enum L {
    Blank,
    Heading(String),
    Quantity(Q),
    Other(String),
}

/// One statement as `name: expression [-> unit]`, with the original expression text.
fn statement(part: &str) -> Option<(String, char, String)> {
    let part = part.trim();
    let p = find_sep(part)?;
    let sep = part[p..].chars().next()?;
    let lhs = part[..p].trim().to_string();
    let rhs = part[p + 1..].trim();
    let top = crate::expr::top_level(rhs);
    let arrow = top
        .iter()
        .rev()
        .find(|&&(q, c)| c == '-' && rhs[q..].starts_with("->"))
        .map(|&(q, _)| q);
    let rhs = match arrow {
        Some(q) => format!("{} -> {}", rhs[..q].trim(), rhs[q + 2..].trim()),
        None => rhs.to_string(),
    };
    Some((lhs, sep, rhs))
}

fn line(l: &str) -> L {
    let t = l.trim_end();
    if t.trim().is_empty() {
        return L::Blank;
    }
    let ws = t.len() - t.trim_start().len();
    let (indent, content) = t.split_at(ws);
    if heading(content).is_some() && ws <= 3 {
        return L::Heading(content.to_string());
    }
    match crate::input::parse(content) {
        Some(Ok(inp)) => {
            let rest = crate::input::canonical(&inp);
            if inp.sep == ':' {
                return L::Quantity(Q {
                    indent: indent.to_string(),
                    name: inp.name,
                    rest,
                });
            }
            return L::Other(format!("{indent}{} = {rest}", inp.name));
        }
        Some(Err(_)) => return L::Other(t.to_string()),
        None => {}
    }
    if !is_quantity_line(content) {
        return L::Other(t.to_string());
    }
    let (parts, note) = split_line(content, true);
    let mut stmts = Vec::new();
    for p in &parts {
        match statement(p) {
            Some(s) => stmts.push(s),
            None => return L::Other(t.to_string()),
        }
    }
    let note = note.map(|n| format!(" | {n}")).unwrap_or_default();
    if stmts.len() == 1 && stmts[0].1 == ':' {
        let (name, _, rest) = stmts.remove(0);
        return L::Quantity(Q {
            indent: indent.to_string(),
            name,
            rest: format!("{rest}{note}"),
        });
    }
    let body = stmts
        .iter()
        .map(|(n, sep, r)| {
            if *sep == ':' {
                format!("{n}: {r}")
            } else {
                format!("{n} = {r}")
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    L::Other(format!("{indent}{body}{note}"))
}

/// The sheet, prettified.
pub fn format(src: &str) -> String {
    let raw: Vec<&str> = src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    let mut kinds: Vec<L> = Vec::new();
    let mut fence = false;
    let mut hash_python = false;
    let mut math = false;
    for l in &raw {
        let t = l.trim();
        if fence {
            if (hash_python && t.eq_ignore_ascii_case("#end python")) || (!hash_python && t.starts_with("```")) {
                fence = false;
                hash_python = false;
            }
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        if math {
            if t == "$$" {
                math = false;
            }
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        if t.eq_ignore_ascii_case("#python") {
            fence = true;
            hash_python = true;
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        if t.starts_with("```") {
            fence = true;
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        if t == "$$" {
            math = true;
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        if t.starts_with(">>") || t.eq_ignore_ascii_case(crate::MARKER) {
            kinds.push(L::Other(l.trim_end().to_string()));
            continue;
        }
        kinds.push(line(l));
    }
    // Align the colons of a run of quantity lines at the same indentation.
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < kinds.len() {
        match &kinds[i] {
            L::Quantity(q) => {
                let mut j = i;
                while j < kinds.len() && matches!(&kinds[j], L::Quantity(r) if r.indent == q.indent) {
                    j += 1;
                }
                let width = (i..j)
                    .map(|k| match &kinds[k] {
                        L::Quantity(r) => r.name.chars().count(),
                        _ => 0,
                    })
                    .max()
                    .unwrap_or(0);
                for item in &kinds[i..j] {
                    if let L::Quantity(r) = item {
                        let pad = " ".repeat(width - r.name.chars().count());
                        out.push(format!("{}{}{}: {}", r.indent, r.name, pad, r.rest));
                    }
                }
                i = j;
            }
            L::Heading(h) => {
                if !out.is_empty()
                    && out.last().is_some_and(|l| !l.is_empty())
                    && !out.last().is_some_and(|l| l.trim() == crate::MARKER)
                {
                    out.push(String::new());
                }
                out.push(h.clone());
                i += 1;
            }
            L::Blank => {
                if out.last().is_some_and(|l| !l.is_empty()) {
                    out.push(String::new());
                }
                i += 1;
            }
            L::Other(t) => {
                out.push(t.clone());
                i += 1;
            }
        }
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::format;

    #[test]
    fn tidies_quantity_lines() {
        let src = "<!--sheet-->\n# T\n## Inputs\nL:6 m|span\nfy :  300 MPa   |  yield\nMs: fy*Ze->kN*m\n\n\n\nprose  with   spaces  \nx=1 ;y : 2\n";
        let f = format(src);
        assert_eq!(
            f,
            "<!--sheet-->\n# T\n\n## Inputs\nL : 6 m | span\nfy: 300 MPa | yield\nMs: fy*Ze -> kN*m\n\nprose  with   spaces\nx = 1; y: 2\n"
        );
    }
}
