//! Port of Parsers/MathParser/MathParser.Output.cs (equation rendering).
use std::rc::Rc;

use super::is_assignment;
use super::tokens::{Rpn, Token, TokenKind, TokenTypes, ValueTypes, DEFAULT_ORDER};
use super::{MathParser, VariableSubstitutionOptions};
use crate::base_types::real_value::RealValue;
use crate::base_types::value::Value;
use crate::calculator::{operator_index, vector_calculator, NEG_CHAR, NEG_STR, OPERATOR_ORDER, SWITCH_INDEX};
use crate::exceptions;
use crate::math_parser_exception::CResult;
use crate::messages;
use crate::netfmt;
use crate::output::html_writer::HtmlWriter;
use crate::output::text_writer::TextWriter;
use crate::output::xml_writer::XmlWriter;
use crate::output::{OutputFormat, OutputWriter};
use crate::validator;

const THIN_SPACE: char = '\u{2009}';

#[derive(Clone)]
struct RenderToken {
    content: String,
    index: i64,
    ttype: TokenTypes,
    order: i8,
    val_type: ValueTypes,
    level: i32,
    offset: i32,
    min_offset: i32,
    max_offset: i32,
    parameter_count: i32,
    argument_map: Option<Rc<Vec<usize>>>,
    positional_count: i32,
    is_composite_value: bool,
    /// For an element of a vector or matrix written as a number with a unit (and for a row of such
    /// elements): the same content without the unit, and the unit. Lets the vector or matrix write
    /// a unit its elements share once, after the closing bracket (see `common_unit_content`).
    unit_once: Option<(String, String)>,
}

impl RenderToken {
    fn simple(content: &str, ttype: TokenTypes, level: i32) -> RenderToken {
        RenderToken {
            content: content.to_string(),
            index: -1,
            ttype,
            order: DEFAULT_ORDER,
            val_type: ValueTypes::None,
            level,
            offset: 0,
            min_offset: 0,
            max_offset: 0,
            parameter_count: 0,
            argument_map: None,
            positional_count: 0,
            is_composite_value: false,
            unit_once: None,
        }
    }

    fn from_token(t: &Token, level: i32) -> RenderToken {
        let mut r = RenderToken::simple(&t.content, t.ttype, level);
        r.order = t.order;
        r.index = t.index;
        r.parameter_count = -1;
        match &t.kind {
            TokenKind::Function {
                parameter_count,
                argument_map,
                positional_count,
            } => {
                r.parameter_count = *parameter_count;
                r.argument_map = argument_map.clone();
                r.positional_count = *positional_count;
            }
            TokenKind::Vector(_) => {
                r.parameter_count = t.index as i32 - 1;
                r.index = -1;
            }
            TokenKind::Matrix(_) => {
                r.parameter_count = (t.index / crate::vector::MAX_LENGTH) as i32 - 1;
                r.index = -1;
            }
            _ => {}
        }
        if matches!(
            t.ttype,
            TokenTypes::Unit | TokenTypes::Constant | TokenTypes::Variable | TokenTypes::Input
        ) {
            let mut v = Value::zero();
            match &t.kind {
                TokenKind::Value(val) => v = val.clone(),
                TokenKind::Variable(Some(var)) => match var.value() {
                    Some(Value::Vector(_)) => {
                        r.val_type = ValueTypes::Vector;
                        return r;
                    }
                    Some(Value::Matrix(_)) => {
                        r.val_type = ValueTypes::Matrix;
                        return r;
                    }
                    Some(val) => v = val,
                    None => {}
                },
                _ => {}
            }
            r.val_type = if v.is_unit() {
                ValueTypes::Unit
            } else if v.units().is_none() && t.ttype != TokenTypes::Variable {
                ValueTypes::Number
            } else {
                ValueTypes::NumberWithUnit
            };
        }
        r
    }
}

fn plus_order() -> i8 {
    OPERATOR_ORDER[operator_index('+').unwrap()]
}

fn minus_order() -> i8 {
    OPERATOR_ORDER[operator_index('-').unwrap()]
}

fn is_negative(t: &RenderToken) -> bool {
    let s: Vec<char> = t.content.chars().collect();
    let n = s.len();
    if t.order != DEFAULT_ORDER || n == 0 {
        return false;
    }
    let mut is_tag = false;
    for i in 0..n {
        let c = s[i];
        if c == '<' {
            is_tag = true;
        } else if c == '>' {
            is_tag = false;
        } else if is_tag {
            if c == 'r' && i > 8 {
                let prev: String = s[i - 7..i].iter().collect();
                if prev.eq_ignore_ascii_case("class=\"") {
                    break;
                }
            }
        } else if validator::is_digit(c) || c == '(' || c == '|' || c == '\u{221A}' {
            break;
        } else if c == '-' {
            return true;
        }
    }
    false
}

/// `double.TryParse(s, out d)` followed by `d.ToString()` (current culture, en-US).
fn fix_power_formatting(s: &str) -> String {
    let t = s.trim();
    let cleaned: String = t.chars().filter(|&c| c != ',').collect();
    let ok = !cleaned.is_empty()
        && cleaned
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'))
        && cleaned.chars().any(|c| c.is_ascii_digit());
    if ok {
        if let Ok(d) = cleaned.parse::<f64>() {
            return netfmt::double_to_string(d);
        }
    }
    s.to_string()
}

