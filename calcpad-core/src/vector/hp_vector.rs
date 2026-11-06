//! Port of Vector/HpVector.cs: a high performance vector of doubles that share one unit.
//!
//! This file holds the storage and the members the other vector and matrix types use.
//! The HpVector overrides of the virtual `Vector` methods are dispatched from
//! `vector/mod.rs` to the `hp_*` methods at the end of this file.
use super::{double_compare_to, Relation, VecData, Vector, MAX_LENGTH};
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::calculator::{self, RealFn, RealOp};
use crate::exceptions;
use crate::extension_methods::almost_equals;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::matrix::hp_matrix::{self, vectorized};

/// `System.Numerics.Vector<double>.Count` on the x64 (AVX2) machines that produced the
/// reference output.
pub const VEC_SIZE: usize = 4;

#[derive(Debug, Clone)]
pub struct HpData {
    /// `_values` (None while not allocated)
    pub values: Option<Vec<f64>>,
    pub size: usize,
    pub capacity: usize,
    pub length: usize,
    pub units: OptUnit,
}

impl HpData {
    pub fn new(length: usize, size: usize, units: OptUnit) -> HpData {
        HpData {
            values: Some(vec![0.0; size]),
            size,
            capacity: size,
            length,
            units,
        }
    }

    /// `GetValue(int index)`
    #[inline]
    pub fn get_value(&self, index: usize) -> f64 {
        if index >= self.size {
            0.0
        } else {
            self.values.as_ref().unwrap()[index]
        }
    }

    /// Indexer get
    pub fn get(&self, index: usize) -> RealValue {
        RealValue::new(self.get_value(index), self.units.clone())
    }

    fn expand_capacity(&mut self, index: usize) {
        self.capacity = 125 * index / 100 + 1;
        if self.capacity > self.length {
            self.capacity = self.length;
        }
        match &mut self.values {
            None => {
                if self.length <= 10 {
                    self.capacity = self.length;
                } else if index == 0 && self.capacity < self.length / 100 {
                    self.capacity = self.length / 100;
                }
                self.values = Some(vec![0.0; self.capacity]);
            }
            Some(v) => v.resize(self.capacity, 0.0),
        }
    }

    /// Indexer set (converts the value to the units of the vector)
    pub fn set(&mut self, index: usize, value: &RealValue) -> CResult<()> {
        if index >= self.size {
            if value.equals(&RealValue::ZERO) {
                return Ok(());
            }
            self.size = index + 1;
            if index >= self.capacity {
                self.expand_capacity(index);
            }
        }
        let d = Unit::convert(&self.units, &value.units, '.')?;
        let vals = self.values.as_mut().unwrap();
        if d == 1.0 {
            vals[index] = value.d;
        } else {
            vals[index] = value.d * d;
        }
        Ok(())
    }

    /// `SetValue(double value, int index)`
    pub fn set_value(&mut self, value: f64, index: usize) {
        if index >= self.size {
            if value == 0.0 {
                return;
            }
            self.size = index + 1;
            if index >= self.capacity {
                self.expand_capacity(index);
            }
        }
        self.values.as_mut().unwrap()[index] = value;
    }

    /// `Values`: note that C# leaves the element at `_size` with no units.
    pub fn values(&self) -> Vec<RealValue> {
        let mut values = vec![RealValue::ZERO; self.length];
        for (i, v) in values.iter_mut().enumerate().take(self.size) {
            *v = RealValue::new(self.values.as_ref().unwrap()[i], self.units.clone());
        }
        if self.units.is_some() {
            let zero = RealValue::new(0.0, self.units.clone());
            for v in values.iter_mut().skip(self.size + 1) {
                *v = zero.clone();
            }
        }
        values
    }

    /// `Fill(double value, int start, int len)`
    pub fn fill_range(&mut self, value: f64, start: usize, len: usize) {
        let end = start + len;
        if self.size < end {
            self.size = end;
        }
        if self.capacity < end {
            self.capacity = end;
            match &mut self.values {
                Some(v) => v.resize(end, 0.0),
                None => self.values = Some(vec![0.0; end]),
            }
        }
        for x in &mut self.values.as_mut().unwrap()[start..end] {
            *x = value;
        }
    }

    /// `Raw`: the whole `_values` array (empty when not allocated).
    pub fn raw_all(&self) -> &[f64] {
        match &self.values {
            Some(v) => v,
            None => &[],
        }
    }

    /// `Copy()` of the storage: capacity equal to the size.
    pub fn copy(&self) -> HpData {
        let mut c = HpData::new(self.length, self.size, self.units.clone());
        c.values.as_mut().unwrap()[..self.size].copy_from_slice(&self.raw_all()[..self.size]);
        c
    }

    /// `RawCopy(int length = -1)`
    pub fn raw_copy(&self, length: Option<usize>) -> Vec<f64> {
        let length = length.unwrap_or(self.length);
        let mut v = vec![0.0; length];
        v[..self.size].copy_from_slice(&self.raw_all()[..self.size]);
        v
    }

    /// `Resize(int newSize)` of the storage.
    pub fn resize(&mut self, new_size: usize) {
        if new_size != self.length {
            self.length = new_size;
            if self.size > self.length {
                self.size = self.length;
            }
            if self.capacity > self.length {
                self.capacity = self.length;
                if let Some(v) = self.values.as_mut() {
                    v.resize(self.length, 0.0);
                }
            }
        }
    }

    /// `Scale(double d)`
    pub fn scale(&mut self, d: f64) {
        let size = self.size;
        if let Some(v) = self.values.as_mut() {
            for x in &mut v[..size] {
                *x *= d;
            }
        }
    }

    /// The stored elements `_values[0.._size]`.
    pub fn raw(&self) -> &[f64] {
        match &self.values {
            Some(v) => &v[..self.size.min(v.len())],
            None => &[],
        }
    }
}

// ----- constructors --------------------------------------------------------------------------

/// `new HpVector(double[] values, Unit units)`
pub fn from_doubles(values: Vec<f64>, units: OptUnit) -> CResult<Vector> {
    if values.len() as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    let length = values.len();
    let mut size = 0;
    for i in (0..length).rev() {
        if values[i] != 0.0 {
            size = i + 1;
            break;
        }
    }
    Ok(Vector::from_data(VecData::Hp(HpData {
        values: Some(values),
        size,
        capacity: 0,
        length,
        units,
    })))
}

/// `new HpVector(RealValue[] values)`
pub fn from_values(values: &[RealValue]) -> CResult<Vector> {
    if values.len() as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    let length = values.len();
    let mut h = HpData {
        values: None,
        size: 0,
        capacity: 0,
        length,
        units: None,
    };
    if length > 0 {
        h.units = values[0].units.clone();
        for i in (0..length).rev() {
            h.set(i, &values[i])?;
        }
    }
    Ok(Vector::from_data(VecData::Hp(h)))
}

/// `new HpVector(int length, Unit units)`
pub fn new_length(length: usize, units: OptUnit) -> CResult<Vector> {
    if length as i64 > MAX_LENGTH {
        return Err(exceptions::VectorSizeLimit());
    }
    Ok(Vector::from_data(VecData::Hp(HpData {
        values: None,
        size: 0,
        capacity: 0,
        length,
        units,
    })))
}

/// `new HpVector(int length, int size, Unit units)`
pub fn new_ls(length: usize, size: usize, units: OptUnit) -> Vector {
    Vector::from_data(VecData::Hp(HpData::new(length, size, units)))
}

/// `new HpVector(Vector vec, Unit units)`
pub fn from_vector(vec: &Vector, units: OptUnit) -> CResult<Vector> {
    let mut h = HpData::new(vec.len(), vec.size(), units);
    if h.size == 0 {
        return Ok(Vector::from_data(VecData::Hp(h)));
    }
    let values = vec.raw().ok_or_else(super::null_reference)?;
    if h.units.is_none() {
        h.units = values[0].units.clone();
    }
    for i in (0..h.size).rev() {
        let v = &values[i];
        let d = v.d * Unit::convert(&h.units, &v.units, ',')?;
        h.values.as_mut().unwrap()[i] = d;
    }
    Ok(Vector::from_data(VecData::Hp(h)))
}

/// `FromIndexes(IEnumerable<int> indexes)`: 1 based indexes as an HpVector.
pub fn from_indexes(indexes: &[i32]) -> Vector {
    let n = indexes.len();
    let mut h = HpData::new(n, n, None);
    let vals = h.values.as_mut().unwrap();
    for (i, &index) in indexes.iter().enumerate() {
        vals[i] = (index + 1) as f64;
    }
    Vector::from_data(VecData::Hp(h))
}

/// `SetValues(int[] indexes)`: replaces the content with the given values.
pub fn set_values(v: &Vector, indexes: &[i32]) {
    v.hp_set_values(indexes);
}

