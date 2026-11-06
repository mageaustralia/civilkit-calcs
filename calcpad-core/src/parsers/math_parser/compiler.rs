//! Port of Parsers/MathParser/MathParser.Compiler.cs and MathParser.Compiler.Metods.cs.
//!
//! C# compiles the RPN into a LINQ expression tree. Constant sub-expressions are folded at
//! compile time and the remaining nodes call the `IValue` operators (not the calculator),
//! which differs from the RPN evaluator in a few observable ways (unit text, lazy `if`,
//! negative conditions in `if`/`switch`). This module builds an equivalent tree (`Expr`)
//! with the same folding rules and interprets it.
use std::rc::Rc;

use super::custom_function::CustomFunctionRef;
use super::solve_block::SolverBlock;
use super::tokens::{Rpn, TokenKind, TokenRef, TokenTypes};
use super::MathParser;
use crate::base_types::real_value::{RealValue, LOGICAL_ZERO};
use crate::base_types::unit::Unit;
use crate::base_types::value::Value;
use crate::base_types::variable::VarRef;
use crate::calculator::{matrix_calculator as mc, vector_calculator as vc};
use crate::calculator::{IF_FUNCTION_INDEX, NEG_STR, SWITCH_INDEX};
use crate::exceptions;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::vector::Vector;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Xor,
}

