//! Port of Parsers/ExpressionParser (ExpressionParser.cs, .Keywords.cs, .Tokens.cs,
//! .Condition.cs, .Loops.cs, .ParseModes.cs). The worksheet driver: splits lines into
//! comments and expressions, runs directives, conditions and loops, and writes HTML.
pub mod condition;
mod data_exchange;
pub use data_exchange::{base64_decode, base64_encode, MAX_EMBEDDED_DATA_SIZE};
pub mod loops;
mod ui;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::base_types::complex::Complex;
use crate::base_types::real_value::RealValue;
use crate::base_types::value::Value;
use crate::calcpad_error::{CalcpadError, CalcpadErrorSource};
use crate::extension_methods::{tag_end_index, trim, trim_end};
use crate::markdig;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::messages;
use crate::netfmt::{self, fmt, html_encode};
use crate::output::html_writer::HtmlWriter;
use crate::output::OutputWriter;
use crate::parsers::math_parser::{MathParser, VariableSubstitutionOptions};
use crate::settings::{MathSettings, PlotSettings};
use crate::validator;

use condition::{Condition, REMOVE_CONDITION_KEYWORD};
use loops::{Loop, LoopKind, MAX_COUNT};

const MAX_HTML_LINES: i32 = 200000;
// Markdig is super-linear on pathological input, so a block is capped by size as well as by line count.
const MAX_MARKDOWN_CHARS: usize = 4_000_000;

/// Appended to the error of a condition that could not be evaluated (deviation D1).
pub const UNRESOLVED_CONDITION: &str =
    "The condition could not be evaluated, so the block is unresolved and none of its branches were run.";

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[repr(i32)]
pub enum Keyword {
    None = 0,
    Hide,
    Show,
    Pre,
    Post,
    EndHide,
    EndShow,
    EndPre,
    EndPost,
    Val,
    Equ,
    Noc,
    EndVal,
    EndEqu,
    EndNoc,
    NoSub,
    NoVar,
    VarSub,
    EndNoSub,
    EndNoVar,
    EndVarSub,
    Const,
    Split,
    Wrap,
    Deg,
    Rad,
    Gra,
    Round,
    Format,
    If,
    ElseIf,
    Else,
    EndIf,
    While,
    For,
    Repeat,
    Loop,
    Break,
    Continue,
    Local,
    Global,
    Pause,
    Input,
    Md,
    Read,
    Write,
    Append,
    Phasor,
    Complex,
    Settings,
    Ui,
    ProjectPath,
    LibraryPath,
    Html,
    Cpd,
    Markdown,
    EndHtml,
    EndCpd,
    EndMarkdown,
    SkipLine,
}

const KEYWORDS: [(Keyword, &str); 59] = [
    (Keyword::Hide, "hide"),
    (Keyword::Show, "show"),
    (Keyword::Pre, "pre"),
    (Keyword::Post, "post"),
    (Keyword::EndHide, "end hide"),
    (Keyword::EndShow, "end show"),
    (Keyword::EndPre, "end pre"),
    (Keyword::EndPost, "end post"),
    (Keyword::Val, "val"),
    (Keyword::Equ, "equ"),
    (Keyword::Noc, "noc"),
    (Keyword::EndVal, "end val"),
    (Keyword::EndEqu, "end equ"),
    (Keyword::EndNoc, "end noc"),
    (Keyword::NoSub, "nosub"),
    (Keyword::NoVar, "novar"),
    (Keyword::VarSub, "varsub"),
    (Keyword::EndNoSub, "end nosub"),
    (Keyword::EndNoVar, "end novar"),
    (Keyword::EndVarSub, "end varsub"),
    (Keyword::Const, "const"),
    (Keyword::Split, "split"),
    (Keyword::Wrap, "wrap"),
    (Keyword::Deg, "deg"),
    (Keyword::Rad, "rad"),
    (Keyword::Gra, "gra"),
    (Keyword::Round, "round"),
    (Keyword::Format, "format"),
    (Keyword::If, "if"),
    (Keyword::ElseIf, "else if"),
    (Keyword::Else, "else"),
    (Keyword::EndIf, "end if"),
    (Keyword::While, "while"),
    (Keyword::For, "for"),
    (Keyword::Repeat, "repeat"),
    (Keyword::Loop, "loop"),
    (Keyword::Break, "break"),
    (Keyword::Continue, "continue"),
    (Keyword::Local, "local"),
    (Keyword::Global, "global"),
    (Keyword::Pause, "pause"),
    (Keyword::Input, "input"),
    (Keyword::Md, "md"),
    (Keyword::Read, "read"),
    (Keyword::Write, "write"),
    (Keyword::Append, "append"),
    (Keyword::Phasor, "phasor"),
    (Keyword::Complex, "complex"),
    (Keyword::Settings, "settings"),
    (Keyword::Ui, "ui"),
    (Keyword::ProjectPath, "projectpath"),
    (Keyword::LibraryPath, "librarypath"),
    (Keyword::Html, "html"),
    (Keyword::Cpd, "cpd"),
    (Keyword::Markdown, "markdown"),
    (Keyword::EndHtml, "end html"),
    (Keyword::EndCpd, "end cpd"),
    (Keyword::EndMarkdown, "end markdown"),
    (Keyword::SkipLine, "skipline"),
];

fn max_keyword_length() -> usize {
    KEYWORDS.iter().map(|(_, n)| n.len()).max().unwrap()
}

/// `GetKeyword`: case-insensitive prefix match, candidates in enum order for the first letter.
fn get_keyword(s: &[char]) -> Keyword {
    let n = max_keyword_length().min(s.len().saturating_sub(1));
    if n < 3 {
        return Keyword::None;
    }
    let first = s[1].to_lowercase().next().unwrap_or('\0');
    if !first.is_ascii_lowercase() || first == 'z' {
        return Keyword::None;
    }
    let lower: String = s[1..1 + n].iter().flat_map(|c| c.to_lowercase()).collect();
    for (k, name) in KEYWORDS.iter() {
        if name.starts_with(first) && lower.starts_with(name) {
            return *k;
        }
    }
    Keyword::None
}

fn keyword_name(k: Keyword) -> &'static str {
    KEYWORDS.iter().find(|(kk, _)| *kk == k).map(|(_, n)| *n).unwrap_or("")
}

