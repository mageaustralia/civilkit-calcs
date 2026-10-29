//! `#unit name = expression [| note]`: a unit of the sheet's own, written to the engine as the
//! custom unit `.name = expression`. Later lines use it after a number (`3 bag`) and after `->`.

use crate::{expr, split_line, tex, Comp, LineKind, Severity};

/// A unit name the engine accepts: letters and `_`, starting with a letter (no digits).
fn is_unit_name(s: &str) -> bool {
    let mut it = s.chars();
    matches!(it.next(), Some(c) if c.is_alphabetic()) && it.all(|c| c.is_alphabetic() || c == '_')
}

/// A `#unit` line: (name, definition, note), or the message for a bad line.
type Parsed<'s> = Result<(&'s str, &'s str, Option<String>), &'static str>;

/// The parts of a `#unit` line. None if the line is not one.
pub(crate) fn parse(content: &str) -> Option<Parsed<'_>> {
    let rest = content.strip_prefix("#unit")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let (parts, note) = split_line(rest, true);
    if parts.len() != 1 {
        return Some(Err("one unit per line: #unit name = value with units"));
    }
    let Some(eq) = parts[0].find(['=', ':']) else {
        return Some(Err(
            "write the unit as #unit name = value with units, such as #unit bag = 25 kg",
        ));
    };
    let name = parts[0][..eq].trim();
    let def = parts[0][eq + 1..].trim();
    if !is_unit_name(name) {
        return Some(Err(
            "a unit name is letters and _ only (no digits), such as bag or ton_short",
        ));
    }
    if def.is_empty() {
        return Some(Err("the unit needs a value, such as #unit bag = 25 kg"));
    }
    Some(Ok((name, def, note)))
}

impl Comp<'_> {
    /// Reads a `#unit` line. False if the line is not one.
    pub(crate) fn unit_def(&mut self, content: &str, idx: usize) -> bool {
        let Some(parsed) = parse(content) else {
            return false;
        };
        self.flush_prose();
        self.mark(idx, LineKind::Quantity);
        let (name, def, note) = match parsed {
            Ok(p) => p,
            Err(msg) => {
                self.diag(idx, Severity::Error, msg);
                return true;
            }
        };
        let e = match expr::translate(def, &self.ctx(false)) {
            Ok(e) => e,
            Err(_) => {
                self.diag(
                    idx,
                    Severity::Error,
                    "the value of a unit is a number with units or an expression, such as 25 kg or 0.3048 m",
                );
                return true;
            }
        };
        let mut text = format!(".{name} = {e}");
        if let Some(nt) = note {
            let n = tex::inline(&nt, self.opts.plain_tex, true, false, &mut self.latex);
            if self.opts.plain_tex {
                text.push_str(&format!(" '{n}"));
            } else {
                text.push_str(&format!(" '<span class=\"sheet-note\">{n}</span>"));
            }
        }
        self.emit(text, idx);
        true
    }
}
