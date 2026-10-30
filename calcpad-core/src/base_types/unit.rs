//! Port of BaseTypes/Unit.cs.
//!
//! C# `Unit` is a mutable reference type and a lot of behaviour depends on reference
//! identity (`ReferenceEquals(u1, u2)`, shared `_powers` arrays) and on lazily cached
//! text. The port keeps those semantics: units are `Rc<Unit>`, the powers array is an
//! `Rc<[f32]>` whose pointer identity stands for the C# array identity, and text, factors
//! and the temperature character use interior mutability.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::base_types::value::Value;
use crate::calculator::NEG_CHAR;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::output::{
    html_writer::HtmlWriter, text_writer::TextWriter, xml_writer::XmlWriter, OutputFormat, OutputWriter,
};

pub type UnitRef = Rc<Unit>;
pub type OptUnit = Option<UnitRef>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Mechanical,
    Electrical,
    Other,
}

/// The most dimensions a unit has (the table's empty key has nine).
const MAX_DIMS: usize = 9;

/// Source of `Unit::pid`.
static NEXT_POWERS_ID: AtomicU64 = AtomicU64::new(1);

/// A unit is one block of memory: the powers and the factors are stored inline, so that making
/// the result of a unit calculation costs a single allocation. `pid` stands for the identity of
/// the C# `_powers` array: units made by `copy` and `alias` carry their source's `pid`.
pub struct Unit {
    text: RefCell<Option<String>>,
    temp_char: Cell<char>,
    n: u8,
    pid: u64,
    pw: [f32; MAX_DIMS],
    fc: [Cell<f64>; MAX_DIMS],
}

/// The text of the target unit `base` (`expr | base`): show the result in SI base units. A control character
/// keeps it from clashing with any name.
pub const BASE_TARGET: &str = "\u{1}base";

const COMPOSITE_UNIT_CHARS: [char; 3] = [NEG_CHAR, '\u{2215}', '^'];
const NAMES: [&str; 9] = ["g", "m", "s", "A", "\u{00B0}C", "mol", "cd", "rad", ""];
const UNIT_NAMES: [&str; 12] = [
    "therm", "cwt", "ton", "fl_oz", "gi", "pt", "qt", "gal", "bbl", "pk", "bu", "tonf",
];

fn is_ambiguous(text: &str) -> bool {
    text == "ton_f" || UNIT_NAMES.contains(&text)
}

fn resolve_key(text: &str, is_us: bool) -> String {
    if !is_ambiguous(text) {
        return text.to_string();
    }
    let base = if text == "ton_f" { "tonf" } else { text };
    format!("{}{}", base, if is_us { "_US" } else { "_UK" })
}

/// The SI prefixes: symbol, power of ten. `u` is the ASCII spelling of micro; `\u{00B5}` (the micro sign)
/// and `\u{03BC}` (Greek mu) are both accepted and shown as Greek mu.
const SI_PREFIXES: [(&str, i32); 22] = [
    ("da", 1),
    ("y", -24),
    ("z", -21),
    ("a", -18),
    ("f", -15),
    ("p", -12),
    ("n", -9),
    ("\u{03BC}", -6),
    ("\u{00B5}", -6),
    ("u", -6),
    ("m", -3),
    ("c", -2),
    ("d", -1),
    ("h", 2),
    ("k", 3),
    ("M", 6),
    ("G", 9),
    ("T", 12),
    ("P", 15),
    ("E", 18),
    ("Z", 21),
    ("Y", 24),
];

/// The units an SI prefix may be put in front of. Anything else (`min`, `h`, `d`, `ft`, `in`, `lb`,
/// `psi`, `\u{00B0}C`, `K`, ...) is not prefixable. `kg` is not here: prefixes go on `g`.
const PREFIXABLE: [&str; 36] = [
    "g", "m", "s", "A", "mol", "cd", "rad", "L", "N", "Nm", "Pa", "bar", "J", "W", "Wh", "VA", "VAR", "Hz",
    "V", "C", "F", "\u{03A9}", "S", "\u{2127}", "Wb", "T", "H", "Bq", "Gy", "Sv", "lm", "lx", "kat", "eV",
    "Ah", "Da",
];

thread_local! {
    static PREFIXED: RefCell<HashMap<String, Option<UnitRef>>> = RefCell::new(HashMap::new());
}

/// A prefix and a prefixable unit (`mN`, `\u{00B5}m`, `YPa`). The table is tried first, so a name it
/// has always means what it meant: `min` is the minute, `Pa` the pascal, `cd` the candela, `ft`
/// the foot, `h` the hour, `d` the day, `T` the tesla, `da` is not a unit.
fn prefixed_unit(text: &str) -> Option<UnitRef> {
    if text.len() < 2 || text.len() > 12 {
        return None;
    }
    if let Some(hit) = PREFIXED.with(|c| c.borrow().get(text).cloned()) {
        return hit;
    }
    let found = (|| {
        for (p, n) in SI_PREFIXES {
            let Some(rest) = text.strip_prefix(p) else { continue };
            if !PREFIXABLE.contains(&rest) {
                continue;
            }
            let base = TABLE.with(|t| t.units.get(rest).cloned())?;
            // The prefix is part of the symbol (`mN`), so the unit's own text carries it.
            let u = base.scale(&format!("{}{}", get_prefix(n), base.text()), get_scale(n));
            return Some(r(u));
        }
        None
    })();
    PREFIXED.with(|c| c.borrow_mut().insert(text.to_string(), found.clone()));
    found
}

/// `Math.Pow` as used by the unit code.
#[inline]
fn my_pow(x: f64, y: f32) -> f64 {
    match y {
        -3.0 => 1.0 / (x * x * x),
        -2.0 => 1.0 / (x * x),
        -1.0 => 1.0 / x,
        0.0 => 1.0,
        1.0 => x,
        2.0 => x * x,
        3.0 => x * x * x,
        _ => x.powf(y as f64),
    }
}

impl Unit {
    // ----- construction -------------------------------------------------------------

    /// `new Unit(int n)`
    pub fn new_n(n: usize) -> Unit {
        Unit::from_parts(&vec![0f32; n], &vec![0f64; n], 'C')
    }

    /// A unit of `n` dimensions with zero powers and factors and a new powers identity, for
    /// the caller to fill in.
    #[inline]
    fn blank(n: usize, temp_char: char) -> Unit {
        debug_assert!(n <= MAX_DIMS);
        Unit {
            text: RefCell::new(None),
            temp_char: Cell::new(temp_char),
            n: n as u8,
            pid: NEXT_POWERS_ID.fetch_add(1, Ordering::Relaxed),
            pw: [0f32; MAX_DIMS],
            fc: [const { Cell::new(0.0) }; MAX_DIMS],
        }
    }

    /// `blank` made directly in its `Rc` block. A unit is 160 bytes; building it in a local and
    /// moving it into the block costs a copy, which in the browser build is a slow bulk-memory
    /// call, and unit results are made by the million.
    #[inline]
    fn blank_rc(n: usize, temp_char: char) -> Rc<Unit> {
        debug_assert!(n <= MAX_DIMS);
        let mut rc = Rc::<Unit>::new_uninit();
        let p = Rc::get_mut(&mut rc).expect("a new Rc is unique").as_mut_ptr();
        // SAFETY: every field of the unit is written before `assume_init`; `Cell<f64>` has
        // the layout of `f64`.
        unsafe {
            std::ptr::addr_of_mut!((*p).text).write(RefCell::new(None));
            std::ptr::addr_of_mut!((*p).temp_char).write(Cell::new(temp_char));
            std::ptr::addr_of_mut!((*p).n).write(n as u8);
            std::ptr::addr_of_mut!((*p).pid).write(NEXT_POWERS_ID.fetch_add(1, Ordering::Relaxed));
            let pw = std::ptr::addr_of_mut!((*p).pw) as *mut f32;
            let fc = std::ptr::addr_of_mut!((*p).fc) as *mut f64;
            // Volatile stores: plain ones are merged into a bulk fill, a slow call in the
            // browser build for 36 and 72 bytes.
            for i in 0..MAX_DIMS {
                pw.add(i).write_volatile(0.0);
                fc.add(i).write_volatile(0.0);
            }
            rc.assume_init()
        }
    }

    fn from_parts(powers: &[f32], factors: &[f64], temp_char: char) -> Unit {
        let n = powers.len();
        debug_assert!(factors.len() == n);
        let mut u = Unit::blank(n, temp_char);
        for i in 0..n {
            u.pw[i] = powers[i];
            u.fc[i].set(factors[i]);
        }
        u
    }