fn hp(d: HpData) -> Vector {
    Vector::from_data(VecData::Hp(d))
}

/// A copy of the storage of an HpVector (operands may be the same object).
pub(crate) fn snap(v: &Vector) -> HpData {
    v.with_hp(|h| h.clone())
}

/// `Clone(HpVector a)`
fn clone1(a: &HpData) -> HpData {
    HpData::new(a.length, a.size, a.units.clone())
}

/// `Clone(HpVector a, HpVector b)`
fn clone2(a: &HpData, b: &HpData) -> HpData {
    HpData::new(a.length.max(b.length), a.size.max(b.size), a.units.clone())
}

fn min_max(n1: usize, n2: usize) -> (usize, usize) {
    if n1 <= n2 {
        (n1, n2)
    } else {
        (n2, n1)
    }
}

/// `double.IsEqual`, `IsNotEqual`, `IsLessThan`, ... (ExtensionMethods.cs)
pub(crate) fn is_equal(a: f64, b: f64) -> f64 {
    if almost_equals(a, b) {
        1.0
    } else {
        0.0
    }
}
pub(crate) fn is_not_equal(a: f64, b: f64) -> f64 {
    if almost_equals(a, b) {
        0.0
    } else {
        1.0
    }
}
pub(crate) fn is_less_than(a: f64, b: f64) -> f64 {
    if a < b && !almost_equals(a, b) {
        1.0
    } else {
        0.0
    }
}
pub(crate) fn is_greater_than(a: f64, b: f64) -> f64 {
    if a > b && !almost_equals(a, b) {
        1.0
    } else {
        0.0
    }
}
pub(crate) fn is_less_than_or_equal(a: f64, b: f64) -> f64 {
    if a <= b || almost_equals(a, b) {
        1.0
    } else {
        0.0
    }
}
pub(crate) fn is_greater_than_or_equal(a: f64, b: f64) -> f64 {
    if a >= b || almost_equals(a, b) {
        1.0
    } else {
        0.0
    }
}

const LOGICAL_ZERO: f64 = crate::base_types::real_value::LOGICAL_ZERO;

/// The SIMD `Remainder(va, vb)` of HpVector.cs: `a - t * b` with `t` the quotient
/// truncated towards zero.
#[inline]
fn simd_remainder(a: f64, b: f64) -> f64 {
    let q = a / b;
    let t = if q >= 0.0 { q.floor() } else { q.ceil() };
    a - t * b
}

/// Remainders of `n` elements: the first `n / 4 * 4` take the SIMD formula when `n > 15`.
fn remainders(n: usize, f: &mut dyn FnMut(usize, bool)) {
    let mut nv = 0;
    if n > 15 {
        nv = n / VEC_SIZE * VEC_SIZE;
        for i in 0..nv {
            f(i, true);
        }
    }
    for i in nv..n {
        f(i, false);
    }
}

fn rem_of(a: f64, b: f64, simd: bool) -> f64 {
    if simd {
        simd_remainder(a, b)
    } else {
        a % b
    }
}

fn text(u: &OptUnit) -> String {
    Unit::get_text_of(u)
}

// ----- arithmetic operators ---------------------------------------------------------------------

/// `-HpVector`
pub fn neg(a: &Vector) -> Vector {
    let a = snap(a);
    let mut b = clone1(&a);
    let sa = a.raw_all();
    let sb = b.values.as_mut().unwrap();
    for i in 0..a.size {
        sb[i] = -sa[i];
    }
    hp(b)
}

/// `HpVector - HpVector`
pub fn sub(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    let d = Unit::convert(&a.units, &b.units, '-')?;
    let (na, nb) = (a.size, b.size);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in 0..n1 {
        sc[i] = sa[i] - sb[i] * d;
    }
    if na > nb {
        sc[n1..n2].copy_from_slice(&sa[n1..n2]);
    } else if nb > na {
        for i in (n1..n2).rev() {
            sc[i] = -sb[i] * d;
        }
    }
    Ok(hp(c))
}

/// `HpVector - RealValue`
pub fn sub_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    let mut c = clone1(&a);
    let db = b.d * Unit::convert(&a.units, &b.units, '-')?;
    let (na, nc) = (a.size, c.length);
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..na {
            sc[i] = sa[i] - db;
        }
    }
    if nc > na && db != 0.0 {
        c.fill_range(-db, na, nc - na);
    }
    Ok(hp(c))
}

/// `RealValue - HpVector`
pub fn sub_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    let b = snap(b);
    let mut c = clone1(&b);
    c.units = a.units.clone();
    let da = a.d;
    let k = Unit::convert(&a.units, &b.units, '-')?;
    let (nb, nc) = (b.size, c.length);
    let sb = b.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..nb {
            sc[i] = da - sb[i] * k;
        }
    }
    if nc > nb && da != 0.0 {
        c.fill_range(da, nb, nc - nb);
    }
    Ok(hp(c))
}

/// `HpVector + HpVector`
pub fn add(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    let d = Unit::convert(&a.units, &b.units, '+')?;
    let (na, nb) = (a.size, b.size);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in 0..n1 {
        sc[i] = sa[i] + sb[i] * d;
    }
    if na > nb {
        sc[n1..n2].copy_from_slice(&sa[n1..n2]);
    } else if nb > na {
        for i in (n1..n2).rev() {
            sc[i] = sb[i] * d;
        }
    }
    Ok(hp(c))
}

/// `HpVector + RealValue` (C# converts the units with the '-' operator symbol)
pub fn add_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    let mut c = clone1(&a);
    let db = b.d * Unit::convert(&a.units, &b.units, '-')?;
    let (na, nc) = (a.size, c.length);
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..na {
            sc[i] = sa[i] + db;
        }
    }
    if nc > na && db != 0.0 {
        c.fill_range(db, na, nc - na);
    }
    Ok(hp(c))
}

/// `HpVector * HpVector`
pub fn mul(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let n1 = a.size.min(b.size);
    let n2 = a.length.max(b.length);
    let (u, d) = Unit::multiply(&a.units, &b.units, true);
    let mut c = HpData::new(n2, n1, u);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in 0..n1 {
        sc[i] = sa[i] * sb[i] * d;
    }
    Ok(hp(c))
}

/// `HpVector * RealValue`
pub fn mul_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    let na = if b.d == 0.0 { 0 } else { a.size };
    let (u, mut db) = Unit::multiply(&a.units, &b.units, true);
    let mut c = HpData::new(a.length, na, u);
    if na == 0 {
        return Ok(hp(c));
    }
    db *= b.d;
    let sa = a.raw_all();
    let sc = c.values.as_mut().unwrap();
    for i in 0..na {
        sc[i] = sa[i] * db;
    }
    Ok(hp(c))
}

/// `HpVector * double`
pub fn mul_d(a: &HpData, b: f64) -> HpData {
    let na = if b == 0.0 { 0 } else { a.size };
    let mut c = HpData::new(a.length, na, a.units.clone());
    if na == 0 {
        return c;
    }
    let sa = a.raw_all();
    let sc = c.values.as_mut().unwrap();
    for i in 0..na {
        sc[i] = sa[i] * b;
    }
    c
}

/// `HpVector / HpVector`
pub fn div(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    let (cu, d) = Unit::divide(&a.units, &b.units, true);
    c.units = cu;
    let (na, nb, nc) = (a.size, b.size, c.length);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..n1 {
            sc[i] = sa[i] / sb[i] * d;
        }
        if na > nb {
            for i in (n1..n2).rev() {
                sc[i] = sa[i] / 0.0;
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                sc[i] = 0.0 / sb[i];
            }
        }
    }
    if nc > n2 {
        c.fill_range(f64::NAN, n2, nc - n2);
    }
    Ok(hp(c))
}

/// `HpVector / RealValue`
pub fn div_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    let mut c = clone1(&a);
    let (cu, mut db) = Unit::divide(&a.units, &b.units, true);
    c.units = cu;
    db /= b.d;
    let (na, nc) = (a.size, c.length);
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..na {
            sc[i] = sa[i] * db;
        }
    }
    if nc > na {
        let v = 0.0 * db;
        if v != 0.0 {
            c.fill_range(v, na, nc - na);
        }
    }
    Ok(hp(c))
}

/// `RealValue / HpVector`
pub fn div_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    let b = snap(b);
    let mut c = clone1(&b);
    let (cu, mut da) = Unit::divide(&a.units, &b.units, true);
    c.units = cu;
    da *= a.d;
    let (nb, nc) = (b.size, c.length);
    let sb = b.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in 0..nb {
            sc[i] = da / sb[i];
        }
    }
    if nc > nb {
        c.fill_range(da / 0.0, nb, nc - nb);
    }
    Ok(hp(c))
}

