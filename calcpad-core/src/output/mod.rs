//! Port of Output/OutputWriter.cs (abstract writer + shared helpers).
pub mod html_writer;
pub mod text_writer;
pub mod xml_writer;

use crate::base_types::complex::{Complex, Types};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{get_product_or_division_operator, UnitRef};
use crate::base_types::value::Value;
use crate::calculator::NEG_CHAR;
use crate::exceptions;
use crate::extension_methods::is_white_space;
use crate::math_parser_exception::CResult;
use crate::matrix::Matrix;
use crate::netfmt;
use crate::settings::MathSettings;
use crate::vector::Vector;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputFormat {
    Text,
    Html,
    Xml,
}

pub const ANGLE_FACTORS: [f64; 3] = [180.0 / std::f64::consts::PI, 1.0, 200.0 / std::f64::consts::PI];
pub const THIN_SPACE: char = '\u{2009}';
pub const HAIR_SPACE: char = '\u{200A}';
pub const VECTOR_SPACING: &str = "\u{2002}";
/// `OperatorOrder['^']`
pub const POWER_ORDER: i32 = 0;

/// Settings captured by an OutputWriter at construction.
#[derive(Clone, Debug)]
pub struct WriterBase {
    pub decimals: i32,
    pub format_string: Option<String>,
    pub format_equations: bool,
    pub zero_small_elements: bool,
    pub max_count: i32,
    /// Write the unit on every element of a vector or matrix (`#settings {"unitPerElement": true}`).
    pub unit_per_element: bool,
    pub phasor: bool,
    pub degrees: i32,
}

impl WriterBase {
    pub fn new(settings: Option<&MathSettings>, phasor: bool) -> WriterBase {
        match settings {
            None => WriterBase {
                decimals: 2,
                format_string: None,
                format_equations: false,
                zero_small_elements: false,
                max_count: 20,
                unit_per_element: true,
                phasor: false,
                degrees: 0,
            },
            Some(s) => WriterBase {
                decimals: s.decimals(),
                format_string: s.format_string.clone(),
                format_equations: s.format_equations,
                zero_small_elements: s.zero_small_matrix_elements,
                max_count: s.max_output_count(),
                unit_per_element: s.unit_per_element,
                phasor,
                degrees: s.degrees,
            },
        }
    }
}

