//! Runs a Calcpad text file through the engine and prints the plain text of the result and the
//! engine's error messages, for trying a construct by hand.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import --example run_cpd -- <file.cpd>
//! ```
fn main() {
    let path = std::env::args().nth(1).expect("usage: run_cpd <file>");
    let code = std::fs::read_to_string(&path).expect("cannot read the file");
    let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    p.debug = true;
    p.parse(&format!("#round 10\n{code}"), true);
    for e in &p.errors_list {
        println!("ERROR line {}: {}", e.source_line, e.message);
    }
    let html = p.html_result.replace("</p>", "\n").replace("</div>", "\n");
    let mut in_tag = false;
    let mut out = String::new();
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    for l in out.replace("&#8722;", "-").lines().filter(|l| !l.trim().is_empty()) {
        println!("{}", l.trim());
    }
}
