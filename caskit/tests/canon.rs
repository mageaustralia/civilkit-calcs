//! Canonical form, simplification, parsing and printing.
mod common;
use caskit::{parse, parse_equation, CasError, Expr};
use common::*;

macro_rules! same {
    ($n:ident, $a:expr, $b:expr) => {
        #[test]
        fn $n() {
            assert_eq!(s($a), s($b), "{} vs {}", $a, $b);
        }
    };
}
macro_rules! differ {
    ($n:ident, $a:expr, $b:expr) => {
        #[test]
        fn $n() {
            assert_ne!(s($a), s($b), "{} vs {}", $a, $b);
        }
    };
}
macro_rules! perr {
    ($n:ident, $src:expr, $pat:pat) => {
        #[test]
        fn $n() {
            let r = parse($src);
            assert!(matches!(r, Err($pat)), "{:?} gave {:?}", $src, r);
        }
    };
}
macro_rules! ascii {
    ($n:ident, $src:expr, $out:expr) => {
        #[test]
        fn $n() {
            assert_eq!(p($src).to_ascii(), $out);
        }
    };
}
macro_rules! latex {
    ($n:ident, $src:expr, $out:expr) => {
        #[test]
        fn $n() {
            assert_eq!(p($src).to_latex(), $out);
        }
    };
}

// ---- the key requirement ----------------------------------------------------------------------
same!(
    key_requirement_reduces_to_zero,
    "-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)",
    "0"
);
#[test]
fn key_requirement_is_exactly_zero() {
    assert!(s("-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)").is_zero());
}
#[test]
fn reciprocal_of_product_is_product_of_reciprocals() {
    assert_eq!(p("1/(E*I)"), p("E^-1*I^-1"));
    assert_eq!(p("1/(E*I)").to_ascii(), "1/(E*I)");
}
same!(ei_cancels, "1/(E*I)*E*I", "1");
same!(ei_cancels_in_sum, "w/(E*I) - w/(I*E)", "0");

// ---- like terms, powers -------------------------------------------------------------------------
same!(like_terms, "x + x", "2*x");
same!(like_terms_three, "x + 2*x - 3*x", "0");
same!(x_times_x, "x*x", "x^2");
same!(powers_add, "x^2*x^3", "x^5");
same!(powers_subtract, "x^5/x^2", "x^3");
same!(x_over_x, "x/x", "1");
same!(rational_coefficient, "2*x/4", "x/2");
same!(sum_collects_products, "a*b + b*a", "2*a*b");
same!(sum_of_equal_sums, "3*(x+y) - 2*(x+y)", "x+y");
same!(zero_term, "x - x", "0");
same!(zero_product, "0*x", "0");
same!(power_zero, "x^0", "1");
same!(power_one, "x^1", "x");
same!(one_to_a_power, "1^x", "1");
same!(commutative_sum, "a + b + c", "c + a + b");
same!(commutative_product, "a*b*c", "c*a*b");
same!(nested_sum_flattens, "(a + (b + c))", "a + b + c");
differ!(different_symbols_differ, "x", "y");
differ!(case_matters, "e*x", "E*x");

// ---- expansion ----------------------------------------------------------------------------------
same!(binomial_square, "(x+1)^2", "x^2+2*x+1");
same!(difference_of_squares, "(x+1)*(x-1)", "x^2-1");
same!(binomial_cube, "(a+b)^3", "a^3+3*a^2*b+3*a*b^2+b^3");
same!(distribute_symbolic, "a*(b+c)", "a*b+a*c");
same!(distribute_through_division, "(a+b)/c", "a/c+b/c");
same!(expand_nested, "(x+1)*((x+2)*(x+3))", "x^3+6*x^2+11*x+6");
same!(expand_inside_function, "sin((x+1)^2)", "sin(x^2+2*x+1)");
same!(expand_difference_of_cubes, "(x-1)*(x^2+x+1)", "x^3-1");
same!(expand_trinomial, "(a+b+c)^2", "a^2+b^2+c^2+2*a*b+2*a*c+2*b*c");

// ---- exact numbers ------------------------------------------------------------------------------
same!(decimal_exact, "0.125", "1/8");
same!(decimal_half, "0.5*x", "x/2");
same!(scientific_notation, "1e3", "1000");
same!(scientific_negative, "2.5e-1", "1/4");
same!(int_power, "2^10", "1024");
same!(negative_int_power, "2^-3", "1/8");
same!(fraction_power, "(2/3)^2", "4/9");
same!(negative_base_odd, "(-2)^3", "-8");
same!(negative_base_even, "(-2)^2", "4");
same!(sum_of_fractions, "1/3 + 1/6", "1/2");
same!(third_plus_third, "1/3 + 1/3 + 1/3", "1");
same!(point_one_times_ten, "0.1*10", "1");

