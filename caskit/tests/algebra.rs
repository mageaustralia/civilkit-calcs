//! Factoring, solving, rearranging formulas and linear systems.
mod common;
use caskit::{parse_equation, solve, solve_expr, solve_linear_system, CasError, Equation, Expr};
use common::*;

macro_rules! factors {
    ($n:ident, $src:expr, $out:expr) => {
        #[test]
        fn $n() {
            let f = p($src).factor().unwrap();
            assert_eq!(f, p($out), "factor({}) gave {}", $src, f);
            // multiplying back gives the original
            assert_eq!(f.expand().unwrap(), s($src), "expand(factor({}))", $src);
        }
    };
}
macro_rules! roots {
    ($n:ident, $eq:expr, $v:expr, [$($r:expr),*]) => {
        #[test]
        fn $n() {
            let got = solve(&parse_equation($eq).unwrap(), $v).unwrap();
            let want: Vec<Expr> = vec![$(s($r)),*];
            assert_eq!(got, want, "{}: got {:?}", $eq, got.iter().map(|e| e.to_ascii()).collect::<Vec<_>>());
        }
    };
}
macro_rules! solve_err {
    ($n:ident, $eq:expr, $v:expr, $pat:pat) => {
        #[test]
        fn $n() {
            let r = solve(&parse_equation($eq).unwrap(), $v);
            assert!(matches!(r, Err($pat)), "{} gave {:?}", $eq, r);
        }
    };
}

// ---- factor -------------------------------------------------------------------------------------
factors!(f_quadratic, "x^2 - 5*x + 6", "(x-2)*(x-3)");
factors!(f_difference_of_squares, "x^2 - 1", "(x-1)*(x+1)");
factors!(f_perfect_square, "x^2 + 2*x + 1", "(x+1)^2");
factors!(f_with_content, "2*x^2 - 8", "2*(x-2)*(x+2)");
factors!(f_cubic, "x^3 - 6*x^2 + 11*x - 6", "(x-1)*(x-2)*(x-3)");
factors!(f_quartic_leftover, "x^4 - 1", "(x-1)*(x+1)*(x^2+1)");
factors!(f_leading_coefficient, "6*x^2 + x - 1", "(2*x+1)*(3*x-1)");
factors!(f_irreducible_quadratic_unchanged, "x^2 + 1", "x^2 + 1");
factors!(f_irrational_roots_unchanged, "x^2 - 2", "x^2 - 2");
factors!(f_common_symbol, "a*x + a*b", "a*(x+b)");
factors!(f_common_symbol_and_quadratic, "w*x^2 - 5*w*x + 6*w", "w*(x-2)*(x-3)");
factors!(f_common_power, "x^3 - x", "x*(x-1)*(x+1)");
factors!(f_numeric_content, "2*x + 4", "2*(x+2)");
factors!(f_rational_content, "x/2 + 1/2", "1/2*(x+1)");
factors!(f_all_negative, "-x - 1", "-(x+1)");
factors!(f_negative_leading_quadratic, "-x^2 + 1", "-(x-1)*(x+1)");
factors!(f_negative_power_common, "x^2/E - x/E", "x*(x-1)/E");
factors!(f_beam_moment, "L^2*w/8 - L*w/4", "L*w*(L-2)/8");
factors!(f_triple_root, "x^3 - 3*x^2 + 3*x - 1", "(x-1)^3");
factors!(f_fractional_root, "2*x^2 - 3*x + 1", "(2*x-1)*(x-1)");
factors!(f_multi_symbol_quadratic_unchanged, "a*x^2 + b*x + c", "a*x^2 + b*x + c");
#[test]
fn f_cancel_against_denominator() {
    // factor cancels a common factor of a quotient; simplify alone does not
    assert_eq!(p("(x^2-1)/(x+1)").factor().unwrap(), p("x-1"));
    assert_ne!(s("(x^2-1)/(x+1)"), p("x-1"));
}
factors!(f_product_of_sums, "(x^2-4)*(x^2-9)", "(x-2)*(x+2)*(x-3)*(x+3)");
factors!(f_power_of_sum, "(x^2-4)^2", "(x-2)^2*(x+2)^2");
factors!(f_monomial_unchanged, "3*x*y", "3*x*y");
factors!(f_number_unchanged, "12", "12");
factors!(f_degree_five, "x^5 - x^4 - x + 1", "(x-1)^2*(x+1)*(x^2+1)");
factors!(f_zero_root, "x^2 + 3*x", "x*(x+3)");

#[test]
fn factor_then_expand_round_trips_on_many() {
    for src in ["x^2+7*x+12", "x^3-x^2-4*x+4", "4*x^2-1", "x^4-5*x^2+4", "3*x^3+x^2-3*x-1", "a*b+a*c+a*d"] {
        let f = p(src).factor().unwrap();
        assert_eq!(f.expand().unwrap(), s(src), "{src} -> {f}");
    }
}

