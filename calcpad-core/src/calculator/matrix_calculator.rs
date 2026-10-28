//! Port of Calculator/MatrixCalculator.cs.
use super::vector_calculator::lookup_ci;
use super::{general_eigen_calc, index_out_of_range, Calculator, POWER_OPERATOR_INDEX};
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::value::Value;
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::matrix::{
    column_matrix, diagonal_matrix, hp_matrix, lower_triangular_matrix, symmetric_matrix, upper_triangular_matrix,
    Matrix, MatrixType,
};
use crate::vector::{cs_int, hp_vector, Relation, Vector};

pub const FUNCTION_NAMES: [(&str, i64); 44] = [
    ("identity", 0),
    ("utriang", 1),
    ("ltriang", 2),
    ("symmetric", 3),
    ("vec2diag", 4),
    ("diag2vec", 5),
    ("vec2row", 6),
    ("vec2col", 7),
    ("n_rows", 8),
    ("n_cols", 9),
    ("mnorm_1", 10),
    ("mnorm_2", 11),
    ("mnorm", 11),
    ("mnorm_e", 12),
    ("mnorm_i", 13),
    ("cond_1", 14),
    ("cond_2", 15),
    ("cond", 15),
    ("cond_e", 16),
    ("cond_i", 17),
    ("det", 18),
    ("rank", 19),
    ("transp", 20),
    ("trace", 21),
    ("inverse", 22),
    ("adj", 23),
    ("cofactor", 24),
    ("qr", 25),
    ("svd", 26),
    ("cholesky", 27),
    ("identity_hp", 28),
    ("utriang_hp", 29),
    ("ltriang_hp", 30),
    ("symmetric_hp", 31),
    ("hp", 32),
    ("ishp", 33),
    ("getunits", 34),
    ("clrunits", 35),
    ("fft", 36),
    ("ift", 37),
    ("lu", 38),
    ("polyroots", 39),
    ("ceigenvals", 40),
    ("ceigenvecs", 41),
];

pub const FUNCTION2_NAMES: [(&str, i64); 36] = [
    ("matrix", 0),
    ("diagonal", 1),
    ("column", 2),
    ("row", 3),
    ("col", 4),
    ("extract_rows", 5),
    ("extract_cols", 6),
    ("mfill", 7),
    ("sort_cols", 8),
    ("rsort_cols", 9),
    ("sort_rows", 10),
    ("rsort_rows", 11),
    ("order_cols", 12),
    ("revorder_cols", 13),
    ("order_rows", 14),
    ("revorder_rows", 15),
    ("mcount", 16),
    ("mfind", 17),
    ("mfind_eq", 18),
    ("mfind_ne", 19),
    ("mfind_lt", 20),
    ("mfind_le", 21),
    ("mfind_gt", 22),
    ("mfind_ge", 23),
    ("lsolve", 24),
    ("clsolve", 25),
    ("msolve", 26),
    ("cmsolve", 27),
    ("hprod", 28),
    ("fprod", 29),
    ("kprod", 30),
    ("matrix_hp", 31),
    ("diagonal_hp", 32),
    ("column_hp", 33),
    ("setunits", 34),
    ("matmul", 35),
];

pub const LU_INDEX: i64 = 38;

pub const ITERATIVE_FUNCTION_NAMES: [(&str, i64); 5] = [
    ("slsolve", 0),
    ("smsolve", 1),
    ("eigenvals", 2),
    ("eigenvecs", 3),
    ("eigen", 4),
];

/// `IsLastParameterOptional(int i)`
pub fn is_last_parameter_optional(i: i64) -> bool {
    matches!(i, 2..=4)
}

pub const FUNCTION3_NAMES: [(&str, i64); 3] = [("fill_row", 0), ("fill_col", 1), ("mresize", 2)];

pub const FUNCTION4_NAMES: [(&str, i64); 17] = [
    ("msearch", 0),
    ("hlookup", 1),
    ("hlookup_eq", 2),
    ("hlookup_ne", 3),
    ("hlookup_lt", 4),
    ("hlookup_le", 5),
    ("hlookup_gt", 6),
    ("hlookup_ge", 7),
    ("vlookup", 8),
    ("vlookup_eq", 9),
    ("vlookup_ne", 10),
    ("vlookup_lt", 11),
    ("vlookup_le", 12),
    ("vlookup_gt", 13),
    ("vlookup_ge", 14),
    ("copy", 15),
    ("add", 16),
];

