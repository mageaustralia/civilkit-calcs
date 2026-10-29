//! Port of Calcpad.Core/Parsers/ExpressionParser/ExpressionParser.UI.cs: `#UI` input
//! controls (entry, dropdown, radio, checkbox, datagrid) and the overrides a form sends back.
use std::collections::HashMap;

use super::{ExpressionParser, KeywordResult};
use crate::exceptions;
use crate::extension_methods::enumerate_comments;
use crate::math_parser_exception::{CResult, MathParserException};
use crate::messages;
use crate::netfmt::{fmt, html_attribute_encode, html_encode};
use crate::ui_dto::{UiDto, MAX_SIZE};
use crate::ui_syntax;

const UI_KEYWORD_LEN: usize = 3; // "#UI"
const THIN_SPACE: char = '\u{2009}';

/// `UiPropertyMetadata`
#[derive(Clone, Debug, Default)]
pub(super) struct UiMeta {
    pub ty: String,
    pub style: Option<String>,
    pub report_style: Option<String>,
    pub variable_name: String,
    pub data_values: Option<String>,
    pub cell_units: Option<Vec<Vec<Option<String>>>>,
    pub source_value: Option<String>,
    pub declaration_key: String,
    pub key: String,
    pub keeps_units: bool,
    pub allows_expression: bool,
    pub rows: i32,
    pub columns: i32,
    pub has_declared_shape: bool,
    pub column_headers: Option<Vec<Option<String>>>,
    pub row_headers: Option<Vec<Option<String>>>,
    pub width: Option<String>,
    pub row_header_width: Option<i32>,
    pub column_widths: Option<Vec<i32>>,
    pub keys: Vec<Option<String>>,
    pub values: Vec<Option<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

/// The per parse state of the `#UI` keyword.
#[derive(Default)]
pub(super) struct UiState {
    pub line_ui_controls: Option<Vec<UiMeta>>,
    pub pending_ui_error: Option<String>,
    pub ui_take_index: usize,
    /// In chars, like the rest of the line handling.
    pub ui_skip_chars: usize,
    pub ui_var_counts: HashMap<String, i32>,
    pub ui_declaration_index: HashMap<(String, isize, i32), i32>,
    pub ui_line_occurrences: HashMap<String, i32>,
}

fn skip_spaces(s: &[char], mut start: usize) -> usize {
    while start < s.len() && s[start] == ' ' {
        start += 1;
    }
    start
}

/// `IndexOfAssignment`: byte offset of the first '=' in code (not in a comment).
pub(super) fn index_of_assignment(s: &str) -> Option<usize> {
    let mut offset = 0;
    for segment in enumerate_comments(s) {
        if ui_syntax::is_code(segment) {
            if let Some(i) = segment.find('=') {
                return Some(offset + i);
            }
        }
        offset += segment.len();
    }
    None
}

fn extract_variable_name(lhs: &str) -> String {
    let mut sb = String::new();
    for segment in enumerate_comments(lhs) {
        if ui_syntax::is_code(segment) {
            sb.push_str(segment);
        }
    }
    sb.trim().to_string()
}

fn check_grid_size(rows: i32, columns: i32) -> CResult<()> {
    if rows as i64 * columns as i64 > MAX_SIZE {
        return Err(exceptions::DatagridSizeLimit());
    }
    Ok(())
}

/// C# `int.TryParse` (integer style: optional sign and surrounding whitespace).
fn try_parse_int(s: &str) -> Option<i32> {
    let t = s.trim_matches(|c: char| c.is_whitespace());
    if t.is_empty() {
        return None;
    }
    t.parse::<i32>().ok()
}

fn auto_detect_grid_size(rhs: &str, rows: &mut i32, columns: &mut i32) {
    let (Some(start), Some(end)) = (rhs.find('['), rhs.rfind(']')) else {
        return;
    };
    if end <= start {
        return;
    }
    let content = &rhs[start + 1..end];
    let pipe = content.find('|');
    let first_row = match pipe {
        None => content,
        Some(p) => &content[..p],
    };
    if *rows == 0 {
        *rows = if pipe.is_none() {
            1
        } else {
            content.matches('|').count() as i32 + 1
        };
    }
    if *columns == 0 {
        *columns = first_row.matches(';').count() as i32 + 1;
    }
}

fn auto_detect_grid_size_from_function(rhs: &str, rows: &mut i32, columns: &mut i32) {
    let (Some(start), Some(end)) = (rhs.find('('), rhs.rfind(')')) else {
        return;
    };
    if end <= start {
        return;
    }
    let args = rhs[start + 1..end].trim();
    if ui_syntax::starts_with_function(rhs, "vector") {
        if let Some(n) = try_parse_int(args) {
            if n > 0 {
                if *rows == 0 {
                    *rows = 1;
                }
                if *columns == 0 {
                    *columns = n;
                }
            }
        }
        return;
    }
    if let Some(semicolon) = args.find(';') {
        if semicolon > 0 {
            if let (Some(m), Some(k)) = (try_parse_int(&args[..semicolon]), try_parse_int(&args[semicolon + 1..])) {
                if m > 0 && k > 0 {
                    if *rows == 0 {
                        *rows = m;
                    }
                    if *columns == 0 {
                        *columns = k;
                    }
                }
            }
        }
    }
}

fn index_of_argument_separator(args: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in args.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ';' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn reshape_matrix_literal(s: &str, rows: i32, columns: i32) -> String {
    let (Some(start), Some(end)) = (s.find('['), s.rfind(']')) else {
        return s.to_string();
    };
    if end <= start {
        return s.to_string();
    }
    let cells = split_matrix_literal(&s[start + 1..end]);
    let pad = format!("0{}", unit_of_first_cell(&cells));
    let mut sb = String::new();
    for r in 0..rows.max(0) as usize {
        if r > 0 {
            sb.push_str(" | ");
        }
        for c in 0..columns.max(0) as usize {
            if c > 0 {
                sb.push_str("; ");
            }
            let value = cells
                .get(r)
                .and_then(|row| row.get(c))
                .map(String::as_str)
                .unwrap_or("");
            sb.push_str(if value.is_empty() { &pad } else { value });
        }
    }
    format!("{}{}{}", &s[..start + 1], sb, &s[end..])
}

fn unit_of_first_cell(cells: &[Vec<String>]) -> String {
    for row in cells {
        for cell in row {
            let (_, unit) = split_cell(cell);
            if !unit.is_empty() {
                return unit.to_string();
            }
        }
    }
    String::new()
}

/// Splits the numeric prefix of a cell from the unit written after it.
fn split_cell(cell: &str) -> (&str, &str) {
    let mut i = 0;
    for (k, c) in cell.char_indices() {
        if c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | '\u{2212}') {
            i = k + c.len_utf8();
        } else {
            break;
        }
    }
    if i == 0 {
        (cell, "")
    } else {
        (&cell[..i], &cell[i..])
    }
}

fn split_matrix_literal(content: &str) -> Vec<Vec<String>> {
    content
        .split('|')
        .map(|row| row.split(';').map(|c| c.trim().to_string()).collect())
        .collect()
}

fn new_units(values: &[Vec<String>]) -> Vec<Vec<Option<String>>> {
    values.iter().map(|r| vec![Some(String::new()); r.len()]).collect()
}

/// The grid's cells, with each unit kept aside when it edits numbers only.
fn set_datagrid_cells(ui: &mut UiMeta, mut values: Vec<Vec<String>>, mut units: Option<Vec<Vec<Option<String>>>>) {
    for r in 0..values.len() {
        for c in 0..values[r].len() {
            if !ui.keeps_units {
                if let Some(Some(u)) = units.as_ref().and_then(|u| u.get(r)).and_then(|row| row.get(c)) {
                    let u = u.clone();
                    values[r][c].push_str(&u);
                }
                continue;
            }
            let cell = values[r][c].clone();
            let (value, unit) = split_cell(&cell);
            values[r][c] = value.to_string();
            if !unit.is_empty() {
                let us = units.get_or_insert_with(|| new_units(&values));
                us[r][c] = Some(unit.to_string());
            }
        }
    }
    ui.cell_units = if ui.keeps_units { units } else { None };
    ui.data_values = Some(values.iter().map(|row| row.join(";")).collect::<Vec<_>>().join("|"));
}

/// Reads the cells out of the literal the line was written or overridden with.
fn capture_datagrid_values(ui: &mut UiMeta, expression: &str) {
    let Some(eq) = index_of_assignment(expression) else {
        return;
    };
    let rhs = expression[eq + 1..].trim();
    let (Some(start), Some(end)) = (rhs.find('['), rhs.rfind(']')) else {
        return;
    };
    if end <= start {
        return;
    }
    let cells = split_matrix_literal(&rhs[start + 1..end].replace(' ', ""));
    set_datagrid_cells(ui, cells, None);
}

fn capture_source_value(ui: &mut UiMeta, expression: &str) {
    if let Some(eq) = index_of_assignment(expression) {
        ui.source_value = Some(expression[eq + 1..].trim().to_string());
    }
}

fn apply_ui_override(ui: &UiMeta, expression: &str, value: &str) -> Option<String> {
    let eq = index_of_assignment(expression)?;
    let lhs = &expression[..eq + 1];
    let rhs = expression[eq + 1..].trim_start();
    if ui.ty == "datagrid" {
        return Some(match (rhs.rfind(']'), rhs.find('[')) {
            (Some(end), Some(_)) => format!("{} {}{}", lhs, value, &rhs[end + 1..]),
            _ => format!("{} {}", lhs, value),
        });
    }
    // Only the default mode rewrites the number in place, leaving the unit as written.
    if ui.ty == "dropdown" || ui.ty == "radio" || !ui.keeps_units {
        return Some(format!("{} {}", lhs, value));
    }
    let i = rhs
        .char_indices()
        .find(|&(_, c)| !is_numeric_char(c))
        .map(|(k, _)| k)
        .unwrap_or(rhs.len());
    if i == 0 {
        None
    } else {
        Some(format!("{} {}{}", lhs, value, &rhs[i..]))
    }
}

fn is_numeric_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')
}

