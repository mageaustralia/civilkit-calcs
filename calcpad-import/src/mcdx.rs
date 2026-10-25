//! The mcdx front door: a zip package in, a [`Sheet`] out.
//!
//! The package holds one worksheet part (`worksheet.xml`, named by the package relationships) written in the same
//! worksheet and math vocabulary as the older plain-XML file (the namespaces differ only in their
//! version digits), so the math reader of [`crate::xmcd`] is reused as it is. What differs is the
//! container:
//!
//! * a text region's prose is not in the worksheet part but in a XAML flow document stored as its
//!   own small zip (`*.XamlPackage`), reached through the part's relationships;
//! * results live in `result.xml` beside it, keyed by the region's `resultRef`;
//! * the array origin is in `settings/calculation.xml` beside it;
//! * pictures, embedded objects and flow documents are separate parts named by relationship id.
//!
//! The package is untrusted: see [`crate::zip`] for the archive caps, and the XML limits of
//! [`Limits`] apply to every part that is parsed. Nothing that cannot be converted is dropped
//! quietly: embedded objects, components, references and worksheet headers and footers become
//! regions of an unconvertible kind (the report lists them), and so does any XAML element that
//! would hide content (a table, a picture, a floating block).

use std::collections::HashMap;

use roxmltree::{Document, Node, ParsingOptions};

use crate::ir::*;
use crate::picture;
use crate::xmcd::{self, is_ml, is_ws, local, ns_kind, push_text, Limits};
use crate::zip::{Archive, ZipError};

/// Why a package could not be read at all.
#[derive(Debug, Clone, PartialEq)]
pub enum McdxError {
    TooLarge,
    /// A damaged zip, or one using a feature the reader refuses (encryption, zip64).
    Container(String),
    /// A valid zip without a worksheet part.
    NotAWorksheet,
    Xml(String),
}

impl From<ZipError> for McdxError {
    fn from(e: ZipError) -> Self {
        match e {
            ZipError::Bad(m) | ZipError::Unsupported(m) => McdxError::Container(m.to_string()),
            ZipError::TooLarge => McdxError::TooLarge,
        }
    }
}

fn utf8(bytes: &[u8]) -> Result<&str, McdxError> {
    let b = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    std::str::from_utf8(b).map_err(|_| McdxError::Xml("a part is not UTF-8 text".into()))
}

fn parse<'a>(xml: &'a str, lim: &Limits) -> Result<Document<'a>, McdxError> {
    if xmcd::xml_depth_exceeds(xml, lim.max_xml_depth) {
        return Err(McdxError::Xml("elements nested too deeply".into()));
    }
    let opts = ParsingOptions {
        allow_dtd: false,
        nodes_limit: lim.max_nodes,
    };
    Document::parse_with_options(xml, opts).map_err(|e| McdxError::Xml(e.to_string()))
}

/// The directory part of a package path (`dir/worksheet.xml` gives `dir/`).
fn dir_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(i) => &path[..=i],
        None => "",
    }
}

/// Resolves a relationship target against the directory of the part that owns it. Targets that
/// start with `/` are package-absolute; `.` and `..` segments are folded and cannot climb out.
fn resolve(base_dir: &str, target: &str) -> String {
    let joined = match target.strip_prefix('/') {
        Some(t) => t.to_string(),
        None => format!("{base_dir}{target}"),
    };
    let joined = joined.replace('\\', "/");
    let mut out: Vec<&str> = Vec::new();
    for seg in joined.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    out.join("/")
}

/// Relationship id to resolved part name, from the `.rels` part of `part` (empty when absent).
fn relationships(zip: &Archive, part: &str, lim: &Limits) -> HashMap<String, (String, String)> {
    let rels_name = format!("{}_rels/{}.rels", dir_of(part), part.rsplit('/').next().unwrap_or(part));
    let mut map = HashMap::new();
    let Ok(Some(bytes)) = zip.read(&rels_name, lim.max_item_bytes) else {
        return map;
    };
    let Ok(text) = utf8(&bytes) else { return map };
    let Ok(doc) = parse(text, lim) else { return map };
    for r in doc
        .descendants()
        .filter(|n| n.is_element() && local(*n) == "Relationship")
    {
        if let (Some(id), Some(target)) = (r.attribute("Id"), r.attribute("Target")) {
            let kind = r
                .attribute("Type")
                .unwrap_or("")
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_string();
            map.insert(id.to_string(), (resolve(dir_of(part), target), kind));
        }
    }
    map
}

