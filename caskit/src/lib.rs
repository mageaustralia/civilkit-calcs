//! Placeholder
pub mod error;
pub mod expr;
pub mod ops;
pub mod parse;
pub mod print;
pub mod rat;
pub use error::*;
pub use expr::*;
pub use parse::*;
pub use rat::Rat;
#[cfg(test)]
mod smoke {
    use super::*;
    #[test]
    fn smoke() {
        for s in ["w*L^2/8", "x^2 - 5*x + 6", "sin(x)*x", "-1/12*L^4*w/(2*E*I) + L^4*w/(24*E*I)", "1/(E*I)", "sqrt(8)", "2^-1", "x^(1/3)*y^-2", "exp(2*x)/3", "(x+1)^2*(x+1)", "sqrt(2)*sqrt(3)", "1/sqrt(2)"] {
            let e = parse(s).unwrap();
            println!("{s} => {} | {} | {:?}", e, e.to_latex(), parse(&e.to_ascii()).unwrap() == e);
        }
    }
}
