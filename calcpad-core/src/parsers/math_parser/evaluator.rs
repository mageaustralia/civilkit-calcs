//! Port of Parsers/MathParser/MathParser.Evaluator.cs (RPN interpreter).
use std::rc::Rc;

use super::custom_function::CustomFunctionRef;
use super::is_assignment;
use super::tokens::{Rpn, TokenKind, TokenRef, TokenTypes};
use super::MathParser;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{Field, OptUnit, Unit, UnitRef};
use crate::base_types::value::Value;
use crate::calculator::{index_out_of_range, matrix_calculator as mc, vector_calculator as vc, NEG_STR};
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::matrix::{hp_matrix, Matrix};
use crate::vector::Vector;

struct Stack {
    values: Vec<Value>,
    base: isize,
}

impl Stack {
    fn len(&self) -> isize {
        self.values.len() as isize
    }
    fn push(&mut self, v: Value) {
        self.values.push(v);
    }
    fn pop(&mut self) -> CResult<Value> {
        if self.len() <= self.base {
            return Err(exceptions::MissingOperand());
        }
        Ok(self.values.pop().unwrap())
    }
    fn at_base(&self) -> bool {
        self.len() == self.base
    }
    /// `StackPopAndExpandValues(count)`: scalars, vector elements and matrix elements.
    fn pop_and_expand_values(&mut self, count: usize) -> CResult<Vec<Value>> {
        let items = self.pop_values(count)?;
        let mut list = Vec::with_capacity(count);
        for ival in items {
            match ival {
                Value::Real(_) | Value::Complex(_) => list.push(ival),
                Value::Vector(v) => {
                    for j in 0..v.len() {
                        list.push(Value::Real(v.get(j)));
                    }
                }
                Value::Matrix(m) => {
                    for i in 0..m.row_count() {
                        for j in 0..m.col_count() {
                            list.push(Value::Real(m.get(i, j)));
                        }
                    }
                }
            }
        }
        Ok(list)
    }

    /// `StackPopAndExpandRealValues(count)`: the `(RealValue)` cast fails on complex values.
    fn pop_and_expand_real_values(&mut self, count: usize) -> CResult<Vec<RealValue>> {
        let list = self.pop_and_expand_values(count)?;
        list.into_iter()
            .map(|v| match v {
                Value::Real(r) => Ok(r),
                other => Err(invalid_cast(&Some(other), "Calcpad.Core.RealValue")),
            })
            .collect()
    }

    fn pop_values(&mut self, count: usize) -> CResult<Vec<Value>> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(self.values.pop().ok_or_else(exceptions::StackEmpty)?);
        }
        v.reverse();
        Ok(v)
    }
}

/// .NET InvalidCastException for an explicit cast of a boxed value.
pub(crate) fn invalid_cast(v: &Option<Value>, target: &str) -> crate::math_parser_exception::MathParserException {
    match v {
        Some(v) => crate::math_parser_exception::MathParserException::unexpected(format!(
            "Unable to cast object of type '{}' to type '{}'.",
            v.to_cs_string_type(),
            target
        )),
        None => crate::vector::null_reference(),
    }
}

