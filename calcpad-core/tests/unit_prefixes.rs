//! SI prefixes apply to every SI-prefixable unit (`3 mN`, `2 GPa`, `5 \u{b5}m`), and a name the unit
//! table already had keeps its meaning. See DEVIATIONS.md, D16.
use calcpad_core::base_types::unit::Unit;
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn unit(s: &str) -> std::rc::Rc<Unit> {
    Unit::try_get(s, false).unwrap_or_else(|| panic!("{s} is not a unit"))
}

fn html(src: &str) -> String {
    let mut p = ExpressionParser::new();
    p.parse(src, true);
    p.html_result.replace(['\u{2009}', '\u{200A}'], " ")
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * b.abs().max(1e-300)
}

/// (prefixed, base, how many bases it is)
const PAIRS: &[(&str, &str, f64)] = &[
    ("mN", "N", 1e-3), ("MN", "N", 1e6), ("GN", "N", 1e9), ("uN", "N", 1e-6), ("nN", "N", 1e-9), ("pN", "N", 1e-12),
    ("fN", "N", 1e-15), ("aN", "N", 1e-18), ("zN", "N", 1e-21), ("yN", "N", 1e-24), ("PN", "N", 1e15), ("EN", "N", 1e18),
    ("ZN", "N", 1e21), ("YN", "N", 1e24), ("cN", "N", 1e-2), ("dN", "N", 1e-1), ("hN", "N", 1e2), ("daN", "N", 10.0),
    ("mPa", "Pa", 1e-3), ("GPa", "Pa", 1e9), ("PPa", "Pa", 1e15), ("kbar", "bar", 1e3), ("mbar", "bar", 1e-3),
    ("mJ", "J", 1e-3), ("EJ", "J", 1e18), ("fJ", "J", 1e-15), ("kJ", "J", 1e3),
    ("mW", "W", 1e-3), ("GW", "W", 1e9), ("YW", "W", 1e24), ("kWh", "Wh", 1e3), ("mWh", "Wh", 1e-3), ("PWh", "Wh", 1e15),
    ("mHz", "Hz", 1e-3), ("GHz", "Hz", 1e9), ("uHz", "Hz", 1e-6), ("kHz", "Hz", 1e3),
    ("mV", "V", 1e-3), ("kV", "V", 1e3), ("uV", "V", 1e-6), ("nV", "V", 1e-9), ("MV", "V", 1e6),
    ("mA", "A", 1e-3), ("kA", "A", 1e3), ("uA", "A", 1e-6), ("pA", "A", 1e-12), ("fA", "A", 1e-15), ("PA", "A", 1e15),
    ("mF", "F", 1e-3), ("uF", "F", 1e-6), ("pF", "F", 1e-12), ("fF", "F", 1e-15), ("kF", "F", 1e3),
    ("mH", "H", 1e-3), ("uH", "H", 1e-6), ("nH", "H", 1e-9), ("kH", "H", 1e3),
    ("m\u{3a9}", "\u{3a9}", 1e-3), ("k\u{3a9}", "\u{3a9}", 1e3), ("G\u{3a9}", "\u{3a9}", 1e9),
    ("mS", "S", 1e-3), ("kS", "S", 1e3), ("uS", "S", 1e-6),
    ("mT", "T", 1e-3), ("nT", "T", 1e-9), ("kT", "T", 1e3), ("uT", "T", 1e-6),
    ("mWb", "Wb", 1e-3), ("kWb", "Wb", 1e3), ("mC", "C", 1e-3), ("uC", "C", 1e-6), ("nC", "C", 1e-9),
    ("mg", "g", 1e-3), ("Mg", "g", 1e6), ("Gg", "g", 1e9), ("ug", "g", 1e-6), ("yg", "g", 1e-24), ("Pg", "g", 1e15),
    ("km", "m", 1e3), ("mm", "m", 1e-3), ("um", "m", 1e-6), ("nm", "m", 1e-9), ("pm", "m", 1e-12), ("fm", "m", 1e-15),
    ("am", "m", 1e-18), ("zm", "m", 1e-21), ("ym", "m", 1e-24), ("Mm", "m", 1e6), ("Gm", "m", 1e9), ("Tm", "m", 1e12),
    ("Pm", "m", 1e15), ("Em", "m", 1e18), ("Zm", "m", 1e21), ("Ym", "m", 1e24), ("dam", "m", 10.0), ("hm", "m", 100.0),
    ("ms", "s", 1e-3), ("us", "s", 1e-6), ("ns", "s", 1e-9), ("ps", "s", 1e-12), ("fs", "s", 1e-15),     ("Ms", "s", 1e6), ("Gs", "s", 1e9), ("ds", "s", 1e-1), ("cs", "s", 1e-2), ("das", "s", 10.0), ("hs", "s", 100.0),
    ("mL", "L", 1e-3), ("uL", "L", 1e-6), ("kL", "L", 1e3), ("ML", "L", 1e6), ("GL", "L", 1e9), ("nL", "L", 1e-9),
    ("mmol", "mol", 1e-3), ("kmol", "mol", 1e3), ("umol", "mol", 1e-6), ("nmol", "mol", 1e-9), ("Mmol", "mol", 1e6),
    ("mcd", "cd", 1e-3), ("kcd", "cd", 1e3), ("Mcd", "cd", 1e6),
    ("mrad", "rad", 1e-3), ("urad", "rad", 1e-6), ("krad", "rad", 1e3),
    ("MNm", "Nm", 1e6), ("mNm", "Nm", 1e-3), ("GNm", "Nm", 1e9),
    ("mBq", "Bq", 1e-3), ("MBq", "Bq", 1e6), ("mGy", "Gy", 1e-3), ("uGy", "Gy", 1e-6), ("mSv", "Sv", 1e-3), ("uSv", "Sv", 1e-6),
    ("klm", "lm", 1e3), ("mlx", "lx", 1e-3), ("klx", "lx", 1e3), ("mkat", "kat", 1e-3),
    ("kVA", "VA", 1e3), ("MVA", "VA", 1e6), ("mVA", "VA", 1e-3), ("kVAR", "VAR", 1e3), ("meV", "eV", 1e-3), ("ZeV", "eV", 1e21),
    ("mAh", "Ah", 1e-3), ("kAh", "Ah", 1e3), ("kDa", "Da", 1e3), ("MDa", "Da", 1e6),
    ("\u{b5}m", "m", 1e-6), ("\u{3bc}m", "m", 1e-6), ("\u{b5}N", "N", 1e-6), ("\u{3bc}N", "N", 1e-6), ("\u{b5}g", "g", 1e-6),
    ("\u{b5}A", "A", 1e-6), ("\u{3bc}s", "s", 1e-6), ("\u{b5}Pa", "Pa", 1e-6), ("\u{b5}L", "L", 1e-6),
];

