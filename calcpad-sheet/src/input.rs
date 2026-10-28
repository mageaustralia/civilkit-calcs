//! Input lines: `name: ? 6 m | note`, `name: ? [A = 1, B = 2] | note`, `name: ? on | note`.
//!
//! An input line becomes the engine's own `#UI` control, so the input form, the report and the
//! Word and Excel exports need nothing new. This module only reads the line: `parse` gives the
//! parts, `canonical` writes them back the way the formatter wants them, and the compiler turns
//! them into `#UI` text. The grammar is in docs/sheet-syntax.md, section "Inputs and templates".

use crate::expr::{self, Ctx};
use crate::names;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A number with an optional unit: the default, as written.
    Number(String),
    /// A drop-down (`[...]`) or radio buttons (`(...)`).
    Choice { radio: bool, options: Vec<Opt> },
    /// A checkbox, checked or not.
    Check(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Opt {
    pub default: bool,
    pub label: String,
    /// The value as written.
    pub value: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub name: String,
    /// The separator the line was written with, `:` or `=`.
    pub sep: char,
    pub kind: Kind,
    pub note: Option<String>,
    /// A number input's slider range, `slider 2..12 step 0.5`.
    pub range: Option<Range>,
}

/// The bounds of a slider as written (numbers in the unit of the default), and the optional step.
#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    pub min: String,
    pub max: String,
    pub step: Option<String>,
}

/// The name and the text after `?` when the line is an input line (`name: ? ...`).
fn head(content: &str) -> Option<(String, char, &str)> {
    let top = expr::top_level(content);
    let &(p, c) = top.iter().find(|(_, c)| matches!(c, ':' | '='))?;
    if c == '=' {
        let prev = content[..p].chars().last();
        let next = content[p + 1..].chars().next();
        if matches!(prev, Some('<' | '>' | '!' | '=')) || next == Some('=') {
            return None;
        }
    }
    let name = content[..p].trim();
    let rhs = content[p + 1..].trim_start();
    if !names::is_name(name) || !rhs.starts_with('?') {
        return None;
    }
    Some((name.to_string(), c, &rhs[1..]))
}

/// True if the line is spelled as an input line, whether or not it is a valid one.
pub fn is_input_line(content: &str) -> bool {
    head(content).is_some()
}

/// Reads an input line. `None` if it is not one; `Some(Err(message))` if it is one with a
/// mistake in it (the message says what to write).
pub fn parse(content: &str) -> Option<Result<Input, String>> {
    let (name, sep, rest) = head(content)?;
    // Prose such as `Why: ? is it` stays prose; only a spec that starts like one is an input.
    let t = rest.trim_start();
    let word = t
        .split(|c: char| !c.is_alphabetic())
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let starts = t.is_empty()
        || t.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | '\u{2212}' | '[' | '(' | '|'))
        || matches!(word.as_str(), "on" | "off" | "true" | "false");
    if !starts {
        return None;
    }
    let (parts, note) = crate::split_line(rest, true);
    if parts.len() > 1 {
        return Some(Err("write one input per line".into()));
    }
    let spec = parts.first().map(|p| p.trim()).unwrap_or("");
    let (spec, range) = match split_slider(spec) {
        Ok(x) => x,
        Err(m) => return Some(Err(m)),
    };
    Some(kind(spec).and_then(|kind| {
        if let (Some(r), Kind::Number(v)) = (&range, &kind) {
            check_default(v, r)?;
        }
        Ok(Input { name, sep, kind, note, range })
    }))
}

/// Splits `6 m slider 2..12 step 0.5` into the default and its range. The spec is returned
/// whole when there is no `slider` word.
fn split_slider(spec: &str) -> Result<(&str, Option<Range>), String> {
    let lower = spec.to_ascii_lowercase();
    let Some(at) = lower.match_indices("slider").map(|(i, _)| i).find(|&i| {
        i > 0
            && lower[..i].ends_with(char::is_whitespace)
            && lower[i + 6..].starts_with(|c: char| c.is_whitespace() || c.is_ascii_digit() || c == '-' || c == '.')
    }) else {
        return Ok((spec, None));
    };
    let head = spec[..at].trim_end();
    if head.is_empty() || head.starts_with(['[', '(']) || matches!(head.to_ascii_lowercase().as_str(), "on" | "off" | "true" | "false") {
        return Err(format!("a slider goes after a number default; {SLIDER_HINT}"));
    }
    let r = spec[at + 6..].trim();
    let (bounds, step) = match r.to_ascii_lowercase().find("step") {
        Some(i) => (r[..i].trim(), Some(r[i + 4..].trim())),
        None => (r, None),
    };
    let num = |t: &str, what: &str| -> Result<f64, String> {
        t.replace('\u{2212}', "-")
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("the slider {what} '{t}' is not a number; {SLIDER_HINT}"))
    };
    let (lo, hi) = bounds
        .find("..")
        .map(|p| (bounds[..p].trim(), bounds[p + 2..].trim()))
        .ok_or_else(|| format!("a slider needs a range `min..max`; {SLIDER_HINT}"))?;
    let (a, b) = (num(lo, "minimum")?, num(hi, "maximum")?);
    if a >= b {
        return Err(format!("the slider minimum {lo} must be less than the maximum {hi}"));
    }
    if let Some(st) = step {
        let v = num(st, "step")?;
        if v <= 0.0 || v > b - a {
            return Err(format!("the slider step {st} must be above 0 and no more than the range"));
        }
    }
    let range = Range {
        min: lo.to_string(),
        max: hi.to_string(),
        step: step.map(str::to_string),
    };
    Ok((head, Some(range)))
}

