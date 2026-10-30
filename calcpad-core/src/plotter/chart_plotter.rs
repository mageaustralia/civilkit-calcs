//! Port of Plotter/ChartPlotter.cs ($Plot).
use std::rc::Rc;

use super::capture::{CapturedChart, CapturedSeries, SERIES_COLORS};
use super::svg_drawing::SvgDrawing;
use super::svg_point::SvgPoint;
use super::{clamp, Node, PlotBox, Plotter};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::base_types::variable::VarRef;
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::parsers::math_parser::compiler::Expr;
use crate::parsers::math_parser::MathParser;
use crate::settings::PlotSettings;

/// A compiled plot function (`Func<IValue>`); None is a null delegate.
pub type PlotFunc = Option<Rc<Expr>>;

struct Chart {
    points: Vec<Node>,
    svg_points: Vec<SvgPoint>,
    bounds: PlotBox,
    fill: bool,
}

impl Chart {
    fn new(size: usize) -> Chart {
        Chart {
            points: Vec::with_capacity(size),
            svg_points: Vec::new(),
            bounds: PlotBox::empty(),
            fill: false,
        }
    }

    fn add_point(&mut self, x: f64, y: f64, z: f64) {
        if x.is_nan() || x.is_infinite() || y.is_nan() || y.is_infinite() {
            return;
        }
        let n = Node::new(x, y, z);
        self.bounds.expand(&n);
        self.points.push(n);
    }

    fn point_count(&self) -> usize {
        self.points.len()
    }

    fn get_chart_svg_points(&mut self, x0: f64, y0: f64, xs: f64, ys: f64) {
        self.svg_points = self
            .points
            .iter()
            .map(|p| SvgPoint::new(x0 + p.x * xs, y0 - p.y * ys))
            .collect();
    }
}

pub struct ChartPlotter<'a> {
    base: Plotter<'a>,
    var: Option<VarRef>,
    x_units: OptUnit,
    y_units: OptUnit,
    /// The expression of each curve as written, for the data of the Excel export.
    pub names: Vec<String>,
    /// The name given to each curve (`sin(x) "sine"`), drawn in a legend when any curve has one.
    pub legend: Vec<Option<String>>,
    /// The plot's data, set by `plot` (the Excel export).
    pub captured: Option<CapturedChart>,
}

/// `Vector(Node p1, Node p2)`: the unit direction from p1 to p2.
fn direction(p1: &Node, p2: &Node) -> Node {
    let x = p2.x - p1.x;
    let y = p2.y - p1.y;
    let l = (x * x + y * y).sqrt();
    if l == 0.0 {
        return Node::default();
    }
    Node::new(x / l, y / l, 0.0)
}

/// `ConvertValue(RealValue v, ref Unit u)`
fn convert_value(v: &RealValue, u: &mut OptUnit) -> CResult<f64> {
    match u {
        None => {
            *u = v.units.clone();
        }
        Some(uu) => {
            if !Unit::ptr_eq(&Some(uu.clone()), &v.units) {
                if !Unit::is_consistent_opt(&Some(uu.clone()), &v.units) {
                    return Err(exceptions::InconsistentUnits(
                        &Unit::get_text_of(&Some(uu.clone())),
                        &Unit::get_text_of(&v.units),
                    ));
                }
                return Ok(v.d * v.units.as_ref().unwrap().convert_to(uu));
            }
        }
    }
    Ok(v.d)
}

