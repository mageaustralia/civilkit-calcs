//! What the importer tells the caller: counts per region kind and every construct that did not
//! convert, each with a stable code, a short reason and a region and line reference.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A stable identifier for one kind of issue. The string form (`as_str`) never changes once
/// released; the UI and the batch summary key on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    // ---- gaps: the region is replaced by a placeholder line ----
    RangeVariable,
    ProgramBlock,
    SolveBlock,
    SymbolicEvaluation,
    CalculusOperator,
    SummationOperator,
    VectorizeOperator,
    MatrixOperator,
    StringValue,
    BareExpression,
    ConstraintEquation,
    ElementAssignment,
    DisabledRegion,
    Plot,
    Image,
    OleObject,
    Component,
    WorksheetReference,
    OtherRegion,
    HeaderFooter,
    RichTextElement,
    TextUnreadable,
    UnknownElement,
    UnmappedUnit,
    BuiltinMissing,
    UndefinedVariable,
    AddinFunction,
    ReferenceFunction,
    DependsOnGap,
    InvalidName,
    BadNumber,
    ArityMismatch,
    BadStructure,
    TooDeep,
    // ---- notes: the region converted, with a caveat ----
    GlobalDefinition,
    RangeLoop,
    FrozenValue,
    AngleMode,
    ArrayOrigin,
    OriginAssigned,
    LogBase,
    RoundDigits,
    AbsOfArray,
    CallAsMultiplication,
    UnitOverride,
    UncoveredPath,
    ControlInput,
    SiUnitScale,
    StaticPicture,
    SvgFallback,
    PlotStyle,
    // ---- whole-file problems ----
    XmlError,
    NotAWorksheet,
    InputTooLarge,
    UnsupportedContainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The region did not convert.
    Gap,
    /// The region converted, with something the reader of the report should know.
    Note,
    /// The file as a whole could not be read.
    Fatal,
}

