//! A worksheet that a reference includes (its definitions are in force below the reference and are
//! not shown), and solve blocks whose solution the file saved. Synthetic packages built here from
//! strings.

use calcpad_import::{import_bytes, Code, ImportOptions, ImportResult};

// ---- a minimal zip writer ------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

struct Part {
    name: String,
    data: Vec<u8>,
    deflate: bool,
    /// Declared uncompressed size when it should lie.
    declared: Option<u32>,
    flags: u16,
}

fn part(name: &str, data: impl Into<Vec<u8>>) -> Part {
    Part {
        name: name.to_string(),
        data: data.into(),
        deflate: true,
        declared: None,
        flags: 0,
    }
}

fn zip(parts: &[Part]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for p in parts {
        let body = if p.deflate {
            miniz_oxide::deflate::compress_to_vec(&p.data, 6)
        } else {
            p.data.clone()
        };
        let offset = out.len() as u32;
        let method: u16 = if p.deflate { 8 } else { 0 };
        let size = p.declared.unwrap_or(p.data.len() as u32);
        let crc = crc32(&p.data);
        let name = p.name.as_bytes();
        let common = |v: &mut Vec<u8>| {
            v.extend_from_slice(&20u16.to_le_bytes());
            v.extend_from_slice(&p.flags.to_le_bytes());
            v.extend_from_slice(&method.to_le_bytes());
            v.extend_from_slice(&0u16.to_le_bytes());
            v.extend_from_slice(&0x0021u16.to_le_bytes());
            v.extend_from_slice(&crc.to_le_bytes());
            v.extend_from_slice(&(body.len() as u32).to_le_bytes());
            v.extend_from_slice(&size.to_le_bytes());
            v.extend_from_slice(&(name.len() as u16).to_le_bytes());
            v.extend_from_slice(&0u16.to_le_bytes());
        };
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        common(&mut out);
        out.extend_from_slice(name);
        out.extend_from_slice(&body);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        common(&mut central);
        central.extend_from_slice(&0u16.to_le_bytes()); // comment
        central.extend_from_slice(&0u16.to_le_bytes()); // disk
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
        central.extend_from_slice(&0u32.to_le_bytes()); // external attributes
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(parts.len() as u16).to_le_bytes());
    out.extend_from_slice(&(parts.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

// ---- package and worksheet builders -------------------------------------------------------------

const WS_NS: &str = "http://example.invalid/worksheet50";
const ML_NS: &str = "http://example.invalid/math50";

fn worksheet_xml(regions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><worksheet xmlns="{WS_NS}" xmlns:ml="{ML_NS}" xmlns:u="http://example.invalid/units10"><regions>{regions}</regions></worksheet>"#
    )
}

fn rels(entries: &[(&str, &str, &str)]) -> String {
    let body: String = entries
        .iter()
        .map(|(id, kind, target)| {
            format!(r#"<Relationship Type="http://example.invalid/rel/{kind}" Target="{target}" Id="{id}" />"#)
        })
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><Relationships xmlns="http://example.invalid/package/relationships">{body}</Relationships>"#
    )
}

fn include_xml(regions: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><cachedWorksheet xmlns="{WS_NS}" xmlns:ml="{ML_NS}" xmlns:u="http://example.invalid/units10"><regions>{regions}</regions></cachedWorksheet>"#
    )
}

fn package(main_regions: &str, include_regions: Option<&str>) -> ImportResult {
    package_with(main_regions, include_regions, "")
}

fn package_with(main_regions: &str, include_regions: Option<&str>, results: &str) -> ImportResult {
    let mut parts = vec![
        part("_rels/.rels", rels(&[("Rdoc", "officeDocument", "/ws/worksheet.xml")])),
        part("ws/worksheet.xml", worksheet_xml(main_regions)),
        part(
            "ws/_rels/worksheet.xml.rels",
            rels(&[("Rinc", "include", "/ws/include/include8.xml")]),
        ),
        part(
            "ws/result.xml",
            format!(
                r#"<resultsList xmlns:ml="{ML_NS}" xmlns="http://example.invalid/result10">{results}</resultsList>"#
            ),
        ),
    ];
    if let Some(r) = include_regions {
        parts.push(part("ws/include/include8.xml", include_xml(r)));
    }
    import_bytes(&zip(&parts), &ImportOptions::default())
}

fn def(id: u32, top: f64, name: &str, value: &str) -> String {
    format!(
        r#"<region region-id="{id}" top="{top}" left="0" actualWidth="10" actualHeight="0"><math><ml:define><ml:id>{name}</ml:id>{value}</ml:define></math></region>"#
    )
}

fn shown(id: u32, top: f64, name: &str) -> String {
    format!(
        r#"<region region-id="{id}" top="{top}" left="0" actualWidth="10" actualHeight="10"><math><ml:eval><ml:id>{name}</ml:id><ml:unitOverride><ml:placeholder/></ml:unitOverride></ml:eval></math></region>"#
    )
}

const REFERENCE: &str = r#"<region region-id="1" top="0" left="0" width="10" height="20"><reference href="module.mcdx" item-idref="Rinc" /></region>"#;

fn engine_text(code: &str) -> String {
    let mut p = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    p.parse(code, true);
    assert!(!p.html_result.contains("class=\"err\""), "{code}\n{}", p.html_result);
    let mut out = String::new();
    let mut tag = false;
    for c in p.html_result.chars() {
        match c {
            '<' => tag = true,
            '>' => {
                tag = false;
                out.push(' ');
            }
            _ if !tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn a_reference_brings_in_what_the_included_worksheet_defines() {
    let include = [
        def(0, 0.0, "Kinc", "<ml:real>7</ml:real>"),
        shown(1, 10.0, "Kinc"),
        // A function the page below calls.
        r#"<region region-id="2" top="20" left="0" actualWidth="10" actualHeight="0"><math><ml:define><ml:function><ml:id>fTwice</ml:id><ml:boundVars><ml:id>x</ml:id></ml:boundVars></ml:function><ml:apply><ml:mult/><ml:real>2</ml:real><ml:id>x</ml:id></ml:apply></ml:define></math></region>"#.to_string(),
    ]
    .concat();
    let main = [
        REFERENCE.to_string(),
        r#"<region region-id="2" top="40" left="0" actualWidth="10" actualHeight="10"><math><ml:define><ml:id>y</ml:id><ml:apply><ml:id>fTwice</ml:id><ml:sequence><ml:id>Kinc</ml:id></ml:sequence></ml:apply></ml:define></math></region>"#.to_string(),
        shown(3, 60.0, "y"),
    ]
    .concat();
    let res = package(&main, Some(&include));
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.report.gaps().any(|i| i.code == Code::WorksheetReference));
    assert!(engine_text(&res.calcpad).contains("y = 14"), "{}", res.calcpad);
    // Not shown: the included equations leave no equation on the page, only the reference line.
    assert!(res.calcpad.contains("Reference: module.mcdx"), "{}", res.calcpad);
    assert!(!res.calcpad.contains("<mn>7</mn>"), "{}", res.calcpad);
    assert!(!res.calcpad.contains("<mn>2</mn>"), "{}", res.calcpad);
}

#[test]
fn a_reference_with_no_copy_in_the_package_stays_a_gap() {
    let res = package(REFERENCE, None);
    assert!(res.report.gaps().any(|i| i.code == Code::WorksheetReference));
}

#[test]
fn a_gap_in_an_included_definition_is_reported_with_the_regions_of_the_include() {
    let include = def(5, 0.0, "bad", "<ml:symEval><ml:id>x</ml:id></ml:symEval>");
    let res = package(REFERENCE, Some(&include));
    assert!(res
        .report
        .gaps()
        .any(|i| i.code == Code::SymbolicEvaluation && i.region_id.as_deref() == Some("i8_5")));
}

fn solve_block(solver_result: &str) -> ImportResult {
    let block = r#"<region region-id="1" top="0" left="0" height="100" width="300"><solveblock><regions>
        <region region-id="2" top="10" left="0" solve-block-category="guess-value"><math resultRef="1"><ml:define><ml:id>x</ml:id><ml:real>1</ml:real></ml:define></math></region>
        <region region-id="3" top="30" left="0" solve-block-category="constraint"><math resultRef="2"><ml:apply><ml:equal/><ml:apply><ml:mult/><ml:id>x</ml:id><ml:id>x</ml:id></ml:apply><ml:real>4</ml:real></ml:apply></math></region>
        <region region-id="4" top="60" left="0" solve-block-category="solver"><math resultRef="3"><ml:define><ml:id>r</ml:id><ml:eval><ml:apply><ml:id labels="KEYWORD">Find</ml:id><ml:id>x</ml:id></ml:apply><ml:unitOverride><ml:id labels="*">in</ml:id></ml:unitOverride></ml:eval></ml:define></math></region>
        </regions></solveblock></region>"#;
    let after = r#"<region region-id="5" top="150" left="0" actualWidth="10" actualHeight="10"><math><ml:define><ml:id>y</ml:id><ml:apply><ml:plus/><ml:id>r</ml:id><ml:apply><ml:mult/><ml:real>1</ml:real><ml:id labels="UNIT">in</ml:id></ml:apply></ml:apply></ml:define></math></region>"#;
    let main = format!("{block}{after}{}", shown(6, 170.0, "y"));
    package_with(&main, None, solver_result)
}

#[test]
fn a_solve_block_takes_the_solution_the_file_saved() {
    // The solver result is stored as the number in the unit shown (inches).
    let res = solve_block(r#"<resultData result-id="3"><ml:result><ml:real>2</ml:real></ml:result></resultData>"#);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.report.issues.iter().any(|i| i.code == Code::SolveConstraint));
    assert!(res.report.issues.iter().any(|i| i.code == Code::FrozenValue));
    assert!(res.calcpad.contains("r = 2*in"), "{}", res.calcpad);
    assert!(res.calcpad.contains("solve block"), "{}", res.calcpad);
    assert!(engine_text(&res.calcpad).contains("y = 3"), "{}", res.calcpad);
}

#[test]
fn a_solve_block_with_no_saved_solution_and_a_result_in_a_unit_stays_a_gap() {
    // Solved here the result would have no dimension to match the unit it is shown in: a gap that says so.
    let res = solve_block(r#"<resultData result-id="3"/>"#);
    assert!(
        res.report.gaps().any(|i| i.code == Code::SolveBlock),
        "{:?}",
        res.report.issues
    );
}

#[test]
fn a_solve_block_that_cannot_be_solved_here_falls_back_to_the_saved_solution() {
    // No guess for x (the live solver needs a value to start from), a solution saved: the unknown takes it, frozen.
    let block = r#"<region region-id="1" top="0" left="0" height="100" width="300"><solveblock><regions>
        <region region-id="3" top="30" left="0" solve-block-category="constraint"><math resultRef="2"><ml:apply><ml:equal/><ml:apply><ml:mult/><ml:id>x</ml:id><ml:id>x</ml:id></ml:apply><ml:real>4</ml:real></ml:apply></math></region>
        <region region-id="4" top="60" left="0" solve-block-category="solver"><math resultRef="3"><ml:define><ml:id>r</ml:id><ml:apply><ml:id labels="KEYWORD">Find</ml:id><ml:id>x</ml:id></ml:apply></ml:define></math></region>
        </regions></solveblock></region>"#;
    let main = format!("{block}{}", shown(6, 170.0, "r"));
    let res = package_with(
        &main,
        None,
        r#"<resultData result-id="3"><ml:result><ml:real>2</ml:real></ml:result></resultData>"#,
    );
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.report.issues.iter().any(|i| i.code == Code::FrozenValue));
    assert!(res.calcpad.contains("r = 2"), "{}", res.calcpad);
}
