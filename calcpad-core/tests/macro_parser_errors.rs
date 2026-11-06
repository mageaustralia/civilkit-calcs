//! Ported from the upstream macro parser error tests. Only the cases
//! that cover the nested #def and duplicate macro fixes; the rest are covered elsewhere.
#![allow(non_snake_case)]
mod common;

use calcpad_core::messages;
use calcpad_core::parsers::macro_parser::MacroParser;
use common::NO_FILES;

fn Parse(source: &str) -> (Vec<(i32, String)>, Vec<String>) {
    let mut parser = MacroParser::new(&NO_FILES);
    let (_, expanded) = parser.parse(source, false);
    let errors = parser.errors.iter().map(|e| (e.source_line, e.message.clone())).collect();
    (errors, expanded.split('\n').map(|l| l.trim().to_string()).collect())
}

#[test]
fn NestedDefInBlock_StaysInTheBlock() {
    let (errors, lines) = Parse("#def a$\n#def b$\nleak = 1\n#end def\n#end def\nx = 1");
    let messages: Vec<_> = errors.iter().map(|e| e.1.as_str()).collect();
    assert_eq!(vec![messages::Invalid_inside_macro_definition], messages);
    assert!(!lines.contains(&"leak = 1".to_string()), "{lines:?}");
    assert!(lines.contains(&"x = 1".to_string()), "{lines:?}");
}

#[test]
fn DuplicateMacro_IsReportedOnItsDefLine() {
    for (source, line) in [
        ("#def a$\n1\n#end def\n#def a$\n2\n#end def\nx = a$", 4),
        ("#def a$ = 1\n#def a$ = 2\nx = a$", 2),
    ] {
        let (errors, lines) = Parse(source);
        assert_eq!(1, errors.len(), "{source:?}: {errors:?}");
        assert_eq!(line, errors[0].0, "{source:?}");
        assert_eq!(messages::Duplicate_macro_name_0.replace("{0}", "a$"), errors[0].1);
        assert!(lines.contains(&"x = 1".to_string()), "{source:?}: {lines:?}");
    }
}
