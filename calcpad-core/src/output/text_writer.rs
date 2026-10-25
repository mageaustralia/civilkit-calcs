//! Port of Output/TextWriter.cs.
use super::*;

pub struct TextWriter {
    base: WriterBase,
}

impl TextWriter {
    pub const UNIT_DIVISION: char = '\u{2215}';
    pub const UNIT_PRODUCT: char = '\u{00B7}';

    pub fn new(settings: Option<&MathSettings>, phasor: bool) -> TextWriter {
        TextWriter {
            base: WriterBase::new(settings, phasor),
        }
    }

    pub fn with_decimals(decimals: i32) -> TextWriter {
        let mut base = WriterBase::new(None, false);
        base.decimals = decimals;
        TextWriter { base }
    }
}

impl OutputWriter for TextWriter {
    fn base(&self) -> &WriterBase {
        &self.base
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Text
    }

    fn angle_units(&self) -> [&'static str; 3] {
        ["\u{00B0}", "", " grad"]
    }

    fn unit_string(&self, units: &UnitRef) -> String {
        units.text().replace('\u{2031}', "\u{2031} ")
    }

    fn format_input(&self, s: &str, units: &Option<UnitRef>, _line: i32, _is_calculated: bool) -> String {
        match units {
            None => s.to_string(),
            Some(u) => format!("{} {}", s, u.text()),
        }
    }

    fn format_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}_{}", sa, sb)
    }

    fn append_subscript(&self, sa: &str, sb: &str) -> String {
        format!("{}.{}", sa, sb)
    }

    fn format_variable(&self, name: &str, _value: &str, _is_bold: bool) -> String {
        if name.starts_with('\u{20D7}') {
            name['\u{20D7}'.len_utf8()..].to_string()
        } else {
            name.to_string()
        }
    }

    fn format_units(&self, s: &str) -> String {
        s.to_string()
    }

    fn format_function(&self, s: &str) -> String {
        s.to_string()
    }

    fn format_switch(&self, sa: &[String], _level: i32) -> String {
        format!("switch({})", sa.join("; "))
    }

    fn format_if(&self, sc: &str, sa: &str, sb: &str, _level: i32) -> String {
        format!("if({}; {}; {})", sc, sa, sb)
    }

    fn format_root(&self, s: &str, _level: i32, n: &str) -> String {
        match n {
            "2" => format!("\u{221A}({})", s),
            "3" => format!("\u{00B3}\u{221A}({})", s),
            "4" => format!("\u{2074}\u{221A}({})", s),
            _ => format!("root({}; {})", s, n),
        }
    }

    fn format_operator(&self, c: char) -> String {
        format_operator_helper(c)
    }

    fn format_power(&self, sa: &str, sb: &str, _level: i32, _order: i32) -> String {
        format!("{}^{}", sa, sb)
    }

    fn format_division(&self, sa: &str, sb: &str, _level: i32) -> String {
        format!("{}/{}", sa, sb)
    }

    fn format_nary(&self, symbol: &str, sub: &str, sup: &str, expr: &str) -> String {
        format!("{}{{{}; {}...{}}}", symbol, expr, sub, sup)
    }

    fn format_value(&self, value: &Value) -> String {
        let u = value.units();
        let fs = u.as_ref().and_then(|x| x.format_string());
        let mut s = self.format_complex(value.re(), value.im(), fs.as_deref());
        let u_text = u.as_ref().map(|x| x.text()).unwrap_or_default();
        if u_text.is_empty() {
            return s;
        }
        if !(value.is_real() || self.base.phasor && s.contains('\u{2220}')) {
            s = self.add_brackets(&s, 0, '(', ')');
        }
        if u_text == "\u{00B0}" {
            return format!("{}{}", s, u_text);
        }
        format!("{} {}", s, u_text)
    }

    fn add_brackets(&self, s: &str, _level: i32, left: char, right: char) -> String {
        format!("{}{}{}", left, s, right)
    }

    fn format_abs(&self, s: &str, _level: i32) -> String {
        format!("|{}|", s)
    }

    fn format_real(&self, d: f64, format: Option<&str>, zero_small: bool) -> String {
        let s = self.format_number_helper(d, format);
        if d.is_nan() || d.is_infinite() {
            return s;
        }
        let i = match s.rfind('E') {
            Some(i) if i > 0 => i,
            _ => return s,
        };
        if zero_small && s[i + 1..].starts_with('-') {
            return "0".to_string();
        }
        s
    }

    fn format_complex(&self, re: f64, im: f64, format: Option<&str>) -> String {
        if re.is_nan() && im.is_nan() {
            return " Undefined ".to_string();
        }
        if re.is_infinite() && im.is_infinite() {
            return "\u{221E}".to_string();
        }
        self.format_complex_helper(re, im, format)
    }

    fn format_matrix(&self, matrix: &Matrix) -> String {
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        let mut sb = String::from("[");
        let nc = matrix.col_count() as i64 - 1;
        let (max, row, col) = super::get_max_visible_matrix_value(matrix, max_count);
        let zero_threshold = max * 1e-14;
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
        let len = if common.is_some() {
            let d = super::scaled_d(&common, &matrix.get(row, col));
            self.format_real(
                d,
                fs.as_deref(),
                self.base.zero_small_elements && d.abs() < zero_threshold,
            )
        } else {
            self.format_matrix_value(&matrix.get(row, col), zero_threshold)
        }
        .encode_utf16()
        .count();
        let pad = |s: &str| -> String {
            let l = s.encode_utf16().count();
            if l >= len {
                s.to_string()
            } else {
                format!("{}{}", " ".repeat(len - l), s)
            }
        };
        let cont = pad("...");
        let nr = matrix.row_count() as i64 - 1;
        let mut i = 0i64;
        while i <= nr {
            if i == mc as i64 {
                for _ in 0..(mc as i64).min(nc).max(0) {
                    sb.push_str(&cont);
                }
                i = nr;
            }
            let mut j = 0i64;
            while j <= nc {
                if j == mc as i64 {
                    sb.push_str(&cont);
                    j = nc;
                }
                let s = if !hp {
                    self.format_matrix_value(&matrix.get(i as usize, j as usize), zero_threshold)
                } else {
                    let d = super::scaled_d(&common, &matrix.get(i as usize, j as usize));
                    self.format_real(
                        d,
                        fs.as_deref(),
                        self.base.zero_small_elements && d.abs() < zero_threshold,
                    )
                };
                sb.push_str(&pad(&s));
                if j < nc {
                    sb.push_str("  ");
                }
                j += 1;
            }
            if i < nr {
                sb.push_str(" |");
            }
            i += 1;
        }
        sb.push(']');
        if let Some(u) = &units {
            sb.push_str(&u.text());
        }
        sb
    }

    fn format_vector(&self, vector: &Vector) -> String {
        let div = "  ";
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        let mut sb = String::from("[");
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
        for i in 0..len {
            if i > 0 {
                sb.push_str(div);
            }
            if i == mc {
                sb.push_str("...");
                sb.push_str(div);
                break;
            }
            append(&mut sb, i);
        }
        let last = len as i64 - 1;
        if (max_count as i64) < last {
            append(&mut sb, last as usize);
        }
        sb.push(']');
        if let Some(u) = &units {
            sb.push_str(&u.text());
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
            Some(u) => format!("{}{}", s, u.text()),
        }
    }

    fn format_block(&self, sa: &[String]) -> String {
        format!(" \u{2502} {}", sa.join(" \u{2502} "))
    }

    fn close_curly_brackets(&self, sa: &str, _level: i32) -> String {
        sa.to_string()
    }
}