/// `FixPowerFormatting` for the XML writer: the number inside the first run is reformatted.
/// Upstream keeps the number's first character in front of the reformatted number
/// (`span[..(i1 + 1)]`), so "2" became "22"; the number replaces the whole text here.
fn fix_power_formatting_xml(s: &str) -> String {
    if let Some(i2) = s.find("</") {
        if i2 > 0 {
            let start = &s[..i2];
            let i1 = start.rfind('>').map(|i| i + 1).unwrap_or(0);
            if i1 > 0 {
                return format!("{}{}{}", &s[..i1], fix_power_formatting(&start[i1..]), &s[i2..]);
            }
        }
    }
    s.to_string()
}

fn fix_offset(s: &str, offset: i32) -> String {
    if offset < 0 {
        format!("<span class=\"dvc up\">{}</span>", s)
    } else if offset > 0 {
        format!("<span class=\"dvc down\">{}</span>", s)
    } else {
        s.to_string()
    }
}

struct Renderer<'a> {
    p: &'a MathParser,
    writer: &'a dyn OutputWriter,
    text_writer: TextWriter,
    rpn: Rpn,
    substitute: bool,
    stack: Vec<RenderToken>,
    div: String,
    has_operators: bool,
    format_equations: bool,
}

impl MathParser {
    /// `Output.Render(format)`
    pub(crate) fn render(&self, format: OutputFormat) -> CResult<String> {
        let settings = self.settings.borrow().clone();
        let phasor = self.phasor.get();
        let writer: Box<dyn OutputWriter> = match format {
            OutputFormat::Html => Box::new(HtmlWriter::new(Some(&settings), phasor)),
            OutputFormat::Xml => Box::new(XmlWriter::new(Some(&settings), phasor)),
            _ => Box::new(TextWriter::new(Some(&settings), phasor)),
        };
        let writer = writer.as_ref();
        self.output_assignment_position.set(0);
        self.output_has_variables.set(false);
        let mut sb = String::new();
        let delimiter = writer.format_operator(';');
        let assignment = writer.format_operator('=');
        let global_assignment = writer.format_operator('\u{2190}');
        let fdi = self.function_definition_index.get();
        if fdi >= 0 {
            let cf = self.functions.borrow().get(fdi).unwrap();
            for i in 0..cf.parameter_count() {
                sb.push_str(&writer.format_variable(&cf.parameter_name(i), "", false));
                let default_rpn = cf.defaults.borrow().as_ref().and_then(|d| d.get(i).cloned().flatten());
                if let Some(d) = default_rpn {
                    let (s, _) = self.render_rpn(&d, false, writer)?;
                    sb.push_str(&assignment);
                    sb.push_str(&s);
                }
                if i + 1 < cf.parameter_count() {
                    sb.push_str(&delimiter);
                }
            }
            let s = writer.add_brackets(&sb, 0, '(', ')');
            sb.clear();
            let last_name = self.functions.borrow().last_name.clone();
            sb.push_str(&writer.format_variable(&last_name, "", false));
            sb.push_str(&s);
            sb.push_str(&assignment);
            let rpn = cf.rpn.borrow().clone();
            let (body, _) = self.render_rpn(&rpn, false, writer)?;
            sb.push_str(&body);
            if self.show_warnings.get() && fdi < self.functions.borrow().count() as i64 - 1 {
                let warning = messages::Function_redefined;
                if format == OutputFormat::Text {
                    sb.push_str(warning);
                } else if format == OutputFormat::Html {
                    sb.push_str(&format!("<b class=\"err\" title = \"{}\"> !!!</b>", warning));
                }
            }
        } else {
            let rpn = self.rpn.borrow().clone().unwrap_or_else(|| Rc::new(Vec::new()));
            let (equation, mut has_operators) = self.render_rpn(&rpn, false, writer)?;
            if self.variable_substitution.get() != VariableSubstitutionOptions::SubstitutionsOnly {
                sb.push_str(&equation);
            }
            if self.is_calculated.get() {
                let mut subst = String::new();
                let mut is_split = false;
                let len = rpn.len();
                let solver_assign =
                    len == 3 && rpn[1].borrow().ttype == TokenTypes::Solver && is_assignment(&rpn[2].borrow().content);
                if !(solver_assign || len == 1) {
                    if self.output_has_variables.get() {
                        if settings.substitute
                            && self.variable_substitution.get() != VariableSubstitutionOptions::VariablesOnly
                        {
                            let (s2, ho) = self.render_rpn(&rpn, true, writer)?;
                            subst = s2;
                            has_operators = ho;
                            let mut eq_len: isize = match equation.rfind(&assignment) {
                                Some(i) => i as isize,
                                None => match equation.rfind(&global_assignment) {
                                    Some(i) => i as isize,
                                    None => -1,
                                },
                            };
                            eq_len += assignment.len() as isize;
                            let tail = if eq_len >= 0
                                && (eq_len as usize) <= equation.len()
                                && equation.is_char_boundary(eq_len as usize)
                            {
                                &equation[eq_len as usize..]
                            } else {
                                ""
                            };
                            if tail != subst {
                                if self.variable_substitution.get() != VariableSubstitutionOptions::SubstitutionsOnly {
                                    sb.push_str(&assignment);
                                    if self.split.get() && format == OutputFormat::Html {
                                        is_split = true;
                                        sb.push_str("<br/><span class=\"indent\">");
                                    }
                                }
                                sb.push_str(&subst);
                            }
                        }
                    } else {
                        let ap = self.output_assignment_position.get();
                        if ap > 0
                            && ap < sb.len()
                            && matches!(&*self.result.borrow(), Value::Real(_) | Value::Complex(_))
                            && sb.is_char_boundary(ap)
                        {
                            subst = sb[ap..].to_string();
                        }
                    }
                }
                let res = match &*self.result.borrow() {
                    Value::Vector(v) => writer.format_vector(v),
                    Value::Matrix(m) => writer.format_matrix(m),
                    r => writer.format_value(r),
                };
                if has_operators && res != subst || subst.is_empty() {
                    if !sb.is_empty() {
                        sb.push_str(&assignment);
                    }
                    sb.push_str(&res);
                }
                if is_split {
                    sb.push_str("</span>");
                }
            }
        }
        Ok(sb)
    }