pub const FUNCTION5_NAMES: [(&str, i64); 1] = [("submatrix", 0)];

pub const MULTI_FUNCTION_NAMES: [(&str, i64); 4] = [("join_cols", 0), ("join_rows", 1), ("augment", 2), ("stack", 3)];

pub fn function_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION_NAMES, name)
}
pub fn function2_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION2_NAMES, name)
}
pub fn function3_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION3_NAMES, name)
}
pub fn function4_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION4_NAMES, name)
}
pub fn function5_index(name: &str) -> Option<i64> {
    lookup_ci(&FUNCTION5_NAMES, name)
}
pub fn iterative_function_index(name: &str) -> Option<i64> {
    lookup_ci(&ITERATIVE_FUNCTION_NAMES, name)
}
pub fn multi_function_index(name: &str) -> Option<i64> {
    lookup_ci(&MULTI_FUNCTION_NAMES, name)
}

fn mat(a: &Value) -> CResult<Matrix> {
    a.as_matrix(Items::Argument)
}

fn int(a: &Value) -> CResult<usize> {
    Ok(a.as_int(Items::Argument)? as usize)
}

fn real(a: &Value) -> CResult<RealValue> {
    a.as_real_checked(Items::Argument)
}

/// `EvaluateMatrixFunction(index, a)`
pub fn evaluate_matrix_function(index: i64, a: &Value) -> CResult<Value> {
    let m = || mat(a);
    Ok(match index {
        0 => Value::Matrix(diagonal_matrix::with_value(int(a)?, &RealValue::ONE)?),
        1 => Value::Matrix(upper_triangular_matrix::new_checked(int(a)?)?),
        2 => Value::Matrix(lower_triangular_matrix::new_checked(int(a)?)?),
        3 => Value::Matrix(symmetric_matrix::new_checked(int(a)?)?),
        4 => {
            let vec = a.as_vector(Items::Argument)?;
            if vec.is_hp() {
                Value::Matrix(hp_matrix::diagonal_from_vector(&vec)?)
            } else {
                Value::Matrix(diagonal_matrix::from_vector(&vec)?)
            }
        }
        5 => Value::Vector(m()?.diagonal()?),
        6 => {
            let vec = a.as_vector(Items::Argument)?;
            if vec.is_hp() {
                Value::Matrix(hp_matrix::from_row_vector(&vec)?)
            } else {
                Value::Matrix(Matrix::from_row_vector(&vec)?)
            }
        }
        7 => {
            let vec = a.as_vector(Items::Argument)?;
            if vec.is_hp() {
                Value::Matrix(hp_matrix::hp_column_from_vector(&vec)?)
            } else {
                Value::Matrix(column_matrix::from_vector(&vec)?)
            }
        }
        8 => Value::Real(RealValue::num(m()?.row_count() as f64)),
        9 => Value::Real(RealValue::num(m()?.col_count() as f64)),
        10 => Value::Real(m()?.l1_norm()?),
        11 => Value::Real(m()?.l2_norm()?),
        12 => Value::Real(m()?.frob_norm()?),
        13 => Value::Real(m()?.inf_norm()?),
        14 => Value::Real(m()?.cond1()?),
        15 => Value::Real(m()?.cond2()?),
        16 => Value::Real(m()?.cond_frob()?),
        17 => Value::Real(m()?.cond_inf()?),
        18 => Value::Real(m()?.determinant()?),
        19 => Value::Real(m()?.rank()?),
        20 => Value::Matrix(m()?.transpose()?),
        21 => Value::Real(m()?.trace()?),
        22 => Value::Matrix(m()?.invert()?),
        23 => Value::Matrix(m()?.adjoint()?),
        24 => Value::Matrix(m()?.cofactor()?),
        25 => Value::Matrix(m()?.qr_decomposition()?),
        26 => Value::Matrix(m()?.svd_decomposition()?),
        27 => {
            let matrix = m()?;
            match matrix.mtype() {
                MatrixType::Symmetric | MatrixType::Diagonal => Value::Matrix(matrix.cholesky_decomposition()?),
                _ => return Err(exceptions::MatrixMustBeSymmetric()),
            }
        }
        28 => Value::Matrix(hp_matrix::diagonal_with_value(int(a)?, &RealValue::ONE)?),
        29 => Value::Matrix(hp_matrix::new_upper(int(a)?, None)?),
        30 => Value::Matrix(hp_matrix::new_lower(int(a)?, None)?),
        31 => Value::Matrix(hp_matrix::new_symmetric(int(a)?, None)?),
        32 => hp(a)?,
        33 => {
            let is_hp = match a {
                Value::Vector(v) => v.is_hp(),
                Value::Matrix(m) => m.is_hp(),
                _ => false,
            };
            Value::Real(if is_hp { RealValue::ONE } else { RealValue::ZERO })
        }
        34 => get_units(a),
        35 => clr_units(a)?,
        36 => Value::Matrix(hp_matrix::fft(&a.as_matrix_hp(Items::Argument)?, false)?),
        37 => Value::Matrix(hp_matrix::fft(&a.as_matrix_hp(Items::Argument)?, true)?),
        39 => poly_roots(&a.as_vector(Items::Argument)?)?,
        40 => general_eigen_calc::complex_eigen(&a.as_matrix(Items::Argument)?, false)?,
        41 => general_eigen_calc::complex_eigen(&a.as_matrix(Items::Argument)?, true)?,
        _ => return Err(index_out_of_range()),
    })
}

