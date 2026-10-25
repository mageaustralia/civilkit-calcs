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

// ---- arrays a function body grows by element ------------------------------------------------------------------

fn local_el(base: &str, idx: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", index1(base, idx), value)
}

#[test]
fn an_array_written_at_numbers_in_several_statements_is_made_once() {
    // z[0] <- 5, z[1] <- 6, z[2] <- 7 in three statements, then z: three elements, the last 7.
    let body = program(&[
        local_el("z", &r("0"), &r("5")),
        local_el("z", &r("1"), &r("6")),
        local_el("z", &r("2"), &r("7")),
        n("z"),
    ]);
    let sheet = sheet_of(
        &[
            func("f", &["x"], &body),
            define("w", &call("f", &[r("0")])),
            eval(&call("rows", &[n("w")])),
            eval(&index1("w", &r("2"))),
        ],
        0,
    );
    let res = import_xmcd(&sheet);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 3.0);
    assert_close(v[1], 7.0);
}

#[test]
fn an_array_the_function_received_grows_when_written_past_its_end() {
    // g(c): c[3] <- 9 on c = [1; 2] gives [1; 2; 9].
    let body = program(&[local_el("c", &r("3"), &r("9")), n("c")]);
    let res = convert(&[
        func("g", &["c"], &body),
        define("w", &call("g", &[mat(2, 1, &["1", "2"])])),
        eval(&call("rows", &[n("w")])),
        eval(&index1("w", &r("3"))),
        eval(&index1("w", &r("1"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 3.0);
    assert_close(v[1], 9.0);
    assert_close(v[2], 1.0);
}

#[test]
fn an_array_filled_in_a_while_loop_has_the_size_the_loop_reached() {
    // k(m): i counts up while i < m, z[i] <- i * i. k(4) has four elements, the last 16.
    let body = program(&[
        local("i", &r("0")),
        while_do(
            &lt(&n("i"), &n("m")),
            &[
                local("i", &plus(&n("i"), &r("1"))),
                local_el("z", &n("i"), &times(&n("i"), &n("i"))),
            ],
        ),
        n("z"),
    ]);
    let res = convert(&[
        func("k", &["m"], &body),
        define("w", &call("k", &[r("4")])),
        eval(&call("rows", &[n("w")])),
        eval(&index1("w", &r("4"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 4.0);
    assert_close(v[1], 16.0);
}

#[test]
fn writes_under_a_condition_extend_an_array_a_write_that_always_runs_started() {
    // h(): w[1] <- 0; for i in 1..6: if i is even, w[i] <- i. w is [0 2 0 4 0 6]: six elements, w[4] = 4.
    let even = app("equal", &[call("mod", &[n("i"), r("2")]), r("0")]);
    let body = program(&[
        local_el("w", &r("1"), &r("0")),
        for_in(
            "i",
            &r("1"),
            &r("6"),
            &[if_then(&even, &[local_el("w", &n("i"), &n("i"))])],
        ),
        n("w"),
    ]);
    let res = convert(&[
        func("h", &["x"], &body),
        define("v", &call("h", &[r("0")])),
        eval(&call("rows", &[n("v")])),
        eval(&index1("v", &r("4"))),
        eval(&index1("v", &r("5"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 6.0);
    assert_close(v[1], 4.0);
    assert_close(v[2], 0.0);
}

#[test]
fn an_array_written_only_under_a_condition_is_not_guessed() {
    // p(x): if x > 0, q[1] <- 5. With x <= 0 the source has no q at all, so the function is a gap.
    let body = program(&[
        if_then(&gt(&n("x"), &r("0")), &[local_el("q", &r("1"), &r("5"))]),
        n("q"),
    ]);
    let res = convert(&[func("p", &["x"], &body)]);
    let why = res
        .report
        .gaps()
        .find(|i| i.code == calcpad_import::Code::ProgramBlock)
        .and_then(|i| i.detail.clone())
        .unwrap_or_default();
    assert!(why.contains("only under a condition"), "{why}");
}

// ---- arrays of arrays --------------------------------------------------------------------------------------

#[test]
fn an_array_whose_elements_are_vectors_is_a_matrix_with_one_row_per_element() {
    // f(x) = [x; 2 x; 3 x]. V[k] := f(k) for k = 1..3, then V[3][3] = 9, V[1][2] = 2 and V has three elements.
    let fx = format!(
        r#"<ml:matrix rows="3" cols="1">{}{}{}</ml:matrix>"#,
        n("x"),
        times(&r("2"), &n("x")),
        times(&r("3"), &n("x"))
    );
    let res = convert(&[
        func("f", &["x"], &fx),
        define("k", &range("1", "3")),
        set_element("V", &n("k"), &call("f", &[n("k")])),
        eval(&index1_of(&index1("V", &r("3")), &r("3"))),
        eval(&index1_of(&index1("V", &r("1")), &r("2"))),
        eval(&call("rows", &[n("V")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 9.0);
    assert_close(v[1], 2.0);
    assert_close(v[2], 3.0);
}

fn index1_of(base_expr: &str, idx: &str) -> String {
    format!("<ml:apply><ml:indexer/>{base_expr}{idx}</ml:apply>")
}

// ---- several names from one program --------------------------------------------------------------------------

#[test]
fn several_names_defined_from_a_program_that_returns_its_locals() {
    // [A, B] := (s = 0; for i in 1..3: s = s + i; t = s * 2; [s; t]): A = 6, B = 12.
    let body = program(&[
        local("s", &r("0")),
        for_in("i", &r("1"), &r("3"), &[local("s", &plus(&n("s"), &n("i")))]),
        local("t", &times(&n("s"), &r("2"))),
        format!(r#"<ml:matrix rows="2" cols="1">{}{}</ml:matrix>"#, n("s"), n("t")),
    ]);
    let target = format!(r#"<ml:matrix rows="1" cols="2">{}{}</ml:matrix>"#, n("A"), n("B"));
    let res = convert(&[
        format!("<ml:define>{target}{body}</ml:define>"),
        eval(&n("A")),
        eval(&n("B")),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 6.0);
    assert_close(v[1], 12.0);
}

// ---- whole columns as assignment targets -------------------------------------------------------------------------

fn matcol_target(base: &str, idx: &str) -> String {
    format!("<ml:apply><ml:matcol/>{}{}</ml:apply>", n(base), idx)
}
fn matcol(base: &str, idx: &str) -> String {
    matcol_target(base, idx)
}
fn set_column(base: &str, idx: &str, value: &str) -> String {
    format!("<ml:define>{}{}</ml:define>", matcol_target(base, idx), value)
}

#[test]
fn a_column_assigned_from_an_expression_replaces_that_column() {
    // M = [1 2; 3 4; 5 6]; M<2> := M<1> * 10 gives the second column 10, 30, 50; the first is unchanged.
    let res = convert(&[
        define("M", &mat(3, 2, &["1", "3", "5", "2", "4", "6"])),
        set_column("M", &r("2"), &times(&matcol("M", &r("1")), &r("10"))),
        eval(&index2("M", &r("1"), &r("2"))),
        eval(&index2("M", &r("3"), &r("2"))),
        eval(&index2("M", &r("3"), &r("1"))),
        eval(&call("cols", &[n("M")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 4);
    assert_close(v[0], 10.0);
    assert_close(v[1], 50.0);
    assert_close(v[2], 5.0);
    assert_close(v[3], 2.0);
}

#[test]
fn a_column_assigned_to_a_new_name_makes_the_array_as_large_as_it_needs() {
    // N<2> := [7; 8] makes N a 2 x 2 matrix with zeros in the first column.
    let res = convert(&[
        set_column("N", &r("2"), &mat(2, 1, &["7", "8"])),
        eval(&call("rows", &[n("N")])),
        eval(&index2("N", &r("2"), &r("2"))),
        eval(&index2("N", &r("1"), &r("1"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 2.0);
    assert_close(v[1], 8.0);
    assert_close(v[2], 0.0);
}

#[test]
fn a_function_assigns_a_column_of_its_argument() {
    // f(c): c<1> <- c<2> + 100 on c = [1 2; 3 4] gives the first column 102, 104.
    let body = program(&[
        format!(
            "<ml:localDefine>{}{}</ml:localDefine>",
            matcol_target("c", &r("1")),
            plus(&matcol("c", &r("2")), &r("100"))
        ),
        n("c"),
    ]);
    let res = convert(&[
        func("f", &["c"], &body),
        define("w", &call("f", &[mat(2, 2, &["1", "3", "2", "4"])])),
        eval(&index2("w", &r("1"), &r("1"))),
        eval(&index2("w", &r("2"), &r("1"))),
        eval(&index2("w", &r("2"), &r("2"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 102.0);
    assert_close(v[1], 104.0);
    assert_close(v[2], 4.0);
}

// ---- complex numbers, as pairs of real ones -------------------------------------------------------------------

const IMAG: &str = "<ml:imag/>";

fn cplx(re: &str, im: &str) -> String {
    plus(&r(re), &times(&r(im), IMAG))
}

#[test]
fn complex_arithmetic_is_written_out_for_the_two_parts() {
    // z = (1 + 2i)(3 - 1i) = 5 + 5i; w = (1 + 2i) / (3 - 1i) = (1 + 7i) / 10; u = (1 + i)^2 = 2i.
    let res = convert(&[
        define("z", &times(&cplx("1", "2"), &cplx("3", "-1"))),
        define("w", &app("div", &[cplx("1", "2"), cplx("3", "-1")])),
        define("u", &app("pow", &[cplx("1", "1"), r("2")])),
        eval(&call("Re", &[n("z")])),
        eval(&call("Im", &[n("z")])),
        eval(&call("Re", &[n("w")])),
        eval(&call("Im", &[n("w")])),
        eval(&call("Re", &[n("u")])),
        eval(&call("Im", &[n("u")])),
        eval(&call("abs", &[n("z")])),
        eval(&call("Im", &[call("conj", &[n("z")])])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 8);
    assert_close(v[0], 5.0);
    assert_close(v[1], 5.0);
    assert_close(v[2], 0.1);
    assert_close(v[3], 0.7);
    assert_close(v[4], 0.0);
    assert_close(v[5], 2.0);
    assert_close(v[6], 50f64.sqrt());
    assert_close(v[7], -5.0);
}

#[test]
fn an_array_of_complex_numbers_is_two_arrays_of_real_ones() {
    // g(): p[i] <- i + (i * i) * 1i for i in 1..3; p. Re(p[3]) = 3 and Im(p[3]) = 9.
    let body = program(&[
        for_in(
            "i",
            &r("1"),
            &r("3"),
            &[local_el(
                "p",
                &n("i"),
                &plus(&n("i"), &times(&times(&n("i"), &n("i")), IMAG)),
            )],
        ),
        n("p"),
    ]);
    let res = convert(&[
        define("P", &body),
        eval(&call("Re", &[index1("P", &r("3"))])),
        eval(&call("Im", &[index1("P", &r("3"))])),
        eval(&call("Im", &[index1("P", &r("2"))])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 3.0);
    assert_close(v[1], 9.0);
    assert_close(v[2], 4.0);
}

#[test]
fn a_complex_value_used_as_a_real_number_is_a_gap() {
    let res = convert(&[
        define("z", &cplx("1", "2")),
        define("y", &plus(&n("z"), &r("1"))),
        eval(&n("y")),
    ]);
    // y = z + 1 is complex itself and fine; asking for it as a number (a bare evaluation) is not.
    assert!(
        res.report
            .gaps()
            .any(|i| i.code == calcpad_import::Code::UnknownElement),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_program_that_writes_elements_of_a_name_the_sheet_also_defines_starts_empty() {
    // v = [9; 9; 9; 9] is defined at the sheet; the program writes v[1] <- 5, v[2] <- 6 and returns its own
    // v: two elements, [5; 6] (the sheet's v is not where it starts), and the sheet's v is unchanged.
    let body = program(&[local_el("v", &r("1"), &r("5")), local_el("v", &r("2"), &r("6")), n("v")]);
    let res = convert(&[
        define("v", &mat(4, 1, &["9", "9", "9", "9"])),
        define("w", &body),
        eval(&call("rows", &[n("w")])),
        eval(&index1("w", &r("2"))),
        eval(&index1("v", &r("4"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 3);
    assert_close(v[0], 2.0);
    assert_close(v[1], 6.0);
    assert_close(v[2], 9.0);
}

#[test]
fn an_array_written_by_element_then_scaled_whole_then_written_again() {
    // a[1] <- 4; a[2] <- 5; a <- a * 2; a[3] <- 1; a is [8; 10; 1].
    let body = program(&[
        local_el("a", &r("1"), &r("4")),
        local_el("a", &r("2"), &r("5")),
        local("a", &times(&n("a"), &r("2"))),
        local_el("a", &r("3"), &r("1")),
        n("a"),
    ]);
    let res = convert(&[
        func("f", &["x"], &body),
        define("w", &call("f", &[r("0")])),
        eval(&call("rows", &[n("w")])),
        eval(&index1("w", &r("1"))),
        eval(&index1("w", &r("2"))),
        eval(&index1("w", &r("3"))),
    ]);
    clean(&res);
    let v = last_numbers(&res, 4);
    assert_close(v[0], 3.0);
    assert_close(v[1], 8.0);
    assert_close(v[2], 10.0);
    assert_close(v[3], 1.0);
}

#[test]
fn an_array_written_only_in_the_arms_that_read_it_is_made_where_it_is_written() {
    // h(x): if x > 0: (s[1] <- x; s[2] <- 2 x; s[1] + s[2]) else x. The array is only read in the arm
    // that writes it, so the arms that do not write it have no use for it: h(2) = 6, h(-1) = -1.
    let body = program(&[format!(
        "<ml:if><ml:test>{}</ml:test><ml:then>{}</ml:then><ml:else>{}</ml:else></ml:if>",
        gt(&n("x"), &r("0")),
        program(&[
            local_el("s", &r("1"), &n("x")),
            local_el("s", &r("2"), &times(&r("2"), &n("x"))),
            plus(&index1("s", &r("1")), &index1("s", &r("2"))),
        ]),
        program(&[n("x")])
    )]);
    let res = convert(&[
        func("h", &["x"], &body),
        eval(&call("h", &[r("2")])),
        eval(&call("h", &[r("-1")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 6.0);
    assert_close(v[1], -1.0);
}

#[test]
fn a_matrix_that_carries_a_display_size_still_has_its_items() {
    // The author's window size (`display`) is stored among the items; it is not one of them.
    let m =
        r#"<ml:matrix rows="1" cols="2"><ml:display size="1,2"/><ml:real>3</ml:real><ml:real>4</ml:real></ml:matrix>"#;
    let res = convert(&[
        define("M", m),
        eval(&index2("M", &r("1"), &r("2"))),
        eval(&call("cols", &[n("M")])),
    ]);
    clean(&res);
    let v = last_numbers(&res, 2);
    assert_close(v[0], 4.0);
    assert_close(v[1], 2.0);
}

// ---- rounding to a multiple --------------------------------------------------------------------------------------

#[test]
fn ceil_and_floor_with_a_second_argument_round_to_a_multiple_of_it() {
    // Ceil(37, 5) = 40, Floor(37, 5) = 35, and a length rounded up to whole metres keeps its unit: Ceil(7.3 m, m) = 8 m.
    let res = convert(&[
        define("a", &call("Ceil", &[r("37"), r("5")])),
        eval(&n("a")),
        define("b", &call("Floor", &[r("37"), r("5")])),
        eval(&n("b")),
        define("L", &times(&r("7.3"), &n("m"))),
        eval(&app("div", &[call("Ceil", &[n("L"), n("m")]), n("m")])),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    // Each definition and each evaluation prints a line; the last of each pair is the value.
    let v = last_numbers(&res, 6);
    assert_close(v[1], 40.0);
    assert_close(v[3], 35.0);
    assert_close(v[5], 8.0);
}
