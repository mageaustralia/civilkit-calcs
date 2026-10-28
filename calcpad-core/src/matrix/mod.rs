//! Port of Matrix/Matrix.cs (with the storage of the derived classes).
//!
//! C# has `Matrix` with the subclasses `ColumnMatrix`, `DiagonalMatrix`,
//! `LowerTriangularMatrix`, `UpperTriangularMatrix` and `SymmetricMatrix`, and `HpMatrix`
//! with the same set of subclasses. The class is identified here by `MatrixType` plus the
//! `hp` flag. Matrices are reference objects, as vectors are (see `vector/mod.rs`).
//!
//! Storage follows C#: `rows` is the `_rows` array of row vectors (LargeVector or
//! HpVector). A full matrix has one row per matrix row; a column or diagonal matrix keeps
//! its values in `rows[0]`; triangular and symmetric matrices store the rows from (or up
//! to) the diagonal.
//!
//! The numerical methods (LU, QR, SVD, solving, inverse, determinant, condition numbers,
//! eigenvalues, Cholesky) are in `matrix_solvers.rs`; the HP overrides are in `hp_matrix`.
pub mod column_matrix;
pub mod diagonal_matrix;
pub mod general_eigen;
pub mod hp_matrix;
pub mod lower_triangular_matrix;
pub mod matrix_solvers;
pub mod symmetric_matrix;
pub mod upper_triangular_matrix;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::variable::ChangeEvent;
use crate::calculator::{self, RealFn, RealOp};
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::{self, column_vector, large_vector, row_vector, Relation, Vector};

/// `Matrix.ParallelThreshold`
pub const PARALLEL_THRESHOLD: usize = 100;
/// `Matrix.MaxSize`
pub const MAX_SIZE: usize = 1_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MatrixType {
    Full,
    UpperTriangular,
    LowerTriangular,
    Diagonal,
    Column,
    Symmetric,
}

#[derive(Debug)]
pub struct MatData {
    pub mtype: MatrixType,
    /// True for the HpMatrix classes.
    pub hp: bool,
    pub row_count: usize,
    pub col_count: usize,
    /// `_rows` (`_hpRows` for HP matrices)
    pub rows: Vec<Vector>,
    /// `HpMatrix._units`
    pub units: OptUnit,
}

pub struct MatrixInner {
    pub(crate) data: RefCell<MatData>,
    pub(crate) on_change: ChangeEvent,
}

/// A reference to a matrix object (the C# `Matrix` reference).
#[derive(Clone)]
pub struct Matrix(pub(crate) Rc<MatrixInner>);

impl std::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}x{})", self.type_name(), self.row_count(), self.col_count())
    }
}

/// `CheckBounds(Matrix a, Matrix b)`
fn check_bounds(a: &Matrix, b: &Matrix) -> CResult<()> {
    if a.row_count() != b.row_count() || a.col_count() != b.col_count() {
        return Err(exceptions::MatrixDimensions());
    }
    Ok(())
}

impl Matrix {
    // ----- construction ---------------------------------------------------------------

    pub(crate) fn from_parts(
        mtype: MatrixType,
        hp: bool,
        row_count: usize,
        col_count: usize,
        rows: Vec<Vector>,
        units: OptUnit,
    ) -> Matrix {
        Matrix(Rc::new(MatrixInner {
            data: RefCell::new(MatData {
                mtype,
                hp,
                row_count,
                col_count,
                rows,
                units,
            }),
            on_change: ChangeEvent::default(),
        }))
    }

    /// `new Matrix(int rows, int cols)`
    pub fn new(rows: usize, cols: usize) -> CResult<Matrix> {
        if rows > MAX_SIZE || cols > MAX_SIZE {
            return Err(exceptions::MatirixSizeLimit());
        }
        Ok(Matrix::full(rows, cols))
    }

    /// `new Matrix(m, n)` for dimensions known to be valid.
    pub(crate) fn full(rows: usize, cols: usize) -> Matrix {
        let r = (0..rows).map(|_| large_vector::new_l(cols)).collect();
        Matrix::from_parts(MatrixType::Full, false, rows, cols, r, None)
    }

    /// `new Matrix(Vector v)`: a one row matrix that shares the vector when it is a
    /// LargeVector.
    pub fn from_row_vector(v: &Vector) -> CResult<Matrix> {
        let len = v.len();
        if len > MAX_SIZE {
            return Err(exceptions::MatirixSizeLimit());
        }
        let row = Matrix::as_large_row(v)?;
        Ok(Matrix::from_parts(MatrixType::Full, false, 1, len, vec![row], None))
    }

    /// `v is LargeVector lv ? lv : new LargeVector(v.Raw)`
    pub(crate) fn as_large_row(v: &Vector) -> CResult<Vector> {
        if v.is_large() {
            Ok(v.clone())
        } else {
            large_vector::from_values(v.raw().ok_or_else(vector::null_reference)?)
        }
    }

    /// The implicit conversion `(Matrix)vector`: a column matrix.
    pub fn from_vector(v: &Vector) -> CResult<Matrix> {
        if v.is_hp() {
            return hp_matrix::hp_column_from_vector(v);
        }
        column_matrix::from_vector(v)
    }