#[test]
fn every_prefix_and_unit_pair_converts_to_its_base() {
    assert!(PAIRS.len() >= 60, "{} pairs", PAIRS.len());
    for (p, base, f) in PAIRS {
        let pu = unit(p);
        let bu = unit(base);
        assert!(pu.is_consistent(&bu), "{p} has the dimension of {base}");
        let got = pu.convert_to(&bu);
        assert!(close(got, *f), "1 {p} = {got} {base}, expected {f}");
    }
}

#[test]
fn a_prefixed_value_computes_in_a_sheet() {
    let h = html("F = 3 mN\nG = F/1 N\nP = 2 GPa\nQ = P/1 MPa\nd = 5 \u{b5}m\ne = d/1 nm");
    assert!(!h.contains("Error"), "{h}");
    assert!(h.contains("0.003"), "{h}");
    assert!(h.contains("2000"), "{h}");
    assert!(h.contains("5000"), "{h}");
    // `3 mN` keeps its spelling in the result.
    assert!(h.contains("<i>mN</i>"), "{h}");
    // Mixed prefixes add: 2 mN + 500 uN = 2.5 mN.
    let h = html("S = 2 mN + 500 uN\nT = S/1 mN");
    assert!(h.contains("2.5"), "{h}");
}

#[test]
fn units_combine_with_prefixed_units() {
    let h = html("a = 6 kN/m\nb = 2 mm\nc = a*b|N\nd = 3 mm*2 MPa|N/m");
    assert!(!h.contains("Error"), "{h}");
    assert!(h.contains("12"), "{h}");
}

