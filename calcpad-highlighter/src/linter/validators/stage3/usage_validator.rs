//! UsageValidator.cs
use crate::content_resolution::result::FunctionInfo;
use crate::linter::constants::builtins;
use crate::linter::constants::patterns;
use crate::linter::helpers::cs;
use crate::linter::helpers::directive_state::{DirectiveState, OutputMode};
use crate::linter::helpers::line_parser;
use crate::linter::helpers::parameter_parser::{parse_macro_parameters, parse_parameters};
use crate::linter::helpers::parsing_helpers::{
    count_required_params, extract_params_string, find_closing_brace, find_closing_paren,
    get_function_params_from_line, parse_param_list, try_parse_keyword_arg,
};
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::{trim, trim_end, trim_start, StrMap, StrSet};
use crate::tokenizer::models::{Token, TokenType};

#[derive(Default)]
pub struct UsageValidator;

impl UsageValidator {
    pub fn new() -> Self {
        UsageValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        validate_commands(stage3, result, tp);
        validate_undefined_identifiers(stage3, result, tp);
        validate_undefined_units(stage3, result, tp);
        validate_function_calls(stage3, result, tp);
        validate_macro_calls(stage3, result, tp);
        validate_unused_definitions(stage3, result);
    }
}

/// `string.IndexOf(value, OrdinalIgnoreCase)` as a char index.
fn index_of_ic(line: &[char], value: &str) -> Option<usize> {
    cs::index_of_ic(line, value, 0)
}

fn validate_commands(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) {
            continue;
        }
        if line_parser::should_skip_line(line) {
            continue;
        }
        let lc = cs::chars(line);
        if line_parser::is_directive_line(trim(&lc)) {
            continue;
        }
        let Some(at_index) = lc.iter().position(|c| *c == '@') else {
            continue;
        };

        let mut command_name = String::new();
        let mut cmd_start_index: i32 = -1;
        let mut has_command = false;
        for (cmd, cmd_with_brace) in builtins::commands_with_brace() {
            if let Some(cmd_index) = index_of_ic(&lc, cmd_with_brace) {
                if cmd_index < at_index {
                    has_command = true;
                    command_name = cmd.clone();
                    cmd_start_index = (cmd_index + cmd.chars().count() + 1) as i32;
                    break;
                }
            }
        }
        if !has_command {
            continue;
        }

        let tokens = tp.get_tokens_for_line(i);

        let mut skip_syntax_phase = false;
        if command_name.eq_ignore_ascii_case("$Plot") {
            let before_at = &lc[..at_index];
            if before_at.contains(&'|') || before_at.contains(&'&') {
                skip_syntax_phase = true;
            }
        }

        let after_at = &lc[at_index + 1..];
        let mut equals_index: i32 = -1;
        for j in 0..after_at.len() {
            if after_at[j] == '=' {
                let is_comparison = (j > 0 && matches!(after_at[j - 1], '<' | '>' | '!' | '='))
                    || (j + 1 < after_at.len() && after_at[j + 1] == '=');
                if !is_comparison {
                    equals_index = j as i32;
                    break;
                }
            }
        }

        let at_abs = at_index as i32;
        let equals_abs = if equals_index >= 0 {
            at_abs + 1 + equals_index
        } else {
            -1
        };

        if !skip_syntax_phase {
            if equals_index < 0 {
                result.add_error(
                    i,
                    at_abs,
                    at_abs + 1,
                    "CPD-3406",
                    "Invalid command syntax: expected 'variable = start : end' after '@'",
                );
                continue;
            }
            let mut variable_tokens: Vec<&Token> = Vec::new();
            let mut number_tokens: Vec<&Token> = Vec::new();
            for token in tokens {
                if token.column > at_abs && token.column < equals_abs {
                    if matches!(token.token_type, TokenType::LocalVariable | TokenType::Variable) {
                        variable_tokens.push(token);
                    } else if token.token_type == TokenType::Const {
                        number_tokens.push(token);
                    }
                }
            }
            if variable_tokens.is_empty() {
                result.add_error(
                    i,
                    at_abs + 1,
                    equals_abs,
                    "CPD-3406",
                    "Invalid command syntax: expected variable name after '@'",
                );
            } else if variable_tokens.len() > 1 {
                result.add_error(
                    i,
                    at_abs + 1,
                    equals_abs,
                    "CPD-3406",
                    "Invalid command syntax: expected single variable name after '@'",
                );
            } else if !number_tokens.is_empty() {
                let first = number_tokens[0];
                result.add_error(
                    i,
                    first.column,
                    first.column + first.length,
                    "CPD-3406",
                    &format!(
                        "Invalid command syntax: unexpected number '{}' between '@' and variable name",
                        first.text
                    ),
                );
            }
        }

        if command_name.eq_ignore_ascii_case("$Plot")
            || command_name.eq_ignore_ascii_case("$Map")
            || command_name.eq_ignore_ascii_case("$Repeat")
        {
            continue;
        }
        if equals_index < 0 {
            continue;
        }

        let mut declared_var: Option<&str> = None;
        for token in tokens {
            if token.column > at_abs
                && token.column < equals_abs
                && matches!(token.token_type, TokenType::LocalVariable | TokenType::Variable)
            {
                declared_var = Some(&token.text);
                break;
            }
        }
        let Some(declared_var) = declared_var.filter(|d| !d.is_empty()) else {
            continue;
        };

        let mut used = StrSet::new();
        for token in tokens {
            if token.column >= cmd_start_index
                && token.column < at_abs
                && matches!(token.token_type, TokenType::Variable | TokenType::LocalVariable)
            {
                used.insert(&token.text);
            }
        }

        let mut uses_loop_var = used.contains(declared_var);
        if !uses_loop_var {
            let dot_suffix = format!(".{declared_var}");
            if used.iter().any(|uv| uv.ends_with(&dot_suffix)) {
                uses_loop_var = true;
            }
        }
        if used.len() > 0 && !uses_loop_var {
            let list = used.iter().map(|v| format!("'{v}'")).collect::<Vec<_>>().join(", ");
            result.add_error(
                i,
                cmd_start_index,
                at_abs,
                "CPD-3408",
                &format!("Expression uses {list} but loop variable is '{declared_var}'"),
            );
        }
    }
}