/// `Hp(in IValue v)`
fn hp(v: &Value) -> CResult<Value> {
    Ok(match v {
        Value::Vector(vec) if vec.is_hp() => v.clone(),
        Value::Matrix(m) if m.is_hp() => v.clone(),
        Value::Real(rv) => Value::Vector(hp_vector::from_doubles(vec![rv.d], rv.units.clone())?),
        Value::Vector(vec) => Value::Vector(hp_vector::from_vector(vec, None)?),
        Value::Matrix(m) => Value::Matrix(hp_matrix::from_matrix(m)?),
        _ => v.clone(),
    })
}

/// `GetUnits(in IValue v)`
fn get_units(v: &Value) -> Value {
    let units = match v {
        Value::Real(rv) => rv.units.clone(),
        Value::Complex(cv) => cv.units.clone(),
        Value::Vector(vec) if vec.is_hp() => vec.hp_units(),
        Value::Matrix(m) if m.is_hp() => hp_matrix::units(m),
        // C# reads vec[1] and matrix[1, 1] here (a second element), as written upstream.
        Value::Vector(vec) => {
            if vec.len() > 0 {
                vec.get(1).units
            } else {
                None
            }
        }
        Value::Matrix(m) => {
            if m.row_count() > 0 && m.col_count() > 0 {
                m.get(1, 1).units
            } else {
                None
            }
        }
    };
    Value::Real(RealValue::unit(units))
}

/// `SetUnits(in IValue v, in IValue unitsValue)`
fn set_units(v: &Value, units_value: &Value) -> CResult<Value> {
    let rvu = match units_value {
        Value::Real(r) if r.d == 1.0 && r.is_unit => r.clone(),
        _ => return Err(exceptions::InvalidUnits(&units_value.to_cs_string())),
    };
    let units = match &rvu.units {
        Some(u) => Some(u.clone()),
        None => return Err(exceptions::InvalidUnits(&units_value.to_cs_string())),
    };
    Ok(match v {
        Value::Real(rv) => Value::Real(RealValue::new(rv.d, units)),
        Value::Complex(cv) => Value::Complex(ComplexValue::from_c(cv.complex(), units)),
        Value::Vector(vec) if vec.is_hp() => {
            vec.set_hp_units(units);
            v.clone()
        }
        Value::Matrix(m) if m.is_hp() => {
            hp_matrix::set_units_prop(m, units);
            v.clone()
        }
        Value::Vector(vec) => {
            vec.set_units(&units)?;
            v.clone()
        }
        Value::Matrix(m) => {
            m.set_units(&units)?;
            v.clone()
        }
    })
}

/// `ClrUnits(in IValue v)`
fn clr_units(v: &Value) -> CResult<Value> {
    Ok(match v {
        Value::Real(rv) => Value::Real(RealValue::new(rv.d, None)),
        Value::Complex(cv) => Value::Complex(ComplexValue::from_c(cv.complex(), None)),
        Value::Vector(vec) if vec.is_hp() => {
            vec.set_hp_units(None);
            v.clone()
        }
        Value::Matrix(m) if m.is_hp() => {
            hp_matrix::set_units_prop(m, None);
            v.clone()
        }
        Value::Vector(vec) => {
            vec.set_units(&None)?;
            v.clone()
        }
        Value::Matrix(m) => {
            m.set_units(&None)?;
            v.clone()
        }
    })
}