/// `HpVector % HpVector`
pub fn rem(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    if b.units.is_some() {
        return Err(exceptions::CannotEvaluateRemainder(&text(&a.units), &text(&b.units)));
    }
    let mut c = clone2(&a, &b);
    let (na, nb, nc) = (a.size, b.size, c.length);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    {
        let sc = c.values.as_mut().unwrap();
        remainders(n1, &mut |i, simd| sc[i] = rem_of(sa[i], sb[i], simd));
    }
    if na > nb {
        c.fill_range(f64::NAN, n1, n2 - n1);
    } else if nb > na {
        let sc = c.values.as_mut().unwrap();
        for i in (n1..n2).rev() {
            sc[i] = 0.0 % sb[i];
        }
    }
    if nc > n2 {
        c.fill_range(f64::NAN, n2, nc - n2);
    }
    Ok(hp(c))
}

/// `HpVector % RealValue` (C# fills the tail with `b`)
pub fn rem_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    if b.units.is_some() {
        return Err(exceptions::CannotEvaluateRemainder(&text(&a.units), &text(&b.units)));
    }
    let mut c = clone1(&a);
    let db = b.d;
    let (na, nc) = (a.size, c.length);
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        remainders(na, &mut |i, simd| sc[i] = rem_of(sa[i], db, simd));
    }
    if nc > na && db != 0.0 {
        c.fill_range(db, na, nc - na);
    }
    Ok(hp(c))
}

/// `RealValue % HpVector`
pub fn rem_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    let b = snap(b);
    if b.units.is_some() {
        return Err(exceptions::CannotEvaluateRemainder(&text(&a.units), &text(&b.units)));
    }
    let mut c = clone1(&b);
    c.units = a.units.clone();
    let da = a.d;
    let (nb, nc) = (b.size, c.length);
    let sb = b.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        remainders(nb, &mut |i, simd| sc[i] = rem_of(da, sb[i], simd));
    }
    if nc > nb {
        c.fill_range(f64::NAN, nb, nc - nb);
    }
    Ok(hp(c))
}

// ----- relational and logical operators ------------------------------------------------------

/// HpVector op HpVector for the relational operators.
fn cmp_vv(a: &Vector, b: &Vector, op: char, f: fn(f64, f64) -> f64, fill_one: bool) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    c.units = None;
    let d = Unit::convert(&a.units, &b.units, op)?;
    let (na, nb, nc) = (a.size, b.size, c.length);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..n1).rev() {
            sc[i] = f(sa[i], sb[i] * d);
        }
        if na > nb {
            for i in (n1..n2).rev() {
                sc[i] = f(sa[i], 0.0);
            }
        } else if nb > na {
            for i in (n1..n2).rev() {
                sc[i] = f(0.0, sb[i] * d);
            }
        }
    }
    if fill_one && nc > n2 {
        c.fill_range(1.0, n2, nc - n2);
    }
    Ok(hp(c))
}

/// HpVector op RealValue for the relational operators: the tail is `f(0, b)`.
fn cmp_vr(a: &Vector, b: &RealValue, op: char, f: fn(f64, f64) -> f64, tail_db_first: bool) -> CResult<Vector> {
    let a = snap(a);
    let mut c = clone1(&a);
    c.units = None;
    let db = Unit::convert(&a.units, &b.units, op)? * b.d;
    let (na, nc) = (a.size, c.length);
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..na).rev() {
            sc[i] = f(sa[i], db);
        }
    }
    if nc > na {
        let v = if tail_db_first { f(db, 0.0) } else { f(0.0, db) };
        if v != 0.0 {
            c.fill_range(v, na, nc - na);
        }
    }
    Ok(hp(c))
}

/// RealValue op HpVector for the relational operators: the tail is `f(a, 0)`.
fn cmp_rv(a: &RealValue, b: &Vector, op: Option<char>, f: fn(f64, f64) -> f64) -> CResult<Vector> {
    let b = snap(b);
    let mut c = clone1(&b);
    c.units = None;
    let da = a.d;
    let k = match op {
        Some(op) => Unit::convert(&a.units, &b.units, op)?,
        None => 1.0,
    };
    let (nb, nc) = (b.size, c.length);
    let sb = b.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..nb).rev() {
            sc[i] = if op.is_some() { f(da, sb[i] * k) } else { f(da, sb[i]) };
        }
    }
    if nc > nb {
        let v = f(da, 0.0);
        if v != 0.0 {
            c.fill_range(v, nb, nc - nb);
        }
    }
    Ok(hp(c))
}

pub fn eq(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '\u{2261}', is_equal, true)
}
pub fn eq_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '\u{2261}', is_equal, true)
}
pub fn ne(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '\u{2260}', is_not_equal, false)
}
pub fn ne_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '\u{2260}', is_not_equal, true)
}
pub fn lt(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '<', is_less_than, false)
}
pub fn lt_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '<', is_less_than, false)
}
pub fn lt_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    cmp_rv(a, b, Some('<'), is_less_than)
}
pub fn gt(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '>', is_greater_than, false)
}
pub fn gt_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '>', is_greater_than, false)
}
pub fn gt_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    cmp_rv(a, b, Some('>'), is_greater_than)
}
pub fn le(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '\u{2264}', is_less_than_or_equal, true)
}
pub fn le_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '\u{2264}', is_less_than_or_equal, false)
}
/// `RealValue <= HpVector`: C# does not convert the units here.
pub fn le_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    cmp_rv(a, b, None, is_less_than_or_equal)
}
pub fn ge(a: &Vector, b: &Vector) -> CResult<Vector> {
    cmp_vv(a, b, '\u{2265}', is_greater_than_or_equal, false)
}
pub fn ge_vr(a: &Vector, b: &RealValue) -> CResult<Vector> {
    cmp_vr(a, b, '\u{2265}', is_greater_than_or_equal, false)
}
pub fn ge_rv(a: &RealValue, b: &Vector) -> CResult<Vector> {
    cmp_rv(a, b, Some('\u{2265}'), is_greater_than_or_equal)
}

/// `HpVector & HpVector`
pub fn and(a: &Vector, b: &Vector) -> Vector {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    c.units = None;
    let n = a.size.min(b.size);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in (0..n).rev() {
        sc[i] = if sa[i].abs() < LOGICAL_ZERO || sb[i].abs() < LOGICAL_ZERO {
            0.0
        } else {
            1.0
        };
    }
    hp(c)
}

/// `HpVector & RealValue`
pub fn and_vr(a: &Vector, b: &RealValue) -> Vector {
    let a = snap(a);
    let mut c = clone1(&a);
    c.units = None;
    let na = a.size;
    if b.d.abs() < LOGICAL_ZERO {
        c.fill_range(0.0, 0, na);
    } else {
        let sa = a.raw_all();
        let sc = c.values.as_mut().unwrap();
        for i in (0..na).rev() {
            sc[i] = if sa[i].abs() < LOGICAL_ZERO { 0.0 } else { 1.0 };
        }
    }
    hp(c)
}

/// `HpVector | HpVector`
pub fn or(a: &Vector, b: &Vector) -> Vector {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    c.units = None;
    let (na, nb) = (a.size, b.size);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in (0..n1).rev() {
        sc[i] = if sa[i].abs() >= LOGICAL_ZERO || sb[i].abs() >= LOGICAL_ZERO {
            1.0
        } else {
            0.0
        };
    }
    if na > nb {
        for i in (n1..n2).rev() {
            sc[i] = if sa[i].abs() >= LOGICAL_ZERO { 1.0 } else { 0.0 };
        }
    } else if nb > na {
        for i in (n1..n2).rev() {
            sc[i] = if sb[i].abs() >= LOGICAL_ZERO { 1.0 } else { 0.0 };
        }
    }
    hp(c)
}

/// `HpVector | RealValue` (C# compares the values without `Math.Abs` here)
pub fn or_vr(a: &Vector, b: &RealValue) -> Vector {
    let a = snap(a);
    let mut c = clone1(&a);
    c.units = None;
    let (na, nc) = (a.size, c.length);
    let v = b.d.abs() >= LOGICAL_ZERO;
    if v {
        c.fill_range(1.0, 0, na);
    } else {
        let sa = a.raw_all();
        let sc = c.values.as_mut().unwrap();
        for i in (0..na).rev() {
            sc[i] = if sa[i] >= LOGICAL_ZERO { 1.0 } else { 0.0 };
        }
    }
    if nc > na && v {
        c.fill_range(1.0, na, nc - na);
    }
    hp(c)
}

