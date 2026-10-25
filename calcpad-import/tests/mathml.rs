//! The author's equations shown as MathML, with the computation unchanged. Every worksheet here
//! is invented; none of it comes from a real file.

mod common;
use calcpad_import::{import_xmcd_opts, ImportOptions, ImportResult};
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

fn convert(inners: &[String]) -> ImportResult {
    import_xmcd_opts(&sheet_of(inners), &ImportOptions::default())
}

fn convert_noc(inners: &[String]) -> ImportResult {
    let opts = ImportOptions {
        mathml: false,
        ..Default::default()
    };
    import_xmcd_opts(&sheet_of(inners), &opts)
}

fn n(name: &str) -> String {
    format!("<ml:id>{name}</ml:id>")
}
fn sub(name: &str, s: &str) -> String {
    format!("<ml:id subscript=\"{s}\">{name}</ml:id>")
}
fn r(v: &str) -> String {
    format!("<ml:real>{v}</ml:real>")
}
fn app(op: &str, args: &[String]) -> String {
    format!("<ml:apply><ml:{op}/>{}</ml:apply>", args.join(""))
}
fn define_id(id: &str, value: &str) -> String {
    format!("<ml:define>{id}{value}</ml:define>")
}
fn define(name: &str, value: &str) -> String {
    define_id(&n(name), value)
}
fn eval(expr: &str) -> String {
    format!("<ml:eval>{expr}</ml:eval>")
}
fn eval_in(expr: &str, unit: &str) -> String {
    format!(
        "<ml:eval>{expr}<ml:unitOverride>{}</ml:unitOverride></ml:eval>",
        n(unit)
    )
}
fn mat(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}
fn indexed(base: &str, idx: &str) -> String {
    format!("<ml:apply><ml:indexer/>{base}{idx}</ml:apply>")
}
fn range(first: &str, last: &str) -> String {
    format!("<ml:range>{}{}</ml:range>", r(first), r(last))
}
fn series(op: &str, var: &str, body: &str, bounds: Option<(&str, &str)>) -> String {
    let b = bounds
        .map(|(lo, hi)| format!("<ml:bounds>{lo}{hi}</ml:bounds>"))
        .unwrap_or_default();
    format!(
        "<ml:apply><ml:{op}/><ml:lambda><ml:boundVars>{}</ml:boundVars>{body}</ml:lambda>{b}</ml:apply>",
        n(var)
    )
}
fn integral(var: &str, body: &str, lo: &str, hi: &str) -> String {
    format!(
        "<ml:apply><ml:integral/><ml:lambda><ml:boundVars>{}</ml:boundVars>{body}</ml:lambda><ml:lowerBound>{lo}</ml:lowerBound><ml:upperBound>{hi}</ml:upperBound></ml:apply>",
        n(var)
    )
}

/// The page the engine prints for `code`, and the errors it recorded (hidden ones included).
fn run(code: &str) -> (String, Vec<String>) {
    let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    p.debug = true;
    p.parse(code, true);
    let errors = p.errors_list.iter().map(|e| e.message.clone()).collect();
    (p.html_result.clone(), errors)
}

fn count(h: &str, needle: &str) -> usize {
    h.matches(needle).count()
}

