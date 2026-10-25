//! Tests on hand-written synthetic worksheets. None of the XML here comes from a real file.
//! Namespaces use a reserved host: the reader matches them by their last path segment.

use calcpad_import::ir::{MathRegion, Region, RegionKind, Stmt};
use calcpad_import::translate::reading_order;
use calcpad_import::{
    import_xmcd_opts, import_xmcd_with, Code, ImportOptions, ImportResult, Limits, Severity, GAP_MARKER,
};

mod common;
use common::legacy;

/// These tests read the structure of the page (gap lines, `#noc` blocks), which is the typeset-Calcpad
/// display; the MathML display is covered in `mathml.rs`.
fn import_xmcd(xml: &str) -> ImportResult {
    let opts = ImportOptions {
        mathml: false,
        ..Default::default()
    };
    import_xmcd_opts(xml, &opts)
}

// ---- builders ---------------------------------------------------------------------------------

fn sheet_with_origin(origin: i64, regions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet version="3.0.3" xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10">
<settings><calculation><builtInVariables array-origin="{origin}"/></calculation></settings>
<regions>{regions}</regions>
</worksheet>"#
    )
}

fn sheet(regions: &str) -> String {
    sheet_with_origin(1, regions)
}

fn region(id: u32, left: f64, top: f64, content: &str) -> String {
    format!(
        r#"<region region-id="{id}" left="{left}" top="{top}" width="10" height="10" align-y="{}">{content}</region>"#,
        top + 10.0
    )
}

fn math(id: u32, left: f64, top: f64, inner: &str) -> String {
    region(id, left, top, &format!("<math>{inner}</math>"))
}

fn n(name: &str) -> String {
    format!("<ml:id>{name}</ml:id>")
}
fn ns(name: &str, sub: &str) -> String {
    format!(r#"<ml:id subscript="{sub}">{name}</ml:id>"#)
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

/// The output lines without the banner. A gap is one line again: its marker comment (as
/// `'[import-gap Rn codes] reason`) and, after ` | `, the lines of its equation block.
fn lines(res: &ImportResult) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_noc = false;
    let mut in_gap = false;
    let code = legacy(&res.calcpad);
    for l in code.lines().filter(|l| {
        !l.starts_with("'<!-- Imported") && !l.starts_with("'<style>") && !l.starts_with("'<!--{\"LintIgnore")
    }) {
        if let Some(rest) = l.strip_prefix("'<!-- [import-gap") {
            let head = rest.split(" --><small>").next().unwrap_or(rest);
            out.push(format!("'[import-gap{}", head.replacen(" -->", "", 1)));
            in_gap = true;
            continue;
        }
        if in_gap {
            if l == "#noc" {
                in_noc = true;
                continue;
            }
            if l == "#end noc" {
                in_noc = false;
                continue;
            }
            if in_noc || l.starts_with("'<code>") || l.starts_with("'&emsp;") {
                let last = out.last_mut().unwrap();
                last.push_str(if last.contains(" | ") { " ; " } else { " | " });
                last.push_str(l.trim_start_matches('\''));
                continue;
            }
            in_gap = false;
        }
        out.push(l.to_string());
    }
    out
}

fn one(inner: &str) -> ImportResult {
    import_xmcd(&sheet(&math(1, 0.0, 0.0, inner)))
}

fn engine_html(code: &str) -> String {
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(code), true);
    parser.html_result.clone()
}

fn assert_engine_clean(code: &str) {
    let html = engine_html(code);
    assert!(!html.contains("class=\"err\""), "engine error in:\n{code}\n---\n{html}");
}

// ---- definitions, evaluations, arithmetic -------------------------------------------------------

#[test]
fn definition_and_evaluation() {
    let xml = sheet(
        &[
            math(1, 0.0, 0.0, &define("a", &r("2"))),
            math(
                2,
                0.0,
                20.0,
                &define("b", &app("plus", &[app("mult", &[n("a"), r("3")]), r("1")])),
            ),
            math(3, 0.0, 40.0, &eval(&n("b"))),
        ]
        .concat(),
    );
    let res = import_xmcd(&xml);
    assert_eq!(lines(&res), vec!["a = 2", "b = a*3 + 1", "b"]);
    assert_eq!(res.report.regions_needing_attention(), 0);
    assert_eq!(res.report.kinds["math"].converted, 3);
    assert_engine_clean(&res.calcpad);
}

#[test]
fn operators_precedence_and_parentheses() {
    // (a + b) * c, a - (b - c), a / (b * c), -(a ^ 2), (-a) ^ 2, a ^ (b + c), 2 ^ -1
    let cases: Vec<(String, &str)> = vec![
        (app("mult", &[app("plus", &[n("a"), n("b")]), n("c")]), "(a + b)*c"),
        (app("minus", &[n("a"), app("minus", &[n("b"), n("c")])]), "a - (b - c)"),
        (app("div", &[n("a"), app("mult", &[n("b"), n("c")])]), "a/(b*c)"),
        (app("neg", &[app("pow", &[n("a"), r("2")])]), "-a^2"),
        (app("pow", &[app("neg", &[n("a")]), r("2")]), "(-a)^2"),
        (app("pow", &[n("a"), app("plus", &[n("b"), n("c")])]), "a^(b + c)"),
        (app("pow", &[r("2"), r("-1")]), "2^(-1)"),
        (app("mult", &[n("a"), app("neg", &[n("b")])]), "a*(-b)"),
        (app("mult", &[n("a"), app("div", &[n("b"), n("c")])]), "a*(b/c)"),
        (app("minus", &[n("a")]), "-a"),
        (app("plus", &[n("a"), n("b"), n("c")]), "a + b + c"),
    ];
    let mut regs = String::from_iter(
        ["a", "b", "c"]
            .iter()
            .enumerate()
            .map(|(i, v)| math(i as u32 + 1, 0.0, i as f64 * 20.0, &define(v, &r("3")))),
    );
    for (i, (e, _)) in cases.iter().enumerate() {
        regs.push_str(&math(10 + i as u32, 0.0, 100.0 + i as f64 * 20.0, &eval(e)));
    }
    let res = import_xmcd(&sheet(&regs));
    let got = lines(&res);
    for (i, (_, want)) in cases.iter().enumerate() {
        assert_eq!(&got[3 + i], want, "case {i}");
    }
    assert_engine_clean(&res.calcpad);
}