const SLIDER_HINT: &str = "write `name: ? 6 m slider 2..12` or `slider 2..12 step 0.5`";

/// The default has to lie on the slider: its leading number is compared with the bounds.
fn check_default(def: &str, r: &Range) -> Result<(), String> {
    let fix = |t: &str| t.replace('\u{2212}', "-");
    let t = fix(def);
    let v = (1..=t.len())
        .rev()
        .filter(|&n| t.is_char_boundary(n))
        .find_map(|n| t[..n].parse::<f64>().ok());
    let (Some(v), Ok(lo), Ok(hi)) = (v, fix(&r.min).parse::<f64>(), fix(&r.max).parse::<f64>()) else {
        return Ok(());
    };
    if v < lo || v > hi {
        return Err(format!("the default {def} is outside the slider range {}..{}", r.min, r.max));
    }
    Ok(())
}

const HINT: &str = "an input is `name: ? 6 m`, `name: ? [Low = 1, High = 2]` or `name: ? on`";

fn kind(spec: &str) -> Result<Kind, String> {
    if spec.is_empty() {
        return Err(format!("the input has no default; {HINT}"));
    }
    match spec.to_ascii_lowercase().as_str() {
        "on" | "true" => return Ok(Kind::Check(true)),
        "off" | "false" => return Ok(Kind::Check(false)),
        _ => {}
    }
    let first = spec.chars().next().unwrap();
    let close = match first {
        '[' => Some(']'),
        '(' => Some(')'),
        _ => None,
    };
    if let Some(close) = close {
        if !spec.ends_with(close) {
            return Err(format!("the options are not closed with '{close}'; {HINT}"));
        }
        return choice(&spec[1..spec.len() - 1], first == '(');
    }
    Ok(Kind::Number(spec.split_whitespace().collect::<Vec<_>>().join(" ")))
}

fn choice(inner: &str, radio: bool) -> Result<Kind, String> {
    let mut starts = vec![0usize];
    let mut cuts = Vec::new();
    for &(p, c) in &expr::top_level(inner) {
        if c == ',' {
            cuts.push(p);
            starts.push(p + 1);
        }
    }
    let mut ends = cuts;
    ends.push(inner.len());
    let mut options: Vec<Opt> = Vec::new();
    for (s, e) in starts.into_iter().zip(ends) {
        let item = inner[s..e].trim();
        if item.is_empty() {
            return Err("an option is empty (a stray comma?)".into());
        }
        let (default, item) = match item.strip_prefix('*') {
            Some(r) => (true, r.trim_start()),
            None => (false, item),
        };
        let eq = expr::top_level(item)
            .iter()
            .find(|(_, c)| *c == '=')
            .map(|&(p, _)| p)
            .ok_or_else(|| format!("the option '{item}' needs a value: write `Label = value`"))?;
        let label = item[..eq].trim().trim_matches('"').trim().to_string();
        let value = item[eq + 1..].trim().to_string();
        if label.is_empty() {
            return Err("an option has no label".into());
        }
        if label.contains('}') {
            return Err(format!("the label '{label}' cannot contain '}}'"));
        }
        if value.is_empty() {
            return Err(format!("the option '{label}' has no value"));
        }
        options.push(Opt { default, label, value });
    }
    if options.len() < 2 {
        return Err("a choice needs at least two options".into());
    }
    if options.iter().filter(|o| o.default).count() > 1 {
        return Err("only one option can be marked with * as the default".into());
    }
    Ok(Kind::Choice { radio, options })
}

