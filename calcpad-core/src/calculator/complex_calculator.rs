//! Port of Calculator/ComplexCalculator.cs.
use super::*;
use crate::base_types::complex::Complex;

fn cv(v: ComplexValue) -> CResult<Value> {
    Ok(Value::Complex(v))
}

/// A RealValue returned where the delegate type is ComplexValue: implicit conversion.
fn rc(v: RealValue) -> ComplexValue {
    ComplexValue::from_real(&v)
}

pub fn evaluate_operator(calc: &Calculator, index: i64, a: &ComplexValue, b: &ComplexValue) -> CResult<Value> {
    match index {
        0 => cv(pow(a, b)?),
        1 => cv(ComplexValue::div(a, b)),
        2 => cv(ComplexValue::int_div(a, b)),
        3 => cv(ComplexValue::rem(a, b)?),
        4 => cv(ComplexValue::mul(a, b)),
        5 => cv(ComplexValue::sub(a, b)?),
        6 => cv(ComplexValue::add(a, b)?),
        7 => cv(ComplexValue::lt_op(a, b)?),
        8 => cv(ComplexValue::gt_op(a, b)?),
        9 => cv(ComplexValue::le_op(a, b)?),
        10 => cv(ComplexValue::ge_op(a, b)?),
        11 => cv(ComplexValue::eq_op(a, b)?),
        12 => cv(ComplexValue::ne_op(a, b)?),
        13 => cv(rc(ComplexValue::and_op(a, b))),
        14 => cv(rc(ComplexValue::or_op(a, b))),
        15 => cv(rc(ComplexValue::xor_op(a, b))),
        16 => cv(phasor(calc, a, b)?),
        _ => Err(index_out_of_range()),
    }
}

pub fn evaluate_function(calc: &Calculator, index: i64, a: &ComplexValue) -> CResult<Value> {
    let c = a.complex();
    let v = match index {
        0 => {
            check_trig_function_units("sin", &a.units)?;
            ComplexValue::c(Complex::sin(from_angle_units(calc, a)?)?)
        }
        1 => {
            check_trig_function_units("cos", &a.units)?;
            ComplexValue::c(Complex::cos(from_angle_units(calc, a)?)?)
        }
        2 => {
            check_trig_function_units("tan", &a.units)?;
            ComplexValue::c(Complex::tan(from_angle_units(calc, a)?))
        }
        3 => {
            check_trig_function_units("csc", &a.units)?;
            ComplexValue::c(Complex::f_div(1.0, Complex::sin(from_angle_units(calc, a)?)?))
        }
        4 => {
            check_trig_function_units("sec", &a.units)?;
            ComplexValue::c(Complex::f_div(1.0, Complex::cos(from_angle_units(calc, a)?)?))
        }
        5 => {
            check_trig_function_units("cot", &a.units)?;
            ComplexValue::c(Complex::cot(from_angle_units(calc, a)?))
        }
        6 => {
            check_trig_function_units("asin", &a.units)?;
            to_angle_units(calc, Complex::asin(c))?
        }
        7 => {
            check_trig_function_units("acos", &a.units)?;
            to_angle_units(calc, Complex::acos(c))?
        }
        8 => {
            check_trig_function_units("atan", &a.units)?;
            to_angle_units(calc, Complex::atan(c))?
        }
        9 => {
            check_trig_function_units("acsc", &a.units)?;
            if a.is_zero() {
                ComplexValue::COMPLEX_INFINITY
            } else {
                to_angle_units(calc, Complex::asin(Complex::f_div(1.0, c)))?
            }
        }
        10 => {
            check_trig_function_units("asec", &a.units)?;
            if a.is_zero() {
                ComplexValue::COMPLEX_INFINITY
            } else {
                to_angle_units(calc, Complex::acos(Complex::f_div(1.0, c)))?
            }
        }
        11 => {
            check_trig_function_units("acot", &a.units)?;
            to_angle_units(calc, Complex::acot(c))?
        }
        12 => {
            check_trig_function_units("sinh", &a.units)?;
            ComplexValue::c(Complex::sinh(c)?)
        }
        13 => {
            check_trig_function_units("cosh", &a.units)?;
            ComplexValue::c(Complex::cosh(c)?)
        }
        14 => {
            check_trig_function_units("tanh", &a.units)?;
            ComplexValue::c(Complex::tanh(c))
        }
        15 => {
            check_trig_function_units("csch", &a.units)?;
            ComplexValue::c(Complex::f_div(1.0, Complex::sinh(c)?))
        }
        16 => {
            check_trig_function_units("sech", &a.units)?;
            ComplexValue::c(Complex::f_div(1.0, Complex::cosh(c)?))
        }
        17 => {
            check_trig_function_units("coth", &a.units)?;
            ComplexValue::c(Complex::coth(c))
        }
        18 => {
            check_trig_function_units("asinh", &a.units)?;
            ComplexValue::c(Complex::asinh(c))
        }
        19 => {
            check_trig_function_units("acosh", &a.units)?;
            ComplexValue::c(Complex::acosh(c))
        }
        20 => {
            check_trig_function_units("atanh", &a.units)?;
            ComplexValue::c(Complex::atanh(c))
        }
        21 => {
            check_trig_function_units("acsch", &a.units)?;
            if a.is_zero() {
                ComplexValue::COMPLEX_INFINITY
            } else {
                ComplexValue::c(Complex::asinh(Complex::f_div(1.0, c)))
            }
        }
        22 => {
            check_trig_function_units("asech", &a.units)?;
            if a.is_zero() {
                ComplexValue::COMPLEX_INFINITY
            } else {
                ComplexValue::c(Complex::acosh(Complex::f_div(1.0, c)))
            }
        }
        23 => {
            check_trig_function_units("acoth", &a.units)?;
            ComplexValue::c(Complex::acoth(c))
        }
        24 => {
            check_trig_function_units("log", &a.units)?;
            ComplexValue::c(Complex::log10(c))
        }
        25 => {
            check_trig_function_units("ln", &a.units)?;
            ComplexValue::c(Complex::log(c))
        }
        26 => {
            check_trig_function_units("log_2", &a.units)?;
            ComplexValue::c(Complex::log2(c))
        }
        27 => {
            check_trig_function_units("exp", &a.units)?;
            ComplexValue::c(Complex::exp(c))
        }
        28 => abs(a),
        29 => ComplexValue::c(Complex::sign(c)?),
        30 | 31 => sqrt(a),
        32 => cbrt(a),
        33 => ComplexValue::from_c(Complex::round(c), a.units.clone()),
        34 => ComplexValue::from_c(Complex::floor(c), a.units.clone()),
        35 => ComplexValue::from_c(Complex::ceiling(c), a.units.clone()),
        36 => ComplexValue::from_c(Complex::truncate(c), a.units.clone()),
        37 => ComplexValue::new(a.a, 0.0, a.units.clone()),
        38 => ComplexValue::new(a.b, 0.0, a.units.clone()),
        39 => to_angle_units(calc, Complex::real(c.phase()))?,
        40 => conjugate(a),
        41 => ComplexValue::from_c(Complex::random(c), a.units.clone()),
        42 => fact_c(a)?,
        43 => a.neg(),
        44 => {
            if a.a.abs() < LOGICAL_ZERO {
                rc(RealValue::ONE)
            } else {
                rc(RealValue::ZERO)
            }
        }
        45 => ComplexValue::new(timer(), 0.0, Unit::try_get("s", false)),
        _ => return Err(index_out_of_range()),
    };
    Ok(Value::Complex(v))
}