/// The unit text with the marks the report draws for it made plain: the report shows `kN/m` as
/// `kN∕m`, `kN*m` as `kN·m` and `m^3` as `m` with a raised 3.
fn plain_unit(u: &str) -> String {
    u.chars()
        .filter(|&c| c != '^' && !c.is_whitespace())
        .map(|c| match c {
            '\u{2215}' | '\u{2044}' => '/',
            '\u{00B7}' | '\u{22C5}' | '\u{2219}' => '*',
            '\u{00B2}' => '2',
            '\u{00B3}' => '3',
            '\u{00B9}' => '1',
            '\u{2212}' => '-',
            c => c,
        })
        .collect()
}

/// A leading number (sign, digits, point; no exponent) and the rest, with the digits after the point.
fn split_number(s: &str) -> Option<(f64, usize, &str)> {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let d0 = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut decimals = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let f0 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        decimals = i - f0;
    }
    if i == d0 || (i == d0 + 1 && b[d0] == b'.') {
        return None;
    }
    // An exponent (`1e3`) is not read here; the texts must then match exactly.
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') && b.get(i + 1).is_some_and(|c| c.is_ascii_digit() || *c == b'-' || *c == b'+') {
        return None;
    }
    Some((s[..i].parse().ok()?, decimals, &s[i..]))
}

