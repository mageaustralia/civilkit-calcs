//! The mapping tables, all in one place: units, built-in functions, constants and the names
//! that mark constructs the importer does not translate.

/// Normalises look-alike characters in a name: the micro sign (U+00B5) becomes Greek mu (U+03BC).
pub fn normalise_name(s: &str) -> String {
    // The engine spells a prime with the prime characters, not the apostrophe.
    s.replace('\u{00B5}', "\u{03BC}")
        .replace("''", "\u{2033}")
        .replace('\'', "\u{2032}")
}

/// Units whose source spelling is also the Calcpad spelling.
pub const SAME_UNITS: &[&str] = &[
    // length
    "m", "km", "dm", "cm", "mm", "μm", "nm", "pm", "in", "ft", "yd", "mi", "nmi", "th", "ly", "AU", // mass
    "g", "kg", "mg", "μg", "lb", "oz", "slug", "gr", "t", "kt", // time and frequency
    "s", "ms", "μs", "ns", "min", "h", "d", "w", "y", "Hz", "kHz", "MHz", "GHz", "rpm", // electrical
    "A", "mA", "μA", "kA", "V", "mV", "kV", "Ω", "kΩ", "F", "μF", "H", "mH", "C", "S", "T", "Wb",
    // thermodynamic temperature, amount, light
    "K", "°C", "°F", "°R", "mol", "cd", "lm", "lx", // force
    "N", "kN", "MN", "GN", "daN", "kNm", "Nm", "lbf", "kgf", "kip", "dyn", // pressure
    "Pa", "kPa", "MPa", "GPa", "hPa", "μPa", "bar", "mbar", "atm", "psi", "ksi", "psf", "ksf", "mmHg", "inHg", "Torr",
    // energy and power
    "J", "kJ", "MJ", "GJ", "mJ", "cal", "kcal", "BTU", "erg", "eV", "Wh", "kWh", "MWh", "W", "kW", "MW", "GW", "mW",
    "hp", // volume and area
    "L", "mL", "μL", "ha", // angle
    "grad", "rev", "%", // speed
    "mph", "knot",
];

/// Units whose source spelling differs from the Calcpad spelling: (source, Calcpad).
pub const RENAMED_UNITS: &[(&str, &str)] = &[
    ("sec", "s"),
    ("hr", "h"),
    ("hour", "h"),
    ("day", "d"),
    ("yr", "y"),
    ("gm", "g"),
    ("tonne", "t"),
    ("lbm", "lb"),
    ("liter", "L"),
    ("litre", "L"),
    ("ml", "mL"),
    ("gal", "gal_US"),
    ("qt", "qt_US"),
    ("pt", "pt_US"),
    ("mile", "mi"),
    ("torr", "Torr"),
    ("dyne", "dyn"),
    ("ohm", "Ω"),
    ("tonf", "tonf_US"),
    ("ton", "ton_US"),
    ("mil", "th"),
    ("acre", "ac"),
    ("nmile", "nmi"),
    ("degC", "°C"),
    ("degF", "°F"),
    ("degR", "°R"),
    ("International_BTU", "BTU"),
    ("pound", "lb"),
    ("foot", "ft"),
    ("inch", "in"),
    ("pounds_per_sq_inch", "psi"),
    ("meter", "m"),
    ("metre", "m"),
    ("gram", "g"),
    ("kilogram", "kg"),
    ("second", "s"),
    ("minute", "min"),
    ("newton", "N"),
    ("pascal", "Pa"),
    ("joule", "J"),
    ("watt", "W"),
    ("volt", "V"),
    ("ampere", "A"),
    ("kelvin", "K"),
    ("hertz", "Hz"),
    ("coulomb", "C"),
    ("farad", "F"),
    ("henry", "H"),
    ("tesla", "T"),
    ("weber", "Wb"),
    ("siemens", "S"),
    // An angle is a plain number in the source (a degree is pi/180, a radian is 1), so it adds to,
    // compares with and multiplies unitless numbers; the engine's angle units would not.
    ("radian", "1"),
    ("rad", "1"),
    ("deg", "(π/180)"),
    ("degree", "(π/180)"),
    ("°", "(π/180)"),
    ("percent", "%"),
    // Force per length and per volume, as the structural handbooks write them.
    ("plf", "(lbf/ft)"),
    ("pli", "(lbf/in)"),
    ("klf", "(kip/ft)"),
    ("pcf", "(lbf/ft^3)"),
    ("kcf", "(kip/ft^3)"),
    ("kips", "kip"),
];

