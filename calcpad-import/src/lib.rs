//! Spike: import the simple subset of a foreign XML worksheet format (`.xmcd`) as Calcpad text.
//!
//! One engine: this crate evaluates nothing. It translates a worksheet into Calcpad worksheet
//! text for the existing engine (`calcpad-core`) and reports, per region kind, what converted
//! and, for everything that did not, a stable code, a reason and a region and line reference.
//! A region that cannot be converted is written to the output as a visible placeholder line (it
//! starts with [`GAP_MARKER`]); it is never dropped silently and nothing is invented.
//!
//! ```
//! let xml = r#"<worksheet xmlns="http://example.invalid/worksheet30" xmlns:ml="http://example.invalid/math30">
//!   <regions>
//!     <region region-id="1" left="0" top="0"><math><ml:define><ml:id>x</ml:id><ml:real>2</ml:real></ml:define></math></region>
//!     <region region-id="2" left="0" top="20"><math><ml:eval><ml:apply><ml:mult/><ml:id>x</ml:id><ml:real>3</ml:real></ml:apply></ml:eval></math></region>
//!   </regions>
//! </worksheet>"#;
//! let r = calcpad_import::import_xmcd(xml);
//! assert!(r.calcpad.contains("x = 2"));
//! assert!(r.calcpad.contains("x*3"));
//! assert_eq!(r.report.regions_needing_attention(), 0);
//! ```
//!
//! # Front doors
//!
//! [`import_xmcd`] reads the plain-XML `.xmcd` file. [`import_mcdx`] reads the zipped `.mcdx`
//! package (see [`mcdx`]); [`import_bytes`] looks at the first bytes and picks the right one.
//! The translator works on the format-neutral [`ir::Sheet`], so another container of the same
//! worksheet language only needs a reader that produces a `Sheet` and then calls [`import_sheet`].

#![forbid(unsafe_code)]

pub mod ir;
pub mod mathml;
pub mod mcdx;
pub mod picture;
mod plotread;
pub mod report;
pub mod tables;
pub mod translate;
pub mod xls;
pub mod xmcd;
pub mod zip;

pub use report::{Code, ImportReport, Issue, KindStats, RankedGap, Severity};
pub use translate::GAP_MARKER;
pub use xmcd::Limits;

/// A binary file the Calcpad text refers to: put it in the page's file library at `path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// Library path relative to the library root, with `/` separators, for example
    /// `imported/R1795.svg`. The Calcpad text refers to it by exactly this string in
    /// `<img src="...">`, so the file must be stored under this path.
    pub path: String,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

/// Switches and limits for an import.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Show pictures: picture regions as images, and a static picture of the original (with a
    /// caption) in place of every region that did not convert and has a saved picture. When off,
    /// those regions are placeholder lines only and no assets are produced.
    pub pictures: bool,
    /// For a region that did not convert, has no picture shown, and has a saved scalar result, write
    /// that result as a labelled static line. Off by default; the report counts the regions it
    /// would cover either way (`report.pictures.saved_result_available`).
    pub saved_results: bool,
    /// Directory (library path) the asset paths start with. Letters, digits, `_`, `-`, `.` and `/`
    /// only; anything else is dropped. Give each imported worksheet its own so that region ids do
    /// not collide between worksheets.
    pub asset_prefix: String,
    /// Reader limits (input size, nesting, picture bytes).
    pub limits: Limits,
    /// Convert metafiles to SVG. Off, picture regions use their raster picture only (the browser
    /// build retries with this off if the converter ever traps).
    pub convert_metafiles: bool,
    /// Metafile conversion limits.
    pub svg_limits: picture::SvgLimits,
    /// Most asset bytes produced for one worksheet; past it, pictures fall back to placeholders.
    pub max_total_asset_bytes: usize,
    /// Typeset the author's equations as MathML (stacked fractions, sums with bounds, subscripts,
    /// `:=`), with results written by the engine. Off, equations are shown as typeset Calcpad
    /// (the `#noc` display), which every export handles. The computation is the same either way.
    pub mathml: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            pictures: true,
            saved_results: false,
            asset_prefix: "imported".to_string(),
            limits: Limits::default(),
            convert_metafiles: true,
            svg_limits: picture::SvgLimits::default(),
            max_total_asset_bytes: 128 * 1024 * 1024,
            mathml: true,
        }
    }
}

