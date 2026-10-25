//! Tests for the mcdx front door on synthetic packages built here from strings. No file of a real
//! worksheet is used and none of the content is taken from one. Math cases are written once and
//! wrapped both as a plain-XML worksheet and as a package, and the two paths must agree.

use calcpad_import::{
    import_bytes, import_mcdx, import_xmcd, Code, ImportOptions, ImportResult, Limits, Severity, GAP_MARKER,
};

mod common;
use common::legacy;

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

fn xaml_package(body: &str) -> Vec<u8> {
    let doc = format!(
        r#"<Section xmlns="http://example.invalid/presentation" xml:space="preserve" FontWeight="Normal">{body}</Section>"#
    );
    zip(&[part("Xaml/Document.xaml", doc)])
}

/// Everything a test may vary in a package.
struct Pkg {
    regions: String,
    origin: i64,
    results: String,
    extra: Vec<Part>,
    rel_list: Vec<(String, String, String)>,
}

impl Pkg {
    fn new(regions: &str) -> Pkg {
        Pkg {
            regions: regions.to_string(),
            origin: 1,
            results: String::new(),
            extra: Vec::new(),
            rel_list: Vec::new(),
        }
    }
    fn rel(mut self, id: &str, kind: &str, target: &str) -> Pkg {
        self.rel_list.push((id.into(), kind.into(), target.into()));
        self
    }
    fn part(mut self, p: Part) -> Pkg {
        self.extra.push(p);
        self
    }
    fn bytes(&self) -> Vec<u8> {
        let rl: Vec<(&str, &str, &str)> = self
            .rel_list
            .iter()
            .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
            .collect();
        let mut parts = vec![
            part("_rels/.rels", rels(&[("Rdoc", "officeDocument", "/ws/worksheet.xml")])),
            part("ws/worksheet.xml", worksheet_xml(&self.regions)),
            part("ws/_rels/worksheet.xml.rels", rels(&rl)),
            part(
                "ws/settings/calculation.xml",
                format!(
                    r#"<calculation xmlns="http://example.invalid/settings/calculation10"><builtInVariables array-origin="{}" /></calculation>"#,
                    self.origin
                ),
            ),
            part(
                "ws/result.xml",
                format!(
                    r#"<resultsList xmlns:ml="{ML_NS}" xmlns:u="http://example.invalid/units10" xmlns="http://example.invalid/result10">{}</resultsList>"#,
                    self.results
                ),
            ),
        ];
        for p in &self.extra {
            parts.push(Part {
                name: p.name.clone(),
                data: p.data.clone(),
                deflate: p.deflate,
                declared: p.declared,
                flags: p.flags,
            });
        }
        zip(&parts)
    }
    fn import(&self) -> ImportResult {
        import_bytes(&self.bytes(), &ImportOptions::default())
    }
}

fn mregion(id: u32, top: f64, inner: &str) -> String {
    format!(
        r#"<region region-id="{id}" top="{top}" left="0" actualWidth="10" actualHeight="10"><math resultRef="{id}">{inner}</math></region>"#
    )
}

/// The same math as a plain-XML worksheet.
fn xmcd_of(inners: &[&str]) -> String {
    let regions: String = inners
        .iter()
        .enumerate()
        .map(|(i, inner)| {
            let top = i as f64 * 20.0;
            format!(
                r#"<region region-id="{i}" left="0" top="{top}" width="10" height="10" align-y="{}"><math>{inner}</math></region>"#,
                top + 10.0
            )
        })
        .collect();
    format!(
        r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30" xmlns:u="http://example.invalid/units10"><settings><calculation><builtInVariables array-origin="1"/></calculation></settings><regions>{regions}</regions></worksheet>"#
    )
}

fn mcdx_of(inners: &[&str]) -> ImportResult {
    let regions: String = inners
        .iter()
        .enumerate()
        .map(|(i, inner)| mregion(i as u32, i as f64 * 20.0, inner))
        .collect();
    Pkg::new(&regions).import()
}

/// The output lines without the banner. A gap is one line again: its marker comment (as
/// `'[import-gap Rn codes] reason`) and, after ` | `, the lines of its equation block.
fn lines(res: &ImportResult) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_noc = false;
    let mut in_gap = false;
    let code = legacy(&res.calcpad);
    for l in code.lines().filter(|l| {
        !l.starts_with("'<!-- Imported") && !l.starts_with("'<style>") && !l.starts_with("'<!--{\"LintIgnore")
    }) {
        if let Some(rest) = l.strip_prefix("'<!-- [import-gap") {
            let head = rest.split(" --><small>").next().unwrap_or(rest);
            out.push(format!("'[import-gap{}", head.replacen(" -->", "", 1)));
            in_gap = true;
            continue;
        }
        if in_gap {
            if l == "#noc" {
                in_noc = true;
                continue;
            }
            if l == "#end noc" {
                in_noc = false;
                continue;
            }
            if in_noc || l.starts_with("'<code>") || l.starts_with("'&emsp;") {
                let last = out.last_mut().unwrap();
                last.push_str(if last.contains(" | ") { " ; " } else { " | " });
                last.push_str(l.trim_start_matches('\''));
                continue;
            }
            in_gap = false;
        }
        out.push(l.to_string());
    }
    out
}

/// Imports the same math through both front doors and checks that they agree on the text and on
/// the codes of the issues. Returns the package result.
fn both(inners: &[&str]) -> ImportResult {
    let old = import_xmcd(&xmcd_of(inners));
    let new = mcdx_of(inners);
    assert_eq!(lines(&old), lines(&new), "output differs");
    let codes = |r: &ImportResult| r.report.issues.iter().map(|i| (i.code, i.severity)).collect::<Vec<_>>();
    assert_eq!(codes(&old), codes(&new), "issues differ");
    assert_eq!(old.report.regions_total(), new.report.regions_total());
    new
}

// ---- math -----------------------------------------------------------------------------------------

