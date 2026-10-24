//! Statistics and data functions added to the engine (not in the upstream function set):
//! `median`, `stdev`, `stdevp`, `var`, `varp` (aggregates over any mix of scalars, vectors
//! and matrices), `cumsum`, `convolve`, `linspace`, `polyfit` and `interp` (vectors).
//!
//! Units follow the other aggregates: every value is converted to the unit of the first one.
//! A median or a standard deviation has that unit, a variance its square, a convolution the
//! product of the two units, a polynomial coefficient `c_k` the unit `y / x^k`.
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::exceptions::Items;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Index of the first statistics function in the engine's multi-function table.
pub const MULTI_BASE: i64 = 14;
pub const MEDIAN: i64 = 14;
pub const STDEV: i64 = 15;
pub const STDEVP: i64 = 16;
pub const VAR: i64 = 17;
pub const VARP: i64 = 18;

fn err(message: &str) -> MathParserException {
    MathParserException::new(message.to_string())
}

/// All numbers of the arguments as plain `f64` in the unit of the first value.
fn flatten(values: &[Value]) -> CResult<(Vec<f64>, OptUnit)> {
    let mut list: Vec<RealValue> = Vec::with_capacity(values.len());
    for v in values {
        match v {
            Value::Real(r) => list.push(r.clone()),
            Value::Vector(vec) => list.extend(vec.values()),
            Value::Matrix(m) => list.extend(m.values()),
            Value::Complex(c) => {
                if !c.is_real() {
                    return Err(err("Complex values are not supported by this function."));
                }
                list.push(RealValue::new(c.a, c.units.clone()));
            }
        }
    }
    let Some(first) = list.first() else {
        return Err(err("The function needs at least one value."));
    };
    let u = first.units.clone();
    let mut out = Vec::with_capacity(list.len());
    for v in &list {
        out.push(v.d * Unit::convert(&u, &v.units, ',')?);
    }
    Ok((out, u))
}

fn square_units(u: &OptUnit) -> OptUnit {
    u.as_ref().and_then(|u| u.pow(2.0))
}

/// The statistics multi-functions (`index` is one of the constants above).
pub fn evaluate(index: i64, values: &[Value]) -> CResult<Value> {
    let (mut x, u) = flatten(values)?;
    let n = x.len();
    if x.iter().any(|d| d.is_nan()) {
        let out = if index == VAR || index == VARP {
            square_units(&u)
        } else {
            u
        };
        return Ok(Value::Real(RealValue::new(f64::NAN, out)));
    }
    let result = match index {
        MEDIAN => {
            x.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let d = if n % 2 == 1 {
                x[n / 2]
            } else {
                (x[n / 2 - 1] + x[n / 2]) / 2.0
            };
            RealValue::new(d, u)
        }
        STDEV | STDEVP | VAR | VARP => {
            let sample = index == STDEV || index == VAR;
            if sample && n < 2 {
                return Err(err(
                    "A sample variance or standard deviation needs at least two values.",
                ));
            }
            let mean = x.iter().sum::<f64>() / n as f64;
            let ss: f64 = x.iter().map(|d| (d - mean) * (d - mean)).sum();
            let var = ss / if sample { (n - 1) as f64 } else { n as f64 };
            if index == VAR || index == VARP {
                RealValue::new(var, square_units(&u))
            } else {
                RealValue::new(var.sqrt(), u)
            }
        }
        _ => return Err(crate::calculator::index_out_of_range()),
    };
    Ok(Value::Real(result))
}

/// The numbers of a vector in the unit of its first element.
fn vector_numbers(v: &Vector) -> CResult<(Vec<f64>, OptUnit)> {
    let vals = v.values();
    let u = vals.first().and_then(|f| f.units.clone());
    let mut out = Vec::with_capacity(vals.len());
    for r in &vals {
        out.push(r.d * Unit::convert(&u, &r.units, ',')?);
    }
    Ok((out, u))
}

fn vector_of(d: Vec<f64>, u: &OptUnit) -> CResult<Value> {
    Ok(Value::Vector(Vector::from_values(
        d.into_iter().map(|d| RealValue::new(d, u.clone())).collect(),
    )?))
}

/// `cumsum(v)`: the running sum.
pub fn cumsum(a: &Value) -> CResult<Value> {
    let (x, u) = vector_numbers(&a.as_vector(Items::Argument)?)?;
    let mut acc = 0.0;
    let out: Vec<f64> = x
        .iter()
        .map(|d| {
            acc += d;
            acc
        })
        .collect();
    vector_of(out, &u)
}

/// `convolve(a; b)`: full discrete convolution, `len(a) + len(b) - 1` elements.
pub fn convolve(a: &Value, b: &Value) -> CResult<Value> {
    let (x, ua) = vector_numbers(&a.as_vector(Items::Argument)?)?;
    let (y, ub) = vector_numbers(&b.as_vector(Items::Argument)?)?;
    if x.is_empty() || y.is_empty() {
        return Err(err("convolve needs two vectors that are not empty."));
    }
    let (u, f) = Unit::multiply(&ua, &ub, false);
    let mut out = vec![0.0; x.len() + y.len() - 1];
    for (i, p) in x.iter().enumerate() {
        for (j, q) in y.iter().enumerate() {
            out[i + j] += p * q * f;
        }
    }
    vector_of(out, &u)
}

