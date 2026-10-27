use super::*;
use serde_json::Value;

const SAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../web/calcpad-web/public/sheet-samples/");

fn sample(name: &str) -> String {
    std::fs::read_to_string(format!("{SAMPLES}{name}.sheet")).unwrap()
}

/// Compiles a sheet and renders the result with the engine. Returns (compiled, html, errors).
fn render(src: &str) -> (Compiled, String, Vec<Value>) {
    let c = compile(src);
    let (html, errs) = run(&c.text);
    (c, html, errs)
}

fn run(calcpad: &str) -> (String, Vec<Value>) {
    let req = serde_json::json!({ "content": calcpad, "includeLineAnchors": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    (
        v["html"].as_str().unwrap_or("").to_string(),
        v["errors"].as_array().cloned().unwrap_or_default(),
    )
}

/// The visible text of the report: tags removed, white space collapsed.
fn text(html: &str) -> String {
    let body = html.split("</head>").nth(1).unwrap_or(html);
    // Drop the style and script blocks.
    let mut s = body.to_string();
    for tag in ["style", "script", "svg"] {
        while let Some(a) = s.find(&format!("<{tag}")) {
            let Some(b) = s[a..].find(&format!("</{tag}>")) else {
                break;
            };
            s.replace_range(a..a + b + tag.len() + 3, " ");
        }
    }
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn no_errors(name: &str, errs: &[Value], c: &Compiled) {
    assert!(errs.is_empty(), "{name}: engine errors {errs:?}\n{}", c.text);
    assert!(c.diagnostics.is_empty(), "{name}: {:?}", c.diagnostics);
}

#[test]
fn beam_bending_values() {
    let (c, html, errs) = render(&sample("Beam bending"));
    no_errors("beam", &errs, &c);
    let t = text(&html);
    // Hand check: M* = 12 x 6^2 / 8 = 54 kN.m; Ms = 300 MPa x 1.23e6 mm3 = 369 kN.m;
    // phiMs = 0.9 x 369 = 332.1; 54 / 332.1 = 0.1626.
    assert!(t.contains("= 54 kN · m"), "{t}");
    assert!(t.contains("= 369 kN · m"));
    assert!(t.contains("= 332.1 kN · m"));
    assert!(t.contains("= 0.163"));
    assert!(t.contains("OK: the beam passes in bending."));
    assert!(!t.contains("Fails: increase"));
}

#[test]
fn column_values() {
    let (c, html, errs) = render(&sample("Column"));
    no_errors("column", &errs, &c);
    let t = text(&html);
    // 7500 mm2 x 350 MPa = 2 625 000 N = 2625 kN; 0.85 x 0.8 x 2625 = 1785; 1200 / 1785 = 0.6723.
    assert!(t.contains("= 2625 kN"), "{t}");
    assert!(t.contains("= 1785 kN"));
    assert!(t.contains("= 0.672"));
}

#[test]
fn loads_table_values() {
    let (c, html, errs) = render(&sample("Loads table"));
    no_errors("loads", &errs, &c);
    let t = text(&html);
    // 1.2 x 4.5 + 1.5 x 3 = 9.9; 4.5 + 0.7 x 3 = 6.6; 9.9 x 2.4 = 23.76; 23.76 x 5^2 / 8 = 74.25.
    assert!(t.contains("= 9.9 kPa"), "{t}");
    assert!(t.contains("= 6.6 kPa"));
    assert!(t.contains("= 23.76 kN ∕ m"));
    assert!(t.contains("= 74.25 kN · m"));
    assert!(html.contains("<table"), "the Markdown table renders");
    assert!(html.contains("<b>K</b>") || html.contains("<b><var>K</var></b>"));
}

#[test]
fn plot_sample() {
    let (c, html, errs) = render(&sample("Plot"));
    no_errors("plot", &errs, &c);
    assert!(c.text.contains("$Plot{ M(x) @ x = 0 : L }"), "{}", c.text);
    assert!(html.contains("<svg"), "a plot is drawn");
    // M(3) = 12 x 3 x 3 / 2 = 54.
    assert!(text(&html).contains("= 54"));
    assert!(c.uses_latex);
}

#[test]
fn latex_sample() {
    let (c, html, errs) = render(&sample("LaTeX prose"));
    no_errors("latex", &errs, &c);
    assert!(c.uses_latex);
    assert!(html.contains("data-display=\"1\""), "the display formula is a block");
    assert!(html.contains("class=\"sheet-tex\""));
    // 54e6 N.mm x 150 mm / 8.5e7 mm4 = 95.29 MPa.
    assert!(text(&html).contains("= 95.29 MPa"));
    // Plain Calcpad (Show compiled, Convert, Word and Excel): the TeX source in italics, readable.
    let plain = compile_with(&sample("LaTeX prose"), &Options { plain_tex: true }).text;
    assert!(plain.contains("<i>\\sigma = \\frac{M\\,y}{I}</i>"), "{plain}");
    assert!(!plain.contains("&#"), "no character references in plain output");
}

#[test]
fn mixing_sample() {
    let (c, html, errs) = render(&sample("Mixing with Calcpad"));
    no_errors("mixing", &errs, &c);
    let t = text(&html);
    // Three bays of 2.5 m added by a raw Calcpad loop, then read by a sheet line.
    assert!(t.contains("= 7.5 m"), "{t}");
    // The friendly loop ran three times; the else branch once and the then branch twice.
    assert_eq!(t.matches("Stage one.").count(), 1);
    assert_eq!(t.matches("flag = k").count(), 2);
    assert!(t.contains("= 30 kN"));
}

#[test]
fn every_sample_compiles_cleanly() {
    for e in std::fs::read_dir(SAMPLES).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "sheet") {
            continue;
        }
        let src = std::fs::read_to_string(&p).unwrap();
        assert!(is_sheet(&src), "{p:?} has the marker");
        // The engine alone cannot run a python fence: the page runs it first (web/e2e/python-check.mjs runs these samples).
        if src.contains("```python") {
            let c = compile(&src);
            assert!(c.diagnostics.is_empty(), "{p:?}: {:?}", c.diagnostics);
            assert!(c.text.contains("#python\n") && c.text.contains("\n#end python"));
            assert_eq!(c.map.len(), c.text.lines().count());
            continue;
        }
        let (c, _, errs) = render(&src);
        no_errors(&format!("{p:?}"), &errs, &c);
        assert_eq!(c.map.len(), c.text.lines().count());
    }
}

fn lines(c: &Compiled) -> Vec<&str> {
    c.text.lines().collect()
}

#[test]
fn quantity_lines() {
    let c = compile("fy: 300 MPa | grade\nZe = 1.5e3 mm³\nMs: fy * Ze -> kN*m\nx: 1; y: 2");
    let l = lines(&c);
    assert!(l[0].starts_with("fy = 300 MPa '<span"));
    assert!(l[0].contains(">grade<"));
    assert_eq!(l[1], "Ze = 1500 mm^3");
    assert_eq!(l[2], "Ms = fy * Ze|kN*m");
    assert_eq!(&l[3..5], ["x = 1", "y = 2"]);
    assert_eq!(c.map, vec![0, 1, 2, 3, 3]);
    assert_eq!(
        c.defs.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["fy", "Ze", "Ms", "x", "y"]
    );
}

#[test]
fn names_typeset() {
    let c = compile("phi: 0.9\nsigma_c: 25 MPa\nMstar: 5 kN*m\nphiMs: phi * Mstar\nf(x): x^2\nDelta_T: 3");
    let l = lines(&c);
    assert_eq!(l[0], "φ = 0.9");
    assert_eq!(l[1], "σ_c = 25 MPa");
    assert_eq!(l[2], "M_star = 5 kN*m");
    assert_eq!(l[3], "φMs = φ * M_star");
    assert_eq!(l[4], "f(x) = x^2");
    assert_eq!(l[5], "Δ_T = 3");
}

#[test]
fn prose_stays_prose() {
    let c = compile("# Title\n\nNote: see the drawing below.\nThe load is 5 kN applied here.\n\n- a: item\n");
    // A heading and a plain paragraph are written as a person would; the list needs Markdown.
    assert_eq!(
        lines(&c),
        [
            "'<h1>Title</h1>",
            "",
            "'Note: see the drawing below. The load is 5 kN applied here.",
            "",
            "#markdown",
            "- a: item",
            "#end markdown"
        ]
    );
    assert_eq!(c.map, vec![0, 1, 2, 4, 5, 5, 5]);
    // A backslash forces text.
    let c = compile("\\fy: 300 MPa");
    assert_eq!(c.text, "'fy: 300 MPa\n");
}

#[test]
fn headings_and_paragraphs_are_hand_written_calcpad() {
    let c = compile("# One\n## Two\n### Three\n#### Four\n\nPlain words.\nMore plain words.\n\nSome *emphasis* here.\n\nMore plain.\n");
    assert_eq!(
        lines(&c),
        [
            "'<h1>One</h1>",
            "'<h2>Two</h2>",
            "\"Three",
            "'<h4>Four</h4>",
            "",
            "'Plain words. More plain words.",
            "",
            "#markdown",
            "Some *emphasis* here.",
            "#end markdown",
            "",
            "'More plain."
        ]
    );
    // Words with underscores (f_y) and a lone asterisk are not Markdown emphasis in prose with spaces around.
    let c = compile("Use f_y for the yield stress.\n");
    assert_eq!(c.text, "'Use f_y for the yield stress.\n");
}

#[test]
fn notes_are_plain_comments_in_plain_calcpad() {
    let src = "x: 5 kN | the load\ny: 2 m\n";
    let page = compile(src).text;
    assert_eq!(page, "x = 5 kN '<span class=\"sheet-note\">the load</span>\ny = 2 m\n");
    let plain = compile_with(src, &Options { plain_tex: true }).text;
    assert_eq!(plain, "x = 5 kN 'the load\ny = 2 m\n");
}

#[test]
fn blank_lines_are_kept_between_sections() {
    let c = compile("a: 1\n\n\nb: 2\nif b > 1:\n    c: 3\n\n    d: 4\ne: 5");
    assert_eq!(
        lines(&c),
        [
            "a = 1",
            "",
            "b = 2",
            "#if b > 1",
            "c = 3",
            "",
            "d = 4",
            "#end if",
            "e = 5"
        ]
    );
}

#[test]
fn step_break_and_continue() {
    let c = compile("for i in 0..10 step 2:\n    if i > 6:\n        break\n    s: i\n");
    assert_eq!(
        lines(&c),
        [
            "#for loopi1 = 1 : floor((10 - 0)/(2) + 0.000000001) + 1",
            "#hide",
            "i = 0 + (loopi1 - 1)*(2)",
            "#show",
            "#if i > 6",
            "#break",
            "#end if",
            "s = i",
            "#loop"
        ]
    );
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    // 0, 2, 4, 6 then i = 8 breaks: the loop ran for i = 0, 2, 4, 6.
    let t = text(&html);
    assert!(t.contains("s = i = 6") && !t.contains("s = i = 8"), "{t}");
    // A step down, and continue.
    let c = compile("for i in 5..1 step -2:\n    if i == 3:\n        continue\n    s: i\n");
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    let t = text(&html);
    assert!(
        t.contains("s = i = 5") && t.contains("s = i = 1") && !t.contains("s = i = 3"),
        "{t}"
    );
    // One-line form and outside a loop.
    let c = compile("while n > 0:\n    if n == 2: break\n    n: n - 1\n");
    assert!(c.text.contains("#if n ≡ 2\n#break\n#end if"));
    let c = compile("break\n");
    assert!(c.diagnostics.iter().any(|d| d.message.contains("outside a loop")));
}

#[test]
fn line_kinds() {
    let c = compile("<!--sheet-->\n# T\na: 1\n>> b = 2\nplot a for x in 0..1\nif a > 0:\n    c: 3\n\ntext");
    assert_eq!(c.kinds.len(), 9);
    use LineKind::*;
    assert_eq!(
        c.kinds,
        vec![Blank, Prose, Quantity, Raw, Plot, Block, Quantity, Blank, Prose]
    );
}

#[test]
fn format_is_idempotent_and_leaves_prose() {
    let src = "<!--sheet-->\n# T\n## Inputs\nL:6 m|span\nfy :  300 MPa   |  yield\nMs: fy*Ze->kN*m\n\n\nprose  with   spaces  \n>> raw   line\nx=1 ;y : 2\n";
    let once = format(src);
    assert_eq!(once, format(&once));
    assert!(once.contains(">> raw   line") && once.contains("prose  with   spaces\n"));
    // The prettified sheet compiles to the same Calcpad.
    let flat = |t: String| t.replace(' ', "").replace("\n\n", "\n");
    assert_eq!(flat(compile(src).text), flat(compile(&once).text));
}

#[test]
fn vectors_and_matrices() {
    let c = compile("v: [1; 2; 3]\nr: [1, 2, 3]\nM: [1, 2; 3, 4]\nN: [max(1, 2), 5]");
    let l = lines(&c);
    assert_eq!(l[0], "v = [1; 2; 3]");
    assert_eq!(l[1], "r = [1; 2; 3]");
    assert_eq!(l[2], "M = [1; 2 | 3; 4]");
    assert_eq!(l[3], "N = [max(1; 2); 5]");
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty());
    assert!(html.contains("matrix"));
}