#[test]
fn roots_abs_factorial_and_comparison() {
    let res = one(&eval(&app("sqrt", &[r("4")])));
    assert_eq!(lines(&res), vec!["sqrt(4)"]);
    let res = one(&eval(&app("absval", &[r("-4")])));
    assert_eq!(lines(&res), vec!["abs(-4)"]);
    let res = one(&eval(&app("factorial", &[r("4")])));
    assert_eq!(lines(&res), vec!["4!"]);
    let res = one(&eval(&app("lessThan", &[r("1"), r("2")])));
    assert_eq!(lines(&res), vec!["1 < 2"]);
    let res = one(&eval(&app("equal", &[r("1"), r("2")])));
    assert_eq!(lines(&res), vec!["1 ≡ 2"]);
}

#[test]
fn scientific_notation_is_expanded() {
    for (lit, want) in [
        ("1.5e-3", "0.0015"),
        ("2.5E+2", "250"),
        ("1e3", "1000"),
        ("-4.5e-1", "-0.45"),
        ("1.25e2", "125"),
        ("0e5", "0"),
        ("1e-30", "(1*10^(-30))"),
    ] {
        let res = one(&eval(&r(lit)));
        assert_eq!(lines(&res), vec![want.to_string()], "{lit}");
    }
    assert_engine_clean(&import_xmcd(&sheet(&math(1, 0.0, 0.0, &eval(&r("1.5e-3"))))).calcpad);
}

#[test]
fn bad_numbers_are_gaps_not_text() {
    let res = one(&eval(&r("1 2")));
    assert_eq!(res.report.regions_needing_attention(), 1);
    assert!(res.report.issues.iter().any(|i| i.code == Code::BadNumber));
    assert!(lines(&res)[0].starts_with(&format!("'{GAP_MARKER}")));
}

// ---- built-in functions ---------------------------------------------------------------------------

#[test]
fn builtin_function_table() {
    let res = one(&eval(&call("ceil", &[r("2.1")])));
    assert_eq!(lines(&res), vec!["ceiling(2.1)"]);
    let res = one(&eval(&call("max", &[r("1"), r("2"), r("3")])));
    assert_eq!(lines(&res), vec!["max(1; 2; 3)"]);
    let res = one(&eval(&call("ln", &[r("2")])));
    assert_eq!(lines(&res), vec!["ln(2)"]);
    let res = one(&eval(&call("log", &[r("100")])));
    assert_eq!(lines(&res), vec!["log(100)"]);
    let res = one(&eval(&call("exp", &[r("1")])));
    assert_eq!(lines(&res), vec!["exp(1)"]);
    let res = one(&eval(&call("floor", &[r("2.5")])));
    assert_eq!(lines(&res), vec!["floor(2.5)"]);
    let res = one(&eval(&call("mod", &[r("7"), r("3")])));
    assert_eq!(lines(&res), vec!["mod(7; 3)"]);
}

#[test]
fn log_with_base_and_round_with_digits_are_noted() {
    let res = one(&eval(&call("log", &[r("8"), r("2")])));
    assert_eq!(lines(&res), vec!["(ln(8)/ln(2))"]);
    assert!(res.report.notes().any(|i| i.code == Code::LogBase));
    assert_eq!(res.report.regions_needing_attention(), 0);
    let res = one(&eval(&call("round", &[r("2.567"), r("2")])));
    assert_eq!(lines(&res), vec!["(round((2.567)*10^(2))/10^(2))"]);
    assert!(res.report.notes().any(|i| i.code == Code::RoundDigits));
    assert_engine_clean(&res.calcpad);
}

#[test]
fn trig_switches_the_output_to_radians() {
    let res = one(&eval(&call("sin", &[r("1")])));
    assert!(res.calcpad.lines().any(|l| l == "#rad"));
    assert!(res.report.notes().any(|i| i.code == Code::AngleMode));
    // No trig, no directive.
    let res = one(&eval(&r("1")));
    assert!(!res.calcpad.contains("#rad"));
    // The generated text runs, and in radians sin(1) is 0.84...
    let sheet = sheet(&math(1, 0.0, 0.0, &eval(&call("sin", &[r("1")]))));
    let html = engine_html(&import_xmcd(&sheet).calcpad);
    assert!(html.contains("0.84"), "{html}");
}

#[test]
fn unmapped_and_undefined_functions_are_told_apart() {
    let res = one(&eval(&call("stdev", &[r("1"), r("2")])));
    let i = res.report.gaps().next().unwrap();
    assert_eq!((i.code, i.detail.as_deref()), (Code::BuiltinMissing, Some("stdev")));
    let res = one(&eval(&call("vendor_fn", &[r("1")])));
    let i = res.report.gaps().next().unwrap();
    assert_eq!((i.code, i.detail.as_deref()), (Code::AddinFunction, Some("vendor_fn")));
}

#[test]
fn wrong_argument_count_is_a_gap() {
    let res = one(&eval(&call("sin", &[r("1"), r("2")])));
    assert!(res.report.gaps().any(|i| i.code == Code::ArityMismatch));
}

// ---- user functions ---------------------------------------------------------------------------------

#[test]
fn function_definition_and_call() {
    let xml = sheet(
        &[
            math(
                1,
                0.0,
                0.0,
                &func("f", &["x", "y"], &app("plus", &[app("pow", &[n("x"), r("2")]), n("y")])),
            ),
            math(2, 0.0, 20.0, &eval(&call("f", &[r("2"), r("3")]))),
            math(3, 0.0, 40.0, &eval(&call("f", &[r("2")]))),
        ]
        .concat(),
    );
    let res = import_xmcd(&xml);
    let l = lines(&res);
    assert_eq!(l[0], "f(x; y) = x^2 + y");
    assert_eq!(l[1], "f(2; 3)");
    assert!(l[2].starts_with(&format!("'{GAP_MARKER}")), "{}", l[2]);
    assert!(res.report.gaps().any(|i| i.code == Code::ArityMismatch));
    assert_engine_clean(&format!("{}\n{}\n", l[0], l[1]));
}

#[test]
fn parameters_shadow_nothing_outside_the_function() {
    // `x` exists only as a parameter, so using it outside is undefined.
    let xml = sheet(
        &[
            math(1, 0.0, 0.0, &func("f", &["x"], &n("x"))),
            math(2, 0.0, 20.0, &eval(&n("x"))),
        ]
        .concat(),
    );
    let res = import_xmcd(&xml);
    assert!(res
        .report
        .gaps()
        .any(|i| i.code == Code::UndefinedVariable && i.detail.as_deref() == Some("x")));
}

#[test]
fn variable_applied_like_a_function_is_a_product_with_a_note() {
    let xml = sheet(
        &[
            math(1, 0.0, 0.0, &define("k", &r("3"))),
            math(2, 0.0, 20.0, &eval(&call("k", &[r("2")]))),
        ]
        .concat(),
    );
    let res = import_xmcd(&xml);
    assert_eq!(lines(&res)[1], "k*2");
    assert!(res.report.notes().any(|i| i.code == Code::CallAsMultiplication));
}