#[test]
fn assignment_and_evaluation() {
    let res = both(&[
        r#"<ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:mult/><ml:id>x</ml:id><ml:real>3</ml:real></ml:apply></ml:eval>"#,
    ]);
    assert_eq!(lines(&res), vec!["x = 2", "x*3"]);
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn arithmetic_with_units_scale_and_placeholder() {
    // The package writes a value and its unit as a `scale` application; the plain file as a product.
    let new = mcdx_of(&[
        r#"<ml:define><ml:id>L</ml:id><ml:apply><ml:scale/><ml:real>4</ml:real><ml:id labels="UNIT">m</ml:id></ml:apply></ml:define>"#,
        r#"<ml:define><ml:id>F</ml:id><ml:apply><ml:scale/><ml:real>10</ml:real><ml:id labels="UNIT">kN</ml:id></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:mult/><ml:id>F</ml:id><ml:id>L</ml:id></ml:apply><ml:unitOverride><ml:placeholder/></ml:unitOverride></ml:eval>"#,
    ]);
    let old = import_xmcd(&xmcd_of(&[
        r#"<ml:define><ml:id>L</ml:id><ml:apply><ml:mult/><ml:real>4</ml:real><ml:id>m</ml:id></ml:apply></ml:define>"#,
        r#"<ml:define><ml:id>F</ml:id><ml:apply><ml:mult/><ml:real>10</ml:real><ml:id>kN</ml:id></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:mult/><ml:id>F</ml:id><ml:id>L</ml:id></ml:apply></ml:eval>"#,
    ]));
    assert_eq!(lines(&new), lines(&old));
    assert_eq!(lines(&new), vec!["L = 4*m", "F = 10*kN", "F*L"]);
    assert_eq!(new.report.regions_needing_attention(), 0);
}

#[test]
fn unit_override_with_a_unit_is_a_conversion() {
    let res = both(&[
        r#"<ml:define><ml:id>p</ml:id><ml:apply><ml:mult/><ml:real>100</ml:real><ml:id>kPa</ml:id></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:id>p</ml:id><ml:unitOverride><ml:id>psi</ml:id></ml:unitOverride></ml:eval>"#,
    ]);
    assert_eq!(lines(&res)[1], "(p/psi)*psi");
    assert!(res.report.notes().any(|i| i.code == Code::UnitOverride));
}

#[test]
fn function_definition_and_call() {
    let res = both(&[
        r#"<ml:define><ml:function><ml:id>f</ml:id><ml:boundVars><ml:id>a</ml:id><ml:id>b</ml:id></ml:boundVars></ml:function><ml:apply><ml:plus/><ml:id>a</ml:id><ml:apply><ml:pow/><ml:id>b</ml:id><ml:real>2</ml:real></ml:apply></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:id>f</ml:id><ml:sequence><ml:real>1</ml:real><ml:real>3</ml:real></ml:sequence></ml:apply></ml:eval>"#,
    ]);
    assert_eq!(lines(&res), vec!["f(a; b) = a + b^2", "f(1; 3)"]);
}

#[test]
fn single_argument_call_has_no_sequence() {
    // The package writes a one-argument call without a `sequence` wrapper.
    let res = mcdx_of(&[
        r#"<ml:define><ml:function><ml:id>g</ml:id><ml:boundVars><ml:id>t</ml:id></ml:boundVars></ml:function><ml:apply><ml:mult/><ml:real>2</ml:real><ml:id>t</ml:id></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:id>g</ml:id><ml:real>5</ml:real></ml:apply></ml:eval>"#,
    ]);
    assert_eq!(lines(&res), vec!["g(t) = 2*t", "g(5)"]);
}

#[test]
fn matrix_is_column_major() {
    let res = both(&[
        r#"<ml:define><ml:id>M</ml:id><ml:matrix rows="2" cols="3"><ml:real>1</ml:real><ml:real>2</ml:real><ml:real>3</ml:real><ml:real>4</ml:real><ml:real>5</ml:real><ml:real>6</ml:real></ml:matrix></ml:define>"#,
    ]);
    // Items 1,2 are the first column: rows read 1 3 5 and 2 4 6.
    assert_eq!(lines(&res), vec!["M = [1; 3; 5|2; 4; 6]"]);
}

#[test]
fn program_with_if_and_for_converts_alike_in_both_paths() {
    let prog = r#"<ml:define><ml:id>y</ml:id><ml:program><ml:localDefine><ml:id>s</ml:id><ml:real>0</ml:real></ml:localDefine><ml:for><ml:id>i</ml:id><ml:range><ml:real>1</ml:real><ml:real>4</ml:real></ml:range><ml:program><ml:if><ml:test><ml:apply><ml:lessThan/><ml:id>i</ml:id><ml:real>3</ml:real></ml:apply></ml:test><ml:then><ml:program><ml:localDefine><ml:id>s</ml:id><ml:id>i</ml:id></ml:localDefine></ml:program></ml:then></ml:if></ml:program></ml:for><ml:id>s</ml:id></ml:program></ml:define>"#;
    let res = both(&[prog]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("#for i_p1 = 1 : 4"), "{}", res.calcpad);
    assert!(res.calcpad.contains("#if i_p1 < 3"), "{}", res.calcpad);
}

#[test]
fn program_with_try_catch_is_a_reported_gap_in_both_paths() {
    let prog = r#"<ml:define><ml:id>y</ml:id><ml:program><ml:tryCatch><ml:program><ml:localDefine><ml:id>s</ml:id><ml:real>1</ml:real></ml:localDefine></ml:program><ml:program><ml:real>0</ml:real></ml:program></ml:tryCatch><ml:id>s</ml:id></ml:program></ml:define>"#;
    let res = both(&[prog]);
    assert_eq!(res.report.regions_needing_attention(), 1);
    assert!(res.report.gaps().any(|i| i.code == Code::ProgramBlock));
    assert!(lines(&res)[0].starts_with(&format!("'{GAP_MARKER} R0 program-block]")));
}

#[test]
fn range_variable_used_outside_a_fill_loop_is_a_reported_gap_in_both_paths() {
    let res = both(&[
        r#"<ml:define><ml:id>k</ml:id><ml:range><ml:real>1</ml:real><ml:real>10</ml:real></ml:range></ml:define>"#,
        r#"<ml:define><ml:id>z</ml:id><ml:apply><ml:mult/><ml:id>k</ml:id><ml:real>2</ml:real></ml:apply></ml:define>"#,
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::RangeVariable));
    assert!(res.report.notes().any(|i| i.code == Code::RangeLoop));
}

#[test]
fn subscript_span_inside_an_identifier() {
    let res = mcdx_of(&[
        r#"<ml:define><ml:id><Span xmlns="http://example.invalid/presentation" xmlns:pw="clr-namespace:Example">T<pw:Subscript>max</pw:Subscript></Span></ml:id><ml:real>7</ml:real></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:mult/><ml:real>2</ml:real><ml:id><Span xmlns="http://example.invalid/presentation" xmlns:pw="clr-namespace:Example">T<pw:Subscript>max</pw:Subscript></Span></ml:id></ml:apply></ml:eval>"#,
    ]);
    assert_eq!(lines(&res), vec!["T_max = 7", "2*T_max"]);
}

#[test]
fn indexing_uses_the_origin_from_the_settings_part() {
    let region = [
        mregion(
            0,
            0.0,
            r#"<ml:define><ml:id>v</ml:id><ml:matrix rows="3" cols="1"><ml:real>1</ml:real><ml:real>2</ml:real><ml:real>3</ml:real></ml:matrix></ml:define>"#,
        ),
        mregion(
            1,
            20.0,
            r#"<ml:eval><ml:apply><ml:indexer/><ml:id>v</ml:id><ml:real>2</ml:real></ml:apply></ml:eval>"#,
        ),
    ]
    .concat();
    let mut p = Pkg::new(&region);
    p.origin = 0;
    let at0 = lines(&p.import());
    p.origin = 1;
    let at1 = lines(&p.import());
    assert_ne!(at0, at1);
    assert_eq!(at0[1], "v.(3)");
    assert_eq!(at1[1], "v.(2)");
}

#[test]
fn unmapped_unit_and_unknown_element_are_reported() {
    let res = mcdx_of(&[
        r#"<ml:define><ml:id>q</ml:id><ml:apply><ml:scale/><ml:real>2</ml:real><ml:id labels="UNIT">kmol</ml:id></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:wobble/><ml:real>2</ml:real></ml:apply></ml:eval>"#,
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::UnmappedUnit));
    assert!(res.report.gaps().any(|i| i.code == Code::UnknownElement));
}

#[test]
fn summation_and_derivative_are_named_gaps() {
    let res = mcdx_of(&[
        r#"<ml:eval><ml:apply><ml:summation/><ml:id>a</ml:id></ml:apply></ml:eval>"#,
        r#"<ml:eval><ml:apply><ml:derivative/><ml:id>a</ml:id></ml:apply></ml:eval>"#,
    ]);
    assert!(res.report.gaps().any(|i| i.code == Code::SummationOperator));
    assert!(res.report.gaps().any(|i| i.code == Code::CalculusOperator));
}

// ---- text regions -----------------------------------------------------------------------------------

fn text_region(id: u32, top: f64, rel: &str, nested: &str) -> String {
    let regions = if nested.is_empty() {
        String::new()
    } else {
        format!("<regions>{nested}</regions>")
    };
    format!(
        r#"<region region-id="{id}" top="{top}" left="0" width="400" actualWidth="400" actualHeight="20"><text item-idref="{rel}"><FlowDocument xmlns="http://example.invalid/presentation" />{regions}</text></region>"#
    )
}

#[test]
fn text_in_the_symbol_font_is_greek() {
    let xaml = xaml_package(
        r#"<Paragraph><Run>Since </Run><Run FontFamily="Symbol">S </Run><Run>F</Run><Run FontFamily="Symbol">a</Run></Paragraph>"#,
    );
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    let l = res.calcpad.lines().find(|x| x.contains("Since")).expect("line");
    assert!(l.contains("\u{3A3}") && l.contains("\u{3B1}"), "{l}");
    assert!(!l.contains("Symbol"), "{l}");
}

#[test]
fn text_region_reads_font_size_colour_and_alignment() {
    // 16 px is 12 pt; the colour is #AARRGGBB.
    let xaml = xaml_package(
        r##"<Paragraph TextAlignment="Center"><Run FontFamily="Arial" FontSize="16" Foreground="#FF0000FF">blue text</Run></Paragraph>"##,
    );
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    let l = res.calcpad.lines().find(|x| x.contains("blue text")).expect("line");
    assert!(l.contains("text-align:center"), "{l}");
    assert!(
        l.contains("font-family:Arial,Helvetica") && l.contains("font-size:12pt"),
        "{l}"
    );
    assert!(l.contains("imp-c-0000ff"), "{l}");
}

#[test]
fn text_region_reads_the_flow_document() {
    let xaml = xaml_package(
        r#"<Paragraph><Run FontWeight="Bold">Beam </Run><Run>check</Run><Run FontStyle="Italic"> (draft)</Run><LineBreak /><Run TextDecorations="Underline">second line</Run></Paragraph><Paragraph><Run>   </Run></Paragraph>"#,
    );
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    assert_eq!(
        lines(&res),
        vec!["'<p><b>Beam </b>check<i> (draft)</i><br/><u>second line</u></p>"]
    );
    assert_eq!(res.report.regions_total(), 1);
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn inline_flow_document_is_read_like_the_package_form() {
    // Older files hold the flow document inside the text element; no relationship, no part.
    let region = r#"<region region-id="1" top="0" left="0" width="400"><text is-text-block="true"><FlowDocument xmlns="http://example.invalid/presentation"><Paragraph Margin="0,0,0,0"><Run FontWeight="Bold">Beam </Run><Run>check</Run><Run><Run.TextDecorations><TextDecorationCollection><TextDecoration Location="Underline" /></TextDecorationCollection></Run.TextDecorations> underlined</Run></Paragraph><List MarkerStyle="Decimal"><ListItem><Paragraph><Run>one</Run></Paragraph></ListItem></List></FlowDocument></text></region>"#;
    let res = Pkg::new(region).import();
    assert_eq!(
        lines(&res),
        vec!["'<p><b>Beam </b>check<u> underlined</u></p>", "'1. one"]
    );
    assert_eq!(res.report.regions_needing_attention(), 0);
    assert!(!res.report.issues.iter().any(|i| i.code == Code::TextUnreadable));
}

#[test]
fn lists_hyperlinks_and_spans() {
    let xaml = xaml_package(
        r#"<List MarkerStyle="Decimal"><ListItem><Paragraph><Run>first</Run></Paragraph></ListItem><ListItem><Paragraph><Run>second</Run></Paragraph></ListItem></List><List MarkerStyle="Disc"><ListItem><Paragraph><Span FontWeight="Bold"><Run>point</Run></Span></Paragraph></ListItem></List><Paragraph><Hyperlink NavigateUri="other.mcdx"><Run>see also</Run></Hyperlink></Paragraph>"#,
    );
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    assert_eq!(
        lines(&res),
        vec!["'1. first", "'2. second", "'&bull; <b>point</b>", "'see also"]
    );
}

#[test]
fn text_is_escaped_and_cannot_start_a_directive() {
    let xaml = xaml_package(r##"<Paragraph><Run>#include other &amp; &lt;b&gt;</Run></Paragraph>"##);
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    let l = lines(&res);
    assert_eq!(l.len(), 1);
    assert!(l[0].starts_with('\''), "{:?}", l);
    assert!(l[0].contains("&lt;b&gt;"), "{:?}", l);
}

#[test]
fn math_nested_in_a_text_region_follows_the_text() {
    let nested = r#"<region region-id="9" actualWidth="10" actualHeight="10"><math resultRef="9"><ml:define><ml:id>w</ml:id><ml:real>5</ml:real></ml:define></math></region>"#;
    let xaml = xaml_package(r#"<Paragraph><Run>Width below</Run></Paragraph>"#);
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", nested))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    assert_eq!(lines(&res), vec!["'Width below", "w = 5"]);
    assert_eq!(res.report.kinds["text"].total, 1);
    assert_eq!(res.report.kinds["math"].total, 1);
}

#[test]
fn xaml_elements_that_would_hide_content_are_reported() {
    let xaml = xaml_package(
        r#"<Paragraph><Run>before</Run><InlineUIContainer><Image Source="x.png" /></InlineUIContainer></Paragraph><Table><TableRowGroup /></Table>"#,
    );
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    let gaps: Vec<_> = res.report.gaps().filter(|i| i.code == Code::RichTextElement).collect();
    assert_eq!(gaps.len(), 2, "{:?}", res.report.issues);
    let details: Vec<_> = gaps.iter().filter_map(|i| i.detail.clone()).collect();
    assert!(details.contains(&"InlineUIContainer".to_string()), "{details:?}");
    assert!(details.contains(&"Table".to_string()), "{details:?}");
    assert!(res
        .calcpad
        .contains(&format!("'<!-- {GAP_MARKER} R1 rich-text-element]")));
    // The prose around them still converts.
    assert!(res.calcpad.contains("'before"));
}

#[test]
fn unreadable_text_is_reported_not_dropped() {
    // The relationship points at a part that is not in the package.
    let res = Pkg::new(&text_region(1, 0.0, "Rmissing", ""))
        .rel("Rmissing", "flowDocument", "/ws/xaml/Nothing.XamlPackage")
        .import();
    assert!(res.report.gaps().any(|i| i.code == Code::TextUnreadable));
    assert!(res.calcpad.contains("text-unreadable]"));
    // A text region with no relationship at all is the same.
    let res = Pkg::new(&text_region(2, 0.0, "Rnone", "")).import();
    assert!(res.report.gaps().any(|i| i.code == Code::TextUnreadable));
}

// ---- things that are skipped, with a reason -----------------------------------------------------------

#[test]
fn objects_components_references_plots_and_page_furniture_are_reported() {
    let regions = [
        r#"<region region-id="1" top="0" left="0" width="10" height="10"><oleObject item-idref="Ro"><OleViewportSize width="0" height="0" /></oleObject></region>"#,
        r#"<region region-id="2" top="20" left="0" width="10" height="10"><excelComponent resultRef="1"><regions><region region-id="3" top="1" left="1"><math resultRef="3"><ml:define><ml:id>inner</ml:id><ml:real>1</ml:real></ml:define></math></region></regions></excelComponent></region>"#,
        r#"<region region-id="4" top="40" left="0" width="10" height="10"><chartComponent resultRef="2" /></region>"#,
        r#"<region region-id="5" top="60" left="0" width="10" height="10"><reference href=".\other.mcdx" item-idref="Rr" /></region>"#,
        r#"<region region-id="6" top="80" left="0" width="10" height="10"><plot><graph /></plot></region>"#,
        r#"<region region-id="7" top="100" left="0" width="10" height="2"><pageBreak /></region>"#,
        r#"<region region-id="8" top="120" left="0" width="10" height="10"><mysteryKind /></region>"#,
    ]
    .concat();
    let res = Pkg::new(&regions)
        .part(part(
            "ws/header.xml",
            worksheet_xml(r#"<region region-id="h1" top="0" left="0"><pageBreak /></region>"#),
        ))
        .import();
    let has = |c: Code, id: &str| {
        res.report
            .gaps()
            .any(|i| i.code == c && i.region_id.as_deref() == Some(id))
    };
    assert!(has(Code::OleObject, "1"));
    assert!(has(Code::Component, "2"));
    assert!(has(Code::Component, "4"));
    assert!(has(Code::WorksheetReference, "5"));
    assert!(has(Code::Plot, "6"));
    assert!(has(Code::OtherRegion, "8"));
    assert!(has(Code::HeaderFooter, "header"));
    assert_eq!(res.report.page_breaks, 1);
    // Each one has a visible placeholder line, and the component's inner region is not emitted
    // as a free-standing definition.
    for tag in [
        "ole-object",
        "component",
        "worksheet-reference",
        "plot",
        "other-region",
        "header-footer",
    ] {
        assert!(res.calcpad.contains(&format!("{tag}]")), "no placeholder for {tag}");
    }
    assert!(!res.calcpad.contains("inner"));
    assert_eq!(res.report.regions_needing_attention(), 7);
}

#[test]
fn a_collapsible_area_is_read_through() {
    let regions = r#"<region region-id="1" top="100" left="0" width="600" height="1"><Area IsCollapsed="true"><regions><region region-id="2" top="30" left="10"><math resultRef="2"><ml:define><ml:id>b</ml:id><ml:real>2</ml:real></ml:define></math></region><region region-id="3" top="10" left="10"><math resultRef="3"><ml:define><ml:id>a</ml:id><ml:real>1</ml:real></ml:define></math></region></regions></Area></region>
        <region region-id="4" top="50" left="0"><math resultRef="4"><ml:define><ml:id>c</ml:id><ml:real>0</ml:real></ml:define></math></region>"#;
    let res = Pkg::new(regions).import();
    // None of the three has a size, so they are calculated in the order the file lists them.
    assert_eq!(lines(&res), vec!["b = 2", "a = 1", "c = 0"]);
    assert_eq!(res.report.regions_needing_attention(), 0);
}

#[test]
fn a_collapsed_area_is_calculated_at_its_place_before_what_follows_it() {
    // The area is closed on the page, so the region after it sits just below its header (top 130),
    // while the area's own regions are positioned as if it were open (down to 5000). The sheet is
    // calculated with the area's regions where the area is, so z may read y.
    let regions = r#"<region region-id="1" top="100" left="0" width="600" height="1"><Area IsCollapsed="true"><regions><region region-id="2" top="10" left="10"><math resultRef="2"><ml:define><ml:id>x</ml:id><ml:real>1</ml:real></ml:define></math></region><region region-id="3" top="5000" left="10"><math resultRef="3"><ml:define><ml:id>y</ml:id><ml:apply><ml:plus/><ml:id>x</ml:id><ml:real>1</ml:real></ml:apply></ml:define></math></region></regions></Area></region>
        <region region-id="4" top="130" left="0"><math resultRef="4"><ml:define><ml:id>z</ml:id><ml:apply><ml:plus/><ml:id>y</ml:id><ml:real>1</ml:real></ml:apply></ml:define></math></region>"#;
    let res = Pkg::new(regions).import();
    assert_eq!(lines(&res), vec!["x = 1", "y = x + 1", "z = y + 1"]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
}

#[test]
fn an_open_area_leaves_the_regions_after_it_where_they_are() {
    // Open, the page already has the next region below the area's content.
    let regions = r#"<region region-id="1" top="100" left="0" width="600" height="1"><Area IsCollapsed="false"><regions><region region-id="2" top="10" left="10"><math resultRef="2"><ml:define><ml:id>x</ml:id><ml:real>1</ml:real></ml:define></math></region><region region-id="3" top="40" left="10"><math resultRef="3"><ml:define><ml:id>y</ml:id><ml:apply><ml:plus/><ml:id>x</ml:id><ml:real>1</ml:real></ml:apply></ml:define></math></region></regions></Area></region>
        <region region-id="4" top="400" left="0"><math resultRef="4"><ml:define><ml:id>z</ml:id><ml:apply><ml:plus/><ml:id>y</ml:id><ml:real>1</ml:real></ml:apply></ml:define></math></region>"#;
    let res = Pkg::new(regions).import();
    assert_eq!(lines(&res), vec!["x = 1", "y = x + 1", "z = y + 1"]);
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
}

// ---- pictures and saved results ---------------------------------------------------------------------

fn tiny_png() -> Vec<u8> {
    // The 8-byte signature plus a chunk-shaped tail: enough for the importer's signature check.
    let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
    v.extend_from_slice(&[0, 0, 0, 13]);
    v.extend_from_slice(b"IHDR");
    v.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]);
    v.extend_from_slice(&[0, 0, 0, 0]);
    v
}

#[test]
fn picture_region_becomes_an_asset() {
    let region = r#"<region region-id="4" top="0" left="0" width="120" height="60" actualWidth="120" actualHeight="60"><picture><png item-idref="Rp" display-width="120" display-height="60" /></picture></region>"#;
    let mut png = part("ws/images/image1.png", tiny_png());
    png.deflate = false;
    let res = Pkg::new(region)
        .rel("Rp", "image", "/ws/images/image1.png")
        .part(png)
        .import();
    assert_eq!(res.assets.len(), 1);
    assert!(res.assets[0].path.ends_with(".png"));
    assert!(res.calcpad.contains("<img"));
    // A picture whose part is missing is a reported gap.
    let res = Pkg::new(region).import();
    assert!(res.assets.is_empty());
    assert!(res.report.gaps().any(|i| i.code == Code::Image));
}

#[test]
fn saved_results_come_from_the_result_part() {
    // An unconvertible region with a stored scalar result: shown only when the option is on.
    let region = mregion(
        3,
        0.0,
        r#"<ml:eval><ml:apply><ml:id>vendorCall</ml:id><ml:real>1</ml:real></ml:apply></ml:eval>"#,
    );
    let mut p = Pkg::new(&region);
    p.results = r#"<resultData result-id="3"><ml:result><ml:real>647.5</ml:real></ml:result></resultData>"#.into();
    let off = import_bytes(&p.bytes(), &ImportOptions::default());
    assert_eq!(off.report.pictures.saved_result_available, 1);
    assert!(!off.calcpad.contains("647.5"));
    let on = import_bytes(
        &p.bytes(),
        &ImportOptions {
            saved_results: true,
            ..ImportOptions::default()
        },
    );
    assert!(on.calcpad.contains("647.5"), "{}", on.calcpad);
}

// ---- the container ------------------------------------------------------------------------------------

#[test]
fn zip_signature_routes_to_the_package_reader() {
    let bytes = Pkg::new(&mregion(
        0,
        0.0,
        r#"<ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define>"#,
    ))
    .bytes();
    assert!(bytes.starts_with(b"PK\x03\x04"));
    let a = import_bytes(&bytes, &ImportOptions::default());
    let b = import_mcdx(&bytes, &ImportOptions::default());
    assert_eq!(a, b);
    assert_eq!(lines(&a), vec!["x = 2"]);
    // And a plain-XML worksheet still goes the old way.
    let old = import_bytes(
        xmcd_of(&[r#"<ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define>"#]).as_bytes(),
        &ImportOptions::default(),
    );
    assert_eq!(lines(&old), vec!["x = 2"]);
}

#[test]
fn stored_entries_and_a_conventional_name_without_relationships() {
    // No package-level relationships: the conventional part name is used. Entries are stored.
    let mut ws = part(
        "ws/worksheet.xml",
        worksheet_xml(&mregion(
            0,
            0.0,
            r#"<ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define>"#,
        )),
    );
    ws.deflate = false;
    let res = import_bytes(&zip(&[ws]), &ImportOptions::default());
    assert_eq!(lines(&res), vec!["x = 2"]);
    // No settings part: the origin defaults to 0 like the older format's default.
    assert_eq!(res.report.origin, 0);
}

fn fatal_code(res: &ImportResult) -> Code {
    assert!(res.report.is_fatal(), "not fatal: {:?}", res.report.issues);
    assert!(res.calcpad.is_empty());
    let i = &res.report.issues[0];
    assert_eq!(i.severity, Severity::Fatal);
    i.code
}

#[test]
fn damaged_and_foreign_zips_are_fatal_with_a_code() {
    // Signature only.
    assert_eq!(
        fatal_code(&import_bytes(b"PK\x03\x04zzzz", &ImportOptions::default())),
        Code::UnsupportedContainer
    );
    // Truncated package.
    let good = Pkg::new("").bytes();
    let cut = &good[..good.len() / 2];
    assert_eq!(
        fatal_code(&import_bytes(cut, &ImportOptions::default())),
        Code::UnsupportedContainer
    );
    // A valid zip without a worksheet.
    let other = zip(&[part("readme.txt", "hello")]);
    assert_eq!(
        fatal_code(&import_bytes(&other, &ImportOptions::default())),
        Code::NotAWorksheet
    );
    // A worksheet part that is not XML.
    let junk = zip(&[part("ws/worksheet.xml", "not xml <")]);
    assert_eq!(
        fatal_code(&import_bytes(&junk, &ImportOptions::default())),
        Code::XmlError
    );
    // A part that is XML but not a worksheet.
    let wrong = zip(&[part("ws/worksheet.xml", "<other/>")]);
    assert_eq!(
        fatal_code(&import_bytes(&wrong, &ImportOptions::default())),
        Code::NotAWorksheet
    );
    // A DTD is refused.
    let dtd = zip(&[part(
        "ws/worksheet.xml",
        r#"<?xml version="1.0"?><!DOCTYPE d [<!ENTITY a "aaaa">]><worksheet xmlns="http://example.invalid/worksheet50"><regions/></worksheet>"#,
    )]);
    assert_eq!(
        fatal_code(&import_bytes(&dtd, &ImportOptions::default())),
        Code::XmlError
    );
}

#[test]
fn encrypted_entries_are_refused() {
    let mut ws = part("ws/worksheet.xml", worksheet_xml(""));
    ws.flags = 1;
    assert_eq!(
        fatal_code(&import_bytes(&zip(&[ws]), &ImportOptions::default())),
        Code::UnsupportedContainer
    );
}

#[test]
fn an_entry_that_lies_about_its_size_cannot_inflate_past_it() {
    // Declares 10 bytes, inflates to far more.
    let mut ws = part("ws/worksheet.xml", vec![b'a'; 100_000]);
    ws.declared = Some(10);
    assert_eq!(
        fatal_code(&import_bytes(&zip(&[ws]), &ImportOptions::default())),
        Code::UnsupportedContainer
    );
}

#[test]
fn size_limits_apply_to_the_package_and_its_parts() {
    let pkg = Pkg::new(&mregion(
        0,
        0.0,
        r#"<ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define>"#,
    ));
    let bytes = pkg.bytes();
    // Whole input over the cap.
    let tiny = ImportOptions {
        limits: Limits {
            max_bytes: 100,
            ..Limits::default()
        },
        ..ImportOptions::default()
    };
    assert_eq!(fatal_code(&import_bytes(&bytes, &tiny)), Code::InputTooLarge);
    // One part over its cap.
    let big = zip(&[part("ws/worksheet.xml", worksheet_xml(&"<!-- pad -->".repeat(5_000)))]);
    let opts = ImportOptions {
        limits: Limits {
            max_bytes: 50_000,
            ..Limits::default()
        },
        ..ImportOptions::default()
    };
    assert!(big.len() < 50_000);
    assert_eq!(fatal_code(&import_bytes(&big, &opts)), Code::InputTooLarge);
    // Total decoded bytes over the budget.
    let opts = ImportOptions {
        limits: Limits {
            max_total_item_bytes: 200,
            ..Limits::default()
        },
        ..ImportOptions::default()
    };
    assert_eq!(fatal_code(&import_bytes(&bytes, &opts)), Code::InputTooLarge);
}

#[test]
fn relationship_targets_cannot_climb_out_of_the_package() {
    // A target with `..` segments resolves inside the archive and simply is not found.
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", ""))
        .rel("Rt1", "flowDocument", "../../../../etc/passwd")
        .import();
    assert!(res.report.gaps().any(|i| i.code == Code::TextUnreadable));
}

#[test]
fn hostile_text_in_names_and_prose_only_reaches_the_output_escaped() {
    let nested = r#"<region region-id="9" actualWidth="10" actualHeight="10"><math resultRef="9"><ml:define><ml:id>evil'&lt;script&gt;</ml:id><ml:real>1</ml:real></ml:define></math></region>"#;
    let xaml = xaml_package(r#"<Paragraph><Run>&lt;script&gt;alert(1)&lt;/script&gt;</Run></Paragraph>"#);
    let res = Pkg::new(&text_region(1, 0.0, "Rt1", nested))
        .rel("Rt1", "flowDocument", "/ws/xaml/FlowDocument1.XamlPackage")
        .part(part("ws/xaml/FlowDocument1.XamlPackage", xaml))
        .import();
    assert!(!res.calcpad.contains("<script"), "{}", res.calcpad);
    // The name is spelled for the engine; the markup in it never reaches the page.
    assert!(!res.calcpad.contains("evil'<"), "{}", res.calcpad);
}

#[test]
fn converted_package_runs_in_the_engine() {
    let res = mcdx_of(&[
        r#"<ml:define><ml:id>L</ml:id><ml:apply><ml:scale/><ml:real>4</ml:real><ml:id labels="UNIT">m</ml:id></ml:apply></ml:define>"#,
        r#"<ml:define><ml:id>F</ml:id><ml:apply><ml:scale/><ml:real>10</ml:real><ml:id labels="UNIT">kN</ml:id></ml:apply></ml:define>"#,
        r#"<ml:define><ml:id>M</ml:id><ml:apply><ml:mult/><ml:id>F</ml:id><ml:id>L</ml:id></ml:apply></ml:define>"#,
        r#"<ml:define><ml:function><ml:id>f</ml:id><ml:boundVars><ml:id>a</ml:id></ml:boundVars></ml:function><ml:apply><ml:pow/><ml:id>a</ml:id><ml:real>2</ml:real></ml:apply></ml:define>"#,
        r#"<ml:eval><ml:apply><ml:id>f</ml:id><ml:real>3</ml:real></ml:apply></ml:eval>"#,
        r#"<ml:define><ml:id>A</ml:id><ml:matrix rows="2" cols="2"><ml:real>1</ml:real><ml:real>2</ml:real><ml:real>3</ml:real><ml:real>4</ml:real></ml:matrix></ml:define>"#,
    ]);
    assert_eq!(res.report.regions_needing_attention(), 0);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(&res.calcpad), true);
    assert!(
        !parser.html_result.contains("class=\"err\""),
        "engine error in:\n{}\n---\n{}",
        res.calcpad,
        parser.html_result
    );
}

// ---- values frozen from input components ---------------------------------------------------------------------------

fn input_component() -> String {
    r#"<region region-id="1" top="0" left="0" width="300" height="100"><excelComponent resultRef="1"><regions><region region-id="2" top="10" left="0"><math resultRef="0"><ml:define><ml:id>Inp</ml:id><ml:apply><ml:indexer/><ml:id>excel</ml:id><ml:str>Sheet1!A1:B2</ml:str></ml:apply></ml:define></math></region></regions></excelComponent></region>"#.to_string()
}

#[test]
fn a_definition_read_from_an_input_component_takes_the_saved_value() {
    let regions = [
        input_component(),
        // n_n := rows(Inp), saved as 2
        mregion(
            3,
            200.0,
            r#"<ml:define><ml:id>n_n</ml:id><ml:apply><ml:id>rows</ml:id><ml:id>Inp</ml:id></ml:apply></ml:define>"#,
        ),
        // N := Inp * cm, saved as a 2x2 matrix in metres
        mregion(
            4,
            220.0,
            r#"<ml:define><ml:id>N</ml:id><ml:apply><ml:mult/><ml:id>Inp</ml:id><ml:id>cm</ml:id></ml:apply></ml:define>"#,
        ),
        // Downstream regions calculate.
        mregion(
            5,
            240.0,
            r#"<ml:eval><ml:apply><ml:mult/><ml:id>n_n</ml:id><ml:real>3</ml:real></ml:apply></ml:eval>"#,
        ),
        mregion(
            6,
            260.0,
            r#"<ml:eval><ml:apply><ml:indexer/><ml:id>N</ml:id><ml:sequence><ml:real>2</ml:real><ml:real>1</ml:real></ml:sequence></ml:apply></ml:eval>"#,
        ),
    ]
    .concat();
    let mut p = Pkg::new(&regions);
    p.results = [
        r#"<resultData result-id="3"><ml:result><ml:real>2</ml:real></ml:result></resultData>"#,
        r#"<resultData result-id="4"><ml:result><ml:unitedValue><ml:matrix rows="2" cols="2"><ml:real>0.01</ml:real><ml:real>0.02</ml:real><ml:real>0.03</ml:real><ml:real>0.04</ml:real></ml:matrix><u:unitMonomial><u:unitReference unit="meter" power-numerator="1" power-denominator="1"/></u:unitMonomial></ml:unitedValue></ml:result></resultData>"#,
    ]
    .concat();
    let res = p.import();
    assert_eq!(res.report.frozen_values, 2, "{:?}", res.report.issues);
    assert!(res.report.notes().any(|i| i.code == Code::FrozenValue));
    assert!(
        res.calcpad
            .contains("input component or a data file (frozen, not live)"),
        "{}",
        res.calcpad
    );
    assert!(res.calcpad.contains("n_n = 2"), "{}", res.calcpad);
    assert!(res.calcpad.contains("N = [0.01; 0.03|0.02; 0.04]*m"), "{}", res.calcpad);
    // Only the component itself needs attention; everything that read the input converted.
    assert_eq!(res.report.regions_needing_attention(), 1, "{:?}", res.report.issues);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(&res.calcpad), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "{html}");
    let text: String = {
        let mut o = String::new();
        let mut tag = false;
        for c in html.chars() {
            match c {
                '<' => tag = true,
                '>' => tag = false,
                c if !tag => o.push(c),
                _ => {}
            }
        }
        o
    };
    // n_n * 3 = 6 and N[2,1] = 0.02 m (0.02 metres).
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("nn · 3 = 2 · 3 = 6"), "{flat}");
    assert!(flat.contains("= 0.02 m"), "{flat}");
}

#[test]
fn a_saved_value_is_not_used_when_something_else_blocks_the_definition() {
    let regions = [
        input_component(),
        // Also reads an add-in function: not only the input.
        mregion(
            3,
            200.0,
            r#"<ml:define><ml:id>q</ml:id><ml:apply><ml:id>vendor_fn</ml:id><ml:id>Inp</ml:id></ml:apply></ml:define>"#,
        ),
        // No saved value at all.
        mregion(
            4,
            220.0,
            r#"<ml:define><ml:id>r2</ml:id><ml:apply><ml:id>rows</ml:id><ml:id>Inp</ml:id></ml:apply></ml:define>"#,
        ),
    ]
    .concat();
    let mut p = Pkg::new(&regions);
    p.results = r#"<resultData result-id="3"><ml:result><ml:real>7</ml:real></ml:result></resultData>"#.into();
    let res = p.import();
    assert_eq!(res.report.frozen_values, 0, "{:?}", res.report.issues);
    assert!(res.report.gaps().any(|i| i.code == Code::AddinFunction));
    assert!(res.report.gaps().any(|i| i.code == Code::UndefinedVariable));
}

#[test]
fn a_variable_named_like_a_unit_is_renamed_so_the_unit_keeps_its_name() {
    // N is a vector of coordinates and also the newton: F := 3 N.
    let regions = [
        mregion(
            1,
            0.0,
            r#"<ml:define><ml:id labels="VARIABLE">N</ml:id><ml:matrix rows="2" cols="1"><ml:real>1</ml:real><ml:real>2</ml:real></ml:matrix></ml:define>"#,
        ),
        mregion(
            2,
            20.0,
            r#"<ml:define><ml:id labels="VARIABLE">F</ml:id><ml:apply><ml:scale/><ml:real>3</ml:real><ml:id labels="UNIT">N</ml:id></ml:apply></ml:define>"#,
        ),
        mregion(
            3,
            40.0,
            r#"<ml:eval><ml:apply><ml:indexer/><ml:id labels="VARIABLE">N</ml:id><ml:real>2</ml:real></ml:apply></ml:eval>"#,
        ),
        mregion(
            4,
            60.0,
            r#"<ml:eval><ml:id labels="VARIABLE">F</ml:id><ml:unitOverride><ml:id labels="UNIT">N</ml:id></ml:unitOverride></ml:eval>"#,
        ),
    ]
    .concat();
    let res = Pkg::new(&regions).import();
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("N_v = [1; 2]"), "{}", res.calcpad);
    assert!(res.calcpad.contains("F = 3*N"), "{}", res.calcpad);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(&res.calcpad), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "{html}");
    let text: String = {
        let mut o = String::new();
        let mut tag = false;
        for c in html.chars() {
            match c {
                '<' => tag = true,
                '>' => tag = false,
                c if !tag => o.push(c),
                _ => {}
            }
        }
        o
    };
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains("= 2"), "{flat}");
    assert!(flat.contains("F|N") || flat.contains("= 3 N"), "{flat}");
}

// ---- numbers read from the workbook an input component saved ------------------------------------------------------

fn rec(id: u16, data: &[u8]) -> Vec<u8> {
    let mut v = id.to_le_bytes().to_vec();
    v.extend_from_slice(&(data.len() as u16).to_le_bytes());
    v.extend_from_slice(data);
    v
}
fn number_cell(row: u16, col: u16, x: f64) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&row.to_le_bytes());
    d.extend_from_slice(&col.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&x.to_le_bytes());
    rec(0x0203, &d)
}
fn rk_cell(row: u16, col: u16, int: i32) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&row.to_le_bytes());
    d.extend_from_slice(&col.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&(((int << 2) | 2) as u32).to_le_bytes());
    rec(0x027E, &d)
}
fn text_cell(row: u16, col: u16) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&row.to_le_bytes());
    d.extend_from_slice(&col.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    rec(0x00FD, &d)
}