/// True if an option's value is what the report shows for the variable: the same unit, and the
/// same number to the digits shown (`25 kN/m` is shown as `25kN∕m`, `1.0` as `1`, `0.9` as `0.90`).
fn ui_value_matches(option: &str, shown: &str) -> bool {
    let (o, s) = (plain_unit(option), plain_unit(shown));
    if o == s {
        return true;
    }
    match (split_number(&o), split_number(&s)) {
        (Some((a, da, ua)), Some((b, db, ub))) if ua == ub => {
            if (a - b).abs() <= 1e-9 * a.abs().max(1.0) {
                return true;
            }
            // A value with more digits than the report shows is shown rounded.
            db >= 2 && da > db && ((a * 10f64.powi(db as i32)).round() / 10f64.powi(db as i32) - b).abs() < 1e-9
        }
        _ => false,
    }
}

fn strip_spaces(s: &str) -> String {
    s.replace(' ', "")
}

fn result_start(equation_html: &str) -> Option<usize> {
    equation_html.rfind(" = ").map(|i| i + 3)
}

fn selected_value(value: &str, unit_html: &str) -> String {
    let mut sb = strip_spaces(value);
    let mut tag = String::new();
    let mut in_tag = false;
    for c in unit_html.chars() {
        if c == '<' {
            in_tag = true;
            tag.clear();
        } else if c == '>' {
            in_tag = false;
            // The bar of a stacked unit (`kN` over `m`) is an empty element; it stands for the slash.
            if tag.contains("class=\"dvl\"") {
                sb.push('/');
            }
        } else if in_tag {
            tag.push(c);
        } else if c != ' ' && c != THIN_SPACE {
            sb.push(c);
        }
    }
    sb
}

fn split_value_and_unit(result_html: &str) -> (String, String) {
    let (mut value, mut unit_html) = match result_html.find('<') {
        None => (result_html.trim().to_string(), String::new()),
        Some(0) => (String::new(), result_html.to_string()),
        Some(k) => (
            result_html[..k].trim_end_matches([THIN_SPACE, ' ']).to_string(),
            format!("{}{}", THIN_SPACE, &result_html[k..]),
        ),
    };
    // Angle units follow the number with no markup of their own.
    let mut i = value.len();
    while let Some(c) = value[..i].chars().next_back() {
        if matches!(c, '\u{00B0}' | '\u{2032}' | '\u{2033}') {
            i -= c.len_utf8();
        } else {
            break;
        }
    }
    if i != value.len() {
        unit_html = format!("{}{}", &value[i..], unit_html);
        value.truncate(i);
    }
    (value, unit_html)
}

fn ui_class(ui: &UiMeta, base_class: &str) -> String {
    html_attribute_encode(&match &ui.style {
        None => base_class.to_string(),
        Some(s) => format!("{} {}", base_class, s),
    })
}

/// `UiModeAttributes`: only written when they differ from the default.
fn ui_mode_attributes(ui: &UiMeta) -> &'static str {
    if ui.allows_expression {
        " data-ui-allow-expression=\"1\""
    } else if ui.keeps_units {
        ""
    } else {
        " data-ui-force-units=\"0\""
    }
}

/// The bounds and the step of an entry (`#UI {"min": 2, "max": 12, "step": 0.5} L = 6m`), written
/// only when given; the form draws a slider from them and refuses a typed value outside them.
fn ui_range_attributes(ui: &UiMeta) -> String {
    let mut s = String::new();
    for (name, v) in [("min", ui.min), ("max", ui.max), ("step", ui.step)] {
        if let Some(v) = v {
            s.push_str(&format!(" data-ui-{}=\"{}\"", name, crate::netfmt::double_to_string(v)));
        }
    }
    s
}

