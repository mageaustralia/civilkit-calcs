//! Parse, simplify, differentiate, integrate, factor and evaluate.
//! Run with `cargo run -p civilkit-calcs-caskit --example basics`.

use caskit::{parse, CasError};
use std::collections::HashMap;

fn main() -> Result<(), CasError> {
    // Equal expressions have identical canonical forms, so this is exactly zero.
    let e = parse("-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)")?;
    println!("{}  ->  {}", e, e.simplify()?);

    // Expand and print as ASCII or LaTeX.
    let p = parse("(x + 1)^3")?;
    let expanded = p.expand()?;
    println!("{}  =  {}", p, expanded);
    println!("LaTeX: {}", expanded.to_latex());

    // Factor.
    println!("factor(x^2 - 5*x + 6) = {}", parse("x^2 - 5*x + 6")?.factor()?);

    // Differentiate and integrate.
    let f = parse("x*sin(x)")?;
    println!("d/dx {} = {}", f, f.diff("x")?.simplify()?);
    println!("integral of {} dx = {}", f, f.integrate("x")?);
    println!(
        "integral 0..pi of sin(x) = {}",
        parse("sin(x)")?.integrate_definite("x", &parse("0")?, &parse("pi")?)?
    );

    // Substitute and evaluate.
    let m = parse("w*L^2/8")?;
    println!("M = {}", m);
    println!(
        "M with w = 10, L = 6: {}",
        m.substitute_all(&[("w", parse("10")?), ("L", parse("6")?)])?
    );
    let mut values = HashMap::new();
    values.insert("w".to_string(), 10.0);
    values.insert("L".to_string(), 6.0);
    println!("as a float: {}", m.eval(&values)?);

    // Unsupported things are explicit errors.
    match parse("1/(x^2 + 1)")?.integrate("x") {
        Err(e) => println!("1/(x^2+1): {e}"),
        Ok(r) => println!("unexpected: {r}"),
    }
    Ok(())
}