/// The line as the formatter writes it (without the name and the separator).
pub fn canonical(i: &Input) -> String {
    let body = match &i.kind {
        Kind::Number(v) => v.clone(),
        Kind::Check(true) => "on".to_string(),
        Kind::Check(false) => "off".to_string(),
        Kind::Choice { radio, options } => {
            let (a, b) = if *radio { ('(', ')') } else { ('[', ']') };
            let items: Vec<String> = options
                .iter()
                .map(|o| {
                    let label = if o.label.starts_with('*')
                        || o.label.contains([',', '=', '"'])
                        || o.label.contains(['[', ']', '(', ')'])
                    {
                        format!("\"{}\"", o.label)
                    } else {
                        o.label.clone()
                    };
                    format!("{}{} = {}", if o.default { "*" } else { "" }, label, o.value)
                })
                .collect();
            format!("{a}{}{b}", items.join(", "))
        }
    };
    let body = match &i.range {
        Some(r) => match &r.step {
            Some(st) => format!("{body} slider {}..{} step {st}", r.min, r.max),
            None => format!("{body} slider {}..{}", r.min, r.max),
        },
        None => body,
    };
    match &i.note {
        Some(n) => format!("? {body} | {n}"),
        None => format!("? {body}"),
    }
}

/// A number with an optional unit, spaces removed: the only values a `#UI` control takes.
fn ui_value(translated: &str) -> Option<String> {
    let s: String = translated.chars().filter(|c| !c.is_whitespace()).collect();
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    if matches!(b.first(), Some('+' | '-' | '\u{2212}')) {
        i = 1;
    }
    let d0 = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - d0;
    if i < b.len() && b[i] == '.' {
        i += 1;
        let f0 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        digits += i - f0;
    }
    if digits == 0 {
        return None;
    }
    let unit = &b[i..];
    if unit.is_empty() {
        return Some(s);
    }
    let ok =
        unit[0].is_alphabetic() || matches!(unit[0], '°' | '%' | '‰' | '′' | '″') || matches!(unit[0], '\u{b5}' | '\u{3bc}') || unit[0] == 'Ω';
    let rest_ok = unit.iter().all(|c| {
        c.is_alphanumeric()
            || matches!(
                c,
                '^' | '*' | '/' | '·' | '-' | '+' | '(' | ')' | '°' | '%' | '‰' | '′' | '″' | '²' | '³'
            )
    });
    if ok && rest_ok {
        Some(s)
    } else {
        None
    }
}

/// The value as the engine's `#UI` takes it, from the text written in the sheet.
pub fn value(text: &str, what: &str) -> Result<String, String> {
    let none = HashSet::new();
    let ctx = Ctx {
        defined: &none,
        cond: false,
    };
    let t = expr::translate(text, &ctx).map_err(|m| format!("{what} '{text}' is not a number: {m}"))?;
    ui_value(&t).ok_or_else(|| {
        format!("{what} '{text}' must be a number with an optional unit (like `6 m`), not an expression")
    })
}

/// The JSON of a `#UI {...}` directive for a choice or a checkbox, and the default value.
pub fn directive(i: &Input) -> Result<(Option<String>, String), String> {
    fn q(s: &str) -> String {
        let mut o = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                c if (c as u32) < 0x20 => o.push(' '),
                c => o.push(c),
            }
        }
        o.push('"');
        o
    }
    match &i.kind {
        Kind::Number(v) => Ok((None, value(v, "the default")?)),
        Kind::Check(on) => Ok((
            Some("{\"type\":\"checkbox\"}".to_string()),
            if *on { "1" } else { "0" }.to_string(),
        )),
        Kind::Choice { radio, options } => {
            let mut keys = Vec::new();
            let mut vals = Vec::new();
            let mut seen: Vec<(String, &str)> = Vec::new();
            for o in options {
                keys.push(q(&o.label));
                let v = value(&o.value, &format!("the value of '{}'", o.label))?;
                // Two options with one value cannot both be the one the form shows as chosen.
                if let Some((_, other)) = seen.iter().find(|(x, _)| *x == v) {
                    return Err(format!(
                        "the options '{other}' and '{}' have the same value; each option needs its own",
                        o.label
                    ));
                }
                seen.push((v.clone(), &o.label));
                vals.push(q(&v));
            }
            let at = options.iter().position(|o| o.default).unwrap_or(0);
            let def = value(&options[at].value, "the default")?;
            Ok((
                Some(format!(
                    "{{\"type\":\"{}\",\"keys\":[{}],\"values\":[{}]}}",
                    if *radio { "radio" } else { "dropdown" },
                    keys.join(","),
                    vals.join(",")
                )),
                def,
            ))
        }
    }
}
