//! median, stdev, stdevp, var, varp, cumsum, convolve, linspace, polyfit, interp, polyroots.
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
        Some(Value::Vector(v)) => v.get(0).units.as_ref().map(|u| u.text()).unwrap_or_default(),
        other => panic!("{name}: {other:?}"),
    }
}

fn vec(p: &ExpressionParser, name: &str) -> Vec<f64> {
    match p.variable(name) {
        Some(Value::Vector(v)) => v.values().iter().map(|r| r.d).collect(),
        other => panic!("{name}: {other:?}"),
    }
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
}

fn close_all(a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
    for (x, y) in a.iter().zip(b) {
        close(*x, *y);
    }
}

/// True when the sheet reports an error whose text contains `needle` and `r` is not set.
fn fails(src: &str, needle: &str) -> bool {
    let p = run(src);
    p.variable("r").is_none() && p.html_result.contains("class=\"err\"") && p.html_result.contains(needle)
}

#[test]
fn median_odd_even_single_and_unsorted() {
    let p = run("a = median(5; 1; 3)\nb = median(4; 1; 3; 2)\nc = median(7)\nd = median([9; 1; 5; 3; 7])");
    assert_eq!(real(&p, "a"), 3.0);
    assert_eq!(real(&p, "b"), 2.5);
    assert_eq!(real(&p, "c"), 7.0);
    assert_eq!(real(&p, "d"), 5.0);
}

#[test]
fn median_of_a_matrix_and_mixed_arguments() {
    let p = run("M = [1; 2|3; 4]\na = median(M)\nb = median(M; 10)");
    assert_eq!(real(&p, "a"), 2.5);
    assert_eq!(real(&p, "b"), 3.0);
}

#[test]
fn median_keeps_the_unit_of_the_first_value() {
    let p = run("v = [3; 1; 2]*kN\na = median(v)\nb = median(1kN; 2000N; 3kN)");
    assert_eq!(real(&p, "a"), 2.0);
    assert_eq!(unit_text(&p, "a"), "kN");
    assert_eq!(real(&p, "b"), 2.0);
    assert_eq!(unit_text(&p, "b"), "kN");
}

#[test]
fn sample_and_population_statistics() {
    // 2 4 4 4 5 5 7 9: mean 5, sum of squares of deviations 32
    let p = run("v = [2; 4; 4; 4; 5; 5; 7; 9]\na = var(v)\nb = varp(v)\nc = stdev(v)\nd = stdevp(v)");
    close(real(&p, "a"), 32.0 / 7.0);
    close(real(&p, "b"), 4.0);
    close(real(&p, "c"), (32.0f64 / 7.0).sqrt());
    close(real(&p, "d"), 2.0);
}

#[test]
fn statistics_of_scalar_lists_and_edge_cases() {
    let p = run("a = stdev(1; 3)\nb = stdevp(5)\nc = varp(5; 5; 5)");
    close(real(&p, "a"), 2.0f64.sqrt());
    assert_eq!(real(&p, "b"), 0.0);
    assert_eq!(real(&p, "c"), 0.0);
    // a sample statistic of one value is undefined
    assert!(fails("r = stdev(5)", "at least two"));
    assert!(fails("r = var([4])", "at least two"));
}

#[test]
fn variance_has_squared_units_and_stdev_the_unit() {
    let p = run("v = [1; 2; 3]*m\na = var(v)\nb = stdev(v)\nc = varp(1m; 300cm)");
    assert_eq!(real(&p, "a"), 1.0);
    assert_eq!(unit_text(&p, "a"), "m^2");
    assert_eq!(real(&p, "b"), 1.0);
    assert_eq!(unit_text(&p, "b"), "m");
    assert_eq!(real(&p, "c"), 1.0);
    assert_eq!(unit_text(&p, "c"), "m^2");
}

#[test]
fn statistics_of_an_empty_vector_is_an_error() {
    assert!(fails("e = find_gt([1; 2]; 5; 1)\nr = median(e)", "at least one"));
    assert!(fails("e = find_gt([1; 2]; 5; 1)\nr = stdevp(e)", "at least one"));
}

#[test]
fn cumsum_runs_and_keeps_units() {
    let p = run("a = cumsum([1; 2; 3; 4])\nb = cumsum([1; 2]*kN)\nc = cumsum([5])");
    close_all(&vec(&p, "a"), &[1.0, 3.0, 6.0, 10.0]);
    close_all(&vec(&p, "b"), &[1.0, 3.0]);
    assert_eq!(unit_text(&p, "b"), "kN");
    close_all(&vec(&p, "c"), &[5.0]);
    let p = run("e = find_gt([1; 2]; 5; 1)\na = cumsum(e)");
    assert!(vec(&p, "e").is_empty(), "{}", p.html_result);
    assert!(vec(&p, "a").is_empty());
}

#[test]
fn convolve_matches_the_polynomial_product() {
    // (1 + 2x + 3x^2)(4 + 5x) = 4 + 13x + 22x^2 + 15x^3
    let p = run("a = convolve([1; 2; 3]; [4; 5])\nb = convolve([2]; [3; 4])");
    close_all(&vec(&p, "a"), &[4.0, 13.0, 22.0, 15.0]);
    close_all(&vec(&p, "b"), &[6.0, 8.0]);
    let p = run("c = convolve([1; 1]*m; [1; 2; 3]*kN)");
    close_all(&vec(&p, "c"), &[1.0, 3.0, 5.0, 3.0]);
    assert!(unit_text(&p, "c").contains('m') && unit_text(&p, "c").contains('N'));
    assert!(fails("e = find_gt([1; 2]; 5; 1)\nr = convolve(e; [1])", "not empty"));
}

