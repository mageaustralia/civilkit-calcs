//! Port of Calculator/Calculator.cs (the shared part of RealCalculator and ComplexCalculator).
//!
//! C# keeps arrays of delegates per calculator; here a `Calculator` value knows whether it
//! is the real or the complex one and dispatches on the function index.
pub mod complex_calculator;
mod general_eigen_calc;
pub mod matrix_calculator;
pub mod out_of_scope_names;
pub mod real_calculator;
pub mod statistics;
pub mod vector_calculator;

use std::cell::Cell;

use crate::base_types::complex::math_sign;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::{OptUnit, Unit, UnitRef};
use crate::base_types::value::Value;
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};

pub const DELTA_PLUS: f64 = 1.0 + 1e-14;
pub const DELTA_MINUS: f64 = 1.0 - 1e-14;
/// Hyphen (U+2010), not minus: the internal negation operator.
pub const NEG_CHAR: char = '\u{2010}';
pub const NEG_STR: &str = "\u{2010}";

/// `Calculator.OperatorOrder`: ^ ÷ \ ⦼ * - + < > ≤ ≥ ≡ ≠ ∧ ∨ ⊕ ∠ = ←
pub const OPERATOR_ORDER: [i8; 19] = [0, 4, 4, 4, 4, 5, 6, 7, 7, 7, 7, 7, 7, 8, 9, 9, 3, 10, 10];
pub const OPERATORS: [char; 18] = [
    '^', '/', '\\', '\u{29BC}', '*', '-', '+', '<', '>', '\u{2264}', '\u{2265}', '\u{2261}', '\u{2260}', '\u{2227}',
    '\u{2228}', '\u{2295}', '=', '\u{2220}',
];

pub fn operator_index(c: char) -> Option<usize> {
    Some(match c {
        '^' => 0,
        '/' | '\u{00F7}' => 1,
        '\\' => 2,
        '\u{29BC}' => 3,
        '*' => 4,
        '-' => 5,
        '+' => 6,
        '<' => 7,
        '>' => 8,
        '\u{2264}' => 9,
        '\u{2265}' => 10,
        '\u{2261}' => 11,
        '\u{2260}' => 12,
        '\u{2227}' => 13,
        '\u{2228}' => 14,
        '\u{2295}' => 15,
        '\u{2220}' => 16,
        '=' => 17,
        '\u{2190}' => 18,
        _ => return None,
    })
}

pub fn is_operator(c: char) -> bool {
    operator_index(c).is_some()
}

const IS_ZERO_PRESERVING_OPERATOR: [bool; 19] = [
    false, false, false, false, true, true, true, true, true, false, false, false, true, true, true, true, true, true,
    true,
];

pub fn is_zero_preserving_operator(index: i64) -> bool {
    if index < 0 {
        index == -1
    } else {
        IS_ZERO_PRESERVING_OPERATOR[index as usize]
    }
}

pub fn get_operator_symbol(index: i64) -> char {
    if index < 0 {
        '\0'
    } else {
        OPERATORS[index as usize]
    }
}

pub fn operator_require_consistent_units(index: i64) -> bool {
    index > 4 && index < 13
}

pub const FUNCTION_NAMES: [&str; 46] = [
    "sin", "cos", "tan", "csc", "sec", "cot", "asin", "acos", "atan", "acsc", "asec", "acot", "sinh", "cosh", "tanh",
    "csch", "sech", "coth", "asinh", "acosh", "atanh", "acsch", "asech", "acoth", "log", "ln", "log_2", "exp", "abs",
    "sign", "sqr", "sqrt", "cbrt", "round", "floor", "ceiling", "trunc", "re", "im", "phase", "conj", "random", "fact",
    NEG_STR, "not", "timer",
];
pub const FUNCTION2_NAMES: [&str; 4] = ["atan2", "root", "mod", "mandelbrot"];
pub const FUNCTION3_NAMES: [&str; 1] = ["if"];
pub const MULTI_FUNCTION_NAMES: [&str; 19] = [
    "min", "max", "sum", "sumsq", "srss", "average", "product", "mean", "switch", "and", "or", "xor", "gcd", "lcm",
    "median", "stdev", "stdevp", "var", "varp",
];
pub const INTERPOLATION_NAMES: [&str; 3] = ["take", "line", "spline"];