/// `HpVector ^ HpVector` (logical xor)
pub fn xor(a: &Vector, b: &Vector) -> Vector {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    c.units = None;
    let (na, nb) = (a.size, b.size);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let sc = c.values.as_mut().unwrap();
    for i in (0..n1).rev() {
        sc[i] = if (sa[i].abs() >= LOGICAL_ZERO) != (sb[i].abs() >= LOGICAL_ZERO) {
            1.0
        } else {
            0.0
        };
    }
    if na > nb {
        for i in (n1..n2).rev() {
            sc[i] = if sa[i].abs() >= LOGICAL_ZERO { 1.0 } else { 0.0 };
        }
    } else if nb > na {
        for i in (n1..n2).rev() {
            sc[i] = if sb[i].abs() >= LOGICAL_ZERO { 1.0 } else { 0.0 };
        }
    }
    hp(c)
}

/// `HpVector ^ RealValue` (logical xor)
pub fn xor_vr(a: &Vector, b: &RealValue) -> Vector {
    let a = snap(a);
    let mut c = clone1(&a);
    c.units = None;
    let (na, nc) = (a.size, c.length);
    let v = b.d.abs() >= LOGICAL_ZERO;
    {
        let sa = a.raw_all();
        let sc = c.values.as_mut().unwrap();
        for i in (0..na).rev() {
            sc[i] = if (sa[i].abs() >= LOGICAL_ZERO) != v { 1.0 } else { 0.0 };
        }
    }
    if nc > na && v {
        c.fill_range(1.0, na, nc - na);
    }
    hp(c)
}

// ----- EvaluateOperator / EvaluateFunction -----------------------------------------------------

/// `HpVector.EvaluateOperator(op, HpVector a, HpVector b, index)`
pub fn evaluate_operator(op: RealOp, a: &Vector, b: &Vector, index: i64) -> CResult<Vector> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div(a, b),
        '\u{29BC}' => rem(a, b),
        '*' => mul(a, b),
        '-' => sub(a, b),
        '+' => add(a, b),
        '<' => lt(a, b),
        '>' => gt(a, b),
        '\u{2264}' => le(a, b),
        '\u{2265}' => ge(a, b),
        '\u{2261}' => eq(a, b),
        '\u{2260}' => ne(a, b),
        '\u{2227}' => Ok(and(a, b)),
        '\u{2228}' => Ok(or(a, b)),
        '\u{2295}' => Ok(xor(a, b)),
        _ => evaluate_operator2(op, a, b),
    }
}

/// `HpVector.EvaluateOperator(op, HpVector a, in RealValue b, index)`
pub fn evaluate_operator_vr(op: RealOp, a: &Vector, b: &RealValue, index: i64) -> CResult<Vector> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div_vr(a, b),
        '\u{29BC}' => rem_vr(a, b),
        '*' => mul_vr(a, b),
        '-' => sub_vr(a, b),
        '+' => add_vr(a, b),
        '<' => lt_vr(a, b),
        '>' => gt_vr(a, b),
        '\u{2264}' => le_vr(a, b),
        '\u{2265}' => ge_vr(a, b),
        '\u{2261}' => eq_vr(a, b),
        '\u{2260}' => ne_vr(a, b),
        '\u{2227}' => Ok(and_vr(a, b)),
        '\u{2228}' => Ok(or_vr(a, b)),
        '\u{2295}' => Ok(xor_vr(a, b)),
        _ => evaluate_operator2_vr(op, a, b),
    }
}

/// `HpVector.EvaluateOperator(op, in RealValue a, HpVector b, index)`
pub fn evaluate_operator_rv(op: RealOp, a: &RealValue, b: &Vector, index: i64) -> CResult<Vector> {
    match calculator::get_operator_symbol(index) {
        '/' | '\u{00F7}' => div_rv(a, b),
        '\u{29BC}' => rem_rv(a, b),
        '*' => mul_vr(b, a),
        '-' => sub_rv(a, b),
        '+' => add_vr(b, a),
        '<' => lt_rv(a, b),
        '>' => gt_rv(a, b),
        '\u{2264}' => le_rv(a, b),
        '\u{2265}' => ge_rv(a, b),
        '\u{2261}' => eq_vr(b, a),
        '\u{2260}' => ne_vr(b, a),
        '\u{2227}' => Ok(and_vr(b, a)),
        '\u{2228}' => Ok(or_vr(b, a)),
        '\u{2295}' => Ok(xor_vr(b, a)),
        _ => evaluate_operator2_rv(op, a, b),
    }
}

fn evaluate_operator2(op: RealOp, a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let mut c = clone2(&a, &b);
    let (na, nb, nc) = (a.size, b.size, c.length);
    let (n1, n2) = min_max(na, nb);
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let (ua, ub) = (a.units.clone(), b.units.clone());
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..n1).rev() {
            sc[i] = op
                .apply(&RealValue::new(sa[i], ua.clone()), &RealValue::new(sb[i], ub.clone()))?
                .d;
        }
    }
    let mut zero = RealValue::ZERO;
    if na > nb {
        if !Unit::ptr_eq(&ua, &zero.units) {
            zero = RealValue::new(0.0, ua.clone());
        }
        let sc = c.values.as_mut().unwrap();
        for i in (n1..n2).rev() {
            sc[i] = op.apply(&RealValue::new(sa[i], ua.clone()), &zero)?.d;
        }
    } else if nb > na {
        if !Unit::ptr_eq(&ub, &zero.units) {
            zero = RealValue::new(0.0, ub.clone());
        }
        let sc = c.values.as_mut().unwrap();
        for i in (n1..n2).rev() {
            sc[i] = op.apply(&zero, &RealValue::new(sb[i], ub.clone()))?.d;
        }
    }
    if nc > n2 {
        let d = op.apply(&zero, &zero)?.d;
        if d != 0.0 {
            c.fill_range(d, n2, nc - n2);
        }
    }
    c.units = op.apply(&RealValue::unit(ua), &RealValue::unit(ub))?.units;
    Ok(hp(c))
}

fn evaluate_operator2_vr(op: RealOp, a: &Vector, b: &RealValue) -> CResult<Vector> {
    let a = snap(a);
    let mut c = clone1(&a);
    let (na, nc) = (a.size, c.length);
    let ua = a.units.clone();
    if nc > na {
        let zero = RealValue::new(0.0, ua.clone());
        let v = op.apply(&zero, b)?.d;
        if v != 0.0 {
            c.fill_range(v, na, nc - na);
        }
    }
    let sa = a.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..na).rev() {
            sc[i] = op.apply(&RealValue::new(sa[i], ua.clone()), b)?.d;
        }
    }
    c.units = op.apply(&RealValue::unit(ua), b)?.units;
    Ok(hp(c))
}

fn evaluate_operator2_rv(op: RealOp, a: &RealValue, b: &Vector) -> CResult<Vector> {
    let b = snap(b);
    let mut c = clone1(&b);
    let (nb, nc) = (b.size, c.length);
    let ub = b.units.clone();
    if nc > nb {
        let zero = RealValue::new(0.0, ub.clone());
        let d = op.apply(a, &zero)?.d;
        if d != 0.0 {
            c.fill_range(d, nb, nc - nb);
        }
    }
    let sb = b.raw_all();
    {
        let sc = c.values.as_mut().unwrap();
        for i in (0..nb).rev() {
            sc[i] = op.apply(a, &RealValue::new(sb[i], ub.clone()))?.d;
        }
    }
    c.units = op.apply(a, &RealValue::unit(ub))?.units;
    Ok(hp(c))
}

/// `HpVector.EvaluateFunction(f, HpVector a)`
pub fn evaluate_function(f: RealFn, a: &Vector) -> CResult<Vector> {
    let a = snap(a);
    let mut b = clone1(&a);
    let (na, nb) = (a.size, b.length);
    let ua = a.units.clone();
    let ub = f.apply(&RealValue::new(1.0, ua.clone()))?.units;
    if na > 0 {
        let sa = a.raw_all();
        let sb = b.values.as_mut().unwrap();
        for i in (0..na).rev() {
            let v = f.apply(&RealValue::new(sa[i], ua.clone()))?;
            sb[i] = v.d * Unit::convert(&ub, &v.units, ',')?;
        }
    }
    if nb > na {
        let fz = f.apply(&RealValue::new(0.0, ua.clone()))?;
        let d = fz.d * Unit::convert(&ub, &fz.units, ',')?;
        if d != 0.0 {
            b.fill_range(d, na, nb - na);
        }
    }
    b.units = ub;
    Ok(hp(b))
}

// ----- static members ----------------------------------------------------------------------------

