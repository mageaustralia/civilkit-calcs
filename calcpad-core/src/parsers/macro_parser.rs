//! Port of Parsers/MacroParser.cs (#def macros, #include, input field values).
//!
//! The core does no file I/O: `#include` goes through the [`FileProvider`] trait.
use std::collections::{HashMap, HashSet, VecDeque};

use crate::exceptions;
use crate::extension_methods::{enumerate_comments, enumerate_lines, trim, trim_start};
use crate::math_parser_exception::{message_codes, CResult, MathParserException};
use crate::messages;
use crate::netfmt::{fmt, html_encode};
use crate::parsers::math_parser::keyword_args::{fill_defaults, index_of_top_level};
use crate::validator;

/// File access used by `#include`.
pub trait FileProvider {
    /// `Path.GetFullPath(path, baseDir)` (or relative to the current directory).
    fn full_path(&self, path: &str, base_dir: Option<&str>) -> String;
    fn exists(&self, path: &str) -> bool;
    /// `Path.GetDirectoryName(path)`
    fn directory_name(&self, path: &str) -> Option<String>;
    /// The `Include` delegate (CalcpadReader.Include in the CLI): file contents with
    /// `#local` sections removed and nested includes expanded.
    fn include(&self, path: &str, fields: Option<VecDeque<String>>) -> CResult<String>;
    /// `Environment.ExpandEnvironmentVariables`
    fn expand_environment_variables(&self, s: &str) -> String {
        s.to_string()
    }
}

#[derive(Clone)]
struct Macro {
    contents: Option<String>,
    parameters: Option<Vec<String>>,
    defaults: Option<Vec<Option<String>>>,
    order: Option<Vec<usize>>,
    has_defaults: bool,
}

impl Macro {
    fn new(
        contents: Option<String>,
        parameters: Option<Vec<String>>,
        defaults: Option<Vec<Option<String>>>,
    ) -> CResult<Macro> {
        match parameters {
            None => Ok(Macro {
                contents,
                parameters: None,
                defaults: None,
                order: None,
                has_defaults: false,
            }),
            Some(p) => {
                let order = Macro::sort(&p)?;
                let d = defaults.unwrap_or_default();
                let has_defaults = d.iter().any(|x| x.is_some());
                Ok(Macro {
                    contents,
                    parameters: Some(p),
                    defaults: Some(d),
                    order: Some(order),
                    has_defaults,
                })
            }
        }
    }

    fn empty() -> Macro {
        Macro {
            contents: None,
            parameters: None,
            defaults: None,
            order: None,
            has_defaults: false,
        }
    }

    /// Orders parameters so that longer names that end with a shorter one are replaced first.
    fn sort(parameters: &[String]) -> CResult<Vec<usize>> {
        let mut sorted: Vec<(String, usize)> = Vec::new();
        for (i, p) in parameters.iter().enumerate() {
            let s: String = p.chars().rev().collect();
            if sorted.iter().any(|(k, _)| *k == s) {
                return Err(exceptions::DuplicateMacroParameters(&s));
            }
            sorted.push((s, i));
        }
        // SortedList<string,int> uses the current culture comparer.
        sorted.sort_by(|a, b| culture_compare(&a.0, &b.0));
        Ok(sorted.into_iter().rev().map(|(_, i)| i).collect())
    }

    fn get_parameter_index(&self, name: &str) -> Option<usize> {
        self.parameters.as_ref()?.iter().position(|p| p == name)
    }

    fn parameter_count(&self) -> usize {
        self.parameters.as_ref().map(|p| p.len()).unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        self.contents.is_none()
    }