    fn render_rpn(&self, rpn: &Rpn, substitute: bool, writer: &dyn OutputWriter) -> CResult<(String, bool)> {
        let settings = self.settings.borrow().clone();
        let mut r = Renderer {
            p: self,
            writer,
            text_writer: TextWriter::new(Some(&settings), self.phasor.get()),
            rpn: rpn.clone(),
            substitute,
            stack: Vec::new(),
            div: writer.format_operator(';'),
            has_operators: self.target_units.borrow().is_some(),
            format_equations: self.output_format_equations,
        };
        r.run()?;
        let s = r.stack.pop().map(|t| t.content).unwrap_or_default();
        Ok((s, r.has_operators))
    }
}

impl<'a> Renderer<'a> {
    fn pop(&mut self) -> CResult<RenderToken> {
        self.stack
            .pop()
            .ok_or_else(|| crate::math_parser_exception::MathParserException::unexpected("Stack empty."))
    }

    fn run(&mut self) -> CResult<()> {
        let rpn = self.rpn.clone();
        for (i, tr) in rpn.iter().enumerate() {
            let mut t = RenderToken::from_token(&tr.borrow(), 0);
            let tt = t.ttype;
            let st = t.content.clone();
            if tt == TokenTypes::Solver {
                self.render_solver_token(&mut t)?;
            } else if tt == TokenTypes::Input {
                self.render_input_token(&mut t, i);
            } else if tt == TokenTypes::Constant || tt == TokenTypes::Unit {
                self.render_constant_token(&mut t, i)?;
            } else if tt == TokenTypes::Variable
                || (tt == TokenTypes::Vector || tt == TokenTypes::Matrix) && t.parameter_count < 0
            {
                self.render_variable_token(&mut t, i);
            } else {
                let b = self.pop()?;
                if st == NEG_STR {
                    self.render_negation_token(&mut t, b);
                } else if tt == TokenTypes::Operator {
                    self.render_operator_token(&mut t, b)?;
                } else {
                    if tt == TokenTypes::VectorIndex {
                        self.render_vector_index_token(&mut t, b)?;
                    } else if tt == TokenTypes::MatrixIndex {
                        let a = self.pop()?;
                        self.render_matrix_index_token(&mut t, a, b)?;
                    } else if matches!(
                        tt,
                        TokenTypes::Function2
                            | TokenTypes::VectorFunction2
                            | TokenTypes::MatrixFunction2
                            | TokenTypes::MatrixIterativeFunction
                    ) {
                        self.render_function2_token(&mut t, b)?;
                    } else if matches!(
                        tt,
                        TokenTypes::Function3 | TokenTypes::VectorFunction3 | TokenTypes::MatrixFunction3
                    ) {
                        self.render_function3_token(&mut t, b)?;
                    } else if tt == TokenTypes::MatrixFunction4 {
                        self.render_function_n_token(&mut t, b, 4)?;
                        t.val_type = ValueTypes::Vector;
                    } else if tt == TokenTypes::MatrixFunction5 {
                        self.render_function_n_token(&mut t, b, 5)?;
                        t.val_type = ValueTypes::Matrix;
                    } else if matches!(
                        tt,
                        TokenTypes::Interpolation
                            | TokenTypes::MultiFunction
                            | TokenTypes::VectorMultiFunction
                            | TokenTypes::MatrixMultiFunction
                    ) {
                        self.render_multi_function_token(&mut t, b)?;
                    } else if tt == TokenTypes::CustomFunction {
                        self.render_custom_function_token(&mut t, b)?;
                    } else if tt == TokenTypes::Vector {
                        let pc = t.parameter_count;
                        let unit = self.common_unit_content(&b, pc);
                        let s = self.render_parameters(&mut t, b, pc, false, unit.is_some())?;
                        t.content = self.add_brackets(&s, t.level, t.min_offset, t.max_offset, '[', ']');
                        self.append_common_unit(&mut t, unit);
                        t.min_offset = 0;
                        t.max_offset = 0;
                    } else if tt == TokenTypes::RowDivisor {
                        let pc = t.parameter_count;
                        let unit = self.common_unit_content(&b, pc);
                        let stripped = unit.as_ref().map(|_| self.stripped_row(&b, pc));
                        let full = self.render_parameters(&mut t, b, pc, false, false)?;
                        t.unit_once = stripped.zip(unit);
                        t.content = full;
                    } else if tt == TokenTypes::Matrix {
                        let pc = t.parameter_count;
                        let unit = self.common_unit_content(&b, pc);
                        let s = self.render_parameters(&mut t, b, pc, true, unit.is_some())?;
                        t.content = self.add_brackets(&s, t.level, t.min_offset, t.max_offset, '[', ']');
                        self.append_common_unit(&mut t, unit);
                        t.min_offset = 0;
                        t.max_offset = 0;
                    } else if st == "!" {
                        let mut sb = b.content.clone();
                        if is_negative(&b) || b.order > DEFAULT_ORDER {
                            sb = self.add_brackets(&sb, b.level, b.min_offset, b.max_offset, '(', ')');
                        }
                        t.content = format!("{}{}", sb, self.writer.format_operator('!'));
                        t.level = b.level;
                    } else if st.eq_ignore_ascii_case("sqr")
                        || st.eq_ignore_ascii_case("sqrt")
                        || st.eq_ignore_ascii_case("cbrt")
                    {
                        let n = if st.eq_ignore_ascii_case("cbrt") { "3" } else { "2" };
                        self.render_root_token(&mut t, b, n);
                    } else if st.eq_ignore_ascii_case("abs") {
                        t.content = self.writer.format_abs(&b.content, b.level);
                        t.level = b.level;
                    } else {
                        self.render_function_token(&mut t, b);
                    }
                    self.has_operators = true;
                }
            }
            self.stack.push(t);
        }
        Ok(())
    }

