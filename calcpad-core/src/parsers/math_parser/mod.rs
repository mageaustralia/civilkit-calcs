//! Port of Parsers/MathParser/MathParser.cs.
//!
//! The C# MathParser is a class with nested Input, Evaluator, Compiler and Output classes
//! that all hold a reference back to the parser. Here the parser is a single struct with
//! interior mutability and the nested classes are modules with `impl MathParser` blocks.
pub mod compiler;
pub mod custom_function;
pub mod data;
pub mod evaluator;
pub mod input;
pub mod keyword_args;
pub mod number_parser;
pub mod output;
pub mod solve_block;
pub mod syntax_analyser;
pub mod tokens;
pub mod ui;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::base_types::complex::Complex;
use crate::base_types::complex_value::ComplexValue;
use crate::base_types::real_value::RealValue;
use crate::base_types::unit::{OptUnit, Unit};
use crate::base_types::value::Value;
use crate::base_types::variable::{ChangeListener, Parameter, VarRef, Variable};
use crate::calculator::Calculator;
use crate::exceptions::{self, Items};
use crate::math_parser_exception::CResult;
use crate::netfmt;
use crate::output::{OutputFormat, VECTOR_SPACING};
use crate::settings::MathSettings;

use compiler::Expr;
use custom_function::{Container, CustomFunction, CustomFunctionRef};
use solve_block::SolverBlock;
use tokens::{Rpn, Token, TokenKind, TokenRef, TokenTypes};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VariableSubstitutionOptions {
    VariablesAndSubstitutions,
    VariablesOnly,
    SubstitutionsOnly,
}

pub type VarDict = Rc<RefCell<HashMap<String, VarRef>>>;

pub struct Equation {
    pub rpn: Rpn,
    pub target_units: OptUnit,
    pub function: Option<Rc<Expr>>,
}

pub struct MathParser {
    pub(crate) settings: Rc<RefCell<MathSettings>>,
    pub(crate) rpn: RefCell<Option<Rpn>>,
    pub(crate) solve_blocks: RefCell<Vec<Rc<SolverBlock>>>,
    pub(crate) functions: RefCell<Container>,
    pub(crate) variables: VarDict,
    pub(crate) units: RefCell<HashMap<String, OptUnit>>,
    // Input state
    pub(crate) defined_variables: RefCell<HashSet<String>>,
    pub(crate) local_variables: RefCell<Vec<VarDict>>,
    // Calculators
    pub(crate) real_calc: Calculator,
    pub(crate) complex_calc: Calculator,
    pub(crate) is_complex_calc: Cell<bool>,
    /// The calculator captured by the Evaluator and Compiler at construction.
    pub(crate) evaluator_calc_complex: bool,
    pub(crate) equation_cache: RefCell<Vec<Equation>>,
    pub(crate) result: RefCell<Value>,
    pub(crate) assignment_index: Cell<usize>,
    pub(crate) is_calculated: Cell<bool>,
    pub(crate) is_solver: Cell<i32>,
    pub(crate) target_units: RefCell<OptUnit>,
    pub is_us: Cell<bool>,
    pub(crate) function_definition_index: Cell<i64>,
    pub(crate) backup_variable: RefCell<(Option<String>, Option<Value>)>,
    pub has_input_fields: Cell<bool>,
    pub line: Cell<i32>,
    pub variable_substitution: Cell<VariableSubstitutionOptions>,
    pub(crate) result_units: RefCell<OptUnit>,
    pub is_canceled: Cell<bool>,
    pub is_enabled: Cell<bool>,
    pub is_plotting: Cell<bool>,
    pub is_calculation: Cell<bool>,
    pub split: Cell<bool>,
    pub show_warnings: Cell<bool>,
    pub phasor: Cell<bool>,
    pub is_const: Cell<bool>,
    /// Shared `Solver._eps` (the C# solver object is shared by nested solve blocks).
    pub(crate) solver_eps: Cell<f64>,
    /// Records the first out of scope feature encountered (vectors, matrices, ...).
    pub out_of_scope: RefCell<Option<String>>,
    /// The data of the last `$Plot` drawn, taken by `ExpressionParser` for the Excel export.
    pub last_captured_chart: RefCell<Option<crate::plotter::capture::CapturedChart>>,
    // Output state (MathParser.Output fields)
    pub(crate) output_assignment_position: Cell<usize>,
    pub(crate) output_has_variables: Cell<bool>,
    pub(crate) output_format_equations: bool,
}

pub fn is_assignment(s: &str) -> bool {
    s == "=" || s == "\u{2190}"
}