#[test]
fn names_the_table_already_had_keep_their_meaning() {
    // min is the minute, not a milli-inch (inch is not prefixable and nothing named `min` was ever prefixed).
    assert!(close(unit("min").convert_to(&unit("s")), 60.0));
    assert!(close(unit("h").convert_to(&unit("s")), 3600.0));
    assert!(close(unit("d").convert_to(&unit("h")), 24.0));
    assert!(close(unit("ft").convert_to(&unit("m")), 0.3048));
    assert!(close(unit("in").convert_to(&unit("mm")), 25.4));
    assert!(close(unit("mm").convert_to(&unit("m")), 1e-3));
    assert!(close(unit("Pa").convert_to(&unit("kPa")), 1e-3));
    assert!(close(unit("ha").convert_to(&unit("a")), 100.0));
    assert!(close(unit("T").convert_to(&unit("mT")), 1e3));
    assert!(close(unit("t").convert_to(&unit("kg")), 1e3));
    assert!(close(unit("u").convert_to(&unit("g")), 1.6605390666050505e-24));
    assert!(close(unit("Da").convert_to(&unit("u")), 1.0));
    assert!(close(unit("psi").convert_to(&unit("Pa")), 6894.75729322959));
    assert!(close(unit("kg").convert_to(&unit("g")), 1e3));
    assert!(close(unit("daN").convert_to(&unit("N")), 10.0));
    assert!(close(unit("Gy").convert_to(&unit("Sv")), 1.0));
    // cd is the candela (luminous intensity), not centi-day.
    assert!(unit("cd").is_consistent(&unit("mcd")));
    assert!(!unit("cd").is_consistent(&unit("s")));
    // Pa is the pascal, not peta-annum, and P is the poise.
    assert!(unit("Pa").is_consistent(&unit("kPa")));
    assert!(!unit("Pa").is_consistent(&unit("a")));
    assert!(close(unit("P").convert_to(&unit("cP")), 100.0));
    assert!(close(unit("PPa").convert_to(&unit("Pa")), 1e15));
}

#[test]
fn the_ambiguous_spellings_resolve_one_way() {
    // `h` is the hour but `hm` is a hectometre and `hs` a hectosecond.
    assert!(close(unit("hm").convert_to(&unit("m")), 100.0));
    assert!(close(unit("hs").convert_to(&unit("s")), 100.0));
    // `d` is the day but `dm` is a decimetre, `ds` a decisecond and `dam` a decametre (`da` + `m`, not `d` + `am`).
    assert!(close(unit("dm").convert_to(&unit("m")), 0.1));
    assert!(close(unit("dam").convert_to(&unit("m")), 10.0));
    // `fT` is a femtotesla and `ft` the foot.
    assert!(unit("fT").is_consistent(&unit("T")));
    assert!(close(unit("fT").convert_to(&unit("T")), 1e-15));
    assert!(close(unit("ft").convert_to(&unit("m")), 0.3048));
    // `u` alone is the atomic mass unit; `u` before a unit is micro.
    assert!(unit("u").is_consistent(&unit("g")));
    assert!(close(unit("um").convert_to(&unit("m")), 1e-6));
    // `Em` is an exametre, `mm` a millimetre, `mmol` a millimole, `Mg` a megagram.
    assert!(close(unit("Mg").convert_to(&unit("kg")), 1e3));
    assert!(close(unit("mmol").convert_to(&unit("mol")), 1e-3));
    // The micro sign and Greek mu are one prefix, shown as Greek mu.
    assert_eq!(unit("\u{b5}m").text(), "\u{3bc}m");
    assert_eq!(unit("\u{3bc}m").text(), "\u{3bc}m");
    assert_eq!(unit("um").text(), "\u{3bc}m");
}

