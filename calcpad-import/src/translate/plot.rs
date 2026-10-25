//! Plot regions: an x-y plot becomes one `$Plot{...}` command of the engine.
//!
//! Trace `k` pairs the k-th x expression with the k-th y expression (the last one of an axis is
//! reused when the other has more). The engine plots expressions of one variable, so the plot
//! needs a parameter:
//!
//! * a range variable of the sheet used in the expressions (`x[i]` and `V(x[i])` over `i = 0 .. 300`)
//!   runs over that range, and an index that mentions it reads the array by linear interpolation
//!   (`line(i + 1; x)`), so a fractional parameter still lands between two elements;
//! * a single name that nothing defines (`f(x)` plotted against `x`) runs over the domain the author
//!   typed, or over the range the axis was drawn with;
//! * whole arrays plotted against each other run over their element number.
//!
//! What the engine cannot draw is not carried and is said in a `plot-style` note: colours, markers,
//! line styles, columns and waterfalls (drawn as lines), logarithmic axes and the axis ranges. An
//! expression that does not convert, or a kind of plot that is not an x-y plot, is a gap.

use super::*;

impl Tr {
    pub(super) fn plot(&mut self, r: &Region, p: &PlotData) {
        match p {
            PlotData::Xy(xy) => self.xy_plot(r, xy),
            PlotData::Other(kind) => {
                let kind: String = kind.chars().filter(|c| c.is_ascii_alphanumeric()).take(32).collect();
                self.simple_gap(
                    "plot",
                    r,
                    Code::Plot,
                    Some(format!("{kind}: only x-y plots are converted")),
                )
            }
            PlotData::Unread => self.simple_gap(
                "plot",
                r,
                Code::Plot,
                Some("stored as a binary item that the importer does not read".into()),
            ),
        }
    }

    /// Why a plot is a gap before any expression is looked at.
    fn plot_gap(&mut self, r: &Region, why: &str) {
        self.simple_gap("plot", r, Code::Plot, Some(why.to_string()))
    }

    /// True when `e` is an array (or elementwise arithmetic or `re`, `im`, `abs` of one).
    fn plot_array(&self, e: &Expr) -> bool {
        match e {
            Expr::Call(f, args) if matches!(f.base.as_str(), "Re" | "Im" | "re" | "im" | "abs") => {
                args.iter().any(|a| self.plot_array(a))
            }
            Expr::Bin(_, a, b) => self.plot_array(a) || self.plot_array(b),
            Expr::Un(_, a) => self.plot_array(a),
            _ => self.holds_array(e),
        }
    }