#[test]
fn conditions_and_plots() {
    let c = compile("a: 2\nif a <= 3: \"small\" else: \"big\"\nif a == 2: b: 1\nplot f(x), g(x) for x in 0..a * 2");
    let l = lines(&c);
    assert_eq!(&l[1..6], ["#if a ≤ 3", "'small", "#else", "'big", "#end if"]);
    assert_eq!(&l[6..9], ["#if a ≡ 2", "b = 1", "#end if"]);
    assert_eq!(l[9], "$Plot{ f(x) & g(x) @ x = 0 : a * 2 }");
    let bad = compile("plot f(x) for x in 0");
    assert!(
        bad.text.contains("'plot f(x) for x in 0"),
        "an invalid plot line stays text"
    );
}

#[test]
fn escapes_pass_through_verbatim() {
    let c = compile("a: 1\n>> b = a + 1 'raw text\n```calcpad\n#if b > 1\n'big\n#end if\n```\nc: b");
    let l = lines(&c);
    assert_eq!(l[1], "b = a + 1 'raw text");
    assert_eq!(&l[2..5], ["#if b > 1", "'big", "#end if"]);
    assert_eq!(l[5], "c = b");
    assert_eq!(c.map, vec![0, 1, 3, 4, 5, 7]);
}

/// Sheet lines and raw Calcpad alternate in any order and share variables both ways.
#[test]
fn mixing_shares_variables() {
    let src = "<!--sheet-->\n# Mix\nn: 4\nphi: 0.5\n>> s = 0\n```calcpad\n#for i = 1 : n\n    s = s + i*phi\n#loop\n```\nprose between\nt: s * 2\n>> #if t > 10\n>> u = t\n>> #else\n>> u = 0\n>> #end if\nw: u + 1\n$$x = y$$\nz: w";
    let (c, html, errs) = render(src);
    no_errors("mix", &errs, &c);
    let t = text(&html);
    // s = 0.5 x (1 + 2 + 3 + 4) = 5; t = 10; the else branch gives u = 0; w = 1; z = 1.
    assert!(t.contains("s = s + i · φ"), "{t}");
    assert!(t.contains("t = s · 2 = 5 · 2 = 10"), "{t}");
    assert!(t.contains("w = u + 1 = 0 + 1 = 1"), "{t}");
    assert!(t.contains("z = w = 1"));
}

#[test]
fn raw_lines_can_use_sheet_spelling() {
    // The sheet defines `phi` (the engine's letter is phi) and a raw line reads it by the sheet spelling.
    let (c, html, errs) = render("<!--sheet-->\nphi: 0.9\nMstar: 10\n>> r = phi * Mstar");
    no_errors("raw names", &errs, &c);
    assert!(c.text.contains("r = φ * M_star"), "{}", c.text);
    assert!(text(&html).contains("= 0.9 · 10 = 9"));
    // A raw line that defines a name the sheet then reads.
    let (c, _, errs) = render("<!--sheet-->\n>> sigma = 3\nb: sigma + 1");
    no_errors("raw defs", &errs, &c);
    assert!(c.text.contains("σ = 3") && c.text.contains("b = σ + 1"), "{}", c.text);
}

#[test]
fn error_lines_map_back_to_the_sheet() {
    let src = "<!--sheet-->\n# T\n\na: 1\nprose line\nb: a + missing\n>> k = nothere\n\nc: 2";
    let (c, _, errs) = render(src);
    assert_eq!(errs.len(), 2, "{errs:?}");
    let sheet_lines: Vec<usize> = errs
        .iter()
        .map(|e| c.map[e["sourceLine"].as_u64().unwrap() as usize - 1])
        .collect();
    assert_eq!(sheet_lines, vec![5, 6]);
}

#[test]
fn blocks_nest_and_close_by_indentation() {
    let src = "for i in 1..2:\n    if i == 1:\n        for j in 1..2:\n            a: i * j\n    else:\n        b: i\nafter: 1";
    let c = compile(src);
    assert_eq!(
        lines(&c),
        [
            "#for i = 1 : 2",
            "#if i ≡ 1",
            "#for j = 1 : 2",
            "a = i * j",
            "#loop",
            "#else",
            "b = i",
            "#end if",
            "#loop",
            "after = 1"
        ]
    );
    assert!(c.diagnostics.is_empty());
    assert_eq!(c.map, vec![0, 1, 2, 3, 2, 4, 5, 1, 0, 6]);
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}");
    let t = text(&html);
    // i = 1: a = 1, 2 ; i = 2: b = 2.
    assert!(t.contains("a = i · j = 1 · 2 = 2"), "{t}");
    assert!(t.contains("b = i = 2"));
}

#[test]
fn else_if_while_and_vector_loops() {
    let c = compile("n: 3\nwhile n > 0:\n    n: n - 1\nif n > 5:\n    a: 1\nelse if n == 0:\n    a: 2\nelse:\n    a: 3\nfor x in [1, 2, 3]:\n    s: x");
    let l = lines(&c);
    assert_eq!(l[1], "#while n > 0");
    assert_eq!(l[4], "#if n > 5");
    assert_eq!(l[6], "#else if n ≡ 0");
    assert_eq!(l[8], "#else");
    assert!(l.contains(&"#hide") && l.iter().any(|x| x.starts_with("#for loopi1 = 1 : len(loopseq1)")));
    let (html, errs) = run(&c.text);
    assert!(errs.is_empty(), "{errs:?}\n{}", c.text);
    let t = text(&html);
    assert!(t.contains("a = 2"));
    assert!(t.contains("s = x = 1") && t.contains("s = x = 3"), "{t}");
}

#[test]
fn quantity_lines_in_loops_match_calcpad() {
    let sheet = "for i in 1..3:\n    q: i * 2 kN\n";
    let calc = "#for i = 1 : 3\nq = i * 2 kN\n#loop\n";
    let strip = |h: String| text(&h);
    let a = strip(run(&compile(sheet).text).0);
    let b = strip(run(calc).0);
    assert_eq!(a, b);
    assert!(a.contains("q = i · 2 kN = 3 · 2 kN = 6 kN"));
}

#[test]
fn prose_and_latex_inside_blocks() {
    let src = "for i in 1..2:\n    Stage $i$ of the loop.\n\n    $$a_i$$\n    r: i\n";
    let (c, html, errs) = render(src);
    assert!(errs.is_empty(), "{errs:?}\n{}", c.text);
    assert!(c.uses_latex);
    assert_eq!(html.matches("class=\"sheet-tex\"").count(), 4);
    assert_eq!(c.diagnostics.len(), 0);
}

