//! Differentiation, integration, substitution and evaluation.
mod common;
use caskit::{CasError, Expr};
use common::*;

macro_rules! deriv {
    ($n:ident, $f:expr, $v:expr, $d:expr) => {
        #[test]
        fn $n() {
            let got = p($f).diff($v).unwrap().simplify().unwrap();
            assert_eq!(got, s($d), "d/d{} of {} gave {}", $v, $f, got);
        }
    };
}
macro_rules! integ {
    ($n:ident, $f:expr, $v:expr, $anti:expr) => {
        #[test]
        fn $n() {
            let got = p($f).integrate($v).unwrap();
            assert_eq!(got, s($anti), "integral of {} gave {}", $f, got);
            // and differentiating the answer returns the integrand
            let back = got.diff($v).unwrap().simplify().unwrap();
            assert_eq!(back, s($f), "derivative of the answer for {}", $f);
        }
    };
}
macro_rules! cannot {
    ($n:ident, $f:expr, $v:expr) => {
        #[test]
        fn $n() {
            let r = p($f).integrate($v);
            assert!(matches!(r, Err(CasError::CannotIntegrate(_))), "{} gave {:?}", $f, r);
        }
    };
}
macro_rules! definite {
    ($n:ident, $f:expr, $v:expr, $a:expr, $b:expr, $val:expr) => {
        #[test]
        fn $n() {
            let got = p($f).integrate_definite($v, &p($a), &p($b)).unwrap();
            assert_eq!(got, s($val), "{}", got);
        }
    };
}
macro_rules! evals {
    ($n:ident, $f:expr, [$(($k:expr, $x:expr)),*], $val:expr) => {
        #[test]
        fn $n() {
            let got = p($f).eval(&vars(&[$(($k, $x)),*])).unwrap();
            assert!(close(got, $val), "{} gave {} expected {}", $f, got, $val);
        }
    };
}

// ---- differentiate ------------------------------------------------------------------------------
deriv!(d_power, "x^3", "x", "3*x^2");
deriv!(d_constant, "5", "x", "0");
deriv!(d_other_symbol, "a*y", "x", "0");
deriv!(d_linear, "a*x + b", "x", "a");
deriv!(d_sin, "sin(x)", "x", "cos(x)");
deriv!(d_cos, "cos(x)", "x", "-sin(x)");
deriv!(d_exp, "exp(x)", "x", "exp(x)");
deriv!(d_exp_chain, "exp(2*x)", "x", "2*exp(2*x)");
deriv!(d_exp_square, "exp(x^2)", "x", "2*x*exp(x^2)");
deriv!(d_ln, "ln(x)", "x", "1/x");
deriv!(d_ln_scaled, "ln(2*x)", "x", "1/x");
deriv!(d_sqrt, "sqrt(x)", "x", "1/(2*sqrt(x))");
deriv!(d_reciprocal, "1/x", "x", "-1/x^2");
deriv!(d_product_rule, "x*sin(x)", "x", "sin(x) + x*cos(x)");
deriv!(d_chain_rule, "sin(x^2)", "x", "2*x*cos(x^2)");
deriv!(d_power_of_sum, "(x^2+1)^3", "x", "6*x*(x^2+1)^2");
deriv!(d_quotient, "x/(x+1)", "x", "1/(x+1) - x/(x+1)^2");
deriv!(d_variable_exponent, "x^x", "x", "x^x*(ln(x) + 1)");
deriv!(d_symbolic_chain, "sin(a*x)", "x", "a*cos(a*x)");
deriv!(d_pi_factor, "pi*x", "x", "pi");
deriv!(d_ei, "E*I*x", "x", "E*I");
deriv!(d_beam_term, "w*x^2/(2*E*I)", "x", "w*x/(E*I)");
deriv!(d_triple_product, "x*y*z", "y", "x*z");
deriv!(d_fractional_power, "x^(3/2)", "x", "3/2*sqrt(x)");
deriv!(d_negative_power, "x^-2", "x", "-2*x^-3");
deriv!(d_exp_base_2, "2^x", "x", "2^x*ln(2)");
deriv!(d_tan_like_quotient, "sin(x)/cos(x)", "x", "1 + sin(x)^2/cos(x)^2");

