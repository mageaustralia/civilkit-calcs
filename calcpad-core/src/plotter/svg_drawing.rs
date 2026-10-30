//! Port of Plotter/SvgDrawing.cs.
use super::svg_point::SvgPoint;
use crate::math_parser_exception::CResult;
use crate::netfmt;

pub struct SvgDrawing {
    sb: String,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

/// `Math.Round(d, 2).ToString(CultureInfo.InvariantCulture)`
fn convert(d: f64) -> String {
    netfmt::double_to_string(netfmt::round_even_digits(d, 2))
}

impl SvgDrawing {
    pub fn new(width: f64, height: f64, scale_factor: f64, series_count: usize) -> CResult<SvgDrawing> {
        let k = 1.35 * scale_factor;
        let w = convert(width);
        let h = convert(height);
        let wk = convert(width / k);
        let hk = convert(height / k);
        let mut sb = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewbox=\" 0 0 {} {}\" width=\"{}px\" height=\"{}px\" style=\"width:{}pt; height:{}pt;\" >",
            w, h, w, h, wk, hk
        );
        sb.push('\n');
        let mut d = SvgDrawing {
            sb,
            width,
            height,
            scale_factor,
        };
        d.add_style(series_count)?;
        Ok(d)
    }

    fn add_class(&mut self, svg_class: &str) {
        if !svg_class.is_empty() {
            self.sb.push_str("\" class=\"");
            self.sb.push_str(svg_class);
        }
        self.sb.push_str("\" />\n");
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, svg_class: &str) {
        self.sb.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}",
            convert(x1),
            convert(y1),
            convert(x2),
            convert(y2)
        ));
        self.add_class(svg_class);
    }

    pub fn draw_rectangle(&mut self, x: f64, y: f64, w: f64, h: f64, svg_class: &str) {
        self.sb.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}",
            convert(x),
            convert(y),
            convert(w),
            convert(h)
        ));
        self.add_class(svg_class);
    }

    pub fn fill_rectangle(&mut self, x: f64, y: f64, w: f64, h: f64, color: &str) {
        self.sb.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"none\" />\n",
            convert(x),
            convert(y),
            convert(w),
            convert(h),
            color
        ));
    }

    /// The frame of a plot legend: white, slightly see-through, with the plot frame's grey border.
    pub fn draw_legend_box(&mut self, x: f64, y: f64, w: f64, h: f64, scale_factor: f64) {
        self.sb.push_str(&format!(
            "<rect class=\"Legend\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"white\" fill-opacity=\"0.9\" stroke=\"rgba(0, 0, 0, 0.24)\" stroke-width=\"{}\" />\n",
            convert(x),
            convert(y),
            convert(w),
            convert(h),
            convert(2.0 * scale_factor),
            convert(scale_factor)
        ));
    }

    pub fn fill_polygon(&mut self, points: &[SvgPoint], svg_class: &str) {
        self.sb.push_str("<polygon");
        self.draw_points(points);
        self.sb.push_str("\" stroke-width=\"0");
        self.add_class(svg_class);
    }

    pub fn draw_circle(&mut self, x: f64, y: f64, r: f64, svg_class: &str) {
        self.sb.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}",
            convert(x),
            convert(y),
            convert(r)
        ));
        self.add_class(svg_class);
    }

    pub fn draw_polyline(&mut self, points: &[SvgPoint], svg_class: &str) {
        self.sb.push_str("<polyline");
        self.draw_points(points);
        self.add_class(svg_class);
    }

    pub fn draw_points(&mut self, points: &[SvgPoint]) {
        self.sb.push_str(" points=\"");
        self.sb.push_str(&points[0].to_string());
        for p in &points[1..] {
            self.sb.push(' ');
            self.sb.push_str(&p.to_string());
        }
    }

    pub fn draw_text(&mut self, text: &str, x: f64, y: f64, svg_class: &str) {
        self.sb
            .push_str(&format!("<text x=\"{}\" y=\"{}", convert(x), convert(y)));
        if !svg_class.is_empty() {
            self.sb.push_str("\" class=\"");
            self.sb.push_str(svg_class);
        }
        self.sb.push_str("\">");
        self.sb.push_str(text);
        self.sb.push_str(" </text>\n");
    }

    pub fn draw_image(&mut self, x: f64, y: f64, w: f64, h: f64, src: &str) {
        self.sb.push_str(&format!(
            "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" href=\"{}\" />\n",
            convert(x),
            convert(y),
            convert(w),
            convert(h),
            src
        ));
    }

    fn add_style(&mut self, n: usize) -> CResult<()> {
        if n > 10 {
            return Err(crate::vector::index_was_outside());
        }
        let sf2 = 1.5 * self.scale_factor;
        const COLORS: [&str; 10] = [
            "Tomato",
            "YellowGreen",
            "CornflowerBlue",
            "#f0d000",
            "MediumVioletRed",
            "MediumSpringGreen",
            "BlueViolet",
            "LightSalmon",
            "DeepPink",
            "DarkTurquoise",
        ];
        const OPACITIES: [&str; 10] = [
            "0.050", "0.045", "0.040", "0.035", "0.030", "0.025", "0.020", "0.020", "0.020", "0.020",
        ];
        let fs = convert(11.0 * self.scale_factor);
        let sb = &mut self.sb;
        sb.push_str("<style type=\"text/css\">\n");
        sb.push_str(&format!(
            ".plot text {{fill:Black; font-family:'Segoe UI', Sans; font-size:{}px}}\n",
            fs
        ));
        sb.push_str(".plot text.left {text-anchor: start;}\n");
        sb.push_str(&format!(".plot text.sm {{font-size:{}px}}\n", fs));
        sb.push_str(".plot text.mid {text-anchor: middle;}\n");
        sb.push_str(".plot text.end {text-anchor: end;}\n");
        sb.push_str(&format!(
            ".plot .Grid, .plot .Frame, .plot .Axis{{fill:none; stroke-width:{};}}\n",
            convert(self.scale_factor)
        ));
        sb.push_str(".plot .Grid {stroke:rgba(0, 0, 0, 0.08);}\n");
        sb.push_str(".plot .Frame {stroke:rgba(0, 0, 0, 0.24);}\n");
        sb.push_str(".plot .Axis {stroke: Black;}\n");
        sb.push_str(&format!(
            ".plot polyline {{fill:none; stroke-width:{};}}\n",
            convert(sf2)
        ));
        sb.push_str(".plot [class^=\"Fill\"] {stroke:none;}\n");
        for i in 1..=n {
            let color = COLORS[i - 1];
            sb.push_str(&format!(".Series{} {{stroke:{};}}\n", i, color));
            sb.push_str(&format!("circle.Series{} {{fill:{};}}\n", i, color));
            sb.push_str(&format!(
                ".Fill{} {{fill:{}; fill-opacity:{};}}\n",
                i,
                color,
                OPACITIES[i - 1]
            ));
        }
        sb.push_str("</style><g class=\"plot\">\n");
        Ok(())
    }
}

impl std::fmt::Display for SvgDrawing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}</g></svg>", self.sb)
    }
}
