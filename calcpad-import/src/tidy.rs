//! Two optional clean-ups of imported Calcpad text, both line-based and both leaving equations,
//! tables, pictures and directives exactly as they are:
//!
//! * [`tidy`] drops the formatting the importer carries over from the original (fonts, sizes,
//!   colours) so a paragraph is plain text or a `<p>`/`<hN>` with only its own tags, turns a gap
//!   marker into a short visible note.
//!   Plain layout wrappers (indent and row divs) are dropped so the text reads top to bottom.
//! * [`markdown`] tidies, then writes runs of simple paragraphs as raw Markdown in `#markdown` /
//!   `#end markdown` blocks (no `'` prefix, no HTML): blank lines between paragraphs, `~sub~`, `^sup^`.
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
    let mut s = flatten(out).join("\n");
    if src.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Drops the plain wrapper divs the importer uses for indent and rows (`<div>`, a `margin-left`
/// div, an `imported-row`) with their closing `</div>`, so the text reads top to bottom. A div
/// with any other role (grids, plots, pictures beside text, equations) stays.
fn flatten(lines: Vec<String>) -> Vec<String> {
    let removable = |l: &str| {
        l == "'<div>"
            || (l.starts_with("'<div style=\"margin-left:") && l.ends_with("\">") && l.matches('<').count() == 1)
            || (l.starts_with("'<div class=\"imported-row\"") && l.ends_with("\">") && l.matches('<').count() == 1)
    };
    let mut stack: Vec<bool> = Vec::new();
    let mut out = Vec::with_capacity(lines.len());
    for l in lines {
        if !l.starts_with('\'') {
            out.push(l);
            continue;
        }
        if removable(&l) {
            stack.push(true);
            continue;
        }
        let delta = count(&l, "<div") - count(&l, "</div>");
        if delta > 0 {
            for _ in 0..delta {
                stack.push(false);
            }
        } else if delta < 0 {
            let mut drop = l == "'</div>" && stack.last() == Some(&true);
            for _ in 0..-delta {
                if stack.pop() != Some(true) {
                    drop = false;
                }
            }
            if drop {
                continue;
            }
        }
        out.push(l);
    }
    out
}

/// Tidies, then writes simple paragraphs as Markdown (see the module documentation).
pub fn markdown(src: &str) -> String {
    let t = tidy(src);
    let mut out: Vec<String> = Vec::new();
    let mut block: Vec<String> = Vec::new();
    let mut depth = 0i32;
    // A run of paragraphs is one `#markdown` block, a blank line between them.
    let flush = |block: &mut Vec<String>, out: &mut Vec<String>| {
        if block.is_empty() {
            return;
        }
        out.push("#markdown".to_string());
        for (i, p) in block.drain(..).enumerate() {
            if i > 0 {
                out.push(String::new());
            }
            out.push(p);
        }
        out.push("#end markdown".to_string());
    };
    let all: Vec<&str> = t.lines().collect();
    let mut idx = 0;
    while idx < all.len() {
        let line = all[idx];
        idx += 1;
        // A text table of plain cells becomes a Markdown pipe table.
        if depth == 0 && line == "'<table class=\"imported-grid\">" {
            if let Some((table, used)) = pipe_table(&all[idx..]) {
                block.push(table);
                idx += used;
                continue;
            }
        }
        let converted = if depth == 0 {
            parse_text_line(line).and_then(|p| render_md(&p))
        } else {
            None
        };
        match converted {
            Some(md) => block.push(md),
            None => {
                flush(&mut block, &mut out);
                out.push(line.to_string());
            }
        }
        if line.starts_with('\'') {
            depth += count(line, "<div") + count(line, "<table") - count(line, "</div>") - count(line, "</table>");
            depth = depth.max(0);
        }
    }
    flush(&mut block, &mut out);
    let mut s = out.join("\n");
    if src.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Reads the rest of an `imported-grid` table (after its opening line): rows of cells that each hold
/// simple paragraphs. Returns the pipe table (first row as the header) and how many lines it used.
fn pipe_table(lines: &[&str]) -> Option<(String, usize)> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut i = 0;
    loop {
        match *lines.get(i)? {
            "'</table>" => {
                i += 1;
                break;
            }
            "'<tr>" => i += 1,
            _ => return None,
        }
        let mut cells = Vec::new();
        loop {
            match *lines.get(i)? {
                "'</tr>" => {
                    i += 1;
                    break;
                }
                "'<td>" => i += 1,
                _ => return None,
            }
            let mut text: Vec<String> = Vec::new();
            while *lines.get(i)? != "'</td>" {
                let para = parse_text_line(lines[i])?;
                if para.level.is_some() {
                    return None;
                }
                text.push(render_md(&para)?);
                i += 1;
            }
            i += 1;
            // A cell is one line: its paragraphs run on, and a `|` is escaped.
            cells.push(text.join(" ").replace('\n', " ").replace('|', "\\|"));
        }
        rows.push(cells);
    }
    let cols = rows.first()?.len();
    if cols == 0 || rows.iter().any(|r| r.len() != cols) {
        return None;
    }
    let row = |r: &Vec<String>| format!("| {} |", r.join(" | "));
    let mut out = vec![row(&rows[0]), format!("|{}", " --- |".repeat(cols))];
    out.extend(rows[1..].iter().map(row));
    Some((out.join("\n"), i))
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
            let name = name_attrs
                .split(|c: char| c.is_whitespace() || c == '/')
                .next()
                .unwrap_or("");
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
    Some(Para {
        level,
        style,
        inline: out,
    })
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
    let style = if p.style.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", p.style)
    };
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
    let md = render_md_with(p, Marks::Stars)?;
    // Back-to-back marks (`***d*****:**`) are read differently by every Markdown engine. Italic in
    // underscores keeps them apart (`_**d**_**:**`); failing that the line keeps `<b>` / `<i>`.
    if !md.contains("****") {
        return Some(md);
    }
    let md = render_md_with(p, Marks::Underscores)?;
    if md.contains("****") {
        render_md_with(p, Marks::Html)
    } else {
        Some(md)
    }
}