/// `LUDecomposition(in IValue M, HpVector indexes)`: stores the permutation in `indexes`.
pub fn lu_decomposition(m: &Value, indexes: &Vector) -> CResult<Value> {
    let (lu, v) = mat(m)?.lu_decomposition()?;
    hp_vector::set_values(indexes, &v);
    Ok(Value::Matrix(lu))
}

/// `EvaluateMatrixFunction2(index, a, b)`
pub fn evaluate_matrix_function2(index: i64, a: &Value, b: &Value) -> CResult<Value> {
    let find = |rel: Relation| -> CResult<Value> { Ok(Value::Matrix(mat(a)?.find_all(&real(b)?, rel)?)) };
    Ok(match index {
        0 => Value::Matrix(Matrix::new(int(a)?, int(b)?)?),
        1 => Value::Matrix(diagonal_matrix::with_value(int(a)?, &real(b)?)?),
        2 => Value::Matrix(column_matrix::with_value(int(a)?, &real(b)?)?),
        3 => Value::Vector(mat(a)?.row(int(b)? as i64)?),
        4 => Value::Vector(mat(a)?.col(int(b)? as i64)?),
        5 => Value::Matrix(mat(a)?.extract_rows(&b.as_vector(Items::Argument)?)?),
        6 => Value::Matrix(mat(a)?.extract_cols(&b.as_vector(Items::Argument)?)?),
        7 => {
            let matrix = mat(a)?;
            matrix.fill(&real(b)?)?;
            matrix.change();
            Value::Matrix(matrix)
        }
        8 => Value::Matrix(mat(a)?.sort_cols(int(b)? as i64, false)?),
        9 => Value::Matrix(mat(a)?.sort_cols(int(b)? as i64, true)?),
        10 => Value::Matrix(mat(a)?.sort_rows(int(b)? as i64, false)?),
        11 => Value::Matrix(mat(a)?.sort_rows(int(b)? as i64, true)?),
        12 => Value::Vector(mat(a)?.order_cols(int(b)? as i64, false)?),
        13 => Value::Vector(mat(a)?.order_cols(int(b)? as i64, true)?),
        14 => Value::Vector(mat(a)?.order_rows(int(b)? as i64, false)?),
        15 => Value::Vector(mat(a)?.order_rows(int(b)? as i64, true)?),
        16 => Value::Real(mat(a)?.count(&real(b)?)?),
        17 | 18 => find(Relation::Equal)?,
        19 => find(Relation::NotEqual)?,
        20 => find(Relation::LessThan)?,
        21 => find(Relation::LessOrEqual)?,
        22 => find(Relation::GreaterThan)?,
        23 => find(Relation::GreaterOrEqual)?,
        24 => {
            let ma = mat(a)?;
            let vb = b.as_vector(Items::Argument)?;
            if ma.row_count() != vb.len() {
                return Err(exceptions::MatrixDimensions());
            }
            if ma.is_hp() && !vb.is_hp() {
                return Err(exceptions::MustBeHpVector(Items::Argument));
            }
            Value::Vector(ma.lsolve(&vb)?)
        }
        25 => {
            let ma = mat(a)?;
            let vb = b.as_vector(Items::Argument)?;
            if ma.row_count() != vb.len() {
                return Err(exceptions::MatrixDimensions());
            }
            match ma.mtype() {
                MatrixType::Symmetric | MatrixType::Diagonal => {
                    if ma.is_hp() && !vb.is_hp() {
                        return Err(exceptions::MustBeHpVector(Items::Argument));
                    }
                    if ma.mtype() == MatrixType::Symmetric {
                        Value::Vector(ma.cl_solve(&vb)?)
                    } else {
                        Value::Vector(ma.lsolve(&vb)?)
                    }
                }
                _ => return Err(exceptions::MatrixMustBeSymmetric()),
            }
        }
        26 => {
            let ma = mat(a)?;
            let mb = mat(b)?;
            if ma.row_count() != mb.row_count() {
                return Err(exceptions::MatrixDimensions());
            }
            if ma.is_hp() && !mb.is_hp() {
                return Err(exceptions::MustBeHpMatrix(Items::Argument));
            }
            Value::Matrix(ma.msolve(&mb)?)
        }
        27 => {
            let ma = mat(a)?;
            let mb = mat(b)?;
            if ma.row_count() != mb.row_count() {
                return Err(exceptions::MatrixDimensions());
            }
            match ma.mtype() {
                MatrixType::Symmetric | MatrixType::Diagonal => {
                    if ma.is_hp() && !mb.is_hp() {
                        return Err(exceptions::MustBeHpMatrix(Items::Argument));
                    }
                    if ma.mtype() == MatrixType::Symmetric {
                        Value::Matrix(ma.cm_solve(&mb)?)
                    } else {
                        Value::Matrix(ma.msolve(&mb)?)
                    }
                }
                _ => return Err(exceptions::MatrixMustBeSymmetric()),
            }
        }
        28 => {
            let (ma, mb) = (mat(a)?, mat(b)?);
            if ma.is_hp() && mb.is_hp() {
                Value::Matrix(hp_matrix::hadamard(&ma, &mb)?)
            } else {
                Value::Matrix(Matrix::hadamard(&ma, &mb)?)
            }
        }
        29 => {
            let (ma, mb) = (mat(a)?, mat(b)?);
            if ma.is_hp() && mb.is_hp() {
                Value::Real(hp_matrix::frobenius(&ma, &mb)?)
            } else {
                Value::Real(Matrix::frobenius(&ma, &mb)?)
            }
        }
        30 => {
            let (ma, mb) = (mat(a)?, mat(b)?);
            if ma.is_hp() && mb.is_hp() {
                Value::Matrix(hp_matrix::kronecker(&ma, &mb)?)
            } else {
                Value::Matrix(Matrix::kronecker(&ma, &mb)?)
            }
        }
        31 => Value::Matrix(hp_matrix::new(int(a)?, int(b)?, None)?),
        32 => Value::Matrix(hp_matrix::diagonal_with_value(int(a)?, &real(b)?)?),
        33 => Value::Matrix(hp_matrix::column_with_value(int(a)?, &real(b)?)?),
        34 => set_units(a, b)?,
        35 => {
            let ma = a.as_matrix_hp(Items::Argument)?;
            let mb = b.as_matrix_hp(Items::Argument)?;
            if ma.col_count() != mb.row_count() {
                return Err(exceptions::MatrixDimensions());
            }
            if ma.row_count() != ma.col_count() || mb.row_count() != mb.col_count() {
                return Err(exceptions::MatrixNotSquare());
            }
            Value::Matrix(hp_matrix::matmul(&ma, &mb)?)
        }
        _ => return Err(index_out_of_range()),
    })
}

