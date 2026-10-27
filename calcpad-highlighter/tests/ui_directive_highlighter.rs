//! UiDirectiveHighlighterTests.cs
mod common;
use calcpad_highlighter::linter::models::{LinterResult, LinterSeverity};
use calcpad_highlighter::snippets::SnippetRegistry;
use calcpad_highlighter::tokenizer::models::TokenType;
use calcpad_highlighter::tokenizer::CalcpadTokenizer;

fn lint(source: &str) -> LinterResult {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ui_lint_{}_{n}.cpd", std::process::id()));
    std::fs::write(&path, source).unwrap();
    let result = common::lint_file(&path);
    let _ = std::fs::remove_file(&path);
    result
}

fn messages_for(source: &str, code: &str) -> Vec<String> {
    lint(source)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == code)
        .map(|d| d.message)
        .collect()
}

fn messages(source: &str) -> Vec<String> {
    messages_for(source, "CPD-3415")
}

/// Assert.Single: exactly one message.
fn single(msgs: Vec<String>) -> String {
    assert_eq!(msgs.len(), 1, "{msgs:?}");
    msgs.into_iter().next().unwrap()
}

fn tokens(source: &str) -> Vec<calcpad_highlighter::tokenizer::models::Token> {
    CalcpadTokenizer::new().tokenize(source).tokens
}

#[test]
fn annotated_assignment_defines_the_variable() {
    // The #UI prefix and its JSON block must not hide the definition from the linter.
    let diagnostics = lint("#UI {\"type\": \"entry\"} L = 10m\nA = L * 2m\nA");
    assert!(!diagnostics
        .diagnostics
        .iter()
        .any(|d| d.severity == LinterSeverity::Error));
}