/// Finds the worksheet part: a part the package-level relationships name, else a part called
/// `worksheet.xml`. The first candidate whose root element is a `worksheet` wins.
fn find_worksheet(zip: &Archive, lim: &Limits) -> Result<Option<(String, Vec<u8>)>, McdxError> {
    let mut candidates: Vec<String> = Vec::new();
    if let Ok(Some(bytes)) = zip.read("_rels/.rels", lim.max_item_bytes) {
        if let Ok(text) = utf8(&bytes) {
            if let Ok(doc) = parse(text, lim) {
                for r in doc
                    .descendants()
                    .filter(|n| n.is_element() && local(*n) == "Relationship")
                {
                    if r.attribute("Type").unwrap_or("").ends_with("/officeDocument") {
                        if let Some(t) = r.attribute("Target") {
                            candidates.push(resolve("", t));
                        }
                    }
                }
            }
        }
    }
    // Without usable relationships: any part called `worksheet.xml`.
    candidates.extend(
        zip.names()
            .filter(|n| *n == "worksheet.xml" || n.ends_with("/worksheet.xml"))
            .map(str::to_string),
    );
    candidates.truncate(8);
    for name in candidates {
        let Some(bytes) = zip.read(&name, lim.max_bytes)? else {
            continue;
        };
        let text = utf8(&bytes)?;
        let doc = parse(text, lim)?;
        if doc.root_element().tag_name().name() == "worksheet" {
            return Ok(Some((name, bytes)));
        }
    }
    Ok(None)
}

/// Reads an mcdx package.
pub fn read(bytes: &[u8], limits: Limits) -> Result<Sheet, McdxError> {
    if bytes.len() > limits.max_bytes {
        return Err(McdxError::TooLarge);
    }
    let zip = Archive::open(bytes, limits.max_total_item_bytes)?;
    let Some((part, ws_bytes)) = find_worksheet(&zip, &limits)? else {
        return Err(McdxError::NotAWorksheet);
    };
    let xml = utf8(&ws_bytes)?;
    let doc = parse(xml, &limits)?;
    let root = doc.root_element();
    let base = dir_of(&part).to_string();

    // Settings: the array origin.
    let mut origin = 0i64;
    if let Ok(Some(b)) = zip.read(&format!("{base}settings/calculation.xml"), limits.max_item_bytes) {
        if let Ok(t) = utf8(&b) {
            if let Ok(d) = parse(t, &limits) {
                if let Some(v) = d
                    .descendants()
                    .find(|n| n.is_element() && local(*n) == "builtInVariables")
                    .and_then(|n| n.attribute("array-origin"))
                    .and_then(|v| v.trim().parse::<i64>().ok())
                {
                    origin = v;
                }
            }
        }
    }

    // The number of decimal places results are shown with, from the presentation settings.
    let mut default_precision = None;
    if let Ok(Some(b)) = zip.read(&format!("{base}settings/presentation.xml"), limits.max_item_bytes) {
        if let Ok(t) = utf8(&b) {
            if let Ok(d) = parse(t, &limits) {
                default_precision = d
                    .descendants()
                    .find(|n| n.is_element() && local(*n) == "general")
                    .and_then(|g| g.attribute("precision"))
                    .and_then(|v| v.trim().parse::<i32>().ok())
                    .filter(|v| (0..=15).contains(v));
            }
        }
    }

    // Saved results, keyed by result id.
    let result_bytes = zip
        .read(&format!("{base}result.xml"), limits.max_item_bytes)
        .ok()
        .flatten();
    let result_text = result_bytes.as_deref().and_then(|b| utf8(b).ok());
    let result_doc = result_text.and_then(|t| parse(t, &limits).ok());
    let mut results: HashMap<&str, Node> = HashMap::new();
    if let Some(d) = &result_doc {
        for n in d.descendants().filter(|n| n.is_element() && local(*n) == "resultData") {
            if let Some(id) = n.attribute("result-id") {
                results.insert(id, n);
            }
        }
    }

    let rels = relationships(&zip, &part, &limits);
    let ctx = Ctx {
        zip: &zip,
        rels,
        results,
        lim: &limits,
        component_names: std::cell::RefCell::new(Vec::new()),
    };

    let mut regions = Vec::new();
    if let Some(container) = root.children().find(|c| is_ws(*c, "regions")) {
        for r in container.children().filter(|c| is_ws(*c, "region")) {
            ctx.collect_region(r, 0, None, &mut regions);
        }
    }

    place_hidden_regions(&mut regions);

    // Page headers and footers are not part of the calculation; say so rather than dropping them.
    for (file, tag) in [("header.xml", "header"), ("footer.xml", "footer")] {
        if let Ok(Some(b)) = zip.read(&format!("{base}{file}"), limits.max_item_bytes) {
            if let Ok(t) = utf8(&b) {
                if let Ok(d) = parse(t, &limits) {
                    if d.descendants().any(|n| is_ws(n, "region")) {
                        regions.push(Region {
                            id: tag.to_string(),
                            left: 0.0,
                            top: f64::MAX / 4.0,
                            baseline: None,
                            line: 0,
                            width: 0.0,
                            height: 0.0,
                            rendering: None,
                            kind: RegionKind::Other(tag.to_string()),
                        });
                    }
                }
            }
        }
    }
    let component_names = ctx.component_names.take();
    Ok(Sheet {
        origin,
        regions,
        component_names,
        text_styles: Default::default(),
        default_precision,
    })
}