/// `EvaluateMatrixIterativeFunction(index, a, b, tol)`
pub fn evaluate_matrix_iterative_function(index: i64, a: &Value, b: &Value, tol: f64) -> CResult<Value> {
    match index {
        0 => {
            let ma = a.as_matrix_hp(Items::Argument)?;
            let vb = b.as_vector_hp(Items::Argument)?;
            if ma.row_count() != vb.len() {
                return Err(exceptions::MatrixDimensions());
            }
            match ma.mtype() {
                MatrixType::Symmetric => Ok(Value::Vector(hp_matrix::numerics::sl_solve(&ma, &vb, tol)?)),
                MatrixType::Diagonal => Ok(Value::Vector(ma.lsolve(&vb)?)),
                _ => Err(exceptions::MatrixMustBeSymmetric()),
            }
        }
        1 => {
            let ma = a.as_matrix_hp(Items::Argument)?;
            let mb = b.as_matrix_hp(Items::Argument)?;
            if ma.row_count() != mb.row_count() {
                return Err(exceptions::MatrixDimensions());
            }
            match ma.mtype() {
                MatrixType::Symmetric => Ok(Value::Matrix(hp_matrix::numerics::sm_solve(&ma, &mb, tol)?)),
                MatrixType::Diagonal => Ok(Value::Matrix(ma.msolve(&mb)?)),
                _ => Err(exceptions::MatrixMustBeSymmetric()),
            }
        }
        2..=4 => {
            let n = cs_int(b.as_real_checked(Items::Argument)?.d);
            let ma = match a {
                Value::Matrix(m) if matches!(m.mtype(), MatrixType::Symmetric | MatrixType::Diagonal) => m.clone(),
                Value::Matrix(m) => match general_eigen_calc::as_symmetric(m)? {
                    Some(s) => s,
                    None => return general_eigen_calc::real_eigen(index, m, n),
                },
                _ => return Err(exceptions::MatrixMustBeSymmetric()),
            };
            match index {
                2 => Ok(Value::Vector(ma.eigen_values(n, tol)?)),
                3 => Ok(Value::Matrix(ma.eigen_vectors(n, tol)?)),
                _ => Ok(Value::Matrix(ma.eigen(n, tol)?)),
            }
        }
        _ => Err(index_out_of_range()),
    }
}