// ---- vectors, matrices, indexing, origin ----------------------------------------------------------------

fn matrix(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}

#[test]
fn matrices_are_read_column_major() {
    // Stored column by column: [1 3; 2 4] is 1,2 then 3,4.
    let res = one(&define("M", &matrix(2, 2, &["1", "2", "3", "4"])));
    assert_eq!(lines(&res), vec!["M = [1; 3|2; 4]"]);
    // A 3x1 vector is a Calcpad vector, a 1x3 is a transposed column matrix.
    let res = one(&define("v", &matrix(3, 1, &["1", "2", "3"])));
    assert_eq!(lines(&res), vec!["v = [1; 2; 3]"]);
    let res = one(&define("w", &matrix(1, 3, &["1", "2", "3"])));
    assert_eq!(lines(&res), vec!["w = transp([1|2|3])"]);
    assert_engine_clean(&format!("{}\n", lines(&res)[0]));
    // A size that does not match the item count is rejected.
    let res = one(&define("M", &matrix(2, 2, &["1", "2", "3"])));
    assert!(res.report.gaps().any(|i| i.code == Code::BadStructure));
}

fn index(base: &str, idx: &[String]) -> String {
    if idx.len() == 1 {
        format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", n(base), idx[0])
    } else {
        format!(
            "<ml:apply><ml:indexer/>{}<ml:sequence>{}</ml:sequence></ml:apply>",
            n(base),
            idx.join("")
        )
    }
}

