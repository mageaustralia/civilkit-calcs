//! Simply supported beam under a uniform load, and a cantilever with a tip load, derived from the
//! bending equation. `y` is measured in the direction of the load, so `E*I*y'' = -M`.
//! Run with `cargo run -p civilkit-calcs-caskit --example beam_deflection`.

use caskit::{parse, solve_linear_system, CasError, Equation, Expr};

fn main() -> Result<(), CasError> {
    // E*I*y'' = -(w*L*x/2 - w*x^2/2)
    let ypp = parse("-(w*L*x/2 - w*x^2/2)/(E*I)")?;
    let slope = Expr::add(ypp.integrate("x")?, parse("C1")?)?;
    let y = Expr::add(slope.integrate("x")?, parse("C2")?)?;
    println!("y(x) = {}", y);

    // y(0) = 0 and y(L) = 0
    let c = solve_linear_system(
        &[
            Equation::new(y.substitute("x", &parse("0")?)?, parse("0")?),
            Equation::new(y.substitute("x", &parse("L")?)?, parse("0")?),
        ],
        &["C1", "C2"],
    )?;
    println!("C1 = {}", c[0]);
    println!("C2 = {}", c[1]);
    let y = y
        .substitute_all(&[("C1", c[0].clone()), ("C2", c[1].clone())])?
        .simplify()?;
    println!("y(x) = {}", y);
    println!("y(L/2) = {}", y.substitute("x", &parse("L/2")?)?.simplify()?);
    println!(
        "LaTeX: y(L/2) = {}",
        y.substitute("x", &parse("L/2")?)?.simplify()?.to_latex()
    );

    // Cantilever, point load P at the tip: y'' = P*(L - x)/(E*I), y(0) = 0, y'(0) = 0
    let ypp = parse("P*(L - x)/(E*I)")?;
    let slope = ypp.integrate("x")?; // y'(0) = 0 makes the first constant zero
    let y = slope.integrate("x")?; // and y(0) = 0 the second
    println!("cantilever y(x) = {}", y);
    println!("tip deflection y(L) = {}", y.substitute("x", &parse("L")?)?.simplify()?);
    Ok(())
}