impl MathParser {
    pub fn new(settings: Rc<RefCell<MathSettings>>) -> MathParser {
        let is_complex = settings.borrow().is_complex;
        let degrees = settings.borrow().degrees;
        let format_equations = settings.borrow().format_equations;
        let variables: VarDict = Rc::new(RefCell::new(HashMap::new()));
        let p = MathParser {
            settings,
            rpn: RefCell::new(None),
            solve_blocks: RefCell::new(Vec::new()),
            functions: RefCell::new(Container::default()),
            variables: variables.clone(),
            units: RefCell::new(HashMap::new()),
            defined_variables: RefCell::new(HashSet::new()),
            local_variables: RefCell::new(vec![variables]),
            real_calc: Calculator::new(false),
            complex_calc: Calculator::new(true),
            is_complex_calc: Cell::new(is_complex),
            evaluator_calc_complex: is_complex,
            equation_cache: RefCell::new(Vec::new()),
            result: RefCell::new(Value::zero()),
            assignment_index: Cell::new(0),
            is_calculated: Cell::new(false),
            is_solver: Cell::new(0),
            target_units: RefCell::new(None),
            is_us: Cell::new(true),
            function_definition_index: Cell::new(-1),
            backup_variable: RefCell::new((None, None)),
            has_input_fields: Cell::new(false),
            line: Cell::new(0),
            variable_substitution: Cell::new(VariableSubstitutionOptions::VariablesAndSubstitutions),
            result_units: RefCell::new(None),
            is_canceled: Cell::new(false),
            is_enabled: Cell::new(true),
            is_plotting: Cell::new(false),
            is_calculation: Cell::new(false),
            split: Cell::new(false),
            show_warnings: Cell::new(true),
            phasor: Cell::new(false),
            is_const: Cell::new(false),
            solver_eps: Cell::new(1e-14),
            out_of_scope: RefCell::new(None),
            last_captured_chart: RefCell::new(None),
            output_assignment_position: Cell::new(0),
            output_has_variables: Cell::new(false),
            output_format_equations: format_equations,
        };
        p.init_variables();
        p.calc().set_degrees(degrees);
        // Input constructor: DefineVariables
        let keys: Vec<String> = p.variables.borrow().keys().cloned().collect();
        for k in keys {
            p.defined_variables.borrow_mut().insert(k);
        }
        p
    }