    /// `new Unit(text, mass, length, time, current, temp, substance, luminosity, angle)`
    #[allow(clippy::too_many_arguments)]
    pub fn new_dims(
        text: &str,
        mass: f32,
        length: f32,
        time: f32,
        current: f32,
        temp: f32,
        substance: f32,
        luminosity: f32,
        angle: f32,
    ) -> Unit {
        let n = if angle != 0.0 {
            8
        } else if luminosity != 0.0 {
            7
        } else if substance != 0.0 {
            6
        } else if temp != 0.0 {
            5
        } else if current != 0.0 {
            4
        } else if time != 0.0 {
            3
        } else if length != 0.0 {
            2
        } else {
            1
        };
        let mut factors = vec![1f64; n];
        factors[0] = 1000.0;
        let all = [mass, length, time, current, temp, substance, luminosity, angle];
        let u = Unit::from_parts(&all[..n], &factors, 'C');
        *u.text.borrow_mut() = Some(text.to_string());
        u
    }

    /// `new Unit(Unit u)`: shares the powers array, copies the factors.
    pub fn copy(u: &Unit) -> Unit {
        let c = Unit {
            text: RefCell::new(u.text.borrow().clone()),
            temp_char: Cell::new(u.temp_char.get()),
            n: u.n,
            pid: u.pid,
            pw: u.pw,
            fc: [const { Cell::new(0.0) }; MAX_DIMS],
        };
        for i in 0..u.n as usize {
            c.fc[i].set(u.fc[i].get());
        }
        c
    }

    /// `new Unit(Unit u, string alias)`: shares powers and factors. The port copies the factors:
    /// nothing changes the factors of a table unit after its aliases are made.
    pub fn alias(u: &Unit, alias: &str) -> Unit {
        let c = Unit {
            text: RefCell::new(Some(alias.to_string())),
            temp_char: Cell::new(u.temp_char.get()),
            n: u.n,
            pid: u.pid,
            pw: u.pw,
            fc: [const { Cell::new(0.0) }; MAX_DIMS],
        };
        for i in 0..u.n as usize {
            c.fc[i].set(u.fc[i].get());
        }
        c
    }

    /// `Unit.GetFormattingUnit(s)`
    pub fn get_formatting_unit(s: &str) -> UnitRef {
        let u = Unit::new_n(0);
        *u.text.borrow_mut() = Some(s.to_string());
        Rc::new(u)
    }

    // ----- properties ---------------------------------------------------------------