#[test]
fn a_quotient_is_a_stacked_fraction_and_a_sum_has_its_bounds() {
    // C_(v,i) := w_i * h_i / sum(w_i * h_i, i = 1..n), with w and h given first.
    let wi = indexed(&n("w"), &n("i"));
    let hi = indexed(&n("h"), &n("i"));
    let body = app("mult", &[wi.clone(), hi.clone()]);
    let res = convert(&[
        define("i", &range("1", "3")),
        define("n", &r("3")),
        define("w", &mat(3, 1, &["1", "2", "3"])),
        define("h", &mat(3, 1, &["4", "5", "6"])),
        define_id(
            &indexed(&sub("C", "v"), &n("i")),
            &app(
                "div",
                &[body.clone(), series("summation", "i", &body, Some((&r("1"), &n("n"))))],
            ),
        ),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let line = res.calcpad.lines().find(|l| l.contains("<mfrac>")).expect("a fraction");
    assert!(line.contains("<munderover"), "{line}");
    assert!(line.contains("\u{2211}"), "{line}");
    // The name is C with the subscript v,i, and the definition is `:=`.
    assert!(
        line.contains("<msub><mi>C</mi><mrow><mi>v</mi><mo>,</mo><mi>i</mi></mrow></msub><mo>:=</mo>"),
        "{line}"
    );
    // Large parentheses around the summed term come from stretchy fences.
    assert!(line.contains("<mo>(</mo>") && line.contains("<mo>)</mo>"), "{line}");
    let (html, errors) = run(&res.calcpad);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(!html.contains("class=\"err\""), "{html}");
}

#[test]
fn a_subscript_is_a_subscript_and_an_index_is_too() {
    let res = convert(&[
        define("V", &mat(2, 1, &["5", "6"])),
        define("x", &indexed(&n("V"), &r("1"))),
    ]);
    let line = res.calcpad.lines().rfind(|l| l.contains("<math")).unwrap();
    assert!(
        line.contains("<msub><mi>V</mi><mrow><mn>1</mn></mrow></msub>"),
        "{line}"
    );
}

#[test]
fn a_vector_arrow_covers_the_whole_product_and_a_transpose_is_a_superscript() {
    let vec_op = |x: &str| format!("<ml:apply><ml:vectorize/>{x}</ml:apply>");
    let tr = |x: &str| format!("<ml:apply><ml:transpose/>{x}</ml:apply>");
    let res = convert(&[
        define("c", &mat(3, 1, &["1", "2", "3"])),
        define("V", &r("10")),
        define("F", &vec_op(&app("mult", &[app("mult", &[n("c"), n("V")]), n("kip")]))),
        eval_in(&tr(&n("F")), "kip"),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let def = res.calcpad.lines().find(|l| l.contains("<mover")).expect("an arrow");
    assert!(
        def.contains("<mi>c</mi><mo class=\"dot\">\u{22C5}</mo><mi>V</mi>"),
        "{def}"
    );
    assert!(def.contains("\u{2192}"), "{def}");
    // The evaluation: F^T on the left, the engine's numbers in a bracketed row, the unit once after it.
    let (html, errors) = run(&res.calcpad);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        html.contains("<msup><mi>F</mi><mi mathvariant=\"normal\">T</mi></msup><mo>=</mo>"),
        "{html}"
    );
    assert!(
        html.contains("<mtd><mn>10</mn></mtd><mtd><mn>20</mn></mtd><mtd><mn>30</mn></mtd>"),
        "{html}"
    );
    let last = html
        .lines()
        .rfind(|l| l.contains("<mover") || l.contains("<mtable>"))
        .unwrap();
    assert_eq!(
        count(last, "<mi mathvariant=\"normal\" class=\"ru\">kip</mi>"),
        1,
        "{last}"
    );
    assert!(
        html.contains(
            "</mtable><mo>]</mo><mspace width=\"0.25em\"></mspace><mi mathvariant=\"normal\" class=\"ru\">kip</mi>"
        ),
        "{html}"
    );
}

#[test]
fn a_definition_uses_the_definition_sign_and_an_evaluation_the_equals_sign() {
    let res = convert(&[define("a", &r("2")), eval(&app("plus", &[n("a"), r("1")]))]);
    let defs: Vec<&str> = res.calcpad.lines().filter(|l| l.contains("<mo>:=</mo>")).collect();
    assert_eq!(defs.len(), 1, "{}", res.calcpad);
    let (html, errors) = run(&res.calcpad);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(html.contains("<mi>a</mi><mo>+</mo><mn>1</mn><mo>=</mo>"), "{html}");
    assert!(html.contains("<mn>3</mn>"), "{html}");
    // Nothing in the page is the engine's own `a = 2` echo.
    assert!(!html.contains("<var>a</var>"), "{html}");
}

#[test]
fn the_results_use_the_precision_of_the_file_and_not_the_engines_twelve_places() {
    let res = convert(&[define("x", &app("div", &[r("2"), r("3")])), eval(&n("x"))]);
    let (html, _) = run(&res.calcpad);
    assert!(html.contains("<mn>0.667</mn>"), "{html}");
}

#[test]
fn a_matrix_of_names_is_a_column_of_names() {
    let names = "<ml:matrix rows=\"2\" cols=\"1\"><ml:id>pp</ml:id><ml:id>qq</ml:id></ml:matrix>";
    let res = convert(&[format!("<ml:define>{names}{}</ml:define>", mat(2, 1, &["1", "2"]))]);
    let line = res.calcpad.lines().find(|l| l.contains("<math")).expect("typeset");
    assert!(
        line.contains("<mtable><mtr><mtd><mi class=\"v\">pp</mi></mtd></mtr><mtr><mtd><mi class=\"v\">qq</mi></mtd></mtr></mtable>"),
        "{line}"
    );
}

#[test]
fn the_computation_is_the_same_with_and_without_the_typeset_display() {
    let regions = [
        define("a", &app("mult", &[r("2"), n("m")])),
        define("b", &app("plus", &[n("a"), n("a")])),
        eval_in(&n("b"), "mm"),
        eval(&app("div", &[n("b"), n("a")])),
    ];
    let with = convert(&regions);
    let without = convert_noc(&regions);
    assert_eq!(legacy(&with.calcpad), legacy(&without.calcpad));
    assert_eq!(with.report.regions_total(), without.report.regions_total());
    assert_eq!(
        with.report.regions_needing_attention(),
        without.report.regions_needing_attention()
    );
}

#[test]
fn an_error_in_the_hidden_computation_is_still_reported() {
    let res = convert(&[define(
        "x",
        &format!(
            "<ml:apply>{}<ml:sequence>{}</ml:sequence></ml:apply>",
            n("length"),
            r("5")
        ),
    )]);
    let (_, errors) = run(&res.calcpad);
    assert!(errors.iter().any(|e| e.contains("must be vector")), "{errors:?}");
}

#[test]
fn a_construct_without_a_typeset_form_falls_back_to_the_calcpad_display() {
    // A program body has no MathML spelling here: the definition keeps the `#noc` display (or none).
    let prog = format!(
        "<ml:define>{}<ml:program><ml:localDefine>{}{}</ml:localDefine>{}</ml:program></ml:define>",
        n("y"),
        n("s"),
        r("4"),
        n("s")
    );
    let res = convert(&[prog, eval(&n("y"))]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(
        !res.calcpad.lines().any(|l| l.contains("<mo>:=</mo>")),
        "{}",
        res.calcpad
    );
    // The option off gives the fallback everywhere, and no MathML line at all.
    let off = convert_noc(&[define("a", &r("2"))]);
    assert!(off.calcpad.contains("#noc\na = 2\n#end noc"), "{}", off.calcpad);
    assert!(!off.calcpad.contains("<math"), "{}", off.calcpad);
    assert!(!off.calcpad.contains("valMathml"), "{}", off.calcpad);
}

#[test]
fn a_constraint_equation_is_typeset_with_its_integrals_and_never_shown_as_code() {
    // (integral of 1 dV from V_AB(L) to V_BC(x)) = (integral of -w dx from L to x), chained, unsolved.
    let call = |name: &str, args: &[String]| {
        format!(
            "<ml:apply>{}<ml:sequence>{}</ml:sequence></ml:apply>",
            sub("V", name),
            args.join("")
        )
    };
    let lhs = integral("V", &r("1"), &call("AB", &[sub("L", "AB")]), &call("BC", &[n("x")]));
    let mid = integral("x", &app("neg", &[n("w")]), &sub("L", "AB"), &n("x"));
    let last = integral("x", &app("neg", &[n("c")]), &sub("L", "AB"), &n("x"));
    let eq = app("equal", &[app("equal", &[lhs, mid]), last]);
    let res = convert(&[eq, eval(&app("mult", &[r("2"), r("3")]))]);
    // It stays a not-calculated region with a quiet note.
    assert_eq!(res.report.regions_needing_attention(), 1, "{:?}", res.report.issues);
    assert!(
        res.calcpad.contains("title=\"constraint-equation: ") && res.calcpad.contains(">not calculated</small>"),
        "{}",
        res.calcpad
    );
    // The equation is MathML: integrals with bounds, the original names, equalities in a chain.
    let line = res
        .calcpad
        .lines()
        .find(|l| l.contains("<msubsup"))
        .expect("an integral");
    assert_eq!(count(line, "\u{222B}"), 3, "{line}");
    assert!(
        line.contains("<msub><mi>L</mi><mrow><mi class=\"v\">AB</mi></mrow></msub>"),
        "{line}"
    );
    assert_eq!(count(line, "<mo class=\"beq\">=</mo>"), 2, "{line}");
    assert!(line.contains("<mi>d</mi>"), "{line}");
    // No Calcpad source anywhere for the reader.
    for bad in ["<code>", "$Integral", "$Slope", "?name?", "?gap?"] {
        assert!(!res.calcpad.contains(bad), "{bad} in {}", res.calcpad);
    }
    // The page still runs without engine errors, and the later region computed.
    let (html, errors) = run(&res.calcpad);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(html.contains("<mn>6</mn>"), "{html}");
}

#[test]
fn names_and_strings_from_the_file_cannot_add_markup_or_directives() {
    let res = convert(&[
        define("a<b>&c", &r("2")),
        eval("<ml:str>x'y$z{q}</ml:str>"),
        eval(&n("a<b>&c")),
    ]);
    for l in res.calcpad.lines().filter(|l| l.contains("<math")) {
        assert!(!l.contains("<b>"), "{l}");
        assert!(!l.contains('\''), "{}", &l[1..]);
    }
}

fn app_call(name: &str, arg: &str) -> String {
    format!("<ml:apply>{name}{arg}</ml:apply>")
}

fn func_def(name: &str, param: &str, body: &str) -> String {
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{}</ml:boundVars></ml:function>{body}</ml:define>",
        name,
        n(param)
    )
}

#[test]
fn the_pages_own_function_names_are_italic_everywhere_and_the_tools_functions_upright() {
    // F_AB(x) := 2 x; F_BC(x) := F_AB(x) - sin(x). The name F_AB is set the same way where it is
    // defined and where it is called; sin stays upright.
    let res = convert(&[
        func_def(&sub("F", "AB"), "x", &app("mult", &[r("2"), n("x")])),
        func_def(
            &sub("F", "BC"),
            "x",
            &app(
                "minus",
                &[app_call(&sub("F", "AB"), &n("x")), app_call(&n("sin"), &n("x"))],
            ),
        ),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let line = res.calcpad.lines().rfind(|l| l.contains("<math")).unwrap();
    assert!(
        line.contains("<msub><mi>F</mi><mrow><mi class=\"v\">AB</mi></mrow></msub><mrow><mo>(</mo>"),
        "{line}"
    );
    assert!(!line.contains("<mi mathvariant=\"normal\">F</mi>"), "{line}");
    assert!(line.contains("<mi mathvariant=\"normal\">sin</mi>"), "{line}");
}

#[test]
fn an_unlabelled_one_letter_name_is_a_variable_not_a_unit() {
    // `d` and `V` carry no unit label in the file: they are variables, never upright unit labels.
    let res = convert(&[eval(&app("plus", &[n("d"), n("V")]))]);
    let line = res
        .calcpad
        .lines()
        .rfind(|l| l.contains("<math"))
        .expect("a typeset line");
    assert!(!line.contains("mathvariant=\"normal\">d<"), "{line}");
    assert!(!line.contains("mathvariant=\"normal\">V<"), "{line}");
}

#[test]
fn an_imported_sheet_asks_the_editor_not_to_report_unused_names() {
    let res = convert(&[define("a", &r("2")), eval(&app("plus", &[n("a"), r("1")]))]);
    let head: Vec<&str> = res.calcpad.lines().take(4).collect();
    assert!(
        head.iter().any(|l| l.contains("\"LintIgnore\": [\"CPD-3312\"]")),
        "{head:?}"
    );
}

/// A worksheet whose regions have the given geometry: `(left, top, align-y, width, height, kind, inner)`.
fn placed(regions: &[(f64, f64, f64, f64, f64, &str)]) -> ImportResult {
    let body: String = regions
        .iter()
        .enumerate()
        .map(|(i, (left, top, ay, w, h, inner))| {
            format!(
                r#"<region region-id="{}" left="{left}" top="{top}" width="{w}" height="{h}" align-y="{ay}">{inner}</region>"#,
                i + 1
            )
        })
        .collect();
    let sheet = format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="1"/></calculation></settings><regions>{body}</regions></worksheet>"#
    );
    import_xmcd_opts(&sheet, &ImportOptions::default())
}

fn text_inner(s: &str) -> String {
    format!(r#"<text><p style="Normal">{s}</p></text>"#)
}

#[test]
fn a_number_before_its_unit_has_no_dot_and_every_unit_is_one_class() {
    let res = convert(&[define("L", &app("mult", &[r("50"), n("ft")]))]);
    let line = res.calcpad.lines().rfind(|l| l.contains("<math")).unwrap();
    assert!(!line.contains("class=\"dot\""), "{line}");
    assert!(
        line.contains("<mn>50</mn><mspace width=\"0.2em\"></mspace><mi mathvariant=\"normal\" class=\"ru\">ft</mi>"),
        "{line}"
    );
    // A product of two names keeps its dot.
    let res = convert(&[define("a", &r("2")), define("b", &app("mult", &[n("a"), n("a")]))]);
    assert!(res.calcpad.contains("class=\"dot\""), "{}", res.calcpad);
}

#[test]
fn the_vector_arrow_is_drawn_by_the_page_across_the_whole_operand() {
    let vec_op = |x: &str| format!("<ml:apply><ml:vectorize/>{x}</ml:apply>");
    let res = convert(&[
        define("c", &mat(3, 1, &["1", "2", "3"])),
        define("F", &vec_op(&app("mult", &[n("c"), n("c")]))),
    ]);
    let line = res.calcpad.lines().find(|l| l.contains("<mover")).expect("an arrow");
    assert!(line.contains("<mover class=\"va\""), "{line}");
    // The arrow is a line and a head painted by the stylesheet; the character stays for the export.
    assert!(res.calcpad.contains("mover.va{background:linear-gradient"), "no style");
    assert!(line.contains("\u{2192}"), "{line}");
    assert!(!res.calcpad.contains("mover.va{font-weight"), "");
}

#[test]
fn a_value_beside_a_label_that_wraps_shares_its_row_and_keeps_its_offset() {
    // The label is two lines tall (top 100, height 40, baseline of its first line at 110); the value
    // sits to its right on the second line (baseline 130).
    let res = placed(&[
        (
            6.0,
            100.0,
            110.0,
            120.0,
            40.0,
            &text_inner("A label that wraps onto two lines"),
        ),
        (
            200.0,
            120.0,
            130.0,
            50.0,
            18.0,
            &format!("<math>{}</math>", define("a", &r("2"))),
        ),
    ]);
    assert!(res.calcpad.contains("class=\"imported-row\""), "{}", res.calcpad);
    assert!(res.calcpad.contains("align-items:last baseline"), "{}", res.calcpad);
    // 200 - (6 + 120) = 74 units from the end of the label, less the row's own gap.
    assert!(res.calcpad.contains("margin-left:50pt"), "{}", res.calcpad);
}

#[test]
fn an_equation_keeps_its_indent_from_the_page_margin() {
    let res = placed(&[
        (6.0, 10.0, 20.0, 100.0, 14.0, &text_inner("Heading line")),
        (
            30.0,
            40.0,
            50.0,
            60.0,
            18.0,
            &format!("<math>{}</math>", define("a", &r("2"))),
        ),
    ]);
    assert!(
        res.calcpad.contains("<div class=\"imp-eq\" style=\"--ind:24pt\">"),
        "{}",
        res.calcpad
    );
}

#[test]
fn an_expression_of_a_range_variable_is_typeset_with_its_column_of_values() {
    // i := 1..3; w := [4 5 6]^T; w_i evaluated: the author's left side and a column of values.
    let res = convert(&[
        define("i", &range("1", "3")),
        define("w", &mat(3, 1, &["4", "5", "6"])),
        eval(&indexed(&n("w"), &n("i"))),
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let (html, errors) = run(&res.calcpad);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        html.contains("<msub><mi>w</mi><mrow><mi>i</mi></mrow></msub><mo>=</mo>"),
        "{html}"
    );
    assert!(
        html.contains("<mtd><mn>4</mn></mtd></mtr><mtr><mtd><mn>5</mn></mtd></mtr><mtr><mtd><mn>6</mn></mtd>"),
        "{html}"
    );
    assert!(!html.contains("tbl_"), "{html}");
}

#[test]
fn a_function_the_tool_provides_by_another_name_is_upright() {
    let res = convert(&[
        define("h", &mat(2, 1, &["1", "2"])),
        define("x", &app_call(&n("SIUnitsOf"), &n("h"))),
    ]);
    let line = res.calcpad.lines().rfind(|l| l.contains("<math")).expect("a line");
    assert!(line.contains("<mi mathvariant=\"normal\">SIUnitsOf</mi>"), "{line}");
}