/// The number a typed value stands for, for the range check (`-` may be a minus sign).
fn leading_number(value: &str) -> Option<f64> {
    let t: String = value.trim().replace('\u{2212}', "-");
    let end = t
        .char_indices()
        .find(|&(_, c)| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+')))
        .map(|(k, _)| k)
        .unwrap_or(t.len());
    t[..end].parse::<f64>().ok()
}

/// System.Text.Json string escaping with the default (HTML safe) encoder.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ' '..='~' if !matches!(c, '<' | '>' | '&' | '\'' | '+' | '`') => out.push(c),
            _ => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04X}", u));
                }
            }
        }
    }
    out.push('"');
    out
}

fn json_opt_string(s: &Option<String>) -> String {
    match s {
        None => "null".to_string(),
        Some(s) => json_string(s),
    }
}

fn json_string_array(a: &[Option<String>]) -> String {
    format!("[{}]", a.iter().map(json_opt_string).collect::<Vec<_>>().join(","))
}

fn json_int_array(a: &[i32]) -> String {
    format!("[{}]", a.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","))
}

fn json_string_grid(a: &[Vec<Option<String>>]) -> String {
    format!(
        "[{}]",
        a.iter().map(|r| json_string_array(r)).collect::<Vec<_>>().join(",")
    )
}

/// The stored units over the grid as drawn.
fn fit_cell_units(ui: &UiMeta) -> Option<Vec<Vec<Option<String>>>> {
    let units = ui.cell_units.as_ref()?;
    let fallback = units.iter().flatten().flatten().find(|u| !u.is_empty())?.clone();
    let mut fitted = Vec::new();
    for r in 0..ui.rows.max(0) as usize {
        let mut row = Vec::new();
        for c in 0..ui.columns.max(0) as usize {
            let v = units
                .get(r)
                .and_then(|row| row.get(c))
                .cloned()
                .flatten()
                .unwrap_or_else(|| fallback.clone());
            row.push(Some(v));
        }
        fitted.push(row);
    }
    Some(fitted)
}

impl ExpressionParser {
    pub(super) fn ui_type_is_not_grid(&self, i: usize) -> bool {
        self.ui_meta(i).ty != "datagrid"
    }

    pub(super) fn has_ui_controls(&self) -> bool {
        self.ui.line_ui_controls.as_ref().is_some_and(|c| !c.is_empty())
    }

    pub(super) fn reset_ui_state(&mut self) {
        self.ui.line_ui_controls = None;
        self.ui.pending_ui_error = None;
        self.ui.ui_take_index = 0;
        self.ui.ui_skip_chars = 0;
        self.ui.ui_line_occurrences.clear();
    }

