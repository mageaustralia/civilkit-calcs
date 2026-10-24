//! Port of Calculator/VectorCalculator.cs.
//!
//! The C# class holds a reference to the scalar calculator; here the functions take it as
//! an argument.
use super::{index_out_of_range, Calculator, POWER_OPERATOR_INDEX};
use crate::base_types::real_value::RealValue;
use crate::base_types::value::Value;
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::vector::{hp_vector, large_vector, Relation, Vector};

pub const FUNCTION_NAMES: [(&str, i64); 16] = [
    ("vector", 0),
    ("len", 1),
    ("size", 2),
    ("sort", 3),
    ("rsort", 4),
    ("order", 5),
    ("revorder", 6),
    ("reverse", 7),
    ("norm", 8),
    ("norm_2", 8),
    ("norm_e", 8),
    ("norm_1", 9),
    ("norm_i", 10),
    ("unit", 11),
    ("vector_hp", 12),
    ("cumsum", 13),
];

pub const FUNCTION2_NAMES: [(&str, i64); 9] = [
    ("resize", 0),
    ("fill", 1),
    ("first", 2),
    ("last", 3),
    ("extract", 4),
    ("dot", 5),
    ("cross", 6),
    ("norm_p", 7),
    ("convolve", 8),
];

pub const FUNCTION3_NAMES: [(&str, i64); 22] = [
    ("slice", 0),
    ("range", 1),
    ("range_hp", 2),
    ("search", 3),
    ("count", 4),
    ("find", 5),
    ("find_eq", 5),
    ("find_ne", 6),
    ("find_lt", 7),
    ("find_le", 8),
    ("find_gt", 9),
    ("find_ge", 10),
    ("lookup", 11),
    ("lookup_eq", 11),
    ("lookup_ne", 12),
    ("lookup_lt", 13),
    ("lookup_le", 14),
    ("lookup_gt", 15),
    ("lookup_ge", 16),
    ("linspace", 17),
    ("polyfit", 18),
    ("interp", 19),
];

pub const MULTI_FUNCTION_NAMES: [(&str, i64); 1] = [("join", 0)];

/// `ValueResultFunctions`
const VALUE_RESULT_FUNCTIONS: [&str; 10] = [
    "count", "dot", "len", "last", "norm", "norm1", "normp", "normi", "search", "interp",
];

pub(crate) fn lookup_ci(table: &[(&str, i64)], name: &str) -> Option<i64> {
    table
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name) || n.to_lowercase() == name.to_lowercase())
        .map(|(_, i)| *i)
}

pub fn function_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION_NAMES, name)
}
pub fn function2_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION2_NAMES, name)
}
pub fn function3_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION3_NAMES, name)
}
pub fn multi_function_index(name: &str) -> Option<i64> {
    lookup_ci(&MULTI_FUNCTION_NAMES, name)
}

/// `IsVectorResultFunction(string name)`
pub fn is_vector_result_function(name: &str) -> bool {
    !VALUE_RESULT_FUNCTIONS.iter().any(|n| n.eq_ignore_ascii_case(name))
}

/// `EvaluateVectorFunction(index, a)`
pub fn evaluate_vector_function(index: i64, a: &Value) -> CResult<Value> {
    let v = || a.as_vector(Items::Argument);
    Ok(match index {
        0 => {
            let n = a.as_int(Items::Argument)? as usize;
            if n > 100 {
                Value::Vector(large_vector::new_length(n)?)
            } else {
                Value::Vector(Vector::new_size(n)?)
            }
        }
        1 => Value::Real(RealValue::num(v()?.len() as f64)),
        2 => Value::Real(RealValue::num(v()?.size() as f64)),
        3 => Value::Vector(v()?.sort(false)?),
        4 => Value::Vector(v()?.sort(true)?),
        5 => Value::Vector(v()?.order(false)?),
        6 => Value::Vector(v()?.order(true)?),
        7 => Value::Vector(v()?.reverse()?),
        8 => Value::Real(v()?.norm()?),
        9 => Value::Real(v()?.l1_norm()?),
        10 => Value::Real(v()?.inf_norm()?),
        11 => Value::Vector(v()?.normalize()?),
        12 => Value::Vector(hp_vector::new_length(a.as_int(Items::Argument)? as usize, None)?),
        13 => super::statistics::cumsum(a)?,
        _ => return Err(index_out_of_range()),
    })
}

