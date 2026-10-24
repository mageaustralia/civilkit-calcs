//! Plot regions: hand-written synthetic worksheets, converted and then drawn by the engine.
//! None of the XML here comes from a real file.

use calcpad_import::{import_xmcd_opts, Code, ImportOptions, ImportResult};

fn sheet(origin: i64, regions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet version="3.0.3" xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10">
<settings><calculation><builtInVariables array-origin="{origin}"/></calculation></settings>
<regions>{regions}</regions>
</worksheet>"#
    )
}

fn region(id: u32, top: f64, content: &str) -> String {
    format!(
        r#"<region region-id="{id}" left="0" top="{top}" width="10" height="10" align-y="{}">{content}</region>"#,
        top + 10.0
    )
}

fn math(id: u32, top: f64, inner: &str) -> String {
    region(id, top, &format!("<math>{inner}</math>"))
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
fn func(name: &str, params: &[&str], body: &str) -> String {
    let p: String = params.iter().map(|x| n(x)).collect();
    format!(
        "<ml:define><ml:function>{}<ml:boundVars>{p}</ml:boundVars></ml:function>{body}</ml:define>",
        n(name)
    )
}
fn range(first: &str, last: &str) -> String {
    format!("<ml:range>{first}{last}</ml:range>")
}
fn matrix(rows: usize, cols: usize, items: &[&str]) -> String {
    let body: String = items.iter().map(|v| r(v)).collect();
    format!(r#"<ml:matrix rows="{rows}" cols="{cols}">{body}</ml:matrix>"#)
}
fn index(base: &str, idx: &str) -> String {
    format!("<ml:apply><ml:indexer/>{}{}</ml:apply>", n(base), idx)
}

/// One expression slot of an axis: the expression and, when given, the unit beside it.
fn slot(expr: &str, unit: Option<&str>) -> String {
    let unit = unit.map(|u| u.to_string()).unwrap_or("<ml:placeholder/>".into());
    format!("<plotEquation><math>{expr}</math><math>{unit}</math></plotEquation>")
}

struct Axis {
    slots: Vec<String>,
    attrs: String,
    domain: String,
    label: String,
}

fn axis(slots: &[String]) -> Axis {
    Axis {
        slots: slots.to_vec(),
        attrs: String::new(),
        domain: String::new(),
        label: String::new(),
    }
}

impl Axis {
    fn xml(&self, tag: &str) -> String {
        format!(
            r#"<{tag} rank="1" {}><axisLabel>{}</axisLabel><plotEquations>{}</plotEquations>{}</{tag}>"#,
            self.attrs,
            self.label,
            self.slots.concat(),
            self.domain
        )
    }
}

fn trace(style: &str) -> String {
    format!(
        r##"<trace><traceStyle color="#FFFF0000" symbol="none" line-weight="1" line-style="Solid">{style}</traceStyle></trace>"##
    )
}

fn plot(id: u32, top: f64, title: &str, traces: &[String], x: &Axis, y: &Axis) -> String {
    region(
        id,
        top,
        &format!(
            r#"<plot><xyPlot><title>{title}</title><legend/><traces>{}</traces><axes>{}{}</axes></xyPlot></plot>"#,
            traces.concat(),
            x.xml("xAxis"),
            y.xml("yAxis")
        ),
    )
}

fn convert(origin: i64, regions: &[String]) -> ImportResult {
    let opts = ImportOptions {
        mathml: false,
        ..Default::default()
    };
    import_xmcd_opts(&sheet(origin, &regions.concat()), &opts)
}

fn plot_line(code: &str) -> String {
    code.lines()
        .find(|l| l.starts_with("$Plot"))
        .unwrap_or_else(|| panic!("no plot line in:\n{code}"))
        .to_string()
}

/// Runs the page and returns the data of every plot drawn; fails on an engine error.
fn draw(code: &str) -> Vec<calcpad_core::plotter::capture::CapturedChart> {
    let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    p.capture_values = true;
    p.parse(&format!("#settings {{\"vectorGraphics\": true}}\n{code}"), true);
    assert!(
        !p.html_result.contains("class=\"err\""),
        "engine error in:\n{code}\n---\n{}",
        p.html_result
    );
    assert!(p.html_result.contains("<svg"), "no drawing in:\n{}", p.html_result);
    p.captured_plots.clone()
}

fn gap_detail(res: &ImportResult, code: Code) -> Option<String> {
    res.report
        .gaps()
        .find(|i| i.code == code)
        .and_then(|i| i.detail.clone())
}

#[test]
fn a_function_plot_runs_over_the_domain_the_author_typed() {
    let x = Axis {
        domain: "<xyDomain><startValue><ml:real>0</ml:real></startValue><endValue><ml:real>4</ml:real></endValue></xyDomain>".into(),
        ..axis(&[slot(&n("x"), None)])
    };
    let y = axis(&[slot(&call("f", &[n("x")]), None)]);
    let regs = [
        math(1, 0.0, &func("f", &["x"], &app("mult", &[n("x"), n("x")]))),
        plot(2, 20.0, "", &[trace("lines")], &x, &y),
    ];
    let res = convert(1, &regs);
    assert_eq!(plot_line(&res.calcpad), "$Plot{x|f(x) @ x = 0 : 4}");
    assert_eq!(res.report.kinds["plot"].converted, 1);
    assert_eq!(res.report.regions_needing_attention(), 0);
    let charts = draw(&res.calcpad);
    let pts = &charts[0].series[0].points;
    assert_eq!(pts.first().copied(), Some((0.0, 0.0)));
    let last = pts.last().unwrap();
    assert!((last.0 - 4.0).abs() < 1e-9 && (last.1 - 16.0).abs() < 1e-9, "{last:?}");
}

#[test]
fn the_axis_range_is_the_domain_when_none_is_typed() {
    let x = Axis {
        attrs: r#"start="1" end="3""#.into(),
        ..axis(&[slot(&n("q"), None)])
    };
    let y = axis(&[slot(&app("mult", &[r("2"), n("q")]), None)]);
    let res = convert(1, &[plot(1, 0.0, "", &[trace("lines")], &x, &y)]);
    assert_eq!(plot_line(&res.calcpad), "$Plot{q|2*q @ q = 1 : 3}");
}

#[test]
fn a_range_variable_plot_reads_the_array_between_elements() {
    // v is 10 20 30 40; i runs 0 .. 3 at origin 0, so v[i] is v.(i + 1) in the engine.
    let regs = [
        math(1, 0.0, &define("v", &matrix(4, 1, &["10", "20", "30", "40"]))),
        math(2, 20.0, &define("i", &range(&r("0"), &r("3")))),
        plot(
            3,
            40.0,
            "",
            &[trace("lines")],
            &axis(&[slot(&n("i"), None)]),
            &axis(&[slot(&index("v", &n("i")), None)]),
        ),
    ];
    let res = convert(0, &regs);
    assert_eq!(plot_line(&res.calcpad), "$Plot{i|line(i + 1; v) @ i = 0 : 3}");
    let pts = &draw(&res.calcpad)[0].series[0].points;
    // 10 at i = 0, 40 at i = 3, and a straight line between (i = 1.5 is 25).
    assert_eq!(pts.first().copied(), Some((0.0, 10.0)));
    let (xl, yl) = *pts.last().unwrap();
    assert!((xl - 3.0).abs() < 1e-9 && (yl - 40.0).abs() < 1e-9);
    // Every drawn point is on y = 10 + 10 i, also between the elements.
    assert!(pts.len() > 10);
    assert!(pts.iter().any(|p| p.0.fract().abs() > 0.01 && p.0.fract().abs() < 0.99));
    assert!(pts.iter().all(|p| (p.1 - (10.0 + 10.0 * p.0)).abs() < 1e-6), "{pts:?}");
}

#[test]
fn a_trace_without_its_own_y_reuses_the_last_one_and_the_axis_has_one_unit() {
    // The second and third x are single values: vertical lines across the data. The unit typed
    // beside the first x applies to the whole axis.
    let regs = [
        math(1, 0.0, &define("L", &app("mult", &[r("6"), n("m")]))),
        math(2, 20.0, &define("v", &matrix(2, 1, &["1", "5"]))),
        math(3, 40.0, &define("i", &range(&r("1"), &r("2")))),
        plot(
            4,
            60.0,
            "",
            &[trace("lines"), trace("lines"), trace("lines")],
            &axis(&[
                slot(&app("mult", &[n("i"), n("m")]), Some(&n("m"))),
                slot(&n("L"), None),
                slot(&app("div", &[n("L"), r("2")]), None),
            ]),
            &axis(&[slot(&index("v", &n("i")), None)]),
        ),
    ];
    let res = convert(1, &regs);
    let line = plot_line(&res.calcpad);
    assert_eq!(
        line,
        "$Plot{i*m/m|line(i; v) & L/m|line(i; v) & L/2/m|line(i; v) @ i = 1 : 2}"
    );
    let charts = draw(&res.calcpad);
    assert_eq!(charts[0].series.len(), 3);
    // Vertical: x stays at 6, y runs over the data.
    let v = &charts[0].series[1].points;
    assert!(v.iter().all(|p| (p.0 - 6.0).abs() < 1e-9));
    assert!(v.iter().any(|p| (p.1 - 1.0).abs() < 1e-9) && v.iter().any(|p| (p.1 - 5.0).abs() < 1e-9));
}

#[test]
fn whole_arrays_plotted_against_each_other_run_over_the_element_number() {
    let regs = [
        math(1, 0.0, &define("a", &matrix(3, 1, &["1", "2", "3"]))),
        math(2, 20.0, &define("b", &matrix(3, 1, &["10", "40", "90"]))),
        plot(
            3,
            40.0,
            "",
            &[trace("lines")],
            &axis(&[slot(&n("a"), None)]),
            &axis(&[slot(&n("b"), None)]),
        ),
    ];
    let res = convert(1, &regs);
    assert_eq!(
        plot_line(&res.calcpad),
        "$Plot{line(i_plot; (a))|line(i_plot; (b)) @ i_plot = 1 : len(a)}"
    );
    let pts = &draw(&res.calcpad)[0].series[0].points;
    assert_eq!(pts.first().copied(), Some((1.0, 10.0)));
    let (xl, yl) = *pts.last().unwrap();
    assert!((xl - 3.0).abs() < 1e-9 && (yl - 90.0).abs() < 1e-9, "{xl} {yl}");
}

#[test]
fn title_and_labels_are_kept_as_text_and_the_dropped_style_is_a_note() {
    let x = Axis {
        label: "distance".into(),
        attrs: r#"start="0" end="9""#.into(),
        domain: "<xyDomain scale-type=\"log\"><startValue><ml:real>1</ml:real></startValue><endValue><ml:real>9</ml:real></endValue></xyDomain>".into(),
        ..axis(&[slot(&n("x"), None)])
    };
    let y = Axis {
        label: "force".into(),
        ..axis(&[slot(&app("mult", &[n("x"), n("x")]), None)])
    };
    let res = convert(1, &[plot(1, 0.0, "Force", &[trace("columns")], &x, &y)]);
    assert!(res.calcpad.contains("<b>Force</b>"), "{}", res.calcpad);
    assert!(res.calcpad.contains("x: distance; y: force"), "{}", res.calcpad);
    let note = res.report.notes().find(|i| i.code == Code::PlotStyle).expect("note");
    let d = note.detail.clone().unwrap();
    for w in ["colours", "columns", "logarithmic", "axis ranges"] {
        assert!(d.contains(w), "{d}");
    }
    assert_eq!(res.report.kinds["plot"].with_notes, 1);
    draw(&res.calcpad);
}

#[test]
fn a_plot_that_uses_a_missing_name_is_a_gap_with_that_reason() {
    let x = axis(&[slot(&n("a"), None)]);
    let y = axis(&[slot(&n("b"), None)]);
    let res = convert(1, &[plot(1, 0.0, "", &[trace("lines")], &x, &y)]);
    assert_eq!(res.report.kinds["plot"].attention, 1);
    assert!(res.report.gaps().any(|i| i.code == Code::UndefinedVariable));
    assert!(!res.calcpad.contains("$Plot"));
}

#[test]
fn an_empty_slot_is_a_gap() {
    let x = axis(&[format!(
        "<plotEquation><math><ml:placeholder/></math><math><ml:placeholder/></math></plotEquation>"
    )]);
    let y = axis(&[slot(&r("1"), None)]);
    let res = convert(1, &[plot(1, 0.0, "", &[trace("lines")], &x, &y)]);
    assert_eq!(res.report.kinds["plot"].attention, 1);
    assert!(gap_detail(&res, Code::Plot).unwrap().contains("empty slot"));
}

#[test]
fn other_kinds_of_plot_and_binary_plots_are_named_gaps() {
    let res = convert(
        1,
        &[
            region(1, 0.0, "<plot><surfacePlot/></plot>"),
            region(2, 20.0, r#"<plot item-idref="7"/>"#),
        ],
    );
    assert_eq!(res.report.kinds["plot"].attention, 2);
    let details: Vec<String> = res
        .report
        .gaps()
        .filter(|i| i.code == Code::Plot)
        .filter_map(|i| i.detail.clone())
        .collect();
    assert!(details.iter().any(|d| d.contains("surfacePlot")), "{details:?}");
    assert!(details.iter().any(|d| d.contains("binary")), "{details:?}");
}

#[test]
fn two_range_variables_are_not_guessed_between() {
    let regs = [
        math(1, 0.0, &define("i", &range(&r("1"), &r("3")))),
        math(2, 20.0, &define("j", &range(&r("1"), &r("3")))),
        plot(
            3,
            40.0,
            "",
            &[trace("lines")],
            &axis(&[slot(&n("i"), None)]),
            &axis(&[slot(&n("j"), None)]),
        ),
    ];
    let res = convert(1, &regs);
    assert!(gap_detail(&res, Code::Plot)
        .unwrap()
        .contains("several range variables"));
}

#[test]
fn a_bare_zero_in_a_plot_condition_or_branch_takes_the_units_of_the_other_side() {
    // y = if(f[i] < 0, -f[i], 0): the zeros are zeros of a force in the source.
    let cond = app("lessThan", &[index("f", &n("i")), r("0")]);
    let body = call("if", &[cond, app("neg", &[index("f", &n("i"))]), r("0")]);
    let regs = [
        math(
            1,
            0.0,
            &define(
                "f",
                &format!(
                    "<ml:apply><ml:mult/>{}{}</ml:apply>",
                    matrix(3, 1, &["1", "-2", "3"]),
                    n("kN")
                ),
            ),
        ),
        math(2, 20.0, &define("i", &range(&r("1"), &r("3")))),
        plot(
            3,
            40.0,
            "",
            &[trace("lines")],
            &axis(&[slot(&n("i"), None)]),
            &axis(&[slot(&body, None)]),
        ),
    ];
    let res = convert(1, &regs);
    let line = plot_line(&res.calcpad);
    assert!(line.contains("0*getunits("), "{line}");
    let pts = &draw(&res.calcpad)[0].series[0].points;
    // Compression positive: 0 at i = 1, 2 at i = 2, 0 at i = 3.
    assert!(
        pts.iter().any(|p| (p.0 - 2.0).abs() < 1e-9 && (p.1 - 2.0).abs() < 1e-9),
        "{pts:?}"
    );
    assert!(pts.iter().any(|p| (p.0 - 3.0).abs() < 1e-9 && p.1.abs() < 1e-9));
}

#[test]
fn a_unitless_axis_shows_the_quantity_in_base_units_not_in_the_unit_it_is_held_in() {
    // f is held in kN (62 kN) and the sheet shows it in N: the source plots f / 1000 as a plain
    // number of its base unit, 62. Held in kN, `f / 1000` would plot 0.062.
    let force = format!(
        "<ml:apply><ml:mult/>{}{}</ml:apply>",
        matrix(3, 1, &["62", "20", "5"]),
        n("kN")
    );
    let eval_f = format!(
        "<ml:eval>{}<ml:unitOverride>{}</ml:unitOverride></ml:eval>",
        n("f"),
        n("N")
    );
    let regs = [
        math(1, 0.0, &define("f", &force)),
        math(2, 20.0, &eval_f),
        math(3, 40.0, &define("i", &range(&r("1"), &r("3")))),
        plot(
            4,
            60.0,
            "",
            &[trace("lines")],
            &axis(&[slot(&n("i"), None)]),
            &axis(&[slot(&app("div", &[index("f", &n("i")), r("1000")]), None)]),
        ),
    ];
    let res = convert(1, &regs);
    let line = plot_line(&res.calcpad);
    assert!(line.ends_with("/N @ i = 1 : 3}") || line.contains("/N @"), "{line}");
    let pts = &draw(&res.calcpad)[0].series[0].points;
    let top = pts.iter().fold(f64::MIN, |a, p| a.max(p.1));
    assert!((top - 62.0).abs() < 1e-6, "{top}");

    // An axis that types its unit, or an expression that names a unit, is left alone.
    let typed = convert(
        1,
        &[
            math(1, 0.0, &define("f", &force)),
            math(2, 20.0, &eval_f),
            math(3, 40.0, &define("i", &range(&r("1"), &r("3")))),
            plot(
                4,
                60.0,
                "",
                &[trace("lines")],
                &axis(&[slot(&n("i"), None)]),
                &axis(&[slot(&index("f", &n("i")), Some(&n("kN")))]),
            ),
        ],
    );
    let line = plot_line(&typed.calcpad);
    assert!(
        line.contains("/kN") && !line.contains("/N @") && !line.contains("m_"),
        "{line}"
    );
    let top = draw(&typed.calcpad)[0].series[0]
        .points
        .iter()
        .fold(f64::MIN, |a, p| a.max(p.1));
    assert!((top - 62.0).abs() < 1e-6, "{top}");
}