/// Units the source language knows and Calcpad does not: reported as `unmapped-unit`.
pub const UNMAPPED_UNITS: &[&str] = &[
    "kmol", "mmol", "lbmol", "gmol", "kgmol", "angstrom", "gal_imp", "dB", "Bq", "Gy", "Sv",
];

/// The Calcpad spelling of a unit name, if the table has one.
///
/// A product written with a middle dot (`kN·m`, `N·m`) is a unit when every factor is; it is
/// spelled `(kN*m)` so that it stays one operand next to `/` and `^`.
pub fn map_unit(name: &str) -> Option<String> {
    if name.contains(['\u{00B7}', '\u{22C5}']) {
        let parts: Option<Vec<&'static str>> = name.split(['\u{00B7}', '\u{22C5}']).map(map_single_unit).collect();
        let parts = parts?;
        return (parts.len() > 1).then(|| format!("({})", parts.join("*")));
    }
    map_single_unit(name).map(str::to_string)
}

fn map_single_unit(name: &str) -> Option<&'static str> {
    let n = normalise_name(name);
    if let Some(u) = SAME_UNITS.iter().find(|u| **u == n) {
        return Some(u);
    }
    RENAMED_UNITS.iter().find(|(s, _)| *s == n).map(|(_, c)| *c)
}

pub fn is_unmapped_unit(name: &str) -> bool {
    UNMAPPED_UNITS.contains(&normalise_name(name).as_str())
}

/// What to do with a call to a built-in function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnRule {
    /// Same arguments, new name.
    Direct(&'static str),
    /// Direct, but the function depends on the angle mode.
    Trig(&'static str),
    /// `log(x)` or `log(x, base)`.
    Log,
    /// `round(x)` or `round(x, digits)`.
    Round,
    /// `Ceil(x, y)`: the smallest multiple of `y` that is not below `x`, with the named engine
    /// function (`ceiling`, `floor`) of `x / y` times `y`.
    Multiple(&'static str),
    /// `submatrix(A, ir, jr, ic, jc)`: the four indexes are shifted from the sheet's array origin.
    Submatrix,
}

/// (source name, rule, minimum arguments, maximum arguments)
pub const FUNCTIONS: &[(&str, FnRule, usize, usize)] = &[
    ("sin", FnRule::Trig("sin"), 1, 1),
    ("cos", FnRule::Trig("cos"), 1, 1),
    ("tan", FnRule::Trig("tan"), 1, 1),
    ("sec", FnRule::Trig("sec"), 1, 1),
    ("csc", FnRule::Trig("csc"), 1, 1),
    ("cot", FnRule::Trig("cot"), 1, 1),
    ("asin", FnRule::Trig("asin"), 1, 1),
    ("acos", FnRule::Trig("acos"), 1, 1),
    ("atan", FnRule::Trig("atan"), 1, 1),
    ("atan2", FnRule::Trig("atan2"), 2, 2),
    ("acot", FnRule::Trig("acot"), 1, 1),
    ("asec", FnRule::Trig("asec"), 1, 1),
    ("acsc", FnRule::Trig("acsc"), 1, 1),
    ("sinh", FnRule::Direct("sinh"), 1, 1),
    ("cosh", FnRule::Direct("cosh"), 1, 1),
    ("tanh", FnRule::Direct("tanh"), 1, 1),
    ("sech", FnRule::Direct("sech"), 1, 1),
    ("csch", FnRule::Direct("csch"), 1, 1),
    ("coth", FnRule::Direct("coth"), 1, 1),
    ("asinh", FnRule::Direct("asinh"), 1, 1),
    ("acosh", FnRule::Direct("acosh"), 1, 1),
    ("atanh", FnRule::Direct("atanh"), 1, 1),
    ("ln", FnRule::Direct("ln"), 1, 1),
    ("log", FnRule::Log, 1, 2),
    ("exp", FnRule::Direct("exp"), 1, 1),
    ("sqrt", FnRule::Direct("sqrt"), 1, 1),
    ("abs", FnRule::Direct("abs"), 1, 1),
    ("sign", FnRule::Direct("sign"), 1, 1),
    ("signum", FnRule::Direct("sign"), 1, 1),
    ("floor", FnRule::Direct("floor"), 1, 1),
    ("ceil", FnRule::Direct("ceiling"), 1, 1),
    ("trunc", FnRule::Direct("trunc"), 1, 1),
    ("round", FnRule::Round, 1, 2),
    ("Ceil", FnRule::Multiple("ceiling"), 2, 2),
    ("Floor", FnRule::Multiple("floor"), 2, 2),
    ("mod", FnRule::Direct("mod"), 2, 2),
    ("min", FnRule::Direct("min"), 1, 64),
    ("max", FnRule::Direct("max"), 1, 64),
    ("gcd", FnRule::Direct("gcd"), 2, 64),
    ("lcm", FnRule::Direct("lcm"), 2, 64),
    ("Re", FnRule::Direct("re"), 1, 1),
    ("Im", FnRule::Direct("im"), 1, 1),
    ("arg", FnRule::Direct("phase"), 1, 1),
    ("if", FnRule::Direct("if"), 3, 3),
    ("rnd", FnRule::Direct("random"), 1, 1),
    ("rows", FnRule::Direct("n_rows"), 1, 1),
    ("cols", FnRule::Direct("n_cols"), 1, 1),
    ("length", FnRule::Direct("len"), 1, 1),
    ("vectorSum", FnRule::Direct("sum"), 1, 1),
    ("identity", FnRule::Direct("identity"), 1, 1),
    ("matrix", FnRule::Direct("matrix"), 2, 2),
    ("det", FnRule::Direct("det"), 1, 1),
    ("rank", FnRule::Direct("rank"), 1, 1),
    ("augment", FnRule::Direct("augment"), 2, 64),
    ("lsolve", FnRule::Direct("lsolve"), 2, 2),
    ("submatrix", FnRule::Submatrix, 5, 5),
    ("stack", FnRule::Direct("stack"), 2, 64),
    ("matcol", FnRule::Direct("col"), 2, 2),
    ("matrow", FnRule::Direct("row"), 2, 2),
    ("sort", FnRule::Direct("sort"), 1, 1),
    // `csort(A, j)` sorts the rows by column j, `rsort(A, i)` the columns by row i; the index counts
    // from the sheet's array origin.
    ("csort", FnRule::Direct("sort_rows"), 2, 2),
    ("rsort", FnRule::Direct("sort_cols"), 2, 2),
    ("reverse", FnRule::Direct("reverse"), 1, 1),
    ("mean", FnRule::Direct("mean"), 1, 64),
    ("norm2", FnRule::Direct("norm_2"), 1, 1),
];

pub fn lookup_function(name: &str) -> Option<(FnRule, usize, usize)> {
    FUNCTIONS
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, r, lo, hi)| (*r, *lo, *hi))
}

