//! The worksheet importer for the browser, as its own module: it is loaded only when the user
//! imports a worksheet, and the engine module is unchanged.
//!
//! `import_worksheet(bytes, options_json)` returns an [`ImportOut`]: the Calcpad text, the report as JSON
//! and the picture files. Options (all optional): `pictures`, `mathml`, `savedResults`, `assetPrefix`,
//! `convertMetafiles`, `tidy`, `markdown`. There are no threads in the browser and no unwinding (`panic = "abort"`), so
//! a worker that sees this module trap restarts it and retries with `convertMetafiles: false`.
use calcpad_import::{import_bytes, Code, ImportOptions, ImportReport, Severity};
use serde_json::{json, Map, Value};
use wasm_bindgen::prelude::*;

/// What an import produced.
#[wasm_bindgen]
pub struct ImportOut {
    calcpad: String,
    report: String,
    assets: Vec<(String, String, Vec<u8>)>,
}

#[wasm_bindgen]
impl ImportOut {
    /// The Calcpad worksheet text.
    #[wasm_bindgen(getter)]
    pub fn calcpad(&self) -> String {
        self.calcpad.clone()
    }
    /// The report as JSON (see [`report_json`]).
    #[wasm_bindgen(getter)]
    pub fn report(&self) -> String {
        self.report.clone()
    }
    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }
    pub fn asset_path(&self, i: usize) -> String {
        self.assets.get(i).map(|a| a.0.clone()).unwrap_or_default()
    }
    pub fn asset_mime(&self, i: usize) -> String {
        self.assets.get(i).map(|a| a.1.clone()).unwrap_or_default()
    }
    pub fn asset_bytes(&self, i: usize) -> Vec<u8> {
        self.assets.get(i).map(|a| a.2.clone()).unwrap_or_default()
    }
}

/// Tidies Calcpad text (see `calcpad_import::tidy::tidy`); works on any text.
#[wasm_bindgen]
pub fn tidy_text(src: &str) -> String {
    calcpad_import::tidy::tidy(src)
}

/// Tidies Calcpad text and writes simple paragraphs as Markdown.
#[wasm_bindgen]
pub fn markdown_text(src: &str) -> String {
    calcpad_import::tidy::markdown(src)
}

/// Imports the bytes of an `.xmcd` file.
#[wasm_bindgen]
pub fn import_worksheet(bytes: &[u8], options_json: &str) -> ImportOut {
    let opts = parse_options(options_json);
    let r = import_bytes(bytes, &opts);
    ImportOut {
        calcpad: r.calcpad,
        report: report_json(&r.report).to_string(),
        assets: r
            .assets
            .into_iter()
            .map(|a| (a.path, a.mime.to_string(), a.bytes))
            .collect(),
    }
}

/// The options object; unknown keys and wrong types are ignored.
pub fn parse_options(json: &str) -> ImportOptions {
    let mut o = ImportOptions::default();
    let v: Value = serde_json::from_str(json).unwrap_or(Value::Null);
    if let Some(b) = v.get("mathml").and_then(Value::as_bool) {
        o.mathml = b;
    }
    if let Some(b) = v.get("tidy").and_then(Value::as_bool) {
        o.tidy = b;
    }
    if let Some(b) = v.get("markdown").and_then(Value::as_bool) {
        o.markdown = b;
    }
    if let Some(b) = v.get("pictures").and_then(Value::as_bool) {
        o.pictures = b;
    }
    if let Some(b) = v.get("savedResults").and_then(Value::as_bool) {
        o.saved_results = b;
    }
    if let Some(b) = v.get("convertMetafiles").and_then(Value::as_bool) {
        o.convert_metafiles = b;
    }
    if let Some(s) = v.get("assetPrefix").and_then(Value::as_str) {
        o.asset_prefix = s.to_string();
    }
    o
}

fn sev(s: Severity) -> &'static str {
    match s {
        Severity::Gap => "gap",
        Severity::Note => "note",
        Severity::Fatal => "fatal",
    }
}

/// The report as JSON: counts per region kind, picture statistics, the ranked missing features
/// (with their reasons) and the first 500 issues.
pub fn report_json(r: &ImportReport) -> Value {
    let kinds = |m: &std::collections::BTreeMap<String, calcpad_import::KindStats>| {
        let mut o = Map::new();
        for (k, v) in m {
            o.insert(
                k.clone(),
                json!({"total": v.total, "converted": v.converted, "withNotes": v.with_notes, "attention": v.attention, "pictured": v.pictured}),
            );
        }
        Value::Object(o)
    };
    let ranked: Vec<Value> = r
        .ranked_gaps()
        .iter()
        .map(|g| {
            json!({
                "code": g.code.as_str(), "reason": g.code.reason(), "direct": g.direct,
                "downstream": g.downstream, "blocked": g.blocked(),
                "topDetails": g.top_details.iter().map(|(n, c)| json!([n, c])).collect::<Vec<_>>(),
            })
        })
        .collect();
    let issues: Vec<Value> = r
        .issues
        .iter()
        .take(500)
        .map(|i| {
            json!({
                "code": i.code.as_str(), "severity": sev(i.severity), "detail": i.detail,
                "regionId": i.region_id, "line": i.line, "reason": i.reason,
            })
        })
        .collect();
    let fatal: Option<&Code> = r.issues.iter().find(|i| i.severity == Severity::Fatal).map(|i| &i.code);
    let p = &r.pictures;
    json!({
        "origin": r.origin,
        "regionsTotal": r.regions_total(),
        "regionsConverted": r.regions_converted(),
        "regionsShowing": r.regions_showing(),
        "regionsNeedingAttention": r.regions_needing_attention(),
        "kinds": kinds(&r.kinds),
        "mathShapes": kinds(&r.math_shapes),
        "pageBreaks": r.page_breaks,
        "frozenValues": r.frozen_values,
        "pictures": {
            "svgOk": p.svg_ok, "svgFailed": p.svg_failed, "pngUsed": p.png_used,
            "fallbackShown": p.fallback_shown, "svgBytes": p.svg_bytes, "pngBytes": p.png_bytes,
            "savedResultAvailable": p.saved_result_available, "savedResultEmitted": p.saved_result_emitted,
        },
        "ranked": ranked,
        "issues": issues,
        "fatal": fatal.map(|c| c.as_str()),
        "fatalMessage": fatal.map(|c| c.reason()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHEET: &[u8] =
        br#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30"><regions>
<region region-id="1" left="0" top="0"><math><ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define></math></region>
<region region-id="2" left="0" top="20"><ole/></region></regions></worksheet>"#;

    #[test]
    fn import_reports_json_and_text() {
        let out = import_worksheet(SHEET, r#"{"assetPrefix":"my sheet","pictures":false}"#);
        assert!(out.calcpad().contains("x = 2"));
        let v: Value = serde_json::from_str(&out.report()).unwrap();
        assert_eq!(v["regionsTotal"], 2);
        assert_eq!(v["regionsConverted"], 1);
        assert_eq!(v["ranked"][0]["code"], "ole-object");
        assert!(v["fatal"].is_null());
        assert_eq!(out.asset_count(), 0);
    }

    #[test]
    fn fatal_and_zip_are_reported_not_thrown() {
        let v: Value = serde_json::from_str(&import_worksheet(b"PK\x03\x04zzzz", "{}").report()).unwrap();
        assert_eq!(v["fatal"], "unsupported-container");
        let v: Value = serde_json::from_str(&import_worksheet(b"<not-xml", "not json").report()).unwrap();
        assert_eq!(v["fatal"], "xml-error");
    }
}