    pub fn settings(&self) -> std::cell::Ref<'_, MathSettings> {
        self.settings.borrow()
    }

    /// `_calc`: the current calculator.
    pub fn calc(&self) -> &Calculator {
        if self.is_complex_calc.get() {
            &self.complex_calc
        } else {
            &self.real_calc
        }
    }

    /// The calculator the Evaluator and Compiler captured when constructed.
    pub fn evaluator_calc(&self) -> &Calculator {
        if self.evaluator_calc_complex {
            &self.complex_calc
        } else {
            &self.real_calc
        }
    }

    pub fn degrees(&self) -> i32 {
        self.calc().degrees()
    }

    pub fn set_degrees(&self, d: i32) {
        self.calc().set_degrees(d);
    }

    pub fn units(&self) -> OptUnit {
        self.result_units.borrow().clone()
    }

    pub(crate) fn set_result_units(&self, u: OptUnit) {
        *self.result_units.borrow_mut() = u;
    }

    pub fn note_out_of_scope(&self, what: &str) {
        let mut o = self.out_of_scope.borrow_mut();
        if o.is_none() {
            *o = Some(what.to_string());
        }
    }

    fn get_settings_clamped(&self, name: &str, default: f64) -> CResult<f64> {
        let v = self.get_settings_variable(name, default)?;
        Ok(if v < 1e-15 {
            1e-15
        } else if v > 1e-2 {
            1e-2
        } else {
            v
        })
    }

    pub fn precision(&self) -> CResult<f64> {
        let d = self.settings.borrow().precision;
        self.get_settings_clamped("Precision", d)
    }

    pub fn tol(&self) -> CResult<f64> {
        let d = self.settings.borrow().tol;
        self.get_settings_clamped("Tol", d)
    }

    pub fn set_complex(&self, is_complex: bool) {
        if is_complex != self.settings.borrow().is_complex {
            self.settings.borrow_mut().is_complex = is_complex;
            let degrees = self.calc().degrees();
            self.is_complex_calc.set(is_complex);
            self.calc().set_degrees(degrees);
        }
    }

    /// `Result`
    pub fn result_complex(&self) -> Complex {
        match &*self.result.borrow() {
            Value::Vector(v) => {
                if v.len() >= 1 {
                    Complex::real(v.get(0).d)
                } else {
                    Complex::real(0.0)
                }
            }
            Value::Matrix(m) => {
                if m.row_count() >= 1 && m.col_count() >= 1 {
                    Complex::real(m.get(0, 0).d)
                } else {
                    Complex::real(0.0)
                }
            }
            r => r.complex(),
        }
    }

    pub fn real(&self) -> f64 {
        self.result_complex().re()
    }

    pub fn imaginary(&self) -> f64 {
        self.result_complex().im()
    }

    pub fn result_value(&self) -> Value {
        self.result.borrow().clone()
    }

    pub fn save_answer(&self) {
        let v = Value::Complex(ComplexValue::from_c(self.result_complex(), self.units()));
        self.set_variable("ans", v.clone());
        self.set_variable("ANS", v);
    }

    pub fn clear_custom_units(&self) {
        self.units.borrow_mut().clear();
    }

    pub fn get_variable(&self, name: &str) -> CResult<Value> {
        match self.variables.borrow().get(name).and_then(|v| v.value()) {
            Some(v) => Ok(v),
            None => Err(exceptions::VariableNotExist(name)),
        }
    }

    pub fn get_variable_ref(&self, name: &str) -> VarRef {
        if let Some(v) = self.variables.borrow().get(name) {
            return v.clone();
        }
        let v = Variable::new();
        self.variables.borrow_mut().insert(name.to_string(), v.clone());
        self.defined_variables.borrow_mut().insert(name.to_string());
        v
    }

    pub fn set_variable_f(&self, name: &str, value: f64) {
        self.set_variable(name, Value::Real(RealValue::num(value)));
    }

    pub fn set_variable(&self, name: &str, value: Value) {
        let scalar = if matches!(value, Value::Real(_)) || self.settings.borrow().is_complex {
            value
        } else {
            Value::Real(RealValue::new(value.re(), value.units().clone()))
        };
        let existing = self.variables.borrow().get(name).cloned();
        match existing {
            Some(v) => v.assign(scalar),
            None => {
                self.variables
                    .borrow_mut()
                    .insert(name.to_string(), Variable::with_value(scalar));
                self.defined_variables.borrow_mut().insert(name.to_string());
            }
        }
    }

    pub fn set_unit(&self, name: &str, value: &ComplexValue) -> CResult<()> {
        let mut d = value.a;
        if d.abs() == 0.0 {
            d = 1.0;
        }
        let base = match &value.units {
            Some(u) => u.clone(),
            None => Unit::get("", self.is_us.get())?,
        };
        let u = Unit::mul_f(&base, d);
        u.set_text(name);
        let mut units = self.units.borrow_mut();
        if let std::collections::hash_map::Entry::Vacant(e) = units.entry(name.to_string()) {
            e.insert(Some(u));
            self.defined_variables.borrow_mut().insert(name.to_string());
        } else {
            units.insert(name.to_string(), Some(u));
        }
        Ok(())
    }

    pub fn get_unit(&self, name: &str) -> CResult<OptUnit> {
        match self.units.borrow().get(name) {
            Some(u) => Ok(u.clone()),
            None => Err(exceptions::UnitNotExist(name)),
        }
    }

    pub fn cancel(&self) {
        self.is_canceled.set(true);
    }

    fn init_variables(&self) {
        let pi = Value::Real(RealValue::num(std::f64::consts::PI));
        let mut vars = self.variables.borrow_mut();
        vars.insert(
            "e".to_string(),
            Variable::with_value(Value::Real(RealValue::num(std::f64::consts::E))),
        );
        vars.insert("pi".to_string(), Variable::with_value(pi.clone()));
        let pv = Variable::with_value(pi);
        pv.is_read_only.set(true);
        vars.insert("\u{03C0}".to_string(), pv);
        if self.settings.borrow().is_complex {
            vars.insert("i".to_string(), Variable::with_complex(Complex::IMAGINARY_ONE));
            vars.insert(
                "ei".to_string(),
                Variable::with_complex(Complex::IMAGINARY_ONE * std::f64::consts::E),
            );
            vars.insert(
                "\u{03C0}i".to_string(),
                Variable::with_complex(Complex::IMAGINARY_ONE * std::f64::consts::PI),
            );
        }
    }

    /// `Parse(expression, allowAssignment)`
    pub fn parse(&self, expression: &str, allow_assignment: bool) -> CResult<()> {
        *self.result.borrow_mut() = Value::zero();
        self.is_calculated.set(false);
        self.function_definition_index.set(-1);
        let input = self.get_input(expression, allow_assignment)?;
        let input = self.resolve_call_arguments(input)?;
        let is_function_definition = self.syntax_check(&input, self.is_calculation.get() && self.is_enabled.get())?;
        self.order_operators(
            &input,
            is_function_definition || self.is_solver.get() > 0 || self.is_plotting.get(),
            self.assignment_index.get(),
        )?;
        if is_function_definition {
            if self.is_solver.get() > 0 {
                return Err(exceptions::FunctionDefinitionInSolver());
            }
            *self.rpn.borrow_mut() = None;
            self.add_function(input)?;
        } else {
            *self.rpn.borrow_mut() = Some(Rc::new(input::get_rpn(&input)));
        }
        Ok(())
    }

    pub fn read_equation_from_cache(&self, cache_id: i32) -> bool {
        *self.result.borrow_mut() = Value::zero();
        self.is_calculated.set(false);
        self.function_definition_index.set(-1);
        let cache = self.equation_cache.borrow();
        if cache_id >= 0 && (cache_id as usize) < cache.len() {
            let e = &cache[cache_id as usize];
            *self.target_units.borrow_mut() = e.target_units.clone();
            *self.rpn.borrow_mut() = Some(e.rpn.clone());
            return true;
        }
        false
    }

    pub fn write_equation_to_cache(&self, is_visible: bool) -> CResult<i32> {
        let rpn = match self.rpn.borrow().clone() {
            None => return Ok(-1),
            Some(r) => r,
        };
        let mut f = None;
        if !is_visible {
            f = Some(Rc::new(self.compile_rpn(&rpn, true)?));
        }
        let eq = Equation {
            rpn,
            target_units: self.target_units.borrow().clone(),
            function: f,
        };
        let mut cache = self.equation_cache.borrow_mut();
        cache.push(eq);
        Ok(cache.len() as i32 - 1)
    }

    pub fn clear_cache(&self) {
        for f in self.functions.borrow().all() {
            f.clear_cache();
        }
    }

    pub fn break_if_canceled(&self) -> CResult<()> {
        if self.is_canceled.get() {
            return Err(exceptions::InteruptedByUser());
        }
        Ok(())
    }

    /// `Calculate(isVisible, cacheId)`
    pub fn calculate(&self, is_visible: bool, cache_id: i32) -> CResult<()> {
        if !self.is_enabled.get() {
            return Err(exceptions::CalculationsNotActive());
        }
        self.break_if_canceled()?;
        if self.function_definition_index.get() < 0 {
            let rpn = self.rpn.borrow().clone().unwrap_or_else(|| Rc::new(Vec::new()));
            let is_assignment = !rpn.is_empty()
                && rpn[0].borrow().ttype == TokenTypes::Variable
                && is_assignment(&rpn[rpn.len() - 1].borrow().content);
            let rau = self.get_settings_variable("ReturnAngleUnits", 0.0)? != 0.0;
            self.calc().set_return_angle_units(rau);
            let mut f = None;
            if !is_visible && cache_id >= 0 {
                let cache = self.equation_cache.borrow();
                if (cache_id as usize) < cache.len() {
                    f = cache[cache_id as usize].function.clone();
                }
            }
            match f {
                None => {
                    let mut r = self.evaluate(&rpn, true)?;
                    if is_visible && !is_assignment {
                        self.normalize_units(&mut r);
                    }
                    *self.result.borrow_mut() = r;
                }
                Some(f) => {
                    let r = self.eval_expr(&f)?;
                    *self.result.borrow_mut() = r;
                    if is_assignment {
                        let var = rpn[0].borrow().variable_ref();
                        if let Some(var) = var {
                            if let Some(mut val) = var.value() {
                                let tu = self.target_units.borrow().clone();
                                let tu = self.resolve_target_units(&val, &tu)?;
                                let u = evaluator::apply_units(&mut val, &tu)?;
                                // ApplyUnits works on the variable's value by reference.
                                var.set_value_silent(val);
                                self.set_result_units(u);
                            }
                        }
                    }
                }
            }
            // PurgeCache()
            for f in self.functions.borrow().all() {
                f.purge_cache();
            }
        }
        self.is_calculated.set(true);
        Ok(())
    }

    pub fn define_custom_units(&self) -> CResult<()> {
        let rpn = match self.rpn.borrow().clone() {
            Some(r) => r,
            None => return Ok(()),
        };
        if rpn.len() > 2 && rpn[0].borrow().ttype == TokenTypes::Unit && rpn[rpn.len() - 1].borrow().content == "=" {
            let s = rpn[0].borrow().content.clone();
            let u = Unit::get("", self.is_us.get())?;
            u.set_text(&s);
            self.units.borrow_mut().insert(s, Some(u));
        }
        Ok(())
    }

    pub fn calculate_real(&self) -> CResult<f64> {
        self.break_if_canceled()?;
        let rpn = self.rpn.borrow().clone().unwrap_or_else(|| Rc::new(Vec::new()));
        let r = self.evaluate(&rpn, false)?;
        match &r {
            Value::Real(real) => {
                *self.result.borrow_mut() = Value::Real(RealValue::new(real.d, self.units()));
                Ok(real.d)
            }
            Value::Complex(c) => {
                self.check_real(&r)?;
                *self.result.borrow_mut() = Value::Real(RealValue::new(c.a, self.units()));
                Ok(c.a)
            }
            _ => {
                *self.result.borrow_mut() = r.clone();
                Err(exceptions::MustBeScalar(exceptions::Items::Result))
            }
        }
    }

    pub fn check_real(&self, value: &Value) -> CResult<()> {
        if self.settings.borrow().is_complex && !value.is_real() {
            let s = crate::output::text_writer::TextWriter::new(Some(&self.decimals_settings()), self.phasor.get());
            use crate::output::OutputWriter;
            return Err(exceptions::ResultNotReal(&s.format_complex(
                value.re(),
                value.im(),
                None,
            )));
        }
        Ok(())
    }

    fn decimals_settings(&self) -> MathSettings {
        MathSettings::default().with_decimals(self.settings.borrow().decimals())
    }

    fn add_function(&self, input: Vec<TokenRef>) -> CResult<()> {
        let mut q = std::collections::VecDeque::from(input);
        let mut parameters: Vec<String> = Vec::new();
        let mut defaults: Option<Vec<Option<Rpn>>> = None;
        let t = q.pop_front();
        if let Some(t) = t {
            if t.borrow().ttype == TokenTypes::CustomFunction {
                let name = t.borrow().content.clone();
                if let Some(t2) = q.pop_front() {
                    if t2.borrow().ttype == TokenTypes::BracketLeft {
                        let mut pt = t2;
                        while let Some(t) = q.pop_front() {
                            let tt = t.borrow().ttype;
                            if tt == TokenTypes::BracketRight {
                                if pt.borrow().ttype != TokenTypes::Variable {
                                    return Err(exceptions::MissingFunctionParameter());
                                }
                                break;
                            }
                            if tt == TokenTypes::Variable {
                                parameters.push(t.borrow().content.clone());
                                let next_is_eq = q.front().is_some_and(|n| {
                                    let n = n.borrow();
                                    n.ttype == TokenTypes::Operator && n.content == "="
                                });
                                if next_is_eq {
                                    q.pop_front();
                                    let d = defaults.get_or_insert_with(Vec::new);
                                    while d.len() < parameters.len() - 1 {
                                        d.push(None);
                                    }
                                    d.push(Some(Rc::new(keyword_args::read_default(&mut q))));
                                } else if defaults.is_some() {
                                    return Err(exceptions::RequiredParameterAfterOptional(&t.borrow().content));
                                }
                            } else if tt != TokenTypes::Divisor {
                                return Err(exceptions::InvalidFunctionToken(&t.borrow().content));
                            }
                            let ptt = pt.borrow().ttype;
                            if (ptt == tt || ptt == TokenTypes::BracketLeft && tt == TokenTypes::Divisor)
                                && tt == TokenTypes::Divisor
                            {
                                return Err(exceptions::MissingFunctionParameter());
                            }
                            pt = t;
                        }
                        let eq = q.pop_front();
                        if let Some(eq) = eq {
                            if eq.borrow().content == "=" {
                                let n = parameters.len();
                                let rest: Vec<TokenRef> = q.into_iter().collect();
                                let rpn: Rpn = Rc::new(input::get_rpn(&rest));
                                let index = self.functions.borrow().index_of(&name);
                                let cf: CustomFunctionRef = if index >= 0 {
                                    let cf = self.functions.borrow().get(index).unwrap();
                                    if cf.is_read_only.get() {
                                        return Err(exceptions::CannotModifyConstant(&name));
                                    }
                                    if cf.parameter_count() != n {
                                        CustomFunction::new(self.is_const.get())
                                    } else {
                                        cf
                                    }
                                } else {
                                    CustomFunction::new(self.is_const.get())
                                };
                                cf.add_parameters(&parameters);
                                let has_defaults = defaults.is_some();
                                *cf.defaults.borrow_mut() = defaults.clone();
                                *cf.default_functions.borrow_mut() =
                                    if has_defaults { Some(vec![None; n]) } else { None };
                                *cf.rpn.borrow_mut() = rpn.clone();
                                let rec = {
                                    let funcs = self.functions.borrow();
                                    cf.check_recursion(None, &funcs)
                                };
                                match rec {
                                    Err(_) => return Err(exceptions::CircularReference(&name)),
                                    Ok(r) => cf.is_recursion.set(r),
                                }
                                if cf.is_recursion.get() {
                                    return Err(exceptions::CircularReference(&name));
                                }
                                if self.is_enabled.get() {
                                    let params = cf.parameters();
                                    self.bind_parameters(&params, &rpn);
                                    let listener: std::rc::Weak<dyn ChangeListener> =
                                        Rc::downgrade(&cf) as std::rc::Weak<dyn ChangeListener>;
                                    self.subscribe_on_change(&rpn, listener.clone());
                                    if let Some(d) = &defaults {
                                        for (i, di) in d.iter().enumerate() {
                                            if let Some(di) = di {
                                                self.bind_parameters(&params[..i], di);
                                                self.subscribe_on_change(di, listener.clone());
                                            }
                                        }
                                    }
                                }
                                *cf.units.borrow_mut() = self.target_units.borrow().clone();
                                if index >= 0 {
                                    cf.clear();
                                }
                                let idx = self.functions.borrow_mut().add(&name, cf);
                                self.function_definition_index.set(idx as i64);
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        Err(exceptions::InvalidFunctionDefinition())
    }

    pub(crate) fn bind_parameters(&self, parameters: &[Parameter], rpn: &Rpn) {
        for t in rpn.iter() {
            let (tt, content, index) = {
                let tb = t.borrow();
                (tb.ttype, tb.content.clone(), tb.index)
            };
            if tt == TokenTypes::Variable || tt == TokenTypes::Vector || tt == TokenTypes::Matrix {
                for p in parameters {
                    if content == p.name {
                        let mut tm = t.borrow_mut();
                        tm.set_variable(Some(p.variable.clone()));
                        tm.index = 1;
                        break;
                    }
                }
            } else if tt == TokenTypes::Solver {
                let sb = self.solve_blocks.borrow()[index as usize].clone();
                sb.bind_parameters(parameters, self);
            }
        }
    }

    /// `Compile(expression, parameters)` used by plots and solvers outside this phase.
    pub fn compile(&self, expression: &str, parameters: &[Parameter]) -> CResult<Option<Rc<Expr>>> {
        if !self.is_enabled.get() {
            return Ok(None);
        }
        let input = self.get_input(expression, false)?;
        let input = self.resolve_call_arguments(input)?;
        self.syntax_check(&input, self.is_enabled.get())?;
        self.order_operators(&input, true, 0)?;
        let rpn: Rpn = Rc::new(input::get_rpn(&input));
        self.bind_parameters(parameters, &rpn);
        Ok(Some(Rc::new(self.compile_rpn(&rpn, false)?)))
    }

    /// `ResultAsString`
    pub fn result_as_string(&self) -> String {
        let r = self.result.borrow().clone();
        let decimals = self.settings.borrow().decimals();
        let w = crate::output::text_writer::TextWriter::new(
            Some(&MathSettings::default().with_decimals(decimals)),
            self.phasor.get(),
        );
        let units = self.units();
        let format_result_value = |value: &Value| -> String {
            use crate::output::OutputWriter;
            let s = w.format_complex(value.re(), value.im(), None);
            match &units {
                None => s,
                Some(u) => {
                    if value.is_complex() {
                        format!("({}){}", s, u.text())
                    } else {
                        format!("{}{}", s, u.text())
                    }
                }
            }
        };
        if r.is_scalar() {
            return format_result_value(&r);
        }
        let spacing = VECTOR_SPACING.chars().next().unwrap();
        let mut sb = String::from("[");
        match &r {
            Value::Vector(vector) => {
                let len = vector.len();
                for i in 0..len {
                    sb.push_str(&format_result_value(&Value::Real(vector.get(i))));
                    if i + 1 < len {
                        sb.push(spacing);
                    }
                }
            }
            Value::Matrix(matrix) => {
                let (m, n) = (matrix.row_count(), matrix.col_count());
                for i in 0..m {
                    for j in 0..n {
                        sb.push_str(&format_result_value(&Value::Real(matrix.get(i, j))));
                        if j + 1 < n {
                            sb.push(spacing);
                        } else if i + 1 < m {
                            sb.push('|');
                        }
                    }
                }
            }
            _ => {}
        }
        sb.push(']');
        sb
    }

    /// `ResultAsVal`
    pub fn result_as_val(&self) -> String {
        if self.settings.borrow().val_mathml {
            return self.result_as_mathml();
        }
        let r = self.result.borrow().clone();
        let decimals = self.settings.borrow().decimals();
        let convert = |d: f64| netfmt::double_to_string(netfmt::round_even_digits(d, decimals));
        let format_result_value = |value: &Value| -> String {
            if let Value::Real(real) = value {
                return convert(real.d);
            }
            let c = value.complex();
            if c.is_real() {
                return convert(c.re());
            }
            if c.is_imaginary() {
                return format!("{}i", convert(c.im()));
            }
            let sa = convert(c.re());
            let sb = convert(c.im().abs());
            if c.im() < 0.0 {
                format!("{} \u{2013} {}i", sa, sb)
            } else {
                format!("{} + {}i", sa, sb)
            }
        };
        if r.is_scalar() {
            return format_result_value(&r);
        }
        let mut sb = String::from("[");
        match &r {
            Value::Vector(vector) => {
                let len = vector.len();
                for i in 0..len {
                    sb.push_str(&format_result_value(&Value::Real(vector.get(i))));
                    if i + 1 < len {
                        sb.push_str(", ");
                    }
                }
            }
            Value::Matrix(matrix) => {
                let (m, n) = (matrix.row_count(), matrix.col_count());
                for i in 0..m {
                    sb.push('[');
                    for j in 0..n {
                        sb.push_str(&format_result_value(&Value::Real(matrix.get(i, j))));
                        if j + 1 < n {
                            sb.push_str(", ");
                        }
                    }
                    sb.push(']');
                    if i + 1 < m {
                        sb.push_str(", ");
                    }
                }
            }
            _ => {}
        }
        sb.push(']');
        sb
    }

    /// The `#val` result as MathML (opt-in, see `MathSettings::val_mathml`): the number, or a bracketed
    /// table of numbers, then the unit once. Numbers are rounded as `ResultAsVal` rounds them.
    fn result_as_mathml(&self) -> String {
        let r = self.result.borrow().clone();
        let decimals = self.settings.borrow().decimals();
        let num = |d: f64| -> String {
            let t = netfmt::double_to_string(netfmt::round_even_digits(d, decimals));
            match t.strip_prefix('-') {
                Some(rest) => format!("<mo>\u{2212}</mo><mn>{rest}</mn>"),
                None => format!("<mn>{t}</mn>"),
            }
        };
        let item = |value: &Value| -> String {
            let c = value.complex();
            if c.is_real() {
                num(c.re())
            } else if c.is_imaginary() {
                format!("{}<mi>i</mi>", num(c.im()))
            } else if c.im() < 0.0 {
                format!("{}<mo>\u{2212}</mo>{}<mi>i</mi>", num(c.re()), num(-c.im()))
            } else {
                format!("{}<mo>+</mo>{}<mi>i</mi>", num(c.re()), num(c.im()))
            }
        };
        let table = |rows: Vec<Vec<String>>| -> String {
            let mut s = String::from("<mo>[</mo><mtable>");
            for row in rows {
                s.push_str("<mtr>");
                for cell in row {
                    s.push_str(&format!("<mtd>{cell}</mtd>"));
                }
                s.push_str("</mtr>");
            }
            s.push_str("</mtable><mo>]</mo>");
            s
        };
        let (body, units) = match &r {
            Value::Vector(v) => (
                table((0..v.len()).map(|i| vec![item(&Value::Real(v.get(i)))]).collect()),
                if v.len() > 0 { v.get(0).units.clone() } else { None },
            ),
            Value::Matrix(m) => (
                table(
                    (0..m.row_count())
                        .map(|i| (0..m.col_count()).map(|j| item(&Value::Real(m.get(i, j)))).collect())
                        .collect(),
                ),
                if m.row_count() > 0 && m.col_count() > 0 {
                    m.get(0, 0).units.clone()
                } else {
                    None
                },
            ),
            v if v.is_scalar() => (item(v), v.units().clone()),
            _ => return String::new(),
        };
        match units {
            Some(u) => format!("{body}<mspace width=\"0.25em\"></mspace>{}", units_to_mathml(&u.text())),
            None => body,
        }
    }

    pub fn to_text(&self) -> CResult<String> {
        self.render(OutputFormat::Text)
    }

    pub fn to_html(&self) -> CResult<String> {
        self.render(OutputFormat::Html)
    }

    /// `ToXml`: the equation as Office Math (OMML), for the Word export.
    pub fn to_xml(&self) -> CResult<String> {
        self.render(OutputFormat::Xml)
    }

    pub fn get_settings_variable(&self, name: &str, default_value: f64) -> CResult<f64> {
        let v = self.variables.borrow().get(name).cloned();
        if let Some(v) = v {
            if let Some(val) = v.value() {
                if val.is_scalar() {
                    return Ok(val.re());
                }
                return Err(exceptions::MustBeScalar(Items::Variable));
            }
        }
        Ok(default_value)
    }

    pub(crate) fn subscribe_on_change(&self, rpn: &Rpn, listener: std::rc::Weak<dyn ChangeListener>) {
        let len = rpn.len();
        if len == 0 {
            return;
        }
        let mut i0 = 0;
        if is_assignment(&rpn[len - 1].borrow().content) {
            let t = rpn[0].borrow().ttype;
            if t == TokenTypes::Variable || t == TokenTypes::Vector || t == TokenTypes::Matrix {
                i0 = 1;
            }
        }
        for t in rpn.iter().skip(i0) {
            let tb = t.borrow();
            if let TokenKind::Variable(Some(v)) = &tb.kind {
                v.on_change.subscribe(listener.clone());
            } else if tb.ttype == TokenTypes::Solver {
                let sb = self.solve_blocks.borrow()[tb.index as usize].clone();
                sb.on_change.subscribe(listener.clone());
            } else if tb.ttype == TokenTypes::CustomFunction && tb.index != -1 {
                if let Some(cf) = self.functions.borrow().get(tb.index) {
                    cf.on_change.subscribe(listener.clone());
                }
            }
        }
    }

    pub fn reset_stack(&self) {}

    /// Variables and the vector separator are exposed for the ExpressionParser.
    pub fn vector_spacing() -> &'static str {
        VECTOR_SPACING
    }

    pub fn as_value_item(v: &Value, _item: Items) -> CResult<Value> {
        Ok(v.clone())
    }

    pub fn new_token(content: &str, tt: TokenTypes) -> TokenRef {
        Token::new(content, tt).into_ref()
    }
}

impl Variable {
    /// Writes a value without raising OnChange or changing the initialized flag
    /// (C# `ref var val = ref vt.Variable.ValueByRef()` followed by an in place update).
    pub fn set_value_silent(&self, v: Value) {
        let init = self.is_initialized();
        self.set_value(v);
        if !init {
            self.reset_initialized();
        }
    }
}

/// A unit's text (`kip·ft`, `m/s^2`, `m²`) as upright MathML identifiers with the powers as superscripts.
fn units_to_mathml(text: &str) -> String {
    // A quotient of units is a stacked fraction, as a result is written on paper.
    let mut depth = 0i32;
    let mut slash = None;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '/' | '\u{2215}' if depth == 0 => {
                slash = Some((i, c.len_utf8()));
                break;
            }
            _ => {}
        }
    }
    if let Some((i, w)) = slash {
        let strip = |t: &str| -> String {
            let t = t.trim();
            match t.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
                Some(inner) if !inner.contains('(') => inner.to_string(),
                _ => t.to_string(),
            }
        };
        let num = strip(&text[..i]);
        let den = strip(&text[i + w..]);
        let num_ml = if num.is_empty() {
            "<mn>1</mn>".to_string()
        } else {
            units_inline(&num)
        };
        return format!(
            "<mfrac><mrow>{num_ml}</mrow><mrow>{}</mrow></mfrac>",
            units_inline(&den)
        );
    }
    units_inline(text)
}

/// A power written as a decimal that is a simple fraction (`0.75`) is shown as one (`3/4`).
fn exponent_ml(p: &str) -> String {
    let esc = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;");
    if let Some((n, d)) = p.split_once('/') {
        if n.trim_start_matches('-').parse::<u32>().is_ok() && d.parse::<u32>().is_ok() {
            let sign = if n.starts_with('-') { "<mo>\u{2212}</mo>" } else { "" };
            return format!(
                "{sign}<mfrac><mn>{}</mn><mn>{}</mn></mfrac>",
                n.trim_start_matches('-'),
                d
            );
        }
    }
    if let Ok(x) = p.parse::<f64>() {
        let (neg, a) = (x < 0.0, x.abs());
        for d in [2.0, 3.0, 4.0, 8.0] {
            let n = (a * d).round();
            if (a * d - n).abs() < 1e-9 && n.rem_euclid(d) != 0.0 {
                let sign = if neg { "<mo>\u{2212}</mo>" } else { "" };
                return format!("{sign}<mfrac><mn>{}</mn><mn>{}</mn></mfrac>", n as i64, d as i64);
            }
        }
    }
    format!("<mn>{}</mn>", esc(p))
}

fn units_inline(text: &str) -> String {
    fn flush(name: &mut String, out: &mut String) {
        if !name.is_empty() {
            out.push_str(&format!(
                "<mi mathvariant=\"normal\" class=\"ru\">{}</mi>",
                escape_ml(name)
            ));
            name.clear();
        }
    }
    fn escape_ml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\'', "&#8242;")
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut name = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let sup = match c {
            '\u{00B2}' => Some("2".to_string()),
            '\u{00B3}' => Some("3".to_string()),
            '\u{2074}'..='\u{2079}' => Some(((c as u32 - 0x2074) as u8 + b'4').to_string()),
            _ => None,
        };
        if c == '^' {
            let mut j = i + 1;
            let mut p = String::new();
            // `^2`, `^-1`, `^0.75` or `^(3/4)`.
            let bracketed = chars.get(j) == Some(&'(');
            if bracketed {
                j += 1;
            }
            while j < chars.len()
                && (chars[j].is_ascii_digit() || chars[j] == '-' || chars[j] == '.' || (bracketed && chars[j] == '/'))
            {
                p.push(chars[j]);
                j += 1;
            }
            if bracketed {
                if chars.get(j) == Some(&')') {
                    j += 1;
                } else {
                    p.clear();
                }
            }
            if !p.is_empty() && !name.is_empty() {
                let base = std::mem::take(&mut name);
                out.push_str(&format!(
                    "<msup><mi mathvariant=\"normal\" class=\"ru\">{}</mi>{}</msup>",
                    escape_ml(&base),
                    exponent_ml(&p)
                ));
                i = j;
                continue;
            }
            name.push(c);
        } else if let Some(p) = sup.filter(|_| !name.is_empty()) {
            let base = std::mem::take(&mut name);
            out.push_str(&format!(
                "<msup><mi mathvariant=\"normal\" class=\"ru\">{}</mi><mn>{}</mn></msup>",
                escape_ml(&base),
                p
            ));
        } else if c == '\u{00B7}' || c == '\u{22C5}' || c == '*' {
            flush(&mut name, &mut out);
            out.push_str("<mo>\u{22C5}</mo>");
        } else if c == '/' || c == '\u{2215}' || c == '(' || c == ')' {
            flush(&mut name, &mut out);
            out.push_str(&format!("<mo>{}</mo>", if c == '\u{2215}' { '/' } else { c }));
        } else if c.is_whitespace() {
            flush(&mut name, &mut out);
        } else {
            name.push(c);
        }
        i += 1;
    }
    flush(&mut name, &mut out);
    out
}