/// Regions with no size on the page (the ones inside a collapsed area) keep the position of the
/// area's header, which is not where they are calculated: the file lists them in calculation order.
/// Each takes the place of the last region with a size that precedes it in the file, in file order,
/// so a name defined in a hidden region is defined before the regions after it use it.
fn place_hidden_regions(regions: &mut [Region]) {
    // The file lists regions in the order they are calculated. For regions that have a size the page
    // agrees, so their own positions stand, except where one is listed after a region lower on the
    // page: it follows that one. A region with no size takes the largest top seen so far. Before any
    // region with a size, hidden ones go to the top of the page, in file order.
    let mut max: Option<(f64, f64)> = None;
    let mut seq = 0u32;
    for r in regions.iter_mut() {
        let hidden = r.height <= 0.0 && matches!(r.kind, RegionKind::Math(_));
        match max {
            Some((top, left)) if hidden || r.top < top - 4.0 => {
                seq += 1;
                r.top = top;
                r.left = left + 0.001 * f64::from(seq);
                r.baseline = None;
            }
            _ if hidden => {
                seq += 1;
                r.top = -1.0e6;
                r.left = 0.001 * f64::from(seq);
                r.baseline = None;
            }
            Some((top, _)) if r.top <= top => {}
            _ => {
                max = Some((r.top, r.left));
                seq = 0;
            }
        }
    }
}

struct Ctx<'a, 'd> {
    zip: &'a Archive<'a>,
    /// Relationship id to (part name, relationship kind).
    rels: HashMap<String, (String, String)>,
    results: HashMap<&'d str, Node<'d, 'd>>,
    lim: &'a Limits,
    /// Names defined inside components that are skipped.
    component_names: std::cell::RefCell<Vec<Name>>,
}

fn num_attr(n: Node, names: &[&str]) -> Option<f64> {
    names.iter().find_map(|name| {
        n.attribute(*name)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite())
    })
}

impl Ctx<'_, '_> {
    /// The bytes of the part behind relationship `id`, or `None`.
    fn part(&self, id: &str) -> Option<Vec<u8>> {
        let (name, _) = self.rels.get(id)?;
        self.zip.read(name, self.lim.max_item_bytes).ok().flatten()
    }

