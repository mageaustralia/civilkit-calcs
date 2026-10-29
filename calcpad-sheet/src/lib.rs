//! Sheet syntax: a Markdown-like front end for the Calcpad worksheet language.
//!
//! `compile` turns sheet text into Calcpad worksheet text plus a source map (output line to
//! sheet line), so that the one engine, calcpad-core, runs both. Deterministic, no I/O.
//! The grammar is documented in docs/sheet-syntax.md.

pub mod expr;
mod format;
pub mod input;
mod names;
mod pyexport;
mod tex;
mod unitdef;

pub use format::format;

use std::collections::HashSet;

/// The first line of a sheet that is not a `.sheet` file.
pub const MARKER: &str = "<!--sheet-->";

/// True if the first non-blank line is the sheet marker.
pub fn is_sheet(src: &str) -> bool {
    src.lines()
        .find(|l| !l.trim().is_empty())
        .is_some_and(|l| l.trim().eq_ignore_ascii_case(MARKER))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// For a raw `#UI` line: the length of what comes before the assigned name (the directive, its
/// `{json}` options and its `'label -'`), and the byte range of the name.
fn ui_head(line: &str) -> Option<(usize, usize, usize)> {
    let t = line.trim_start();
    let rest = t.strip_prefix("#UI")?;
    if !rest.starts_with(|c: char| c.is_whitespace() || c == '{' || c == '\'') {
        return None;
    }
    let base = line.len() - rest.len();
    let mut i = 0;
    let skip = |i: &mut usize| {
        while rest[*i..].starts_with(char::is_whitespace) {
            *i += rest[*i..].chars().next().unwrap().len_utf8();
        }
    };
    skip(&mut i);
    if rest[i..].starts_with('{') {
        // To the matching brace, skipping strings.
        let (mut depth, mut instr, mut esc) = (0i32, false, false);
        let mut end = None;
        for (k, c) in rest[i..].char_indices() {
            if instr {
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == '"' {
                    instr = false;
                }
            } else if c == '"' {
                instr = true;
            } else if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    end = Some(i + k + 1);
                    break;
                }
            }
        }
        i = end?;
        skip(&mut i);
    }
    if rest[i..].starts_with('\'') {
        let close = rest[i + 1..].find('\'')?;
        i += close + 2;
        skip(&mut i);
    }
    let n = rest[i..].find(|c: char| !names::is_name_char(c)).unwrap_or(rest.len() - i);
    if n == 0 || !names::is_name_start(rest[i..].chars().next()?) {
        return None;
    }
    let tail = rest[i + n..].trim_start();
    if !tail.starts_with('=') || tail.starts_with("==") {
        return None;
    }
    Some((base + i, base + i, base + i + n))
}

/// A problem found by the compiler, on a sheet line (0 based).
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

/// A name the sheet defines, as written and as the engine spells it.
#[derive(Clone, Debug)]
pub struct Def {
    pub name: String,
    pub calcpad: String,
    pub line: usize,
}

/// What a line of the sheet is, for the editor (which lines are code, which are prose).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    Prose,
    Quantity,
    Block,
    Raw,
    Plot,
    Condition,
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Write formulas as italic TeX source instead of placeholders (for a Calcpad file that
    /// has to read without the page, and for the Word and Excel exports).
    pub plain_tex: bool,
}

#[derive(Clone, Debug)]
pub struct Compiled {
    /// The Calcpad worksheet.
    pub text: String,
    /// For each line of `text` (0 based), the sheet line (0 based) it came from.
    pub map: Vec<usize>,
    pub diagnostics: Vec<Diagnostic>,
    /// True when the output carries formula placeholders.
    pub uses_latex: bool,
    pub defs: Vec<Def>,
    /// The kind of each sheet line (0 based).
    pub kinds: Vec<LineKind>,
}

pub fn compile(src: &str) -> Compiled {
    compile_with(src, &Options::default())
}

