//! Port of Calculator/RealCalculator.cs.
use super::*;

fn rv(v: RealValue) -> CResult<Value> {
    Ok(Value::Real(v))
}

/// `_operators[index](a, b)`
pub fn evaluate_operator(index: i64, a: &RealValue, b: &RealValue) -> CResult<Value> {
    match index {
        0 => rv(unit_pow(a, b)?),
        1 => rv(RealValue::divide(a, b)),
        2 => rv(RealValue::int_div(a, b)),
        3 => rv(RealValue::rem(a, b)?),
        4 => rv(RealValue::multiply(a, b)),
        5 => rv(RealValue::sub(a, b)?),
        6 => rv(RealValue::add(a, b)?),
        7 => rv(RealValue::lt_op(a, b)?),
        8 => rv(RealValue::gt_op(a, b)?),
        9 => rv(RealValue::le_op(a, b)?),
        10 => rv(RealValue::ge_op(a, b)?),
        11 => rv(RealValue::eq_op(a, b)?),
        12 => rv(RealValue::ne_op(a, b)?),
        13 => rv(RealValue::and_op(a, b)),
        14 => rv(RealValue::or_op(a, b)),
        15 => rv(RealValue::xor_op(a, b)),
        _ => Err(index_out_of_range()),
    }
}

pub fn evaluate_function(calc: &Calculator, index: i64, a: &RealValue) -> CResult<Value> {
    let v = match index {
        0 => sin(calc, a)?,
        1 => cos(calc, a)?,
        2 => trig(calc, a, "tan", f64::tan)?,
        3 => trig(calc, a, "csc", |x| 1.0 / x.sin())?,
        4 => trig(calc, a, "sec", |x| 1.0 / x.cos())?,
        5 => trig(calc, a, "cot", |x| 1.0 / x.tan())?,
        6 => inv_trig(calc, a, "asin", f64::asin)?,
        7 => inv_trig(calc, a, "acos", f64::acos)?,
        8 => inv_trig(calc, a, "atan", f64::atan)?,
        9 => {
            check_trig_function_units("acsc", &a.units)?;
            if a.d == 0.0 {
                calc.to_angle_units_real(f64::INFINITY)?
            } else {
                calc.to_angle_units_real((1.0 / a.d).asin())?
            }
        }
        10 => {
            check_trig_function_units("asec", &a.units)?;
            if a.d == 0.0 {
                calc.to_angle_units_real(f64::INFINITY)?
            } else {
                calc.to_angle_units_real((1.0 / a.d).acos())?
            }
        }
        11 => inv_trig(calc, a, "acot", |x| (1.0 / x).atan())?,
        12 => hyp(a, "sinh", f64::sinh)?,
        13 => hyp(a, "cosh", f64::cosh)?,
        14 => hyp(a, "tanh", f64::tanh)?,
        15 => hyp(a, "csch", |x| 1.0 / x.sinh())?,
        16 => hyp(a, "sech", |x| 1.0 / x.cosh())?,
        17 => hyp(a, "coth", |x| 1.0 / x.tanh())?,
        18 => hyp(a, "asinh", f64::asinh)?,
        19 => hyp(a, "acosh", dotnet_acosh)?,
        20 => hyp(a, "atanh", dotnet_atanh)?,
        21 => hyp(a, "acsch", |x| (1.0 / x).asinh())?,
        22 => hyp(a, "asech", |x| dotnet_acosh(1.0 / x))?,
        23 => hyp(a, "acoth", |x| dotnet_atanh(1.0 / x))?,
        24 => hyp(a, "log", f64::log10)?,
        25 => hyp(a, "ln", f64::ln)?,
        26 => hyp(a, "log_2", f64::log2)?,
        27 => hyp(a, "exp", f64::exp)?,
        28 => abs(a),
        29 => sign(a)?,
        30 | 31 => sqrt(a, a.is_unit),
        32 => cbrt(a, a.is_unit),
        33 => RealValue::new(a.d.round(), a.units.clone()),
        34 => RealValue::new(a.d.floor(), a.units.clone()),
        35 => RealValue::new(a.d.ceil(), a.units.clone()),
        36 => RealValue::new(a.d.trunc(), a.units.clone()),
        37 => a.clone(),
        38 => RealValue::ZERO,
        39 => calc.to_angle_units_real(if a.d >= 0.0 { 0.0 } else { std::f64::consts::PI })?,
        40 => a.clone(),
        41 => RealValue::new(crate::base_types::complex::Complex::real_random(a.d), a.units.clone()),
        42 => fact_r(a)?,
        43 => a.neg(),
        44 => not(a),
        45 => RealValue::new(timer(), Unit::try_get("s", false)),
        _ => return Err(index_out_of_range()),
    };
    Ok(Value::Real(v))
}