    fn collect_region(&self, r: Node, depth: usize, parent: Option<(f64, f64)>, out: &mut Vec<Region>) {
        let lim = self.lim;
        let id = r.attribute("region-id").unwrap_or("?").to_string();
        // A nested region is placed relative to its container. One nested in a text region
        // carries no position at all: it sits where the text does and follows it (the reading
        // order is stable for equal positions).
        let (pl, pt) = parent.unwrap_or((0.0, 0.0));
        let left = pl + num_attr(r, &["left"]).unwrap_or(0.0);
        let top = pt + num_attr(r, &["top"]).unwrap_or(0.0);
        let width = num_attr(r, &["width", "actualWidth"]).unwrap_or(0.0).max(0.0);
        let height = num_attr(r, &["height", "actualHeight"]).unwrap_or(0.0).max(0.0);
        let mk = |kind: RegionKind| Region {
            id: id.clone(),
            left,
            top,
            baseline: None,
            line: 0,
            width,
            height,
            rendering: None,
            kind,
        };
        let content = r
            .children()
            .find(|c| c.is_element() && ns_kind(*c) == "worksheet" && local(*c) != "link");
        let Some(c) = content else {
            out.push(mk(RegionKind::Other("empty".into())));
            return;
        };
        let tag = local(c);
        match tag {
            "math" => {
                let mut m = xmcd::read_math(c, lim);
                m.saved_result = c
                    .attribute("resultRef")
                    .and_then(|id| self.results.get(id))
                    .and_then(|rd| rd.children().find(|k| is_ml(*k) && local(*k) == "result"))
                    .and_then(xmcd::read_saved_result);
                m.saved_expr = c
                    .attribute("resultRef")
                    .and_then(|id| self.results.get(id))
                    .and_then(|rd| rd.children().find(|k| is_ml(*k) && local(*k) == "result"))
                    .and_then(xmcd::read_saved_expr);
                out.push(mk(RegionKind::Math(m)));
            }
            "text" => {
                let mut paras = Vec::new();
                let mut skipped = Vec::new();
                let mut readable = false;
                // Older files carry the flow document inline, as a child of the text element.
                let inline_doc = c.children().find(|k| {
                    k.is_element() && k.tag_name().name() == "FlowDocument" && k.children().any(|g| g.is_element())
                });
                if let Some(fd) = inline_doc {
                    let mut st = BlockState::default();
                    blocks(fd, &Run::default(), &mut st, &mut paras, &mut skipped, 0, lim);
                    readable = true;
                } else if let Some(bytes) = c.attribute("item-idref").and_then(|i| self.part(i)) {
                    if let Some(xaml) = self.xaml_text(&bytes) {
                        if let Ok(doc) = parse(&xaml, lim) {
                            let mut st = BlockState::default();
                            blocks(
                                doc.root_element(),
                                &Run::default(),
                                &mut st,
                                &mut paras,
                                &mut skipped,
                                0,
                                lim,
                            );
                            readable = true;
                        }
                    }
                }
                if readable {
                    out.push(mk(RegionKind::Text(paras)));
                } else {
                    // The prose could not be read: say so instead of emitting an empty text region.
                    out.push(mk(RegionKind::Other("textUnreadable".into())));
                }
                for s in skipped {
                    out.push(mk(RegionKind::Other(format!("xaml{s}"))));
                }
                if let Some(nested) = c.children().find(|k| is_ws(*k, "regions")) {
                    for n in nested.children().filter(|k| is_ws(*k, "region")) {
                        if depth < lim.max_depth {
                            self.collect_region(n, depth + 1, Some((left, top)), out);
                        }
                    }
                }
            }
            "picture" => {
                let mut pic = Picture::default();
                for k in c.children().filter(|k| k.is_element()) {
                    let bytes = k.attribute("item-idref").and_then(|i| self.part(i));
                    match local(k) {
                        "png" if pic.png.is_none() => pic.png = bytes.filter(|b| picture::is_png(b)),
                        "metafile" if pic.metafile.is_none() => pic.metafile = bytes,
                        _ => {}
                    }
                }
                out.push(mk(RegionKind::Picture(pic)));
            }
            // A collapsible area only groups regions: they are read as if they stood on the page.
            "Area" => {
                if let Some(nested) = c.children().find(|k| is_ws(*k, "regions")) {
                    for n in nested.children().filter(|k| is_ws(*k, "region")) {
                        if depth < lim.max_depth {
                            self.collect_region(n, depth + 1, Some((left, top)), out);
                        }
                    }
                }
            }
            // A solve block groups its regions (guess values, constraints, the solver) as an area does;
            // each keeps the part it plays.
            "solveblock" => {
                if let Some(nested) = c.children().find(|k| is_ws(*k, "regions")) {
                    for n in nested.children().filter(|k| is_ws(*k, "region")) {
                        if depth < lim.max_depth {
                            let from = out.len();
                            self.collect_region(n, depth + 1, Some((left, top)), out);
                            let role = n.attribute("solve-block-category");
                            for reg in &mut out[from..] {
                                if let RegionKind::Math(m) = &mut reg.kind {
                                    m.solve = role.map(str::to_string);
                                }
                            }
                        }
                    }
                }
            }
            "plot" => out.push(mk(RegionKind::Plot(crate::plotread::read_plot(c, lim)))),
            "oleObject" | "ole" => out.push(mk(RegionKind::Ole)),
            "reference" => out.push(mk(RegionKind::Reference)),
            "pageBreak" => out.push(mk(RegionKind::PageBreak)),
            t if t.ends_with("Component") || t == "component" => {
                // What the component defines is not run here; note the names so that regions
                // that depend on them can be told apart from regions that depend on nothing.
                for d in c.descendants().filter(|d| is_ml(*d) && local(*d) == "define") {
                    if let Some(id) = d.children().find(|k| is_ml(*k)).filter(|k| local(*k) == "id") {
                        self.component_names.borrow_mut().push(xmcd::read_name_pub(id));
                    }
                }
                // A spreadsheet component reads ranges of its embedded workbook into names
                // (`Name := excel["Sheet1!A1:C5"]`). The workbook is saved in the file, so the
                // numbers in a range can be read and the name defined from them.
                let ole = c
                    .descendants()
                    .find(|d| d.is_element() && local(*d) == "excelTable")
                    .and_then(|t| t.attribute("item-idref"))
                    .and_then(|i| self.part(i));
                if let Some(ole) = ole {
                    for reg in c.descendants().filter(|d| is_ws(*d, "region")) {
                        let Some(m) = reg.children().find(|k| is_ws(*k, "math")) else {
                            continue;
                        };
                        if let Some(region) = self.excel_definition(reg, m, &ole, (left, top)) {
                            out.push(region);
                        }
                    }
                }
                out.push(mk(RegionKind::Component))
            }
            other => out.push(mk(RegionKind::Other(other.to_string()))),
        }
    }