impl MathParser {
    /// `Evaluator.Evaluate(rpn, isVisible)`
    pub(crate) fn evaluate(&self, rpn: &Rpn, is_visible: bool) -> CResult<Value> {
        let rpn_length = rpn.len();
        if rpn_length < 1 {
            return Err(exceptions::ExpressionEmpty());
        }
        let mut stack = Stack {
            values: Vec::with_capacity(16),
            base: 0,
        };
        let mut i0 = 0;
        if rpn[0].borrow().ttype == TokenTypes::Variable && is_assignment(&rpn[rpn_length - 1].borrow().content) {
            i0 = 1;
        }
        *self.backup_variable.borrow_mut() = (None, Some(Value::zero()));
        for t in rpn.iter().skip(i0) {
            if stack.len() < stack.base {
                return Err(exceptions::StackEmpty());
            }
            let (tt, content) = {
                let tb = t.borrow();
                (tb.ttype, tb.content.clone())
            };
            match tt {
                TokenTypes::Constant => {
                    let v = t.borrow().token_value().cloned().unwrap_or_else(Value::zero);
                    stack.push(v);
                }
                TokenTypes::Unit | TokenTypes::Variable | TokenTypes::Input => {
                    let v = self.evaluate_token0(t)?;
                    stack.push(v);
                }
                TokenTypes::Operator if content == NEG_STR => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let a = stack.pop()?;
                    stack.push(a.neg()?);
                }
                TokenTypes::Function => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    stack.push(Value::evaluate_function(self.calc(), index, &a)?);
                }
                TokenTypes::VectorFunction | TokenTypes::MatrixFunction | TokenTypes::MatrixOptionalFunction => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    let r = match tt {
                        TokenTypes::VectorFunction => vc::evaluate_vector_function(index, &a)?,
                        TokenTypes::MatrixOptionalFunction => {
                            mc::evaluate_matrix_iterative_function(index, &a, &Value::zero(), self.tol()?)?
                        }
                        _ => {
                            if index == mc::LU_INDEX {
                                self.evaluate_lu_decomposition(&a)?
                            } else {
                                mc::evaluate_matrix_function(index, &a)?
                            }
                        }
                    };
                    stack.push(r);
                }
                TokenTypes::Operator if is_assignment(&content) => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let b = stack.pop()?;
                    return self.evaluate_assignment(b, rpn, is_visible);
                }
                TokenTypes::Operator | TokenTypes::Function2 => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let b = stack.pop()?;
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let c = stack.pop()?;
                    let index = t.borrow().index;
                    let r = if tt == TokenTypes::Operator {
                        Value::evaluate_operator(self.calc(), index, &c, &b)?
                    } else {
                        Value::evaluate_function2(self.calc(), index, &c, &b)?
                    };
                    stack.push(r);
                }
                TokenTypes::VectorFunction2 | TokenTypes::MatrixFunction2 | TokenTypes::MatrixIterativeFunction => {
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let b = stack.pop()?;
                    if stack.at_base() {
                        return Err(exceptions::MissingOperand());
                    }
                    let c = stack.pop()?;
                    let index = t.borrow().index;
                    let r = match tt {
                        TokenTypes::VectorFunction2 => vc::evaluate_vector_function2(index, &c, &b)?,
                        TokenTypes::MatrixIterativeFunction => {
                            mc::evaluate_matrix_iterative_function(index, &c, &b, self.tol()?)?
                        }
                        _ => mc::evaluate_matrix_function2(index, &c, &b)?,
                    };
                    stack.push(r);
                }
                TokenTypes::Function3 | TokenTypes::VectorFunction3 | TokenTypes::MatrixFunction3 => {
                    let c = stack.pop()?;
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    let r = match tt {
                        TokenTypes::Function3 => self.evaluator_calc().evaluate_function3(index, &a, &b, &c)?,
                        TokenTypes::VectorFunction3 => vc::evaluate_vector_function3(index, &a, &b, &c)?,
                        _ => mc::evaluate_matrix_function3(index, &a, &b, &c)?,
                    };
                    stack.push(r);
                }
                TokenTypes::MatrixFunction4 => {
                    let d = stack.pop()?;
                    let c = stack.pop()?;
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    stack.push(mc::evaluate_matrix_function4(index, &a, &b, &c, &d)?);
                }
                TokenTypes::MatrixFunction5 => {
                    let e = stack.pop()?;
                    let d = stack.pop()?;
                    let c = stack.pop()?;
                    let b = stack.pop()?;
                    let a = stack.pop()?;
                    let index = t.borrow().index;
                    stack.push(mc::evaluate_matrix_function5(index, &a, &b, &c, &d, &e)?);
                }
                TokenTypes::Interpolation => {
                    let (index, pc) = {
                        let tb = t.borrow();
                        (tb.index, tb.parameter_count())
                    };
                    let r = self.evaluate_interpolation_token(index, pc, &mut stack)?;
                    stack.push(r);
                }
                TokenTypes::MultiFunction | TokenTypes::VectorMultiFunction | TokenTypes::MatrixMultiFunction => {
                    let (index, pc) = {
                        let tb = t.borrow();
                        (tb.index, tb.parameter_count())
                    };
                    let r = self.evaluate_multi_function_token(tt, index, pc, &mut stack)?;
                    stack.push(r);
                }
                TokenTypes::CustomFunction => {
                    let v = self.evaluate_function_token(t, &mut stack)?;
                    stack.push(v);
                }
                TokenTypes::Vector | TokenTypes::RowDivisor => {
                    let r = self.evaluate_vector_token(t, &mut stack)?;
                    stack.push(r);
                }
                TokenTypes::Matrix => {
                    let r = self.evaluate_matrix_token(t, &mut stack)?;
                    stack.push(r);
                }
                TokenTypes::VectorIndex | TokenTypes::MatrixIndex => {
                    let r = self.evaluate_index_token(t, &mut stack)?;
                    stack.push(r);
                }
                TokenTypes::Solver => {
                    let index = t.borrow().index;
                    let sb = self.solve_blocks.borrow()[index as usize].clone();
                    sb.calculate(self)?;
                    stack.push(sb.result());
                }
                _ => return Err(exceptions::CannotEvaluateAsType(&content, &tt.name())),
            }
        }
        if stack.len() > stack.base {
            let mut v = stack.values.pop().unwrap();
            let tu = self.target_units.borrow().clone();
            let tu = self.resolve_target_units(&v, &tu)?;
            let u = apply_units(&mut v, &tu)?;
            self.set_result_units(u);
            return Ok(v);
        }
        self.set_result_units(None);
        Ok(Value::zero())
    }

    /// `EvaluateLuDecomposition(a, variables)`: stores the permutation in the variable `ind`.
    fn evaluate_lu_decomposition(&self, a: &Value) -> CResult<Value> {
        mc::lu_decomposition(a, &self.lu_index_vector())
    }

    /// The HpVector of the variable `ind` that receives the LU permutation (created when
    /// missing or not an HpVector), as `EvaluateLuDecomposition` and `ParseLuDecomposition` do.
    pub(crate) fn lu_index_vector(&self) -> Vector {
        let existing = self.variables.borrow().get("ind").cloned();
        match existing.as_ref().and_then(|v| v.value()) {
            Some(Value::Vector(v)) if v.is_hp() => v,
            _ => {
                let v = crate::vector::hp_vector::new_ls(0, 0, None);
                self.variables.borrow_mut().insert(
                    "ind".to_string(),
                    crate::base_types::variable::Variable::with_value(Value::Vector(v.clone())),
                );
                v
            }
        }
    }

    /// `EvaluateInterpolationToken(t)`
    fn evaluate_interpolation_token(&self, index: i64, param_count: i32, stack: &mut Stack) -> CResult<Value> {
        if param_count == 2 {
            if let Some(Value::Vector(vec1)) = stack.values.last().cloned() {
                stack.pop()?;
                let ival = stack.pop()?;
                if let Value::Real(x) = &ival {
                    return vc::evaluate_interpolation(index, x, &vec1);
                }
                return Err(exceptions::CannotInterpolateWithNonScalarValue());
            }
        }
        if param_count == 3 {
            if let Some(Value::Matrix(matrix)) = stack.values.last().cloned() {
                stack.pop()?;
                let ival = stack.pop()?;
                if let Value::Real(x) = &ival {
                    let ival = stack.pop()?;
                    if let Value::Real(y) = &ival {
                        return mc::evaluate_interpolation(index, x, y, &matrix);
                    }
                }
                return Err(exceptions::CannotInterpolateWithNonScalarValue());
            }
        }
        let params = stack.pop_and_expand_values(param_count.max(0) as usize)?;
        self.evaluator_calc().evaluate_interpolation(index, &params)
    }

    /// `EvaluateMultiFunctionToken(t)`
    fn evaluate_multi_function_token(
        &self,
        tt: TokenTypes,
        index: i64,
        param_count: i32,
        stack: &mut Stack,
    ) -> CResult<Value> {
        if tt == TokenTypes::MultiFunction {
            if param_count == 1 {
                match stack.values.last().cloned() {
                    Some(Value::Vector(v)) if stack.len() > stack.base => {
                        stack.pop()?;
                        return vc::evaluate_multi_function(index, &v);
                    }
                    Some(Value::Matrix(m)) if stack.len() > stack.base => {
                        stack.pop()?;
                        return mc::evaluate_multi_function(index, &m);
                    }
                    _ => {}
                }
            }
            if index == crate::calculator::SWITCH_INDEX {
                let params = stack.pop_values(param_count.max(0) as usize)?;
                return Value::evaluate_switch(&params);
            }
            let params = stack.pop_and_expand_values(param_count.max(0) as usize)?;
            return self.evaluator_calc().evaluate_multi_function(index, &params);
        }
        let params = stack.pop_values(param_count.max(0) as usize)?;
        if tt == TokenTypes::VectorMultiFunction {
            return vc::evaluate_vector_multi_function(index, &params);
        }
        mc::evaluate_matrix_multi_function(index, &params)
    }

    /// `EvaluateVectorToken(t)`: a vector literal, or a variable that holds a vector.
    fn evaluate_vector_token(&self, t: &TokenRef, stack: &mut Stack) -> CResult<Value> {
        let is_literal = matches!(t.borrow().kind, TokenKind::Vector(_));
        if is_literal {
            let count = t.borrow().index.max(0) as usize;
            let values = stack.pop_and_expand_real_values(count)?;
            let v = Vector::from_values(values)?;
            t.borrow_mut().kind = TokenKind::Vector(Some(v.clone()));
            return Ok(Value::Vector(v));
        }
        self.evaluate_token0(t)
    }

    /// `EvaluateMatrixToken(t)`: a matrix literal, or a variable that holds a matrix.
    fn evaluate_matrix_token(&self, t: &TokenRef, stack: &mut Stack) -> CResult<Value> {
        let is_literal = matches!(t.borrow().kind, TokenKind::Matrix(_));
        if is_literal {
            let count = (t.borrow().index / crate::vector::MAX_LENGTH) as usize;
            let values = stack.pop_values(count)?;
            let m = mc::join_rows(&values)?;
            t.borrow_mut().kind = TokenKind::Matrix(Some(m.clone()));
            return Ok(Value::Matrix(m));
        }
        self.evaluate_token0(t)
    }

    /// `EvaluateIndexToken(t)`
    fn evaluate_index_token(&self, t: &TokenRef, stack: &mut Stack) -> CResult<Value> {
        let value = stack.pop()?;
        let i = crate::vector::cs_int(value.as_value(exceptions::Items::Index)?.re());
        let tt = t.borrow().ttype;
        if tt == TokenTypes::VectorIndex {
            let vector = stack.pop()?.as_vector(exceptions::Items::IndexTarget)?;
            if i < 1 || i > vector.len() as i64 {
                return Err(exceptions::IndexOutOfRange(&i.to_string()));
            }
            t.borrow_mut().index = i;
            return Ok(Value::Real(vector.get(i as usize - 1)));
        }
        let value = stack.pop()?;
        let ii = crate::vector::cs_int(value.as_value(exceptions::Items::Index)?.re());
        let matrix = stack.pop()?.as_matrix(exceptions::Items::IndexTarget)?;
        let j = i;
        let i = ii;
        if i < 1 || i > matrix.row_count() as i64 || j < 1 || j > matrix.col_count() as i64 {
            return Err(exceptions::IndexOutOfRange(&format!("{}, {}", i, j)));
        }
        t.borrow_mut().index = i * crate::vector::MAX_LENGTH + j;
        Ok(Value::Real(matrix.get(i as usize - 1, j as usize - 1)))
    }

    /// `EvaluateToken(Token t)` for Unit, Variable and Input tokens.
    pub(crate) fn evaluate_token0(&self, t: &TokenRef) -> CResult<Value> {
        let tt = t.borrow().ttype;
        match tt {
            TokenTypes::Unit => {
                let tb = t.borrow();
                match &tb.kind {
                    TokenKind::Value(v) => Ok(v.clone()),
                    TokenKind::Variable(Some(var)) => var.value().ok_or_else(|| exceptions::InvalidOperand("")),
                    _ => Err(exceptions::InvalidOperand("")),
                }
            }
            TokenTypes::Variable => self.evaluate_variable_token(t),
            TokenTypes::Vector | TokenTypes::Matrix => {
                let lit = match &t.borrow().kind {
                    TokenKind::Vector(Some(v)) => Some(Value::Vector(v.clone())),
                    TokenKind::Matrix(Some(m)) => Some(Value::Matrix(m.clone())),
                    _ => None,
                };
                match lit {
                    Some(v) => Ok(v),
                    None => self.evaluate_variable_token(t),
                }
            }
            TokenTypes::Input if t.borrow().content == "?" => Err(exceptions::UndefinedInputField()),
            _ => Ok(t.borrow().token_value().cloned().unwrap_or_else(Value::zero)),
        }
    }

    fn evaluate_variable_token(&self, t: &TokenRef) -> CResult<Value> {
        let (v, content, tt) = {
            let tb = t.borrow();
            (tb.variable_ref(), tb.content.clone(), tb.ttype)
        };
        let v = match v {
            Some(v) => v,
            None => return Err(exceptions::UndefinedVariableOrUnits(&content)),
        };
        if v.is_initialized() || tt == TokenTypes::Vector || tt == TokenTypes::Matrix {
            return v.value().ok_or_else(|| exceptions::UndefinedVariableOrUnits(&content));
        }
        let custom = self.units.borrow().get(&content).cloned();
        let u: OptUnit = match custom {
            Some(u) => u,
            None => match Unit::try_get(&content, self.is_us.get()) {
                Some(u) => Some(u),
                None => return Err(exceptions::UndefinedVariableOrUnits(&content)),
            },
        };
        t.borrow_mut().ttype = TokenTypes::Unit;
        v.set_unit_value(u);
        v.value().ok_or_else(|| exceptions::UndefinedVariableOrUnits(&content))
    }

    /// The target unit for `ival`: as given, except the target `base`, which is the value's own dimensions written
    /// in SI base units (`kg*m^2/s^2`). A value with no unit, a temperature or an angle is left as it is.
    pub(crate) fn resolve_target_units(&self, ival: &Value, tu: &OptUnit) -> CResult<OptUnit> {
        let Some(t) = tu else { return Ok(None) };
        if !(t.is_empty() && t.raw_text().as_deref() == Some(crate::base_types::unit::BASE_TARGET)) {
            return Ok(tu.clone());
        }
        let Some(own) = ival.units() else { return Ok(None) };
        let Some(text) = own.si_base_text() else { return Ok(Some(own.clone())) };
        let units = self.units.borrow().clone();
        crate::parsers::units_parser::parse(&text, Some(&units), self.is_us.get())
    }

    fn evaluate_assignment(&self, b: Value, rpn: &Rpn, is_visible: bool) -> CResult<Value> {
        let mut b = b;
        let tu = self.target_units.borrow().clone();
        let tu = self.resolve_target_units(&b, &tu)?;
        let u = apply_units(&mut b, &tu)?;
        self.set_result_units(u);
        if is_visible {
            self.normalize_units(&mut b);
        }
        let t0 = rpn[0].clone();
        let (t0_type, is_var, content) = {
            let t = t0.borrow();
            (t.ttype, t.is_variable_token(), t.content.clone())
        };
        if is_var {
            if t0_type == TokenTypes::Vector || t0_type == TokenTypes::Matrix {
                let (mut i, mut j) = (-1i64, -1i64);
                for t in rpn.iter() {
                    let t = t.borrow();
                    if t.ttype == TokenTypes::VectorIndex && t.content == content {
                        i = t.index;
                        break;
                    }
                    if t.ttype == TokenTypes::MatrixIndex && t.content == content {
                        i = t.index / crate::vector::MAX_LENGTH;
                        j = t.index % crate::vector::MAX_LENGTH;
                        break;
                    }
                }
                let variable = t0.borrow().variable_ref().unwrap();
                let value = variable.value();
                if t0_type == TokenTypes::Vector {
                    let vector = match value {
                        Some(Value::Vector(v)) => v,
                        other => return Err(invalid_cast(&other, "Calcpad.Core.Vector")),
                    };
                    if i < 1 || i > vector.len() as i64 {
                        return Err(exceptions::IndexOutOfRange(&i.to_string()));
                    }
                    match &b {
                        Value::Real(rb) => {
                            let old = vector.get(i as usize - 1);
                            *self.backup_variable.borrow_mut() =
                                (Some(format!("{}.{}", content, i)), Some(Value::Real(old)));
                            vector.set(i as usize - 1, rb.clone())?;
                        }
                        _ => return Err(exceptions::CannotAssignVectorToScalar()),
                    }
                } else {
                    let matrix = match value {
                        Some(Value::Matrix(m)) => m,
                        other => return Err(invalid_cast(&other, "Calcpad.Core.Matrix")),
                    };
                    if i < 1 || i > matrix.row_count() as i64 || j < 1 || j > matrix.col_count() as i64 {
                        return Err(exceptions::IndexOutOfRange(&format!("{},{}", i, j)));
                    }
                    match &b {
                        Value::Real(rb) => {
                            let old = matrix.get(i as usize - 1, j as usize - 1);
                            *self.backup_variable.borrow_mut() =
                                (Some(format!("{}.{}.{}", content, i, j)), Some(Value::Real(old)));
                            matrix.set(i as usize - 1, j as usize - 1, rb.clone())?;
                        }
                        _ => return Err(exceptions::CannotAssignVectorToScalar()),
                    }
                }
                variable.change();
            } else if t0_type == TokenTypes::Variable {
                let var = t0.borrow().variable_ref().unwrap();
                *self.backup_variable.borrow_mut() = (Some(content), var.value());
                var.assign(b.clone());
            }
        } else if t0_type == TokenTypes::Unit && t0.borrow().is_value_token() {
            let existing_units = t0.borrow().token_value().and_then(|v| v.units().clone());
            if let Some(eu) = existing_units {
                return Err(exceptions::CannotRewriteUnits(&eu.text()));
            }
            self.set_unit(&content, &b.as_complex())?;
            let u = self.get_unit(&content)?;
            t0.borrow_mut().set_value(Value::Real(RealValue::unit(u)));
        }
        Ok(b)
    }

    fn evaluate_function_token(&self, t: &TokenRef, stack: &mut Stack) -> CResult<Value> {
        let (index, map) = {
            let tb = t.borrow();
            (tb.index, tb.argument_map())
        };
        let cf = match self.functions.borrow().get(index) {
            Some(cf) => cf,
            None => return Err(index_out_of_range()),
        };
        let cf_param_count = map.as_ref().map(|m| m.len()).unwrap_or(cf.parameter_count());
        if cf.is_recursion.get() {
            stack.base -= cf_param_count as isize;
            return Ok(Value::nan());
        }
        let args = stack.pop_values(cf_param_count)?;
        match map {
            Some(m) => self.evaluate_custom_function_map(&cf, &m, args),
            None => self.evaluate_custom_function(&cf, args),
        }
    }

    pub(crate) fn ensure_compiled(&self, cf: &CustomFunctionRef) -> CResult<Rc<super::compiler::Expr>> {
        if let Some(f) = cf.function.borrow().clone() {
            return Ok(f);
        }
        let rpn = cf.rpn.borrow().clone();
        let f = Rc::new(self.compile_rpn(&rpn, false)?);
        *cf.function.borrow_mut() = Some(f.clone());
        Ok(f)
    }

    /// `Evaluator.EvaluateFunction(cf, map, supplied)`
    pub(crate) fn evaluate_custom_function_map(
        &self,
        cf: &CustomFunctionRef,
        map: &[usize],
        supplied: Vec<Value>,
    ) -> CResult<Value> {
        if cf.function.borrow().is_none() {
            let rpn = cf.rpn.borrow().clone();
            let f = Rc::new(self.compile_rpn(&rpn, false)?);
            *cf.function.borrow_mut() = Some(f);
            if let Some(df) = cf.default_functions.borrow_mut().as_mut() {
                for x in df.iter_mut() {
                    *x = None;
                }
            }
        }
        let n = cf.parameter_count();
        let mut arguments: Vec<Option<Value>> = vec![None; n];
        for (k, v) in supplied.into_iter().enumerate() {
            arguments[map[k]] = Some(v);
        }
        let parameters = cf.parameters();
        let mut args: Vec<Value> = Vec::with_capacity(n);
        for i in 0..n {
            let a = match arguments[i].take() {
                Some(a) => a,
                None => {
                    let rpn = cf.defaults.borrow().as_ref().and_then(|d| d.get(i).cloned().flatten());
                    let rpn = match rpn {
                        Some(r) => r,
                        None => return Err(exceptions::InvalidNumberOfArguments()),
                    };
                    let existing = cf
                        .default_functions
                        .borrow()
                        .as_ref()
                        .and_then(|d| d.get(i).cloned().flatten());
                    let f = match existing {
                        Some(f) => f,
                        None => {
                            let f = Rc::new(self.compile_rpn(&rpn, false)?);
                            if let Some(df) = cf.default_functions.borrow_mut().as_mut() {
                                df[i] = Some(f.clone());
                            }
                            f
                        }
                    };
                    self.eval_expr(&f)?
                }
            };
            parameters[i].set_value(a.clone());
            args.push(a);
        }
        self.evaluate_custom_function(cf, args)
    }

    /// `Evaluator.EvaluateFunction(cf, x...)`
    pub(crate) fn evaluate_custom_function(&self, cf: &CustomFunctionRef, args: Vec<Value>) -> CResult<Value> {
        let f = self.ensure_compiled(cf)?;
        let mut result = if cf.has_cache() {
            // CustomFunction1/2.Calculate: a result cache per scalar argument value.
            if cf.cache().borrow().len() >= super::custom_function::MAX_CACHE_SIZE {
                cf.cache().borrow_mut().clear();
            }
            if args.iter().all(Value::is_scalar) {
                let cached = cf.cache().borrow().get(&args);
                match cached {
                    Some(Some(v)) => v,
                    Some(None) => return Err(crate::vector::null_reference()),
                    None => {
                        cf.set_parameter_values(&args);
                        match self.eval_expr(&f) {
                            Ok(v) => {
                                cf.cache().borrow_mut().insert(args, Some(v.clone()));
                                v
                            }
                            Err(e) => {
                                cf.cache().borrow_mut().insert(args, None);
                                return Err(e);
                            }
                        }
                    }
                }
            } else {
                cf.set_parameter_values(&args);
                self.eval_expr(&f)?
            }
        } else {
            cf.set_parameter_values(&args);
            self.eval_expr(&f)?
        };
        let cu = cf.units.borrow().clone();
        let u = apply_units(&mut result, &cu)?;
        self.set_result_units(u);
        Ok(result)
    }

    /// `Evaluator.NormalizeUnits(ref IValue)`
    pub(crate) fn normalize_units(&self, ival: &mut Value) {
        match ival {
            Value::Real(real) => normalize_real(real),
            Value::Complex(c) => {
                if let Some(u) = &c.units {
                    let d = u.normalize();
                    if d != 1.0 {
                        *c = ComplexValue::from_c(c.complex() * d, c.units.clone());
                    }
                }
            }
            Value::Vector(v) => normalize_vector(v),
            Value::Matrix(m) => {
                if m.is_hp() {
                    if let Some(u) = hp_matrix::units(m) {
                        let d = u.normalize();
                        if d != 1.0 {
                            hp_matrix::scale(m, d);
                        }
                    }
                } else {
                    for r in m.rows() {
                        normalize_vector(&r);
                    }
                }
            }
        }
    }
}