    #[inline]
    pub fn len(&self) -> usize {
        self.n as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    #[inline]
    pub fn powers(&self) -> &[f32] {
        &self.pw[..self.n as usize]
    }

    pub fn factor(&self, i: usize) -> f64 {
        assert!(i < self.n as usize);
        self.fc[i].get()
    }

    pub fn raw_text(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    pub fn set_text(&self, s: &str) {
        *self.text.borrow_mut() = Some(s.to_string());
    }

    pub fn temp_char(&self) -> char {
        self.temp_char.get()
    }

    pub fn set_temp_char(&self, c: char) {
        self.temp_char.set(c);
    }

    pub fn get_field(&self) -> Field {
        let p = self.powers();
        match self.len() {
            3 => {
                if p[0] == 1.0 {
                    if p[2] == -2.0 {
                        let t = self.text.borrow();
                        match t.as_ref() {
                            None => return Field::Mechanical,
                            Some(t) => {
                                if let Some(i) = t.find('s') {
                                    if t[i..].contains('2') {
                                        return Field::Mechanical;
                                    }
                                }
                            }
                        }
                    } else if p[2] == -3.0 && p[1] == 2.0 {
                        return Field::Electrical;
                    }
                }
            }
            4 => {
                if p[3] != 0.0 {
                    return Field::Electrical;
                }
            }
            _ => {}
        }
        Field::Other
    }

    fn has_temp(&self) -> bool {
        self.len() >= 5 && self.pw[4] != 0.0
    }

    pub fn is_temp(&self) -> bool {
        let p = self.powers();
        self.len() == 5 && p[4] == 1.0 && p[0] == 0.0 && p[1] == 0.0 && p[2] == 0.0 && p[3] == 0.0
    }

    pub fn is_angle(&self) -> bool {
        let p = self.powers();
        self.len() == 8 && p[7] == 1.0 && p[..7].iter().all(|&x| x == 0.0)
    }

    pub fn is_dimensionless(&self) -> bool {
        self.n == 0
    }

    /// The coherent SI base units of this unit as text a unit parser reads (`kg*m^2/s^2`), for the target `base`.
    /// None when the unit has a temperature or an angle in it, or no dimension at all: those are left as they are.
    pub fn si_base_text(&self) -> Option<String> {
        const SYMBOLS: [&str; 7] = ["kg", "m", "s", "A", "", "mol", "cd"];
        let n = (self.n as usize).min(8);
        // Temperature (an offset scale) and angle are not converted.
        if (n > 4 && self.pw[4] != 0.0) || (n > 7 && self.pw[7] != 0.0) {
            return None;
        }
        let power = |name: &str, p: f32| -> String {
            if p == 1.0 { name.to_string() } else if p.fract() == 0.0 { format!("{name}^{}", p as i32) } else { format!("{name}^{p}") }
        };
        let mut num = Vec::new();
        let mut den = Vec::new();
        for i in 0..n.min(7) {
            let p = self.pw[i];
            if p == 0.0 || SYMBOLS[i].is_empty() {
                continue;
            }
            if p > 0.0 { num.push(power(SYMBOLS[i], p)); } else { den.push((SYMBOLS[i], -p)); }
        }
        if num.is_empty() && den.is_empty() {
            return None;
        }
        if num.is_empty() {
            return Some(den.iter().map(|(name, p)| power(name, -p)).collect::<Vec<_>>().join("*"));
        }
        let mut text = num.join("*");
        match den.len() {
            0 => {}
            1 => text.push_str(&format!("/{}", power(den[0].0, den[0].1))),
            _ => {
                for (name, p) in &den {
                    text.push_str(&format!("*{}", power(name, -p)));
                }
            }
        }
        Some(text)
    }

    /// `Unit.Text`
    pub fn text(&self) -> String {
        let needs = match self.text.borrow().as_ref() {
            None => true,
            Some(s) => s.is_empty(),
        };
        if needs {
            let t = self.get_text(OutputFormat::Text);
            *self.text.borrow_mut() = Some(t.clone());
            return t;
        }
        let t = self.text.borrow();
        let t = t.as_ref().unwrap();
        match t.find(':') {
            None => t.clone(),
            Some(i) => t[..i].to_string(),
        }
    }

    /// `Unit.Html`
    pub fn html(&self) -> String {
        if self.text.borrow().is_none() {
            return self.get_text(OutputFormat::Html);
        }
        let t = self.text();
        if t == "\u{00B0}" {
            return "\u{00B0}".to_string();
        }
        HtmlWriter::new(None, false).format_units_text(&t).unwrap_or_default()
    }

    /// `Unit.Xml`: the units as Office Math runs.
    pub fn xml(&self) -> String {
        if self.text.borrow().is_none() {
            return self.get_text(OutputFormat::Xml);
        }
        XmlWriter::new(None, false)
            .format_units_text(&self.text())
            .unwrap_or_default()
    }

    pub fn format_string(&self) -> Option<String> {
        let t = self.text.borrow();
        let t = t.as_ref()?;
        let i = t.find(':')?;
        Some(t[i + 1..].to_string())
    }

    pub fn format_string_with_prefix(&self) -> Option<String> {
        let t = self.text.borrow();
        let t = t.as_ref()?;
        let i = t.find(':')?;
        Some(t[i..].to_string())
    }

    /// `Unit.Equals(Unit)`
    pub fn equals(a: &UnitRef, b: &UnitRef) -> bool {
        Rc::ptr_eq(a, b) || a.is_consistent(b) && a.factors_slice_eq(b)
    }

    pub fn get_text_of(u: &OptUnit) -> String {
        match u {
            None => "unitless".to_string(),
            Some(u) => u.text(),
        }
    }

    // ----- table --------------------------------------------------------------------

    pub fn get(text: &str, is_us: bool) -> CResult<UnitRef> {
        Unit::try_get(text, is_us).ok_or_else(|| {
            MathParserException::unexpected(format!("The given key '{}' was not present in the dictionary.", text))
        })
    }

    pub fn try_get(text: &str, is_us: bool) -> Option<UnitRef> {
        let key = resolve_key(text, is_us);
        TABLE
            .with(|t| t.units.get(&key).cloned())
            .or_else(|| prefixed_unit(&key))
    }

    /// `key|text|powers|factors` for a unit, the line of the regression snapshot of the units
    /// that existed before prefixes applied to every SI unit (tests/data/unit_table_before.txt).
    #[doc(hidden)]
    pub fn snapshot_line(key: &str, u: &Unit) -> String {
        let n = u.n as usize;
        format!(
            "{}|{}|{:?}|{:?}",
            key,
            u.text(),
            &u.pw[..n],
            (0..n).map(|i| u.fc[i].get()).collect::<Vec<_>>()
        )
    }

    /// The unit a name reads as when its first letters are a prefix and the rest a prefixable unit, whether
    /// or not the table has the name as well (for the tests that look for names the table shadows).
    #[doc(hidden)]
    pub fn prefix_reading(text: &str) -> Option<UnitRef> {
        prefixed_unit(text)
    }

    pub fn exists(unit: &str) -> bool {
        TABLE.with(|t| t.units.contains_key(unit)) || prefixed_unit(unit).is_some()
    }

    // ----- scaling ------------------------------------------------------------------

    /// `void Scale(double factor)` (in place)
    pub fn scale_in_place(&self, factor: f64) {
        for (i, &p) in self.powers().iter().enumerate() {
            if p != 0.0 {
                self.fc[i].set(self.fc[i].get() * factor.powf(1.0 / p as f64));
                return;
            }
        }
    }

    /// The factors of two units of the same length are equal.
    #[inline]
    fn factors_slice_eq(&self, other: &Unit) -> bool {
        (0..self.n as usize).all(|i| self.fc[i].get() == other.fc[i].get())
    }

    /// `Unit Scale(string s, double factor)`
    pub fn scale(&self, s: &str, factor: f64) -> Unit {
        let unit = Unit::copy(self);
        *unit.text.borrow_mut() = Some(s.to_string());
        if factor != 1.0 {
            unit.scale_in_place(factor);
        }
        unit
    }

    pub fn shift(&self, n: i32) -> Unit {
        let t = self.text.borrow().clone().unwrap_or_default();
        self.scale(&format!("{}{}", get_prefix(n), t), get_scale(n))
    }

    fn unit_name(&self, i: usize) -> &'static str {
        if i == 4 {
            match self.temp_char.get() {
                'K' => "K",
                'F' => "\u{00B0}F",
                'R' => "\u{00B0}R",
                _ => NAMES[i],
            }
        } else {
            NAMES[i]
        }
    }

    fn get_text(&self, format: OutputFormat) -> String {
        let writer: Box<dyn OutputWriter> = match format {
            OutputFormat::Html => Box::new(HtmlWriter::new(None, false)),
            OutputFormat::Xml => Box::new(XmlWriter::new(None, false)),
            _ => Box::new(TextWriter::new(None, false)),
        };
        let mut sb = String::new();
        let mut is_first = true;
        for (i, &p) in self.powers().iter().enumerate() {
            if p != 0.0 {
                let abs_pow = if is_first { p } else { p.abs() };
                let mut s = get_dim_text(writer.as_ref(), self.unit_name(i), self.fc[i].get(), abs_pow);
                if i == 4 && !sb.is_empty() {
                    s = temperature_to_delta(&s);
                }
                if is_first {
                    is_first = false;
                } else {
                    sb.push_str(get_product_or_division_operator(format, p > 0.0));
                }
                sb.push_str(&s);
            }
        }
        sb
    }

    // ----- arithmetic ---------------------------------------------------------------

    /// `Unit * double`
    pub fn mul_f(u: &Unit, d: f64) -> UnitRef {
        let unit = Unit::copy(u);
        unit.scale_in_place(d);
        Rc::new(unit)
    }

    /// `Unit / double`
    pub fn div_f(u: &Unit, d: f64) -> UnitRef {
        Unit::mul_f(u, 1.0 / d)
    }

    /// `double / Unit`
    pub fn f_div(d: f64, u: &Unit) -> OptUnit {
        u.pow(-1.0).map(|p| Unit::mul_f(&p, d))
    }

    fn multiply_or_divide(u1: &UnitRef, u2: &UnitRef, divide: bool) -> OptUnit {
        if Rc::ptr_eq(u1, u2) || u1.pid == u2.pid {
            return if divide { None } else { u1.pow(2.0) };
        }
        let mut n1 = u1.len();
        let mut n2 = u2.len();
        let mut n = n1.max(n2);
        let p1 = u1.powers();
        let p2 = u2.powers();
        if n1 == n2 {
            let mut k = n as isize;
            if divide {
                loop {
                    k -= 1;
                    if k < 0 {
                        return None;
                    }
                    if p1[k as usize] != p2[k as usize] {
                        break;
                    }
                }
            } else {
                loop {
                    k -= 1;
                    if k < 0 {
                        return None;
                    }
                    if p1[k as usize] != -p2[k as usize] {
                        break;
                    }
                }
            }
            n = (k + 1) as usize;
        }
        let temp_char = if u1.has_temp() {
            u1.temp_char.get()
        } else if u2.has_temp() {
            u2.temp_char.get()
        } else {
            'C'
        };
        if n1 > n {
            n1 = n;
        }
        if n2 > n {
            n2 = n;
        }
        let f1 = &u1.fc;
        let f2 = &u2.fc;
        let mut rc = Unit::blank_rc(n, temp_char);
        let u = Rc::get_mut(&mut rc).expect("a new Rc is unique");
        for i in 0..n1 {
            if i < n2 {
                u.pw[i] = if divide { p1[i] - p2[i] } else { p1[i] + p2[i] };
                u.fc[i].set(if p1[i] == 0.0 { f2[i].get() } else { f1[i].get() });
            } else {
                u.pw[i] = p1[i];
                u.fc[i].set(if p1[i] != 0.0 { f1[i].get() } else { 1.0 });
            }
        }
        if n2 > n1 {
            for i in n1..n2 {
                u.fc[i].set(f2[i].get());
                u.pw[i] = if divide { -p2[i] } else { p2[i] };
            }
        }
        Some(rc)
    }

    /// `operator *(Unit, Unit)`: MultiplyOrDivide without the conversion factor.
    pub fn multiply_raw(u1: &UnitRef, u2: &UnitRef) -> OptUnit {
        Unit::multiply_or_divide(u1, u2, false)
    }

    /// `operator /(Unit, Unit)`
    pub fn divide_raw(u1: &UnitRef, u2: &UnitRef) -> OptUnit {
        Unit::multiply_or_divide(u1, u2, true)
    }

    pub fn get_product_or_division_factor(u1: &UnitRef, u2: &UnitRef, divide: bool) -> f64 {
        if Rc::ptr_eq(u1, u2) {
            return 1.0;
        }
        let n = u1.len().min(u2.len());
        let mut factor = 1.0;
        let p1 = u1.powers();
        let p2 = u2.powers();
        let f1 = &u1.fc;
        let f2 = &u2.fc;
        if divide {
            for i in 0..n {
                if p1[i] != 0.0 && p2[i] != 0.0 {
                    factor *= my_pow(f1[i].get() / f2[i].get(), p2[i]);
                }
            }
        } else {
            for i in 0..n {
                if p1[i] != 0.0 && p2[i] != 0.0 {
                    factor *= my_pow(f2[i].get() / f1[i].get(), p2[i]);
                }
            }
        }
        factor
    }

    /// `Unit Pow(float x)`
    pub fn pow(&self, x: f32) -> OptUnit {
        if x == 0.0 {
            return None;
        }
        let n = self.len();
        let mut rc = Unit::blank_rc(n, self.temp_char.get());
        let u = Rc::get_mut(&mut rc).expect("a new Rc is unique");
        for i in 0..n {
            u.pw[i] = self.pw[i] * x;
            u.fc[i].set(self.fc[i].get());
        }
        Some(rc)
    }

    pub fn is_consistent_opt(u1: &OptUnit, u2: &OptUnit) -> bool {
        match (u1, u2) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b) || a.is_consistent(b),
            _ => false,
        }
    }

    pub fn is_consistent(&self, other: &Unit) -> bool {
        if self.pid == other.pid {
            return true;
        }
        self.powers() == other.powers()
    }

    pub fn is_multiple(&self, other: &Unit) -> bool {
        let n = self.len();
        if n != other.len() {
            return false;
        }
        let mut d1 = 0f32;
        for i in 0..n {
            let p1 = self.pw[i];
            let p2 = other.pw[i];
            if p1 != p2 {
                if p1 == 0.0 || p2 == 0.0 {
                    return false;
                }
                if d1 == 0.0 {
                    d1 = p2 - p1;
                } else {
                    let d2 = p2 - p1;
                    if d1 != d2 {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn convert_to(&self, u: &Unit) -> f64 {
        if self.is_dimensionless() {
            return self.get_dimensionless_factor() / u.get_dimensionless_factor();
        }
        let mut factor = 1.0;
        let uf = &u.fc;
        let f = &self.fc;
        for i in 0..self.len() {
            if self.pw[i] != 0.0 {
                factor *= my_pow(f[i].get() / uf[i].get(), self.pw[i]);
            }
        }
        factor
    }

    pub fn get_force_unit(u: &UnitRef) -> UnitRef {
        let i = u.pw[1] as i32 + 3;
        if !(0..=5).contains(&i) {
            return u.clone();
        }
        let si = u.fc[0].get() % 1.0 == 0.0;
        TABLE.with(|t| {
            if si {
                t.force_units[i as usize].clone()
            } else {
                t.force_units_us[i as usize].clone()
            }
        })
    }

    pub fn get_electrical_unit(u: &UnitRef) -> UnitRef {
        let starts_va = match u.text.borrow().as_ref() {
            Some(t) => t.starts_with("VA"),
            None => false,
        };
        if !starts_va {
            let found = TABLE.with(|t| {
                for (k, v) in &t.electrical {
                    if Unit::equals(k, u) {
                        return Some(v.clone());
                    }
                }
                None
            });
            if let Some(eu) = found {
                return eu;
            }
        }
        u.clone()
    }

    pub fn get_temp_units_delta(src: &str, tgt: &str) -> f64 {
        match src {
            "\u{00B0}C" => match tgt {
                "K" => 273.15,
                "\u{00B0}F" => 32.0,
                "\u{00B0}R" => 491.67,
                _ => 0.0,
            },
            "K" => match tgt {
                "\u{00B0}C" => -273.15,
                "\u{00B0}F" => -459.67,
                _ => 0.0,
            },
            "\u{00B0}F" => match tgt {
                "\u{00B0}C" => -17.7777777777777778,
                "K" => 255.3722222222222222,
                "\u{00B0}R" => 459.67,
                _ => 0.0,
            },
            "\u{00B0}R" => match tgt {
                "\u{00B0}C" => -273.15,
                "\u{00B0}F" => -459.67,
                _ => 0.0,
            },
            _ => 0.0,
        }
    }

    /// `double Normalize()`
    pub fn normalize(&self) -> f64 {
        let n = self.len();
        if n == 0 || self.text.borrow().is_some() {
            return 1.0;
        }
        let mut factor = 1.0;
        for i in 0..n {
            if self.pw[i] == 0.0 {
                continue;
            }
            let d = self.fc[i].get();
            if d != 1.0 {
                let d = norm_dim(d, self.unit_name(i));
                self.fc[i].set(self.fc[i].get() / d);
                factor *= d.powf(self.pw[i] as f64);
            }
        }
        factor
    }

    pub fn is_composite(d: f64, units: &OptUnit) -> bool {
        match units {
            None => false,
            Some(u) => {
                d > 0.0 && d != 1.0 || {
                    let t = u.text.borrow();
                    match t.as_ref() {
                        Some(t) => t.contains(&COMPOSITE_UNIT_CHARS[..]),
                        None => false,
                    }
                }
            }
        }
    }

    pub fn convert(ua: &OptUnit, ub: &OptUnit, op: char) -> CResult<f64> {
        match (ua, ub) {
            (Some(a), Some(b)) if Rc::ptr_eq(a, b) => return Ok(1.0),
            (None, None) => return Ok(1.0),
            (None, Some(b)) => {
                if b.is_dimensionless() {
                    return Ok(b.get_dimensionless_factor());
                }
            }
            (Some(a), None) => {
                if a.is_dimensionless() {
                    return Ok(1.0 / a.get_dimensionless_factor());
                }
            }
            (Some(a), Some(b)) => {
                if a.is_consistent(b) {
                    return Ok(b.convert_to(a));
                }
            }
        }
        Err(exceptions::InconsistentUnitsOperation(
            &Unit::get_text_of(ua),
            op,
            &Unit::get_text_of(ub),
        ))
    }

    pub fn multiply(ua: &OptUnit, ub: &OptUnit, update_text: bool) -> (OptUnit, f64) {
        let ub = match ub {
            None => {
                if let Some(a) = ua {
                    if a.is_dimensionless() {
                        return (None, a.get_dimensionless_factor());
                    }
                }
                return (ua.clone(), 1.0);
            }
            Some(b) => b,
        };
        let ua = match ua {
            None => {
                if ub.is_dimensionless() {
                    return (None, ub.get_dimensionless_factor());
                }
                return (Some(ub.clone()), 1.0);
            }
            Some(a) => a,
        };
        if ub.is_dimensionless() {
            let mut d = ub.get_dimensionless_factor();
            if ua.is_dimensionless() {
                d *= ua.get_dimensionless_factor();
                return (None, d);
            }
            return (Some(ua.clone()), d);
        }
        if ua.is_dimensionless() {
            return (Some(ub.clone()), ua.get_dimensionless_factor());
        }
        let mut d = Unit::get_product_or_division_factor(ua, ub, false);
        let uc = match Unit::multiply_or_divide(ua, ub, false) {
            None => return (None, d),
            Some(u) => u,
        };
        if update_text && !ua.is_multiple(ub) {
            uc.scale_in_place(d);
            d = 1.0;
            *uc.text.borrow_mut() = Some(format!("{}{}{}", ua.text(), TextWriter::UNIT_PRODUCT, ub.text()));
        }
        (Some(uc), d)
    }

    pub fn divide(ua: &OptUnit, ub: &OptUnit, update_text: bool) -> (OptUnit, f64) {
        let ub = match ub {
            None => {
                if let Some(a) = ua {
                    if a.is_dimensionless() {
                        return (None, a.get_dimensionless_factor());
                    }
                }
                return (ua.clone(), 1.0);
            }
            Some(b) => b,
        };
        let ua = match ua {
            None => {
                if ub.is_dimensionless() {
                    return (None, 1.0 / ub.get_dimensionless_factor());
                }
                return (ub.pow(-1.0), 1.0);
            }
            Some(a) => a,
        };
        if ub.is_dimensionless() {
            let mut d = 1.0 / ub.get_dimensionless_factor();
            if ua.is_dimensionless() {
                d *= ua.get_dimensionless_factor();
                return (None, d);
            }
            return (Some(ua.clone()), d);
        }
        if ua.is_dimensionless() {
            return (ub.pow(-1.0), ua.get_dimensionless_factor());
        }
        let mut d = Unit::get_product_or_division_factor(ua, ub, true);
        let uc = match Unit::multiply_or_divide(ua, ub, true) {
            None => return (None, d),
            Some(u) => u,
        };
        if update_text && !ua.is_multiple(ub) {
            uc.scale_in_place(d);
            d = 1.0;
            let bt = ub.text();
            let t = if bt.contains(['\u{00B7}', '\u{2215}']) {
                format!("{}{}({})", ua.text(), TextWriter::UNIT_DIVISION, bt)
            } else {
                format!("{}{}{}", ua.text(), TextWriter::UNIT_DIVISION, bt)
            };
            *uc.text.borrow_mut() = Some(t);
        }
        (Some(uc), d)
    }

    /// `Unit.Pow(Unit u, IScalarValue power, bool updateText)`
    pub fn pow_scalar(u: &OptUnit, power: &Value, update_text: bool) -> CResult<OptUnit> {
        if power.units().is_some() {
            return Err(exceptions::PowerNotUnitless());
        }
        let u = match u {
            None => return Ok(None),
            Some(u) => u,
        };
        if !power.is_real() {
            return Err(exceptions::UnitsToComplexPower());
        }
        let d = power.re();
        let f = d as f32;
        let result = u.pow(f);
        if update_text {
            if let Some(r) = &result {
                let s = u.text();
                if !s.contains('^') {
                    let writer = TextWriter::new(None, false);
                    let ps = match get_fraction(f) {
                        Some(x) => x,
                        None => {
                            if d < 0.0 {
                                format!("({})", writer.format_number_helper(d, None))
                            } else {
                                writer.format_number_helper(d, None)
                            }
                        }
                    };
                    let t = if s.contains(&COMPOSITE_UNIT_CHARS[..]) {
                        format!("({})^{}", s, ps)
                    } else {
                        format!("{}^{}", s, ps)
                    };
                    *r.text.borrow_mut() = Some(t);
                }
            }
        }
        Ok(result)
    }

    pub fn root(u: &UnitRef, n: i32, update_text: bool) -> OptUnit {
        let result = u.pow(1.0 / n as f32);
        if update_text && n.abs() > 1 {
            let s = u.text();
            if !s.contains('^') {
                if let Some(r) = &result {
                    let t = if s.contains(&COMPOSITE_UNIT_CHARS[..]) {
                        format!("({})^(1/{})", s, n)
                    } else {
                        format!("{}^(1/{})", s, n)
                    };
                    *r.text.borrow_mut() = Some(t);
                }
            }
        }
        result
    }

    pub fn get_dimensionless_factor(&self) -> f64 {
        match self.text().as_str() {
            "%" => 1e-2,
            "\u{2030}" => 1e-3,
            "\u{2031}" => 1e-4,
            "pcm" => 1e-5,
            "ppm" => 1e-6,
            "ppb" => 1e-9,
            "ppt" => 1e-12,
            "ppq" => 1e-15,
            _ => 1.0,
        }
    }

    pub fn ptr_eq(a: &OptUnit, b: &OptUnit) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text.borrow().clone().unwrap_or_default())
    }
}

fn temperature_to_delta(s: &str) -> String {
    let mut it = s.chars();
    match it.next() {
        Some('\u{00B0}') => {
            let c1 = it.next().map(|c| c.to_string()).unwrap_or_default();
            format!("\u{0394}\u{00B0}{}", c1)
        }
        Some('K') => "\u{0394}\u{00B0}C".to_string(),
        Some('R') => "\u{0394}\u{00B0}F".to_string(),
        _ => s.to_string(),
    }
}

pub fn get_product_or_division_operator(format: OutputFormat, is_product: bool) -> &'static str {
    if is_product {
        match format {
            OutputFormat::Html => HtmlWriter::UNIT_PRODUCT,
            OutputFormat::Xml => crate::output::xml_writer::unit_product_static(),
            _ => "\u{00B7}",
        }
    } else {
        match format {
            OutputFormat::Html => HtmlWriter::UNIT_DIVISION,
            OutputFormat::Xml => crate::output::xml_writer::unit_division_static(),
            _ => "\u{2215}",
        }
    }
}

pub fn get_prefix(n: i32) -> &'static str {
    match n {
        -24 => "y",
        -21 => "z",
        -18 => "a",
        -15 => "f",
        -12 => "p",
        -9 => "n",
        -6 => "\u{03BC}",
        -3 => "m",
        -2 => "c",
        -1 => "d",
        1 => "da",
        2 => "h",
        3 => "k",
        6 => "M",
        9 => "G",
        12 => "T",
        15 => "P",
        18 => "E",
        21 => "Z",
        24 => "Y",
        _ => "",
    }
}