fn index_of_ci(names: &[&str], s: &str) -> Option<usize> {
    names
        .iter()
        .position(|n| n.eq_ignore_ascii_case(s) || n.to_lowercase() == s.to_lowercase())
}

pub fn function_index(s: &str) -> Option<usize> {
    index_of_ci(&FUNCTION_NAMES, s)
}
pub fn function2_index(s: &str) -> Option<usize> {
    index_of_ci(&FUNCTION2_NAMES, s)
}
pub fn function3_index(s: &str) -> Option<usize> {
    index_of_ci(&FUNCTION3_NAMES, s)
}
pub fn multi_function_index(s: &str) -> Option<usize> {
    index_of_ci(&MULTI_FUNCTION_NAMES, s)
}
pub fn interpolation_index(s: &str) -> Option<usize> {
    index_of_ci(&INTERPOLATION_NAMES, s)
}

pub const SWITCH_INDEX: i64 = 8;
pub const POWER_OPERATOR_INDEX: i64 = 0;
pub const MULTIPLY_OPERATOR_INDEX: i64 = 4;
pub const ADD_OPERATOR_INDEX: i64 = 6;
pub const SQR_FUNCTION_INDEX: i64 = 30;
pub const SQRT_FUNCTION_INDEX: i64 = 31;
pub const CBRT_FUNCTION_INDEX: i64 = 32;
pub const ROOT_FUNCTION_INDEX: i64 = 1;
pub const SRSS_FUNCTION_INDEX: i64 = 4;
pub const SUM_FUNCTION_INDEX: i64 = 2;
pub const IF_FUNCTION_INDEX: i64 = 0;
pub const FACT_FUNCTION_INDEX: i64 = 42;
pub const NEG_FUNCTION_INDEX: i64 = 43;
/// `Calculator.UnitMultOrder` = OperatorOrder['*'] - 2
pub const UNIT_MULT_ORDER: i8 = 2;

pub const FROM_RAD: [f64; 3] = [180.0 / std::f64::consts::PI, 1.0, 200.0 / std::f64::consts::PI];
pub const TO_RAD: [f64; 3] = [std::f64::consts::PI / 180.0, 1.0, std::f64::consts::PI / 200.0];

pub fn angle_unit(i: usize) -> UnitRef {
    Unit::get(["deg", "rad", "grad"][i], false).unwrap()
}

/// Index out of range in a C# delegate array.
pub fn index_out_of_range() -> MathParserException {
    MathParserException::unexpected("Index was outside the bounds of the array.")
}

pub struct Calculator {
    pub is_complex: bool,
    degrees: Cell<i32>,
    return_angle_units: Cell<bool>,
}

impl Calculator {
    pub fn new(is_complex: bool) -> Calculator {
        Calculator {
            is_complex,
            degrees: Cell::new(0),
            return_angle_units: Cell::new(false),
        }
    }

    pub fn degrees(&self) -> i32 {
        self.degrees.get()
    }

    pub fn set_degrees(&self, d: i32) {
        self.degrees.set(d);
    }

    pub fn set_return_angle_units(&self, v: bool) {
        self.return_angle_units.set(v);
    }

    pub fn return_angle_units(&self) -> bool {
        self.return_angle_units.get()
    }

    fn deg_index(&self) -> CResult<usize> {
        let d = self.degrees.get();
        if (0..3).contains(&d) {
            Ok(d as usize)
        } else {
            Err(index_out_of_range())
        }
    }

    pub fn evaluate_operator(&self, index: i64, a: &Value, b: &Value) -> CResult<Value> {
        if self.is_complex {
            complex_calculator::evaluate_operator(index, &a.as_complex(), &b.as_complex())
        } else if let (Value::Real(x), Value::Real(y)) = (a, b) {
            // The common case, without copying the operands (as_real clones a real value).
            real_calculator::evaluate_operator(index, x, y)
        } else {
            real_calculator::evaluate_operator(index, &a.as_real(), &b.as_real())
        }
    }

