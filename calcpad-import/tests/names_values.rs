//! Names the engine cannot spell, text values, input controls and rich-text runs, on synthetic
//! worksheets written here.

use calcpad_import::{import_xmcd_opts, Code, ImportOptions, ImportResult, Severity};

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

// ---- names ------------------------------------------------------------------------------------

#[test]
fn a_name_with_dots_gets_one_engine_spelling_everywhere() {
    let res = import(&[
        math(1, 0.0, &define(&ids("SF", "crack.step2"), &real("2.5"))),
        math(
            2,
            20.0,
            &define(&id("y"), &app("mult", &[ids("SF", "crack.step2"), real("4")])),
        ),
        math(3, 40.0, &eval(&id("y"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("SF_crack_step2 = 2.5"), "{}", res.calcpad);
    assert!(res.calcpad.contains("y = SF_crack_step2*4"), "{}", res.calcpad);
    assert!(!res.report.issues.iter().any(|i| i.code == Code::InvalidName));
    assert!(engine_text(&res).contains("y = 10"));
    // The page shows the author's name: base SF with the subscript as typed.
    assert!(
        res.calcpad.contains("SF</mi>") && res.calcpad.contains("crack.step2"),
        "{}",
        res.calcpad
    );
}

#[test]
fn slash_hash_and_leading_digit_names_are_readable() {
    let res = import(&[
        math(1, 0.0, &define(&ids("U/R", "twist"), &real("0.8"))),
        math(2, 20.0, &define(&ids("Rod#", "top"), &real("5"))),
        math(3, 40.0, &define(&id("#count"), &real("12"))),
        math(
            4,
            60.0,
            &define(
                &id("t"),
                &app("plus", &[ids("U/R", "twist"), ids("Rod#", "top"), id("#count")]),
            ),
        ),
        math(5, 80.0, &eval(&id("t"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    for want in ["U_R_twist = 0.8", "RodNum_top = 5", "Num_count = 12"] {
        assert!(res.calcpad.contains(want), "missing {want} in {}", res.calcpad);
    }
    assert!(engine_text(&res).contains("t = 17.8"));
    assert!(res.calcpad.contains("U/R"), "the original name is shown");
}

#[test]
fn two_names_with_one_spelling_stay_apart() {
    // `a.b` and `a_b` (base a, subscript b) would both be a_b.
    let res = import(&[
        math(1, 0.0, &define(&id("a.b"), &real("1"))),
        math(2, 20.0, &define(&ids("a", "b"), &real("2"))),
        math(
            3,
            40.0,
            &define(
                &id("c"),
                &app("plus", &[id("a.b"), app("mult", &[ids("a", "b"), real("10")])]),
            ),
        ),
        math(4, 60.0, &eval(&id("c"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("c = 21"));
}

#[test]
fn a_unit_with_a_middle_dot_is_not_renamed() {
    let res = import(&[
        math(1, 0.0, &define(&id("M"), &app("mult", &[real("3"), id("kN\u{b7}m")]))),
        math(2, 20.0, &eval(&id("M"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.calcpad.contains("kN_m"), "{}", res.calcpad);
}

#[test]
fn force_per_length_units_are_units() {
    let res = import(&[
        math(1, 0.0, &define(&id("w"), &app("mult", &[real("200"), id("plf")]))),
        math(2, 20.0, &define(&id("g"), &app("mult", &[real("100"), id("pcf")]))),
        math(3, 40.0, &eval(&app("div", &[id("w"), id("klf")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("0.2"));
}

// ---- text values ------------------------------------------------------------------------------

fn check_sheet(x: &str) -> ImportResult {
    import(&[
        math(1, 0.0, &define(&id("x"), &real(x))),
        math(
            2,
            20.0,
            &define(
                &id("Check"),
                &call("if", &[app("greaterOrEqual", &[id("x"), real("2")]), s("OK"), s("NG")]),
            ),
        ),
        math(3, 40.0, &eval(&id("Check"))),
    ])
}

#[test]
fn a_check_that_returns_text_shows_the_text() {
    let ok = check_sheet("3");
    assert_eq!(ok.report.regions_needing_attention(), 0, "{:?}", ok.report.issues);
    assert!(ok.calcpad.contains("<ms>OK</ms>") && ok.calcpad.contains("<ms>NG</ms>"));
    let t = engine_text(&ok);
    assert!(t.contains("OK") && !t.contains("NG"), "{t}");
    let ng = check_sheet("1");
    let t = engine_text(&ng);
    assert!(t.contains("NG") && !t.contains("OK"), "{t}");
}

#[test]
fn texts_are_compared_and_chosen_by_value() {
    let res = import(&[
        math(1, 0.0, &define(&id("Key"), &s("NO"))),
        math(
            2,
            20.0,
            &define(
                &id("h"),
                &call("if", &[app("equal", &[id("Key"), s("NO")]), real("0"), real("5")]),
            ),
        ),
        math(
            3,
            40.0,
            &define(
                &id("g"),
                &call("if", &[app("equal", &[id("Key"), s("YES")]), real("0"), real("7")]),
            ),
        ),
        math(4, 60.0, &eval(&app("plus", &[id("h"), id("g")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("= 7"));
}

#[test]
fn a_function_that_returns_text_converts() {
    let f = format!(
        "<ml:define><ml:function>{}<ml:boundVars>{}{}</ml:boundVars></ml:function>{}</ml:define>",
        id("fCheck"),
        id("t"),
        id("tmin"),
        call("if", &[app("greaterOrEqual", &[id("t"), id("tmin")]), s("OK"), s("NG")])
    );
    let res = import(&[
        math(1, 0.0, &f),
        math(2, 20.0, &define(&id("r"), &call("fCheck", &[real("5"), real("3")]))),
        math(3, 40.0, &eval(&id("r"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    assert!(t.contains("OK") && !t.contains("NG"), "{t}");
}

#[test]
fn a_text_used_as_a_number_is_still_a_gap() {
    let res = import(&[
        math(1, 0.0, &define(&id("a"), &s("abc"))),
        math(2, 20.0, &define(&id("b"), &app("plus", &[id("a"), real("1")]))),
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::StringValue) || res.report.regions_needing_attention() > 0);
}

#[test]
fn a_matrix_with_text_labels_keeps_its_numbers() {
    let m = "<ml:matrix rows=\"2\" cols=\"2\"><ml:str>P</ml:str><ml:real>1.5</ml:real><ml:str>M</ml:str><ml:real>2.5</ml:real></ml:matrix>";
    let res = import(&[
        math(1, 0.0, &define(&id("T"), m)),
        math(2, 20.0, &eval(&app("plus", &[id("x"), real("0")]))),
    ]);
    // The definition converts; the labels are codes, the numbers are the file's.
    let d = res.report.kinds["math"].converted;
    assert!(d >= 1, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("T = [1; 2|1.5; 2.5]"), "{}", res.calcpad);
}

// ---- input controls ---------------------------------------------------------------------------

fn combo(selected: usize) -> String {
    format!(
        r#"<ml:ComboBoxControl rows="3" cols="1" SelectedRow="{selected}"><ml:ComboBoxRowNames><ml:rowName>Low</ml:rowName><ml:rowName>Mid</ml:rowName><ml:rowName>High</ml:rowName></ml:ComboBoxRowNames><ml:ComboBoxScaleFactors><ml:placeholder/></ml:ComboBoxScaleFactors><ml:ComboBoxValues><ml:real>10</ml:real><ml:real>20</ml:real><ml:real>30</ml:real></ml:ComboBoxValues></ml:ComboBoxControl>"#
    )
}

#[test]
fn a_combo_box_is_the_value_it_has_stored() {
    let res = import(&[
        math(1, 0.0, &define(&id("Grade"), &combo(1))),
        math(2, 20.0, &define(&id("f"), &app("mult", &[id("Grade"), real("2")]))),
        math(3, 40.0, &eval(&id("f"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("Grade = 20"), "{}", res.calcpad);
    assert!(
        res.calcpad.contains("combo box input") && res.calcpad.contains("Mid"),
        "{}",
        res.calcpad
    );
    assert!(res
        .report
        .issues
        .iter()
        .any(|i| i.code == Code::ControlInput && i.severity == Severity::Note));
    assert!(engine_text(&res).contains("f = 40"));
}

#[test]
fn a_combo_box_scale_factor_is_a_unit() {
    let c = r#"<ml:ComboBoxControl rows="2" cols="1" SelectedRow="1"><ml:ComboBoxRowNames><ml:rowName>a</ml:rowName><ml:rowName>b</ml:rowName></ml:ComboBoxRowNames><ml:ComboBoxScaleFactors><ml:id labels="UNIT">ksi</ml:id></ml:ComboBoxScaleFactors><ml:ComboBoxValues><ml:real>1.08</ml:real><ml:real>1.83</ml:real></ml:ComboBoxValues></ml:ComboBoxControl>"#;
    let res = import(&[math(1, 0.0, &define(&id("fc"), c)), math(2, 20.0, &eval(&id("fc")))]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("fc = 1.83*ksi"), "{}", res.calcpad);
    assert!(engine_text(&res).contains("1.83"));
}

fn text_box(text: &str) -> String {
    format!(
        r#"<ml:TextBoxScriptableControl SelectedIndex="0" Script="x"><ml:vals><ml:val>{text}</ml:val></ml:vals></ml:TextBoxScriptableControl>"#
    )
}

#[test]
fn a_text_box_holds_its_stored_text() {
    let res = import(&[
        math(1, 0.0, &define(&id("inpLen"), &text_box("12.5"))),
        math(
            2,
            20.0,
            &define(&id("L"), &app("mult", &[call("str2num", &[id("inpLen")]), id("ft")])),
        ),
        math(3, 40.0, &define(&id("inpName"), &text_box("SDO"))),
        math(
            4,
            60.0,
            &define(
                &id("lab"),
                &call("if", &[app("equal", &[id("inpName"), s("SDO")]), real("1"), real("0")]),
            ),
        ),
        math(5, 80.0, &eval(&app("plus", &[id("L"), id("lab")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("inpLen = 12.5"), "{}", res.calcpad);
    assert!(res.calcpad.contains("text box input"), "{}", res.calcpad);
}

#[test]
fn a_list_box_holds_the_position_of_its_selected_item() {
    let c = r#"<ml:ListBoxScriptableControl SelectedIndex="1" Script="x"><ml:vals><ml:val>one.dat</ml:val><ml:val>two.dat</ml:val></ml:vals></ml:ListBoxScriptableControl>"#;
    let res = import(&[
        math(1, 0.0, &define(&id("pick"), c)),
        math(
            2,
            20.0,
            &define(
                &id("n"),
                &call("if", &[app("equal", &[id("pick"), real("1")]), real("2"), real("1")]),
            ),
        ),
        math(3, 40.0, &eval(&id("n"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("n = 2"));
}

#[test]
fn a_button_stays_a_gap() {
    let c = r#"<ml:ButtonScriptableControl SelectedIndex="1" Script="x"><ml:vals><ml:val>Go</ml:val></ml:vals></ml:ButtonScriptableControl>"#;
    let res = import(&[math(1, 0.0, &define(&id("btn"), c))]);
    assert!(res.report.gaps().any(|i| i.code == Code::UnknownElement));
}

// ---- percent and nth roots ------------------------------------------------------------------------

#[test]
fn percent_and_nth_root_convert() {
    let res = import(&[
        math(1, 0.0, &define(&id("r"), &app("percent", &[real("2.5")]))),
        math(2, 20.0, &define(&id("c"), &app("nthRoot", &[real("3"), real("8")]))),
        math(
            3,
            40.0,
            &eval(&app("plus", &[app("mult", &[id("r"), real("100")]), id("c")])),
        ),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("= 4.5"));
}

// ---- programs that give a value by their last `if` ----------------------------------------------------

fn prog(stmts: &[String]) -> String {
    format!("<ml:program>{}</ml:program>", stmts.concat())
}
fn iff(test: &str, then: &str) -> String {
    format!(
        "<ml:if><ml:test>{test}</ml:test><ml:then>{}</ml:then></ml:if>",
        prog(&[then.to_string()])
    )
}
fn fun(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| id(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        id(name)
    )
}

#[test]
fn the_last_if_that_runs_gives_the_value() {
    // f(n): 0.26 below 20, then 0.5 below 16: the value is the last one that ran.
    let body = prog(&[
        iff(&app("lessThan", &[id("n"), real("20")]), &real("0.26")),
        iff(&app("lessThan", &[id("n"), real("16")]), &real("0.5")),
    ]);
    let res = import(&[
        math(1, 0.0, &fun("f", &["n"], &body)),
        math(2, 20.0, &eval(&call("f", &[real("10")]))),
        math(3, 40.0, &eval(&call("f", &[real("18")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.report.notes().any(|i| i.code == Code::UncoveredPath));
    let t = engine_text(&res);
    assert!(t.contains("= 0.5") && t.contains("= 0.26"), "{t}");
}

#[test]
fn alsoif_is_an_else_if_and_return_of_an_assignment_returns_its_value() {
    let alsoif = format!(
        "<ml:alsoif><ml:test>{}</ml:test><ml:then>{}</ml:then></ml:alsoif>",
        app("lessThan", &[id("n"), real("10")]),
        prog(&[real("2")])
    );
    let first = format!(
        "<ml:if><ml:test>{}</ml:test><ml:then>{}</ml:then>{alsoif}<ml:else>{}</ml:else></ml:if>",
        app("lessThan", &[id("n"), real("5")]),
        prog(&[format!(
            "<ml:return><ml:localDefine>{}{}</ml:localDefine></ml:return>",
            id("r"),
            real("1")
        )]),
        prog(&[real("3")])
    );
    let res = import(&[
        math(1, 0.0, &fun("g", &["n"], &prog(&[first]))),
        math(2, 20.0, &eval(&call("g", &[real("3")]))),
        math(3, 40.0, &eval(&call("g", &[real("7")]))),
        math(4, 60.0, &eval(&call("g", &[real("12")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    assert!(t.contains("= 1") && t.contains("= 2") && t.contains("= 3"), "{t}");
}

#[test]
fn a_chained_comparison_is_both_comparisons() {
    let chain = app("lessOrEqual", &[app("lessThan", &[real("1"), id("x")]), real("3")]);
    let res = import(&[
        math(1, 0.0, &define(&id("x"), &app("mult", &[real("2"), id("m")]))),
        math(2, 20.0, &define(&id("y"), &call("if", &[chain, real("1"), real("0")]))),
        math(3, 40.0, &eval(&id("y"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(engine_text(&res).contains("y = 1"));
}

#[test]
fn a_message_set_by_ifs_is_a_text() {
    let body = prog(&[
        iff(
            &app("greaterThan", &[id("x"), real("1")]),
            "<ml:localDefine><ml:id>msg</ml:id><ml:str>OK</ml:str></ml:localDefine>",
        ),
        iff(
            &app("lessOrEqual", &[id("x"), real("1")]),
            "<ml:localDefine><ml:id>msg</ml:id><ml:str>LOW</ml:str></ml:localDefine>",
        ),
        id("msg"),
    ]);
    let res = import(&[
        math(1, 0.0, &define(&id("x"), &real("0.5"))),
        math(2, 20.0, &define(&id("Check"), &body)),
        math(3, 40.0, &eval(&id("Check"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    assert!(t.contains("LOW") && !t.contains("OK"), "{t}");
}

#[test]
fn a_text_on_its_own_is_a_label() {
    let res = import(&[math(1, 0.0, &s("Case"))]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("<ms>Case</ms>"), "{}", res.calcpad);
}

#[test]
fn an_empty_text_box_read_as_a_number_is_zero_and_says_so() {
    let res = import(&[
        math(1, 0.0, &define(&id("inpD"), &text_box(""))),
        math(
            2,
            20.0,
            &define(&id("D"), &app("mult", &[call("str2num", &[id("inpD")]), id("in")])),
        ),
        math(3, 40.0, &eval(&id("D"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.report.notes().any(|i| i.code == Code::ControlInput));
}

#[test]
fn a_bare_zero_in_a_sum_takes_the_units_of_the_other_term() {
    let res = import(&[
        math(
            1,
            0.0,
            &define(
                &id("a"),
                &app("minus", &[real("0"), app("mult", &[real("6"), id("in")])]),
            ),
        ),
        math(2, 20.0, &eval(&id("a"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    assert!(t.contains("a = \u{2212} 6 in") || t.contains("a = - 6 in"), "{t}");
}

fn local(name: &str, value: &str) -> String {
    format!("<ml:localDefine>{}{}</ml:localDefine>", id(name), value)
}

#[test]
fn a_program_that_ends_with_an_assignment_gives_that_value() {
    // k := 2; if n > 1 { k := 3 }; k := k * 10
    let body = prog(&[
        local("k", &real("2")),
        iff(&app("greaterThan", &[id("n"), real("1")]), &local("k", &real("3"))),
        local("k", &app("mult", &[id("k"), real("10")])),
    ]);
    let res = import(&[
        math(1, 0.0, &fun("h", &["n"], &body)),
        math(2, 20.0, &eval(&call("h", &[real("5")]))),
        math(3, 40.0, &eval(&call("h", &[real("0")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let t = engine_text(&res);
    assert!(t.contains("= 30") && t.contains("= 20"), "{t}");
}

#[test]
fn a_block_inside_a_block_runs_in_place_and_many_ifs_do_not_multiply() {
    // Eight value-only ifs in a row, inside a nested block: one expression, not 2^8 paths.
    let mut stmts = Vec::new();
    for k in 1..=8 {
        stmts.push(iff(
            &app("lessThan", &[id("n"), real(&(k * 10).to_string())]),
            &real(&k.to_string()),
        ));
    }
    let inner = format!("<ml:program>{}</ml:program>", stmts.concat());
    let body = prog(&[local("m", &real("1")), inner]);
    let res = import(&[
        math(1, 0.0, &fun("q", &["n"], &body)),
        math(2, 20.0, &eval(&call("q", &[real("35")]))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    // The last if that runs is n < 80 (k = 8).
    assert!(engine_text(&res).contains("= 8"));
}