fn validate_undefined_identifiers(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    let mut directives = DirectiveState::new();

    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) {
            continue;
        }
        if line_parser::should_skip_line(line) {
            continue;
        }
        let lc = cs::chars(line);
        let trimmed = trim(&lc);

        let was_no_calc = directives.output == OutputMode::NoCalculation;
        let mut expression_start = 0;
        if line_parser::is_directive_line(trimmed) {
            directives.apply(trimmed);
            expression_start = line_parser::get_directive_expression_start(&lc);
            if expression_start < 0 {
                continue;
            }
        }
        if was_no_calc {
            continue;
        }

        let trimmed_s: String = trimmed.iter().collect();
        if patterns::function_with_command_block().is_match(&trimmed_s) {
            continue;
        }

        let tokens = tp.get_tokens_for_line(i);
        let function_params = get_function_params_from_line(line);
        let command_scope_vars = get_command_scope_variables(&lc, tokens);

        for token in tokens {
            if token.column < expression_start {
                continue;
            }
            if token.token_type == TokenType::LocalVariable || token.token_type == TokenType::FilePath {
                continue;
            }
            if token.token_type != TokenType::Variable {
                continue;
            }
            let identifier = &token.text;
            let is_element_access = identifier.ends_with('.');
            let base_name: &str = if is_element_access {
                &identifier[..identifier.len() - 1]
            } else {
                identifier
            };

            if is_being_defined(&lc, token.column, base_name) {
                continue;
            }
            if function_params.contains(base_name) {
                continue;
            }
            if command_scope_vars.contains(base_name) {
                continue;
            }

            let is_defined = stage3.defined_variables.contains(base_name)
                || stage3.defined_functions.contains_key(base_name)
                || builtins::functions().contains(base_name)
                || builtins::common_constants().contains(base_name)
                || builtins::units().contains(base_name)
                || stage3.custom_units.contains(base_name);

            if !is_defined {
                if let Some(dot_idx) = base_name.find('.').filter(|d| *d > 0) {
                    let before_dot = &base_name[..dot_idx];
                    if function_params.contains(before_dot) || command_scope_vars.contains(before_dot) {
                        let after_dot = &base_name[dot_idx + 1..];
                        result.add_error(
                            i,
                            token.column,
                            token.column + token.length,
                            "CPD-3301",
                            &format!(
                                "'{base_name}'. Function parameters require .() syntax for element access: {before_dot}.({after_dot})"
                            ),
                        );
                        continue;
                    }
                    if stage3.defined_variables.contains(before_dot) {
                        let var_info = stage3.type_tracker.get_variable_info(before_dot);
                        if var_info.is_none_or(|v| v.supports_element_access()) {
                            continue;
                        }
                    }
                }
                result.add_error(
                    i,
                    token.column,
                    token.column + token.length,
                    "CPD-3301",
                    &format!("'{base_name}'"),
                );
                continue;
            }

            if is_element_access {
                if let Some(var_info) = stage3.type_tracker.get_variable_info(base_name) {
                    if !var_info.supports_element_access() {
                        result.add_warning(
                            i,
                            token.column,
                            token.column + token.length,
                            "CPD-3306",
                            &format!(
                                "'{}' is a {}, element access .() is only valid for vectors and matrices",
                                base_name,
                                format!("{:?}", var_info.type_).to_lowercase()
                            ),
                        );
                    }
                }
            }
        }
    }
}