#[test]
fn block_errors() {
    let c = compile("if 1 > 0:\n\ta: 1\n\t    b: 2\n");
    assert!(
        c.diagnostics
            .iter()
            .any(|d| d.line == 2 && d.severity == Severity::Error && d.message.contains("tabs and spaces")),
        "{:?}",
        c.diagnostics
    );
    let c = compile("else:\n    a: 1");
    assert!(c
        .diagnostics
        .iter()
        .any(|d| d.line == 0 && d.message.contains("without a matching")));
    let c = compile("if x:\ny: 1");
    assert!(c
        .diagnostics
        .iter()
        .any(|d| d.line == 0 && d.message.contains("no body")));
    let c = compile("If the load is large:\n    note");
    assert!(!c.text.contains("#if"));
    let c = compile("$$\na\n");
    assert!(c.diagnostics.iter().any(|d| d.message.contains("never closed")));
}

#[test]
fn deterministic_with_consistent_map() {
    for name in [
        "Beam bending",
        "Column",
        "Loads table",
        "Plot",
        "LaTeX prose",
        "Mixing with Calcpad",
    ] {
        let src = sample(name);
        let a = compile(&src);
        let b = compile(&src);
        assert_eq!(a.text, b.text);
        assert_eq!(a.map, b.map);
        let n = src.lines().count();
        assert_eq!(a.map.len(), a.text.lines().count());
        assert!(a.map.iter().all(|&l| l < n));
        // The map never goes backwards across top level statements, apart from block closers.
        assert!(a.text.ends_with('\n'));
    }
}

#[test]
fn prose_only_input_round_trips_through_markdown() {
    let src = "Some *text* with a [link](http://example.com).\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n![alt](pic.png)\n";
    let c = compile(src);
    // One markdown block, the text untouched.
    let l = lines(&c);
    assert_eq!(l[0], "#markdown");
    assert_eq!(l[1..l.len() - 1].join("\n"), src.trim_end());
}

#[test]
fn marker_detection() {
    assert!(is_sheet("\n  <!--sheet-->\na: 1"));
    assert!(!is_sheet("a = 1\n<!--sheet-->"));
}

#[test]
fn deflection_samples_values() {
    let (c, html, errs) = render(&sample("Beam deflection"));
    no_errors("deflection", &errs, &c);
    let t = text(&html);
    // EI = 30 GPa x 160000 cm4 = 48000 kN.m2; d = 5 x 30 x 6^4 / (384 x 48000) = 10.55 mm; phi = 30 x 6^3 / (24 x 48000) = 0.005625 (shown as 0.00562).
    assert!(t.contains("= 48000 kN · m 2"), "{t}");
    assert!(t.contains("= 10.55 mm"), "{t}");
    assert!(t.contains("= 0.00562"), "{t}");
    assert!(html.contains("<svg"), "the deflection curve is drawn");
    let (c, html, errs) = render(&sample("Cantilever tip load"));
    no_errors("cantilever", &errs, &c);
    let t = text(&html);
    // 20 x 3^3 / (3 x 48000) = 3.75 mm; M = 20 x 3 = 60 kN.m; for a load of 3 P: 11.25 mm.
    assert!(
        t.contains("= 3.75 mm") && t.contains("= 60 kN · m") && t.contains("= 11.25 mm"),
        "{t}"
    );
}

#[test]
fn python_fence_becomes_a_python_block() {
    let src = "<!--sheet-->\n# T\n\n```python\nimport numpy as np\nfor i in range(2):\n    export(a=i)\n```\n\nx: 1";
    let c = compile(src);
    let lines: Vec<&str> = c.text.split('\n').collect();
    let at = lines.iter().position(|l| *l == "#python").expect("opens a block");
    assert_eq!(
        &lines[at..at + 5],
        [
            "#python",
            "import numpy as np",
            "for i in range(2):",
            "    export(a=i)",
            "#end python"
        ]
    );
    // every line of the block maps to its own sheet line
    assert_eq!(c.map[at], 3);
    assert_eq!(c.map[at + 1], 4);
    assert_eq!(c.map[at + 3], 6);
    assert_eq!(c.map[at + 4], 7);
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics.len());
}

#[test]
fn python_fence_keeps_indentation_relative_to_its_block() {
    let src = "<!--sheet-->\nif 1 > 0:\n    ```python\n    if True:\n        print(1)\n    ```\n";
    let c = compile(src);
    assert!(
        c.text.contains("#python\nif True:\n    print(1)\n#end python"),
        "{}",
        c.text
    );
}

#[test]
fn python_fence_is_not_renamed_or_formatted_as_a_sheet() {
    let src = "<!--sheet-->\nphi: 2\n```python\nphi = 3   # not sheet syntax: a | b\n```\n";
    let c = compile(src);
    assert!(c.text.contains("\nphi = 3   # not sheet syntax: a | b\n"), "{}", c.text);
    assert_eq!(crate::format::format(src), src);
}

#[test]
fn an_unclosed_python_fence_is_an_error_and_is_closed() {
    let c = compile("<!--sheet-->\n```python\nx = 1\n");
    assert!(c.text.trim_end().ends_with("#end python"));
    assert!(c.diagnostics.iter().any(|d| matches!(d.severity, Severity::Error)));
}

#[test]
fn a_plain_code_fence_is_still_prose() {
    let c = compile("<!--sheet-->\n```rust\nlet a = 1;\n```\n");
    assert!(!c.text.contains("#python"), "{}", c.text);
}

fn python_body(src: &str) -> String {
    let c = compile(src);
    let t = c.text;
    let a = t.find("#python\n").expect("a python block") + "#python\n".len();
    let b = t.find("\n#end python").expect("closed");
    t[a..b].to_string()
}

#[test]
fn export_keywords_use_the_sheet_spelling() {
    let body =
        python_body("<!--sheet-->\n```python\nexport(sigma_c=2.0, phi=3.0, Mstar=4.0, y_c=5.0, plain=6.0)\n```\nx: 1");
    assert_eq!(body, "export(σ_c=2.0, φ=3.0, M_star=4.0, y_c=5.0, plain=6.0)");
}

#[test]
fn export_names_follow_across_lines_and_export_data() {
    let body = python_body("<!--sheet-->\n```python\nexport_data(\n    sigma_c=1,\n    phi=2)\nphi = 3\n```\nx: 1");
    assert_eq!(body, "export_data(\n    σ_c=1,\n    φ=2)\nphi = 3");
}

#[test]
fn export_positional_name_string_is_renamed() {
    let body = python_body("<!--sheet-->\n```python\nexport(\"sigma_c\", 2.0)\n```\nx: 1");
    assert_eq!(body, "export(\"σ_c\", 2.0)");
}

#[test]
fn only_the_export_call_keywords_are_renamed() {
    let src = "<!--sheet-->\n```python\nphi = 3\nsigma_c = f(phi=1)  # export(phi=1)\nexport(a=g(phi=2), s=\"phi=3\")\nobj.export(phi=1)\nmyexport(phi=1)\n```\nx: 1";
    let body = python_body(src);
    assert_eq!(
        body,
        "phi = 3\nsigma_c = f(phi=1)  # export(phi=1)\nexport(a=g(phi=2), s=\"phi=3\")\nobj.export(phi=1)\nmyexport(phi=1)"
    );
}

#[test]
fn a_name_exported_from_python_is_defined_for_the_lines_after_it() {
    let c = compile("<!--sheet-->\n```python\nexport(sigma_c=2.0)\n```\na: sigma_c * 2\n");
    assert!(c.text.contains("σ_c=2.0"), "{}", c.text);
    assert!(c.text.contains("σ_c"), "{}", c.text);
}

#[test]
fn a_bare_python_block_in_a_sheet_is_a_python_block() {
    let src = "<!--sheet-->\n#python\nphi = 3   # not sheet syntax: a | b\nexport(sigma_c=phi)\n#end python\nx: sigma_c * 2\n";
    let c = compile(src);
    assert!(
        c.text
            .contains("#python\nphi = 3   # not sheet syntax: a | b\nexport(σ_c=phi)\n#end python"),
        "{}",
        c.text
    );
    assert!(!c.text.contains("#markdown"), "{}", c.text);
    assert!(c.diagnostics.is_empty(), "{:?}", c.diagnostics);
    assert_eq!(crate::format::format(src), src);
}

#[test]
fn an_unclosed_bare_python_block_is_an_error_and_is_closed() {
    let c = compile("<!--sheet-->\n#python\nx = 1\n");
    assert!(c.text.trim_end().ends_with("#end python"), "{:?}", c.text);
    assert!(!c.diagnostics.is_empty());
}

// ---- inputs (docs/sheet-syntax.md, "Inputs and templates") ----

fn lines_of(src: &str) -> Vec<String> {
    compile(&format!("<!--sheet-->\n{src}"))
        .text
        .lines()
        .map(str::to_string)
        .collect()
}

fn errors_of(src: &str) -> Vec<(usize, String)> {
    compile(&format!("<!--sheet-->\n{src}"))
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| (d.line, d.message.clone()))
        .collect()
}

#[test]
fn a_number_input_is_a_ui_entry_with_its_note_after() {
    let l = lines_of("L: ? 6 m | span");
    assert_eq!(l[1], "#UI L = 6m '<span class=\"sheet-note\">span</span>");
    let l = lines_of("w = ? 12 kN/m");
    assert_eq!(l[1], "#UI w = 12kN/m");
    let l = lines_of("phi: ? 0.9\nfy: ? 300 MPa");
    assert_eq!(l[1], "#UI φ = 0.9");
    assert_eq!(l[2], "#UI fy = 300MPa");
    let l = lines_of("a: ? -2.5 m\nn: ? 1.5e3");
    assert_eq!(l[1], "#UI a = -2.5m");
    assert_eq!(l[2], "#UI n = 1500");
}

#[test]
fn a_choice_is_a_dropdown_or_radio_with_the_note_as_label() {
    let l = lines_of("E: ? [Steel = 200 GPa, *Timber = 11 GPa] | material");
    assert_eq!(
        l[1],
        "#UI {\"type\":\"dropdown\",\"keys\":[\"Steel\",\"Timber\"],\"values\":[\"200GPa\",\"11GPa\"]} 'material -'E = 11GPa"
    );
    let l = lines_of("k: ? (Low = 1, High = 2)");
    assert_eq!(
        l[1],
        "#UI {\"type\":\"radio\",\"keys\":[\"Low\",\"High\"],\"values\":[\"1\",\"2\"]} 'k -'k = 1"
    );
    // A quoted label may hold commas and brackets, and an apostrophe is fine.
    let l = lines_of("s: ? [\"Beam (A), long\" = 1, Bob's = 2]");
    assert!(l[1].contains("\"keys\":[\"Beam (A), long\",\"Bob's\"]"), "{}", l[1]);
}