impl Code {
    /// Every code, in declaration order.
    pub const ALL: &'static [Code] = &[
        Code::RangeVariable,
        Code::ProgramBlock,
        Code::SolveBlock,
        Code::SymbolicEvaluation,
        Code::CalculusOperator,
        Code::SummationOperator,
        Code::VectorizeOperator,
        Code::MatrixOperator,
        Code::StringValue,
        Code::BareExpression,
        Code::ConstraintEquation,
        Code::ElementAssignment,
        Code::DisabledRegion,
        Code::Plot,
        Code::Image,
        Code::OleObject,
        Code::Component,
        Code::WorksheetReference,
        Code::OtherRegion,
        Code::HeaderFooter,
        Code::RichTextElement,
        Code::TextUnreadable,
        Code::UnknownElement,
        Code::UnmappedUnit,
        Code::BuiltinMissing,
        Code::UndefinedVariable,
        Code::AddinFunction,
        Code::ReferenceFunction,
        Code::DependsOnGap,
        Code::InvalidName,
        Code::BadNumber,
        Code::ArityMismatch,
        Code::BadStructure,
        Code::TooDeep,
        Code::GlobalDefinition,
        Code::RangeLoop,
        Code::FrozenValue,
        Code::AngleMode,
        Code::ArrayOrigin,
        Code::OriginAssigned,
        Code::LogBase,
        Code::RoundDigits,
        Code::AbsOfArray,
        Code::CallAsMultiplication,
        Code::UnitOverride,
        Code::UncoveredPath,
        Code::ControlInput,
        Code::SiUnitScale,
        Code::StaticPicture,
        Code::SvgFallback,
        Code::PlotStyle,
        Code::XmlError,
        Code::NotAWorksheet,
        Code::InputTooLarge,
        Code::UnsupportedContainer,
    ];

    pub fn as_str(self) -> &'static str {
        use Code::*;
        match self {
            RangeVariable => "range-variable",
            ProgramBlock => "program-block",
            SolveBlock => "solve-block",
            SymbolicEvaluation => "symbolic-evaluation",
            CalculusOperator => "calculus-operator",
            SummationOperator => "summation-operator",
            VectorizeOperator => "vectorize-operator",
            MatrixOperator => "matrix-operator",
            StringValue => "string-value",
            BareExpression => "bare-expression",
            ConstraintEquation => "constraint-equation",
            ElementAssignment => "element-assignment",
            DisabledRegion => "disabled-region",
            Plot => "plot",
            Image => "image",
            OleObject => "ole-object",
            Component => "component",
            WorksheetReference => "worksheet-reference",
            OtherRegion => "other-region",
            HeaderFooter => "header-footer",
            RichTextElement => "rich-text-element",
            TextUnreadable => "text-unreadable",
            UnknownElement => "unknown-element",
            UnmappedUnit => "unmapped-unit",
            BuiltinMissing => "builtin-missing",
            UndefinedVariable => "undefined-variable",
            AddinFunction => "addin-function",
            ReferenceFunction => "reference-function",
            DependsOnGap => "depends-on-gap",
            InvalidName => "invalid-name",
            BadNumber => "bad-number",
            ArityMismatch => "arity-mismatch",
            BadStructure => "bad-structure",
            TooDeep => "too-deep",
            GlobalDefinition => "global-definition",
            RangeLoop => "range-loop",
            FrozenValue => "frozen-value",
            AngleMode => "angle-mode",
            ArrayOrigin => "array-origin",
            OriginAssigned => "origin-assigned",
            LogBase => "log-base",
            RoundDigits => "round-digits",
            AbsOfArray => "abs-of-array",
            CallAsMultiplication => "call-as-multiplication",
            UnitOverride => "unit-override",
            UncoveredPath => "uncovered-path",
            ControlInput => "control-input",
            SiUnitScale => "si-unit-scale",
            StaticPicture => "static-picture",
            SvgFallback => "svg-fallback",
            PlotStyle => "plot-style",
            XmlError => "xml-error",
            NotAWorksheet => "not-a-worksheet",
            InputTooLarge => "input-too-large",
            UnsupportedContainer => "unsupported-container",
        }
    }

    pub fn severity(self) -> Severity {
        use Code::*;
        match self {
            GlobalDefinition | RangeLoop | FrozenValue | AngleMode | ArrayOrigin | OriginAssigned | LogBase
            | RoundDigits | AbsOfArray | CallAsMultiplication | UnitOverride | UncoveredPath | ControlInput
            | SiUnitScale | StaticPicture | SvgFallback | PlotStyle => Severity::Note,
            XmlError | NotAWorksheet | InputTooLarge | UnsupportedContainer => Severity::Fatal,
            _ => Severity::Gap,
        }
    }

    /// A short, fixed reason that the UI can show next to the region reference.
    pub fn reason(self) -> &'static str {
        use Code::*;
        match self {
            RangeVariable => {
                "range variable (a..b) or an expression iterating over one; the engine has no equivalent yet"
            }
            ProgramBlock => "multi-line program (local definitions, loops, conditions)",
            SolveBlock => "solve block (Given/Find and relatives)",
            SymbolicEvaluation => "symbolic evaluation",
            CalculusOperator => "derivative or integral operator",
            SummationOperator => "sum, product or limit operator",
            VectorizeOperator => "vectorize operator",
            MatrixOperator => "matrix column/row or vector sum operator",
            StringValue => "text value the engine cannot hold (texts are held as codes only to compare and to choose between; no joining, slicing or file names)",
            ConstraintEquation => "equation written as a constraint (an unknown to solve for, or a check): shown, not solved",
            BareExpression => "expression without a definition or evaluation operator (documentation or constraint)",
            ElementAssignment => "assignment to a matrix element",
            DisabledRegion => "the region is disabled in the source worksheet",
            Plot => "plot",
            Image => "picture",
            OleObject => "embedded object of another application",
            Component => "scripted or data component",
            WorksheetReference => "reference to another worksheet",
            OtherRegion => "region kind the importer does not know",
            HeaderFooter => "page header or footer (not part of the calculation; not converted)",
            RichTextElement => "element inside a text region that is not converted (table, picture or floating block)",
            TextUnreadable => "the text of this region could not be read from the file",
            UnknownElement => "element the importer does not know",
            UnmappedUnit => "unit with no Calcpad equivalent in the mapping table",
            BuiltinMissing => "built-in function of the source tool that the engine lacks (no Calcpad equivalent in the table)",
            UndefinedVariable => "name used before it is defined (or defined by an add-in or an included worksheet)",
            AddinFunction => "function that is not defined in the sheet and is not a built-in: most likely supplied by a native add-in",
            ReferenceFunction => "function that is not defined in this sheet; the sheet references other worksheets that may define it",
            DependsOnGap => "uses a name that an earlier unconverted region defines",
            InvalidName => "name that Calcpad cannot spell",
            BadNumber => "number literal the importer cannot read",
            ArityMismatch => "function called with the wrong number of arguments",
            BadStructure => "expression tree with an unexpected shape",
            TooDeep => "expression nested too deeply",
            FrozenValue => "value taken from the saved result of an input component the importer cannot run: it is frozen and does not follow a change of the input",
            RangeLoop => "range variable read as the loop bounds of the element assignments that use it; any other use is reported",
            GlobalDefinition => "global definition converted as an ordinary definition at its reading-order position",
            AngleMode => "trigonometric functions: the output switches the sheet to radians (#rad), the source default",
            ArrayOrigin => "array origin differs from 1: indexes were shifted",
            OriginAssigned => "ORIGIN was assigned; later indexes use the new value",
            LogBase => "log with a base converted as ln(x)/ln(base)",
            RoundDigits => "round with a digit count converted by scaling",
            AbsOfArray => "|x| is abs(x): for a vector the source may mean its magnitude, for a matrix its determinant",
            CallAsMultiplication => {
                "name applied like a function but defined as a variable: converted as multiplication"
            }
            UnitOverride => "result shown in a unit: converted to a `|unit` conversion",
            UncoveredPath => "a program path that no condition covers (the source stops there): that path is undefined, never another branch's value",
            ControlInput => "input control: its stored value is the input; the options and the script behind it are not carried",
            SiUnitScale => "the SI unit of a quantity is read as the unit it is held in: the same number only when that unit is the SI one or the scale cancels in the result",
            StaticPicture => "shown as a static picture of the original (not live, not editable)",
            SvgFallback => "metafile could not be converted to SVG (failed or too large): the raster picture is used",
            PlotStyle => "plot converted: the engine draws its own colours, markers, line styles and axis ranges, and the traces keep only their expressions",
            XmlError => "the input is not well-formed XML (or uses a DTD, which is refused)",
            NotAWorksheet => "the XML is not a worksheet document",
            InputTooLarge => "the input exceeds the size limit",
            UnsupportedContainer => "the file is a zip package that cannot be read (damaged, encrypted or using an unsupported zip feature)",
        }
    }
}

