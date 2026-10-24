//! The format-neutral intermediate form between a reader (a "front door") and the translator.
//!
//! A reader turns one file format into a [`Sheet`]; the translator turns a [`Sheet`] into
//! Calcpad text. Nothing in this module knows about XML. A reader for another container of the
//! same worksheet language (for instance a zipped one) only has to produce a [`Sheet`].

use crate::report::Code;

/// A whole worksheet: the array origin of its settings and its regions in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    /// The array origin setting (0 or 1 in practice). Indexes in the sheet are relative to it.
    pub origin: i64,
    /// Regions, nested ones (math inside a text region) already flattened out. Not yet in
    /// reading order: the translator sorts them.
    pub regions: Vec<Region>,
    /// Names that components the reader does not run (spreadsheet and similar input components)
    /// define in the file.
    pub component_names: Vec<Name>,
    /// Named paragraph styles the file defines (empty when it has none or the reader does not read them).
    pub text_styles: std::collections::HashMap<String, TextStyle>,
    /// The number of decimal places the file shows results with unless a region says otherwise.
    pub default_precision: Option<i32>,
}

/// What a named paragraph style says about its text. `None` and `false` mean "not stated".
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextStyle {
    pub font: Option<String>,
    /// Size in points.
    pub size: Option<f64>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// `#rrggbb`.
    pub color: Option<String>,
    pub align: Option<String>,
}

/// What the reader found in a plot region.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlotData {
    /// An x-y plot whose traces and axis expressions were read.
    Xy(Box<XyPlot>),
    /// A kind of plot the importer does not read (the element name: a surface, polar or contour plot).
    Other(String),
    /// The plot is stored as an opaque binary item (the older file format), or has no content.
    #[default]
    Unread,
}

/// An x-y plot. Trace `k` pairs the k-th x expression with the k-th y expression; when one axis has
/// fewer expressions than there are traces, its last expression is used for the remaining traces.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct XyPlot {
    pub title: Option<String>,
    pub traces: Vec<TraceStyle>,
    pub x: PlotAxis,
    pub y: PlotAxis,
}

/// How the file draws one trace. The engine draws its own colours and markers, so this is only
/// counted for the note that says it is not carried.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TraceStyle {
    pub color: Option<String>,
    pub symbol: Option<String>,
    pub line_style: Option<String>,
    /// `lines`, `columns`, `waterfall`, ...
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlotAxis {
    /// One slot per expression typed on the axis; `None` is an empty slot (a placeholder).
    pub exprs: Vec<Option<Expr>>,
    /// The unit typed beside each expression (the axis shows the quantity in that unit).
    pub units: Vec<Option<Expr>>,
    /// The range the axis was last drawn with.
    pub start: Option<f64>,
    pub end: Option<f64>,
    /// The domain the author typed (`startValue` and `endValue`), when it is given.
    pub from: Option<Expr>,
    pub to: Option<Expr>,
    pub log: bool,
    pub label: Option<String>,
}

/// A picture region: a vector metafile (WMF or EMF bytes) and/or a PNG.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Picture {
    pub metafile: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
}