#[test]
fn second_derivative() {
    let d2 = p("x^4").diff_n("x", 2).unwrap().simplify().unwrap();
    assert_eq!(d2, s("12*x^2"));
}

#[test]
fn zeroth_derivative_is_identity() {
    assert_eq!(p("x^2").diff_n("x", 0).unwrap(), p("x^2"));
}

// ---- integrate ----------------------------------------------------------------------------------
integ!(i_constant, "5", "x", "5*x");
integ!(i_symbolic_constant, "a", "x", "a*x");
integ!(i_one, "1", "x", "x");
integ!(i_x, "x", "x", "x^2/2");
integ!(i_x_squared, "x^2", "x", "x^3/3");
integ!(i_symbolic_coefficient, "a*x", "x", "a*x^2/2");
integ!(i_polynomial, "3*x^2 - 2*x + 7", "x", "x^3 - x^2 + 7*x");
integ!(i_reciprocal, "1/x", "x", "ln(x)");
integ!(i_inverse_square, "1/x^2", "x", "-1/x");
integ!(i_sqrt, "sqrt(x)", "x", "2/3*x^(3/2)");
integ!(i_sin, "sin(x)", "x", "-cos(x)");
integ!(i_cos_scaled, "cos(2*x)", "x", "sin(2*x)/2");
integ!(i_sin_symbolic_arg, "sin(a*x+b)", "x", "-cos(a*x+b)/a");
integ!(i_exp, "exp(x)", "x", "exp(x)");
integ!(i_exp_scaled, "exp(3*x)", "x", "exp(3*x)/3");
integ!(i_square_of_sum, "(x+1)^2", "x", "x^3/3 + x^2 + x");
integ!(i_inverse_linear, "1/(2*x+1)", "x", "ln(2*x+1)/2");
integ!(i_root_of_linear, "sqrt(x+1)", "x", "2/3*(x+1)^(3/2)");
integ!(i_x_sin, "x*sin(x)", "x", "sin(x) - x*cos(x)");
integ!(i_x_cos, "x*cos(x)", "x", "cos(x) + x*sin(x)");
integ!(i_x_exp, "x*exp(x)", "x", "x*exp(x) - exp(x)");
integ!(i_x_squared_exp, "x^2*exp(x)", "x", "x^2*exp(x) - 2*x*exp(x) + 2*exp(x)");
integ!(i_ln, "ln(x)", "x", "x*ln(x) - x");
integ!(i_beam_moment, "L*x/2 - x^2/2", "x", "L*x^2/4 - x^3/6");
integ!(i_beam_with_ei, "-(w*L*x/2 - w*x^2/2)/(E*I)", "x", "-L*w*x^2/(4*E*I) + w*x^3/(6*E*I)");
integ!(i_other_symbols_constant, "y*sin(z)", "x", "x*y*sin(z)");
integ!(i_sum_of_kinds, "x + sin(x) + exp(x)", "x", "x^2/2 - cos(x) + exp(x)");
integ!(i_pi_coefficient, "pi*x", "x", "pi*x^2/2");
integ!(i_integration_constant_symbol, "C1", "x", "C1*x");

cannot!(c_sin_squared, "sin(x)^2", "x");
cannot!(c_x_ln_x, "x*ln(x)", "x");
cannot!(c_rational_function, "1/(x^2+1)", "x");
cannot!(c_gaussian, "exp(x^2)", "x");
cannot!(c_variable_exponent, "x^x", "x");
cannot!(c_sin_cos_product, "sin(x)*cos(x)", "x");
cannot!(c_sqrt_of_sin, "sqrt(sin(x))", "x");
cannot!(c_exp_of_reciprocal, "exp(1/x)", "x");
cannot!(c_sin_of_square, "sin(x^2)", "x");
cannot!(c_x_over_linear, "x/(x+1)", "x");
cannot!(c_symbolic_exponent, "x^n", "x");
cannot!(c_three_factors, "x*sin(x)*exp(x)", "x");
cannot!(c_ln_of_square, "ln(x^2+1)", "x");