fn normalize_real(real: &mut RealValue) {
    if let Some(u) = &real.units {
        let d = u.normalize();
        if d != 1.0 {
            *real = RealValue::new(real.d * d, real.units.clone());
        }
    }
}

/// `Evaluator.NormalizeUnits(Vector)`
fn normalize_vector(vector: &Vector) {
    if vector.is_hp() {
        if let Some(u) = vector.hp_units() {
            let d = u.normalize();
            if d != 1.0 {
                vector.hp_scale(d);
            }
        }
        return;
    }
    for i in 0..vector.size() {
        let _ = vector.update_by_ref(i, &mut |v| {
            if v.units.is_some() {
                normalize_real(v);
            }
            Ok(())
        });
    }
}

/// `MathParser.GetFieldUnit`
pub fn get_field_unit(vu: &UnitRef) -> UnitRef {
    match vu.get_field() {
        Field::Mechanical => Unit::get_force_unit(vu),
        Field::Electrical => Unit::get_electrical_unit(vu),
        Field::Other => vu.clone(),
    }
}

/// `Evaluator.ApplyUnits(ref IValue, Unit)`
pub fn apply_units(ival: &mut Value, u: &OptUnit) -> CResult<OptUnit> {
    match ival {
        Value::Real(real) => apply_units_real(real, u),
        Value::Complex(c) => apply_units_complex(c, u),
        Value::Vector(v) => apply_units_vector(v, u),
        Value::Matrix(m) => apply_units_matrix(m, u),
    }
}