pub fn compile_with(src: &str, opts: &Options) -> Compiled {
    let lines: Vec<&str> = src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
    // First pass: which names the sheet defines, so that raw Calcpad lines and function calls
    // can use the sheet's spelling. Second pass: the output.
    let none = HashSet::new();
    let mut first = Comp::new(opts, &none);
    first.run(&lines);
    let found = first.found;
    let mut c = Comp::new(opts, &found);
    c.run(&lines);
    let mut text = c.out.join("\n");
    text.push('\n');
    Compiled {
        text,
        map: c.map,
        diagnostics: c.diags,
        uses_latex: c.latex,
        defs: c.defs,
        kinds: c.kinds,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    If,
    For,
    While,
    Repeat,
}

struct Block {
    kind: Kind,
    header_indent: usize,
    body_indent: Option<usize>,
    header_line: usize,
    had_body: bool,
}

enum Fence {
    Calcpad,
    /// A ```python fence: its lines are the body of a `#python` block, indentation kept.
    /// (indent to strip, export-name scan, opened by a bare `#python` line instead of a fence)
    Python(usize, pyexport::ExportScan, bool),
    Code(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum PKind {
    Normal,
    Code,
    Display,
    Blank,
}

/// A line of prose waiting to be written.
struct Pl {
    text: String,
    idx: usize,
    kind: PKind,
}

struct Stmt {
    sheet: String,
    calcpad: String,
    expr: String,
    unit: Option<String>,
}

struct Comp<'a> {
    opts: &'a Options,
    defined: &'a HashSet<String>,
    found: HashSet<String>,
    defs: Vec<Def>,
    out: Vec<String>,
    map: Vec<usize>,
    diags: Vec<Diagnostic>,
    prose: Vec<Pl>,
    /// A blank sheet line seen since the last output line: written as a blank line before the next one.
    pending_blank: Option<usize>,
    kinds: Vec<LineKind>,
    latex: bool,
    stack: Vec<Block>,
    seq: usize,
    /// A `sheet-form` wrapper is open around a run of input lines (the page styles it).
    form_open: bool,
    /// The statement being read ends with `{key}`.
    key_pending: bool,
    /// The key quantities written so far at the top level, for a `{key results}` line.
    keys: Vec<KeyItem>,
    /// The levels of the collapsible sections that are open (`## Workings {collapse}`).
    collapses: Vec<usize>,
}

/// A quantity or input marked `{key}`.
struct KeyItem {
    /// The name as the engine spells it.
    name: String,
    unit: Option<String>,
    note: Option<String>,
}

/// Leading whitespace: (width with a tab as 4, mixed tabs and spaces, byte length).
fn leading(line: &str) -> (usize, bool, usize) {
    let (mut w, mut sp, mut tab, mut n) = (0, false, false, 0);
    for c in line.chars() {
        match c {
            ' ' => {
                w += 1;
                sp = true;
            }
            '\t' => {
                w += 4;
                tab = true;
            }
            _ => break,
        }
        n += 1;
    }
    (w, sp && tab, n)
}

impl<'a> Comp<'a> {
    fn new(opts: &'a Options, defined: &'a HashSet<String>) -> Self {
        Comp {
            opts,
            defined,
            found: HashSet::new(),
            defs: Vec::new(),
            out: Vec::new(),
            map: Vec::new(),
            diags: Vec::new(),
            prose: Vec::new(),
            pending_blank: None,
            kinds: Vec::new(),
            latex: false,
            stack: Vec::new(),
            seq: 0,
            form_open: false,
            key_pending: false,
            keys: Vec::new(),
            collapses: Vec::new(),
        }
    }

    fn ctx(&self, cond: bool) -> expr::Ctx<'_> {
        expr::Ctx {
            defined: self.defined,
            cond,
        }
    }

    fn diag(&mut self, line: usize, severity: Severity, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            line,
            severity,
            message: message.into(),
        });
    }

    fn emit(&mut self, s: impl Into<String>, line: usize) {
        self.close_form(line);
        self.push_line(s, line);
    }

    /// Ends the wrapper around a run of input lines, before any other output.
    fn close_form(&mut self, line: usize) {
        if self.form_open {
            self.form_open = false;
            self.push_line("'</div>", line);
        }
    }

    fn push_line(&mut self, s: impl Into<String>, line: usize) {
        if let Some(b) = self.pending_blank.take() {
            if !self.out.is_empty() && self.out.last().is_some_and(|l| !l.is_empty()) {
                self.out.push(String::new());
                self.map.push(b);
            }
        }
        self.out.push(s.into());
        self.map.push(line);
    }

    fn mark(&mut self, line: usize, kind: LineKind) {
        if let Some(k) = self.kinds.get_mut(line) {
            *k = kind;
        }
    }

    /// Writes the waiting prose as a person would write it in Calcpad: a heading as a heading line,
    /// a paragraph of plain text as one text line, and Markdown (lists, tables, emphasis, links,
    /// code) in `#markdown` blocks, which the engine renders.
    fn flush_prose(&mut self) {
        let lines = std::mem::take(&mut self.prose);
        let mut trailing = None;
        type Para = (PKind, Vec<(String, usize)>, Option<usize>);
        let mut paras: Vec<Para> = Vec::new();
        let mut open = false;
        let mut blank_before: Option<usize> = None;
        for l in lines {
            match l.kind {
                PKind::Blank => {
                    open = false;
                    trailing = Some(l.idx);
                    blank_before = Some(l.idx);
                    continue;
                }
                PKind::Display => {
                    paras.push((PKind::Display, vec![(l.text, l.idx)], blank_before.take()));
                    open = false;
                }
                PKind::Code => {
                    match paras.last_mut() {
                        Some((PKind::Code, v, _)) if open => v.push((l.text, l.idx)),
                        _ => paras.push((PKind::Code, vec![(l.text, l.idx)], blank_before.take())),
                    }
                    open = true;
                }
                PKind::Normal => {
                    // A heading is a paragraph of its own, whatever surrounds it.
                    let is_heading = heading(&l.text).is_some();
                    match paras.last_mut() {
                        Some((PKind::Normal, v, _)) if open && !is_heading => v.push((l.text, l.idx)),
                        _ => paras.push((PKind::Normal, vec![(l.text, l.idx)], blank_before.take())),
                    }
                    open = !is_heading;
                }
            }
            trailing = None;
        }
        let plain_tex = self.opts.plain_tex;
        let mut md: Vec<(String, usize)> = Vec::new();
        let mut first = true;
        for (kind, ls, blank) in paras {
            let at = ls[0].1;
            let gap = blank.unwrap_or(at);
            // Plain output for this paragraph, if it has one.
            let mut plain: Option<Vec<String>> = None;
            match kind {
                PKind::Display => {
                    let mut used = false;
                    let t = ls[0].0.clone();
                    plain = Some(vec![format!("'{}", tex::block(&t, plain_tex))]);
                    let _ = &mut used;
                    self.latex = self.latex || !plain_tex;
                }
                PKind::Normal => {
                    if let Some((level, text)) = heading(&ls[0].0).filter(|_| ls.len() == 1) {
                        if !needs_md(&tex::mask(text)) {
                            let h = tex::inline(text, plain_tex, false, false, &mut self.latex);
                            plain = Some(vec![if level == 3 {
                                format!("\"{h}")
                            } else {
                                format!("'<h{level}>{h}</h{level}>")
                            }]);
                        }
                    } else if ls.iter().all(|(t, _)| !needs_md(&tex::mask(t))) {
                        let joined = ls.iter().map(|(t, _)| t.trim()).collect::<Vec<_>>().join(" ");
                        let h = tex::inline(&joined, plain_tex, false, false, &mut self.latex);
                        plain = Some(vec![format!("'{h}")]);
                    }
                }
                PKind::Code | PKind::Blank => {}
            }
            match plain {
                Some(out) => {
                    self.flush_md(&mut md);
                    if !first && blank.is_some() {
                        self.pending_blank = blank;
                    }
                    for o in out {
                        self.emit(o, at);
                    }
                }
                None => {
                    if !md.is_empty() {
                        md.push((String::new(), gap));
                    } else if !first && blank.is_some() {
                        self.pending_blank = blank;
                    }
                    for (t, i) in ls {
                        let t = if kind == PKind::Code {
                            t
                        } else {
                            tex::inline(&t, plain_tex, false, true, &mut self.latex)
                        };
                        md.push((t, i));
                    }
                }
            }
            first = false;
        }
        self.flush_md(&mut md);
        if !first {
            self.pending_blank = trailing;
        }
    }

    fn flush_md(&mut self, md: &mut Vec<(String, usize)>) {
        if md.is_empty() {
            return;
        }
        let first = md[0].1;
        let last = md[md.len() - 1].1;
        self.emit("#markdown", first);
        for (t, i) in md.drain(..) {
            self.emit(t, i);
        }
        self.emit("#end markdown", last);
    }

    /// The Calcpad spelling of the sheet names a raw Calcpad line uses.
    fn raw(&self, line: &str) -> String {
        if let Some((head, _, _)) = ui_head(line) {
            // `#UI {json} 'label -'name = value`: the options and the label are text, the rest is a definition.
            return format!("{}{}", &line[..head], self.raw(&line[head..]));
        }
        let cs: Vec<char> = line.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < cs.len() {
            let c = cs[i];
            if c == '\'' || c == '"' {
                out.extend(&cs[i..]);
                break;
            }
            if names::is_name_start(c) || c == '_' {
                let st = i;
                while i < cs.len() && names::is_name_char(cs[i]) {
                    i += 1;
                }
                let run: String = cs[st..i].iter().collect();
                let after_digit = st > 0 && cs[st - 1].is_ascii_digit();
                if !after_digit && self.defined.contains(&run) {
                    out.push_str(&names::rename(&run));
                } else {
                    out.push_str(&run);
                }
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    }

    /// Notes the name a raw Calcpad line assigns, so the sheet can use its spelling.
    fn raw_defs(&mut self, line: &str) {
        if let Some((_, a, b)) = ui_head(line) {
            self.found.insert(line[a..b].to_string());
            return;
        }
        let t = line.trim_start();
        let n = t.find(|c: char| !names::is_name_char(c)).unwrap_or(t.len());
        if n == 0 || !names::is_name_start(t.chars().next().unwrap()) {
            return;
        }
        let mut rest = t[n..].trim_start();
        if rest.starts_with('(') {
            if let Some(p) = rest.find(')') {
                rest = rest[p + 1..].trim_start();
            }
        }
        if rest.starts_with('=') && !rest.starts_with("==") {
            self.found.insert(t[..n].to_string());
        }
    }

    fn close(&mut self) {
        let b = self.stack.pop().unwrap();
        self.flush_prose();
        self.pending_blank = None;
        if !b.had_body {
            self.diag(
                b.header_line,
                Severity::Warning,
                "this block has no body (indent the lines that belong to it)",
            );
        }
        self.emit(if b.kind == Kind::If { "#end if" } else { "#loop" }, b.header_line);
    }

    fn text_line(&mut self, s: &str) -> String {
        let h = tex::inline(s, self.opts.plain_tex, true, false, &mut self.latex);
        format!("'{h}")
    }

    fn run(&mut self, lines: &[&str]) {
        self.kinds = vec![LineKind::Blank; lines.len()];
        let mut fence: Option<Fence> = None;
        let mut math: Option<(Vec<String>, usize)> = None;
        let mut marker_seen = false;
        for (idx, line) in lines.iter().enumerate() {
            let line = *line;
            if !marker_seen && !line.trim().is_empty() {
                marker_seen = true;
                if line.trim().eq_ignore_ascii_case(MARKER) {
                    continue;
                }
            }
            if let Some(f) = &mut fence {
                let closes = match f {
                    Fence::Python(_, _, true) => line.trim().eq_ignore_ascii_case("#end python"),
                    _ => line.trim_start().starts_with("```"),
                };
                match f {
                    Fence::Calcpad => {
                        if closes {
                            fence = None;
                        } else {
                            self.raw_defs(line);
                            let r = self.raw(line.trim_start_matches([' ', '\t']));
                            self.mark(idx, LineKind::Raw);
                            self.emit(r, idx);
                        }
                    }
                    Fence::Python(strip, scan, _) => {
                        self.mark(idx, LineKind::Raw);
                        if closes {
                            fence = None;
                            self.emit("#end python", idx);
                        } else {
                            let (w, _, n) = leading(line);
                            let extra = w.saturating_sub(*strip);
                            let s = format!("{}{}", " ".repeat(extra), scan.line(line[n..].trim_end()));
                            self.emit(s, idx);
                        }
                    }
                    Fence::Code(strip) => {
                        let (w, _, n) = leading(line);
                        let extra = w.saturating_sub(*strip);
                        let s = format!("{}{}", " ".repeat(extra), line[n..].trim_end());
                        self.prose.push(Pl {
                            text: s,
                            idx,
                            kind: PKind::Code,
                        });
                        if closes {
                            fence = None;
                        }
                    }
                }
                continue;
            }
            if let Some((buf, start)) = &mut math {
                if line.trim() == "$$" {
                    let t = buf.join(" ");
                    let start = *start;
                    math = None;
                    self.display(&t, start);
                } else {
                    buf.push(line.trim().to_string());
                }
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !self.prose.is_empty() {
                    self.prose.push(Pl {
                        text: String::new(),
                        idx,
                        kind: PKind::Blank,
                    });
                } else if !self.out.is_empty() {
                    self.pending_blank = Some(idx);
                }
                continue;
            }
            let (indent, mixed, wsn) = leading(line);
            if mixed {
                self.diag(idx, Severity::Error, "the indentation mixes tabs and spaces");
            }
            let content_t = &line[wsn..];
            let content_t = content_t.trim_end();
            let is_else = content_t.starts_with("else") && content_t.ends_with(':') && {
                let h = content_t[..content_t.len() - 1].trim_end();
                h == "else" || h.starts_with("else if ")
            };
            // Close the blocks this line is not part of.
            while let Some(top) = self.stack.last() {
                let keeps_else = is_else && top.kind == Kind::If && indent == top.header_indent;
                if keeps_else {
                    break;
                }
                let under = top.body_indent.is_some_and(|b| indent < b);
                if indent <= top.header_indent || under {
                    self.close();
                } else {
                    break;
                }
            }
            if is_else {
                if self
                    .stack
                    .last()
                    .is_some_and(|t| t.kind == Kind::If && t.header_indent == indent)
                {
                    let h = content_t[..content_t.len() - 1].trim_end();
                    self.flush_prose();
                    self.pending_blank = None;
                    if h == "else" {
                        self.emit("#else", idx);
                    } else {
                        let c = &h["else if ".len()..];
                        match expr::translate(c, &self.ctx(true)) {
                            Ok(e) => self.emit(format!("#else if {e}"), idx),
                            Err(m) => {
                                self.diag(idx, Severity::Error, format!("the condition is not valid: {m}"));
                                self.emit("#else", idx);
                            }
                        }
                    }
                    self.mark(idx, LineKind::Block);
                    let t = self.stack.last_mut().unwrap();
                    t.body_indent = None;
                    t.had_body = false;
                    continue;
                }
                self.diag(
                    idx,
                    Severity::Warning,
                    "'else' without a matching 'if' at the same indentation; read as text",
                );
            }
            if let Some(t) = self.stack.last_mut() {
                if t.body_indent.is_none() && indent > t.header_indent {
                    t.body_indent = Some(indent);
                }
                if indent > t.header_indent {
                    t.had_body = true;
                }
            }
            let strip = self.stack.last().and_then(|b| b.body_indent).unwrap_or(0);
            let extra = indent.saturating_sub(strip);
            let content = content_t.to_string();
            let body = format!("{}{}", " ".repeat(extra), content);
            let can_stmt = !self.stack.is_empty() || indent <= 3;

            if content.trim().eq_ignore_ascii_case("#python") {
                // Calcpad's own spelling of a Python block, as people paste it: the same block as a python fence.
                self.flush_prose();
                self.mark(idx, LineKind::Raw);
                self.emit("#python", idx);
                fence = Some(Fence::Python(strip, pyexport::ExportScan::default(), true));
                continue;
            }
            if content.starts_with("```") {
                let info = content.trim_start_matches('`').trim();
                if info.eq_ignore_ascii_case("calcpad") {
                    self.flush_prose();
                    fence = Some(Fence::Calcpad);
                } else if info.eq_ignore_ascii_case("python") {
                    self.flush_prose();
                    self.mark(idx, LineKind::Raw);
                    self.emit("#python", idx);
                    fence = Some(Fence::Python(strip, pyexport::ExportScan::default(), false));
                } else {
                    self.prose.push(Pl {
                        text: body,
                        idx,
                        kind: PKind::Code,
                    });
                    fence = Some(Fence::Code(strip));
                }
                continue;
            }
            if let Some(rest) = content.strip_prefix(">>") {
                self.flush_prose();
                let rest = rest.strip_prefix(' ').unwrap_or(rest);
                self.raw_defs(rest);
                let r = self.raw(rest);
                self.mark(idx, LineKind::Raw);
                self.emit(r, idx);
                continue;
            }
            if self.unit_def(&content, idx) {
                continue;
            }
            if content.starts_with("'<!--{") && content.ends_with("-->") {
                // A metadata comment (the values saved from the input form): the engine's own comment, as written.
                self.flush_prose();
                self.mark(idx, LineKind::Raw);
                self.emit(content.clone(), idx);
                continue;
            }
            if content == "$$" {
                math = Some((Vec::new(), idx));
                continue;
            }
            if content.len() > 4
                && content.starts_with("$$")
                && content.ends_with("$$")
                && !content[2..content.len() - 2].contains("$$")
            {
                self.display(&content[2..content.len() - 2], idx);
                continue;
            }
            if let Some((level, htext)) = heading(&content) {
                if (indent <= 3 || !self.stack.is_empty()) && self.collapse_heading(level, htext, idx, self.stack.is_empty()) {
                    continue;
                }
            }
            if content.trim() == KEY_RESULTS && can_stmt {
                self.key_results(idx);
                continue;
            }
            if can_stmt {
                let (stripped, key) = split_key(&content);
                self.key_pending = key;
                let done = self.statement(stripped, idx, indent);
                self.key_pending = false;
                if done {
                    continue;
                }
            }
            let escaped = content
                .strip_prefix('\\')
                .filter(|r| r.chars().next().is_some_and(char::is_alphabetic));
            let text = match escaped {
                Some(r) => format!("{}{}", " ".repeat(extra), r),
                None => body,
            };
            self.mark(idx, LineKind::Prose);
            self.prose.push(Pl {
                text,
                idx,
                kind: PKind::Normal,
            });
        }
        if let Some(Fence::Python(_, _, hash)) = fence {
            let last = lines.len().saturating_sub(1);
            let msg = if hash {
                "the #python block is never closed with #end python"
            } else {
                "the python fence is never closed with ```"
            };
            self.diag(last, Severity::Error, msg);
            self.emit("#end python", last);
        }
        if math.is_some() {
            let (buf, start) = math.take().unwrap();
            self.diag(
                start,
                Severity::Error,
                "display math started here is never closed with $$",
            );
            let _ = buf;
        }
        while !self.stack.is_empty() {
            self.close();
        }
        self.flush_prose();
        self.close_form(lines.len().saturating_sub(1));
        let last = lines.len().saturating_sub(1);
        while self.collapses.pop().is_some() {
            self.emit("'</details>", last);
        }
    }

    /// A heading marked `{collapse}` opens a section that the page can fold; any heading closes the
    /// open sections of its own level or deeper. True if the heading was written here.
    fn collapse_heading(&mut self, level: usize, text: &str, idx: usize, top: bool) -> bool {
        let marked = split_collapse(text);
        if marked.is_none() && self.collapses.is_empty() {
            return false;
        }
        let (clean, open) = match marked {
            Some((c, o)) => (c.to_string(), o),
            None => (text.to_string(), false),
        };
        if marked.is_some() && !top {
            self.diag(
                idx,
                Severity::Warning,
                "{collapse} works on a heading outside any if, for, while or repeat block; ignored here",
            );
            self.mark(idx, LineKind::Prose);
            self.prose.push(Pl {
                text: format!("{} {}", "#".repeat(level), clean),
                idx,
                kind: PKind::Normal,
            });
            return true;
        }
        if !top {
            return false;
        }
        self.flush_prose();
        while self.collapses.last().is_some_and(|&l| l >= level) {
            self.collapses.pop();
            self.pending_blank = None;
            self.emit("'</details>", idx);
        }
        self.mark(idx, LineKind::Prose);
        if marked.is_some() {
            self.emit(
                format!("'<details class=\"sheet-collapse\"{}><summary>", if open { " open" } else { "" }),
                idx,
            );
            self.collapses.push(level);
        }
        self.prose.push(Pl {
            text: format!("{} {}", "#".repeat(level), clean),
            idx,
            kind: PKind::Normal,
        });
        if marked.is_some() {
            self.flush_prose();
            self.pending_blank = None;
            self.emit("'</summary>", idx);
        }
        true
    }

    /// `{key results}`: the key quantities defined above, one line each.
    fn key_results(&mut self, idx: usize) {
        self.flush_prose();
        self.mark(idx, LineKind::Quantity);
        let items: Vec<(String, Option<String>, Option<String>)> = self
            .keys
            .iter()
            .map(|k| (k.name.clone(), k.unit.clone(), k.note.clone()))
            .collect();
        if items.is_empty() {
            self.diag(
                idx,
                Severity::Warning,
                "no quantity above this line is marked {key}; add {key} to the end of the lines to list",
            );
            return;
        }
        if !self.opts.plain_tex {
            self.emit("'<div class=\"sheet-key-results\">", idx);
        }
        for (name, unit, note) in items {
            let mut t = match &unit {
                Some(u) => format!("{name}|{u}"),
                None => name,
            };
            if let Some(n) = note {
                t.push_str(&format!(" '{n}"));
            }
            self.emit(t, idx);
        }
        if !self.opts.plain_tex {
            self.emit("'</div>", idx);
        }
    }

    fn display(&mut self, t: &str, line: usize) {
        self.mark(line, LineKind::Prose);
        self.prose.push(Pl {
            text: t.to_string(),
            idx: line,
            kind: PKind::Display,
        });
    }

    /// Reads a line that starts a block, is a one line conditional, a plot or a list of
    /// quantities. False if it is prose.
    fn statement(&mut self, content: &str, idx: usize, indent: usize) -> bool {
        if let Some(head) = content.strip_suffix(':') {
            let head = head.trim_end();
            if let Some(b) = self.header(head, idx, indent) {
                self.flush_prose();
                self.mark(idx, LineKind::Block);
                self.stack.push(b);
                return true;
            }
            let kw = head.split_whitespace().next().unwrap_or("");
            if matches!(kw, "if" | "while" | "for" | "repeat") && head.contains(' ') {
                self.diag(
                    idx,
                    Severity::Warning,
                    format!("looks like a '{kw}' block but is not valid; read as text"),
                );
            }
            return false;
        }
        if self.input(content, idx) {
            return true;
        }
        if content == "break" || content == "continue" {
            if self.stack.iter().any(|b| b.kind != Kind::If) {
                self.flush_prose();
                self.emit(format!("#{content}"), idx);
                return true;
            }
            self.diag(
                idx,
                Severity::Warning,
                format!("'{content}' outside a loop; read as text"),
            );
            return false;
        }
        if content.starts_with("if ") && self.one_line_if(content, idx) {
            self.mark(idx, LineKind::Condition);
            return true;
        }
        if let Some(p) = content.strip_prefix("plot ") {
            let ok = self.plot(p, idx);
            if ok {
                self.mark(idx, LineKind::Plot);
            }
            return ok;
        }
        let ok = self.quantities(content, idx);
        if ok {
            self.mark(idx, LineKind::Quantity);
        }
        ok
    }

    fn header(&mut self, head: &str, idx: usize, indent: usize) -> Option<Block> {
        let mk = |kind| Block {
            kind,
            header_indent: indent,
            body_indent: None,
            header_line: idx,
            had_body: false,
        };
        if let Some(c) = head.strip_prefix("if ") {
            let e = expr::translate(c, &self.ctx(true)).ok()?;
            self.flush_prose();
            self.emit(format!("#if {e}"), idx);
            return Some(mk(Kind::If));
        }
        if let Some(c) = head.strip_prefix("while ") {
            let e = expr::translate(c, &self.ctx(true)).ok()?;
            self.flush_prose();
            self.emit(format!("#while {e}"), idx);
            return Some(mk(Kind::While));
        }
        if let Some(c) = head.strip_prefix("repeat ") {
            let e = expr::translate(c, &self.ctx(false)).ok()?;
            self.flush_prose();
            self.emit(format!("#repeat {e}"), idx);
            return Some(mk(Kind::Repeat));
        }
        if let Some(c) = head.strip_prefix("for ") {
            let p = c.find(" in ")?;
            let var = c[..p].trim();
            let range = c[p + 4..].trim();
            let (range, step) = match range.rfind(" step ") {
                Some(q) => (range[..q].trim(), Some(range[q + 6..].trim())),
                None => (range, None),
            };
            if !names::is_name(var) {
                return None;
            }
            self.found.insert(var.to_string());
            let v = names::rename(var);
            let dots = expr::top_level(range)
                .windows(2)
                .find(|w| w[0].1 == '.' && w[1].1 == '.' && w[1].0 == w[0].0 + 1)
                .map(|w| w[0].0);
            if let Some(d) = dots {
                let a = expr::translate(&range[..d], &self.ctx(false)).ok()?;
                let b = expr::translate(&range[d + 2..], &self.ctx(false)).ok()?;
                self.flush_prose();
                match step {
                    None => self.emit(format!("#for {v} = {a} : {b}"), idx),
                    Some(st) => {
                        // The engine's #for has no step: count the steps and compute the value.
                        let st = expr::translate(st, &self.ctx(false)).ok()?;
                        self.seq += 1;
                        let n = self.seq;
                        self.emit(
                            format!("#for loopi{n} = 1 : floor(({b} - {a})/({st}) + 0.000000001) + 1"),
                            idx,
                        );
                        self.emit("#hide", idx);
                        self.emit(format!("{v} = {a} + (loopi{n} - 1)*({st})"), idx);
                        self.emit("#show", idx);
                    }
                }
            } else {
                if step.is_some() {
                    return None;
                }
                let seq = expr::translate(range, &self.ctx(false)).ok()?;
                self.seq += 1;
                let n = self.seq;
                self.flush_prose();
                let name = if names::is_name(range) {
                    seq
                } else {
                    self.emit("#hide", idx);
                    self.emit(format!("loopseq{n} = {seq}"), idx);
                    self.emit("#show", idx);
                    format!("loopseq{n}")
                };
                self.emit(format!("#for loopi{n} = 1 : len({name})"), idx);
                self.emit("#hide", idx);
                self.emit(format!("{v} = {name}.(loopi{n})"), idx);
                self.emit("#show", idx);
            }
            return Some(mk(Kind::For));
        }
        None
    }

    fn one_line_if(&mut self, content: &str, idx: usize) -> bool {
        let top = expr::top_level(content);
        let Some(&(colon, _)) = top.iter().find(|(_, c)| *c == ':') else {
            return false;
        };
        let Ok(cond) = expr::translate(&content[3..colon], &self.ctx(true)) else {
            return false;
        };
        let rest = &content[colon + 1..];
        let rt = expr::top_level(rest);
        let mut else_at = None;
        for &(p, c) in &rt {
            if c == 'e' && rest[p..].starts_with("else") {
                let after = rest[p + 4..].trim_start();
                let before_ok = rest[..p].ends_with(char::is_whitespace);
                if before_ok && after.starts_with(':') {
                    else_at = Some((p, p + 4 + (rest[p + 4..].len() - after.len()) + 1));
                    break;
                }
            }
        }
        let (a, b) = match else_at {
            Some((p, q)) => (rest[..p].trim(), Some(rest[q..].trim())),
            None => (rest.trim(), None),
        };
        let Some(ea) = self.action(a, false) else { return false };
        let eb = match b {
            Some(b) => match self.action(b, false) {
                Some(x) => Some(x),
                None => return false,
            },
            None => None,
        };
        // Everything parsed: emit.
        self.flush_prose();
        self.emit(format!("#if {cond}"), idx);
        for l in self.action(a, true).unwrap_or_default() {
            self.emit(l, idx);
        }
        let _ = ea;
        if let Some(b) = b {
            self.emit("#else", idx);
            for l in self.action(b, true).unwrap_or_default() {
                self.emit(l, idx);
            }
        }
        let _ = eb;
        self.emit("#end if", idx);
        true
    }

    /// The output lines of one branch: a quoted text or quantities. `commit` records the names.
    fn action(&mut self, a: &str, commit: bool) -> Option<Vec<String>> {
        if a == "break" || a == "continue" {
            return Some(vec![format!("#{a}")]);
        }
        if a.len() >= 2 && a.starts_with('"') && a.ends_with('"') && !a[1..a.len() - 1].contains('"') {
            let t = if commit {
                self.text_line(&a[1..a.len() - 1])
            } else {
                String::new()
            };
            return Some(vec![t]);
        }
        let (stmts, _) = self.parse_quantities(a, false)?;
        if commit {
            for s in &stmts {
                self.found.insert(s.sheet.clone());
            }
        }
        Some(stmts.iter().map(stmt_text).collect())
    }

    /// An input line (`L: ? 6 m | span`): the engine's `#UI` control. False if the line is not one.
    fn input(&mut self, content: &str, idx: usize) -> bool {
        let inp = match input::parse(content) {
            None => {
                if input::is_input_line(content) {
                    self.diag(
                        idx,
                        Severity::Warning,
                        "looks like an input line (`name: ? default`) but is not valid; read as text",
                    );
                }
                return false;
            }
            Some(Err(m)) => {
                self.mark(idx, LineKind::Quantity);
                self.error_line(idx, &format!("input: {m}"));
                return true;
            }
            Some(Ok(i)) => i,
        };
        let (json, default) = match input::directive(&inp) {
            Ok(x) => x,
            Err(m) => {
                self.mark(idx, LineKind::Quantity);
                self.error_line(idx, &format!("input: {m}"));
                return true;
            }
        };
        self.flush_prose();
        self.mark(idx, LineKind::Quantity);
        let cp = names::rename(&inp.name);
        self.found.insert(inp.name.clone());
        if !self.defs.iter().any(|d| d.name == inp.name) {
            self.defs.push(Def {
                name: inp.name.clone(),
                calcpad: cp.clone(),
                line: idx,
            });
        }
        let plain = self.opts.plain_tex;
        let note = inp
            .note
            .as_deref()
            .map(|n| tex::inline(n, plain, true, false, &mut self.latex).replace('\'', "&#39;"));
        let mut text = String::from("#UI ");
        if let Some(j) = &json {
            text.push_str(j);
            text.push(' ');
        }
        // A drop-down or a checkbox shows no name, so the note is its label. A number shows its
        // name and the note follows it, as on a quantity line.
        let labelled = !matches!(inp.kind, input::Kind::Number(_) | input::Kind::Slider { .. });
        if labelled {
            let l = note
                .clone()
                .unwrap_or_else(|| tex::escape_html(&inp.name).replace('\'', "&#39;"));
            text.push_str(&format!("'{l} -'"));
        }
        text.push_str(&format!("{cp} = {default}"));
        let key = self.key_pending;
        if !labelled {
            text.push_str(&comment_tail(note.as_deref(), key, plain));
        } else if key {
            text.push_str(&comment_tail(None, true, plain));
        }
        if key && self.stack.is_empty() {
            self.keys.push(KeyItem {
                name: cp.clone(),
                unit: None,
                note: note.clone(),
            });
        }
        if !plain && !self.form_open {
            self.close_form(idx);
            self.push_line("'<div class=\"sheet-form\">", idx);
            self.form_open = true;
        }
        self.push_line(text, idx);
        true
    }

    fn plot(&mut self, p: &str, idx: usize) -> bool {
        let Some(fpos) = p.rfind(" for ") else { return false };
        let (left, right) = (p[..fpos].trim(), p[fpos + 5..].trim());
        let Some(ip) = right.find(" in ") else { return false };
        let var = right[..ip].trim();
        let range = right[ip + 4..].trim();
        if !names::is_name(var) {
            return false;
        }
        let dots = expr::top_level(range)
            .windows(2)
            .find(|w| w[0].1 == '.' && w[1].1 == '.' && w[1].0 == w[0].0 + 1)
            .map(|w| w[0].0);
        let Some(d) = dots else { return false };
        // An optional `y =` before the expressions is dropped.
        let mut left = left;
        let top = expr::top_level(left);
        if let Some(&(eq, _)) = top.iter().find(|(_, c)| *c == '=') {
            let n = &left[..eq];
            if names::is_name(n.trim()) || (n.contains('(') && n.trim_end().ends_with(')')) {
                left = left[eq + 1..].trim();
            }
        }
        let mut parts = Vec::new();
        let mut start = 0;
        let tl = expr::top_level(left);
        for &(p, c) in &tl {
            if c == ',' {
                parts.push(&left[start..p]);
                start = p + 1;
            }
        }
        parts.push(&left[start..]);
        let mut es = Vec::new();
        for part in parts {
            match expr::translate(part, &self.ctx(false)) {
                Ok(e) => es.push(e),
                Err(m) => {
                    self.error_line(idx, &format!("plot: {m}"));
                    return true;
                }
            }
        }
        let (a, b) = (
            expr::translate(&range[..d], &self.ctx(false)),
            expr::translate(&range[d + 2..], &self.ctx(false)),
        );
        match (a, b) {
            (Ok(a), Ok(b)) => {
                self.flush_prose();
                let v = names::rename(var);
                self.emit(format!("$Plot{{ {} @ {v} = {a} : {b} }}", es.join(" & ")), idx);
            }
            (Err(m), _) | (_, Err(m)) => self.error_line(idx, &format!("plot range: {m}")),
        }
        true
    }

    /// A line that is meant as something but is wrong: an error in the report and a diagnostic.
    fn error_line(&mut self, idx: usize, message: &str) {
        self.diag(idx, Severity::Error, message);
        self.flush_prose();
        self.emit(
            format!(
                "'<span class=\"err\">Sheet syntax: {}</span>",
                tex::escape_html(message)
            ),
            idx,
        );
    }

    fn quantities(&mut self, content: &str, idx: usize) -> bool {
        let Some((stmts, note)) = self.parse_quantities(content, true) else {
            return false;
        };
        self.flush_prose();
        let n = stmts.len();
        for (i, s) in stmts.iter().enumerate() {
            self.found.insert(s.sheet.clone());
            let base = s.sheet.split('(').next().unwrap_or(&s.sheet).to_string();
            self.found.insert(base.clone());
            if !self.defs.iter().any(|d| d.name == base) {
                let cp = s.calcpad.split('(').next().unwrap_or(&s.calcpad).to_string();
                self.defs.push(Def {
                    name: base,
                    calcpad: cp,
                    line: idx,
                });
            }
            let mut text = stmt_text(s);
            if i + 1 == n {
                let h = note
                    .as_ref()
                    .map(|nt| tex::inline(nt, self.opts.plain_tex, true, false, &mut self.latex));
                let key = self.key_pending;
                text.push_str(&comment_tail(h.as_deref(), key, self.opts.plain_tex));
            }
            if self.key_pending && self.stack.is_empty() && !s.sheet.contains('(') {
                self.keys.push(KeyItem {
                    name: s.calcpad.clone(),
                    unit: s.unit.clone(),
                    note: note
                        .as_ref()
                        .map(|nt| tex::inline(nt, self.opts.plain_tex, true, false, &mut self.latex)),
                });
            }
            self.emit(text, idx);
        }
        true
    }

    /// `name: expression [-> unit]` or `name = expression`, several separated by `;`, then
    /// an optional `| note`. None if any part is not one.
    fn parse_quantities(&self, content: &str, allow_note: bool) -> Option<(Vec<Stmt>, Option<String>)> {
        let (parts, note) = split_line(content, allow_note);
        let mut out = Vec::new();
        for part in parts {
            out.push(self.parse_stmt(part.trim())?);
        }
        if out.is_empty() {
            return None;
        }
        Some((out, note))
    }

    fn parse_stmt(&self, s: &str) -> Option<Stmt> {
        let top = expr::top_level(s);
        let cs: Vec<char> = s.chars().collect();
        let _ = cs;
        let mut sep = None;
        for &(p, c) in &top {
            if c == ':' {
                sep = Some((p, 1));
                break;
            }
            if c == '=' {
                let prev = s[..p].chars().last();
                let next = s[p + 1..].chars().next();
                if matches!(prev, Some('<' | '>' | '!' | '=')) || next == Some('=') {
                    return None;
                }
                sep = Some((p, 1));
                break;
            }
        }
        let (p, w) = sep?;
        let lhs = s[..p].trim();
        let rhs_all = s[p + w..].trim();
        let (name, params) = match lhs.find('(') {
            Some(q) if lhs.ends_with(')') => (lhs[..q].trim(), Some(&lhs[q + 1..lhs.len() - 1])),
            _ => (lhs, None),
        };
        if !names::is_name(name) || matches!(name, "if" | "for" | "while" | "repeat" | "else" | "plot") {
            return None;
        }
        let mut cp = names::rename(name);
        if let Some(ps) = params {
            let mut v = Vec::new();
            for q in ps.split(',') {
                let q = q.trim();
                if !names::is_name(q) {
                    return None;
                }
                v.push(names::rename(q));
            }
            cp = format!("{cp}({})", v.join("; "));
        }
        let (rhs, unit) = match top
            .iter()
            .rev()
            .find(|&&(q, c)| c == '-' && rhs_all_pos(s, p + w, q) && s[q..].starts_with("->"))
        {
            Some(&(q, _)) => {
                let u = expr::translate_unit(s[q + 2..].trim()).ok()?;
                (s[p + w..q].trim(), Some(u))
            }
            None => (rhs_all, None),
        };
        let e = expr::translate(rhs, &self.ctx(false)).ok()?;
        let sheet = match params {
            Some(ps) => format!("{name}({})", ps.trim()),
            None => name.to_string(),
        };
        Some(Stmt {
            sheet,
            calcpad: cp,
            expr: e,
            unit,
        })
    }
}

/// A quantity line split into its statements (on `;` outside brackets) and its `| note`
/// (the first single `|`; `||` is the logical or).
pub(crate) fn split_line(content: &str, allow_note: bool) -> (Vec<&str>, Option<String>) {
    let top = expr::top_level(content);
    let mut main = content;
    let mut note = None;
    if allow_note {
        for (k, &(p, c)) in top.iter().enumerate() {
            if c == '|' {
                let dbl = top.get(k + 1).is_some_and(|&(q, d)| d == '|' && q == p + 1)
                    || (k > 0 && top[k - 1].1 == '|' && top[k - 1].0 + 1 == p);
                if !dbl {
                    main = &content[..p];
                    let n = content[p + 1..].trim();
                    if !n.is_empty() {
                        note = Some(n.to_string());
                    }
                    break;
                }
            }
        }
    }
    let mut parts = Vec::new();
    let mut start = 0;
    for &(p, c) in &expr::top_level(main) {
        if c == ';' {
            parts.push(&main[start..p]);
            start = p + 1;
        }
    }
    parts.push(&main[start..]);
    if parts.last().is_some_and(|p| p.trim().is_empty()) && parts.len() > 1 {
        parts.pop();
    }
    (parts, note)
}

/// True if `content` is a line of quantities (the same test the compiler applies).
pub(crate) fn is_quantity_line(content: &str) -> bool {
    let none = HashSet::new();
    let o = Options::default();
    Comp::new(&o, &none).parse_quantities(content, true).is_some()
}

/// The first top-level `:` or `=` of a statement (not part of a comparison), as a byte position.
pub(crate) fn find_sep(s: &str) -> Option<usize> {
    for &(p, c) in &expr::top_level(s) {
        if c == ':' {
            return Some(p);
        }
        if c == '=' {
            let prev = s[..p].chars().last();
            let next = s[p + 1..].chars().next();
            if matches!(prev, Some('<' | '>' | '!' | '=')) || next == Some('=') {
                return None;
            }
            return Some(p);
        }
    }
    None
}

/// `# text` to `###### text`: the level and the text.
/// The marker that ends a line to list in the key results.
const KEY: &str = "{key}";
/// A line that is replaced by the list of key quantities above it.
const KEY_RESULTS: &str = "{key results}";

/// `x: 1 {key}` is `x: 1` and true.
pub(crate) fn split_key(content: &str) -> (&str, bool) {
    let t = content.trim_end();
    match t.strip_suffix(KEY) {
        Some(r) if r.ends_with([' ', '\t']) && !r.trim().is_empty() => (r.trim_end(), true),
        _ => (content, false),
    }
}

/// `Workings {collapse}` is (`Workings`, closed) and `Workings {collapse open}` is (`Workings`, open).
pub(crate) fn split_collapse(text: &str) -> Option<(&str, bool)> {
    let t = text.trim_end();
    for (m, open) in [("{collapse open}", true), ("{collapse}", false)] {
        if let Some(r) = t.strip_suffix(m) {
            if r.ends_with([' ', '\t']) && !r.trim().is_empty() {
                return Some((r.trim_end(), open));
            }
        }
    }
    None
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let t = line.trim_start();
    let n = t.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&n) && t[n..].starts_with(' ') {
        Some((n, t[n..].trim()))
    } else {
        None
    }
}

