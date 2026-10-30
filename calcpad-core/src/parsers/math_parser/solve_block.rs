//! Port of Parsers/MathParser/MathParser.SolveBlock.cs.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use super::compiler::{compile_while_block, Expr};
use super::tokens::{Rpn, TokenTypes};
use super::{is_assignment, MathParser, VarDict};
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::base_types::variable::{ChangeEvent, ChangeListener, Parameter, VarRef};
use crate::calculator::{operator_index, OPERATOR_ORDER};
use crate::exceptions::{self, Items};
use crate::extension_methods::trim;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::output::html_writer::HtmlWriter;
use crate::output::text_writer::TextWriter;
use crate::output::xml_writer::{self, XmlWriter};
use crate::output::OutputWriter;
use crate::solver::{QuadratureMethods, Solver};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolverTypes {
    None,
    Find,
    Root,
    Sup,
    Inf,
    Area,
    Integral,
    Slope,
    Derivative,
    Repeat,
    While,
    Sum,
    Product,
    Inline,
    Block,
    Error,
}

pub fn get_solver_type(keyword: &str) -> SolverTypes {
    match trim(keyword).to_lowercase().as_str() {
        "$find" => SolverTypes::Find,
        "$root" => SolverTypes::Root,
        "$sup" => SolverTypes::Sup,
        "$inf" => SolverTypes::Inf,
        "$area" => SolverTypes::Area,
        "$integral" => SolverTypes::Integral,
        "$slope" => SolverTypes::Slope,
        "$derivative" => SolverTypes::Derivative,
        "$repeat" => SolverTypes::Repeat,
        "$while" => SolverTypes::While,
        "$sum" => SolverTypes::Sum,
        "$product" => SolverTypes::Product,
        "$inline" => SolverTypes::Inline,
        "$block" => SolverTypes::Block,
        _ => SolverTypes::Error,
    }
}

fn type_name(st: SolverTypes) -> &'static str {
    match st {
        SolverTypes::None => "",
        SolverTypes::Find => "$Find",
        SolverTypes::Root => "$Root",
        SolverTypes::Sup => "$Sup",
        SolverTypes::Inf => "$inf",
        SolverTypes::Area | SolverTypes::Integral => "\u{222B}",
        SolverTypes::Slope | SolverTypes::Derivative => "d/dt",
        SolverTypes::Repeat => "$Repeat",
        SolverTypes::While => "$While",
        SolverTypes::Sum => "\u{2211}",
        SolverTypes::Product => "\u{220F}",
        SolverTypes::Inline | SolverTypes::Block => "",
        SolverTypes::Error => "$Error",
    }
}

fn enum_name(st: SolverTypes) -> String {
    format!("{:?}", st)
}

#[derive(Clone, Default)]
pub struct SolverItem {
    pub input: Option<String>,
    pub html: Option<String>,
    pub xml: Option<String>,
    pub rpn: Option<Rpn>,
}

pub struct SolverBlock {
    local_variables: VarDict,
    stype: SolverTypes,
    var: RefCell<Option<VarRef>>,
    va: RefCell<Value>,
    vb: RefCell<Value>,
    items: RefCell<Vec<SolverItem>>,
    a: RefCell<Option<Rc<Expr>>>,
    b: RefCell<Option<Rc<Expr>>>,
    f: RefCell<Option<Rc<Expr>>>,
    y: RefCell<Option<Rc<Expr>>>,
    script: String,
    pub on_change: ChangeEvent,
    result: RefCell<Value>,
    is_figure: Cell<bool>,
    self_ref: RefCell<Weak<SolverBlock>>,
}

impl ChangeListener for SolverBlock {
    fn on_change(&self) {
        self.on_change.invoke();
    }
}

impl SolverBlock {
    pub fn new(script: &str, stype: SolverTypes, parser: &MathParser) -> CResult<Rc<SolverBlock>> {
        let sb = Rc::new(SolverBlock {
            local_variables: Rc::new(RefCell::new(HashMap::new())),
            stype,
            var: RefCell::new(None),
            va: RefCell::new(Value::nan()),
            vb: RefCell::new(Value::nan()),
            items: RefCell::new(Vec::new()),
            a: RefCell::new(None),
            b: RefCell::new(None),
            f: RefCell::new(None),
            y: RefCell::new(None),
            script: script.to_string(),
            on_change: ChangeEvent::default(),
            result: RefCell::new(Value::zero()),
            is_figure: Cell::new(false),
            self_ref: RefCell::new(Weak::new()),
        });
        *sb.self_ref.borrow_mut() = Rc::downgrade(&sb);
        sb.parse(parser)?;
        Ok(sb)
    }