// ---- radicals -----------------------------------------------------------------------------------
same!(sqrt_perfect, "sqrt(4)", "2");
same!(sqrt_extract, "sqrt(8)", "2*sqrt(2)");
same!(sqrt_extract_18, "sqrt(18)", "3*sqrt(2)");
same!(sqrt_squared, "sqrt(2)*sqrt(2)", "2");
same!(sqrt_product_of_primes, "sqrt(6)", "sqrt(2)*sqrt(3)");
same!(cube_root, "8^(1/3)", "2");
same!(rational_exponent_perfect, "4^(3/2)", "8");
same!(root_of_fraction, "(1/4)^(1/2)", "1/2");
same!(two_thirds_power, "27^(2/3)", "9");
same!(rationalise, "1/sqrt(2)", "sqrt(2)/2");
same!(sqrt_x_squared, "sqrt(x)*sqrt(x)", "x");
same!(sqrt_of_square, "(x^2)^(1/2)", "x");
same!(sqrt_of_product, "(x*y)^(1/2)", "sqrt(x)*sqrt(y)");
same!(power_of_power, "(x^3)^2", "x^6");
same!(power_of_product, "(2*x)^2", "4*x^2");
same!(fractional_powers_add, "x^(1/2)*x^(1/3)", "x^(5/6)");
same!(sqrt_of_coefficient, "sqrt(4*x)", "2*sqrt(x)");
differ!(sqrt_of_difference_not_collapsed, "sqrt((x-3)^2)", "x-3");

// ---- functions ----------------------------------------------------------------------------------
same!(exp_of_sum, "exp(x)*exp(y)", "exp(x+y)");
same!(exp_zero, "exp(0)", "1");
same!(exp_ln, "exp(ln(x))", "x");
same!(ln_exp, "ln(exp(x))", "x");
same!(ln_one, "ln(1)", "0");
same!(ln_e, "ln(e)", "1");
same!(ln_power, "ln(x^3)", "3*ln(x)");
same!(sin_zero, "sin(0)", "0");
same!(cos_zero, "cos(0)", "1");
same!(sin_odd, "sin(-x)", "-sin(x)");
same!(cos_even, "cos(-x)", "cos(x)");
same!(sin_pi, "sin(pi)", "0");
same!(cos_pi, "cos(pi)", "-1");
same!(sin_half_pi, "sin(pi/2)", "1");
same!(cos_third_pi, "cos(pi/3)", "1/2");
same!(sin_quarter_pi, "sin(pi/4)", "sqrt(2)/2");
same!(sin_sixth_pi, "sin(pi/6)", "1/2");
same!(cos_two_pi, "cos(2*pi)", "1");
same!(sin_three_half_pi, "sin(3*pi/2)", "-1");
same!(sin_seven_sixth_pi, "sin(7*pi/6)", "-1/2");
same!(sin_squared_stays_a_product, "sin(x)*sin(x)", "sin(x)^2");
differ!(sin_cos_identity_not_applied, "sin(x)^2+cos(x)^2", "1");

// ---- parser -------------------------------------------------------------------------------------
same!(precedence_mul_over_add, "1+2*3", "7");
same!(precedence_pow_over_mul, "2*3^2", "18");
same!(pow_right_assoc, "2^3^2", "512");
same!(unary_minus_below_pow, "-2^2", "-4");
same!(unary_minus_in_exponent, "2^-2", "1/4");
same!(left_assoc_division, "8/4/2", "1");
same!(double_negative, "--x", "x");
same!(unary_plus, "+x", "x");
same!(spaces_ignored, "  x  +   y ", "x+y");
perr!(parse_empty, "", CasError::Parse { .. });
perr!(parse_dangling_plus, "1 +", CasError::Parse { .. });
perr!(parse_unclosed_paren, "(1+2", CasError::Parse { .. });
perr!(parse_extra_close, "1+2)", CasError::Parse { .. });
perr!(parse_two_numbers, "2 3", CasError::Parse { .. });
perr!(parse_unknown_function, "foo(x)", CasError::Parse { .. });
perr!(parse_bad_char, "x $ y", CasError::Parse { .. });
perr!(parse_double_dot, "1..2", CasError::Parse { .. });
perr!(parse_implicit_mult_not_supported, "2x", CasError::Parse { .. });
perr!(parse_division_by_zero, "1/0", CasError::DivisionByZero);
perr!(parse_zero_negative_power, "0^-1", CasError::DivisionByZero);
perr!(parse_division_by_cancelled_zero, "1/(x-x)", CasError::DivisionByZero);
perr!(parse_ln_zero, "ln(0)", CasError::Domain(_));
perr!(parse_ln_negative, "ln(-1)", CasError::Domain(_));
perr!(parse_overflow, "2^200", CasError::Overflow);
perr!(
    parse_huge_literal,
    "99999999999999999999999999999999999999999",
    CasError::Overflow
);