    /// `ParseKeywordUi`
    pub(super) fn parse_keyword_ui(&mut self, line: &str) -> CResult<KeywordResult> {
        self.reset_ui_state();
        let s: Vec<char> = line.chars().collect();
        let cursor = skip_spaces(&s, UI_KEYWORD_LEN);
        let mut properties = UiDto::default();
        if cursor < s.len() && s[cursor] == '{' {
            let Some(brace_end) = s.iter().position(|&c| c == '}') else {
                self.append_error(line, messages::Improper_format_for_UI_keyword_Missing_closing_brace);
                self.ui.ui_skip_chars = s.len();
                return Ok(KeywordResult::None);
            };
            let json: String = if brace_end >= cursor {
                s[cursor..=brace_end].iter().collect()
            } else {
                String::new()
            };
            match UiDto::parse(&json) {
                Ok(d) => properties = d.unwrap_or_default(),
                Err(_) => {
                    self.append_error(line, &fmt(messages::Invalid_JSON_in_0, &["#UI"]));
                    self.ui.ui_skip_chars = skip_spaces(&s, UI_KEYWORD_LEN);
                    return Ok(KeywordResult::None);
                }
            }
            self.ui.ui_skip_chars = skip_spaces(&s, brace_end + 1);
        } else {
            self.ui.ui_skip_chars = cursor;
        }
        let errors = properties.validate();
        if !errors.is_empty() {
            for e in errors {
                self.append_error(line, &e);
            }
            return Ok(KeywordResult::None);
        }
        let ui_rows = properties.rows.unwrap_or(0);
        let ui_columns = properties.columns.unwrap_or(0);
        let has_declared_shape = ui_rows > 0 && ui_columns > 0;
        let mut controls = Vec::new();
        let rest: String = s[self.ui.ui_skip_chars.min(s.len())..].iter().collect();
        for (name, rhs) in ui_syntax::enumerate_assignments(&rest) {
            if name.ends_with('$') {
                self.append_error(line, messages::Only_numbers_are_supported_by_the_UI_keyword);
                return Ok(KeywordResult::None);
            }
            let ty = ui_syntax::resolve_type(properties.ty.as_deref(), &rhs);
            if !ui_syntax::is_value_for(&rhs, &ty, properties.allows_expression()) {
                self.append_error(line, messages::UI_expressions_require_allowExpression);
                return Ok(KeywordResult::None);
            }
            if let Some(mismatch) =
                ui_syntax::type_mismatch(properties.ty.as_deref(), &rhs, properties.allows_expression())
            {
                self.append_error(line, &mismatch);
                return Ok(KeywordResult::None);
            }
            let type_errors = properties.validate_resolved_type(&ty);
            if !type_errors.is_empty() {
                for e in type_errors {
                    self.append_error(line, &e);
                }
                return Ok(KeywordResult::None);
            }
            let (mut rows, mut columns) = (ui_rows, ui_columns);
            if ty == "datagrid" && !has_declared_shape {
                if rhs.chars().count() > 1 && rhs.starts_with('[') && rhs.ends_with(']') {
                    auto_detect_grid_size(&rhs, &mut rows, &mut columns);
                } else {
                    auto_detect_grid_size_from_function(&rhs, &mut rows, &mut columns);
                }
            }
            if ty == "datagrid" {
                check_grid_size(rows, columns)?;
            }
            let (declaration_key, key) = self.get_ui_key(&name);
            controls.push(UiMeta {
                ty,
                style: properties.style.clone(),
                report_style: properties.report_style.clone(),
                variable_name: name,
                source_value: Some(rhs),
                declaration_key,
                key,
                keeps_units: properties.keeps_units(),
                allows_expression: properties.allows_expression(),
                rows,
                columns,
                has_declared_shape,
                column_headers: properties.column_headers.clone(),
                row_headers: properties.row_headers.clone(),
                width: properties.get_width(),
                row_header_width: properties.row_header_width,
                column_widths: properties.column_widths.clone(),
                keys: properties.keys.clone().unwrap_or_default(),
                values: properties.values.clone().unwrap_or_default(),
                min: properties.min,
                max: properties.max,
                step: properties.step,
                ..Default::default()
            });
        }
        if controls.is_empty() {
            self.ui.line_ui_controls = None;
            self.append_error(line, messages::The_UI_keyword_requires_a_variable_assignment);
        } else {
            self.ui.line_ui_controls = Some(controls);
        }
        Ok(KeywordResult::None)
    }

    /// `TakeUiControl`: the control declared for the variable this expression assigns.
    pub(super) fn take_ui_control(&mut self, expression: &str) -> Option<usize> {
        if !self.has_ui_controls() {
            return None;
        }
        let eq = index_of_assignment(expression)?;
        if eq < 1 {
            return None;
        }
        let name = extract_variable_name(&expression[..eq]);
        let controls = self.ui.line_ui_controls.as_ref()?;
        for i in self.ui.ui_take_index..controls.len() {
            if controls[i].variable_name != name {
                continue;
            }
            self.ui.ui_take_index = i + 1;
            return Some(i);
        }
        None
    }

    fn get_ui_key(&mut self, var_name: &str) -> (String, String) {
        let occurrence = *self.ui.ui_line_occurrences.get(var_name).unwrap_or(&0);
        self.ui.ui_line_occurrences.insert(var_name.to_string(), occurrence + 1);
        let id = (var_name.to_string(), self.current_line, occurrence);
        let ordinal = match self.ui.ui_declaration_index.get(&id) {
            Some(&o) => o,
            None => {
                let o = self.ui.ui_var_counts.get(var_name).copied().unwrap_or(0) + 1;
                self.ui.ui_var_counts.insert(var_name.to_string(), o);
                self.ui.ui_declaration_index.insert(id, o);
                o
            }
        };
        let declaration_key = format!("{}:{}", var_name, ordinal);
        if self.loops.is_empty() {
            return (declaration_key.clone(), declaration_key);
        }
        let passes: Vec<String> = self.loops.iter().map(|l| l.pass().to_string()).collect();
        let key = format!("{}:{}", declaration_key, passes.join("."));
        (declaration_key, key)
    }

    fn ui_meta(&self, i: usize) -> &UiMeta {
        &self.ui.line_ui_controls.as_ref().unwrap()[i]
    }

    fn ui_meta_mut(&mut self, i: usize) -> &mut UiMeta {
        &mut self.ui.line_ui_controls.as_mut().unwrap()[i]
    }

    fn resolve_datagrid_shape(&mut self, i: usize) -> CResult<()> {
        let (rows, columns, name) = {
            let ui = self.ui_meta(i);
            (ui.rows, ui.columns, ui.variable_name.clone())
        };
        if rows > 0 && columns > 0 {
            return check_grid_size(rows, columns);
        }
        let (r, c) = self.p().get_variable_shape(&name);
        let ui = self.ui_meta_mut(i);
        if ui.rows == 0 {
            ui.rows = r;
        }
        if ui.columns == 0 {
            ui.columns = c;
        }
        check_grid_size(ui.rows, ui.columns)
    }

