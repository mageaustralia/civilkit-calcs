//! The coded messages: the code and arguments that travel with an English message so a host can show it in another
//! language (math_parser_exception::message_codes). The English text itself must not change.
use calcpad_core::exceptions;
use calcpad_core::math_parser_exception::message_codes;
use calcpad_core::messages;
use calcpad_core::netfmt::fmt;
use calcpad_core::parsers::expression_parser::ExpressionParser;

#[test]
fn a_coded_message_keeps_its_english_text_and_gains_a_code_and_arguments() {
    let e = exceptions::VariableNotExist("speed");
    assert_eq!(e.message, fmt(messages::Variable_0_does_not_exist, &["speed"]));
    assert_eq!(e.code, Some("Variable_0_does_not_exist"));
    assert_eq!(e.args, vec!["speed".to_string()]);

    let e = exceptions::InconsistentUnits("m", "s");
    assert_eq!(e.message, fmt(messages::Inconsistent_units_0_and_1, &["m", "s"]));
    assert_eq!(e.code, Some("Inconsistent_units_0_and_1"));
    assert_eq!(e.args, vec!["m".to_string(), "s".to_string()]);

    let e = exceptions::StackEmpty();
    assert_eq!(e.message, messages::Stack_empty_Invalid_expression);
    assert!(e.args.is_empty());
}

#[test]
fn a_message_that_is_not_coded_has_no_code() {
    let e = calcpad_core::MathParserException::new("something else");
    assert_eq!(e.code, None);
    assert!(e.args.is_empty());
}

/// The code of every coded constructor is the name of the constant its text comes from: the catalogue key is that name.
#[test]
fn every_code_is_the_name_of_its_message_constant() {
    let src = include_str!("../src/exceptions.rs");
    let mut n = 0;
    // The call may be written on one line or broken after the parenthesis.
    for part in src.split("MathParserException::coded(").skip(1) {
        let part = part.trim_start().strip_prefix('"').expect("the first argument is the code");
        let code = &part[..part.find('"').unwrap()];
        let rest = &part[part.find("messages::").unwrap() + "messages::".len()..];
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        assert_eq!(code, name, "the code and the constant differ");
        n += 1;
    }
    assert!(n >= 120, "only {n} coded constructors found");
}

#[test]
fn collecting_is_off_by_default_and_records_when_started() {
    let _ = exceptions::VariableNotExist("a");
    assert!(message_codes::finish().is_empty());
    message_codes::start();
    let e = exceptions::VariableNotExist("b");
    let got = message_codes::finish();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].text, e.message);
    assert_eq!(got[0].code, "Variable_0_does_not_exist");
    assert_eq!(got[0].args, vec!["b".to_string()]);
}

#[test]
fn a_failed_line_records_the_message_and_the_frame_that_holds_it() {
    message_codes::start();
    let mut parser = ExpressionParser::new();
    parser.parse("a = nosuchvariable + 1", true);
    let html = parser.html_result.clone();
    let got = message_codes::finish();
    let frame = got
        .iter()
        .find(|c| c.code.starts_with("Error_in_0"))
        .expect("a frame is recorded");
    assert!(html.contains(&frame.text), "{html}");
    let inner = got
        .iter()
        .find(|c| !c.code.starts_with("Error_in_0"))
        .expect("the message is recorded");
    assert!(frame.text.contains(&inner.text));
    assert!(frame.args.iter().any(|a| *a == inner.text));
}

#[test]
fn the_page_is_the_same_whether_or_not_messages_are_recorded() {
    let run = || {
        let mut parser = ExpressionParser::new();
        parser.parse("a = nosuchvariable + 1\nb = 2 m + 3 s", true);
        parser.html_result.clone()
    };
    let plain = run();
    message_codes::start();
    let recorded = run();
    message_codes::finish();
    assert_eq!(plain, recorded);
}