/// One region of the page. Position is kept only so that the translator can order the regions.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub id: String,
    pub left: f64,
    pub top: f64,
    /// The baseline the region sits on, when the file gives one; used to group regions into rows.
    pub baseline: Option<f64>,
    /// 1-based line in the source file (0 when the reader has no line information).
    pub line: u32,
    /// Size on the page in points (0 when the file gives none).
    pub width: f64,
    pub height: f64,
    /// The tool's own saved picture of the region (PNG bytes), when the file carries one.
    pub rendering: Option<Vec<u8>>,
    pub kind: RegionKind,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum RegionKind {
    Text(Vec<Paragraph>),
    Math(MathRegion),
    Plot(PlotData),
    Picture(Picture),
    /// An embedded object of another application (an OLE object).
    Ole,
    /// A scripted or data component (data table, spreadsheet, ...).
    Component,
    /// A reference to another worksheet.
    Reference,
    /// Layout only, carries no content.
    PageBreak,
    /// Anything else, with the element name the reader saw.
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Paragraph {
    /// The paragraph style name as the file spells it ("Heading 1", "Normal", ...).
    pub style: String,
    pub pieces: Vec<Piece>,
    /// `left`, `right`, `center` or `justify` when the paragraph says so.
    pub align: Option<String>,
    /// Left margin and first-line indent in points (0 when none).
    pub margin_left: f64,
    pub text_indent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    Run(Run),
    Break,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub sub: bool,
    pub sup: bool,
    /// Font family name, size in points and colour (`#rrggbb`), when the run states them.
    pub font: Option<String>,
    pub size: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MathRegion {
    pub stmt: Stmt,
    /// The region is switched off in the source (it is not evaluated there either).
    pub disabled: bool,
    /// The last calculated result the file stored for the region, when it is a single value.
    pub saved_result: Option<SavedResult>,
    /// The stored result when it is a complete value: a number, or a matrix of numbers, with its
    /// unit as a product of unit names (as an expression the translator renders).
    pub saved_expr: Option<Expr>,
    /// The value is data the reader took from an input component's saved workbook: a definition
    /// that is live from here on, but frozen at what the file held.
    pub frozen: bool,
    /// The decimal places the file shows this region's result with (its own result format).
    pub precision: Option<i32>,
}

/// A stored scalar result: the number (or string) as written, and its unit in the source's spelling.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedResult {
    pub value: String,
    pub unit: Option<String>,
}

/// What a math region does.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `target := value` (or `target == value` for a global definition).
    Define { target: Target, value: Expr, global: bool },
    /// `expr =`, with an optional unit the result is shown in.
    Eval { expr: Expr, unit: Option<Expr> },
    /// An expression with no definition or evaluation operator.
    Bare(Expr),
    /// A construct the reader recognises by name but does not read further.
    Unsupported { code: Code, detail: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Var(Name),
    Func(Name, Vec<Name>),
    /// `v[i] := ...`: the array's name when the target is a plain indexed name, and the indexes.
    Element {
        base: Option<Name>,
        index: Vec<Expr>,
    },
    /// `[a, b] := ...`: several names defined at once by a matrix of names.
    Multi(Vec<Name>),
}

/// An identifier with its literal subscript (`x_i` is base `x`, sub `i`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub base: String,
    pub sub: Option<String>,
    /// The file labels the identifier as a unit (so `N` is the newton even when a variable `N`
    /// exists).
    pub unit: bool,
    /// The file labels the identifier as a variable: it is never read as a unit label.
    pub var: bool,
}