    fn resolve_shape_from_sizing_call(&mut self, i: usize, expression: &str) -> CResult<()> {
        {
            let ui = self.ui_meta(i);
            if ui.rows > 0 && ui.columns > 0 || !self.calculate || self.is_val < 0 {
                return Ok(());
            }
        }
        let Some(eq) = index_of_assignment(expression) else {
            return Ok(());
        };
        let rhs = expression[eq + 1..].trim();
        let is_vector = ui_syntax::starts_with_function(rhs, "vector");
        if !is_vector && !ui_syntax::starts_with_function(rhs, "matrix") {
            return Ok(());
        }
        let (Some(start), Some(end)) = (rhs.find('('), rhs.rfind(')')) else {
            return Ok(());
        };
        if end <= start {
            return Ok(());
        }
        let args = &rhs[start + 1..end];
        if is_vector {
            let n = self.evaluate_count(args);
            if n > 0 {
                let ui = self.ui_meta_mut(i);
                if ui.rows == 0 {
                    ui.rows = 1;
                }
                if ui.columns == 0 {
                    ui.columns = n;
                }
                return check_grid_size(ui.rows, ui.columns);
            }
            return Ok(());
        }
        let Some(sep) = index_of_argument_separator(args) else {
            return Ok(());
        };
        let m = self.evaluate_count(&args[..sep]);
        let k = self.evaluate_count(&args[sep + 1..]);
        if m > 0 && k > 0 {
            let ui = self.ui_meta_mut(i);
            if ui.rows == 0 {
                ui.rows = m;
            }
            if ui.columns == 0 {
                ui.columns = k;
            }
            return check_grid_size(ui.rows, ui.columns);
        }
        Ok(())
    }

    fn evaluate_count(&mut self, expression: &str) -> i32 {
        let p = self.p();
        let r = p.parse(expression, false).and_then(|_| p.calculate(false, -1));
        match r {
            Ok(()) => {
                let n = p.real().round_ties_even();
                if n > 0.0 && n < i32::MAX as f64 {
                    n as i32
                } else {
                    0
                }
            }
            Err(_) => {
                p.reset_stack();
                0
            }
        }
    }

    /// `PrepareUiExpression`: applies an entered value and records what the control shows.
    pub(super) fn prepare_ui_expression(&mut self, i: usize, expression: &str) -> CResult<String> {
        let mut expression = expression.to_string();
        let is_datagrid = self.ui_meta(i).ty == "datagrid";
        if is_datagrid && self.enable_ui {
            let ui = self.ui_meta(i);
            if ui.has_declared_shape {
                expression = reshape_matrix_literal(&expression, ui.rows, ui.columns);
            }
        }
        if let Some(mut value) = self.try_get_ui_override(i) {
            if is_datagrid {
                self.resolve_shape_from_sizing_call(i, &expression)?;
                let ui = self.ui_meta(i);
                if ui.rows > 0 && ui.columns > 0 {
                    value = reshape_matrix_literal(&value, ui.rows, ui.columns);
                }
            }
            let ui = self.ui_meta(i);
            if ui.ty != "datagrid" && (ui.min.is_some() || ui.max.is_some()) {
                if let Some(x) = leading_number(&value) {
                    if ui.min.is_some_and(|lo| x < lo) || ui.max.is_some_and(|hi| x > hi) {
                        let show = |v: Option<f64>| v.map_or("-\u{221e}".to_string(), crate::netfmt::double_to_string);
                        return Err(MathParserException::new(fmt(
                            messages::The_UI_value_0_is_outside_1_to_2,
                            &[&format!("{} = {}", ui.variable_name, value.trim()), &show(ui.min), &show(ui.max)],
                        )));
                    }
                }
            }
            if let Some(overridden) = apply_ui_override(self.ui_meta(i), &expression, &value) {
                expression = overridden;
            }
        }
        let ui = self.ui_meta_mut(i);
        if is_datagrid {
            capture_datagrid_values(ui, &expression);
        } else {
            capture_source_value(ui, &expression);
        }
        Ok(expression)
    }

    fn try_get_ui_override(&self, i: usize) -> Option<String> {
        let overrides = self.ui_overrides.as_ref()?;
        let ui = self.ui_meta(i);
        overrides
            .get(&ui.key)
            .or_else(|| overrides.get(&ui.declaration_key))
            .or_else(|| overrides.get(&ui.variable_name))
            .cloned()
    }

    fn resolve_datagrid_values(&mut self, i: usize) {
        if self.ui_meta(i).data_values.is_some() {
            return;
        }
        let (name, rows, columns) = {
            let ui = self.ui_meta(i);
            (ui.variable_name.clone(), ui.rows, ui.columns)
        };
        let grid = if self.calculate {
            self.p().get_ui_grid_values(&name, rows, columns)
        } else {
            None
        };
        let ui = self.ui_meta_mut(i);
        match grid {
            Some((values, units)) => set_datagrid_cells(ui, values, Some(units)),
            None => {
                let zeros = vec!["0"; ui.columns.max(0) as usize].join(";");
                ui.data_values = Some(vec![zeros; ui.rows.max(0) as usize].join("|"));
            }
        }
    }