/// `EvaluateMatrixFunction3(index, a, b, c)`
pub fn evaluate_matrix_function3(index: i64, a: &Value, b: &Value, c: &Value) -> CResult<Value> {
    let m = mat(a)?;
    match index {
        0 => {
            m.fill_row(int(b)? as i64, &real(c)?)?;
            m.change();
        }
        1 => {
            m.fill_col(int(b)? as i64, &real(c)?)?;
            m.change();
        }
        2 => {
            m.resize(int(b)?, int(c)?)?;
            m.change();
        }
        _ => return Err(index_out_of_range()),
    }
    Ok(Value::Matrix(m))
}

/// `EvaluateMatrixFunction4(index, a, b, c, d)`
pub fn evaluate_matrix_function4(index: i64, a: &Value, b: &Value, c: &Value, d: &Value) -> CResult<Value> {
    let m = mat(a)?;
    let h = |rel: Relation| -> CResult<Value> {
        Ok(Value::Vector(m.h_lookup(
            &real(b)?,
            int(c)? as i64,
            int(d)? as i64,
            rel,
        )?))
    };
    let v = |rel: Relation| -> CResult<Value> {
        Ok(Value::Vector(m.v_lookup(
            &real(b)?,
            int(c)? as i64,
            int(d)? as i64,
            rel,
        )?))
    };
    match index {
        0 => Ok(Value::Vector(m.search(&real(b)?, int(c)?, int(d)?)?)),
        1 | 2 => h(Relation::Equal),
        3 => h(Relation::NotEqual),
        4 => h(Relation::LessThan),
        5 => h(Relation::LessOrEqual),
        6 => h(Relation::GreaterThan),
        7 => h(Relation::GreaterOrEqual),
        8 | 9 => v(Relation::Equal),
        10 => v(Relation::NotEqual),
        11 => v(Relation::LessThan),
        12 => v(Relation::LessOrEqual),
        13 => v(Relation::GreaterThan),
        14 => v(Relation::GreaterOrEqual),
        15 | 16 => {
            let target = mat(b)?;
            let ii = int(c)? as i64;
            let jj = int(d)? as i64;
            if index == 15 {
                m.copy_to(&target, ii, jj)?;
            } else {
                m.add_to(&target, ii, jj)?;
            }
            target.change();
            Ok(Value::Matrix(target))
        }
        _ => Err(index_out_of_range()),
    }
}

/// `EvaluateMatrixFunction5(index, a, b, c, d, e)`
pub fn evaluate_matrix_function5(index: i64, a: &Value, b: &Value, c: &Value, d: &Value, e: &Value) -> CResult<Value> {
    let m = mat(a)?;
    match index {
        0 => Ok(Value::Matrix(m.submatrix(
            int(b)? as i64,
            int(c)? as i64,
            int(d)? as i64,
            int(e)? as i64,
        )?)),
        _ => Err(index_out_of_range()),
    }
}

/// `JoinCols(IValue[] v)`
pub fn join_cols(v: &[Value]) -> CResult<Matrix> {
    let mut m = 0;
    let mut is_hp = true;
    let mut vecs = Vec::with_capacity(v.len());
    for item in v {
        match item {
            Value::Vector(vec) => {
                if !vec.is_hp() {
                    is_hp = false;
                }
                m = m.max(vec.len());
                vecs.push(vec.clone());
            }
            _ => return Err(exceptions::MustBeVector(Items::Argument)),
        }
    }
    if is_hp {
        return hp_matrix::create_from_cols(&vecs, m);
    }
    Matrix::create_from_cols(&vecs, m)
}

/// `JoinRows(IValue[] v)`
pub fn join_rows(v: &[Value]) -> CResult<Matrix> {
    let mut n = 0;
    let mut is_hp = true;
    let mut vecs = Vec::with_capacity(v.len());
    for item in v {
        match item {
            Value::Vector(vec) => {
                if !vec.is_hp() {
                    is_hp = false;
                }
                n = n.max(vec.len());
                vecs.push(vec.clone());
            }
            _ => return Err(exceptions::MustBeVector(Items::Argument)),
        }
    }
    if is_hp {
        return hp_matrix::create_from_rows(&vecs, n);
    }
    Matrix::create_from_rows(&vecs, n)
}

