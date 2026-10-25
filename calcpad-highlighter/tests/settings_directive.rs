//! SettingsDirectiveHighlighterTests.cs
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};
use calcpad_highlighter::text::StrMap;
use calcpad_highlighter::tokenizer::models::TokenType;
use calcpad_highlighter::tokenizer::CalcpadTokenizer;

fn lint(content: &str) -> LinterResult {
    let empty: StrMap<String> = StrMap::new();
    let staged = ContentResolver::new().get_staged_content(content, Some(&empty), None, None);
    let ignore = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &ignore)
}

#[test]
fn settings_directive_tokenizes_json_payload_as_single_token() {
    let result = CalcpadTokenizer::new().tokenize("#settings {\"decimals\": 4}");
    let json_tokens: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.token_type == TokenType::SettingsJson)
        .collect();
    assert_eq!(json_tokens.len(), 1);
    assert_eq!(json_tokens[0].text, "{\"decimals\": 4}");
}

#[test]
fn settings_directive_is_recognized_directive() {
    let result = lint("#settings {\"decimals\": 4}\nx = 1");
    assert!(!result.diagnostics.iter().any(|d| d.code == "CPD-3406"));
    assert!(!result.diagnostics.iter().any(|d| d.code == "CPD-3413"));
}

#[test]
fn settings_directive_malformed_json_warns_cpd3413() {
    let result = lint("#settings {bad}\nx = 1");
    assert!(result.diagnostics.iter().any(|d| d.code == "CPD-3413"));
}

#[test]
fn settings_directive_unknown_key_warns_cpd3413() {
    let result = lint("#settings {\"nonsense\": 4}\nx = 1");
    assert!(result.diagnostics.iter().any(|d| d.code == "CPD-3413"));
}

/// Core matches keywords case-insensitively, so the payload directives must too.
#[test]
fn payload_directives_are_matched_case_insensitively() {
    let cases = [
        ("#Settings {\"decimals\": 4}", TokenType::SettingsJson),
        ("#SETTINGS {\"decimals\": 4}", TokenType::SettingsJson),
        ("#Include lib/beam.cpd", TokenType::Include),
        ("#Format 0.##", TokenType::Format),
        ("#ProjectPath work/proj", TokenType::FilePath),
        ("#LibraryPath work/lib", TokenType::FilePath),
    ];
    for (source, payload) in cases {
        let result = CalcpadTokenizer::new().tokenize(source);
        assert!(result.tokens.iter().any(|t| t.token_type == payload), "{source}");
    }
}

#[test]
fn def_directive_is_matched_case_insensitively() {
    let result = CalcpadTokenizer::new().tokenize("#Def m$(a$)");
    assert!(result
        .tokens
        .iter()
        .any(|t| t.token_type == TokenType::Macro && t.text == "m$"));
}

/// The civilkit extension key `valMathml` is a known boolean setting; a wrong type still warns.
#[test]
fn val_mathml_is_a_known_boolean_setting() {
    let ok = lint("#settings {\"valMathml\": true}\nx = 1");
    assert!(
        !ok.diagnostics.iter().any(|d| d.code == "CPD-3413"),
        "{:?}",
        ok.diagnostics
    );
    let bad = lint("#settings {\"valMathml\": 3}\nx = 1");
    assert!(bad.diagnostics.iter().any(|d| d.code == "CPD-3413"));
}

/// The civilkit extension key `unitPerElement` is a known boolean setting; a wrong type still warns.
#[test]
fn unit_per_element_is_a_known_boolean_setting() {
    let ok = lint("#settings {\"unitPerElement\": true}\nx = 1");
    assert!(
        !ok.diagnostics.iter().any(|d| d.code == "CPD-3413"),
        "{:?}",
        ok.diagnostics
    );
    let bad = lint("#settings {\"unitPerElement\": 3}\nx = 1");
    assert!(bad.diagnostics.iter().any(|d| d.code == "CPD-3413"));
}
