//! Port of Output/XmlWriter.cs: equations as Office Math (OMML) for the Word export.
//!
//! Upstream writes its fragments as indented C# verbatim strings and strips groups of four
//! spaces afterwards (`MathParser.ToXml`); the fragments here are written without the
//! indentation, which Word ignores between elements.
use super::*;
use std::sync::OnceLock;

pub struct XmlWriter {
    base: WriterBase,
}

pub const NORMAL_TEXT: &str = "<m:rPr><m:nor/></m:rPr>";
const RED: &str = "<w:rPr><w:color w:val=\"FF0000\" /></w:rPr>";

/// `XmlWriter.Run(content)`
pub fn run(content: &str) -> String {
    format!("<m:r><m:t>{}</m:t></m:r>", content)
}

/// `XmlWriter.Run(content, format)`
pub fn run_f(content: &str, format: &str) -> String {
    format!("<m:r>{}<m:t>{}</m:t></m:r>", format, content)
}

/// `XmlWriter.Brackets`
pub fn brackets(s: &str, left: char, right: char) -> String {
    format!(
        "<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr><m:e>{}</m:e></m:d>",
        left, right, s
    )
}

fn units_format() -> String {
    format!(
        "{}<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\" /><w:sz w:val=\"22\" /></w:rPr>",
        NORMAL_TEXT
    )
}

fn format_units_static(s: &str) -> String {
    run_f(s, &units_format())
}

const M_PR: &str =
    "<m:mPr><m:mcs><m:mc><m:mcPr><m:count m:val=\"2\"/><m:mcJc m:val=\"left\"/></m:mcPr></m:mc></m:mcs></m:mPr>";

fn if_row(sa: &str, sb: &str) -> String {
    format!(
        "<m:mr><m:e>{}{}{}</m:e><m:e>{}</m:e></m:mr>",
        run_f(" if ", NORMAL_TEXT),
        sa,
        run(":"),
        sb
    )
}

fn else_row(sa: &str) -> String {
    format!(
        "<m:mr><m:e>{}</m:e><m:e>{}</m:e></m:mr>",
        run_f(" else:", NORMAL_TEXT),
        sa
    )
}

fn format_vector_name(s: &str, is_bold: bool) -> String {
    const VEC: char = '\u{20D7}';
    let format = "<w:rPr><w:color w:val=\"0066DD\"/></w:rPr>";
    let format_bold = "<w:rPr><w:color w:val=\"0044AA\"/></w:rPr>";
    if let Some(rest) = s.strip_prefix(VEC) {
        return format!(
            "<m:acc><m:accPr><m:chr m:val=\"{}\"/></m:accPr><m:e>{}</m:e></m:acc>",
            VEC,
            run_f(rest, format)
        );
    }
    run_f(s, if is_bold { format_bold } else { format })
}

/// `XmlWriter.UnitProduct` as a static string.
pub fn unit_product_static() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(XmlWriter::unit_product).as_str()
}

/// `XmlWriter.UnitDivision` as a static string.
pub fn unit_division_static() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(XmlWriter::unit_division).as_str()
}

impl XmlWriter {
    pub fn new(settings: Option<&MathSettings>, phasor: bool) -> XmlWriter {
        XmlWriter {
            base: WriterBase::new(settings, phasor),
        }
    }

    /// `XmlWriter.UnitDivision`
    pub fn unit_division() -> String {
        run_f(
            "\u{2215}",
            &format!(
                "{}<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\" /><w:sz w:val=\"20\" /></w:rPr>",
                NORMAL_TEXT
            ),
        )
    }

    /// `XmlWriter.UnitProduct`
    pub fn unit_product() -> String {
        run_f("\u{00B7}", NORMAL_TEXT)
    }

    /// `XmlWriter.RunThinSpace`
    pub fn run_thin_space() -> String {
        run(&THIN_SPACE.to_string())
    }
}

impl OutputWriter for XmlWriter {
    fn base(&self) -> &WriterBase {
        &self.base
    }