    fn render_solver_token(&mut self, t: &mut RenderToken) -> CResult<()> {
        t.content = self.render_solver(t.index as usize)?;
        let sb = self.p.solve_blocks.borrow()[t.index as usize].clone();
        if sb.is_figure() && !self.substitute {
            t.ttype = TokenTypes::Solver;
            t.order = minus_order();
            t.level = 2;
        } else {
            t.ttype = TokenTypes::Constant;
        }
        Ok(())
    }

    fn render_solver(&mut self, index: usize) -> CResult<String> {
        let sb = self.p.solve_blocks.borrow()[index].clone();
        if self.substitute {
            return Ok(match sb.result() {
                Value::Vector(v) => self.writer.format_vector(&v),
                Value::Matrix(m) => self.writer.format_matrix(&m),
                r => self.writer.format_value(&r),
            });
        }
        self.p.output_has_variables.set(true);
        if self.writer.format_type() == OutputFormat::Html {
            return Ok(sb.to_html(self.format_equations, self.p));
        }
        if self.writer.format_type() == OutputFormat::Xml {
            return Ok(sb.to_xml(self.p));
        }
        Ok(sb.to_text())
    }

    fn render_input_token(&mut self, t: &mut RenderToken, i: usize) {
        let units = self.rpn[i].borrow().token_value().and_then(|v| v.units().clone());
        t.content = self
            .writer
            .format_input(&t.content, &units, self.p.line.get(), self.p.is_calculated.get());
    }

    fn render_constant_token(&mut self, t: &mut RenderToken, i: usize) -> CResult<()> {
        let tb = self.rpn[i].borrow();
        let mut value = Value::zero();
        let mut has_value = true;
        match &tb.kind {
            TokenKind::Value(v) => value = v.clone(),
            TokenKind::Variable(var) => match var.as_ref().and_then(|v| v.value()) {
                Some(v) => value = v,
                None => has_value = false,
            },
            _ => has_value = false,
        }
        drop(tb);
        if has_value {
            if t.ttype == TokenTypes::Unit {
                match value.units() {
                    None => {
                        if self.p.is_calculated.get() {
                            return Err(exceptions::InvalidUnits(&t.content));
                        }
                    }
                    Some(u) => t.content = self.writer.unit_string(u),
                }
            } else {
                t.content = self.writer.format_value(&value);
            }
            t.is_composite_value = value.is_composite();
            if t.ttype == TokenTypes::Constant {
                t.unit_once = self.number_and_unit(&value);
            }
        }
        Ok(())
    }

    /// A real number with a unit as its content without the unit and the unit, for `unit_once`.
    fn number_and_unit(&self, value: &Value) -> Option<(String, String)> {
        if self.p.settings.borrow().unit_per_element {
            return None;
        }
        let (Value::Real(r), Some(u)) = (value, value.units()) else {
            return None;
        };
        if r.is_unit || u.is_angle() || u.format_string().is_some() {
            return None;
        }
        let number = self.writer.format_value(&Value::Real(RealValue::new(r.d, None)));
        Some((number, self.writer.unit_string(u)))
    }

    fn render_variable_token(&mut self, t: &mut RenderToken, i: usize) {
        let (content, var) = {
            let tb = self.rpn[i].borrow();
            (tb.content.clone(), tb.variable_ref())
        };
        let backup = self.p.backup_variable.borrow().clone();
        let ival = if i > 0 && backup.0.as_deref() == Some(content.as_str()) {
            backup.1.clone()
        } else {
            var.and_then(|v| v.value())
        };
        match ival {
            Some(Value::Vector(vector)) => {
                let s = if !self.p.settings.borrow().substitute
                    && self.p.function_definition_index.get() < 0
                    && self.p.is_calculated.get()
                {
                    self.text_writer.format_vector(&vector)
                } else {
                    String::new()
                };
                t.content = self.writer.format_variable(&format!("\u{20D7}{}", t.content), &s, true);
            }
            Some(Value::Matrix(matrix)) => {
                let s = if !self.p.settings.borrow().substitute
                    && self.p.function_definition_index.get() < 0
                    && self.p.is_calculated.get()
                {
                    self.text_writer.format_matrix(&matrix)
                } else {
                    String::new()
                };
                t.content = self.writer.format_variable(&t.content, &s, true);
            }
            Some(scalar) => {
                if self.substitute {
                    t.content = self.writer.format_value(&scalar);
                    t.is_composite_value = scalar.is_composite() || t.content.contains('\u{00D7}');
                    t.unit_once = self.number_and_unit(&scalar);
                    t.order = DEFAULT_ORDER;
                    if self.p.settings.borrow().is_complex && scalar.is_complex() && scalar.units().is_none() {
                        t.order = if scalar.im() < 0.0 { minus_order() } else { plus_order() };
                    }
                } else {
                    let s = if !self.p.settings.borrow().substitute
                        && self.p.function_definition_index.get() < 0
                        && self.p.is_calculated.get()
                    {
                        self.text_writer.format_value(&scalar)
                    } else {
                        String::new()
                    };
                    t.content = self.writer.format_variable(&t.content, &s, false);
                    let last_is_assignment = is_assignment(&self.rpn[self.rpn.len() - 1].borrow().content);
                    self.p.output_has_variables.set(i > 0 || !last_is_assignment);
                }
            }
            None => {
                t.content = self.writer.format_variable(&t.content, "", false);
            }
        }
    }