    /// `GetUiAttributes`: what the line's paragraph carries in input mode.
    pub(super) fn get_ui_attributes(&self, i: usize) -> String {
        let ui = self.ui_meta(i);
        let mut sb = format!(
            " data-ui-type=\"{}\" data-ui-line=\"{}\" data-ui-var=\"{}\"",
            ui.ty,
            self.p().line.get(),
            html_attribute_encode(&ui.key)
        );
        if let Some(style) = &ui.style {
            sb.push_str(&format!(" data-ui-style=\"{}\"", html_attribute_encode(style)));
        }
        if ui.ty == "datagrid" {
            sb.push_str(&format!(
                " data-ui-rows=\"{}\" data-ui-columns=\"{}\"",
                ui.rows, ui.columns
            ));
        }
        sb
    }

    /// `AddReportStyle`
    pub(super) fn add_report_style(&self, i: usize, attributes: &str) -> String {
        let style = match &self.ui_meta(i).report_style {
            Some(s) if !s.is_empty() => s,
            _ => return attributes.to_string(),
        };
        const CLASS_ATTR: &str = "class=\"";
        match attributes.find(CLASS_ATTR) {
            None => format!("{} class=\"{}\"", attributes, html_attribute_encode(style)),
            Some(k) => {
                let at = k + CLASS_ATTR.len();
                format!(
                    "{}{} {}",
                    &attributes[..at],
                    html_attribute_encode(style),
                    &attributes[at..]
                )
            }
        }
    }

    fn ui_binding(&self, ui: &UiMeta) -> String {
        format!(
            " data-ui-var=\"{}\" data-ui-line=\"{}\"",
            html_attribute_encode(&ui.key),
            self.p().line.get()
        )
    }

    fn ui_input_binding(&self, ui: &UiMeta) -> String {
        format!("{}{}", self.ui_binding(ui), ui_mode_attributes(ui))
    }

    fn build_ui_entry(&self, ui: &UiMeta, value: &str) -> String {
        format!(
            "<input type=\"text\" class=\"{}\" value=\"{}\"{}{}>",
            ui_class(ui, "calcpad-ui-input"),
            html_attribute_encode(value),
            self.ui_input_binding(ui),
            ui_range_attributes(ui)
        )
    }

    fn build_ui_dropdown(&self, ui: &UiMeta, value: &str) -> String {
        let mut sb = format!(
            "<select class=\"{}\"{}>",
            ui_class(ui, "calcpad-ui-dropdown"),
            self.ui_binding(ui)
        );
        for k in 0..ui.keys.len() {
            let v = ui.values.get(k).cloned().flatten().unwrap_or_default();
            let selected = if ui_value_matches(&v, value) { " selected" } else { "" };
            sb.push_str(&format!("<option value=\"{}\"{}>", html_attribute_encode(&v), selected));
            sb.push_str(&format!(
                "{}</option>",
                html_encode(ui.keys[k].as_deref().unwrap_or(""))
            ));
        }
        sb.push_str("</select>");
        sb
    }

    fn build_ui_radio(&self, ui: &UiMeta, value: &str) -> String {
        let group = html_attribute_encode(&format!("ui-radio-{}", ui.key));
        let mut sb = format!(
            "<span class=\"{}\"{}>",
            ui_class(ui, "calcpad-ui-radio"),
            self.ui_binding(ui)
        );
        for k in 0..ui.keys.len() {
            let v = ui.values.get(k).cloned().flatten().unwrap_or_default();
            let checked = if ui_value_matches(&v, value) { " checked" } else { "" };
            sb.push_str("<label class=\"calcpad-ui-radio-label\">");
            sb.push_str(&format!(
                "<input type=\"radio\" name=\"{}\" value=\"{}\"{}>",
                group,
                html_attribute_encode(&v),
                checked
            ));
            sb.push_str(&format!(
                " {}</label>",
                html_encode(ui.keys[k].as_deref().unwrap_or(""))
            ));
        }
        sb.push_str("</span>");
        sb
    }

    fn build_ui_checkbox(&self, ui: &UiMeta, value: &str) -> String {
        let checked = if value.trim() == "1" { " checked" } else { "" };
        format!(
            "<input type=\"checkbox\" class=\"{}\"{}{}>",
            ui_class(ui, "calcpad-ui-checkbox"),
            self.ui_binding(ui),
            checked
        )
    }

