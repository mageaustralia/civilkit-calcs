//! Percent conversion, compiled and run by the engine.

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
    let text = out.replace("&nbsp;", " ").split_whitespace().collect::<Vec<_>>().join(" ");
    (c.text, text, v["errors"].as_array().cloned().unwrap_or_default())
}

#[test]
fn convert_to_percent() {
    let (cp, text, errs) = run("<!--sheet-->\nr: 0.125 -> %\nu: 1/3 -> %\nw: 0.07\nx: w -> %\ny: 2.5‰ -> %\nz: 0.0123 -> ‰\n");
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