#[test]
fn element_assignment_poisons_the_array_name() {
    // After `v[1] := 9` (not converted) the earlier value of `v` is stale, and a name such as `L`
    // that is also a unit must not quietly turn into the unit.
    let regs = [
        math(1, 0.0, 0.0, &define("v", &matrix(2, 1, &["1", "2"]))),
        math(
            2,
            0.0,
            20.0,
            &format!("<ml:define>{}{}</ml:define>", index("v", &[n("kk")]), r("9")),
        ),
        math(3, 0.0, 40.0, &eval(&n("v"))),
        math(
            4,
            0.0,
            60.0,
            &format!("<ml:define>{}{}</ml:define>", index("L", &[n("kk")]), r("9")),
        ),
        math(5, 0.0, 80.0, &eval(&n("L"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let dep: Vec<_> = res
        .report
        .gaps()
        .filter(|i| i.code == Code::DependsOnGap)
        .filter_map(|i| i.region_id.clone())
        .collect();
    assert_eq!(dep, vec!["3", "5"], "{:?}", res.report.issues);
    assert!(!lines(&res).iter().any(|l| l == "v" || l == "L"));
}

#[test]
fn matrix_of_names_target_poisons_each_name() {
    // `[P, H] := f(...)` is not converted, and P and H must not keep an earlier value.
    let regs = [
        math(1, 0.0, 0.0, &define("P", &r("1200"))),
        math(
            2,
            0.0,
            20.0,
            &format!(
                "<ml:define><ml:matrix rows=\"2\" cols=\"1\">{}{}</ml:matrix>{}</ml:define>",
                n("P"),
                n("H"),
                call("solver", &[r("1")])
            ),
        ),
        math(3, 0.0, 40.0, &eval(&n("P"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert!(res
        .report
        .gaps()
        .any(|i| i.code == Code::BadStructure && i.region_id.as_deref() == Some("2")));
    assert!(res
        .report
        .gaps()
        .any(|i| i.code == Code::DependsOnGap && i.region_id.as_deref() == Some("3")));
    assert!(!lines(&res).iter().any(|l| l == "P"));
}

#[test]
fn indexing_is_shifted_from_origin_zero() {
    let regs = [
        math(1, 0.0, 0.0, &define("v", &matrix(3, 1, &["10", "20", "30"]))),
        math(2, 0.0, 20.0, &define("i", &r("1"))),
        math(3, 0.0, 40.0, &eval(&index("v", &[r("0")]))),
        math(4, 0.0, 60.0, &eval(&index("v", &[n("i")]))),
        math(5, 0.0, 80.0, &eval(&index("v", &[app("plus", &[n("i"), r("1")])]))),
        math(6, 0.0, 100.0, &define("M", &matrix(2, 2, &["1", "2", "3", "4"]))),
        math(7, 0.0, 120.0, &eval(&index("M", &[r("1"), r("0")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet_with_origin(0, &regs));
    let l = lines(&res);
    assert_eq!(l[2], "v.(1)");
    assert_eq!(l[3], "v.(i + 1)");
    assert_eq!(l[4], "v.(i + 1 + 1)");
    assert_eq!(l[6], "M.(2; 1)");
    assert!(res.report.notes().any(|i| i.code == Code::ArrayOrigin));
    assert_eq!(res.report.origin, 0);
    // The engine agrees: v[0] is the first element (10), v[i=1] the second (20).
    let html = engine_html(&res.calcpad);
    assert!(!html.contains("class=\"err\""), "{html}");
    assert!(html.contains("= 10"), "{html}");
    assert!(html.contains("= 20"), "{html}");
}

#[test]
fn indexing_is_unchanged_at_origin_one() {
    let regs = [
        math(1, 0.0, 0.0, &define("v", &matrix(2, 1, &["10", "20"]))),
        math(2, 0.0, 20.0, &eval(&index("v", &[r("2")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet_with_origin(1, &regs));
    assert_eq!(lines(&res)[1], "v.(2)");
    assert!(!res.report.notes().any(|i| i.code == Code::ArrayOrigin));
}

#[test]
fn origin_can_be_assigned_in_the_sheet() {
    let regs = [
        math(1, 0.0, 0.0, &define("v", &matrix(2, 1, &["10", "20"]))),
        math(2, 0.0, 20.0, &define("ORIGIN", &r("1"))),
        math(3, 0.0, 40.0, &eval(&index("v", &[r("1")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet_with_origin(0, &regs));
    assert_eq!(lines(&res).last().unwrap(), "v.(1)");
    assert!(res.report.notes().any(|i| i.code == Code::OriginAssigned));
}

// ---- names ----------------------------------------------------------------------------------------------

#[test]
fn subscripts_and_greek_names() {
    let regs = [
        math(
            1,
            0.0,
            0.0,
            &format!("<ml:define>{}{}</ml:define>", ns("x", "i"), r("2")),
        ),
        math(
            2,
            0.0,
            20.0,
            &format!("<ml:define>{}{}</ml:define>", ns("ρ", "tp"), r("3")),
        ),
        math(3, 0.0, 40.0, &eval(&app("mult", &[ns("x", "i"), ns("ρ", "tp")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(lines(&res), vec!["x_i = 2", "ρ_tp = 3", "x_i*ρ_tp"]);
    assert_engine_clean(&res.calcpad);
}

#[test]
fn names_calcpad_cannot_spell_are_gaps() {
    let res = one(&define("a b", &r("1")));
    assert!(res.report.gaps().any(|i| i.code == Code::InvalidName));
    let res = one(&define("1a", &r("1")));
    assert!(res.report.gaps().any(|i| i.code == Code::InvalidName));
}

#[test]
fn builtin_constants() {
    let res = one(&eval(&app("mult", &[n("π"), r("2")])));
    assert_eq!(lines(&res), vec!["π*2"]);
    let res = one(&eval(&n("g")));
    assert_eq!(lines(&res), vec!["(9.80665*m/s^2)"]);
}

#[test]
fn undefined_before_defined_follows_reading_order() {
    // `b` is used above where it is defined: reading order makes it undefined there.
    let regs = [
        math(1, 0.0, 0.0, &eval(&n("b"))),
        math(2, 0.0, 20.0, &define("b", &r("1"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert!(res.report.gaps().any(|i| i.code == Code::UndefinedVariable));
}

#[test]
fn global_definitions_convert_with_a_note() {
    let res = one(&format!("<ml:globalDefine>{}{}</ml:globalDefine>", n("c"), r("5")));
    // Defined out of sight at the top, as a global holds everywhere, and converted where it stands.
    assert_eq!(lines(&res), vec!["#hide", "c = 5", "#show", "c = 5"]);
    assert!(res.report.notes().any(|i| i.code == Code::GlobalDefinition));
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn a_constant_global_defined_below_its_use_is_still_defined_there() {
    // gg == 4 is written under the line that uses it; a global holds from the top.
    let regs = [
        math(1, 0.0, 0.0, &eval(&app("mult", &[n("gg"), r("2")]))),
        math(
            2,
            0.0,
            20.0,
            &format!("<ml:globalDefine>{}{}</ml:globalDefine>", n("gg"), r("4")),
        ),
        math(
            3,
            0.0,
            40.0,
            &format!(
                "<ml:globalDefine>{}{}</ml:globalDefine>",
                n("hh"),
                app("mult", &[n("gg"), r("2")])
            ),
        ),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert!(
        !res.report.gaps().any(|i| i.code == Code::UndefinedVariable),
        "{:?}",
        res.report.issues
    );
    // Only the constant is hoisted: h reads another name, so it stays where it is.
    assert_eq!(lines(&res)[..3], ["#hide", "gg = 4", "#show"]);
    assert_engine_clean(&res.calcpad);
}

#[test]
fn provenance_wrappers_are_peeled() {
    let inner = define("a", &r("2"));
    let wrapped = format!(
        r#"<ml:provenance expr-id="1"><originRef xmlns="http://example.invalid/provenance10" doc-id="x"><hash/></originRef>{inner}</ml:provenance>"#
    );
    let res = one(&wrapped);
    assert_eq!(lines(&res), vec!["a = 2"]);
}

#[test]
fn definition_with_display_is_a_definition() {
    // `x := expr =` stores an evaluation inside the definition; a display unit is dropped.
    let res = one(&define("d", &eval(&app("mult", &[r("2"), n("m")]))));
    assert_eq!(lines(&res), vec!["d = 2*m"]);
}

// ---- units ----------------------------------------------------------------------------------------------

#[test]
fn units_map_and_run_in_the_engine() {
    let regs = [
        math(1, 0.0, 0.0, &define("P", &app("mult", &[r("1200"), n("psi")]))),
        math(2, 0.0, 20.0, &define("t", &app("mult", &[r("2"), n("hr")]))),
        math(3, 0.0, 40.0, &define("m1", &app("mult", &[r("3"), n("gm")]))),
        math(4, 0.0, 60.0, &eval(&app("div", &[n("P"), n("MPa")]))),
        math(5, 0.0, 80.0, &define("e0", &app("mult", &[r("1"), n("joule")]))),
        math(6, 0.0, 100.0, &define("mu", &app("mult", &[r("2"), n("\u{00B5}Pa")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    assert_eq!(l[0], "P = 1200*psi");
    assert_eq!(l[1], "t = 2*h");
    assert_eq!(l[2], "m1 = 3*g");
    assert_eq!(l[3], "P/MPa");
    assert_eq!(l[4], "e0 = 1*J");
    assert_eq!(l[5], "mu = 2*μPa");
    assert_eq!(res.report.regions_needing_attention(), 0);
    assert_engine_clean(&res.calcpad);
}

#[test]
fn postfix_unit_application() {
    let postfix = format!(r#"<ml:apply fixity="postfix">{}{}</ml:apply>"#, n("degF"), r("30"));
    let res = one(&eval(&postfix));
    assert_eq!(lines(&res), vec!["30*°F"]);
    assert_engine_clean(&res.calcpad);
}

#[test]
fn unit_override_becomes_a_conversion() {
    let regs = [
        math(1, 0.0, 0.0, &define("p", &app("mult", &[r("100"), n("kPa")]))),
        math(
            2,
            0.0,
            20.0,
            &format!(
                "<ml:eval>{}<ml:unitOverride>{}</ml:unitOverride></ml:eval>",
                app("plus", &[n("p"), app("mult", &[r("50"), n("kPa")])]),
                n("psi")
            ),
        ),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(lines(&res)[1], "(p + 50*kPa)|psi");
    assert!(res.report.notes().any(|i| i.code == Code::UnitOverride));
    assert_engine_clean(&res.calcpad);
}

#[test]
fn unmapped_units_are_listed() {
    let res = one(&define("q", &app("mult", &[r("2"), n("kmol")])));
    let i = res.report.gaps().next().unwrap();
    assert_eq!((i.code, i.detail.as_deref()), (Code::UnmappedUnit, Some("kmol")));
}

#[test]
fn every_mapped_unit_exists_in_the_engine() {
    use calcpad_core::base_types::unit::Unit;
    use calcpad_import::tables::{RENAMED_UNITS, SAME_UNITS};
    for u in SAME_UNITS.iter().copied().chain(RENAMED_UNITS.iter().map(|(_, c)| *c)) {
        // `%` is parsed as a unit sign by the engine rather than stored in the table.
        if u == "%" {
            continue;
        }
        assert!(Unit::exists(u), "unit {u:?} is not in the engine's table");
    }
}

#[test]
fn every_mapped_function_runs_in_the_engine() {
    use calcpad_import::tables::{FnRule, FUNCTIONS};
    for (src, rule, lo, _) in FUNCTIONS {
        let target = match rule {
            FnRule::Direct(t) | FnRule::Trig(t) => *t,
            FnRule::Log => "log",
            FnRule::Round => "round",
            FnRule::Submatrix => "submatrix",
        };
        // Matrix functions take matrix arguments; give them one.
        let arg = match target {
            "n_rows" | "n_cols" | "len" | "sort" | "reverse" | "norm_2" | "mean" => "[1; 2; 3]",
            "det" | "rank" => "[1; 2|3; 4]",
            "identity" | "matrix" => "2",
            "augment" | "stack" => "[1; 2]",
            "lsolve" => "[1; 2|3; 4]",
            "if" => "1",
            "acos" | "asin" | "atanh" => "0.5",
            "acosh" => "2",
            _ => "1",
        };
        let args: Vec<&str> = match target {
            "if" => vec!["1 > 0", "2", "3"],
            "matrix" => vec!["2", "2"],
            "augment" | "stack" => vec![arg, arg],
            "lsolve" => vec!["[1; 2|3; 4]", "[1; 2]"],
            "col" | "row" => vec!["[1; 2|3; 4]", "1"],
            "submatrix" => vec!["[1; 2|3; 4]", "1", "2", "1", "2"],
            "mod" | "gcd" | "lcm" | "atan2" => vec!["4", "6"],
            _ => vec![arg; (*lo).max(1)],
        };
        let code = format!("{}({})\n", target, args.join("; "));
        let html = engine_html(&code);
        assert!(!html.contains("class=\"err\""), "{src} -> {code}: {html}");
    }
}

// ---- gaps: ranges, programs, solve blocks, symbolic, plots, images ----------------------------------------

/// A range with a step (`1, 2 .. last`): not read, so it is the standing example of a gap.
fn range(first: &str, last: &str) -> String {
    format!("<ml:range><ml:sequence>{first}<ml:real>2</ml:real></ml:sequence>{last}</ml:range>")
}

#[test]
fn range_variable_is_a_marked_placeholder_with_downstream_counts() {
    let regs = [
        math(1, 0.0, 0.0, &define("i", &range(&r("1"), &r("10")))),
        math(2, 0.0, 20.0, &define("y", &app("mult", &[n("i"), r("2")]))),
        math(3, 0.0, 40.0, &eval(&n("y"))),
        math(4, 0.0, 60.0, &define("ok", &r("1"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    assert!(
        l[0].starts_with(&format!("'{GAP_MARKER} R1 range-variable]")),
        "{}",
        l[0]
    );
    assert!(
        l[1].starts_with(&format!("'{GAP_MARKER} R2 depends-on-gap]")),
        "{}",
        l[1]
    );
    assert!(
        l[2].starts_with(&format!("'{GAP_MARKER} R3 depends-on-gap]")),
        "{}",
        l[2]
    );
    assert_eq!(l[3], "ok = 1");
    assert_eq!(res.report.regions_needing_attention(), 3);
    let ranked = res.report.ranked_gaps();
    assert_eq!(ranked[0].code, Code::RangeVariable);
    assert_eq!((ranked[0].direct, ranked[0].downstream), (1, 2));
    // The report points at the regions and their lines.
    let first = res.report.gaps().next().unwrap();
    assert_eq!(first.region_id.as_deref(), Some("1"));
    assert!(first.line.unwrap() >= 4);
    // The placeholder is harmless to the engine.
    assert_engine_clean(&res.calcpad);
}

#[test]
fn programs_solve_blocks_and_calculus_are_counted_not_translated() {
    // A return from inside a loop is an early exit, which a function body cannot express.
    let program = "<ml:program><ml:for><ml:id>i</ml:id><ml:range><ml:real>1</ml:real><ml:id>x</ml:id></ml:range><ml:program><ml:return><ml:id>i</ml:id></ml:return></ml:program></ml:for><ml:real>0</ml:real></ml:program>";
    let res = one(&func("f", &["x"], program));
    assert!(res.report.gaps().any(|i| i.code == Code::ProgramBlock));
    let res = one(&eval(&call("Find", &[n("x")])));
    assert!(res.report.gaps().any(|i| i.code == Code::SolveBlock));
    let res = one(&n("Given"));
    assert!(res.report.gaps().any(|i| i.code == Code::SolveBlock));
    let deriv = "<ml:apply><ml:derivative/><ml:lambda><ml:boundVars><ml:id>x</ml:id></ml:boundVars><ml:id>x</ml:id></ml:lambda></ml:apply>";
    let res = one(&eval(deriv));
    assert!(res.report.gaps().any(|i| i.code == Code::CalculusOperator));
    let res = one("<ml:symEval><ml:id>x</ml:id></ml:symEval>");
    assert!(res.report.gaps().any(|i| i.code == Code::SymbolicEvaluation));
    let res = one("<ml:somethingNew/>");
    let g = res.report.gaps().next().unwrap();
    assert_eq!(
        (g.code, g.detail.as_deref()),
        (Code::UnknownElement, Some("somethingNew"))
    );
}

#[test]
fn string_values_and_bare_expressions_and_disabled_regions() {
    let res = one(&define("s", "<ml:str>hello</ml:str>"));
    assert!(res.report.gaps().any(|i| i.code == Code::StringValue));
    let res = one(&app("plus", &[r("1"), r("2")]));
    assert!(res.report.gaps().any(|i| i.code == Code::BareExpression));
    let xml = sheet(&region(
        1,
        0.0,
        0.0,
        &format!(r#"<math disable-calc="true">{}</math>"#, define("a", &r("1"))),
    ));
    let res = import_xmcd(&xml);
    assert!(res.report.gaps().any(|i| i.code == Code::DisabledRegion));
    assert!(!res.calcpad.contains("a = 1"));
}

#[test]
fn non_math_region_kinds_are_counted() {
    let regs = [
        region(1, 0.0, 0.0, r#"<plot/>"#),
        region(2, 0.0, 20.0, r#"<picture/>"#),
        region(3, 0.0, 40.0, r#"<ole clsid="x" type="embedded"/>"#),
        region(4, 0.0, 60.0, r#"<component/>"#),
        region(5, 0.0, 80.0, r#"<reference/>"#),
        region(6, 0.0, 100.0, r#"<pageBreak/>"#),
        region(7, 0.0, 120.0, r#"<mystery/>"#),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let k = &res.report.kinds;
    for key in ["plot", "image", "ole", "component", "reference", "other"] {
        assert_eq!((k[key].total, k[key].attention), (1, 1), "{key}");
    }
    assert_eq!(res.report.page_breaks, 1);
    assert_eq!(res.report.regions_total(), 6);
    assert_eq!(res.report.regions_needing_attention(), 6);
    assert_eq!(
        res.calcpad
            .lines()
            .filter(|l| l.contains(GAP_MARKER) && l.starts_with('\''))
            .count(),
        6 + 1
    ); // 6 placeholders + the banner
}

// ---- text -------------------------------------------------------------------------------------------------

fn text(id: u32, top: f64, paras: &str) -> String {
    region(id, 0.0, top, &format!("<text>{paras}</text>"))
}

#[test]
fn text_headings_and_inline_formatting() {
    let regs = [
        text(1, 0.0, r#"<p style="Heading 1">Title</p>"#),
        text(2, 20.0, r#"<p style="Heading 2"><b>Sub</b> head</p>"#),
        text(3, 40.0, r#"<p style="Normal">Plain <b>bold</b> and <i>italic</i>.</p>"#),
        text(4, 60.0, r#"<p style="Normal">   </p>"#),
        text(5, 80.0, r#"<p style="Normal">x<sub>2</sub> y<sup>3</sup><br/>next</p>"#),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(
        lines(&res),
        vec![
            "'<h1>Title</h1>",
            "'<h2>Sub head</h2>",
            "'Plain <b>bold</b> and <i>italic</i>.",
            "'x<sub>2</sub> y<sup>3</sup><br/>next",
        ]
    );
    assert_eq!(res.report.kinds["text"].total, 5);
    assert_eq!(res.report.kinds["text"].attention, 0);
}

#[test]
fn text_is_escaped_and_cannot_start_a_directive() {
    let regs = text(
        1,
        0.0,
        r##"<p style="Normal">#include evil.cpd &lt;script&gt; it's "quoted" &amp; done</p>"##,
    );
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    assert_eq!(l.len(), 1);
    assert!(l[0].starts_with('\''));
    assert!(!l[0].contains("<script>"));
    assert!(l[0].contains("it''s \"\"quoted\"\" &amp; done"), "{}", l[0]);
    // The engine shows it as text.
    assert_engine_clean(&res.calcpad);
}

#[test]
fn whitespace_runs_and_empty_formatting_are_tidied() {
    let regs = text(
        1,
        0.0,
        "<p style=\"Normal\">  <b> </b>one   <b> </b> two\n\t three <i> </i></p>",
    );
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(lines(&res), vec!["'one two three"]);
}

#[test]
fn math_nested_in_text_is_flattened_and_ordered_by_position() {
    // The inline definition sits at top 50, below the text region that holds it (top 40) and above
    // the next one (top 100).
    let nested = format!(
        r#"<region region-id="9" left="30" top="50" align-y="60"><math>{}</math></region>"#,
        define("k", &r("7"))
    );
    let regs = [
        region(
            1,
            0.0,
            40.0,
            &format!(r#"<text><p style="Normal">Set k: {nested} done</p></text>"#),
        ),
        math(2, 0.0, 100.0, &eval(&n("k"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(lines(&res), vec!["'Set k: done", "k = 7", "k"]);
    assert_eq!(res.report.regions_total(), 3);
}

#[test]
fn indentation_between_runs_of_one_word_is_not_a_space() {
    let regs = text(
        1,
        0.0,
        "<p style=\"Normal\">\n\t<b>exte</b>\n\t<b>rn</b>\n\t<b>al</b> load\n</p>",
    );
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    assert_eq!(l.len(), 1, "{l:?}");
    assert!(l[0].contains("external"), "{}", l[0]);
    assert!(!l[0].contains("exte rn"), "{}", l[0]);
}

// ---- reading order ----------------------------------------------------------------------------------------------

fn mk(id: &str, left: f64, top: f64, baseline: Option<f64>) -> Region {
    Region {
        id: id.into(),
        left,
        top,
        baseline,
        line: 0,
        width: 0.0,
        height: 0.0,
        rendering: None,
        kind: RegionKind::Math(MathRegion {
            stmt: Stmt::Bare(calcpad_import::ir::Expr::Num("1".into())),
            disabled: false,
            saved_result: None,
            saved_expr: None,
            frozen: false,
            precision: None,
        }),
    }
}

fn order(mut v: Vec<Region>) -> Vec<String> {
    reading_order(&mut v);
    v.into_iter().map(|r| r.id).collect()
}

#[test]
fn reading_order_is_top_to_bottom_then_left_to_right() {
    assert_eq!(
        order(vec![
            mk("c", 0.0, 100.0, None),
            mk("b", 200.0, 0.0, None),
            mk("a", 0.0, 0.0, None),
            mk("d", 100.0, 100.0, None)
        ]),
        ["a", "b", "c", "d"]
    );
}

#[test]
fn side_by_side_regions_within_the_tolerance_are_one_row() {
    // The right region sits 2 points higher: still one row, so left goes first.
    assert_eq!(
        order(vec![mk("right", 300.0, 98.0, None), mk("left", 10.0, 100.0, None)]),
        ["left", "right"]
    );
    // 10 points apart: different rows, the upper one goes first.
    assert_eq!(
        order(vec![mk("left", 10.0, 110.0, None), mk("right", 300.0, 100.0, None)]),
        ["right", "left"]
    );
}

#[test]
fn baseline_keeps_a_tall_region_on_the_row_of_a_short_one() {
    // The tall region on the right starts higher (top 90) but shares the baseline of the short one
    // on the left; the left definition must come first because the right one uses it.
    let o = order(vec![
        mk("tall_right", 180.0, 90.0, Some(110.0)),
        mk("short_left", 40.0, 100.0, Some(110.0)),
    ]);
    assert_eq!(o, ["short_left", "tall_right"]);
}

#[test]
fn definitions_side_by_side_are_used_in_reading_order() {
    // End to end: the use is higher (top) than the definition but on the same row, to its right.
    let regs = [
        region(2, 200.0, 90.0, &format!("<math>{}</math>", eval(&n("w"))))
            .replace(r#"align-y="100""#, r#"align-y="110""#),
        region(1, 10.0, 100.0, &format!("<math>{}</math>", define("w", &r("4")))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    // The two regions share a row, so they are also laid out side by side.
    let text: Vec<String> = lines(&res).into_iter().filter(|l| !l.starts_with('\'')).collect();
    assert_eq!(text, vec!["w = 4", "w"]);
}

// ---- report ------------------------------------------------------------------------------------------------------

#[test]
fn report_counts_and_references() {
    let regs = [
        text(1, 0.0, r#"<p style="Normal">hello</p>"#),
        math(2, 0.0, 20.0, &define("a", &r("1"))),
        math(3, 0.0, 40.0, &define("i", &range(&r("1"), &r("3")))),
        region(4, 0.0, 60.0, "<plot/>"),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let rep = &res.report;
    assert_eq!(rep.regions_total(), 4);
    assert_eq!(rep.regions_converted(), 2);
    assert_eq!(rep.regions_needing_attention(), 2);
    assert!((rep.convertible_ratio() - 0.5).abs() < 1e-9);
    assert_eq!(rep.kinds["math"].total, 2);
    assert_eq!(rep.math_shapes["definition"].total, 2);
    let ids: Vec<_> = rep.gaps().map(|i| (i.code, i.region_id.clone().unwrap())).collect();
    assert_eq!(
        ids,
        vec![(Code::RangeVariable, "3".to_string()), (Code::Plot, "4".to_string())]
    );
    for i in rep.gaps() {
        assert!(i.line.is_some(), "every gap carries a source line");
        assert!(!i.reason.is_empty());
        assert_eq!(i.severity, Severity::Gap);
    }
    let text = rep.to_text();
    assert!(text.contains("2 of 4 regions need attention"), "{text}");
    assert!(text.contains("range-variable"));
    // Stable code spellings.
    assert_eq!(Code::RangeVariable.as_str(), "range-variable");
    assert_eq!(Code::DependsOnGap.as_str(), "depends-on-gap");
}

#[test]
fn ranking_orders_by_blocked_regions() {
    let regs = [
        math(1, 0.0, 0.0, &define("i", &range(&r("1"), &r("3")))),
        math(2, 0.0, 20.0, &eval(&n("i"))),
        math(3, 0.0, 40.0, &eval(&n("i"))),
        math(4, 0.0, 60.0, &eval(&call("stdev", &[r("1")]))),
    ]
    .concat();
    let ranked = import_xmcd(&sheet(&regs)).report.ranked_gaps();
    assert_eq!(ranked[0].code, Code::RangeVariable);
    assert_eq!(ranked[0].blocked(), 3);
    assert_eq!(ranked[1].code, Code::BuiltinMissing);
    assert_eq!(ranked[1].top_details, vec![("stdev".to_string(), 1)]);
}

#[test]
fn a_gap_redefinition_blocks_later_uses_and_a_good_one_clears_it() {
    let regs = [
        math(1, 0.0, 0.0, &define("x", &r("1"))),
        math(2, 0.0, 20.0, &define("x", &range(&r("1"), &r("3")))),
        math(3, 0.0, 40.0, &eval(&n("x"))),
        math(4, 0.0, 60.0, &define("x", &r("2"))),
        math(5, 0.0, 80.0, &eval(&n("x"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    assert!(l[1].contains("range-variable"));
    assert!(l[2].contains("depends-on-gap"));
    assert_eq!(l[3], "x = 2");
    assert_eq!(l[4], "x");
}

// ---- hostile and malformed input ---------------------------------------------------------------------------------------

#[test]
fn malformed_xml_is_a_fatal_report_not_a_panic() {
    for bad in [
        "",
        "not xml",
        "<worksheet><regions>",
        "<a><b></a></b>",
        "<?xml version=\"1.0\"?>",
        "\u{0}\u{1}",
    ] {
        let res = import_xmcd(bad);
        assert!(res.report.is_fatal(), "{bad:?}");
        assert!(res.calcpad.is_empty());
    }
}

#[test]
fn wrong_root_element_is_refused() {
    let res = import_xmcd("<html><body/></html>");
    assert!(res.report.issues.iter().any(|i| i.code == Code::NotAWorksheet));
}

#[test]
fn dtd_and_entity_bombs_are_refused() {
    let bomb = r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
 <!ENTITY lol "lol">
 <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
 <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
 <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
]>
<worksheet><regions><region region-id="1"><text><p>&lol4;</p></text></region></regions></worksheet>"#;
    let res = import_xmcd(bomb);
    assert!(res.report.issues.iter().any(|i| i.code == Code::XmlError));
    assert!(res.calcpad.is_empty());
}

#[test]
fn external_entities_are_never_read() {
    let xxe = r#"<?xml version="1.0"?>
<!DOCTYPE d [ <!ENTITY e SYSTEM "file:///etc/hosts"> ]>
<worksheet><regions><region region-id="1"><text><p>&e;</p></text></region></regions></worksheet>"#;
    let res = import_xmcd(xxe);
    assert!(res.report.is_fatal());
    assert!(!res.calcpad.contains("localhost"));
}

#[test]
fn deeply_nested_xml_does_not_overflow_the_stack() {
    // 200k nested elements, then 200k nested expression applies.
    let depth = 200_000;
    let mut xml = String::from("<worksheet><regions>");
    xml.push_str(&"<x>".repeat(depth));
    xml.push_str(&"</x>".repeat(depth));
    xml.push_str("</regions></worksheet>");
    let _ = import_xmcd(&xml);

    // Past the XML nesting limit the file is refused outright.
    let mut e = String::new();
    for _ in 0..depth {
        e.push_str("<ml:apply><ml:plus/><ml:real>1</ml:real>");
    }
    e.push_str("<ml:real>1</ml:real>");
    for _ in 0..depth {
        e.push_str("</ml:apply>");
    }
    let res = import_xmcd(&sheet(&math(1, 0.0, 0.0, &eval(&e))));
    assert!(res.report.is_fatal());

    // Within the XML limit but past the expression limit the region becomes a `too-deep` gap.
    let depth = 200;
    let mut e = String::new();
    for _ in 0..depth {
        e.push_str("<ml:apply><ml:plus/><ml:real>1</ml:real>");
    }
    e.push_str("<ml:real>1</ml:real>");
    for _ in 0..depth {
        e.push_str("</ml:apply>");
    }
    let res = import_xmcd(&sheet(&math(1, 0.0, 0.0, &eval(&e))));
    assert_eq!(res.report.regions_needing_attention(), 1);
    assert!(res.report.gaps().any(|i| i.code == Code::TooDeep));
}

#[test]
fn a_very_wide_sum_is_capped_not_recursed() {
    let terms: Vec<String> = (0..5000).map(|_| r("1")).collect();
    let res = one(&eval(&app("plus", &terms)));
    assert!(res.report.gaps().any(|i| i.code == Code::TooDeep));
}

#[test]
fn deeply_nested_text_regions_do_not_overflow_the_stack() {
    let depth = 50_000;
    let mut inner = String::from(r#"<text><p style="Normal">x</p></text>"#);
    for i in 0..depth {
        inner = format!(r#"<text><p style="Normal"><region region-id="{i}">{inner}</region></p></text>"#);
        if inner.len() > 4_000_000 {
            break;
        }
    }
    let res = import_xmcd(&sheet(&region(1, 0.0, 0.0, &inner)));
    assert!(!res.report.is_fatal() || res.calcpad.is_empty());
}

#[test]
fn size_and_node_limits_apply() {
    let xml = sheet(&math(1, 0.0, 0.0, &define("a", &r("1"))));
    let res = import_xmcd_with(
        &xml,
        Limits {
            max_bytes: 10,
            ..Limits::default()
        },
    );
    assert!(res.report.issues.iter().any(|i| i.code == Code::InputTooLarge));
    let res = import_xmcd_with(
        &xml,
        Limits {
            max_nodes: 5,
            ..Limits::default()
        },
    );
    assert!(res.report.issues.iter().any(|i| i.code == Code::XmlError));
}

#[test]
fn huge_flat_input_with_many_regions_is_fine() {
    let mut regs = String::new();
    for i in 0..5000u32 {
        regs.push_str(&math(
            i + 1,
            0.0,
            f64::from(i) * 20.0,
            &define(&format!("v{i}"), &r("1")),
        ));
    }
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(res.report.regions_total(), 5000);
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn odd_attribute_values_do_not_panic() {
    let xml = r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30"><regions>
      <region region-id="a b&#10;#include x" left="NaN" top="inf"><math><ml:matrix rows="99999999999999999999" cols="-1"/></math></region>
      <region left="1e999" top="-1e999"><math><ml:define/></math></region>
      <region><math/></region>
      <region><text/></region>
      <region/>
    </regions></worksheet>"#;
    let res = import_xmcd(xml);
    for l in res.calcpad.lines() {
        assert!(l.starts_with('\'') || l.starts_with('#'), "{l}");
    }
}

// ---- the invariant: no unconverted content without the marker --------------------------------------------------------------------

#[test]
fn unconverted_regions_never_leak_text_without_the_marker() {
    let payload = "LEAK_PAYLOAD_7731";
    let regs = [
        // Strings, names, ids and tags carrying the payload, in regions that cannot convert.
        math(1, 0.0, 0.0, &define("s", &format!("<ml:str>{payload}</ml:str>"))),
        math(2, 0.0, 20.0, &eval(&call(&format!("fn_{payload}"), &[r("1")]))),
        math(3, 0.0, 40.0, &format!("<ml:{payload}/>")),
        region(4, 0.0, 60.0, &format!("<{payload}/>")),
        region(5, 0.0, 80.0, &format!("<ole clsid=\"{payload}\" type=\"{payload}\"/>")),
        math(6, 0.0, 100.0, &define(&format!("v {payload}"), &r("1"))),
        math(7, 0.0, 120.0, &eval(&n(&format!("undef {payload}")))),
        r#"<region region-id="8&#10;#include evil.cpd" left="0" top="140"><plot/></region>"#.to_string(),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(res.report.regions_needing_attention(), 8);
    for l in lines(&res) {
        if l.contains(payload) {
            // Allowed only inside a marked placeholder, as a bounded, escaped attempt.
            assert!(
                l.starts_with(&format!("'{GAP_MARKER}")),
                "payload outside a placeholder: {l}"
            );
        }
        assert!(!l.starts_with('#') || l == "#rad", "directive from input: {l}");
        assert!(!l.contains("#include"), "{l}");
    }
    // Exactly one placeholder per unconverted region, each carrying its (sanitised) region id.
    let all = lines(&res);
    let ph: Vec<&String> = all
        .iter()
        .filter(|l| l.starts_with(&format!("'{GAP_MARKER}")))
        .collect();
    assert_eq!(ph.len(), 8);
}

#[test]
fn every_gap_in_the_report_has_a_placeholder_line() {
    let regs = [
        math(1, 0.0, 0.0, &define("i", &range(&r("1"), &r("3")))),
        math(2, 0.0, 20.0, &eval(&call("stdev", &[r("1")]))),
        math(3, 0.0, 40.0, &define("ok", &r("1"))),
        region(4, 0.0, 60.0, "<picture/>"),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let mut gap_regions: Vec<String> = res.report.gaps().map(|i| i.region_id.clone().unwrap()).collect();
    gap_regions.sort();
    gap_regions.dedup();
    for id in &gap_regions {
        let tag = format!("'{GAP_MARKER} R{id} ");
        assert!(
            lines(&res).iter().any(|l| l.starts_with(&tag)),
            "no placeholder for region {id}"
        );
    }
    assert_eq!(
        lines(&res)
            .iter()
            .filter(|l| l.starts_with(&format!("'{GAP_MARKER}")))
            .count(),
        gap_regions.len()
    );
}

#[test]
fn output_is_deterministic() {
    let regs = [
        math(1, 0.0, 0.0, &define("a", &r("1"))),
        math(2, 50.0, 0.0, &define("b", &r("2"))),
        math(3, 0.0, 20.0, &eval(&app("plus", &[n("a"), n("b")]))),
    ]
    .concat();
    let xml = sheet(&regs);
    assert_eq!(import_xmcd(&xml), import_xmcd(&xml));
}

// ---- layout of regions that share a row ---------------------------------------------------------------------------

#[test]
fn math_regions_on_one_row_are_laid_out_side_by_side_in_reading_order() {
    // Two definitions on a row (left to right), then one alone below.
    let regs = [
        math(2, 100.0, 0.0, &eval(&n("a"))),
        math(1, 0.0, 0.0, &define("a", &r("4"))),
        math(3, 0.0, 40.0, &eval(&app("plus", &[n("a"), r("1")]))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    let l = lines(&res);
    let open = l.iter().position(|x| x.contains("imported-row")).expect("a row");
    assert_eq!(l.iter().filter(|x| x.contains("imported-row")).count(), 1, "{l:?}");
    // The definition is evaluated before the use, inside the row.
    assert_eq!(l[open + 1], "'<div>");
    assert_eq!(l[open + 2], "a = 4");
    assert_eq!(l[open + 5], "a");
    // The region below is not wrapped.
    assert_eq!(l.last().map(String::as_str), Some("a + 1"));
    assert_engine_clean(&res.calcpad);
}

#[test]
fn rows_of_two_text_regions_become_one_table() {
    let regs = [
        region(1, 0.0, 0.0, "<text><p style=\"Normal\">alpha</p></text>"),
        region(2, 100.0, 0.0, "<text><p style=\"Normal\">first letter</p></text>"),
        region(3, 0.0, 20.0, "<text><p style=\"Normal\">beta</p></text>"),
        region(4, 100.0, 20.0, "<text><p style=\"Normal\">second letter</p></text>"),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert_eq!(res.calcpad.matches("<table").count(), 1, "{}", res.calcpad);
    assert_eq!(res.calcpad.matches("<tr>").count(), 2, "{}", res.calcpad);
    assert!(
        res.calcpad.contains("'alpha\n'</td>\n'<td>\n'first letter"),
        "{}",
        res.calcpad
    );
    assert_engine_clean(&res.calcpad);
}

#[test]
fn regions_that_overlap_across_are_stacked() {
    // The second region starts inside the first (width 10): not beside it.
    let regs = [
        math(1, 0.0, 0.0, &define("a", &r("4"))),
        math(2, 5.0, 0.0, &eval(&n("a"))),
    ]
    .concat();
    let res = import_xmcd(&sheet(&regs));
    assert!(!res.calcpad.contains("imported-row"), "{}", res.calcpad);
}