/// `HpVector.CrossProduct(a, b)`
pub fn cross_product(a: &Vector, b: &Vector) -> CResult<Vector> {
    let (a, b) = (snap(a), snap(b));
    let (na, nb) = (a.length, b.length);
    if !(2..=3).contains(&na) || !(2..=3).contains(&nb) {
        return Err(exceptions::CrossProductVectorDimensions());
    }
    let (u, d) = Unit::multiply(&a.units, &b.units, true);
    let mut c = HpData::new(3, 3, u);
    // C# reads _values[0..na] and _values[0..nb]: a shorter array throws.
    let sa = a.raw_all();
    let sb = b.raw_all();
    if sa.len() < na || sb.len() < nb {
        return Err(MathParserException::unexpected(
            "Specified argument was out of the range of valid values.",
        ));
    }
    let sc = c.values.as_mut().unwrap();
    sc[2] = (sa[0] * sb[1] - sa[1] * sb[0]) * d;
    if na == 3 {
        sc[0] = -sa[2] * sb[1] * d;
        sc[1] = sa[2] * sb[0] * d;
    }
    if nb == 3 {
        sc[0] += sa[1] * sb[2] * d;
        sc[1] -= sa[0] * sb[2] * d;
    }
    Ok(hp(c))
}

/// `HpVector.RawDotProduct(a, b)`
pub fn raw_dot_product(a: &HpData, b: &HpData) -> f64 {
    let n1 = a.size.min(b.size);
    if n1 == 0 {
        return 0.0;
    }
    let (sa, sb) = (a.raw_all(), b.raw_all());
    let mut sum = 0.0;
    let mut nv = 0;
    if n1 > 15 {
        nv = n1 / VEC_SIZE;
        for i in 0..nv {
            sum += vectorized::vdot(sa, i * VEC_SIZE, sb, i * VEC_SIZE);
        }
        nv *= VEC_SIZE;
    }
    for i in nv..n1 {
        sum += sa[i] * sb[i];
    }
    sum
}

/// `HpVector.DotProduct(a, b)`
pub fn dot_product(a: &Vector, b: &Vector) -> CResult<RealValue> {
    let (a, b) = (snap(a), snap(b));
    Ok(dot_product_data(&a, &b))
}

pub(crate) fn dot_product_data(a: &HpData, b: &HpData) -> RealValue {
    let (u, d) = Unit::multiply(&a.units, &b.units, true);
    let sum = raw_dot_product(a, b);
    RealValue::new(sum * d, u)
}

/// `HpVector.Range(start, end, step)` (C# converts the step with the units of `start`)
pub fn range(start: &RealValue, end: &RealValue, step: &RealValue) -> CResult<Vector> {
    if almost_equals(step.d, 0.0) {
        return Err(exceptions::StepCannotBeZero());
    }
    let u = start.units.clone();
    let start_val = start.d;
    let end_val = end.d * Unit::convert(&u, &end.units, ':')?;
    let step_val = step.d * Unit::convert(&u, &start.units, ':')?;
    let mut len = ((end_val - start_val) / step_val) + 1.0;
    if len <= 1.0 {
        return from_doubles(vec![start_val, end_val], u);
    }
    if len > MAX_LENGTH as f64 {
        return Err(exceptions::VectorSizeLimit());
    }
    len = len.trunc();
    let n = len as usize;
    let mut values = vec![0.0; n];
    for (i, v) in values.iter_mut().enumerate() {
        *v = start_val + step_val * i as f64;
    }
    from_doubles(values, u)
}

/// `HpVector.Relate(double a, double b, Relation rel)`
pub fn relate(a: f64, b: f64, rel: Relation) -> bool {
    let c = double_compare_to(a, b);
    match rel {
        Relation::Equal => almost_equals(a, b),
        Relation::NotEqual => !almost_equals(a, b),
        Relation::LessThan => c < 0 && !almost_equals(a, b),
        Relation::LessOrEqual => c <= 0 || almost_equals(a, b),
        Relation::GreaterThan => c > 0 && !almost_equals(a, b),
        Relation::GreaterOrEqual => c >= 0 || almost_equals(a, b),
    }
}

fn find_all_indexes(h: &HpData, value: &RealValue, start: usize, rel: Relation) -> CResult<Vec<i32>> {
    if !Unit::is_consistent_opt(&h.units, &value.units) {
        return Ok(vec![]);
    }
    let d = value.d * Unit::convert(&h.units, &value.units, ',')?;
    let mut start = start;
    if start < 1 {
        start = 1;
    } else if start > h.size {
        if start > h.length {
            return Ok(vec![]);
        }
        if relate(d, 0.0, rel) {
            return Ok((h.size as i32..h.length as i32).collect());
        }
        return Ok(vec![]);
    }
    let vals = h.raw_all();
    let mut indexes = Vec::new();
    for i in start - 1..h.size {
        if relate(vals[i], d, rel) {
            indexes.push(i as i32);
        }
    }
    if h.size < h.length && relate(0.0, d, rel) {
        indexes.extend(h.size as i32..h.length as i32);
    }
    Ok(indexes)
}

/// `hpVec.Lookup(vecY, val, rel)` as VectorCalculator calls it. `vecY` is typed `Vector`
/// there, so C# binds the call to the base `Vector.Lookup(Vector dest, ...)`, not to
/// `HpVector.Lookup(HpVector dest, ...)`: the result is the base one (a dense vector).
pub fn lookup(x: &Vector, dest: &Vector, value: &RealValue, rel: Relation) -> CResult<Vector> {
    x.lookup(dest, value, rel)
}

/// `HpVector.Lookup(HpVector dest, RealValue value, Relation rel)` (only reachable with an
/// HpVector typed destination).
pub fn lookup_hp(x: &Vector, dest: &Vector, value: &RealValue, rel: Relation) -> CResult<Vector> {
    let xh = snap(x);
    let dh = snap(dest);
    let indexes = find_all_indexes(&xh, value, 1, rel)?;
    let n = indexes.len();
    let mut v = HpData::new(n, n, dh.units.clone());
    for (j, &i) in indexes.iter().enumerate() {
        let i = i as usize;
        if i > dh.length {
            return Err(exceptions::IndexOutOfRange(&(i + 1).to_string()));
        }
        v.values.as_mut().unwrap()[j] = dh.get_value(i);
    }
    Ok(hp(v))
}