#[test]
fn units_that_are_not_si_prefixable_stay_unprefixed() {
    for bad in [
        "mmin", "kmin", "mh", "kh", "kd", "md", "mft", "kft", "kin", "min2", "mlb", "kpsi", "Mpsi", "m\u{b0}C", "kK", "mK",
        "da", "mda", "kkg", "mkg", "kdeg", "mdeg", "mmi", "kyd", "kt2", "mx", "k", "m_", "PaP",
    ] {
        assert!(Unit::try_get(bad, false).is_none(), "{bad} must not be a unit");
        assert!(!Unit::exists(bad), "{bad} must not exist");
    }
}

#[test]
fn a_prefix_alone_or_on_nothing_is_not_a_unit() {
    for bad in ["k", "M", "u", "da", "\u{b5}", "\u{3bc}"] {
        // `u` is the atomic mass unit; the others are not units.
        if bad == "u" {
            assert!(Unit::try_get(bad, false).is_some());
        } else {
            assert!(Unit::try_get(bad, false).is_none(), "{bad}");
        }
    }
}

#[test]
fn a_defined_variable_still_wins_over_a_prefixed_unit() {
    let h = html("mN = 5\nx = mN*2");
    assert!(!h.contains("Error"), "{h}");
    assert!(h.contains("10"), "{h}");
}

#[test]
fn every_unit_that_existed_before_still_parses_to_the_same_unit() {
    let before = include_str!("data/unit_table_before.txt");
    let by_key: std::collections::HashMap<&str, &str> =
        before.lines().filter(|l| !l.is_empty()).map(|l| (l.split('|').next().unwrap(), l)).collect();
    let mut n = 0;
    for (&key, &line) in &by_key {
        let u = Unit::try_get(key, false).unwrap_or_else(|| panic!("{key} no longer parses"));
        // `gal`, `pt` and the like mean the UK or the US unit, as the lookup has always resolved them.
        let ambiguous = ["therm", "cwt", "ton", "fl_oz", "gi", "pt", "qt", "gal", "bbl", "pk", "bu", "tonf"];
        let (want_key, want) = if ambiguous.contains(&key) {
            let k = format!("{key}_UK");
            (key.to_string(), by_key[k.as_str()].replacen(&k, key, 1))
        } else {
            (key.to_string(), line.to_string())
        };
        assert_eq!(Unit::snapshot_line(&want_key, &u), want, "{key} changed");
        n += 1;
    }
    assert!(n > 400, "{n} units in the snapshot");
    let us = Unit::try_get("gal", true).unwrap();
    assert_eq!(us.text(), "gal_US");
}

/// Of the names the table had, only these are also a prefix and a unit with another meaning, and the
/// table wins: `ks` is the metric horsepower, so a kilosecond is written `1000 s` (DEVIATIONS.md, D16).
#[test]
fn a_table_name_that_reads_as_a_prefix_and_a_unit_is_listed() {
    let before = include_str!("data/unit_table_before.txt");
    let mut shadowed = Vec::new();
    for line in before.lines().filter(|l| !l.is_empty()) {
        let key = line.split('|').next().unwrap();
        let Some(p) = Unit::prefix_reading(key) else { continue };
        let u = Unit::try_get(key, false).unwrap();
        let same = u.is_consistent(&p) && close(u.convert_to(&p), 1.0);
        if !same {
            shadowed.push(key.to_string());
        }
    }
    shadowed.sort();
    assert_eq!(shadowed, vec!["ks".to_string()]);
}
