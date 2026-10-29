//! eigenvals(K; M), eigenvecs(K; M) and eigen(K; M): the generalized symmetric eigenproblem K x = λ M x.
use calcpad_core::base_types::value::Value;
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn run(src: &str) -> ExpressionParser {
    let mut p = ExpressionParser::new();
    p.parse(src, true);
    p
}

fn vec(p: &ExpressionParser, name: &str) -> Vec<f64> {
    match p.variable(name) {
        Some(Value::Vector(v)) => v.values().iter().map(|r| r.d).collect(),
        Some(Value::Real(r)) => vec![r.d],
        other => panic!("{name}: {other:?}"),
    }
}

fn mat(p: &ExpressionParser, name: &str) -> Vec<Vec<f64>> {
    match p.variable(name) {
        Some(Value::Matrix(m)) => (0..m.row_count())
            .map(|i| (0..m.col_count()).map(|j| m.get(i, j).d).collect())
            .collect(),
        other => panic!("{name}: {other:?}"),
    }
}

fn close_all(a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
    for (x, y) in a.iter().zip(b) {
        assert!((x - y).abs() <= 1e-9 * y.abs().max(1.0), "{a:?} vs {b:?}");
    }
}

fn units_of(p: &ExpressionParser, name: &str) -> String {
    match p.variable(name) {
        Some(Value::Vector(v)) => v.get(0).units.as_ref().map(|u| u.text()).unwrap_or_default(),
        Some(Value::Matrix(m)) => m.get(0, 0).units.as_ref().map(|u| u.text()).unwrap_or_default(),
        Some(Value::Real(r)) => r.units.as_ref().map(|u| u.text()).unwrap_or_default(),
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn shear_frame_gives_the_same_frequencies_as_the_inverse_workaround() {
    let p = run(concat!(
        "K = [70000; -30000|-30000; 30000]*1kN/m\n",
        "M = [30; 0|0; 20]*1t\n",
        "L = eigenvals(K; M)\n",
        "L0 = eigenvals(inverse(M)*K)\n",
        "V = eigenvecs(K; M)\n",
        "E = eigen(K; M)\n",
        "c11 = dot(row(V; 1); M*row(V; 1))\n",
        "c12 = dot(row(V; 1); M*row(V; 2))\n",
        "f_1 = sqrt(L.1)/(2*π)|Hz\n",
    ));
    close_all(&vec(&p, "L"), &vec(&p, "L0"));
    assert_eq!(units_of(&p, "L"), "s^(-2)");
    close_all(&vec(&p, "c11"), &[1.0]);
    assert!(vec(&p, "c12")[0].abs() < 1e-12);
    assert!((vec(&p, "f_1")[0] - 3.972456).abs() < 1e-6);
    let (v, e) = (mat(&p, "V"), mat(&p, "E"));
    close_all(&e[0][1..], &v[0]);
    close_all(&[e[0][0], e[1][0]], &vec(&p, "L"));
    // Largest component positive.
    for r in &v {
        let big = r.iter().cloned().fold(0.0f64, |a, c| if c.abs() > a.abs() { c } else { a });
        assert!(big > 0.0, "{r:?}");
    }
}

#[test]
fn a_count_still_works_with_one_matrix() {
    let p = run("A = [2; 1|1; 2]\nL = eigenvals(A; 1)\nB = [1; 0|0; 4]\nG = eigenvals(A; B)");
    close_all(&vec(&p, "L"), &[1.0]);
    // det(A - λB) = (2 - λ)(2 - 4λ) - 1 = 4λ² - 10λ + 3.
    let r = 13f64.sqrt();
    close_all(&vec(&p, "G"), &[(5.0 - r) / 4.0, (5.0 + r) / 4.0]);
}

#[test]
fn bad_pairs_are_refused() {
    let p = run(concat!(
        "K = [2; 1|1; 2]\n",
        "a = eigenvals(K; [1; 2|2; 1])\n",
        "b = eigenvecs([2; 1|0; 2]; K)\n",
        "c = eigen(K; [1; 0; 0|0; 1; 0|0; 0; 1])\n",
    ));
    for n in ["a", "b", "c"] {
        assert!(p.variable(n).is_none(), "{n}");
    }
    assert!(p.html_result.contains("must be symmetric and positive definite"), "{}", p.html_result);
    assert!(p.html_result.contains("(K) must be symmetric"));
}

#[test]
fn mode_shapes_carry_units_that_make_the_modal_mass_one() {
    let p = run("K = [2; -1|-1; 1]*1kN/m\nM = [2; 0|0; 1]*1kg\nV = eigenvecs(K; M)\nm_1 = dot(row(V; 1); M*row(V; 1))");
    assert_eq!(units_of(&p, "V"), "kg^(-0.5)");
    close_all(&vec(&p, "m_1"), &[1.0]);
}
