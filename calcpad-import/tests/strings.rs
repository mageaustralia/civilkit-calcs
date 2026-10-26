//! Text values that label or flag a result: programs that set a text local, string comparisons in
//! `if` chains, functions that take a text, matrices that mix texts and numbers, and what stays a
//! gap. Synthetic worksheets written here, with generic names.

use calcpad_import::{import_xmcd_opts, Code, ImportOptions, ImportResult, Severity};

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
fn fun(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| id(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        id(name)
    )
}
fn prog(stmts: &[String]) -> String {
    format!("<ml:program>{}</ml:program>", stmts.concat())
}
fn local(name: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", id(name), value)
}
fn iff(test: &str, then: &[String]) -> String {
    format!(
        "<ml:if><ml:test>{test}</ml:test><ml:then>{}</ml:then></ml:if>",
        prog(then)
    )
}
fn for_in(var: &str, lo: &str, hi: &str, body: &[String]) -> String {
    format!(
        "<ml:for>{}<ml:range>{lo}{hi}</ml:range>{}</ml:for>",
        id(var),
        prog(body)
    )
}
fn col(items: &[String]) -> String {
    format!(
        "<ml:matrix rows=\"{}\" cols=\"1\">{}</ml:matrix>",
        items.len(),
        items.concat()
    )
}
fn row(items: &[String]) -> String {
    format!(
        "<ml:matrix rows=\"1\" cols=\"{}\">{}</ml:matrix>",
        items.len(),
        items.concat()
    )
}
fn indexed(name: &str, i: &str) -> String {
    format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", id(name), i)
}
fn gt(a: &str, b: &str) -> String {
    app("greaterThan", &[a.to_string(), b.to_string()])
}
fn eq(a: &str, b: &str) -> String {
    app("equal", &[a.to_string(), b.to_string()])
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
    let page = match (p.html_result.find("<style>"), p.html_result.find("</style>")) {
        (Some(a), Some(b)) if a < b => format!("{}{}", &p.html_result[..a], &p.html_result[b + 8..]),
        _ => p.html_result.clone(),
    };
    for c in page.chars() {
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

fn clean(res: &ImportResult) {
    assert_eq!(
        res.report.regions_needing_attention(),
        0,
        "{:?}\n{}",
        res.report.issues,
        res.calcpad
    );
}

fn has_gap(res: &ImportResult, code: Code) -> bool {
    res.report
        .issues
        .iter()
        .any(|i| i.code == code && i.severity == Severity::Gap)
}

// ---- a program that sets a text local ----------------------------------------------------------------

/// `check := "OK"; for i in 1..nn { if v[i] > lim { check := "NG" } }; check`
fn loop_check_body(lim: &str) -> String {
    prog(&[
        local("check", &s("OK")),
        for_in(
            "i",
            &real("1"),
            &id("nn"),
            &[iff(&gt(&indexed("v", &id("i")), lim), &[local("check", &s("NG"))])],
        ),
        id("check"),
    ])
}

fn loop_check_sheet(lim: &str) -> ImportResult {
    import(&[
        math(1, 0.0, &define(&id("nn"), &real("3"))),
        math(2, 20.0, &define(&id("v"), &col(&[real("1"), real("4"), real("2")]))),
        math(3, 40.0, &define(&id("Res"), &loop_check_body(&real(lim)))),
        math(4, 60.0, &eval(&id("Res"))),
        math(
            5,
            80.0,
            &define(
                &id("flag"),
                &call("if", &[eq(&id("Res"), &s("NG")), real("10"), real("20")]),
            ),
        ),
        math(6, 100.0, &eval(&id("flag"))),
    ])
}

#[test]
fn a_text_local_set_in_a_loop_is_the_result_of_a_variable() {
    let res = loop_check_sheet("3");
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("NG") && !t.contains("OK") && t.contains("flag = 10"), "{t}");
    let res = loop_check_sheet("5");
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("OK") && !t.contains("NG") && t.contains("flag = 20"), "{t}");
}

#[test]
fn a_function_with_a_text_local_set_in_a_loop_converts() {
    let mk = |lim: &str| {
        import(&[
            math(1, 0.0, &define(&id("nn"), &real("3"))),
            math(2, 20.0, &define(&id("v"), &col(&[real("1"), real("4"), real("2")]))),
            math(3, 40.0, &fun("fCheck", &["lim"], &loop_check_body(&id("lim")))),
            math(4, 60.0, &define(&id("r"), &call("fCheck", &[real(lim)]))),
            math(5, 80.0, &eval(&id("r"))),
            math(
                6,
                100.0,
                &define(
                    &id("flag"),
                    &call(
                        "if",
                        &[eq(&call("fCheck", &[real(lim)]), &s("NG")), real("10"), real("20")],
                    ),
                ),
            ),
            math(7, 120.0, &eval(&id("flag"))),
        ])
    };
    let res = mk("3");
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("NG") && !t.contains("OK") && t.contains("flag = 10"), "{t}");
    let res = mk("5");
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("OK") && !t.contains("NG") && t.contains("flag = 20"), "{t}");
}

