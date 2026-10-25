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

// ---- early exits in function bodies -------------------------------------------------------------------------

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
fn gt(a: &str, b: &str) -> String {
    app("greaterThan", &[a.to_string(), b.to_string()])
}
fn lt(a: &str, b: &str) -> String {
    app("lessThan", &[a.to_string(), b.to_string()])
}
fn for_in(var: &str, lo: &str, hi: &str, body: &[String]) -> String {
    format!(
        "<ml:for>{}<ml:range>{lo}{hi}</ml:range>{}</ml:for>",
        n(var),
        program(body)
    )
}
fn while_do(cond: &str, body: &[String]) -> String {
    format!("<ml:while>{cond}{}</ml:while>", program(body))
}
fn if_then(test: &str, then: &[String]) -> String {
    format!(
        "<ml:if><ml:test>{test}</ml:test><ml:then>{}</ml:then></ml:if>",
        program(then)
    )
}
fn ret(value: &str) -> String {
    format!("<ml:return>{value}</ml:return>")
}
const BREAK: &str = "<ml:break/>";
const CONTINUE: &str = "<ml:continue/>";

/// The numbers the last `count` evaluations printed.
fn last_numbers(res: &ImportResult, count: usize) -> Vec<f64> {
    let nums = numbers(&res.calcpad);
    nums[nums.len() - count..].to_vec()
}