fn are_all_hp(values: &[Value]) -> bool {
    values.iter().all(|v| match v {
        Value::Vector(vec) => vec.is_hp(),
        Value::Matrix(m) => m.is_hp(),
        _ => false,
    })
}

/// `EvaluateMatrixMultiFunction(index, IValue[] a)`
pub fn evaluate_matrix_multi_function(index: i64, a: &[Value]) -> CResult<Value> {
    Ok(Value::Matrix(match index {
        0 => join_cols(a)?,
        1 => join_rows(a)?,
        2 | 3 => {
            if are_all_hp(a) {
                let ms = a
                    .iter()
                    .map(|v| v.as_matrix_hp(Items::Argument))
                    .collect::<CResult<Vec<_>>>()?;
                if index == 2 {
                    hp_matrix::augment(&ms)?
                } else {
                    hp_matrix::stack(&ms)?
                }
            } else {
                let ms = a.iter().map(mat).collect::<CResult<Vec<_>>>()?;
                if index == 2 {
                    Matrix::augment(&ms)?
                } else {
                    Matrix::stack(&ms)?
                }
            }
        }
        _ => return Err(index_out_of_range()),
    }))
}

/// `EvaluateOperator(index, Matrix a, in RealValue b)`
pub fn evaluate_operator_mr(calc: &Calculator, index: i64, a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    if a.is_hp() {
        hp_matrix::evaluate_operator_mr(calc.get_operator(index), a, b, index)
    } else {
        Matrix::evaluate_operator_mr(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateOperator(index, in RealValue a, Matrix b)`
pub fn evaluate_operator_rm(calc: &Calculator, index: i64, a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    if b.is_hp() {
        hp_matrix::evaluate_operator_rm(calc.get_operator(index), a, b, index)
    } else {
        Matrix::evaluate_operator_rm(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateOperator(index, Matrix a, Matrix b)`
pub fn evaluate_operator(calc: &Calculator, index: i64, a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if a.is_hp() && b.is_hp() && (index != POWER_OPERATOR_INDEX || hp_matrix::units(a).is_none()) {
        hp_matrix::evaluate_operator(calc.get_operator(index), a, b, index)
    } else {
        Matrix::evaluate_operator(calc.get_operator(index), a, b, index)
    }
}

/// `EvaluateFunction(index, Matrix a)`
pub fn evaluate_function(calc: &Calculator, index: i64, a: &Matrix) -> CResult<Matrix> {
    if a.is_hp() {
        hp_matrix::evaluate_function(calc.get_function(index), a, index)
    } else {
        Matrix::evaluate_function(calc.get_function(index), a, index)
    }
}

/// `EvaluateFunction2(index, Matrix a, Matrix b)`
pub fn evaluate_function2(calc: &Calculator, index: i64, a: &Matrix, b: &Matrix) -> CResult<Matrix> {
    if a.is_hp() && b.is_hp() {
        hp_matrix::evaluate_operator(calc.get_function2(index), a, b, -index - 1)
    } else {
        Matrix::evaluate_operator(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateFunction2(index, Matrix a, in RealValue b)`
pub fn evaluate_function2_mr(calc: &Calculator, index: i64, a: &Matrix, b: &RealValue) -> CResult<Matrix> {
    if a.is_hp() {
        hp_matrix::evaluate_operator_mr(calc.get_function2(index), a, b, -index - 1)
    } else {
        Matrix::evaluate_operator_mr(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateFunction2(index, in RealValue a, Matrix b)`
pub fn evaluate_function2_rm(calc: &Calculator, index: i64, a: &RealValue, b: &Matrix) -> CResult<Matrix> {
    if b.is_hp() {
        hp_matrix::evaluate_operator_rm(calc.get_function2(index), a, b, -index - 1)
    } else {
        Matrix::evaluate_operator_rm(calc.get_function2(index), a, b, -index - 1)
    }
}

/// `EvaluateMultiFunction(index, Matrix a)`
pub fn evaluate_multi_function(index: i64, a: &Matrix) -> CResult<Value> {
    if index >= super::statistics::MULTI_BASE {
        return super::statistics::evaluate(index, &[Value::Matrix(a.clone())]);
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
        8 => a.try_get(0, 0)?,
        9 => a.and_all(),
        10 => a.or_all(),
        11 => a.xor_all()?,
        12 => a.gcd()?,
        13 => a.lcm()?,
        _ => return Err(index_out_of_range()),
    }))
}

/// `EvaluateInterpolation(index, RealValue x, RealValue y, Matrix c)`
pub fn evaluate_interpolation(index: i64, x: &RealValue, y: &RealValue, c: &Matrix) -> CResult<Value> {
    Ok(Value::Real(match index {
        0 => c.take(x, y)?,
        1 => c.line(x, y)?,
        2 => c.spline(x, y)?,
        _ => return Err(index_out_of_range()),
    }))
}

/// `GetElement(IValue matrix, IValue ii, IValue jj)`
pub fn get_element(matrix: &Value, ii: &Value, jj: &Value) -> CResult<Value> {
    let m = matrix.as_matrix(Items::IndexTarget)?;
    let i = cs_int(ii.as_value(Items::Index)?.re());
    let j = cs_int(jj.as_value(Items::Index)?.re());
    if i < 1 || i > m.row_count() as i64 || j < 1 || j > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&format!("{}, {}", i, j)));
    }
    Ok(Value::Real(m.get(i as usize - 1, j as usize - 1)))
}

/// `SetElement(IValue matrix, IValue ii, IValue jj, IValue value)`
pub fn set_element(matrix: &Value, ii: &Value, jj: &Value, value: &Value) -> CResult<Value> {
    let m = matrix.as_matrix(Items::IndexTarget)?;
    let i = cs_int(ii.as_value(Items::Index)?.re());
    let j = cs_int(jj.as_value(Items::Index)?.re());
    if i < 1 || i > m.row_count() as i64 || j < 1 || j > m.col_count() as i64 {
        return Err(exceptions::IndexOutOfRange(&format!("{}, {}", i, j)));
    }
    let real = value.as_real_checked(Items::Value)?;
    m.set(i as usize - 1, j as usize - 1, real.clone())?;
    m.change();
    Ok(Value::Real(real))
}

/// `polyroots(v)`: the real roots of `v.1 + v.2*x + v.3*x^2 + ...`, ascending.
/// Durand-Kerner on all the complex roots, polished with Newton steps; roots
/// whose imaginary part is negligible are kept (vectors hold real values).
fn poly_roots(v: &Vector) -> CResult<Value> {
    let mut c: Vec<f64> = (0..v.len()).map(|i| v.get(i).d).collect();
    while c.last() == Some(&0.0) {
        c.pop();
    }
    if c.len() < 2 || c.iter().any(|x| !x.is_finite()) {
        return Err(exceptions::ArgumentOutOfRange("polyroots"));
    }
    let n = c.len() - 1;
    let lead = c[n];
    let a: Vec<f64> = c.iter().map(|x| x / lead).collect();
    let eval = |z: (f64, f64)| -> (f64, f64) {
        let mut r = (0.0, 0.0);
        for k in (0..=n).rev() {
            r = (r.0 * z.0 - r.1 * z.1 + a[k], r.0 * z.1 + r.1 * z.0);
        }
        r
    };
    let div = |p: (f64, f64), q: (f64, f64)| -> (f64, f64) {
        let d = q.0 * q.0 + q.1 * q.1;
        ((p.0 * q.0 + p.1 * q.1) / d, (p.1 * q.0 - p.0 * q.1) / d)
    };
    let radius = 1.0 + a[..n].iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    let mut z: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let t = 2.0 * std::f64::consts::PI * (k as f64) / (n as f64) + 0.4;
            (radius * 0.5 * t.cos(), radius * 0.5 * t.sin())
        })
        .collect();
    for _ in 0..500 {
        let mut delta = 0.0_f64;
        for i in 0..n {
            let mut den = (1.0, 0.0);
            for j in 0..n {
                if i != j {
                    let d = (z[i].0 - z[j].0, z[i].1 - z[j].1);
                    den = (den.0 * d.0 - den.1 * d.1, den.0 * d.1 + den.1 * d.0);
                }
            }
            if den.0 == 0.0 && den.1 == 0.0 {
                continue;
            }
            let step = div(eval(z[i]), den);
            z[i] = (z[i].0 - step.0, z[i].1 - step.1);
            delta = delta.max(step.0.abs() + step.1.abs());
        }
        if delta < 1e-15 * radius {
            break;
        }
    }
    let tol = 1e-7 * radius;
    let mut roots: Vec<f64> = z.iter().filter(|r| r.1.abs() <= tol).map(|r| r.0).collect();
    roots.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    if roots.is_empty() {
        return Err(exceptions::ArgumentOutOfRange("polyroots"));
    }
    Ok(Value::Vector(Vector::from_values(roots.into_iter().map(RealValue::num).collect())?))
}