pub fn evaluate_function2(calc: &Calculator, index: i64, a: &ComplexValue, b: &ComplexValue) -> CResult<Value> {
    let v = match index {
        0 => to_angle_units(
            calc,
            Complex::atan2(b.complex() * Unit::convert(&a.units, &b.units, ',')?, a.complex()),
        )?,
        1 => root(a, b)?,
        2 => ComplexValue::rem(a, b)?,
        3 => rc(mandelbrot_set(&Value::Complex(a.clone()), &Value::Complex(b.clone()))?),
        _ => return Err(index_out_of_range()),
    };
    Ok(Value::Complex(v))
}

pub fn evaluate_multi_function(index: i64, values: &[Value]) -> CResult<Value> {
    match index {
        0 => {
            if are_all_real(values) {
                min(values)
            } else {
                Ok(Value::Real(RealValue::new(f64::NAN, values[0].units().clone())))
            }
        }
        1 => {
            if are_all_real(values) {
                max(values)
            } else {
                Ok(Value::Real(RealValue::new(f64::NAN, values[0].units().clone())))
            }
        }
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
        12 => {
            if are_all_real(values) {
                gcd(values)
            } else {
                Ok(Value::Real(RealValue::new(f64::NAN, values[0].units().clone())))
            }
        }
        13 => {
            if are_all_real(values) {
                lcm(values)
            } else {
                Ok(Value::Real(RealValue::new(f64::NAN, values[0].units().clone())))
            }
        }
        _ => Err(index_out_of_range()),
    }
}

fn fact_c(value: &ComplexValue) -> CResult<ComplexValue> {
    if !value.is_real() {
        return Err(exceptions::FactorialArgumentComplex());
    }
    if value.units.is_some() {
        return Err(exceptions::FactorialArgumentUnitless());
    }
    Ok(ComplexValue::num(fact(value.a)?))
}

pub fn abs(value: &ComplexValue) -> ComplexValue {
    ComplexValue::from_c(Complex::abs_c(value.complex()), value.units.clone())
}

pub fn conjugate(value: &ComplexValue) -> ComplexValue {
    ComplexValue::from_c(value.complex().conjugate(), value.units.clone())
}

pub fn phasor(calc: &Calculator, a: &ComplexValue, b: &ComplexValue) -> CResult<ComplexValue> {
    check_trig_function_units("phasor", &b.units)?;
    let phi = from_angle_units(calc, b)?.re();
    let c = (Complex::real(phi.cos()) + Complex::IMAGINARY_ONE * phi.sin()) * a.a;
    Ok(ComplexValue::with_flag(c, a.units.clone(), a.is_unit))
}