#[test]
fn a_return_inside_a_loop_ends_the_function() {
    // f(x): the first i in 1..5 with i * i > x, else 0. f(10) = 4, f(1) = 2, f(100) = 0.
    let body = program(&[
        for_in(
            "i",
            &r("1"),
            &r("5"),
            &[if_then(&gt(&times(&n("i"), &n("i")), &n("x")), &[ret(&n("i"))])],
        ),
        r("0"),
    ]);
    let res = convert(&[
        func("f", &["x"], &body),
        eval(&call("f", &[r("10")])),
        eval(&call("f", &[r("1")])),
        eval(&call("f", &[r("100")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 4.0);
    assert_close(v[1], 2.0);
    assert_close(v[2], 0.0);
}

#[test]
fn a_break_stops_the_loop_and_what_follows_it_in_the_pass() {
    // g(n): the sum of 1..10 stopping when i exceeds n, the statement after the break not run:
    // s = s + i runs only for i <= n. g(4) = 10, g(20) = 55.
    let body = program(&[
        local("s", &r("0")),
        for_in(
            "i",
            &r("1"),
            &r("10"),
            &[
                if_then(&gt(&n("i"), &n("n")), &[BREAK.to_string()]),
                local("s", &plus(&n("s"), &n("i"))),
            ],
        ),
        n("s"),
    ]);
    let res = convert(&[
        func("g", &["n"], &body),
        eval(&call("g", &[r("4")])),
        eval(&call("g", &[r("20")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 10.0);
    assert_close(v[1], 55.0);
}

#[test]
fn a_continue_skips_the_rest_of_the_pass_only() {
    // h(): the sum of the odd numbers up to 6, skipping the even ones: 1 + 3 + 5 = 9.
    let even = app("equal", &[call("mod", &[n("i"), r("2")]), r("0")]);
    let body = program(&[
        local("s", &r("0")),
        for_in(
            "i",
            &r("1"),
            &r("6"),
            &[
                if_then(&even, &[CONTINUE.to_string()]),
                local("s", &plus(&n("s"), &n("i"))),
            ],
        ),
        n("s"),
    ]);
    let res = convert(&[func("h", &["x"], &body), eval(&call("h", &[r("0")]))]);
    clean(&res);
    assert_close(last_numbers(&res, 1)[0], 9.0);
}

#[test]
fn a_while_loop_with_a_break_and_a_return_after_it() {
    // w(lim): count up until the square passes lim, then return the count: w(50) = 8.
    let body = program(&[
        local("i", &r("0")),
        while_do(
            &lt(&n("i"), &r("100")),
            &[
                local("i", &plus(&n("i"), &r("1"))),
                if_then(&gt(&times(&n("i"), &n("i")), &n("lim")), &[BREAK.to_string()]),
            ],
        ),
        n("i"),
    ]);
    let res = convert(&[func("w", &["lim"], &body), eval(&call("w", &[r("50")]))]);
    clean(&res);
    assert_close(last_numbers(&res, 1)[0], 8.0);
}

#[test]
fn a_local_set_only_in_a_loop_is_not_a_number_where_nothing_set_it() {
    // k(x): the first j in 0..5 with j * j > x, found by break; read after the loop.
    // k(5) = 3. k(100) finds none: the source has no value there, and so does the engine.
    let body = program(&[
        for_in(
            "j",
            &r("0"),
            &r("5"),
            &[if_then(
                &gt(&times(&n("j"), &n("j")), &n("x")),
                &[local("ans", &n("j")), BREAK.to_string()],
            )],
        ),
        n("ans"),
    ]);
    let res = convert(&[
        func("k", &["x"], &body),
        eval(&call("k", &[r("5")])),
        eval(&call("k", &[r("100")])),
    ]);
    clean(&res);
    // The engine shows a not-a-number result as an error span, so read the page directly.
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{}", legacy(&res.calcpad)), true);
    let html = parser.html_result;
    assert!(html.contains("= 3"), "{html}");
    assert_eq!(html.matches("Undefined").count(), 1, "{html}");
}

#[test]
fn a_return_keeps_the_units_of_the_value_returned() {
    // u(x): returns 2 m early when x > 0, else 7 m: u(1) = 2 m, u(-1) = 7 m, shown in metres.
    let body = program(&[
        if_then(&gt(&n("x"), &r("0")), &[ret(&times(&r("2"), &n("m")))]),
        times(&r("7"), &n("m")),
    ]);
    let res = convert(&[
        func("u", &["x"], &body),
        define("a", &call("u", &[r("1")])),
        define("b", &call("u", &[r("-1")])),
        eval(&app("div", &[n("a"), n("m")])),
        eval(&app("div", &[n("b"), n("m")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 2.0);
    assert_close(v[1], 7.0);
}

#[test]
fn a_function_defined_twice_keeps_each_definition_for_the_calls_after_it() {
    // c(x) = x + 1, then c(x) = x + 100: c(1) before is 2, after is 101, and the engine sees no redefinition.
    let res = convert(&[
        func("c", &["x"], &plus(&n("x"), &r("1"))),
        eval(&call("c", &[r("1")])),
        func("c", &["x"], &plus(&n("x"), &r("100"))),
        eval(&call("c", &[r("1")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 2.0);
    assert_close(v[1], 101.0);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(&res.calcpad), true);
    assert!(!parser.html_result.contains("redefined"), "{}", parser.html_result);
}

// ---- try on error over worksheet functions, a function that is an if, stack of a number --------------------

fn try_catch(body: &str, fallback: &str) -> String {
    format!("<ml:tryCatch><ml:program>{body}</ml:program><ml:program>{fallback}</ml:program></ml:tryCatch>")
}

#[test]
fn try_on_error_looks_through_worksheet_functions_and_only_the_branch_that_runs() {
    // g(x) = 1 / x; k(x) = if x > 0 then 1 / x else 0.
    // t(x) = try g(x) + 1 on error -1: t(2) = 1.5, t(0) = -1.
    // u(x) = try k(x) on error 99: the division sits on the branch x > 0, so u(0) = 0, not 99.
    let kbody = format!(
        "<ml:if><ml:test>{}</ml:test><ml:then>{}</ml:then><ml:else>{}</ml:else></ml:if>",
        gt(&n("x"), &r("0")),
        program(&[app("div", &[r("1"), n("x")])]),
        program(&[r("0")])
    );
    let res = convert(&[
        func("g", &["x"], &app("div", &[r("1"), n("x")])),
        func("k", &["x"], &kbody),
        func(
            "t",
            &["x"],
            &program(&[try_catch(&plus(&call("g", &[n("x")]), &r("1")), &r("-1"))]),
        ),
        func("u", &["x"], &program(&[try_catch(&call("k", &[n("x")]), &r("99"))])),
        eval(&call("t", &[r("2")])),
        eval(&call("t", &[r("0")])),
        eval(&call("u", &[r("4")])),
        eval(&call("u", &[r("0")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 4);
    assert_close(v[0], 1.5);
    assert_close(v[1], -1.0);
    assert_close(v[2], 0.25);
    assert_close(v[3], 0.0);
}

#[test]
fn a_function_whose_value_is_an_if_without_a_block_around_it_converts() {
    let body = format!(
        "<ml:if><ml:test>{}</ml:test><ml:then>{}</ml:then><ml:else>{}</ml:else></ml:if>",
        gt(&n("x"), &r("1")),
        program(&[r("10")]),
        program(&[r("20")])
    );
    let res = convert(&[
        func("p", &["x"], &body),
        eval(&call("p", &[r("5")])),
        eval(&call("p", &[r("0")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 10.0);
    assert_close(v[1], 20.0);
}

#[test]
fn stack_takes_a_number_as_a_one_element_array() {
    // stack(5, [1; 2]) is [5; 1; 2]: the third element is 2 and there are three rows.
    let res = convert(&[
        define("x", &r("5")),
        define("v", &mat(2, 1, &["1", "2"])),
        define("w", &call("stack", &[n("x"), n("v")])),
        eval(&call("rows", &[n("w")])),
        eval(&index2("w", &r("1"), &r("1"))),
        eval(&index2("w", &r("3"), &r("1"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 3.0);
    assert_close(v[1], 5.0);
    assert_close(v[2], 2.0);
}

// ---- stepped range variables ---------------------------------------------------------------------------------

fn stepped(first: &str, second: &str, last: &str) -> String {
    format!(
        "<ml:range><ml:sequence>{}{}</ml:sequence>{}</ml:range>",
        r(first),
        r(second),
        r(last)
    )
}

#[test]
fn a_stepped_range_variable_gives_a_table_with_its_step() {
    // x := 0, 5 .. 20 has five values; 2 x is the column 0 10 20 30 40. A step of 0.1 keeps its last pass.
    let res = convert(&[
        define("x", &stepped("0", "5", "20")),
        eval(&times(&n("x"), &r("2"))),
        define("y", &stepped("0", "0.1", "0.3")),
        eval(&times(&n("y"), &r("1"))),
    ]);
    clean(&res);
    let text = engine_lines(&res.calcpad).join(" ").replace('\u{2002}', " ");
    assert!(text.contains("[0 10 20 30 40]"), "{text}");
    // 0, 0.1, 0.2, 0.3: four values, the last of them 0.3.
    assert!(text.contains("[0 0.1 0.2 0.3]"), "{text}");
}

#[test]
fn a_descending_stepped_range_counts_down() {
    // z := 10, 8 .. 4: 10, 8, 6, 4.
    let res = convert(&[define("z", &stepped("10", "8", "4")), eval(&times(&n("z"), &r("1")))]);
    clean(&res);
    let text = engine_lines(&res.calcpad).join(" ").replace('\u{2002}', " ");
    assert!(text.contains("[10 8 6 4]"), "{text}");
}