/// One thing the report wants the caller to see.
#[derive(Debug, Clone, PartialEq)]
pub struct Issue {
    pub code: Code,
    pub severity: Severity,
    /// The specific name or element behind the code (a unit name, a function name, an element tag).
    pub detail: Option<String>,
    /// The `region-id` of the region it was found in.
    pub region_id: Option<String>,
    /// 1-based line in the source file.
    pub line: Option<u32>,
    pub reason: String,
    /// For `DependsOnGap`: the code of the unconverted region this one ultimately depends on.
    pub root: Option<Code>,
}

/// Converted / needing attention for one kind of region.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KindStats {
    pub total: usize,
    pub converted: usize,
    /// Converted but with at least one note.
    pub with_notes: usize,
    pub attention: usize,
    /// Of the regions needing attention, how many still show something: a static picture of the original.
    pub pictured: usize,
}

/// One row of the ranked list of missing features.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedGap {
    pub code: Code,
    /// Regions this code blocked directly.
    pub direct: usize,
    /// Further regions blocked only because they use a name that a `direct` region defines.
    pub downstream: usize,
    /// The most frequent details (unit, function or element names) with their counts.
    pub top_details: Vec<(String, usize)>,
}

impl RankedGap {
    pub fn blocked(&self) -> usize {
        self.direct + self.downstream
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportReport {
    /// The array origin the sheet declared.
    pub origin: i64,
    /// Count per top-level region kind: "text", "math", "plot", "image", "ole", "component",
    /// "reference", "other". Page breaks are layout and are not counted anywhere.
    pub kinds: BTreeMap<String, KindStats>,
    /// Math regions by what they do: "definition", "function", "evaluation", "expression", "other".
    pub math_shapes: BTreeMap<String, KindStats>,
    pub issues: Vec<Issue>,
    pub page_breaks: usize,
    /// Definitions whose value is the saved result of an input component (frozen, not live).
    pub frozen_values: usize,
    pub pictures: PictureStats,
}

/// What happened to pictures and picture fallbacks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PictureStats {
    /// Picture regions whose metafile became an SVG.
    pub svg_ok: usize,
    /// Picture regions with a metafile that could not be used (conversion failed or too large).
    pub svg_failed: usize,
    /// Picture regions shown from a raster picture.
    pub png_used: usize,
    /// Regions that did not convert but are shown as a static picture of the original.
    pub fallback_shown: usize,
    /// Total bytes of SVG and of raster assets produced.
    pub svg_bytes: usize,
    pub png_bytes: usize,
    /// Unconverted math regions that carry a saved scalar result.
    pub saved_result_available: usize,
    /// Of those, how many were written out as a labelled static line (option on, no picture used).
    pub saved_result_emitted: usize,
}

impl ImportReport {
    /// Regions that carry content (everything except page breaks).
    pub fn regions_total(&self) -> usize {
        self.kinds.values().map(|k| k.total).sum()
    }
    pub fn regions_converted(&self) -> usize {
        self.kinds.values().map(|k| k.converted).sum()
    }
    /// How many regions need the user's attention: the "5 of 140" of the UI.
    pub fn regions_needing_attention(&self) -> usize {
        self.kinds.values().map(|k| k.attention).sum()
    }
    /// Regions that show something: converted ones plus static pictures of unconverted ones.
    pub fn regions_showing(&self) -> usize {
        self.regions_converted() + self.kinds.values().map(|k| k.pictured).sum::<usize>()
    }
    /// Share of content regions that converted, 0.0 to 1.0 (1.0 for an empty sheet).
    pub fn convertible_ratio(&self) -> f64 {
        let t = self.regions_total();
        if t == 0 {
            1.0
        } else {
            self.regions_converted() as f64 / t as f64
        }
    }
    pub fn gaps(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Gap)
    }
    pub fn notes(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Note)
    }
    pub fn is_fatal(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Fatal)
    }

    /// The missing features ranked by how many regions they block (directly plus downstream).
    /// A region is counted once per code however many times the construct occurs in it.
    pub fn ranked_gaps(&self) -> Vec<RankedGap> {
        let mut rows: BTreeMap<Code, (usize, usize, BTreeMap<String, usize>)> = BTreeMap::new();
        let mut seen_direct: std::collections::HashSet<(Option<String>, Option<u32>, Code)> = Default::default();
        let mut seen_down: std::collections::HashSet<(Option<String>, Option<u32>, Code)> = Default::default();
        for i in self.gaps() {
            let key = (i.region_id.clone(), i.line, i.code);
            if i.code == Code::DependsOnGap {
                if let Some(root) = i.root {
                    if seen_down.insert((i.region_id.clone(), i.line, root)) {
                        rows.entry(root).or_default().1 += 1;
                    }
                }
                continue;
            }
            let row = rows.entry(i.code).or_default();
            if seen_direct.insert(key) {
                row.0 += 1;
            }
            if let Some(d) = &i.detail {
                *row.2.entry(d.clone()).or_default() += 1;
            }
        }
        let mut out: Vec<RankedGap> = rows
            .into_iter()
            .map(|(code, (direct, downstream, details))| {
                let mut top: Vec<(String, usize)> = details.into_iter().collect();
                top.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                top.truncate(5);
                RankedGap {
                    code,
                    direct,
                    downstream,
                    top_details: top,
                }
            })
            .collect();
        out.sort_by(|a, b| {
            b.blocked()
                .cmp(&a.blocked())
                .then_with(|| b.direct.cmp(&a.direct))
                .then_with(|| a.code.as_str().cmp(b.code.as_str()))
        });
        out
    }

    /// A plain-text rendering for the command line.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "{} of {} regions need attention ({:.1} % converted); array origin {}",
            self.regions_needing_attention(),
            self.regions_total(),
            self.convertible_ratio() * 100.0,
            self.origin
        );
        for (k, v) in &self.kinds {
            let _ = writeln!(
                s,
                "  {:<10} total {:>4}  converted {:>4}  (with notes {:>3})  attention {:>4}",
                k, v.total, v.converted, v.with_notes, v.attention
            );
        }
        for (k, v) in &self.math_shapes {
            let _ = writeln!(
                s,
                "    math/{:<11} total {:>4}  converted {:>4}  attention {:>4}",
                k, v.total, v.converted, v.attention
            );
        }
        let ranked = self.ranked_gaps();
        if !ranked.is_empty() {
            let _ = writeln!(s, "Missing features, by regions blocked:");
            for r in &ranked {
                let _ = write!(
                    s,
                    "  {:<22} {:>4} (direct {}, downstream {})",
                    r.code.as_str(),
                    r.blocked(),
                    r.direct,
                    r.downstream
                );
                if !r.top_details.is_empty() {
                    let d: Vec<String> = r.top_details.iter().map(|(n, c)| format!("{} x{}", n, c)).collect();
                    let _ = write!(s, "  [{}]", d.join(", "));
                }
                s.push('\n');
            }
        }
        let _ = writeln!(s, "Issues:");
        for i in &self.issues {
            let sev = match i.severity {
                Severity::Gap => "gap ",
                Severity::Note => "note",
                Severity::Fatal => "FATAL",
            };
            let _ = writeln!(
                s,
                "  {} {:<20} region {:<6} line {:<6} {}{}",
                sev,
                i.code.as_str(),
                i.region_id.as_deref().unwrap_or("-"),
                i.line.map(|l| l.to_string()).unwrap_or_else(|| "-".into()),
                i.reason,
                i.detail.as_ref().map(|d| format!(" [{}]", d)).unwrap_or_default()
            );
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_strings_are_unique_kebab_case_and_have_reasons() {
        let mut seen = std::collections::HashSet::new();
        for c in Code::ALL {
            let s = c.as_str();
            assert!(seen.insert(s), "duplicate code {s}");
            assert!(s.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-'), "{s}");
            assert!(!c.reason().is_empty());
        }
        assert_eq!(seen.len(), Code::ALL.len());
    }
}
