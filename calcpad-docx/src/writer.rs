//! Port of OpenXmlWriter.cs and TableBuilder.cs: the HTML of a worksheet as WordprocessingML.
//!
//! The DocumentFormat.OpenXml SDK's element tree is replaced by `N`, which holds only the
//! elements and properties the writer makes, and is written out in schema order by `xml()`.
use std::collections::VecDeque;

use crate::css::{self, ERR_BG, ERR_FG, OK_BG, OK_FG};
use crate::html::{self, Element, Node};
use crate::image::{self, ImageResolver};
use crate::package::Package;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shd {
    pub val: &'static str,
    pub color: Option<String>,
    pub fill: Option<String>,
}

fn make_shading(fill: &str) -> Shd {
    Shd {
        val: "clear",
        color: None,
        fill: Some(fill.to_string()),
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RPr {
    pub fonts: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub caps: bool,
    pub small_caps: bool,
    pub strike: bool,
    pub dstrike: bool,
    pub color: Option<String>,
    pub size: Option<String>,
    pub highlight: Option<String>,
    pub underline: Option<String>,
    pub shading: Option<Shd>,
    pub vert_align: Option<&'static str>,
}

impl RPr {
    /// `CopyRunProperties`: fills each property this run does not have from `src`.
    fn fill_from(&mut self, src: &RPr) {
        if self.fonts.is_none() {
            self.fonts.clone_from(&src.fonts);
        }
        self.bold |= src.bold;
        self.italic |= src.italic;
        self.caps |= src.caps;
        self.small_caps |= src.small_caps;
        self.strike |= src.strike;
        self.dstrike |= src.dstrike;
        if self.color.is_none() {
            self.color.clone_from(&src.color);
        }
        if self.size.is_none() {
            self.size.clone_from(&src.size);
        }
        if self.highlight.is_none() {
            self.highlight.clone_from(&src.highlight);
        }
        if self.underline.is_none() {
            self.underline.clone_from(&src.underline);
        }
        if self.shading.is_none() {
            self.shading.clone_from(&src.shading);
        }
        if self.vert_align.is_none() {
            self.vert_align = src.vert_align;
        }
    }

    fn write(&self, s: &mut String) {
        if *self == RPr::default() {
            return;
        }
        s.push_str("<w:rPr>");
        if let Some(f) = &self.fonts {
            s.push_str(&format!("<w:rFonts w:ascii=\"{0}\" w:hAnsi=\"{0}\"/>", f));
        }
        for (on, tag) in [
            (self.bold, "b"),
            (self.italic, "i"),
            (self.caps, "caps"),
            (self.small_caps, "smallCaps"),
            (self.strike, "strike"),
            (self.dstrike, "dstrike"),
        ] {
            if on {
                s.push_str(&format!("<w:{}/>", tag));
            }
        }
        if let Some(c) = &self.color {
            s.push_str(&format!("<w:color w:val=\"{}\"/>", esc(c)));
        }
        if let Some(z) = &self.size {
            s.push_str(&format!("<w:sz w:val=\"{}\"/>", z));
        }
        if let Some(h) = &self.highlight {
            s.push_str(&format!("<w:highlight w:val=\"{}\"/>", h));
        }
        if let Some(u) = &self.underline {
            s.push_str(&format!("<w:u w:val=\"{}\"/>", u));
        }
        if let Some(shd) = &self.shading {
            write_shd(shd, s);
        }
        if let Some(v) = self.vert_align {
            s.push_str(&format!("<w:vertAlign w:val=\"{}\"/>", v));
        }
        s.push_str("</w:rPr>");
    }
}

fn write_shd(shd: &Shd, s: &mut String) {
    s.push_str(&format!("<w:shd w:val=\"{}\"", shd.val));
    if let Some(c) = &shd.color {
        s.push_str(&format!(" w:color=\"{}\"", esc(c)));
    }
    if let Some(f) = &shd.fill {
        s.push_str(&format!(" w:fill=\"{}\"", esc(f)));
    }
    s.push_str("/>");
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Spacing {
    pub before: Option<&'static str>,
    pub after: Option<&'static str>,
    pub line: Option<&'static str>,
    pub exact: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PPr {
    pub style: Option<String>,
    pub numbering: Option<(i64, i64)>,
    pub bottom_border: Option<&'static str>,
    pub shading: Option<Shd>,
    pub spacing: Option<Spacing>,
    pub indent: Option<(String, String)>,
    pub justify: Option<&'static str>,
}

impl PPr {
    fn write(&self, s: &mut String) {
        s.push_str("<w:pPr>");
        if let Some(st) = &self.style {
            s.push_str(&format!("<w:pStyle w:val=\"{}\"/>", esc(st)));
        }
        if let Some((lvl, id)) = self.numbering {
            s.push_str(&format!(
                "<w:numPr><w:ilvl w:val=\"{}\"/><w:numId w:val=\"{}\"/></w:numPr>",
                lvl, id
            ));
        }
        if let Some(color) = self.bottom_border {
            s.push_str(&format!(
                "<w:pBdr><w:bottom w:val=\"single\" w:sz=\"2\" w:space=\"0\" w:color=\"{}\"/></w:pBdr>",
                color
            ));
        }
        if let Some(shd) = &self.shading {
            write_shd(shd, s);
        }
        if let Some(sp) = &self.spacing {
            s.push_str("<w:spacing");
            if let Some(b) = sp.before {
                s.push_str(&format!(" w:before=\"{}\"", b));
            }
            if let Some(a) = sp.after {
                s.push_str(&format!(" w:after=\"{}\"", a));
            }
            if let Some(l) = sp.line {
                s.push_str(&format!(" w:line=\"{}\"", l));
            }
            if sp.exact {
                s.push_str(" w:lineRule=\"exact\"");
            }
            s.push_str("/>");
        }
        if let Some((left, hanging)) = &self.indent {
            s.push_str(&format!("<w:ind w:left=\"{}\" w:hanging=\"{}\"/>", left, hanging));
        }
        if let Some(j) = self.justify {
            s.push_str(&format!("<w:jc w:val=\"{}\"/>", j));
        }
        s.push_str("</w:pPr>");
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TcPr {
    pub grid_span: Option<i64>,
    pub v_merge: Option<&'static str>,
    pub border_color: Option<&'static str>,
    pub shading: Option<Shd>,
}

/// The document tree: the SDK classes the writer uses.
#[derive(Clone, Debug, PartialEq)]
pub enum N {
    Body(Vec<N>),
    P(Option<PPr>, Vec<N>),
    R(Option<RPr>, Vec<N>),
    T(String),
    Br,
    Tbl {
        bordered: bool,
        children: Vec<N>,
    },
    Tr(Vec<N>),
    /// A cell, with the continued (vertically merged) cells that go in front of it.
    Tc {
        pr: TcPr,
        children: Vec<N>,
        merged_before: Vec<N>,
    },
    Hyperlink {
        id: Option<String>,
        children: Vec<N>,
    },
    /// `CustomXmlBlock`: a div, list or table section; its children are moved into its parent.
    Custom {
        element: String,
        children: Vec<N>,
    },
    /// `M.Paragraph` with an `OfficeMath` holding the equation.
    MathPara(String),
    /// Ready-made markup: an image's drawing.
    Raw(String),
}

impl N {
    fn children_mut(&mut self) -> Option<&mut Vec<N>> {
        match self {
            N::Body(c) | N::P(_, c) | N::R(_, c) | N::Tr(c) => Some(c),
            N::Tbl { children, .. }
            | N::Tc { children, .. }
            | N::Hyperlink { children, .. }
            | N::Custom { children, .. } => Some(children),
            _ => None,
        }
    }

    fn push(&mut self, child: N) {
        if let Some(c) = self.children_mut() {
            c.push(child);
        }
    }

    /// `CloneNode(false)`: the element without its children (run properties are a child).
    fn clone_shallow(&self) -> N {
        match self {
            N::R(_, _) => N::R(None, Vec::new()),
            N::P(_, _) => N::P(None, Vec::new()),
            other => other.clone(),
        }
    }

    fn is_run(&self) -> bool {
        matches!(self, N::R(..))
    }
}

fn esc(s: &str) -> String {
    calcpad_core::openxml::xml::escape(s, true)
}

/// `TableBuilder`: row spans carried down to the next rows.
#[derive(Default)]
struct TableBuilder {
    bordered: bool,
    current: Vec<(i64, bool)>,
    previous: Vec<(i64, bool)>,
}

impl TableBuilder {
    fn add_table(&mut self) -> N {
        self.current.clear();
        self.previous.clear();
        N::Tbl {
            bordered: self.bordered,
            children: Vec::new(),
        }
    }

    fn add_row(&mut self) -> N {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clear();
        N::Tr(Vec::new())
    }

    fn cell(&self, is_head: bool) -> TcPr {
        let mut pr = TcPr::default();
        if !self.bordered {
            return pr;
        }
        pr.border_color = Some(if is_head { "AAAAAA" } else { "CCCCCC" });
        if is_head {
            pr.shading = Some(make_shading("F0F0F0"));
        }
        pr
    }

    fn add_cell(&mut self, is_head: bool, col_span: i64, row_span: i64) -> N {
        // AddMergedCells: the cells a row span above carries into this row, before this one.
        let mut merged_before = Vec::new();
        for i in self.current.len()..self.previous.len() {
            let (span, head) = self.previous[i];
            let span = (span - 1).max(0);
            if span <= 0 {
                break;
            }
            self.current.push((span, head));
            let mut pr = self.cell(head);
            pr.v_merge = Some("continue");
            merged_before.push(N::Tc {
                pr,
                children: Vec::new(),
                merged_before: Vec::new(),
            });
        }
        let mut pr = self.cell(is_head);
        let n = if col_span == 0 { 1 } else { col_span };
        for _ in 0..n {
            self.current.push((row_span, is_head));
        }
        if col_span > 1 {
            pr.grid_span = Some(col_span);
        }
        if row_span > 1 {
            pr.v_merge = Some("restart");
        }
        N::Tc {
            pr,
            children: Vec::new(),
            merged_before,
        }
    }
}

pub struct Writer<'a> {
    expressions: &'a [String],
    buffer: VecDeque<N>,
    table: TableBuilder,
    list_levels: Vec<(i64, bool)>,
    next_num_id: i64,
    pub package: Package,
    resolver: Option<&'a ImageResolver<'a>>,
    root: Option<&'a Element>,
}

/// Elements whose content is never part of the document.
const SKIPPED: [&str; 16] = [
    "head", "title", "meta", "link", "script", "style", "template", "noscript", "iframe", "object", "embed", "canvas",
    "video", "audio", "button", "textarea",
];

/// `IsRunParent`
fn is_run_parent(e: &N) -> bool {
    !matches!(e, N::MathPara(_) | N::T(_) | N::Br | N::Hyperlink { .. } | N::R(..))
}

impl<'a> Writer<'a> {
    pub fn new(expressions: &'a [String], resolver: Option<&'a ImageResolver<'a>>) -> Writer<'a> {
        Writer {
            expressions,
            buffer: VecDeque::new(),
            table: TableBuilder::default(),
            list_levels: Vec::new(),
            next_num_id: 3,
            package: Package::default(),
            resolver,
            root: None,
        }
    }

    /// `WriteDocument`: the body of the document (`<w:body>...</w:body>`).
    pub fn body(&mut self, doc: &'a Element) -> String {
        self.root = Some(doc);
        let main = doc.find("body").unwrap_or(doc);
        let body = self.parse_node(&Node::Element(main.clone()), "");
        let mut body = match body {
            Some(b @ N::Body(_)) => b,
            Some(other) => N::Body(vec![other]),
            None => N::Body(Vec::new()),
        };
        // Anything still buffered belongs to the end of the document.
        while let Some(r) = self.buffer.pop_front() {
            self.append_child(&mut body, r);
        }
        let mut s = String::new();
        write_node(&body, &mut s);
        s
    }

    /// `ParseHtmlNode`
    fn parse_node(&mut self, dom: &Node, parent_name: &str) -> Option<N> {
        let mut parent = self.node_to_element(dom, parent_name)?;
        if matches!(parent, N::MathPara(_)) {
            return Some(parent);
        }
        let Node::Element(el) = dom else { return Some(parent) };
        let is_list = el.name == "ol" || el.name == "ul";
        if is_list {
            // Upstream takes the highest numbering instance + 1 without creating it; a list
            // gets its own instance here, so an ordered list is numbered (D6).
            let id = self.next_num_id;
            self.next_num_id += 1;
            if el.name == "ol" {
                self.package.ordered_lists.push(id);
            }
            self.list_levels.push((id, el.name == "ol"));
        }
        let count = self.buffer.len();
        let n = el.children.len();
        for child in &el.children {
            let Some(mut child_element) = self.parse_node(child, &el.name) else {
                continue;
            };
            if parent.is_run() {
                if matches!(child_element, N::T(_) | N::Br) {
                    let mut e = parent.clone_shallow();
                    e.push(child_element);
                    self.buffer.push_back(e);
                } else {
                    self.buffer.push_back(child_element);
                }
            } else {
                if !self.buffer.is_empty() {
                    if is_run_parent(&child_element) {
                        while let Some(b) = self.buffer.pop_front() {
                            self.append_child(&mut child_element, b);
                        }
                    } else if is_run_parent(&parent) {
                        while let Some(b) = self.buffer.pop_front() {
                            self.append_child(&mut parent, b);
                        }
                    }
                }
                self.append_child(&mut parent, child_element);
            }
        }
        if is_list {
            self.list_levels.pop();
        }
        if let N::R(props, _) = &parent {
            let props = props.clone().unwrap_or_default();
            for (i, e) in self.buffer.iter_mut().enumerate() {
                if i >= count {
                    if let N::R(p, _) = e {
                        p.get_or_insert_with(RPr::default).fill_from(&props);
                    }
                }
            }
            if n > 0 {
                return None;
            }
        }
        Some(parent)
    }

    /// `AppendChild`
    fn append_child(&mut self, parent: &mut N, child: N) {
        if matches!(parent, N::Tbl { .. })
            && !(matches!(child, N::Tr(_))
                || matches!(&child, N::Custom { element, .. } if element == "tbody" || element == "thead"))
            || matches!(parent, N::Tr(_)) && !matches!(child, N::Tc { .. })
        {
            return;
        }
        if let (
            N::Tr(cells),
            N::Tc {
                pr,
                children,
                merged_before,
            },
        ) = (&mut *parent, &child)
        {
            cells.extend(merged_before.iter().cloned());
            cells.push(N::Tc {
                pr: pr.clone(),
                children: children.clone(),
                merged_before: Vec::new(),
            });
            return;
        }
        let in_cell = matches!(parent, N::Tc { .. }) && !matches!(child, N::Tbl { .. });
        if in_cell {
            self.ensure_cell_paragraph(parent);
        }
        let cell_shaded = matches!(parent, N::Tc { pr, .. } if pr.shading.is_some());
        match child {
            N::Br => {
                if let N::R(_, c) = parent {
                    c.push(N::Br);
                } else if let N::P(_, c) = parent {
                    c.push(N::R(None, vec![N::Br]));
                } else if matches!(parent, N::Tc { .. }) {
                    cell_paragraph(parent).push(N::R(None, vec![N::Br]));
                } else {
                    parent.push(N::P(None, vec![N::R(None, vec![N::Br])]));
                }
            }
            N::T(t) => {
                if let N::R(_, c) = parent {
                    c.push(N::T(t));
                } else if matches!(parent, N::Tc { .. }) {
                    let r = N::R(
                        cell_shaded.then(|| RPr {
                            bold: true,
                            ..Default::default()
                        }),
                        vec![N::T(t)],
                    );
                    cell_paragraph(parent).push(r);
                } else if matches!(parent, N::Body(_) | N::Custom { .. }) {
                    parent.push(N::P(None, vec![N::R(None, vec![N::T(t)])]));
                } else if let N::P(Some(ppr), c) = parent {
                    if let Some(shd) = &ppr.shading {
                        let color = shd.color.clone();
                        c.push(N::R(
                            Some(RPr {
                                color,
                                ..Default::default()
                            }),
                            vec![N::T(t)],
                        ));
                    } else {
                        c.push(N::R(None, vec![N::T(t)]));
                    }
                } else {
                    parent.push(N::R(None, vec![N::T(t)]));
                }
            }
            child if matches!(parent, N::Tc { .. }) => {
                if matches!(child, N::P(..) | N::Tbl { .. } | N::Custom { .. }) {
                    parent.push(child);
                } else {
                    cell_paragraph(parent).push(child);
                }
            }
            child @ (N::R(..) | N::MathPara(_)) => match parent {
                N::P(ppr, c) => {
                    let mut child = child;
                    if let (Some(ppr), N::R(rpr, _)) = (ppr.as_ref(), &mut child) {
                        if let Some(shd) = &ppr.shading {
                            if rpr.is_none() {
                                *rpr = Some(RPr {
                                    color: shd.color.clone(),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    c.push(child);
                }
                N::R(..) => self.buffer.push_back(child),
                _ => parent.push(N::P(None, vec![child])),
            },
            child @ N::P(..) => parent.push(child),
            N::Custom { children, .. } => {
                // AppendGrandChildren: the block's children move into its parent.
                for g in children {
                    self.append_child(parent, g);
                }
            }
            child => parent.push(child),
        }
    }

    fn ensure_cell_paragraph(&self, cell: &mut N) {
        let bordered = self.table.bordered;
        if let N::Tc { children, .. } = cell {
            if !children.iter().any(|c| matches!(c, N::P(..))) {
                children.push(if bordered {
                    cell_paragraph_props("center")
                } else {
                    N::P(None, Vec::new())
                });
            }
        }
    }

    /// `Node2Element`
    fn node_to_element(&mut self, dom: &Node, parent_name: &str) -> Option<N> {
        let el = match dom {
            Node::Text(t) => return add_text(t),
            Node::Comment(_) => return None,
            Node::Element(e) => e,
        };
        if is_hidden(el) {
            return None;
        }
        match el.name.as_str() {
            "#document" | "body" => Some(N::Body(Vec::new())),
            "div" => Some(N::Custom {
                element: if el.has_class("fold") {
                    "fold".into()
                } else {
                    el.raw_name.clone()
                },
                children: Vec::new(),
            }),
            "ol" | "ul" | "thead" | "tbody" => Some(N::Custom {
                element: el.name.clone(),
                children: Vec::new(),
            }),
            "small" => Some(N::R(
                Some(RPr {
                    size: Some("22".into()),
                    ..Default::default()
                }),
                Vec::new(),
            )),
            "img" | "svg" => Some(self.add_image(el)),
            "hr" => Some(add_hr(parent_name)),
            "table" => {
                if el.has_class("matrix") {
                    return None;
                }
                // A Markdown pipe table (the engine writes it with a header section) is ruled like a bordered one.
                self.table.bordered = el.has_class("bordered")
                    || el
                        .children
                        .iter()
                        .any(|c| matches!(c, Node::Element(t) if t.name == "thead"));
                Some(self.table.add_table())
            }
            "tr" => Some(self.table.add_row()),
            "th" | "td" => {
                let col_span = el.attr_int("colspan", 1);
                let row_span = el.attr_int("rowspan", 1);
                Some(self.table.add_cell(el.name == "th", col_span, row_span))
            }
            "li" => Some(self.add_list_item()),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some(add_paragraph(el)),
            "br" => Some(N::Br),
            "a" => Some(self.add_hyperlink(el)),
            "span" => Some(self.add_span(el)),
            "input" => {
                let ty = el.attr("type").unwrap_or("text");
                let checked = el.attr("checked").is_some();
                let text = match ty {
                    "radio" => (if checked {
                        "\u{200A}\u{29BF}\u{200A}"
                    } else {
                        "\u{200A}\u{25EF}\u{200A}"
                    })
                    .to_string(),
                    "checkbox" => (if checked {
                        "\u{200A}\u{2611}\u{200A}"
                    } else {
                        "\u{200A}\u{2610}\u{200A}"
                    })
                    .to_string(),
                    _ => el.attr("value").unwrap_or("0").to_string(),
                };
                Some(N::R(None, vec![N::T(text)]))
            }
            "select" | "label" => Some(N::R(None, Vec::new())),
            "option" => el.attr("selected").map(|_| N::R(None, Vec::new())),
            other => add_run(el).or_else(|| {
                // Upstream drops any other element with its content; a container it does not
                // know (blockquote, section, center, abbr, ...) passes its content on here (D6).
                (!SKIPPED.contains(&other)).then(|| N::Custom {
                    element: el.raw_name.clone(),
                    children: Vec::new(),
                })
            }),
        }
    }

    /// `AddSpan`
    fn add_span(&self, el: &Element) -> N {
        if el.has_class("eq") {
            let mut s = el.attr("data-xml").unwrap_or("").to_string();
            if s.is_empty() {
                if let Some(id) = el.attr("id") {
                    if id.len() > 3 {
                        if let Ok(i) = id[3..].parse::<usize>() {
                            if let Some(x) = self.expressions.get(i) {
                                s = x.clone();
                            }
                        }
                    }
                }
            }
            return N::MathPara(s.replace("<<", "&lt;<").replace(">>", ">&gt;").replace("&quot;", "\""));
        }
        if el.has_class("err") {
            return N::R(
                Some(RPr {
                    color: Some(ERR_FG.into()),
                    shading: Some(make_shading(ERR_BG)),
                    ..Default::default()
                }),
                Vec::new(),
            );
        }
        if el.has_class("ok") {
            return N::R(
                Some(RPr {
                    color: Some(OK_FG.into()),
                    shading: Some(make_shading(OK_BG)),
                    ..Default::default()
                }),
                Vec::new(),
            );
        }
        N::R(None, Vec::new())
    }

    /// `AddListItem`
    fn add_list_item(&self) -> N {
        let level = self.list_levels.len() as i64;
        let (id, ordered) = self.list_levels.last().copied().unwrap_or((0, false));
        let left = 240 * (level + 1);
        N::P(
            Some(PPr {
                numbering: Some(((level - 1).clamp(0, 3), if ordered { id } else { 1 })),
                indent: Some((left.to_string(), "240".into())),
                ..Default::default()
            }),
            Vec::new(),
        )
    }

    /// `AddHyperlink`
    fn add_hyperlink(&mut self, el: &Element) -> N {
        let href = el.attr("href").unwrap_or("").trim();
        let mut id = None;
        if !href.is_empty() && is_absolute_uri(href) {
            id = Some(self.package.add_hyperlink(href));
        }
        N::Hyperlink {
            id,
            children: Vec::new(),
        }
    }

    /// `img` and `svg` (`ImageWriter.AddImage`), or the error as text.
    fn add_image(&mut self, el: &Element) -> N {
        let size = |dir: &str| -> f64 {
            let a = el.attr_int(dir, 0);
            if a != 0 {
                return a as f64;
            }
            el.attr("style").map(|s| css::css_size_px(s, dir)).unwrap_or(0.0)
        };
        let w = size("width") as i64;
        let h = size("height") as i64;
        let mut align = el.attr("style").and_then(|s| css::css_parameter(s, "float"));
        if el.has_class("side") {
            align = Some("right".into());
        }
        let alt = el.attr("alt").unwrap_or("").to_string();
        let src = if el.name == "svg" {
            let mut svg = el.clone();
            if let Some(root) = self.root {
                clone_svg_uses(&mut svg, root);
            }
            svg.outer_html().replace("viewbox", "viewBox")
        } else {
            el.attr("src").unwrap_or("").to_string()
        };
        match image::add_image(&mut self.package, &src, &alt, w, h, align.as_deref(), self.resolver) {
            Ok(drawing) => drawing,
            Err(e) => N::P(None, vec![N::R(None, vec![N::T(format!("Error adding image: {}", e))])]),
        }
    }
}

fn is_absolute_uri(s: &str) -> bool {
    match s.find(':') {
        Some(i) if i > 0 => s[..i]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')),
        _ => false,
    }
}

/// The cell's first paragraph (made when there is none).
fn cell_paragraph(cell: &mut N) -> &mut Vec<N> {
    let N::Tc { children, .. } = cell else { unreachable!() };
    let i = children.iter().position(|c| matches!(c, N::P(..))).unwrap_or_else(|| {
        children.push(N::P(None, Vec::new()));
        children.len() - 1
    });
    match &mut children[i] {
        N::P(_, c) => c,
        _ => unreachable!(),
    }
}

/// `AddCellParagraph`
fn cell_paragraph_props(justify: &'static str) -> N {
    N::P(
        Some(PPr {
            spacing: Some(Spacing {
                before: Some("0"),
                after: Some("0"),
                line: None,
                exact: true,
            }),
            justify: Some(justify),
            ..Default::default()
        }),
        Vec::new(),
    )
}

/// `IsHidden`
fn is_hidden(el: &Element) -> bool {
    if el.has_class("lineLink") || el.has_class("errorHeader") {
        return true;
    }
    let Some(style) = el.attr("style") else { return false };
    let compact: String = style.to_ascii_lowercase().split_whitespace().collect();
    compact.contains("display:none") || compact.contains("visibility:hidden")
}

/// `AddText`
fn add_text(raw: &str) -> Option<N> {
    let s = html::decode(raw);
    let has_new_line = s.contains('\n');
    if has_new_line && s.trim().is_empty() {
        return None;
    }
    let s = if has_new_line { s.replace(['\n', '\r'], "") } else { s };
    Some(N::T(s))
}

/// `AddHr`
fn add_hr(parent_name: &str) -> N {
    const HR_COLOR: &str = "888888";
    let in_paragraph = parent_name == "p" || parent_name.len() == 2 && parent_name.starts_with('h');
    if in_paragraph {
        return N::R(
            Some(RPr {
                color: Some(HR_COLOR.into()),
                ..Default::default()
            }),
            vec![N::Br, N::T("\u{2500}".repeat(55)), N::Br],
        );
    }
    N::P(
        Some(PPr {
            bottom_border: Some(HR_COLOR),
            ..Default::default()
        }),
        vec![N::R(
            Some(RPr {
                size: Some("2".into()),
                ..Default::default()
            }),
            vec![N::T(".".into())],
        )],
    )
}

/// `GetColors`: the colour and background of a style attribute.
fn get_colors(el: &Element) -> (Option<String>, Option<String>) {
    let (mut fg, mut bg) = (None, None);
    if let Some(style) = el.attr("style") {
        for prop in style.split(';') {
            let Some(i) = prop.find(':') else { continue };
            let name = prop[..i].trim();
            let value = prop[i + 1..].trim();
            if name.eq_ignore_ascii_case("color") {
                fg = css::css_to_openxml_color(value);
            } else if name.eq_ignore_ascii_case("background-color") {
                bg = css::css_to_openxml_color(value);
            } else if name.eq_ignore_ascii_case("background") {
                let end = if value.to_ascii_lowercase().starts_with("rgb") {
                    value.find(')').map(|e| e + 1)
                } else {
                    value.find(' ')
                };
                let v = match end {
                    Some(e) if e > 0 => &value[..e],
                    _ => value,
                };
                bg = css::css_to_openxml_color(v);
            }
        }
    }
    (fg.filter(|s| !s.is_empty()), bg.filter(|s| !s.is_empty()))
}

/// `AddRun`
fn add_run(el: &Element) -> Option<N> {
    let mut rp = RPr::default();
    match el.name.as_str() {
        "var" | "em" | "i" => rp.italic = true,
        "b" | "strong" => rp.bold = true,
        "u" | "ins" => rp.underline = Some("single".into()),
        "s" | "del" | "strike" => rp.strike = true,
        "sup" => rp.vert_align = Some("superscript"),
        "sub" => rp.vert_align = Some("subscript"),
        "mark" => rp.highlight = Some("yellow".into()),
        "tt" | "kbd" | "code" | "samp" => rp.fonts = Some("Consolas".into()),
        _ => return None,
    }
    if el.has_class("err") {
        rp.color = Some(ERR_FG.into());
        rp.shading = Some(make_shading(ERR_BG));
    } else if el.has_class("ok") {
        rp.color = Some(OK_FG.into());
        rp.shading = Some(make_shading(OK_BG));
    } else {
        let (fg, bg) = get_colors(el);
        if let Some(f) = fg {
            rp.color = Some(f);
        }
        if let Some(b) = bg {
            rp.shading = Some(make_shading(&b));
        }
    }
    Some(N::R(Some(rp), Vec::new()))
}

fn props(p: &mut Option<PPr>) -> &mut PPr {
    p.get_or_insert_with(PPr::default)
}

/// `AddParagraph`
fn add_paragraph(el: &Element) -> N {
    let mut ppr: Option<PPr> = None;
    if el.name != "p" {
        ppr = Some(PPr {
            style: Some(el.name.clone()),
            ..Default::default()
        });
    }
    if el.has_class("ref") {
        let p = props(&mut ppr);
        p.justify = Some("right");
        p.spacing = Some(Spacing {
            before: Some("0"),
            after: Some("0"),
            line: None,
            exact: true,
        });
        let mut shd = make_shading(OK_BG);
        shd.color = Some(OK_FG.into());
        p.shading = Some(shd);
    } else if el.has_class("err") {
        let mut shd = make_shading(ERR_BG);
        shd.color = Some(ERR_FG.into());
        props(&mut ppr).shading = Some(shd);
    } else if el.has_class("ok") {
        let mut shd = make_shading(OK_BG);
        shd.color = Some(OK_FG.into());
        props(&mut ppr).shading = Some(shd);
    } else {
        let (fg, bg) = get_colors(el);
        if fg.is_some() || bg.is_some() {
            props(&mut ppr).shading = Some(Shd {
                val: "clear",
                color: fg,
                fill: bg,
            });
        }
    }
    N::P(ppr, Vec::new())
}

/// `CloneSvgUses`: a `<use>` pointing at an element outside the drawing is replaced by a copy.
fn clone_svg_uses(svg: &mut Element, root: &Element) {
    fn walk(e: &mut Element, root: &Element, inside: &Element) {
        for c in e.children.iter_mut() {
            if let Node::Element(child) = c {
                if child.name == "use" {
                    let href = child
                        .attr("href")
                        .or_else(|| child.attr("xlink:href"))
                        .unwrap_or("")
                        .to_string();
                    if let Some(id) = href.strip_prefix('#') {
                        if inside.by_id(id).is_none() {
                            if let Some(src) = root.by_id(id) {
                                *c = Node::Element(src.clone());
                            }
                        }
                    }
                } else {
                    walk(child, root, inside);
                }
            }
        }
    }
    let inside = svg.clone();
    walk(svg, root, &inside);
}

/// The document tree as WordprocessingML.
pub fn write_node(n: &N, s: &mut String) {
    match n {
        N::Body(c) => {
            s.push_str("<w:body>");
            for x in c {
                write_block(x, s);
            }
            s.push_str(
                "<w:sectPr><w:pgMar w:top=\"850\" w:right=\"850\" w:bottom=\"850\" w:left=\"1134\" w:header=\"850\" w:footer=\"850\" w:gutter=\"0\"/></w:sectPr>",
            );
            s.push_str("</w:body>");
        }
        N::P(ppr, c) => {
            s.push_str("<w:p>");
            if let Some(p) = ppr {
                p.write(s);
            }
            for x in c {
                write_inline(x, s);
            }
            s.push_str("</w:p>");
        }
        N::R(rpr, c) => {
            s.push_str("<w:r>");
            if let Some(r) = rpr {
                r.write(s);
            }
            for x in c {
                match x {
                    N::T(t) => {
                        s.push_str("<w:t xml:space=\"preserve\">");
                        s.push_str(&calcpad_core::openxml::xml::escape(t, false));
                        s.push_str("</w:t>");
                    }
                    N::Br => s.push_str("<w:br/>"),
                    N::Raw(r) => s.push_str(r),
                    _ => {}
                }
            }
            s.push_str("</w:r>");
        }
        N::Tbl { bordered, children } => {
            s.push_str(&format!(
                "<w:tbl><w:tblPr><w:tblStyle w:val=\"{}\"/></w:tblPr><w:tblGrid/>",
                if *bordered { "bcpd" } else { "cpd" }
            ));
            for r in children {
                if let N::Tr(_) = r {
                    write_node(r, s);
                }
            }
            s.push_str("</w:tbl>");
        }
        N::Tr(c) => {
            s.push_str("<w:tr>");
            for x in c {
                write_node(x, s);
            }
            s.push_str("</w:tr>");
        }
        N::Tc { pr, children, .. } => {
            s.push_str("<w:tc><w:tcPr>");
            if let Some(g) = pr.grid_span {
                s.push_str(&format!("<w:gridSpan w:val=\"{}\"/>", g));
            }
            if let Some(v) = pr.v_merge {
                s.push_str(&format!("<w:vMerge w:val=\"{}\"/>", v));
            }
            if let Some(c) = pr.border_color {
                s.push_str("<w:tcBorders>");
                for side in ["top", "left", "bottom", "right"] {
                    s.push_str(&format!(
                        "<w:{} w:val=\"single\" w:sz=\"8\" w:space=\"0\" w:color=\"{}\"/>",
                        side, c
                    ));
                }
                s.push_str("</w:tcBorders>");
            }
            if let Some(shd) = &pr.shading {
                write_shd(shd, s);
            }
            s.push_str("</w:tcPr>");
            // A cell must end with a paragraph.
            for x in children {
                write_block(x, s);
            }
            if !matches!(children.last(), Some(N::P(..))) {
                s.push_str("<w:p/>");
            }
            s.push_str("</w:tc>");
        }
        other => write_block(other, s),
    }
}

/// A block-level child: paragraphs and tables as they are, anything else in a paragraph.
fn write_block(n: &N, s: &mut String) {
    match n {
        N::P(..) | N::Tbl { .. } => write_node(n, s),
        N::Custom { children, .. } => {
            for c in children {
                write_block(c, s);
            }
        }
        N::MathPara(m) => {
            s.push_str("<w:p>");
            write_math(m, s);
            s.push_str("</w:p>");
        }
        N::Body(_) | N::Tr(_) | N::Tc { .. } => {}
        other => {
            s.push_str("<w:p>");
            write_inline(other, s);
            s.push_str("</w:p>");
        }
    }
}

fn write_math(m: &str, s: &mut String) {
    s.push_str("<m:oMathPara><m:oMathParaPr><m:jc m:val=\"left\"/></m:oMathParaPr><m:oMath>");
    s.push_str(m);
    s.push_str("</m:oMath></m:oMathPara>");
}

/// A child of a paragraph.
fn write_inline(n: &N, s: &mut String) {
    match n {
        N::R(..) => write_node(n, s),
        N::T(t) => write_node(&N::R(None, vec![N::T(t.clone())]), s),
        N::Br => s.push_str("<w:r><w:br/></w:r>"),
        N::Raw(r) => {
            s.push_str("<w:r>");
            s.push_str(r);
            s.push_str("</w:r>");
        }
        N::MathPara(m) => write_math(m, s),
        N::Hyperlink { id, children } => {
            match id {
                Some(id) => s.push_str(&format!("<w:hyperlink r:id=\"{}\" w:tgtFrame=\"_blank\">", id)),
                None => s.push_str("<w:hyperlink>"),
            }
            for c in children {
                if let N::R(..) = c {
                    write_node(c, s);
                } else {
                    write_inline(c, s);
                }
            }
            s.push_str("</w:hyperlink>");
        }
        // A paragraph inside a paragraph (an image in a text line): its content goes inline.
        N::P(_, c) => {
            for x in c {
                write_inline(x, s);
            }
        }
        N::Custom { children, .. } => {
            for c in children {
                write_inline(c, s);
            }
        }
        _ => {}
    }
}