    fn resolve_arguments(&self, positional: &[String], keywords: &[(String, String)]) -> CResult<Vec<String>> {
        if positional.len() > self.parameter_count() {
            return Err(exceptions::InvalidNumberOfArguments());
        }
        let mut result: Vec<Option<String>> = vec![None; self.parameter_count()];
        for (i, p) in positional.iter().enumerate() {
            result[i] = Some(p.clone());
        }
        for (k, v) in keywords {
            let j = match self.get_parameter_index(k) {
                None => return Err(exceptions::UnknownKeywordArgument(k)),
                Some(j) => j,
            };
            if result[j].is_some() {
                return Err(exceptions::DuplicateArgument(k));
            }
            result[j] = Some(v.clone());
        }
        fill_defaults(&mut result, self.defaults.as_deref().unwrap_or(&[]))?;
        Ok(result.into_iter().map(|x| x.unwrap()).collect())
    }

    fn run(&self, arguments: &[String]) -> String {
        let contents = self.contents.clone().unwrap_or_default();
        if self.parameter_count() == 0 {
            return contents;
        }
        let mut sb = contents;
        let params = self.parameters.as_ref().unwrap();
        for &j in self.order.as_ref().unwrap() {
            let s = &arguments[j];
            if s.chars().count() > 1 && s.starts_with(' ') {
                sb = sb.replace(&params[j], &s[1..]);
            } else {
                sb = sb.replace(&params[j], s);
            }
        }
        sb
    }
}

/// Culture-aware (en-US) string comparison approximation used by SortedList.
fn culture_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let la = a.to_lowercase();
    let lb = b.to_lowercase();
    match la.cmp(&lb) {
        std::cmp::Ordering::Equal => b.cmp(a),
        o => o,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Keywords {
    None,
    Def,
    EndDef,
    Include,
    ProjectPath,
    LibraryPath,
}

const DEF_KEYWORD: &str = "#def";
const END_DEF_KEYWORD: &str = "#end def";
const INCLUDE_KEYWORD: &str = "#include";
const FIELDS_MARKER: &str = "#{";

fn get_keyword(s: &str) -> Keywords {
    if s.chars().count() < DEF_KEYWORD.len() {
        return Keywords::None;
    }
    if validator::starts_with_ignore_case(s, DEF_KEYWORD) {
        return Keywords::Def;
    }
    if validator::starts_with_ignore_case(s, END_DEF_KEYWORD) {
        return Keywords::EndDef;
    }
    if validator::starts_with_ignore_case(s, INCLUDE_KEYWORD) {
        return Keywords::Include;
    }
    if validator::starts_with_ignore_case(s, "#projectpath") {
        return Keywords::ProjectPath;
    }
    if validator::starts_with_ignore_case(s, "#librarypath") {
        return Keywords::LibraryPath;
    }
    Keywords::None
}

/// `TryGetIncludePath`: (start, length) in chars.
pub fn try_get_include_path(line: &[char]) -> Option<(usize, usize)> {
    let path_start = INCLUDE_KEYWORD.len();
    let mut n: isize = line
        .iter()
        .position(|&c| c == '\'' || c == '"')
        .map(|x| x as isize)
        .unwrap_or(-1);
    let nf1: isize = line.iter().rposition(|&c| c == '#').map(|x| x as isize).unwrap_or(-1);
    if n <= path_start as isize || nf1 > 0 && nf1 < n {
        n = nf1;
    }
    if n <= path_start as isize {
        n = line.len() as isize;
    }
    let name: String = line[path_start..n as usize].iter().collect();
    let start = path_start + name.chars().count() - trim_start(&name).chars().count();
    let length = trim(&name).chars().count();
    if length > 0 {
        Some((start, length))
    } else {
        None
    }
}

/// `MacroParser.GetFields`
pub fn get_fields(s: &str, delimiter: char) -> VecDeque<String> {
    let mut fields = VecDeque::new();
    if s.is_empty() {
        return fields;
    }
    for item in s.split(delimiter) {
        fields.push_back(trim(item).to_string());
    }
    // SplitEnumerator does not yield a trailing empty item.
    if s.ends_with(delimiter) {
        fields.pop_back();
    }
    fields
}

pub struct MacroParser<'a> {
    files: &'a dyn FileProvider,
    pub source_file_path: Option<String>,
    macros: HashMap<String, Macro>,
    include_stack: HashSet<String>,
    parsed_line_number: i32,
    line_numbers: Vec<i32>,
    /// `Errors`
    pub errors: Vec<crate::calcpad_error::CalcpadError>,
}