pub fn get_scale(n: i32) -> f64 {
    match n {
        -24 => 1E-24,
        -21 => 1E-21,
        -18 => 1E-18,
        -15 => 1E-15,
        -12 => 1E-12,
        -9 => 1E-09,
        -6 => 1E-06,
        -3 => 0.001,
        -2 => 0.01,
        -1 => 0.1,
        0 => 1.0,
        1 => 10.0,
        2 => 100.0,
        3 => 1E+3,
        6 => 1E+6,
        9 => 1E+9,
        12 => 1E+12,
        15 => 1E+15,
        18 => 1E+18,
        21 => 1E+21,
        24 => 1E+24,
        _ => 10f64.powi(n),
    }
}

fn get_power(factor: f64) -> i32 {
    if factor <= 1e-2 {
        if factor <= 1E-24 {
            -24
        } else if factor <= 1E-21 {
            -21
        } else if factor <= 1E-18 {
            -18
        } else if factor <= 1E-15 {
            -15
        } else if factor <= 1E-12 {
            -12
        } else if factor <= 1E-9 {
            -9
        } else if factor <= 1E-6 {
            -6
        } else if factor <= 1E-3 {
            -3
        } else {
            -2
        }
    } else if factor >= 1E+3 {
        if factor >= 1E+24 {
            24
        } else if factor >= 1E+21 {
            21
        } else if factor >= 1E+18 {
            18
        } else if factor >= 1E+15 {
            15
        } else if factor >= 1E+12 {
            12
        } else if factor >= 1E+9 {
            9
        } else if factor >= 1E+6 {
            6
        } else {
            3
        }
    } else {
        0
    }
}

