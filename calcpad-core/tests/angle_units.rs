//! Ported from the upstream "Angle Units" tests,
//! adapted to ReturnAngleUnits being off by default: tests that expect an angle unit in the result
//! switch it on with `ReturnAngleUnits = 1`.
#![allow(non_snake_case)]
mod common;
use common::*;

const RAU: &str = "ReturnAngleUnits = 1";

fn RunAngle(settings: calcpad_core::MathSettings, lines: &[&str]) -> String {
    let mut calc = TestCalc::new(settings);
    calc.RunAll(lines);
    calc.ToString()
}

#[test]
fn Phase_Matches_Atan2() {
    for (degrees, expected) in [(0.0, "45°"), (1.0, "0.785rad"), (2.0, "50grad")] {
        let settings = || ms().Degrees(degrees).IsComplex(true);
        assert_eq!(expected, RunAngle(settings(), &[RAU, "phase(1 + 1i)"]));
        assert_eq!(expected, RunAngle(settings(), &[RAU, "atan2(1; 1)"]));
    }
}

#[test]
fn Phase_Real_Deg() {
    assert_eq!("180°", RunAngle(ms(), &[RAU, "phase(-1)"]));
}

#[test]
fn Phase_ReturnAngleUnits_Off() {
    assert_eq!("45", RunAngle(ms().IsComplex(true), &["ReturnAngleUnits = 0", "phase(1 + 1i)"]));
    assert_eq!("45", RunAngle(ms().IsComplex(true), &["phase(1 + 1i)"]));
}

#[test]
fn Phasor_Unitless_Angle_Uses_Current_Units() {
    assert_eq!("0.707", RunAngle(ms().IsComplex(true), &["re(1∠45)"]));
    assert_eq!("0.707", RunAngle(ms().IsComplex(true), &["re(1∠(π/4*rad))"]));
    assert_eq!("0.707", RunAngle(ms().IsComplex(true).Degrees(1.0), &["re(1∠(π/4))"]));
}

#[test]
fn Phasor_In_Compiled_Function() {
    assert_eq!("2i", RunAngle(ms().IsComplex(true), &["f(r; t) = r∠t", "f(2; 90)"]));
    assert_eq!("5.2", RunAngle(ms().IsComplex(true), &["$Sum{re(k∠30) @ k = 1 : 3}"]));
    assert_eq!("1.5", RunAngle(ms().IsComplex(true), &["$Repeat{re(k∠60) @ k = 1 : 3}"]));
}

#[test]
fn Acsc_Zero_Keeps_Units() {
    assert_eq!("[+∞°\u{2002}30°]", RunAngle(ms(), &[RAU, "acsc([0; 2])"]));
}

#[test]
fn HpVector_Sparse_Asin() {
    assert_eq!(
        "[30°\u{2002}0°\u{2002}0°\u{2002}0°]",
        RunAngle(ms(), &[RAU, "v = vector(4)", "v.1 = 0.5", "asin(hp(v))"])
    );
}

#[test]
fn HpVector_Asec_Zero() {
    assert_eq!("[+∞°\u{2002}60°]", RunAngle(ms(), &[RAU, "asec(hp([0; 2]))"]));
}

#[test]
fn HpDiagonal_InverseTrig_Uniform_Units() {
    for expression in ["atan(hp(D)) + 1°", "atan2(hp(D); hp(D)) + 1°"] {
        let result = RunAngle(ms(), &[RAU, "D = diagonal(2; 1)", expression]);
        assert!(!result.contains("unitless"), "{expression}: {result}");
        assert_eq!(4, result.split('°').count() - 1, "{expression}: {result}");
    }
}
