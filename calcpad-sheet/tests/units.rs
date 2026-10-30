//! Percent conversion and a sheet's own units (`#unit`), compiled and run by the engine.

use calcpad_sheet::compile;
use serde_json::Value;

/// Compiles a sheet, runs it and returns (compiled text, visible text of the report, engine errors).
fn run(src: &str) -> (String, String, Vec<Value>) {
    let c = compile(src);
    let req = serde_json::json!({ "content": c.text, "includeLineAnchors": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    let html = v["html"].as_str().unwrap_or("").to_string();
    let body = html.split("</head>").nth(1).unwrap_or(&html).to_string();
    let mut out = String::new();
    let mut in_tag = false;
    for ch in body.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let text = out
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (c.text, text, v["errors"].as_array().cloned().unwrap_or_default())
}

#[test]
fn convert_to_percent() {
    let (cp, text, errs) =
        run("<!--sheet-->\nr: 0.125 -> %\nu: 1/3 -> %\nw: 0.07\nx: w -> %\ny: 2.5‰ -> %\nz: 0.0123 -> ‰\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(cp.contains("r = 0.125|%"), "{cp}");
    assert!(cp.contains("z = 0.0123|‰"), "{cp}");
    assert!(text.contains("r = 0.125 = 12.5 %"), "{text}");
    assert!(text.contains("u = 1 3 = 33.33 %"), "{text}");
    assert!(text.contains("x = w = 0.07 = 7 %"), "{text}");
    assert!(text.contains("y = 2.5 ‰ = 0.25 %"), "{text}");
    assert!(text.contains("z = 0.0123 = 12.3 ‰"), "{text}");
}

#[test]
fn percent_input_round_trips() {
    // An input in percent is shown in percent, and a ratio worked out from it can be shown in percent again.
    let (_, text, errs) = run("<!--sheet-->\nrate: ? 12.5 % | rate\nf: rate * 200 kN\ng: rate * 2 -> %\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(text.contains("25 kN"), "{text}");
    assert!(text.contains("= 25 %"), "{text}");
}

#[test]
fn own_unit() {
    let src = "<!--sheet-->\n#unit bag = 25 kg | cement bag\n#unit pallet = 40 bag\nload: 3 pallet -> t\nn: 1000 kg -> bag\nq: 2 bag/m² -> kg/m²\n";
    let (cp, text, errs) = run(src);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        cp.contains(".bag = 25 kg '<span class=\"sheet-note\">cement bag</span>"),
        "{cp}"
    );
    assert!(cp.contains(".pallet = 40 bag"), "{cp}");
    assert!(text.contains("bag = 25 kg cement bag"), "{text}");
    assert!(text.contains("load = 3 pallet = 3 t"), "{text}");
    assert!(text.contains("n = 1000 kg = 40 bag"), "{text}");
    assert!(text.contains("= 50 kg ∕ m 2"), "{text}");
}

#[test]
fn own_unit_errors() {
    for (line, msg) in [
        ("#unit ft2 = 1 ft", "letters and _ only"),
        ("#unit bag", "#unit name = value"),
        ("#unit bag =", "needs a value"),
        ("#unit bag = 25 kg kg kg (", "number with units"),
        ("#unit a = 1 m; b = 2 m", "one unit per line"),
    ] {
        let c = compile(&format!("<!--sheet-->\n{line}\n"));
        assert!(
            c.diagnostics.iter().any(|d| d.line == 1 && d.message.contains(msg)),
            "{line}: {:?}",
            c.diagnostics
        );
    }
    // A built-in unit cannot be redefined: the engine says so on the line.
    let (_, _, errs) = run("<!--sheet-->\n#unit kip = 4448 N\n");
    assert!(!errs.is_empty());
    // A heading and prose that start with the word are not unit lines.
    let c = compile("<!--sheet-->\n# unit = weight\n#units of the sheet\n");
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
    assert!(!c.text.contains(".unit"), "{}", c.text);
}

#[test]
fn convert_to_base_units() {
    let (cp, text, errs) = run("<!--sheet-->\nF: 5 kN\nL: 2 m\nM: F * L -> base\nP: 3 MPa -> base\nf: 3 kHz -> base\nA: 2 cm * 3 cm -> base\nn: 12 -> base\nt: 20 °C -> base\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(cp.contains("M = F * L|base"), "{cp}");
    assert!(text.contains("M = F · L = 5 kN · 2 m = 10000 kg · m 2 ∕ s 2"), "{text}");
    assert!(text.contains("P = 3 MPa = 3000000 kg · m -1 · s -2"), "{text}");
    assert!(text.contains("f = 3 kHz = 3000 s -1"), "{text}");
    assert!(text.contains("A = 2 cm · 3 cm = 0.0006 m 2"), "{text}");
    // A plain number has no units to write, and a temperature is left as it is.
    assert!(text.contains("n = 12"), "{text}");
    assert!(text.contains("t = 20"), "{text}");
}