#[test]
fn equation_parsing() {
    let eq = parse_equation("M = w*L^2/8").unwrap();
    assert_eq!(eq.lhs, p("M"));
    assert_eq!(eq.rhs, p("w*L^2/8"));
    assert!(matches!(parse_equation("x + 1"), Err(CasError::Parse { .. })));
    assert!(matches!(parse_equation("a = b = c"), Err(CasError::Parse { .. })));
}

#[test]
fn constants_are_lower_case_only() {
    assert_eq!(p("e"), Expr::e());
    assert_eq!(p("pi"), Expr::pi());
    assert_eq!(p("E"), Expr::sym("E"));
    assert_eq!(p("Pi"), Expr::sym("Pi"));
}

// ---- printing -----------------------------------------------------------------------------------
ascii!(print_polynomial, "x^2 - 5*x + 6", "x^2 - 5*x + 6");
ascii!(print_polynomial_reordered, "6 - 5*x + x^2", "x^2 - 5*x + 6");
ascii!(print_fraction, "w*L^2/8", "L^2*w/8");
ascii!(print_reciprocal, "1/(E*I)", "1/(E*I)");
ascii!(print_negative, "-x", "-x");
ascii!(print_difference, "x-y", "x - y");
ascii!(print_sqrt, "2*sqrt(x)", "2*sqrt(x)");
ascii!(print_exp, "exp(x)", "exp(x)");
ascii!(print_pi_r_squared, "pi*r^2", "pi*r^2");
ascii!(print_power_of_function, "sin(x)^2", "sin(x)^2");
ascii!(print_negative_exponent_in_denominator, "x/y^2", "x/y^2");
ascii!(print_fractional_exponent, "x^(1/3)", "x^(1/3)");
ascii!(print_sum_in_denominator, "1/(a+b)", "1/(a + b)");
ascii!(print_sum_factor, "2*(a+b)", "2*(a + b)");
ascii!(print_negative_fraction_coefficient, "-3*x/4", "-3*x/4");
ascii!(print_sqrt_in_denominator, "x/sqrt(y)", "x/sqrt(y)");

latex!(latex_fraction, "w*L^2/8", "\\frac{L^{2} w}{8}");
latex!(latex_sqrt, "sqrt(x)", "\\sqrt{x}");
latex!(latex_cube_root, "x^(1/3)", "\\sqrt[3]{x}");
latex!(latex_sin, "sin(x)", "\\sin\\left(x\\right)");
latex!(latex_pi, "pi", "\\pi");
latex!(latex_greek, "alpha*x", "\\alpha x");
latex!(latex_subscript, "M_max", "M_{max}");
latex!(latex_trailing_digits, "C1", "C_{1}");
latex!(latex_negative, "-x", "-x");
latex!(latex_exp, "exp(2*x)", "e^{2 x}");
latex!(latex_two_numbers_need_a_dot, "2*sqrt(3)", "2 \\sqrt{3}");

#[test]
fn ascii_round_trip() {
    for src in [
        "w*L^2/8",
        "x^2 - 5*x + 6",
        "sin(x)*x",
        "1/(E*I)",
        "sqrt(8)",
        "x^(1/3)*y^-2",
        "exp(2*x)/3",
        "(x+1)^2*(x+1)",
        "-3*x/4 + 2*sqrt(x)",
        "(a+b)^(-1/2)",
        "sin(x+1)^3/(cos(x) + 2)",
        "pi*r^2/4",
        "x^y",
        "2^(1/3)*x",
        "ln(x)/x^2",
    ] {
        let e = p(src);
        assert_eq!(p(&e.to_ascii()), e, "{src} printed as {}", e.to_ascii());
    }
}

#[test]
fn display_matches_ascii() {
    let e = p("w*L^2/8");
    assert_eq!(format!("{e}"), e.to_ascii());
}

#[test]
fn expand_is_idempotent() {
    for src in [
        "(x+1)^3*(y-2)",
        "(a+b)/(c+d)",
        "sin((x+1)^2)*(x-1)",
        "1/(x+1)^2*(x+1)",
        "sqrt(x+1)*(x+2)",
    ] {
        let once = p(src).expand().unwrap();
        assert_eq!(once.expand().unwrap(), once, "{src}");
    }
}

#[test]
fn expansion_limit_is_an_error() {
    assert_eq!(p("(x+1)^100").expand(), Err(CasError::TooLarge));
    assert_eq!(p("(a+b+c+d+e+f+g+h)^40").expand(), Err(CasError::TooLarge));
}