    pub(crate) fn data(&self) -> Ref<'_, MatData> {
        self.0.data.borrow()
    }

    pub(crate) fn data_mut(&self) -> RefMut<'_, MatData> {
        self.0.data.borrow_mut()
    }

    pub fn ptr_eq(a: &Matrix, b: &Matrix) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }

    /// `Change()`
    pub fn change(&self) {
        self.0.on_change.invoke();
    }

    pub fn on_change(&self) -> &ChangeEvent {
        &self.0.on_change
    }

    // ----- members ---------------------------------------------------------------------

    pub fn row_count(&self) -> usize {
        self.data().row_count
    }

    pub fn col_count(&self) -> usize {
        self.data().col_count
    }

    pub fn mtype(&self) -> MatrixType {
        self.data().mtype
    }

    /// `is HpMatrix`
    pub fn is_hp(&self) -> bool {
        self.data().hp
    }

    /// `_rows.Length`
    pub fn rows_len(&self) -> usize {
        self.data().rows.len()
    }

    /// `_rows[i]`
    pub fn row_ref(&self, i: usize) -> Vector {
        self.data().rows[i].clone()
    }

    /// `_rows[i] = v`
    pub fn set_row_ref(&self, i: usize, v: Vector) {
        self.data_mut().rows[i] = v;
    }

    pub fn rows(&self) -> Vec<Vector> {
        self.data().rows.clone()
    }

    pub(crate) fn set_dims(&self, rows: usize, cols: usize) {
        let mut d = self.data_mut();
        d.row_count = rows;
        d.col_count = cols;
    }

    /// `IsStructurallyConsistentType`
    pub fn is_structurally_consistent_type(&self) -> bool {
        matches!(
            self.mtype(),
            MatrixType::Full | MatrixType::Column | MatrixType::Symmetric
        )
    }

    /// The C# class name, used by `ToString()` in error messages.
    pub fn type_name(&self) -> &'static str {
        let d = self.data();
        match (d.mtype, d.hp) {
            (MatrixType::Full, false) => "Calcpad.Core.Matrix",
            (MatrixType::Column, false) => "Calcpad.Core.ColumnMatrix",
            (MatrixType::Diagonal, false) => "Calcpad.Core.DiagonalMatrix",
            (MatrixType::LowerTriangular, false) => "Calcpad.Core.LowerTriangularMatrix",
            (MatrixType::UpperTriangular, false) => "Calcpad.Core.UpperTriangularMatrix",
            (MatrixType::Symmetric, false) => "Calcpad.Core.SymmetricMatrix",
            (MatrixType::Full, true) => "Calcpad.Core.HpMatrix",
            (MatrixType::Column, true) => "Calcpad.Core.HpColumnMatrix",
            (MatrixType::Diagonal, true) => "Calcpad.Core.HpDiagonalMatrix",
            (MatrixType::LowerTriangular, true) => "Calcpad.Core.HpLowerTriangularMatrix",
            (MatrixType::UpperTriangular, true) => "Calcpad.Core.HpUpperTriangularMatrix",
            (MatrixType::Symmetric, true) => "Calcpad.Core.HpSymmetricMatrix",
        }
    }

    pub fn to_cs_string(&self) -> String {
        self.type_name().to_string()
    }

    /// Indexer get: `this[row, col]`
    pub fn get(&self, row: usize, col: usize) -> RealValue {
        let (v, j) = {
            let d = self.data();
            let zero = || {
                if d.hp {
                    RealValue::new(0.0, d.units.clone())
                } else {
                    RealValue::ZERO
                }
            };
            match d.mtype {
                MatrixType::Full => (d.rows[row].clone(), col),
                MatrixType::Column => (d.rows[0].clone(), row),
                MatrixType::Diagonal => {
                    if row != col {
                        return zero();
                    }
                    (d.rows[0].clone(), row)
                }
                MatrixType::LowerTriangular => {
                    if row < col {
                        return zero();
                    }
                    (d.rows[row].clone(), col)
                }
                MatrixType::UpperTriangular => {
                    if row > col {
                        return zero();
                    }
                    (d.rows[row].clone(), col - row)
                }
                MatrixType::Symmetric => {
                    if row <= col {
                        (d.rows[row].clone(), col - row)
                    } else {
                        (d.rows[col].clone(), row - col)
                    }
                }
            }
        };
        v.get(j)
    }

    /// Indexer get that reports an out of range index like .NET does.
    pub fn try_get(&self, row: usize, col: usize) -> CResult<RealValue> {
        let ok = {
            let d = self.data();
            match d.mtype {
                MatrixType::Full | MatrixType::LowerTriangular => row < d.rows.len(),
                MatrixType::Column | MatrixType::Diagonal => true,
                MatrixType::UpperTriangular => row < d.rows.len(),
                MatrixType::Symmetric => row.min(col) < d.rows.len(),
            }
        };
        if !ok {
            return Err(vector::index_was_outside());
        }
        Ok(self.get(row, col))
    }

    /// Indexer set: `this[row, col] = value`
    pub fn set(&self, row: usize, col: usize, value: RealValue) -> CResult<()> {
        let (v, j) = {
            let d = self.data();
            match d.mtype {
                MatrixType::Full => (d.rows[row].clone(), col),
                MatrixType::Column => (d.rows[0].clone(), row),
                MatrixType::Diagonal => {
                    if row != col {
                        return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                    }
                    (d.rows[0].clone(), row)
                }
                MatrixType::LowerTriangular => {
                    if row < col {
                        return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row + 1, col + 1)));
                    }
                    (d.rows[row].clone(), col)
                }
                MatrixType::UpperTriangular => {
                    if row > col {
                        // C# reports the zero based indexes here.
                        return Err(exceptions::IndexOutOfRange(&format!("{}, {}", row, col)));
                    }
                    (d.rows[row].clone(), col - row)
                }
                MatrixType::Symmetric => {
                    if row <= col {
                        (d.rows[row].clone(), col - row)
                    } else {
                        (d.rows[col].clone(), row - col)
                    }
                }
            }
        };
        v.set(j, value)
    }

    /// `Clone()` (virtual): an empty matrix of the same class and size.
    pub fn clone_m(&self) -> Matrix {
        if self.is_hp() {
            return hp_matrix::clone_m(self);
        }
        let (t, m, n) = {
            let d = self.data();
            (d.mtype, d.row_count, d.col_count)
        };
        match t {
            MatrixType::Full => Matrix::full(m, n),
            MatrixType::Column => column_matrix::new(m),
            MatrixType::Diagonal => diagonal_matrix::new(m),
            MatrixType::LowerTriangular => lower_triangular_matrix::new(m),
            MatrixType::UpperTriangular => upper_triangular_matrix::new(m),
            MatrixType::Symmetric => symmetric_matrix::new(m),
        }
    }

    /// `a is HpMatrix hp_a ? hp_a.CloneAsMatrix() : a.Clone()`
    fn clone_for_rows(&self) -> Matrix {
        if self.is_hp() {
            hp_matrix::clone_as_matrix(self)
        } else {
            self.clone_m()
        }
    }

    /// `Equals(Matrix)`
    pub fn equals(&self, other: &Matrix) -> bool {
        if self.is_hp() {
            return hp_matrix::equals(self, other);
        }
        if self.mtype() != other.mtype()
            || self.row_count() != other.row_count()
            || self.col_count() != other.col_count()
        {
            return false;
        }
        let m = self.rows_len();
        for i in (0..m).rev() {
            if !self.row_ref(i).equals(&other.row_ref(i)) {
                return false;
            }
        }
        true
    }

    /// `Copy()` (protected virtual): a full copy.
    pub fn copy(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::copy(self);
        }
        let (t, m, n) = {
            let d = self.data();
            (d.mtype, d.row_count, d.col_count)
        };
        let mm = Matrix::full(m, n);
        if t == MatrixType::Full {
            for i in 0..m {
                mm.set_row_ref(i, self.row_ref(i).copy());
            }
        } else if t == MatrixType::LowerTriangular {
            for i in 0..m {
                let row = self.row_ref(i).copy();
                row.resize(n)?;
                mm.set_row_ref(i, row);
            }
        } else {
            for i in (0..m).rev() {
                let row = mm.row_ref(i);
                for j in (0..n).rev() {
                    row.set(j, self.get(i, j))?;
                }
            }
        }
        Ok(mm)
    }

    /// `Resize(int m, int n)` (virtual)
    pub fn resize(&self, m: usize, n: usize) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::resize(self, m, n);
        }
        if m > MAX_SIZE || n > MAX_SIZE {
            return Err(exceptions::MatrixDimensions());
        }
        let (t, rc, cc) = {
            let d = self.data();
            (d.mtype, d.row_count, d.col_count)
        };
        let m1 = m.min(rc) as i64 - 1;
        if t == MatrixType::Full {
            if m != rc {
                let mut d = self.data_mut();
                d.rows.truncate(m);
                for _ in rc..m {
                    d.rows.push(large_vector::new_l(n));
                }
                d.row_count = m;
            }
            if n != cc {
                for i in 0..=m1 {
                    self.row_ref(i as usize).resize(n)?;
                }
                self.data_mut().col_count = n;
            }
            return Ok(self.clone());
        }
        if n == 1 && t == MatrixType::Column {
            return column_matrix::resize(self, m);
        }
        if m == n {
            match t {
                MatrixType::Diagonal => return diagonal_matrix::resize(self, n),
                MatrixType::Symmetric => return symmetric_matrix::resize(self, n),
                MatrixType::UpperTriangular => return upper_triangular_matrix::resize(self, n),
                MatrixType::LowerTriangular => return lower_triangular_matrix::resize(self, n),
                _ => {}
            }
        }
        if m == rc && n == cc {
            return Ok(self.clone());
        }
        let mm = Matrix::full(m, n);
        let n1 = n.min(cc) as i64 - 1;
        let mut i = m1;
        while i >= 0 {
            let mut j = n1;
            while j >= 0 {
                mm.set(i as usize, j as usize, self.get(i as usize, j as usize))?;
                j -= 1;
            }
            i -= 1;
        }
        Ok(mm)
    }

    // ----- operators ---------------------------------------------------------------------

    /// Applies `f` to the rows into `c` (the `c._rows[i] = ... as LargeVector` loops).
    fn with_rows(c: Matrix, a: &Matrix, f: &dyn Fn(&Vector, usize) -> CResult<Vector>) -> CResult<Matrix> {
        let m = a.rows_len();
        for i in (0..m).rev() {
            let r = f(&a.row_ref(i), i)?;
            c.set_row_ref(i, r);
        }
        Ok(c)
    }

    /// Element by element into a new full matrix of the size of `a`.
    fn elementwise(m: usize, n: usize, f: &dyn Fn(usize, usize) -> CResult<RealValue>) -> CResult<Matrix> {
        let c = Matrix::full(m, n);
        for i in (0..m).rev() {
            for j in (0..n).rev() {
                c.set(i, j, f(i, j)?)?;
            }
        }
        Ok(c)
    }

    /// The pattern of the Matrix op Matrix operators.
    fn mm(
        a: &Matrix,
        b: &Matrix,
        by_rows: bool,
        vf: &dyn Fn(&Vector, &Vector) -> CResult<Vector>,
        ef: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
    ) -> CResult<Matrix> {
        check_bounds(a, b)?;
        if by_rows {
            let c = a.clone_for_rows();
            return Matrix::with_rows(c, a, &|r, i| vf(r, &b.row_ref(i)));
        }
        Matrix::elementwise(a.row_count(), a.col_count(), &|i, j| ef(&a.get(i, j), &b.get(i, j)))
    }

    /// The pattern of the Matrix op RealValue operators.
    fn mr(
        a: &Matrix,
        b: &RealValue,
        by_rows: bool,
        vf: &dyn Fn(&Vector, &RealValue) -> CResult<Vector>,
        ef: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
    ) -> CResult<Matrix> {
        if by_rows {
            return Matrix::with_rows(a.clone_m(), a, &|r, _| vf(r, b));
        }
        Matrix::elementwise(a.row_count(), a.col_count(), &|i, j| ef(&a.get(i, j), b))
    }

    /// The pattern of the RealValue op Matrix operators.
    fn rm(
        a: &RealValue,
        b: &Matrix,
        by_rows: bool,
        vf: &dyn Fn(&RealValue, &Vector) -> CResult<Vector>,
        ef: &dyn Fn(&RealValue, &RealValue) -> CResult<RealValue>,
    ) -> CResult<Matrix> {
        if by_rows {
            return Matrix::with_rows(b.clone_m(), b, &|r, _| vf(a, r));
        }
        Matrix::elementwise(b.row_count(), b.col_count(), &|i, j| ef(a, &b.get(i, j)))
    }

    fn same_type(a: &Matrix, b: &Matrix) -> bool {
        a.mtype() == b.mtype()
    }

    fn same_consistent(a: &Matrix, b: &Matrix) -> bool {
        a.mtype() == b.mtype() && a.is_structurally_consistent_type()
    }

    /// `-a`
    pub fn neg(a: &Matrix) -> CResult<Matrix> {
        Matrix::with_rows(a.clone_m(), a, &|r, _| Ok(Vector::neg(r)))
    }

    pub fn sub(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::sub, &RealValue::sub)
    }
    pub fn sub_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::sub_vr, &RealValue::sub)
    }
    pub fn sub_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type() || a.equals(&RealValue::ZERO);
        Matrix::rm(a, b, r, &Vector::sub_rv, &RealValue::sub)
    }
    pub fn add(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::add, &RealValue::add)
    }
    pub fn add_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::add_vr, &RealValue::add)
    }
    pub fn mul_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.units.is_none();
        Matrix::mr(a, b, r, &Vector::mul_vr, &|x, y| Ok(RealValue::mul_op(x, y)))
    }
    pub fn div(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_consistent(a, b), &Vector::div, &|x, y| {
            Ok(RealValue::div_op(x, y))
        })
    }
    pub fn div_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() && !b.equals(&RealValue::ZERO) && b.units.is_none();
        Matrix::mr(a, b, r, &Vector::div_vr, &|x, y| Ok(RealValue::div_op(x, y)))
    }
    pub fn div_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type();
        Matrix::rm(a, b, r, &Vector::div_rv, &|x, y| Ok(RealValue::div_op(x, y)))
    }
    pub fn rem(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_consistent(a, b), &Vector::rem, &RealValue::rem)
    }
    pub fn rem_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        if b.equals(&RealValue::ZERO) {
            return Matrix::full(a.row_count(), a.col_count()).fill(&RealValue::NAN);
        }
        let r = a.is_structurally_consistent_type();
        Matrix::mr(a, b, r, &Vector::rem_vr, &RealValue::rem)
    }
    pub fn rem_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type();
        Matrix::rm(a, b, r, &Vector::rem_rv, &RealValue::rem)
    }
    pub fn eq(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_consistent(a, b), &Vector::eq, &RealValue::eq_op)
    }
    pub fn eq_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type();
        Matrix::mr(a, b, r, &Vector::eq_vr, &RealValue::eq_op)
    }
    pub fn ne(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::ne, &RealValue::ne_op)
    }
    pub fn ne_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::ne_vr, &RealValue::ne_op)
    }
    pub fn lt(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::lt, &RealValue::lt_op)
    }
    pub fn lt_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::lt_vr, &RealValue::lt_op)
    }
    pub fn lt_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type() || a.equals(&RealValue::ZERO);
        Matrix::rm(a, b, r, &Vector::lt_rv, &RealValue::lt_op)
    }
    pub fn gt(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::gt, &RealValue::gt_op)
    }
    pub fn gt_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::gt_vr, &RealValue::gt_op)
    }
    pub fn gt_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type() || a.equals(&RealValue::ZERO);
        Matrix::rm(a, b, r, &Vector::gt_rv, &RealValue::gt_op)
    }
    pub fn ge(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_consistent(a, b), &Vector::ge, &RealValue::ge_op)
    }
    pub fn ge_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type();
        Matrix::mr(a, b, r, &Vector::ge_vr, &RealValue::ge_op)
    }
    pub fn ge_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type();
        Matrix::rm(a, b, r, &Vector::ge_rv, &RealValue::ge_op)
    }
    pub fn le(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_consistent(a, b), &Vector::le, &RealValue::le_op)
    }
    pub fn le_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type();
        Matrix::mr(a, b, r, &Vector::le_vr, &RealValue::le_op)
    }
    pub fn le_rm(a: &RealValue, b: &Matrix) -> CResult<Matrix> {
        let r = b.is_structurally_consistent_type();
        Matrix::rm(a, b, r, &Vector::le_rv, &RealValue::le_op)
    }
    pub fn and(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::and, &|x, y| {
            Ok(RealValue::and_op(x, y))
        })
    }
    pub fn and_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::and_vr, &|x, y| Ok(RealValue::and_op(x, y)))
    }
    pub fn or(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::or, &|x, y| {
            Ok(RealValue::or_op(x, y))
        })
    }
    pub fn or_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::or_vr, &|x, y| Ok(RealValue::or_op(x, y)))
    }
    pub fn xor(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        Matrix::mm(a, b, Matrix::same_type(a, b), &Vector::xor, &|x, y| {
            Ok(RealValue::xor_op(x, y))
        })
    }
    pub fn xor_mr(a: &Matrix, b: &RealValue) -> CResult<Matrix> {
        let r = a.is_structurally_consistent_type() || b.equals(&RealValue::ZERO);
        Matrix::mr(a, b, r, &Vector::xor_vr, &|x, y| Ok(RealValue::xor_op(x, y)))
    }

    /// `Matrix * Matrix`
    pub fn mul(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        if a.col_count() != b.row_count() {
            return Err(exceptions::MatrixDimensions());
        }
        let (ta, tb) = (a.mtype(), b.mtype());
        if tb == MatrixType::Column && !b.is_hp() {
            return column_matrix::mul(a, b);
        }
        if ta == MatrixType::Diagonal && !a.is_hp() {
            return match (tb, b.is_hp()) {
                (MatrixType::Diagonal, false) => diagonal_matrix::mul_dd(a, b),
                (MatrixType::LowerTriangular, false) => diagonal_matrix::mul_dl(a, b),
                (MatrixType::UpperTriangular, false) => diagonal_matrix::mul_du(a, b),
                (MatrixType::Symmetric, false) => diagonal_matrix::mul_ds(a, b),
                _ => diagonal_matrix::mul_dm(a, b),
            };
        } else if tb == MatrixType::Diagonal && !b.is_hp() {
            return match (ta, a.is_hp()) {
                (MatrixType::LowerTriangular, false) => diagonal_matrix::mul_ld(a, b),
                (MatrixType::UpperTriangular, false) => diagonal_matrix::mul_ud(a, b),
                (MatrixType::Symmetric, false) => diagonal_matrix::mul_sd(a, b),
                _ => diagonal_matrix::mul_md(a, b),
            };
        }
        let m = a.row_count();
        let c = Matrix::full(m, b.col_count());
        let na = a.col_count();
        let nb = b.col_count();
        if ta == MatrixType::Full || ta == MatrixType::LowerTriangular {
            let cols: Vec<Vector> = (0..nb).map(|j| column_vector::new(b, j)).collect();
            for i in (0..m).rev() {
                let cr = c.row_ref(i);
                let ar = a.row_ref(i);
                for j in (0..nb).rev() {
                    cr.set(j, Vector::dot_product(&ar, &cols[j])?)?;
                }
            }
        } else {
            for i in (0..m).rev() {
                let c_i = c.row_ref(i);
                for j in (0..nb).rev() {
                    if na == 0 {
                        return Err(vector::index_was_outside());
                    }
                    let mut c_ij = RealValue::mul_op(&a.get(i, na - 1), &b.get(na - 1, j));
                    for k in (0..na - 1).rev() {
                        c_ij = RealValue::add(&c_ij, &RealValue::mul_op(&a.get(i, k), &b.get(k, j)))?;
                    }
                    c_i.set(j, c_ij)?;
                }
            }
        }
        Ok(c)
    }

    /// `EvaluateOperator(op, Matrix a, Matrix b, index)`
    pub fn evaluate_operator(op: RealOp, a: &Matrix, b: &Matrix, index: i64) -> CResult<Matrix> {
        match calculator::get_operator_symbol(index) {
            '/' => Matrix::div(a, b),
            '\u{29BC}' => Matrix::rem(a, b),
            '*' => Matrix::mul(a, b),
            '-' => Matrix::sub(a, b),
            '+' => Matrix::add(a, b),
            '<' => Matrix::lt(a, b),
            '>' => Matrix::gt(a, b),
            '\u{2264}' => Matrix::le(a, b),
            '\u{2265}' => Matrix::ge(a, b),
            '\u{2261}' => Matrix::eq(a, b),
            '\u{2260}' => Matrix::ne(a, b),
            '\u{2227}' => Matrix::and(a, b),
            '\u{2228}' => Matrix::or(a, b),
            '\u{2295}' => Matrix::xor(a, b),
            _ => Matrix::evaluate_operator2(op, a, b, index),
        }
    }

    /// `EvaluateOperator(op, Matrix a, RealValue b, index)`
    pub fn evaluate_operator_mr(op: RealOp, a: &Matrix, b: &RealValue, index: i64) -> CResult<Matrix> {
        match calculator::get_operator_symbol(index) {
            '/' => Matrix::div_mr(a, b),
            '\u{29BC}' => Matrix::rem_mr(a, b),
            '*' => Matrix::mul_mr(a, b),
            '-' => Matrix::sub_mr(a, b),
            '+' => Matrix::add_mr(a, b),
            '<' => Matrix::lt_mr(a, b),
            '>' => Matrix::gt_mr(a, b),
            '\u{2264}' => Matrix::le_mr(a, b),
            '\u{2265}' => Matrix::ge_mr(a, b),
            '\u{2261}' => Matrix::eq_mr(a, b),
            '\u{2260}' => Matrix::ne_mr(a, b),
            '\u{2227}' => Matrix::and_mr(a, b),
            '\u{2228}' => Matrix::or_mr(a, b),
            '\u{2295}' => Matrix::xor_mr(a, b),
            _ => Matrix::evaluate_operator2_mr(op, a, b, index),
        }
    }

    /// `EvaluateOperator(op, RealValue a, Matrix b, index)`
    pub fn evaluate_operator_rm(op: RealOp, a: &RealValue, b: &Matrix, index: i64) -> CResult<Matrix> {
        match calculator::get_operator_symbol(index) {
            '/' => Matrix::div_rm(a, b),
            '\u{29BC}' => Matrix::rem_rm(a, b),
            '*' => Matrix::mul_mr(b, a),
            '-' => Matrix::sub_rm(a, b),
            '+' => Matrix::add_mr(b, a),
            '<' => Matrix::lt_rm(a, b),
            '>' => Matrix::gt_rm(a, b),
            '\u{2264}' => Matrix::le_rm(a, b),
            '\u{2265}' => Matrix::ge_rm(a, b),
            '\u{2261}' => Matrix::eq_mr(b, a),
            '\u{2260}' => Matrix::ne_mr(b, a),
            '\u{2227}' => Matrix::and_mr(b, a),
            '\u{2228}' => Matrix::or_mr(b, a),
            '\u{2295}' => Matrix::xor_mr(b, a),
            _ => Matrix::evaluate_operator2_rm(op, a, b, index),
        }
    }

    fn evaluate_operator2(op: RealOp, a: &Matrix, b: &Matrix, index: i64) -> CResult<Matrix> {
        check_bounds(a, b)?;
        if a.mtype() == b.mtype()
            && (a.is_structurally_consistent_type() || calculator::is_zero_preserving_operator(index))
        {
            let c = a.clone_for_rows();
            return Matrix::with_rows(c, a, &|r, i| Vector::evaluate_operator(op, r, &b.row_ref(i), index));
        }
        Matrix::elementwise(a.row_count(), a.col_count(), &|i, j| {
            op.apply(&a.get(i, j), &b.get(i, j))
        })
    }

    /// Note: C# compares the matrix `a` with RealValue.Zero here (always false).
    pub(crate) fn evaluate_operator2_mr(op: RealOp, a: &Matrix, b: &RealValue, index: i64) -> CResult<Matrix> {
        if a.is_structurally_consistent_type() {
            return Matrix::with_rows(a.clone_m(), a, &|r, _| Vector::evaluate_operator_vr(op, r, b, index));
        }
        Matrix::elementwise(a.row_count(), a.col_count(), &|i, j| op.apply(&a.get(i, j), b))
    }

    pub(crate) fn evaluate_operator2_rm(op: RealOp, a: &RealValue, b: &Matrix, index: i64) -> CResult<Matrix> {
        if b.is_structurally_consistent_type()
            || calculator::is_zero_preserving_operator(index) && a.equals(&RealValue::ZERO)
        {
            return Matrix::with_rows(b.clone_m(), b, &|r, _| Vector::evaluate_operator_rv(op, a, r, index));
        }
        Matrix::elementwise(b.row_count(), b.col_count(), &|i, j| op.apply(a, &b.get(i, j)))
    }

    /// `EvaluateFunction(f, Matrix a, index)`
    pub fn evaluate_function(f: RealFn, a: &Matrix, index: i64) -> CResult<Matrix> {
        if a.is_structurally_consistent_type() || calculator::is_zero_preserving_function(index) {
            return Matrix::with_rows(a.clone_m(), a, &|r, _| Vector::evaluate_function(f, r));
        }
        Matrix::elementwise(a.row_count(), a.col_count(), &|i, j| f.apply(&a.get(i, j)))
    }

    /// `SetUnits(Unit units)`
    pub fn set_units(&self, units: &OptUnit) -> CResult<()> {
        if self.is_hp() {
            return hp_matrix::set_units(self, units);
        }
        for r in self.rows().iter().rev() {
            r.set_units(units)?;
        }
        Ok(())
    }

    /// `Values`: all elements, row by row.
    pub fn values(&self) -> Vec<RealValue> {
        let (m, n) = (self.row_count(), self.col_count());
        let mut values = Vec::with_capacity(m * n);
        for i in 0..m {
            for j in 0..n {
                values.push(self.get(i, j));
            }
        }
        values
    }

    /// `Hadamard(Matrix a, Matrix b)`
    pub fn hadamard(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        check_bounds(a, b)?;
        if a.mtype() == b.mtype() {
            let c = a.clone_for_rows();
            return Matrix::with_rows(c, a, &|r, i| Vector::mul(r, &b.row_ref(i)));
        }
        let (m, n) = (a.row_count(), a.col_count());
        let c = if a.mtype() == MatrixType::Column && b.mtype() == MatrixType::Column {
            column_matrix::new(m)
        } else {
            Matrix::full(m, n)
        };
        for i in (0..m).rev() {
            for j in (0..n).rev() {
                c.set(i, j, RealValue::mul_op(&a.get(i, j), &b.get(i, j)))?;
            }
        }
        Ok(c)
    }

    /// `Kronecker(Matrix a, Matrix b)`
    pub fn kronecker(a: &Matrix, b: &Matrix) -> CResult<Matrix> {
        let (m, n) = (a.row_count(), a.col_count());
        let (p, q) = (b.row_count(), b.col_count());
        let c = Matrix::new(m * p, n * q)?;
        for i in (0..m).rev() {
            let ip = i * p;
            for j in (0..n).rev() {
                let jq = j * q;
                for k in (0..p).rev() {
                    for l in (0..q).rev() {
                        c.set(ip + k, jq + l, RealValue::mul_op(&a.get(i, j), &b.get(k, l)))?;
                    }
                }
            }
        }
        Ok(c)
    }

    /// `Frobenius(Matrix a, Matrix b)`
    pub fn frobenius(a: &Matrix, b: &Matrix) -> CResult<RealValue> {
        check_bounds(a, b)?;
        let m = a.rows_len();
        if m > PARALLEL_THRESHOLD {
            let mut row_sums = Vec::with_capacity(m);
            for i in 0..m {
                row_sums.push(Vector::dot_product(&a.row_ref(i), &b.row_ref(i))?);
            }
            return Vector::from_values(row_sums)?.sum();
        }
        let mut sum = RealValue::ZERO;
        for i in (0..m).rev() {
            sum = RealValue::add(&sum, &Vector::dot_product(&a.row_ref(i), &b.row_ref(i))?)?;
        }
        Ok(sum)
    }

    /// `Row(int index)` (virtual), 1 based
    pub fn row(&self, index: i64) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::row(self, index);
        }
        if index < 1 || index > self.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&index.to_string()));
        }
        let i = (index - 1) as usize;
        let t = self.mtype();
        if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            return Ok(self.row_ref(i).copy());
        }
        let n = self.col_count();
        let row = large_vector::new_l(n);
        for j in (0..n).rev() {
            row.set(j, self.get(i, j))?;
        }
        Ok(row)
    }

    /// `Col(int index)` (virtual), 1 based
    pub fn col(&self, index: i64) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::col(self, index);
        }
        if index < 1 || index > self.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&index.to_string()));
        }
        let j = (index - 1) as usize;
        if self.mtype() == MatrixType::Column {
            return Ok(self.row_ref(j).copy());
        }
        let m = self.row_count();
        let col = large_vector::new_l(m);
        for i in (0..m).rev() {
            col.set(i, self.get(i, j))?;
        }
        Ok(col)
    }

    /// `SetRow(int index, Vector row)`
    pub(crate) fn set_row(&self, index: usize, row: &Vector) -> CResult<()> {
        let n = self.col_count();
        let r = if row.is_large() {
            row.resize(n)?
        } else {
            large_vector::from_values(row.raw().ok_or_else(vector::null_reference)?)?.resize(n)?
        };
        self.set_row_ref(index, r);
        Ok(())
    }

    /// `SetCol(int index, Vector col)`
    pub(crate) fn set_col(&self, index: usize, col: &Vector) -> CResult<()> {
        let n = col.len().min(self.row_count());
        for i in (0..n).rev() {
            self.set(i, index, col.get(i))?;
        }
        Ok(())
    }

    /// `Submatrix(int i1, int i2, int j1, int j2)` (virtual), 1 based
    pub fn submatrix(&self, i1: i64, i2: i64, j1: i64, j2: i64) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::submatrix(self, i1, i2, j1, j2);
        }
        let (mut i1, mut i2, mut j1, mut j2) = (i1, i2, j1, j2);
        if i2 < i1 {
            std::mem::swap(&mut i1, &mut i2);
        }
        if j2 < j1 {
            std::mem::swap(&mut j1, &mut j2);
        }
        if i1 < 1 {
            return Err(exceptions::IndexOutOfRange(&i1.to_string()));
        }
        if i2 > self.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&i2.to_string()));
        }
        if j1 < 1 {
            return Err(exceptions::IndexOutOfRange(&j1.to_string()));
        }
        if j2 > self.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&j2.to_string()));
        }
        let rows = (i2 - i1) as usize;
        let cols = (j2 - j1) as usize;
        let mm = Matrix::new(rows + 1, cols + 1)?;
        let i1 = (i1 - 1) as usize;
        let t = self.mtype();
        if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            for i in (0..=rows).rev() {
                mm.set_row_ref(i, self.row_ref(i1 + i).slice(j1, j2)?);
            }
        } else {
            let j1 = (j1 - 1) as usize;
            for i in (0..=rows).rev() {
                let row = mm.row_ref(i);
                for j in (0..=cols).rev() {
                    row.set(j, self.get(i1 + i, j1 + j))?;
                }
            }
        }
        Ok(mm)
    }

    /// `ExtractRows(Vector vector)` (virtual): shares the rows of a full matrix.
    pub fn extract_rows(&self, vector: &Vector) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::extract_rows(self, vector);
        }
        let row_count = vector.len();
        let n = self.col_count();
        let mm = Matrix::new(row_count, n)?;
        let t = self.mtype();
        for i in (0..row_count).rev() {
            let row_index = vector::cs_int(vector.get(i).d);
            if row_index < 1 || row_index > self.row_count() as i64 {
                return Err(exceptions::IndexOutOfRange(&row_index.to_string()));
            }
            let ri = (row_index - 1) as usize;
            if t == MatrixType::Full || t == MatrixType::LowerTriangular {
                mm.set_row_ref(i, self.row_ref(ri));
            } else {
                let row = mm.row_ref(i);
                for j in (0..n).rev() {
                    row.set(j, self.get(ri, j))?;
                }
            }
        }
        Ok(mm)
    }

    /// `ExtractCols(Vector vector)` (virtual)
    pub fn extract_cols(&self, vector: &Vector) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::extract_cols(self, vector);
        }
        let col_count = vector.len();
        let m = self.row_count();
        let mm = Matrix::new(m, col_count)?;
        for j in (0..col_count).rev() {
            let col_index = vector::cs_int(vector.get(j).d);
            if col_index < 1 || col_index > self.col_count() as i64 {
                return Err(exceptions::IndexOutOfRange(&col_index.to_string()));
            }
            for i in (0..m).rev() {
                mm.set(i, j, self.get(i, (col_index - 1) as usize))?;
            }
        }
        Ok(mm)
    }

    /// `Fill(RealValue value)` (virtual): fills in place.
    pub fn fill(&self, value: &RealValue) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::fill(self, value);
        }
        for r in self.rows().iter().rev() {
            r.fill(value)?;
        }
        Ok(self.clone())
    }

    /// `FillRow(int i, in RealValue value)` (virtual), 1 based
    pub fn fill_row(&self, i: i64, value: &RealValue) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::fill_row(self, i, value);
        }
        if i > self.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&i.to_string()));
        }
        let i = (i - 1) as usize;
        match self.mtype() {
            MatrixType::Full | MatrixType::Symmetric | MatrixType::UpperTriangular | MatrixType::LowerTriangular => {
                self.row_ref(i).fill(value)?;
            }
            _ => self.row_ref(0).set(i, value.clone())?,
        }
        Ok(self.clone())
    }

    /// `FillCol(int j, in RealValue value)` (virtual), 1 based
    pub fn fill_col(&self, j: i64, value: &RealValue) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::fill_col(self, j, value);
        }
        if j > self.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&j.to_string()));
        }
        let j = (j - 1) as usize;
        let m = self.row_count();
        match self.mtype() {
            MatrixType::Full => {
                for i in (0..m).rev() {
                    self.set(i, j, value.clone())?;
                }
            }
            MatrixType::Column | MatrixType::Symmetric => {
                self.row_ref(j).fill(value)?;
            }
            MatrixType::Diagonal => self.row_ref(0).set(j, value.clone())?,
            MatrixType::LowerTriangular => {
                for i in (j..m).rev() {
                    self.set(i, j, value.clone())?;
                }
            }
            MatrixType::UpperTriangular => {
                for i in (0..=j).rev() {
                    self.set(i, j, value.clone())?;
                }
            }
        }
        Ok(self.clone())
    }

    /// `Augment(params Matrix[] matrices)`
    pub fn augment(matrices: &[Matrix]) -> CResult<Matrix> {
        let (mut m, mut n) = (0, 0);
        for matrix in matrices {
            n += matrix.col_count();
            m = m.max(matrix.row_count());
        }
        let mm = Matrix::new(m, n)?;
        let mut c = 0;
        for matrix in matrices {
            let (mr, nr) = (matrix.row_count(), matrix.col_count());
            for i in (0..mr).rev() {
                let row = mm.row_ref(i);
                for j in (0..nr).rev() {
                    row.set(c + j, matrix.get(i, j))?;
                }
            }
            c += nr;
        }
        Ok(mm)
    }

    /// `Stack(params Matrix[] matrices)`
    pub fn stack(matrices: &[Matrix]) -> CResult<Matrix> {
        let (mut m, mut n) = (0, 0);
        for matrix in matrices {
            n = n.max(matrix.col_count());
            m += matrix.row_count();
        }
        let mm = Matrix::new(m, n)?;
        let mut r = 0;
        for matrix in matrices {
            let (mr, nr) = (matrix.row_count(), matrix.col_count());
            for i in (0..mr).rev() {
                let row = mm.row_ref(r + i);
                for j in (0..nr).rev() {
                    row.set(j, matrix.get(i, j))?;
                }
            }
            r += mr;
        }
        Ok(mm)
    }

    /// `Search(in RealValue value, int iStart, int jStart)` (virtual), 1 based
    pub fn search(&self, value: &RealValue, i_start: usize, j_start: usize) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::search(self, value, i_start, j_start);
        }
        let t = self.mtype();
        let (m, n) = (self.row_count(), self.col_count());
        if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            let mut j_start = j_start;
            for i in i_start.saturating_sub(1)..m {
                let j_value = self.row_ref(i).search(value, j_start)?;
                if !j_value.equals(&RealValue::ZERO) {
                    return Vector::from_values(vec![RealValue::num((i + 1) as f64), j_value]);
                }
                j_start = 1;
            }
        } else {
            let mut j_start = j_start.saturating_sub(1);
            for i in i_start.saturating_sub(1)..m {
                for j in j_start..n {
                    if vector::real_almost_equals(&self.get(i, j), value)? {
                        return Vector::from_values(vec![
                            RealValue::num((i + 1) as f64),
                            RealValue::num((j + 1) as f64),
                        ]);
                    }
                }
                j_start = 0;
            }
        }
        Ok(Vector::dense(2))
    }

    /// `Count(RealValue value)` (virtual)
    pub fn count(&self, value: &RealValue) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::count(self, value);
        }
        let t = self.mtype();
        if t == MatrixType::Symmetric {
            return symmetric_matrix::count(self, value);
        }
        if t == MatrixType::Full || t == MatrixType::LowerTriangular || !value.equals(&RealValue::ZERO) {
            let mut count = 0.0;
            for r in self.rows() {
                count += r.count(value, 1)?.d;
            }
            return Ok(RealValue::num(count));
        }
        let mut count = 0;
        for i in 0..self.row_count() {
            for j in 0..self.col_count() {
                if vector::real_almost_equals(&self.get(i, j), value)? {
                    count += 1;
                }
            }
        }
        Ok(RealValue::num(count as f64))
    }

    /// `FindAll(in RealValue value, Relation rel)` (virtual)
    pub fn find_all(&self, value: &RealValue, rel: Relation) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::find_all(self, value, rel);
        }
        let mm = Matrix::from_indexes(&self.find_all_indexes(value, rel)?)?;
        if mm.col_count() == 0 {
            return Matrix::new(2, 1);
        }
        Ok(mm)
    }

    /// `HLookup(in RealValue value, int searchRow, int returnRow, Relation rel)` (virtual)
    pub fn h_lookup(&self, value: &RealValue, search_row: i64, return_row: i64, rel: Relation) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::h_lookup(self, value, search_row, return_row, rel);
        }
        let m = self.row_count() as i64;
        if search_row < 1 || search_row > m {
            return Err(exceptions::IndexOutOfRange(&search_row.to_string()));
        }
        if return_row < 1 || return_row > m {
            return Err(exceptions::IndexOutOfRange(&return_row.to_string()));
        }
        let i = (search_row - 1) as usize;
        let mut indexes = Vec::new();
        for j in 0..self.col_count() {
            if Vector::relate(&self.get(i, j), value, rel)? {
                indexes.push(j);
            }
        }
        let vector = Vector::dense(indexes.len());
        let i = (return_row - 1) as usize;
        for (j, &k) in indexes.iter().enumerate() {
            vector.set(j, self.get(i, k))?;
        }
        Ok(vector)
    }

    /// `VLookup(in RealValue value, int searchCol, int returnCol, Relation rel)` (virtual)
    pub fn v_lookup(&self, value: &RealValue, search_col: i64, return_col: i64, rel: Relation) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::v_lookup(self, value, search_col, return_col, rel);
        }
        let n = self.col_count() as i64;
        if search_col > n {
            return Err(exceptions::IndexOutOfRange(&search_col.to_string()));
        }
        if return_col > n {
            return Err(exceptions::IndexOutOfRange(&return_col.to_string()));
        }
        let j = (search_col - 1) as usize;
        let mut indexes = Vec::new();
        for i in 0..self.row_count() {
            if Vector::relate(&self.get(i, j), value, rel)? {
                indexes.push(i);
            }
        }
        let j = (return_col - 1) as usize;
        let vector = Vector::dense(indexes.len());
        for (i, &k) in indexes.iter().enumerate() {
            vector.set(i, self.get(k, j))?;
        }
        Ok(vector)
    }

    fn find_all_indexes(&self, value: &RealValue, rel: Relation) -> CResult<Vec<(usize, usize)>> {
        let mut indexes = Vec::new();
        for i in 0..self.row_count() {
            for j in 0..self.col_count() {
                if Vector::relate(&self.get(i, j), value, rel)? {
                    indexes.push((i + 1, j + 1));
                }
            }
        }
        Ok(indexes)
    }

    fn from_indexes(indexes: &[(usize, usize)]) -> CResult<Matrix> {
        let n = indexes.len();
        let matrix = Matrix::new(2, n)?;
        for (k, &(i, j)) in indexes.iter().enumerate() {
            matrix.set(0, k, RealValue::num(i as f64))?;
            matrix.set(1, k, RealValue::num(j as f64))?;
        }
        Ok(matrix)
    }

    /// `SortRows(int j, bool reverse)` (virtual), 1 based column
    pub fn sort_rows(&self, j: i64, reverse: bool) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::sort_rows(self, j, reverse);
        }
        let j = j - 1;
        if j < 0 {
            return Err(vector::index_was_outside());
        }
        let indexes = column_vector::new(self, j as usize).get_order_indexes(reverse, None)?;
        self.reorder_rows(&indexes)
    }

    /// `ReorderRows(int[] indexes)` (virtual): shares the rows of a full matrix.
    pub fn reorder_rows(&self, indexes: &[i32]) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::reorder_rows(self, indexes);
        }
        let (m, n) = (self.row_count(), self.col_count());
        let mm = Matrix::new(m, n)?;
        let t = self.mtype();
        if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            for i in (0..m).rev() {
                mm.set_row_ref(i, self.row_ref(indexes[i] as usize));
            }
        } else {
            for i in (0..m).rev() {
                let row = mm.row_ref(i);
                for k in (0..n).rev() {
                    row.set(k, self.get(indexes[i] as usize, k))?;
                }
            }
        }
        Ok(mm)
    }

    /// `SortCols(int i, bool reverse)` (virtual), 1 based row
    pub fn sort_cols(&self, i: i64, reverse: bool) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::sort_cols(self, i, reverse);
        }
        if i > self.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&i.to_string()));
        }
        let i = (i - 1) as usize;
        let t = self.mtype();
        let row = if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            self.row_ref(i)
        } else {
            row_vector::new(self, i)
        };
        let indexes = row.get_order_indexes(reverse, Some(self.col_count()))?;
        self.reorder_cols(&indexes)
    }

    /// `ReorderCols(int[] indexes)` (virtual)
    pub fn reorder_cols(&self, indexes: &[i32]) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::reorder_cols(self, indexes);
        }
        let (m, n) = (self.row_count(), self.col_count());
        let mm = Matrix::new(m, n)?;
        for k in (0..m).rev() {
            let row = mm.row_ref(k);
            for j in (0..n).rev() {
                row.set(j, self.get(k, indexes[j] as usize))?;
            }
        }
        Ok(mm)
    }

    /// `OrderRows(int j, bool reverse)` (virtual)
    pub fn order_rows(&self, j: i64, reverse: bool) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::order_rows(self, j, reverse);
        }
        if j > self.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&j.to_string()));
        }
        let j = (j - 1) as usize;
        let col = if self.mtype() == MatrixType::Column {
            self.row_ref(0)
        } else {
            column_vector::new(self, j)
        };
        col.order(reverse)
    }

    /// `OrderCols(int i, bool reverse)` (virtual)
    pub fn order_cols(&self, i: i64, reverse: bool) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::order_cols(self, i, reverse);
        }
        if i > self.row_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&i.to_string()));
        }
        let i = (i - 1) as usize;
        let t = self.mtype();
        let row = if t == MatrixType::Full || t == MatrixType::LowerTriangular {
            self.row_ref(i)
        } else {
            row_vector::new(self, i)
        };
        row.order(reverse)
    }

    // ----- aggregates -----------------------------------------------------------------------

    /// `Min()` (virtual)
    pub fn min(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::min(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let v = rows[0].min()?;
        let mut min = v.d;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.min()?;
            let row_min = v.d * Unit::convert(&u, &v.units, ',')?;
            if rows.len() > PARALLEL_THRESHOLD {
                // LINQ Min over the row minima propagates NaN.
                if row_min.is_nan() || row_min < min {
                    min = row_min;
                }
            } else if row_min < min {
                min = row_min;
            }
        }
        if !self.is_structurally_consistent_type() && min > 0.0 {
            min = 0.0;
        }
        Ok(RealValue::new(min, u))
    }

    /// `Max()` (virtual)
    pub fn max(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::max(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let v = rows[0].max()?;
        let mut max = v.d;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.max()?;
            let row_max = v.d * Unit::convert(&u, &v.units, ',')?;
            if row_max > max {
                max = row_max;
            }
        }
        if !self.is_structurally_consistent_type() && max < 0.0 {
            max = 0.0;
        }
        Ok(RealValue::new(max, u))
    }

    /// `Sum()` (virtual)
    pub fn sum(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::sum(self);
        }
        if self.mtype() == MatrixType::Symmetric {
            return symmetric_matrix::sum(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let v = rows[0].sum()?;
        let mut sum = v.d;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.sum()?;
            sum += v.d * Unit::convert(&u, &v.units, ',')?;
        }
        Ok(RealValue::new(sum, u))
    }

    /// `SumSq()` (virtual)
    pub fn sum_sq(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::sum_sq(self);
        }
        if self.mtype() == MatrixType::Symmetric {
            return symmetric_matrix::sum_sq(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let v = rows[0].sum_sq()?;
        let mut sumsq = v.d;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.sum_sq()?;
            sumsq += v.d * Unit::convert(&u, &v.units, ',')?;
        }
        Ok(RealValue::new(sumsq, u))
    }

    /// `Srss()` (virtual)
    pub fn srss(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::srss(self);
        }
        let sumsq = self.sum_sq()?;
        let u = sumsq.units.clone();
        Ok(RealValue::new(sumsq.d.sqrt(), u.and_then(|u| u.pow(0.5))))
    }

    /// `Average()` (virtual)
    pub fn average(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::average(self);
        }
        let sum = self.sum()?;
        let n = (self.row_count() * self.col_count()) as f64;
        Ok(RealValue::new(sum.d / n, sum.units))
    }

    /// `Product()` (virtual)
    pub fn product(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::product(self);
        }
        if self.mtype() == MatrixType::Symmetric {
            return symmetric_matrix::product(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Ok(RealValue::ZERO);
        }
        let p = rows[0].product()?;
        let mut product = p.d;
        let mut u = p.units.clone();
        for r in &rows[1..] {
            let p = r.product()?;
            let (uc, b) = Unit::multiply(&u, &p.units, false);
            u = uc;
            product *= p.d * b;
        }
        if !self.is_structurally_consistent_type() {
            return Ok(RealValue::new(0.0, u));
        }
        Ok(RealValue::new(product, u))
    }

    /// `Mean()` (virtual)
    pub fn mean(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::mean(self);
        }
        let product = self.product()?;
        let n = self.row_count() * self.col_count();
        let result = product.d.powf(1.0 / n as f64);
        match &product.units {
            None => Ok(RealValue::num(result)),
            Some(u) => Ok(RealValue::new(result, Unit::root(u, n as i32, false))),
        }
    }

    /// `And()` (virtual)
    pub fn and_all(&self) -> RealValue {
        if self.is_hp() {
            return hp_matrix::and_all(self);
        }
        if !self.is_structurally_consistent_type() {
            return RealValue::ZERO;
        }
        for r in self.rows() {
            if r.and_all().d.abs() < LOGICAL_ZERO {
                return RealValue::ZERO;
            }
        }
        RealValue::ONE
    }

    /// `Or()` (virtual)
    pub fn or_all(&self) -> RealValue {
        if self.is_hp() {
            return hp_matrix::or_all(self);
        }
        for r in self.rows() {
            if r.or_all().d.abs() >= LOGICAL_ZERO {
                return RealValue::ONE;
            }
        }
        RealValue::ZERO
    }

    /// `Xor()` (virtual)
    pub fn xor_all(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::xor_all(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Err(vector::index_was_outside());
        }
        let mut b = rows[0].xor_all().d.abs() >= LOGICAL_ZERO;
        for r in &rows[1..] {
            b = b != (r.xor_all().d.abs() >= LOGICAL_ZERO);
        }
        Ok(if b { RealValue::ONE } else { RealValue::ZERO })
    }

    /// `Gcd()` (virtual)
    pub fn gcd(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::gcd(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Err(vector::index_was_outside());
        }
        let v = rows[0].gcd()?;
        let mut a = calculator::as_long(v.d)?;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.gcd()?;
            let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
            a = calculator::gcd_l(a, b);
        }
        Ok(RealValue::num(a as f64))
    }

    /// `Lcm()` (virtual)
    pub fn lcm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::lcm(self);
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Err(vector::index_was_outside());
        }
        let v = rows[0].lcm()?;
        let mut a = calculator::as_long(v.d)?;
        let u = v.units.clone();
        for r in &rows[1..] {
            let v = r.lcm()?;
            let b = calculator::as_long(v.d * Unit::convert(&u, &v.units, ',')?)?;
            let g = calculator::gcd_l(a, b);
            if g == 0 {
                return Err(MathParserException::unexpected("Attempted to divide by zero."));
            }
            a = a.wrapping_mul(b) / g;
        }
        Ok(RealValue::num(a as f64))
    }

    fn out_of_limits(&self, dx: f64, dy: f64) -> bool {
        let (m, n) = (self.row_count() as f64, self.col_count() as f64);
        !dy.is_normal()
            || dy < calculator::DELTA_MINUS
            || dy > m * calculator::DELTA_PLUS
            || !dx.is_normal()
            || dx < calculator::DELTA_MINUS
            || dx > n * calculator::DELTA_PLUS
    }

    /// `Take(in RealValue x, in RealValue y)` (virtual)
    pub fn take(&self, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::take(self, x, y);
        }
        let dx = x.d.round();
        let dy = y.d.round();
        if self.out_of_limits(dx, dy) {
            return Ok(RealValue::NAN);
        }
        self.try_get(dy as usize - 1, dx as usize - 1)
    }

    /// `Line(in RealValue x, in RealValue y)` (virtual)
    pub fn line(&self, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::line(self, x, y);
        }
        let (dx, dy) = (x.d, y.d);
        if self.out_of_limits(dx, dy) {
            return Ok(RealValue::NAN);
        }
        let (m, n) = (self.row_count() as f64, self.col_count() as f64);
        let j = dx.floor() as usize;
        let i = dy.floor() as usize;
        let z11 = self.try_get(i - 1, j - 1)?;
        let lerp = |z1: &RealValue, z2: &RealValue, t: f64| -> CResult<RealValue> {
            RealValue::add(z1, &RealValue::sub(z2, z1)?.mul_f(t))
        };
        if j as f64 == dx || dx >= n {
            if i as f64 == dy || dy >= m {
                return Ok(z11);
            }
            let z21 = self.try_get(i, j - 1)?;
            return lerp(&z11, &z21, dy - i as f64);
        }
        if i as f64 == dy || dy >= m {
            let z12 = self.try_get(i - 1, j)?;
            return lerp(&z11, &z12, dx - j as f64);
        }
        let z12 = self.try_get(i - 1, j)?;
        let z21 = self.try_get(i, j - 1)?;
        let z22 = self.try_get(i, j)?;
        let z1 = lerp(&z11, &z12, dx - j as f64)?;
        let z2 = lerp(&z21, &z22, dx - j as f64)?;
        lerp(&z1, &z2, dy - i as f64)
    }

    /// `Spline(in RealValue x, in RealValue y)` (virtual)
    pub fn spline(&self, x: &RealValue, y: &RealValue) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::spline(self, x, y);
        }
        let (dx, dy) = (x.d, y.d);
        if self.out_of_limits(dx, dy) {
            return Ok(RealValue::NAN);
        }
        let m = self.row_count() as i64;
        let j = dx.floor() as i64 - 1;
        let i = dy.floor() as i64 - 1;
        let v = self.spline_row(i, j, dx)?;
        if i as f64 == dy || dy >= m as f64 {
            return Ok(v);
        }
        let u = v.units.clone();
        let z0 = v.d;
        let v = self.spline_row(i + 1, j, dx)?;
        let z1 = v.d * Unit::convert(&u, &v.units, ',')?;
        let mut dz = z1 - z0;
        let mut a = dz;
        let mut b = dz;
        let sign = crate::base_types::complex::math_sign;
        dz = sign(dz)? as f64;
        if i > 0 {
            let v = self.spline_row(i - 1, j, dx)?;
            let z2 = v.d * Unit::convert(&u, &v.units, ',')?;
            a = (z1 - z2) * if sign(z0 - z2)? as f64 == dz { 0.5 } else { 0.25 };
        }
        if i < m - 2 {
            let v = self.spline_row(i + 2, j, dx)?;
            let z2 = v.d * Unit::convert(&u, &v.units, ',')?;
            b = (z2 - z0) * if sign(z2 - z1)? as f64 == dz { 0.5 } else { 0.25 };
        }
        if i == 0 {
            a += (a - b) / 2.0;
        }
        if i == m - 2 {
            b += (b - a) / 2.0;
        }
        let t = dy - i as f64 - 1.0;
        let z = z0 + ((z1 - z0) * (3.0 - 2.0 * t) * t + ((a + b) * t - a) * (t - 1.0)) * t;
        Ok(RealValue::new(z, u))
    }

    fn spline_row(&self, i: i64, j: i64, dx: f64) -> CResult<RealValue> {
        let n = self.col_count() as i64;
        let at = |i: i64, j: i64| -> CResult<RealValue> {
            if i < 0 || j < 0 {
                return Err(vector::index_was_outside());
            }
            self.try_get(i as usize, j as usize)
        };
        let v = at(i, j)?;
        if j as f64 == dx || dx >= n as f64 {
            return Ok(v);
        }
        let u = v.units.clone();
        let y0 = v.d;
        let v = at(i, j + 1)?;
        let y1 = v.d * Unit::convert(&u, &v.units, ',')?;
        let mut dy = y1 - y0;
        let mut a = dy;
        let mut b = dy;
        let sign = crate::base_types::complex::math_sign;
        dy = sign(dy)? as f64;
        if j > 0 {
            let v = at(i, j - 1)?;
            let y2 = v.d * Unit::convert(&u, &v.units, ',')?;
            a = (y1 - y2) * if sign(y0 - y2)? as f64 == dy { 0.5 } else { 0.25 };
        }
        if j < n - 2 {
            let v = at(i, j + 2)?;
            let y2 = v.d * Unit::convert(&u, &v.units, ',')?;
            b = (y2 - y0) * if sign(y2 - y1)? as f64 == dy { 0.5 } else { 0.25 };
        }
        if j == 0 {
            a += (a - b) / 2.0;
        }
        if j == n - 2 {
            b += (b - a) / 2.0;
        }
        let t = dx - j as f64 - 1.0;
        let y = y0 + ((y1 - y0) * (3.0 - 2.0 * t) * t + ((a + b) * t - a) * (t - 1.0)) * t;
        Ok(RealValue::new(y, u))
    }

    /// `Transpose()` (virtual)
    pub fn transpose(&self) -> CResult<Matrix> {
        if self.is_hp() {
            return hp_matrix::transpose(self);
        }
        match self.mtype() {
            MatrixType::Column => return column_matrix::transpose(self),
            MatrixType::Diagonal => return diagonal_matrix::transpose(self),
            MatrixType::LowerTriangular => return lower_triangular_matrix::transpose(self),
            MatrixType::UpperTriangular => return upper_triangular_matrix::transpose(self),
            MatrixType::Symmetric => return symmetric_matrix::transpose(self),
            MatrixType::Full => {}
        }
        if self.row_count() == 1 {
            return column_matrix::from_vector(&self.row_ref(0));
        }
        let (m, n) = (self.row_count(), self.col_count());
        let mm = Matrix::full(n, m);
        for i in (0..n).rev() {
            mm.set_row_ref(i, self.col(i as i64 + 1)?);
        }
        Ok(mm)
    }

    /// `Trace()` (virtual)
    pub fn trace(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::trace(self);
        }
        let n = self.row_count();
        if n != self.col_count() {
            return Err(exceptions::MatrixNotSquare());
        }
        let mut trace = self.try_get(0, 0)?;
        for i in 1..n {
            trace = RealValue::add(&trace, &self.get(i, i))?;
        }
        Ok(trace)
    }

    /// `FrobNorm()`
    pub fn frob_norm(&self) -> CResult<RealValue> {
        self.srss()
    }

    /// `L1Norm()` (virtual)
    pub fn l1_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::l1_norm(self);
        }
        let mut norm = column_vector::new(self, 0).l1_norm()?;
        for j in 1..self.col_count() {
            let col_norm = column_vector::new(self, j).l1_norm()?;
            if vector::real_compare_to(&col_norm, &norm)? > 0 {
                norm = col_norm;
            }
        }
        Ok(norm)
    }

    /// `InfNorm()` (virtual)
    pub fn inf_norm(&self) -> CResult<RealValue> {
        if self.is_hp() {
            return hp_matrix::inf_norm(self);
        }
        match self.mtype() {
            MatrixType::Column | MatrixType::Diagonal => return self.row_ref(0).inf_norm(),
            MatrixType::Symmetric => return symmetric_matrix::inf_norm(self),
            _ => {}
        }
        let rows = self.rows();
        if rows.is_empty() {
            return Err(vector::index_was_outside());
        }
        let mut norm = rows[0].l1_norm()?;
        for r in &rows[1..] {
            let row_norm = r.l1_norm()?;
            if vector::real_compare_to(&row_norm, &norm)? > 0 {
                norm = row_norm;
            }
        }
        Ok(norm)
    }

    /// `CopyTo(Matrix target, int i, int j)` (virtual), 1 based
    pub fn copy_to(&self, target: &Matrix, i: i64, j: i64) -> CResult<Matrix> {
        if self.is_hp() && target.is_hp() {
            return hp_matrix::copy_to(self, target, i, j);
        }
        let (i, j) = (i - 1, j - 1);
        let m = (self.row_count() as i64).min(target.row_count() as i64 - i);
        let n = (self.col_count() as i64).min(target.col_count() as i64 - j);
        let tt = target.mtype();
        for ii in 0..m {
            let i1 = i + ii;
            for jj in 0..n {
                let j1 = j + jj;
                if is_target(tt, i1, j1) {
                    target.set(i1 as usize, j1 as usize, self.get(ii as usize, jj as usize))?;
                }
            }
        }
        Ok(target.clone())
    }

    /// `AddTo(Matrix target, int i, int j)` (virtual), 1 based
    pub fn add_to(&self, target: &Matrix, i: i64, j: i64) -> CResult<Matrix> {
        if self.is_hp() && target.is_hp() {
            return hp_matrix::add_to(self, target, i, j);
        }
        let (i, j) = (i - 1, j - 1);
        let m = (self.row_count() as i64).min(target.row_count() as i64 - i);
        let n = (self.col_count() as i64).min(target.col_count() as i64 - j);
        let tt = target.mtype();
        for ii in 0..m {
            let i1 = i + ii;
            for jj in 0..n {
                let j1 = j + jj;
                if is_target(tt, i1, j1) {
                    let v = target.get(i1 as usize, j1 as usize);
                    let w = self.get(ii as usize, jj as usize);
                    if v.equals(&RealValue::ZERO) {
                        target.set(i1 as usize, j1 as usize, w)?;
                    } else {
                        target.set(i1 as usize, j1 as usize, RealValue::add(&v, &w)?)?;
                    }
                }
            }
        }
        Ok(target.clone())
    }

    /// `Diagonal()` (virtual)
    pub fn diagonal(&self) -> CResult<Vector> {
        if self.is_hp() {
            return hp_matrix::diagonal(self);
        }
        let n = self.row_count().min(self.col_count());
        let v = Vector::dense(n);
        for i in 0..n {
            v.raw_set(i, self.get(i, i));
        }
        Ok(v)
    }

    /// `CreateFromCols(Vector[] cols, int m)`
    pub fn create_from_cols(cols: &[Vector], m: usize) -> CResult<Matrix> {
        let n = cols.len();
        let mm = Matrix::new(m, n)?;
        for j in (0..n).rev() {
            mm.set_col(j, &cols[j])?;
        }
        Ok(mm)
    }

    /// `CreateFromRows(Vector[] rows, int n)`
    pub fn create_from_rows(rows: &[Vector], n: usize) -> CResult<Matrix> {
        let m = rows.len();
        let mm = Matrix::new(m, n)?;
        for i in (0..m).rev() {
            mm.set_row(i, &rows[i])?;
        }
        Ok(mm)
    }

    /// `CheckCount(ref int count)`: returns true for reverse order.
    pub(crate) fn check_count(&self, count: &mut i64) -> bool {
        if *count < 0 {
            *count = -*count;
            return true;
        }
        if *count == 0 || *count > self.row_count() as i64 {
            *count = self.row_count() as i64;
        }
        false
    }

    /// `Identity(int n)`
    pub(crate) fn identity(n: usize) -> CResult<Matrix> {
        let mm = Matrix::new(n, n)?;
        for i in 0..n {
            mm.set(i, i, RealValue::ONE)?;
        }
        Ok(mm)
    }
}

/// `IsTarget(MatrixType type, int i, int j)`
fn is_target(t: MatrixType, i: i64, j: i64) -> bool {
    match t {
        MatrixType::Full => true,
        MatrixType::Symmetric | MatrixType::UpperTriangular => j >= i,
        MatrixType::LowerTriangular => j <= i,
        MatrixType::Diagonal => i == j,
        MatrixType::Column => true,
    }
}