    pub fn evaluate_function(&self, index: i64, a: &Value) -> CResult<Value> {
        if self.is_complex {
            complex_calculator::evaluate_function(self, index, &a.as_complex())
        } else if let Value::Real(x) = a {
            real_calculator::evaluate_function(self, index, x)
        } else {
            real_calculator::evaluate_function(self, index, &a.as_real())
        }
    }

    pub fn evaluate_function2(&self, index: i64, a: &Value, b: &Value) -> CResult<Value> {
        if self.is_complex {
            complex_calculator::evaluate_function2(self, index, &a.as_complex(), &b.as_complex())
        } else if let (Value::Real(x), Value::Real(y)) = (a, b) {
            real_calculator::evaluate_function2(self, index, x, y)
        } else {
            real_calculator::evaluate_function2(self, index, &a.as_real(), &b.as_real())
        }
    }

    /// `EvaluateFunction3` (only `if`)
    pub fn evaluate_function3(&self, index: i64, a: &Value, b: &Value, c: &Value) -> CResult<Value> {
        if index != 0 {
            return Err(index_out_of_range());
        }
        if_function(a, b, c)
    }

    pub fn evaluate_multi_function(&self, index: i64, a: &[Value]) -> CResult<Value> {
        if index >= statistics::MULTI_BASE {
            return statistics::evaluate(index, a);
        }
        if self.is_complex {
            complex_calculator::evaluate_multi_function(index, a)
        } else {
            real_calculator::evaluate_multi_function(index, a)
        }
    }

    pub fn evaluate_interpolation(&self, index: i64, a: &[Value]) -> CResult<Value> {
        match index {
            0 => take(a),
            1 => line(a),
            2 => spline(a),
            _ => Err(index_out_of_range()),
        }
    }

    pub(crate) fn from_angle_units_factor(&self, units: &OptUnit) -> CResult<f64> {
        Ok(match units {
            None => TO_RAD[self.deg_index()?],
            Some(u) => u.convert_to(&angle_unit(1)),
        })
    }

    pub(crate) fn to_angle_units_real(&self, value: f64) -> CResult<RealValue> {
        let d = self.deg_index()?;
        Ok(if self.return_angle_units.get() {
            RealValue::new(value * FROM_RAD[d], Some(angle_unit(d)))
        } else {
            RealValue::num(value * FROM_RAD[d])
        })
    }
}

pub fn check_trig_function_units(func: &str, unit: &OptUnit) -> CResult<()> {
    if let Some(u) = unit {
        if !u.is_angle() {
            return Err(exceptions::InvalidUnitsFunction(func, &Unit::get_text_of(unit)));
        }
    }
    Ok(())
}

pub fn check_function_units(func: &str, unit: &OptUnit) -> CResult<()> {
    if unit.is_some() {
        return Err(exceptions::InvalidUnitsFunction(func, &Unit::get_text_of(unit)));
    }
    Ok(())
}

pub fn get_root(root: &Value) -> CResult<i32> {
    if root.units().is_some() {
        return Err(exceptions::RootUnitless());
    }
    if !root.is_real() {
        return Err(exceptions::RootComplex());
    }
    let n = root.re() as i32;
    if n < 2 || n as f64 != root.re() {
        return Err(exceptions::RootInteger());
    }
    Ok(n)
}

pub fn if_function(condition: &Value, value_if_true: &Value, value_if_false: &Value) -> CResult<Value> {
    Ok(if condition.re().abs() < LOGICAL_ZERO {
        value_if_false.clone()
    } else {
        value_if_true.clone()
    })
}

thread_local! {
    static FACTORIAL: Vec<f64> = {
        let mut f = vec![0f64; 171];
        f[0] = 1.0;
        for i in 1..171 {
            f[i] = f[i - 1] * i as f64;
        }
        f
    };
}

pub fn fact(value: f64) -> CResult<f64> {
    if !(0.0..=170.0).contains(&value) {
        return Err(exceptions::FactorialArgumentOutOfRange());
    }
    let i = value as i32;
    if i as f64 != value {
        return Err(exceptions::FactorialArgumentPositiveInteger());
    }
    Ok(FACTORIAL.with(|f| f[i as usize]))
}

