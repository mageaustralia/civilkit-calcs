//! linterp: linear interpolation in a table that continues along the end segments outside it.
use calcpad_core::base_types::value::Value;
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn run(src: &str) -> ExpressionParser {
    let mut p = ExpressionParser::new();
    p.parse(src, true);
    p
}

fn real(p: &ExpressionParser, name: &str) -> f64 {
    match p.variable(name) {
        Some(Value::Real(r)) => r.d,
        other => panic!("{name}: {other:?}"),
    }
}

fn unit_text(p: &ExpressionParser, name: &str) -> String {
    match p.variable(name) {
        Some(Value::Real(r)) => r.units.as_ref().map(|u| u.text()).unwrap_or_default(),
        other => panic!("{name}: {other:?}"),
    }
}

fn fails(src: &str, needle: &str) -> bool {
    let p = run(src);
    p.html_result.contains("class=\"err\"") && p.html_result.contains(needle)
}

#[test]
fn linterp_agrees_with_the_table_inside_it() {
    // Segments 0..10 (slope 10) and 10..20 (slope 20).
    let p = run("xs = [0; 10; 20]\nys = [0; 100; 300]\na = linterp(5; xs; ys)\nb = linterp(10; xs; ys)\nc = linterp(15; xs; ys)\nd = linterp(20; xs; ys)\ne = linterp(0; xs; ys)");
    assert_eq!(real(&p, "a"), 50.0);
    assert_eq!(real(&p, "b"), 100.0);
    assert_eq!(real(&p, "c"), 200.0);
    assert_eq!(real(&p, "d"), 300.0);
    assert_eq!(real(&p, "e"), 0.0);
}

#[test]
fn linterp_continues_along_the_end_segments_outside_the_table() {
    // Above 20 the last segment goes on (slope 20): 300 + 20 * 5 = 400. Below 0 the first one
    // (slope 10): 0 - 10 * 5 = -50.
    let p = run("xs = [0; 10; 20]\nys = [0; 100; 300]\nu = linterp(25; xs; ys)\nl = linterp(-5; xs; ys)");
    assert_eq!(real(&p, "u"), 400.0);
    assert_eq!(real(&p, "l"), -50.0);
}

#[test]
fn linterp_converts_x_units_and_returns_y_units() {
    // 150 cm = 1.5 m on the table [0 1 2] m -> [0 10 30] kN: 10 + 20 * 0.5 = 20 kN; 3 m extrapolates to 50 kN.
    let p = run("xs = [0; 1; 2]*m\nys = [0; 10; 30]*kN\na = linterp(150cm; xs; ys)\nb = linterp(3m; xs; ys)");
    assert_eq!(real(&p, "a"), 20.0);
    assert_eq!(unit_text(&p, "a"), "kN");
    assert_eq!(real(&p, "b"), 50.0);
}

#[test]
fn linterp_refuses_bad_tables() {
    assert!(fails("r = linterp(5; [0; 10; 5]; [0; 1; 2])", "increasing"));
    assert!(fails("r = linterp(5; [0; 10]; [0; 1; 2])", "same length"));
    assert!(fails("r = linterp(5; [3]; [9])", "two points"));
}

#[test]
fn linterp_stays_usable_as_a_name_inside_a_function() {
    let p = run("f(x) = linterp(x; [0; 1]; [10; 20])\ng = f(3)");
    assert_eq!(real(&p, "g"), 40.0);
}