    /// `Name := excel["Sheet!A1:B2"]` with the numbers of that range from the workbook, as a
    /// definition of `Name` as a literal matrix (a number for a single cell).
    fn excel_definition(&self, reg: Node, math: Node, ole: &[u8], parent: (f64, f64)) -> Option<Region> {
        let define = math.children().find(|k| is_ml(*k) && local(*k) == "define")?;
        let kids: Vec<Node> = define.children().filter(|k| is_ml(*k)).collect();
        let [target, value] = kids.as_slice() else { return None };
        if local(*target) != "id" || local(*value) != "apply" {
            return None;
        }
        let v: Vec<Node> = value.children().filter(|k| is_ml(*k)).collect();
        let [op, book, range] = v.as_slice() else { return None };
        if local(*op) != "indexer" || local(*book) != "id" || book.text()?.trim() != "excel" || local(*range) != "str" {
            return None;
        }
        let (rows, cols, values) = crate::xls::read_range(ole, range.text()?.trim())?;
        let items: Vec<Expr> = values.iter().map(|x| Expr::Num(format!("{x}"))).collect();
        let expr = if rows == 1 && cols == 1 {
            items.into_iter().next()?
        } else {
            Expr::Matrix { rows, cols, items }
        };
        Some(Region {
            id: reg.attribute("region-id").unwrap_or("?").to_string(),
            left: parent.0 + num_attr(reg, &["left"]).unwrap_or(0.0),
            top: parent.1 + num_attr(reg, &["top"]).unwrap_or(0.0),
            baseline: None,
            line: 0,
            width: 0.0,
            height: 0.0,
            rendering: None,
            kind: RegionKind::Math(MathRegion {
                stmt: Stmt::Define {
                    target: Target::Var(xmcd::read_name_pub(*target)),
                    value: expr,
                    global: false,
                },
                disabled: false,
                saved_result: None,
                saved_expr: None,
                frozen: true,
                precision: None,
                control: None,
                solve: None,
            }),
        })
    }