#[test]
fn a_checkbox_is_on_or_off() {
    assert_eq!(
        lines_of("ribs: ? on | stiffeners fitted")[1],
        "#UI {\"type\":\"checkbox\"} 'stiffeners fitted -'ribs = 1"
    );
    assert_eq!(
        lines_of("ribs: ? off")[1],
        "#UI {\"type\":\"checkbox\"} 'ribs -'ribs = 0"
    );
    assert_eq!(lines_of("a: ? true")[1], "#UI {\"type\":\"checkbox\"} 'a -'a = 1");
    assert_eq!(lines_of("a: ? False")[1], "#UI {\"type\":\"checkbox\"} 'a -'a = 0");
}

#[test]
fn input_lines_are_defined_names_and_mapped() {
    let c = compile("<!--sheet-->\nphi: ? 0.9 | factor\nx: phi * 2\n>> y = phi + 1\n");
    assert_eq!(c.defs[0].name, "phi");
    assert_eq!(c.defs[0].calcpad, "φ");
    assert_eq!(c.kinds[1], LineKind::Quantity);
    assert_eq!(c.map.len(), c.text.lines().count());
    assert!(c.text.contains("y = φ + 1"), "{}", c.text);
}

#[test]
fn a_run_of_inputs_is_one_form_closed_before_anything_else() {
    let c = compile("<!--sheet-->\n## Inputs\nL: ? 6 m\nw: ? 12 kN/m\n\nThe load.\nM: w * L\nk: ? 2\n");
    let t: Vec<&str> = c.text.lines().collect();
    assert_eq!(t[0], "'<h2>Inputs</h2>");
    assert_eq!(t[1], "'<div class=\"sheet-form\">");
    assert!(t[2].starts_with("#UI L") && t[3].starts_with("#UI w"));
    let close = t.iter().position(|l| *l == "'</div>").unwrap();
    assert!(close < t.iter().position(|l| l.starts_with("'The load")).unwrap());
    assert_eq!(t.iter().filter(|l| l.starts_with("'<div")).count(), 2);
    assert_eq!(t.iter().filter(|l| **l == "'</div>").count(), 2);
    assert_eq!(t.last().copied(), Some("'</div>"));
    assert_eq!(c.map.len(), t.len());
    // Inside a block the wrapper closes before the block does.
    let c = compile("<!--sheet-->\nn: 1\nif n == 1:\n    a: ? 1\nelse:\n    b: ? 2\n");
    let t: Vec<&str> = c.text.lines().collect();
    for (i, l) in t.iter().enumerate() {
        if *l == "#else" || *l == "#end if" {
            assert_eq!(t[i - 1], "'</div>", "{t:?}");
        }
    }
}

#[test]
fn plain_calcpad_has_no_wrapper() {
    let c = compile_with(
        "<!--sheet-->\nL: ? 6 m | span\nE: ? [A = 1, B = 2] | pick\n",
        &Options { plain_tex: true },
    );
    assert_eq!(
        c.text,
        "#UI L = 6m 'span\n#UI {\"type\":\"dropdown\",\"keys\":[\"A\",\"B\"],\"values\":[\"1\",\"2\"]} 'pick -'E = 1\n"
    );
}

#[test]
fn input_mistakes_are_errors_on_their_line_and_never_panic() {
    let cases = [
        ("L: ?", "no default"),
        ("L: ? 2*3 m", "not an expression"),
        ("L: ? six", ""),
        ("L: ? [a = 1]", "at least two"),
        ("L: ? [a, b]", "needs a value"),
        ("L: ? [a = 1, b = 2", "not closed"),
        ("L: ? [a = 1,, b = 2]", "empty"),
        ("L: ? [*a = 1, *b = 2]", "only one"),
        ("L: ? [a = x, b = 2]", "must be a number"),
        ("L: ? [a} = 1, b = 2]", "cannot contain"),
        ("L: ? [ = 1, b = 2]", "no label"),
        ("L: ? [a = , b = 2]", "no value"),
        ("L: ? 1 m; M: ? 2", "one input per line"),
        ("L: ? 1 +", "must be a number"),
    ];
    for (src, want) in cases {
        let e = errors_of(src);
        if want.is_empty() {
            // Prose that merely contains a question mark stays prose, with a warning.
            let c = compile(&format!("<!--sheet-->\n{src}"));
            assert!(c.diagnostics.iter().all(|d| d.severity == Severity::Warning), "{src}");
            continue;
        }
        assert_eq!(e.len(), 1, "{src}: {e:?}");
        assert_eq!(e[0].0, 1, "{src}");
        assert!(e[0].1.contains(want), "{src}: {}", e[0].1);
        let c = compile(&format!("<!--sheet-->\n{src}"));
        assert!(c.text.contains("Sheet syntax: input:"), "{src}: {}", c.text);
        assert_eq!(c.map.len(), c.text.lines().count());
    }
}

#[test]
fn a_question_in_prose_is_still_prose() {
    for src in ["Why: ? is it so", "Note: ? maybe", "Is it ok? yes", "a: what?"] {
        let c = compile(&format!("<!--sheet-->\n{src}"));
        assert!(!c.text.contains("#UI"), "{src}: {}", c.text);
        assert!(c.diagnostics.iter().all(|d| d.severity == Severity::Warning), "{src}");
        assert_eq!(c.kinds[1], LineKind::Prose, "{src}");
    }
}

#[test]
fn inputs_run_in_the_engine_and_results_follow_the_default() {
    let src = "<!--sheet-->\n## Inputs\nL: ? 6 m | span\nw: ? 12 kN/m | load\nE: ? [Steel = 200 GPa, *Timber = 11 GPa] | material\nribs: ? on | ribs\nM: w * L^2 / 8 -> kN*m\nif ribs == 1: \"Ribs on\" else: \"Ribs off\"\nEE: E * 1\n";
    let (c, html, errs) = render(src);
    no_errors("inputs", &errs, &c);
    let t = text(&html);
    assert!(t.contains("L = 6 m span"), "{t}");
    assert!(t.contains("material - E = 11 GPa"), "{t}");
    assert!(t.contains("ribs - ribs = 1"), "{t}");
    assert!(t.contains("= 54 kN"), "{t}");
    assert!(t.contains("Ribs on"), "{t}");
}

#[test]
fn an_entered_value_reaches_the_report() {
    // The page sends the entered values with the request, as it does for a Calcpad worksheet.
    let c = compile("<!--sheet-->\nL: ? 6 m\nA: L * 2 m\n");
    let req = serde_json::json!({ "content": c.text, "enableUi": true, "uiOverrides": { "L:1": "9" } });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    let html = v["html"].as_str().unwrap();
    assert!(html.contains("calcpad-ui-input"), "the form shows the entry");
    let req = serde_json::json!({ "content": c.text, "uiOverrides": { "L:1": "9" } });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    assert!(text(v["html"].as_str().unwrap()).contains("A = L · 2 m = 9 m · 2 m = 18 m"));
}

#[test]
fn the_values_comment_written_by_the_page_passes_through() {
    let c = compile("<!--sheet-->\n'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->\n# T\nL: ? 6 m\n");
    assert_eq!(c.text.lines().next(), Some("'<!--{\"uiOverrides\":{\"L:1\":\"8\"}}-->"));
    assert!(is_sheet("<!--sheet-->\n'<!--{}-->"));
    assert_eq!(c.kinds[1], LineKind::Raw);
}

#[test]
fn formatting_inputs_is_tidy_and_idempotent() {
    let src = "<!--sheet-->\n## Inputs\nL:?6 m|span\nwidth :   ?  300  mm  | b\nE: ?[ Steel=200 GPa,*Timber =11 GPa ]|material\nk = ?(a=1,b=2)\nribs:?on|r\nM: w*L\n";
    let f = format(src);
    assert_eq!(
        f,
        "<!--sheet-->\n## Inputs\nL    : ? 6 m | span\nwidth: ? 300 mm | b\nE    : ? [Steel = 200 GPa, *Timber = 11 GPa] | material\nk = ? (a = 1, b = 2)\nribs: ? on | r\nM   : w*L\n"
    );
    assert_eq!(format(&f), f);
    // A mistake is left as written.
    let bad = "<!--sheet-->\nL: ? [a = 1\n";
    assert_eq!(format(bad), bad);
    assert_eq!(format(&format(bad)), format(bad));
    // The formatted text compiles to the same worksheet.
    assert_eq!(compile(&f).text.replace(' ', ""), compile(src).text.replace(' ', ""));
}

