//! Values that are sometimes a number and sometimes a message, scripted controls and unit parse
//! cases, on synthetic worksheets written here.

use calcpad_import::{import_xmcd_opts, ImportOptions, ImportResult};

mod common;

fn sheet(regions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet version="3.0.3" xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10">
<settings><calculation><builtInVariables array-origin="1"/></calculation></settings>
<regions>{regions}</regions>
</worksheet>"#
    )
}

fn math(id: u32, top: f64, inner: &str) -> String {
    format!(
        r#"<region region-id="{id}" left="0" top="{top}" width="10" height="10" align-y="{}"><math>{inner}</math></region>"#,
        top + 10.0
    )
}

fn id(name: &str) -> String {
    format!("<ml:id>{name}</ml:id>")
}
fn ids(name: &str, sub: &str) -> String {
    format!(r#"<ml:id subscript="{sub}">{name}</ml:id>"#)
}
fn real(v: &str) -> String {
    format!("<ml:real>{v}</ml:real>")
}
fn s(v: &str) -> String {
    format!("<ml:str>{v}</ml:str>")
}
fn app(op: &str, args: &[String]) -> String {
    format!("<ml:apply><ml:{op}/>{}</ml:apply>", args.join(""))
}
fn define(target: &str, value: &str) -> String {
    format!("<ml:define>{target}{value}</ml:define>")
}
fn eval(expr: &str) -> String {
    format!("<ml:eval>{expr}</ml:eval>")
}

fn import(regions: &[String]) -> ImportResult {
    import_xmcd_opts(&sheet(&regions.concat()), &ImportOptions::default())
}

/// The engine's page for the converted text, tags removed.
fn engine_text(res: &ImportResult) -> String {
    let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    p.parse(&res.calcpad, true);
    assert!(
        !p.html_result.contains("class=\"err\"") && p.errors_list.is_empty(),
        "engine error in:\n{}\n---\n{}",
        res.calcpad,
        p.html_result
    );
    let mut out = String::new();
    let mut tag = false;
    for c in p.html_result.chars() {
        match c {
            '<' => tag = true,
            '>' => {
                tag = false;
                out.push(' ');
            }
            _ if !tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn prog(stmts: &[String]) -> String {
    format!("<ml:program>{}</ml:program>", stmts.join(""))
}
fn iff(test: &str, then: &str) -> String {
    format!("<ml:if><ml:test>{test}</ml:test><ml:then><ml:program>{then}</ml:program></ml:then></ml:if>")
}
fn local(name: &str, v: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", id(name), v)
}

/// A factor that is a number, or the message "Testing required" when c is small.
fn mixed_factor(c: &str) -> Vec<String> {
    let body = prog(&[
        iff(&app("greaterOrEqual", &[id("c"), real("8")]), &local("f", &real("1.0"))),
        iff(
            &app("lessThan", &[id("c"), real("3")]),
            &local("f", &s("Testing required")),
        ),
        iff(
            &app(
                "and",
                &[
                    app("greaterOrEqual", &[id("c"), real("3")]),
                    app("lessThan", &[id("c"), real("8")]),
                ],
            ),
            &local(
                "f",
                &app(
                    "plus",
                    &[
                        real("0.7"),
                        app("mult", &[real("0.3"), app("div", &[id("c"), real("8")])]),
                    ],
                ),
            ),
        ),
        id("f"),
    ]);
    vec![
        math(1, 0.0, &define(&id("c"), &real(c))),
        math(2, 20.0, &define(&ids("\u{3a8}", "e"), &body)),
        math(3, 40.0, &define(&id("h"), &app("div", &[real("100"), id("\u{3a8}_e")]))),
        math(4, 60.0, &eval(&id("h"))),
        math(5, 80.0, &eval(&id("\u{3a8}_e"))),
    ]
}

#[test]
fn a_value_that_is_sometimes_a_number_and_sometimes_a_message_computes_when_it_is_a_number() {
    let res = import(&mixed_factor("4"));
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    // 0.7 + 0.3 * 4 / 8 = 0.85, so h = 100 / 0.85
    let t = engine_text(&res);
    assert!(t.contains("117.647"), "{t}");
    // The evaluation of the value itself prints the number.
    assert!(t.contains("0.85"), "{t}");
}

#[test]
fn a_value_that_is_a_message_is_undefined_in_arithmetic_and_prints_the_message() {
    let res = import(&mixed_factor("2"));
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    // The message is printed for the value; the quotient that would divide by it is undefined, not 33.
    assert!(t.contains("Testing required"), "{t}");
    assert!(t.contains("h = NaN"), "{t}");
}