fn from_angle_units(calc: &Calculator, value: &ComplexValue) -> CResult<Complex> {
    Ok(value.complex() * calc.from_angle_units_factor(&value.units)?)
}

fn to_angle_units(calc: &Calculator, value: Complex) -> CResult<ComplexValue> {
    let d = calc.degrees();
    if !(0..3).contains(&d) {
        return Err(index_out_of_range());
    }
    let d = d as usize;
    Ok(if calc.return_angle_units() {
        ComplexValue::from_c(value * FROM_RAD[d], Some(angle_unit(d)))
    } else {
        ComplexValue::c(value * FROM_RAD[d])
    })
}

pub fn pow(value: &ComplexValue, power: &ComplexValue) -> CResult<ComplexValue> {
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return Ok(ComplexValue::c(Complex::pow(
                value.complex() * uu.get_dimensionless_factor(),
                power.complex(),
            )));
        }
    }
    let result = Complex::pow(value.complex(), power.complex());
    let unit = Unit::pow_scalar(u, &Value::Complex(power.clone()), value.is_unit)?;
    Ok(ComplexValue::with_flag(result, unit, value.is_unit))
}

pub fn sqrt(value: &ComplexValue) -> ComplexValue {
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return ComplexValue::c(Complex::sqrt(value.complex() * uu.get_dimensionless_factor()));
        }
    }
    let result = Complex::sqrt(value.complex());
    match u {
        None => ComplexValue::c(result),
        Some(uu) => ComplexValue::with_flag(result, Unit::root(uu, 2, value.is_unit), value.is_unit),
    }
}

fn cbrt(value: &ComplexValue) -> ComplexValue {
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return ComplexValue::c(Complex::cbrt(value.complex() * uu.get_dimensionless_factor()));
        }
    }
    let result = Complex::cbrt(value.complex());
    match u {
        None => ComplexValue::c(result),
        Some(uu) => ComplexValue::with_flag(result, Unit::root(uu, 3, value.is_unit), value.is_unit),
    }
}

fn root(value: &ComplexValue, root: &ComplexValue) -> CResult<ComplexValue> {
    let n = get_root(&Value::Complex(root.clone()))?;
    let u = &value.units;
    if let Some(uu) = u {
        if uu.is_dimensionless() {
            return Ok(ComplexValue::c(Complex::pow_f(
                value.complex() * uu.get_dimensionless_factor(),
                1.0 / n as f64,
            )));
        }
    }
    let result = Complex::pow_f(value.complex(), 1.0 / n as f64);
    Ok(match u {
        None => ComplexValue::c(result),
        Some(uu) => ComplexValue::with_flag(result, Unit::root(uu, n, value.is_unit), value.is_unit),
    })
}

fn are_all_real(values: &[Value]) -> bool {
    values.iter().all(|v| v.is_real())
}

fn sum(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let u = values[0].units().clone();
    for v in &values[1..] {
        result = result + v.complex() * Unit::convert(&u, v.units(), ',')?;
    }
    Ok(Value::Complex(ComplexValue::from_c(result, u)))
}

fn sum_sq(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let u = values[0].units().clone();
    result = result * result;
    for v in &values[1..] {
        let b = v.complex() * Unit::convert(&u, v.units(), ',')?;
        result = result + b * b;
    }
    let u2 = u.as_ref().and_then(|x| x.pow(2.0));
    Ok(Value::Complex(ComplexValue::from_c(result, u2)))
}

fn srss(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let u = values[0].units().clone();
    result = result * result;
    for v in &values[1..] {
        let b = v.complex() * Unit::convert(&u, v.units(), ',')?;
        result = result + b * b;
    }
    Ok(Value::Complex(ComplexValue::from_c(Complex::sqrt(result), u)))
}

fn average(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let u = values[0].units().clone();
    for v in &values[1..] {
        result = result + v.complex() * Unit::convert(&u, v.units(), ',')?;
    }
    Ok(Value::Complex(ComplexValue::from_c(
        result.div_f(values.len() as f64),
        u,
    )))
}

fn product(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let mut u = values[0].units().clone();
    for v in &values[1..] {
        let (nu, b) = Unit::multiply(&u, v.units(), false);
        u = nu;
        result = result * (v.complex() * b);
    }
    Ok(Value::Complex(ComplexValue::from_c(result, u)))
}

fn mean(values: &[Value]) -> CResult<Value> {
    let mut result = values[0].complex();
    let mut u = values[0].units().clone();
    for v in &values[1..] {
        let (nu, b) = Unit::multiply(&u, v.units(), false);
        u = nu;
        result = result * (v.complex() * b);
    }
    result = Complex::pow_f(result, 1.0 / values.len() as f64);
    match u {
        None => Ok(Value::Complex(ComplexValue::c(result))),
        Some(uu) => Ok(Value::Complex(ComplexValue::from_c(
            result,
            Unit::root(&uu, values.len() as i32, false),
        ))),
    }
}
