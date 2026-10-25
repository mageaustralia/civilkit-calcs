//! Values that are sometimes a number and sometimes a message, scripted controls and unit parse
//! cases, on synthetic worksheets written here.

use calcpad_import::{import_xmcd_opts, Code, ImportOptions, ImportResult};

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
fn call(name: &str, args: &[String]) -> String {
    format!(
        "<ml:apply>{}<ml:sequence>{}</ml:sequence></ml:apply>",
        id(name),
        args.join("")
    )
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

// ---- scripted controls that store a value --------------------------------------------------------

fn b64(s: &str) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = s.as_bytes();
    let mut out = String::new();
    for ch in b.chunks(3) {
        let n = (ch[0] as u32) << 16 | (*ch.get(1).unwrap_or(&0) as u32) << 8 | *ch.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= ch.len() {
                out.push(T[((n >> (18 - 6 * k)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn button(rl: &str, caption: &str) -> String {
    format!(
        r#"<ml:ButtonScriptableControl SelectedIndex="0" RL="{}"><ml:vals><ml:val>{caption}</ml:val></ml:vals></ml:ButtonScriptableControl>"#,
        b64(rl)
    )
}

#[test]
fn a_button_that_stores_a_counter_defines_it() {
    let res = import(&[
        math(
            1,
            0.0,
            &define(&id("Refresh"), &button("(number 3:0x00000003)", "Refresh")),
        ),
        math(2, 20.0, &define(&id("n"), &app("plus", &[id("Refresh"), real("1")]))),
        math(3, 40.0, &eval(&id("n"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("Refresh = 3"), "{}", res.calcpad);
    assert!(res.calcpad.contains("button input"), "{}", res.calcpad);
    assert!(engine_text(&res).contains("n = 4"));
}

#[test]
fn a_button_on_its_own_is_its_label() {
    let res = import(&[math(1, 0.0, &button("(string \"\":0x00000001)", "Calculate"))]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("[Calculate]"), "{}", res.calcpad);
}

#[test]
fn a_control_with_several_outputs_defines_each_name() {
    let names = "<ml:matrix rows=\"2\" cols=\"1\"><ml:id>Flag</ml:id><ml:id>Folder</ml:id></ml:matrix>";
    let c = button(
        "(op_matrix:0x5 (unboxed 2) (unboxed 1) (list (number 1:0x5) (string \"a\\\\b\":0x5)))",
        "Go",
    );
    let res = import(&[
        math(1, 0.0, &define(names, &c)),
        math(2, 20.0, &define(&id("n"), &app("plus", &[id("Flag"), real("1")]))),
        math(3, 40.0, &eval(&id("n"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("Flag = 1"), "{}", res.calcpad);
    assert!(engine_text(&res).contains("n = 2"));
}

// ---- an expression on its own ---------------------------------------------------------------------

#[test]
fn a_name_on_its_own_with_no_saved_result_is_shown_as_written() {
    let res = import(&[
        math(1, 0.0, &id("Width")),
        math(2, 20.0, &app("mult", &[real("20"), id("ft")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert_eq!(
        res.report
            .issues
            .iter()
            .filter(|i| i.code == Code::InertExpression)
            .count(),
        2,
        "{:?}",
        res.report.issues
    );
    assert!(res.calcpad.contains("Width"), "{}", res.calcpad);
    engine_text(&res);
}

#[test]
fn a_comparison_on_its_own_stays_a_gap() {
    let res = import(&[
        math(1, 0.0, &define(&id("a"), &real("1"))),
        math(2, 20.0, &app("greaterOrEqual", &[id("a"), real("2")])),
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::BareExpression));
}

// ---- vectors of texts ---------------------------------------------------------------------------------

fn indexed(name: &str, i: &str) -> String {
    format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", id(name), i)
}

fn text_vector_sheet() -> ImportResult {
    let element = |text: &str| {
        format!(
            "<ml:localDefine>{}<ml:str>{text}</ml:str></ml:localDefine>",
            indexed("msg", "<ml:id labels=\"*\">i</ml:id>")
        )
    };
    let body = format!(
        "<ml:program><ml:for><ml:id>i</ml:id><ml:range><ml:real>1</ml:real><ml:real>2</ml:real></ml:range><ml:program>{}</ml:program></ml:for>{}</ml:program>",
        format!(
            "<ml:if><ml:test>{}</ml:test><ml:then><ml:program>{}</ml:program></ml:then><ml:else><ml:program>{}</ml:program></ml:else></ml:if>",
            app("greaterOrEqual", &[indexed("x", &id("i")), real("2")]),
            element("OK"),
            element("NG")
        ),
        id("msg")
    );
    import(&[
        math(
            1,
            0.0,
            &define(
                &id("x"),
                "<ml:matrix rows=\"2\" cols=\"1\"><ml:real>3</ml:real><ml:real>1</ml:real></ml:matrix>",
            ),
        ),
        math(2, 20.0, &define(&id("Chk"), &body)),
        math(
            3,
            40.0,
            &define(
                &id("n"),
                &call(
                    "if",
                    &[
                        app("equal", &[indexed("Chk", &real("2")), s("NG")]),
                        real("10"),
                        real("20"),
                    ],
                ),
            ),
        ),
        math(4, 60.0, &eval(&id("Chk"))),
        math(5, 80.0, &eval(&id("n"))),
    ])
}

#[test]
fn a_vector_of_texts_is_held_as_codes_and_prints_its_texts() {
    let res = text_vector_sheet();
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    // The second element is "NG": the comparison with the text holds, and the column prints both.
    assert!(t.contains("n = 10"), "{t}");
    assert!(t.contains("OK") && t.contains("NG"), "{t}");
}

#[test]
fn a_vector_that_mixes_texts_and_numbers_stays_a_gap() {
    let element = |value: &str| {
        format!(
            "<ml:localDefine>{}{value}</ml:localDefine>",
            indexed("val", "<ml:id labels=\"*\">i</ml:id>")
        )
    };
    let body = format!(
        "<ml:program><ml:for><ml:id>i</ml:id><ml:range><ml:real>1</ml:real><ml:real>2</ml:real></ml:range><ml:program>{}{}</ml:program></ml:for>{}</ml:program>",
        element("<ml:str>n/a</ml:str>"),
        element("<ml:real>4</ml:real>"),
        id("val")
    );
    let res = import(&[math(1, 0.0, &define(&id("V"), &body))]);
    assert!(
        res.report.gaps().any(|i| i.code == Code::StringValue),
        "{:?}",
        res.report.issues
    );
}
