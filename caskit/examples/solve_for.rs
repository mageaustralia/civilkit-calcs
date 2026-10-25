//! Rearranging formulas, solving quadratics and a small linear system.
//! Run with `cargo run -p civilkit-calcs-caskit --example solve_for`.

use caskit::{parse_equation, solve, solve_linear_system, CasError};

fn show(eq: &str, var: &str) -> Result<(), CasError> {
    let roots = solve(&parse_equation(eq)?, var)?;
    let list: Vec<String> = roots.iter().map(|r| r.to_ascii()).collect();
    println!("{eq}   solve for {var}:   {}", list.join("   or   "));
    Ok(())
}

fn main() -> Result<(), CasError> {
    // Rearrange a formula; a square gives both branches, the positive one first.
    show("M = w*L^2/8", "w")?;
    show("M = w*L^2/8", "L")?;
    show("d = 5*w*L^4/(384*E*I)", "E")?;

    // Quadratics: numeric roots are exact, symbolic ones use the discriminant.
    show("x^2 - 5*x + 6 = 0", "x")?;
    show("a*x^2 + b*x + c = 0", "x")?;

    // A system with symbolic coefficients (Cramer's rule, up to 4 unknowns).
    let eqs = [parse_equation("a*x + b*y = e")?, parse_equation("c*x + d*y = f")?];
    let sol = solve_linear_system(&eqs, &["x", "y"])?;
    println!("x = {}", sol[0]);
    println!("y = {}", sol[1]);

    // What cannot be solved is an error, not a guess.
    match solve(&parse_equation("x^3 + x = 1")?, "x") {
        Err(e) => println!("x^3 + x = 1: {e}"),
        Ok(r) => println!("unexpected {r:?}"),
    }
    match solve(&parse_equation("x^2 + 1 = 0")?, "x") {
        Err(e) => println!("x^2 + 1 = 0: {e}"),
        Ok(r) => println!("unexpected {r:?}"),
    }
    Ok(())
}