/// `ApplyUnits(Matrix M, Unit u)`: converts the matrix in place.
fn apply_units_matrix(m: &Matrix, u: &OptUnit) -> CResult<OptUnit> {
    if m.is_hp() {
        let mut value = RealValue::new(1.0, hp_matrix::units(m));
        let result = apply_units_real(&mut value, u)?;
        let d = value.d;
        if d != 1.0 {
            hp_matrix::scale(m, d);
        }
        hp_matrix::set_units_prop(m, result.clone());
        return Ok(result);
    }
    let rows = m.rows();
    if rows.is_empty() {
        return Err(crate::vector::index_was_outside());
    }
    let result = apply_units_vector(&rows[0], u)?;
    for r in &rows[1..] {
        apply_units_vector(r, u)?;
    }
    Ok(result)
}

/// `ApplyUnits(Vector vector, Unit u)`: converts the vector in place.
fn apply_units_vector(vector: &Vector, u: &OptUnit) -> CResult<OptUnit> {
    if vector.size() == 0 {
        return Ok(u.clone());
    }
    if vector.is_hp() {
        let hu = vector.hp_units();
        if u.is_none() && hu.is_none() {
            return Ok(None);
        }
        let mut value = RealValue::new(1.0, hu);
        let result = apply_units_real(&mut value, u)?;
        let d = value.d;
        if d != 1.0 {
            vector.hp_scale(d);
        }
        vector.set_hp_units(result.clone());
        return Ok(result);
    }
    for i in (0..vector.size()).rev() {
        vector.update_by_ref(i, &mut |v| apply_units_real(v, u).map(|_| ()))?;
    }
    let mut result = None;
    vector.update_by_ref(0, &mut |v| {
        result = apply_units_real(v, u)?;
        Ok(())
    })?;
    Ok(result)
}