impl Name {
    pub fn new(base: &str) -> Name {
        Name {
            base: base.to_string(),
            sub: None,
            unit: false,
            var: false,
        }
    }
    /// The Calcpad spelling: `base_sub`.
    pub fn flat(&self) -> String {
        match &self.sub {
            Some(s) if !s.is_empty() => format!("{}_{}", self.base, s),
            _ => self.base.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Abs,
    Sqrt,
    Factorial,
    Transpose,
    /// The elementwise operator over an expression (`vectorize`).
    Vectorize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal, kept as the file spells it.
    Num(String),
    Str(String),
    Name(Name),
    Un(UnOp, Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// A function call.
    Call(Name, Vec<Expr>),
    /// A unit written after its value (`30 degF`).
    Postfix(Name, Box<Expr>),
    /// `base[i, j]` (array indexing, not a literal subscript).
    Index(Box<Expr>, Vec<Expr>),
    /// Column-major, as the file stores it.
    Matrix {
        rows: usize,
        cols: usize,
        items: Vec<Expr>,
    },
    /// `first, second .. last`.
    Range {
        first: Box<Expr>,
        second: Option<Box<Expr>>,
        last: Box<Expr>,
    },
    /// `integral of body d var from lower to upper`.
    Integral {
        var: Name,
        body: Box<Expr>,
        lower: Box<Expr>,
        upper: Box<Expr>,
    },
    /// `sum (or product) of body over var`, from `bounds` or, without bounds, over the range
    /// variable `var` that an earlier region defined.
    Series {
        product: bool,
        var: Name,
        body: Box<Expr>,
        bounds: Option<(Box<Expr>, Box<Expr>)>,
    },
    /// First derivative of `body` with respect to `var`, at the current value of `var`.
    Derivative {
        var: Name,
        body: Box<Expr>,
    },
    /// A multi-statement program (local definitions, loops, conditions).
    Program(Vec<PStmt>),
    /// A construct the reader recognises by name but does not read further.
    Unsupported {
        code: Code,
        detail: Option<String>,
    },
}

/// What a program statement assigns to.
#[derive(Debug, Clone, PartialEq)]
pub enum PTarget {
    Var(Name),
    /// `v[i, j] <- ...`
    Element {
        base: Name,
        index: Vec<Expr>,
    },
}

/// One statement of a program.
#[derive(Debug, Clone, PartialEq)]
pub enum PStmt {
    /// `target <- value`
    Local(PTarget, Expr),
    /// `for var in range` (the range is kept as the expression the file has).
    For {
        var: Name,
        range: Expr,
        body: Vec<PStmt>,
    },
    While {
        cond: Expr,
        body: Vec<PStmt>,
    },
    /// `if` with its `elseif` branches in order, and an optional `otherwise` branch.
    If {
        branches: Vec<(Expr, Vec<PStmt>)>,
        otherwise: Option<Vec<PStmt>>,
    },
    Return(Expr),
    Break,
    Continue,
    /// A bare expression: the value of the program when it comes last.
    Expr(Expr),
    /// A statement the reader recognises but does not read (the element name is the detail).
    Unsupported(String),
}

// ---------------------------------------------------------------------------------------------
// Walking the tree

impl Expr {
    /// Calls `f` on every identifier that is read or bound in this expression (not on the name of
    /// a function being called, nor on the unit of a postfix application).
    pub fn for_each_name_mut(&mut self, f: &mut dyn FnMut(&mut Name)) {
        match self {
            Expr::Name(n) => f(n),
            Expr::Num(_) | Expr::Str(_) | Expr::Unsupported { .. } => {}
            Expr::Un(_, a) | Expr::Postfix(_, a) => a.for_each_name_mut(f),
            Expr::Bin(_, a, b) => {
                a.for_each_name_mut(f);
                b.for_each_name_mut(f);
            }
            Expr::Call(_, args) => args.iter_mut().for_each(|a| a.for_each_name_mut(f)),
            Expr::Index(b, idx) => {
                b.for_each_name_mut(f);
                idx.iter_mut().for_each(|a| a.for_each_name_mut(f));
            }
            Expr::Matrix { items, .. } => items.iter_mut().for_each(|a| a.for_each_name_mut(f)),
            Expr::Range { first, second, last } => {
                first.for_each_name_mut(f);
                if let Some(s) = second {
                    s.for_each_name_mut(f);
                }
                last.for_each_name_mut(f);
            }
            Expr::Integral {
                var,
                body,
                lower,
                upper,
            } => {
                f(var);
                body.for_each_name_mut(f);
                lower.for_each_name_mut(f);
                upper.for_each_name_mut(f);
            }
            Expr::Series { var, body, bounds, .. } => {
                f(var);
                body.for_each_name_mut(f);
                if let Some((lo, hi)) = bounds {
                    lo.for_each_name_mut(f);
                    hi.for_each_name_mut(f);
                }
            }
            Expr::Derivative { var, body } => {
                f(var);
                body.for_each_name_mut(f);
            }
            Expr::Program(stmts) => stmts.iter_mut().for_each(|s| s.for_each_name_mut(f)),
        }
    }
}

impl PStmt {
    /// Calls `f` on every identifier read, assigned or bound in this statement.
    pub fn for_each_name_mut(&mut self, f: &mut dyn FnMut(&mut Name)) {
        match self {
            PStmt::Local(t, e) => {
                match t {
                    PTarget::Var(n) => f(n),
                    PTarget::Element { base, index } => {
                        f(base);
                        index.iter_mut().for_each(|a| a.for_each_name_mut(f));
                    }
                }
                e.for_each_name_mut(f);
            }
            PStmt::For { var, range, body } => {
                f(var);
                range.for_each_name_mut(f);
                body.iter_mut().for_each(|s| s.for_each_name_mut(f));
            }
            PStmt::While { cond, body } => {
                cond.for_each_name_mut(f);
                body.iter_mut().for_each(|s| s.for_each_name_mut(f));
            }
            PStmt::If { branches, otherwise } => {
                for (c, b) in branches.iter_mut() {
                    c.for_each_name_mut(f);
                    b.iter_mut().for_each(|s| s.for_each_name_mut(f));
                }
                if let Some(o) = otherwise {
                    o.iter_mut().for_each(|s| s.for_each_name_mut(f));
                }
            }
            PStmt::Return(e) | PStmt::Expr(e) => e.for_each_name_mut(f),
            PStmt::Break | PStmt::Continue | PStmt::Unsupported(_) => {}
        }
    }
}

impl Stmt {
    /// Calls `f` on every identifier read, assigned or bound in this statement.
    pub fn for_each_name_mut(&mut self, f: &mut dyn FnMut(&mut Name)) {
        match self {
            Stmt::Define { target, value, .. } => {
                match target {
                    Target::Var(n) => f(n),
                    Target::Func(_, params) => {
                        for p in params.iter_mut() {
                            f(p);
                        }
                    }
                    Target::Element { base, index } => {
                        if let Some(b) = base {
                            f(b);
                        }
                        index.iter_mut().for_each(|a| a.for_each_name_mut(f));
                    }
                    Target::Multi(names) => {
                        for n in names.iter_mut() {
                            f(n);
                        }
                    }
                }
                value.for_each_name_mut(f);
            }
            Stmt::Eval { expr, unit } => {
                expr.for_each_name_mut(f);
                if let Some(u) = unit {
                    u.for_each_name_mut(f);
                }
            }
            Stmt::Bare(e) => e.for_each_name_mut(f),
            Stmt::Unsupported { .. } => {}
        }
    }
}
