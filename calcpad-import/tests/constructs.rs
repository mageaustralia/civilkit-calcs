//! Language constructs the importer converts by lowering: early exits in function bodies, stepped ranges,
//! complex numbers, element assignment with computed targets. Each case builds a small invented
//! worksheet, converts it, runs the produced Calcpad text through the engine and compares the printed
//! numbers with values worked out by hand.

use calcpad_import::{import_xmcd, ImportResult};

mod common;
use common::legacy;

fn sheet_of(inners: &[String], origin: i64) -> String {
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
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="{origin}"/></calculation></settings><regions>{regions}</regions></worksheet>"#
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
fn range(first: &str, last: &str) -> String {
    format!("<ml:range>{}{}</ml:range>", r(first), r(last))
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
fn set_element(base: &str, idx: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", index1(base, idx), value)
}
fn set_element2(base: &str, i: &str, j: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", index2(base, i, j), value)
}
fn mat(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}
fn plus(a: &str, b: &str) -> String {
    app("plus", &[a.to_string(), b.to_string()])
}
fn times(a: &str, b: &str) -> String {
    app("mult", &[a.to_string(), b.to_string()])
}

fn convert(inners: &[String]) -> ImportResult {
    import_xmcd(&sheet_of(inners, 1))
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

// ---- element assignment on arrays that already have a value ------------------------------------------------

#[test]
fn a_range_and_a_literal_index_update_a_column_of_an_existing_matrix() {
    // M = [1 2; 3 4; 5 6]; M[r, 2] := M[r, 1] * 10 for r = 1..3 gives the second column 10, 30, 50.
    let res = convert(&[
        define("M", &mat(3, 2, &["1", "3", "5", "2", "4", "6"])),
        define("r", &range("1", "3")),
        set_element2("M", &n("r"), &r("2"), &times(&index2("M", &n("r"), &r("1")), &r("10"))),
        eval(&index2("M", &r("1"), &r("2"))),
        eval(&index2("M", &r("2"), &r("2"))),
        eval(&index2("M", &r("3"), &r("2"))),
        eval(&index2("M", &r("3"), &r("1"))),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    let k = nums.len();
    assert_close(nums[k - 4], 10.0);
    assert_close(nums[k - 3], 30.0);
    assert_close(nums[k - 2], 50.0);
    assert_close(nums[k - 1], 5.0);
}

#[test]
fn a_range_update_may_read_the_vector_it_writes_in_the_order_of_the_range() {
    // v = [1; 2; 3]; v[i] := v[i - 1] + v[i] for i = 2..3 is a running sum: 1, 3, 6.
    let res = convert(&[
        define("v", &mat(3, 1, &["1", "2", "3"])),
        define("i", &range("2", "3")),
        set_element(
            "v",
            &n("i"),
            &plus(&index1("v", &app("minus", &[n("i"), r("1")])), &index1("v", &n("i"))),
        ),
        eval(&index1("v", &r("2"))),
        eval(&index1("v", &r("3"))),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    let k = nums.len();
    assert_close(nums[k - 2], 3.0);
    assert_close(nums[k - 1], 6.0);
}

#[test]
fn a_literal_element_past_the_end_of_a_matrix_grows_it_with_zeros() {
    // M = [1 2; 3 4]; M[3, 3] := 7 makes a 3 x 3 matrix with zeros where nothing was set.
    let res = convert(&[
        define("M", &mat(2, 2, &["1", "2", "3", "4"])),
        set_element2("M", &r("3"), &r("3"), &r("7")),
        eval(&call("rows", &[n("M")])),
        eval(&call("cols", &[n("M")])),
        eval(&index2("M", &r("3"), &r("3"))),
        eval(&index2("M", &r("3"), &r("1"))),
        eval(&index2("M", &r("2"), &r("2"))),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    let k = nums.len();
    assert_close(nums[k - 5], 3.0);
    assert_close(nums[k - 4], 3.0);
    assert_close(nums[k - 3], 7.0);
    assert_close(nums[k - 2], 0.0);
    assert_close(nums[k - 1], 4.0);
}