fn apply_units_complex(value: &mut ComplexValue, u: &OptUnit) -> CResult<OptUnit> {
    let vu = value.units.clone();
    let u = match u {
        None => {
            let vu = match vu {
                None => return Ok(None),
                Some(vu) => vu,
            };
            let u = get_field_unit(&vu);
            if Rc::ptr_eq(&u, &vu) {
                return Ok(Some(vu));
            }
            let c = vu.convert_to(&u);
            *value = ComplexValue::from_c(value.complex() * c, Some(u.clone()));
            return Ok(Some(u));
        }
        Some(u) => u.clone(),
    };
    if u.is_dimensionless() {
        match &vu {
            None => {
                *value = ComplexValue::from_c(value.complex().div_f(u.get_dimensionless_factor()), Some(u.clone()));
                return Ok(Some(u));
            }
            Some(vuu) => {
                if let Some(format) = u.format_string_with_prefix() {
                    let fu = get_field_unit(vuu);
                    let c = if Rc::ptr_eq(&fu, vuu) { 1.0 } else { vuu.convert_to(&fu) };
                    let nvu = Rc::new(Unit::copy(&fu));
                    nvu.set_text(&format!("{}{}", fu.text(), format));
                    *value = ComplexValue::from_c(value.complex() * c, Some(nvu.clone()));
                    return Ok(Some(nvu));
                }
            }
        }
    }
    if !Unit::is_consistent_opt(&vu, &Some(u.clone())) {
        return Err(exceptions::InconsistentTargetUnits(&Unit::get_text_of(&vu), &u.text()));
    }
    let vu = vu.unwrap();
    let d = vu.convert_to(&u);
    if u.is_temp() {
        let c = value.complex() * d
            + crate::base_types::complex::Complex::real(Unit::get_temp_units_delta(&vu.text(), &u.text()));
        *value = ComplexValue::from_c(c, Some(u.clone()));
    } else {
        *value = ComplexValue::from_c(value.complex() * d, Some(u.clone()));
    }
    Ok(value.units.clone())
}

