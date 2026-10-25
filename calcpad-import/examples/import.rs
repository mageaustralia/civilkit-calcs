//! Prints the Calcpad text and the report for one `.xmcd` or `.mcdx` file.
//!
//! ```text
//! cargo run -p civilkit-calcs-import --example import -- <file> [--tidy] [--markdown]
//! ```
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.iter().find(|a| !a.starts_with("--")).cloned() else {
        eprintln!("usage: import <file> [--tidy] [--markdown]");
        return ExitCode::from(2);
    };
    let bytes = match std::fs::read(&path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("cannot read {}: {}", path, e);
            return ExitCode::from(1);
        }
    };
    let r = calcpad_import::import_bytes(
        &bytes,
        &calcpad_import::ImportOptions {
            tidy: args.iter().any(|a| a == "--tidy"),
            markdown: args.iter().any(|a| a == "--markdown"),
            mathml: !args.iter().any(|a| a == "--no-mathml"),
            ..Default::default()
        },
    );
    print!("{}", r.calcpad);
    for a in &r.assets {
        eprintln!("asset {} ({}, {} bytes)", a.path, a.mime, a.bytes.len());
    }
    eprintln!("---- report ----");
    eprint!("{}", r.report.to_text());
    if r.report.is_fatal() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
