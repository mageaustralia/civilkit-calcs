//! Imports one `.xmcd` or `.mcdx` file, runs the Calcpad text through the engine and writes the
//! text and the engine's HTML next to each other, for checking converted lines against saved results.
//!
//! ```text
//! cargo run --release -p civilkit-calcs-import --example engine_check -- <file> <out-prefix> [probe-file]
//! ```
//! A probe file holds extra Calcpad lines appended to the converted text before the engine runs.
//!
//! For an `.mcdx` file the sheet is first extended so that every math region with a saved scalar
//! result is announced by a marker line (`@@<region id>@@`) and shown in SI base units, and a
//! definition is followed by a line that evaluates the defined name. `<out-prefix>.pairs` lists
//! `id`, saved value and the unit expression, so a script can compare the engine's number with the
//! saved one. The engine prints 12 decimals.
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: engine_check <file> <out-prefix> [probe-file]");
        return ExitCode::from(2);
    }
    let bytes = match std::fs::read(&args[1]) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {}: {}", args[1], e);
            return ExitCode::from(1);
        }
    };
    let opts = calcpad_import::ImportOptions {
        // NO_MATHML=1 checks the typeset-Calcpad display instead of the MathML one.
        mathml: std::env::var_os("NO_MATHML").is_none(),
        region_markers: true,
        ..Default::default()
    };
    let mut pairs = String::new();
    let r = if bytes.starts_with(b"PK") {
        match calcpad_import::mcdx::read(&bytes, opts.limits) {
            Ok(mut sheet) => {
                calcpad_import::translate::reading_order(&mut sheet.regions);
                sheet.regions = with_probes(sheet.regions, &mut pairs);
                calcpad_import::import_sheet_opts(sheet, &opts)
            }
            Err(_) => calcpad_import::import_bytes(&bytes, &opts),
        }
    } else {
        calcpad_import::import_bytes(&bytes, &opts)
    };
    // What the page shows (author's equations, `#nosub`, precisions) is checked for engine errors as
    // it stands; the numbers are read from the computed form (no display wrappers, 12 decimals).
    let shown_errors = {
        let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
        p.debug = true;
        p.plot_settings.vector_graphics = true;
        p.parse(&r.calcpad, true);
        p.errors_list.len() + p.html_result.matches("class=\"err\"").count()
    };
    let mut code = format!("#round 12\n{}", computed_view(&r.calcpad));
    let _ = std::fs::write(format!("{}.pairs", args[2]), &pairs);
    if let Some(p) = args.get(3) {
        code.push_str(&std::fs::read_to_string(p).unwrap_or_default());
    }
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.debug = true;
    parser.plot_settings.vector_graphics = true;
    parser.parse(&code, true);
    let _ = std::fs::write(format!("{}.cpd", args[2]), &code);
    parser.debug = true;
    eprintln!(
        "engine errors: shown form {shown_errors}, computed form {}",
        parser.errors_list.len() + parser.html_result.matches("class=\"err\"").count()
    );
    for e in &parser.errors_list {
        eprintln!(
            "error at output line {} (source line {}): {}",
            e.output_line, e.source_line, e.message
        );
    }
    let _ = std::fs::write(format!("{}.html", args[2]), &parser.html_result);
    ExitCode::SUCCESS
}

use calcpad_import::ir::{BinOp, Expr, Name, Region, RegionKind, Stmt, Target};