// ---- a text local set on branches, a note among the statements ------------------------------------------

#[test]
fn a_text_local_set_on_both_branches_of_a_function_converts() {
    // grade(x) := msg <- "low"; if x > 2 { msg <- "high" }; msg
    let body = prog(&[
        local("msg", &s("low")),
        iff(&gt(&id("x"), &real("2")), &[local("msg", &s("high"))]),
        id("msg"),
    ]);
    let res = import(&[
        math(1, 0.0, &fun("grade", &["x"], &body)),
        math(2, 20.0, &define(&id("a"), &call("grade", &[real("5")]))),
        math(3, 40.0, &eval(&id("a"))),
        math(4, 60.0, &define(&id("b"), &call("grade", &[real("1")]))),
        math(5, 80.0, &eval(&id("b"))),
    ]);
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("high") && t.contains("low"), "{t}");
}

#[test]
fn a_note_among_the_statements_of_a_program_is_not_a_value() {
    // The text between the statements is a note: the value is the last statement.
    let body = prog(&[
        local("k", &real("4")),
        s("a note the author left"),
        for_in(
            "i",
            &real("1"),
            &id("nn"),
            &[
                iff(&gt(&indexed("v", &id("i")), &real("3")), &[local("k", &real("9"))]),
                s("another note"),
            ],
        ),
        id("k"),
    ]);
    let res = import(&[
        math(1, 0.0, &define(&id("nn"), &real("3"))),
        math(2, 20.0, &define(&id("v"), &col(&[real("1"), real("4"), real("2")]))),
        math(3, 40.0, &define(&id("K"), &body)),
        math(4, 60.0, &eval(&id("K"))),
    ]);
    clean(&res);
    assert!(engine_text(&res).contains("K = 9"));
}

// ---- comparisons of texts inside a chain of ifs -------------------------------------------------------------

fn chain_sheet(x: &str, y: &str) -> ImportResult {
    let flag = |n: &str, v: &str| {
        math(
            0,
            0.0,
            &define(&id(n), &call("if", &[gt(&id(v), &real("2")), s("OK"), s("No Good")])),
        )
    };
    let _ = flag;
    import(&[
        math(1, 0.0, &define(&id("x"), &real(x))),
        math(2, 20.0, &define(&id("y"), &real(y))),
        math(
            3,
            40.0,
            &define(
                &id("CheckA"),
                &call("if", &[gt(&id("x"), &real("2")), s("OK"), s("No Good")]),
            ),
        ),
        math(
            4,
            60.0,
            &define(
                &id("CheckB"),
                &call("if", &[gt(&id("y"), &real("2")), s("OK"), s("No Good")]),
            ),
        ),
        math(
            5,
            80.0,
            &define(
                &id("CheckBoth"),
                &call(
                    "if",
                    &[
                        app(
                            "plus",
                            &[eq(&id("CheckA"), &s("No Good")), eq(&id("CheckB"), &s("No Good"))],
                        ),
                        s("No Good"),
                        s("OK"),
                    ],
                ),
            ),
        ),
        math(6, 100.0, &eval(&id("CheckBoth"))),
    ])
}

#[test]
fn a_sum_of_text_comparisons_chooses_a_text() {
    let res = chain_sheet("3", "3");
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("= OK") && !t.contains("No Good"), "{t}");
    for (x, y) in [("1", "3"), ("3", "1"), ("1", "1")] {
        let res = chain_sheet(x, y);
        clean(&res);
        let t = engine_text(&res);
        assert!(t.contains("= No Good"), "{x} {y}: {t}");
    }
}

// ---- a function that takes a text ----------------------------------------------------------------------------

fn text_arg_function() -> String {
    // thk(kind) := if kind = "main" { 0.18 }; if kind = "side" { 0.125 }  (as statements, no else)
    let body = prog(&[
        iff(&eq(&id("kind"), &s("main")), &[real("0.18")]),
        iff(&eq(&id("kind"), &s("side")), &[real("0.125")]),
    ]);
    fun("thk", &["kind"], &body)
}

#[test]
fn a_function_that_takes_a_text_is_called_with_a_text() {
    let res = import(&[
        math(1, 0.0, &text_arg_function()),
        math(2, 20.0, &define(&id("Kind"), &s("side"))),
        math(3, 40.0, &define(&id("a"), &call("thk", &[s("main")]))),
        math(4, 60.0, &define(&id("b"), &call("thk", &[id("Kind")]))),
        math(5, 80.0, &eval(&app("plus", &[id("a"), id("b")]))),
    ]);
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("0.305"), "{t}");
}