fn clean_prefix(p: &str) -> String {
    let segs: Vec<String> = p
        .split('/')
        .map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
                .collect::<String>()
        })
        .filter(|s| !s.is_empty() && !s.chars().all(|c| c == '.'))
        .collect();
    if segs.is_empty() {
        "imported".to_string()
    } else {
        segs.join("/")
    }
}

/// The result of an import: the Calcpad text, the report and the picture files.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResult {
    pub calcpad: String,
    pub report: ImportReport,
    /// Files to store in the library, each under its `path`. Empty when `pictures` is off.
    pub assets: Vec<Asset>,
}

/// Imports an `.xmcd` worksheet with the default options.
pub fn import_xmcd(xml: &str) -> ImportResult {
    import_xmcd_opts(xml, &ImportOptions::default())
}

fn fatal(code: Code, detail: Option<String>) -> ImportResult {
    let mut report = ImportReport::default();
    report.issues.push(Issue {
        code,
        severity: code.severity(),
        detail,
        region_id: None,
        line: None,
        reason: code.reason().to_string(),
        root: None,
    });
    ImportResult {
        calcpad: String::new(),
        report,
        assets: Vec::new(),
    }
}

/// Imports an `.mcdx` package (a zip) with the given options.
pub fn import_mcdx(bytes: &[u8], opts: &ImportOptions) -> ImportResult {
    match mcdx::read(bytes, opts.limits) {
        Ok(sheet) => import_sheet_opts(sheet, opts),
        Err(e) => {
            let short = |m: String| Some(m.chars().take(120).collect::<String>());
            match e {
                mcdx::McdxError::TooLarge => fatal(Code::InputTooLarge, None),
                mcdx::McdxError::Container(m) => fatal(Code::UnsupportedContainer, short(m)),
                mcdx::McdxError::NotAWorksheet => fatal(Code::NotAWorksheet, None),
                mcdx::McdxError::Xml(m) => fatal(Code::XmlError, short(m)),
            }
        }
    }
}

/// Imports the raw bytes of a file. A zip signature routes to the `.mcdx` reader ([`import_mcdx`]);
/// anything else is read as the plain-XML `.xmcd` file, and input that is not UTF-8 is an
/// `xml-error`; a UTF-8 byte order mark is skipped.
pub fn import_bytes(bytes: &[u8], opts: &ImportOptions) -> ImportResult {
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return import_mcdx(bytes, opts);
    }
    if bytes.len() > opts.limits.max_bytes {
        return fatal(Code::InputTooLarge, None);
    }
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(s) => import_xmcd_opts(s, opts),
        Err(_) => fatal(Code::XmlError, Some("the input is not UTF-8 text".into())),
    }
}

/// Imports an `.xmcd` worksheet with explicit reader limits and otherwise default options.
pub fn import_xmcd_with(xml: &str, limits: Limits) -> ImportResult {
    import_xmcd_opts(
        xml,
        &ImportOptions {
            limits,
            ..ImportOptions::default()
        },
    )
}

/// Imports an `.xmcd` worksheet.
pub fn import_xmcd_opts(xml: &str, opts: &ImportOptions) -> ImportResult {
    match xmcd::read(xml, opts.limits) {
        Ok(sheet) => import_sheet_opts(sheet, opts),
        Err(e) => {
            let (code, detail) = match e {
                xmcd::ReadError::TooLarge => (Code::InputTooLarge, None),
                xmcd::ReadError::NotAWorksheet => (Code::NotAWorksheet, None),
                // The parser's message can quote positions only, never input text, but keep it short.
                xmcd::ReadError::Xml(m) => (Code::XmlError, Some(m.chars().take(120).collect::<String>())),
            };
            let mut report = ImportReport::default();
            report.issues.push(Issue {
                code,
                severity: code.severity(),
                detail,
                region_id: None,
                line: None,
                reason: code.reason().to_string(),
                root: None,
            });
            ImportResult {
                calcpad: String::new(),
                report,
                assets: Vec::new(),
            }
        }
    }
}

/// Translates a sheet that a reader already produced, with the default options.
pub fn import_sheet(sheet: ir::Sheet) -> ImportResult {
    import_sheet_opts(sheet, &ImportOptions::default())
}

/// Translates a sheet that a reader already produced.
pub fn import_sheet_opts(sheet: ir::Sheet, opts: &ImportOptions) -> ImportResult {
    let mut opts = opts.clone();
    opts.asset_prefix = clean_prefix(&opts.asset_prefix);
    let (calcpad, report, assets) = translate::translate(sheet, &opts);
    ImportResult {
        calcpad,
        report,
        assets,
    }
}