fn near_int(a: f64) -> bool {
    (a - a.round_ties_even()).abs() < 1e-12
}

fn norm_dim(factor: f64, name: &str) -> f64 {
    let mut factor = factor;
    if factor == 1.0 {
        return 1.0;
    }
    match name {
        "s" => {
            if factor == 60.0 || factor == 3600.0 || factor == 86400.0 {
                return 1.0;
            }
        }
        "g" => {
            let a1 = factor / 14.59390294;
            if near_int(a1) {
                factor = a1;
            } else {
                let a2 = factor / 453.59237;
                if a2 < 1.0 {
                    let a3 = 1.0 / a2;
                    if near_int(a3) {
                        let a3 = a3.round_ties_even();
                        if a3 == 7000.0 || a3 == 256.0 || a3 == 16.0 {
                            return 1.0;
                        } else {
                            factor = a3;
                        }
                    }
                } else if near_int(a2) {
                    let a2 = a2.round_ties_even();
                    if [14.0, 28.0, 100.0, 112.0, 1000.0, 2000.0, 2240.0].contains(&a2) {
                        return 1.0;
                    } else {
                        factor = a2;
                    }
                } else if factor >= 1000000.0 {
                    factor /= 1000000.0;
                } else if factor >= 1000.0 {
                    factor /= 1000.0;
                }
            }
        }
        "m" => {
            if factor == 1852.0 {
                return 1.0;
            } else {
                let a = factor / 2.54e-05;
                if near_int(a) {
                    let a = a.round_ties_even();
                    if [1.0, 1000.0, 36000.0, 792000.0, 7920000.0, 63360000.0].contains(&a) {
                        return 1.0;
                    } else {
                        factor = a / 12000.0;
                    }
                }
            }
        }
        "rad" => {
            if (factor - std::f64::consts::TAU).abs() < 1e-12 {
                return 1.0;
            }
            let a = std::f64::consts::PI / factor;
            if near_int(a) {
                let b = a.round_ties_even();
                if b == 180.0 || b == 200.0 || b == 10800.0 || b == 648000.0 {
                    return 1.0;
                } else {
                    return a * std::f64::consts::PI;
                }
            }
        }
        _ => {}
    }
    let n = get_power(factor);
    factor /= get_scale(n);
    if (factor - 1.0).abs() > 1e-12 {
        if name == "g" {
            factor *= 1000.0;
        }
        return factor;
    }
    1.0
}