    fn format_type(&self) -> OutputFormat {
        OutputFormat::Xml
    }

    fn angle_units(&self) -> [&'static str; 3] {
        static DEG: OnceLock<String> = OnceLock::new();
        static GRA: OnceLock<String> = OnceLock::new();
        [
            DEG.get_or_init(|| run("\u{00B0}")).as_str(),
            "",
            GRA.get_or_init(|| format_units_static("gra")).as_str(),
        ]
    }

    fn unit_string(&self, units: &UnitRef) -> String {
        units.xml()
    }

    fn format_input(&self, s: &str, units: &Option<UnitRef>, _line: i32, is_calculated: bool) -> String {
        const FORMAT1: &str = "<w:rPr><w:color w:val=\"FF0000\" /><w:shd w:fill=\"FFFCCC\" /></w:rPr>";
        const FORMAT2: &str = "<w:rPr><w:u /><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"FFFCCC\" /></w:rPr>";
        const FORMAT3: &str = "<w:rPr><w:shd w:val=\"clear\" w:color=\"auto\"  w:fill=\"FFFCCC\" /></w:rPr>";
        let output = if s == "?" {
            run_f(s, FORMAT1)
        } else if is_calculated {
            run_f(s, FORMAT2)
        } else {
            run_f(s, FORMAT3)
        };
        match units {
            Some(u) => output + &u.xml(),
            None => output,
        }
    }

    fn append_subscript(&self, sa: &str, sb: &str) -> String {
        let s = if sb.starts_with('<') {
            run(".") + sb
        } else {
            run(&format!(".{}", sb))
        };
        let i = sa.rfind("</m:sub>").unwrap_or(sa.len());
        format!("{}{}</m:sub></m:sSub>", &sa[..i], s)
    }

    fn format_subscript(&self, sa: &str, sb: &str) -> String {
        let s = if sb.starts_with('<') { sb.to_string() } else { run(sb) };
        format!("<m:sSub><m:e>{}</m:e><m:sub>{}</m:sub></m:sSub>", sa, s)
    }

    fn format_variable(&self, name: &str, _value: &str, is_bold: bool) -> String {
        match name.find('_') {
            Some(i) if i > 0 && i + 1 < name.len() => {
                self.format_subscript(&format_vector_name(&name[..i], is_bold), &name[i + 1..])
            }
            _ => format_vector_name(name, is_bold),
        }
    }

    fn format_units(&self, s: &str) -> String {
        format_units_static(s)
    }

    fn format_function(&self, s: &str) -> String {
        let format = format!(
            "{}<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\" /><w:b w:val=\"true\" /></w:rPr>",
            NORMAL_TEXT
        );
        match s.find('_') {
            Some(i) if i > 0 && i + 1 < s.len() => self.format_subscript(&run_f(&s[..i], &format), &s[i + 1..]),
            _ => run_f(s, &format),
        }
    }

    fn format_root(&self, s: &str, _level: i32, n: &str) -> String {
        match n {
            "2" => format!(
                "<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/><m:e>{}</m:e></m:rad>",
                s
            ),
            "3" => format!("<m:rad><m:deg>{}</m:deg><m:e>{}</m:e></m:rad>", run(n), s),
            _ => format!("<m:rad><m:deg>{}</m:deg><m:e>{}</m:e></m:rad>", n, s),
        }
    }

    fn format_operator(&self, c: char) -> String {
        match c {
            '<' => run("&lt;"),
            '>' => run("&gt;"),
            '\u{2264}' => run("\u{2264}"),
            '\u{2265}' => run("\u{2265}"),
            '*' => run("\u{00B7}"),
            '/' | '\u{00F7}' => run("/"),
            '\u{29BC}' => run_f(" mod ", NORMAL_TEXT),
            '\u{2227}' => run_f(" and ", NORMAL_TEXT),
            '\u{2228}' => run_f(" or ", NORMAL_TEXT),
            '\u{2295}' => run_f(" xor ", NORMAL_TEXT),
            '|' => run(" | "),
            NEG_CHAR => run_f("-", NORMAL_TEXT),
            _ => run(&c.to_string()),
        }
    }