/// `KeywordLength(keyword)`: characters including the leading '#'.
fn keyword_length(k: Keyword) -> usize {
    keyword_name(k).len() + 1
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeywordResult {
    None,
    Continue,
    Break,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenTypes {
    Expression,
    Heading,
    Text,
    Html,
    Error,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub value: String,
    pub ttype: TokenTypes,
    pub cache_id: i32,
}

type TokenList = Rc<RefCell<Vec<Token>>>;

#[derive(Clone)]
struct LineInfo {
    tokens: Option<TokenList>,
    keyword: Keyword,
    source_line: i32,
}

impl LineInfo {
    fn empty() -> LineInfo {
        LineInfo {
            tokens: None,
            keyword: Keyword::None,
            source_line: 0,
        }
    }
    fn is_cached(&self) -> bool {
        self.tokens.is_some()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ParseMode {
    Cpd,
    Html,
    Markdown,
}

/// Port of the public ExpressionParser class.
/// Records the variable the expression just calculated assigns (`x = ...` or `x ← ...`),
/// with its value; an input field (`?`) or a `#UI` control marks it as an input.
fn capture_assignment(p: &MathParser, captured: &mut Vec<CapturedValue>, from_ui: bool) {
    let rpn = p.rpn.borrow();
    let Some(rpn) = rpn.as_ref() else { return };
    let (Some(first), Some(last)) = (rpn.first(), rpn.last()) else {
        return;
    };
    let (first, last) = (first.borrow(), last.borrow());
    if first.ttype != crate::parsers::math_parser::tokens::TokenTypes::Variable
        || !(last.content == "=" || last.content == "\u{2190}")
    {
        return;
    }
    let name = first.content.clone();
    let is_input = from_ui
        || rpn
            .iter()
            .any(|t| t.borrow().ttype == crate::parsers::math_parser::tokens::TokenTypes::Input);
    let Ok(value) = p.get_variable(&name) else { return };
    match captured.iter_mut().find(|c| c.name == name) {
        Some(c) => {
            c.value = value;
            c.is_input |= is_input;
        }
        None => captured.push(CapturedValue { name, value, is_input }),
    }
}

/// A variable recorded for the Excel export (`ExpressionParser::capture_values`).
#[derive(Clone, Debug)]
pub struct CapturedValue {
    pub name: String,
    pub value: Value,
    /// Set by an input field (`?`) or a `#UI` control.
    pub is_input: bool,
}

pub struct ExpressionParser {
    pub math_settings: Rc<RefCell<MathSettings>>,
    pub plot_settings: PlotSettings,
    pub is_us: bool,
    pub debug: bool,
    pub for_print: bool,
    pub show_warnings: bool,
    pub show_error_lines: bool,
    pub enable_ui: bool,
    /// Values entered into `#UI` controls, keyed by `data-ui-var` (e.g. "L:1").
    pub ui_overrides: Option<HashMap<String, String>>,
    /// Full path of the worksheet; relative `#read`/`#write` paths resolve against its folder.
    pub source_file_path: Option<String>,
    /// Whether `#write`/`#append` write their file (they are reported either way).
    pub allow_data_write: bool,
    /// The files `#read`/`#write`/`#append` use.
    pub data_files: crate::data_files::DataFilesRef,
    pub html_result: String,
    /// `getXml`: also render every equation as Office Math (OMML) for the Word export. The
    /// equation's span then carries `id="eq-N"`, its index in `open_xml_expressions`.
    pub capture_open_xml: bool,
    /// `OpenXmlExpressions`
    pub open_xml_expressions: Vec<String>,
    /// Record each variable an expression assigns, for the Excel export (`captured_values`).
    pub capture_values: bool,
    /// The variables in the order they were first assigned, each with its last value.
    pub captured_values: Vec<CapturedValue>,
    /// The data of each `$Plot` drawn, in the order they appear in the page (with `capture_values`).
    pub captured_plots: Vec<crate::plotter::capture::CapturedChart>,
    /// The first out of scope feature hit while parsing, if any.
    pub out_of_scope: Option<String>,
    error_count: i32,
    is_val: i32,
    start_line: usize,
    current_line: isize,
    html_lines: i32,
    decimals: i32,
    calculate: bool,
    is_visible: bool,
    visibility_stack: Vec<bool>,
    output_mode_stack: Vec<i32>,
    substitution_stack: Vec<VariableSubstitutionOptions>,
    is_paused_by_user: bool,
    pause_char_count: usize,
    is_markdown_on: bool,
    parser: Option<MathParser>,
    sb: String,
    errors: VecDeque<usize>,
    line_cache: Vec<LineInfo>,
    condition: Condition,
    loops: Vec<Loop>,
    previous_keyword: Keyword,
    parse_mode: ParseMode,
    mode_suppressed: bool,
    parse_mode_stack: Vec<(ParseMode, bool)>,
    md_buffer: String,
    md_lines: Vec<(usize, i32)>,
    last_expression_failed: Option<(String, String)>,
    err_index: i32,
    first_err_index_by_line: std::collections::HashMap<usize, i32>,
    /// `Errors`: recorded only in Debug mode, like CalcpadCE.
    pub errors_list: Vec<CalcpadError>,
    ui: ui::UiState,
}

fn is_line_extension_char(c: char) -> bool {
    ";|&@:({[".contains(c)
}

fn has_line_extension(s: &str) -> bool {
    s.ends_with(" _") || s.chars().last().is_some_and(is_line_extension_char) && !validator::is_comment(s)
}

impl Default for ExpressionParser {
    fn default() -> Self {
        ExpressionParser::new()
    }
}

impl ExpressionParser {
    pub fn new() -> ExpressionParser {
        ExpressionParser {
            math_settings: Rc::new(RefCell::new(MathSettings::default())),
            plot_settings: PlotSettings::default(),
            is_us: true,
            debug: false,
            for_print: false,
            show_warnings: true,
            show_error_lines: true,
            enable_ui: false,
            ui_overrides: None,
            source_file_path: None,
            allow_data_write: true,
            data_files: crate::data_files::default_files(),
            html_result: String::new(),
            capture_open_xml: false,
            open_xml_expressions: Vec::new(),
            capture_values: false,
            captured_values: Vec::new(),
            captured_plots: Vec::new(),
            out_of_scope: None,
            error_count: 0,
            is_val: 0,
            start_line: 0,
            current_line: -1,
            html_lines: 0,
            decimals: 2,
            calculate: true,
            is_visible: true,
            visibility_stack: Vec::new(),
            output_mode_stack: Vec::new(),
            substitution_stack: Vec::new(),
            is_paused_by_user: false,
            pause_char_count: 0,
            is_markdown_on: false,
            parser: None,
            sb: String::new(),
            errors: VecDeque::new(),
            line_cache: Vec::new(),
            condition: Condition::new(),
            loops: Vec::new(),
            previous_keyword: Keyword::None,
            parse_mode: ParseMode::Cpd,
            mode_suppressed: false,
            parse_mode_stack: Vec::new(),
            md_buffer: String::new(),
            md_lines: Vec::new(),
            last_expression_failed: None,
            err_index: 0,
            first_err_index_by_line: std::collections::HashMap::new(),
            errors_list: Vec::new(),
            ui: ui::UiState::default(),
        }
    }

    /// The value of a variable after the last parse, for callers that read results (the
    /// template tests, exports). `None` for a name the worksheet did not define.
    pub fn variable(&self, name: &str) -> Option<Value> {
        self.parser.as_ref().and_then(|p| p.get_variable(name).ok())
    }

    fn p(&self) -> &MathParser {
        self.parser.as_ref().unwrap()
    }

    fn is_paused(&self) -> bool {
        self.start_line > 0
    }

    fn is_non_cpd_mode(&self) -> bool {
        self.parse_mode != ParseMode::Cpd
    }

    fn html_id(&self) -> String {
        let source_line = self.parser.as_ref().map(|p| p.line.get()).unwrap_or(0);
        self.line_attributes(self.cur(), source_line)
    }

    /// `LineAttributes(currentLine, sourceLine)`
    fn line_attributes(&self, current_line: usize, source_line: i32) -> String {
        if !self.debug {
            return String::new();
        }
        let is_first_pass = self.loops.is_empty() || self.loops.last().unwrap().is_first_pass();
        let id_attribute = if is_first_pass {
            format!(" id=\"line-{}\"", current_line + 1)
        } else {
            String::new()
        };
        format!("{} data-source-line=\"{}\" class=\"line\"", id_attribute, source_line)
    }

    /// `Id(line)`
    fn err_id(&mut self, line: usize) -> String {
        if !self.debug {
            return String::new();
        }
        self.err_index += 1;
        self.first_err_index_by_line.entry(line).or_insert(self.err_index);
        format!(" id=\"err-{}\" data-source-line=\"{}\"", self.err_index, line + 1)
    }

    /// `RecordError(line, message, enabled)`
    fn record_error(&mut self, line: usize, message: &str, enabled: bool) {
        if !enabled {
            return;
        }
        if self.is_visible {
            self.errors.push_back(line);
        }
        let pl = self.parser.as_ref().map(|p| p.line.get()).unwrap_or(0);
        let source_line = if pl > 0 { pl } else { line as i32 + 1 };
        self.errors_list.push(CalcpadError {
            source_line,
            output_line: line as i32 + 1,
            message: message.to_string(),
            source: CalcpadErrorSource::Expression,
        });
    }

    /// `AppendErrors()`
    fn append_errors(&mut self) {
        if self.errors.len() == 1 {
            self.sb.push_str(messages::Error_found_on_line);
            self.sb.push('\n');
        } else {
            self.sb
                .push_str(&fmt(messages::Errors_found_on_lines, &[&self.errors.len().to_string()]));
            self.sb.push('\n');
        }
        let mut count = 0;
        let mut prev_line = 0usize;
        while count < 20 {
            let src_line = match self.errors.pop_front() {
                None => break,
                Some(l) => l,
            };
            let err_line = src_line + 1;
            if err_line != prev_line {
                count += 1;
                let attr = match self.first_err_index_by_line.get(&src_line) {
                    Some(idx) => format!(" data-error=\"err-{}\"", idx),
                    None => String::new(),
                };
                self.sb.push_str(&format!(
                    " <span class=\"roundBox\" data-line=\"{}\"{}>{}</span>",
                    err_line, attr, err_line
                ));
            }
            prev_line = err_line;
        }
        if !self.errors.is_empty() {
            self.sb.push_str(" ...");
        }
        self.sb.push_str("</div>");
        self.errors.clear();
    }

    fn cur(&self) -> usize {
        self.current_line.max(0) as usize
    }

    fn is_enabled(&self) -> bool {
        self.condition.is_satisfied() && (self.loops.is_empty() || !self.loops.last().unwrap().is_broken())
            || !self.calculate
    }

    /// `Parse(sourceCode, calculate, getXml)`
    pub fn parse(&mut self, code: &str, calculate: bool) {
        let mut lines: Vec<usize> = vec![0];
        for (i, b) in code.bytes().enumerate() {
            if b == b'\n' {
                lines.push(i + 1);
            }
        }
        if *lines.last().unwrap() < code.len() {
            lines.push(code.len());
        }
        self.initialize(calculate, lines.len());
        let line_count = lines.len() - 1;
        let mut text_span = String::new();
        let r = self.parse_lines(code, &lines, line_count, &mut text_span);
        if let Err(ex) = r {
            if ex.unexpected {
                let msg = fmt(
                    messages::Unexpected_error_0_Please_check_the_expression_consistency,
                    &[&ex.message],
                );
                if self.is_visible {
                    let h = self.err_html(&msg);
                    self.sb.push_str(&h);
                }
                let line = self.cur();
                self.record_error(line, &msg, self.debug);
            } else {
                let t = text_span.clone();
                self.append_error(&t, &ex.message);
            }
        }
        self.flush_markdown();
        self.finalize(line_count);
    }

    fn parse_lines(&mut self, code: &str, lines: &[usize], line_count: usize, text_span: &mut String) -> CResult<()> {
        loop {
            self.current_line += 1;
            if self.current_line as usize >= line_count {
                break;
            }
            // #UI metadata is scoped to the line that declared it. Clearing here
            // covers the paths that skip ParseLine, e.g. an unsatisfied condition.
            self.reset_ui_state();
            let cl = self.cur();
            let current_line_cache = self.line_cache[cl].clone();
            let mut keyword = current_line_cache.keyword;
            if keyword == Keyword::SkipLine {
                continue;
            }
            if keyword == Keyword::Continue && !self.mode_suppressed {
                self.parse_keyword_continue()?;
                continue;
            }
            if !self.mode_suppressed && current_line_cache.is_cached() && keyword == Keyword::None {
                if self.is_enabled() {
                    let line = if current_line_cache.source_line != 0 {
                        current_line_cache.source_line
                    } else {
                        cl as i32 + 1
                    };
                    self.p().line.set(line);
                    self.condition.set_condition(-1)?;
                    self.p().is_calculation.set(self.is_val != -1);
                    self.parse_line(current_line_cache.tokens.clone().unwrap(), Keyword::None)?;
                }
                continue;
            }
            let i1 = lines[cl];
            let i2 = lines[cl + 1];
            let mut line_span: &str = &code[i1..i2];
            if let Some(eol) = line_span.find('\u{000B}') {
                let num = trim(&line_span[eol + 1..]);
                let n: i32 = num.parse().map_err(|_| {
                    MathParserException::unexpected(format!("The input string '{}' was not in a correct format.", num))
                })?;
                self.p().line.set(n);
                line_span = &line_span[..eol];
            } else {
                self.p().line.set(cl as i32 + 1);
            }
            let raw_line = trim_end(line_span).to_string();
            let line_span = trim(line_span);
            if !self.is_non_cpd_mode() && has_line_extension(trim_end(text_span)) {
                let prev = text_span.clone();
                let c = prev.chars().last().unwrap();
                if c == '_' {
                    let chars: Vec<char> = prev.chars().collect();
                    let head: String = chars[..chars.len() - 2].iter().collect();
                    *text_span = format!("{}{}", head, line_span);
                } else {
                    *text_span = format!("{} {}", prev, line_span);
                }
            } else {
                *text_span = line_span.to_string();
            }
            if !self.is_non_cpd_mode() && has_line_extension(trim_end(text_span)) {
                self.line_cache[cl] = LineInfo {
                    tokens: None,
                    keyword: Keyword::SkipLine,
                    source_line: 0,
                };
                continue;
            }
            if self.p().is_canceled.get() {
                break;
            }
            if text_span.is_empty() {
                if self.mode_suppressed {
                    continue;
                }
                if self.is_non_cpd_mode() {
                    if self.is_enabled() {
                        self.parse_non_cpd_mode_line("");
                    }
                    continue;
                }
                if self.is_visible && self.is_val != 1 && self.html_lines < MAX_HTML_LINES && self.is_enabled() {
                    let id = self.html_id();
                    self.sb.push_str(&format!("<p{}>&nbsp;</p>\n", id));
                }
                continue;
            }
            let line_cache = cl;
            self.p().is_const.set(false);
            let ts = text_span.clone();
            let result = self.parse_keyword(&ts, &mut keyword)?;
            if keyword != current_line_cache.keyword {
                self.line_cache[line_cache] = LineInfo {
                    tokens: current_line_cache.tokens.clone(),
                    keyword,
                    source_line: self.p().line.get(),
                };
            }
            match result {
                KeywordResult::Continue => continue,
                KeywordResult::Break => break,
                KeywordResult::None => {}
            }
            if self.is_non_cpd_mode() {
                if self.is_enabled() {
                    self.parse_non_cpd_mode_line(&raw_line);
                }
                continue;
            }
            self.p().is_calculation.set(self.is_val != -1);
            let first = ts.chars().next().unwrap_or('\0');
            let is_plot = first == '$' && self.parse_plot(&ts)?;
            if !is_plot && self.parse_condition(&ts, keyword)? {
                let cl = self.cur();
                let tokens = if self.line_cache[cl].is_cached() {
                    self.line_cache[cl].tokens.clone().unwrap()
                } else {
                    let skip_chars = if keyword == Keyword::Ui {
                        self.ui.ui_skip_chars
                    } else if keyword == Keyword::Const {
                        keyword_length(Keyword::Const)
                    } else {
                        self.condition.keyword_length()
                    };
                    let rest: String = ts.chars().skip(skip_chars).collect();
                    let mut toks = self.get_tokens(&rest);
                    if self.is_markdown_on {
                        parse_markdown(&mut toks);
                    }
                    let toks = Rc::new(RefCell::new(toks));
                    self.line_cache[cl] = LineInfo {
                        tokens: Some(toks.clone()),
                        keyword,
                        source_line: self.p().line.get(),
                    };
                    toks
                };
                self.p().has_input_fields.set(false);
                self.parse_line(tokens, keyword)?;
                if self.p().has_input_fields.get() {
                    let cl = self.cur();
                    self.line_cache[cl] = LineInfo {
                        tokens: None,
                        keyword,
                        source_line: 0,
                    };
                }
            }
        }
        self.flush_markdown();
        if self.current_line as usize == line_count && (self.calculate || !self.is_paused()) {
            let if_not_closed = self.condition.id > 0 && !self.condition.is_loop();
            let loop_not_closed = !self.loops.is_empty();
            if if_not_closed {
                let h = self.err_html(messages::if_block_not_closed_Missing_end_if);
                self.sb.push_str(&h);
            }
            if loop_not_closed {
                let h = self.err_html(messages::Iteration_block_not_closed_Missing_loop);
                self.sb.push_str(&h);
            }
            let msg = if if_not_closed {
                messages::if_block_not_closed_Missing_end_if
            } else {
                messages::Iteration_block_not_closed_Missing_loop
            };
            let line = self.cur();
            self.record_error(line, msg, self.debug && (if_not_closed || loop_not_closed));
        }
        Ok(())
    }

    fn parse_plot(&mut self, s: &str) -> CResult<bool> {
        if validator::starts_with_ignore_case(s, "$plot") || validator::starts_with_ignore_case(s, "$map") {
            if self.is_visible && self.is_enabled() {
                let is_chart = validator::starts_with_ignore_case(s, "$p");
                let calculate = self.calculate;
                let parser = self.parser.as_ref().unwrap();
                parser.is_plotting.set(true);
                let result = if is_chart {
                    crate::parsers::chart_parser::parse(parser, &self.plot_settings, s, calculate)
                } else {
                    crate::parsers::map_parser::parse(parser, &mut self.plot_settings, s, calculate)
                };
                let chart = if is_chart && self.capture_values {
                    parser.last_captured_chart.borrow_mut().take()
                } else {
                    None
                };
                match result {
                    Ok(s1) => {
                        if let Some(c) = chart {
                            self.captured_plots.push(c);
                        }
                        let id = self.html_id();
                        self.sb.push_str(&insert_attribute(&s1, &id));
                        self.p().is_plotting.set(false);
                    }
                    // C# leaves IsPlotting set when the plot fails.
                    Err(e) if e.unexpected => return Err(e),
                    Err(e) => self.append_error(s, &e.message),
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn parse_condition(&mut self, s: &str, keyword: Keyword) -> CResult<bool> {
        if self.is_paused() && !self.calculate {
            self.condition.set_condition(-1)?;
            return Ok(keyword == Keyword::None);
        }
        self.condition.set_condition(keyword as i32 - Keyword::If as i32)?;
        if self.is_enabled() {
            let s_len = s.chars().count();
            if self.condition.keyword_length() == s_len {
                if self.condition.is_unchecked {
                    return Err(crate::exceptions::ConditionEmpty());
                }
                if self.is_visible && !self.calculate {
                    let id = self.html_id();
                    if keyword == Keyword::Else {
                        self.sb.push_str(&format!(
                            "</div><p{}>{}</p><div class = \"indent\">",
                            id,
                            self.condition.to_html()
                        ));
                    } else {
                        self.sb
                            .push_str(&format!("</div><p{}>{}</p>", id, self.condition.to_html()));
                    }
                }
            } else if self.condition.keyword_length() > 0
                && self.condition.is_found
                && self.condition.is_unchecked
                && self.calculate
            {
                self.condition.check(Complex::real(0.0))?;
            } else {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn parse_line(&mut self, tokens: TokenList, keyword: Keyword) -> CResult<()> {
        let kwd_length = self.condition.keyword_length();
        let is_output = self.is_visible && (!self.calculate || kwd_length == 0) && self.html_lines < MAX_HTML_LINES;
        self.last_expression_failed = None;
        if is_output {
            self.html_lines += 1;
            if self.html_lines == MAX_HTML_LINES {
                let all: String = tokens.borrow().iter().map(|t| t.value.clone()).collect();
                let msg = fmt(
                    messages::The_output_is_longer_than_0_lines_The_rest_will_be_skipped,
                    &[&MAX_HTML_LINES.to_string()],
                );
                self.append_error(&all, &msg);
            } else {
                let is_indent = keyword == Keyword::ElseIf || keyword == Keyword::EndIf;
                let line_type = tokens.borrow().first().map(|t| t.ttype).unwrap_or(TokenTypes::Text);
                let mut html_id: Option<String> = None;
                if self.is_val != 1 {
                    let mut id = self.html_id();
                    if self.has_ui_controls() {
                        id = if self.enable_ui {
                            if self.ui.line_ui_controls.as_ref().map_or(0, |c| c.len()) == 1 {
                                format!("{}{}", id, self.get_ui_attributes(0))
                            } else {
                                id
                            }
                        } else {
                            self.add_report_style(0, &id)
                        };
                    }
                    self.append_html_line_start(line_type, is_indent, &id);
                    html_id = Some(id);
                }
                // In debug mode the line attributes go into the first HTML token.
                let html_token = match &html_id {
                    Some(id) if line_type == TokenTypes::Html && !id.is_empty() => Some(tokens.borrow()[0].clone()),
                    _ => None,
                };
                if let Some(ht) = &html_token {
                    let v = insert_attribute(&ht.value, html_id.as_deref().unwrap());
                    tokens.borrow_mut()[0] = Token {
                        value: v,
                        ttype: TokenTypes::Html,
                        cache_id: -1,
                    };
                }
                if kwd_length > 0 {
                    let h = self.condition.to_html();
                    self.sb.push_str(&h);
                }
                let r = self.parse_tokens(&tokens, true);
                if let Some(ht) = html_token {
                    tokens.borrow_mut()[0] = ht;
                }
                r?;
                if self.is_val != 1 {
                    self.append_html_line_end(line_type, keyword == Keyword::If);
                }
                if let Some(e) = self.ui.pending_ui_error.take() {
                    let h = self.err_html(&e);
                    self.sb.push_str(&h);
                }
                if self.enable_ui && self.has_ui_controls() {
                    self.append_ui_datagrids();
                }
            }
        } else {
            self.parse_tokens(&tokens, false)?;
        }
        if self.condition.is_unchecked {
            if self.calculate {
                if let Some((expr, msg)) = self.last_expression_failed.take() {
                    // Deviation from CalcpadCE (see DEVIATIONS.md): a failed condition
                    // runs neither branch and is reported, instead of silently running #else.
                    self.condition.check_failed();
                    if self.is_visible {
                        let text = self.format_error(&html_encode(&expr), &format!("{} {}", msg, UNRESOLVED_CONDITION));
                        let h = self.err_html(&text);
                        self.sb.push_str(&h);
                    }
                } else {
                    let r = self.p().result_complex();
                    self.condition.check(r)?;
                }
            } else {
                self.condition.check_unchecked();
            }
        }
        Ok(())
    }

    fn append_html_line_start(&mut self, line_type: TokenTypes, is_indent: bool, html_id: &str) {
        if is_indent {
            self.sb.push_str("</div>");
        }
        if line_type == TokenTypes::Heading {
            self.sb.push_str(&format!("<h3{}>", html_id));
        } else if line_type != TokenTypes::Html {
            self.sb.push_str(&format!("<p{}>", html_id));
        }
    }

    fn append_html_line_end(&mut self, line_type: TokenTypes, indent: bool) {
        if line_type == TokenTypes::Heading {
            self.sb.push_str("</h3>");
        } else if line_type != TokenTypes::Html {
            self.sb.push_str("</p>");
        }
        if indent {
            self.sb.push_str("<div class = \"indent\">");
        }
        self.sb.push('\n');
    }

    fn initialize(&mut self, calculate: bool, line_count: usize) {
        self.html_lines = 0;
        self.error_count = 0;
        self.calculate = calculate;
        self.errors.clear();
        self.err_index = 0;
        self.first_err_index_by_line.clear();
        self.errors_list.clear();
        if !self.calculate {
            self.start_line = 0;
        }
        if self.start_line == 0 {
            self.math_settings.borrow_mut().format_string = None;
            let p = MathParser::new(self.math_settings.clone());
            p.show_warnings.set(self.show_warnings);
            self.parser = Some(p);
            self.decimals = self.math_settings.borrow().decimals();
            self.line_cache = vec![LineInfo::empty(); line_count];
            self.sb.clear();
            self.condition = Condition::new();
            self.loops.clear();
            self.is_val = 0;
            self.output_mode_stack.clear();
            self.substitution_stack.clear();
            self.previous_keyword = Keyword::None;
            self.is_markdown_on = false;
            self.parse_mode = ParseMode::Cpd;
            self.mode_suppressed = false;
            self.parse_mode_stack.clear();
            self.ui.ui_var_counts.clear();
            self.ui.ui_declaration_index.clear();
            self.open_xml_expressions.clear();
            self.captured_values.clear();
            self.captured_plots.clear();
        } else {
            if self.line_cache.len() < line_count {
                self.line_cache.resize(line_count, LineInfo::empty());
            }
            if self.sb.len() > self.pause_char_count {
                self.sb.truncate(self.pause_char_count);
            }
        }
        self.p().is_us.set(self.is_us);
        self.p().is_enabled.set(self.calculate);
        self.current_line = self.start_line as isize - 1;
        self.is_visible = true;
        self.visibility_stack.clear();
        self.md_buffer.clear();
        self.md_lines.clear();
        self.reset_ui_state();
    }

    fn finalize(&mut self, line_count: usize) {
        if self.current_line as usize == line_count && self.calculate {
            self.start_line = 0;
        }
        if self.start_line > 0 {
            self.sb.push_str(messages::Paused_Press_F5_to_continue);
        }
        if self.debug && !self.errors.is_empty() {
            self.append_errors();
        }
        self.html_result = self.sb.clone();
        if let Some(p) = &self.parser {
            if let Some(o) = p.out_of_scope.borrow().clone() {
                if self.out_of_scope.is_none() {
                    self.out_of_scope = Some(o);
                }
            }
        }
        if self.calculate && self.start_line == 0 {
            if let Some(p) = &self.parser {
                p.clear_cache();
            }
        }
    }

    fn parse_tokens(&mut self, tokens: &TokenList, is_output: bool) -> CResult<()> {
        let is_loop = !self.loops.is_empty() && self.calculate && self.is_val > -1;
        let count = tokens.borrow().len();
        for i in 0..count {
            let token = tokens.borrow()[i].clone();
            if token.ttype == TokenTypes::Expression {
                // What was actually parsed: a control's entered value replaces the right
                // hand side, and an error has to name that rather than the source.
                let mut expression = token.value.clone();
                let ui = self.take_ui_control(&token.value);
                let r = match ui {
                    Some(ui) => match self.prepare_ui_expression(ui, &token.value) {
                        Ok(e) => {
                            expression = e;
                            self.parse_ui_expression_token(ui, &expression, is_output)
                        }
                        Err(e) => Err(e),
                    },
                    None => self.parse_expression_token(tokens, i, &token, is_loop, is_output),
                };
                if let Err(ex) = r {
                    if ex.unexpected {
                        return Err(ex);
                    }
                    self.last_expression_failed = Some((expression.clone(), ex.message.clone()));
                    let err_text = if !self.calculate && expression.contains('?') {
                        expression.replace('?', "<input type=\"text\" size=\"2\" name=\"Var\">")
                    } else {
                        html_encode(&expression)
                    };
                    let err_text = self.format_error(&err_text, &ex.message);
                    if is_output {
                        // Keeping the control is what lets the user correct the value that
                        // broke the line; the error goes below it rather than in its place.
                        match ui {
                            Some(ui) if self.enable_ui && self.ui_type_is_not_grid(ui) => {
                                let control = self.build_ui_control_from_source(ui);
                                self.sb.push_str(&format!("<span class=\"eq\">{}</span>", control));
                                self.ui.pending_ui_error = Some(err_text.clone());
                            }
                            _ => {
                                let line = self.cur();
                                let id = self.err_id(line);
                                self.sb
                                    .push_str(&format!("<span class=\"err\"{}>{}</span>", id, err_text));
                            }
                        }
                    }
                    let line = self.cur();
                    self.record_error(line, &ex.message, self.debug);
                    self.error_count += 1;
                    if self.error_count == 40 {
                        return Err(MathParserException::new(messages::Too_many_errors));
                    }
                }
            } else if is_output {
                self.sb.push_str(&token.value);
            }
        }
        Ok(())
    }

    fn parse_expression_token(
        &mut self,
        tokens: &TokenList,
        i: usize,
        token: &Token,
        is_loop: bool,
        is_output: bool,
    ) -> CResult<()> {
        let cache_id = token.cache_id;
        let p = self.parser.as_ref().unwrap();
        if cache_id < 0 {
            p.parse(&token.value, true)?;
            if is_loop {
                let id = p.write_equation_to_cache(is_output)?;
                tokens.borrow_mut()[i].cache_id = id;
            }
        } else {
            p.read_equation_from_cache(cache_id);
        }
        if self.calculate && self.is_val > -1 {
            p.calculate(is_output, cache_id)?;
            if self.capture_values {
                capture_assignment(p, &mut self.captured_values, false);
            }
        } else {
            p.define_custom_units()?;
        }
        if is_output {
            if self.is_val == 1 && self.calculate {
                let v = p.result_as_val();
                self.sb.push_str(&v);
            } else {
                let html = p.to_html()?;
                if !html.is_empty() {
                    self.push_equation(&html)?;
                }
            }
        }
        Ok(())
    }

    /// The uncached path of `ParseTokens` for an expression that a `#UI` control drives.
    fn parse_ui_expression_token(&mut self, ui: usize, expression: &str, is_output: bool) -> CResult<()> {
        let p = self.parser.as_ref().unwrap();
        p.parse(expression, true)?;
        if self.calculate && self.is_val > -1 {
            p.calculate(is_output, -1)?;
            if self.capture_values {
                capture_assignment(p, &mut self.captured_values, true);
            }
        } else {
            p.define_custom_units()?;
        }
        if is_output {
            if self.is_val == 1 && self.calculate {
                let v = p.result_as_val();
                self.sb.push_str(&v);
            } else {
                let mut html = p.to_html()?;
                if self.enable_ui {
                    html = if self.ui_type_is_not_grid(ui) {
                        self.inject_ui_input(ui, &html)
                    } else {
                        String::new()
                    };
                }
                if !html.is_empty() {
                    self.push_equation(&html)?;
                }
            }
        }
        Ok(())
    }

    /// Appends an equation's span; with `capture_open_xml` (and formatted equations, as upstream)
    /// its Office Math goes to `open_xml_expressions` and the span is numbered.
    fn push_equation(&mut self, html: &str) -> CResult<()> {
        if self.capture_open_xml && self.math_settings.borrow().format_equations {
            let xml = self.parser.as_ref().unwrap().to_xml()?;
            self.open_xml_expressions.push(xml);
            let n = self.open_xml_expressions.len() - 1;
            self.sb
                .push_str(&format!("<span class=\"eq\" id=\"eq-{}\">{}</span>", n, html));
        } else {
            self.sb.push_str(&format!("<span class=\"eq\">{}</span>", html));
        }
        Ok(())
    }

    fn append_error(&mut self, line_content: &str, text: &str) {
        let s = line_content.replace('<', "&lt;").replace('>', "&gt;");
        if self.is_visible {
            let e = self.format_error(&s, text);
            let h = self.err_html(&e);
            self.sb.push_str(&h);
        }
        let line = self.cur();
        self.record_error(line, text, self.debug);
    }

    fn line_html(line: usize) -> String {
        format!("[<a href=\"#0\" data-text=\"{}\">{}</a>]", line + 1, line + 1)
    }

    fn format_error(&self, expr: &str, msg: &str) -> String {
        if self.show_error_lines {
            fmt(
                messages::Error_in_0_on_line_1_2,
                &[expr, &ExpressionParser::line_html(self.cur()), msg],
            )
        } else {
            fmt(messages::Error_in_0_1, &[expr, msg])
        }
    }

    fn err_html(&mut self, text: &str) -> String {
        let line = self.cur();
        let id = self.err_id(line);
        format!("<p class=\"err\"{}>{}</p>", id, text)
    }

    // ----- tokens (ExpressionParser.Tokens.cs) ------------------------------------------

    fn get_tokens(&self, s: &str) -> Vec<Token> {
        let chars: Vec<char> = s.chars().collect();
        let mut tokens: Vec<Token> = Vec::new();
        let mut start = 0usize;
        let mut end = 0usize;
        let mut current_separator = ' ';
        let len = chars.len();
        let mut i = 0;
        while i < len {
            let c = chars[i];
            if c == '\'' || c == '"' {
                if current_separator == ' ' || current_separator == c {
                    if current_separator == c {
                        let i1 = i + 1;
                        if i1 < len && chars[i1] == current_separator {
                            end += 2;
                            i = i1 + 1;
                            continue;
                        }
                    }
                    if end > start {
                        let v: String = chars[start..end].iter().collect();
                        self.add_token(&mut tokens, &v, current_separator);
                    }
                    start = i + 1;
                    end = i + 1;
                    current_separator = if current_separator == c { ' ' } else { c };
                } else {
                    end += 1;
                }
            } else {
                end += 1;
            }
            i += 1;
        }
        if end > start {
            let v: String = chars[start..end].iter().collect();
            self.add_token(&mut tokens, &v, current_separator);
        }
        tokens
    }

    fn add_token(&self, tokens: &mut Vec<Token>, value: &str, separator: char) {
        let mut token_value = value.replace("\"\"", "&quot;").replace("''", "&apos;");
        let token_type = match separator {
            ' ' => TokenTypes::Expression,
            '"' => TokenTypes::Heading,
            '\'' => TokenTypes::Text,
            _ => TokenTypes::Error,
        };
        if token_type == TokenTypes::Expression {
            if value.chars().all(char::is_whitespace) {
                return;
            }
        } else if self.is_val < 1 {
            if tokens.is_empty() {
                token_value.push(' ');
            } else {
                token_value = format!(" {} ", token_value);
            }
        }
        let mut token = Token {
            value: token_value.clone(),
            ttype: token_type,
            cache_id: -1,
        };
        if token.ttype == TokenTypes::Text {
            let t = token_value.trim_start();
            if t.starts_with('<') {
                token.ttype = TokenTypes::Html;
            }
        }
        tokens.push(token);
    }

    // ----- parse modes (ExpressionParser.ParseModes.cs) ---------------------------------

    fn parse_non_cpd_mode_line(&mut self, line: &str) {
        if !self.is_visible || self.html_lines >= MAX_HTML_LINES {
            return;
        }
        self.html_lines += 1;
        if self.html_lines == MAX_HTML_LINES {
            let msg = fmt(
                messages::The_output_is_longer_than_0_lines_The_rest_will_be_skipped,
                &[&MAX_HTML_LINES.to_string()],
            );
            self.flush_markdown();
            self.append_error(line, &msg);
            return;
        }
        if self.parse_mode == ParseMode::Markdown {
            if self.md_buffer.len() + line.len() > MAX_MARKDOWN_CHARS {
                self.html_lines = MAX_HTML_LINES;
                self.flush_markdown();
                let msg = fmt(
                    messages::The_output_is_longer_than_0_lines_The_rest_will_be_skipped,
                    &[&MAX_HTML_LINES.to_string()],
                );
                self.append_error(line, &msg);
                return;
            }
            self.md_buffer.push_str(line);
            self.md_buffer.push('\n');
            let source_line = self.p().line.get();
            self.md_lines.push((self.cur(), source_line));
            return;
        }
        let html_id = self.html_id();
        let trimmed = line.trim_start();
        if html_id.is_empty() || trimmed.is_empty() {
            self.sb.push_str(line);
        } else {
            self.sb.push_str(&line[..line.len() - trimmed.len()]);
            self.sb.push_str(&insert_attribute(trimmed, &html_id));
        }
        self.sb.push('\n');
    }

    /// `FlushMarkdown`: renders the buffered markdown, tagging each top level block with
    /// its first source line.
    fn flush_markdown(&mut self) {
        if self.md_lines.is_empty() {
            return;
        }
        let doc = markdig::parse(&self.md_buffer, &markdig::Pipeline::markdown_mode());
        for (block_line, html) in markdig::render_blocks(&doc) {
            if self.parser.as_ref().is_some_and(|p| p.is_canceled.get()) {
                break;
            }
            let html = html.trim_end();
            if html.is_empty() {
                continue;
            }
            let (line, source_line) = self.md_lines[block_line.min(self.md_lines.len() - 1)];
            let attributes = self.line_attributes(line, source_line);
            if attributes.is_empty() {
                self.sb.push_str(html);
            } else {
                self.sb.push_str(&insert_attribute(html, &attributes));
            }
            self.sb.push('\n');
        }
        self.md_buffer.clear();
        self.md_lines.clear();
    }

    // ----- keywords (ExpressionParser.Keywords.cs) --------------------------------------

    fn is_active_mode_keyword(&self, keyword: Keyword) -> bool {
        match keyword {
            Keyword::Html | Keyword::Cpd | Keyword::Markdown => true,
            Keyword::EndHtml => self.parse_mode == ParseMode::Html,
            Keyword::EndMarkdown => self.parse_mode == ParseMode::Markdown,
            Keyword::EndCpd => self.parse_mode == ParseMode::Cpd,
            _ => false,
        }
    }

    fn parse_keyword(&mut self, s: &str, keyword: &mut Keyword) -> CResult<KeywordResult> {
        let chars: Vec<char> = s.chars().collect();
        if self.is_paused_by_user {
            *keyword = Keyword::Pause;
        } else if chars[0] == '#' && *keyword == Keyword::None {
            *keyword = get_keyword(&chars);
        }
        let is_mode_keyword = self.is_active_mode_keyword(*keyword);
        if self.mode_suppressed && !is_mode_keyword {
            *keyword = Keyword::None;
            return Ok(KeywordResult::Continue);
        }
        if self.is_non_cpd_mode() && !is_mode_keyword {
            *keyword = Keyword::None;
        }
        if *keyword == Keyword::None {
            return Ok(KeywordResult::None);
        }
        let kw = *keyword;
        match kw {
            Keyword::Html => self.set_parse_mode(&chars, kw, ParseMode::Html),
            Keyword::Cpd => self.set_parse_mode(&chars, kw, ParseMode::Cpd),
            Keyword::Markdown => self.set_parse_mode(&chars, kw, ParseMode::Markdown),
            Keyword::EndHtml | Keyword::EndCpd | Keyword::EndMarkdown => {
                self.flush_markdown();
                let (m, sup) = self.parse_mode_stack.pop().unwrap_or((ParseMode::Cpd, false));
                self.parse_mode = m;
                self.mode_suppressed = sup;
            }
            Keyword::Hide => {
                let v = self.math_settings.borrow().show_hidden_output && self.is_visible;
                self.set_visibility(&chars, kw, v);
            }
            Keyword::Show => self.set_visibility(&chars, kw, true),
            Keyword::Pre => {
                let v = !self.for_print;
                self.set_visibility(&chars, kw, v);
            }
            Keyword::Post => {
                let v = !self.enable_ui;
                self.set_visibility(&chars, kw, v);
            }
            Keyword::EndHide | Keyword::EndShow | Keyword::EndPre | Keyword::EndPost => {
                self.is_visible = self.visibility_stack.pop().unwrap_or(true);
            }
            Keyword::Input => return Ok(self.parse_keyword_input()),
            Keyword::Pause => return Ok(self.parse_keyword_pause()),
            Keyword::Val => self.set_output_mode(&chars, kw, 1),
            Keyword::Equ => self.set_output_mode(&chars, kw, 0),
            Keyword::Noc => self.set_output_mode(&chars, kw, -1),
            Keyword::EndVal | Keyword::EndEqu | Keyword::EndNoc => {
                self.is_val = self.output_mode_stack.pop().unwrap_or(0);
            }
            Keyword::NoSub => self.set_substitution(&chars, kw, VariableSubstitutionOptions::VariablesOnly),
            Keyword::NoVar => self.set_substitution(&chars, kw, VariableSubstitutionOptions::SubstitutionsOnly),
            Keyword::VarSub => {
                self.set_substitution(&chars, kw, VariableSubstitutionOptions::VariablesAndSubstitutions)
            }
            Keyword::EndNoSub | Keyword::EndNoVar | Keyword::EndVarSub => {
                let v = self
                    .substitution_stack
                    .pop()
                    .unwrap_or(VariableSubstitutionOptions::VariablesAndSubstitutions);
                self.p().variable_substitution.set(v);
            }
            Keyword::Const => {
                self.p().is_const.set(true);
                return Ok(KeywordResult::None);
            }
            Keyword::Split => self.p().split.set(true),
            Keyword::Wrap => self.p().split.set(false),
            Keyword::Deg => self.p().set_degrees(0),
            Keyword::Rad => self.p().set_degrees(1),
            Keyword::Gra => self.p().set_degrees(2),
            Keyword::Round => self.parse_keyword_round(&chars, s)?,
            Keyword::Format => self.parse_keyword_format(&chars),
            Keyword::Repeat => self.parse_keyword_repeat(&chars, s)?,
            Keyword::For => self.parse_keyword_for(&chars, s)?,
            Keyword::While => self.parse_keyword_while(&chars, s)?,
            Keyword::Loop => self.parse_keyword_loop(s)?,
            Keyword::Break => {
                if self.parse_keyword_break() {
                    return Ok(KeywordResult::Break);
                }
            }
            Keyword::Continue => self.parse_keyword_continue()?,
            Keyword::Md => {
                let kl = keyword_length(Keyword::Md);
                if chars.len() > kl {
                    let expr: String = chars[kl..].iter().collect();
                    let expr = trim(&expr);
                    if expr.eq_ignore_ascii_case("on") {
                        self.is_markdown_on = true;
                    } else if expr.eq_ignore_ascii_case("off") {
                        self.is_markdown_on = false;
                    } else {
                        let msg = fmt(messages::Invalid_keyword_0, &[expr]);
                        self.append_error(s, &msg);
                    }
                } else {
                    self.is_markdown_on = true;
                }
            }
            Keyword::Read => self.parse_keyword_read(s)?,
            Keyword::Write | Keyword::Append => self.parse_keyword_write(s, kw)?,
            Keyword::Phasor => self.p().phasor.set(true),
            Keyword::Complex => self.p().phasor.set(false),
            Keyword::Settings => self.parse_keyword_settings(s),
            Keyword::Ui => return self.parse_keyword_ui(s),
            Keyword::ProjectPath | Keyword::LibraryPath => {}
            Keyword::Global | Keyword::Local => {}
            _ => return Ok(KeywordResult::None),
        }
        Ok(KeywordResult::Continue)
    }

    fn set_visibility(&mut self, s: &[char], keyword: Keyword, value: bool) {
        self.visibility_stack.push(self.is_visible);
        if self.is_directive_condition_met(s, keyword) {
            self.is_visible = value;
        }
    }

    fn set_parse_mode(&mut self, s: &[char], keyword: Keyword, mode: ParseMode) {
        self.flush_markdown();
        self.parse_mode_stack.push((self.parse_mode, self.mode_suppressed));
        self.parse_mode = mode;
        let kl = keyword_length(keyword);
        let has_condition = s.len() > kl && !s[kl..].iter().all(|c| c.is_whitespace());
        if has_condition
            && self.calculate
            && self.condition.is_satisfied()
            && !self.is_directive_condition_met(s, keyword)
        {
            self.mode_suppressed = true;
        }
    }

    fn set_output_mode(&mut self, s: &[char], keyword: Keyword, value: i32) {
        self.output_mode_stack.push(self.is_val);
        if self.is_directive_condition_met(s, keyword) {
            self.is_val = value;
        }
    }

    fn set_substitution(&mut self, s: &[char], keyword: Keyword, value: VariableSubstitutionOptions) {
        self.substitution_stack.push(self.p().variable_substitution.get());
        if self.is_directive_condition_met(s, keyword) {
            self.p().variable_substitution.set(value);
        }
    }

    fn is_directive_condition_met(&mut self, s: &[char], keyword: Keyword) -> bool {
        if !self.condition.is_satisfied() {
            return false;
        }
        let kl = keyword_length(keyword);
        if s.len() <= kl {
            return true;
        }
        let rest: String = s[kl..].iter().collect();
        let expr = trim(&rest).to_string();
        if expr.chars().all(char::is_whitespace) {
            return true;
        }
        if !self.calculate {
            return false;
        }
        let r = (|| -> CResult<bool> {
            self.p().parse(&expr, true)?;
            self.p().calculate(true, -1)?;
            Condition::is_true(self.p().result_complex())
        })();
        match r {
            Ok(b) => b,
            Err(ex) => {
                let line: String = s.iter().collect();
                self.append_error(&line, &ex.message);
                false
            }
        }
    }

    fn parse_keyword_input(&mut self) -> KeywordResult {
        if self.condition.is_satisfied() {
            self.previous_keyword = Keyword::Input;
            if self.calculate {
                self.start_line = self.cur() + 1;
                self.pause_char_count = self.sb.len();
                self.calculate = false;
                return KeywordResult::Continue;
            }
            return KeywordResult::Break;
        }
        if self.calculate {
            KeywordResult::Continue
        } else {
            KeywordResult::Break
        }
    }

    fn parse_keyword_pause(&mut self) -> KeywordResult {
        if self.condition.is_satisfied() && (self.calculate || self.start_line > 0) {
            if self.calculate {
                if self.is_paused_by_user {
                    self.start_line = self.cur();
                } else {
                    self.start_line = self.cur() + 1;
                }
            }
            if self.previous_keyword != Keyword::Input {
                self.pause_char_count = self.sb.len();
            }
            self.previous_keyword = Keyword::Pause;
            self.is_paused_by_user = false;
            return KeywordResult::Break;
        }
        if self.is_visible && !self.calculate {
            let id = self.html_id();
            self.sb.push_str(&format!("<p{} class=\"cond\">#pause</p>", id));
        }
        KeywordResult::Continue
    }

    fn keyword_expression(s: &[char], keyword: Keyword) -> String {
        let kl = keyword_length(keyword);
        if s.len() > kl {
            let rest: String = s[kl..].iter().collect();
            trim(&rest).to_string()
        } else {
            String::new()
        }
    }

    fn parse_keyword_round(&mut self, s: &[char], line: &str) -> CResult<()> {
        let kl = keyword_length(Keyword::Round);
        if s.len() > kl {
            let expr = ExpressionParser::keyword_expression(s, Keyword::Round);
            if expr == "default" {
                self.math_settings.borrow_mut().set_decimals(self.decimals);
            } else if let Some(n) = parse_int(&expr) {
                self.math_settings.borrow_mut().set_decimals(n);
            } else {
                let r = (|| -> CResult<f64> {
                    self.p().parse(&expr, true)?;
                    self.p().calculate(true, -1)?;
                    Ok(self.p().real())
                })();
                match r {
                    Ok(v) => {
                        let n = v.round();
                        // (int) cast of a double
                        let n = if n.is_nan() {
                            i32::MIN
                        } else if n >= i32::MAX as f64 {
                            i32::MAX
                        } else if n <= i32::MIN as f64 {
                            i32::MIN
                        } else {
                            n as i32
                        };
                        self.math_settings.borrow_mut().set_decimals(n);
                    }
                    Err(ex) => {
                        if ex.unexpected {
                            return Err(ex);
                        }
                        self.append_error(line, &ex.message)
                    }
                }
            }
        } else {
            self.math_settings.borrow_mut().set_decimals(self.decimals);
        }
        Ok(())
    }

    fn parse_keyword_format(&mut self, s: &[char]) {
        let kl = keyword_length(Keyword::Format);
        if s.len() > kl {
            let expr = ExpressionParser::keyword_expression(s, Keyword::Format);
            if expr == "default" {
                self.math_settings.borrow_mut().format_string = None;
            } else if validator::is_valid_format_string(&expr) {
                self.math_settings.borrow_mut().format_string = Some(expr);
            } else {
                self.append_error(&format!("#format {}", expr), messages::Invalid_format_string_0);
            }
        } else {
            self.math_settings.borrow_mut().format_string = None;
        }
    }

    fn parse_keyword_repeat(&mut self, s: &[char], line: &str) -> CResult<()> {
        let expression = ExpressionParser::keyword_expression(s, Keyword::Repeat);
        if self.calculate {
            if self.condition.is_satisfied() {
                let mut count = 0.0;
                if !expression.chars().all(char::is_whitespace) {
                    let r = (|| -> CResult<f64> {
                        self.p().parse(&expression, true)?;
                        self.p().calculate(true, -1)?;
                        Ok(self.p().real())
                    })();
                    match r {
                        Ok(v) => {
                            if v > MAX_COUNT as f64 {
                                let msg = fmt(
                                    messages::Number_of_iterations_exceeds_the_maximum_0,
                                    &[&MAX_COUNT.to_string()],
                                );
                                self.append_error(line, &msg);
                            } else {
                                count = v.round();
                            }
                        }
                        Err(ex) => {
                            if ex.unexpected {
                                return Err(ex);
                            }
                            self.append_error(line, &ex.message);
                        }
                    }
                } else {
                    count = -1.0;
                }
                let id = self.condition.id;
                self.loops.push(Loop::repeat(self.cur(), count, id));
            }
        } else if self.is_visible {
            let id = self.html_id();
            if expression.chars().all(char::is_whitespace) {
                self.sb
                    .push_str(&format!("<p{} class=\"cond\">#repeat</p><div class=\"indent\">", id));
            } else {
                let r = (|| -> CResult<String> {
                    self.p().parse(&expression, true)?;
                    self.p().to_html()
                })();
                match r {
                    Ok(h) => self.sb.push_str(&format!("<p{}><span class=\"cond\">#repeat</span> <span class=\"eq\">{}</span></p><div class=\"indent\">", id, h)),
                    Err(ex) => {
                        if ex.unexpected {
                            return Err(ex);
                        }
                        self.append_error(line, &ex.message)
                    }
                }
            }
        }
        Ok(())
    }

    fn get_for_loop_limits(expression: &[char]) -> (isize, isize) {
        let (mut start, mut end) = (-1isize, -1isize);
        let (mut n1, mut n2, mut n3) = (0, 0, 0);
        for (i, &c) in expression.iter().enumerate() {
            match c {
                '=' => start = i as isize,
                ':' if n1 == 0 && n2 == 0 && n3 == 0 => {
                    end = i as isize;
                    return (start, end);
                }
                '(' => n1 += 1,
                ')' => n1 -= 1,
                '{' => n2 += 1,
                '}' => n2 -= 1,
                '[' => n3 += 1,
                ']' => n3 -= 1,
                _ => {}
            }
        }
        (start, end)
    }

    fn parse_keyword_for(&mut self, s: &[char], line: &str) -> CResult<()> {
        let expression = ExpressionParser::keyword_expression(s, Keyword::For);
        if expression.chars().all(char::is_whitespace) {
            return Err(crate::exceptions::ExpressionEmpty());
        }
        let ex: Vec<char> = expression.chars().collect();
        let (loop_start, loop_end) = ExpressionParser::get_for_loop_limits(&ex);
        if loop_start > -1 && loop_end > loop_start {
            let (ls, le) = (loop_start as usize, loop_end as usize);
            let var_name = trim(&ex[..ls].iter().collect::<String>()).to_string();
            let start_expr = trim(&ex[ls + 1..le].iter().collect::<String>()).to_string();
            let end_expr = trim(&ex[le + 1..].iter().collect::<String>()).to_string();
            if validator::is_variable(&var_name) {
                if self.calculate {
                    if self.condition.is_satisfied() {
                        let r = (|| -> CResult<Option<Loop>> {
                            let p = self.p();
                            p.parse(&start_expr, true)?;
                            p.calculate(true, -1)?;
                            let r1 = p.result_complex();
                            let u1 = p.units();
                            p.parse(&end_expr, true)?;
                            p.calculate(true, -1)?;
                            let r2 = p.result_complex();
                            let u2 = p.units();
                            let (start, end) = if r1.is_real() && r2.is_real() {
                                (
                                    Value::Real(RealValue::new(r1.re(), u1)),
                                    Value::Real(RealValue::new(r2.re(), u2)),
                                )
                            } else {
                                (
                                    Value::Complex(crate::base_types::complex_value::ComplexValue::from_c(r1, u1)),
                                    Value::Complex(crate::base_types::complex_value::ComplexValue::from_c(r2, u2)),
                                )
                            };
                            let count = Value::s_sub(&end, &start)?.re().abs() + 1.0;
                            if count > MAX_COUNT as f64 {
                                return Ok(None);
                            }
                            let counter = p.get_variable_ref(&var_name);
                            let lp = Loop::for_loop(self.cur(), &start, &end, counter, self.condition.id)?;
                            p.set_variable(&var_name, start);
                            Ok(Some(lp))
                        })();
                        match r {
                            Ok(Some(lp)) => self.loops.push(lp),
                            Ok(None) => {
                                let msg = fmt(
                                    messages::Number_of_iterations_exceeds_the_maximum_0,
                                    &[&MAX_COUNT.to_string()],
                                );
                                self.append_error(line, &msg);
                            }
                            Err(ex) => {
                                if ex.unexpected {
                                    return Err(ex);
                                }
                                self.append_error(line, &ex.message)
                            }
                        }
                    }
                } else if self.is_visible {
                    let id = self.html_id();
                    let r = (|| -> CResult<(String, String, String)> {
                        let var_html =
                            HtmlWriter::new(None, self.p().phasor.get()).format_variable(&var_name, "", false);
                        self.p().parse(&start_expr, true)?;
                        let sh = self.p().to_html()?;
                        self.p().parse(&end_expr, true)?;
                        let eh = self.p().to_html()?;
                        Ok((var_html, sh, eh))
                    })();
                    match r {
                        Ok((v, sh, eh)) => self.sb.push_str(&format!("<p{}><span class=\"cond\">#for</span> <span class=\"eq\">{} = {} : {}</span></p><div class=\"indent\">", id, v, sh, eh)),
                        Err(ex) => {
                            if ex.unexpected {
                                return Err(ex);
                            }
                            self.append_error(line, &ex.message)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_keyword_while(&mut self, s: &[char], line: &str) -> CResult<()> {
        let expression = ExpressionParser::keyword_expression(s, Keyword::While);
        if expression.chars().all(char::is_whitespace) {
            return Err(crate::exceptions::ExpressionEmpty());
        }
        if self.calculate {
            if self.condition.is_satisfied() {
                let comment_start = expression.find('\'');
                let condition = match comment_start {
                    None => expression.clone(),
                    Some(c) => expression[..c].to_string(),
                };
                let r = (|| -> CResult<()> {
                    self.p().parse(&condition, true)?;
                    self.p().calculate(true, -1)?;
                    self.condition
                        .set_condition(Keyword::While as i32 - Keyword::If as i32)?;
                    let rc = self.p().result_complex();
                    self.condition.check(rc)?;
                    if self.condition.is_satisfied() {
                        let id = self.condition.id;
                        self.loops.push(Loop::while_loop(self.cur(), expression.clone(), id));
                        if let Some(c) = comment_start {
                            let toks = Rc::new(RefCell::new(self.get_tokens(&expression[c..])));
                            self.parse_tokens(&toks, false)?;
                        }
                    }
                    Ok(())
                })();
                if let Err(ex) = r {
                    if ex.unexpected {
                        return Err(ex);
                    }
                    self.append_error(line, &ex.message);
                }
            }
        } else if self.is_visible {
            let id = self.html_id();
            self.sb
                .push_str(&format!("<p{}><span class=\"cond\">#while</span> ", id));
            let toks = Rc::new(RefCell::new(self.get_tokens(&expression)));
            if let Err(ex) = self.parse_tokens(&toks, true) {
                if ex.unexpected {
                    return Err(ex);
                }
                self.append_error(line, &ex.message);
            }
            self.sb.push_str("</p><div class=\"indent\">");
        }
        Ok(())
    }

    fn parse_keyword_loop(&mut self, line: &str) -> CResult<()> {
        if self.calculate {
            if self.condition.is_satisfied() {
                if self.loops.is_empty() {
                    self.append_error(line, messages::loop_without_a_corresponding_repeat);
                } else {
                    let next_id = self.loops.last().unwrap().id;
                    if next_id != self.condition.id {
                        self.append_error(line, messages::Entangled_if__end_if__and_repeat__loop_blocks);
                    } else if !self.iterate(true)? {
                        self.loops.pop();
                    }
                }
            } else if self.condition.is_loop() {
                self.condition.set_condition(REMOVE_CONDITION_KEYWORD)?;
            }
        } else if self.is_visible {
            let id = self.html_id();
            self.sb.push_str(&format!("</div><p{} class=\"cond\">#loop</p>", id));
        }
        Ok(())
    }

    /// `Iterate(loop, removeWhileCondition)` on the innermost loop.
    fn iterate(&mut self, remove_while_condition: bool) -> CResult<bool> {
        let n = self.loops.len() - 1;
        let condition_text = match &self.loops[n].kind {
            LoopKind::While { condition } => Some(condition.clone()),
            _ => None,
        };
        if condition_text.is_none() {
            self.loops[n].increment_counter();
        }
        if let Some(expression) = condition_text {
            let comment_start = expression.find(['\'', '"']).unwrap_or(expression.len());
            let condition = &expression[..comment_start];
            self.p().parse(condition, true)?;
            self.p().calculate(true, -1)?;
            let rc = self.p().result_complex();
            self.condition.check(rc)?;
            if self.condition.is_satisfied() {
                if comment_start + 1 < expression.len() {
                    let toks = Rc::new(RefCell::new(self.get_tokens(&expression[comment_start..])));
                    self.parse_tokens(&toks, false)?;
                }
            } else {
                if remove_while_condition {
                    self.condition.set_condition(REMOVE_CONDITION_KEYWORD)?;
                }
                self.loops[n].break_loop();
            }
        }
        let mut cl = self.cur();
        if self.loops[n].iterate(&mut cl) {
            self.current_line = cl as isize;
            self.p().reset_stack();
            return Ok(true);
        }
        Ok(false)
    }

    fn parse_keyword_break(&mut self) -> bool {
        if self.calculate {
            if self.condition.is_satisfied() {
                if let Some(l) = self.loops.last_mut() {
                    l.break_loop();
                } else {
                    return true;
                }
            }
        } else if self.is_visible {
            let id = self.html_id();
            self.sb.push_str(&format!("<p{} class=\"cond\">#break</p>", id));
        }
        false
    }

    fn parse_keyword_continue(&mut self) -> CResult<()> {
        if self.calculate {
            if self.condition.is_satisfied() {
                if self.loops.is_empty() {
                    self.append_error("#continue", messages::continue_without_a_corresponding_repeat);
                } else {
                    let loop_id = self.loops.last().unwrap().id;
                    if self.iterate(false)? {
                        while self.condition.id > loop_id {
                            self.condition.set_condition(REMOVE_CONDITION_KEYWORD)?;
                        }
                    } else {
                        self.loops.last_mut().unwrap().break_loop();
                    }
                }
            }
        } else if self.is_visible {
            let id = self.html_id();
            self.sb.push_str(&format!("<p{} class=\"cond\">#continue</p>", id));
        }
        Ok(())
    }

    fn parse_keyword_settings(&mut self, s: &str) {
        let rest = s.chars().skip("#settings".len()).collect::<String>();
        let json = trim(&rest).to_string();
        let invalid = || fmt(messages::Invalid_JSON_in_0, &["#settings"]);
        if json.is_empty() {
            let m = invalid();
            self.append_error(s, &m);
            return;
        }
        match parse_flat_json(&json) {
            None => {
                let m = invalid();
                self.append_error(s, &m);
            }
            Some(pairs) => {
                for (k, v) in pairs {
                    self.apply_setting(&k.to_lowercase(), &v, s);
                }
            }
        }
    }

    fn apply_setting(&mut self, key: &str, v: &JsonValue, line: &str) {
        let range_err = |name: &str, min: f64, max: Option<f64>| match max {
            Some(mx) => format!(
                "'{}' must be between {} and {}",
                name,
                netfmt::double_to_string(min),
                netfmt::double_to_string(mx)
            ),
            None => format!("'{}' must be at least {}", name, netfmt::double_to_string(min)),
        };
        match (key, v) {
            ("decimals", JsonValue::Num(d)) => {
                if *d < 0.0 || *d > 15.0 {
                    let m = range_err("decimals", 0.0, Some(15.0));
                    self.append_error(line, &m);
                } else {
                    self.math_settings.borrow_mut().set_decimals(*d as i32);
                }
            }
            ("degrees", JsonValue::Num(d)) => {
                if *d < 0.0 || *d > 2.0 {
                    let m = range_err("degrees", 0.0, Some(2.0));
                    self.append_error(line, &m);
                } else {
                    self.math_settings.borrow_mut().degrees = *d as i32;
                    self.p().set_degrees(*d as i32);
                }
            }
            ("complex", JsonValue::Bool(b)) => self.p().set_complex(*b),
            ("substitute", JsonValue::Bool(b)) => self.math_settings.borrow_mut().substitute = *b,
            ("formatequations", JsonValue::Bool(b)) => self.math_settings.borrow_mut().format_equations = *b,
            ("zerosmallmatrixelements", JsonValue::Bool(b)) => {
                self.math_settings.borrow_mut().zero_small_matrix_elements = *b
            }
            ("showhiddenoutput", JsonValue::Bool(b)) => self.math_settings.borrow_mut().show_hidden_output = *b,
            ("valmathml", JsonValue::Bool(b)) => self.math_settings.borrow_mut().val_mathml = *b,
            ("unitperelement", JsonValue::Bool(b)) => self.math_settings.borrow_mut().unit_per_element = *b,
            ("maxoutputcount", JsonValue::Num(d)) => self.math_settings.borrow_mut().set_max_output_count(*d as i32),
            ("isus", JsonValue::Bool(b)) => {
                self.is_us = *b;
                self.p().is_us.set(*b);
            }
            ("precision", JsonValue::Num(d)) => {
                self.math_settings.borrow_mut().precision = *d;
                self.p().set_variable_f("Precision", *d);
            }
            ("vectorgraphics", JsonValue::Bool(b)) => {
                self.plot_settings.vector_graphics = *b;
                self.p().set_variable_f("PlotSVG", if *b { 1.0 } else { 0.0 });
            }
            ("colorscale", JsonValue::Str(name)) => match crate::settings::ColorScales::parse(name) {
                Some(cs) => {
                    self.plot_settings.color_scale = cs;
                    self.p().set_variable_f("PlotPalette", cs as i32 as f64);
                }
                None => {
                    let m = format!(
                        "'colorScale' must be one of: {}",
                        crate::settings::ColorScales::names().join(", ")
                    );
                    self.append_error(line, &m);
                }
            },
            ("smoothscale", JsonValue::Bool(b)) => {
                self.plot_settings.smooth_scale = *b;
                self.p().set_variable_f("PlotSmooth", if *b { 1.0 } else { 0.0 });
            }
            ("shadows", JsonValue::Bool(b)) => {
                self.plot_settings.set_shadows(*b);
                self.p().set_variable_f("PlotShadows", if *b { 1.0 } else { 0.0 });
            }
            ("adaptiveplot", JsonValue::Bool(b)) => {
                self.plot_settings.is_adaptive = *b;
                self.p().set_variable_f("PlotAdaptive", if *b { 1.0 } else { 0.0 });
            }
            ("plotwidth", JsonValue::Num(d)) => {
                if *d < 1.0 {
                    let m = range_err("plotWidth", 1.0, None);
                    self.append_error(line, &m);
                } else {
                    self.plot_settings.width = *d as i32;
                    self.p().set_variable_f("PlotWidth", (*d as i32) as f64);
                }
            }
            ("plotheight", JsonValue::Num(d)) => {
                if *d < 1.0 {
                    let m = range_err("plotHeight", 1.0, None);
                    self.append_error(line, &m);
                } else {
                    self.plot_settings.height = *d as i32;
                    self.p().set_variable_f("PlotHeight", (*d as i32) as f64);
                }
            }
            ("plotstep", JsonValue::Num(d)) => {
                if *d < 0.0 {
                    let m = range_err("plotStep", 0.0, None);
                    self.append_error(line, &m);
                } else {
                    self.plot_settings.step = *d as i32;
                    self.p().set_variable_f("PlotStep", (*d as i32) as f64);
                }
            }
            ("tol", JsonValue::Num(d)) => {
                self.math_settings.borrow_mut().tol = *d;
                self.p().set_variable_f("Tol", *d);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum JsonValue {
    Num(f64),
    Bool(bool),
    Str(String),
    Null,
}

/// Minimal parser for the flat JSON object of `#settings {...}`.
fn parse_flat_json(s: &str) -> Option<Vec<(String, JsonValue)>> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let skip_ws = |i: &mut usize| {
        while *i < chars.len() && chars[*i].is_whitespace() {
            *i += 1;
        }
    };
    let read_string = |i: &mut usize| -> Option<String> {
        if chars.get(*i) != Some(&'"') {
            return None;
        }
        *i += 1;
        let mut out = String::new();
        while *i < chars.len() && chars[*i] != '"' {
            if chars[*i] == '\\' {
                *i += 1;
            }
            out.push(*chars.get(*i)?);
            *i += 1;
        }
        *i += 1;
        Some(out)
    };
    skip_ws(&mut i);
    if chars.get(i) != Some(&'{') {
        return None;
    }
    i += 1;
    let mut out = Vec::new();
    loop {
        skip_ws(&mut i);
        if chars.get(i) == Some(&'}') {
            break;
        }
        let key = read_string(&mut i)?;
        skip_ws(&mut i);
        if chars.get(i) != Some(&':') {
            return None;
        }
        i += 1;
        skip_ws(&mut i);
        let v = match chars.get(i)? {
            '"' => JsonValue::Str(read_string(&mut i)?),
            't' if chars[i..].iter().take(4).collect::<String>() == "true" => {
                i += 4;
                JsonValue::Bool(true)
            }
            'f' if chars[i..].iter().take(5).collect::<String>() == "false" => {
                i += 5;
                JsonValue::Bool(false)
            }
            'n' if chars[i..].iter().take(4).collect::<String>() == "null" => {
                i += 4;
                JsonValue::Null
            }
            _ => {
                let st = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || "+-.eE".contains(chars[i])) {
                    i += 1;
                }
                let num: String = chars[st..i].iter().collect();
                JsonValue::Num(num.parse().ok()?)
            }
        };
        out.push((key, v));
        skip_ws(&mut i);
        match chars.get(i) {
            Some(',') => i += 1,
            Some('}') => break,
            _ => return None,
        }
    }
    let _ = JsonValue::Null;
    Some(out)
}

/// `int.TryParse(s)` (current culture).
fn parse_int(s: &str) -> Option<i32> {
    let t = trim(s);
    if t.is_empty() {
        return None;
    }
    let (neg, digits) = match t.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let v: i64 = digits.parse().ok()?;
    let v = if neg { -v } else { v };
    if v > i32::MAX as i64 || v < i32::MIN as i64 {
        return None;
    }
    Some(v as i32)
}

/// `InsertAttribute(s, attr)` (only used with debug ids).
/// `ParseMarkdown(tokens)`: renders the comment tokens of a line as markdown (`#md on`).
/// The text tokens are joined with a record separator, rendered together and split back.
fn parse_markdown(tokens: &mut Vec<Token>) {
    if tokens.is_empty() {
        return;
    }
    const RS: char = '\u{001E}';
    let mut sb = String::new();
    let starts_with_expression = tokens[0].ttype == TokenTypes::Expression;
    if starts_with_expression {
        sb.push(RS);
    }
    let n = tokens.len();
    for token in tokens.iter() {
        if token.ttype != TokenTypes::Expression {
            if n == 1 {
                sb.push_str(token.value.trim_end());
            } else {
                sb.push_str(&token.value);
                sb.push(RS);
            }
        }
    }
    let doc = markdig::parse(&sb, &markdig::Pipeline::comments());
    let result = markdig::render(&doc, true);
    // SplitEnumerator: Current starts empty, MoveNext fails once the rest is empty.
    let mut rest: &str = &result;
    let mut move_next = |current: &mut String| -> bool {
        if rest.is_empty() {
            return false;
        }
        match rest.find(RS) {
            None => {
                *current = rest.to_string();
                rest = "";
            }
            Some(i) => {
                *current = rest[..i].to_string();
                rest = &rest[i + RS.len_utf8()..];
            }
        }
        true
    };
    let mut cs = String::new();
    if starts_with_expression {
        // `Current` is still empty here, so CalcpadCE always skips the first section.
        move_next(&mut cs);
    }
    let mut i = 0;
    while i < n {
        let mut t = tokens[i].ttype;
        if t != TokenTypes::Expression {
            if !move_next(&mut cs) {
                break;
            }
            if tokens[i].value.starts_with('#') {
                t = TokenTypes::Html;
            }
            tokens[i] = Token {
                value: cs.clone(),
                ttype: t,
                cache_id: -1,
            };
        }
        i += 1;
    }
    while move_next(&mut cs) {
        if !cs.is_empty() {
            tokens.push(Token {
                value: cs.clone(),
                ttype: TokenTypes::Html,
                cache_id: -1,
            });
        }
    }
}

pub fn insert_attribute(s: &str, attr: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > 2 && chars[0] == '<' && chars[1].is_alphabetic() {
        if let Some(i) = tag_end_index(s) {
            let mut i = i;
            if i > 1 {
                let bytes = s.as_bytes();
                let mut j = i;
                while j > 1 {
                    j -= 1;
                    if bytes[j] != b' ' {
                        if bytes[j] == b'/' {
                            i = j;
                        }
                        break;
                    }
                }
                return format!("{}{}{}", &s[..i], attr, &s[i..]);
            }
        }
    }
    s.to_string()
}
