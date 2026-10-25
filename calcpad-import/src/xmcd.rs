//! The `.xmcd` front door: plain XML in, a [`Sheet`] out.
//!
//! Namespaces are matched by their last path segment (`worksheet30`, `math30`, `units10`), not by
//! host, so the reader does not depend on who publishes the schema.
//!
//! The XML is untrusted: a DTD (and so every entity definition) is refused, the input size and the
//! node count are capped, and every recursion below has a depth limit.

use std::cell::Cell;
use std::collections::HashMap;

use roxmltree::{Document, Node, ParsingOptions};

use crate::ir::*;
use crate::picture;
use crate::report::Code;

/// Limits applied while reading. The defaults suit worksheets of a few megabytes.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Largest input accepted, in bytes.
    pub max_bytes: usize,
    /// Most XML nodes accepted.
    pub max_nodes: u32,
    /// Deepest expression or region nesting followed; deeper parts become `too-deep` gaps.
    pub max_depth: usize,
    /// Deepest XML element nesting accepted at all (checked before the parser runs).
    pub max_xml_depth: usize,
    /// Largest decoded binary item (a picture), in bytes. Larger items are ignored.
    pub max_item_bytes: usize,
    /// Most decoded bytes over all items of one file; past it, items are ignored.
    pub max_total_item_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_bytes: 64 * 1024 * 1024,
            max_nodes: 4_000_000,
            max_depth: 64,
            max_xml_depth: 256,
            max_item_bytes: 16 * 1024 * 1024,
            max_total_item_bytes: 192 * 1024 * 1024,
        }
    }
}

/// Why a file could not be read at all.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadError {
    TooLarge,
    Xml(String),
    NotAWorksheet,
}

/// A cheap scan for runaway element nesting, done before the real parser sees the input. It
/// understands comments, CDATA, processing instructions and self-closing tags; malformed input is
/// left for the parser to reject.
pub(crate) fn xml_depth_exceeds(xml: &str, max: usize) -> bool {
    let b = xml.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &b[i..];
        if rest.starts_with(b"<!--") {
            i += 4;
            match xml[i..].find("-->") {
                Some(p) => i += p + 3,
                None => return false,
            }
        } else if rest.starts_with(b"<![CDATA[") {
            i += 9;
            match xml[i..].find("]]>") {
                Some(p) => i += p + 3,
                None => return false,
            }
        } else if rest.starts_with(b"<?") || rest.starts_with(b"<!") {
            match xml[i..].find('>') {
                Some(p) => i += p + 1,
                None => return false,
            }
        } else if rest.starts_with(b"</") {
            depth = depth.saturating_sub(1);
            match xml[i..].find('>') {
                Some(p) => i += p + 1,
                None => return false,
            }
        } else {
            // A start tag; find its end, honouring quoted attribute values.
            let mut j = i + 1;
            let mut quote = 0u8;
            while j < b.len() {
                let c = b[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    break;
                }
                j += 1;
            }
            if j >= b.len() {
                return false;
            }
            if b[j - 1] != b'/' {
                depth += 1;
                if depth > max {
                    return true;
                }
            }
            i = j + 1;
        }
    }
    false
}

fn ns_tail<'a>(n: Node<'a, '_>) -> &'a str {
    n.tag_name()
        .namespace()
        .map(|u| u.trim_end_matches('/').rsplit('/').next().unwrap_or(""))
        .unwrap_or("")
}

/// The namespace family of an element: its last path segment without the version digits, so
/// `worksheet30` (the older format) and `worksheet50` (the mcdx format) are both `worksheet`.
pub(crate) fn ns_kind<'a>(n: Node<'a, '_>) -> &'a str {
    ns_tail(n).trim_end_matches(|c: char| c.is_ascii_digit())
}

pub(crate) fn is_ws(n: Node, local: &str) -> bool {
    n.is_element() && n.tag_name().name() == local && ns_kind(n) == "worksheet"
}

pub(crate) fn is_ml(n: Node) -> bool {
    n.is_element() && ns_kind(n) == "math"
}

pub(crate) fn ml_kids<'a, 'i>(n: Node<'a, 'i>) -> Vec<Node<'a, 'i>> {
    n.children().filter(|c| is_ml(*c)).collect()
}

pub(crate) fn local<'a>(n: Node<'a, '_>) -> &'a str {
    n.tag_name().name()
}