pub fn min(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    for v in &values[1..] {
        let b = v.re() * Unit::convert(&u, v.units(), ',')?;
        if b < result {
            result = b;
        }
    }
    Ok(Value::Real(RealValue::new(result, u)))
}

pub fn max(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    for v in &values[1..] {
        let b = v.re() * Unit::convert(&u, v.units(), ',')?;
        if b > result {
            result = b;
        }
    }
    Ok(Value::Real(RealValue::new(result, u)))
}

pub fn switch(v: &[Value]) -> Value {
    let len = v.len();
    let mut i = 0;
    while i + 1 < len {
        if v[i].re().abs() >= LOGICAL_ZERO {
            return v[i + 1].clone();
        }
        i += 2;
    }
    if len % 2 != 0 {
        return v[len - 1].clone();
    }
    Value::nan()
}

fn is_normal(d: f64) -> bool {
    d.is_normal()
}

pub fn take(values: &[Value]) -> CResult<Value> {
    let x = &values[0];
    let y = &values[1..];
    let d = x.re().round();
    if !is_normal(d) || d < DELTA_MINUS || d > y.len() as f64 * DELTA_PLUS {
        return Ok(Value::nan());
    }
    Ok(y[d as usize - 1].clone())
}

pub fn line(values: &[Value]) -> CResult<Value> {
    let x = &values[0];
    let y = &values[1..];
    let d = x.re();
    if !is_normal(d) || d < DELTA_MINUS || d > y.len() as f64 * DELTA_PLUS {
        return Ok(Value::nan());
    }
    let i = d.floor() as usize;
    let y1 = &y[i - 1];
    if i as f64 == d || d >= y.len() as f64 {
        return Ok(y1.clone());
    }
    let diff = Value::s_sub(&y[i], y1)?;
    Value::s_add(y1, &Value::s_mul_f(&diff, d - i as f64))
}

pub fn spline(values: &[Value]) -> CResult<Value> {
    let x = &values[0];
    let y = &values[1..];
    let mut d = x.re();
    if !is_normal(d) || d < DELTA_MINUS || d > y.len() as f64 * DELTA_PLUS {
        return Ok(Value::nan());
    }
    let i = d.floor() as i64 - 1;
    let iu = i as usize;
    let v = &y[iu];
    if i as f64 == d || d >= y.len() as f64 {
        return Ok(v.clone());
    }
    let u = v.units().clone();
    let y0 = v.re();
    let v1 = &y[iu + 1];
    let y1 = v1.re() * Unit::convert(&u, v1.units(), ',')?;
    let mut dy = y1 - y0;
    if dy.is_nan() {
        return Ok(Value::nan());
    }
    let mut a = dy;
    let mut b = dy;
    dy = math_sign(dy)? as f64;
    if i > 0 {
        let v2 = &y[iu - 1];
        let y2 = v2.re() * Unit::convert(&u, v2.units(), ',')?;
        a = (y1 - y2) * if math_sign(y0 - y2)? as f64 == dy { 0.5 } else { 0.25 };
    }
    if i < y.len() as i64 - 2 {
        let v2 = &y[iu + 2];
        let y2 = v2.re() * Unit::convert(&u, v2.units(), ',')?;
        b = (y2 - y0) * if math_sign(y2 - y1)? as f64 == dy { 0.5 } else { 0.25 };
    }
    if i == 0 {
        a += (a - b) / 2.0;
    }
    if i == y.len() as i64 - 2 {
        b += (b - a) / 2.0;
    }
    let t = d - i as f64 - 1.0;
    d = y0 + ((y1 - y0) * (3.0 - 2.0 * t) * t + ((a + b) * t - a) * (t - 1.0)) * t;
    Ok(Value::Real(RealValue::new(d, u)))
}

pub fn and(v: &[Value]) -> Value {
    for x in v {
        if x.re().abs() < LOGICAL_ZERO {
            return Value::Real(RealValue::ZERO);
        }
    }
    Value::Real(RealValue::ONE)
}