    fn xy_plot(&mut self, r: &Region, xy: &XyPlot) {
        self.cur.clear();
        self.params.clear();
        self.depth = 0;
        let (nx, ny) = (xy.x.exprs.len(), xy.y.exprs.len());
        let nt = if xy.traces.is_empty() {
            nx.max(ny)
        } else {
            xy.traces.len()
        };
        if nx == 0 || ny == 0 || nt == 0 {
            return self.plot_gap(r, "the plot has no expression on an axis");
        }
        // The expression of each trace, with the unit the axis shows it in. The axis has one unit:
        // the first one typed beside any of its expressions.
        let shown = |axis: &PlotAxis, k: usize| -> Option<Expr> {
            let i = k.min(axis.exprs.len() - 1);
            let e = axis.exprs[i].clone()?;
            Some(match axis.units.iter().flatten().next().cloned() {
                Some(u) => Expr::Bin(BinOp::Div, Box::new(e), Box::new(u)),
                None => e,
            })
        };
        let mut pairs: Vec<(Expr, Expr)> = Vec::new();
        for k in 0..nt {
            match (shown(&xy.x, k), shown(&xy.y, k)) {
                (Some(x), Some(y)) => pairs.push((x, y)),
                _ => return self.plot_gap(r, "a trace has an empty slot on an axis"),
            }
        }
        let all: Vec<&Expr> = pairs.iter().flat_map(|(x, y)| [x, y]).collect();
        // An array of several columns is drawn as one trace per column; the engine plots a vector
        // against a vector.
        if let Some(m) = self.matrices.iter().find(|m| all.iter().any(|e| mentions(e, m))) {
            let m = m.trim_end_matches("__im").trim_end_matches("__re").to_string();
            return self.plot_gap(
                r,
                &format!("{m} is an array with several columns, which the source draws as one trace per column"),
            );
        }

        // Range variables the expressions use.
        let mut range_vars: Vec<String> = self
            .ranges
            .keys()
            .filter(|n| all.iter().any(|e| mentions(e, n)))
            .cloned()
            .collect();
        range_vars.sort();

        // Names nothing defines: found by rendering once with no parameter.
        let free: Vec<String> = {
            let mark = self.cur.len();
            for e in &all {
                let _ = self.expr(e);
            }
            let mut f: Vec<String> = Vec::new();
            for p in &self.cur[mark..] {
                if p.code == Code::UndefinedVariable {
                    if let Some(d) = &p.detail {
                        if !f.contains(d) {
                            f.push(d.clone());
                        }
                    }
                }
            }
            f
        };
        self.cur.clear();
        self.hoisted.clear();

        // The parameter, and the range it runs over.
        // With no usable parameter (several range variables, or several names nothing defines) the
        // expressions are rendered as they are, which reports the names that are missing.
        let (param, from, to, synthetic): (Option<String>, String, String, bool);
        if range_vars.len() > 1 {
            return self.plot_gap(r, &format!("several range variables ({})", range_vars.join(", ")));
        } else if let Some(v) = range_vars.first() {
            let info = self.ranges[v].clone();
            param = Some(v.clone());
            from = info.a;
            to = info.z;
            synthetic = false;
        } else if free.len() > 1 {
            param = None;
            from = String::new();
            to = String::new();
            synthetic = false;
        } else if let Some(v) = free.first() {
            let bound = |tr: &mut Tr, e: &Option<Expr>, n: Option<f64>| -> Option<String> {
                match e {
                    Some(e) => Some(tr.expr(e).s),
                    None => n.filter(|n| n.is_finite()).map(|n| format!("{n}")),
                }
            };
            let a = bound(self, &xy.x.from, xy.x.start);
            let z = bound(self, &xy.x.to, xy.x.end);
            match (a, z) {
                (Some(a), Some(z)) => {
                    param = Some(v.clone());
                    from = a;
                    to = z;
                    synthetic = false;
                }
                _ => return self.plot_gap(r, "the plot variable has no range"),
            }
        } else {
            // Arrays against each other, or constants.
            param = Some("i_plot".to_string());
            from = "1".to_string();
            let first_array = all.iter().find(|e| self.plot_array(e)).map(|e| (*e).clone());
            to = match first_array {
                Some(e) => format!("len({})", self.expr(&e).s),
                None => {
                    // Constants only: a point per trace.
                    "2".to_string()
                }
            };
            synthetic = true;
        }

        // An axis with no unit typed shows a quantity in the source's base units. The engine plots in
        // the unit the value is held in, which can be a multiple (a force held in kN), so such a trace
        // is divided by the base unit of the unit the sheet shows the quantity in.
        let x_typed = xy.x.units.iter().any(|u| u.is_some());
        let y_typed = xy.y.units.iter().any(|u| u.is_some());
        let mut si: Vec<(Option<String>, Option<String>)> = Vec::new();
        let mut unscaled: Vec<String> = Vec::new();
        for (x, y) in &pairs {
            let hx = if x_typed {
                None
            } else {
                self.base_unit_for(x, &mut unscaled)
            };
            let hy = if y_typed {
                None
            } else {
                self.base_unit_for(y, &mut unscaled)
            };
            si.push((hx, hy));
        }
        self.cur.clear();

        // Render the traces.
        if let Some(p) = &param {
            self.params.push(p.clone());
        }
        self.plot_param = param.clone();
        let pname = param.clone().unwrap_or_default();
        let mut traces: Vec<String> = Vec::new();
        for ((x, y), (hx, hy)) in pairs.iter().zip(&si) {
            let side = |tr: &mut Tr, e: &Expr, base: &Option<String>| -> String {
                let r = tr.expr(e);
                let mut s = r.s;
                if let Some(b) = base {
                    s = if r.prec < P_MUL {
                        format!("({})/{}", s, b)
                    } else {
                        format!("{}/{}", s, b)
                    };
                }
                if synthetic && tr.plot_array(e) {
                    format!("line({}; ({}))", pname, s)
                } else {
                    s
                }
            };
            let sx = side(self, x, hx);
            let sy = side(self, y, hy);
            traces.push(format!("{}|{}", sx, sy));
        }
        self.plot_param = None;
        self.params.clear();
        let hoisted = std::mem::take(&mut self.hoisted);
        if !hoisted.is_empty() {
            self.issue(
                Code::BadStructure,
                Some("the result of a call is indexed inside a plot".into()),
            );
        }

        let gap_codes: Vec<Code> = self
            .cur
            .iter()
            .filter(|p| p.code.severity() == Severity::Gap)
            .map(|p| p.code)
            .collect();
        if !gap_codes.is_empty() {
            let pictured = self.show_gap(r, &gap_codes, None, None);
            self.flush_issues(&r.id, r.line, None);
            Self::bump_p(&mut self.report.kinds, "plot", false, false, pictured);
            return;
        }

        // What the engine does not draw.
        let mut dropped: Vec<String> = Vec::new();
        if xy.traces.iter().any(|t| t.color.is_some()) {
            dropped.push("colours".to_string());
        }
        if xy
            .traces
            .iter()
            .any(|t| t.symbol.as_deref().is_some_and(|s| s != "none"))
        {
            dropped.push("markers".to_string());
        }
        if xy.traces.iter().any(|t| {
            t.line_style
                .as_deref()
                .is_some_and(|s| !s.eq_ignore_ascii_case("solid"))
        }) {
            dropped.push("line styles".to_string());
        }
        if xy
            .traces
            .iter()
            .any(|t| t.kind.as_deref().is_some_and(|k| !k.eq_ignore_ascii_case("lines")))
        {
            dropped.push("columns and waterfalls (drawn as lines)".to_string());
        }
        for u in unscaled {
            dropped.push(format!(
                "unit {u} is not a known base unit: axis left in the unit the value is held in"
            ));
        }
        if xy.x.log || xy.y.log {
            dropped.push("logarithmic axes (drawn linear)".to_string());
        }
        if xy.x.start.is_some() || xy.y.start.is_some() {
            dropped.push("axis ranges".to_string());
        }
        if !dropped.is_empty() {
            self.issue(Code::PlotStyle, Some(dropped.join(", ")));
        }

        if let Some(t) = &xy.title {
            self.lines
                .push(format!("'<p style=\"text-align:center\"><b>{}</b></p>", escape_text(t)));
        }
        self.lines.push(format!(
            "$Plot{{{} @ {} = {} : {}}}",
            traces.join(" & "),
            pname,
            from,
            to
        ));
        let labels: Vec<String> = [("x", &xy.x.label), ("y", &xy.y.label)]
            .iter()
            .filter_map(|(a, l)| l.as_ref().map(|l| format!("{}: {}", a, escape_text(l))))
            .collect();
        if !labels.is_empty() {
            self.lines.push(format!(
                "'<p style=\"text-align:center\"><small>{}</small></p>",
                labels.join("; ")
            ));
        }
        let has_notes = self.cur.iter().any(|p| p.code.severity() == Severity::Note);
        self.flush_issues(&r.id, r.line, None);
        Self::bump(&mut self.report.kinds, "plot", true, has_notes);
    }
}

