//! Programs, array assignments and matrix functions: each case builds a small invented worksheet,
//! converts it, runs the produced Calcpad text through the engine and compares the printed numbers
//! with values worked out by hand.

use calcpad_import::{import_xmcd, Code, ImportResult};

mod common;
use common::legacy;

fn sheet_of(inners: &[String]) -> String {
    let regions: String = inners
        .iter()
        .enumerate()
        .map(|(i, inner)| {
            let top = i as f64 * 20.0;
            format!(
                r#"<region region-id="{}" left="0" top="{top}" width="10" height="10" align-y="{}"><math>{inner}</math></region>"#,
                i + 1,
                top + 10.0
            )
        })
        .collect();
    format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="1"/></calculation></settings><regions>{regions}</regions></worksheet>"#
    )
}

fn n(name: &str) -> String {
    format!("<ml:id>{name}</ml:id>")
}
fn r(v: &str) -> String {
    format!("<ml:real>{v}</ml:real>")
}
fn app(op: &str, args: &[String]) -> String {
    format!("<ml:apply><ml:{op}/>{}</ml:apply>", args.join(""))
}
fn call(name: &str, args: &[String]) -> String {
    format!(
        "<ml:apply>{}<ml:sequence>{}</ml:sequence></ml:apply>",
        n(name),
        args.join("")
    )
}
fn define(name: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", n(name), value)
}
fn eval(expr: &str) -> String {
    format!("<ml:eval>{expr}</ml:eval>")
}
fn func(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| n(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        n(name)
    )
}
fn program(stmts: &[String]) -> String {
    format!("<ml:program>{}</ml:program>", stmts.join(""))
}
fn local(name: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", n(name), value)
}
fn index1(base: &str, idx: &str) -> String {
    format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", n(base), idx)
}
fn index2(base: &str, i: &str, j: &str) -> String {
    format!(
        "<ml:apply><ml:indexer/>{}<ml:sequence>{i}{j}</ml:sequence></ml:apply>",
        n(base)
    )
}
fn mat(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}
fn gt(a: &str, b: &str) -> String {
    app("greaterThan", &[a.to_string(), b.to_string()])
}
fn lt(a: &str, b: &str) -> String {
    app("lessThan", &[a.to_string(), b.to_string()])
}
fn plus(a: &str, b: &str) -> String {
    app("plus", &[a.to_string(), b.to_string()])
}
fn times(a: &str, b: &str) -> String {
    app("mult", &[a.to_string(), b.to_string()])
}

/// `if test then <then>` with `alsoif`/`else` arms, in the package format's spelling.
fn if_chain(arms: &[(String, Vec<String>)], otherwise: Option<&[String]>) -> String {
    let mut out = String::from("<ml:if>");
    for (k, (test, body)) in arms.iter().enumerate() {
        let (open, close) = if k == 0 {
            ("", "")
        } else {
            ("<ml:alsoif>", "</ml:alsoif>")
        };
        out.push_str(&format!(
            "{open}<ml:test>{test}</ml:test><ml:then>{}</ml:then>{close}",
            program(body)
        ));
    }
    if let Some(o) = otherwise {
        out.push_str(&format!("<ml:else>{}</ml:else>", program(o)));
    }
    out.push_str("</ml:if>");
    out
}
fn if_then(test: &str, then: &[String]) -> String {
    if_chain(&[(test.to_string(), then.to_vec())], None)
}

fn convert(inners: &[String]) -> ImportResult {
    import_xmcd(&sheet_of(inners))
}