    /// `RenderVectorIndexToken(t, b)`
    fn render_vector_index_token(&mut self, t: &mut RenderToken, b: RenderToken) -> CResult<()> {
        let a = self.pop()?;
        if self.substitute {
            let variable_name = format!("{}.{}", t.content, t.index);
            let backup = self.p.backup_variable.borrow().clone();
            let value = if backup.0.as_deref() == Some(variable_name.as_str()) {
                backup.1.clone().unwrap_or_else(Value::zero)
            } else {
                let var = self.p.variables.borrow().get(&t.content).cloned();
                match var.and_then(|v| v.value()) {
                    Some(Value::Vector(vector)) => Value::Real(vector.try_get((t.index - 1).max(0) as usize)?),
                    other => return Err(super::evaluator::invalid_cast(&other, "Calcpad.Core.Vector")),
                }
            };
            t.content = self.writer.format_value(&value);
        } else {
            let s = if self.p.is_calculated.get() && self.p.function_definition_index.get() < 0 {
                t.index.to_string()
            } else {
                b.content.clone()
            };
            t.content = if t.content.contains('_') {
                self.writer.append_subscript(&a.content, &s)
            } else {
                self.writer.format_subscript(&a.content, &s)
            };
        }
        Ok(())
    }

    /// `RenderMatrixIndexToken(t, b, c)`
    fn render_matrix_index_token(&mut self, t: &mut RenderToken, b: RenderToken, c: RenderToken) -> CResult<()> {
        let a = match self.stack.pop() {
            Some(a) => a,
            None => return Err(exceptions::MissingMatrixIndex(&t.content)),
        };
        let i = t.index / crate::vector::MAX_LENGTH;
        let j = t.index % crate::vector::MAX_LENGTH;
        if self.substitute {
            let variable_name = format!("{}.{}.{}", t.content, i, j);
            let backup = self.p.backup_variable.borrow().clone();
            let value = if backup.0.as_deref() == Some(variable_name.as_str()) {
                backup.1.clone().unwrap_or_else(Value::zero)
            } else {
                let var = self.p.variables.borrow().get(&t.content).cloned();
                match var.and_then(|v| v.value()) {
                    Some(Value::Matrix(matrix)) => {
                        Value::Real(matrix.try_get((i - 1).max(0) as usize, (j - 1).max(0) as usize)?)
                    }
                    other => return Err(super::evaluator::invalid_cast(&other, "Calcpad.Core.Matrix")),
                }
            };
            t.content = self.writer.format_value(&value);
        } else {
            let s = if self.p.is_calculated.get() && self.p.function_definition_index.get() < 0 {
                format!("{},{}", i, j)
            } else {
                format!("{}{}{}", b.content, self.writer.format_operator(','), c.content)
            };
            t.content = if t.content.contains('_') {
                self.writer.append_subscript(&a.content, &s)
            } else {
                self.writer.format_subscript(&a.content, &s)
            };
        }
        Ok(())
    }

    fn render_negation_token(&mut self, t: &mut RenderToken, b: RenderToken) {
        let mut sb = b.content.clone();
        let neg = is_negative(&b);
        if neg || b.order > DEFAULT_ORDER && b.ttype != TokenTypes::Solver {
            if b.index == 1 && b.level > 0 {
                sb = format!("{}{}", THIN_SPACE, sb);
            } else {
                sb = self.add_brackets(&sb, b.level, b.min_offset, b.max_offset, '(', ')');
            }
            self.has_operators = true;
        } else if b.ttype == TokenTypes::Variable {
            self.has_operators = true;
        }
        t.content = format!("{}{}", self.writer.format_operator(NEG_CHAR), sb);
        t.level = b.level;
        if let Some((n, u)) = &b.unit_once {
            if sb == b.content {
                t.unit_once = Some((format!("{}{}", self.writer.format_operator(NEG_CHAR), n), u.clone()));
            }
        }
        if b.ttype == TokenTypes::Constant {
            t.ttype = b.ttype;
            t.order = b.order;
            t.val_type = b.val_type;
        }
        if b.val_type == ValueTypes::Vector {
            t.val_type = ValueTypes::Vector;
        }
    }

