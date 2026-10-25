//! Input controls (combo box, text box, list box) that a worksheet places in a definition.
//!
//! A control is a variable the reader of the worksheet sets by choosing or typing; the file keeps
//! what was chosen. The importer takes that stored value as the input the control represents, so
//! the definition `x := <control>` becomes `x := <stored value>`, and says what it did: which
//! option or text was stored, and that the options, the script behind the control and its links to
//! other names are not carried. A button runs a script and holds no value, so it stays a gap.

use crate::ir::*;
use crate::xmcd::{is_ml, local, ml_kids, read_expr, Limits};
use roxmltree::Node;

/// Whether the element is a control this module reads.
pub(crate) fn is_control(tag: &str) -> bool {
    matches!(
        tag,
        "ComboBoxControl"
            | "TextBoxScriptableControl"
            | "ListBoxScriptableControl"
            | "ButtonScriptableControl"
            | "CheckBoxScriptableControl"
    )
}

fn vals(n: Node) -> Vec<String> {
    n.children()
        .find(|c| is_ml(*c) && local(*c) == "vals")
        .map(|v| {
            v.children()
                .filter(|c| is_ml(*c) && local(*c) == "val")
                .map(|c| c.text().unwrap_or("").trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// A short label: control characters and markup are removed and the length is capped.
fn label_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | '&' | '"' | '\''))
        .take(80)
        .collect::<String>()
        .trim()
        .to_string()
}

/// The stored value of a control and what is said about it, or `None` for a control that holds no
/// value or one that cannot be read.
pub(crate) fn read_control(n: Node, lim: &Limits, depth: usize) -> Option<Control> {
    let tag = local(n);
    match tag {
        "ComboBoxControl" => {
            let sel: usize = n.attribute("SelectedRow").and_then(|v| v.trim().parse().ok())?;
            let child = |name: &str| n.children().find(|c| is_ml(*c) && local(*c) == name);
            let values: Vec<Node> = child("ComboBoxValues").map(|v| ml_kids(v)).unwrap_or_default();
            let names: Vec<String> = child("ComboBoxRowNames")
                .map(|v| {
                    v.children()
                        .filter(|c| is_ml(*c) && local(*c) == "rowName")
                        .map(|c| c.text().unwrap_or("").trim().to_string())
                        .collect()
                })
                .unwrap_or_default();
            let mut value = read_expr(*values.get(sel)?, lim, depth + 1);
            // A scale factor (a unit, usually) multiplies the stored value: one for every row, or
            // one per row.
            let scales: Vec<Node> = child("ComboBoxScaleFactors")
                .map(|v| ml_kids(v))
                .unwrap_or_default()
                .into_iter()
                .filter(|s| local(*s) != "placeholder")
                .collect();
            let scale = match scales.len() {
                0 => None,
                1 => Some(scales[0]),
                _ => scales.get(sel).copied(),
            };
            if let Some(s) = scale {
                value = Expr::Bin(BinOp::Mul, Box::new(value), Box::new(read_expr(s, lim, depth + 1)));
            }
            Some(Control {
                kind: "combo box".into(),
                value,
                chosen: names.get(sel).map(|s| label_text(s)).filter(|s| !s.is_empty()),
                options: values.len(),
                caption: None,
            })
        }
        "TextBoxScriptableControl" => {
            let text = vals(n).into_iter().next().unwrap_or_default();
            let value = match text.parse::<f64>() {
                Ok(v) if v.is_finite() && !text.is_empty() && text.len() <= 64 => Expr::Num(text.clone()),
                _ => Expr::Str(text.clone()),
            };
            Some(Control {
                kind: "text box".into(),
                value,
                chosen: Some(label_text(&text)).filter(|s| !s.is_empty()),
                options: 0,
                caption: None,
            })
        }
        "ListBoxScriptableControl" => {
            let items = vals(n);
            let sel: usize = n.attribute("SelectedIndex").and_then(|v| v.trim().parse().ok())?;
            let text = items.get(sel)?.clone();
            // The worksheets compare a list box with whole numbers (`box = 0`): its value is the
            // position of the selected item, counted from 0.
            Some(Control {
                kind: "list box".into(),
                value: Expr::Num(sel.to_string()),
                chosen: Some(label_text(&text)).filter(|s| !s.is_empty()),
                options: items.len(),
                caption: None,
            })
        }
        "ButtonScriptableControl" | "CheckBoxScriptableControl" => {
            let caption = vals(n)
                .into_iter()
                .next()
                .map(|t| label_text(&t))
                .filter(|t| !t.is_empty());
            let stored = n.attribute("RL").and_then(stored_value)?;
            let value = match stored {
                Stored::One(e) => e,
                // One value per name the control defines (`[Flag, Folder] := <control>`).
                Stored::List(items) => Expr::Matrix {
                    rows: items.len(),
                    cols: 1,
                    items,
                },
            };
            Some(Control {
                kind: if tag == "ButtonScriptableControl" {
                    "button"
                } else {
                    "check box"
                }
                .into(),
                value,
                chosen: None,
                options: 0,
                caption,
            })
        }
        _ => None,
    }
}

enum Stored {
    One(Expr),
    List(Vec<Expr>),
}

/// The value a scripted control stored (its `RL` attribute: base64 of a short s-expression such as
/// `(number 1:0x3)`, `(string "text":0x3)` or `(op_matrix:0x9 (unboxed 2) (unboxed 1) (list ...))`).
/// A list is read only when every item is a number or a text.
fn stored_value(rl: &str) -> Option<Stored> {
    let bytes = crate::picture::base64_decode(rl)?;
    let text = String::from_utf8(bytes).ok()?;
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("(op_matrix") {
        let list = rest.find("(list")?;
        let mut items = Vec::new();
        let mut at = list + "(list".len();
        let body = &rest[..];
        loop {
            let tail = body[at..].trim_start();
            let skipped = body[at..].len() - tail.len();
            at += skipped;
            if tail.starts_with(')') || tail.is_empty() {
                break;
            }
            let (item, used) = stored_item(tail)?;
            items.push(item);
            at += used;
        }
        return (!items.is_empty() && items.len() <= 64).then_some(Stored::List(items));
    }
    stored_item(t).map(|(e, _)| Stored::One(e))
}

/// One `(number N:0x..)` or `(string "...":0x..)`, and how many bytes it takes.
fn stored_item(t: &str) -> Option<(Expr, usize)> {
    if let Some(rest) = t.strip_prefix("(number ") {
        let end = rest.find(':')?;
        let num = rest[..end].trim();
        num.parse::<f64>().ok().filter(|v| v.is_finite())?;
        let close = rest.find(')')?;
        return Some((Expr::Num(num.to_string()), "(number ".len() + close + 1));
    }
    if let Some(rest) = t.strip_prefix("(string \"") {
        let mut out = String::new();
        let mut chars = rest.char_indices();
        let mut end = None;
        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => {
                    if let Some((_, d)) = chars.next() {
                        out.push(d);
                    }
                }
                '"' => {
                    end = Some(i);
                    break;
                }
                _ => out.push(c),
            }
        }
        let end = end?;
        let close = rest[end..].find(')')? + end;
        return Some((Expr::Str(out), "(string \"".len() + close + 1));
    }
    None
}