/// A compound file holding one `Workbook` stream with the sheet "Data" and the cells given.
fn workbook(cells: &[Vec<u8>]) -> Vec<u8> {
    let name = b"Data";
    let mut bound = Vec::new();
    bound.extend_from_slice(&0u32.to_le_bytes()); // patched below
    bound.extend_from_slice(&[0, 0, name.len() as u8, 0]);
    bound.extend_from_slice(name);
    let bof = rec(0x0809, &[0u8; 16]);
    let eof = rec(0x000A, &[]);
    let globals_len = bof.len() + rec(0x0085, &bound).len() + eof.len();
    bound[..4].copy_from_slice(&(globals_len as u32).to_le_bytes());
    let mut wb = Vec::new();
    wb.extend(&bof);
    wb.extend(rec(0x0085, &bound));
    wb.extend(&eof);
    wb.extend(&bof);
    for c in cells {
        wb.extend(c);
    }
    wb.extend(&eof);
    // Pad past the size at which a stream moves out of the mini stream.
    while wb.len() < 4200 {
        wb.extend(rec(0x00FF, &[0u8; 1000]));
    }
    let n = wb.len().div_ceil(512);
    let mut file = vec![0u8; 512 * (3 + n)];
    file[..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    let put32 = |f: &mut Vec<u8>, at: usize, v: u32| f[at..at + 4].copy_from_slice(&v.to_le_bytes());
    file[0x1E..0x20].copy_from_slice(&9u16.to_le_bytes());
    file[0x20..0x22].copy_from_slice(&6u16.to_le_bytes());
    put32(&mut file, 0x2C, 1);
    put32(&mut file, 0x30, 1);
    put32(&mut file, 0x38, 4096);
    put32(&mut file, 0x3C, 0xFFFF_FFFE);
    put32(&mut file, 0x44, 0xFFFF_FFFE);
    for k in 0..109 {
        put32(&mut file, 0x4C + 4 * k, if k == 0 { 0 } else { 0xFFFF_FFFF });
    }
    // Sector 0: the allocation table. 1: the directory. 2..: the stream.
    let fat = 512;
    for k in 0..128 {
        put32(&mut file, fat + 4 * k, 0xFFFF_FFFF);
    }
    put32(&mut file, fat, 0xFFFF_FFFD);
    put32(&mut file, fat + 4, 0xFFFF_FFFE);
    for k in 0..n {
        put32(
            &mut file,
            fat + 4 * (2 + k),
            if k + 1 == n { 0xFFFF_FFFE } else { (3 + k) as u32 },
        );
    }
    let dir = 1024;
    let entry = |f: &mut Vec<u8>, at: usize, name: &str, kind: u8, start: u32, size: u32| {
        let units: Vec<u8> = name.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        f[at..at + units.len()].copy_from_slice(&units);
        f[at + 0x40..at + 0x42].copy_from_slice(&((units.len() + 2) as u16).to_le_bytes());
        f[at + 0x42] = kind;
        f[at + 0x74..at + 0x78].copy_from_slice(&start.to_le_bytes());
        f[at + 0x78..at + 0x7C].copy_from_slice(&size.to_le_bytes());
    };
    entry(&mut file, dir, "Root Entry", 5, 0xFFFF_FFFE, 0);
    entry(&mut file, dir + 128, "Workbook", 2, 2, wb.len() as u32);
    file[512 * 3..512 * 3 + wb.len()].copy_from_slice(&wb);
    file
}

fn component_with_workbook(range: &str) -> (String, Vec<(String, String, String)>) {
    let region = format!(
        r#"<region region-id="1" top="0" left="0" width="300" height="100"><excelComponent resultRef="1"><regions><region region-id="2" top="10" left="0"><excelTable item-idref="Rx"/></region><region region-id="3" top="50" left="0"><math resultRef="0"><ml:define><ml:id>Inp</ml:id><ml:apply><ml:indexer/><ml:id>excel</ml:id><ml:str>{range}</ml:str></ml:apply></ml:define></math></region></regions></excelComponent></region>"#
    );
    (region, vec![("Rx".into(), "excel".into(), "/ws/excel/x.ole".into())])
}

#[test]
fn the_numbers_of_a_components_workbook_define_the_name_and_downstream_regions_calculate() {
    let ole = workbook(&[
        number_cell(0, 0, 1.5),
        rk_cell(1, 0, 2),
        number_cell(0, 1, -3.0),
        number_cell(1, 1, 4.0),
    ]);
    let (comp, rels) = component_with_workbook("Data!A1:B2");
    let downstream = [
        mregion(
            5,
            200.0,
            r#"<ml:eval><ml:apply><ml:indexer/><ml:id>Inp</ml:id><ml:sequence><ml:real>2</ml:real><ml:real>1</ml:real></ml:sequence></ml:apply></ml:eval>"#,
        ),
        mregion(
            6,
            220.0,
            r#"<ml:define><ml:id>s</ml:id><ml:apply><ml:plus/><ml:apply><ml:indexer/><ml:id>Inp</ml:id><ml:sequence><ml:real>1</ml:real><ml:real>2</ml:real></ml:sequence></ml:apply><ml:real>10</ml:real></ml:apply></ml:define>"#,
        ),
        mregion(7, 240.0, r#"<ml:eval><ml:id>s</ml:id></ml:eval>"#),
    ]
    .concat();
    let mut p = Pkg::new(&format!("{comp}{downstream}"));
    for (id, kind, target) in rels {
        p = p.rel(&id, &kind, &target);
    }
    let mut part = part("ws/excel/x.ole", ole);
    part.deflate = true;
    let res = p.part(part).import();
    assert_eq!(res.report.frozen_values, 1, "{:?}", res.report.issues);
    assert!(res.calcpad.contains("Inp = [1.5; -3|2; 4]"), "{}", res.calcpad);
    assert!(res.calcpad.contains("frozen, not live"), "{}", res.calcpad);
    // Only the component needs attention; the regions that read the name converted.
    assert_eq!(res.report.regions_needing_attention(), 1, "{:?}", res.report.issues);
    let mut parser = calcpad_core::parsers::expression_parser::ExpressionParser::new();
    parser.parse(&legacy(&res.calcpad), true);
    let html = parser.html_result.clone();
    assert!(!html.contains("class=\"err\""), "{html}");
    let text: String = {
        let mut o = String::new();
        let mut tag = false;
        for c in html.chars() {
            match c {
                '<' => tag = true,
                '>' => tag = false,
                c if !tag => o.push(c),
                _ => {}
            }
        }
        o
    };
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // Inp[2,1] = 2 (the RK integer); s = Inp[1,2] + 10 = -3 + 10 = 7.
    assert!(flat.contains("Inp2,1 = 2"), "{flat}");
    assert!(flat.contains("s = Inp1,2 + 10 = 7"), "{flat}");
}

#[test]
fn a_range_that_is_not_all_numbers_or_not_in_the_workbook_is_not_read() {
    let ole = workbook(&[
        number_cell(0, 0, 1.0),
        text_cell(0, 1),
        number_cell(1, 0, 3.0),
        number_cell(1, 1, 4.0),
    ]);
    for range in ["Data!A1:B2", "Missing!A1:B2", "Data!D9:E10", "bad range"] {
        let (comp, rels) = component_with_workbook(range);
        let mut p = Pkg::new(&comp);
        for (id, kind, target) in rels {
            p = p.rel(&id, &kind, &target);
        }
        let res = p.part(part("ws/excel/x.ole", ole.clone())).import();
        assert_eq!(res.report.frozen_values, 0, "{range}: {:?}", res.report.issues);
        assert!(!res.calcpad.contains("Inp ="), "{range}: {}", res.calcpad);
    }
    // A cell nothing was written to reads as 0.
    let ole = workbook(&[number_cell(0, 0, 1.0), number_cell(1, 0, 3.0)]);
    let (comp, rels) = component_with_workbook("Data!A1:B2");
    let mut p = Pkg::new(&comp);
    for (id, kind, target) in rels {
        p = p.rel(&id, &kind, &target);
    }
    let res = p.part(part("ws/excel/x.ole", ole)).import();
    assert!(res.calcpad.contains("Inp = [1; 0|3; 0]"), "{}", res.calcpad);
}

#[test]
fn subscript_and_superscript_runs_are_read_as_text() {
    let region = r#"<region region-id="1" top="0" left="0" width="400"><text is-text-block="true"><FlowDocument xmlns="http://example.invalid/presentation"><Paragraph><Run>A</Run><SubscriptRun Typography.Variants="Subscript">cr</SubscriptRun><Run> and B</Run><SuperscriptRun>2</SuperscriptRun></Paragraph></FlowDocument></text></region>"#;
    let res = Pkg::new(region).import();
    let l = res.calcpad.lines().find(|x| x.contains("and B")).expect("line");
    assert!(l.contains("<sub>cr</sub>") && l.contains("<sup>2</sup>"), "{l}");
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    assert!(!res.report.issues.iter().any(|i| i.code == Code::RichTextElement));
}

#[test]
fn a_table_read_from_a_data_file_takes_the_value_saved_where_the_sheet_shows_it() {
    // T := READEXCEL(file; range) has no result of its own; `T =` further down has the table
    // (a heading, numbers and an empty cell stored as NaN).
    let regions = [
        mregion(
            1,
            0.0,
            r#"<ml:define><ml:id>T</ml:id><ml:apply><ml:id>READEXCEL</ml:id><ml:sequence><ml:str>data.xlsx</ml:str><ml:str>A1:B2</ml:str></ml:sequence></ml:apply></ml:define>"#,
        ),
        mregion(2, 20.0, r#"<ml:eval><ml:id>T</ml:id></ml:eval>"#),
        mregion(
            3,
            40.0,
            r#"<ml:define><ml:id>s</ml:id><ml:apply><ml:indexer/><ml:id>T</ml:id><ml:sequence><ml:real>1</ml:real><ml:real>0</ml:real></ml:sequence></ml:apply></ml:define>"#,
        ),
    ]
    .concat();
    let mut p = Pkg::new(&regions);
    p.results = [
        r#"<resultData result-id="1"/>"#,
        r#"<resultData result-id="2"><ml:result><ml:matrix rows="2" cols="2"><ml:str>Case</ml:str><ml:real>7</ml:real><ml:real>NaN</ml:real><ml:real>9</ml:real></ml:matrix></ml:result></resultData>"#,
    ]
    .concat();
    let res = p.import();
    assert_eq!(res.report.frozen_values, 1, "{:?}", res.report.issues);
    assert!(
        res.calcpad
            .contains("data file (frozen, not live; 1 empty cells read as 0)"),
        "{}",
        res.calcpad
    );
    // The heading is a code, the numbers are the file's; the table is stored by columns.
    assert!(res.calcpad.contains("T = [1; 0|7; 9]"), "{}", res.calcpad);
    // The read itself no longer blocks; printing a table that holds a heading does.
    assert!(!res.report.gaps().any(|i| i.code == Code::BuiltinMissing));
    assert_eq!(res.report.regions_needing_attention(), 1, "{:?}", res.report.issues);
}

#[test]
fn regions_without_a_size_are_calculated_in_the_order_the_file_lists_them() {
    // `a` is defined in a collapsed region (no size) whose stored top is far below the regions that
    // use it; the file lists it first, so it is defined when they read it.
    let hidden = r#"<region region-id="1" top="900" left="0"><math resultRef="1"><ml:define><ml:id>a</ml:id><ml:real>4</ml:real></ml:define></math></region>"#;
    let visible = |id: u32, top: f64, inner: &str| {
        format!(
            r#"<region region-id="{id}" top="{top}" left="0" actualWidth="50" actualHeight="20"><math resultRef="{id}">{inner}</math></region>"#
        )
    };
    let regions = [
        hidden.to_string(),
        visible(2, 100.0, r#"<ml:define><ml:id>b</ml:id><ml:apply><ml:mult/><ml:id>a</ml:id><ml:real>2</ml:real></ml:apply></ml:define>"#),
        visible(3, 140.0, r#"<ml:eval><ml:id>b</ml:id></ml:eval>"#),
    ]
    .concat();
    let res = Pkg::new(&regions).import();
    assert_eq!(res.report.regions_needing_attention(), 0, "{:?}", res.report.issues);
    let ia = res.calcpad.find("a = 4").expect("a defined");
    let ib = res.calcpad.find("b = a*2").expect("b defined");
    assert!(ia < ib, "{}", res.calcpad);
}
