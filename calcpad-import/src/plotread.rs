//! Reads the plot region of a worksheet: the traces and the expressions typed on the two axes of an
//! x-y plot. Other kinds of plot, and the older binary item, are named so the report can say what
//! was not read.
use roxmltree::Node;

use crate::ir::{PlotAxis, PlotData, TraceStyle, XyPlot};
use crate::xmcd::{local, ml_kids, read_expr, unwrap_provenance};
use crate::Limits;

/// Reads the `plot` element `p`.
pub(crate) fn read_plot(p: Node, lim: &Limits) -> PlotData {
    for k in p.children().filter(|k| k.is_element()) {
        let tag = local(k);
        if tag.ends_with("Plot") || tag.ends_with("plot") {
            if tag == "xyPlot" {
                return PlotData::Xy(Box::new(read_xy(k, lim)));
            }
            return PlotData::Other(tag.to_string());
        }
    }
    PlotData::Unread
}

/// All the text under `n`, whitespace collapsed; `None` when there is none.
fn text_of(n: Node) -> Option<String> {
    let t: String = n
        .descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect::<Vec<_>>()
        .join(" ");
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

fn child<'a, 'i>(n: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    n.children().find(|k| k.is_element() && k.tag_name().name() == name)
}

fn read_xy(xy: Node, lim: &Limits) -> XyPlot {
    let mut out = XyPlot {
        title: child(xy, "title").and_then(text_of),
        ..Default::default()
    };
    if let Some(ts) = child(xy, "traces") {
        for t in ts
            .children()
            .filter(|k| k.is_element() && k.tag_name().name() == "trace")
        {
            let st = child(t, "traceStyle");
            let attr = |n: &str| {
                st.and_then(|s| s.attribute(n))
                    .map(str::to_string)
                    .filter(|v| !v.is_empty())
            };
            out.traces.push(TraceStyle {
                color: attr("color"),
                symbol: attr("symbol"),
                line_style: attr("line-style"),
                kind: st.and_then(text_of),
            });
        }
    }
    if let Some(axes) = child(xy, "axes") {
        if let Some(x) = child(axes, "xAxis") {
            out.x = read_axis(x, lim);
        }
        if let Some(y) = child(axes, "yAxis") {
            out.y = read_axis(y, lim);
        }
    }
    out
}

/// The first math child of `n` that holds something (an empty slot is a placeholder).
fn filled<'a, 'i>(n: Node<'a, 'i>) -> Option<Node<'a, 'i>> {
    ml_kids(n)
        .into_iter()
        .next()
        .map(unwrap_provenance)
        .filter(|k| local(*k) != "placeholder")
}

fn read_axis(a: Node, lim: &Limits) -> PlotAxis {
    let num = |n: &str| a.attribute(n).and_then(|v| v.trim().parse::<f64>().ok());
    let mut ax = PlotAxis {
        start: num("start"),
        end: num("end"),
        label: child(a, "axisLabel").and_then(text_of),
        ..Default::default()
    };
    if let Some(pe) = child(a, "plotEquations") {
        for eq in pe
            .children()
            .filter(|k| k.is_element() && k.tag_name().name() == "plotEquation")
        {
            let maths: Vec<Node> = eq
                .children()
                .filter(|k| k.is_element() && k.tag_name().name() == "math")
                .collect();
            ax.exprs
                .push(maths.first().and_then(|m| filled(*m)).map(|e| read_expr(e, lim, 0)));
            ax.units
                .push(maths.get(1).and_then(|m| filled(*m)).map(|e| read_expr(e, lim, 0)));
        }
    }
    if let Some(d) = child(a, "xyDomain") {
        ax.log = d.attribute("scale-type").is_some_and(|s| s.eq_ignore_ascii_case("log"));
        let bound = |name: &str| child(d, name).and_then(filled).map(|e| read_expr(e, lim, 0));
        ax.from = bound("startValue");
        ax.to = bound("endValue");
    }
    ax
}