    fn format_power(&self, sa: &str, sb: &str, _level: i32, order: i32) -> String {
        const SE: &str = "</m:sup></m:sSup>";
        if order == POWER_ORDER && sa.ends_with(SE) {
            return format!(
                "{}{}{}{}",
                &sa[..sa.len() - SE.len()],
                self.format_operator('*'),
                sb,
                SE
            );
        }
        format!("<m:sSup><m:e>{}</m:e><m:sup>{}</m:sup></m:sSup>", sa, sb)
    }

    fn format_division(&self, sa: &str, sb: &str, _level: i32) -> String {
        format!("<m:f><m:num>{}</m:num><m:den>{}</m:den></m:f>", sa, sb)
    }

    fn format_nary(&self, symbol: &str, sub: &str, sup: &str, expr: &str) -> String {
        format!(
            "<m:nary><m:naryPr><m:chr m:val=\"{}\"/><m:limLoc m:val=\"undOvr\"/></m:naryPr><m:sub>{}</m:sub><m:sup>{}</m:sup><m:e>{}</m:e></m:nary>",
            symbol, sub, sup, expr
        )
    }

    fn format_value(&self, value: &Value) -> String {
        let u = value.units();
        let fs = u.as_ref().and_then(|x| x.format_string());
        let mut s = self.format_complex(value.re(), value.im(), fs.as_deref());
        let Some(u) = u else { return s };
        if !(value.is_real() || self.base.phasor && s.contains('\u{2220}')) {
            s = self.add_brackets(&s, 0, '(', ')');
        }
        format!("{}{}{}", s, XmlWriter::run_thin_space(), u.xml())
    }

    fn add_brackets(&self, s: &str, _level: i32, left: char, right: char) -> String {
        brackets(s, left, right)
    }

    fn format_abs(&self, s: &str, level: i32) -> String {
        self.add_brackets(s, level, '|', '|')
    }