    fn render_operator_token(&mut self, t: &mut RenderToken, b: RenderToken) -> CResult<()> {
        let mut sb = b.content.clone();
        if self.substitute && is_assignment(&t.content) {
            t.content = sb;
            t.level = b.level;
            return Ok(());
        }
        let content = t.content.clone();
        let is_text = self.writer.format_type() == OutputFormat::Text;
        let format_equation =
            !is_text && (self.format_equations && content == "/" || !self.format_equations && content == "\u{00F7}");
        let a = match self.stack.pop() {
            Some(a) => a,
            None => RenderToken::simple(
                "",
                if content == "-" {
                    TokenTypes::Constant
                } else {
                    TokenTypes::None
                },
                0,
            ),
        };
        let mut sa = a.content.clone();
        if a.order > t.order && !format_equation {
            sa = self.add_brackets(&sa, a.level, a.min_offset, a.max_offset, '(', ')');
        } else if a.ttype == TokenTypes::MultiFunction && a.index == SWITCH_INDEX && a.parameter_count > 2 {
            sa = self.writer.close_curly_brackets(&sa, a.level);
        }
        if content == "^" {
            if a.is_composite_value || is_negative(&a) {
                sa = self.add_brackets(&sa, a.level, a.min_offset, a.max_offset, '(', ')');
            }
            if is_text && (is_negative(&b) || b.order != DEFAULT_ORDER) {
                sb = self.add_brackets(&sb, b.level, b.min_offset, b.max_offset, '(', ')');
            }
            if a.ttype == TokenTypes::Vector
                || a.ttype == TokenTypes::Matrix
                || a.val_type == ValueTypes::Vector
                || a.val_type == ValueTypes::Matrix
            {
                sb = format!("{}{}{}", self.writer.format_operator('\u{2299}'), THIN_SPACE, sb);
            }
            if a.val_type == ValueTypes::Unit {
                if self.p.settings.borrow().format_string.is_some() {
                    sb = if self.writer.format_type() == OutputFormat::Xml {
                        fix_power_formatting_xml(&sb)
                    } else {
                        fix_power_formatting(&sb)
                    };
                }
                t.content = self.writer.format_power(&sa, &sb, -1, -1);
            } else {
                t.content = self.writer.format_power(&sa, &sb, a.level, a.order as i32);
            }
            t.level = a.level;
            t.val_type = a.val_type;
            if a.val_type != ValueTypes::Unit {
                self.has_operators = true;
            }
        } else {
            if !format_equation && b.ttype != TokenTypes::Solver {
                let is_division = content == "/" || content == "\u{00F7}";
                if b.order > t.order
                    || b.order == t.order && (content == "-" || is_division)
                    || b.order == 2 && sb.contains(['\u{00B7}', '\u{2215}']) && is_division
                    || is_negative(&b) && !is_assignment(&content)
                {
                    sb = self.add_brackets(&sb, b.level, b.min_offset, b.max_offset, '(', ')');
                }
            }
            let mut level = 0;
            if format_equation {
                level = a.level + b.level + 1;
                if level >= 4 {
                    if a.order > t.order {
                        sa = self.add_brackets(&sa, a.level, a.min_offset, a.max_offset, '(', ')');
                    }
                    if b.order > t.order {
                        sb = self.add_brackets(&sb, b.level, b.min_offset, b.max_offset, '(', ')');
                    }
                }
                t.content = self.writer.format_division(&sa, &sb, level);
                if level < 4 {
                    if level == 2 && self.writer.format_type() == OutputFormat::Html {
                        t.offset = a.level - b.level;
                        if t.offset != 0 {
                            t.content = HtmlWriter::offset_division(&t.content, t.offset);
                        }
                    }
                } else {
                    level = a.level.max(b.level);
                }
            } else {
                if !is_text {
                    level = a.level.max(b.level);
                }
                if content == "*" {
                    if a.val_type == ValueTypes::Number && b.val_type == ValueTypes::Unit {
                        if a.ttype == TokenTypes::Constant
                            && b.content != "\u{00B0}"
                            && !self.p.settings.borrow().unit_per_element
                        {
                            t.unit_once = Some((sa.clone(), sb.clone()));
                        }
                        if b.content == "\u{00B0}" || is_text {
                            t.content = format!("{}{}", sa, sb);
                        } else if self.writer.format_type() == OutputFormat::Xml {
                            t.content = format!("{}{}{}", sa, XmlWriter::run_thin_space(), sb);
                        } else {
                            t.content = format!("{}{}{}", sa, THIN_SPACE, sb);
                        }
                    } else if a.val_type == ValueTypes::Vector && b.val_type == ValueTypes::Vector {
                        t.content = format!("{}{}{}", sa, self.writer.format_operator('\u{2299}'), sb);
                    } else if t.order == 1 && self.writer.format_type() == OutputFormat::Html {
                        t.content = format!("{}{}{}", sa, HtmlWriter::UNIT_PRODUCT, sb);
                    } else if t.order == 1 && self.writer.format_type() == OutputFormat::Xml {
                        t.content = format!("{}{}{}", sa, XmlWriter::unit_product(), sb);
                    } else {
                        t.content = format!("{}{}{}", sa, self.writer.format_operator('*'), sb);
                    }
                } else if (content == "\u{00F7}" || content == "/") && t.order == 1 {
                    if self.writer.format_type() == OutputFormat::Html {
                        t.content = format!("{}{}{}", sa, HtmlWriter::UNIT_DIVISION, sb);
                    } else if self.writer.format_type() == OutputFormat::Xml {
                        t.content = format!("{}{}{}", sa, XmlWriter::unit_division(), sb);
                    } else {
                        t.content = format!("{}{}{}", sa, TextWriter::UNIT_DIVISION, sb);
                    }
                } else {
                    let c0 = content.chars().next().unwrap_or('\0');
                    t.content = format!("{}{}{}", sa, self.writer.format_operator(c0), sb);
                }
            }
            t.val_type = if a.val_type == b.val_type {
                a.val_type
            } else {
                ValueTypes::NumberWithUnit
            };
            t.level = level;
            t.min_offset = a.min_offset.min(b.min_offset).min(t.offset);
            t.max_offset = a.max_offset.max(b.max_offset).max(t.offset);
            if is_assignment(&content) {
                self.p.output_assignment_position.set(t.content.len() - sb.len());
            } else if t.order != 1 && !(a.val_type == ValueTypes::Number && b.val_type == ValueTypes::Unit) {
                self.has_operators = true;
            }
        }
        if a.val_type == ValueTypes::Vector || b.val_type == ValueTypes::Vector {
            t.val_type = ValueTypes::Vector;
        }
        Ok(())
    }