/// `TryJoin(IValue[] items, int len, out HpVector result)`
pub fn try_join(items: &[Value], len: usize) -> CResult<Option<Vector>> {
    let units_of = |v: &Value| -> OptUnit {
        match v {
            Value::Vector(v) if v.is_hp() => v.hp_units(),
            Value::Real(r) => r.units.clone(),
            Value::Matrix(m) if m.is_hp() => hp_matrix::units(m),
            _ => None,
        }
    };
    if items.is_empty() {
        return Err(super::index_was_outside());
    }
    let n = items.len();
    let units = units_of(&items[0]);
    let mut factors = vec![1.0; n];
    for i in 1..n {
        let u = units_of(&items[i]);
        if !Unit::is_consistent_opt(&u, &units) {
            return Ok(None);
        }
        factors[i] = match (&units, &u) {
            (None, _) => 1.0,
            (Some(us), Some(uu)) => uu.convert_to(us),
            (Some(_), None) => return Err(super::null_reference()),
        };
    }
    let mut values = vec![0.0; len];
    let mut index = 0usize;
    for (k, item) in items.iter().enumerate() {
        match item {
            Value::Real(real) => {
                values[index] = real.d * factors[k];
                index += 1;
            }
            Value::Vector(vector) => {
                let h = snap(vector);
                let d = factors[k];
                let size = h.size;
                let vv = &h.raw_all()[..size];
                if d == 1.0 {
                    values[index..index + size].copy_from_slice(vv);
                } else {
                    // C# writes values[i] here (not values[index + i]).
                    for i in 0..size {
                        values[i] = vv[i] * d;
                    }
                }
                index += h.length;
            }
            Value::Matrix(matrix) => {
                let nc = matrix.col_count();
                let d = factors[k];
                for i in 0..matrix.row_count() {
                    for j in 0..nc {
                        values[index] = hp_matrix::get_value(matrix, i, j) * d;
                        index += 1;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(Some(from_doubles(values, units)?))
}

// ----- HpVector overrides of the virtual Vector members ---------------------------------------

impl Vector {
    /// Runs `f` on the HP storage of this vector.
    pub(crate) fn with_hp<R>(&self, f: impl FnOnce(&HpData) -> R) -> R {
        match &*self.data() {
            VecData::Hp(h) => f(h),
            _ => unreachable!("not an HpVector"),
        }
    }

    pub(crate) fn with_hp_mut<R>(&self, f: impl FnOnce(&mut HpData) -> R) -> R {
        match &mut *self.data_mut() {
            VecData::Hp(h) => f(h),
            _ => unreachable!("not an HpVector"),
        }
    }

    /// `HpVector.Units`
    pub fn hp_units(&self) -> OptUnit {
        self.with_hp(|h| h.units.clone())
    }

    /// `HpVector.Units = units`
    pub fn set_hp_units(&self, units: OptUnit) {
        self.with_hp_mut(|h| h.units = units);
    }

    /// `Scale(double d)`: multiplies the stored values by `d`.
    pub fn hp_scale(&self, d: f64) {
        self.with_hp_mut(|h| h.scale(d));
    }

    /// `Equals(HpVector)` (note the C# condition `_size != other._size`)
    pub(crate) fn hp_equals(&self, other: &Vector) -> bool {
        if !other.is_hp() {
            return false;
        }
        let (len_a, size_a, raw_a) = self.with_hp(|h| (h.length, h.size, h.raw().to_vec()));
        let (len_b, size_b, raw_b) = other.with_hp(|h| (h.length, h.size, h.raw().to_vec()));
        len_a == len_b && size_a != size_b && raw_a == raw_b
    }

    /// `Copy()`
    pub(crate) fn hp_copy(&self) -> Vector {
        hp(self.with_hp(|h| h.copy()))
    }

    /// `Sort(bool reverse)`
    pub(crate) fn hp_sort(&self, reverse: bool) -> CResult<Vector> {
        let h = snap(self);
        let mut v = HpData::new(h.length, h.length, h.units.clone());
        if h.size == 0 {
            return Ok(hp(v));
        }
        {
            let span = v.values.as_mut().unwrap();
            span[..h.size].copy_from_slice(&h.raw_all()[..h.size]);
            if reverse {
                crate::sort::sort_by::<f64, ()>(span, None, &mut |x, y| Ok(-double_compare_to(*x, *y)))?;
            } else {
                crate::sort::sort_doubles::<()>(span, None);
            }
        }
        Ok(hp(v))
    }

    /// `Reverse()`
    pub(crate) fn hp_reverse(&self) -> CResult<Vector> {
        let h = snap(self);
        let mut v = HpData::new(h.length, h.length, h.units.clone());
        let vals = h.raw_all();
        let mut start = 0;
        while start < h.size {
            if vals[start] != 0.0 {
                break;
            }
            start += 1;
        }
        let span = v.values.as_mut().unwrap();
        for i in start..h.size {
            span[h.length - i - 1] = vals[i];
        }
        Ok(hp(v))
    }

    /// `Order(bool reverse)`
    pub(crate) fn hp_order(&self, reverse: bool) -> CResult<Vector> {
        Ok(from_indexes(&self.hp_get_order_indexes(reverse, None)?))
    }

    /// `GetOrderIndexes(bool reverse, int len = -1)`
    pub(crate) fn hp_get_order_indexes(&self, reverse: bool, len: Option<usize>) -> CResult<Vec<i32>> {
        let h = snap(self);
        Ok(order_indexes(&h, reverse, len))
    }

    /// `First(int length)`
    pub(crate) fn hp_first(&self, length: usize) -> CResult<Vector> {
        let h = snap(self);
        let mut length = length;
        if length < 1 {
            length = 1;
        } else if length >= h.length {
            return Ok(hp(h.copy()));
        }
        let mut v = HpData::new(length, length, h.units.clone());
        if length > h.size {
            length = h.size;
        }
        v.values.as_mut().unwrap()[..length].copy_from_slice(&h.raw_all()[..length]);
        Ok(hp(v))
    }

    /// `Last(int length)`
    pub(crate) fn hp_last(&self, length: usize) -> CResult<Vector> {
        let h = snap(self);
        let mut length = length;
        if length < 1 {
            length = 1;
        } else if length >= h.length {
            return Ok(hp(h.copy()));
        }
        let mut v = HpData::new(length, length, h.units.clone());
        let start = h.length - length;
        if start < h.size {
            v.values.as_mut().unwrap()[..h.size - start].copy_from_slice(&h.raw_all()[start..h.size]);
        }
        Ok(hp(v))
    }

    /// `Slice(int start, int end)`
    pub(crate) fn hp_slice(&self, start: i64, end: i64) -> CResult<Vector> {
        let h = snap(self);
        let (mut start, mut end) = (start, end);
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        let len_v = h.length as i64;
        if start > len_v {
            start = len_v;
        }
        if end > len_v {
            end = len_v;
        }
        if start == 1 && end == len_v {
            return Ok(hp(h.copy()));
        }
        start -= 1;
        let len = (end - start) as usize;
        if end > h.size as i64 {
            end = h.size as i64;
        }
        let size = if start < end { (end - start) as usize } else { 0 };
        let mut v = HpData::new(len, size, h.units.clone());
        if start < end {
            let s = start as usize;
            v.values.as_mut().unwrap()[..size].copy_from_slice(&h.raw_all()[s..s + size]);
        }
        Ok(hp(v))
    }

    /// `Fill(RealValue value)`: fills in place.
    pub(crate) fn hp_fill(&self, value: &RealValue) -> CResult<Vector> {
        self.with_hp_mut(|h| hp_fill_data(h, value));
        Ok(self.clone())
    }

    /// `Search(RealValue value, int start)`
    pub(crate) fn hp_search(&self, value: &RealValue, start: usize) -> CResult<RealValue> {
        let h = snap(self);
        if !Unit::is_consistent_opt(&h.units, &value.units) {
            return Ok(RealValue::new(0.0, h.units.clone()));
        }
        let d = value.d * Unit::convert(&h.units, &value.units, ',')?;
        let mut start = start;
        if start < 1 {
            start = 1;
        } else if start > h.size {
            if start > h.length {
                return Ok(RealValue::new(0.0, h.units.clone()));
            }
            if almost_equals(d, 0.0) {
                return Ok(RealValue::num(start as f64));
            }
            return Ok(RealValue::new(0.0, h.units.clone()));
        }
        let vals = h.raw_all();
        for i in start - 1..h.size {
            if almost_equals(d, vals[i]) {
                return Ok(RealValue::num((i + 1) as f64));
            }
        }
        if h.size < h.length && almost_equals(d, 0.0) {
            return Ok(RealValue::num((h.size + 1) as f64));
        }
        Ok(RealValue::new(0.0, h.units.clone()))
    }

    /// `FindAll(RealValue value, int start, Relation rel)`
    pub(crate) fn hp_find_all(&self, value: &RealValue, start: usize, rel: Relation) -> CResult<Vector> {
        let h = snap(self);
        Ok(from_indexes(&find_all_indexes(&h, value, start, rel)?))
    }

    /// `Extract(Vector indexes)`
    pub(crate) fn hp_extract(&self, indexes: &Vector) -> CResult<Vector> {
        let h = snap(self);
        let n = indexes.len();
        let mut v = HpData::new(n, n, h.units.clone());
        for i in 0..n {
            let d = indexes.get(i).d;
            if d < 1.0 || d > i32::MAX as f64 {
                return Err(exceptions::MustBePositiveInteger(exceptions::Items::Index));
            }
            // (int)NaN is int.MinValue in C#; GetValue(j - 1) then reads past the size: 0.
            let value = if d.is_nan() {
                0.0
            } else {
                let j = d as usize;
                if j > h.length {
                    return Err(exceptions::IndexOutOfRange(&j.to_string()));
                }
                h.get_value(j - 1)
            };
            v.values.as_mut().unwrap()[i] = value;
        }
        Ok(hp(v))
    }

    /// `Count(RealValue value, int start)`
    pub(crate) fn hp_count(&self, value: &RealValue, start: usize) -> CResult<RealValue> {
        let h = snap(self);
        Ok(RealValue::num(hp_count_data(&h, value, start)? as f64))
    }

    /// `Resize(int newSize)`: resizes in place.
    pub(crate) fn hp_resize(&self, new_size: usize) -> CResult<Vector> {
        if new_size as i64 > MAX_LENGTH {
            return Err(exceptions::VectorSizeLimit());
        }
        self.with_hp_mut(|h| h.resize(new_size));
        Ok(self.clone())
    }

    /// `L1Norm()`
    pub(crate) fn hp_l1_norm(&self) -> CResult<RealValue> {
        Ok(l1_norm_data(&snap(self)))
    }

    /// `Norm()`
    pub(crate) fn hp_norm(&self) -> CResult<RealValue> {
        Ok(norm_data(&snap(self)))
    }

    /// `LpNorm(int p)`
    pub(crate) fn hp_lp_norm(&self, p: i32) -> CResult<RealValue> {
        if p < 1 {
            return Err(exceptions::InvalidLpNormArgument());
        }
        let h = snap(self);
        if h.size == 0 {
            return Ok(zero_or_units(&h.units));
        }
        let vals = h.raw_all();
        let pf = p as f64;
        let mut norm = 0.0;
        for v in &vals[..h.size] {
            norm += v.abs().powf(pf);
        }
        Ok(RealValue::new(norm.powf(1.0 / pf), h.units.clone()))
    }

    /// `InfNorm()`
    pub(crate) fn hp_inf_norm(&self) -> CResult<RealValue> {
        Ok(inf_norm_data(&snap(self)))
    }

    /// `Normalize()`
    pub(crate) fn hp_normalize(&self) -> CResult<Vector> {
        let n = self.hp_norm()?;
        div_vr(self, &n)
    }

    /// `Min()`
    pub(crate) fn hp_min(&self) -> CResult<RealValue> {
        Ok(min_data(&snap(self)))
    }

    /// `Max()`
    pub(crate) fn hp_max(&self) -> CResult<RealValue> {
        Ok(max_data(&snap(self)))
    }

    /// `Sum()`
    pub(crate) fn hp_sum(&self) -> CResult<RealValue> {
        Ok(sum_data(&snap(self)))
    }

    /// `SumSq()`
    pub(crate) fn hp_sum_sq(&self) -> CResult<RealValue> {
        Ok(sum_sq_data(&snap(self)))
    }

    /// `Srss()`
    pub(crate) fn hp_srss(&self) -> CResult<RealValue> {
        let h = snap(self);
        if h.size == 0 {
            return Ok(zero_or_units(&h.units));
        }
        Ok(RealValue::new(sum_sq_data(&h).d.sqrt(), h.units.clone()))
    }

    /// `Average()`
    pub(crate) fn hp_average(&self) -> CResult<RealValue> {
        let h = snap(self);
        if h.size == 0 {
            return Ok(zero_or_units(&h.units));
        }
        Ok(RealValue::new(sum_data(&h).d / h.length as f64, h.units.clone()))
    }

    /// `Product()`
    pub(crate) fn hp_product(&self) -> CResult<RealValue> {
        Ok(product_data(&snap(self)))
    }

    /// `Mean()`
    pub(crate) fn hp_mean(&self) -> CResult<RealValue> {
        let h = snap(self);
        let product = product_data(&h);
        let result = product.d.powf(1.0 / h.length as f64);
        match &product.units {
            None => Ok(RealValue::num(result)),
            Some(u) => Ok(RealValue::new(result, Unit::root(u, h.length as i32, false))),
        }
    }

    /// `And()`
    pub fn hp_and(&self) -> RealValue {
        let h = snap(self);
        if h.length > h.size {
            return RealValue::ZERO;
        }
        for v in &h.raw_all()[..h.size] {
            if v.abs() < LOGICAL_ZERO {
                return RealValue::ZERO;
            }
        }
        RealValue::ONE
    }

    /// `Or()`
    pub fn hp_or(&self) -> RealValue {
        let h = snap(self);
        for v in &h.raw_all()[..h.size] {
            if v.abs() >= LOGICAL_ZERO {
                return RealValue::ONE;
            }
        }
        RealValue::ZERO
    }

    /// `Xor()`
    pub fn hp_xor(&self) -> RealValue {
        let h = snap(self);
        if h.size == 0 {
            return RealValue::ZERO;
        }
        let vals = h.raw_all();
        let mut b = vals[0].abs() >= LOGICAL_ZERO;
        for v in &vals[1..h.size] {
            b = b != (v.abs() >= LOGICAL_ZERO);
        }
        if b {
            RealValue::ONE
        } else {
            RealValue::ZERO
        }
    }

    /// `Gcd()`
    pub(crate) fn hp_gcd(&self) -> CResult<RealValue> {
        let h = snap(self);
        if h.size == 0 {
            return Ok(RealValue::ONE);
        }
        let vals = h.raw_all();
        let mut a = calculator::as_long(vals[0])?;
        for &v in &vals[1..h.size] {
            let b = calculator::as_long(v)?;
            a = calculator::gcd_l(a, b);
        }
        Ok(RealValue::num(a as f64))
    }

    /// `Lcm()`
    pub(crate) fn hp_lcm(&self) -> CResult<RealValue> {
        let h = snap(self);
        if h.size == 0 {
            return Ok(RealValue::ONE);
        }
        let vals = h.raw_all();
        let mut a = calculator::as_long(vals[0])?;
        for &v in &vals[1..h.size] {
            let b = calculator::as_long(v)?;
            let g = calculator::gcd_l(a, b);
            if g == 0 {
                return Err(MathParserException::unexpected("Attempted to divide by zero."));
            }
            a = a.wrapping_mul(b) / g;
        }
        Ok(RealValue::num(a as f64))
    }

    /// `Take(in RealValue x)` (HpVector override: `d > Length * DeltaPlus`)
    pub fn hp_take(&self, x: &RealValue) -> CResult<RealValue> {
        let h = snap(self);
        let d = x.d.round();
        if !d.is_normal() || d < calculator::DELTA_MINUS || d > h.length as f64 * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        Ok(h.get(d as usize - 1))
    }

    /// `Line(in RealValue x)`
    pub fn hp_line(&self, x: &RealValue) -> CResult<RealValue> {
        let h = snap(self);
        let d = x.d;
        let len = h.length as f64;
        if !d.is_normal() || d < calculator::DELTA_MINUS || d > len * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        let i = d.floor() as usize;
        let v1 = h.get(i - 1);
        if i as f64 == d || d >= len {
            return Ok(v1);
        }
        let dv = RealValue::sub(&h.get(i), &v1)?;
        RealValue::add(&v1, &dv.mul_f(d - i as f64))
    }

    /// `Spline(in RealValue x)`
    pub fn hp_spline(&self, x: &RealValue) -> CResult<RealValue> {
        let h = snap(self);
        let mut d = x.d;
        let length = h.length as i64;
        if !d.is_normal() || d < calculator::DELTA_MINUS || d > length as f64 * calculator::DELTA_PLUS {
            return Ok(RealValue::NAN);
        }
        let i = d.floor() as i64 - 1;
        if i as f64 == d || d >= length as f64 {
            return Ok(h.get(i as usize));
        }
        let y0 = h.get_value(i as usize);
        let y1 = h.get_value(i as usize + 1);
        let mut dy = y1 - y0;
        if dy.is_nan() {
            return Ok(RealValue::NAN);
        }
        let mut a = dy;
        let mut b = dy;
        let sign = crate::base_types::complex::math_sign;
        dy = sign(dy)? as f64;
        if i > 0 {
            let y2 = h.get_value(i as usize - 1);
            a = (y1 - y2) * if sign(y0 - y2)? as f64 == dy { 0.5 } else { 0.25 };
        }
        if i < length - 2 {
            let y2 = h.get_value(i as usize + 2);
            b = (y2 - y0) * if sign(y2 - y1)? as f64 == dy { 0.5 } else { 0.25 };
        }
        if i == 0 {
            a += (a - b) / 2.0;
        }
        if i == length - 2 {
            b += (b - a) / 2.0;
        }
        let t = d - i as f64 - 1.0;
        d = y0 + ((y1 - y0) * (3.0 - 2.0 * t) * t + ((a + b) * t - a) * (t - 1.0)) * t;
        Ok(RealValue::new(d, h.units.clone()))
    }

    /// `SetValues(int[] indexes)`: replaces the content with the (0 based) indexes.
    pub fn hp_set_values(&self, indexes: &[i32]) {
        self.with_hp_mut(|h| {
            let n = indexes.len();
            h.length = n;
            h.size = n;
            h.capacity = n;
            h.values = Some(indexes.iter().map(|&i| i as f64).collect());
        });
    }
}

/// `Fill(RealValue value)` on the storage
pub(crate) fn hp_fill_data(h: &mut HpData, value: &RealValue) {
    if value.equals(&RealValue::ZERO) {
        h.size = 0;
        h.capacity = 0;
        h.values = Some(vec![]);
        return;
    }
    if h.size < h.length {
        h.size = h.length;
        if h.capacity < h.length {
            h.capacity = h.length;
            h.values = Some(vec![0.0; h.capacity]);
        }
    }
    h.units = value.units.clone();
    if let Some(v) = h.values.as_mut() {
        for x in v.iter_mut() {
            *x = value.d;
        }
    }
}

/// `GetOrderIndexes(bool reverse, int len = -1)` on the storage
pub(crate) fn order_indexes(h: &HpData, reverse: bool, len: Option<usize>) -> Vec<i32> {
    let len = len.unwrap_or(h.length);
    let mut values = vec![0.0; len];
    values[..h.size].copy_from_slice(&h.raw_all()[..h.size]);
    let mut indexes: Vec<i32> = (0..len as i32).collect();
    if reverse {
        let _ = crate::sort::sort_by(&mut values, Some(&mut indexes), &mut |x, y| {
            Ok(-double_compare_to(*x, *y))
        });
    } else {
        crate::sort::sort_doubles(&mut values, Some(&mut indexes));
    }
    indexes
}

/// `Count(RealValue value, int start)` on the storage
pub(crate) fn hp_count_data(h: &HpData, value: &RealValue, start: usize) -> CResult<i64> {
    if !Unit::is_consistent_opt(&h.units, &value.units) {
        return Ok(0);
    }
    let d = value.d * Unit::convert(&h.units, &value.units, ',')?;
    let mut count: i64 = 0;
    let vals = h.raw_all();
    let mut i = start as i64 - 1;
    while i < h.size as i64 {
        if i < 0 {
            return Err(super::index_was_outside());
        }
        if almost_equals(d, vals[i as usize]) {
            count += 1;
        }
        i += 1;
    }
    if d == 0.0 {
        count += (h.length - h.size) as i64;
    }
    Ok(count)
}

fn zero_or_units(u: &OptUnit) -> RealValue {
    if u.is_none() {
        RealValue::ZERO
    } else {
        RealValue::new(0.0, u.clone())
    }
}

/// `L1Norm()` on the storage
pub(crate) fn l1_norm_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    let sa = &h.raw_all()[..h.size];
    let mut norm = 0.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0].abs(), sa[1].abs(), sa[2].abs(), sa[3].abs()];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] += sa[i * VEC_SIZE + k].abs();
            }
        }
        for v in vb {
            norm += v;
        }
        nv *= VEC_SIZE;
    }
    for v in &sa[nv..] {
        norm += v.abs();
    }
    RealValue::new(norm, h.units.clone())
}