/// `EvaluateVectorFunction2(index, a, b)`
pub fn evaluate_vector_function2(index: i64, a: &Value, b: &Value) -> CResult<Value> {
    Ok(match index {
        0 => {
            let vec = a.as_vector(Items::Argument)?;
            vec.resize(b.as_int(Items::Argument)? as usize)?;
            vec.change();
            Value::Vector(vec)
        }
        1 => {
            let vec = a.as_vector(Items::Argument)?;
            vec.fill(&b.as_real_checked(Items::Argument)?)?;
            vec.change();
            Value::Vector(vec)
        }
        2 => Value::Vector(
            a.as_vector(Items::Argument)?
                .first(b.as_int(Items::Argument)? as usize)?,
        ),
        3 => Value::Vector(
            a.as_vector(Items::Argument)?
                .last(b.as_int(Items::Argument)? as usize)?,
        ),
        4 => Value::Vector(a.as_vector(Items::Argument)?.extract(&b.as_vector(Items::Argument)?)?),
        5 => {
            let va = a.as_vector(Items::Argument)?;
            let vb = b.as_vector(Items::Argument)?;
            if va.is_hp() && vb.is_hp() {
                Value::Real(hp_vector::dot_product(&va, &vb)?)
            } else {
                Value::Real(Vector::dot_product(&va, &vb)?)
            }
        }
        6 => {
            let va = a.as_vector(Items::Argument)?;
            let vb = b.as_vector(Items::Argument)?;
            if va.is_hp() && vb.is_hp() {
                Value::Vector(hp_vector::cross_product(&va, &vb)?)
            } else {
                Value::Vector(Vector::cross_product(&va, &vb)?)
            }
        }
        7 => Value::Real(a.as_vector(Items::Argument)?.lp_norm(b.as_int(Items::Argument)?)?),
        8 => super::statistics::convolve(a, b)?,
        _ => return Err(index_out_of_range()),
    })
}

/// `EvaluateVectorFunction3(index, a, b, c)`
pub fn evaluate_vector_function3(index: i64, a: &Value, b: &Value, c: &Value) -> CResult<Value> {
    let find = |rel: Relation| -> CResult<Value> {
        Ok(Value::Vector(a.as_vector(Items::Argument)?.find_all(
            &b.as_real_checked(Items::Argument)?,
            c.as_int(Items::Argument)? as usize,
            rel,
        )?))
    };
    let lookup = |rel: Relation| -> CResult<Value> {
        let vec_x = a.as_vector(Items::Argument)?;
        let vec_y = b.as_vector(Items::Argument)?;
        let val = c.as_real_checked(Items::Argument)?;
        if vec_x.is_hp() {
            return Ok(Value::Vector(hp_vector::lookup(&vec_x, &vec_y, &val, rel)?));
        }
        Ok(Value::Vector(vec_x.lookup(&vec_y, &val, rel)?))
    };
    match index {
        0 => Ok(Value::Vector(a.as_vector(Items::Argument)?.slice(
            b.as_int(Items::Argument)? as i64,
            c.as_int(Items::Argument)? as i64,
        )?)),
        1 => Ok(Value::Vector(Vector::range(
            &a.as_real_checked(Items::Argument)?,
            &b.as_real_checked(Items::Argument)?,
            &c.as_real_checked(Items::Argument)?,
        )?)),
        2 => Ok(Value::Vector(hp_vector::range(
            &a.as_real_checked(Items::Argument)?,
            &b.as_real_checked(Items::Argument)?,
            &c.as_real_checked(Items::Argument)?,
        )?)),
        3 => Ok(Value::Real(a.as_vector(Items::Argument)?.search(
            &b.as_real_checked(Items::Argument)?,
            c.as_int(Items::Argument)? as usize,
        )?)),
        4 => Ok(Value::Real(a.as_vector(Items::Argument)?.count(
            &b.as_real_checked(Items::Argument)?,
            c.as_int(Items::Argument)? as usize,
        )?)),
        5 => find(Relation::Equal),
        6 => find(Relation::NotEqual),
        7 => find(Relation::LessThan),
        8 => find(Relation::LessOrEqual),
        9 => find(Relation::GreaterThan),
        10 => find(Relation::GreaterOrEqual),
        11 => lookup(Relation::Equal),
        12 => lookup(Relation::NotEqual),
        13 => lookup(Relation::LessThan),
        14 => lookup(Relation::LessOrEqual),
        15 => lookup(Relation::GreaterThan),
        16 => lookup(Relation::GreaterOrEqual),
        17 => super::statistics::linspace(a, b, c),
        18 => super::statistics::polyfit(a, b, c),
        19 => super::statistics::interp(a, b, c),
        _ => Err(index_out_of_range()),
    }
}

/// `EvaluateVectorMultiFunction(index, IValue[] a)`
pub fn evaluate_vector_multi_function(index: i64, a: &[Value]) -> CResult<Value> {
    match index {
        0 => Ok(Value::Vector(Vector::join(a)?)),
        _ => Err(index_out_of_range()),
    }
}