pub trait OutputWriter {
    fn base(&self) -> &WriterBase;
    fn format_type(&self) -> OutputFormat;
    fn angle_units(&self) -> [&'static str; 3];

    fn unit_string(&self, units: &UnitRef) -> String;
    fn format_input(&self, s: &str, units: &Option<UnitRef>, line: i32, is_calculated: bool) -> String;
    fn format_subscript(&self, sa: &str, sb: &str) -> String;
    fn append_subscript(&self, sa: &str, sb: &str) -> String;
    fn format_variable(&self, name: &str, value: &str, is_bold: bool) -> String;
    fn format_units(&self, s: &str) -> String;
    fn format_function(&self, s: &str) -> String;
    fn format_switch(&self, sa: &[String], level: i32) -> String;
    fn format_if(&self, sc: &str, sa: &str, sb: &str, level: i32) -> String;
    fn format_root(&self, s: &str, level: i32, n: &str) -> String;
    fn format_operator(&self, c: char) -> String;
    fn format_power(&self, sa: &str, sb: &str, level: i32, order: i32) -> String;
    fn format_division(&self, sa: &str, sb: &str, level: i32) -> String;
    fn format_nary(&self, symbol: &str, sub: &str, sup: &str, expr: &str) -> String;
    fn format_value(&self, value: &Value) -> String;
    fn add_brackets(&self, s: &str, level: i32, left: char, right: char) -> String;
    fn format_abs(&self, s: &str, level: i32) -> String;
    fn format_real(&self, d: f64, format: Option<&str>, zero_small: bool) -> String;
    fn format_complex(&self, re: f64, im: f64, format: Option<&str>) -> String;
    fn format_block(&self, sa: &[String]) -> String;
    fn close_curly_brackets(&self, sa: &str, level: i32) -> String;
    fn format_matrix(&self, matrix: &Matrix) -> String;
    fn format_vector(&self, vector: &Vector) -> String;
    fn format_matrix_value(&self, value: &RealValue, zero_threshold: f64) -> String;

    /// `FormatUnitsText`
    fn format_units_text(&self, text: &str) -> CResult<String> {
        let mut sb = String::new();
        let mut literal = String::new();
        let mut power = String::new();
        let mut is_power = false;
        let mut sub: u8 = 0;
        let mut brackets: Vec<usize> = Vec::new();
        let is_text = self.format_type() == OutputFormat::Text;

        // Helper closures cannot borrow self mutably; implemented as inner fns.
        fn format_local(w: &(impl OutputWriter + ?Sized), s: &str, sub: &mut u8) -> String {
            let n = *sub as usize;
            if (2..=4).contains(&n) {
                *sub = 0;
                let chars: Vec<char> = s.chars().collect();
                let head: String = chars[..chars.len() - n].iter().collect();
                let tail: String = chars[chars.len() - (n - 1)..].iter().collect();
                return w.format_subscript(&w.format_units(&head), &format!("{}{}", HAIR_SPACE, tail));
            }
            if s.contains(['/', '\u{2215}', '\u{00B7}']) {
                return s.to_string();
            }
            w.format_units(s)
        }

        fn append_power(
            w: &(impl OutputWriter + ?Sized),
            sb: &mut String,
            literal: &mut String,
            power: &mut String,
            is_power: &mut bool,
            sub: &mut u8,
        ) {
            let is_xml = w.format_type() == OutputFormat::Xml;
            let ls = if literal.chars().count() != 1 || !literal.starts_with('\u{00B0}') {
                format_local(w, literal, sub)
            } else if is_xml {
                xml_writer::run(literal)
            } else {
                literal.clone()
            };
            if *is_power {
                let ps = if is_xml { xml_writer::run(power) } else { power.clone() };
                sb.push_str(&w.format_power(&ls, &ps, -1, -1));
            } else {
                sb.push_str(&ls);
            }
            literal.clear();
            power.clear();
            *is_power = false;
        }

        for ch in text.chars() {
            let mut c = ch;
            if is_white_space(c) {
                continue;
            }
            if c == '\u{00B7}' {
                c = '*';
            }
            match c {
                '\u{2215}' | '/' | '*' => {
                    let in_power_bracket = is_power && brackets.last().is_some_and(|&ind| ind < power.len());
                    if in_power_bracket {
                        let oper = self.format_operator(c);
                        power.push_str(&oper);
                    } else {
                        let oper = get_product_or_division_operator(self.format_type(), c == '*');
                        append_power(self, &mut sb, &mut literal, &mut power, &mut is_power, &mut sub);
                        sb.push_str(oper);
                    }
                }
                '^' => is_power = true,
                '(' => brackets.push(if is_power { power.len() } else { sb.len() }),
                ')' => {
                    let index = match brackets.pop() {
                        None => return Err(exceptions::MissingLeftBracket()),
                        Some(i) => i,
                    };
                    if is_power && index < power.len() {
                        if index == 0 {
                            if is_text {
                                power.insert(0, '(');
                                power.push(')');
                            }
                        } else {
                            power.insert(index, '(');
                            power.push(')');
                        }
                    } else {
                        append_power(self, &mut sb, &mut literal, &mut power, &mut is_power, &mut sub);
                        let s = sb[index..].to_string();
                        sb.truncate(index);
                        literal.clear();
                        literal.push_str(&self.add_brackets(&s, 0, '(', ')'));
                    }
                }
                _ => {
                    if is_power {
                        power.push(c);
                    } else {
                        let cl = literal.chars().last().unwrap_or('\0');
                        let is_sub = match sub {
                            0 => c == '_',
                            1 => c == 'U' || c == 'd' || c == 'm' || c == 'f',
                            2 => match cl {
                                'U' => c == 'K' || c == 'S',
                                'd' => c == 'r',
                                _ => false,
                            },
                            3 => cl == 'r' && c == 'y',
                            _ => false,
                        };
                        if is_sub {
                            sub += 1;
                        } else if c == '_' {
                            sub = 1;
                        } else {
                            sub = 0;
                        }
                        literal.push(c);
                    }
                }
            }
        }
        if !brackets.is_empty() {
            return Err(exceptions::MissingRightBracket());
        }
        if !literal.is_empty() {
            append_power(self, &mut sb, &mut literal, &mut power, &mut is_power, &mut sub);
        }
        Ok(sb)
    }

    fn format_complex_helper(&self, re: f64, im: f64, format: Option<&str>) -> String {
        let t = Complex::get_type(re, im);
        if t == Types::Real {
            return self.format_real(re, format, false);
        }
        if t == Types::Imaginary {
            return format!("{}i", self.format_real(im, format, false));
        }
        let b = self.base();
        if b.phasor {
            let abs = (re * re + im * im).sqrt();
            let deg = b.degrees.clamp(0, 2) as usize;
            let phase = im.atan2(re) * ANGLE_FACTORS[deg];
            let abs_string = self.format_real(abs, format, false);
            let phase_string = format!("{}{}", self.format_real(phase, format, false), self.angle_units()[deg]);
            return format!("{}\u{2220}{}", abs_string, phase_string);
        }
        let s_re = self.format_real(re, format, false);
        let s_im = self.format_real(im.abs(), format, false);
        if im < 0.0 {
            format!("{} \u{2013} {}i", s_re, s_im)
        } else {
            format!("{} + {}i", s_re, s_im)
        }
    }

    /// `FormatNumberHelper`
    fn format_number_helper(&self, d: f64, format: Option<&str>) -> String {
        format_number(d, format, self.base())
    }
}

/// `OutputWriter.FormatNumberHelper` with explicit writer settings.
pub fn format_number(d: f64, format: Option<&str>, base: &WriterBase) -> String {
    if d.is_nan() {
        return " Undefined ".to_string();
    }
    if d == f64::INFINITY {
        return "+\u{221E}".to_string();
    }
    if d == f64::NEG_INFINITY {
        return "-\u{221E}".to_string();
    }
    let format = format.map(|s| s.to_string()).or_else(|| base.format_string.clone());
    if let Some(format) = format.as_deref() {
        if !format.is_empty() {
            let s = if let Some(rest) = format.strip_prefix('D') {
                if d == d.trunc() && d < i64::MAX as f64 {
                    let p = if rest.is_empty() {
                        None
                    } else {
                        rest.parse::<usize>().ok()
                    };
                    netfmt::format_long_d(d as i64, p)
                } else {
                    // InvalidFormatString is thrown in C#; FormatReal callers would surface it.
                    return exceptions::InvalidFormatString(format).message;
                }
            } else {
                netfmt::double_to_string_format(d, format)
            };
            return if s == "-0" { "0".to_string() } else { s };
        }
    }
    let decimals = base.decimals;
    if d.abs() < 1e16 {
        let a = d.abs();
        let mut i = get_digits(a);
        if i >= -2 * decimals - 1 {
            if i == 0 {
                i = if a < 1.0 { decimals + 1 } else { decimals };
            } else if i > 0 {
                i = decimals - i;
                if i < 0 {
                    i = 0;
                }
            } else {
                i = decimals - i + 1;
            }
            if i <= 16 {
                if let Some(s) = netfmt::g17_decimal_round(d, i) {
                    return if s == "-0" { "0".to_string() } else { s };
                }
            }
        }
    }
    let sharps = &"################"[..decimals.clamp(0, 16) as usize];
    let format = format!("#.{}E+0", sharps);
    netfmt::format_custom(d, &format)
}

fn get_digits(d: f64) -> i32 {
    if d >= 1.0 {
        return if d <= 1e4 {
            0
        } else if d <= 1e5 {
            1
        } else if d <= 1e6 {
            2
        } else if d <= 1e7 {
            3
        } else if d <= 1e8 {
            4
        } else if d <= 1e9 {
            5
        } else if d <= 1e10 {
            6
        } else if d <= 1e11 {
            7
        } else if d <= 1e12 {
            8
        } else if d <= 1e13 {
            9
        } else if d <= 1e14 {
            10
        } else if d <= 1e15 {
            11
        } else {
            0
        };
    }
    if d == 0.0 {
        0
    } else if d < 1e-14 {
        -14
    } else if d < 1e-13 {
        -13
    } else if d < 1e-12 {
        -12
    } else if d < 1e-11 {
        -11
    } else if d < 1e-10 {
        -10
    } else if d < 1e-9 {
        -9
    } else if d < 1e-8 {
        -8
    } else if d < 1e-7 {
        -7
    } else if d < 1e-6 {
        -6
    } else if d < 1e-5 {
        -5
    } else if d < 1e-4 {
        -4
    } else if d < 1e-3 {
        -3
    } else if d < 1e-2 {
        -2
    } else if d < 1e-1 {
        -1
    } else {
        0
    }
}

/// `OutputWriter.FormatOperatorHelper`
pub fn format_operator_helper(c: char) -> String {
    match c {
        NEG_CHAR => "-".to_string(),
        '-' => " \u{2212} ".to_string(),
        '*' => "\u{00B7}".to_string(),
        '/' => "/".to_string(),
        '\u{00F7}' => "/".to_string(),
        '<' => " < ".to_string(),
        '>' => " > ".to_string(),
        '+' => " + ".to_string(),
        '=' => " = ".to_string(),
        '\u{2190}' => " \u{2190} ".to_string(),
        '\u{2261}' => " \u{2261} ".to_string(),
        '\u{2260}' => " \u{2260} ".to_string(),
        '\u{2264}' => " \u{2264} ".to_string(),
        '\u{2265}' => " \u{2265} ".to_string(),
        '\u{29BC}' => " mod ".to_string(),
        '\u{2227}' => " and ".to_string(),
        '\u{2228}' => " or ".to_string(),
        '\u{2295}' => " xor ".to_string(),
        ';' => "; ".to_string(),
        ',' => ",".to_string(),
        '|' => " | ".to_string(),
        _ => c.to_string(),
    }
}

/// `GetMaxVisibleMatrixValue(Matrix matrix, out int row, out int col)`
pub fn get_max_visible_matrix_value(matrix: &Matrix, max_count: i32) -> (f64, usize, usize) {
    let mut max_abs = 0.0;
    let max_count = max_count.max(0) as usize;
    let m = max_count.min(matrix.row_count());
    let n = max_count.min(matrix.col_count());
    let last_row = matrix.row_count() as i64 - 1;
    let last_col = matrix.col_count() as i64 - 1;
    let (mut ii, mut jj) = (0usize, 0usize);
    let mut check = |i: usize, j: usize| {
        let d = matrix.get(i, j).d.abs();
        if d > max_abs {
            max_abs = d;
            ii = i;
            jj = j;
        }
    };
    let row = |i: usize, check: &mut dyn FnMut(usize, usize)| {
        for j in 0..n {
            check(i, j);
        }
        if (max_count as i64) < last_col {
            check(i, last_col as usize);
        }
    };
    for i in 0..m {
        row(i, &mut check);
    }
    if (max_count as i64) < last_row {
        row(last_row as usize, &mut check);
    }
    (max_abs, ii, jj)
}

/// `GetMaxVisibleVectorValue(Vector vector)`
pub fn get_max_visible_vector_value(vector: &Vector, max_count: i32) -> f64 {
    let mut max_abs = 0.0;
    let len = (max_count.max(0) as usize).min(vector.size());
    for i in 0..len {
        let d = vector.get(i).d.abs();
        if d > max_abs {
            max_abs = d;
        }
    }
    let last = vector.len() as i64 - 1;
    if (max_count as i64) < last {
        let d = vector.get(last as usize).d.abs();
        if d > max_abs {
            max_abs = d;
        }
    }
    max_abs
}

/// The unit all elements of a non-HP vector or matrix share, so the writers can show it once after the
/// closing bracket. `None` (keep the unit on each element) when the setting `unit_per_element` is on,
/// when any element is unitless, when two elements differ in dimension, or when a temperature unit
/// would need an offset to convert. Elements in other units of the same dimension convert to the first
/// element's unit (see [`scaled_d`]).
fn common_unit_of(mut it: impl Iterator<Item = RealValue>) -> Option<UnitRef> {
    use std::rc::Rc;
    let first = it.next()?.units?;
    for v in it {
        let u = v.units?;
        if Rc::ptr_eq(&u, &first) || u.text() == first.text() {
            continue;
        }
        if !u.is_consistent(&first) || first.is_temp() || u.is_temp() {
            return None;
        }
        // A format string on the unit (`unit:format`) is per unit; do not merge different ones.
        if u.format_string() != first.format_string() {
            return None;
        }
    }
    Some(first)
}

/// See [`common_unit_of`], for a matrix.
pub fn common_matrix_unit(base: &WriterBase, matrix: &Matrix) -> Option<UnitRef> {
    if base.unit_per_element {
        return None;
    }
    let (m, n) = (matrix.row_count(), matrix.col_count());
    common_unit_of((0..m).flat_map(|i| (0..n).map(move |j| matrix.get(i, j))))
}

/// See [`common_unit_of`], for a vector.
pub fn common_vector_unit(base: &WriterBase, vector: &Vector) -> Option<UnitRef> {
    if base.unit_per_element {
        return None;
    }
    common_unit_of((0..vector.len()).map(|i| vector.get(i)))
}

/// The element's number in the common unit (the number unchanged without one).
pub fn scaled_d(common: &Option<UnitRef>, value: &RealValue) -> f64 {
    use crate::base_types::unit::Unit;
    match (common, &value.units) {
        (Some(c), Some(u)) if !std::rc::Rc::ptr_eq(c, u) && u.text() != c.text() => {
            value.d * Unit::convert(&Some(c.clone()), &Some(u.clone()), '+').unwrap_or(1.0)
        }
        _ => value.d,
    }
}