/// The unit of a saved result (file spelling: `kilogram/(meter^3)`) as an expression over Calcpad names.
fn unit_expr(u: &str) -> Option<Expr> {
    let base = |n: &str| -> Option<&'static str> {
        Some(match n {
            "kilogram" => "kg",
            "meter" => "m",
            "second" => "s",
            "ampere" => "A",
            "kelvin" => "K",
            "mole" => "mol",
            "candela" => "cd",
            _ => return None,
        })
    };
    let term = |t: &str| -> Option<Expr> {
        let t = t.trim().trim_matches(|c| c == '(' || c == ')');
        match t.split_once('^') {
            Some((n, p)) => {
                let p = p.trim_matches(|c| c == '(' || c == ')');
                Some(Expr::Bin(
                    BinOp::Pow,
                    Box::new(Expr::Name(Name::new(base(n)?))),
                    Box::new(Expr::Num(p.to_string())),
                ))
            }
            None => Some(Expr::Name(Name::new(base(t)?))),
        }
    };
    let product = |s: &str| -> Option<Expr> {
        let mut it = s.split('*').map(&term);
        let mut acc = it.next()??;
        for t in it {
            acc = Expr::Bin(BinOp::Mul, Box::new(acc), Box::new(t?));
        }
        Some(acc)
    };
    match u.split_once('/') {
        None => product(u),
        Some((a, b)) => {
            let num = if a.trim() == "1" {
                Expr::Num("1".into())
            } else {
                product(a)?
            };
            Some(Expr::Bin(BinOp::Div, Box::new(num), Box::new(product(b)?)))
        }
    }
}

fn with_probes(regions: Vec<Region>, pairs: &mut String) -> Vec<Region> {
    let mut out = Vec::new();
    for r in regions {
        let RegionKind::Math(m) = &r.kind else {
            out.push(r);
            continue;
        };
        let (Some(sr), false) = (&m.saved_result, m.disabled) else {
            out.push(r);
            continue;
        };
        let unit = match &sr.unit {
            Some(u) => match unit_expr(u) {
                Some(e) => Some(e),
                None => {
                    out.push(r);
                    continue;
                }
            },
            None => None,
        };
        let mut m2 = m.clone();
        match &m.stmt {
            Stmt::Eval { expr, unit: own } => {
                // A saved result with no unit of its own keeps the display unit the region names (a
                // `deg` override shows the angle in degrees, as the saved number is).
                m2.stmt = Stmt::Eval {
                    expr: expr.clone(),
                    unit: unit.clone().or_else(|| own.clone()),
                };
                pairs.push_str(&format!(
                    "{}\t{}\t{}\teval\n",
                    r.id,
                    sr.value,
                    sr.unit.clone().unwrap_or_default()
                ));
                out.push(Region {
                    kind: RegionKind::Math(m2),
                    ..r
                });
            }
            Stmt::Define {
                target: Target::Var(n), ..
            } => {
                pairs.push_str(&format!(
                    "{}\t{}\t{}\tdefine\n",
                    r.id,
                    sr.value,
                    sr.unit.clone().unwrap_or_default()
                ));
                let probe = Region {
                    kind: RegionKind::Math(calcpad_import::ir::MathRegion {
                        stmt: Stmt::Eval {
                            expr: Expr::Name(n.clone()),
                            unit,
                        },
                        disabled: false,
                        saved_result: None,
                        saved_expr: None,
                        frozen: false,
                        precision: None,
                        control: None,
                        solve: None,
                    }),
                    ..r.clone()
                };
                out.push(r.clone());
                out.push(probe);
            }
            _ => out.push(r),
        }
    }
    out
}

/// The computed form of the converted text: the `#round` and `#nosub` lines and the author's
/// equation blocks (a hidden computation followed by a `#noc` block) are removed so that every
/// definition prints its value, to 12 decimals, as the number checks expect.
fn computed_view(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut skip = vec![false; lines.len()];
    for i in 0..lines.len() {
        let t = lines[i].trim();
        if t == "#nosub" || t.starts_with("#round") || t.starts_with("#format") {
            skip[i] = true;
        } else if t == "#end hide" && lines.get(i + 1).map(|l| l.trim()) == Some("#noc") {
            let close = (i + 2..lines.len())
                .find(|&k| lines[k].trim() == "#end noc")
                .unwrap_or(i + 1);
            for s in &mut skip[i + 1..=close] {
                *s = true;
            }
            if i >= 2 && lines[i - 2].trim() == "#hide" {
                skip[i - 2] = true;
                skip[i] = true;
            }
        }
    }
    lines
        .iter()
        .zip(&skip)
        .filter(|(_, s)| !**s)
        .map(|(l, _)| format!("{l}\n"))
        .collect()
}