fn apply_units_real(value: &mut RealValue, u: &OptUnit) -> CResult<OptUnit> {
    let vu = value.units.clone();
    let u = match u {
        None => {
            let vu = match vu {
                None => return Ok(None),
                Some(vu) => vu,
            };
            let u = get_field_unit(&vu);
            if Rc::ptr_eq(&u, &vu) {
                return Ok(Some(vu));
            }
            let c = vu.convert_to(&u);
            *value = RealValue::new(value.d * c, Some(u.clone()));
            return Ok(Some(u));
        }
        Some(u) => u.clone(),
    };
    if u.is_dimensionless() {
        match &vu {
            None => {
                *value = RealValue::new(value.d / u.get_dimensionless_factor(), Some(u.clone()));
                return Ok(Some(u));
            }
            Some(vuu) => {
                if let Some(format) = u.format_string_with_prefix() {
                    let fu = get_field_unit(vuu);
                    let c = if Rc::ptr_eq(&fu, vuu) { 1.0 } else { vuu.convert_to(&fu) };
                    let nvu = Rc::new(Unit::copy(&fu));
                    nvu.set_text(&format!("{}{}", fu.text(), format));
                    *value = RealValue::new(value.d * c, Some(nvu.clone()));
                    return Ok(Some(nvu));
                }
            }
        }
    }
    if !Unit::is_consistent_opt(&vu, &Some(u.clone())) {
        return Err(exceptions::InconsistentTargetUnits(&Unit::get_text_of(&vu), &u.text()));
    }
    let vu = vu.unwrap();
    let d = vu.convert_to(&u);
    if u.is_temp() {
        let re = value.d * d + Unit::get_temp_units_delta(&vu.text(), &u.text());
        *value = RealValue::new(re, Some(u.clone()));
    } else {
        *value = RealValue::new(value.d * d, Some(u.clone()));
    }
    Ok(value.units.clone())
}