/// Built-in functions of the source tool that the engine lacks (no Calcpad equivalent in the table).
/// A call to one of these is `builtin-missing`. Together with [`FUNCTIONS`] this is the table of the
/// built-in names the importer knows; a call to any other undefined name is a function supplied
/// from elsewhere (`addin-function` or `reference-function`).
pub const BUILTIN_MISSING: &[&str] = &[
    "acoth",
    "asech",
    "acsch",
    "angle",
    "Ci",
    "Si",
    "Ei",
    "erf",
    "erfc",
    "cnorm",
    "dnorm",
    "pnorm",
    "qnorm",
    "Gamma",
    "gamma",
    "Beta",
    "J0",
    "J1",
    "Jn",
    "Y0",
    "Y1",
    "Yn",
    "I0",
    "I1",
    "In",
    "K0",
    "K1",
    "Kn",
    "Ai",
    "Bi",
    "combin",
    "permut",
    "csgn",
    "sgn",
    "root",
    "polyroots",
    "until",
    "stdev",
    "Stdev",
    "var",
    "Var",
    "skew",
    "kurt",
    "corr",
    "cvar",
    "hist",
    "histogram",
    "median",
    "mode",
    "rref",
    "eigenvals",
    "eigenvec",
    "eigenvecs",
    "genvals",
    "genvecs",
    "lu",
    "qr",
    "svd",
    "svds",
    "cholesky",
    "geninv",
    "tr",
    "norm1",
    "norme",
    "normi",
    "cond1",
    "cond2",
    "conde",
    "condi",
    "cumsum",
    "cumint",
    "linsolve",
    "lookup",
    "hlookup",
    "vlookup",
    "match",
    "interp",
    "cspline",
    "lspline",
    "pspline",
    "bspline",
    "regress",
    "loess",
    "line",
    "slope",
    "intercept",
    "medfit",
    "linfit",
    "genfit",
    "expfit",
    "logfit",
    "lgsfit",
    "pwrfit",
    "sinfit",
    "polyfit",
    "fft",
    "ifft",
    "cfft",
    "icfft",
    "FFT",
    "IFFT",
    "CFFT",
    "ICFFT",
    "rkfixed",
    "Rkadapt",
    "Bulstoer",
    "bulstoer",
    "Odesolve",
    "sbval",
    "bvalfit",
    "relax",
    "multigrid",
    "numol",
    "stiffb",
    "stiffr",
    "Statespace",
    "READPRN",
    "WRITEPRN",
    "APPENDPRN",
    "READ",
    "WRITE",
    "READBMP",
    "READRGB",
    "READFILE",
    "WRITEBMP",
    "READEXCEL",
    "WRITEEXCEL",
    "strlen",
    "substr",
    "num2str",
    "str2num",
    "concat",
    "search",
    "str2vec",
    "vec2str",
    "error",
    "string",
    "dbinom",
    "pbinom",
    "qbinom",
    "rbinom",
    "dunif",
    "punif",
    "qunif",
    "runif",
    "dexp",
    "pexp",
    "qexp",
    "rexp",
    "dgamma",
    "pgamma",
    "qgamma",
    "rgamma",
    "dt",
    "pt",
    "qt",
    "rt",
    "dchisq",
    "pchisq",
    "qchisq",
    "rchisq",
    "dF",
    "pF",
    "qF",
    "rF",
    "dpois",
    "ppois",
    "qpois",
    "rpois",
    "dgeom",
    "pgeom",
    "qgeom",
    "rgeom",
    "dlnorm",
    "plnorm",
    "qlnorm",
    "rlnorm",
    "rnorm",
    "tic",
    "toc",
    "time",
    "IsArray",
    "IsScalar",
    "IsString",
    "iscomplex",
    "isreal",
    "tail",
    "mid",
    "last",
    "diag",
    "hypot",
    "clamp",
    "mfill",
    "Fill",
    "Heaviside",
    "Dirac",
    "Fresnelc",
    "Fresnels",
    "unitsof",
    "cond",
    "trace",
    "kronecker",
    "lsq",
];

