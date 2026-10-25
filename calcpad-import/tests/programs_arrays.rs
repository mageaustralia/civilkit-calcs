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
fn a_program_that_ends_in_an_assignment_gives_the_assigned_value() {
    // The value of a program is the value of the last statement it runs, an assignment included:
    // h(x) = (1; z = 2 x) gives 2 x.  h(3) = 6.
    let body = program(&[r("1"), local("z", &times(&r("2"), &n("x")))]);
    let res = convert(&[func("h", &["x"], &body), eval(&call("h", &[r("3")]))]);
    clean(&res);
    assert_close(*numbers(&res.calcpad).last().unwrap(), 6.0);
}

#[test]
fn an_if_whose_arms_end_in_assignments_gives_the_value_assigned_in_the_arm_run() {
    // With a loop in front the function is a statement block: n = loop count, t = 1 + 2 + .. + n,
    // then (if t > 5: u = 2 t else u = t).  g(4): t = 10, u = 20.  g(2): t = 3, u = 3.
    let body = program(&[
        local("t", &r("0")),
        for_in("i", &r("1"), &n("n"), &[local("t", &plus(&n("t"), &n("i")))]),
        if_else(
            &gt(&n("t"), &r("5")),
            &[local("u", &times(&n("t"), &r("2")))],
            &[local("u", &n("t"))],
        ),
    ]);
    let res = convert(&[
        func("g", &["n"], &body),
        eval(&call("g", &[r("4")])),
        eval(&call("g", &[r("2")])),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 20.0);
    assert_close(nums[nums.len() - 1], 3.0);
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

// ---- programs that keep their value in a local, nested programs, start values --------------------------------

fn for_in(var: &str, lo: &str, hi: &str, body: &[String]) -> String {
    format!(
        "<ml:for>{}<ml:range>{lo}{hi}</ml:range>{}</ml:for>",
        n(var),
        program(body)
    )
}
fn if_else(test: &str, then: &[String], otherwise: &[String]) -> String {
    if_chain(&[(test.to_string(), then.to_vec())], Some(otherwise))
}

#[test]
fn a_program_ending_in_an_if_with_a_loop_in_one_arm_converts() {
    // f(n) = (t = 0; if n > 0: (for i in 1..n: t = t + i; t) else 0).  f(4) = 10, f(-1) = 0.
    let body = program(&[
        local("t", &r("0")),
        if_else(
            &gt(&n("n"), &r("0")),
            &[
                for_in("i", &r("1"), &n("n"), &[local("t", &plus(&n("t"), &n("i")))]),
                n("t"),
            ],
            &[r("0")],
        ),
    ]);
    let res = convert(&[
        func("f", &["n"], &body),
        eval(&call("f", &[r("4")])),
        eval(&call("f", &[r("-1")])),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 10.0);
    assert_close(nums[nums.len() - 1], 0.0);
}

#[test]
fn a_program_that_ends_in_a_loop_is_not_guessed() {
    // The value of a program is the last statement it runs; with a loop last, that is not read.
    let body = program(&[for_in(
        "i",
        &r("1"),
        &n("n"),
        &[if_then(&lt(&n("i"), &r("3")), &[times(&n("i"), &r("10"))])],
    )]);
    let res = convert(&[func("f", &["n"], &body)]);
    assert!(res.report.gaps().any(|i| i.code == Code::ProgramBlock));
}

#[test]
fn a_user_local_first_set_under_a_condition_is_a_reported_gap() {
    // g(n) = (for i in 1..n: if i > 1: a = i); a * 2. Where the loop never sets `a` the source has no
    // value, and a not-a-number inside a loop is an engine error, so the function is not converted.
    let body = program(&[
        for_in(
            "i",
            &r("1"),
            &n("n"),
            &[if_then(&gt(&n("i"), &r("1")), &[local("a", &n("i"))])],
        ),
        times(&n("a"), &r("2")),
    ]);
    let res = convert(&[func("g", &["n"], &body)]);
    let why = res
        .report
        .gaps()
        .find(|i| i.code == Code::ProgramBlock)
        .and_then(|i| i.detail.clone())
        .unwrap_or_default();
    assert!(why.contains("no value"), "{why}");
}

#[test]
fn the_result_of_a_program_with_a_path_that_gives_none_is_undefined_there() {
    // f(n) = (t = 0; for i in 1..n: t = t + i; if t > 5: t * 2). Where t <= 5 the program gives no value.
    let body = program(&[
        local("t", &r("0")),
        for_in("i", &r("1"), &n("n"), &[local("t", &plus(&n("t"), &n("i")))]),
        if_then(&gt(&n("t"), &r("5")), &[times(&n("t"), &r("2"))]),
    ]);
    let res = convert(&[
        func("f", &["n"], &body),
        eval(&call("f", &[r("4")])),
        eval(&call("f", &[r("1")])),
    ]);
    clean(&res);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&format!("#round 10\n{}", legacy(&res.calcpad)), true);
    let html = parser.html_result;
    assert!(html.contains("= 20"), "{html}");
    assert!(html.contains("Undefined"), "{html}");
}

#[test]
fn a_nested_program_runs_in_place() {
    // f(x) = (program(a = x + 1); a * 2): f(2) = 6.
    let body = program(&[program(&[local("a", &plus(&n("x"), &r("1")))]), times(&n("a"), &r("2"))]);
    let res = convert(&[func("f", &["x"], &body), eval(&call("f", &[r("2")]))]);
    clean(&res);
    assert_close(*numbers(&res.calcpad).last().unwrap(), 6.0);
}

#[test]
fn an_element_of_a_call_result_in_a_function_body_needs_no_temporary() {
    // A x = b with A = diag(2, 4), b = (2, 8): x = (1, 2).  h(k) = lsolve(A, b)[k]; h(2) = 2.
    let at = |base: &str, i: &str| format!("<ml:apply><ml:indexer/>{base}{i}</ml:apply>");
    let res = convert(&[
        define("A", &mat(2, 2, &["2", "0", "0", "4"])),
        define("b", &mat(2, 1, &["2", "8"])),
        func("h", &["k"], &at(&call("lsolve", &[n("A"), n("b")]), &n("k"))),
        eval(&call("h", &[r("2")])),
    ]);
    clean(&res);
    assert!(res.calcpad.contains("take("), "{}", res.calcpad);
    assert_close(*numbers(&res.calcpad).last().unwrap(), 2.0);
}

// ---- sorting by a column or a row, linear interpolation ---------------------------------------------------

#[test]
fn csort_orders_the_rows_by_a_column_and_rsort_the_columns_by_a_row() {
    // M = [3 1; 2 9; 5 4] (items go column by column). csort(M, 1) orders the rows by column 1:
    // [2 9; 3 1; 5 4]. rsort(M, 1) orders the columns by row 1: [1 3; 9 2; 4 5].
    let res = convert(&[
        define("M", &mat(3, 2, &["3", "2", "5", "1", "9", "4"])),
        define("S", &call("csort", &[n("M"), r("1")])),
        eval(&index2("S", &r("1"), &r("2"))),
        eval(&index2("S", &r("2"), &r("2"))),
        define("T", &call("rsort", &[n("M"), r("1")])),
        eval(&index2("T", &r("1"), &r("1"))),
        eval(&index2("T", &r("2"), &r("1"))),
    ]);
    clean(&res);
    assert!(res.calcpad.contains("sort_rows(M; 1)"), "{}", res.calcpad);
    assert!(res.calcpad.contains("sort_cols(M; 1)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 4], 9.0);
    assert_close(nums[nums.len() - 3], 1.0);
    assert_close(nums[nums.len() - 2], 1.0);
    assert_close(nums[nums.len() - 1], 9.0);
}

#[test]
fn linterp_takes_the_tables_first_and_continues_past_the_ends() {
    // linterp(vx, vy, x) on (0, 0), (10, 100), (20, 300): x = 15 gives 200, x = 25 gives 400.
    let res = convert(&[
        define("vx", &mat(3, 1, &["0", "10", "20"])),
        define("vy", &mat(3, 1, &["0", "100", "300"])),
        define("a", &call("linterp", &[n("vx"), n("vy"), r("15")])),
        eval(&n("a")),
        define("b", &call("linterp", &[n("vx"), n("vy"), r("25")])),
        eval(&n("b")),
    ]);
    clean(&res);
    assert!(res.calcpad.contains("linterp(15; vx; vy)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 3], 200.0);
    assert_close(nums[nums.len() - 1], 400.0);
}

// ---- try / on error, sums with limit elements -------------------------------------------------------------

fn try_catch(body: &str, fallback: &str) -> String {
    format!("<ml:tryCatch><ml:program>{body}</ml:program><ml:program>{fallback}</ml:program></ml:tryCatch>")
}

#[test]
fn try_on_error_over_a_division_tests_the_divisor() {
    // f(a, b) = try a / b on error -1: f(6, 3) = 2, f(1, 0) = -1.
    let body = try_catch(&app("div", &[n("a"), n("b")]), &r("-1"));
    let res = convert(&[
        func("f", &["a", "b"], &body),
        eval(&call("f", &[r("6"), r("3")])),
        eval(&call("f", &[r("1"), r("0")])),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    assert_close(nums[nums.len() - 2], 2.0);
    assert_close(nums[nums.len() - 1], -1.0);
}

#[test]
fn try_on_error_keeps_the_units_of_the_fallback_and_the_body() {
    // q = try 6 m / d on error 0 m: d = 3 gives 2 m, d = 0 gives 0 m.
    let res = convert(&[
        define("d", &r("3")),
        define(
            "q",
            &try_catch(
                &app("div", &[app("mult", &[r("6"), n("m")]), n("d")]),
                &app("mult", &[r("0"), n("m")]),
            ),
        ),
        eval_in(&n("q"), "m"),
        define("d", &r("0")),
        define(
            "p",
            &try_catch(
                &app("div", &[app("mult", &[r("6"), n("m")]), n("d")]),
                &app("mult", &[r("0"), n("m")]),
            ),
        ),
        eval_in(&n("p"), "m"),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    // Each definition prints its value, then the evaluation: d, q, q in m, d, p, p in m.
    assert_close(nums[2], 2.0);
    assert_close(*nums.last().unwrap(), 0.0);
}

fn eval_in(expr: &str, unit: &str) -> String {
    format!(
        "<ml:eval>{expr}<ml:unitOverride>{}</ml:unitOverride></ml:eval>",
        n(unit)
    )
}

#[test]
fn try_on_error_over_a_function_call_is_a_reported_gap() {
    // A call could fail in other ways than a division, which the engine cannot trap.
    let body = try_catch(&call("g", &[n("x")]), &r("0"));
    let res = convert(&[func("g", &["x"], &n("x")), func("f", &["x"], &body)]);
    let why = res
        .report
        .gaps()
        .find(|i| i.code == Code::ProgramBlock)
        .and_then(|i| i.detail.clone())
        .unwrap_or_default();
    assert!(why.contains("tryCatch") && why.contains("function"), "{why}");
}

#[test]
fn a_sum_with_limit_elements_converts() {
    // The package format writes the limits as lowerBound and upperBound: the sum of n^2 for n = 0 .. 3 is 14.
    let sum = format!(
        "<ml:apply><ml:summation/><ml:lambda><ml:boundVars>{}</ml:boundVars>{}</ml:lambda><ml:lowerBound>{}</ml:lowerBound><ml:upperBound>{}</ml:upperBound></ml:apply>",
        n("k"),
        app("pow", &[n("k"), r("2")]),
        r("0"),
        r("3")
    );
    let res = convert(&[define("s", &sum), eval(&n("s"))]);
    clean(&res);
    assert_close(*numbers(&res.calcpad).last().unwrap(), 14.0);
}

// ---- nth roots, percent ---------------------------------------------------------------------------------

#[test]
fn nthroot_takes_the_degree_first() {
    // The cube root of 27 is 3 and of -8 is -2 (a real root of a negative number).
    let res = convert(&[
        define("a", &app("nthRoot", &[r("3"), r("27")])),
        eval(&n("a")),
        define("b", &app("nthRoot", &[r("3"), r("-8")])),
        eval(&n("b")),
    ]);
    clean(&res);
    assert!(res.calcpad.contains("root(27; 3)"), "{}", res.calcpad);
    let nums = numbers(&res.calcpad);
    assert_close(nums[1], 3.0);
    assert_close(*nums.last().unwrap(), -2.0);
}

#[test]
fn a_percent_sign_is_the_percent_unit() {
    // 0.5 / 2% = 25.
    let res = convert(&[
        define("p", &app("percent", &[r("2")])),
        define("q", &app("div", &[r("0.5"), n("p")])),
        eval(&n("q")),
    ]);
    clean(&res);
    assert_close(*numbers(&res.calcpad).last().unwrap(), 25.0);
}

// ---- a bare zero in a comparison -------------------------------------------------------------------------

#[test]
fn a_comparison_with_a_bare_zero_takes_the_other_sides_units() {
    // c = 5 m is not zero (2); d = 0 m is (1); a unitless 7 is not (2).
    let test = |name: &str| call("if", &[app("equal", &[n(name), r("0")]), r("1"), r("2")]);
    let res = convert(&[
        define("c", &app("mult", &[r("5"), n("m")])),
        define("d", &app("mult", &[r("0"), n("m")])),
        define("e", &r("7")),
        define("p", &test("c")),
        define("q", &test("d")),
        define("s", &test("e")),
    ]);
    clean(&res);
    let nums = numbers(&res.calcpad);
    let k = nums.len();
    assert_close(nums[k - 3], 2.0);
    assert_close(nums[k - 2], 1.0);
    assert_close(nums[k - 1], 2.0);
}
