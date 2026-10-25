//! Two optional clean-ups of imported Calcpad text, both line-based and both leaving equations,
//! tables, pictures and directives exactly as they are:
//!
//! * [`tidy`] drops the formatting the importer carries over from the original (fonts, sizes,
//!   colours) so a paragraph is plain text or a `<p>`/`<hN>` with only its own tags, turns a gap
//!   marker into a short visible note.
//! * [`markdown`] tidies, then writes each simple paragraph as Markdown inside `#md on` / `#md off`.
//!
//! A line that is not a simple paragraph is never rewritten, so both are safe to run on any text.

/// A paragraph read back from an imported text line.
struct Para {
    /// Heading level, from the line's own `<hN>` or inferred from a large bold line.
    level: Option<u8>,
    /// `text-align` / `padding-left` of the paragraph, the only block styling tidy keeps.
    style: String,
    /// Inline content: text with entities, and only `b`, `i`, `u`, `sub`, `sup`, `br` tags.
    inline: String,
}

const SKIP_NOTE: &str = " Lines that start with [import-gap mark what did not convert.";

/// Removes imported formatting and gap markers (see the module documentation).
pub fn tidy(src: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in src.lines() {
        if let Some(note) = gap_note(line) {
            out.push(note);
            continue;
        }
        if line.starts_with("'<!-- Imported from a foreign worksheet") {
            out.push(line.replacen(SKIP_NOTE, "", 1));
        } else if let Some(p) = parse_text_line(line) {
            out.push(render_html(&p));
        } else {
            out.push(line.to_string());
        }
    }
    let mut s = out.join("\n");
    if src.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Tidies, then writes simple paragraphs as Markdown (see the module documentation).
pub fn markdown(src: &str) -> String {
    let t = tidy(src);
    let mut out: Vec<String> = Vec::new();
    let mut open = false;
    let mut depth = 0i32;
    for line in t.lines() {
        let converted = if depth == 0 { parse_text_line(line).and_then(|p| render_md(&p)) } else { None };
        match converted {
            Some(md) => {
                if !open {
                    out.push("#md on".to_string());
                    open = true;
                }
                out.push(md);
            }
            None => {
                if open {
                    out.push("#md off".to_string());
                    open = false;
                }
                out.push(line.to_string());
            }
        }
        if line.starts_with('\'') {
            depth += count(line, "<div") + count(line, "<table") - count(line, "</div>") - count(line, "</table>");
            depth = depth.max(0);
        }
    }
    if open {
        out.push("#md off".to_string());
    }
    let mut s = out.join("\n");
    if src.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn is_punct(c: Option<char>) -> bool {
    c.is_some_and(|c| !c.is_alphanumeric() && !c.is_whitespace())
}

fn count(s: &str, pat: &str) -> i32 {
    s.matches(pat).count() as i32
}

/// `'<!-- [import-gap Rn code] reason --><small ...>text</small>` becomes a short visible note.
fn gap_note(line: &str) -> Option<String> {
    let rest = line.strip_prefix("'<!-- [import-gap ")?;
    let after = &rest[rest.find(" --><small")? + 4..];
    let open_end = after.find('>')?;
    let body = after[open_end + 1..].strip_suffix("</small>")?;
    if body.contains('<') {
        return None;
    }
    Some(format!("'<small><i>{body}</i></small>"))
}

/// Reads a `'` line that is one paragraph: plain text, or a `<p>` / `<hN>` holding only text and
/// simple inline tags. Anything else (layout, equations, tables, pictures, comments) is `None`.
fn parse_text_line(line: &str) -> Option<Para> {
    let body = line.strip_prefix('\'')?;
    let (outer_style, mut level, inner): (String, Option<u8>, &str) = if let Some(rest) = body.strip_prefix("<p") {
        let (attrs, inner) = split_open(rest, "</p>")?;
        (attrs.to_string(), None, inner)
    } else if let Some(n) = heading_level(body) {
        let rest = &body[3..];
        let (attrs, inner) = split_open(rest, &format!("</h{n}>"))?;
        (attrs.to_string(), Some(n), inner)
    } else if body.starts_with('<') || body.is_empty() {
        return None;
    } else {
        (String::new(), None, body)
    };

    let mut out = String::new();
    let mut size = style_size(&outer_style).unwrap_or(0.0);
    let (mut chars, mut bold_chars, mut bold_depth) = (0usize, 0usize, 0i32);
    let mut i = 0;
    let b = inner.as_bytes();
    while i < b.len() {
        if b[i] == b'<' {
            let end = inner[i..].find('>')? + i;
            let tag = &inner[i + 1..end];
            let (closing, name_attrs) = match tag.strip_prefix('/') {
                Some(t) => (true, t),
                None => (false, tag),
            };
            let name = name_attrs.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
            match name {
                "span" => {
                    if !closing {
                        if let Some(z) = style_size(name_attrs) {
                            size = size.max(z);
                        }
                    }
                }
                "b" => {
                    bold_depth += if closing { -1 } else { 1 };
                    out.push_str(&inner[i..=end]);
                }
                "i" | "u" | "sub" | "sup" | "br" => out.push_str(&inner[i..=end]),
                _ => return None,
            }
            i = end + 1;
        } else {
            let next = inner[i..].find('<').map_or(inner.len(), |p| p + i);
            let text = &inner[i..next];
            let n = text.chars().filter(|c| !c.is_whitespace()).count();
            chars += n;
            if bold_depth > 0 {
                bold_chars += n;
            }
            out.push_str(text);
            i = next;
        }
    }
    if chars == 0 {
        return None;
    }
    for t in ["</b><b>", "</i><i>", "</u><u>"] {
        while out.contains(t) {
            out = out.replace(t, "");
        }
    }
    // A short line that is all bold and larger than body text is a heading.
    if level.is_none() && bold_chars == chars && !out.contains("<br") && chars <= 120 {
        level = match size {
            z if z >= 20.0 => Some(1),
            z if z >= 16.0 => Some(2),
            z if z >= 13.0 => Some(3),
            _ => None,
        };
        if level.is_some() {
            out = out.replace("<b>", "").replace("</b>", "");
        }
    }
    let style = ["text-align", "padding-left"]
        .iter()
        .filter_map(|p| style_prop(&outer_style, p).map(|v| format!("{p}:{v}")))
        .collect::<Vec<_>>()
        .join(";");
    Some(Para { level, style, inline: out })
}

fn heading_level(body: &str) -> Option<u8> {
    let b = body.as_bytes();
    if b.len() > 3 && b[0] == b'<' && b[1] == b'h' && (b'1'..=b'6').contains(&b[2]) && matches!(b[3], b' ' | b'>') {
        Some(b[2] - b'0')
    } else {
        None
    }
}

/// `rest` follows `<p` or `<hN`; returns the attribute text and the content up to `close`.
fn split_open<'a>(rest: &'a str, close: &str) -> Option<(&'a str, &'a str)> {
    if !(rest.starts_with(' ') || rest.starts_with('>')) {
        return None;
    }
    let gt = rest.find('>')?;
    let inner = rest[gt + 1..].strip_suffix(close)?;
    Some((&rest[..gt], inner))
}

fn style_prop(attrs: &str, prop: &str) -> Option<String> {
    let s = attrs.find("style=\"")? + 7;
    let style = &attrs[s..s + attrs[s..].find('"')?];
    style.split(';').find_map(|d| {
        let (k, v) = d.split_once(':')?;
        (k.trim() == prop).then(|| v.trim().to_string())
    })
}

fn style_size(attrs: &str) -> Option<f64> {
    style_prop(attrs, "font-size")?.trim_end_matches("pt").parse().ok()
}

fn render_html(p: &Para) -> String {
    let style = if p.style.is_empty() { String::new() } else { format!(" style=\"{}\"", p.style) };
    match p.level {
        Some(n) => format!("'<h{n}{style}>{}</h{n}>", p.inline.trim()),
        None if p.inline.contains('<') || !style.is_empty() => format!("'<p{style}>{}</p>", p.inline),
        None => format!("'{}", p.inline),
    }
}

/// The Markdown form of a paragraph, or `None` when Markdown cannot say it (alignment, indent).
fn render_md(p: &Para) -> Option<String> {
    if !p.style.is_empty() {
        return None;
    }
    let md = render_md_with(p, false)?;
    // Back-to-back marks (`***d*****:**`) are read differently by every Markdown engine, so such a
    // line keeps `<b>` / `<i>` and still gets Markdown for the rest.
    if md.contains("****") { render_md_with(p, true) } else { Some(md) }
}

fn render_md_with(p: &Para, html_marks: bool) -> Option<String> {
    let mut out = String::new();
    // Open emphasis: (marker, where its text starts in `out`).
    let mut stack: Vec<(&str, usize)> = Vec::new();
    let b = p.inline.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            let end = p.inline[i..].find('>')? + i;
            let tag = &p.inline[i + 1..end];
            match tag {
                "b" | "i" | "/b" | "/i" if html_marks => out.push_str(&p.inline[i..=end]),
                "b" => stack.push(("**", out.len())),
                "i" => stack.push(("*", out.len())),
                "/b" | "/i" => {
                    let (m, start) = stack.pop()?;
                    let seg = out.split_off(start);
                    let t = seg.trim();
                    if t.is_empty() {
                        out.push_str(&seg);
                    } else {
                        let lead = &seg[..seg.len() - seg.trim_start().len()];
                        let trail = &seg[seg.trim_end().len()..];
                        let before = out.chars().last();
                        let after = p.inline[end + 1..].chars().next();
                        // Markdown only reads a mark as emphasis next to punctuation when the
                        // other side is a space or punctuation too; otherwise keep the HTML tag.
                        let (open_ok, close_ok) = (
                            !is_punct(t.chars().next()) || lead.len() > 0 || before.map_or(true, |c| !c.is_alphanumeric()),
                            !is_punct(t.chars().last()) || trail.len() > 0 || after.map_or(true, |c| !c.is_alphanumeric()),
                        );
                        if open_ok && close_ok {
                            out.push_str(&format!("{lead}{m}{t}{m}{trail}"));
                        } else {
                            let tag = if m == "**" { "b" } else { "i" };
                            out.push_str(&format!("{lead}<{tag}>{t}</{tag}>{trail}"));
                        }
                    }
                }
                // Inline HTML that Markdown has no mark for stays as it is.
                _ => out.push_str(&p.inline[i..=end]),
            }
            i = end + 1;
        } else {
            let next = p.inline[i..].find('<').map_or(p.inline.len(), |q| q + i);
            for c in p.inline[i..next].chars() {
                if matches!(c, '\\' | '*' | '_' | '[' | ']' | '`') {
                    out.push('\\');
                }
                out.push(c);
            }
            i = next;
        }
    }
    if !stack.is_empty() {
        return None;
    }
    let text = out.trim();
    let mut lead = text.chars();
    let first = lead.next()?;
    let block_start = matches!(first, '#' | '>' | '-' | '+')
        || (first.is_ascii_digit() && text.trim_start_matches(|c: char| c.is_ascii_digit()).starts_with(['.', ')']));
    let text = if block_start { format!("\\{text}") } else { text.to_string() };
    Some(match p.level {
        Some(n) => format!("'{} {text}", "#".repeat(n as usize)),
        None => format!("'{text}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SANS: &str = "font-family:Arial,Helvetica,Liberation Sans,sans-serif;font-size:10pt";

    #[test]
    fn plain_paragraph_loses_its_fonts() {
        let src = format!("'<p><span style=\"{SANS}\">Some of the examples.</span></p>");
        assert_eq!(tidy(&src), "'Some of the examples.");
    }

    #[test]
    fn a_large_bold_line_becomes_a_heading() {
        let src = "'<p><span class=\"imp-c-000080\" style=\"font-family:Arial;font-size:18pt\"><b>A.1 Density, </b></span><span class=\"imp-c-000080\" style=\"font-size:18pt\"><b>ρ</b></span></p>";
        assert_eq!(tidy(src), "'<h2>A.1 Density, ρ</h2>");
        assert_eq!(markdown(src), "#md on\n'## A.1 Density, ρ\n#md off");
    }

    #[test]
    fn a_styled_heading_keeps_its_level() {
        let src = "'<h2 class=\"imp-c-800000\" style=\"font-family:Times;font-size:12pt\">Input</h2>";
        assert_eq!(tidy(src), "'<h2>Input</h2>");
    }

    #[test]
    fn inline_marks_survive_and_alignment_is_kept() {
        let src = "'<p style=\"font-size:12pt;text-align:center\"><span class=\"imp-c-2e3192\"><b>Total</b></span> is <i>x</i><sub>1</sub></p>";
        assert_eq!(tidy(src), "'<p style=\"text-align:center\"><b>Total</b> is <i>x</i><sub>1</sub></p>");
        // Alignment has no Markdown form, so that line stays HTML.
        assert_eq!(markdown(src), tidy(src));
    }

    #[test]
    fn markdown_escapes_and_moves_spaces_out_of_emphasis() {
        let src = "'<p><b>Note: </b>use a_b * 2 and [x]</p>";
        assert_eq!(markdown(src), "#md on\n'**Note:** use a\\_b \\* 2 and \\[x\\]\n#md off");
        // Punctuation against a letter is not emphasis to Markdown, so those marks stay HTML.
        assert_eq!(markdown("'<p><b>d:</b>x and <b>ok</b></p>"), "#md on\n'<b>d:</b>x and **ok**\n#md off");
        assert_eq!(markdown("'<p><i><b>d</b></i><b>:</b> x</p>"), "#md on\n'<i><b>d</b></i><b>:</b> x\n#md off");
        assert_eq!(markdown("'1. first"), "#md on\n'\\1. first\n#md off");
    }

    #[test]
    fn gap_markers_become_one_visible_note() {
        let g = "'<!-- [import-gap R56 worksheet-reference] reference to another worksheet --><small class=\"imp-gap\" style=\"font-size:8pt\" title=\"x: y\">reference to another worksheet (not converted)</small>";
        let src = format!("{g}\n'text");
        assert_eq!(tidy(&src), "'<small><i>reference to another worksheet (not converted)</i></small>\n'text");
    }

    #[test]
    fn equations_tables_and_layout_are_untouched() {
        let src = "'<div class=\"imp-eq\"><math class=\"imp-m\"><mi>x</mi></math></div>\n'<table>\n'<td>\nplain in a cell\n'</table>\n#hide\nx = 1\n#end hide";
        assert_eq!(tidy(src), src);
        let md = "'<table>\n'cell text\n'</table>\n'after";
        assert_eq!(markdown(md), "'<table>\n'cell text\n'</table>\n#md on\n'after\n#md off");
    }

    #[test]
    fn the_header_comment_drops_its_pointer_to_removed_markers() {
        let src = "'<!-- Imported from a foreign worksheet: 1 of 2 regions need attention. Lines that start with [import-gap mark what did not convert. -->";
        assert_eq!(tidy(src), "'<!-- Imported from a foreign worksheet: 1 of 2 regions need attention. -->");
    }
}
