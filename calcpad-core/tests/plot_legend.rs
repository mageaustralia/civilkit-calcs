//! A `$Plot` curve may carry a name in double quotes after its expression; a plot with a name gets a legend in its SVG
//! (DEVIATIONS.md D19).
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn render(source: &str) -> (String, ExpressionParser) {
    let mut parser = ExpressionParser::new();
    parser.capture_values = true;
    parser.plot_settings.vector_graphics = true;
    parser.parse(source, true);
    (parser.html_result.clone(), parser)
}

#[test]
fn named_curves_draw_a_legend_and_name_the_captured_series() {
    let (html, p) = render("$Plot{sin(x) \"sine\" & cos(x) \"cosine\" @ x = 0 : 6}");
    assert!(!html.contains("class=\"err\""), "{html}");
    assert!(html.contains("<rect class=\"Legend\""), "{html}");
    assert!(html.contains(">sine </text>") && html.contains(">cosine </text>"));
    // Swatches in the series colours, in curve order.
    let legend = &html[html.find("class=\"Legend\"").unwrap()..];
    let s1 = legend.find("class=\"Series1\"").unwrap();
    let s2 = legend.find("class=\"Series2\"").unwrap();
    assert!(s1 < s2);
    let c = &p.captured_plots[0];
    assert_eq!(
        c.series.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["sine", "cosine"]
    );
}

#[test]
fn a_plot_without_names_is_unchanged() {
    let (html, p) = render("$Plot{sin(x) & cos(x) @ x = 0 : 6}");
    assert!(!html.contains("Legend"));
    assert_eq!(p.captured_plots[0].series[0].name, "sin(x)");
    // A name on one curve only: the other is listed by its expression.
    let (html, _) = render("$Plot{sin(x) \"sine\" & cos(x) @ x = 0 : 6}");
    assert!(
        html.contains(">sine </text>") && html.contains(">cos(x) </text>"),
        "{html}"
    );
}

#[test]
fn the_formula_view_leaves_the_names_out() {
    let mut parser = ExpressionParser::new();
    parser.parse("$Plot{sin(x) \"sine\" @ x = 0 : 6}", false);
    assert!(!parser.html_result.contains("sine"), "{}", parser.html_result);
    assert!(!parser.html_result.contains("class=\"err\""), "{}", parser.html_result);
}