    fn build_ui_datagrid(&self, ui: &UiMeta) -> String {
        let mut sb = format!(
            "<div class=\"{}\"{} data-ui-rows=\"{}\" data-ui-columns=\"{}\" data-ui-values=\"{}\"{}",
            ui_class(ui, "calcpad-ui-datagrid"),
            self.ui_binding(ui),
            ui.rows,
            ui.columns,
            html_attribute_encode(ui.data_values.as_deref().unwrap_or("")),
            ui_mode_attributes(ui)
        );
        let mut attr = |name: &str, json: Option<String>| {
            if let Some(j) = json {
                sb.push_str(&format!(" {}=\"{}\"", name, html_attribute_encode(&j)));
            }
        };
        attr(
            "data-ui-col-headers",
            ui.column_headers.as_deref().map(json_string_array),
        );
        attr("data-ui-row-headers", ui.row_headers.as_deref().map(json_string_array));
        attr("data-ui-column-widths", ui.column_widths.as_deref().map(json_int_array));
        attr(
            "data-ui-cell-units",
            fit_cell_units(ui).as_deref().map(json_string_grid),
        );
        if let Some(w) = &ui.width {
            sb.push_str(&format!(" data-ui-width=\"{}\"", w));
        }
        if let Some(w) = ui.row_header_width {
            sb.push_str(&format!(" data-ui-row-header-width=\"{}\"", w));
        }
        sb.push_str("></div>");
        sb
    }

    /// `InjectUiInput`
    pub(super) fn inject_ui_input(&self, i: usize, equation_html: &str) -> String {
        let ui = self.ui_meta(i);
        match ui.ty.as_str() {
            "dropdown" => {
                self.replace_equation(equation_html, |v, u| self.build_ui_dropdown(ui, &selected_value(v, u)))
            }
            "radio" => self.replace_equation(equation_html, |v, u| self.build_ui_radio(ui, &selected_value(v, u))),
            "checkbox" => self.replace_equation(equation_html, |v, _| self.build_ui_checkbox(ui, v)),
            _ if ui.allows_expression => {
                let control = self.build_ui_entry(ui, ui.source_value.as_deref().unwrap_or(""));
                match equation_html.find(" = ") {
                    None => control,
                    Some(k) => format!("{}{}", &equation_html[..k + 3], control),
                }
            }
            _ if !ui.keeps_units => {
                let units = self.p().result_units_input_text();
                self.inject_ui_control(equation_html, false, |v, _| {
                    self.build_ui_entry(ui, &format!("{}{}", strip_spaces(v), units))
                })
            }
            _ => self.inject_ui_control(equation_html, true, |v, _| self.build_ui_entry(ui, v)),
        }
    }

    /// `BuildUiControlFromSource`: the control for a line that did not parse.
    pub(super) fn build_ui_control_from_source(&self, i: usize) -> String {
        let ui = self.ui_meta(i);
        let value = ui.source_value.clone().unwrap_or_default();
        match ui.ty.as_str() {
            "dropdown" => self.build_ui_dropdown(ui, &strip_spaces(&value)),
            "radio" => self.build_ui_radio(ui, &strip_spaces(&value)),
            "checkbox" => self.build_ui_checkbox(ui, &value),
            _ => self.build_ui_entry(ui, &value),
        }
    }

    fn inject_ui_control(&self, equation_html: &str, keep_unit: bool, build: impl Fn(&str, &str) -> String) -> String {
        let Some(start) = result_start(equation_html) else {
            return equation_html.to_string();
        };
        let (value, unit_html) = split_value_and_unit(&equation_html[start..]);
        format!(
            "{}{}{}",
            &equation_html[..start],
            build(&value, &unit_html),
            if keep_unit { unit_html.as_str() } else { "" }
        )
    }

    fn replace_equation(&self, equation_html: &str, build: impl Fn(&str, &str) -> String) -> String {
        let Some(start) = result_start(equation_html) else {
            return equation_html.to_string();
        };
        let (value, unit_html) = split_value_and_unit(&equation_html[start..]);
        build(&value, &unit_html)
    }

    /// The grids of the line, written after its paragraph in input mode.
    pub(super) fn append_ui_datagrids(&mut self) {
        let n = self.ui.line_ui_controls.as_ref().map_or(0, |c| c.len());
        for i in 0..n {
            if self.ui_meta(i).ty != "datagrid" {
                continue;
            }
            let r = self.resolve_datagrid_shape(i);
            match r {
                Ok(()) => {
                    self.resolve_datagrid_values(i);
                    let html = self.build_ui_datagrid(self.ui_meta(i));
                    self.sb.push_str(&html);
                    self.sb.push('\n');
                }
                Err(ex) => {
                    let h = self.err_html(&ex.message);
                    self.sb.push_str(&h);
                    let line = self.cur();
                    self.record_error(line, &ex.message, self.debug);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reshapes_and_splits() {
        assert_eq!(reshape_matrix_literal("a = [1m; 2m]", 2, 2), "a = [1m; 2m | 0m; 0m]");
        assert_eq!(split_cell("2.5kN"), ("2.5", "kN"));
        assert_eq!(
            split_value_and_unit("45\u{00B0}"),
            ("45".to_string(), "\u{00B0}".to_string())
        );
        assert_eq!(json_string("kN\u{00B7}m <a>"), "\"kN\\u00B7m \\u003Ca\\u003E\"");
    }
}