#[test]
fn a_number_given_where_the_function_takes_a_text_is_a_gap() {
    let res = import(&[
        math(1, 0.0, &text_arg_function()),
        math(2, 20.0, &define(&id("a"), &call("thk", &[real("1")]))),
    ]);
    assert!(has_gap(&res, Code::StringValue), "{:?}", res.report.issues);
    let why = res
        .report
        .issues
        .iter()
        .find(|i| i.code == Code::StringValue)
        .and_then(|i| i.detail.clone());
    assert!(why.is_some_and(|d| d.contains("not a text")), "{:?}", res.report.issues);
}

#[test]
fn a_text_given_where_the_function_takes_a_number_is_a_gap() {
    let res = import(&[
        math(1, 0.0, &fun("dbl", &["x"], &app("mult", &[id("x"), real("2")]))),
        math(2, 20.0, &define(&id("a"), &call("dbl", &[s("main")]))),
    ]);
    assert!(has_gap(&res, Code::StringValue), "{:?}", res.report.issues);
}

#[test]
fn a_function_that_gives_back_its_text_argument_is_a_gap() {
    let body = prog(&[
        iff(&eq(&id("kind"), &s("main")), &[s("primary")]),
        iff(&eq(&id("kind"), &s("side")), &[id("kind")]),
    ]);
    let res = import(&[
        math(1, 0.0, &fun("same", &["kind"], &body)),
        math(2, 20.0, &define(&id("a"), &call("same", &[s("side")]))),
    ]);
    assert!(has_gap(&res, Code::StringValue), "{:?}", res.report.issues);
}

// ---- tables that mix texts and numbers ------------------------------------------------------------------------

fn at2(name: &str, i: &str, j: &str) -> String {
    format!(
        "<ml:apply><ml:indexer/>{}<ml:sequence>{}{}</ml:sequence></ml:apply>",
        id(name),
        real(i),
        real(j)
    )
}

fn table_regions() -> Vec<String> {
    vec![
        math(1, 0.0, &define(&id("Title"), &s("Job A"))),
        math(2, 20.0, &define(&id("Width"), &real("4"))),
        math(
            3,
            40.0,
            &define(&id("Out"), &app("transpose", &[row(&[id("Title"), id("Width")])])),
        ),
    ]
}

#[test]
fn a_transposed_row_of_text_names_and_numbers_is_a_table_that_prints_both() {
    let mut regions = table_regions();
    regions.push(math(4, 60.0, &define(&id("w"), &indexed("Out", &real("2")))));
    regions.push(math(5, 80.0, &eval(&id("w"))));
    regions.push(math(6, 100.0, &eval(&id("Out"))));
    let res = import(&regions);
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("w = 4"), "{t}");
    // The table shows its text and its number, not the code of the text.
    assert!(t.contains("Job A"), "{t}");
}

#[test]
fn a_two_index_element_of_a_table_is_a_gap_only_at_a_text_cell() {
    let mk = |i: &str, j: &str| {
        import(&[
            math(1, 0.0, &define(&id("Title"), &s("Job A"))),
            math(2, 20.0, &define(&id("Width"), &real("4"))),
            math(3, 40.0, &define(&id("Tab"), &row(&[id("Title"), id("Width")]))),
            math(4, 60.0, &define(&id("t"), &at2("Tab", i, j))),
            math(5, 80.0, &eval(&id("t"))),
        ])
    };
    let number = mk("1", "2");
    clean(&number);
    assert!(engine_text(&number).contains("t = 4"));
    let text = mk("1", "1");
    assert!(
        text.report
            .issues
            .iter()
            .any(|i| i.code == Code::StringValue && i.detail.as_deref().is_some_and(|d| d.contains("holds a text"))),
        "{:?}",
        text.report.issues
    );
}

#[test]
fn a_copy_of_such_a_table_is_one_too() {
    let mut regions = table_regions();
    regions.push(math(4, 60.0, &define(&id("Copy"), &id("Out"))));
    regions.push(math(5, 80.0, &eval(&id("Copy"))));
    let res = import(&regions);
    clean(&res);
    assert!(engine_text(&res).contains("Job A"));
}

