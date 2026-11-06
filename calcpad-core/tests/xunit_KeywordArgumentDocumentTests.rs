//! Transcribed from Calcpad.Tests/ExpressionParser/KeywordArgumentDocumentTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::useless_format,
    clippy::clone_on_copy,
    unused_imports,
    dead_code,
    suspicious_double_ref_op,
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

// Mirrors the real pipeline: macro expansion, then expression parsing.
fn Render(source: &str) -> String {
    let macro_parser = TestMacroParser::new();
    let (has_errors, unwrapped) = macro_parser.Parse(source);
    assert!(!has_errors, "{unwrapped}");
    let mut parser = new_expression_parser(&[]);
    parser.parse(&unwrapped, true);
    parser.html_result
}

#[test]
fn FunctionDefaults_RenderThroughTheFullPipeline() {
    let mut html = Render("f(x; y = 2; z = 3) = x + y + z\nr = f(1)\ns = f(1; z = 10)\nt = f(z = 10; x = 1; y = 2)");
    assert_does_not_contain("Error", &html);
    assert_contains("6", &html);
    assert_contains("13", &html);
}
#[test]
fn MacroDefaults_RenderThroughTheFullPipeline() {
    let mut html = Render("#def m$(a$; b$ = 10; c$ = 5) = a$ + b$ + c$\nr = m$(1)\ns = m$(1; c$ = 7)");
    assert_does_not_contain("Error", &html);
    assert_contains("16", &html);
    assert_contains("18", &html);
}
#[test]
fn MacroDefaultsFeedingAFunctionWithDefaults() {
    let mut html = Render("f(x; k = 2) = x*k\n#def call$(v$; f$ = 3) = f(v$; f$)\nr = call$(4)\ns = call$(4; f$ = 5)");
    assert_does_not_contain("Error", &html);
    assert_contains("12", &html);
    assert_contains("20", &html);
}
#[test]
fn CommandBlockFunctionDefaults_RenderThroughTheFullPipeline_case1() {
    let block = "$Inline";
    let mut html = Render(&format!(
        "f(x; y = 2; z = 3) = {}{{a = x*100 + y*10 + z}}\nr = f(1)\ns = f(1; z = 7)\nt = f(z = 5; x = 4; y = 6)",
        block
    ));
    assert_does_not_contain("Error", &html);
    assert_contains("123", &html);
    assert_contains("127", &html);
    assert_contains("465", &html);
}
#[test]
fn CommandBlockFunctionDefaults_RenderThroughTheFullPipeline_case2() {
    let block = "$Block";
    let mut html = Render(&format!(
        "f(x; y = 2; z = 3) = {}{{a = x*100 + y*10 + z}}\nr = f(1)\ns = f(1; z = 7)\nt = f(z = 5; x = 4; y = 6)",
        block
    ));
    assert_does_not_contain("Error", &html);
    assert_contains("123", &html);
    assert_contains("127", &html);
    assert_contains("465", &html);
}
#[test]
fn DefinitionRendersDefaults() {
    let mut html = Render("f(x; y = 2) = x + y");
    assert_contains("<var>y</var> = 2", &html);
}
#[test]
fn CallRendersAsWritten() {
    let mut html = Render("f(x; y = 2; z = 3) = x + y + z\nr = f(1; z = 10)");
    assert_contains("1; <var>z</var> = 10", &html);
    assert_does_not_contain("1; 2; 10", &html);
}
#[test]
fn DoubleEquals_IsNotAKeywordArgument() {
    for source in [
        "f(a; b) = a + b\na = 1\ny = f(a == 1; 2)",
        "f(x; b) = x + b\na = 1\ny = f(a == 1; 2)",
    ] {
        let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
        parser.debug = true;
        parser.parse(source, true);
        assert_eq!(1, parser.errors_list.len(), "{source:?}: {:?}", parser.errors_list);
        assert!(!parser.errors_list[0].message.to_lowercase().contains("argument"));
    }
}