impl<'a> ChartPlotter<'a> {
    pub fn new(parser: &'a MathParser, settings: &'a PlotSettings) -> ChartPlotter<'a> {
        ChartPlotter {
            base: Plotter::new(parser, settings),
            var: None,
            x_units: None,
            y_units: None,
            names: Vec::new(),
            legend: Vec::new(),
            captured: None,
        }
    }

    fn get_image_size(&mut self) -> CResult<()> {
        let b = &mut self.base;
        b.left = (1.5 * b.margin as f64) as i32;
        b.right = (0.75 * b.margin as f64) as i32;
        let w = b.parser.get_settings_variable("PlotWidth", b.settings.width as f64)? as i32;
        let h = b.parser.get_settings_variable("PlotHeight", b.settings.height as f64)? as i32;
        b.width = (w as f32 * b.screen_scale_factor) as i32 + b.left + b.right;
        b.height = (h as f32 * b.screen_scale_factor) as i32 + 2 * b.margin;
        Ok(())
    }

    /// `Plot(fx, fy, var, start, end, u)`
    pub fn plot(
        &mut self,
        fx: &[PlotFunc],
        fy: &[PlotFunc],
        var: &VarRef,
        start: f64,
        end: f64,
        u: &OptUnit,
    ) -> CResult<String> {
        self.var = Some(var.clone());
        self.x_units = None;
        self.y_units = None;
        self.get_image_size()?;
        let len = fx.len();
        let mut charts: Vec<Chart> = (0..len).map(|_| Chart::new(0)).collect();
        let mut limits = PlotBox::empty();
        let n = if self.base.is_adaptive()? { 2 } else { 1 };
        let (mut x0, mut y0, mut xs, mut ys) = (0.0, 0.0, 0.0, 0.0);
        let (left, right, margin) = (self.base.left, self.base.right, self.base.margin);
        let (width, height) = (self.base.width as f64, self.base.height as f64);
        for k in 0..n {
            limits = PlotBox::empty();
            for i in 0..len {
                let points = if k == 0 {
                    self.calculate(&fx[i], &fy[i], start, end, u)?
                } else {
                    let kxy = if xs == 0.0 { 1.0 } else { ys / xs };
                    self.calculate_adaptive(&charts[i], &fx[i], &fy[i], kxy)?
                };
                let mut c = Chart::new(points.len());
                for p in &points {
                    c.add_point(p.x, p.y, p.z);
                }
                c.fill = fx[i].is_none();
                limits.expand_box(&c.bounds);
                charts[i] = c;
            }
            limits = fix_limits(&limits);
            if limits.width() == 0.0 || !limits.width().is_normal() || ys == limits.height() {
                if crate::extension_methods::almost_equals(start, end) {
                    let padding = if limits.height().is_normal() {
                        limits.height().max(0.1)
                    } else {
                        0.1
                    };
                    let l = start - padding;
                    let r = end + padding;
                    limits = PlotBox::new(l, l, r, r);
                } else {
                    limits = PlotBox::new(start, start, end, end);
                }
                xs = (width - left as f64 - right as f64) / limits.width();
                x0 = left as f64 - ((limits.left * xs) as i32) as f64;
                ys = (height - 2.0 * margin as f64) / limits.height();
                y0 = height - margin as f64 + ((limits.bottom * ys) as i32) as f64;
                break;
            }
            xs = (width - left as f64 - right as f64) / limits.width();
            x0 = left as f64 - ((limits.left * xs) as i32) as f64;
            ys = (height - 2.0 * margin as f64) / limits.height();
            y0 = height - margin as f64 + ((limits.bottom * ys) as i32) as f64;
        }
        if !self.base.is_vector()? {
            return Err(super::raster_not_supported(self.base.parser));
        }
        if !self.base.settings.image_path.is_empty() {
            return Err(super::image_files_not_supported(self.base.parser));
        }
        for c in charts.iter_mut() {
            c.get_chart_svg_points(x0, y0, xs, ys);
        }
        let svg = self.draw_svg(&charts, x0, y0, xs, ys, &limits)?;
        self.captured = Some(self.capture(&charts, &limits, var));
        Ok(svg)
    }

    /// The curves as drawn: in the colour order of `draw_svg` (an empty curve takes none).
    fn capture(&self, charts: &[Chart], limits: &PlotBox, var: &VarRef) -> CapturedChart {
        let _ = var;
        let mut series = Vec::new();
        let mut pen_no = 1usize;
        for (i, c) in charts.iter().enumerate() {
            if c.point_count() == 0 {
                continue;
            }
            series.push(CapturedSeries {
                name: self.names.get(i).cloned().unwrap_or_default(),
                color: SERIES_COLORS[pen_no - 1].to_string(),
                points: c.points.iter().map(|p| (p.x, p.y)).collect(),
                dot: c.bounds.width() == 0.0 && c.bounds.height() == 0.0,
                fill: c.fill,
            });
            pen_no += 1;
            if pen_no > 10 {
                pen_no = 1;
            }
        }
        CapturedChart {
            x_name: String::new(),
            x_unit: self.x_units.as_ref().map(|u| u.text()).unwrap_or_default(),
            y_unit: self.y_units.as_ref().map(|u| u.text()).unwrap_or_default(),
            series,
            bounds: (limits.left, limits.bottom, limits.right, limits.top),
        }
    }

    fn calculate(&mut self, fx: &PlotFunc, fy: &PlotFunc, start: f64, end: f64, u: &OptUnit) -> CResult<Vec<Node>> {
        let n = if self.base.is_adaptive()? { 31 } else { 512 };
        let s = (end - start) / n as f64;
        let mut points = Vec::with_capacity(n + 1);
        let mut t = start;
        self.var.as_ref().unwrap().set_value_units(t, u.clone());
        for _ in 0..=n {
            points.push(self.calculate_point(fx, fy, t)?);
            t += s;
        }
        Ok(points)
    }

    fn calculate_adaptive(&mut self, chart: &Chart, fx: &PlotFunc, fy: &PlotFunc, kxy: f64) -> CResult<Vec<Node>> {
        const TOL: f64 = 0.996;
        let mut points: Vec<Node> = chart.points.clone();
        if chart.point_count() < 4 {
            return Ok(points);
        }
        let scaled = |p: &Node| Node::new(p.x, p.y * kxy, p.z);
        for _ in 0..5 {
            let mut i2 = 1usize;
            let mut i3 = 2usize;
            let mut p1 = scaled(&points[0]);
            let mut p2 = scaled(&points[1]);
            let mut v1 = direction(&p1, &p2);
            let mut added = false;
            while i3 < points.len() {
                let p3 = scaled(&points[i3]);
                let v2 = direction(&p2, &p3);
                let d = (v1.x * v2.x + v1.y * v2.y).abs();
                if d < TOL {
                    let p = self.calculate_point(fx, fy, (p2.z + p3.z) / 2.0)?;
                    if p.y.is_nan() {
                        added = false;
                    } else {
                        points.insert(i2 + 1, p);
                        i3 += 1;
                        if !added {
                            let p = self.calculate_point(fx, fy, (p1.z + p2.z) / 2.0)?;
                            if !p.y.is_nan() {
                                points.insert(i2, p);
                                i3 += 1;
                            }
                        }
                        added = true;
                    }
                } else {
                    added = false;
                }
                v1 = v2;
                p1 = p2;
                p2 = p3;
                i2 = i3;
                i3 += 1;
            }
        }
        Ok(points)
    }

    fn eval(&self, f: &PlotFunc) -> CResult<Value> {
        match f {
            Some(e) => self.base.parser.eval_expr(e),
            None => Err(crate::vector::null_reference()),
        }
    }

    fn calculate_point(&mut self, fx: &PlotFunc, fy: &PlotFunc, t: f64) -> CResult<Node> {
        self.base.parser.break_if_canceled()?;
        let var = self.var.clone().unwrap();
        var.set_number(t);
        let xv = match fx {
            Some(_) => self.eval(fx)?,
            None => var.value().unwrap_or_else(Value::nan),
        };
        let x = convert_value(&xv.as_real_checked(Items::Argument)?, &mut self.x_units)?;
        let yv = self.eval(fy)?;
        let y = convert_value(&yv.as_real_checked(Items::Argument)?, &mut self.y_units)?;
        Ok(Node::new(x, y, t))
    }

    fn draw_svg(&self, charts: &[Chart], _x0: f64, y0: f64, xs: f64, ys: f64, bounds: &PlotBox) -> CResult<String> {
        let b = &self.base;
        let mut svg = SvgDrawing::new(
            b.width as f64,
            b.height as f64,
            b.screen_scale_factor as f64,
            charts.len(),
        )?;
        b.draw_grid_svg(&mut svg, _x0, y0, xs, ys, bounds);
        let mut pen_no = 1;
        let dot_radius = 2.5 * b.screen_scale_factor as f64;
        for c in charts {
            if c.point_count() == 0 {
                continue;
            }
            if c.bounds.width() == 0.0 && c.bounds.height() == 0.0 {
                svg.draw_circle(
                    c.svg_points[0].x,
                    c.svg_points[0].y,
                    dot_radius,
                    &format!("Series{}", pen_no),
                );
            } else {
                svg.draw_polyline(&c.svg_points, &format!("Series{}", pen_no));
                if c.fill {
                    let yf = y0 - clamp(0.0, bounds.bottom * ys, bounds.top * ys)?;
                    fill_chart_svg(&mut svg, &format!("Fill{}", pen_no), yf, &c.svg_points);
                }
            }
            pen_no += 1;
            if pen_no > 10 {
                pen_no = 1;
            }
        }
        self.draw_legend(&mut svg, charts);
        let s = svg.to_string();
        Ok(format!("<svg class=\"plot\" data-plot=\"svg\" {}", &s[4..]))
    }

    /// A box in the top right corner of the plot area with a swatch and the name of each curve, drawn
    /// when at least one curve has a name (a curve without one is listed by its expression).
    fn draw_legend(&self, svg: &mut SvgDrawing, charts: &[Chart]) {
        if !self.legend.iter().any(|n| n.is_some()) {
            return;
        }
        let b = &self.base;
        let sf = b.screen_scale_factor as f64;
        let mut rows = Vec::new();
        let mut pen_no = 1;
        for (i, c) in charts.iter().enumerate() {
            if c.point_count() == 0 {
                continue;
            }
            let name = match self.legend.get(i) {
                Some(Some(n)) => n.clone(),
                _ => self.names.get(i).cloned().unwrap_or_default(),
            };
            rows.push((pen_no, escape_xml(&name), name.chars().count()));
            pen_no += 1;
            if pen_no > 10 {
                pen_no = 1;
            }
        }
        if rows.is_empty() {
            return;
        }
        let fs = 11.0 * sf;
        let row_h = 1.4 * fs;
        let pad = 0.5 * fs;
        let swatch = 2.0 * fs;
        let chars = rows.iter().map(|r| r.2).max().unwrap_or(0) as f64;
        let w = pad * 3.0 + swatch + 0.6 * fs * chars;
        let h = pad * 2.0 + row_h * rows.len() as f64;
        let x = b.width as f64 - b.right as f64 - w - pad;
        let y = b.margin as f64 + pad;
        svg.draw_legend_box(x, y, w, h, sf);
        for (k, (pen, name, _)) in rows.iter().enumerate() {
            let cy = y + pad + row_h * (k as f64 + 0.5);
            let pts = [SvgPoint::new(x + pad, cy), SvgPoint::new(x + pad + swatch, cy)];
            svg.draw_polyline(&pts, &format!("Series{}", pen));
            svg.draw_text(name, x + 2.0 * pad + swatch, cy + 0.35 * fs, "left");
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// `FixLimits(Box limits)`
fn fix_limits(limits: &PlotBox) -> PlotBox {
    let mut left = limits.left;
    let mut right = limits.right;
    let mut bottom = limits.bottom;
    let mut top = limits.top;
    Plotter::fix_bounds(&mut left, &mut right);
    Plotter::fix_bounds(&mut bottom, &mut top);
    if bottom > 0.0 && bottom < 0.35 * top {
        bottom = 0.0;
    } else if top < 0.0 && bottom > 0.35 * top {
        top = 0.0;
    }
    PlotBox::new(left, bottom, right, top)
}

/// `FillChartSVG(svgDrawing, svgClass, y0, points)`
fn fill_chart_svg(svg: &mut SvgDrawing, svg_class: &str, y0: f64, points: &[SvgPoint]) {
    let mut fill = Vec::with_capacity(points.len() + 2);
    fill.push(SvgPoint::new(points[0].x, y0));
    fill.extend_from_slice(points);
    fill.push(SvgPoint::new(points[points.len() - 1].x, y0));
    svg.fill_polygon(&fill, svg_class);
}
