//! UiValidator.cs: the JSON block of `#UI {...} name = value`.
use crate::linter::helpers::cs;
use crate::linter::helpers::directive_json_reporter::DirectiveJsonReporter;
use crate::linter::helpers::tokenized_line_provider::TokenizedLineProvider;
use crate::linter::models::{LinterResult, Stage3Context};
use crate::text::is_white_space;
use calcpad_core::messages;
use calcpad_core::ui_dto::UiDto;
use calcpad_core::ui_syntax;

const CODE: &str = "CPD-3415";
const KEYWORD: &str = "#ui";

/// `UiKey` names: the recognised `#UI` properties.
const UI_KEY_NAMES: [&str; 18] = [
    "Type",
    "Mode",
    "Style",
    "ReportStyle",
    "ForceUnits",
    "AllowExpression",
    "Rows",
    "Columns",
    "ColumnHeaders",
    "RowHeaders",
    "Width",
    "RowHeaderWidth",
    "ColumnWidths",
    "Keys",
    "Values",
    // An addition to the C# payload (DEVIATIONS.md, D17): the bounds and step of a number entry.
    "Min",
    "Max",
    "Step",
];

#[derive(Default)]
pub struct UiValidator;

impl UiValidator {
    pub fn new() -> Self {
        UiValidator
    }

    pub fn validate(&self, stage3: &Stage3Context, result: &mut LinterResult, tp: &TokenizedLineProvider) {
        for (idx, line) in stage3.lines.iter().enumerate() {
            let i = idx as i32;
            if !tp.is_cpd_mode(i) {
                continue;
            }
            let lc = cs::chars(line);
            let start = lc.iter().take_while(|c| is_white_space(**c)).count();
            if !cs::starts_with_ic(&lc[start..], KEYWORD) {
                continue;
            }
            validate_ui_line(i, &lc, start + KEYWORD.len(), result);
        }
    }
}

fn reporter(line_index: i32, from: usize, to: usize) -> DirectiveJsonReporter {
    DirectiveJsonReporter::new(line_index, from as i32, to as i32, CODE, "#UI")
}

fn validate_ui_line(line_index: i32, line: &[char], after_keyword: usize, result: &mut LinterResult) {
    let mut cursor = after_keyword;
    while cursor < line.len() && line[cursor] == ' ' {
        cursor += 1;
    }
    let end = line.len();
    if cursor >= end {
        reporter(line_index, after_keyword, end).warn(result, messages::The_UI_keyword_requires_a_variable_assignment);
        return;
    }

    let mut json_end = cursor;
    let mut properties: Option<UiDto> = None;
    if line[cursor] == '{' {
        let Some(rel) = line[cursor..].iter().position(|c| *c == '}') else {
            reporter(line_index, cursor, end)
                .warn(result, messages::Improper_format_for_UI_keyword_Missing_closing_brace);
            return;
        };
        let brace_end = cursor + rel;
        let json: String = line[cursor..=brace_end].iter().collect();
        match validate_json(&json, &reporter(line_index, cursor, brace_end + 1), result) {
            None => return,
            Some(p) => properties = p,
        }
        json_end = brace_end + 1;
    }

    let rep = reporter(line_index, json_end, end);
    let rest: String = line[json_end..].iter().collect();
    let assignments = ui_syntax::enumerate_assignments(&rest);
    if assignments.is_empty() {
        rep.warn(result, messages::The_UI_keyword_requires_a_variable_assignment);
        return;
    }
    let declared = properties.as_ref().and_then(|p| p.ty.as_deref());
    let allow_expression = properties.as_ref().is_some_and(|p| p.allows_expression());
    for (name, rhs) in &assignments {
        if name.ends_with('$') {
            rep.warn(result, messages::Only_numbers_are_supported_by_the_UI_keyword);
            return;
        }
        let ty = ui_syntax::resolve_type(declared, rhs);
        if !ui_syntax::is_value_for(rhs, &ty, allow_expression) {
            rep.warn(result, messages::UI_expressions_require_allowExpression);
            return;
        }
        if let Some(mismatch) = ui_syntax::type_mismatch(declared, rhs, allow_expression) {
            rep.warn(result, &mismatch);
            return;
        }
        if let Some(p) = &properties {
            let type_errors = p.validate_resolved_type(&ty);
            if !type_errors.is_empty() {
                for e in type_errors {
                    rep.warn(result, &e);
                }
                return;
            }
        }
    }
}

/// None when the block is malformed (a diagnostic was reported), otherwise the deserialised
/// payload (None inside when it could not be deserialised).
fn validate_json(json: &str, rep: &DirectiveJsonReporter, result: &mut LinterResult) -> Option<Option<UiDto>> {
    let doc = rep.try_parse(result, json)?;
    rep.check_known_keys(
        result,
        &doc,
        |n| UI_KEY_NAMES.iter().any(|k| crate::text::eq_ignore_case(k, n)),
        "#UI property",
    );
    let properties = match UiDto::parse(json) {
        Ok(Some(p)) => p,
        Ok(None) => return Some(None),
        Err(_) => {
            // The checks below only need where the block ends, not a deserialised payload.
            rep.warn(result, messages::A_UI_value_has_the_wrong_type);
            return Some(None);
        }
    };
    for error in properties.validate() {
        rep.warn(result, &error);
    }
    Some(Some(properties))
}