#[test]
fn cannot_integrate_message_says_so() {
    let e = p("1/(x^2+1)").integrate("x").unwrap_err();
    assert!(e.to_string().starts_with("cannot integrate"), "{e}");
}

definite!(def_x_squared, "x^2", "x", "0", "1", "1/3");
definite!(def_sin_zero_to_pi, "sin(x)", "x", "0", "pi", "2");
definite!(def_uniform_load, "w", "x", "0", "L", "w*L");
definite!(def_exp, "exp(x)", "x", "0", "2", "exp(2) - 1");
definite!(def_reciprocal_to_e, "1/x", "x", "1", "e", "1");
definite!(def_cos_quarter, "cos(x)", "x", "0", "pi/2", "1");
definite!(def_symbolic_limits, "x", "x", "a", "b", "b^2/2 - a^2/2");
definite!(def_triangle_load, "x*w/L", "x", "0", "L", "w*L/2");

// ---- substitute ---------------------------------------------------------------------------------
#[test]
fn substitute_number() {
    assert_eq!(p("x^2+x").substitute("x", &p("2")).unwrap(), p("6"));
}
#[test]
fn substitute_expression() {
    let r = p("y^2").substitute("y", &p("x+1")).unwrap().simplify().unwrap();
    assert_eq!(r, s("x^2+2*x+1"));
}
#[test]
fn substitute_into_function() {
    assert!(p("sin(x)").substitute("x", &p("pi")).unwrap().is_zero());
}
#[test]
fn substitute_zero_into_reciprocal_fails() {
    assert_eq!(p("1/x").substitute("x", &p("0")), Err(CasError::DivisionByZero));
}
#[test]
fn substitute_leaves_other_symbols() {
    assert_eq!(p("a*x + b").substitute("x", &p("0")).unwrap(), p("b"));
}
#[test]
fn substitute_all_at_once() {
    let r = p("x + y").substitute_all(&[("x", p("y")), ("y", p("x"))]).unwrap();
    assert_eq!(r, p("x + y"));
    let r = p("x - y").substitute_all(&[("x", p("y")), ("y", p("x"))]).unwrap();
    assert_eq!(r, p("y - x"));
}
#[test]
fn substitute_ln_of_zero_fails() {
    assert!(matches!(p("ln(x)").substitute("x", &p("0")), Err(CasError::Domain(_))));
}

// ---- evaluation ---------------------------------------------------------------------------------
evals!(eval_polynomial, "x^2+1", [("x", 3.0)], 10.0);
evals!(eval_beam_formula, "5*w*L^4/(384*E*I)", [("w", 10.0), ("L", 4.0), ("E", 200.0), ("I", 0.5)], 5.0 * 10.0 * 256.0 / (384.0 * 100.0));
evals!(eval_trig, "sin(pi/2)+cos(0)", [], 2.0);
evals!(eval_exp, "exp(1)", [], std::f64::consts::E);
evals!(eval_sqrt, "sqrt(2)", [], std::f64::consts::SQRT_2);
evals!(eval_ln, "ln(10)", [], std::f64::consts::LN_10);
evals!(eval_rational_power, "x^(2/3)", [("x", 8.0)], 4.0);
evals!(eval_negative_base_integer_power, "x^3", [("x", -2.0)], -8.0);
evals!(eval_reciprocal, "1/x", [("x", 4.0)], 0.25);

#[test]
fn eval_unbound_symbol() {
    assert_eq!(p("x+y").eval(&vars(&[("x", 1.0)])), Err(CasError::UnboundSymbol("y".into())));
}

#[test]
fn symbols_listed() {
    let names: Vec<String> = p("w*L^2/(E*I) + sin(x)").symbols().into_iter().collect();
    assert_eq!(names, vec!["E", "I", "L", "w", "x"]);
}

#[test]
fn has_symbol_looks_inside_functions() {
    assert!(p("sin(x^2)").has_symbol("x"));
    assert!(!p("sin(y)").has_symbol("x"));
}

#[test]
fn constructors_report_errors() {
    assert_eq!(Expr::div(Expr::int(1), Expr::int(0)), Err(CasError::DivisionByZero));
    assert_eq!(Expr::rational(1, 0), Err(caskit::CasError::DivisionByZero));
}