#[test]
fn linspace_includes_both_ends() {
    let p = run("a = linspace(0; 1; 5)\nb = linspace(2; 8; 1)\nc = linspace(0m; 100cm; 3)\nd = linspace(3; 3; 2)");
    close_all(&vec(&p, "a"), &[0.0, 0.25, 0.5, 0.75, 1.0]);
    close_all(&vec(&p, "b"), &[2.0]);
    close_all(&vec(&p, "c"), &[0.0, 0.5, 1.0]);
    assert_eq!(unit_text(&p, "c"), "m");
    close_all(&vec(&p, "d"), &[3.0, 3.0]);
    assert!(fails("r = linspace(0; 1; 0)", "positive integer"));
}

#[test]
fn polyfit_recovers_exact_polynomials() {
    // y = 1 + 2x + 3x^2
    let p = run("x = [0; 1; 2; 3; 4]\ny = 1 + 2*x + 3*x^2\nc = polyfit(x; y; 2)");
    close_all(&vec(&p, "c"), &[1.0, 2.0, 3.0]);
    let p = run("x = [1; 2; 3]\nc = polyfit(x; [7; 7; 7]; 0)");
    close_all(&vec(&p, "c"), &[7.0]);
}

#[test]
fn polyfit_least_squares_line() {
    // points (1,1) (2,2) (3,2): slope 0.5, intercept 2/3
    let p = run("c = polyfit([1; 2; 3]; [1; 2; 2]; 1)");
    close_all(&vec(&p, "c"), &[2.0 / 3.0, 0.5]);
}

#[test]
fn polyfit_coefficients_carry_y_over_x_to_the_k() {
    let p = run("x = [0; 1; 2]*m\ny = [1; 3; 5]*kN\nc = polyfit(x; y; 1)");
    let c = vec(&p, "c");
    close_all(&c, &[1.0, 2.0]);
    match p.variable("c") {
        Some(Value::Vector(v)) => {
            assert_eq!(v.get(0).units.as_ref().unwrap().text(), "kN");
            let u1 = v.get(1).units.as_ref().unwrap().text();
            assert!(u1.contains("kN") && u1.contains('m'), "{u1}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn polyfit_refuses_too_few_points_and_bad_input() {
    assert!(fails("r = polyfit([1; 2]; [1; 2]; 2)", "degree + 1"));
    assert!(fails("r = polyfit([1; 2; 3]; [1; 2]; 1)", "same length"));
    // all x equal: the normal equations are singular
    assert!(fails("r = polyfit([2; 2; 2]; [1; 2; 3]; 1)", "ingular"));
}

#[test]
fn interp_is_linear_between_table_points() {
    let p = run("xs = [0; 10; 20]\nys = [0; 100; 300]\na = interp(5; xs; ys)\nb = interp(10; xs; ys)\nc = interp(15; xs; ys)\nd = interp(20; xs; ys)\ne = interp(0; xs; ys)");
    assert_eq!(real(&p, "a"), 50.0);
    assert_eq!(real(&p, "b"), 100.0);
    assert_eq!(real(&p, "c"), 200.0);
    assert_eq!(real(&p, "d"), 300.0);
    assert_eq!(real(&p, "e"), 0.0);
}

#[test]
fn interp_converts_x_units_and_returns_y_units() {
    let p = run("xs = [0; 1; 2]*m\nys = [0; 10; 30]*kN\na = interp(150cm; xs; ys)");
    assert_eq!(real(&p, "a"), 20.0);
    assert_eq!(unit_text(&p, "a"), "kN");
}

#[test]
fn interp_does_not_extrapolate_or_accept_bad_tables() {
    assert!(fails("r = interp(21; [0; 10; 20]; [0; 1; 2])", "outside the table"));
    assert!(fails("r = interp(-1; [0; 10; 20]; [0; 1; 2])", "outside the table"));
    assert!(fails("r = interp(5; [0; 10; 5]; [0; 1; 2])", "increasing"));
    assert!(fails("r = interp(5; [0; 10]; [0; 1; 2])", "same length"));
    let p = run("r = interp(3; [3]; [9])");
    assert_eq!(real(&p, "r"), 9.0);
}

#[test]
fn names_stay_usable_as_variables_and_inside_functions() {
    let p = run("var = 3\nmedian = 4\nr = var + median\nf(x) = stdevp(x; 2*x)\ng = f(2)");
    assert_eq!(real(&p, "r"), 7.0);
    assert_eq!(real(&p, "g"), 1.0);
}

#[test]
fn polyroots_finds_the_real_roots_in_ascending_order() {
    let p = run("a = vector(3)\na.1 = 6\na.2 = -5\na.3 = 1\nr = polyroots(a)");
    close_all(&vec(&p, "r"), &[2.0, 3.0]);
    let p = run("a = vector(4)\na.1 = 0\na.2 = -1\na.3 = 0\na.4 = 1\nr = polyroots(a)");
    close_all(&vec(&p, "r"), &[-1.0, 0.0, 1.0]);
}