fn engine_lines(code: &str) -> Vec<String> {
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{}", legacy(code)), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "engine error in:\n{code}\n---\n{html}");
    html.replace("<p>", "\n<p>")
        .lines()
        .map(|l| {
            let mut out = String::new();
            let mut in_tag = false;
            for c in l.chars() {
                match c {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    c if !in_tag => out.push(c),
                    _ => {}
                }
            }
            out.replace("&#8722;", "-").trim().to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

fn try_last_number(line: &str) -> Option<f64> {
    let tail = line.rsplit('=').next().unwrap();
    let digits: String = tail
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | 'E' | '+'))
        .collect();
    digits.parse().ok()
}

fn numbers(code: &str) -> Vec<f64> {
    engine_lines(code)
        .iter()
        .filter(|l| l.contains('=') && !l.contains('['))
        .filter_map(|l| try_last_number(l))
        .collect()
}

fn assert_close(got: f64, want: f64) {
    assert!(
        (got - want).abs() <= 1e-9 * want.abs().max(1.0),
        "got {got}, want {want}"
    );
}

fn clean(res: &ImportResult) {
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
}

// ---- programs whose value is the last expression run ------------------------------------------------------

#[test]
fn sequential_ifs_give_the_last_expression_that_ran() {
    // f(x) = (if x > 0: 10; if x > 5: 20): the value is the last expression run.
    let body = program(&[
        if_then(&gt(&n("x"), &r("0")), &[r("10")]),
        if_then(&gt(&n("x"), &r("5")), &[r("20")]),
    ]);
    let res = convert(&[
        func("f", &["x"], &body),
        eval(&call("f", &[r("7")])),
        eval(&call("f", &[r("3")])),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 20.0);
    assert_close(nums[nums.len() - 1], 10.0);
}

#[test]
fn a_path_with_no_value_is_undefined_not_zero() {
    let body = program(&[if_then(&gt(&n("x"), &r("0")), &[r("10")])]);
    let res = convert(&[func("f", &["x"], &body), eval(&call("f", &[r("-2")]))]);
    clean(&res);
    // The engine shows the not-a-number result as an error span, so read the page directly.
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{}", legacy(&res.calcpad)), true);
    assert!(parser.html_result.contains("Undefined"), "{}", parser.html_result);
}

#[test]
fn alsoif_is_else_if() {
    // The package format's "else if": the first true test wins.
    let chain = if_chain(
        &[
            (lt(&n("x"), &r("0")), vec![r("1")]),
            (lt(&n("x"), &r("10")), vec![r("2")]),
        ],
        Some(&[r("3")]),
    );
    let body = program(&[local("y", &times(&n("x"), &r("1"))), chain]);
    let res = convert(&[
        func("g", &["x"], &body),
        eval(&call("g", &[r("-4")])),
        eval(&call("g", &[r("5")])),
        eval(&call("g", &[r("50")])),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    let k = nums.len();
    assert_close(nums[k - 3], 1.0);
    assert_close(nums[k - 2], 2.0);
    assert_close(nums[k - 1], 3.0);
}

#[test]
fn a_program_that_ends_in_an_assignment_after_its_value_is_not_guessed() {
    let body = program(&[r("1"), local("z", &r("2"))]);
    let res = convert(&[func("h", &["x"], &body)]);
    assert!(res.report.gaps().any(|i| i.code == Code::ProgramBlock));
}

#[test]
fn unused_helpers_stay_silent() {
    // Keeps the builders above in use.
    let _ = (
        plus("a", "b"),
        mat(1, 1, &["1"]),
        index1("a", "b"),
        index2("a", "b", "c"),
    );
}

// ---- column and row extraction ------------------------------------------------------------------------------

#[test]
fn matcol_and_matrow_take_a_column_or_row_counted_from_the_array_origin() {
    // The file stores a matrix column by column: the items 1 2 3 4 are M = [1 3; 2 4]. With the
    // origin 1 column 2 is [3; 4] (sum 7) and row 1 is [1 3] (sum 4).
    let res = convert(&[
        define("M", &mat(2, 2, &["1", "2", "3", "4"])),
        define("s", &call("vectorSum", &[app("matcol", &[n("M"), r("2")])])),
        eval(&n("s")),
        define("t", &call("vectorSum", &[app("matrow", &[n("M"), r("1")])])),
        eval(&n("t")),
    ]);
    clean(&res);
    assert!(res.calcpad.contains("col(M; 2)"), "{}", res.calcpad);
    assert!(res.calcpad.contains("row(M; 1)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    // Each definition prints its value, then the evaluation prints it again.
    assert_close(nums[0], 7.0);
    assert_close(nums[nums.len() - 1], 4.0);
}

#[test]
fn matcol_index_is_shifted_from_a_zero_origin() {
    // With the origin 0 the first column is 0: matcol(M, 0) is [1; 2], sum 3.
    let sheet = sheet_of(&[
        define("M", &mat(2, 2, &["1", "2", "3", "4"])),
        define("s", &call("vectorSum", &[app("matcol", &[n("M"), r("0")])])),
        eval(&n("s")),
    ])
    .replace(r#"array-origin="1""#, r#"array-origin="0""#);
    let res = import_xmcd(&sheet);
    clean(&res);
    assert!(res.calcpad.contains("col(M; 1)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(*nums.last().unwrap(), 3.0);
}
