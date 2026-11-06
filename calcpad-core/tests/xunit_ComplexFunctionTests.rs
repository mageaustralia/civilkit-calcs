//! Transcribed from Calcpad.Tests/Scalars/ComplexFunctionTests.cs by tools/transcribe_xunit.py.
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
fn _sqrt2() -> f64 {
    Math::Sqrt(2.0)
}
fn _sqrt3() -> f64 {
    Math::Sqrt(3.0)
}

#[test]
fn Sin() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("sin(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::Sinh(1.0), im, Tol);
}
#[test]
fn Cos() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("cos(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::Cosh(1.0), re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Tan() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("tan(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::Tanh(1.0), im, Tol);
}
#[test]
fn Csc() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("csc(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-1.0 / Math::Sinh(1.0), im, Tol);
}
#[test]
fn Sec() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("sec(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0 / Math::Cosh(1.0), re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Cot() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("cot(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-1.0 / Math::Tanh(1.0), im, Tol);
}
#[test]
fn Sinh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("sinh(πi/2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(1.0, im, Tol);
}
#[test]
fn Cosh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("cosh(-3*πi/4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(-_sqrt2() / 2.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Tanh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("Tanh(πi/4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(1.0, im, Tol);
}
#[test]
fn Csch() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("Csch(πi/3)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-2.0 / _sqrt3(), im, Tol);
}
#[test]
fn Sech() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("sech(-πi/6)");
    let mut im = calc.Imaginary();
    assert_equal_tol(2.0 / _sqrt3(), re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Coth() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("Coth(πi/4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-1.0, im, Tol);
}
#[test]
fn Asin() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("asin(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::Log(_sqrt2() + 1.0), im, Tol);
}
#[test]
fn Acos() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acos(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::PI / 2.0, re, Tol);
    assert_equal_tol(Math::Log(_sqrt2() - 1.0), im, Tol);
}
#[test]
fn Atan() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("atan(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal(f64::INFINITY, im.clone());
}
#[test]
fn Atan_I_div_2() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("atan(1i/2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal(Math::Log(3.0) / 2.0, im.clone());
}
#[test]
fn Acsc() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acsc(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-Math::Log(_sqrt2() + 1.0), im, Tol);
}
#[test]
fn Acec() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("asec(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(Math::PI / 2.0, re, Tol);
    assert_equal_tol(Math::Log(_sqrt2() + 1.0), im, Tol);
}
#[test]
fn Acot_i() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acot(1i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(f64::NEG_INFINITY, im, Tol);
}
#[test]
fn Acot_2i() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acot(2i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-Math::Log(3.0) / 2.0, im, Tol);
}
#[test]
fn Asinh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("asinh(i*sqrt(3)/2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 3.0, im, Tol);
}
#[test]
fn Acosh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acosh(sqrt(2)/2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 4.0, im, Tol);
}
#[test]
fn Atanh() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("atanh(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 4.0, im, Tol);
}
#[test]
fn Acsch() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acsch(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-Math::PI / 2.0, im, Tol);
}
#[test]
fn Asech() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("asech(sqrt(2))");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 4.0, im, Tol);
}
#[test]
fn Acoth() {
    let mut calc = TestCalc::new(ms().Degrees(1.0).IsComplex(true));
    let mut re = calc.Run("acoth(0)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 2.0, im, Tol);
}
#[test]
fn Ln_Minus_1() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("ln(-1)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI, im, Tol);
}
#[test]
fn Ln_I() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("ln(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / 2.0, im, Tol);
}
#[test]
fn Ln_Minus_e_Pow_3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("ln(-e^3)");
    let mut im = calc.Imaginary();
    assert_equal_tol(3.0, re, Tol);
    assert_equal_tol(Math::PI, im, Tol);
}
#[test]
fn Log_I() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("log(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(Math::PI / (2.0 * Math::Log(10.0)), im, Tol);
}
#[test]
fn Log2_Minus_2() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("log_2(-2)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(Math::PI / Math::Log(2.0), im, Tol);
}
#[test]
fn Exp_Minus_PI_I() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("-exp(πi)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Sqrt_I() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sqrt(i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0 / _sqrt2(), re, Tol);
    assert_equal_tol(1.0 / _sqrt2(), im, Tol);
}
#[test]
fn Sqrt_Minus_4() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sqrt(-4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(2.0, im, Tol);
}
#[test]
fn Sqrt_Minus_2_Plus_2i_Sqrt_3() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sqrt(-2 + 2*sqrt(3)*i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(_sqrt3(), im, Tol);
}
#[test]
fn Cbrt_Minus_8() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("cbrt(-8)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.0, re, Tol);
    assert_equal_tol(_sqrt3(), im, Tol);
}
#[test]
fn Root_Minus_16() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("root(-16; 4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(_sqrt2(), re, Tol);
    assert_equal_tol(_sqrt2(), im, Tol);
}
#[test]
fn Root_Minus_32_5() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("root(-32; 5)");
    let mut im = calc.Imaginary();
    let mut sqrt5 = Math::Sqrt(5.0);
    assert_equal_tol((1.0 + sqrt5) / 2.0, re, Tol);
    assert_equal_tol(Math::Sqrt((5.0 - sqrt5) / 2.0), im, Tol);
}
#[test]
fn Round_Pos() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("round(3.5 + 4.5i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(4.0, re, Tol);
    assert_equal_tol(5.0, im, Tol);
}
#[test]
fn Round_Neg() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("round(-4.5i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-5.0, im, Tol);
}
#[test]
fn Floor_Pos() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("floor(4.8i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(4.0, im, Tol);
}
#[test]
fn Floor_Neg() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("floor(-2.4 - 4.8i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(-3.0, re, Tol);
    assert_equal_tol(-5.0, im, Tol);
}
#[test]
fn Ceiling_Pos() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("ceiling(4.2i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(5.0, im, Tol);
}
#[test]
fn Ceiling_Neg() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("ceiling(-4.2i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-4.0, im, Tol);
}
#[test]
fn Trunc_Pos() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("trunc(2.4 + 4.8i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(2.0, re, Tol);
    assert_equal_tol(4.0, im, Tol);
}
#[test]
fn Trunc_Neg() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("trunc(-4.8i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(-4.0, im, Tol);
}
#[test]
fn Mod() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("mod(7i; 5i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Gcd() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("gcd(18i; 24i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Lcm() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("lcm(6i; 4i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Fact() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut e = assert_throws(|| {
        calc.Run("fact(5i)");
    });
    assert_contains("The argument of n! cannot be complex.", &e.message);
}
#[test]
fn Abs() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("abs((3 + 4i))");
    let mut im = calc.Imaginary();
    assert_equal_tol(5.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Re() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("re(3 + 4i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(3.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Im() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("im(3 + 4i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(4.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Phase() {
    // C#: calc.Run($"phase({_sqrt3.ToString(CultureInfo.InvariantCulture)}/2 + i/2)")
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run(format!(
        "phase({}/2 + i/2)",
        calcpad_core::netfmt::double_to_string(_sqrt3())
    ));
    let mut im = calc.Imaginary();
    assert_equal_tol(30.0, re, 1e-12);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Min() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("min(1i; 2i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Max() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("max(1i; 2i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Sum() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sum(1i; 2i; 3; 4)");
    let mut im = calc.Imaginary();
    assert_equal_tol(7.0, re, Tol);
    assert_equal_tol(3.0, im, Tol);
}
#[test]
fn SumSq() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sumsq(3i; 4i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(-25.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Srss() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("srss(3i; 4i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(5.0, im, Tol);
}
#[test]
fn Average() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("average(2i; 4i; 6i; 8i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(5.0, im, Tol);
}
#[test]
fn Product() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("product(1i; 2i; 3i; 4i; 5i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(0.0, re, Tol);
    assert_equal_tol(120.0, im, Tol);
}
#[test]
fn Mean() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("mean(2i; 4i; 8i; 16i; 32)");
    let mut im = calc.Imaginary();
    assert_equal_tol(8.0, re, Tol);
    assert_equal_tol(0.0, im, Tol);
}
#[test]
fn Take() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("take(2.5; 1i; 2i; 3i)");
    let mut im = calc.Imaginary();
    assert_equal(0.0, re.clone());
    assert_equal(3.0, im.clone());
}
#[test]
fn Line() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("line(1.5 + 2.5i; 1 + 4i; 2 + 5i; 3 + 6i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(1.5, re, Tol);
    assert_equal(4.5, im.clone());
}
#[test]
fn Spline() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("spline(1.5 + 2.5i; 2 + 16i; 4 + 32i; 8 + 64i)");
    let mut im = calc.Imaginary();
    assert_equal_tol(2.8125, re, Tol);
    assert_equal(0.0, im.clone());
}
#[test]
fn If() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("if(1; 1 + 2i; 3i)");
    let mut im = calc.Imaginary();
    assert_equal(1.0, re.clone());
    assert_equal(2.0, im.clone());
}
#[test]
fn Switch() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    calc.Run("x = 10");
    let mut re = calc.Run("switch(x ≤ 1; 1i; x ≤ 2; 2i; x ≤ 3; 3i; 4i)");
    let mut im = calc.Imaginary();
    assert_equal(0.0, re.clone());
    assert_equal(4.0, im.clone());
}
#[test]
fn Sign_Neg() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sign(-10i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Sign_Zero() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sign(0)");
    assert_equal_tol(0.0, re, Tol);
}
#[test]
fn Sign_Pos() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("sign(10i)");
    assert_equal(f64::NAN, re.clone());
}
#[test]
fn Random() {
    let mut calc = TestCalc::new(ms().IsComplex(true));
    let mut re = calc.Run("random(1)");
    let mut im = calc.Imaginary();
    assert_in_range(re, 0.0, 1.0);
    assert_in_range(im, 0.0, 1.0);
}