pub fn read(xml: &str, limits: Limits) -> Result<Sheet, ReadError> {
    if xml.len() > limits.max_bytes {
        return Err(ReadError::TooLarge);
    }
    if xml_depth_exceeds(xml, limits.max_xml_depth) {
        return Err(ReadError::Xml("elements nested too deeply".into()));
    }
    let opts = ParsingOptions {
        allow_dtd: false,
        nodes_limit: limits.max_nodes,
    };
    let doc = Document::parse_with_options(xml, opts).map_err(|e| ReadError::Xml(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "worksheet" {
        return Err(ReadError::NotAWorksheet);
    }
    let mut origin = 0i64;
    for n in root.descendants() {
        if n.is_element() && n.tag_name().name() == "builtInVariables" {
            if let Some(v) = n.attribute("array-origin").and_then(|v| v.trim().parse::<i64>().ok()) {
                origin = v;
            }
            break;
        }
    }
    let mut items = HashMap::new();
    for n in root.descendants() {
        if is_ws(n, "item") {
            if let Some(id) = n.attribute("item-id") {
                items.insert(id, n);
            }
        }
    }
    let ctx = Ctx {
        doc: &doc,
        items,
        lim: &limits,
        budget: Cell::new(limits.max_total_item_bytes),
    };
    let mut regions = Vec::new();
    if let Some(container) = root.children().find(|c| is_ws(*c, "regions")) {
        for r in container.children().filter(|c| is_ws(*c, "region")) {
            collect_region(&ctx, r, 0, &mut regions);
        }
    }
    let mut text_styles = HashMap::new();
    for n in root
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "textStyle")
    {
        let Some(name) = n.attribute("name") else { continue };
        let mut st = TextStyle::default();
        for k in n.children().filter(|k| k.is_element()) {
            match k.tag_name().name() {
                "blockAttr" => {
                    st.align = k.attribute("text-align").and_then(parse_align);
                }
                "inlineAttr" => {
                    st.font = k
                        .attribute("font-family")
                        .map(|f| f.trim().to_string())
                        .filter(|f| !f.is_empty());
                    st.size = k
                        .attribute("font-size")
                        .and_then(|v| v.trim().parse::<f64>().ok())
                        .filter(|v| *v > 0.0);
                    st.bold = k.attribute("font-weight") == Some("bold");
                    st.italic = k.attribute("font-style") == Some("italic");
                    st.underline = k.attribute("underline") == Some("true");
                    st.color = k.attribute("color").and_then(parse_hex_color);
                }
                _ => {}
            }
        }
        text_styles.insert(name.to_string(), st);
    }
    // The document default sits in the presentation settings: `results` holds a `general` element.
    let default_precision = root
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "results")
        .find_map(general_precision);
    Ok(Sheet {
        origin,
        regions,
        component_names: Vec::new(),
        text_styles,
        default_precision,
    })
}

struct Ctx<'a, 'i> {
    doc: &'a Document<'i>,
    items: HashMap<&'a str, Node<'a, 'i>>,
    lim: &'a Limits,
    /// Decoded bytes still allowed.
    budget: Cell<usize>,
}

impl Ctx<'_, '_> {
    /// The decoded bytes of binary item `id`, or `None` if it is missing, malformed or over a cap.
    fn item(&self, id: &str) -> Option<Vec<u8>> {
        let node = self.items.get(id)?;
        let text = node.text()?;
        let mut bytes = picture::base64_decode(text)?;
        if node.attribute("content-encoding") == Some("gzip") {
            bytes = picture::gunzip(&bytes, self.lim.max_item_bytes)?;
        }
        if bytes.len() > self.lim.max_item_bytes || bytes.len() > self.budget.get() {
            return None;
        }
        self.budget.set(self.budget.get() - bytes.len());
        Some(bytes)
    }

    fn item_of(&self, n: Node) -> Option<Vec<u8>> {
        self.item(n.attribute("item-idref")?)
    }
}

/// `#rrggbb` (any case) as lower-case `#rrggbb`; `inherit` and anything else is `None`.
pub(crate) fn parse_hex_color(v: &str) -> Option<String> {
    let v = v.trim();
    let h = v.strip_prefix('#')?;
    (h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit())).then(|| format!("#{}", h.to_ascii_lowercase()))
}

pub(crate) fn parse_align(v: &str) -> Option<String> {
    let v = v.trim().to_ascii_lowercase();
    matches!(v.as_str(), "left" | "right" | "center" | "justify").then_some(v)
}

fn num_attr(n: Node, name: &str) -> Option<f64> {
    n.attribute(name)
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
}

fn collect_region(ctx: &Ctx, r: Node, depth: usize, out: &mut Vec<Region>) {
    let lim = ctx.lim;
    let id = r.attribute("region-id").unwrap_or("?").to_string();
    let line = ctx.doc.text_pos_at(r.range().start).row;
    let left = num_attr(r, "left").unwrap_or(0.0);
    let top = num_attr(r, "top").unwrap_or(0.0);
    let width = num_attr(r, "width").unwrap_or(0.0).max(0.0);
    let height = num_attr(r, "height").unwrap_or(0.0).max(0.0);
    let baseline = num_attr(r, "align-y");
    // The content element is the first child that is not a link or a cached rendering.
    let content = r
        .children()
        .find(|c| c.is_element() && ns_kind(*c) == "worksheet" && !matches!(local(*c), "link" | "rendering"));
    let tag = content.map(local).unwrap_or("");
    // The saved raster picture of the region, wanted for everything that is not prose or layout.
    let rendering = if matches!(tag, "text" | "pageBreak" | "") {
        None
    } else {
        r.children()
            .find(|c| is_ws(*c, "rendering"))
            .and_then(|c| ctx.item_of(c))
            .filter(|b| picture::is_png(b))
    };
    let mk = |kind: RegionKind, rendering: Option<Vec<u8>>| Region {
        id: id.clone(),
        left,
        top,
        baseline,
        line,
        width,
        height,
        rendering,
        kind,
    };
    let Some(c) = content else {
        out.push(mk(RegionKind::Other("empty".into()), None));
        return;
    };
    match local(c) {
        "math" => out.push(mk(RegionKind::Math(read_math(c, lim)), rendering)),
        "text" => {
            let mut paras = Vec::new();
            let mut nested = Vec::new();
            for p in c.children().filter(|x| is_ws(*x, "p")) {
                let mut pieces = Vec::new();
                walk_inline(p, &Run::default(), &mut pieces, &mut nested, depth, lim);
                let pt = |name: &str| num_attr(p, name).unwrap_or(0.0);
                paras.push(Paragraph {
                    style: p.attribute("style").unwrap_or("Normal").to_string(),
                    pieces,
                    align: p.attribute("text-align").and_then(parse_align),
                    margin_left: pt("margin-left"),
                    text_indent: pt("text-indent"),
                });
            }
            // The text region precedes the regions nested in it.
            out.push(mk(RegionKind::Text(paras), None));
            for n in nested {
                if depth < lim.max_depth {
                    collect_region(ctx, n, depth + 1, out);
                }
            }
        }
        "picture" => {
            let mut pic = Picture::default();
            for k in c.children().filter(|k| k.is_element()) {
                match local(k) {
                    "metafile" if pic.metafile.is_none() => pic.metafile = ctx.item_of(k),
                    "png" if pic.png.is_none() => pic.png = ctx.item_of(k).filter(|b| picture::is_png(b)),
                    _ => {}
                }
            }
            out.push(mk(RegionKind::Picture(pic), rendering));
        }
        // A picture written straight into the region (no `picture` wrapper).
        "png" | "metafile" => {
            let mut pic = Picture::default();
            for k in r.children().filter(|k| k.is_element() && ns_kind(*k) == "worksheet") {
                match local(k) {
                    "metafile" if pic.metafile.is_none() => pic.metafile = ctx.item_of(k),
                    "png" if pic.png.is_none() => pic.png = ctx.item_of(k).filter(|b| picture::is_png(b)),
                    _ => {}
                }
            }
            out.push(mk(RegionKind::Picture(pic), rendering));
        }
        "plot" => out.push(mk(RegionKind::Plot(crate::plotread::read_plot(c, lim)), rendering)),
        "ole" => out.push(mk(RegionKind::Ole, rendering)),
        "component" => out.push(mk(RegionKind::Component, rendering)),
        "reference" => out.push(mk(RegionKind::Reference, rendering)),
        "pageBreak" => out.push(mk(RegionKind::PageBreak, None)),
        other => out.push(mk(RegionKind::Other(other.to_string()), rendering)),
    }
}