struct ParseState {
    sb: String,
    macro_builder: String,
    macro_name: String,
    line_number: i32,
    macro_def_count: i32,
    has_errors: bool,
    macro_parameters: Option<Vec<String>>,
    macro_defaults: Option<Vec<Option<String>>>,
}

impl<'a> MacroParser<'a> {
    pub fn new(files: &'a dyn FileProvider) -> MacroParser<'a> {
        MacroParser {
            files,
            source_file_path: None,
            macros: HashMap::new(),
            include_stack: HashSet::new(),
            parsed_line_number: 0,
            line_numbers: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// `Parse(sourceCode, out outCode, sb, includeLine, addLineNumbers)`; returns (hasErrors, outCode).
    pub fn parse(&mut self, source_code: &str, add_line_numbers: bool) -> (bool, String) {
        self.macros.clear();
        self.line_numbers.clear();
        self.include_stack.clear();
        self.parsed_line_number = 0;
        self.errors.clear();
        let mut sb = String::with_capacity(source_code.len());
        let has_errors = self.parse_inner(source_code, &mut sb, 0, add_line_numbers);
        (has_errors, sb)
    }

    /// One (possibly nested) Parse call: its own locals, the shared output builder.
    fn parse_inner(&mut self, source_code: &str, sb: &mut String, include_line: i32, add_line_numbers: bool) -> bool {
        let mut st = ParseState {
            sb: std::mem::take(sb),
            macro_builder: String::new(),
            macro_name: String::new(),
            line_number: include_line,
            macro_def_count: 0,
            has_errors: false,
            macro_parameters: None,
            macro_defaults: None,
        };
        let mut line_content_for_error = "code".to_string();
        let r = (|| -> CResult<()> {
            for source_line in enumerate_lines(source_code) {
                if include_line == 0 {
                    self.line_numbers.push(self.parsed_line_number);
                    st.line_number += 1;
                }
                let line_content = trim_start(source_line);
                line_content_for_error = line_content.to_string();
                if line_content.is_empty() {
                    self.append_line(&mut st, source_line, add_line_numbers);
                    continue;
                }
                if line_content.starts_with('#')
                    && self.parse_keyword(&mut st, line_content, source_line, add_line_numbers)?
                {
                    continue;
                }
                if st.macro_def_count == 1 {
                    st.macro_builder.push_str(source_line);
                    st.macro_builder.push('\n');
                    continue;
                }
                if !self.macros.is_empty() {
                    match self.apply_macros(source_line, None) {
                        Ok(insert_code) => {
                            for line in enumerate_lines(&insert_code) {
                                self.append_line(&mut st, line, add_line_numbers);
                            }
                        }
                        Err(ex) => self.append_error(&mut st, line_content, &ex.message, add_line_numbers),
                    }
                    continue;
                }
                self.append_line(&mut st, source_line, add_line_numbers);
            }
            if include_line == 0 {
                self.line_numbers.push(self.parsed_line_number);
            }
            if st.macro_def_count > 0 {
                st.sb
                    .push_str(messages::Macro_definition_block_not_closed_Missing_end_def);
                st.has_errors = true;
            }
            Ok(())
        })();
        if let Err(ex) = r {
            let lc = line_content_for_error.clone();
            self.append_error(&mut st, &lc, &ex.message, add_line_numbers);
        }
        *sb = st.sb;
        st.has_errors
    }

    fn parse_keyword(
        &mut self,
        st: &mut ParseState,
        line_content: &str,
        source_line: &str,
        add_line_numbers: bool,
    ) -> CResult<bool> {
        match get_keyword(line_content) {
            Keywords::Include => {
                self.parse_include(st, line_content, add_line_numbers)?;
                Ok(true)
            }
            Keywords::Def => {
                self.parse_def(st, line_content, add_line_numbers)?;
                Ok(true)
            }
            Keywords::EndDef => {
                self.parse_end_def(st, line_content, add_line_numbers)?;
                Ok(true)
            }
            Keywords::ProjectPath | Keywords::LibraryPath => {
                // Path roots are kept as lines; resolving them needs a FileProvider with
                // directory support, which the CLI does not use for the conformance corpus.
                self.append_line(st, source_line, add_line_numbers);
                Ok(true)
            }
            Keywords::None => Ok(false),
        }
    }

    fn parse_include(&mut self, st: &mut ParseState, line_content: &str, add_line_numbers: bool) -> CResult<()> {
        let chars: Vec<char> = line_content.chars().collect();
        if chars.len() <= INCLUDE_KEYWORD.len() {
            self.append_error(
                st,
                line_content,
                messages::Missing_source_file_for_include,
                add_line_numbers,
            );
        }
        let (start, length) = try_get_include_path(&chars).unwrap_or((0, 0));
        let raw_file_name: String = chars[start..start + length].iter().collect();
        let nf1 = chars.iter().rposition(|&c| c == '#');
        let mut fields: VecDeque<String> = VecDeque::new();
        if let Some(nf1) = nf1 {
            if nf1 > 0 {
                let nf2 = chars.iter().rposition(|&c| c == '}');
                match nf2 {
                    None => self.append_error(st, line_content, messages::Brackets_not_closed, add_line_numbers),
                    Some(nf2) => {
                        let from = nf1 + FIELDS_MARKER.len();
                        let inner: String = if from <= nf2 {
                            chars[from..nf2].iter().collect()
                        } else {
                            String::new()
                        };
                        for item in inner.split(';') {
                            fields.push_back(trim(item).to_string());
                        }
                        if inner.ends_with(';') {
                            fields.pop_back();
                        }
                        if inner.is_empty() {
                            fields.clear();
                        }
                    }
                }
            }
        }
        let source_dir = self
            .source_file_path
            .as_deref()
            .and_then(|p| self.files.directory_name(p));
        let expanded = self.files.expand_environment_variables(&raw_file_name);
        let resolved = self.files.full_path(&expanded, source_dir.as_deref());
        let exists = self.files.exists(&resolved);
        if !self.include_stack.insert(resolved.clone()) {
            let msg = fmt(messages::Circular_include_detected_0, &[&raw_file_name]);
            self.append_error(st, line_content, &msg, add_line_numbers);
            return Ok(());
        }
        let r = (|| -> CResult<()> {
            if exists {
                let content = self.files.include(&resolved, Some(fields))?;
                let saved = self.source_file_path.clone();
                self.source_file_path = Some(resolved.clone());
                let line_number = st.line_number;
                let mut out = std::mem::take(&mut st.sb);
                // Errors inside included files do not make the outer Parse report errors.
                self.parse_inner(&content, &mut out, line_number, add_line_numbers);
                st.sb = out;
                self.source_file_path = saved;
                return Ok(());
            }
            self.append_error(st, line_content, messages::File_not_found, add_line_numbers);
            Ok(())
        })();
        self.include_stack.remove(&resolved);
        r
    }

    #[allow(unused_assignments)]
    fn parse_def(&mut self, st: &mut ParseState, line_content: &str, add_line_numbers: bool) -> CResult<()> {
        let chars: Vec<char> = line_content.chars().collect();
        let len = chars.len();
        let mut ts_start;
        let mut ts_end;
        if st.macro_def_count == 0 {
            let mut i = DEF_KEYWORD.len();
            let mut c = eat_space(&chars, &mut i);
            ts_start = i;
            ts_end = i;
            while i < len {
                c = chars[i];
                if c == '$' {
                    ts_end += 1;
                    st.macro_name = chars[ts_start..ts_end].iter().collect();
                    break;
                }
                if validator::is_macro_letter(c, ts_end - ts_start) {
                    ts_end += 1;
                } else {
                    let msg = fmt(messages::Invalid_symbol_0_in_macro_name, &[&c.to_string()]);
                    self.append_error(st, line_content, &msg, add_line_numbers);
                    break;
                }
                i += 1;
            }
            c = eat_space(&chars, &mut i);
            if c == '(' {
                let mut params: Vec<String> = Vec::new();
                let mut defaults: Vec<Option<String>> = Vec::new();
                let mut seen_optional = false;
                c = eat_space(&chars, &mut i);
                ts_start = i;
                ts_end = i;
                while i < len {
                    c = chars[i];
                    if c == ' ' {
                        c = eat_space(&chars, &mut i);
                        let _ = c;
                        continue;
                    }
                    let mut default_value: Option<String> = None;
                    if c == '$' {
                        ts_end += 1;
                        c = eat_space(&chars, &mut i);
                        if c == '=' {
                            eat_space(&chars, &mut i);
                            let end = index_of_top_level(&chars[i..], ';').unwrap_or(len - i);
                            let dv: String = chars[i..i + end].iter().collect();
                            default_value = Some(trim(&dv).to_string());
                            i += end;
                            c = if i < len { chars[i] } else { '\0' };
                        }
                    } else if c != ';' && c != ')' {
                        if validator::is_macro_letter(c, ts_end - ts_start) {
                            ts_end += 1;
                        } else if c != '\n' {
                            let msg = fmt(messages::Invalid_symbol_0_in_macro_name, &[&c.to_string()]);
                            self.append_error(st, line_content, &msg, add_line_numbers);
                        }
                        i += 1;
                        if i >= len {
                            return Err(MathParserException::unexpected(
                                "Index was outside the bounds of the array.",
                            ));
                        }
                        c = chars[i];
                        let _ = c;
                        continue;
                    }
                    let parameter_name: String = chars[ts_start..ts_end.min(len)].iter().collect();
                    if default_value.is_some() {
                        seen_optional = true;
                    } else if seen_optional {
                        return Err(exceptions::RequiredParameterAfterOptional(&parameter_name));
                    }
                    params.push(parameter_name);
                    defaults.push(default_value);
                    if c == ')' {
                        break;
                    }
                    c = eat_space(&chars, &mut i);
                    let _ = c;
                    ts_start = i;
                    ts_end = i;
                }
                c = eat_space(&chars, &mut i);
                st.macro_parameters = Some(params);
                st.macro_defaults = Some(defaults);
            } else {
                st.macro_parameters = None;
            }
            if c == '=' {
                eat_space(&chars, &mut i);
                let contents: String = chars[i.min(len)..].iter().collect();
                let m = Macro::new(Some(contents), st.macro_parameters.clone(), st.macro_defaults.clone())?;
                let name = st.macro_name.clone();
                self.add_macro(st, line_content, &name, m, add_line_numbers);
                st.macro_name = String::new();
                st.macro_defaults = None;
            } else {
                if st.macro_name.trim().is_empty() {
                    let name: String = chars[ts_start.min(len)..ts_end.min(len)].iter().collect();
                    let msg = fmt(messages::Invalid_macro_name_0, &[&name]);
                    self.append_error(st, line_content, &msg, add_line_numbers);
                    st.macro_name = String::new();
                }
                st.macro_def_count += 1;
            }
        } else {
            self.append_error(
                st,
                line_content,
                messages::Invalid_inside_macro_definition,
                add_line_numbers,
            );
            st.macro_def_count += 1;
        }
        Ok(())
    }

    fn parse_end_def(&mut self, st: &mut ParseState, line_content: &str, add_line_numbers: bool) -> CResult<()> {
        if st.macro_def_count < 1 {
            self.append_error(st, line_content, messages::There_is_no_matching_def, add_line_numbers);
        } else {
            // RemoveLastLineIfEmpty
            let mb = &mut st.macro_builder;
            if mb.len() > 2 && mb.ends_with('\n') {
                mb.pop();
                if mb.ends_with('\r') {
                    mb.pop();
                }
            }
            let content = st.macro_builder.clone();
            let m = Macro::new(Some(content), st.macro_parameters.clone(), st.macro_defaults.clone())?;
            let name = st.macro_name.clone();
            self.add_macro(st, line_content, &name, m, add_line_numbers);
            st.macro_name = String::new();
            st.macro_defaults = None;
            st.macro_builder.clear();
        }
        st.macro_def_count -= 1;
        Ok(())
    }

    fn append_line(&mut self, st: &mut ParseState, line: &str, add_line_numbers: bool) {
        st.sb.push_str(line);
        if add_line_numbers {
            st.sb.push('\u{000B}');
            st.sb.push_str(&st.line_number.to_string());
        }
        st.sb.push('\n');
        self.parsed_line_number += 1;
    }

    fn append_error(&mut self, st: &mut ParseState, line_content: &str, error_message: &str, add_line_numbers: bool) {
        let marker = if add_line_numbers {
            format!("\u{000B}{}", st.line_number)
        } else {
            String::new()
        };
        let line_html = format!(
            "[<a href=\"#0\" data-text=\"{}\">{}</a>]",
            st.line_number, st.line_number
        );
        st.sb.push_str(&message_codes::format(
            "Error_in_0_on_line_1_2",
            messages::Error_in_0_on_line_1_2,
            &[&html_encode(line_content), &line_html, error_message],
        ));
        st.sb.push_str(&marker);
        st.sb.push('\n');
        self.parsed_line_number += 1;
        st.has_errors = true;
        self.errors.push(crate::calcpad_error::CalcpadError {
            source_line: st.line_number,
            output_line: self.parsed_line_number,
            message: error_message.to_string(),
            source: crate::calcpad_error::CalcpadErrorSource::Macro,
        });
    }

    fn add_macro(&mut self, st: &mut ParseState, line_content: &str, name: &str, m: Macro, add_line_numbers: bool) {
        if self.macros.contains_key(name) {
            let msg = fmt(messages::Duplicate_macro_name_0, &[name]);
            self.append_error(st, line_content, &msg, add_line_numbers);
        } else {
            self.macros.insert(name.to_string(), m);
        }
    }

    fn apply_macros(&self, line_content: &str, currently_expanding: Option<&mut HashSet<String>>) -> CResult<String> {
        let mut local_set: HashSet<String> = HashSet::new();
        let has_set = currently_expanding.is_some();
        let expanding: &mut HashSet<String> = match currently_expanding {
            Some(s) => s,
            None => &mut local_set,
        };
        if !line_content.contains('$') {
            return Ok(line_content.to_string());
        }
        let mut lc: Vec<char> = line_content.chars().collect();
        let mut fields: Option<VecDeque<String>> = None;
        if let Some(index) = line_content.find(FIELDS_MARKER) {
            let ci = line_content[..index].chars().count();
            let s: String = lc[ci + 2..].iter().collect();
            lc.truncate(ci);
            let n = s.find('}').unwrap_or(s.len());
            fields = Some(get_fields(&s[..n], ';'));
        }
        let mut sb = String::new();
        let mut macro_arguments: Vec<String> = Vec::new();
        let mut keyword_arguments: Vec<(String, String)> = Vec::new();
        let mut bracket_count = 0i32;
        let mut inside_arg_list = false;
        let mut seen_keyword = false;
        let mut mac = Macro::empty();
        let mut macro_key: Option<String> = None;
        let mut ts_start = 0usize;
        let mut ts_end = 0usize;
        let len = lc.len();
        let mut i = 0usize;
        while i < len {
            let c = lc[i];
            if inside_arg_list {
                if c == '(' {
                    if bracket_count == 0 {
                        ts_start = i + 1;
                        ts_end = i + 1;
                    }
                    bracket_count += 1;
                } else if c == ')' {
                    bracket_count -= 1;
                }
                if c == ';' && bracket_count == 1 || c == ')' && bracket_count == 0 {
                    let raw: String = lc[ts_start..ts_end].iter().collect();
                    let raw_argument = trim(&raw).to_string();
                    if c == ')'
                        && raw_argument.is_empty()
                        && mac.has_defaults
                        && macro_arguments.is_empty()
                        && keyword_arguments.is_empty()
                    {
                        inside_arg_list = false;
                        ts_start = i + 1;
                        ts_end = i + 1;
                        i += 1;
                        continue;
                    }
                    if let Some((name, value)) = self.try_parse_keyword_argument(&raw_argument, &mac) {
                        seen_keyword = true;
                        if keyword_arguments.iter().any(|(k, _)| *k == name) {
                            return Err(exceptions::DuplicateArgument(&name));
                        }
                        let v = self.apply_macros(&value, Some(&mut *expanding))?;
                        keyword_arguments.push((name, v));
                    } else {
                        if seen_keyword {
                            return Err(exceptions::InvalidNumberOfArguments());
                        }
                        let s = self.apply_macros(&raw, Some(&mut *expanding))?;
                        macro_arguments.push(if s.trim().is_empty() { String::new() } else { s });
                    }
                    ts_start = i + 1;
                    ts_end = i + 1;
                    if c == ')' {
                        inside_arg_list = false;
                    }
                } else if bracket_count > 1 || c != '(' {
                    ts_end += 1;
                }
            } else if c == '$' && ts_end > ts_start {
                ts_end += 1;
                let macro_name: Vec<char> = lc[ts_start..ts_end].to_vec();
                let mlen = macro_name.len() - 1;
                macro_key = None;
                mac = Macro::empty();
                let mut j = 0;
                while j < mlen {
                    let candidate: String = macro_name[j..].iter().collect();
                    if let Some(m) = self.macros.get(&candidate) {
                        mac = m.clone();
                        macro_key = Some(candidate);
                        break;
                    }
                    j += 1;
                }
                if mac.is_empty() {
                    let name: String = macro_name.iter().collect();
                    return Err(exceptions::UndefinedMacro(&name));
                } else if j > 0 {
                    sb.extend(macro_name[..j].iter());
                }
                bracket_count = 0;
                macro_arguments.clear();
                keyword_arguments.clear();
                seen_keyword = false;
                inside_arg_list = mac.parameter_count() > 0;
                ts_start = i;
                ts_end = i;
            } else {
                if !mac.is_empty() {
                    let s = self.expand_macro(
                        &mac,
                        &macro_arguments,
                        &keyword_arguments,
                        macro_key.as_deref().unwrap_or(""),
                        expanding,
                    )?;
                    let sb_len = sb.len();
                    set_line_input_fields(&s, &mut sb, fields.as_mut(), false);
                    if sb.len() == sb_len {
                        sb.push_str(&s);
                    }
                    ts_start = i;
                    ts_end = i;
                    mac = Macro::empty();
                }
                if validator::is_macro_letter(c, ts_end - ts_start) {
                    if ts_end == ts_start {
                        ts_start = i;
                        ts_end = i;
                    }
                    ts_end += 1;
                } else {
                    if ts_end > ts_start {
                        sb.extend(lc[ts_start..ts_end].iter());
                        ts_start = i;
                        ts_end = i;
                    }
                    sb.push(c);
                }
            }
            i += 1;
        }
        if mac.is_empty() {
            if ts_end > ts_start {
                sb.extend(lc[ts_start..ts_end].iter());
            }
        } else if !inside_arg_list {
            let s = self.expand_macro(
                &mac,
                &macro_arguments,
                &keyword_arguments,
                macro_key.as_deref().unwrap_or(""),
                expanding,
            )?;
            sb.push_str(&s);
        }
        let _ = has_set;
        Ok(sb)
    }

    fn expand_macro(
        &self,
        mac: &Macro,
        macro_arguments: &[String],
        keyword_arguments: &[(String, String)],
        macro_key: &str,
        expanding: &mut HashSet<String>,
    ) -> CResult<String> {
        if expanding.contains(macro_key) {
            return Err(exceptions::CircularMacroReference(macro_key));
        }
        expanding.insert(macro_key.to_string());
        let r = (|| -> CResult<String> {
            let arguments = mac.resolve_arguments(macro_arguments, keyword_arguments)?;
            let run = mac.run(&arguments);
            self.apply_macros(&run, Some(&mut *expanding))
        })();
        expanding.remove(macro_key);
        r
    }

    fn try_parse_keyword_argument(&self, arg: &str, mac: &Macro) -> Option<(String, String)> {
        let chars: Vec<char> = arg.chars().collect();
        let mut i = 0;
        while i < chars.len() && validator::is_macro_letter(chars[i], i) {
            i += 1;
        }
        if i > 0 && i < chars.len() && chars[i] == '$' {
            let rest: String = chars[i + 1..].iter().collect();
            let rest = trim_start(&rest);
            let rc: Vec<char> = rest.chars().collect();
            if rc.len() > 1 && rc[0] == '=' && rc[1] != '=' {
                let name: String = chars[..=i].iter().collect();
                if mac.get_parameter_index(&name).is_some() || !self.macros.contains_key(&name) {
                    let value: String = rc[1..].iter().collect();
                    return Some((name, trim(&value).to_string()));
                }
            }
        }
        None
    }
}

fn eat_space(s: &[char], index: &mut usize) -> char {
    let len = s.len() as isize - 1;
    while (*index as isize) < len {
        *index += 1;
        if s[*index] != ' ' {
            return s[*index];
        }
    }
    '\0'
}

/// `MacroParser.HasInputFields`
pub fn has_input_fields(s: &str) -> bool {
    for item in enumerate_comments(s) {
        if !item.is_empty() && !item.starts_with('"') && !item.starts_with('\'') && item.contains('?') {
            return true;
        }
    }
    false
}

/// `MacroParser.SetLineInputFields`
pub fn set_line_input_fields(
    s: &str,
    sb: &mut String,
    fields: Option<&mut VecDeque<String>>,
    _force_line: bool,
) -> bool {
    let fields = match fields {
        None => return false,
        Some(f) => f,
    };
    if s.is_empty() || fields.is_empty() {
        return false;
    }
    let mut input_char = '\0';
    let count = fields.len();
    for item in enumerate_comments(s) {
        if item.is_empty() {
            continue;
        }
        if item.starts_with('"') || item.starts_with('\'') {
            sb.push_str(item);
            continue;
        }
        let ic: Vec<char> = item.chars().collect();
        let mut j0 = 0usize;
        let len = ic.len();
        let mut j = 0;
        while j < len {
            let c = ic[j];
            if c == '?' {
                input_char = c;
            } else if c == '{' && input_char == '?' {
                input_char = c;
                sb.extend(ic[j0..=j].iter());
            } else if c == '}' && input_char == '{' {
                input_char = '\0';
                match fields.pop_front() {
                    None => return false,
                    Some(val) => sb.push_str(&val),
                }
                j0 = j;
            } else if input_char == '{' {
                j += 1;
                continue;
            } else if c != ' ' {
                let seg: String = ic[j0..j].iter().collect();
                if input_char == '?' {
                    input_char = '\0';
                    if let Some(val) = fields.pop_front() {
                        sb.push_str(&format!("{}{{{}}}", seg, val));
                        j0 = j;
                    }
                }
            }
            j += 1;
        }
        let seg: String = ic[j0..].iter().collect();
        let mut added = false;
        if input_char == '?' {
            input_char = '\0';
            if let Some(val) = fields.pop_front() {
                sb.push_str(&format!("{} {{{}}}", seg, val));
                added = true;
            }
        }
        if !added {
            sb.push_str(&seg);
        }
    }
    fields.len() < count
}