/// The base unit with the same dimension, for the prefixed and derived spellings a sheet shows.
fn coherent(unit: &str) -> Option<String> {
    let mut out = String::new();
    let mut tok = String::new();
    let flush = |tok: &mut String, out: &mut String| -> bool {
        if tok.is_empty() {
            return true;
        }
        let m = match tok.as_str() {
            "N" | "kN" | "MN" | "GN" | "daN" => "N",
            "Pa" | "kPa" | "MPa" | "GPa" | "hPa" => "Pa",
            "m" | "mm" | "cm" | "km" | "dm" => "m",
            "J" | "kJ" | "MJ" => "J",
            "W" | "kW" | "MW" => "W",
            "s" | "ms" => "s",
            "kg" => "kg",
            "kNm" => "N*m",
            "A" | "V" | "K" | "Hz" | "rad" => tok.as_str(),
            _ => return false,
        };
        out.push_str(m);
        tok.clear();
        true
    };
    for c in unit.chars() {
        if c.is_alphabetic() {
            tok.push(c);
        } else {
            if !flush(&mut tok, &mut out) {
                return None;
            }
            out.push(c);
        }
    }
    if !flush(&mut tok, &mut out) {
        return None;
    }
    Some(out)
}

impl Tr {
    /// For an axis expression with no unit typed: the base unit to divide it by, when the sheet shows
    /// one of the names in it in a unit and the expression itself names no unit. A unit that is not
    /// one of the known spellings is recorded in `unscaled` and the expression is left as it is.
    fn base_unit_for(&mut self, e: &Expr, unscaled: &mut Vec<String>) -> Option<String> {
        fn walk(e: &Expr, names: &mut Vec<Name>) {
            match e {
                Expr::Name(n) => names.push(n.clone()),
                Expr::Un(_, a) => walk(a, names),
                Expr::Postfix(u, a) => {
                    names.push(Name {
                        unit: true,
                        ..u.clone()
                    });
                    walk(a, names)
                }
                Expr::Bin(_, a, b) => {
                    walk(a, names);
                    walk(b, names)
                }
                Expr::Call(_, args) => args.iter().for_each(|a| walk(a, names)),
                Expr::Index(b, idx) => {
                    walk(b, names);
                    idx.iter().for_each(|a| walk(a, names))
                }
                _ => {}
            }
        }
        let mut names = Vec::new();
        walk(e, &mut names);
        if names.iter().any(|n| matches!(n.unit, true) || self.is_unit_name(n)) {
            return None;
        }
        let mut hint: Option<Expr> = None;
        for n in &names {
            if let Some(u) = self.unit_hints.get(&tables::normalise_name(&n.flat())) {
                hint = Some(u.clone());
                break;
            }
        }
        let u = hint?;
        let mark = self.cur.len();
        let text = self.expr(&u).s;
        self.cur.truncate(mark);
        match coherent(&text) {
            Some(c) => Some(if c.contains(|ch: char| "*/^ ".contains(ch)) {
                format!("({})", c)
            } else {
                c
            }),
            None => {
                unscaled.push(text);
                None
            }
        }
    }
}
