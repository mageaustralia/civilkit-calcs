//! Port of Parsers/ExpressionParser/ExpressionParser.Loops.cs.
use crate::base_types::complex::math_sign;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::OptUnit;
use crate::base_types::value::Value;
use crate::base_types::variable::VarRef;
use crate::math_parser_exception::CResult;

pub const MAX_COUNT: i32 = 100000000;

pub enum LoopKind {
    Repeat,
    For {
        counter_re: f64,
        counter_im: f64,
        inc_re: f64,
        inc_im: f64,
        inc_units: OptUnit,
        counter: VarRef,
    },
    While {
        condition: String,
    },
}

pub struct Loop {
    start_line: usize,
    iteration: i32,
    pub id: usize,
    pub initial_count: i32,
    /// Source line of the loop keyword, 0 when unknown
    pub source_line: i32,
    pub kind: LoopKind,
}

impl Loop {
    fn base(start_line: usize, count: f64, id: usize, kind: LoopKind) -> Loop {
        let count = if count < 0.0 || count > MAX_COUNT as f64 {
            MAX_COUNT as f64
        } else {
            count
        };
        // (int)NaN is int.MinValue in .NET.
        let it = if count.is_nan() { i32::MIN } else { count as i32 };
        Loop {
            start_line,
            iteration: it,
            id,
            initial_count: it,
            source_line: 0,
            kind,
        }
    }

    pub fn repeat(start_line: usize, count: f64, id: usize) -> Loop {
        Loop::base(start_line, count, id, LoopKind::Repeat)
    }

    pub fn while_loop(start_line: usize, condition: String, id: usize) -> Loop {
        Loop::base(start_line, MAX_COUNT as f64, id, LoopKind::While { condition })
    }

    pub fn for_loop(start_line: usize, start: &Value, end: &Value, counter: VarRef, id: usize) -> CResult<Loop> {
        let diff = Value::s_sub(start, end)?;
        let count = diff.re().abs() + 1.0;
        let inc = Value::s_sub_neg(&diff);
        let inc_re = math_sign(inc.re())? as f64;
        let inc_im = math_sign(inc.im())? as f64;
        let inc_units = inc.units().clone();
        counter.assign(start.clone());
        Ok(Loop::base(
            start_line,
            count,
            id,
            LoopKind::For {
                counter_re: start.re(),
                counter_im: start.im(),
                inc_re,
                inc_im,
                inc_units,
                counter,
            },
        ))
    }

    pub fn start_line(&self) -> usize {
        self.start_line
    }

    /// 1 based pass number. `iteration` counts down, so this counts up.
    pub fn pass(&self) -> i32 {
        self.initial_count - self.iteration + 1
    }

    pub fn is_first_pass(&self) -> bool {
        self.iteration == self.initial_count
    }

    pub fn iterate(&mut self, current_line: &mut usize) -> bool {
        if self.iteration <= 1 {
            return false;
        }
        *current_line = self.start_line;
        self.iteration -= 1;
        true
    }

    pub fn break_loop(&mut self) {
        self.iteration = 0;
    }

    pub fn is_broken(&self) -> bool {
        self.iteration == 0
    }

    pub fn increment_counter(&mut self) {
        if let LoopKind::For {
            counter_re,
            counter_im,
            inc_re,
            inc_im,
            inc_units,
            counter,
        } = &mut self.kind
        {
            *counter_re += *inc_re;
            if *inc_im == 0.0 {
                counter.assign(Value::Real(RealValue::new(*counter_re, inc_units.clone())));
            } else {
                *counter_im += *inc_im;
                counter.assign(Value::Complex(ComplexValue::new(
                    *counter_re,
                    *counter_im,
                    inc_units.clone(),
                )));
            }
        }
    }
}
