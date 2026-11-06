//! Transcribed from Calcpad.Tests/Scalars/FunctionTests.cs by tools/transcribe_xunit.py.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unnecessary_cast,
    clippy::eq_op,
    clippy::identity_op,
    clippy::needless_borrow,
    clippy::redundant_clone,
    clippy::bool_assert_comparison,
    clippy::useless_format,
    clippy::clone_on_copy,
    unused_imports,
    dead_code,
    suspicious_double_ref_op,
    noop_method_call,
    clippy::needless_borrows_for_generic_args
)]
mod common;
use common::*;

const Tol: f64 = 1e-14;
const E: f64 = std::f64::consts::E;
const expressions: [&str; 2] = ["ReturnAngleUnits = 1", "asin(sqrt(3)/2)"];
fn _sqrt2() -> f64 {
    Math::Sqrt(2.0)
}
fn _sqrt3() -> f64 {
    Math::Sqrt(3.0)
}

#[test]
fn Sin() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("sin(pi/2)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Cos() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("cos(-3*pi/4)");
    assert_equal_tol(-_sqrt2() / 2.0, result, Tol);
}
#[test]
fn Tan() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("Tan(pi/4)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Tan_Pi_Div_2() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("Tan(pi/2)");
    assert_equal_tol(16331239353195370.0, result, Tol);
}
#[test]
fn Csc() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("Csc(π/3)");
    assert_equal_tol(2.0 / _sqrt3(), result, Tol);
}
#[test]
fn Sec() {
    let mut result = TestCalc::new(ms().Degrees(1.0)).Run("SEC(-30°)");
    assert_equal_tol(2.0 / _sqrt3(), result, Tol);
}
#[test]
fn Cot_90_deg() {
    let mut result = TestCalc::new(ms()).Run("Cot(90)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Cot_30_deg() {
    let mut result = TestCalc::new(ms()).Run("Cot(30)");
    assert_equal_tol(_sqrt3(), result, Tol);
}
#[test]
fn Csc_0() {
    let mut result = TestCalc::new(ms()).Run("csc(0)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Sec_0() {
    let mut result = TestCalc::new(ms()).Run("sec(0)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Cot_Minus_0() {
    let mut result = TestCalc::new(ms()).Run("Cot(-0)");
    assert_equal_tol(f64::NEG_INFINITY, result, Tol);
}
#[test]
fn Sinh_1() {
    let mut result = TestCalc::new(ms()).Run("sinh(1)");
    let mut expected = (E * E - 1.0) / (2.0 * E);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Cosh_1() {
    let mut result = TestCalc::new(ms()).Run("cosh(1)");
    let mut expected = (E * E + 1.0) / (2.0 * E);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Tanh_1() {
    let mut result = TestCalc::new(ms()).Run("Tanh(1)");
    let mut expected = (E * E - 1.0) / (E * E + 1.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Csch_1() {
    let mut result = TestCalc::new(ms()).Run("Csch(1)");
    let mut expected = 2.0 * E / (E * E - 1.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Sech_1() {
    let mut result = TestCalc::new(ms()).Run("sech(1)");
    let mut expected = 2.0 * E / (E * E + 1.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Coth_1() {
    let mut result = TestCalc::new(ms()).Run("Coth(1)");
    let mut expected = (E * E + 1.0) / (E * E - 1.0);
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Sinh_0() {
    let mut result = TestCalc::new(ms()).Run("sinh(0)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Cosh_0() {
    let mut result = TestCalc::new(ms()).Run("cosh(0)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Tanh_0() {
    let mut result = TestCalc::new(ms()).Run("Tanh(0)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Csch_0() {
    let mut result = TestCalc::new(ms()).Run("Csc(0)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Sech_0() {
    let mut result = TestCalc::new(ms()).Run("sech(0)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Coth_0() {
    let mut result = TestCalc::new(ms()).Run("Coth(0)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Asin() {
    let mut calc = TestCalc::new(ms());
    calc.Run(expressions);
    assert_equal_str("60°", &calc.ToString());
}
#[test]
fn Acos() {
    let mut result = TestCalc::new(ms()).Run("acos(-1/2)");
    assert_equal_tol(120.0, result, 10.0 * Tol);
}
#[test]
fn Atan() {
    let mut result = TestCalc::new(ms()).Run("atan(1)");
    assert_equal_tol(45.0, result, Tol);
}
#[test]
fn Acsc() {
    let mut result = TestCalc::new(ms()).Run("acsc(2)");
    assert_equal_tol(30.0, result, Tol);
}
#[test]
fn Acec() {
    let mut result = TestCalc::new(ms()).Run("asec(sqrt(2))");
    assert_equal_tol(45.0, result, Tol);
}
#[test]
fn Acot() {
    let mut result = TestCalc::new(ms().IsComplex(true)).Run("acot(0)");
    assert_equal_tol(90.0, result, Tol);
}
#[test]
fn Asinh_1() {
    let mut result = TestCalc::new(ms()).Run("asinh(1)");
    let mut expected = Math::Log(1.0 + _sqrt2());
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Acosh_1() {
    let mut result = TestCalc::new(ms()).Run("acosh(1)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Acosh_2() {
    let mut result = TestCalc::new(ms()).Run("acosh(2)");
    let mut expected = Math::Log(2.0 + _sqrt3());
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Atanh_1() {
    let mut result = TestCalc::new(ms()).Run("atanh(1)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Atanh_05() {
    let mut result = TestCalc::new(ms()).Run("atanh(0.5)");
    let mut expected = Math::Log(3.0) / 2.0;
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Acsch_1() {
    let mut result = TestCalc::new(ms()).Run("acsch(1)");
    let mut expected = Math::Log(1.0 + _sqrt2());
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Asech_1() {
    let mut result = TestCalc::new(ms()).Run("asech(1)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Asech_05() {
    let mut result = TestCalc::new(ms()).Run("asech(0.5)");
    let mut expected = Math::Log(2.0 + _sqrt3());
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Acoth_1() {
    let mut result = TestCalc::new(ms()).Run("acoth(1)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Acoth_2() {
    let mut result = TestCalc::new(ms()).Run("acoth(2)");
    let mut expected = Math::Log(3.0) / 2.0;
    assert_equal_tol(expected, result, Tol);
}
#[test]
fn Asinh_0() {
    let mut result = TestCalc::new(ms()).Run("asinh(0)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Acosh_0() {
    let mut result = TestCalc::new(ms()).Run("acosh(0)");
    assert_equal_tol(f64::NAN, result, Tol);
}
#[test]
fn Atanh_0() {
    let mut result = TestCalc::new(ms()).Run("atanh(0)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Acsch_0() {
    let mut result = TestCalc::new(ms()).Run("acsch(0)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Asech_0() {
    let mut result = TestCalc::new(ms()).Run("asech(0)");
    assert_equal_tol(f64::INFINITY, result, Tol);
}
#[test]
fn Acoth_0() {
    let mut result = TestCalc::new(ms()).Run("acoth(0)");
    assert_equal_tol(f64::NAN, result, Tol);
}
#[test]
fn Log_1000() {
    let mut result = TestCalc::new(ms()).Run("log(1000)");
    assert_equal_tol(3.0, result, Tol);
}
#[test]
fn Ln_e_Pow_3() {
    let mut result = TestCalc::new(ms()).Run("ln(e^3)");
    assert_equal_tol(3.0, result, Tol);
}
#[test]
fn Ln_Minus_One() {
    let mut result = TestCalc::new(ms()).Run("ln(-1)");
    assert_equal_tol(f64::NAN, result, Tol);
}
#[test]
fn Log2_1048576() {
    let mut result = TestCalc::new(ms()).Run("log_2(1048576)");
    assert_equal_tol(20.0, result, Tol);
}
#[test]
fn Sqrt_25() {
    let mut result = TestCalc::new(ms()).Run("sqrt(25)");
    assert_equal_tol(5.0, result, Tol);
}
#[test]
fn Sqrt_Minus_4() {
    let mut result = TestCalc::new(ms()).Run("sqrt(-4)");
    assert_equal_tol(f64::NAN, result, Tol);
}
#[test]
fn Cbrt_64() {
    let mut result = TestCalc::new(ms()).Run("cbrt(64)");
    assert_equal_tol(4.0, result, Tol);
}
#[test]
fn Cbrt_Minus_8() {
    let mut result = TestCalc::new(ms()).Run("cbrt(-8)");
    assert_equal_tol(-2.0, result, Tol);
}
#[test]
fn Root_81_4() {
    let mut result = TestCalc::new(ms()).Run("root(81; 4)");
    assert_equal_tol(3.0, result, Tol);
}
#[test]
fn Root_Minus_16_4() {
    let mut result = TestCalc::new(ms()).Run("root(-16; 4)");
    assert_equal_tol(f64::NAN, result, Tol);
}
#[test]
fn Root_Minus_32_5() {
    let mut result = TestCalc::new(ms()).Run("root(-32; 5)");
    assert_equal_tol(-2.0, result, Tol);
}
#[test]
fn Round_Pos() {
    let mut result = TestCalc::new(ms()).Run("round(4.5)");
    assert_equal_tol(5.0, result, Tol);
}
#[test]
fn Round_Neg() {
    let mut result = TestCalc::new(ms()).Run("round(-4.5)");
    assert_equal_tol(-5.0, result, Tol);
}
#[test]
fn Floor_Pos() {
    let mut result = TestCalc::new(ms()).Run("floor(4.8)");
    assert_equal_tol(4.0, result, Tol);
}
#[test]
fn Floor_Neg() {
    let mut result = TestCalc::new(ms()).Run("floor(-4.8)");
    assert_equal_tol(-5.0, result, Tol);
}
#[test]
fn Ceiling_Pos() {
    let mut result = TestCalc::new(ms()).Run("ceiling(4.2)");
    assert_equal_tol(5.0, result, Tol);
}
#[test]
fn Ceiling_Neg() {
    let mut result = TestCalc::new(ms()).Run("ceiling(-4.2)");
    assert_equal_tol(-4.0, result, Tol);
}
#[test]
fn TruncPos() {
    let mut result = TestCalc::new(ms()).Run("trunc(4.8)");
    assert_equal_tol(4.0, result, Tol);
}
#[test]
fn TruncNeg() {
    let mut result = TestCalc::new(ms()).Run("trunc(-4.8)");
    assert_equal_tol(-4.0, result, Tol);
}
#[test]
fn Mod() {
    let mut result = TestCalc::new(ms()).Run("mod(7; 5)");
    assert_equal_tol(2.0, result, Tol);
}
#[test]
fn Gcd() {
    let mut result = TestCalc::new(ms()).Run("gcd(18; 24)");
    assert_equal_tol(6.0, result, Tol);
}
#[test]
fn Lcm() {
    let mut result = TestCalc::new(ms()).Run("lcm(6; 4)");
    assert_equal_tol(12.0, result, Tol);
}
#[test]
fn Fact() {
    let mut result = TestCalc::new(ms()).Run("fact(5)");
    assert_equal_tol(120.0, result, Tol);
}
#[test]
fn Abs() {
    let mut result = TestCalc::new(ms()).Run("abs(-2)");
    assert_equal(2.0, result.clone());
}
#[test]
fn Re() {
    let mut result = TestCalc::new(ms()).Run("re(1)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Im() {
    let mut result = TestCalc::new(ms()).Run("im(1)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Phase_1() {
    let mut result = TestCalc::new(ms()).Run("phase(1)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Phase_Minus_1() {
    let mut result = TestCalc::new(ms()).Run("phase(-1)");
    assert_equal_tol(180.0, result, Tol);
}
#[test]
fn Min() {
    let mut result = TestCalc::new(ms()).Run("min(1; 2; -3; 0)");
    assert_equal(-3.0, result.clone());
}
#[test]
fn Max() {
    let mut result = TestCalc::new(ms()).Run("max(1; 2; -3; 0)");
    assert_equal(2.0, result.clone());
}
#[test]
fn Sum() {
    let mut result = TestCalc::new(ms()).Run("sum(1; 2; 3; 4)");
    assert_equal_tol(10.0, result, Tol);
}
#[test]
fn SumSq() {
    let mut result = TestCalc::new(ms()).Run("sumsq(3; 4)");
    assert_equal_tol(25.0, result, Tol);
}
#[test]
fn Srss() {
    let mut result = TestCalc::new(ms()).Run("srss(3; 4)");
    assert_equal_tol(5.0, result, Tol);
}
#[test]
fn Average() {
    let mut result = TestCalc::new(ms()).Run("average(2; 4; 6; 8)");
    assert_equal_tol(5.0, result, Tol);
}
#[test]
fn Product() {
    let mut result = TestCalc::new(ms()).Run("product(1; 2; 3; 4; 5)");
    assert_equal_tol(120.0, result, Tol);
}
#[test]
fn Mean() {
    let mut result = TestCalc::new(ms()).Run("mean(2; 4; 8; 16; 32)");
    assert_equal_tol(8.0, result, Tol);
}
#[test]
fn Take() {
    let mut result = TestCalc::new(ms()).Run("take(2.5; 1; 2; 3)");
    assert_equal_tol(3.0, result, Tol);
}
#[test]
fn Line() {
    let mut result = TestCalc::new(ms()).Run("line(1.5; 1; 2; 3)");
    assert_equal_tol(1.5, result, Tol);
}
#[test]
fn Spline() {
    let mut result = TestCalc::new(ms()).Run("spline(1.5; 2; 4; 8)");
    assert_equal_tol(2.8125, result, Tol);
}
#[test]
fn If() {
    let mut result = TestCalc::new(ms()).Run("if(1; 2; 3)");
    assert_equal(2.0, result.clone());
}
#[test]
fn Switch1() {
    let mut result = TestCalc::new(ms()).Run("switch(0)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Switch2() {
    let mut result = TestCalc::new(ms()).Run("switch(1; 2)");
    assert_equal(2.0, result.clone());
}
#[test]
fn Switch() {
    let mut result = TestCalc::new(ms()).Run("switch(0; 1; 0; 2; 1; 3; 4)");
    assert_equal(3.0, result.clone());
}
#[test]
fn Not_0() {
    let mut result = TestCalc::new(ms()).Run("not(0)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Not_1() {
    let mut result = TestCalc::new(ms()).Run("not(1)");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_0_0() {
    let mut result = TestCalc::new(ms()).Run("and(0; 0)");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_0_1() {
    let mut result = TestCalc::new(ms()).Run("and(0; 1)");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_1_0() {
    let mut result = TestCalc::new(ms()).Run("and(1; 0)");
    assert_equal(0.0, result.clone());
}
#[test]
fn And_1_1() {
    let mut result = TestCalc::new(ms()).Run("and(1; 1)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_0_0() {
    let mut result = TestCalc::new(ms()).Run("or(0; 0)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Or_0_1() {
    let mut result = TestCalc::new(ms()).Run("or(0; 1)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_1_0() {
    let mut result = TestCalc::new(ms()).Run("or(1; 0)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Or_1_1() {
    let mut result = TestCalc::new(ms()).Run("or(1; 1)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_0_0() {
    let mut result = TestCalc::new(ms()).Run("xor(0; 0)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Xor_0_1() {
    let mut result = TestCalc::new(ms()).Run("xor(0; 1)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_1_0() {
    let mut result = TestCalc::new(ms()).Run("xor(1; 0)");
    assert_equal(1.0, result.clone());
}
#[test]
fn Xor_1_1() {
    let mut result = TestCalc::new(ms()).Run("xor(1; 1)");
    assert_equal(0.0, result.clone());
}
#[test]
fn Sign_Neg() {
    let mut result = TestCalc::new(ms()).Run("sign(-10)");
    assert_equal_tol(-1.0, result, Tol);
}
#[test]
fn Sign_Zero() {
    let mut result = TestCalc::new(ms()).Run("sign(0)");
    assert_equal_tol(0.0, result, Tol);
}
#[test]
fn Sign_Pos() {
    let mut result = TestCalc::new(ms()).Run("sign(10)");
    assert_equal_tol(1.0, result, Tol);
}
#[test]
fn Random() {
    let mut result = TestCalc::new(ms()).Run("random(1)");
    assert_in_range(result, 0.0, 1.0);
}