/// Walks the children of a paragraph or a formatting element, collecting runs. Regions found
/// inside are pushed to `nested` and left out of the text.
fn walk_inline<'a, 'i>(
    n: Node<'a, 'i>,
    fmt: &Run,
    pieces: &mut Vec<Piece>,
    nested: &mut Vec<Node<'a, 'i>>,
    depth: usize,
    lim: &Limits,
) {
    if depth > lim.max_depth {
        return;
    }
    for c in n.children() {
        if c.is_text() {
            let t = c.text().unwrap_or("");
            // Indentation between formatting elements (a line break and tabs only) is not text:
            // `<b>e</b>` newline `<b>xternal</b>` is one word.
            let indentation = t.contains('\n') && t.trim().is_empty();
            if !t.is_empty() && !indentation {
                push_text(pieces, t, fmt);
            }
        } else if c.is_element() && ns_kind(c) == "worksheet" {
            match local(c) {
                "region" => nested.push(c),
                "sp" | "tab" => push_text(pieces, " ", fmt),
                "br" => pieces.push(Piece::Break),
                "b" => walk_inline(
                    c,
                    &Run {
                        bold: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "i" => walk_inline(
                    c,
                    &Run {
                        italic: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "u" => walk_inline(
                    c,
                    &Run {
                        underline: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "sub" => walk_inline(
                    c,
                    &Run {
                        sub: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "sup" => walk_inline(
                    c,
                    &Run {
                        sup: true,
                        ..fmt.clone()
                    },
                    pieces,
                    nested,
                    depth + 1,
                    lim,
                ),
                "f" => {
                    let mut f = fmt.clone();
                    if let Some(v) = c.attribute("family").map(str::trim).filter(|v| !v.is_empty()) {
                        f.font = Some(v.to_string());
                    }
                    if let Some(v) = c
                        .attribute("size")
                        .and_then(|v| v.trim().parse::<f64>().ok())
                        .filter(|v| *v > 0.0)
                    {
                        f.size = Some(v);
                    }
                    walk_inline(c, &f, pieces, nested, depth + 1, lim)
                }
                "c" => {
                    let mut f = fmt.clone();
                    if let Some(v) = c.attribute("val").and_then(parse_hex_color) {
                        f.color = Some(v);
                    }
                    walk_inline(c, &f, pieces, nested, depth + 1, lim)
                }
                // inlineAttr and unknown wrappers: keep their text.
                _ => walk_inline(c, fmt, pieces, nested, depth + 1, lim),
            }
        }
    }
}

pub(crate) fn push_text(pieces: &mut Vec<Piece>, t: &str, fmt: &Run) {
    // Collapse whitespace runs (pretty-printing, tabs, newlines) into single spaces.
    let mut s = String::with_capacity(t.len());
    let mut prev_space = false;
    for ch in t.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                s.push(' ');
            }
            prev_space = true;
        } else {
            s.push(ch);
            prev_space = false;
        }
    }
    pieces.push(Piece::Run(Run { text: s, ..fmt.clone() }));
}

// ---------------------------------------------------------------------------------------------
// Math

pub(crate) fn read_math(m: Node, lim: &Limits) -> MathRegion {
    let disabled = m.attribute("disable-calc") == Some("true");
    // The statement is the first math-namespace child (provenance wrappers are peeled off).
    let stmt_node = m.children().find(|c| is_ml(*c));
    let stmt = match stmt_node {
        None => Stmt::Unsupported {
            code: Code::BadStructure,
            detail: Some("empty math region".into()),
        },
        Some(n) => read_stmt(unwrap_provenance(n), lim),
    };
    let saved_result = m
        .descendants()
        .find(|n| is_ml(*n) && local(*n) == "result")
        .and_then(read_saved_result);
    let saved_expr = m
        .descendants()
        .find(|n| is_ml(*n) && local(*n) == "result")
        .and_then(read_saved_expr);
    MathRegion {
        stmt,
        disabled,
        saved_result,
        saved_expr,
        frozen: false,
        precision: m
            .children()
            .find(|c| c.is_element() && c.tag_name().name() == "resultFormat")
            .and_then(general_precision),
    }
}

/// The `precision` attribute of the `general` element under `n`.
pub(crate) fn general_precision(n: Node) -> Option<i32> {
    n.children()
        .find(|c| c.is_element() && c.tag_name().name() == "general")
        .and_then(|g| g.attribute("precision"))
        .and_then(|v| v.trim().parse::<i32>().ok())
        .filter(|v| (0..=15).contains(v))
}

/// The stored result as an expression, only when it is complete and exact: a number, or a matrix of
/// numbers (at most 4000), optionally times a unit written as a product of unit names and powers.
pub(crate) fn read_saved_expr(res: Node) -> Option<Expr> {
    fn number(n: Node) -> Option<Expr> {
        if local(n) != "real" {
            return None;
        }
        let t = n.text()?.trim();
        (!t.is_empty() && t.len() <= 64 && t.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false))
            .then(|| Expr::Num(t.to_string()))
    }
    fn plain(n: Node) -> Option<Expr> {
        match local(n) {
            "real" => number(n),
            "matrix" => {
                let rows = n.attribute("rows")?.parse::<usize>().ok()?;
                let cols = n.attribute("cols")?.parse::<usize>().ok()?;
                let items: Vec<Expr> = n
                    .children()
                    .filter(|c| c.is_element())
                    .map(number)
                    .collect::<Option<_>>()?;
                (rows >= 1 && cols >= 1 && rows.checked_mul(cols)? == items.len() && items.len() <= 4000)
                    .then_some(Expr::Matrix { rows, cols, items })
            }
            _ => None,
        }
    }
    let kid = res.children().find(|c| c.is_element())?;
    if local(kid) != "unitedValue" {
        return plain(kid);
    }
    let value = kid.children().filter(|c| c.is_element()).find_map(plain)?;
    let mono = kid.children().find(|c| c.is_element() && local(*c) == "unitMonomial");
    let mut unit: Option<Expr> = None;
    if let Some(m) = mono {
        for u in m.children().filter(|c| c.is_element() && local(*c) == "unitReference") {
            let name = u.attribute("unit")?;
            let pn: i32 = u.attribute("power-numerator").and_then(|v| v.parse().ok()).unwrap_or(1);
            let pd: i32 = u
                .attribute("power-denominator")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            if pd != 1 || pn == 0 || name.len() > 40 {
                return None;
            }
            let base = Expr::Name(Name::new(name));
            let term = if pn.abs() == 1 {
                base
            } else {
                Expr::Bin(BinOp::Pow, Box::new(base), Box::new(Expr::Num(pn.abs().to_string())))
            };
            unit = Some(match (unit, pn < 0) {
                (None, false) => term,
                (None, true) => Expr::Bin(BinOp::Div, Box::new(Expr::Num("1".into())), Box::new(term)),
                (Some(u), false) => Expr::Bin(BinOp::Mul, Box::new(u), Box::new(term)),
                (Some(u), true) => Expr::Bin(BinOp::Div, Box::new(u), Box::new(term)),
            });
        }
    }
    Some(match unit {
        Some(u) => Expr::Bin(BinOp::Mul, Box::new(value), Box::new(u)),
        None => value,
    })
}

/// The stored result of an evaluation when it is a single number or string, with its unit.
pub(crate) fn read_saved_result(res: Node) -> Option<SavedResult> {
    let kid = res.children().find(|c| c.is_element())?;
    let take_value = |n: Node| -> Option<String> {
        let t = n.text()?.trim().to_string();
        if t.is_empty() || t.len() > 64 {
            return None;
        }
        // Numbers only; strings and anything else are not carried.
        (local(n) == "real" && t.parse::<f64>().map(|v| v.is_finite()).unwrap_or(false)).then_some(t)
    };
    match local(kid) {
        "real" => Some(SavedResult {
            value: take_value(kid)?,
            unit: None,
        }),
        "unitedValue" => {
            let value = take_value(kid.children().find(|c| c.is_element() && local(*c) == "real")?)?;
            let mono = kid.children().find(|c| c.is_element() && local(*c) == "unitMonomial");
            let mut num: Vec<String> = Vec::new();
            let mut den: Vec<String> = Vec::new();
            if let Some(m) = mono {
                for u in m.children().filter(|c| c.is_element() && local(*c) == "unitReference") {
                    let name = u.attribute("unit")?;
                    if name.len() > 40 {
                        return None;
                    }
                    let pn: i32 = u.attribute("power-numerator").and_then(|v| v.parse().ok()).unwrap_or(1);
                    let pd: i32 = u
                        .attribute("power-denominator")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1);
                    if pd == 0 {
                        return None;
                    }
                    let (list, p) = if pn < 0 { (&mut den, -pn) } else { (&mut num, pn) };
                    let mut t = name.to_string();
                    if p != 1 || pd != 1 {
                        t = if pd == 1 {
                            format!("{}^{}", t, p)
                        } else {
                            format!("{}^({}/{})", t, p, pd)
                        };
                    }
                    list.push(t);
                }
            }
            let unit = match (num.is_empty(), den.is_empty()) {
                (true, true) => None,
                (false, true) => Some(num.join("*")),
                (true, false) => Some(format!("1/({})", den.join("*"))),
                (false, false) => Some(format!("{}/({})", num.join("*"), den.join("*"))),
            };
            Some(SavedResult { value, unit })
        }
        _ => None,
    }
}

pub(crate) fn unwrap_provenance<'a, 'i>(mut n: Node<'a, 'i>) -> Node<'a, 'i> {
    let mut guard = 0;
    while is_ml(n) && local(n) == "provenance" && guard < 8 {
        match n.children().find(|c| is_ml(*c)) {
            Some(c) => n = c,
            None => break,
        }
        guard += 1;
    }
    n
}

fn unsupported_code(tag: &str) -> Option<Code> {
    Some(match tag {
        "program" | "ifThen" | "otherwise" | "for" | "while" | "break" | "continue" | "return" | "onError"
        | "localDefine" | "if" | "elseif" | "else" | "tryCatch" => Code::ProgramBlock,
        "symEval" | "symbolicEval" | "symbolic" | "keywordEval" => Code::SymbolicEvaluation,
        "derivative" | "integral" | "nthDerivative" | "partialDerivative" => Code::CalculusOperator,
        "sum" | "product" | "limit" | "iteratedSum" | "iteratedProduct" | "summation" => Code::SummationOperator,
        "vectorize" => Code::VectorizeOperator,
        "matcol" | "matrow" | "magnitude" | "dot" | "cross" => Code::MatrixOperator,
        _ => return None,
    })
}

fn read_stmt(n: Node, lim: &Limits) -> Stmt {
    let tag = local(n);
    match tag {
        "define" | "globalDefine" => {
            let kids = ml_kids(n);
            if kids.len() != 2 {
                return bad("definition without a target and a value");
            }
            let target = match read_target(kids[0], lim) {
                Some(t) => t,
                None => return bad("definition target not understood"),
            };
            Stmt::Define {
                target,
                value: read_expr(unwrap_provenance(kids[1]), lim, 0),
                global: tag == "globalDefine",
            }
        }
        "eval" => {
            let kids = ml_kids(n);
            let mut expr = None;
            let mut unit = None;
            for k in kids {
                match local(k) {
                    "unitOverride" => {
                        // An empty slot (the mcdx format writes a placeholder) means no unit.
                        if let Some(u) = ml_kids(k).into_iter().next().filter(|u| local(*u) != "placeholder") {
                            unit = Some(read_expr(unwrap_provenance(u), lim, 0));
                        }
                    }
                    "result" => {}
                    _ => {
                        if expr.is_none() {
                            expr = Some(read_expr(unwrap_provenance(k), lim, 0));
                        }
                    }
                }
            }
            match expr {
                Some(expr) => Stmt::Eval { expr, unit },
                None => bad("evaluation without an expression"),
            }
        }
        _ => {
            if let Some(code) = unsupported_code(tag) {
                return Stmt::Unsupported {
                    code,
                    detail: Some(tag.to_string()),
                };
            }
            Stmt::Bare(read_expr(n, lim, 0))
        }
    }
}

fn bad(why: &str) -> Stmt {
    Stmt::Unsupported {
        code: Code::BadStructure,
        detail: Some(why.to_string()),
    }
}

pub(crate) fn read_name_pub(n: Node) -> Name {
    read_name(n)
}

fn read_name(n: Node) -> Name {
    // The mcdx format writes a subscript as a styled span inside the identifier; the older
    // format as an attribute. Text inside a superscript cannot be a name: it is kept with a
    // caret so that the name fails the identifier check instead of reading as another name.
    let mut base = String::new();
    let mut sub = String::new();
    for t in n.descendants().filter(|d| d.is_text()) {
        let text = t.text().unwrap_or("");
        let mut in_sub = false;
        let mut in_sup = false;
        for a in t.ancestors().take_while(|a| *a != n) {
            if a.is_element() {
                match a.tag_name().name() {
                    "Subscript" => in_sub = true,
                    "Superscript" => in_sup = true,
                    _ => {}
                }
            }
        }
        if in_sub {
            sub.push_str(text);
        } else if in_sup {
            base.push('^');
            base.push_str(text);
        } else {
            base.push_str(text);
        }
    }
    let attr = n
        .attribute("subscript")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let sub = attr.or_else(|| Some(sub.trim().to_string()).filter(|s| !s.is_empty()));
    Name {
        base: base.trim().to_string(),
        sub,
        unit: n.attribute("labels") == Some("UNIT"),
        var: n.attribute("labels") == Some("VARIABLE"),
    }
}

fn read_target(n: Node, lim: &Limits) -> Option<Target> {
    let n = unwrap_provenance(n);
    match local(n) {
        "id" => Some(Target::Var(read_name(n))),
        "function" => {
            let kids = ml_kids(n);
            let name = kids.iter().find(|k| local(**k) == "id")?;
            let params: Vec<Name> = kids
                .iter()
                .find(|k| local(**k) == "boundVars")
                .map(|b| {
                    ml_kids(*b)
                        .into_iter()
                        .filter(|k| local(*k) == "id")
                        .map(read_name)
                        .collect()
                })
                .unwrap_or_default();
            Some(Target::Func(read_name(*name), params))
        }
        "matrix" => {
            let names: Vec<Name> = ml_kids(n)
                .into_iter()
                .map(unwrap_provenance)
                .filter(|k| local(*k) == "id")
                .map(read_name)
                .collect();
            (!names.is_empty() && names.len() == ml_kids(n).len()).then_some(Target::Multi(names))
        }
        "apply" => {
            let kids = ml_kids(n);
            let (base, index) = match kids.as_slice() {
                [op, first, ..]
                    if local(unwrap_provenance(*op)) == "indexer" && local(unwrap_provenance(*first)) == "id" =>
                {
                    let args = operands(&kids);
                    // The indexes are further operands or one `sequence`.
                    let idx_nodes: Vec<Node> = if args.len() == 2 && local(args[1]) == "sequence" {
                        ml_kids(args[1])
                    } else {
                        args.iter().skip(1).copied().collect()
                    };
                    let index = idx_nodes.iter().map(|a| read_expr(*a, lim, 1)).collect();
                    (Some(read_name(unwrap_provenance(*first))), index)
                }
                _ => (None, Vec::new()),
            };
            Some(Target::Element { base, index })
        }
        _ => None,
    }
}

fn unsupported(code: Code, detail: &str) -> Expr {
    Expr::Unsupported {
        code,
        detail: Some(detail.to_string()),
    }
}

pub(crate) fn read_expr(n: Node, lim: &Limits, depth: usize) -> Expr {
    if depth > lim.max_depth {
        return Expr::Unsupported {
            code: Code::TooDeep,
            detail: None,
        };
    }
    let n = unwrap_provenance(n);
    let tag = local(n);
    match tag {
        "id" => Expr::Name(read_name(n)),
        "real" => Expr::Num(n.text().unwrap_or("").trim().to_string()),
        "str" => Expr::Str(n.text().unwrap_or("").to_string()),
        "parens" => {
            let kids = ml_kids(n);
            if kids.len() == 1 {
                read_expr(kids[0], lim, depth + 1)
            } else {
                unsupported(Code::BadStructure, "parentheses with several children")
            }
        }
        "apply" => read_apply(n, lim, depth),
        // `x := expr =`: the definition also shows its value. Only the expression matters; a
        // display unit on the result is dropped.
        "eval" => {
            let inner = ml_kids(n)
                .into_iter()
                .find(|k| !matches!(local(*k), "unitOverride" | "result"));
            match inner {
                Some(k) => read_expr(k, lim, depth + 1),
                None => unsupported(Code::BadStructure, "empty evaluation"),
            }
        }
        "matrix" => {
            let rows = n.attribute("rows").and_then(|v| v.parse::<usize>().ok());
            let cols = n.attribute("cols").and_then(|v| v.parse::<usize>().ok());
            let items: Vec<Expr> = ml_kids(n).into_iter().map(|k| read_expr(k, lim, depth + 1)).collect();
            match (rows, cols) {
                (Some(r), Some(c)) if r >= 1 && c >= 1 && r.checked_mul(c) == Some(items.len()) => Expr::Matrix {
                    rows: r,
                    cols: c,
                    items,
                },
                _ => unsupported(Code::BadStructure, "matrix size does not match its items"),
            }
        }
        "range" => {
            let kids = ml_kids(n);
            if kids.len() != 2 {
                return unsupported(Code::BadStructure, "range");
            }
            let (first, second) = if local(kids[0]) == "sequence" {
                let s = ml_kids(kids[0]);
                match s.len() {
                    2 => (
                        read_expr(s[0], lim, depth + 1),
                        Some(Box::new(read_expr(s[1], lim, depth + 1))),
                    ),
                    _ => return unsupported(Code::BadStructure, "range"),
                }
            } else {
                (read_expr(kids[0], lim, depth + 1), None)
            };
            Expr::Range {
                first: Box::new(first),
                second,
                last: Box::new(read_expr(kids[1], lim, depth + 1)),
            }
        }
        "program" => Expr::Program(read_program(n, lim, depth + 1)),
        other => match unsupported_code(other) {
            Some(code) => Expr::Unsupported {
                code,
                detail: Some(other.to_string()),
            },
            None => unsupported(Code::UnknownElement, other),
        },
    }
}

fn single<'a, 'i>(n: Node<'a, 'i>) -> Option<Node<'a, 'i>> {
    let k = ml_kids(n);
    (k.len() == 1).then(|| k[0])
}

/// `integral(lambda(boundVars x, body), lowerBound a, upperBound b)` and
/// `derivative(lambda(boundVars x, body), degree 1)`. Any other layout is left unread (`None`).
fn read_calculus(tag: &str, args: &[Node], lim: &Limits, depth: usize) -> Option<Expr> {
    let lambda = args
        .first()
        .copied()
        .map(unwrap_provenance)
        .filter(|l| local(*l) == "lambda")?;
    let parts = ml_kids(lambda);
    let [bound, body] = parts.as_slice() else { return None };
    if local(*bound) != "boundVars" {
        return None;
    }
    let vars = ml_kids(*bound);
    let [v] = vars.as_slice() else { return None };
    let v = unwrap_provenance(*v);
    if local(v) != "id" {
        return None;
    }
    let var = read_name(v);
    let body = Box::new(read_expr(*body, lim, depth + 1));
    match tag {
        "integral" => {
            let [_, lo, hi] = args else { return None };
            if local(*lo) != "lowerBound" || local(*hi) != "upperBound" {
                return None;
            }
            let (lo, hi) = (single(*lo)?, single(*hi)?);
            if matches!(local(lo), "placeholder") || matches!(local(hi), "placeholder") {
                return None;
            }
            Some(Expr::Integral {
                var,
                body,
                lower: Box::new(read_expr(lo, lim, depth + 1)),
                upper: Box::new(read_expr(hi, lim, depth + 1)),
            })
        }
        _ => {
            // The degree is an empty slot (first order) or the literal 1.
            let [_, deg] = args else { return None };
            if local(*deg) != "degree" {
                return None;
            }
            match single(*deg) {
                Some(d) if local(d) == "placeholder" => {}
                Some(d) if local(d) == "real" && d.text().map(str::trim) == Some("1") => {}
                _ => return None,
            }
            Some(Expr::Derivative { var, body })
        }
    }
}

/// A sum or product: a lambda over one bound name and, when the operator has them, its bounds.
/// Without bounds the operator runs over the range variable of that name.
fn read_series(product: bool, args: &[Node], lim: &Limits, depth: usize) -> Option<Expr> {
    let lambda = args
        .first()
        .copied()
        .map(unwrap_provenance)
        .filter(|l| local(*l) == "lambda")?;
    let parts = ml_kids(lambda);
    let [bound, body] = parts.as_slice() else { return None };
    if local(*bound) != "boundVars" {
        return None;
    }
    let vars = ml_kids(*bound);
    let [v] = vars.as_slice() else { return None };
    let v = unwrap_provenance(*v);
    if local(v) != "id" {
        return None;
    }
    let bounds = match args {
        [_] => None,
        [_, b] if local(*b) == "bounds" => {
            let ends = ml_kids(*b);
            let [lo, hi] = ends.as_slice() else { return None };
            if local(*lo) == "placeholder" || local(*hi) == "placeholder" {
                return None;
            }
            Some((
                Box::new(read_expr(*lo, lim, depth + 1)),
                Box::new(read_expr(*hi, lim, depth + 1)),
            ))
        }
        _ => return None,
    };
    Some(Expr::Series {
        product,
        var: read_name(v),
        body: Box::new(read_expr(*body, lim, depth + 1)),
        bounds,
    })
}

/// The statements of a `program` element.
fn read_program(n: Node, lim: &Limits, depth: usize) -> Vec<PStmt> {
    if depth > lim.max_depth {
        return vec![PStmt::Unsupported("nested too deeply".into())];
    }
    ml_kids(n)
        .into_iter()
        .map(|k| read_pstmt(unwrap_provenance(k), lim, depth))
        .collect()
}

/// The statements of the `program` inside a `then`, `else` or loop body element.
fn read_body(n: Node, lim: &Limits, depth: usize) -> Vec<PStmt> {
    match ml_kids(n)
        .into_iter()
        .map(unwrap_provenance)
        .find(|k| local(*k) == "program")
    {
        Some(p) => read_program(p, lim, depth + 1),
        None => vec![PStmt::Unsupported("block without a program".into())],
    }
}

fn read_pstmt(k: Node, lim: &Limits, depth: usize) -> PStmt {
    let kids = ml_kids(k);
    let bad = |what: &str| PStmt::Unsupported(what.to_string());
    match local(k) {
        "localDefine" => {
            let [t, v] = kids.as_slice() else {
                return bad("localDefine");
            };
            let t = unwrap_provenance(*t);
            let value = read_expr(unwrap_provenance(*v), lim, depth + 1);
            match local(t) {
                "id" => PStmt::Local(PTarget::Var(read_name(t)), value),
                "apply" => match read_target(t, lim) {
                    Some(Target::Element {
                        base: Some(base),
                        index,
                    }) => PStmt::Local(PTarget::Element { base, index }, value),
                    _ => bad("local assignment target"),
                },
                _ => bad("local assignment target"),
            }
        }
        "for" => {
            let [v, range, body] = kids.as_slice() else {
                return bad("for");
            };
            let v = unwrap_provenance(*v);
            if local(v) != "id" || local(*body) != "program" {
                return bad("for");
            }
            PStmt::For {
                var: read_name(v),
                range: read_expr(unwrap_provenance(*range), lim, depth + 1),
                body: read_program(*body, lim, depth + 1),
            }
        }
        "while" => {
            let [cond, body] = kids.as_slice() else {
                return bad("while");
            };
            if local(*body) != "program" {
                return bad("while");
            }
            PStmt::While {
                cond: read_expr(unwrap_provenance(*cond), lim, depth + 1),
                body: read_program(*body, lim, depth + 1),
            }
        }
        "if" => {
            let mut branches = Vec::new();
            let mut otherwise = None;
            let mut pending: Option<Expr> = None;
            // `if` holds `test`, `then`, any number of `elseif` (each `test`, `then`) and an `else`.
            let mut items: Vec<Node> = kids;
            let mut i = 0;
            while i < items.len() {
                let it = unwrap_provenance(items[i]);
                match local(it) {
                    "test" => match ml_kids(it).first() {
                        Some(c) => pending = Some(read_expr(unwrap_provenance(*c), lim, depth + 1)),
                        None => return bad("if"),
                    },
                    "then" => match pending.take() {
                        Some(c) => branches.push((c, read_body(it, lim, depth))),
                        None => return bad("if"),
                    },
                    // The package format spells "else if" `alsoif`.
                    "elseif" | "alsoif" => {
                        // Flatten its children into the list being walked.
                        let inner = ml_kids(it);
                        items.splice(i + 1..i + 1, inner);
                    }
                    "else" => otherwise = Some(read_body(it, lim, depth)),
                    _ => return bad("if"),
                }
                i += 1;
            }
            if branches.is_empty() || pending.is_some() {
                return bad("if");
            }
            PStmt::If { branches, otherwise }
        }
        "return" => match kids.as_slice() {
            [v] => PStmt::Return(read_expr(unwrap_provenance(*v), lim, depth + 1)),
            _ => bad("return"),
        },
        "break" => PStmt::Break,
        "continue" => PStmt::Continue,
        "tryCatch" => bad("tryCatch"),
        "program" => bad("nested program"),
        _ => PStmt::Expr(read_expr(k, lim, depth + 1)),
    }
}

/// The operands of an `apply` after the operator; a single `sequence` operand is the argument list.
fn operands<'a, 'i>(kids: &[Node<'a, 'i>]) -> Vec<Node<'a, 'i>> {
    let rest = &kids[1..];
    if rest.len() == 1 && local(rest[0]) == "sequence" {
        ml_kids(rest[0])
    } else {
        rest.to_vec()
    }
}

fn read_apply(n: Node, lim: &Limits, depth: usize) -> Expr {
    let kids = ml_kids(n);
    let Some(op) = kids.first().copied() else {
        return unsupported(Code::BadStructure, "empty apply");
    };
    let op = unwrap_provenance(op);
    let tag = local(op);
    let args = operands(&kids);
    let rd = |x: &Node| read_expr(*x, lim, depth + 1);
    let bin = |o: BinOp, a: &[Node]| -> Expr {
        if a.len() < 2 {
            return unsupported(Code::BadStructure, "binary operator with fewer than two operands");
        }
        // An n-ary operator folds into a left-leaning chain; cap it so the chain stays shallow.
        if a.len() > 100 {
            return Expr::Unsupported {
                code: Code::TooDeep,
                detail: None,
            };
        }
        let mut it = a.iter().map(&rd);
        let mut acc = it.next().unwrap();
        for next in it {
            acc = Expr::Bin(o, Box::new(acc), Box::new(next));
        }
        acc
    };
    let un = |o: UnOp, a: &[Node]| -> Expr {
        if a.len() != 1 {
            return unsupported(Code::BadStructure, "unary operator without exactly one operand");
        }
        Expr::Un(o, Box::new(rd(&a[0])))
    };
    match tag {
        "id" => {
            let name = read_name(op);
            if n.attribute("fixity") == Some("postfix") {
                if args.len() == 1 {
                    return Expr::Postfix(name, Box::new(rd(&args[0])));
                }
                return unsupported(Code::BadStructure, "postfix operator");
            }
            Expr::Call(name, args.iter().map(&rd).collect())
        }
        "plus" => bin(BinOp::Add, &args),
        "minus" => {
            if args.len() == 1 {
                un(UnOp::Neg, &args)
            } else {
                bin(BinOp::Sub, &args)
            }
        }
        "neg" => un(UnOp::Neg, &args),
        // The mcdx format writes a value followed by its unit as a `scale` application.
        "mult" | "scale" => bin(BinOp::Mul, &args),
        "div" => bin(BinOp::Div, &args),
        "pow" => bin(BinOp::Pow, &args),
        "equal" => bin(BinOp::Eq, &args),
        "notEqual" => bin(BinOp::Ne, &args),
        "lessThan" => bin(BinOp::Lt, &args),
        "greaterThan" => bin(BinOp::Gt, &args),
        "lessOrEqual" => bin(BinOp::Le, &args),
        "greaterOrEqual" => bin(BinOp::Ge, &args),
        "and" => bin(BinOp::And, &args),
        "or" => bin(BinOp::Or, &args),
        "xor" => bin(BinOp::Xor, &args),
        "not" => un(UnOp::Not, &args),
        "absval" => un(UnOp::Abs, &args),
        "sqrt" => un(UnOp::Sqrt, &args),
        // `nthRoot` with an empty degree slot is the square root; any other degree is not read
        // (its operand order is unverified).
        "nthRoot" if args.len() == 2 && local(args[0]) == "placeholder" => un(UnOp::Sqrt, &args[1..]),
        "factorial" => un(UnOp::Factorial, &args),
        "transpose" => un(UnOp::Transpose, &args),
        "vectorize" => un(UnOp::Vectorize, &args),
        "vectorSum" => Expr::Call(Name::new("vectorSum"), args.iter().map(&rd).collect()),
        // `matcol(M, j)` and `matrow(M, i)`: the column or row at an index counted from the array origin.
        "matcol" | "matrow" if args.len() == 2 => Expr::Call(Name::new(tag), args.iter().map(&rd).collect()),
        "summation" | "product" => {
            read_series(tag == "product", &args, lim, depth).unwrap_or_else(|| Expr::Unsupported {
                code: Code::SummationOperator,
                detail: Some(tag.to_string()),
            })
        }
        "integral" | "derivative" => read_calculus(tag, &args, lim, depth).unwrap_or_else(|| Expr::Unsupported {
            code: Code::CalculusOperator,
            detail: Some(tag.to_string()),
        }),
        "indexer" => {
            if args.len() < 2 {
                return unsupported(Code::BadStructure, "indexer");
            }
            let base = rd(&args[0]);
            // Indexes are either several operands or one sequence.
            let idx: Vec<Expr> = if args.len() == 2 && local(args[1]) == "sequence" {
                ml_kids(args[1]).iter().map(&rd).collect()
            } else {
                args[1..].iter().map(&rd).collect()
            };
            Expr::Index(Box::new(base), idx)
        }
        other => match unsupported_code(other) {
            Some(code) => Expr::Unsupported {
                code,
                detail: Some(other.to_string()),
            },
            None => unsupported(Code::UnknownElement, other),
        },
    }
}