#[test]
fn undefined_variable_is_still_reported() {
    assert!(lint("#UI L = zzzMissing * 2")
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3301" && d.message.contains("zzzMissing")));
}

#[test]
fn undefined_variable_is_reported_behind_a_json_block() {
    assert!(lint("#UI {\"type\": \"entry\"} L = zzzMissing * 2")
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3301" && d.message.contains("zzzMissing")));
}

#[test]
fn the_keyword_itself_is_not_treated_as_an_identifier() {
    assert!(!lint("#UI L = 10m\nL").diagnostics.iter().any(|d| d.code == "CPD-3301"));
}

#[test]
fn valid_directive_produces_no_diagnostic() {
    let cases = [
        "#UI L = 10m",
        "#UI {\"type\": \"checkbox\"} f = 1",
        "#UI {\"type\": \"datagrid\", \"rows\": 2, \"columns\": 2, \"columnHeaders\": [\"a\", \"b\"]} T = [1; 2 | 3; 4]",
        "#UI {\"type\": \"dropdown\", \"keys\": [\"a\", \"b\"], \"values\": [\"1\", \"2\"]} g = 1",
        "#UI {\"type\": \"radio\", \"keys\": [\"a\"], \"values\": [\"1\"]} r = 1",
        "#UI {\"style\": \"s\", \"reportStyle\": \"b\", \"mode\": \"number\"} v = 1",
    ];
    for source in cases {
        assert!(messages(source).is_empty(), "{source}");
    }
}

#[test]
fn every_snippet_is_valid_code() {
    let snippets: Vec<String> = SnippetRegistry::all_snippets()
        .iter()
        .filter(|s| s.category == "UI Inputs")
        .map(|s| s.insert.clone())
        .collect();

    assert!(!snippets.is_empty());
    for snippet in snippets {
        // CPD-3312 is inherent to a snippet on its own - nothing reads the variable it defines.
        let got: Vec<String> = lint(&snippet)
            .diagnostics
            .iter()
            .filter(|d| d.code != "CPD-3312")
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        assert!(got.is_empty(), "{snippet}: {got:?}");
    }
}

#[test]
fn unclosed_brace_is_reported() {
    assert!(single(messages("#UI {\"type\": \"entry\" a = 1")).contains("Missing closing brace"));
}

#[test]
fn malformed_json_is_reported() {
    assert!(single(messages("#UI {not json} a = 1")).contains("Invalid JSON in #UI"));
}

#[test]
fn unknown_property_is_reported() {
    assert!(single(messages("#UI {\"colour\": \"red\"} a = 1")).contains("'colour' is not a recognized"));
}

#[test]
fn unknown_type_is_reported() {
    assert!(single(messages("#UI {\"type\": \"slider\"} a = 1")).contains("'slider' is not a recognized"));
}

#[test]
fn missing_option_arrays_are_reported() {
    let cases = [
        "#UI {\"type\": \"dropdown\"} a = 1",
        "#UI {\"type\": \"radio\", \"keys\": [\"x\"]} a = 1",
        "#UI {\"type\": \"dropdown\", \"values\": [\"1\"]} a = 1",
    ];
    for source in cases {
        assert!(
            single(messages(source)).contains("requires both 'keys' and 'values'"),
            "{source}"
        );
    }
}

#[test]
fn mismatched_option_arrays_are_reported() {
    assert!(single(messages(
        "#UI {\"type\": \"dropdown\", \"keys\": [\"a\", \"b\"], \"values\": [\"1\"]} a = 1"
    ))
    .contains("2 keys but 1 values"));
}

#[test]
fn string_mode_is_reported() {
    for source in ["#UI {\"mode\": \"string\"} a = 1", "#UI a$ = 1"] {
        assert!(
            messages(source)
                .iter()
                .any(|m| m.contains("Only numbers are supported")),
            "{source}"
        );
    }
}

#[test]
fn missing_assignment_is_reported() {
    assert!(single(messages("#UI {\"type\": \"entry\"}")).contains("requires a variable assignment"));
}

#[test]
fn assigned_expression_is_reported() {
    let cases = [
        "#UI {\"type\": \"entry\"} k = sin(E)",
        "#UI k = 2*E",
        "#UI k = E",
        "#UI k = 1 + 2",
        "#UI Z = 2*vector(3)",
        "#UI {\"type\": \"entry\"} k = sin(2)",
    ];
    for source in cases {
        assert!(
            messages(source)
                .iter()
                .any(|m| m.contains("does not support expressions")),
            "{source}"
        );
    }
}

#[test]
fn assigned_value_is_accepted() {
    let cases = [
        "#UI L = 10m",
        "#UI q = -3.5kN/m",
        "#UI I = 2500000mm^4",
        "#UI a = 9.81m/s^2",
        "#UI t = 12\u{b0}",
        "#UI r = 50%",
        "#UI v = [1; 2m; 3]",
        "#UI M = [1; 2 | 3; 4]",
        "#UI Z = vector(5)",
        "#UI G = matrix(3; 4)",
        // A computed size is sized from the value the call produces, so it is a grid too.
        "#UI Z = vector(n)",
        "#UI G = matrix(r; c)",
        "#UI G = matrix(len(x); len(y))",
        "#UI '2&middot;<i>r</i> ='d = 1",
        // A grid replaces its whole right hand side when edited, so anything may seed it.
        "#UI v = [1; 2*E]",
        "#UI G = matrix(2; 2) + 1",
        // So does any control that allows expressions.
        "#UI {\"allowExpression\": true} k = 1 + 2",
        "#UI {\"allowExpression\": true} Z = 2*vector(3)",
    ];
    for source in cases {
        assert!(messages(source).is_empty(), "{source}: {:?}", messages(source));
    }
}

#[test]
fn negative_grid_size_is_reported() {
    let cases = [
        ("#UI {\"rows\": -1} a = [1; 2]", "'rows' must not be negative"),
        ("#UI {\"columns\": -2} a = [1; 2]", "'columns' must not be negative"),
    ];
    for (source, expected) in cases {
        assert!(single(messages(source)).contains(expected), "{source}");
    }
}

#[test]
fn declared_type_against_the_value_is_reported() {
    let cases = [
        (
            "#UI {\"type\": \"datagrid\"} x = 5",
            "'datagrid' requires a vector or matrix",
        ),
        (
            "#UI {\"type\": \"datagrid\"} x = 5m",
            "'datagrid' requires a vector or matrix",
        ),
        (
            "#UI {\"type\": \"entry\"} v = [1; 2]",
            "'entry' cannot hold a vector or matrix",
        ),
        (
            "#UI {\"type\": \"checkbox\"} f = matrix(2; 2)",
            "'checkbox' cannot hold a vector or matrix",
        ),
    ];
    for (source, expected) in cases {
        assert!(single(messages(source)).contains(expected), "{source}");
    }
}

#[test]
fn declared_type_against_the_value_is_not_reported_when_the_value_can_be_it() {
    let cases = [
        "#UI {\"type\": \"datagrid\"} M = 2*vector(3)",
        "#UI {\"type\": \"datagrid\"} M = A",
        "#UI {\"type\": \"entry\"} x = 5m",
        "#UI {\"type\": \"entry\", \"allowExpression\": true} v = [1; 2]",
    ];
    for source in cases {
        assert!(messages(source).is_empty(), "{source}");
    }
}

#[test]
fn grid_property_on_another_control_is_reported() {
    let cases = [
        // Declared on another type.
        (
            "#UI {\"type\": \"entry\", \"width\": 300} x = 1",
            "'width' only applies to a datagrid",
        ),
        (
            "#UI {\"type\": \"checkbox\", \"rowHeaderWidth\": 40} x = 1",
            "'rowHeaderWidth' only applies to a datagrid",
        ),
        (
            "#UI {\"type\": \"dropdown\", \"keys\": [\"A\"], \"values\": [\"1\"], \"columnWidths\": [2, 1]} x = 1",
            "'columnWidths' only applies to a datagrid",
        ),
        // Or on a right hand side that is not a grid.
        ("#UI {\"width\": \"100%\"} x = 1", "'width' only applies to a datagrid"),
        (
            "#UI {\"rowHeaders\": [\"r1\"]} x = 1",
            "'rowHeaders' only applies to a datagrid",
        ),
        ("#UI {\"rows\": 2} x = 1", "'rows' only applies to a datagrid"),
    ];
    for (source, expected) in cases {
        assert!(single(messages(source)).contains(expected), "{source}");
    }
}

#[test]
fn grid_property_on_a_grid_is_accepted() {
    let cases = [
        "#UI {\"type\": \"datagrid\", \"width\": 300, \"rowHeaderWidth\": 40} M = matrix(2; 2)",
        // The type is auto-detected from the right hand side.
        "#UI {\"width\": 300, \"columnWidths\": [2, 1]} T = [1; 2]",
        "#UI {\"rows\": 2, \"columns\": 2} M = matrix(2; 2)",
    ];
    for source in cases {
        assert!(messages(source).is_empty(), "{source}");
    }
}

#[test]
fn more_headers_than_cells_is_reported() {
    let cases = [
        (
            "#UI {\"columns\": 2, \"columnHeaders\": [\"a\", \"b\", \"c\"]} T = [1; 2]",
            "3 entries but the grid has 2 columns",
        ),
        (
            "#UI {\"rows\": 1, \"rowHeaders\": [\"r1\", \"r2\"]} T = [1; 2]",
            "2 entries but the grid has 1 rows",
        ),
    ];
    for (source, expected) in cases {
        assert!(single(messages(source)).contains(expected), "{source}");
    }
}

/// An omitted size is auto-detected from the right hand side, which the JSON block cannot see, so
/// headers are only counted against a size the payload declared.
#[test]
fn header_count_is_not_reported_without_a_declared_size() {
    let cases = [
        "#UI {\"columnHeaders\": [\"a\", \"b\", \"c\"]} T = [1; 2]",
        "#UI {\"columns\": 3, \"columnHeaders\": [\"a\"]} T = [1; 2; 3]",
    ];
    for source in cases {
        assert!(messages(source).is_empty(), "{source}");
    }
}

#[test]
fn wrong_value_type_is_reported() {
    let cases = [
        "#UI {\"rows\": \"two\"} a = 1",
        "#UI {\"keys\": \"a\"} a = 1",
        "#UI {\"style\": 5} a = 1",
    ];
    for source in cases {
        assert!(single(messages(source)).contains("wrong type"), "{source}");
    }
}

/// The first identifier on a line is a definition even when its name happens to be a unit ('a' is
/// Are, 'L' is litre). A directive keyword sits before that position and must not consume it.
#[test]
fn unit_named_variable_is_defined_not_a_unit() {
    let cases = [
        ("#UI a = 5", "a"),
        ("#UI L = 10m", "L"),
        ("#UI {\"type\": \"entry\"} a = 5", "a"),
        ("#UI 'label ='a = 5", "a"),
        ("#const h = 6", "h"),
        ("a = 5", "a"),
    ];
    for (source, name) in cases {
        let toks = tokens(source);
        assert!(
            toks.iter()
                .any(|t| t.token_type == TokenType::Variable && t.text == name),
            "{source}"
        );
        assert!(
            !toks.iter().any(|t| t.token_type == TokenType::Units && t.text == name),
            "{source}"
        );
    }
}

#[test]
fn units_after_the_first_identifier_stay_units() {
    let cases = [
        ("#UI x = 5*ft", "ft"),
        ("#const y = 5*ft", "ft"),
        ("#UI L = 10m", "m"),
        ("A = 5*a", "a"),
    ];
    for (source, unit) in cases {
        assert!(
            tokens(source)
                .iter()
                .any(|t| t.token_type == TokenType::Units && t.text == unit),
            "{source}"
        );
    }
}

#[test]
fn unit_named_variable_does_not_cascade_to_later_uses() {
    let diagnostics = lint("#UI a = 5\nb = a * 2\nb").diagnostics;
    assert!(!diagnostics.iter().any(|d| d.severity == LinterSeverity::Error));
}

#[test]
fn ui_is_a_recognized_keyword() {
    assert!(!lint("#UI L = 10m\nL").diagnostics.iter().any(|d| d.code == "CPD-3403"));
}

/// The block shares the SettingsJson token with #settings so both colour the same, but #UI's ends
/// at the closing brace rather than at the end of the line.
#[test]
fn json_block_is_a_single_settings_json_token() {
    let cases = [
        ("#UI {\"type\": \"entry\"} L = 10m", "{\"type\": \"entry\"}"),
        (
            "#ui {\"style\": \"highlight\"} depth = 2m",
            "{\"style\": \"highlight\"}",
        ),
        (
            "#UI {\"type\": \"datagrid\", \"columnHeaders\": [\"a\", \"b\"]} T = [1; 2 | 3; 4]",
            "{\"type\": \"datagrid\", \"columnHeaders\": [\"a\", \"b\"]}",
        ),
    ];
    for (source, json) in cases {
        let toks = tokens(source);
        let found: Vec<_> = toks
            .iter()
            .filter(|t| t.token_type == TokenType::SettingsJson)
            .collect();
        assert_eq!(found.len(), 1, "{source}");
        assert_eq!(found[0].text, json, "{source}");
    }
}

#[test]
fn without_a_json_block_no_settings_json_token() {
    assert!(!tokens("#UI L = 10m")
        .iter()
        .any(|t| t.token_type == TokenType::SettingsJson));
}

#[test]
fn after_the_json_block_the_matrix_still_tokenizes() {
    let toks = tokens("#UI {\"type\": \"datagrid\"} T = [1; 2 | 3; 4]");
    assert!(toks
        .iter()
        .any(|t| t.token_type == TokenType::Variable && t.text == "T"));
    assert_eq!(toks.iter().filter(|t| t.token_type == TokenType::Const).count(), 4);
}

/// #UI's keys are not #settings' keys, so SettingsValidator must skip them.
#[test]
fn json_block_is_not_validated_against_settings_keys() {
    assert!(!lint("#UI {\"type\": \"entry\"} L = 10m\nL")
        .diagnostics
        .iter()
        .any(|d| d.code == "CPD-3413"));
}

// An addition to the C# tests (DEVIATIONS.md, D17): the bounds and the step of a number entry.
#[test]
fn bounds_and_step_of_an_entry_are_known_properties() {
    assert!(messages("#UI {\"min\": 1, \"max\": 5, \"step\": 0.5} a = 2").is_empty());
    assert!(messages("#UI {\"type\": \"entry\", \"min\": 1} a = 2").is_empty());
}

#[test]
fn a_bad_range_is_reported_by_the_linter() {
    assert!(single(messages("#UI {\"min\": 5, \"max\": 1} a = 2")).contains("'min' (5) must not be above 'max' (1)"));
    assert!(single(messages("#UI {\"max\": 5, \"step\": 0} a = 2")).contains("'step' must be above zero"));
    assert!(single(messages("#UI {\"type\": \"checkbox\", \"max\": 5} a = 1")).contains("'max' only applies to an entry"));
}