    /// The XAML text of a flow document part: a nested zip (`Xaml/Document.xaml`) or plain XML.
    fn xaml_text(&self, bytes: &[u8]) -> Option<String> {
        if bytes.starts_with(b"PK") {
            let inner = Archive::open(bytes, self.lim.max_item_bytes).ok()?;
            let name = inner
                .find(|n| n.to_ascii_lowercase().ends_with(".xaml"))
                .map(str::to_string)?;
            let data = inner.read(&name, self.lim.max_item_bytes).ok()??;
            utf8(&data).ok().map(str::to_string)
        } else {
            utf8(bytes).ok().map(str::to_string)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// XAML flow documents

#[derive(Default)]
struct BlockState {
    /// Inside a list: the list's kind and the running number.
    list: Option<ListKind>,
}

#[derive(Clone, Copy)]
enum ListKind {
    Bullet,
    Decimal(u32),
}

fn flag(v: &str, yes: &[&str]) -> bool {
    yes.iter().any(|y| v.eq_ignore_ascii_case(y))
}

/// Applies the formatting attributes of a XAML element to the inherited run format.
fn format_of(n: Node, base: &Run) -> Run {
    let mut f = base.clone();
    if let Some(v) = n.attribute("FontWeight") {
        f.bold = flag(
            v,
            &[
                "Bold",
                "SemiBold",
                "ExtraBold",
                "UltraBold",
                "Black",
                "Heavy",
                "DemiBold",
            ],
        );
    }
    if let Some(v) = n.attribute("FontFamily").map(str::trim).filter(|v| !v.is_empty()) {
        // A font list (`Arial, Helvetica`): the first name.
        f.font = Some(v.split(',').next().unwrap_or(v).trim().to_string());
    }
    // Device-independent pixels, 96 to the inch: 4/3 pixels to the point.
    if let Some(px) = n
        .attribute("FontSize")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| *v > 0.0)
    {
        f.size = Some((px * 0.75 * 100.0).round() / 100.0);
    }
    if let Some(v) = n.attribute("Foreground") {
        // `#AARRGGBB` or `#RRGGBB`; a fully transparent colour is not a colour.
        let h = v.trim().trim_start_matches('#');
        let rgb = match h.len() {
            8 if !h.starts_with("00") => Some(&h[2..]),
            6 => Some(h),
            _ => None,
        };
        if let Some(rgb) = rgb.and_then(|c| crate::xmcd::parse_hex_color(&format!("#{c}"))) {
            f.color = Some(rgb);
        }
    }
    if let Some(v) = n.attribute("FontStyle") {
        f.italic = flag(v, &["Italic", "Oblique"]);
    }
    if let Some(v) = n.attribute("TextDecorations") {
        f.underline = v.to_ascii_lowercase().contains("underline");
    }
    for key in ["Typography.Variants", "BaselineAlignment"] {
        if let Some(v) = n.attribute(key) {
            f.sub = flag(v, &["Subscript"]);
            f.sup = flag(v, &["Superscript"]);
        }
    }
    // The long form of an attribute: `<Run.TextDecorations><TextDecorationCollection>...`.
    for prop in n
        .children()
        .filter(|k| k.is_element() && k.tag_name().name().ends_with(".TextDecorations"))
    {
        f.underline = prop.descendants().any(|d| {
            d.is_element()
                && d.tag_name().name() == "TextDecoration"
                && d.attribute("Location")
                    .is_none_or(|l| l.eq_ignore_ascii_case("Underline"))
        });
    }
    match n.tag_name().name() {
        "Bold" => f.bold = true,
        "Italic" => f.italic = true,
        "Underline" => f.underline = true,
        _ => {}
    }
    f
}

/// Walks block-level XAML, collecting paragraphs and the names of elements it cannot show.
fn blocks(
    n: Node,
    fmt: &Run,
    st: &mut BlockState,
    paras: &mut Vec<Paragraph>,
    skipped: &mut Vec<String>,
    depth: usize,
    lim: &Limits,
) {
    if depth > lim.max_depth {
        return;
    }
    let fmt = format_of(n, fmt);
    for c in n.children().filter(|c| c.is_element()) {
        match c.tag_name().name() {
            "Paragraph" => {
                let mut pieces = Vec::new();
                let cf = format_of(c, &fmt);
                inline(c, &cf, &mut pieces, skipped, depth + 1, lim);
                let mut style = "Normal";
                match &mut st.list {
                    Some(ListKind::Bullet) => style = "List",
                    Some(ListKind::Decimal(k)) => {
                        *k += 1;
                        pieces.insert(
                            0,
                            Piece::Run(Run {
                                text: format!("{}. ", k),
                                ..Run::default()
                            }),
                        );
                    }
                    None => {}
                }
                let margin_left = c
                    .attribute("Margin")
                    .and_then(|m| m.split(',').next().and_then(|v| v.trim().parse::<f64>().ok()))
                    .map_or(0.0, |px| px * 0.75);
                paras.push(Paragraph {
                    style: style.to_string(),
                    pieces,
                    align: c.attribute("TextAlignment").and_then(crate::xmcd::parse_align),
                    margin_left,
                    text_indent: c
                        .attribute("TextIndent")
                        .and_then(|v| v.trim().parse::<f64>().ok())
                        .map_or(0.0, |px| px * 0.75),
                });
            }
            "List" => {
                let kind = match c.attribute("MarkerStyle") {
                    Some(m) if flag(m, &["Decimal", "LowerLatin", "UpperLatin", "LowerRoman", "UpperRoman"]) => {
                        ListKind::Decimal(0)
                    }
                    _ => ListKind::Bullet,
                };
                let mut inner = BlockState { list: Some(kind) };
                blocks(c, &fmt, &mut inner, paras, skipped, depth + 1, lim);
            }
            "Section" | "ListItem" | "FlowDocument" => blocks(c, &fmt, st, paras, skipped, depth + 1, lim),
            // A property element (`Paragraph.Margin` and the like) is formatting, not content.
            other if other.contains('.') => {}
            other => skipped.push(other.chars().filter(|ch| ch.is_ascii_alphanumeric()).take(24).collect()),
        }
    }
}

/// Walks the inline content of a paragraph or span.
fn inline(n: Node, fmt: &Run, pieces: &mut Vec<Piece>, skipped: &mut Vec<String>, depth: usize, lim: &Limits) {
    if depth > lim.max_depth {
        return;
    }
    for c in n.children() {
        if c.is_text() {
            let t = c.text().unwrap_or("");
            if !t.is_empty() {
                if fmt.font.as_deref().is_some_and(|f| f.eq_ignore_ascii_case("symbol")) {
                    // The run is in the symbol font: its letters are Greek ones.
                    let plain = Run {
                        font: None,
                        ..fmt.clone()
                    };
                    push_text(pieces, &symbol_font_to_unicode(t), &plain);
                } else {
                    push_text(pieces, t, fmt);
                }
            }
        } else if c.is_element() {
            match c.tag_name().name() {
                // A subscript or superscript run (the attributes say which, as on a plain run).
                "Run" | "SubscriptRun" | "SuperscriptRun" => {
                    let mut rf = format_of(c, fmt);
                    match c.tag_name().name() {
                        "SubscriptRun" => rf.sub = true,
                        "SuperscriptRun" => rf.sup = true,
                        _ => {}
                    }
                    inline(c, &rf, pieces, skipped, depth + 1, lim);
                }
                "LineBreak" => pieces.push(Piece::Break),
                "Span" | "Bold" | "Italic" | "Underline" | "Hyperlink" => {
                    let sf = format_of(c, fmt);
                    inline(c, &sf, pieces, skipped, depth + 1, lim);
                }
                other if other.contains('.') => {}
                other => skipped.push(other.chars().filter(|ch| ch.is_ascii_alphanumeric()).take(24).collect()),
            }
        }
    }
}

/// Text typed in the symbol font, where the letter `S` is the capital sigma, as Unicode.
fn symbol_font_to_unicode(t: &str) -> String {
    const UPPER: &str = "ΑΒΧΔΕΦΓΗΙϑΚΛΜΝΟΠΘΡΣΤΥςΩΞΨΖ";
    const LOWER: &str = "αβχδεφγηιϕκλμνοπθρστυϖωξψζ";
    t.chars()
        .map(|c| match c {
            'A'..='Z' => UPPER.chars().nth((c as u8 - b'A') as usize).unwrap_or(c),
            'a'..='z' => LOWER.chars().nth((c as u8 - b'a') as usize).unwrap_or(c),
            c => c,
        })
        .collect()
}