pub fn is_builtin_missing(name: &str) -> bool {
    BUILTIN_MISSING.contains(&name)
}

/// Names that begin or are part of a solve block.
pub const SOLVE_NAMES: &[&str] = &[
    "Given", "Find", "Minerr", "Minimize", "Maximize", "Odesolve", "Pfind", "Pminerr",
];

pub fn is_solve_name(name: &str) -> bool {
    SOLVE_NAMES.contains(&name)
}

/// What a built-in constant becomes.
pub enum Constant {
    /// Emitted as given.
    Text(&'static str),
    /// The array origin setting at that point of the sheet.
    Origin,
}

/// Built-in constants that are not units: the name as the source spells it and its Calcpad text.
pub fn lookup_constant(name: &str) -> Option<Constant> {
    match name {
        "π" => Some(Constant::Text("π")),
        "e" => Some(Constant::Text("e")),
        // The source's built-in gravitational acceleration. Calcpad's `g` is the gram.
        "g" => Some(Constant::Text("(9.80665*m/s^2)")),
        // Not a number: the engine's undefined value (a square root of a negative number).
        "NaN" => Some(Constant::Text("sqrt(-1)")),
        "ORIGIN" => Some(Constant::Origin),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn micro_sign_is_normalised() {
        assert_eq!(map_unit("\u{00B5}Pa").as_deref(), Some("μPa"));
    }

    #[test]
    fn same_and_renamed_units_resolve() {
        assert_eq!(map_unit("kg").as_deref(), Some("kg"));
        assert_eq!(map_unit("hr").as_deref(), Some("h"));
        assert_eq!(map_unit("gm").as_deref(), Some("g"));
        assert_eq!(map_unit("nonsense"), None);
    }

    #[test]
    fn no_duplicate_source_names() {
        let mut seen = std::collections::HashSet::new();
        for u in SAME_UNITS {
            assert!(seen.insert(*u), "duplicate unit {u}");
        }
        for (s, _) in RENAMED_UNITS {
            assert!(seen.insert(*s), "duplicate unit {s}");
        }
        let mut f = std::collections::HashSet::new();
        for (n, ..) in FUNCTIONS {
            assert!(f.insert(*n), "duplicate function {n}");
            assert!(!is_builtin_missing(n), "{n} is both mapped and missing");
        }
    }
}

/// The engine's own function names, lowercase. The engine reads a function name without regard to
/// case, so a worksheet function called `Norm` or `Max` cannot be defined under that name; the
/// importer gives it another (`Norm_f`). A test checks this list against the engine's tables.
pub const ENGINE_FUNCTIONS: &[&str] = &[
    "abs",
    "acos",
    "acosh",
    "acot",
    "acoth",
    "acsc",
    "acsch",
    "add",
    "adj",
    "and",
    "asec",
    "asech",
    "asin",
    "asinh",
    "atan",
    "atan2",
    "atanh",
    "augment",
    "average",
    "cbrt",
    "ceigenvals",
    "ceigenvecs",
    "ceiling",
    "cholesky",
    "clrunits",
    "clsolve",
    "cmsolve",
    "cofactor",
    "col",
    "column",
    "column_hp",
    "cond",
    "cond_1",
    "cond_2",
    "cond_e",
    "cond_i",
    "conj",
    "convolve",
    "copy",
    "cos",
    "cosh",
    "cot",
    "coth",
    "count",
    "cross",
    "csc",
    "csch",
    "cumsum",
    "det",
    "diag2vec",
    "diagonal",
    "diagonal_hp",
    "dot",
    "eigen",
    "eigenvals",
    "eigenvecs",
    "exp",
    "extract",
    "extract_cols",
    "extract_rows",
    "fact",
    "fft",
    "fill",
    "fill_col",
    "fill_row",
    "find",
    "find_eq",
    "find_ge",
    "find_gt",
    "find_le",
    "find_lt",
    "find_ne",
    "first",
    "floor",
    "fprod",
    "gcd",
    "getunits",
    "hlookup",
    "hlookup_eq",
    "hlookup_ge",
    "hlookup_gt",
    "hlookup_le",
    "hlookup_lt",
    "hlookup_ne",
    "hp",
    "hprod",
    "identity",
    "identity_hp",
    "if",
    "ift",
    "im",
    "interp",
    "inverse",
    "ishp",
    "join",
    "join_cols",
    "join_rows",
    "kprod",
    "last",
    "lcm",
    "len",
    "line",
    "linspace",
    "linterp",
    "ln",
    "log",
    "log_2",
    "lookup",
    "lookup_eq",
    "lookup_ge",
    "lookup_gt",
    "lookup_le",
    "lookup_lt",
    "lookup_ne",
    "lsolve",
    "ltriang",
    "ltriang_hp",
    "lu",
    "mandelbrot",
    "matmul",
    "matrix",
    "matrix_hp",
    "max",
    "mcount",
    "mean",
    "median",
    "mfill",
    "mfind",
    "mfind_eq",
    "mfind_ge",
    "mfind_gt",
    "mfind_le",
    "mfind_lt",
    "mfind_ne",
    "min",
    "mnorm",
    "mnorm_1",
    "mnorm_2",
    "mnorm_e",
    "mnorm_i",
    "mod",
    "mresize",
    "msearch",
    "msolve",
    "n_cols",
    "n_rows",
    "norm",
    "norm_1",
    "norm_2",
    "norm_e",
    "norm_i",
    "norm_p",
    "not",
    "or",
    "order",
    "order_cols",
    "order_rows",
    "phase",
    "polyfit",
    "polyroots",
    "product",
    "qr",
    "random",
    "range",
    "range_hp",
    "rank",
    "re",
    "resize",
    "reverse",
    "revorder",
    "revorder_cols",
    "revorder_rows",
    "root",
    "round",
    "row",
    "rsort",
    "rsort_cols",
    "rsort_rows",
    "search",
    "sec",
    "sech",
    "setunits",
    "sign",
    "sin",
    "sinh",
    "size",
    "slice",
    "slsolve",
    "smsolve",
    "sort",
    "sort_cols",
    "sort_rows",
    "spline",
    "sqr",
    "sqrt",
    "srss",
    "stack",
    "stdev",
    "stdevp",
    "submatrix",
    "sum",
    "sumsq",
    "svd",
    "switch",
    "symmetric",
    "symmetric_hp",
    "take",
    "tan",
    "tanh",
    "timer",
    "trace",
    "transp",
    "trunc",
    "unit",
    "utriang",
    "utriang_hp",
    "var",
    "varp",
    "vec2col",
    "vec2diag",
    "vec2row",
    "vector",
    "vector_hp",
    "vlookup",
    "vlookup_eq",
    "vlookup_ge",
    "vlookup_gt",
    "vlookup_le",
    "vlookup_lt",
    "vlookup_ne",
    "xor",
];

/// True when a user function of this name would be taken for one of the engine's.
pub fn is_engine_function(name: &str) -> bool {
    let l = name.to_lowercase();
    ENGINE_FUNCTIONS.binary_search(&l.as_str()).is_ok()
}
