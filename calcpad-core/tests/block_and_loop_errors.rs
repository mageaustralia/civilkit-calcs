//! Ported from the upstream block and loop error tests.
#![allow(non_snake_case)]
mod common;

use calcpad_core::messages;
use calcpad_core::parsers::expression_parser::ExpressionParser;
use calcpad_core::parsers::macro_parser::MacroParser;
use common::NO_FILES;

fn Errors(code: &str) -> Vec<String> {
    let mut parser = ExpressionParser::new();
    parser.debug = true;
    parser.parse(code, true);
    parser
        .errors_list
        .iter()
        .map(|e| format!("{}: {}", e.source_line, e.message))
        .collect()
}

fn UnclosedLines(code: &str) -> Vec<i32> {
    let (_, expanded) = MacroParser::new(&NO_FILES).parse(code, true);
    let mut parser = ExpressionParser::new();
    parser.debug = true;
    parser.parse(&expanded, true);
    parser
        .errors_list
        .iter()
        .filter(|e| {
            e.message == messages::if_block_not_closed_Missing_end_if
                || e.message == messages::Iteration_block_not_closed_Missing_loop
        })
        .map(|e| e.source_line)
        .collect()
}

#[test]
fn UnclosedBlock_IsReportedOnItsOpeningLine() {
    for (code, line) in [
        ("x = 1\n#if x > 0\ny = 2", 2),
        ("#if 1\n#end if\n#for i = 1 : 2\ny = i\n#loop\n#if i\nz = 1", 6),
        ("#repeat 2\ny = 1\nz = 2", 1),
        ("#def m$ = 1\ny = 1\n#while y < 3\ny = y + m$", 3),
    ] {
        assert_eq!(vec![line], UnclosedLines(code), "{code:?}");
    }
}

#[test]
fn UnclosedIfAndLoop_AreBothReported() {
    let mut lines = UnclosedLines("x = 1\n#if x > 0\n#while x < 3\nx = x + 1");
    lines.sort();
    assert_eq!(vec![2, 3], lines);
}

#[test]
fn LoopPasses_ReportALineOnce() {
    let errors = Errors("#for i = 1 : 100\nx = 1m + 1s\n#loop\ny = 1 + * 2");
    assert_eq!(2, errors.len(), "{errors:?}");
    assert!(errors[1].starts_with("4: "), "{errors:?}");
}

#[test]
fn LongLoopWithOneBadLine_IsNotStopped() {
    let errors = Errors("#for i = 1 : 500\nx = 1m + 1s\n#loop\ny = 2");
    assert_eq!(1, errors.len(), "{errors:?}");
    assert!(!errors[0].contains(messages::Too_many_errors), "{errors:?}");
}

#[test]
fn ManyDifferentErrors_StopTheRun() {
    let code: String = (0..60).map(|i| format!("x{i} = 1m + 1s\n")).collect();
    let errors = Errors(&code);
    assert!(errors.last().unwrap().contains(messages::Too_many_errors), "{errors:?}");
}

#[test]
fn InvalidFormat_NamesTheFormat() {
    let errors = Errors("#format q9z\nx = 1");
    assert_eq!(1, errors.len(), "{errors:?}");
    assert!(errors[0].contains("q9z"), "{errors:?}");
    assert!(!errors[0].contains("{0}"), "{errors:?}");
}

#[test]
fn BadUiJson_ReportsOneError() {
    let errors = Errors("#UI {\"type\": bad} x = 5\ny = x + 1");
    assert_eq!(1, errors.len(), "{errors:?}");
}

#[test]
fn OversizedDatagrid_DoesNotStopTheRun() {
    let errors = Errors("#UI {\"type\": \"datagrid\", \"rows\": 100000, \"columns\": 100000} M = [1]\ny = 1 + * 2");
    assert_eq!(2, errors.len(), "{errors:?}");
    assert!(errors[1].starts_with("2: "), "{errors:?}");
}
