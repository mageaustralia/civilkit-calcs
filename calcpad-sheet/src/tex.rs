//! LaTeX in prose. `$...$` and `$$...$$` become placeholder elements that the page fills in
//! (KaTeX); the TeX source is carried in an attribute, written as character references so that
//! neither the Markdown pass nor the worksheet parser can touch it.

/// Writes every character except ASCII letters and digits as a numeric character reference.
pub fn encode(tex: &str) -> String {
    let mut s = String::with_capacity(tex.len() * 2);
    for c in tex.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else {
            s.push_str(&format!("&#{};", c as u32));
        }
    }
    s
}

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The placeholder for one formula. `plain` writes the TeX source as italic text instead
/// (for a Calcpad file that has to read without the page). `md` says the text goes through the
/// Markdown pass, which would touch readable TeX, so the source is written as character references.
pub fn span(tex: &str, display: bool, plain: bool, md: bool) -> String {
    let tex = tex.trim();
    if plain {
        if md {
            return format!("<span class=\"sheet-tex-src\"><i>{}</i></span>", encode(tex));
        }
        return format!("<i>{}</i>", escape_html(tex));
    }
    format!(
        "<span class=\"sheet-tex\"{} data-tex=\"{}\"></span>",
        if display { " data-display=\"1\"" } else { "" },
        encode(tex)
    )
}

/// A formula on a line of its own.
pub fn block(tex: &str, plain: bool) -> String {
    let tex = tex.trim();
    if plain {
        return format!("<p><i>{}</i></p>", escape_html(tex));
    }
    format!(
        "<div class=\"sheet-tex\" data-display=\"1\" data-tex=\"{}\"></div>",
        encode(tex)
    )
}

/// Replaces the formulas of a line of prose. Code spans and `\$` are left alone. Sets `used`
/// when a formula was found.
pub fn inline(line: &str, plain: bool, esc: bool, md: bool, used: &mut bool) -> String {
    inline_with(line, esc, used, &|tex, display| span(tex, display, plain, md))
}

/// Writes the apostrophes of a prose line, outside HTML tags, as `&#39;`. In a Calcpad text line a second `'` starts an
/// expression, so `'It's the sheet's own.` would be read as text followed by code; the entity shows the same character.
pub fn apostrophes(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                out.push(c);
            }
            '>' if in_tag => {
                in_tag = false;
                out.push(c);
            }
            '\'' if !in_tag => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// The line with every formula replaced by a letter, for asking what else the line contains.
pub fn mask(line: &str) -> String {
    let mut used = false;
    inline_with(line, false, &mut used, &|_, _| "x".to_string())
}

fn inline_with(line: &str, esc: bool, used: &mut bool, make: &dyn Fn(&str, bool) -> String) -> String {
    let cs: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '`' {
            let mut n = 0;
            while i + n < cs.len() && cs[i + n] == '`' {
                n += 1;
            }
            // The closing run has the same length.
            let mut j = i + n;
            let mut end = None;
            while j < cs.len() {
                if cs[j] == '`' {
                    let mut m = 0;
                    while j + m < cs.len() && cs[j + m] == '`' {
                        m += 1;
                    }
                    if m == n {
                        end = Some(j + m);
                        break;
                    }
                    j += m;
                } else {
                    j += 1;
                }
            }
            let stop = end.unwrap_or(i + n);
            for ch in &cs[i..stop] {
                put(&mut out, *ch, esc);
            }
            i = stop;
            continue;
        }
        if c == '\\' && i + 1 < cs.len() {
            put(&mut out, c, esc);
            put(&mut out, cs[i + 1], esc);
            i += 2;
            continue;
        }
        if c == '$' {
            if cs.get(i + 1) == Some(&'$') {
                if let Some(end) = find(&cs, i + 2, true) {
                    let tex: String = cs[i + 2..end].iter().collect();
                    out.push_str(&make(&tex, true));
                    *used = true;
                    i = end + 2;
                    continue;
                }
            } else if cs.get(i + 1).is_some_and(|n| !n.is_whitespace()) {
                if let Some(end) = find(&cs, i + 1, false) {
                    let tex: String = cs[i + 1..end].iter().collect();
                    out.push_str(&make(&tex, false));
                    *used = true;
                    i = end + 1;
                    continue;
                }
            }
        }
        put(&mut out, c, esc);
        i += 1;
    }
    out
}

fn put(out: &mut String, c: char, esc: bool) {
    match c {
        '&' if esc => out.push_str("&amp;"),
        '<' if esc => out.push_str("&lt;"),
        '>' if esc => out.push_str("&gt;"),
        _ => out.push(c),
    }
}

/// The index of the closing `$` (or `$$`) at or after `from`.
fn find(cs: &[char], from: usize, double: bool) -> Option<usize> {
    let mut j = from;
    while j < cs.len() {
        match cs[j] {
            '\\' => j += 2,
            '$' => {
                if double {
                    if cs.get(j + 1) == Some(&'$') && j > from {
                        return Some(j);
                    }
                } else if j > from && !cs[j - 1].is_whitespace() && !cs.get(j + 1).is_some_and(|n| n.is_ascii_digit()) {
                    return Some(j);
                }
                j += 1;
            }
            _ => j += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_formulas() {
        let mut used = false;
        let s = inline("the cost is $5 and $6.", false, false, false, &mut used);
        assert!(!used && s == "the cost is $5 and $6.");
        let s = inline("here $x_1$ is a span and `$y$` is code", false, false, false, &mut used);
        assert!(used);
        assert!(s.contains("data-tex=\"x&#95;1\""));
        assert!(s.contains("`$y$`"));
    }

    #[test]
    fn display_spans() {
        let mut used = false;
        let s = inline("so $$a^2$$ holds", false, false, false, &mut used);
        assert!(s.contains("data-display=\"1\"") && s.contains("a&#94;2"));
    }
}
