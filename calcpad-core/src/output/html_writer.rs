//! Port of Output/HtmWriter.cs.
use super::*;

pub struct HtmlWriter {
    base: WriterBase,
}

const SQR_PAD: [&str; 4] = [
    "&ensp;&hairsp;&hairsp;",
    "&ensp;&ensp;",
    "&emsp;&thinsp;",
    "&emsp;&ensp;",
];

fn root_pad(level: i32, n: &str) -> String {
    match level {
        0 => format!("&hairsp;<sup class=\"nth\">{}</sup>&hairsp;&hairsp;", n),
        1 => format!("&nbsp;<small class=\"nth\">{}</small>&nbsp;&hairsp;", n),
        2 => format!("&hairsp;&nbsp;<small class=\"nth\">{}</small>&nbsp;&thinsp;", n),
        _ => format!("&ensp;<small class=\"nth\">{}</small>&ensp;", n),
    }
}

fn format_left_curl(level: i32) -> String {
    format!("<span class=\"c{}\">{{</span>", level)
}

fn format_right_curl(level: i32) -> String {
    format!("<span class=\"c{}\">}}</span>", level)
}

fn format_brackets(symbol: char, level: i32) -> String {
    format!("<span class=\"b{}\">{}</span>", level, symbol)
}

/// `s[..^n]` on chars.
fn drop_last_chars(s: &str, n: usize) -> &str {
    let mut idx = s.len();
    for _ in 0..n {
        match s[..idx].char_indices().next_back() {
            Some((i, _)) => idx = i,
            None => return "",
        }
    }
    &s[..idx]
}

impl HtmlWriter {
    pub const UNIT_DIVISION: &'static str = "<i class=\"unit\">\u{2009}\u{2215}\u{2009}</i>";
    pub const UNIT_PRODUCT: &'static str = "<i>\u{200A}\u{00B7}\u{200A}</i>";

    pub fn new(settings: Option<&MathSettings>, phasor: bool) -> HtmlWriter {
        HtmlWriter {
            base: WriterBase::new(settings, phasor),
        }
    }

    pub fn offset_division(s: &str, offset: i32) -> String {
        let n = s.find("\"dvc\"").map(|i| i as i64).unwrap_or(-1) + 4;
        let n = n as usize;
        match offset {
            -1 => format!("{} down{}", &s[..n], &s[n..]),
            1 => format!("{} up{}", &s[..n], &s[n..]),
            _ => s.to_string(),
        }
    }
}

impl OutputWriter for HtmlWriter {
    fn base(&self) -> &WriterBase {
        &self.base
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Html
    }

    fn angle_units(&self) -> [&'static str; 3] {
        ["\u{00B0}", "", "\u{2009}<i>grad</i>"]
    }

    fn unit_string(&self, units: &UnitRef) -> String {
        units.html()
    }

    fn format_input(&self, s: &str, units: &Option<UnitRef>, line: i32, is_calculated: bool) -> String {
        let output = if s == "?" {
            format!(
                "<input type=\"text\" size=\"2\" name=\"Var\" class=\"input-{}\">\u{2009}",
                line
            )
        } else if is_calculated {
            format!("<u class=\"input-{}\">{}</u>\u{2009}", line, s)
        } else {
            format!(
                "<input type=\"text\" size=\"2\" name=\"Var\" class=\"input-{}\" value=\"{}\">\u{2009}",
                line, s
            )
        };
        match units {
            Some(u) => format!("{} <i>{}</i>", output, u.html()),
            None => output,
        }
    }