#[test]
fn a_literal_index_at_a_text_cell_of_a_table_is_a_gap() {
    let mut regions = table_regions();
    regions.push(math(4, 60.0, &define(&id("t"), &indexed("Out", &real("1")))));
    let res = import(&regions);
    let why = res
        .report
        .issues
        .iter()
        .find(|i| i.code == Code::StringValue)
        .and_then(|i| i.detail.clone());
    assert!(
        why.is_some_and(|d| d.contains("holds a text")),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_column_of_text_checks_written_out_prints_each_text() {
    let col3 = format!(
        "<ml:matrix rows=\"3\" cols=\"1\">{}{}{}</ml:matrix>",
        id("CheckA"),
        id("CheckB"),
        id("Width")
    );
    let res = import(&[
        math(1, 0.0, &define(&id("x"), &real("3"))),
        math(
            2,
            20.0,
            &define(
                &id("CheckA"),
                &call("if", &[gt(&id("x"), &real("2")), s("OK"), s("No Good")]),
            ),
        ),
        math(
            3,
            40.0,
            &define(
                &id("CheckB"),
                &call("if", &[gt(&id("x"), &real("5")), s("OK"), s("No Good")]),
            ),
        ),
        math(4, 60.0, &define(&id("Width"), &real("4"))),
        math(5, 80.0, &eval(&col3)),
    ]);
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("= [ OK No Good 4 ]"), "{t}");
}

// ---- what stays a gap -------------------------------------------------------------------------------------------

#[test]
fn a_function_that_gives_a_text_or_a_number_prints_whichever_it_gives() {
    let body = prog(&[
        local("r", &s("n/a")),
        iff(&gt(&id("x"), &real("0")), &[local("r", &real("3"))]),
        id("r"),
    ]);
    let res = import(&[
        math(1, 0.0, &fun("g", &["x"], &body)),
        math(2, 20.0, &eval(&call("g", &[real("1")]))),
        math(3, 40.0, &eval(&call("g", &[real("-1")]))),
    ]);
    clean(&res);
    let t = engine_text(&res);
    assert!(t.contains("= 3") && t.contains("n/a"), "{t}");
}

#[test]
fn a_local_that_holds_a_text_and_a_number_in_a_loop_is_a_gap_that_says_so() {
    let body = prog(&[
        local("r", &s("n/a")),
        for_in(
            "i",
            &real("1"),
            &id("nn"),
            &[iff(&gt(&indexed("v", &id("i")), &real("3")), &[local("r", &real("3"))])],
        ),
        id("r"),
    ]);
    let res = import(&[
        math(1, 0.0, &define(&id("nn"), &real("3"))),
        math(2, 20.0, &define(&id("v"), &col(&[real("1"), real("4"), real("2")]))),
        math(3, 40.0, &define(&id("a"), &body)),
    ]);
    let why = res
        .report
        .issues
        .iter()
        .find(|i| i.code == Code::StringValue)
        .and_then(|i| i.detail.clone());
    assert!(
        why.is_some_and(|d| d.contains("r holds a text on one path") && d.contains("a number on another")),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_text_local_used_as_a_number_is_a_gap() {
    let body = prog(&[local("msg", &s("OK")), app("plus", &[id("msg"), real("1")])]);
    let res = import(&[math(1, 0.0, &define(&id("Res"), &body))]);
    assert!(has_gap(&res, Code::StringValue), "{:?}", res.report.issues);
}

#[test]
fn a_text_argument_used_as_a_number_is_a_gap() {
    // `kind` is compared with a text, so it is a text; adding to it is not defined.
    let body = call(
        "if",
        &[
            eq(&id("kind"), &s("main")),
            real("1"),
            app("plus", &[id("kind"), real("1")]),
        ],
    );
    let res = import(&[math(1, 0.0, &fun("bad", &["kind"], &body))]);
    assert!(has_gap(&res, Code::StringValue), "{:?}", res.report.issues);
}

#[test]
fn joining_and_formatting_texts_are_gaps_that_name_the_function() {
    let joined = call("concat", &[s("L = "), call("num2str", &[id("x")])]);
    let res = import(&[
        math(1, 0.0, &define(&id("x"), &real("2"))),
        math(2, 20.0, &define(&id("Label"), &joined)),
    ]);
    let gaps: Vec<_> = res.report.gaps().filter(|i| i.code == Code::StringValue).collect();
    assert!(
        gaps.iter()
            .any(|i| i.detail.as_deref().is_some_and(|d| d.starts_with("concat "))),
        "{:?}",
        res.report.issues
    );
    assert!(
        !res.report.issues.iter().any(|i| i.code == Code::BuiltinMissing),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_vector_that_mixes_texts_and_numbers_names_the_element_assignment() {
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
    let why = res
        .report
        .issues
        .iter()
        .find(|i| i.code == Code::StringValue)
        .and_then(|i| i.detail.clone());
    assert!(
        why.is_some_and(|d| d.contains("element of val")),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_comparison_with_a_text_of_something_unknown_is_a_gap_that_says_so() {
    let res = import(&[math(
        1,
        0.0,
        &define(&id("a"), &call("if", &[eq(&id("q"), &s("OK")), real("1"), real("2")])),
    )]);
    assert!(
        has_gap(&res, Code::StringValue) || has_gap(&res, Code::UndefinedVariable),
        "{:?}",
        res.report.issues
    );
}