#[test]
fn existing_sheets_compile_exactly_as_before_inputs_existed() {
    // The hash of each sample's compile (text, map, diagnostics, names, kinds, both TeX modes)
    // and of its formatting, taken with the compiler as it was before input lines.
    const BEFORE: &[(&str, u64)] = &[
        ("3D biaxial column interaction", 0xcb6686cf191eda00),
        ("3D footing reactions bar chart", 0x2fbd90a5fd36f409),
        ("3D mode shapes explorer", 0xc3da61c4f185ebab),
        ("3D plate deflection surface", 0x3913bf7be812c36d),
        ("3D space frame geometry", 0x325e79090f6505c1),
        ("Animated beam moving load", 0x52b30ee4a5fa3934),
        ("Beam bending", 0x78eb724dc4d9901e),
        ("Beam deflection derived symbolically", 0x65203ad17a1a88b3),
        ("Beam deflection", 0xa43c19124c0de730),
        ("Beam explorer with Python and Plotly", 0x4dca43234ce4129b),
        ("Cantilever tip load", 0xbb7160ba04bf8ef9),
        ("Column", 0x71d27984a6746167),
        ("LaTeX prose", 0xb825c1d403834f11),
        ("Laguerre polynomial roots", 0x5ffba579fd06a1af),
        ("Loads table", 0xe98ccde9a2b332ed),
        ("Mixing with Calcpad", 0x6642b809d9a868a1),
        ("Plot gallery implicit curves", 0xf236cda174914be7),
        ("Plot", 0xc7953b29c520bc2f),
        ("Tennis ball flight", 0x854dc1cf43307572),
    ];
    fn fnv(s: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
    for (name, want) in BEFORE {
        let src = sample(name);
        let mut acc = String::new();
        for plain in [false, true] {
            let c = compile_with(&src, &Options { plain_tex: plain });
            acc.push_str(&c.text);
            acc.push_str(&format!(
                "{:?}{:?}{}",
                c.map,
                c.diagnostics
                    .iter()
                    .map(|d| (d.line, d.severity == Severity::Error, d.message.clone()))
                    .collect::<Vec<_>>(),
                c.uses_latex
            ));
            acc.push_str(&format!(
                "{:?}",
                c.defs
                    .iter()
                    .map(|d| (d.name.clone(), d.calcpad.clone(), d.line))
                    .collect::<Vec<_>>()
            ));
            acc.push_str(&format!("{:?}", c.kinds));
        }
        acc.push_str(&format(&src));
        assert_eq!(fnv(&acc), *want, "{name}: the compiled output changed");
    }
}

/// The report of a sheet with the values entered in its input form (what the page sends).
fn with_inputs(name: &str, entered: &[(&str, &str)]) -> (Vec<Value>, String) {
    let c = compile(&sample(name));
    let ov: serde_json::Map<String, Value> = entered.iter().map(|(k, v)| (k.to_string(), Value::from(*v))).collect();
    let req = serde_json::json!({ "content": c.text, "uiOverrides": ov });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    (
        v["errors"].as_array().cloned().unwrap_or_default(),
        text(v["html"].as_str().unwrap_or("")),
    )
}

#[test]
fn template_beam_values_and_inputs() {
    let (errs, t) = with_inputs("Template simply supported beam", &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // h = 300 mm: I = 10376.4 cm^4, Z = 691.76 cm^3, own weight 0.502 kN/m, M = 12.5 * 36 / 8.
    for want in [
        "= 10376.4 cm",
        "= 691.76 cm",
        "= 0.502 kN",
        "= 56.26 kN",
        "= 81.33 MPa",
        "= 10.17 mm",
        "= 24 mm",
        "Bending: OK",
        "Deflection: OK",
        "The material is structural steel.",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // Timber (3) over a longer span (10 m) with a 200 mm section fails both checks.
    let (errs, t) = with_inputs(
        "Template simply supported beam",
        &[("L:1", "10"), ("mat:1", "3"), ("h:1", "200mm")],
    );
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("The material is timber."), "{t}");
    assert!(
        t.contains("Bending: the stress exceeds") && t.contains("Deflection: over the limit"),
        "{t}"
    );
    // Own weight off removes it from the load.
    let (_, t) = with_inputs("Template simply supported beam", &[("sw:1", "0")]);
    assert!(t.contains("w tot = w + w sw = 12 kN") && t.contains("= 54 kN"), "{t}");
}

#[test]
fn template_cantilever_values_and_inputs() {
    let (errs, t) = with_inputs("Template cantilever tip load", &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // P L + w L^2 / 2 with w = 7850 * 0.02 m^2 * 9.81 = 1.54 kN/m; delta = 3.69 mm.
    for want in [
        "= 24.81 kN",
        "= 11.85 kN",
        "= 37.22 MPa",
        "= 3.69 mm",
        "= 27.78 mm",
        "Bending: OK",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    let (errs, t) = with_inputs("Template cantilever tip load", &[("P:1", "60"), ("limit:1", "360")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        t.contains("Bending: the stress exceeds") || t.contains("Deflection: over the limit"),
        "{t}"
    );
}

#[test]
fn template_column_values_and_inputs() {
    let (errs, t) = with_inputs("Template column slenderness", &[]);
    assert!(errs.is_empty(), "{errs:?}");
    for want in [
        "= 17.32 mm",
        "= 138.56",
        "= 125.66",
        "= 102.81 MPa",
        "= 616.85 kN",
        "= 308.43 kN",
        "The column is slender",
        "OK: the applied load",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // Fixed at both ends (0.5) makes it intermediate: lambda 69.3, parabolic formula.
    let (errs, t) = with_inputs("Template column slenderness", &[("k:1", "0.5")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        t.contains("= 69.28") && t.contains("The column is intermediate") && t.contains("parabolic formula"),
        "{t}"
    );
    let (_, t) = with_inputs("Template column slenderness", &[("N:1", "400")]);
    assert!(t.contains("Fails: the applied load exceeds"), "{t}");
}

#[test]
fn template_footing_values_and_inputs() {
    let (errs, t) = with_inputs("Template pad footing bearing", &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // P = 600 + 57.6 + 43.2, e = 60 / 700.8, q = 146 * (1 +- 6 e / L).
    for want in [
        "= 700.8 kN",
        "= 0.0856 m",
        "= 146 kPa",
        "= 177.25 kPa",
        "= 114.75 kPa",
        "= 0.886",
        "OK: the largest pressure",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // A large moment lifts the base off the soil: e > L / 6.
    let (errs, t) = with_inputs("Template pad footing bearing", &[("M:1", "400")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        t.contains("Part of the base lifts off the soil.") && t.contains("Fails: the largest pressure"),
        "{t}"
    );
    let (_, t) = with_inputs("Template pad footing bearing", &[("q_a:1", "300kPa")]);
    assert!(t.contains("= 300 kPa"), "{t}");
}

#[test]
fn template_pipe_values_and_inputs() {
    let (errs, t) = with_inputs("Template pipe flow Darcy", &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // V = 0.025 / 0.017671, Re = V D / nu, Swamee-Jain f = 0.0186, h_f = f (L/D) V^2 / 2g.
    for want in [
        "= 1.41 m",
        "= 139610",
        "= 0.0186",
        "= 3.79 m",
        "= 0.255 m",
        "= 9.05 m",
        "= 2.22 kW",
        "The flow is turbulent.",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // A very small flow is laminar, f = 64 / Re.
    let (errs, t) = with_inputs("Template pipe flow Darcy", &[("Q:1", "0.05"), ("temp:1", "1")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("The flow is laminar.") && t.contains("f = 64"), "{t}");
    let (_, t) = with_inputs("Template pipe flow Darcy", &[("Q:1", "100")]);
    assert!(t.contains("High velocity"), "{t}");
}

#[test]
fn the_manifest_marks_the_samples_that_have_inputs() {
    let m: Value = serde_json::from_str(&std::fs::read_to_string(format!("{SAMPLES}manifest.json")).unwrap()).unwrap();
    let mut templates = 0;
    for s in m["samples"].as_array().unwrap() {
        let file = s["file"].as_str().unwrap().trim_start_matches("sheet-samples/");
        let src = std::fs::read_to_string(format!("{SAMPLES}{file}")).unwrap();
        let has_inputs = compile(&src).text.lines().any(|l| l.starts_with("#UI "));
        assert_eq!(
            s["ui"].as_bool().unwrap(),
            has_inputs,
            "{file}: \"ui\" must say whether the sheet has inputs"
        );
        if s["group"] == "Templates" {
            templates += 1;
            assert!(has_inputs, "{file}");
        }
    }
    assert_eq!(templates, 7);
}

#[test]
fn the_documented_input_examples_run() {
    let doc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/sheet-syntax.md")).unwrap();
    let sec = &doc[doc.find("## Inputs and templates").unwrap()..];
    let sec = &sec[..sec.find("### What the compiler writes").unwrap()];
    let mut src = String::from("<!--sheet-->\n");
    let mut n = 0;
    for (i, part) in sec.split("```\n").enumerate() {
        if i % 2 == 1 {
            src.push_str(part);
            n += 1;
        }
    }
    assert_eq!(n, 2, "two examples before the table");
    let (c, html, errs) = render(&src);
    no_errors("docs", &errs, &c);
    assert_eq!(c.text.matches("#UI ").count(), 6);
    assert!(text(&html).contains("The material is aluminium.") || !text(&html).contains("Error"));
    // The table of what is compiled is true as well.
    let rows = [
        ("L: ? 6 m | span", "#UI L = 6m '<span class=\"sheet-note\">span</span>"),
        (
            "k: ? (Low = 1, High = 2)",
            "#UI {\"type\":\"radio\",\"keys\":[\"Low\",\"High\"],\"values\":[\"1\",\"2\"]} 'k -'k = 1",
        ),
        ("ribs: ? on", "#UI {\"type\":\"checkbox\"} 'ribs -'ribs = 1"),
    ];
    for (sheet, want) in rows {
        assert!(lines_of(sheet)[1] == want, "{sheet}");
    }
}

#[test]
fn raw_ui_names_are_renamed_like_every_other_definition() {
    let src = "<!--sheet-->\n>> #UI phi = 0.9\n>> #UI 'Moment -'Mstar = 120 'moment\n>> #UI {\"type\":\"dropdown\",\"keys\":[\"a\",\"b\"],\"values\":[\"1\",\"2\"]} 'Rise -'Delta_T = 2\nx: phi * 2\ny: Mstar * 2\nz: Delta_T * 3\n";
    let (c, html, errs) = render(src);
    no_errors("raw ui names", &errs, &c);
    assert!(c.text.contains("#UI φ = 0.9"), "{}", c.text);
    assert!(c.text.contains("'Moment -'M_star = 120 'moment"), "{}", c.text);
    assert!(c.text.contains("'Rise -'Δ_T = 2"), "{}", c.text);
    let t = text(&html);
    assert!(t.contains("x = φ · 2 = 0.9 · 2 = 1.8"), "{t}");
    assert!(t.contains("y = M star · 2 = 120 · 2 = 240"), "{t}");
    assert!(t.contains("z = Δ T · 3 = 2 · 3 = 6"), "{t}");
}

#[test]
fn input_form_names_with_greek_subscripts_and_digits_run() {
    let src = "<!--sheet-->\nsigma_c: ? 25 MPa\nlambda_1: ? 0.5\nx: sigma_c * lambda_1\n";
    let (c, html, errs) = render(src);
    no_errors("input names", &errs, &c);
    assert!(c.text.contains("σ_c = 25MPa"), "{}", c.text);
    assert!(text(&html).contains("x = σ c · λ 1 = 25 MPa · 0.5 = 12.5 MPa"), "{}", text(&html));
}

#[test]
fn a_raw_ui_name_works_as_an_index_after_a_dot() {
    // A dot is the engine's index operator, never part of a name; a name from a raw #UI line is
    // still the name before or after it (`tab.mat`).
    let src = "<!--sheet-->\n>> #UI mat = 2\ntab: [10, 20, 30]\nv: tab.mat\n>> #UI Delta_T = 3\nw: tab.Delta_T\n";
    let (c, html, errs) = render(src);
    no_errors("dot index", &errs, &c);
    let t = text(&html);
    assert!(t.contains("v = ⃗ tab 2 = 20") && t.contains("w = ⃗ tab 3 = 30"), "{t}");
}

/// The values of the options the form shows as chosen, for the first choice in a compiled sheet.
fn chosen(src: &str) -> Vec<String> {
    let c = compile(src);
    let req = serde_json::json!({ "content": c.text, "enableUi": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    assert!(v["errors"].as_array().unwrap().is_empty(), "{:?}\n{}", v["errors"], c.text);
    let h = v["html"].as_str().unwrap_or("");
    h.split('<')
        .filter(|l| l.len() < 200 && (l.starts_with("option ") || l.starts_with("input type=\"radio\"")))
        .filter(|l| l.contains(" checked") || l.contains(" selected"))
        .map(|l| l.split("value=\"").nth(1).unwrap().split('"').next().unwrap().to_string())
        .collect()
}

#[test]
fn a_radio_group_or_drop_down_opens_on_its_default_whatever_the_unit() {
    for (unit, a, b) in [
        ("kN/m", "10", "25"),
        ("kN*m", "10", "25"),
        ("N/mm^2", "10", "25"),
        ("m^3", "10", "25"),
        ("kN/m^2", "10", "25"),
        ("MPa", "10", "25"),
        ("\u{b0}", "10", "25"),
        ("%", "10", "25"),
        ("", "1.0", "0.90"),
        ("m", "0.100", "2.50"),
    ] {
        for (open, close) in [("(", ")"), ("[", "]")] {
            // The default is the second option, then the first, then no star.
            for (star_a, star_b) in [("", "*"), ("*", ""), ("", "")] {
                let src = format!(
                    "<!--sheet-->\nw: ? {open}{star_a}Low = {a} {unit}, {star_b}High = {b} {unit}{close}\n"
                );
                let want = if star_a == "*" || star_b.is_empty() { a } else { b };
                let got = chosen(&src);
                assert_eq!(got.len(), 1, "{src}: {got:?}");
                let digits: String = got[0].chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                assert_eq!(digits, want, "{src}: {got:?}");
            }
        }
    }
}

#[test]
fn two_options_with_one_value_are_an_error_on_the_input_line() {
    let c = compile("<!--sheet-->\nk: ? (Low = 1 m, High = 1  m)\n");
    assert!(c.diagnostics.iter().any(|d| d.message.contains("same value")), "{:?}", c.diagnostics);
    assert!(c.text.contains("same value"), "{}", c.text);
}

#[test]
fn units_in_inputs_take_the_usual_unicode_spellings() {
    for (written, emitted) in [
        ("5 m\u{b3}", "5m^3"),
        ("5 m^3", "5m^3"),
        ("5 kN\u{b7}m", "5kN*m"),
        ("5 kN\u{22c5}m", "5kN*m"),
        ("5 N/mm\u{b2}", "5N/mm^2"),
        ("5 kg/m\u{b3}", "5kg/m^3"),
        ("5 mm\u{2074}", "5mm^4"),
        ("5 m\u{b7}s\u{207b}\u{b9}", "5m*s^-1"),
        ("5 MPa", "5MPa"),
        ("5 \u{b0}", "5\u{b0}"),
        ("5 \u{b0}C", "5\u{b0}C"),
        ("5%", "5%"),
        ("5 \u{b5}m", "5\u{3bc}m"),
        ("5 \u{3a9}", "5\u{3a9}"),
    ] {
        let src = format!("<!--sheet-->\nv: ? {written}\nw: v * 1\n");
        let (c, html, errs) = render(&src);
        no_errors(written, &errs, &c);
        let ui = c.text.lines().find(|l| l.starts_with("#UI")).unwrap();
        assert_eq!(ui, format!("#UI v = {emitted}"), "{written}");
        assert!(text(&html).contains("w = v"), "{written}: {}", text(&html));
        // The same units in a choice, with the default chosen.
        let src = format!("<!--sheet-->\nv: ? (Low = 3 {u}, *High = {written})\n", u = written.trim_start_matches('5').trim());
        let got = chosen(&src);
        assert_eq!(got.len(), 1, "{written}: {got:?}");
        assert!(got[0].starts_with('5'), "{written}: {got:?}");
    }
}

#[test]
fn saved_values_round_trip_through_the_sheet() {
    // What Save values writes: the comment directly under the marker. Reopening reads it back.
    let original = sample("Template simply supported beam");
    let saved = serde_json::json!({ "uiOverrides": { "L:1": "8", "mat:1": "3", "sw:1": "0" } });
    let comment = format!("'<!--{saved}-->");
    let (marker, rest) = original.split_once('\n').unwrap();
    assert_eq!(marker.trim(), "<!--sheet-->");
    let reopened = format!("{marker}\n{comment}\n{rest}");

    assert!(is_sheet(&reopened));
    let before = compile(&original);
    let c = compile(&reopened);
    assert!(c.diagnostics.iter().all(|d| d.severity != Severity::Error), "{:?}", c.diagnostics);
    assert_eq!(c.text.lines().next(), Some(comment.as_str()));
    // Every input is still an input, in the same place after the comment line.
    let ui = |t: &str| t.lines().filter(|l| l.starts_with("#UI")).map(String::from).collect::<Vec<_>>();
    assert_eq!(ui(&c.text), ui(&before.text));
    assert_eq!(c.text.lines().skip(1).collect::<Vec<_>>(), before.text.lines().collect::<Vec<_>>());

    // The page reads the overrides from the comment and sends them with the request.
    let line = c.text.lines().next().unwrap();
    let json = line.trim_start_matches("'<!--").trim_end_matches("-->");
    let read: Value = serde_json::from_str(json).unwrap();
    assert_eq!(read, saved);
    let req = serde_json::json!({ "content": c.text, "enableUi": true, "uiOverrides": read["uiOverrides"] });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    assert!(v["errors"].as_array().unwrap().is_empty(), "{:?}", v["errors"]);
    let html = v["html"].as_str().unwrap();
    // The form shows the saved values: the entry holds 8, Timber is chosen, own weight is unchecked.
    assert!(html.contains("value=\"8\""), "entry shows 8");
    assert!(html.contains("<option value=\"3\" selected>Timber"), "Timber is chosen");
    let boxes: Vec<&str> = html.split('<').filter(|l| l.starts_with("input") && l.contains("type=\"checkbox\"")).collect();
    assert_eq!(boxes.len(), 1, "{boxes:?}");
    assert!(!boxes[0].contains(" checked"), "own weight is off: {boxes:?}");
    let t = text(html);
    assert!(t.contains("The material is timber."), "{t}");
    // Saving again replaces the comment, it does not stack a second one.
    let again = format!("{marker}\n'<!--{}-->\n{rest}", serde_json::json!({ "uiOverrides": { "L:1": "9" } }));
    let c2 = compile(&again);
    assert_eq!(c2.text.lines().filter(|l| l.starts_with("'<!--{")).count(), 1);
}

#[test]
fn template_retaining_wall_values_and_inputs() {
    let name = "Template retaining wall stability";
    let (errs, t) = with_inputs(name, &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // Medium sand, no surcharge: Ka = 0.3073, P = 0.5 Ka 18 3.4^2 = 31.97 kN, Mo = 36.23 kN.m,
    // W = 28.8 + 23.04 + 75.6 = 127.44 kN, Mr = 179.21 kN.m, mu = tan(21.33) = 0.3906.
    for want in [
        "= 0.307 active",
        "= 31.97 kN",
        "= 36.23 kN",
        "= 127.44 kN",
        "= 179.21 kN",
        "= 4.95 factor of safety against overturning",
        "= 0.391 friction",
        "= 49.77 kN",
        "= 1.56 factor of safety against sliding",
        "Overturning: OK.",
        "Sliding: OK.",
        "inside the middle third",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // The surcharge on: more thrust than the extra weight is worth, and sliding fails.
    let (errs, t) = with_inputs(name, &[("sur:1", "1")]);
    assert!(errs.is_empty(), "{errs:?}");
    for want in ["= 42.41 kN", "= 53.99 kN", "= 141.44 kN", "= 203.01 kN", "= 3.76 factor", "= 1.3 factor", "Sliding: the factor of safety is too low"] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // Dense sand (the radio group) has a smaller thrust and a larger friction.
    let (errs, t) = with_inputs(name, &[("soil:1", "36\u{b0}")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("= 0.26 active") && t.contains("Sliding: OK."), "{t}");
    // A narrow base overturns and lifts.
    let (errs, t) = with_inputs(name, &[("B:1", "1.2"), ("sur:1", "1")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("Overturning: the factor of safety is too low") && t.contains("outside the middle third"), "{t}");
}

#[test]
fn template_pump_values_and_inputs() {
    let name = "Template pump operating point";
    let (errs, t) = with_inputs(name, &[]);
    assert!(errs.is_empty(), "{errs:?}");
    // K = 0.018 * (400 / 0.15) / (2 g A^2) = 7834 s2/m5; Q^2 = 25 / (7834 + 40 / 0.075^2); Q = 40.9 L/s.
    for want in [
        "= 7834.25 s",
        "= 40.9 L",
        "= 28.1 m head at the operating point",
        "= 2.31 m",
        "The velocity is within 3 m/s.",
        "The pump is the medium one.",
        "40 39.29 37.16 33.6 28.62 22.22 14.4",
        "15 15.78 18.13 22.05 27.53 34.59 43.2",
    ] {
        assert!(t.contains(want), "{want}: {t}");
    }
    // An aged pipe has a larger friction factor: less flow.
    let (errs, t) = with_inputs(name, &[("aged:1", "1")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("= 0.03 friction factor") && t.contains("= 35.21 L"), "{t}");
    // The large pump on a 200 mm pipe.
    let (errs, t) = with_inputs(name, &[("pump:1", "3"), ("D:1", "200mm")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("The pump is the large one.") && t.contains("= 66.01 L"), "{t}");
    // A small pump cannot lift 35 m.
    let (errs, t) = with_inputs(name, &[("pump:1", "1"), ("H_st:1", "35")]);
    assert!(errs.is_empty(), "{errs:?}");
    assert!(t.contains("cannot lift the water") && t.contains("= 0 L"), "{t}");
}

// ---- compiled samples stay as they were -------------------------------------------------------

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// The sample with the bounds of its sliders taken out (` [2 .. 12 step 0.5]` is removed).
fn without_bounds(src: &str) -> String {
    let mut out = String::new();
    let mut rest = src;
    while let Some(i) = rest.find(" [") {
        let tail = &rest[i + 2..];
        let close = tail.find(']').unwrap_or(0);
        let inner = &tail[..close];
        if close > 0 && inner.contains(" .. ") && inner.starts_with(|c: char| c.is_ascii_digit()) {
            out.push_str(&rest[..i]);
            rest = &tail[close + 1..];
        } else {
            out.push_str(&rest[..i + 2]);
            rest = tail;
        }
    }
    out.push_str(rest);
    out
}

/// A hash of what the compiler gives for every shipped sample, in both modes, recorded before sliders and
/// collapsible sections existed: a sheet that does not use them compiles to exactly what it did. The two
/// templates that now use sliders compile to the recorded text once their bounds are taken away.
#[test]
fn every_sample_compiles_to_what_it_did_before() {
    let golden = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests-data/samples_compiled.txt")).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(SAMPLES)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "sheet"))
        .collect();
    files.sort();
    let mut want = std::collections::HashMap::new();
    for l in golden.lines() {
        let mut p = l.rsplitn(3, '|');
        let hash = p.next().unwrap().to_string();
        let plain = p.next().unwrap() == "true";
        want.insert((p.next().unwrap().to_string(), plain), hash);
    }
    assert_eq!(want.len(), files.len() * 2, "a sample was added or removed: regenerate tests-data/samples_compiled.txt");
    let mut with_sliders = 0;
    for f in &files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let src = std::fs::read_to_string(f).unwrap();
        let plain_src = without_bounds(&src);
        if plain_src != src {
            with_sliders += 1;
        }
        for plain in [false, true] {
            let c = compile_with(&plain_src, &Options { plain_tex: plain });
            let all = format!(
                "{}\n{:?}\n{:?}",
                c.text,
                c.map,
                c.diagnostics.iter().map(|d| format!("{}:{}", d.line, d.message)).collect::<Vec<_>>()
            );
            assert_eq!(format!("{:016x}", fnv(&all)), want[&(name.clone(), plain)], "{name} (plain {plain}) compiles differently");
        }
    }
    assert_eq!(with_sliders, 2, "two templates use sliders");
}

#[test]
fn the_slider_templates_give_the_same_results_as_their_plain_selves() {
    for name in ["Template simply supported beam", "Template cantilever tip load"] {
        let src = sample(name);
        assert!(src.contains(" .. "), "{name} has sliders");
        let (c1, h1, e1) = render(&src);
        no_errors(name, &e1, &c1);
        let (c2, h2, e2) = render(&without_bounds(&src));
        no_errors(name, &e2, &c2);
        assert_eq!(text(&h1), text(&h2), "{name}: the defaults give the same report");
    }
}

// ---- sliders ----------------------------------------------------------------------------------

fn compiled_line(src: &str) -> String {
    let c = compile(&format!("<!--sheet-->\n{src}\n"));
    assert!(c.diagnostics.is_empty(), "{src}: {:?}", c.diagnostics);
    c.text.lines().find(|l| l.starts_with("#UI")).unwrap_or_else(|| panic!("{}", c.text)).to_string()
}

#[test]
fn a_slider_input_compiles_to_an_entry_with_bounds() {
    assert_eq!(
        compiled_line("L: ? 6 m [2 .. 12 step 0.5]"),
        "#UI {\"min\":2,\"max\":12,\"step\":0.5} L = 6m"
    );
    assert_eq!(compiled_line("L: ? 6 m [2 .. 12]"), "#UI {\"min\":2,\"max\":12} L = 6m");
    assert_eq!(
        compiled_line("w = ? 12 kN/m [-5 .. 40 step 1] | load"),
        "#UI {\"min\":-5,\"max\":40,\"step\":1} w = 12kN/m '<span class=\"sheet-note\">load</span>"
    );
    // The bounds may be written tight, with a minus sign, a decimal point without a leading digit, or a unicode minus.
    assert_eq!(compiled_line("a: ? 0.5 [.25 .. 2 step .25]"), "#UI {\"min\":0.25,\"max\":2,\"step\":0.25} a = 0.5");
    assert_eq!(compiled_line("a: ? 0 [\u{2212}1..1 step 0.1]"), "#UI {\"min\":-1,\"max\":1,\"step\":0.1} a = 0");
    // A plain input and a choice are as they were.
    assert_eq!(compiled_line("L: ? 6 m"), "#UI L = 6m");
}

#[test]
fn a_slider_with_a_mistake_is_an_error_on_its_line() {
    for (src, want) in [
        ("L: ? 20 m [2 .. 12]", "outside the bounds"),
        ("L: ? 1 m [2 .. 12]", "outside the bounds"),
        ("L: ? 6 m [12 .. 2]", "minimum 12 must be below the maximum 2"),
        ("L: ? 6 m [6 .. 6]", "must be below"),
        ("L: ? 6 m [2 .. 12 step 0]", "step must be above zero"),
        ("L: ? 6 m [2 .. 12 step -1]", "step must be above zero"),
        ("L: ? 6 m [2 .. 12 step 20]", "larger than the range"),
        ("L: ? 6 m [2 .. 12 step]", "no step"),
        ("L: ? 6 m [2 .. x]", "maximum 'x' must be a plain number"),
        ("L: ? 6 m [2 m .. 12 m]", "plain number"),
        ("L: ? 6 m [2 12]", "written `[min .. max]"),
        ("L: ? 6 m [.. 12]", "no minimum"),
        ("L: ? 6 m [2 ..]", "no maximum"),
        ("L: ? 6 m [1e1 .. 12]", "plain number"),
    ] {
        let c = compile(&format!("<!--sheet-->\n{src}\n"));
        assert!(
            c.diagnostics.iter().any(|d| d.severity == Severity::Error && d.message.contains(want)),
            "{src}: {want}: {:?}",
            c.diagnostics
        );
        assert!(c.text.contains("Sheet syntax: input:"), "{src}: {}", c.text);
    }
}

#[test]
fn a_slider_line_is_tidied_and_stays_tidy() {
    let f = format("<!--sheet-->\nL:?6 m[2..12 step .5]|span\nw : ? 12 kN/m  [ 2 ..  40 ]\n");
    assert_eq!(f, "<!--sheet-->\nL: ? 6 m [2 .. 12 step 0.5] | span\nw: ? 12 kN/m [2 .. 40]\n");
    assert_eq!(format(&f), f);
}

#[test]
fn a_slider_runs_in_the_engine_with_its_bounds_on_the_entry() {
    let c = compile("<!--sheet-->\nL: ? 6 m [2 .. 12 step 0.5] | span\nA: L * 2\n");
    let req = serde_json::json!({ "content": c.text, "enableUi": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    assert!(v["errors"].as_array().unwrap().is_empty(), "{:?}", v["errors"]);
    let html = v["html"].as_str().unwrap();
    assert!(
        html.contains("data-ui-min=\"2\" data-ui-max=\"12\" data-ui-step=\"0.5\""),
        "the entry carries the bounds"
    );
    // The report (no form) shows the number only.
    let (_, html, errs) = render("<!--sheet-->\nL: ? 6 m [2 .. 12 step 0.5] | span\nA: L * 2\n");
    assert!(errs.is_empty());
    assert!(!html.contains("data-ui-min=") && !html.contains("<input"), "no control in the report");
    // A plain input has no bounds attributes.
    let c = compile("<!--sheet-->\nL: ? 6 m\n");
    let req = serde_json::json!({ "content": c.text, "enableUi": true });
    let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
    assert!(!v["html"].as_str().unwrap().contains("data-ui-min=\""), "a plain entry is as it was");
}

#[test]
fn a_saved_value_outside_the_bounds_is_an_error_on_the_line() {
    let c = compile("<!--sheet-->\nL: ? 6 m [2 .. 12]\nA: L * 2\n");
    for (value, bad) in [("8", false), ("12", false), ("2", false), ("15", true), ("1.5", true), ("-3", true)] {
        let req = serde_json::json!({ "content": c.text, "uiOverrides": { "L:1": value } });
        let v: Value = serde_json::from_str(&calcpad_wasm::convert(&req.to_string(), false)).unwrap();
        let t = text(v["html"].as_str().unwrap());
        assert_eq!(t.contains("is outside the range 2 to 12"), bad, "{value}: {t}");
        if !bad {
            assert!(!t.contains("Error"), "{value}: {t}");
        }
    }
}

// ---- folding sections and key results ---------------------------------------------------------

const FOLD: &str = "<!--sheet-->\n# T\n\nL: ? 6 m | span {key}\n\n## Workings {collapse}\nSome prose.\nw: 12 kN/m | load\nM: w * L^2 / 8 -> kN*m | moment {key}\n\n### Detail\nx: 1\n\n## Results\ny: M * 2 {key}\n\n## Key results\n{key results}\n";

fn out_lines(src: &str, plain: bool) -> (Vec<String>, Vec<(usize, String)>) {
    let c = compile_with(src, &Options { plain_tex: plain });
    (
        c.text.lines().map(str::to_string).collect(),
        c.diagnostics.iter().map(|d| (d.line, d.message.clone())).collect(),
    )
}

#[test]
fn a_collapse_heading_opens_a_details_block_that_closes_at_the_next_heading_of_its_level() {
    let (l, d) = out_lines(FOLD, false);
    assert!(d.is_empty(), "{d:?}");
    let at = |s: &str| l.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s}: {l:?}"));
    let open = at("'<details class=\"sheet-collapse\"><summary>");
    assert_eq!(l[open + 1], "'<h2>Workings</h2>");
    assert_eq!(l[open + 2], "'</summary>");
    let close = at("'</details>");
    // The section holds its prose, its quantities and the deeper heading, and ends before `## Results`.
    assert!(open < at("'Some prose.") && at("\"Detail") < close && close + 1 == at("'<h2>Results</h2>"));
    assert_eq!(l.iter().filter(|x| x.as_str() == "'</details>").count(), 1);
    assert!(!l.iter().any(|x| x.contains("{collapse")), "the marker is not shown");
}

#[test]
fn a_collapse_open_section_starts_open_and_the_last_section_closes_at_the_end() {
    let (l, d) = out_lines("<!--sheet-->\n## A {collapse open}\nx: 1\n## B {collapse}\ny: 2\n", false);
    assert!(d.is_empty(), "{d:?}");
    assert!(l.iter().any(|x| x == "'<details class=\"sheet-collapse\" open><summary>"), "{l:?}");
    assert!(l.iter().any(|x| x == "'<details class=\"sheet-collapse\"><summary>"), "{l:?}");
    assert_eq!(l.iter().filter(|x| x.as_str() == "'</details>").count(), 2);
    assert_eq!(l.last().unwrap(), "'</details>");
}

#[test]
fn nested_collapse_sections_close_in_order() {
    let (l, _) = out_lines("<!--sheet-->\n# Top {collapse}\n## Inner {collapse}\na: 1\n## Next\nb: 2\n# Last\nc: 3\n", false);
    let pos: Vec<usize> = l.iter().enumerate().filter(|(_, x)| x.as_str() == "'</details>").map(|(i, _)| i).collect();
    assert_eq!(pos.len(), 2, "{l:?}");
    // The inner section ends before `## Next`, the outer one before `# Last`.
    assert_eq!(l[pos[0] + 1], "'<h2>Next</h2>");
    assert_eq!(l[pos[1] + 1], "'<h1>Last</h1>");
}

#[test]
fn a_collapse_heading_inside_a_block_is_ignored_with_a_warning() {
    let (l, d) = out_lines("<!--sheet-->\nif 1 == 1:\n    ## Inner {collapse}\n    a: 1\n", false);
    assert!(d.iter().any(|(_, m)| m.contains("outside any if")), "{d:?}");
    assert!(!l.iter().any(|x| x.contains("details")), "{l:?}");
    assert!(!l.iter().any(|x| x.contains("{collapse")), "{l:?}");
}

#[test]
fn a_folded_section_is_calculated_and_in_the_page() {
    let (c, html, errs) = render(FOLD);
    no_errors("fold", &errs, &c);
    assert!(html.contains("<details class=\"sheet-collapse\""), "{html}");
    assert!(html.contains("<summary>") && html.contains("</summary>") && html.contains("</details>"));
    // The section's calculation is in the page and later lines use it.
    let t = text(&html);
    assert!(t.contains("= 12 kN ∕ m · ( 6 m ) 2 8 = 54 kN · m moment"), "{t}");
    assert!(t.contains("y = M · 2 = 54 kN · m · 2 = 108 kN · m"), "{t}");
    // No `open` attribute: it is folded until the reader opens it.
    assert!(!html.contains("<details class=\"sheet-collapse\" open"), "{html}");
}

#[test]
fn a_key_line_gets_a_star_in_its_comment() {
    let (l, d) = out_lines("<!--sheet-->\nM: 2 kN*m | moment {key}\nN: 3 {key}\nP: 4 | note\na: 1; b: 2 | two {key}\n", false);
    assert!(d.is_empty(), "{d:?}");
    let star = "<span class=\"sheet-key\" title=\"key result\">\u{2605}</span>";
    assert_eq!(l[0], format!("M = 2 kN*m '<span class=\"sheet-note\">moment</span> {star}"));
    assert_eq!(l[1], format!("N = 3 '{star}"));
    assert_eq!(l[2], "P = 4 '<span class=\"sheet-note\">note</span>");
    assert_eq!(l[3], "a = 1");
    assert_eq!(l[4], format!("b = 2 '<span class=\"sheet-note\">two</span> {star}"));
    // In plain Calcpad (and so Word and Excel) the star is plain text.
    let (l, _) = out_lines("<!--sheet-->\nM: 2 kN*m | moment {key}\nN: 3 {key}\n", true);
    assert_eq!(l, vec!["M = 2 kN*m 'moment \u{2605}".to_string(), "N = 3 '\u{2605}".to_string()]);
}

#[test]
fn an_input_can_be_a_key_line() {
    let (l, d) = out_lines("<!--sheet-->\nL: ? 6 m [2 .. 12] | span {key}\nE: ? [Steel = 200 GPa, *Timber = 11 GPa] | material {key}\nr: ? on {key}\n", false);
    assert!(d.is_empty(), "{d:?}");
    let ui: Vec<&String> = l.iter().filter(|x| x.starts_with("#UI")).collect();
    assert_eq!(ui.len(), 3);
    assert!(ui.iter().all(|x| x.ends_with("\u{2605}</span>")), "{ui:?}");
    assert!(ui[0].contains("sheet-note\">span</span> <span class=\"sheet-key\""), "{}", ui[0]);
}

#[test]
fn the_key_results_list_has_the_key_lines_above_it() {
    let (l, d) = out_lines(FOLD, false);
    assert!(d.is_empty(), "{d:?}");
    let a = l.iter().position(|x| x == "'<div class=\"sheet-key-results\">").unwrap();
    let list: Vec<&str> = l[a + 1..].iter().take_while(|x| x.as_str() != "'</div>").map(String::as_str).collect();
    assert_eq!(list, vec!["L 'span", "M|kN*m 'moment", "y"]);
    // Plain Calcpad has no wrapper.
    let (l, _) = out_lines(FOLD, true);
    assert!(!l.iter().any(|x| x.contains("sheet-key-results")), "{l:?}");
    assert!(l.contains(&"M|kN*m 'moment".to_string()));
}

#[test]
fn the_key_results_list_runs_in_the_engine_with_the_values() {
    let (c, html, errs) = render(FOLD);
    no_errors("key", &errs, &c);
    let t = text(&html);
    let tail = &t[t.rfind("Key results").unwrap()..];
    assert!(tail.contains("L = 6 m span"), "{tail}");
    assert!(tail.contains("M = 54 kN · m moment"), "{tail}");
    assert!(tail.contains("y = 108 kN · m"), "{tail}");
    assert_eq!(html.matches("class=\"sheet-key\"").count(), 3, "a star on each of the three key lines");
}

#[test]
fn key_lines_that_cannot_be_listed_are_not() {
    // Below the list, in a block, or a function: a star on the line if it is a key line, never in the list.
    let src = "<!--sheet-->\na: 1 {key}\nf(x): x * 2 {key}\nif a == 1:\n    c: 3 {key}\n{key results}\nlater: 5 {key}\n";
    let (l, d) = out_lines(src, true);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(l.iter().position(|x| x == "a"), Some(l.len() - 2), "{l:?}");
    assert_eq!(l.last().unwrap(), "later = 5 '\u{2605}");
    assert_eq!(l.iter().filter(|x| x.as_str() == "a").count(), 1);
    assert!(!l.iter().any(|x| x == "c" || x == "f" || x == "later"), "{l:?}");
    // The three lines that are not listed still carry the star.
    assert_eq!(l.iter().filter(|x| x.ends_with('\u{2605}')).count(), 4, "{l:?}");
}

#[test]
fn a_key_results_line_with_nothing_above_it_is_a_warning() {
    let (l, d) = out_lines("<!--sheet-->\n## Key results\n{key results}\nx: 1\n", false);
    assert!(d.iter().any(|(n, m)| *n == 2 && m.contains("no quantity above this line is marked {key}")), "{d:?}");
    assert!(!l.iter().any(|x| x.contains("sheet-key-results")), "{l:?}");
}

#[test]
fn prose_that_ends_with_the_marker_is_prose() {
    let (l, d) = out_lines("<!--sheet-->\nWrite {key} to mark a line {key}\n", false);
    assert!(d.is_empty(), "{d:?}");
    assert!(l.iter().any(|x| x.contains("{key}")), "{l:?}");
    assert!(!l.iter().any(|x| x.contains("sheet-key")), "{l:?}");
}

#[test]
fn the_marker_lines_are_tidied_and_stay_tidy() {
    let src = "<!--sheet-->\n## Workings   {collapse}\n##   Open one  {collapse open}\nM:2 kN*m|moment   {key}\nx=3  {key}\nL:?6 m [2..12]|span {key}\n{key results}\n";
    let f = format(src);
    assert_eq!(
        f,
        "<!--sheet-->\n## Workings {collapse}\n\n## Open one {collapse open}\nM: 2 kN*m | moment {key}\nx = 3 {key}\nL: ? 6 m [2 .. 12] | span {key}\n{key results}\n"
    );
    assert_eq!(format(&f), f);
    // The formatted sheet compiles to the same worksheet.
    assert_eq!(compile(&f).text.replace(' ', ""), compile(src).text.replace(' ', ""));
}

#[test]
fn a_sheet_without_the_markers_is_untouched_by_the_collapse_and_key_code() {
    // Headings, a trailing brace in a note and braces in LaTeX are as they were.
    let src = "<!--sheet-->\n# T\n## A\nx: 1 | note {braces}\n$a_{key}$\n## B\n";
    let (l, _) = out_lines(src, false);
    assert!(!l.iter().any(|x| x.contains("details") || x.contains("sheet-key")), "{l:?}");
}