/// True if a line of prose has Markdown in it that only the Markdown renderer can write: lists,
/// quotes, tables, emphasis, code, links, backslashes and the like.
fn needs_md(l: &str) -> bool {
    let t = l.trim_start();
    if t.starts_with(['-', '+', '>', '|', '#']) {
        return true;
    }
    let digits = t.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && t[digits..].starts_with(['.', ')']) {
        return true;
    }
    if l.ends_with("  ")
        || l.contains("==")
        || l.contains("++")
        || l.contains("http://")
        || l.contains("https://")
        || l.contains("www.")
    {
        return true;
    }
    let cs: Vec<char> = l.chars().collect();
    for (i, &c) in cs.iter().enumerate() {
        match c {
            '*' | '`' | '[' | '\\' | '~' | '^' => return true,
            '_' => {
                let before = i == 0 || cs[i - 1].is_whitespace();
                let after = i + 1 >= cs.len() || cs[i + 1].is_whitespace();
                if before || after {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn rhs_all_pos(_s: &str, from: usize, q: usize) -> bool {
    q >= from
}

/// The comment that ends a quantity line: the note, then the star of a key line. Plain Calcpad gets
/// plain text; the page styles the classes (SHEET_CSS in sheet.ts).
fn comment_tail(note: Option<&str>, key: bool, plain: bool) -> String {
    match (note, key, plain) {
        (None, false, _) => String::new(),
        (Some(h), false, true) => format!(" '{h}"),
        (Some(h), false, false) => format!(" '<span class=\"sheet-note\">{h}</span>"),
        (None, true, true) => " '\u{2605}".to_string(),
        (Some(h), true, true) => format!(" '{h} \u{2605}"),
        (None, true, false) => " '<span class=\"sheet-key\" title=\"key result\">\u{2605}</span>".to_string(),
        (Some(h), true, false) => {
            format!(" '<span class=\"sheet-note\">{h}</span> <span class=\"sheet-key\" title=\"key result\">\u{2605}</span>")
        }
    }
}

fn stmt_text(s: &Stmt) -> String {
    match &s.unit {
        Some(u) => format!("{} = {}|{u}", s.calcpad, s.expr),
        None => format!("{} = {}", s.calcpad, s.expr),
    }
}

#[cfg(test)]
mod tests;