fn validate_undefined_units(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        if line_parser::is_directive_line_str(cs::trim_str(line)) {
            continue;
        }
        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Units {
                continue;
            }
            let unit = &token.text;
            let valid = builtins::units().contains(unit)
                || stage3.custom_units.contains(unit)
                || is_base_units_target(line, token.column);
            if !valid {
                result.add_error(
                    i,
                    token.column,
                    token.column + token.length,
                    "CPD-3310",
                    &format!("'{unit}'"),
                );
            }
        }
    }
}

/// `|base` after an expression asks for the result in SI base units (a target, not a unit name).
fn is_base_units_target(line: &str, column: i32) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let end = column.max(0) as usize;
    if end > chars.len() || chars[end..].iter().take(4).collect::<String>() != "base" {
        return false;
    }
    chars[..end].iter().rev().find(|c| !c.is_whitespace()) == Some(&'|')
}

fn get_command_scope_variables(line: &[char], tokens: &[Token]) -> StrSet {
    let mut result = StrSet::new();

    for cmd_block in ["$While", "$Block", "$Inline"] {
        let needle = format!("{cmd_block}{{");
        if let Some(cmd_index) = index_of_ic(line, &needle) {
            if let Some(brace_start) = line[cmd_index..].iter().position(|c| *c == '{').map(|p| p + cmd_index) {
                let brace_end = find_closing_brace(line, brace_start);
                extract_block_local_variables(tokens, brace_start as i32, brace_end as i32, &mut result);
            }
            break;
        }
    }

    let mut after_at = false;
    for token in tokens {
        if token.token_type == TokenType::Operator && token.text == "@" {
            after_at = true;
            continue;
        }
        if after_at && matches!(token.token_type, TokenType::LocalVariable | TokenType::Variable) {
            result.insert(&token.text);
            after_at = false;
        }
    }
    result
}