    fn render_function_token(&mut self, t: &mut RenderToken, b: RenderToken) {
        let st = t.content.clone();
        let head = if matches!(
            t.ttype,
            TokenTypes::Function
                | TokenTypes::VectorFunction
                | TokenTypes::MatrixFunction
                | TokenTypes::MatrixOptionalFunction
        ) {
            self.writer.format_function(&st)
        } else {
            self.writer.format_variable(&st, "", false)
        };
        t.content = format!(
            "{}{}",
            head,
            self.add_brackets(&b.content, b.level, b.min_offset, b.max_offset, '(', ')')
        );
        t.level = b.level;
        if t.ttype == TokenTypes::Function && b.val_type == ValueTypes::Vector
            || t.ttype == TokenTypes::VectorFunction && vector_calculator::is_vector_result_function(&st)
        {
            t.val_type = ValueTypes::Vector;
        }
    }

    fn render_function2_token(&mut self, t: &mut RenderToken, b: RenderToken) -> CResult<()> {
        let st = t.content.clone();
        let a = self.pop()?;
        if st.eq_ignore_ascii_case("root") {
            let bc = b.content.clone();
            self.render_root_token(t, a.clone(), &bc);
        } else {
            t.level = a.level.max(b.level);
            t.min_offset = a.min_offset.min(b.min_offset);
            t.max_offset = a.max_offset.max(b.max_offset);
            let inner = format!("{}{}{}", a.content, self.div, b.content);
            t.content = format!(
                "{}{}",
                self.writer.format_function(&st),
                self.add_brackets(&inner, t.level, t.min_offset, t.max_offset, '(', ')')
            );
        }
        if t.ttype == TokenTypes::Function2 && (a.val_type == ValueTypes::Vector || b.val_type == ValueTypes::Vector)
            || t.ttype == TokenTypes::VectorFunction2 && vector_calculator::is_vector_result_function(&st)
        {
            t.val_type = ValueTypes::Vector;
        }
        Ok(())
    }

    fn render_function3_token(&mut self, t: &mut RenderToken, c: RenderToken) -> CResult<()> {
        let st = t.content.clone();
        let b = self.pop()?;
        let a = self.pop()?;
        if self.format_equations && t.ttype == TokenTypes::Function3 && t.index == 0 {
            t.level = a.level.max(b.level) + c.level + 1;
            t.content = self.writer.format_if(&a.content, &b.content, &c.content, t.level);
        } else {
            t.level = a.level.max(b.level.max(c.level));
            let inner = format!("{}{}{}{}{}", a.content, self.div, b.content, self.div, c.content);
            t.content = format!(
                "{}{}",
                self.writer.format_function(&st),
                self.add_brackets(&inner, t.level, t.min_offset, t.max_offset, '(', ')')
            );
        }
        if t.ttype == TokenTypes::Function3 && (b.val_type == ValueTypes::Vector || c.val_type == ValueTypes::Vector)
            || t.ttype == TokenTypes::VectorFunction3 && vector_calculator::is_vector_result_function(&st)
        {
            t.val_type = ValueTypes::Vector;
        }
        Ok(())
    }

    fn render_function_n_token(&mut self, t: &mut RenderToken, last: RenderToken, n: usize) -> CResult<()> {
        let st = t.content.clone();
        let mut args = vec![last];
        for _ in 1..n {
            args.insert(0, self.pop()?);
        }
        t.level = args.iter().map(|x| x.level).max().unwrap_or(0);
        let inner: Vec<String> = args.iter().map(|x| x.content.clone()).collect();
        let inner = inner.join(&self.div);
        t.content = format!(
            "{}{}",
            self.writer.format_function(&st),
            self.add_brackets(&inner, t.level, t.min_offset, t.max_offset, '(', ')')
        );
        Ok(())
    }

    fn render_multi_function_token(&mut self, t: &mut RenderToken, b: RenderToken) -> CResult<()> {
        let st = t.content.clone();
        let mf_param_count = t.parameter_count - 1;
        if st.eq_ignore_ascii_case("switch") && self.format_equations {
            let count = (mf_param_count + 1).max(0) as usize;
            let mut args = vec![String::new(); count];
            let mut b = b;
            if count > 0 {
                args[count - 1] = b.content.clone();
            }
            let mut j = mf_param_count - 1;
            while j >= 0 {
                let a = self.pop()?;
                args[j as usize] = a.content.clone();
                if (mf_param_count - j) % 2 == 1 || j == 0 {
                    t.level += a.level.max(b.level) + 1;
                }
                b = a;
                j -= 1;
            }
            if mf_param_count == 0 {
                t.level = b.level;
            } else {
                t.level -= 1;
            }
            t.content = self.writer.format_switch(&args, t.level);
        } else {
            let s = self.render_parameters(t, b, mf_param_count, false, false)?;
            t.content = format!(
                "{}{}",
                self.writer.format_function(&st),
                self.add_brackets(&s, t.level, t.min_offset, t.max_offset, '(', ')')
            );
            t.min_offset = 0;
            t.max_offset = 0;
        }
        if t.ttype == TokenTypes::VectorMultiFunction {
            t.val_type = ValueTypes::Vector;
        }
        Ok(())
    }