// ---- solve: linear and rearrangement ------------------------------------------------------------
roots!(s_linear_numeric, "2*x + 3 = 7", "x", ["2"]);
roots!(s_linear_symbolic, "a*x + b = c", "x", ["(c-b)/a"]);
roots!(s_rearrange_for_w, "M = w*L^2/8", "w", ["8*M/L^2"]);
roots!(s_rearrange_for_m, "M = w*L^2/8", "M", ["w*L^2/8"]);
roots!(s_rearrange_for_l, "M = w*L^2/8", "L", ["sqrt(8*M/w)", "-sqrt(8*M/w)"]);
roots!(s_deflection_for_e, "d = 5*w*L^4/(384*E*I)", "E", ["5*w*L^4/(384*d*I)"]);
roots!(s_deflection_for_l, "d = 5*w*L^4/(384*E*I)", "L", ["(384*d*E*I/(5*w))^(1/4)", "-(384*d*E*I/(5*w))^(1/4)"]);
roots!(s_stress_for_section, "s = M/Z", "Z", ["M/s"]);
roots!(s_reciprocal, "1/x = 4", "x", ["1/4"]);
roots!(s_inverse_square, "1/x^2 = 4", "x", ["1/2", "-1/2"]);
roots!(s_square_root_equation, "sqrt(x) = 3", "x", ["9"]);
roots!(s_cube, "x^3 = 8", "x", ["2"]);
roots!(s_negative_cube, "x^3 = -8", "x", ["-2"]);
roots!(s_fractional_power, "x^(2/3) = 4", "x", ["8", "-8"]);
roots!(s_fourth_power, "x^4 = 1", "x", ["1", "-1"]);
roots!(s_zero_power, "x^2 = 0", "x", ["0"]);
roots!(s_variable_on_both_sides, "3*x + 2 = x + 10", "x", ["4"]);
roots!(s_circle_area, "A = pi*r^2", "r", ["sqrt(A/pi)", "-sqrt(A/pi)"]);
roots!(s_euler_buckling, "P = pi^2*E*I/L^2", "L", ["pi*sqrt(E*I/P)", "-pi*sqrt(E*I/P)"]);

// ---- solve: quadratics --------------------------------------------------------------------------
roots!(s_quadratic_numeric, "x^2 - 5*x + 6 = 0", "x", ["2", "3"]);
roots!(s_quadratic_pure_square, "x^2 = 4", "x", ["2", "-2"]);
roots!(s_quadratic_irrational, "x^2 - 2 = 0", "x", ["sqrt(2)", "-sqrt(2)"]);
roots!(s_quadratic_double_root, "x^2 + 2*x + 1 = 0", "x", ["-1"]);
roots!(s_quadratic_zero_constant, "x^2 + x = 0", "x", ["0", "-1"]);
roots!(s_quadratic_rational, "2*x^2 - 3*x + 1 = 0", "x", ["1/2", "1"]);
roots!(s_quadratic_leading_negative, "-x^2 + 5*x - 6 = 0", "x", ["2", "3"]);
roots!(s_quadratic_irrational_general, "x^2 - 2*x - 1 = 0", "x", ["1 - sqrt(2)", "1 + sqrt(2)"]);
roots!(s_quadratic_with_fraction_terms, "x + 1/x = 3", "x", ["(3 - sqrt(5))/2", "(3 + sqrt(5))/2"]);
roots!(s_quadratic_rhs, "x^2 = 5*x - 6", "x", ["2", "3"]);

#[test]
fn symbolic_quadratic_formula() {
    let roots = solve(&parse_equation("a*x^2 + b*x + c = 0").unwrap(), "x").unwrap();
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0], s("(-b - sqrt(b^2 - 4*a*c))/(2*a)"));
    assert_eq!(roots[1], s("(-b + sqrt(b^2 - 4*a*c))/(2*a)"));
    // numerically: x^2 - 5x + 6
    let v = vars(&[("a", 1.0), ("b", -5.0), ("c", 6.0)]);
    let mut got: Vec<f64> = roots.iter().map(|r| r.eval(&v).unwrap()).collect();
    got.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(close(got[0], 2.0) && close(got[1], 3.0));
}

#[test]
fn every_root_satisfies_the_equation() {
    for (eq, v) in [
        ("a*x^2 + b*x + c = 0", "x"),
        ("M = w*L^2/8", "L"),
        ("x + 1/x = k", "x"),
        ("d = 5*w*L^4/(384*E*I)", "L"),
        ("a*x + b = c*x + d", "x"),
    ] {
        let eqn = parse_equation(eq).unwrap();
        let rs = solve(&eqn, v).unwrap();
        let point = vars(&[("a", 1.5), ("b", 7.0), ("c", 2.0), ("d", 3.0), ("k", 5.0), ("M", 4.0), ("w", 2.0), ("E", 3.0), ("I", 2.0)]);
        for r in rs {
            let mut vals = point.clone();
            vals.insert(v.to_string(), r.eval(&point).unwrap());
            let (l, rr) = (eqn.lhs.eval(&vals).unwrap(), eqn.rhs.eval(&vals).unwrap());
            if l.is_finite() {
                assert!(close(l, rr), "{eq}: root {r} gives {l} vs {rr}");
            }
        }
    }
}