    fn listener(&self) -> Weak<dyn ChangeListener> {
        let w: Weak<SolverBlock> = self.self_ref.borrow().clone();
        w as Weak<dyn ChangeListener>
    }

    pub fn result(&self) -> Value {
        self.result.borrow().clone()
    }

    pub fn is_figure(&self) -> bool {
        self.is_figure.get()
    }

    fn is_block(&self) -> bool {
        matches!(
            self.stype,
            SolverTypes::Inline | SolverTypes::Block | SolverTypes::While
        )
    }

    fn parse(&self, parser: &MathParser) -> CResult<()> {
        let target_units = parser.target_units.borrow().clone();
        let items = if self.is_block() || self.stype == SolverTypes::Repeat {
            parser.add_local_variables(self.local_variables.clone());
            let r = if self.is_block() {
                self.parse_block_or_inline(&self.script, parser)
            } else {
                self.parse_solver(&self.script, parser)
            };
            parser.remove_local_variables();
            r?
        } else {
            self.parse_solver(&self.script, parser)?
        };
        *self.items.borrow_mut() = items;
        *parser.target_units.borrow_mut() = target_units;
        self.render_output(parser)
    }

    fn parse_solver(&self, script: &str, parser: &MathParser) -> CResult<Vec<SolverItem>> {
        let mut n = 3usize;
        if self.stype == SolverTypes::Slope || self.stype == SolverTypes::Derivative {
            n = 2;
        }
        let delimiters = ['@', '=', ':'];
        let chars: Vec<char> = script.chars().collect();
        let mut items: Vec<SolverItem> = vec![SolverItem::default(); n + 1];
        let mut current = 0usize;
        let mut bracket_counter = 0i32;
        let mut equality_index: Option<usize> = None;
        let mut ts_start = 0usize;
        let mut ts_end = 0usize;
        for (i, &c) in chars.iter().enumerate() {
            if c == '{' {
                bracket_counter += 1;
            } else if c == '}' {
                bracket_counter -= 1;
            }
            if self.stype == SolverTypes::Root && bracket_counter == 0 && current == 0 && c == '=' {
                if equality_index.is_none() {
                    equality_index = Some(i);
                } else {
                    let cut: String = chars[ts_start..ts_end].iter().collect();
                    return Err(exceptions::MultipleAssignments(&format!("{} ...", cut)));
                }
            }
            if bracket_counter == 0 && current < n && c == delimiters[current] {
                ts_end = i;
                items[current].input = Some(chars[ts_start..ts_end].iter().collect());
                ts_start = i + 1;
                ts_end = i + 1;
                current += 1;
            } else {
                ts_end += 1;
            }
        }
        ts_end = chars.len();
        items[current].input = Some(chars[ts_start.min(ts_end)..ts_end].iter().collect());
        for i in 0..=n {
            let blank = items[i].input.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true);
            if blank {
                let j = if i == 0 { 0 } else { i - 1 };
                return Err(exceptions::MissingDelimiter(
                    delimiters[j],
                    &format!("{}{{{}}}", type_name(self.stype), script),
                ));
            }
        }
        if self.stype == SolverTypes::Repeat {
            let first = items[0].input.clone().unwrap();
            let additional = self.parse_block_or_inline(&first, parser)?;
            let len = additional.len() - 1;
            if len > 0 {
                items[0] = additional[0].clone();
                n = 3 + len;
                let mut result: Vec<SolverItem> = items[..4].to_vec();
                result.extend_from_slice(&additional[1..]);
                items = result;
            }
        } else if self.stype == SolverTypes::Root {
            if let Some(eq) = equality_index {
                let s: Vec<char> = items[0].input.clone().unwrap().chars().collect();
                items[0].input = Some(s[..eq].iter().collect());
                items.resize(5, SolverItem::default());
                items[4].input = Some(s[eq + 1..].iter().collect());
                n = 4;
            }
        }
        let allow_assignment = self.stype == SolverTypes::Repeat || self.stype == SolverTypes::Root;
        for (i, item) in items.iter_mut().enumerate().take(n + 1) {
            let input = trim(item.input.as_deref().unwrap_or("")).to_string();
            item.input = Some(input.clone());
            parser.parse(&input, (i == 0 || i > 3) && allow_assignment)?;
            item.rpn = parser.rpn.borrow().clone();
        }
        let rpn = items[1].rpn.clone().unwrap_or_else(|| Rc::new(Vec::new()));
        let parameters: Vec<Parameter>;
        if rpn.len() == 1 && rpn[0].borrow().is_variable_token() {
            let content = rpn[0].borrow().content.clone();
            let p = Parameter::new(&content);
            rpn[0].borrow_mut().set_variable(Some(p.variable.clone()));
            *self.var.borrow_mut() = Some(p.variable.clone());
            parameters = vec![p];
        } else {
            return Err(exceptions::CounterMustBeASingleVariableName());
        }
        if self.stype == SolverTypes::Inf || self.stype == SolverTypes::Sup {
            let s = format!(
                "{}{}",
                items[1].input.as_deref().unwrap_or(""),
                if self.stype == SolverTypes::Sup { "_sup" } else { "_inf" }
            );
            parser.set_variable(&s, Value::nan());
        }
        if parser.is_enabled.get() {
            if let Some(r0) = &items[0].rpn {
                parser.bind_parameters(&parameters, r0);
                parser.subscribe_on_change(r0, self.listener());
            }
            if let Some(r2) = &items[2].rpn {
                parser.subscribe_on_change(r2, self.listener());
            }
            if items.len() > 3 {
                if let Some(r3) = &items[3].rpn {
                    parser.subscribe_on_change(r3, self.listener());
                }
            }
        }
        if self.stype == SolverTypes::Repeat {
            for i in 0..items.len() {
                if i == 0 || i > 3 {
                    if let Some(r) = items[i].rpn.clone() {
                        if !r.is_empty() {
                            fix_repeat(&r);
                            if i > 3 {
                                parser.bind_parameters(&parameters, &r);
                                parser.subscribe_on_change(&r, self.listener());
                            }
                        }
                    }
                }
            }
        }
        self.is_figure.set(matches!(
            self.stype,
            SolverTypes::Sum | SolverTypes::Product | SolverTypes::Integral | SolverTypes::Area
        ));
        Ok(items)
    }

    fn parse_block_or_inline(&self, script: &str, parser: &MathParser) -> CResult<Vec<SolverItem>> {
        let chars: Vec<char> = script.chars().collect();
        let (mut br, mut cur, mut sq) = (0i32, 0i32, 0i32);
        let mut list: Vec<SolverItem> = Vec::new();
        let mut start = 0usize;
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '(' => br += 1,
                ')' => br -= 1,
                '[' => sq += 1,
                ']' => sq -= 1,
                '{' => cur += 1,
                '}' => cur -= 1,
                _ => {}
            }
            if br == 0 && sq == 0 && cur == 0 && c == ';' {
                list.push(SolverItem {
                    input: Some(chars[start..i].iter().collect()),
                    ..Default::default()
                });
                start = i + 1;
            }
        }
        list.push(SolverItem {
            input: Some(chars[start.min(chars.len())..].iter().collect()),
            ..Default::default()
        });
        for item in list.iter_mut() {
            let input = trim(item.input.as_deref().unwrap_or("")).to_string();
            item.input = Some(input.clone());
            parser.parse(&input, true)?;
            item.rpn = parser.rpn.borrow().clone();
        }
        Ok(list)
    }

    fn render_output(&self, parser: &MathParser) -> CResult<()> {
        let n = self.items.borrow().len();
        for i in 0..n {
            let rpn = self.items.borrow()[i].rpn.clone();
            if let Some(r) = rpn {
                *parser.rpn.borrow_mut() = Some(r);
                let html = parser.to_html()?;
                let xml = parser.to_xml()?;
                let mut items = self.items.borrow_mut();
                items[i].html = Some(html);
                items[i].xml = Some(xml);
            }
        }
        if self.is_figure.get() {
            let order = OPERATOR_ORDER[operator_index('*').unwrap()];
            let last_order = {
                let items = self.items.borrow();
                let r = items[0].rpn.as_ref().unwrap();
                let o = r[r.len() - 1].borrow().order;
                o
            };
            if last_order > order {
                let w = HtmlWriter::new(Some(&parser.settings.borrow()), parser.phasor.get());
                let mut items = self.items.borrow_mut();
                let h = items[0].html.clone().unwrap_or_default();
                items[0].html = Some(w.add_brackets(&h, 1, '(', ')'));
                let x = items[0].xml.clone().unwrap_or_default();
                items[0].xml = Some(xml_writer::brackets(&x, '(', ')'));
            }
        }
        Ok(())
    }

    fn compile(&self, parser: &MathParser) -> CResult<()> {
        let allow_assignment = self.stype == SolverTypes::Repeat || self.is_block();
        let items = self.items.borrow().clone();
        let rpn0 = items[0].rpn.clone().unwrap_or_else(|| Rc::new(Vec::new()));
        *self.f.borrow_mut() = Some(Rc::new(parser.compile_rpn(&rpn0, allow_assignment)?));
        let len = items.len();
        if allow_assignment {
            let i0 = if self.stype == SolverTypes::Repeat { 4 } else { 1 };
            if len > i0 {
                let e = parser.rpn_to_expression_tree(&rpn0, allow_assignment)?;
                let mut expressions = vec![e];
                for item in items.iter().skip(i0) {
                    if let Some(r) = &item.rpn {
                        if !r.is_empty() {
                            expressions.push(parser.rpn_to_expression_tree(r, true)?);
                        }
                    }
                }
                let f = if self.stype == SolverTypes::While {
                    compile_while_block(expressions)?
                } else {
                    Expr::Block(expressions)
                };
                *self.f.borrow_mut() = Some(Rc::new(f));
            }
            if i0 == 1 {
                return Ok(());
            }
        }
        let rpn = items[2].rpn.clone().unwrap_or_else(|| Rc::new(Vec::new()));
        if rpn.len() == 1 && rpn[0].borrow().ttype == TokenTypes::Constant {
            *self.va.borrow_mut() = rpn[0].borrow().token_value().cloned().unwrap_or_else(Value::zero);
        } else {
            *self.a.borrow_mut() = Some(Rc::new(parser.compile_rpn(&rpn, false)?));
        }
        if len > 3 {
            if let Some(rpn) = items[3].rpn.clone() {
                if rpn.len() == 1 && rpn[0].borrow().ttype == TokenTypes::Constant {
                    *self.vb.borrow_mut() = rpn[0].borrow().token_value().cloned().unwrap_or_else(Value::zero);
                } else {
                    *self.b.borrow_mut() = Some(Rc::new(parser.compile_rpn(&rpn, false)?));
                }
            }
        }
        if len > 4 {
            if let Some(rpn) = items[4].rpn.clone() {
                if !rpn.is_empty() {
                    *self.y.borrow_mut() = Some(Rc::new(parser.compile_rpn(&rpn, false)?));
                }
            }
        }
        Ok(())
    }

    pub fn bind_parameters(&self, parameters: &[Parameter], parser: &MathParser) {
        if parser.is_enabled.get() {
            let items = self.items.borrow().clone();
            for (i, item) in items.iter().enumerate() {
                if i != 1 || self.is_block() {
                    if let Some(r) = &item.rpn {
                        parser.bind_parameters(parameters, r);
                    }
                }
            }
        }
    }

    fn eval_opt(&self, parser: &MathParser, e: &RefCell<Option<Rc<Expr>>>, v: &RefCell<Value>) -> CResult<Value> {
        let f = e.borrow().clone();
        match f {
            Some(f) => parser.eval_expr(&f),
            None => Ok(v.borrow().clone()),
        }
    }

    /// `SolverBlock.Calculate()`
    pub fn calculate(&self, parser: &MathParser) -> CResult<Value> {
        if self.f.borrow().is_none() {
            self.compile(parser)?;
        }
        let f = self.f.borrow().clone().unwrap();
        if self.is_block() {
            let r = parser.eval_expr(&f)?;
            *self.result.borrow_mut() = r.clone();
            return Ok(r);
        }
        let x1 = self
            .eval_opt(parser, &self.a, &self.va)?
            .as_real_checked(Items::Argument)?;
        let var = self.var.borrow().clone().unwrap();
        if self.stype == SolverTypes::Derivative {
            let r = self.derivative(parser, &f, &var, &x1)?;
            *self.result.borrow_mut() = r.clone();
            return Ok(r);
        }
        let mut x2 = RealValue::ZERO;
        let mut y = 0.0;
        let ux1 = x1.units.clone();
        if self.stype != SolverTypes::Slope {
            x2 = self
                .eval_opt(parser, &self.b, &self.vb)?
                .as_real_checked(Items::Limit)?;
            let ux2 = x2.units.clone();
            if !Unit::is_consistent_opt(&ux1, &ux2) {
                return Err(exceptions::InconsistentUnits2(
                    self.items.borrow()[0].input.as_deref().unwrap_or(""),
                    &Unit::get_text_of(&ux1),
                    &Unit::get_text_of(&ux2),
                ));
            }
            if let Some(u2) = &ux2 {
                x2 = x2.mul_f(u2.convert_to(ux1.as_ref().unwrap()));
            }
        }
        var.set_value(Value::Real(x1.clone()));
        let yf = self.y.borrow().clone();
        if self.stype == SolverTypes::Root {
            if let Some(yf) = yf {
                let y1 = parser.eval_expr(&f)?.as_real_checked(Items::Result)?;
                let uy1 = y1.units.clone();
                let y1 = parser.eval_expr(&yf)?.as_real_checked(Items::Result)?;
                var.set_number(x2.d);
                let y2 = parser.eval_expr(&yf)?.as_real_checked(Items::Result)?;
                if (y2.d - y1.d).abs() > 1e-14 {
                    return Err(exceptions::NotConstantExpression(
                        self.items.borrow()[4].input.as_deref().unwrap_or(""),
                    ));
                }
                y = y1.d;
                let uy2 = y2.units.clone();
                if !Unit::is_consistent_opt(&uy1, &uy2) {
                    let items = self.items.borrow();
                    return Err(exceptions::InconsistentUnits1(
                        items[0].input.as_deref().unwrap_or(""),
                        items[4].input.as_deref().unwrap_or(""),
                    ));
                }
                if let Some(u2) = &uy2 {
                    y *= u2.convert_to(uy1.as_ref().unwrap());
                }
            }
        }
        let mut solver = Solver {
            parser,
            is_complex: parser.settings.borrow().is_complex,
            precision: parser.precision()?,
            quadrature_method: QuadratureMethods::AdaptiveLobatto,
            units: None,
            function: f.clone(),
            variable: var.clone(),
        };
        let mut d = 0.0;
        // `++_isSolver` is not undone when the solver throws (as in C#).
        parser.is_solver.set(parser.is_solver.get() + 1);
        let is_complex = parser.settings.borrow().is_complex;
        let r: CResult<Value> = (|| -> CResult<Value> {
            Ok(match self.stype {
                SolverTypes::Find => {
                    d = solver.find(x1.d, x2.d)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Root => {
                    d = solver.root(x1.d, x2.d, y)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Sup => {
                    d = solver.sup(x1.d, x2.d)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Inf => {
                    d = solver.inf(x1.d, x2.d)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Area => {
                    solver.quadrature_method = QuadratureMethods::AdaptiveLobatto;
                    d = solver.area(x1.d, x2.d)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Integral => {
                    solver.quadrature_method = QuadratureMethods::TanhSinh;
                    d = solver.area(x1.d, x2.d)?;
                    Value::Real(RealValue::num(d))
                }
                SolverTypes::Repeat => {
                    if is_complex {
                        Value::Complex(ComplexValue::c(solver.complex_repeat(x1.d, x2.d)?))
                    } else {
                        solver.repeat(x1.d, x2.d)?
                    }
                }
                SolverTypes::Sum => {
                    if is_complex {
                        Value::Complex(ComplexValue::c(solver.complex_sum(x1.d, x2.d)?))
                    } else {
                        Value::Real(RealValue::num(solver.sum(x1.d, x2.d)?))
                    }
                }
                SolverTypes::Product => {
                    if is_complex {
                        Value::Complex(ComplexValue::c(solver.complex_product(x1.d, x2.d)?))
                    } else {
                        Value::Real(RealValue::num(solver.product(x1.d, x2.d)?))
                    }
                }
                SolverTypes::Slope => {
                    d = solver.slope(x1.d)?;
                    Value::Real(RealValue::num(d))
                }
                _ => Value::nan(),
            })
        })();
        let result = match r {
            Ok(v) => v,
            Err(e) => {
                if e.message.contains("%F") {
                    let items = self.items.borrow();
                    let s = e
                        .message
                        .replace("%F", items[0].input.as_deref().unwrap_or(""))
                        .replace("%V", items[1].input.as_deref().unwrap_or(""));
                    // The words changed after the message was made, so it no longer carries its code.
                    return Err(MathParserException {
                        message: s,
                        unexpected: e.unexpected,
                        code: None,
                        args: Vec::new(),
                    });
                }
                return Err(e);
            }
        };
        if self.stype == SolverTypes::Sup || self.stype == SolverTypes::Inf {
            let s = format!(
                "{}{}",
                self.items.borrow()[1].input.as_deref().unwrap_or(""),
                if self.stype == SolverTypes::Sup { "_sup" } else { "_inf" }
            );
            let v = var.value().unwrap_or_else(Value::nan);
            let rv =
                match &v {
                    Value::Real(r) => r.clone(),
                    Value::Complex(_) => return Err(MathParserException::unexpected(
                        "Unable to cast object of type 'Calcpad.Core.ComplexValue' to type 'Calcpad.Core.RealValue'.",
                    )),
                    other => {
                        return Err(MathParserException::unexpected(format!(
                            "Unable to cast object of type '{}' to type 'Calcpad.Core.RealValue'.",
                            other.to_cs_string()
                        )))
                    }
                };
            parser.set_variable(&s, Value::Real(rv));
        }
        parser.is_solver.set(parser.is_solver.get() - 1);
        if d.is_nan() && !parser.is_plotting.get() {
            return Err(exceptions::NoSolution(&self.to_text()));
        }
        let su: OptUnit = solver.units.clone();
        let res = match result {
            Value::Real(real) => Value::Real(RealValue::new(real.d, su)),
            Value::Complex(c) => {
                if c.b == 0.0 {
                    Value::Real(RealValue::new(c.a, su))
                } else {
                    Value::Complex(ComplexValue::new(c.a, c.b, su))
                }
            }
            other => other,
        };
        *self.result.borrow_mut() = res.clone();
        Ok(res)
    }

    fn derivative(&self, parser: &MathParser, f: &Rc<Expr>, var: &VarRef, value: &RealValue) -> CResult<Value> {
        let is_real = !parser.settings.borrow().is_complex;
        let h = 1e-20;
        let x = ComplexValue::new(value.d, h, value.units.clone());
        var.set_value(Value::Complex(x));
        let result = if is_real {
            parser.set_complex(true);
            let r = parser.eval_expr(f)?;
            parser.set_complex(false);
            r
        } else {
            parser.eval_expr(f)?
        };
        let cv = result.as_complex();
        let units = match (&cv.units, &value.units) {
            (None, None) => None,
            (Some(a), Some(b)) => Unit::divide_units(a, b),
            _ => {
                return Err(MathParserException::unexpected(
                    "Object reference not set to an instance of an object.",
                ))
            }
        };
        Ok(Value::Real(RealValue::new(cv.b / h, units)))
    }

    pub fn to_html(&self, format_equations: bool, parser: &MathParser) -> String {
        let items = self.items.borrow();
        let len = items.len();
        let w = HtmlWriter::new(Some(&parser.settings.borrow()), parser.phasor.get());
        let html = |i: usize| items[i].html.clone().unwrap_or_default();
        let input = |i: usize| items[i].input.clone().unwrap_or_default();
        if self.is_block() {
            let mut h: Vec<String> = (0..len).map(html).collect();
            if self.stype == SolverTypes::Inline {
                h = vec![h.join("; ")];
            } else if self.stype == SolverTypes::While {
                h[0] = format!("<span class=\"cond\">while</span> {}", h[0]);
            }
            return w.format_block(&h);
        }
        if format_equations {
            if self.stype == SolverTypes::Integral || self.stype == SolverTypes::Area {
                return w.format_nary(
                    &format!("<em>{}</em>", type_name(self.stype)),
                    &format!("{}&nbsp;", html(2)),
                    &format!("&emsp; {}", html(3)),
                    &format!("{} d{}", html(0), html(1)),
                );
            }
            if self.stype == SolverTypes::Sum || self.stype == SolverTypes::Product {
                return w.format_nary(
                    type_name(self.stype),
                    &format!("{}=&hairsp;{}", html(1), html(2)),
                    &html(3),
                    &html(0),
                );
            }
            if self.stype == SolverTypes::Slope || self.stype == SolverTypes::Derivative {
                return format!(
                    "{}<span class=\"low\"><em>{}</em>\u{200A}=\u{200A}{}</span>",
                    w.add_brackets(
                        &format!(
                            "{}&nbsp;{}",
                            w.format_division("<em>d</em>", &format!("<em>d</em>\u{200A}{}", html(1)), 0),
                            html(0)
                        ),
                        1,
                        ' ',
                        '|'
                    ),
                    input(1),
                    html(2)
                );
            }
        }
        let mut sb = format!("<span class=\"cond\">{}</span>", type_name(self.stype));
        if self.stype == SolverTypes::Repeat && len > 4 {
            let mut h: Vec<String> = vec![String::new(); len - 2];
            h[0] = format!(
                "<span class=\"cond\">for</span> {} = {}...{}",
                html(1),
                html(2),
                html(3)
            );
            h[1] = html(0);
            for i in 4..len {
                h[i - 2] = html(i);
            }
            sb.push(' ');
            sb.push_str(&w.format_block(&h));
            return sb;
        }
        sb.push('{');
        sb.push_str(&html(0));
        if self.stype == SolverTypes::Root {
            if len > 4 && items[4].html.is_some() {
                sb.push_str(&format!(" = {}", html(4)));
            } else {
                sb.push_str(" = 0");
            }
        }
        if self.stype == SolverTypes::Repeat {
            sb.push_str(" <span class=\"cond\">for</span> ");
        } else {
            sb.push_str("; ");
        }
        sb.push_str(&html(1));
        if matches!(
            self.stype,
            SolverTypes::Repeat | SolverTypes::Slope | SolverTypes::Derivative
        ) {
            sb.push_str(" = ");
            sb.push_str(&html(2));
            if self.stype == SolverTypes::Repeat {
                sb.push_str("...");
                sb.push_str(&html(3));
            }
            sb.push('}');
        } else {
            sb.push_str(" \u{2208} [");
            sb.push_str(&html(2));
            sb.push_str("; ");
            sb.push_str(&html(3));
            sb.push_str("]}");
        }
        sb
    }

    /// `ToXml()`: the block as Office Math, for the Word export.
    pub fn to_xml(&self, parser: &MathParser) -> String {
        use xml_writer::{brackets, run, run_f, NORMAL_TEXT};
        let items = self.items.borrow();
        let len = items.len();
        let w = XmlWriter::new(Some(&parser.settings.borrow()), parser.phasor.get());
        let xml = |i: usize| items[i].xml.clone().unwrap_or_default();
        let input = |i: usize| items[i].input.clone().unwrap_or_default();
        let name = type_name(self.stype);
        if self.is_block() {
            let mut x: Vec<String> = (0..len).map(xml).collect();
            if self.stype == SolverTypes::Inline {
                x = vec![x.join(&run("; "))];
            } else if self.stype == SolverTypes::While {
                x[0] = format!("{} {}", run_f(name, NORMAL_TEXT), x[0]);
            }
            return w.format_block(&x);
        }
        if self.stype == SolverTypes::Integral || self.stype == SolverTypes::Area {
            return w.format_nary(name, &xml(2), &xml(3), &format!("{}{}{}", xml(0), run(" d"), xml(1)));
        }
        if self.stype == SolverTypes::Sum || self.stype == SolverTypes::Product {
            return w.format_nary(name, &format!("{}{}{}", xml(1), run("="), xml(2)), &xml(3), &xml(0));
        }
        if self.stype == SolverTypes::Slope || self.stype == SolverTypes::Derivative {
            return w.format_subscript(
                &w.add_brackets(
                    &(w.format_division(&run("d"), &format!("{}{}", run("d"), xml(1)), 0) + &xml(0)),
                    1,
                    ' ',
                    '|',
                ),
                &format!("{}{}{}", run(&input(1)), run("\u{2009}=\u{2009}"), xml(2)),
            );
        }
        let mut sb = String::new();
        if self.stype == SolverTypes::Repeat && len > 4 {
            sb.push_str(&run_f(name, NORMAL_TEXT));
            let mut x: Vec<String> = vec![String::new(); len - 2];
            x[0] = format!(
                "{}{}{}{}{}{}",
                run_f("for ", NORMAL_TEXT),
                xml(1),
                run("="),
                xml(2),
                run("..."),
                xml(3)
            );
            x[1] = xml(0);
            for i in 4..len {
                x[i - 2] = xml(i);
            }
            sb.push_str(&w.format_block(&x));
            return sb;
        }
        sb.push_str(&xml(0));
        if self.stype == SolverTypes::Root {
            if len > 4 && items[4].xml.is_some() {
                sb.push_str(&(run("=") + &xml(4)));
            } else {
                sb.push_str(&run("=0"));
            }
        }
        if self.stype == SolverTypes::Repeat {
            sb.push_str(&run_f(" for ", NORMAL_TEXT));
        } else {
            sb.push_str(&run(";"));
        }
        sb.push_str(&xml(1));
        if matches!(
            self.stype,
            SolverTypes::Repeat | SolverTypes::Slope | SolverTypes::Derivative
        ) {
            sb.push_str(&run("="));
            sb.push_str(&xml(2));
            if self.stype == SolverTypes::Repeat {
                sb.push_str(&run("..."));
                sb.push_str(&xml(3));
            }
        } else {
            sb.push_str(&run("\u{2208}"));
            sb.push_str(&brackets(&format!("{}{}{}", xml(2), run(";"), xml(3)), '[', ']'));
        }
        run_f(name, NORMAL_TEXT) + &brackets(&sb, '{', '}')
    }

    /// `ToString()`
    pub fn to_text(&self) -> String {
        let items = self.items.borrow();
        let len = items.len();
        let w = TextWriter::new(None, false);
        let input = |i: usize| items[i].input.clone().unwrap_or_default();
        if self.is_block() {
            let mut t: Vec<String> = (0..len).map(input).collect();
            if self.stype == SolverTypes::Inline {
                t = vec![t.join("; ")];
            } else if self.stype == SolverTypes::While {
                t[0] = format!("<span class=\"cond\">while</span> {}", t[0]);
            }
            return w.format_block(&t);
        }
        if matches!(
            self.stype,
            SolverTypes::Sum | SolverTypes::Product | SolverTypes::Integral | SolverTypes::Area | SolverTypes::Repeat
        ) {
            return w.format_nary(
                &format!("${}", enum_name(self.stype)),
                &format!("{} = {}", input(1), input(2)),
                &input(3),
                &input(0),
            );
        }
        let mut sb = String::new();
        sb.push_str(type_name(self.stype));
        sb.push('{');
        sb.push_str(&input(0));
        if self.stype == SolverTypes::Root {
            if len > 4 && items[4].input.is_some() {
                sb.push_str(&format!(" = {}", input(4)));
            } else {
                sb.push_str(" = 0");
            }
        }
        sb.push_str("; ");
        sb.push_str(&input(1));
        if self.stype == SolverTypes::Slope || self.stype == SolverTypes::Derivative {
            sb.push_str(" = ");
            sb.push_str(&input(2));
            sb.push('}');
        } else {
            sb.push_str(" \u{2208} [");
            sb.push_str(&input(2));
            sb.push_str("; ");
            sb.push_str(&input(3));
            sb.push_str("]}");
        }
        sb
    }
}

fn fix_repeat(rpn: &Rpn) {
    let last_is_assignment = is_assignment(&rpn[rpn.len() - 1].borrow().content);
    let (idx0, content0) = {
        let r0 = rpn[0].borrow();
        (r0.index, r0.content.clone())
    };
    if last_is_assignment && idx0 == -1 {
        rpn[0].borrow_mut().index = 1;
        for t in rpn.iter() {
            let mut tb = t.borrow_mut();
            if tb.ttype == TokenTypes::Variable && tb.content == content0 {
                tb.index = 1;
            }
        }
    }
}

impl Unit {
    /// `operator /(Unit, Unit)` (MultiplyOrDivide without the factor).
    pub fn divide_units(a: &crate::base_types::unit::UnitRef, b: &crate::base_types::unit::UnitRef) -> OptUnit {
        Unit::divide_raw(a, b)
    }
}