/// .NET Math.Acosh returns NaN for x < 1 (Rust too); kept as a named hook.
fn dotnet_acosh(x: f64) -> f64 {
    x.acosh()
}

/// .NET Math.Atanh: NaN outside [-1, 1], +-Infinity at +-1.
fn dotnet_atanh(x: f64) -> f64 {
    x.atanh()
}

pub fn evaluate_function2(calc: &Calculator, index: i64, a: &RealValue, b: &RealValue) -> CResult<Value> {
    let v = match index {
        0 => calc.to_angle_units_real((b.d * Unit::convert(&a.units, &b.units, ',')?).atan2(a.d))?,
        1 => root(a, b, a.is_unit)?,
        2 => RealValue::rem(a, b)?,
        3 => mandelbrot_set(&Value::Real(a.clone()), &Value::Real(b.clone()))?,
        _ => return Err(index_out_of_range()),
    };
    Ok(Value::Real(v))
}

pub fn evaluate_multi_function(index: i64, values: &[Value]) -> CResult<Value> {
    match index {
        0 => min(values),
        1 => max(values),
        2 => sum(values),
        3 => sum_sq(values),
        4 => srss(values),
        5 => average(values),
        6 => product(values),
        7 => mean(values),
        8 => Ok(switch(values)),
        9 => Ok(and(values)),
        10 => Ok(or(values)),
        11 => Ok(xor(values)),
        12 => gcd(values),
        13 => lcm(values),
        _ => Err(index_out_of_range()),
    }
}

fn fact_r(a: &RealValue) -> CResult<RealValue> {
    if a.units.is_some() {
        return Err(exceptions::FactorialArgumentUnitless());
    }
    Ok(RealValue::num(fact(a.d)?))
}

pub fn abs(value: &RealValue) -> RealValue {
    RealValue::new(value.d.abs(), value.units.clone())
}

fn sign(value: &RealValue) -> CResult<RealValue> {
    if value.d.is_nan() {
        return Ok(RealValue::NAN);
    }
    Ok(RealValue::num(math_sign(value.d)? as f64))
}

fn from_angle_units(calc: &Calculator, value: &RealValue) -> CResult<f64> {
    Ok(value.d * calc.from_angle_units_factor(&value.units)?)
}

fn sin(calc: &Calculator, value: &RealValue) -> CResult<RealValue> {
    check_trig_function_units("sin", &value.units)?;
    Ok(RealValue::num(crate::base_types::complex::Complex::real_sin(
        from_angle_units(calc, value)?,
    )?))
}

fn cos(calc: &Calculator, value: &RealValue) -> CResult<RealValue> {
    check_trig_function_units("cos", &value.units)?;
    Ok(RealValue::num(crate::base_types::complex::Complex::real_cos(
        from_angle_units(calc, value)?,
    )?))
}

fn trig(calc: &Calculator, value: &RealValue, name: &str, f: fn(f64) -> f64) -> CResult<RealValue> {
    check_trig_function_units(name, &value.units)?;
    Ok(RealValue::num(f(from_angle_units(calc, value)?)))
}

fn inv_trig(calc: &Calculator, value: &RealValue, name: &str, f: fn(f64) -> f64) -> CResult<RealValue> {
    check_trig_function_units(name, &value.units)?;
    calc.to_angle_units_real(f(value.d))
}

fn hyp(value: &RealValue, name: &str, f: fn(f64) -> f64) -> CResult<RealValue> {
    check_trig_function_units(name, &value.units)?;
    Ok(RealValue::num(f(value.d)))
}

