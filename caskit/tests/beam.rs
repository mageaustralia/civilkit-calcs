//! The beam derivations from the specification, done end to end with the public API.
//!
//! Sign convention: `y` is the deflection measured in the direction of the load (downwards for a
//! gravity load), so the bending moment `M` (sagging positive) gives `E*I*y'' = -M`. With this convention
//! the mid-span deflection of the simply supported beam under a uniform load comes out positive.

use caskit::{parse, solve_linear_system, Equation, Expr};

fn p(s: &str) -> Expr {
    parse(s).unwrap()
}

#[test]
fn simply_supported_uniform_load() {
    // E*I*y'' = -(w*L*x/2 - w*x^2/2)
    let moment = p("w*L*x/2 - w*x^2/2");
    let ypp = Expr::div(Expr::neg(moment).unwrap(), p("E*I")).unwrap();

    let yp = Expr::add(ypp.integrate("x").unwrap(), p("C1")).unwrap();
    let y = Expr::add(yp.integrate("x").unwrap(), p("C2")).unwrap();
    println!("y' = {yp}");
    println!("y  = {y}");

    let at0 = y.substitute("x", &p("0")).unwrap();
    let at_l = y.substitute("x", &p("L")).unwrap();
    let sol = solve_linear_system(
        &[Equation::new(at0, p("0")), Equation::new(at_l, p("0"))],
        &["C1", "C2"],
    )
    .unwrap();
    println!("C1 = {}", sol[0]);
    println!("C2 = {}", sol[1]);
    assert_eq!(sol[0], p("w*L^3/(24*E*I)"));
    assert_eq!(sol[1], p("0"));

    let ys = y
        .substitute_all(&[("C1", sol[0].clone()), ("C2", sol[1].clone())])
        .unwrap()
        .simplify()
        .unwrap();
    println!("y(x) = {ys}");
    let expected_y = p("w*L^3*x/(24*E*I) - w*L*x^3/(12*E*I) + w*x^4/(24*E*I)").simplify().unwrap();
    assert_eq!(ys, expected_y);

    // boundary conditions hold
    assert!(ys.substitute("x", &p("0")).unwrap().simplify().unwrap().is_zero());
    assert!(ys.substitute("x", &p("L")).unwrap().simplify().unwrap().is_zero());

    // mid-span deflection, exactly
    let mid = ys.substitute("x", &p("L/2")).unwrap().simplify().unwrap();
    println!("y(L/2) = {mid}");
    assert_eq!(mid, p("5*w*L^4/(384*E*I)").simplify().unwrap());

    // y'' reproduces the input
    let back = ys.diff_n("x", 2).unwrap();
    let residual = Expr::sub(back, ypp).unwrap().simplify().unwrap();
    assert!(residual.is_zero(), "residual {residual}");
}

#[test]
fn cantilever_tip_load() {
    // y'' = P*(L - x)/(E*I), y(0) = 0, y'(0) = 0
    let ypp = p("P*(L - x)/(E*I)");
    let yp = Expr::add(ypp.integrate("x").unwrap(), p("C1")).unwrap();
    let y = Expr::add(yp.integrate("x").unwrap(), p("C2")).unwrap();
    let sol = solve_linear_system(
        &[
            Equation::new(yp.substitute("x", &p("0")).unwrap(), p("0")),
            Equation::new(y.substitute("x", &p("0")).unwrap(), p("0")),
        ],
        &["C1", "C2"],
    )
    .unwrap();
    assert!(sol[0].is_zero() && sol[1].is_zero());
    let ys = y.substitute_all(&[("C1", sol[0].clone()), ("C2", sol[1].clone())]).unwrap().simplify().unwrap();
    println!("y(x) = {ys}");
    let tip = ys.substitute("x", &p("L")).unwrap().simplify().unwrap();
    println!("y(L) = {tip}");
    assert_eq!(tip, p("P*L^3/(3*E*I)").simplify().unwrap());
    let slope = ys.diff("x").unwrap().substitute("x", &p("L")).unwrap().simplify().unwrap();
    assert_eq!(slope, p("P*L^2/(2*E*I)").simplify().unwrap());
    let back = ys.diff_n("x", 2).unwrap();
    assert!(Expr::sub(back, ypp).unwrap().simplify().unwrap().is_zero());
}