pub enum Expr {
    Const(Value),
    /// `Expression.Property(Expression.Constant(v), "Value")`
    Var(VarRef),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// `IValue.EvaluateOperator(matrixCalc, index, a, b)`
    CalcOp(i64, Box<Expr>, Box<Expr>),
    Func(i64, Box<Expr>),
    Func2(i64, Box<Expr>, Box<Expr>),
    /// `Expression.Condition(Re(test) < LogicalZero, if_false, if_true)`
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Interp(i64, Vec<Expr>),
    Multi(i64, Vec<Expr>),
    Custom {
        cf: CustomFunctionRef,
        args: Vec<Expr>,
        map: Option<Rc<Vec<usize>>>,
    },
    Assign(VarRef, Box<Expr>),
    Solver(Rc<SolverBlock>),
    Block(Vec<Expr>),
    While(Box<Expr>, Vec<Expr>),
    /// `VectorCalculator.GetFunction(index)` and the other vector and matrix delegates;
    /// the token type selects the table.
    VmFunc(TokenTypes, i64, Vec<Expr>),
    /// `ParseLuDecomposition`: the `ind` vector is captured at compile time.
    Lu(Box<Expr>, Vector),
    /// `new Vector(IValue.ExpandRealValues(values))`
    VecLit(Vec<Expr>),
    /// `MatrixCalculator.JoinRows(values)`
    MatLit(Vec<Expr>),
    /// `VectorCalculator.GetElement(vector, i)`
    GetVecElem(Box<Expr>, Box<Expr>),
    /// `MatrixCalculator.GetElement(matrix, i, j)`
    GetMatElem(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `VectorCalculator.SetElement(vector, i, value)`
    SetVecElem(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `MatrixCalculator.SetElement(matrix, i, j, value)`
    SetMatElem(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    fn is_const(&self) -> bool {
        matches!(self, Expr::Const(_))
    }

    fn const_value(&self) -> Value {
        match self {
            Expr::Const(v) => v.clone(),
            _ => unreachable!(),
        }
    }
}

fn stack_empty() -> MathParserException {
    MathParserException::unexpected("Stack empty.")
}

fn pop_values(stack: &mut Vec<Expr>, count: usize) -> CResult<Vec<Expr>> {
    if stack.len() < count {
        return Err(stack_empty());
    }
    let at = stack.len() - count;
    Ok(stack.split_off(at))
}

impl MathParser {
    /// `Compiler.Compile(rpn, allowAssignment)` (returns the tree instead of a delegate).
    pub(crate) fn compile_rpn(&self, rpn: &Rpn, allow_assignment: bool) -> CResult<Expr> {
        if rpn.is_empty() {
            return Err(exceptions::ExpressionEmpty());
        }
        self.compile_parse(rpn, allow_assignment)
    }

    /// `Compiler.RpnToExpressionTree`
    pub(crate) fn rpn_to_expression_tree(&self, rpn: &Rpn, allow_assignment: bool) -> CResult<Expr> {
        self.compile_rpn(rpn, allow_assignment)
    }

    fn compile_parse(&self, rpn: &Rpn, allow_assignment: bool) -> CResult<Expr> {
        let len = rpn.len();
        if allow_assignment && rpn[len - 1].borrow().content == "=" {
            let rpn0 = rpn[0].borrow();
            if rpn0.ttype == TokenTypes::Variable {
                if let TokenKind::Variable(Some(v)) = &rpn0.kind {
                    if !v.is_initialized() {
                        let v = v.clone();
                        drop(rpn0);
                        v.assign(Value::Real(RealValue::ZERO));
                    }
                }
            }
        }
        let mut stack: Vec<Expr> = Vec::new();
        for t in rpn.iter() {
            let (tt, content, index) = {
                let tb = t.borrow();
                (tb.ttype, tb.content.clone(), tb.index)
            };
            match tt {
                TokenTypes::Constant => {
                    let v = t.borrow().token_value().cloned().unwrap_or_else(Value::zero);
                    stack.push(Expr::Const(v));
                }
                TokenTypes::Unit | TokenTypes::Variable | TokenTypes::Input => stack.push(self.compile_token0(t)?),
                TokenTypes::Operator if content == NEG_STR => {
                    let a = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                    stack.push(self.compile_token1(t, a)?);
                }
                TokenTypes::Function
                | TokenTypes::VectorFunction
                | TokenTypes::MatrixFunction
                | TokenTypes::MatrixOptionalFunction => {
                    let a = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                    stack.push(self.compile_token1(t, a)?);
                }
                TokenTypes::Operator
                | TokenTypes::Function2
                | TokenTypes::VectorFunction2
                | TokenTypes::MatrixFunction2
                | TokenTypes::MatrixIterativeFunction => {
                    let b = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                    let a = stack.pop().ok_or_else(exceptions::MissingOperand)?;
                    stack.push(self.compile_token2(t, a, b)?);
                }
                TokenTypes::Function3 | TokenTypes::VectorFunction3 | TokenTypes::MatrixFunction3 => {
                    let c = stack.pop().ok_or_else(stack_empty)?;
                    let b = stack.pop().ok_or_else(stack_empty)?;
                    let a = stack.pop().ok_or_else(stack_empty)?;
                    if tt == TokenTypes::Function3 {
                        if index == IF_FUNCTION_INDEX {
                            stack.push(Expr::Cond(Box::new(a), Box::new(c), Box::new(b)));
                        } else {
                            return Err(crate::calculator::index_out_of_range());
                        }
                    } else {
                        stack.push(Expr::VmFunc(tt, index, vec![a, b, c]));
                    }
                }
                TokenTypes::MatrixFunction4 => {
                    let args = pop_values(&mut stack, 4)?;
                    stack.push(Expr::VmFunc(tt, index, args));
                }
                TokenTypes::MatrixFunction5 => {
                    let args = pop_values(&mut stack, 5)?;
                    stack.push(Expr::VmFunc(tt, index, args));
                }
                TokenTypes::Interpolation => {
                    let n = t.borrow().parameter_count().max(0) as usize;
                    let args = pop_values(&mut stack, n)?;
                    if args.iter().all(Expr::is_const) {
                        let values: Vec<Value> = args.iter().map(Expr::const_value).collect();
                        stack.push(Expr::Const(Value::evaluate_interpolation(self.calc(), index, &values)?));
                    } else {
                        stack.push(Expr::Interp(index, args));
                    }
                }
                TokenTypes::MultiFunction | TokenTypes::VectorMultiFunction | TokenTypes::MatrixMultiFunction => {
                    let n = t.borrow().parameter_count().max(0) as usize;
                    let args = pop_values(&mut stack, n)?;
                    if args.iter().all(Expr::is_const) {
                        let values: Vec<Value> = args.iter().map(Expr::const_value).collect();
                        let v = match tt {
                            TokenTypes::MultiFunction => Value::evaluate_multi_function(self.calc(), index, &values)?,
                            TokenTypes::VectorMultiFunction => vc::evaluate_vector_multi_function(index, &values)?,
                            _ => mc::evaluate_matrix_multi_function(index, &values)?,
                        };
                        stack.push(Expr::Const(v));
                    } else if index == SWITCH_INDEX {
                        stack.push(parse_switch(args));
                    } else if tt == TokenTypes::MultiFunction {
                        stack.push(Expr::Multi(index, args));
                    } else {
                        stack.push(Expr::VmFunc(tt, index, args));
                    }
                }
                TokenTypes::CustomFunction => {
                    if index < 0 {
                        return Err(exceptions::InvalidFunction(&content));
                    }
                    let cf = match self.functions.borrow().get(index) {
                        Some(cf) => cf,
                        None => return Err(crate::calculator::index_out_of_range()),
                    };
                    let map = t.borrow().argument_map();
                    let n = map.as_ref().map(|m| m.len()).unwrap_or(cf.parameter_count());
                    let args = pop_values(&mut stack, n)?;
                    if cf.is_recursion.get() {
                        stack.push(Expr::Const(Value::nan()));
                    } else {
                        stack.push(Expr::Custom { cf, args, map });
                    }
                }
                TokenTypes::Vector | TokenTypes::RowDivisor => {
                    let is_literal = matches!(t.borrow().kind, TokenKind::Vector(_));
                    if is_literal {
                        let args = pop_values(&mut stack, index.max(0) as usize)?;
                        stack.push(Expr::VecLit(args));
                    } else {
                        stack.push(self.compile_token0(t)?);
                    }
                }
                TokenTypes::Matrix => {
                    let is_literal = matches!(t.borrow().kind, TokenKind::Matrix(_));
                    if is_literal {
                        let count = (index / crate::vector::MAX_LENGTH) as usize;
                        let args = pop_values(&mut stack, count)?;
                        stack.push(Expr::MatLit(args));
                    } else {
                        stack.push(self.compile_token0(t)?);
                    }
                }
                TokenTypes::VectorIndex => {
                    let i = stack.pop().ok_or_else(stack_empty)?;
                    let target = stack.pop().ok_or_else(stack_empty)?;
                    stack.push(Expr::GetVecElem(Box::new(target), Box::new(i)));
                }
                TokenTypes::MatrixIndex => {
                    let j = stack.pop().ok_or_else(stack_empty)?;
                    let i = stack.pop().ok_or_else(stack_empty)?;
                    let target = stack.pop().ok_or_else(stack_empty)?;
                    stack.push(Expr::GetMatElem(Box::new(target), Box::new(i), Box::new(j)));
                }
                TokenTypes::Solver => {
                    let sb = self.solve_blocks.borrow()[index as usize].clone();
                    stack.push(Expr::Solver(sb));
                }
                _ => return Err(exceptions::CannotEvaluateAsType(&content, &tt.name())),
            }
        }
        stack.pop().ok_or_else(exceptions::StackLeak)
    }

    fn compile_token0(&self, t: &TokenRef) -> CResult<Expr> {
        let tt = t.borrow().ttype;
        match tt {
            TokenTypes::Unit => {
                let tb = t.borrow();
                let v = match &tb.kind {
                    TokenKind::Value(v) => v.clone(),
                    TokenKind::Variable(Some(var)) => var.value().unwrap_or_else(Value::zero),
                    _ => Value::zero(),
                };
                match v {
                    Value::Real(_) => Ok(Expr::Const(v)),
                    _ => Err(MathParserException::unexpected(
                        "Unable to cast object of type 'Calcpad.Core.ComplexValue' to type 'Calcpad.Core.RealValue'.",
                    )),
                }
            }
            TokenTypes::Variable => self.compile_variable_token(t),
            TokenTypes::Vector | TokenTypes::Matrix => {
                let lit = match &t.borrow().kind {
                    TokenKind::Vector(Some(v)) => Some(Value::Vector(v.clone())),
                    TokenKind::Matrix(Some(m)) => Some(Value::Matrix(m.clone())),
                    _ => None,
                };
                match lit {
                    Some(v) => Ok(Expr::Const(v)),
                    None => self.compile_variable_token(t),
                }
            }
            TokenTypes::Input if t.borrow().content == "?" => Err(exceptions::UndefinedInputField()),
            _ => Ok(Expr::Const(
                t.borrow().token_value().cloned().unwrap_or_else(Value::zero),
            )),
        }
    }

    fn compile_variable_token(&self, t: &TokenRef) -> CResult<Expr> {
        let (v, tt, content) = {
            let tb = t.borrow();
            (tb.variable_ref(), tb.ttype, tb.content.clone())
        };
        let v = match v {
            Some(v) => v,
            None => return Err(exceptions::UndefinedVariableOrUnits(&content)),
        };
        if v.is_initialized() || tt == TokenTypes::Vector || tt == TokenTypes::Matrix {
            return Ok(Expr::Var(v));
        }
        match Unit::try_get(&content, self.is_us.get()) {
            Some(u) => {
                t.borrow_mut().ttype = TokenTypes::Unit;
                v.set_unit_value(Some(u));
                Ok(Expr::Const(v.value().unwrap()))
            }
            None => Err(exceptions::UndefinedVariableOrUnits(&content)),
        }
    }

    fn compile_token1(&self, t: &TokenRef, a: Expr) -> CResult<Expr> {
        let (tt, content, index) = {
            let tb = t.borrow();
            (tb.ttype, tb.content.clone(), tb.index)
        };
        if a.is_const() {
            let s = content.to_lowercase();
            if !(s == "random"
                || s.starts_with("range")
                || s.starts_with("vector")
                || s.starts_with("identity")
                || s.starts_with("utriang")
                || s.starts_with("ltriang")
                || s.starts_with("symmetric"))
            {
                let va = a.const_value();
                if content == NEG_STR {
                    return Ok(Expr::Const(va.neg()?));
                }
                let mut vb = match tt {
                    TokenTypes::Function => Value::evaluate_function(self.calc(), index, &va)?,
                    TokenTypes::VectorFunction => vc::evaluate_vector_function(index, &va)?,
                    TokenTypes::MatrixOptionalFunction => {
                        mc::evaluate_matrix_iterative_function(index, &va, &Value::zero(), self.tol()?)?
                    }
                    _ => {
                        if index == mc::LU_INDEX {
                            mc::lu_decomposition(&va, &self.lu_index_vector())?
                        } else {
                            mc::evaluate_matrix_function(index, &va)?
                        }
                    }
                };
                self.normalize_units(&mut vb);
                return Ok(Expr::Const(vb));
            }
        }
        Ok(match tt {
            TokenTypes::Operator => Expr::Neg(Box::new(a)),
            TokenTypes::Function => Expr::Func(index, Box::new(a)),
            TokenTypes::MatrixFunction if index == mc::LU_INDEX => Expr::Lu(Box::new(a), self.lu_index_vector()),
            _ => Expr::VmFunc(tt, index, vec![a]),
        })
    }

    fn compile_token2(&self, t: &TokenRef, a: Expr, b: Expr) -> CResult<Expr> {
        let (tt, content, index) = {
            let tb = t.borrow();
            (tb.ttype, tb.content.clone(), tb.index)
        };
        if a.is_const() && b.is_const() {
            let s = content.to_lowercase();
            if !(s.starts_with("matrix") || s.starts_with("diagonal") || s.starts_with("column")) {
                let va = a.const_value();
                let vb = b.const_value();
                let mut vc = match tt {
                    TokenTypes::Operator => Value::evaluate_operator(self.calc(), index, &va, &vb)?,
                    TokenTypes::Function2 => Value::evaluate_function2(self.calc(), index, &va, &vb)?,
                    TokenTypes::VectorFunction2 => vc::evaluate_vector_function2(index, &va, &vb)?,
                    TokenTypes::MatrixIterativeFunction => {
                        mc::evaluate_matrix_iterative_function(index, &va, &vb, self.tol()?)?
                    }
                    _ => mc::evaluate_matrix_function2(index, &va, &vb)?,
                };
                self.normalize_units(&mut vc);
                return Ok(Expr::Const(vc));
            }
        }
        match tt {
            TokenTypes::Function2 => return Ok(Expr::Func2(index, Box::new(a), Box::new(b))),
            TokenTypes::VectorFunction2 | TokenTypes::MatrixIterativeFunction | TokenTypes::MatrixFunction2 => {
                return Ok(Expr::VmFunc(tt, index, vec![a, b]))
            }
            _ => {}
        }
        self.compile_operator(&content, index, a, b)
    }

    fn compile_operator(&self, content: &str, index: i64, a: Expr, b: Expr) -> CResult<Expr> {
        let c0 = content.chars().next().unwrap_or('\0');
        if c0 == '=' || c0 == '\u{2190}' {
            return match a {
                Expr::GetVecElem(v, i) => Ok(Expr::SetVecElem(v, i, Box::new(b))),
                Expr::GetMatElem(m, i, j) => Ok(Expr::SetMatElem(m, i, j, Box::new(b))),
                Expr::Var(v) => Ok(Expr::Assign(v, Box::new(b))),
                _ => Err(MathParserException::unexpected(
                    "Expression must be writeable (Parameter 'left')",
                )),
            };
        }
        let op = match c0 {
            '+' => BinOp::Add,
            '-' => BinOp::Sub,
            '/' | '\u{00F7}' => BinOp::Div,
            '*' => BinOp::Mul,
            '<' => BinOp::Lt,
            '>' => BinOp::Gt,
            '\u{2264}' => BinOp::Le,
            '\u{2265}' => BinOp::Ge,
            '\u{2261}' => BinOp::Eq,
            '\u{2260}' => BinOp::Ne,
            '\u{2227}' => BinOp::And,
            '\u{2228}' => BinOp::Or,
            '\u{2295}' => BinOp::Xor,
            '\u{29BC}' => BinOp::Rem,
            _ => return Ok(Expr::CalcOp(index, Box::new(a), Box::new(b))),
        };
        Ok(Expr::Bin(op, Box::new(a), Box::new(b)))
    }

    /// Interprets a compiled tree (`Func<IValue>()`).
    pub(crate) fn eval_expr(&self, e: &Expr) -> CResult<Value> {
        match e {
            Expr::Const(v) => Ok(v.clone()),
            Expr::Var(v) => v.value().ok_or_else(|| exceptions::InvalidOperand("")),
            Expr::Neg(a) => self.eval_expr(a)?.neg(),
            Expr::Bin(op, a, b) => {
                let (oa, ob);
                let va: &Value = match &**a {
                    Expr::Const(c) => c,
                    _ => {
                        oa = self.eval_expr(a)?;
                        &oa
                    }
                };
                let vb: &Value = match &**b {
                    Expr::Const(c) => c,
                    _ => {
                        ob = self.eval_expr(b)?;
                        &ob
                    }
                };
                // Two real numbers: the same call `Value::binary` would reach.
                if let (Value::Real(x), Value::Real(y)) = (va, vb) {
                    match op {
                        BinOp::Add => return Ok(Value::Real(RealValue::add(x, y)?)),
                        BinOp::Sub => return Ok(Value::Real(RealValue::sub(x, y)?)),
                        BinOp::Mul => return Ok(Value::Real(RealValue::mul_op(x, y))),
                        BinOp::Div => return Ok(Value::Real(RealValue::div_op(x, y))),
                        _ => {}
                    }
                }
                match op {
                    BinOp::Add => Value::add(va, vb),
                    BinOp::Sub => Value::sub(va, vb),
                    BinOp::Mul => Value::mul(va, vb),
                    BinOp::Div => Value::div(va, vb),
                    BinOp::Rem => Value::rem(va, vb),
                    BinOp::Lt => Value::lt(va, vb),
                    BinOp::Gt => Value::gt(va, vb),
                    BinOp::Le => Value::le(va, vb),
                    BinOp::Ge => Value::ge(va, vb),
                    BinOp::Eq => Value::equal(va, vb),
                    BinOp::Ne => Value::not_equal(va, vb),
                    BinOp::And => Value::and(va, vb),
                    BinOp::Or => Value::or(va, vb),
                    BinOp::Xor => Value::xor(va, vb),
                }
            }
            Expr::CalcOp(index, a, b) => {
                let (oa, ob);
                let va: &Value = match &**a {
                    Expr::Const(c) => c,
                    _ => {
                        oa = self.eval_expr(a)?;
                        &oa
                    }
                };
                let vb: &Value = match &**b {
                    Expr::Const(c) => c,
                    _ => {
                        ob = self.eval_expr(b)?;
                        &ob
                    }
                };
                Value::evaluate_operator(self.calc(), *index, va, vb)
            }
            Expr::Func(index, a) => {
                let va = self.eval_expr(a)?;
                Value::evaluate_function(self.calc(), *index, &va)
            }
            Expr::Func2(index, a, b) => {
                let (oa, ob);
                let va: &Value = match &**a {
                    Expr::Const(c) => c,
                    _ => {
                        oa = self.eval_expr(a)?;
                        &oa
                    }
                };
                let vb: &Value = match &**b {
                    Expr::Const(c) => c,
                    _ => {
                        ob = self.eval_expr(b)?;
                        &ob
                    }
                };
                Value::evaluate_function2(self.calc(), *index, va, vb)
            }
            Expr::Cond(test, if_false_when_negative, if_true) => {
                let c = self.eval_expr(test)?;
                if c.re() < LOGICAL_ZERO {
                    self.eval_expr(if_false_when_negative)
                } else {
                    self.eval_expr(if_true)
                }
            }
            Expr::Interp(index, args) => {
                let values = self.eval_all(args)?;
                Value::evaluate_interpolation(self.calc(), *index, &values)
            }
            Expr::Multi(index, args) => {
                let values = self.eval_all(args)?;
                Value::evaluate_multi_function(self.calc(), *index, &values)
            }
            Expr::Custom { cf, args, map } => {
                let values = self.eval_all(args)?;
                match map {
                    Some(m) => self.evaluate_custom_function_map(cf, m, values),
                    None => self.evaluate_custom_function(cf, values),
                }
            }
            Expr::Assign(v, b) => {
                let vb = self.eval_expr(b)?;
                v.assign(vb);
                v.value().ok_or_else(|| exceptions::InvalidOperand(""))
            }
            Expr::Solver(sb) => {
                sb.calculate(self)?;
                Ok(sb.result())
            }
            Expr::Block(items) => {
                let mut last = Value::zero();
                for it in items {
                    last = self.eval_expr(it)?;
                }
                Ok(last)
            }
            Expr::VmFunc(tt, index, args) => {
                let v = self.eval_all(args)?;
                let index = *index;
                match tt {
                    TokenTypes::VectorFunction => vc::evaluate_vector_function(index, &v[0]),
                    TokenTypes::MatrixFunction => mc::evaluate_matrix_function(index, &v[0]),
                    TokenTypes::MatrixOptionalFunction => {
                        mc::evaluate_matrix_iterative_function(index, &v[0], &Value::zero(), self.tol()?)
                    }
                    TokenTypes::VectorFunction2 => vc::evaluate_vector_function2(index, &v[0], &v[1]),
                    TokenTypes::MatrixFunction2 => mc::evaluate_matrix_function2(index, &v[0], &v[1]),
                    TokenTypes::MatrixIterativeFunction => {
                        mc::evaluate_matrix_iterative_function(index, &v[0], &v[1], self.tol()?)
                    }
                    TokenTypes::VectorFunction3 => vc::evaluate_vector_function3(index, &v[0], &v[1], &v[2]),
                    TokenTypes::MatrixFunction3 => mc::evaluate_matrix_function3(index, &v[0], &v[1], &v[2]),
                    TokenTypes::MatrixFunction4 => mc::evaluate_matrix_function4(index, &v[0], &v[1], &v[2], &v[3]),
                    TokenTypes::MatrixFunction5 => {
                        mc::evaluate_matrix_function5(index, &v[0], &v[1], &v[2], &v[3], &v[4])
                    }
                    TokenTypes::VectorMultiFunction => vc::evaluate_vector_multi_function(index, &v),
                    _ => mc::evaluate_matrix_multi_function(index, &v),
                }
            }
            Expr::Lu(a, ind) => {
                let va = self.eval_expr(a)?;
                mc::lu_decomposition(&va, ind)
            }
            Expr::VecLit(args) => {
                let v = self.eval_all(args)?;
                Ok(Value::Vector(Vector::from_values(Value::expand_real_values(&v)?)?))
            }
            Expr::MatLit(args) => {
                let v = self.eval_all(args)?;
                Ok(Value::Matrix(mc::join_rows(&v)?))
            }
            Expr::GetVecElem(a, i) => {
                let va = self.eval_expr(a)?;
                let vi = self.eval_expr(i)?;
                vc::get_element(&va, &vi)
            }
            Expr::GetMatElem(a, i, j) => {
                let va = self.eval_expr(a)?;
                let vi = self.eval_expr(i)?;
                let vj = self.eval_expr(j)?;
                mc::get_element(&va, &vi, &vj)
            }
            Expr::SetVecElem(a, i, b) => {
                let va = self.eval_expr(a)?;
                let vi = self.eval_expr(i)?;
                let vb = self.eval_expr(b)?;
                vc::set_element(&va, &vi, &vb)
            }
            Expr::SetMatElem(a, i, j, b) => {
                let va = self.eval_expr(a)?;
                let vi = self.eval_expr(i)?;
                let vj = self.eval_expr(j)?;
                let vb = self.eval_expr(b)?;
                mc::set_element(&va, &vi, &vj, &vb)
            }
            Expr::While(cond, body) => {
                let mut result = Value::zero();
                let mut counter: i64 = 0;
                loop {
                    let c = self.eval_expr(cond)?;
                    if c.re() < LOGICAL_ZERO {
                        return Ok(result);
                    }
                    let n = body.len();
                    for (k, it) in body.iter().enumerate() {
                        let v = self.eval_expr(it)?;
                        if k == n - 1 {
                            result = v;
                        }
                    }
                    counter += 1;
                    if counter > 1_000_000_000 {
                        return Err(exceptions::InfiniteLoop(1_000_000_000));
                    }
                    self.break_if_canceled()?;
                }
            }
        }
    }

    fn eval_all(&self, args: &[Expr]) -> CResult<Vec<Value>> {
        let mut v = Vec::with_capacity(args.len());
        for a in args {
            v.push(self.eval_expr(a)?);
        }
        Ok(v)
    }
}

/// `ParseSwitchFunctionToken`: nested conditions with the negative-condition test.
fn parse_switch(mut args: Vec<Expr>) -> Expr {
    fn build(args: &mut std::collections::VecDeque<Expr>) -> Expr {
        let len = args.len();
        if len == 0 {
            return Expr::Const(Value::Real(RealValue::NAN));
        }
        if len == 1 {
            return args.pop_front().unwrap();
        }
        let a = args.pop_front().unwrap();
        let b = args.pop_front().unwrap();
        let c = build(args);
        Expr::Cond(Box::new(a), Box::new(c), Box::new(b))
    }
    let mut q: std::collections::VecDeque<Expr> = args.drain(..).collect();
    build(&mut q)
}

/// `Compiler.CompileWhileBLock`
pub fn compile_while_block(mut expressions: Vec<Expr>) -> CResult<Expr> {
    if expressions.len() < 2 {
        return Err(exceptions::InvalidNumberOfArguments());
    }
    let cond = expressions.remove(0);
    Ok(Expr::While(Box::new(cond), expressions))
}