/// `linspace(a; b; n)`: `n` equally spaced values from `a` to `b` inclusive.
pub fn linspace(a: &Value, b: &Value, n: &Value) -> CResult<Value> {
    let a = a.as_real_checked(Items::Argument)?;
    let b = b.as_real_checked(Items::Argument)?;
    let n = n.as_int(Items::Count)? as usize;
    let end = b.d * Unit::convert(&a.units, &b.units, ',')?;
    let out: Vec<f64> = (0..n)
        .map(|i| {
            if i + 1 == n {
                if n == 1 {
                    a.d
                } else {
                    end
                }
            } else {
                a.d + (end - a.d) * i as f64 / (n - 1) as f64
            }
        })
        .collect();
    vector_of(out, &a.units)
}

/// `interp(x; xs; ys)`: linear interpolation in a table with increasing `xs`.
/// A value outside the table is an error, never an extrapolation.
pub fn interp(x: &Value, xs: &Value, ys: &Value) -> CResult<Value> {
    let x = x.as_real_checked(Items::Argument)?;
    let (tx, ux) = vector_numbers(&xs.as_vector(Items::Argument)?)?;
    let (ty, uy) = vector_numbers(&ys.as_vector(Items::Argument)?)?;
    if tx.len() != ty.len() {
        return Err(err("interp needs two vectors of the same length."));
    }
    if tx.is_empty() {
        return Err(err("interp needs a table with at least one point."));
    }
    if tx.windows(2).any(|w| !(w[1] > w[0])) {
        return Err(err("interp needs the x values in strictly increasing order."));
    }
    let xv = x.d * Unit::convert(&ux, &x.units, ',')?;
    if xv.is_nan() {
        return Ok(Value::Real(RealValue::new(f64::NAN, uy)));
    }
    let last = tx.len() - 1;
    let tol = 1e-12 * tx[last].abs().max(tx[0].abs()).max(1.0);
    if xv < tx[0] - tol || xv > tx[last] + tol {
        return Err(err("interp: the value is outside the table."));
    }
    let k = match tx.iter().position(|&t| t >= xv) {
        Some(k) => k,
        None => last,
    };
    let d = if k == 0 || (tx[k] - xv).abs() <= tol {
        ty[k]
    } else {
        ty[k - 1] + (ty[k] - ty[k - 1]) * (xv - tx[k - 1]) / (tx[k] - tx[k - 1])
    };
    Ok(Value::Real(RealValue::new(d, uy)))
}

/// `polyfit(x; y; n)`: least-squares polynomial of degree `n`; the coefficients are returned
/// in ascending order `[c0; c1; ...; cn]`, `y = c0 + c1*x + ... + cn*x^n`. It solves the
/// normal equations with the engine's `lsolve` (x is scaled by its largest magnitude first
/// and the scale is taken back out of the coefficients).
pub fn polyfit(x: &Value, y: &Value, n: &Value) -> CResult<Value> {
    let (tx, ux) = vector_numbers(&x.as_vector(Items::Argument)?)?;
    let (ty, uy) = vector_numbers(&y.as_vector(Items::Argument)?)?;
    let deg = n.as_int(Items::Argument).map(|d| d as usize).or_else(|_| {
        // degree 0 is allowed (as_int rejects it)
        let r = n.as_real_checked(Items::Argument)?;
        if r.d == 0.0 {
            Ok(0)
        } else {
            Err(err("The degree must be a non-negative integer."))
        }
    })?;
    if tx.len() != ty.len() {
        return Err(err("polyfit needs two vectors of the same length."));
    }
    let m = deg + 1;
    if tx.len() < m {
        return Err(err("polyfit needs at least degree + 1 points."));
    }
    let scale = tx.iter().fold(0.0f64, |s, d| s.max(d.abs()));
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let t: Vec<f64> = tx.iter().map(|d| d / scale).collect();
    // power sums S_k = sum t^k (k = 0..2n) and T_k = sum y t^k (k = 0..n)
    let mut s = vec![0.0; 2 * deg + 1];
    let mut r = vec![0.0; m];
    for (ti, yi) in t.iter().zip(&ty) {
        let mut p = 1.0;
        for (k, sk) in s.iter_mut().enumerate() {
            *sk += p;
            if k < m {
                r[k] += yi * p;
            }
            p *= ti;
        }
    }
    let coeffs: Vec<f64> = if m == 1 {
        vec![r[0] / s[0]]
    } else {
        let a = Matrix::new(m, m)?;
        for i in 0..m {
            for j in 0..m {
                a.set(i, j, RealValue::num(s[i + j]))?;
            }
        }
        let b = Vector::from_values(r.iter().map(|d| RealValue::num(*d)).collect())?;
        let sol = a.lsolve(&b)?;
        sol.values().iter().map(|v| v.d).collect()
    };
    let mut out = Vec::with_capacity(m);
    for (k, c) in coeffs.iter().enumerate() {
        let xk = ux.as_ref().and_then(|u| u.pow(k as f32));
        let (u, f) = Unit::divide(&uy, &xk, false);
        out.push(RealValue::new(c / scale.powi(k as i32) * f, u));
    }
    Ok(Value::Vector(Vector::from_values(out)?))
}