fn pow_impl(value: &RealValue, power: &RealValue, is_unit: bool) -> CResult<RealValue> {
    if let Some(u) = &value.units {
        if u.is_dimensionless() {
            return Ok(RealValue::num((value.d * u.get_dimensionless_factor()).powf(power.d)));
        }
    }
    Ok(RealValue::with_flag(
        value.d.powf(power.d),
        Unit::pow_scalar(&value.units, &Value::Real(power.clone()), is_unit)?,
        is_unit,
    ))
}

/// `RealCalculator.Pow(value, power)` (no unit text update)
pub fn pow(value: &RealValue, power: &RealValue) -> CResult<RealValue> {
    pow_impl(value, power, false)
}

fn unit_pow(value: &RealValue, power: &RealValue) -> CResult<RealValue> {
    pow_impl(value, power, value.is_unit)
}

pub fn sqrt(value: &RealValue, is_unit: bool) -> RealValue {
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return RealValue::num((value.d * uu.get_dimensionless_factor()).sqrt());
        }
    }
    let result = value.d.sqrt();
    match u {
        None => RealValue::num(result),
        Some(uu) => RealValue::with_flag(result, Unit::root(uu, 2, is_unit), is_unit),
    }
}

pub fn cbrt(value: &RealValue, is_unit: bool) -> RealValue {
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return RealValue::num((value.d * uu.get_dimensionless_factor()).cbrt());
        }
    }
    let result = value.d.cbrt();
    match u {
        None => RealValue::num(result),
        Some(uu) => RealValue::with_flag(result, Unit::root(uu, 3, is_unit), is_unit),
    }
}

pub fn root(value: &RealValue, root: &RealValue, is_unit: bool) -> CResult<RealValue> {
    let n = get_root(&Value::Real(root.clone()))?;
    let u = &value.units;
    let d = match u {
        Some(uu) if uu.is_dimensionless() => value.d * uu.get_dimensionless_factor(),
        _ => value.d,
    };
    let result = if n % 2 != 0 && d < 0.0 {
        -(-d).powf(1.0 / n as f64)
    } else {
        d.powf(1.0 / n as f64)
    };
    Ok(match u {
        None => RealValue::num(result),
        Some(uu) => RealValue::with_flag(result, Unit::root(uu, n, is_unit), is_unit),
    })
}

fn sum(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    for v in &values[1..] {
        result += v.re() * Unit::convert(&u, v.units(), ',')?;
    }
    Ok(Value::Real(RealValue::new(result, u)))
}

fn sum_sq(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    result *= result;
    for v in &values[1..] {
        let b = v.re() * Unit::convert(&u, v.units(), ',')?;
        result += b * b;
    }
    let u2 = u.as_ref().and_then(|x| x.pow(2.0));
    Ok(Value::Real(RealValue::new(result, u2)))
}

pub fn srss(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    result *= result;
    for v in &values[1..] {
        let b = v.re() * Unit::convert(&u, v.units(), ',')?;
        result += b * b;
    }
    Ok(Value::Real(RealValue::new(result.sqrt(), u)))
}

fn average(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let u = values[0].units().clone();
    for v in &values[1..] {
        result += v.re() * Unit::convert(&u, v.units(), ',')?;
    }
    Ok(Value::Real(RealValue::new(result / values.len() as f64, u)))
}

fn product(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let mut u = values[0].units().clone();
    for v in &values[1..] {
        let (nu, b) = Unit::multiply(&u, v.units(), false);
        u = nu;
        result *= v.re() * b;
    }
    Ok(Value::Real(RealValue::new(result, u)))
}

fn mean(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].re();
    let mut u = values[0].units().clone();
    for v in &values[1..] {
        let (nu, b) = Unit::multiply(&u, v.units(), false);
        u = nu;
        result *= v.re() * b;
    }
    result = result.powf(1.0 / values.len() as f64);
    match u {
        None => Ok(Value::Real(RealValue::num(result))),
        Some(uu) => Ok(Value::Real(RealValue::new(
            result,
            Unit::root(&uu, values.len() as i32, false),
        ))),
    }
}

fn not(value: &RealValue) -> RealValue {
    if value.d.abs() < LOGICAL_ZERO {
        RealValue::ONE
    } else {
        RealValue::ZERO
    }
}