pub fn or(v: &[Value]) -> Value {
    for x in v {
        if x.re().abs() >= LOGICAL_ZERO {
            return Value::Real(RealValue::ONE);
        }
    }
    Value::Real(RealValue::ZERO)
}

pub fn xor(v: &[Value]) -> Value {
    let mut b = v[0].re().abs() >= LOGICAL_ZERO;
    for x in &v[1..] {
        b = b != (x.re().abs() >= LOGICAL_ZERO);
    }
    Value::Real(if b { RealValue::ONE } else { RealValue::ZERO })
}

pub fn mandelbrot_set(a: &Value, b: &Value) -> CResult<RealValue> {
    Ok(RealValue::new(
        mandelbrot(a.re(), b.re() * Unit::convert(a.units(), b.units(), ',')?),
        a.units().clone(),
    ))
}

fn mandelbrot(x: f64, y: f64) -> f64 {
    let log2_inv = 1.0 / 2f64.ln();
    if x > -1.25 && x < 0.375 {
        if x < -0.75 {
            if y > -0.25 && y < 0.25 {
                let x1 = x + 1.0;
                let y2 = y * y;
                let x2 = x1 * x1;
                if x2 + y2 <= 0.0625 {
                    return f64::NAN;
                }
            }
        } else if y > -0.65 && y < 0.65 {
            let x1 = x - 0.25;
            let y2 = y * y;
            let q = x1 * x1 + y2;
            if q * (q + x1) <= 0.25 * y2 {
                return f64::NAN;
            }
        }
    }
    let (mut re, mut im) = (x, y);
    for i in 0..=1000 {
        let re_sq = re * re;
        let im_sq = im * im;
        if re_sq + im_sq > 4.0 {
            let log_zn = (im * im + re * re).ln() / 2.0;
            let nu = (log_zn * log2_inv).ln() * log2_inv;
            return (1.01 - (i as f64 + 1.0 - nu).powf(0.001)) * 1000.0;
        }
        im = 2.0 * re * im + y;
        re = re_sq - im_sq + x;
    }
    f64::NAN
}

pub fn as_long(d: f64) -> CResult<i64> {
    let a = d.abs();
    if a > i64::MAX as f64 || a != a.trunc() {
        return Err(exceptions::BothValuesInteger());
    }
    Ok(a as i64)
}

pub fn gcd_l(mut a: i64, mut b: i64) -> i64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let mut k = 0;
    while ((a | b) & 1) == 0 {
        a >>= 1;
        b >>= 1;
        k += 1;
    }
    while (a & 1) == 0 {
        a >>= 1;
    }
    loop {
        while (b & 1) == 0 {
            b >>= 1;
        }
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        b -= a;
        if b == 0 {
            break;
        }
    }
    a << k
}

pub fn gcd(v: &[Value]) -> CResult<Value> {
    let mut a = as_long(v[0].re())?;
    let u = v[0].units().clone();
    for x in &v[1..] {
        let b = as_long(x.re() * Unit::convert(&u, x.units(), ',')?)?;
        a = gcd_l(a, b);
    }
    Ok(Value::Real(RealValue::num(a as f64)))
}

pub fn lcm(v: &[Value]) -> CResult<Value> {
    let mut a = as_long(v[0].re())?;
    let u = v[0].units().clone();
    for x in &v[1..] {
        let b = as_long(x.re() * Unit::convert(&u, x.units(), ',')?)?;
        if a == 0 && b == 0 {
            return Ok(Value::nan());
        }
        a = a.wrapping_mul(b) / gcd_l(a, b);
    }
    Ok(Value::Real(RealValue::num(a as f64)))
}

thread_local! {
    static START: f64 = crate::clock::now_ms();
}

/// Seconds since the first call (`Stopwatch` based `timer()` in C#).
pub fn timer() -> f64 {
    START.with(|s| (crate::clock::now_ms() - s) / 1000.0)
}

pub fn complex_from_real(r: RealValue) -> ComplexValue {
    ComplexValue::from_real(&r)
}

// ----- Typed delegates used by vectors and matrices -----------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RealOpKind {
    Operator,
    Function2,
}