/// `Norm()` on the storage
pub(crate) fn norm_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    let sa = &h.raw_all()[..h.size];
    let mut norm = 0.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        for i in 0..nv {
            norm += vectorized::vdot(sa, i * VEC_SIZE, sa, i * VEC_SIZE);
        }
        nv *= VEC_SIZE;
    }
    for &d in &sa[nv..] {
        norm += d * d;
    }
    RealValue::new(norm.sqrt(), h.units.clone())
}

/// `MaxAbs()` on the storage
pub(crate) fn max_abs(h: &HpData) -> f64 {
    let sa = &h.raw_all()[..h.size];
    let mut max_abs = 0.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0].abs(), sa[1].abs(), sa[2].abs(), sa[3].abs()];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] = vectorized::vmax(vb[k], sa[i * VEC_SIZE + k].abs());
            }
        }
        for v in vb {
            let b = v.abs();
            if b > max_abs {
                max_abs = b;
            }
        }
        nv *= VEC_SIZE;
    }
    for v in &sa[nv..] {
        let d = v.abs();
        if d > max_abs {
            max_abs = d;
        }
    }
    max_abs
}

/// `InfNorm()` on the storage
pub(crate) fn inf_norm_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    RealValue::new(max_abs(h), h.units.clone())
}

/// `Min()` on the storage
pub(crate) fn min_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    let sa = &h.raw_all()[..h.size];
    let mut min = f64::MAX;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0], sa[1], sa[2], sa[3]];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] = vectorized::vmin(vb[k], sa[i * VEC_SIZE + k]);
            }
        }
        for b in vb {
            if b < min {
                min = b;
            }
        }
        nv *= VEC_SIZE;
    }
    for &b in &sa[nv..] {
        if b < min {
            min = b;
        }
    }
    if h.length > h.size && min > 0.0 {
        min = 0.0;
    }
    RealValue::new(min, h.units.clone())
}

