//! Worksheets that a reference includes.
//!
//! A `.mcdx` package keeps a copy of each worksheet a reference region points at. What that
//! worksheet defines is in force from the reference down, but its equations are not shown on the
//! referring page. The math regions of the copy are therefore read in at the place of the reference,
//! with ids of their own (`i8_12`), and the translator computes them without showing them (see
//! [`quiet_lines`]). Text, pictures, plots and components of the copy are not read, and a copy that
//! itself holds references is not followed.

use crate::ir::*;
use crate::mcdx::{self, Ctx};
use crate::xmcd::is_ws;
use roxmltree::Node;
use std::collections::HashMap;

/// The regions of the worksheet a reference includes, and the line that says so, or `None` when the
/// package holds no copy (or it cannot be read).
pub(crate) fn read_include(ctx: &Ctx, reference: Node, at: (f64, f64)) -> Option<(Paragraph, Vec<Region>)> {
    if ctx.in_include {
        return None;
    }
    let idref = reference.attribute("item-idref")?;
    let (part, _) = ctx.rels.get(idref)?.clone();
    let bytes = ctx.zip.read(&part, ctx.lim.max_item_bytes).ok().flatten()?;
    let xml = mcdx::utf8(&bytes).ok()?;
    let doc = mcdx::parse(xml, ctx.lim).ok()?;
    let container = doc.root_element().children().find(|c| is_ws(*c, "regions"))?;
    let inner = Ctx {
        zip: ctx.zip,
        rels: mcdx::relationships(ctx.zip, &part, ctx.lim),
        results: HashMap::new(),
        lim: ctx.lim,
        component_names: Default::default(),
        in_include: true,
    };
    let mut all = Vec::new();
    for r in container.children().filter(|c| is_ws(*c, "region")) {
        inner.collect_region(r, 0, None, &mut all);
    }
    let tag: String = part
        .rsplit('/')
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let mut regions = Vec::new();
    for r in all {
        if !matches!(r.kind, RegionKind::Math(_)) {
            continue;
        }
        regions.push(Region {
            id: format!("i{}_{}", if tag.is_empty() { "0" } else { &tag }, r.id),
            left: at.0,
            top: at.1,
            baseline: None,
            line: 0,
            width: 0.0,
            height: 0.0,
            rendering: None,
            kind: r.kind,
        });
    }
    if regions.is_empty() {
        return None;
    }
    let name: String = reference
        .attribute("href")
        .unwrap_or("another worksheet")
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | '&' | '"' | '\''))
        .take(80)
        .collect();
    let note = Paragraph {
        style: "Normal".into(),
        pieces: vec![Piece::Run(Run {
            text: format!("Reference: {name} (its definitions are included, not shown)"),
            italic: true,
            ..Run::default()
        })],
        ..Paragraph::default()
    };
    Some((note, regions))
}

/// Whether a region id is one of an included worksheet's.
pub(crate) fn is_included(id: &str) -> bool {
    let mut ch = id.chars();
    ch.next() == Some('i') && ch.next().is_some_and(|c| c.is_ascii_digit()) && id.contains('_')
}

/// What an included region leaves on the page: its computation and nothing it would display. An
/// evaluation leaves nothing; a definition leaves its code, in one hidden block.
pub(crate) fn quiet_lines(lines: &[String]) -> Vec<String> {
    let mut keep: Vec<String> = Vec::new();
    let mut skip_to: Option<&str> = None;
    for block in lines {
        for l in block.lines() {
            let t = l.trim();
            if let Some(end) = skip_to {
                if t == end {
                    skip_to = None;
                }
                continue;
            }
            match t {
                "#noc" => skip_to = Some("#end noc"),
                "#val" => skip_to = Some("#end val"),
                "#hide" | "#end hide" | "#show" | "#nosub" | "" => {}
                _ if t.starts_with('\'') || t.starts_with("#round") || t.starts_with("#format") => {}
                _ => keep.push(l.to_string()),
            }
        }
    }
    if keep.is_empty() {
        return Vec::new();
    }
    let mut out = vec!["#hide".to_string()];
    out.extend(keep);
    out.push("#end hide".to_string());
    vec![out.join("\n")]
}