/// `Calculator.Operator<RealValue>` as returned by `GetOperator` and `GetFunction2`.
#[derive(Clone, Copy)]
pub struct RealOp<'a> {
    calc: &'a Calculator,
    index: i64,
    kind: RealOpKind,
}

/// `Calculator.Function<RealValue>` as returned by `GetFunction`.
#[derive(Clone, Copy)]
pub struct RealFn<'a> {
    calc: &'a Calculator,
    index: i64,
}

/// The explicit `(RealValue)ComplexValue` conversion applied by the complex calculator's
/// typed delegates.
fn explicit_real(v: Value) -> CResult<RealValue> {
    match v {
        Value::Real(r) => Ok(r),
        Value::Complex(c) => {
            if !c.is_real() {
                return Err(exceptions::MustBeReal(exceptions::Items::Value));
            }
            Ok(RealValue::new(c.a, c.units))
        }
        _ => Err(MathParserException::unexpected("Specified cast is not valid.")),
    }
}

fn scalar_real(v: Value) -> CResult<RealValue> {
    match v {
        Value::Real(r) => Ok(r),
        other => explicit_real(other),
    }
}

impl<'a> RealOp<'a> {
    pub fn apply(&self, a: &RealValue, b: &RealValue) -> CResult<RealValue> {
        if self.calc.is_complex {
            // ComplexCalculator.GetOperator and GetFunction2 both index _operators.
            let r = complex_calculator::evaluate_operator(
                self.index,
                &ComplexValue::from_real(a),
                &ComplexValue::from_real(b),
            )?;
            return explicit_real(r);
        }
        match self.kind {
            RealOpKind::Operator => {
                if self.index == 0 {
                    real_calculator::pow(a, b)
                } else {
                    scalar_real(real_calculator::evaluate_operator(self.index, a, b)?)
                }
            }
            RealOpKind::Function2 => {
                if self.index == ROOT_FUNCTION_INDEX {
                    real_calculator::root(a, b, false)
                } else {
                    scalar_real(real_calculator::evaluate_function2(self.calc, self.index, a, b)?)
                }
            }
        }
    }
}

impl<'a> RealFn<'a> {
    pub fn apply(&self, a: &RealValue) -> CResult<RealValue> {
        if self.calc.is_complex {
            let r = complex_calculator::evaluate_function(self.calc, self.index, &ComplexValue::from_real(a))?;
            return explicit_real(r);
        }
        match self.index {
            SQR_FUNCTION_INDEX | SQRT_FUNCTION_INDEX => Ok(real_calculator::sqrt(a, false)),
            CBRT_FUNCTION_INDEX => Ok(real_calculator::cbrt(a, false)),
            i => scalar_real(real_calculator::evaluate_function(self.calc, i, a)?),
        }
    }
}

/// `Calculator.IsZeroPreservingFunction(index)`: f(0) == 0 for the real calculator.
pub fn is_zero_preserving_function(index: i64) -> bool {
    if index < 0 {
        return index == -1;
    }
    thread_local! {
        static ZERO_PRESERVING: Vec<bool> = {
            let calc = Calculator::new(false);
            (0..FUNCTION_NAMES.len() as i64)
                .map(|i| match calc.get_function(i).apply(&RealValue::ZERO) {
                    Ok(v) => v.equals(&RealValue::ZERO),
                    Err(_) => false,
                })
                .collect()
        };
    }
    ZERO_PRESERVING.with(|z| z.get(index as usize).copied().unwrap_or(false))
}

impl Calculator {
    /// `GetOperator(index)`
    pub fn get_operator(&self, index: i64) -> RealOp<'_> {
        RealOp {
            calc: self,
            index,
            kind: RealOpKind::Operator,
        }
    }

    /// `GetFunction2(index)`
    pub fn get_function2(&self, index: i64) -> RealOp<'_> {
        RealOp {
            calc: self,
            index,
            kind: RealOpKind::Function2,
        }
    }

    /// `GetFunction(index)`
    pub fn get_function(&self, index: i64) -> RealFn<'_> {
        RealFn { calc: self, index }
    }
}