    fn append_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}.{}</sub>", drop_last_chars(sa, 6), sb)
    }

    fn format_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}<sub>{}</sub>", sa, sb)
    }

    fn format_variable(&self, name: &str, value: &str, is_bold: bool) -> String {
        let s = if value.is_empty() {
            String::new()
        } else {
            format!(" class=\"value\" data-value='{}'", value)
        };
        let mut is_bold = is_bold;
        let mut name = name.to_string();
        if name.starts_with('\u{20D7}') {
            is_bold = false;
            name = format!("<span class=\"vec\">\u{20D7}</span>{}", &name['\u{20D7}'.len_utf8()..]);
        }
        let format_bold = |x: String| if is_bold { format!("<b>{}</b>", x) } else { x };
        if let Some(i) = name.find('_') {
            if i > 0 {
                let i1 = i + 1;
                if i1 < name.len() {
                    return self.format_subscript(&format_bold(format!("<var{}>{}</var>", s, &name[..i])), &name[i1..]);
                }
            }
        }
        format_bold(format!("<var{}>{}</var>", s, name))
    }

    fn format_units(&self, s: &str) -> String {
        if s == "10" {
            "10".to_string()
        } else {
            format!("<i>{}</i>", s)
        }
    }

    fn format_function(&self, s: &str) -> String {
        if let Some(i) = s.find('_') {
            if i > 0 {
                let i1 = i + 1;
                if i1 < s.len() {
                    return self.format_subscript(&format!("<b>{}</b>", &s[..i]), &s[i1..]);
                }
            }
        }
        format!("<b>{}</b>", s)
    }

    fn format_root(&self, s: &str, level: i32, n: &str) -> String {
        if n != "2" {
            if level > 0 {
                return format!(
                    "{}<span class=\"o{}\"><span class=\"r{}\"></span>{}</span>",
                    root_pad(level, n),
                    level,
                    level,
                    s
                );
            }
            return format!(
                "{}<span class=\"o0\"><span class=\"r\">\u{221A}</span>&hairsp;{}</span>",
                root_pad(0, n),
                s
            );
        }
        if level > 0 {
            let pad = SQR_PAD.get(level as usize).copied().unwrap_or(SQR_PAD[3]);
            return format!(
                "{}<span class=\"o{}\"><span class=\"r{}\"></span>{}</span>",
                pad, level, level, s
            );
        }
        format!(
            "{}<span class=\"o0\"><span class=\"r\">\u{221A}</span>&hairsp;{}</span>",
            SQR_PAD[0], s
        )
    }

    fn format_operator(&self, c: char) -> String {
        match c {
            '/' | '\u{00F7}' => "<em>\u{200A}/\u{200A}</em>".to_string(),
            '*' => "\u{2006}\u{00B7}\u{2006}".to_string(),
            '<' => " &lt; ".to_string(),
            '>' => " &gt; ".to_string(),
            '\u{2264}' => " &le; ".to_string(),
            '\u{2265}' => " &ge; ".to_string(),
            '|' => format!(" {} ", format_brackets('|', 0)),
            '\u{2299}' => "<small class=\"nth\">\u{2299}</small>".to_string(),
            _ => format_operator_helper(c),
        }
    }

    fn format_power(&self, sa: &str, sb: &str, level: i32, order: i32) -> String {
        let s = if level > 0 {
            "<sup class=\"raised\">"
        } else if level < 0 {
            "<sup class=\"unit\">"
        } else {
            "<sup>"
        };
        if order == POWER_ORDER {
            return format!("{}\u{2219}{}{}</sup>", sa, s, sb);
        }
        format!("{}{}{}</sup>", sa, s, sb)
    }

    fn format_division(&self, sa: &str, sb: &str, level: i32) -> String {
        if level < 4 {
            let (sa, sb) = if level == 3 {
                (
                    sa.replace("dvc up", "dvc").replace("dvc down", "dvc"),
                    sb.replace("dvc up", "dvc").replace("dvc down", "dvc"),
                )
            } else {
                (sa.to_string(), sb.to_string())
            };
            return format!("<span class=\"dvc\">{}<span class=\"dvl\"></span>{}</span>", sa, sb);
        }
        format!("{} \u{00F7} {}", sa, sb)
    }

    fn format_nary(&self, symbol: &str, sub: &str, sup: &str, expr: &str) -> String {
        format!(
            "<span class=\"dvr\"><small>{}</small><span class=\"nary\">{}</span><small>{}</small></span>{}",
            sup, symbol, sub, expr
        )
    }

    fn format_value(&self, value: &Value) -> String {
        let u = value.units();
        let fs = u.as_ref().and_then(|x| x.format_string());
        let mut s = self.format_complex(value.re(), value.im(), fs.as_deref());
        let u_html = u.as_ref().map(|x| x.html()).unwrap_or_default();
        if u_html.is_empty() {
            return s;
        }
        if !(value.is_real() || self.base.phasor && s.contains('\u{2220}')) {
            s = self.add_brackets(&s, 0, '(', ')');
        }
        if u.as_ref().unwrap().is_angle() && u_html.starts_with('\u{00B0}') {
            return format!("{}{}", s, u_html);
        }
        format!("{}{}{}", s, THIN_SPACE, u_html)
    }

    fn add_brackets(&self, s: &str, level: i32, left: char, right: char) -> String {
        if level == 0 && left == '(' {
            format!("\u{200A}{}\u{200A}{}\u{200A}{}\u{200A}", left, s, right)
        } else {
            format!("{}{}{}", format_brackets(left, level), s, format_brackets(right, level))
        }
    }

    fn format_abs(&self, s: &str, level: i32) -> String {
        match level {
            0 => format!("<b class=\"b0\">|</b>&hairsp;{}&hairsp;<b class=\"b0\">|</b>", s),
            _ => format!(
                "<span class=\"b{}\">|</span>{}<span class=\"b{}\">|</span>",
                level, s, level
            ),
        }
    }

    fn format_real(&self, d: f64, format: Option<&str>, zero_small: bool) -> String {
        let s = self.format_number_helper(d, format);
        if d.is_nan() || d.is_infinite() {
            return format!("<span class=\"err\">{}</span>", s);
        }
        let i = match s.rfind('E') {
            Some(i) if i > 0 => i,
            _ => return s,
        };
        let bytes = s.as_bytes();
        let mut i1 = i + 1;
        let sign = bytes.get(i1).map(|&b| b as char).unwrap_or('\0');
        if i1 < s.len() - 1 {
            i1 += 1;
        }
        let fmt_owned: Option<String> = format
            .map(|x| x.to_string())
            .or_else(|| self.base.format_string.clone());
        if sign == '+' || sign == '-' || sign == '0' {
            if zero_small && sign == '-' {
                return "0".to_string();
            }
            let has_e = fmt_owned.as_deref().is_some_and(|f| f.contains('E'));
            if bytes.get(i1) == Some(&b'0') && !has_e {
                i1 += 1;
            }
        }
        let mut ms = s[..i].to_string();
        if ms == "1" {
            ms = String::new();
        } else {
            ms.push('\u{00D7}');
        }
        let es = &s[i1.min(s.len())..];
        if sign == '+' {
            if let Some(f) = fmt_owned.as_deref() {
                if f.contains("E+") || f.starts_with('E') {
                    return format!("{}10<sup>+{}</sup>", ms, es);
                }
            }
        }
        if sign == '-' {
            format!("{}10<sup>-{}</sup>", ms, es)
        } else {
            format!("{}10<sup>{}</sup>", ms, es)
        }
    }

    fn format_complex(&self, re: f64, im: f64, format: Option<&str>) -> String {
        if re.is_nan() && im.is_nan() {
            return "<span class=\"err\"> Undefined </span>".to_string();
        }
        if re.is_infinite() && im.is_infinite() {
            return "<span class=\"err\"> \u{221E} </span>".to_string();
        }
        self.format_complex_helper(re, im, format)
    }

    fn format_switch(&self, sa: &[String], level: i32) -> String {
        let len = sa.len();
        if len == 1 {
            return format!("<b>switch</b>{}", self.add_brackets(&sa[0], level, '(', ')'));
        }
        if len == 2 {
            return format!(
                "<b>switch</b>{}",
                self.add_brackets(&format!("{}; {}", sa[0], sa[1]), level, '(', ')')
            );
        }
        let level = level.min(8);
        let mut sb = String::from("<span class=\"dvcs\">");
        sb.push_str(&format_left_curl(level));
        sb.push_str(&format!("<span class=\"dvs\">if {}: {}", sa[0], sa[1]));
        let mut i = 2;
        while i < len {
            if len - i == 1 {
                sb.push_str(&format!("<br />else: {}", sa[i]));
            } else {
                sb.push_str(&format!("<br />else if {}: {}", sa[i], sa[i + 1]));
            }
            i += 2;
        }
        sb.push_str("</span></span>");
        sb
    }

    fn close_curly_brackets(&self, sa: &str, level: i32) -> String {
        format!("{}{}</span>", drop_last_chars(sa, 7), format_right_curl(level))
    }

    fn format_if(&self, sc: &str, sa: &str, sb: &str, level: i32) -> String {
        let s = format!("if {}: {}<br />else: {}", sc, sa, sb);
        format!(
            "<span class=\"dvcs\">{}<span class=\"dvs\">{}</span></span>",
            format_left_curl(level),
            s
        )
    }

    fn format_matrix(&self, matrix: &Matrix) -> String {
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        let mut sb = String::new();
        let nc = matrix.col_count();
        let zero_threshold = super::get_max_visible_matrix_value(matrix, max_count).0 * 1e-14;
        let is_hp = matrix.is_hp();
        let common = if is_hp {
            None
        } else {
            super::common_matrix_unit(&self.base, matrix)
        };
        let hp = is_hp || common.is_some();
        let units = if is_hp {
            crate::matrix::hp_matrix::units(matrix)
        } else {
            common.clone()
        };
        let fs = units.as_ref().and_then(|u| u.format_string());
        sb.push_str("<span class=\"matrix\">\n");
        let nr = matrix.row_count();
        let mut i = 0usize;
        while i < nr {
            sb.push_str("<span class=\"tr\"><span class=\"td\"></span>");
            if i == mc {
                for _ in 0..mc.min(nc) {
                    sb.push_str("<span class=\"td\">\u{22EE}</span>");
                }
                if nc > mc {
                    sb.push_str("<span class=\"td\">\u{22F1}</span><span class=\"td\">\u{22EE}</span>");
                }
                sb.push_str("<span class=\"td\"></span></span><span class=\"tr\"><span class=\"td\"></span>");
                i = nr - 1;
            }
            let mut j = 0usize;
            while j < nc {
                if j == mc {
                    sb.push_str("<span class=\"td\">\u{22EF}</span>");
                    j = nc - 1;
                }
                let s = if !hp {
                    self.format_matrix_value(&matrix.get(i, j), zero_threshold)
                } else {
                    let d = super::scaled_d(&common, &matrix.get(i, j));
                    self.format_real(
                        d,
                        fs.as_deref(),
                        self.base.zero_small_elements && d.abs() < zero_threshold,
                    )
                };
                sb.push_str("<span class=\"td\">");
                sb.push_str(&s);
                sb.push_str("</span>");
                j += 1;
            }
            sb.push_str("<span class=\"td\"></span></span>\n");
            i += 1;
        }
        sb.push_str("</span>\n");
        if let Some(u) = &units {
            sb.push(HAIR_SPACE);
            sb.push_str(&u.html());
        }
        sb
    }

    fn format_vector(&self, vector: &Vector) -> String {
        let div = VECTOR_SPACING;
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        let mut sb = String::new();
        const TOL: f64 = 1e-14;
        let mut zero_threshold = super::get_max_visible_vector_value(vector, max_count) * TOL;
        if zero_threshold > TOL {
            zero_threshold = TOL;
        }
        let is_hp = vector.is_hp();
        let common = if is_hp {
            None
        } else {
            super::common_vector_unit(&self.base, vector)
        };
        let hp = is_hp || common.is_some();
        let units = if is_hp { vector.hp_units() } else { common.clone() };
        let fs = units.as_ref().and_then(|u| u.format_string());
        let append = |sb: &mut String, index: usize| {
            if !hp {
                sb.push_str(&self.format_matrix_value(&vector.get(index), zero_threshold));
            } else {
                let d = super::scaled_d(&common, &vector.get(index));
                sb.push_str(&self.format_real(
                    d,
                    fs.as_deref(),
                    self.base.zero_small_elements && d.abs() < zero_threshold,
                ));
            }
        };
        let len = vector.len();
        sb.push_str("<b class=\"b0\">[</b>");
        for i in 0..len {
            if i > 0 {
                sb.push_str(div);
            }
            if i == mc {
                let n = len as i64 - mc as i64;
                let skipped = n - (n - 1).signum();
                sb.push_str(&format!("<span title=\"{} elements skipped.\">...</span>", skipped));
                sb.push_str(div);
                break;
            }
            append(&mut sb, i);
        }
        let last = len as i64 - 1;
        if (max_count as i64) < last {
            append(&mut sb, last as usize);
        }
        sb.push_str("<b class=\"b0\">]</b>");
        if let Some(u) = &units {
            sb.push(HAIR_SPACE);
            sb.push_str(&u.html());
        }
        sb
    }

    fn format_matrix_value(&self, value: &RealValue, zero_threshold: f64) -> String {
        let d = value.d;
        let fs = value.units.as_ref().and_then(|u| u.format_string());
        let s = self.format_real(
            d,
            fs.as_deref(),
            self.base.zero_small_elements && d.abs() < zero_threshold,
        );
        match &value.units {
            None => s,
            Some(u) => format!("{}{}{}", s, THIN_SPACE, u.html()),
        }
    }

    fn format_block(&self, sa: &[String]) -> String {
        format!(
            "<small class=\"arr\">\u{1F780}</small><span class=\"block\">{}</span>",
            sa.join("<br /> ")
        )
    }
}
