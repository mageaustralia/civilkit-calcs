//! eigenvals, eigenvecs and eigen of full matrices (symmetric or not), ceigenvals and ceigenvecs.
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

#[test]
fn a_symmetric_literal_no_longer_needs_copy_to_symmetric() {
    let p = run("A = [2; 1|1; 2]\nL = eigenvals(A)\nS = copy(A; symmetric(2); 1; 1)\nLs = eigenvals(S)");
    close_all(&vec(&p, "L"), &[1.0, 3.0]);
    close_all(&vec(&p, "L"), &vec(&p, "Ls"));
}

#[test]
fn a_nonsymmetric_matrix_with_real_eigenvalues() {
    let p = run("A = [4; 1|2; 3]\nL = eigenvals(A)\nV = eigenvecs(A)\nE = eigen(A)\nL1 = eigenvals(A; -1)");
    close_all(&vec(&p, "L"), &[2.0, 5.0]);
    let s5 = 5f64.sqrt();
    let h = 0.5f64.sqrt();
    let v = mat(&p, "V");
    close_all(&v[0], &[-1.0 / s5, 2.0 / s5]);
    close_all(&v[1], &[h, h]);
    let e = mat(&p, "E");
    close_all(&e[0], &[2.0, -1.0 / s5, 2.0 / s5]);
    close_all(&e[1], &[5.0, h, h]);
    close_all(&vec(&p, "L1"), &[5.0]);
}

#[test]
fn complex_eigenvalues_need_the_c_functions() {
    let p = run("R = [0; -1|1; 0]\nL = eigenvals(R)\nC = ceigenvals(R)\nW = ceigenvecs(R)");
    assert!(p.variable("L").is_none());
    assert!(p.html_result.contains("complex eigenvalues"), "{}", p.html_result);
    let c = mat(&p, "C");
    close_all(&c[0], &[0.0, 0.0]);
    close_all(&c[1], &[-1.0, 1.0]);
    let h = 0.5f64.sqrt();
    let w = mat(&p, "W");
    // Eigenvalue -i: v = [h, h i].
    close_all(&w[0], &[h, 0.0]);
    close_all(&w[1], &[0.0, h]);
    close_all(&w[2], &[h, 0.0]);
    close_all(&w[3], &[0.0, -h]);
}

#[test]
fn units_are_converted_and_kept() {
    let p = run("A = [2MPa; 1000kPa|1MPa; 3MPa]\nL = eigenvals(A)\nB = [2; 3|1; 4]*1MPa\nM = eigenvals(B)");
    let r = 5f64.sqrt();
    close_all(&vec(&p, "L"), &[(5.0 - r) / 2.0, (5.0 + r) / 2.0]);
    close_all(&vec(&p, "M"), &[1.0, 5.0]);
    match p.variable("M") {
        Some(Value::Vector(v)) => assert_eq!(v.get(0).units.as_ref().map(|u| u.text()).unwrap_or_default(), "MPa"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn principal_stresses_of_a_plane_stress_state() {
    // sigma_x = 80, sigma_y = -40, tau_xy = 25: centre 20, radius 65.
    let p = run("S = [80; 25|25; -40]\nL = eigenvals(S)\nV = eigenvecs(S)");
    close_all(&vec(&p, "L"), &[-45.0, 85.0]);
    let v = mat(&p, "V");
    let theta = v[1][1].atan2(v[1][0]).to_degrees();
    let want = 0.5 * (50f64 / 120.0).atan().to_degrees();
    assert!(
        (theta.abs() - want).abs() < 1e-9 || (theta.abs() - (180.0 - want)).abs() < 1e-9,
        "{theta}"
    );
}

#[test]
fn a_rectangular_matrix_is_refused() {
    let p = run("A = [1; 2; 3|4; 5; 6]\nL = eigenvals(A)\nC = ceigenvals(A)");
    assert!(p.variable("L").is_none() && p.variable("C").is_none());
    assert!(p.html_result.contains("Matrix must be square"));
}
