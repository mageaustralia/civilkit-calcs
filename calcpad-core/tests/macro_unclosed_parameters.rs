//! A `#def` whose parameter list ends before the closing bracket reports an error and returns.
#![allow(non_snake_case)]
mod common;
use common::*;

fn parse(source: &str) -> (bool, String) {
    TestMacroParser::new().Parse(source)
}

#[test]
fn UnclosedParameterList_ReportsBracketsNotClosed() {
    for source in ["#def m$(a$\ny = 3\n", "#def m$(a$", "#def m$(a$; b$", "#def m$(a$;\n", "#def m$(a$ = 1\n"] {
        let (has_errors, text) = parse(source);
        assert!(has_errors, "{source:?}: {text}");
    }
}

#[test]
fn ClosedParameterList_StillExpands() {
    let (has_errors, text) = parse("#def m$(a$; b$) = a$ + b$\nm$(1; 2)\n");
    assert!(!has_errors, "{text}");
    assert!(text.contains("1 + 2"), "{text}");
}

#[test]
fn AsecAcscAtZero_CarryTheAngleUnit() {
    for f in ["asec", "acsc"] {
        let mut calc = TestCalc::new(ms().Degrees(0.0));
        calc.Run("ReturnAngleUnits = 1");
        calc.Run(format!("{f}(0)").as_str());
        let result = calc.ToString();
        assert!(result.contains('\u{b0}'), "{f}(0) = {result}");
    }
}

#[test]
fn RedefiningABuiltInFunction_NamesTheFunction() {
    let html = render("sin(x) = x\n");
    assert_contains("\"sin\" is a built-in function and cannot be redefined", &html);
}