/// Stands for a hard line break while a paragraph is built; it is written as a backslash and a newline.
const BREAK: char = '\u{1}';

#[derive(Clone, Copy, PartialEq)]
enum Marks {
    Stars,
    Underscores,
    Html,
}

fn render_md_with(p: &Para, marks: Marks) -> Option<String> {
    let html_marks = marks == Marks::Html;
    // A ruled line of `=====` or `-----` is a thematic break (as a bare `===` under text it would
    // turn that text into a heading).
    let bare = p.inline.trim();
    if p.level.is_none() && bare.len() >= 3 && bare.chars().all(|c| c == '=' || c == '-') {
        return Some("---".to_string());
    }
    let mut out = String::new();
    // Open marks: (tag, where its text starts in `out`).
    let mut stack: Vec<(&str, usize)> = Vec::new();
    let b = p.inline.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            let end = p.inline[i..].find('>')? + i;
            let tag = &p.inline[i + 1..end];
            // The engine reads `>=` as the operator `≥`, even right after a tag, so a following `=`
            // is written as an entity.
            let mut skip = 0usize;
            let mut eq_after = |out: &mut String, skip: &mut usize| {
                if p.inline[end + 1..].starts_with('=') {
                    out.push_str("&#61;");
                    *skip = 1;
                }
            };
            match tag {
                "b" | "i" | "u" | "sub" | "sup" | "/b" | "/i" | "/u" | "/sub" | "/sup" if html_marks => {
                    out.push_str(&p.inline[i..=end]);
                    eq_after(&mut out, &mut skip);
                }
                "b" | "i" | "u" | "sub" | "sup" => stack.push((&tag[..], out.len())),
                "/b" | "/i" | "/u" | "/sub" | "/sup" => {
                    let (name, start) = stack.pop()?;
                    if name != &tag[1..] {
                        return None;
                    }
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
                        let flank = (
                            !is_punct(t.chars().next())
                                || lead.len() > 0
                                || before.map_or(true, |c| !c.is_alphanumeric()),
                            !is_punct(t.chars().last())
                                || trail.len() > 0
                                || after.map_or(true, |c| !c.is_alphanumeric()),
                        );
                        let mark = match name {
                            "b" => Some("**"),
                            "i" if marks == Marks::Underscores => Some("_"),
                            "i" => Some("*"),
                            "u" => Some("++"),
                            // `~x~` and `^x^` cannot hold a space.
                            _ if t.contains(char::is_whitespace) => None,
                            "sub" => Some("~"),
                            _ => Some("^"),
                        };
                        match mark {
                            Some(m) if (flank.0 && flank.1) || matches!(name, "sub" | "sup") => {
                                out.push_str(&format!("{lead}{m}{t}{m}{trail}"))
                            }
                            _ => {
                                out.push_str(&format!("{lead}<{name}>{t}</{name}>{trail}"));
                                eq_after(&mut out, &mut skip);
                            }
                        }
                    }
                }
                // A hard line break: a backslash at the end of the line.
                "br" | "br/" | "br /" if p.level.is_none() => out.push(BREAK),
                // Inline HTML that Markdown has no mark for stays as it is.
                _ => {
                    out.push_str(&p.inline[i..=end]);
                    eq_after(&mut out, &mut skip);
                }
            }
            i = end + 1 + skip;
        } else {
            let next = p.inline[i..].find('<').map_or(p.inline.len(), |q| q + i);
            let mut chunk = String::new();
            // Comment lines double their quotes (`''`, `""`); a raw Markdown line does not.
            let text = p.inline[i..next].replace("''", "'").replace("\"\"", "\"");
            let chars: Vec<char> = text.chars().collect();
            for (k, &c) in chars.iter().enumerate() {
                // `==`, `!=`, `>=` and the like are operators to the engine even in prose.
                if c == '=' && (k > 0 && matches!(chars[k - 1], '=' | '!') || chars.get(k + 1) == Some(&'=')) {
                    chunk.push_str("&#61;");
                    continue;
                }
                // `||` is the operator `∨`.
                if c == '|' && (k > 0 && chars[k - 1] == '|' || chars.get(k + 1) == Some(&'|')) {
                    chunk.push_str("&#124;");
                    continue;
                }
                if matches!(c, '\\' | '*' | '_' | '[' | ']' | '`' | '~' | '^') {
                    chunk.push('\\');
                }
                chunk.push(c);
            }
            out.push_str(&chunk.replace("++", "\\+\\+"));
            i = next;
        }
    }
    if !stack.is_empty() {
        return None;
    }
    // A break at either end of a paragraph says nothing (and a trailing backslash would be shown as it is).
    let text = out
        .trim_matches(|c: char| c.is_whitespace() || c == BREAK)
        .replace(BREAK, "\\\n");
    let text = text.as_str();
    let first = text.chars().next()?;
    let second = text.chars().nth(1);
    // A line that would start a block is escaped at the character Markdown can escape: the mark itself
    // for `#`, `>`, `-` and `+`, and the `.` or `)` after the number of an ordered list (`1\.`; a
    // backslash before the digit would be shown as it is).
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let text = if matches!(first, '#' | '>') || (matches!(first, '-' | '+') && second.map_or(true, char::is_whitespace))
    {
        format!("\\{text}")
    } else if digits > 0 && text[digits..].starts_with(['.', ')']) {
        format!("{}\\{}", &text[..digits], &text[digits..])
    } else {
        text.to_string()
    };
    Some(match p.level {
        Some(n) => format!("{} {text}", "#".repeat(n as usize)),
        None => text,
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
        assert_eq!(markdown(src), "#markdown\n## A.1 Density, ρ\n#end markdown");
    }

    #[test]
    fn a_styled_heading_keeps_its_level() {
        let src = "'<h2 class=\"imp-c-800000\" style=\"font-family:Times;font-size:12pt\">Input</h2>";
        assert_eq!(tidy(src), "'<h2>Input</h2>");
    }

    #[test]
    fn inline_marks_survive_and_alignment_is_kept() {
        let src = "'<p style=\"font-size:12pt;text-align:center\"><span class=\"imp-c-2e3192\"><b>Total</b></span> is <i>x</i><sub>1</sub></p>";
        assert_eq!(
            tidy(src),
            "'<p style=\"text-align:center\"><b>Total</b> is <i>x</i><sub>1</sub></p>"
        );
        // Alignment has no Markdown form, so that line stays HTML.
        assert_eq!(markdown(src), tidy(src));
    }

    #[test]
    fn markdown_escapes_and_moves_spaces_out_of_emphasis() {
        let src = "'<p><b>Note: </b>use a_b * 2 and [x]</p>";
        assert_eq!(
            markdown(src),
            "#markdown\n**Note:** use a\\_b \\* 2 and \\[x\\]\n#end markdown"
        );
        // Punctuation against a letter is not emphasis to Markdown, so those marks stay HTML.
        assert_eq!(
            markdown("'<p><b>d:</b>x and <b>ok</b></p>"),
            "#markdown\n<b>d:</b>x and **ok**\n#end markdown"
        );
        assert_eq!(
            markdown("'<p><i><b>d</b></i><b>:</b> x</p>"),
            "#markdown\n_**d**_**:** x\n#end markdown"
        );
        // The engine reads `>=` as an operator, even right after a tag, so a tag followed by `=` is spelled safely.
        assert_eq!(
            markdown("'<p><u>a</u>=1 <sub>x y</sub>=2</p>"),
            "#markdown\n++a++=1 <sub>x y</sub>&#61;2\n#end markdown"
        );
        assert_eq!(
            markdown("'Designer''s 1/8\"\" a == b != c ====="),
            "#markdown\nDesigner's 1/8\" a &#61;&#61; b !&#61; c &#61;&#61;&#61;&#61;&#61;\n#end markdown"
        );
        assert_eq!(
            markdown("'Total\n'=========="),
            "#markdown\nTotal\n\n---\n#end markdown"
        );
        assert_eq!(
            markdown("'a || b | c"),
            "#markdown\na &#124;&#124; b | c\n#end markdown"
        );
        assert_eq!(markdown("'1. first"), "#markdown\n1\\. first\n#end markdown");
        assert_eq!(markdown("'2) second"), "#markdown\n2\\) second\n#end markdown");
        assert_eq!(markdown("'-5 kN"), "#markdown\n-5 kN\n#end markdown");
    }

    #[test]
    fn gap_markers_become_one_visible_note() {
        let g = "'<!-- [import-gap R56 worksheet-reference] reference to another worksheet --><small class=\"imp-gap\" style=\"font-size:8pt\" title=\"x: y\">reference to another worksheet (not converted)</small>";
        let src = format!("{g}\n'text");
        assert_eq!(
            tidy(&src),
            "'<small><i>reference to another worksheet (not converted)</i></small>\n'text"
        );
    }

    #[test]
    fn equations_tables_and_layout_are_untouched() {
        let src = "'<div class=\"imp-eq\"><math class=\"imp-m\"><mi>x</mi></math></div>\n'<table>\n'<td>\nplain in a cell\n'</table>\n#hide\nx = 1\n#end hide";
        assert_eq!(tidy(src), src);
        let md = "'<table>\n'cell text\n'</table>\n'after";
        assert_eq!(
            markdown(md),
            "'<table>\n'cell text\n'</table>\n#markdown\nafter\n#end markdown"
        );
    }

    #[test]
    fn plain_wrapper_divs_are_dropped_and_others_kept() {
        let src = "'<div class=\"imported-row\" style=\"display:flex\">\n'<div>\n'text\n'</div>\n'<div style=\"margin-left:13pt\">\n'<div class=\"imported-grid\">\n'x\n'</div>\n'</div>\n'</div>";
        assert_eq!(tidy(src), "'text\n'<div class=\"imported-grid\">\n'x\n'</div>");
    }

    #[test]
    fn paragraphs_share_one_block_with_marks_markdown_has_for_sub_sup_and_underline() {
        let src = "'<p><b>Note</b></p>\n'<p>f<sub>c</sub> = 4 and x<sup>2</sup>, <u>under</u>, y<sub>a b</sub><br/>next</p>\na = 1\n'tail";
        assert_eq!(
            markdown(src),
            "#markdown\n**Note**\n\nf~c~ = 4 and x^2^, ++under++, y<sub>a b</sub>\\\nnext\n#end markdown\na = 1\n#markdown\ntail\n#end markdown"
        );
        // The marks themselves in plain text are escaped.
        assert_eq!(markdown("'a~b^c ++d"), "#markdown\na\\~b\\^c \\+\\+d\n#end markdown");
    }

    #[test]
    fn a_text_table_becomes_a_pipe_table() {
        let src = "'<table class=\"imported-grid\">\n'<tr>\n'<td>\n'<p style=\"font-size:12pt\">h</p>\n'</td>\n'<td>\n'<p>total <b>height</b> a|b</p>\n'<p>second</p>\n'</td>\n'</tr>\n'<tr>\n'<td>\n'F<sub>w</sub>\n'</td>\n'<td>\n'force\n'</td>\n'</tr>\n'</table>\n'after";
        assert_eq!(
            markdown(src),
            "#markdown\n| h | total **height** a\\|b second |\n| --- | --- |\n| F~w~ | force |\n\nafter\n#end markdown"
        );
        // Uneven rows stay HTML.
        let odd = "'<table class=\"imported-grid\">\n'<tr>\n'<td>\n'a\n'</td>\n'</tr>\n'<tr>\n'<td>\n'b\n'</td>\n'<td>\n'c\n'</td>\n'</tr>\n'</table>";
        assert_eq!(markdown(odd), tidy(odd));
    }

    #[test]
    fn the_header_comment_drops_its_pointer_to_removed_markers() {
        let src = "'<!-- Imported from a foreign worksheet: 1 of 2 regions need attention. Lines that start with [import-gap mark what did not convert. -->";
        assert_eq!(
            tidy(src),
            "'<!-- Imported from a foreign worksheet: 1 of 2 regions need attention. -->"
        );
    }
}