solve_err!(s_complex_roots, "x^2 + 1 = 0", "x", CasError::ComplexRoots);
solve_err!(s_complex_roots_general, "x^2 + 2*x + 5 = 0", "x", CasError::ComplexRoots);
solve_err!(s_no_solution_sqrt, "sqrt(x) = -3", "x", CasError::NoSolution);
solve_err!(s_identity, "x = x", "x", CasError::Identity);
solve_err!(s_no_solution_constant, "x + 1 = x", "x", CasError::NoSolution);
solve_err!(s_variable_absent, "y = 3", "x", CasError::NoSolution);
solve_err!(s_cubic_unsupported, "x^3 + x = 1", "x", CasError::Unsupported(_));
solve_err!(s_trig_unsupported, "sin(x) = 0.5", "x", CasError::Unsupported(_));
solve_err!(s_quintic_unsupported, "x^5 + x = 0", "x", CasError::Unsupported(_));
solve_err!(s_exponential_unsupported, "exp(x) = 3", "x", CasError::Unsupported(_));
solve_err!(s_fractional_sum_unsupported, "sqrt(x) + x = 2", "x", CasError::Unsupported(_));
solve_err!(s_zero_negative_power, "1/x = 0", "x", CasError::NoSolution);

#[test]
fn solve_expr_means_equals_zero() {
    let r = solve_expr(&p("x - 7"), "x").unwrap();
    assert_eq!(r, vec![p("7")]);
}

#[test]
fn equation_residual() {
    let e = Equation::parse("a = b").unwrap();
    assert_eq!(e.residual().unwrap(), p("a - b"));
}

// ---- linear systems -----------------------------------------------------------------------------
fn sys(eqs: &[&str], vars: &[&str]) -> Result<Vec<Expr>, CasError> {
    let eqs: Vec<Equation> = eqs.iter().map(|e| parse_equation(e).unwrap()).collect();
    solve_linear_system(&eqs, vars)
}

#[test]
fn system_two_numeric() {
    let r = sys(&["x + y = 3", "x - y = 1"], &["x", "y"]).unwrap();
    assert_eq!(r, vec![p("2"), p("1")]);
}

#[test]
fn system_two_symbolic() {
    let r = sys(&["a*x + b*y = e", "c*x + d*y = f"], &["x", "y"]).unwrap();
    assert_eq!(r[0], s("(e*d - b*f)/(a*d - b*c)"));
    assert_eq!(r[1], s("(a*f - e*c)/(a*d - b*c)"));
}

#[test]
fn system_three_numeric() {
    let r = sys(&["x + y + z = 6", "2*y + 5*z = -4", "2*x + 5*y - z = 27"], &["x", "y", "z"]).unwrap();
    assert_eq!(r, vec![p("5"), p("3"), p("-2")]);
}

#[test]
fn system_four_numeric() {
    let r = sys(
        &["a + b + c + d = 10", "a - b = -1", "b - c = -1", "c - d = -1"],
        &["a", "b", "c", "d"],
    )
    .unwrap();
    assert_eq!(r, vec![p("1"), p("2"), p("3"), p("4")]);
}

#[test]
fn system_with_symbolic_right_hand_side() {
    let r = sys(&["x + y = s", "x - y = d"], &["x", "y"]).unwrap();
    assert_eq!(r[0], s("(s + d)/2"));
    assert_eq!(r[1], s("(s - d)/2"));
}

#[test]
fn system_single_unknown() {
    assert_eq!(sys(&["2*x = 6"], &["x"]).unwrap(), vec![p("3")]);
}

#[test]
fn system_singular() {
    assert_eq!(sys(&["x + y = 1", "2*x + 2*y = 2"], &["x", "y"]), Err(CasError::Singular));
}

#[test]
fn system_nonlinear_rejected() {
    assert!(matches!(sys(&["x*y = 1", "x + y = 2"], &["x", "y"]), Err(CasError::Unsupported(_))));
}

#[test]
fn system_wrong_equation_count_rejected() {
    assert!(matches!(sys(&["x + y = 1"], &["x", "y"]), Err(CasError::Unsupported(_))));
}

#[test]
fn system_five_unknowns_rejected() {
    let r = sys(&["a=1", "b=1", "c=1", "d=1", "e1=1"], &["a", "b", "c", "d", "e1"]);
    assert!(matches!(r, Err(CasError::Unsupported(_))));
}

#[test]
fn system_solution_satisfies_equations() {
    let eqs = ["2*x + 3*y - z = a", "x - y + 4*z = b", "3*x + y + z = c"];
    let r = sys(&eqs, &["x", "y", "z"]).unwrap();
    let v = vars(&[("a", 3.0), ("b", -2.0), ("c", 7.0)]);
    let mut vals = v.clone();
    for (n, e) in ["x", "y", "z"].iter().zip(&r) {
        vals.insert(n.to_string(), e.eval(&v).unwrap());
    }
    for e in eqs {
        let eq = parse_equation(e).unwrap();
        assert!(close(eq.lhs.eval(&vals).unwrap(), eq.rhs.eval(&vals).unwrap()), "{e}");
    }
}