fn extract_block_local_variables(line_tokens: &[Token], brace_start: i32, brace_end: i32, result: &mut StrSet) {
    for i in 0..line_tokens.len() {
        let token = &line_tokens[i];
        if token.column <= brace_start || token.column >= brace_end {
            continue;
        }
        if token.token_type != TokenType::Variable && token.token_type != TokenType::LocalVariable {
            continue;
        }
        for j in i + 1..line_tokens.len() {
            let next = &line_tokens[j];
            if next.column >= brace_end {
                break;
            }
            if next.token_type == TokenType::Operator {
                if next.text == "=" {
                    let mut var_name = token.text.clone();
                    if var_name.ends_with('.') {
                        var_name.pop();
                    }
                    if !var_name.is_empty() {
                        result.insert(&var_name);
                    }
                }
                break;
            }
            if next.token_type == TokenType::Bracket {
                break;
            }
        }
    }
}

/// Char index of a byte offset in `s`.
fn char_index(s: &str, byte: usize) -> i32 {
    s[..byte].chars().count() as i32
}

fn validate_function_calls(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    let last_def = build_last_definition_param_counts(stage3, tp);

    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        if line_parser::is_directive_line_str(cs::trim_str(line)) {
            continue;
        }
        let lc = cs::chars(line);

        let def_match = patterns::function_definition().captures(line);
        let (def_name, def_name_col) = match &def_match {
            Some(m) => {
                let g = m.get(1).unwrap();
                (Some(g.as_str().to_string()), char_index(line, g.start()))
            }
            None => (None, -1),
        };

        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Function {
                continue;
            }
            let func_name = &token.text;
            if def_name.as_deref() == Some(func_name.as_str()) && token.column == def_name_col {
                continue;
            }
            if builtins::functions().contains(func_name) {
                continue;
            }

            if let Some(func_info) = stage3.defined_functions.get(func_name) {
                let (found, params_str) = extract_params_string(&lc, (token.column + token.length) as usize);
                if found {
                    let mut arg_list = parse_parameters(&params_str, None, 0, None);
                    if arg_list.len() == 1 && arg_list[0].is_empty() {
                        arg_list.clear();
                    }
                    let func_info: &FunctionInfo = last_def.get(func_name).unwrap_or(func_info);
                    let end_col = call_end_column(&lc, token);
                    for arg in &arg_list {
                        if let Some(kw) = unknown_keyword_arg(arg, &func_info.param_names) {
                            result.add_error(
                                i,
                                token.column,
                                end_col,
                                "CPD-3315",
                                &format!("'{kw}' is not a parameter of '{func_name}'"),
                            );
                        }
                    }
                    if let Some(message) = arg_count_message(
                        func_name,
                        arg_list.len() as i32,
                        func_info.required_param_count,
                        func_info.param_count,
                    ) {
                        result.add_error(i, token.column, end_col, "CPD-3302", &message);
                    }
                }
            } else {
                result.add_error(
                    i,
                    token.column,
                    token.column + token.length,
                    "CPD-3305",
                    &format!("'{func_name}' is not defined"),
                );
            }
        }
    }
}

fn build_last_definition_param_counts(stage3: &Stage3Context, tp: &TokenizedLineProvider) -> StrMap<FunctionInfo> {
    let mut counts: StrMap<FunctionInfo> = StrMap::new();
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let Some(m) = patterns::function_definition().captures(line) else {
            continue;
        };
        let (names, defaults) = parse_param_list(m.get(2).map_or("", |g| g.as_str()));
        counts.insert(
            m.get(1).map_or("", |g| g.as_str()),
            FunctionInfo {
                line_number: 0,
                param_count: names.len() as i32,
                required_param_count: count_required_params(names.len() as i32, defaults.as_ref()),
                param_names: names,
            },
        );
    }
    counts
}