fn get_dim_text(writer: &dyn OutputWriter, name: &str, factor: f64, power: f32) -> String {
    let mut name: String = name.to_string();
    let mut factor = factor;
    if factor != 1.0 {
        match name.as_str() {
            "s" => {
                if factor == 60.0 || factor == 3600.0 || factor == 86400.0 {
                    name = match factor as i64 {
                        60 => "min",
                        3600 => "h",
                        86400 => "d",
                        _ => "s",
                    }
                    .to_string();
                    factor = 1.0;
                }
            }
            "g" => {
                let a1 = factor / 14.59390294;
                if near_int(a1) {
                    name = "slug".to_string();
                    factor = a1;
                } else {
                    let a2 = factor / 453.59237;
                    if a2 < 1.0 {
                        let a3 = 1.0 / a2;
                        if near_int(a3) {
                            let a3 = a3.round_ties_even();
                            factor = 1.0;
                            match a3 as i64 {
                                7000 => name = "gr".to_string(),
                                256 => name = "dr".to_string(),
                                16 => name = "oz".to_string(),
                                _ => {
                                    name = "lb".to_string();
                                    factor = a3;
                                }
                            }
                        }
                    } else if near_int(a2) {
                        let a2 = a2.round_ties_even();
                        factor = 1.0;
                        match a2 as i64 {
                            14 => name = "st".to_string(),
                            28 => name = "qr".to_string(),
                            100 => name = "cwt_US".to_string(),
                            112 => name = "cwt_UK".to_string(),
                            1000 => name = "kip_m".to_string(),
                            2000 => name = "ton_US".to_string(),
                            2240 => name = "ton_UK".to_string(),
                            _ => {
                                name = "lb".to_string();
                                factor = a2;
                            }
                        }
                    } else if factor >= 1000000.0 {
                        factor /= 1000000.0;
                        name = "t".to_string();
                    } else if factor >= 1000.0 {
                        factor /= 1000.0;
                        name = "kg".to_string();
                    }
                }
            }
            "m" => {
                if factor == 1852.0 {
                    name = "nmi".to_string();
                    factor = 1.0;
                } else {
                    let a = factor / 2.54e-05;
                    if near_int(a) {
                        let a = a.round_ties_even();
                        factor = 1.0;
                        match a as i64 {
                            1 => name = "th".to_string(),
                            1000 => name = "in".to_string(),
                            36000 => name = "yd".to_string(),
                            792000 => name = "ch".to_string(),
                            7920000 => name = "fur".to_string(),
                            63360000 => name = "mi".to_string(),
                            _ => {
                                name = "ft".to_string();
                                factor = a / 12000.0;
                            }
                        }
                    }
                }
            }
            "rad" => {
                if (factor - std::f64::consts::TAU).abs() < 1e-12 {
                    name = "rev".to_string();
                    factor = 1.0;
                } else {
                    let a = std::f64::consts::PI / factor;
                    if near_int(a) {
                        let b = a.round_ties_even();
                        factor = 1.0;
                        match b as i64 {
                            180 => name = "deg".to_string(),
                            200 => name = "grad".to_string(),
                            10800 => name = "\u{2032}".to_string(),
                            648000 => name = "\u{2033}".to_string(),
                            _ => {
                                name = "rad".to_string();
                                factor = a * std::f64::consts::PI;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        let n = get_power(factor);
        name = writer.format_units(&format!("{}{}", get_prefix(n), name));
        factor /= get_scale(n);
        if (factor - 1.0).abs() > 1e-12 {
            name = writer.add_brackets(
                &format!("{}\u{00B7}{}", writer.format_real(factor, None, false), name),
                0,
                '(',
                ')',
            );
        }
    } else {
        name = writer.format_units(&name);
    }
    if power == 1.0 {
        return name;
    }
    let mut sp = writer.format_real(power as f64, None, false);
    if power < 0.0 && writer.format_type() == OutputFormat::Text {
        sp = writer.add_brackets(&sp, 0, '(', ')');
    }
    writer.format_power(&name, &sp, -1, -1)
}

fn get_fraction(f: f32) -> Option<String> {
    let uf = f.abs();
    if uf >= 1.0 {
        return None;
    }
    let s = if f < 0.0 { "-" } else { "" };
    let r = if uf == 1.0 / 2.0 {
        "1/2"
    } else if uf == 1.0 / 3.0 {
        "1/3"
    } else if uf == 2.0 / 3.0 {
        "2/3"
    } else if uf == 1.0 / 4.0 {
        "1/4"
    } else if uf == 3.0 / 4.0 {
        "3/4"
    } else if uf == 1.0 / 5.0 {
        "1/5"
    } else if uf == 2.0 / 5.0 {
        "2/5"
    } else if uf == 3.0 / 5.0 {
        "3/5"
    } else if uf == 4.0 / 5.0 {
        "4/5"
    } else {
        return None;
    };
    Some(format!("({}{})", s, r))
}

// ----- static table (Unit static constructor) -----------------------------------------

struct UnitTable {
    units: HashMap<String, UnitRef>,
    force_units: Vec<UnitRef>,
    force_units_us: Vec<UnitRef>,
    electrical: Vec<(UnitRef, UnitRef)>,
}

thread_local! {
    static TABLE: UnitTable = UnitTable::build();
}

fn r(u: Unit) -> UnitRef {
    Rc::new(u)
}

fn mul(a: &UnitRef, b: &UnitRef) -> UnitRef {
    let (u, d) = (
        Unit::multiply_or_divide(a, b, false),
        Unit::get_product_or_division_factor(a, b, false),
    );
    // C# operator * (Unit, Unit) is MultiplyOrDivide only; the factor is not applied.
    let _ = d;
    u.expect("unit product")
}

fn div(a: &UnitRef, b: &UnitRef) -> UnitRef {
    Unit::multiply_or_divide(a, b, true).expect("unit quotient")
}

fn powu(u: &UnitRef, x: f32) -> UnitRef {
    u.pow(x).unwrap()
}

impl UnitTable {
    fn build() -> UnitTable {
        let nd = |t: &str, m: f32, l: f32, s: f32, a: f32, k: f32, n: f32, c: f32, ang: f32| {
            r(Unit::new_dims(t, m, l, s, a, k, n, c, ang))
        };
        let g = r(nd("kg", 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0).scale("g", 0.001));
        let m = nd("m", 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mi = r(m.scale("mi", 1609.344));
        let nmi = r(m.scale("nmi", 1852.0));
        let a = r(powu(&m, 2.0).scale("a", 100.0));
        let l = r(Unit::copy(&powu(&r(m.shift(-1)), 3.0)));
        l.set_text("L");
        let s = nd("s", 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let h = r(s.scale("h", 3600.0));
        let deg = nd("\u{00B0}C", 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let amp = nd("A", 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let ah = r(Unit::alias(&mul(&amp, &h), "Ah"));
        let n = nd("N", 1.0, 1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let kn = r(n.shift(3));
        let nm = nd("Nm", 1.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let knm = r(nm.shift(3));
        let hz = nd("Hz", 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let pa = nd("Pa", 1.0, -1.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let kpa = r(pa.shift(3));
        let j = nd("J", 1.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let w = nd("W", 1.0, 2.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let c = nd("C", 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let v = nd("V", 1.0, 2.0, -3.0, -1.0, 0.0, 0.0, 0.0, 0.0);
        let kv = r(v.shift(3));
        let f = nd("F", -1.0, -2.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0);
        let ohm = nd("\u{03A9}", 1.0, 2.0, -3.0, -2.0, 0.0, 0.0, 0.0, 0.0);
        let siemens = nd("S", -1.0, -2.0, 3.0, 2.0, 0.0, 0.0, 0.0, 0.0);
        let wb = nd("Wb", 1.0, 2.0, -2.0, -1.0, 0.0, 0.0, 0.0, 0.0);
        let t = nd("T", 1.0, 0.0, -2.0, -1.0, 0.0, 0.0, 0.0, 0.0);
        let hy = nd("H", 1.0, 2.0, -2.0, -2.0, 0.0, 0.0, 0.0, 0.0);
        let bq = nd("Bq", 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let gy = nd("Gy", 0.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let sv = nd("Sv", 0.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        let lbm = r(g.scale("lb", 453.59237));
        let kipm = r(g.scale("kipm", 453592.37));
        let lbf = r(n.scale("lbf", 4.4482216153));
        let kipf = r(n.scale("kip", 4448.2216153));
        let ksi = r(pa.scale("ksi", 6894757.29322959));

        let mut force_units: Vec<UnitRef> = vec![
            mul(&kn, &powu(&m, -4.0)),
            mul(&kn, &powu(&m, -3.0)),
            kpa.clone(),
            mul(&kn, &powu(&m, -1.0)),
            kn.clone(),
            knm.clone(),
            mul(&kn, &powu(&m, 2.0)),
            mul(&kn, &powu(&m, 3.0)),
            mul(&kn, &powu(&m, 4.0)),
        ];
        force_units[0].set_text("kN\u{2215}m^4");
        force_units[1].set_text("kN\u{2215}m^3");
        force_units[3].set_text("kN\u{2215}m");
        force_units[6].set_text("kN\u{00B7}m^2");
        force_units[7].set_text("kN\u{00B7}m^3");
        force_units[8].set_text("kN\u{00B7}m^4");
        // keep vector type mutable warning away
        force_units.shrink_to_fit();

        let force_units_us: Vec<UnitRef> = vec![
            r(mul(&kipf, &powu(&m, -4.0)).scale("kip\u{2215}in^4", 1.0 / 4.162314256E-7)),
            r(mul(&kipf, &powu(&m, -3.0)).scale("kip\u{2215}in^3", 1.0 / 0.000016387064)),
            ksi.clone(),
            r(mul(&kipf, &powu(&m, -1.0)).scale("kip\u{2215}ft", 1.0 / 0.3048)),
            kipf.clone(),
            r(mul(&kipf, &m).scale("kip\u{00B7}ft", 0.3048)),
            r(mul(&kipf, &powu(&m, 2.0)).scale("kip\u{00B7}ft^2", 0.09290304)),
            r(mul(&kipf, &powu(&m, 3.0)).scale("kip\u{00B7}ft^3", 0.028316846592)),
            r(mul(&kipf, &powu(&m, 4.0)).scale("kip\u{00B7}ft^4", 0.0086309748412416)),
        ];

        let electrical = vec![
            (siemens.clone(), siemens.clone()),
            (f.clone(), f.clone()),
            (c.clone(), c.clone()),
            (t.clone(), t.clone()),
            (ohm.clone(), ohm.clone()),
            (v.clone(), v.clone()),
            (kv.clone(), v.clone()),
            (w.clone(), w.clone()),
            (hy.clone(), hy.clone()),
            (ah.clone(), ah.clone()),
            (wb.clone(), wb.clone()),
        ];

        let mut units: HashMap<String, UnitRef> = HashMap::new();
        let mut add = |k: &str, u: UnitRef| {
            units.insert(k.to_string(), u);
        };
        let dimless = |t: &str| {
            let u = Unit::new_n(0);
            u.set_text(t);
            r(u)
        };
        // The empty key: 9 dimensions with a phantom ninth one.
        {
            let mut p = [0f32; 9];
            let mut fct = [0f64; 9];
            p[8] = 1.0;
            fct[8] = 1.0;
            add("", r(Unit::from_parts(&p, &fct, 'C')));
        }
        for t in ["%", "\u{2030}", "\u{2031}", "pcm", "ppm", "ppb", "ppt", "ppq"] {
            add(t, dimless(t));
        }
        add("g", g.clone());
        add("dag", r(g.shift(1)));
        add("hg", r(g.shift(2)));
        add("kg", r(g.shift(3)));
        add("t", r(g.scale("t", 1000000.0)));
        add("kt", r(g.scale("kt", 1000000000.0)));
        add("Mt", r(g.scale("Mt", 1000000000000.0)));
        add("Gt", r(g.scale("Gt", 1E+15)));
        add("dg", r(g.shift(-1)));
        add("cg", r(g.shift(-2)));
        add("mg", r(g.shift(-3)));
        add("\u{03BC}g", r(g.shift(-6)));
        add("ng", r(g.shift(-9)));
        add("pg", r(g.shift(-12)));
        add("Da", r(g.scale("Da", 1.6605390666050505e-24)));
        add("u", r(g.scale("u", 1.6605390666050505e-24)));
        add("gr", r(g.scale("gr", 0.06479891)));
        add("dr", r(g.scale("dr", 1.7718451953125)));
        add("oz", r(g.scale("oz", 28.349523125)));
        add("lb", lbm.clone());
        add("lbm", r(Unit::alias(&lbm, "lbm")));
        add("lb_m", r(Unit::alias(&lbm, "lb_m")));
        add("kipm", kipm.clone());
        add("kip_m", r(Unit::alias(&kipm, "kip_m")));
        add("klb", r(Unit::alias(&kipm, "klb")));
        add("st", r(g.scale("st", 6350.29318)));
        add("qr", r(g.scale("qr", 12700.58636)));
        add("cwt_US", r(g.scale("cwt_US", 45359.237)));
        add("cwt_UK", r(g.scale("cwt_UK", 50802.34544)));
        add("ton_US", r(g.scale("ton_US", 907184.74)));
        add("ton_UK", r(g.scale("ton_UK", 1016046.9088)));
        add("slug", r(g.scale("slug", 14593.90294)));

        add("m", m.clone());
        add("km", r(m.shift(3)));
        add("dm", r(m.shift(-1)));
        add("cm", r(m.shift(-2)));
        add("mm", r(m.shift(-3)));
        add("\u{03BC}m", r(m.shift(-6)));
        add("nm", r(m.shift(-9)));
        add("pm", r(m.shift(-12)));
        add("AU", r(m.scale("AU", 149597870700.0)));
        add("ly", r(m.scale("ly", 9460730472580800.0)));
        add("th", r(m.scale("th", 2.54E-05)));
        add("in", r(m.scale("in", 0.0254)));
        add("ft", r(m.scale("ft", 0.3048)));
        add("yd", r(m.scale("yd", 0.9144)));
        add("ch", r(m.scale("ch", 20.1168)));
        add("fur", r(m.scale("fur", 201.168)));
        add("mi", mi.clone());
        add("ftm", r(m.scale("ftm", 1.8288)));
        add("ftm_UK", r(m.scale("ftm_UK", 1.852)));
        add("ftm_US", r(m.scale("ftm_US", 1.8288)));
        add("cable", r(m.scale("cable", 182.88)));
        add("cable_UK", r(m.scale("cable_UK", 185.2)));
        add("cable_US", r(m.scale("cable_US", 219.456)));
        add("nmi", nmi.clone());
        add("li", r(m.scale("li", 0.201168)));
        add("rod", r(m.scale("rod", 5.0292)));
        add("pole", r(m.scale("pole", 5.0292)));
        add("perch", r(m.scale("perch", 5.0292)));
        add("lea", r(m.scale("lea", 4828.032)));

        add("a", a.clone());
        add("daa", r(a.scale("daa", 10.0)));
        add("ha", r(a.scale("ha", 100.0)));
        add("L", l.clone());
        add("dL", r(l.shift(-1)));
        add("cL", r(l.shift(-2)));
        add("mL", r(l.shift(-3)));
        add("\u{03BC}L", r(l.shift(-6)));
        add("nL", r(l.shift(-9)));
        add("pL", r(l.shift(-12)));
        add("daL", r(l.scale("daL", 10.0)));
        add("hL", r(l.scale("hL", 100.0)));
        add("rood", r(powu(&m, 2.0).scale("rood", 1011.7141056)));
        add("ac", r(powu(&m, 2.0).scale("ac", 4046.8564224)));

        add("fl_dr_UK", r(l.scale("fl_dr_UK", 3.5516328125e-3)));
        add("fl_oz_UK", r(l.scale("fl_oz_UK", 0.0284130625)));
        add("gi_UK", r(l.scale("gi_UK", 0.1420653125)));
        add("pt_UK", r(l.scale("pt_UK", 0.56826125)));
        add("qt_UK", r(l.scale("qt_UK", 1.1365225)));
        add("gal_UK", r(l.scale("gal_UK", 4.54609)));
        add("bbl_UK", r(l.scale("bbl_UK", 163.65924)));
        add("pk_UK", r(l.scale("pk_UK", 9.09218)));
        add("bu_UK", r(l.scale("bu_UK", 36.36872)));
        add("fl_dr_US", r(l.scale("fl_dr_US", 3.6966911953125e-3)));
        add("fl_oz_US", r(l.scale("fl_oz_US", 0.0295735295625)));
        add("gi_US", r(l.scale("gi_US", 0.11829411825)));
        add("pt_US", r(l.scale("pt_US", 0.473176473)));
        add("qt_US", r(l.scale("qt_US", 0.946352946)));
        add("gal_US", r(l.scale("gal_US", 3.785411784)));
        add("bbl_US", r(l.scale("bbl_US", 119.240471196)));
        add("pt_dry", r(l.scale("pt_dry", 0.5506104713575)));
        add("qt_dry", r(l.scale("qt_dry", 1.101220942715)));
        add("gal_dry", r(l.scale("gal_dry", 4.40488377086)));
        add("bbl_dry", r(l.scale("bbl_dry", 115.628198985075)));
        add("pk_US", r(l.scale("pk_US", 8.80976754172)));
        add("bu_US", r(l.scale("bu_US", 35.23907016688)));

        add("s", s.clone());
        add("ms", r(s.shift(-3)));
        add("\u{03BC}s", r(s.shift(-6)));
        add("ns", r(s.shift(-9)));
        add("ps", r(s.shift(-12)));
        add("min", r(s.scale("min", 60.0)));
        add("h", h.clone());
        add("d", r(h.scale("d", 24.0)));
        add("w", r(h.scale("w", 168.0)));
        add("y", r(h.scale("y", 8760.0)));
        add("kmh", r(Unit::alias(&div(&r(m.shift(3)), &h), "kmh")));
        add("mph", r(Unit::alias(&div(&mi, &h), "mph")));
        add("knot", r(Unit::alias(&div(&nmi, &h), "knot")));
        add("Hz", hz.clone());
        add("kHz", r(hz.shift(3)));
        add("MHz", r(hz.shift(6)));
        add("GHz", r(hz.shift(9)));
        add("THz", r(hz.shift(12)));
        add("mHz", r(hz.shift(-3)));
        add("\u{03BC}Hz", r(hz.shift(-6)));
        add("nHz", r(hz.shift(-9)));
        add("pHz", r(hz.shift(-12)));
        add("rpm", r(hz.scale("rpm", 1.0 / 60.0)));

        add("A", amp.clone());
        add("kA", r(amp.shift(3)));
        add("MA", r(amp.shift(6)));
        add("GA", r(amp.shift(9)));
        add("TA", r(amp.shift(12)));
        add("mA", r(amp.shift(-3)));
        add("\u{03BC}A", r(amp.shift(-6)));
        add("nA", r(amp.shift(-9)));
        add("pA", r(amp.shift(-12)));
        add("Ah", ah.clone());
        add("mAh", r(Unit::alias(&mul(&r(amp.shift(-3)), &h), "mAh")));

        add("\u{00B0}C", r(deg.scale("\u{00B0}C", 1.0)));
        add("\u{0394}\u{00B0}C", r(deg.scale("\u{0394}\u{00B0}C", 1.0)));
        add("K", r(deg.scale("K", 1.0)));
        add("\u{00B0}F", r(deg.scale("\u{00B0}F", 5.0 / 9.0)));
        add("\u{0394}\u{00B0}F", r(deg.scale("\u{0394}\u{00B0}F", 5.0 / 9.0)));
        add("\u{00B0}R", r(deg.scale("\u{00B0}R", 5.0 / 9.0)));

        add("mol", nd("mol", 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0));
        add("cd", nd("cd", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0));

        add("N", n.clone());
        add("daN", r(n.shift(1)));
        add("hN", r(n.shift(2)));
        add("kN", kn.clone());
        add("MN", r(n.shift(6)));
        add("GN", r(n.shift(9)));
        add("TN", r(n.shift(12)));
        add("Nm", nm.clone());
        add("kNm", knm.clone());
        add("gf", r(n.scale("gf", 0.00980665)));
        add("kgf", r(n.scale("kgf", 9.80665)));
        add("tf", r(n.scale("tf", 9806.65)));
        add("dyn", r(n.scale("dyn", 1e-5)));
        add("ozf", r(n.scale("ozf", 0.278013851)));
        add("lbf", lbf.clone());
        add("kip", kipf.clone());
        add("kipf", r(Unit::alias(&kipf, "kipf")));
        add("tonf_US", r(n.scale("tonf_US", 8896.443230521)));
        add("tonf_UK", r(n.scale("tonf_UK", 9964.01641818352)));
        add("pdl", r(n.scale("pdl", 0.138254954376)));
        add("oz_f", r(n.scale("oz_f", 0.278013851)));
        add("lb_f", r(Unit::alias(&lbf, "lb_f")));
        add("kip_f", r(Unit::alias(&kipf, "kip_f")));

        add("Pa", pa.clone());
        add("daPa", r(pa.shift(1)));
        add("hPa", r(pa.shift(2)));
        add("kPa", kpa.clone());
        add("MPa", r(pa.shift(6)));
        add("GPa", r(pa.shift(9)));
        add("TPa", r(pa.shift(12)));
        add("dPa", r(pa.shift(-1)));
        add("cPa", r(pa.shift(-2)));
        add("mPa", r(pa.shift(-3)));
        add("\u{03BC}Pa", r(pa.shift(-6)));
        add("nPa", r(pa.shift(-9)));
        add("pPa", r(pa.shift(-12)));
        add("bar", r(pa.scale("bar", 100000.0)));
        add("mbar", r(pa.scale("mbar", 100.0)));
        add("\u{03BC}bar", r(pa.scale("\u{03BC}bar", 0.1)));
        add("atm", r(pa.scale("atm", 101325.0)));
        add("mmHg", r(pa.scale("mmHg", 133.322387415)));
        add("at", r(pa.scale("at", 98066.5)));
        add("Torr", r(pa.scale("Torr", 133.32236842)));
        add("osi", r(pa.scale("osi", 430.922330894662)));
        add("osf", r(pa.scale("osf", 2.99251618676848)));
        add("psi", r(pa.scale("psi", 6894.75729322959)));
        add("ksi", ksi.clone());
        add("tsi", r(pa.scale("tsi", 15444256.3366971)));
        add("psf", r(pa.scale("psf", 47.880258980761)));
        add("ksf", r(pa.scale("ksf", 47880.258980761)));
        add("tsf", r(pa.scale("tsf", 107251.780115952)));
        add("inHg", r(pa.scale("inHg", 3386.389)));
        add("P", r(mul(&pa, &s).scale("P", 0.1)));
        add("cP", r(mul(&pa, &s).scale("cP", 0.001)));
        add("St", r(div(&mul(&m, &m), &s).scale("St", 0.0001)));
        add("cSt", r(div(&mul(&m, &m), &s).scale("cSt", 0.000001)));

        add("J", j.clone());
        add("kJ", r(j.shift(3)));
        add("MJ", r(j.shift(6)));
        add("GJ", r(j.shift(9)));
        add("TJ", r(j.shift(12)));
        add("mJ", r(j.shift(-3)));
        add("\u{03BC}J", r(j.shift(-6)));
        add("nJ", r(j.shift(-9)));
        add("pJ", r(j.shift(-12)));
        for (k, f) in [
            ("Wh", 3.6e+3),
            ("kWh", 3.6e+6),
            ("MWh", 3.6e+9),
            ("GWh", 3.6e+12),
            ("TWh", 3.6e+15),
            ("mWh", 3.6),
            ("\u{03BC}Wh", 3.6e-3),
            ("nWh", 3.6e-6),
            ("pWh", 3.6e-9),
            ("VAh", 3.6e+3),
            ("kVAh", 3.6e+6),
            ("MVAh", 3.6e+9),
            ("GVAh", 3.6e+12),
            ("VARh", 3.6e+3),
            ("kVARh", 3.6e+6),
            ("MVARh", 3.6e+9),
            ("GVARh", 3.6e+12),
            ("erg", 1e-7),
            ("eV", 1.6021773300241367e-19),
            ("keV", 1.6021773300241367e-16),
            ("MeV", 1.6021773300241367e-13),
            ("GeV", 1.6021773300241367e-10),
            ("TeV", 1.6021773300241367e-7),
            ("PeV", 1.6021773300241367e-4),
            ("EeV", 1.6021773300241367e-1),
            ("BTU", 1055.05585262),
            ("therm_US", 1054.804e+5),
            ("therm_UK", 1055.05585262e+5),
            ("quad", 1055.05585262e+15),
            ("cal", 4.1868),
            ("kcal", 4186.8),
        ] {
            add(k, r(j.scale(k, f)));
        }

        add("W", w.clone());
        add("kW", r(w.shift(3)));
        add("MW", r(w.shift(6)));
        add("GW", r(w.shift(9)));
        add("TW", r(w.shift(12)));
        add("mW", r(w.shift(-3)));
        add("\u{03BC}W", r(w.shift(-6)));
        add("nW", r(w.shift(-9)));
        add("pW", r(w.shift(-12)));
        for (k, f) in [
            ("VA", 1.0),
            ("kVA", 1e3),
            ("MVA", 1e6),
            ("GVA", 1e9),
            ("TVA", 1e12),
            ("mVA", 1e-3),
            ("\u{03BC}VA", 1e-6),
            ("nVA", 1e-9),
            ("pVA", 1e-12),
            ("VAR", 1.0),
            ("kVAR", 1e3),
            ("MVAR", 1e6),
            ("GVAR", 1e9),
            ("TVAR", 1e12),
            ("mVAR", 1e-3),
            ("\u{03BC}VAR", 1e-6),
            ("nVAR", 1e-9),
            ("pVAR", 1e-12),
            ("hp", 745.69987158227022),
            ("hp_M", 735.49875),
            ("ks", 735.49875),
            ("hp_E", 746.0),
            ("hp_S", 9812.5),
        ] {
            add(k, r(w.scale(k, f)));
        }

        let prefixed = |add: &mut dyn FnMut(&str, UnitRef), base: &Unit, sym: &str, include_base: Option<UnitRef>| {
            add(sym, include_base.unwrap_or_else(|| r(Unit::copy(base))));
            for (p, n) in [
                ("k", 3),
                ("M", 6),
                ("G", 9),
                ("T", 12),
                ("m", -3),
                ("\u{03BC}", -6),
                ("n", -9),
                ("p", -12),
            ] {
                add(&format!("{}{}", p, sym), r(base.shift(n)));
            }
        };
        prefixed(&mut add, &c, "C", Some(c.clone()));
        prefixed(&mut add, &v, "V", Some(v.clone()));
        // kV must be the same object that is in the electrical table.
        add("kV", kv.clone());
        prefixed(&mut add, &f, "F", Some(f.clone()));
        prefixed(&mut add, &ohm, "\u{03A9}", Some(ohm.clone()));
        prefixed(&mut add, &siemens, "S", Some(siemens.clone()));
        for (k, fct) in [
            ("\u{2127}", 1.0),
            ("k\u{2127}", 1e3),
            ("M\u{2127}", 1e6),
            ("G\u{2127}", 1e9),
            ("T\u{2127}", 1e12),
            ("m\u{2127}", 1e-3),
            ("\u{03BC}\u{2127}", 1e-6),
            ("n\u{2127}", 1e-9),
            ("p\u{2127}", 1e-12),
        ] {
            add(k, r(siemens.scale(k, fct)));
        }
        prefixed(&mut add, &wb, "Wb", Some(wb.clone()));
        prefixed(&mut add, &t, "T", Some(t.clone()));
        prefixed(&mut add, &hy, "H", Some(hy.clone()));
        prefixed(&mut add, &bq, "Bq", Some(bq.clone()));
        add("Ci", r(bq.scale("Ci", 3.7e+10)));
        add("Rd", r(bq.scale("Rd", 1e+6)));
        prefixed(&mut add, &gy, "Gy", Some(gy.clone()));
        prefixed(&mut add, &sv, "Sv", Some(sv.clone()));

        add("lm", nd("lm", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        add("lx", nd("lx", 0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0));
        add("kat", nd("kat", 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0));

        let angle = |t: &str| nd(t, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let deg_u = angle("\u{00B0}");
        let min_u = angle("\u{2032}");
        let sec_u = angle("\u{2033}");
        let rad_u = angle("rad");
        let grad_u = angle("grad");
        let rev_u = angle("rev");
        deg_u.scale_in_place(std::f64::consts::PI / 180.0);
        min_u.scale_in_place(std::f64::consts::PI / 10800.0);
        sec_u.scale_in_place(std::f64::consts::PI / 648000.0);
        grad_u.scale_in_place(std::f64::consts::PI / 200.0);
        rev_u.scale_in_place(std::f64::consts::TAU);
        add("\u{00B0}", deg_u.clone());
        add("\u{2032}", min_u);
        add("\u{2033}", sec_u);
        add("rad", rad_u);
        add("grad", grad_u);
        add("rev", rev_u);
        add("deg", deg_u);

        units["K"].set_temp_char('K');
        units["\u{00B0}F"].set_temp_char('F');
        units["\u{0394}\u{00B0}F"].set_temp_char('F');
        units["\u{00B0}R"].set_temp_char('R');
        for name in UNIT_NAMES {
            let uk = units[&format!("{}_UK", name)].clone();
            units.insert(name.to_string(), r(Unit::alias(&uk, name)));
        }
        UnitTable {
            units,
            force_units,
            force_units_us,
            electrical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_lookup() {
        let mm = Unit::get("mm", true).unwrap();
        assert_eq!(mm.text(), "mm");
        let m = Unit::get("m", true).unwrap();
        assert!((mm.convert_to(&m) - 0.001).abs() < 1e-18);
        assert_eq!(Unit::try_get("gal", true).unwrap().text(), "gal_US");
    }
}