/// `Max()` on the storage
pub(crate) fn max_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    let sa = &h.raw_all()[..h.size];
    let mut max = f64::MIN;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0], sa[1], sa[2], sa[3]];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] = vectorized::vmax(vb[k], sa[i * VEC_SIZE + k]);
            }
        }
        for b in vb {
            if b > max {
                max = b;
            }
        }
        nv *= VEC_SIZE;
    }
    for &b in &sa[nv..] {
        if b > max {
            max = b;
        }
    }
    if h.length > h.size && max < 0.0 {
        max = 0.0;
    }
    RealValue::new(max, h.units.clone())
}

/// `Sum()` on the storage
pub(crate) fn sum_data(h: &HpData) -> RealValue {
    if h.size == 0 {
        return zero_or_units(&h.units);
    }
    let sa = &h.raw_all()[..h.size];
    let mut sum = 0.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0], sa[1], sa[2], sa[3]];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] += sa[i * VEC_SIZE + k];
            }
        }
        sum = vectorized::vsum(vb);
        nv *= VEC_SIZE;
    }
    for &v in &sa[nv..] {
        sum += v;
    }
    RealValue::new(sum, h.units.clone())
}

/// `SumSq()` on the storage
pub(crate) fn sum_sq_data(h: &HpData) -> RealValue {
    let u2 = h.units.as_ref().and_then(|u| u.pow(2.0));
    if h.size == 0 {
        return if h.units.is_none() {
            RealValue::ZERO
        } else {
            RealValue::new(0.0, u2)
        };
    }
    let sa = &h.raw_all()[..h.size];
    let mut sumsq = 0.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        for i in 0..nv {
            sumsq += vectorized::vdot(sa, i * VEC_SIZE, sa, i * VEC_SIZE);
        }
        nv *= VEC_SIZE;
    }
    for &b in &sa[nv..] {
        sumsq += b * b;
    }
    RealValue::new(sumsq, u2)
}

/// `Product()` on the storage
pub(crate) fn product_data(h: &HpData) -> RealValue {
    let un = h.units.as_ref().and_then(|u| u.pow(h.length as f32));
    if h.size == 0 || h.length > h.size {
        return if h.units.is_none() {
            RealValue::ZERO
        } else {
            RealValue::new(0.0, un)
        };
    }
    let sa = &h.raw_all()[..h.size];
    let mut product = 1.0;
    let mut nv = 0;
    if h.size > 15 {
        nv = h.size / VEC_SIZE;
        let mut vb = [sa[0], sa[1], sa[2], sa[3]];
        for i in 1..nv {
            for k in 0..4 {
                vb[k] *= sa[i * VEC_SIZE + k];
            }
        }
        for v in vb {
            product *= v;
        }
        nv *= VEC_SIZE;
    }
    for &v in &sa[nv..] {
        product *= v;
    }
    RealValue::new(product, un)
}

/// The HpVector operators (`HpVector op RealValue`, `RealValue op HpVector`,
/// `HpVector op HpVector`), dispatched from `Value::binary`.
pub mod ops {
    use crate::base_types::real_value::RealValue;
    use crate::base_types::value::BinOp;
    use crate::math_parser_exception::CResult;
    use crate::vector::Vector;

    /// `-HpVector`
    pub fn neg(a: &Vector) -> CResult<Vector> {
        Ok(super::neg(a))
    }

    /// `HpVector op RealValue`
    pub fn op_vr(op: BinOp, a: &Vector, b: &RealValue) -> CResult<Vector> {
        use super::*;
        match op {
            BinOp::Add => add_vr(a, b),
            BinOp::Sub => sub_vr(a, b),
            BinOp::Mul => mul_vr(a, b),
            BinOp::Div => div_vr(a, b),
            BinOp::Rem => rem_vr(a, b),
            BinOp::Lt => lt_vr(a, b),
            BinOp::Gt => gt_vr(a, b),
            BinOp::Le => le_vr(a, b),
            BinOp::Ge => ge_vr(a, b),
            BinOp::Eq => eq_vr(a, b),
            BinOp::Ne => ne_vr(a, b),
            BinOp::And => Ok(and_vr(a, b)),
            BinOp::Or => Ok(or_vr(a, b)),
            BinOp::Xor => Ok(xor_vr(a, b)),
        }
    }

    /// `RealValue op HpVector`: the commuting operators are evaluated as `b op a`, as
    /// in IValue.cs.
    pub fn op_rv(op: BinOp, a: &RealValue, b: &Vector) -> CResult<Vector> {
        use super::*;
        match op {
            BinOp::Sub => sub_rv(a, b),
            BinOp::Div => div_rv(a, b),
            BinOp::Rem => rem_rv(a, b),
            BinOp::Lt => lt_rv(a, b),
            BinOp::Gt => gt_rv(a, b),
            BinOp::Le => le_rv(a, b),
            BinOp::Ge => ge_rv(a, b),
            _ => op_vr(op, b, a),
        }
    }

    /// `HpVector op HpVector`
    pub fn op_vv(op: BinOp, a: &Vector, b: &Vector) -> CResult<Vector> {
        use super::*;
        match op {
            BinOp::Add => add(a, b),
            BinOp::Sub => sub(a, b),
            BinOp::Mul => mul(a, b),
            BinOp::Div => div(a, b),
            BinOp::Rem => rem(a, b),
            BinOp::Lt => lt(a, b),
            BinOp::Gt => gt(a, b),
            BinOp::Le => le(a, b),
            BinOp::Ge => ge(a, b),
            BinOp::Eq => eq(a, b),
            BinOp::Ne => ne(a, b),
            BinOp::And => Ok(and(a, b)),
            BinOp::Or => Ok(or(a, b)),
            BinOp::Xor => Ok(xor(a, b)),
        }
    }
}