fn validate_macro_calls(stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
    for (idx, line) in stage3.lines.iter().enumerate() {
        let i = idx as i32;
        if !tp.is_cpd_mode(i) || line_parser::should_skip_line(line) {
            continue;
        }
        let lc = cs::chars(line);
        if line_parser::is_def_statement(trim(&lc)) {
            continue;
        }
        let mut keyword_arg_names = StrSet::new();

        for token in tp.get_tokens_for_line(i) {
            if token.token_type != TokenType::Macro {
                continue;
            }
            let macro_name = &token.text;
            if !stage3.defined_macros.contains_key(macro_name) {
                if keyword_arg_names.contains(macro_name) {
                    continue;
                }
                result.add_error(
                    i,
                    token.column,
                    token.column + token.length,
                    "CPD-3303",
                    &format!("'{macro_name}'"),
                );
                continue;
            }
            if let Some(info) = stage3.defined_macros.get(macro_name) {
                let (found, params_str) = extract_params_string(&lc, (token.column + token.length) as usize);
                let raw_args: Vec<String> = if found {
                    parse_macro_parameters(&params_str)
                } else {
                    Vec::new()
                };
                let end_col = call_end_column(&lc, token);
                for arg in &raw_args {
                    if let Some(kw) = unknown_keyword_arg(arg, &info.param_names) {
                        if !stage3.defined_macros.contains_key(&kw) {
                            keyword_arg_names.insert(&kw);
                            result.add_error(
                                i,
                                token.column,
                                end_col,
                                "CPD-3314",
                                &format!("'{kw}' is not a parameter of '{macro_name}'"),
                            );
                        }
                    }
                }
                if let Some(message) = arg_count_message(
                    macro_name,
                    raw_args.len() as i32,
                    info.required_param_count,
                    info.param_count,
                ) {
                    result.add_error(i, token.column, end_col, "CPD-3304", &message);
                }
            }
        }
    }
}

/// The keyword argument's name when `arg` is `name=value` and `name` is not a parameter.
fn unknown_keyword_arg(arg: &str, param_names: &[String]) -> Option<String> {
    let (name, _) = try_parse_keyword_arg(arg)?;
    if param_names.contains(&name) {
        None
    } else {
        Some(name)
    }
}

fn call_end_column(line: &[char], token: &Token) -> i32 {
    let name_end = (token.column + token.length) as usize;
    find_closing_paren(line, name_end).max(name_end) as i32
}

fn arg_count_message(name: &str, actual: i32, required: i32, total: i32) -> Option<String> {
    if actual >= required && actual <= total {
        return None;
    }
    let expected = if required == total {
        total.to_string()
    } else {
        format!("{required}-{total}")
    };
    Some(format!("'{name}' expects {expected} parameter(s) but got {actual}"))
}

fn is_being_defined(line: &[char], identifier_start: i32, identifier: &str) -> bool {
    let start = (identifier_start.max(0) as usize).min(line.len());
    let before = trim_end(&line[..start]);
    if before.is_empty() {
        let after_pos = start + identifier.chars().count();
        if after_pos < line.len() {
            let after = trim_start(&line[after_pos..]);
            if after.first() == Some(&'=') && !(after.len() >= 2 && after[0] == '=' && after[1] == '=') {
                return true;
            }
        }
    }
    false
}

fn validate_unused_definitions(stage3: &Stage3Context, result: &mut LinterResult) {
    let mut last_assignment: StrMap<(i32, i32, i32)> = StrMap::new();
    for (name, line, column, length) in &stage3.variable_assignments {
        let replace = match last_assignment.get(name) {
            Some(existing) => *line > existing.0,
            None => true,
        };
        if replace {
            last_assignment.insert(name, (*line, *column, *length));
        }
    }

    let mut used_after = StrSet::new();
    for (name, line, _column) in &stage3.variable_usages {
        let Some(info) = last_assignment.get(name) else {
            continue;
        };
        if *line >= info.0 {
            used_after.insert(name);
        }
    }

    for (var_name, (line, column, length)) in last_assignment.iter() {
        if !used_after.contains(var_name) && !builtins::is_settings_variable(var_name) {
            result.add_information(
                *line,
                *column,
                column + length,
                "CPD-3312",
                &format!("Variable '{var_name}' is defined but never used"),
            );
        }
    }
}
