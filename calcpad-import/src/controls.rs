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
        "ComboBoxControl" | "TextBoxScriptableControl" | "ListBoxScriptableControl"
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
            })
        }
        _ => None,
    }
}