    fn format_real(&self, d: f64, format: Option<&str>, zero_small: bool) -> String {
        let s = self.format_number_helper(d, format);
        if d.is_nan() {
            return run_f(&s, &format!("{}{}", NORMAL_TEXT, RED));
        }
        if d.is_infinite() {
            return run_f(&s, RED);
        }
        let value_run = |s: &str| match s.strip_prefix('-') {
            Some(rest) => run_f("-", NORMAL_TEXT) + &run(rest),
            None => run(s),
        };
        let i = match s.rfind('E') {
            Some(i) if i > 0 => i,
            _ => {
                let Some(i) = s.rfind('e') else { return value_run(&s) };
                let mut i1 = i + 1;
                if matches!(s.as_bytes().get(i1), Some(b'+' | b'-')) {
                    i1 += 1;
                }
                return format!(
                    "{}{}{}",
                    value_run(&s[..i]),
                    run_f(&s[i..i1], NORMAL_TEXT),
                    run(&s[i1..])
                );
            }
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
                return run("0");
            }
            let has_e = fmt_owned.as_deref().is_some_and(|f| f.contains('E'));
            if bytes.get(i1) == Some(&b'0') && !has_e {
                i1 += 1;
            }
        }
        let ms = &s[..i];
        let x_format = format!(
            "{}<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\" /><w:b/><w:sz w:val=\"20\" /><w:vertAlign w:val=\"superscript\" /></w:rPr>",
            NORMAL_TEXT
        );
        let ms = if ms == "1" {
            String::new()
        } else {
            value_run(ms) + &run_f("\u{00D7}", &x_format)
        };
        let rest = &s[i1.min(s.len())..];
        let es = if sign == '+'
            && fmt_owned
                .as_deref()
                .is_some_and(|f| f.contains("E+") || f.starts_with('E'))
        {
            run(&format!("+{}", rest))
        } else if sign == '-' {
            run_f("-", NORMAL_TEXT) + &run(rest)
        } else {
            run(rest)
        };
        format!("{}<m:sSup><m:e>{}</m:e><m:sup>{}</m:sup></m:sSup>", ms, run("10"), es)
    }

    fn format_complex(&self, re: f64, im: f64, format: Option<&str>) -> String {
        if re.is_nan() && im.is_nan() {
            return run_f("Undefined", &format!("{}{}", NORMAL_TEXT, RED));
        }
        if re.is_infinite() && im.is_infinite() {
            return run_f("\u{221E}", RED);
        }
        let t = Complex::get_type(re, im);
        if t == Types::Real {
            return self.format_real(re, format, false);
        }
        let b = &self.base;
        if b.phasor {
            let abs = (re * re + im * im).sqrt();
            let deg = b.degrees.clamp(0, 2) as usize;
            let phase = im.atan2(re) * ANGLE_FACTORS[deg];
            return format!(
                "{}{}{}{}",
                self.format_real(abs, format, false),
                run("\u{2220}"),
                self.format_real(phase, format, false),
                self.angle_units()[deg]
            );
        }
        let s_im = self.format_real(im.abs(), format, false) + &run("i");
        if t == Types::Imaginary {
            return s_im;
        }
        let s_re = self.format_real(re, format, false);
        format!("{}{}{}", s_re, if im < 0.0 { run("-") } else { run("+") }, s_im)
    }

    fn format_switch(&self, sa: &[String], level: i32) -> String {
        let len = sa.len();
        let run_switch = run_f("switch", NORMAL_TEXT);
        if len == 1 {
            return run_switch + &self.add_brackets(&sa[0], level, '(', ')');
        }
        if len == 2 {
            return run_switch + &self.add_brackets(&format!("{}{}{}", sa[0], run("; "), sa[1]), level, '(', ')');
        }
        let mut sb = String::from("<m:d><m:dPr><m:begChr m:val=\"{\"/><m:endChr m:val=\" \"/></m:dPr><m:e><m:m>");
        sb.push_str(M_PR);
        sb.push_str(&if_row(&sa[0], &sa[1]));
        let mut i = 2;
        while i < len {
            if len - i == 1 {
                sb.push_str(&else_row(&sa[i]));
            } else {
                sb.push_str(&if_row(&sa[i], &sa[i + 1]));
            }
            i += 2;
        }
        sb.push_str("</m:m></m:e></m:d>");
        sb
    }

    fn close_curly_brackets(&self, sa: &str, _level: i32) -> String {
        const RIGHT: &str = "<m:endChr m:val=\" \"/>";
        match sa.find(RIGHT) {
            None => sa.to_string(),
            Some(i) => format!("{}<m:endChr m:val=\"}}\"/>{}", &sa[..i], &sa[i + RIGHT.len()..]),
        }
    }

    fn format_if(&self, sc: &str, sa: &str, sb: &str, _level: i32) -> String {
        format!(
            "<m:d><m:dPr><m:begChr m:val=\"{{\"/><m:endChr m:val=\" \"/></m:dPr><m:e><m:m>{}{}{}</m:m></m:e></m:d>",
            M_PR,
            if_row(sc, sa),
            else_row(sb)
        )
    }

    fn format_matrix(&self, matrix: &Matrix) -> String {
        const SMALLER: &str = "<w:rPr><w:sz w:val=\"21\" /></w:rPr>";
        let td = |s: &str| format!("<m:e><m:r>{}<m:t>{}</m:t></m:r></m:e>", SMALLER, s);
        let append_element = |sb: &mut String, s: &str| {
            // The first run without properties gets the smaller font.
            let find_from = |from: usize| s[from..].find("<m:r>").map(|k| k + from + 5);
            let mut k1 = find_from(0);
            while let Some(a) = k1 {
                let k2 = s[a..].find("</m:r>").map(|k| k + a);
                match k2 {
                    Some(b) if s[a..b].contains(":rPr>") => k1 = find_from(b),
                    _ => break,
                }
            }
            sb.push_str("<m:e>");
            match k1 {
                None => sb.push_str(s),
                Some(5) => {
                    sb.push_str("<m:r>");
                    sb.push_str(SMALLER);
                    sb.push_str(&s[5..]);
                }
                Some(k) => {
                    sb.push_str(&s[..k]);
                    sb.push_str(SMALLER);
                    sb.push_str(&s[k..]);
                }
            }
            sb.push_str("</m:e>");
        };
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        let nr = matrix.row_count();
        let nc = matrix.col_count();
        let zero_threshold = get_max_visible_matrix_value(matrix, max_count).0 * 1e-14;
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
        let count = if nc > mc { mc + 2 } else { nc };
        let mut sb = format!(
            "<m:d><m:dPr><m:begChr m:val=\"[\"/><m:endChr m:val=\"]\"/></m:dPr><m:e><m:m><m:mPr><m:rSpRule m:val=\"3\"/><m:cGpRule m:val=\"3\"/><m:rSp m:val=\"300\"/><m:cGp m:val=\"120\"/><m:mcs><m:mc><m:mcPr><m:count m:val=\"{}\"/><m:mcJc m:val=\"center\"/></m:mcPr></m:mc></m:mcs></m:mPr>",
            count
        );
        let mut i = 0usize;
        while i < nr {
            sb.push_str("<m:mr>");
            if i == mc {
                for _ in 0..mc.min(nc) {
                    sb.push_str(&td("\u{22EE}"));
                }
                if nc > mc {
                    sb.push_str(&td("\u{22F1}"));
                    sb.push_str(&td("\u{22EE}"));
                }
                sb.push_str("</m:mr><m:mr>");
                i = nr - 1;
            }
            let mut j = 0usize;
            while j < nc {
                if j == mc {
                    sb.push_str(&td("\u{22EF}"));
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
                append_element(&mut sb, &s);
                j += 1;
            }
            sb.push_str("</m:mr>");
            i += 1;
        }
        sb.push_str("</m:m></m:e></m:d>");
        if let Some(u) = &units {
            sb.push_str(&u.xml());
        }
        sb
    }

    fn format_vector(&self, vector: &Vector) -> String {
        let div = run(VECTOR_SPACING);
        let max_count = self.base.max_count;
        let mc = max_count.max(0) as usize;
        const TOL: f64 = 1e-14;
        let mut zero_threshold = get_max_visible_vector_value(vector, max_count) * TOL;
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
        let element = |index: usize| {
            if !hp {
                self.format_matrix_value(&vector.get(index), zero_threshold)
            } else {
                let d = super::scaled_d(&common, &vector.get(index));
                self.format_real(
                    d,
                    fs.as_deref(),
                    self.base.zero_small_elements && d.abs() < zero_threshold,
                )
            }
        };
        let len = vector.len();
        let mut sb = String::new();
        for i in 0..len {
            if i > 0 {
                sb.push_str(&div);
            }
            if i == mc {
                sb.push_str(&run("..."));
                sb.push_str(&div);
                break;
            }
            sb.push_str(&element(i));
        }
        if len > 0 && mc < len - 1 {
            sb.push_str(&element(len - 1));
        }
        let mut s = self.add_brackets(&sb, 0, '[', ']');
        if let Some(u) = &units {
            s.push_str(&u.xml());
        }
        s
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
            Some(u) => s + &u.xml(),
        }
    }

    fn format_block(&self, sa: &[String]) -> String {
        let mut sb = String::from("<m:d><m:dPr><m:begChr m:val=\"|\"/><m:endChr m:val=\" \"/></m:dPr><m:e><m:m>");
        sb.push_str(M_PR);
        for s in sa {
            if !s.is_empty() {
                sb.push_str("<m:mr><m:e>");
                sb.push_str(s);
                sb.push_str("</m:e></m:mr>");
            }
        }
        sb.push_str("</m:m></m:e></m:d>");
        sb
    }
}