    fn render_custom_function_token(&mut self, t: &mut RenderToken, b: RenderToken) -> CResult<()> {
        let mut cf_parameter_count = t.parameter_count - 1;
        let mut b = b;
        if cf_parameter_count < 0 {
            let cf = self
                .p
                .functions
                .borrow()
                .get(t.index)
                .ok_or_else(crate::calculator::index_out_of_range)?;
            cf_parameter_count = cf.parameter_count() as i32 - 1;
        } else if let Some(map) = t.argument_map.clone() {
            if (t.positional_count as usize) < map.len() {
                b = self.render_keyword_arguments(t, b, &map)?;
            }
        }
        let s = self.render_parameters(t, b, cf_parameter_count, false, false)?;
        t.content = format!(
            "{}\u{200A}{}",
            self.writer.format_variable(&t.content, "", false),
            self.add_brackets(&s, t.level, t.min_offset, t.max_offset, '(', ')')
        );
        t.min_offset = 0;
        t.max_offset = 0;
        Ok(())
    }

    fn render_keyword_arguments(&mut self, t: &RenderToken, b: RenderToken, map: &[usize]) -> CResult<RenderToken> {
        let cf = self
            .p
            .functions
            .borrow()
            .get(t.index)
            .ok_or_else(crate::calculator::index_out_of_range)?;
        let n = map.len();
        let mut args: Vec<RenderToken> = vec![RenderToken::simple("", TokenTypes::None, 0); n];
        args[n - 1] = b;
        let mut k = n as isize - 2;
        while k >= 0 {
            args[k as usize] = self.pop()?;
            k -= 1;
        }
        for (k, arg) in args.iter_mut().enumerate().skip(t.positional_count as usize) {
            arg.content = format!(
                "{}{}{}",
                self.writer.format_variable(&cf.parameter_name(map[k]), "", false),
                self.writer.format_operator('='),
                arg.content
            );
        }
        let last = args.pop().unwrap();
        for a in args {
            self.stack.push(a);
        }
        Ok(last)
    }

    /// The unit written by all of `b` and the `count` tokens below it on the stack, when each is a number
    /// with a unit (or a row of them) and the unit is the same one; the unit as the writer renders it.
    fn common_unit_content(&self, b: &RenderToken, count: i32) -> Option<String> {
        let n = self.stack.len();
        let count = count.max(0) as usize;
        if count > n {
            return None;
        }
        let unit = &b.unit_once.as_ref()?.1;
        self.stack[n - count..]
            .iter()
            .all(|a| a.unit_once.as_ref().is_some_and(|(_, u)| u == unit))
            .then(|| unit.clone())
    }

    /// The elements of a row (`b` and the `count` tokens below it) without their unit, joined as the row is.
    fn stripped_row(&self, b: &RenderToken, count: i32) -> String {
        let n = self.stack.len();
        let count = (count.max(0) as usize).min(n);
        let mut parts: Vec<&str> = self.stack[n - count..]
            .iter()
            .map(|a| a.unit_once.as_ref().map_or(a.content.as_str(), |u| u.0.as_str()))
            .collect();
        parts.push(b.unit_once.as_ref().map_or(b.content.as_str(), |u| u.0.as_str()));
        parts.join(&self.div)
    }

    /// Writes the unit the elements share once, after the closing bracket (as the writers do for a result).
    fn append_common_unit(&self, t: &mut RenderToken, unit: Option<String>) {
        if let Some(u) = unit {
            if self.writer.format_type() == OutputFormat::Html {
                t.content.push('\u{200A}');
            }
            t.content.push_str(&u);
        }
    }

    fn render_parameters(
        &mut self,
        t: &mut RenderToken,
        b: RenderToken,
        count: i32,
        matrix: bool,
        strip: bool,
    ) -> CResult<String> {
        let own = |x: &RenderToken| -> String {
            match (&x.unit_once, strip) {
                (Some((n, _)), true) => n.clone(),
                _ => x.content.clone(),
            }
        };
        let mut s = own(&b);
        t.level = b.level;
        t.min_offset = b.min_offset;
        t.max_offset = b.max_offset;
        let d = if matrix {
            self.writer.format_operator('|')
        } else {
            self.div.clone()
        };
        for _ in 0..count.max(0) {
            let a = self.pop()?;
            s = format!("{}{}{}", own(&a), d, s);
            if a.level > t.level {
                t.level = a.level;
            }
            t.min_offset = a.min_offset.min(t.min_offset);
            t.max_offset = a.max_offset.max(t.max_offset);
        }
        Ok(s)
    }

    fn render_root_token(&mut self, t: &mut RenderToken, b: RenderToken, s: &str) {
        let mut b = b;
        let offset = b.max_offset + b.min_offset;
        b.level += (b.max_offset - b.min_offset) / 2;
        let sb = if offset == 0 {
            b.content.clone()
        } else {
            fix_offset(&b.content, offset)
        };
        t.content = self.writer.format_root(&sb, b.level, s);
        t.level = b.level;
    }

    fn add_brackets(&self, s: &str, level: i32, min_offset: i32, max_offset: i32, left: char, right: char) -> String {
        let offset = min_offset + max_offset;
        let level = level + (max_offset - min_offset) / 2;
        if offset == 0 {
            return self.writer.add_brackets(s, level, left, right);
        }
        self.writer.add_brackets(&fix_offset(s, offset), level, left, right)
    }
}