/// `EvaluateOperator(index, Vector a, in RealValue b)`
pub fn evaluate_operator_vr(calc: &Calculator, index: i64, a: &Vector, b: &RealValue) -> CResult<Vector> {
    if a.is_hp() {
        hp_vector::evaluate_operator_vr(calc.get_operator(index), a, b, index)
    } else {
        Vector::evaluate_operator_vr(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateOperator(index, in RealValue a, Vector b)`
pub fn evaluate_operator_rv(calc: &Calculator, index: i64, a: &RealValue, b: &Vector) -> CResult<Vector> {
    if b.is_hp() {
        hp_vector::evaluate_operator_rv(calc.get_operator(index), a, b, index)
    } else {
        Vector::evaluate_operator_rv(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateOperator(index, Vector a, Vector b)`
pub fn evaluate_operator(calc: &Calculator, index: i64, a: &Vector, b: &Vector) -> CResult<Vector> {
    if a.is_hp() && b.is_hp() && (index != POWER_OPERATOR_INDEX || a.hp_units().is_none()) {
        hp_vector::evaluate_operator(calc.get_operator(index), a, b, index)
    } else {
        Vector::evaluate_operator(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateFunction(index, Vector a)`
pub fn evaluate_function(calc: &Calculator, index: i64, a: &Vector) -> CResult<Vector> {
    if a.is_hp() {
        hp_vector::evaluate_function(calc.get_function(index), a)
    } else {
        Vector::evaluate_function(calc.get_function(index), a)
    }
}

/// `EvaluateFunction2(index, Vector a, in RealValue b)`
pub fn evaluate_function2_vr(calc: &Calculator, index: i64, a: &Vector, b: &RealValue) -> CResult<Vector> {
    if a.is_hp() {
        hp_vector::evaluate_operator_vr(calc.get_function2(index), a, b, -index - 1)
    } else {
        Vector::evaluate_operator_vr(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateFunction2(index, in RealValue a, Vector b)`
pub fn evaluate_function2_rv(calc: &Calculator, index: i64, a: &RealValue, b: &Vector) -> CResult<Vector> {
    if b.is_hp() {
        hp_vector::evaluate_operator_rv(calc.get_function2(index), a, b, -index - 1)
    } else {
        Vector::evaluate_operator_rv(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateFunction2(index, Vector a, Vector b)`
pub fn evaluate_function2(calc: &Calculator, index: i64, a: &Vector, b: &Vector) -> CResult<Vector> {
    if a.is_hp() && b.is_hp() {
        hp_vector::evaluate_operator(calc.get_function2(index), a, b, -index - 1)
    } else {
        Vector::evaluate_operator(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateMultiFunction(index, Vector a)`: min, max, sum, ... of the elements.
pub fn evaluate_multi_function(index: i64, a: &Vector) -> CResult<Value> {
    if index >= super::statistics::MULTI_BASE {
        return super::statistics::evaluate(index, &[Value::Vector(a.clone())]);
    }
    Ok(Value::Real(match index {
        0 => a.min()?,
        1 => a.max()?,
        2 => a.sum()?,
        3 => a.sum_sq()?,
        4 => a.srss()?,
        5 => a.average()?,
        6 => a.product()?,
        7 => a.mean()?,
        8 => a.try_get(0)?,
        9 => a.and_all(),
        10 => a.or_all(),
        11 => a.xor_all(),
        12 => a.gcd()?,
        13 => a.lcm()?,
        _ => return Err(index_out_of_range()),
    }))
}

/// `EvaluateInterpolation(index, RealValue a, Vector b)`
pub fn evaluate_interpolation(index: i64, a: &RealValue, b: &Vector) -> CResult<Value> {
    Ok(Value::Real(match index {
        0 => b.take(a)?,
        1 => b.line(a)?,
        2 => b.spline(a)?,
        _ => return Err(index_out_of_range()),
    }))
}

/// `GetElement(IValue vector, IValue ii)`
pub fn get_element(vector: &Value, ii: &Value) -> CResult<Value> {
    let vec = vector.as_vector(Items::IndexTarget)?;
    let val = ii.as_value(Items::Index)?;
    let i = crate::vector::cs_int(val.re());
    if i < 1 || i > vec.len() as i64 {
        return Err(exceptions::IndexOutOfRange(&i.to_string()));
    }
    Ok(Value::Real(vec.get(i as usize - 1)))
}

/// `SetElement(IValue vector, IValue ii, IValue value)`
pub fn set_element(vector: &Value, ii: &Value, value: &Value) -> CResult<Value> {
    let vec = vector.as_vector(Items::IndexTarget)?;
    let val = ii.as_value(Items::Index)?;
    let i = crate::vector::cs_int(val.re());
    if i < 1 || i > vec.len() as i64 {
        return Err(exceptions::IndexOutOfRange(&i.to_string()));
    }
    let real = value.as_real_checked(Items::Value)?;
    vec.set(i as usize - 1, real)?;
    vec.change();
    Ok(value.clone())
}
