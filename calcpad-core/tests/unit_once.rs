//! A vector or matrix whose elements all carry one unit shows the unit once, after the closing bracket
//! (`[16; 12] ft`). `#settings {"unitPerElement": true}` restores the unit on every element.
use calcpad_core::parsers::expression_parser::ExpressionParser;

fn html(src: &str) -> String {
    let mut p = ExpressionParser::new();
    p.parse(src, true);
    // Thin and hair spaces between a number and its unit read as plain spaces here.
    p.html_result.replace(['\u{2009}', '\u{200A}'], " ")
}

fn parts(src: &str) -> (String, Vec<String>) {
    let mut p = ExpressionParser::new();
    p.capture_open_xml = true;
    p.parse(src, true);
    (p.html_result.clone(), p.open_xml_expressions.clone())
}

const OPEN: &str = "<span class=\"b0\">[</span>";
const CLOSE: &str = "<span class=\"b0\">]</span>";

#[test]
fn a_column_vector_has_its_unit_once() {
    let h = html("s = [16*ft; 12*ft; -3*ft]");
    assert!(h.contains(&format!("{OPEN}16; 12; -3{CLOSE} <i>ft</i>")), "{h}");
    assert_eq!(h.matches("<i>ft</i>").count(), 2, "{h}"); // the expression and the result
}

#[test]
fn a_row_stays_a_row_and_a_column_a_column() {
    let h = html("c = [1*m; 2*m]\nr = transp(c)");
    // The result of transp is a 1 x 2 matrix, one row of the matrix table.
    assert_eq!(h.matches("<span class=\"tr\">").count(), 1, "{h}");
    assert!(h.contains(&format!("{OPEN}1; 2{CLOSE} <i>m</i>")), "{h}");
}

#[test]
fn a_matrix_has_its_unit_once() {
    let h = html("m = [1*ft; 2*ft | 3*ft; 4*ft]");
    assert!(
        h.contains(&format!("{OPEN}1; 2 <span class=\"b0\">|</span> 3; 4{CLOSE} <i>ft</i>")),
        "{h}"
    );
    assert_eq!(h.matches("<span class=\"tr\">").count(), 2, "{h}");
}

#[test]
fn a_matrix_with_one_odd_row_keeps_the_unit_on_each_element() {
    let h = html("q = [1*ft; 2*ft | 3*m; 4*m]");
    assert!(h.contains("1 <i>ft</i>; 2 <i>ft</i>"), "{h}");
    assert!(h.contains("3 <i>m</i>; 4 <i>m</i>"), "{h}");
}

#[test]
fn different_units_of_one_dimension_stay_as_written_and_the_result_uses_the_first() {
    let h = html("s = [1*kN; 500*N]");
    assert!(h.contains("1 <i>kN</i>; 500 <i>N</i>"), "{h}");
    assert!(h.contains("0.5<b class=\"b0\">]</b> <i>kN</i>"), "{h}");
    assert_eq!(h.matches("<i>kN</i>").count(), 2, "{h}");
}

#[test]
fn different_dimensions_keep_the_unit_on_each_element() {
    let h = html("s = [16*ft; 12*kN]");
    assert!(h.contains("16 <i>ft</i>; 12 <i>kN</i>"), "{h}");
}

#[test]
fn a_unitless_element_keeps_the_unit_on_each_element() {
    let h = html("s = [1*kN; 2]");
    assert!(h.contains(&format!("{OPEN}1 <i>kN</i>; 2{CLOSE}")), "{h}");
}

#[test]
fn substituted_variables_share_the_unit_and_so_does_a_negated_one() {
    let h = html("a = 5*ft\nb = 6*ft\ns = [a; b; -a]");
    assert!(h.contains(&format!("{OPEN}5; 6; -5{CLOSE} <i>ft</i>")), "{h}");
}

#[test]
fn an_element_that_is_an_expression_keeps_the_unit_on_each_element() {
    let h = html("a = 5*ft\ns = [a; 2*a]");
    assert!(h.contains("5 <i>ft</i>; 2"), "{h}");
    assert_eq!(h.matches("<i>ft</i>").count(), 4, "{h}"); // a, the 5 ft in each row, the result
}

#[test]
fn a_single_element_vector_and_decimals_and_a_compound_unit() {
    let h = html("z = [3.25*kN*m/s^2]");
    assert!(
        h.contains("3.25<span class=\"b1\">]</span> <span class=\"dvc\"><i>kN</i>"),
        "{h}"
    );
}

#[test]
fn a_large_vector_has_one_unit_in_the_expression_and_one_in_the_result() {
    let items: Vec<String> = (1..=40).map(|i| format!("{i}*m")).collect();
    let h = html(&format!("w = [{}]", items.join("; ")));
    assert_eq!(h.matches("<i>m</i>").count(), 2, "{h}");
    assert!(h.contains(&format!("39; 40{CLOSE} <i>m</i>")), "{h}");
}

#[test]
fn a_vector_of_mixed_but_convertible_units_converts_to_the_first_unit() {
    use calcpad_core::base_types::real_value::RealValue;
    use calcpad_core::base_types::unit::Unit;
    use calcpad_core::output::html_writer::HtmlWriter;
    use calcpad_core::output::text_writer::TextWriter;
    use calcpad_core::output::xml_writer::XmlWriter;
    use calcpad_core::output::OutputWriter;
    use calcpad_core::settings::MathSettings;
    use calcpad_core::vector::Vector;
    let kn = Unit::get("kN", false).unwrap();
    let n = Unit::get("N", false).unwrap();
    let v = Vector::from_values(vec![
        RealValue::new(1.5, Some(kn.clone())),
        RealValue::new(-500.0, Some(n.clone())),
        RealValue::new(2.0, Some(kn.clone())),
    ])
    .unwrap();
    let settings = MathSettings::default();
    let h = HtmlWriter::new(Some(&settings), false).format_vector(&v);
    assert!(h.contains("1.5") && h.contains("-0.5") && h.contains("2"), "{h}");
    assert_eq!(h.matches("<i>kN</i>").count(), 1, "{h}");
    assert!(!h.contains("<i>N</i>"), "{h}");
    let t = TextWriter::new(Some(&settings), false).format_vector(&v);
    assert_eq!(t.matches("kN").count(), 1, "{t}");
    assert!(t.ends_with("kN"), "{t}");
    let x = XmlWriter::new(Some(&settings), false).format_vector(&v);
    assert_eq!(x.matches("<m:t>kN</m:t>").count(), 1, "{x}");
    // With the setting on, each element keeps its own unit.
    let mut per = MathSettings::default();
    per.unit_per_element = true;
    let h = HtmlWriter::new(Some(&per), false).format_vector(&v);
    assert_eq!(h.matches("<i>kN</i>").count(), 2, "{h}");
    assert_eq!(h.matches("<i>N</i>").count(), 1, "{h}");
    // A unitless element, or one of another dimension, keeps the per-element display.
    let m = Unit::get("m", false).unwrap();
    let mixed = Vector::from_values(vec![
        RealValue::new(1.0, Some(kn.clone())),
        RealValue::new(2.0, Some(m)),
    ])
    .unwrap();
    let h = HtmlWriter::new(Some(&settings), false).format_vector(&mixed);
    assert_eq!(h.matches("<i>").count(), 2, "{h}");
    let bare = Vector::from_values(vec![RealValue::new(1.0, Some(kn)), RealValue::new(2.0, None)]).unwrap();
    let h = HtmlWriter::new(Some(&settings), false).format_vector(&bare);
    assert_eq!(h.matches("<i>").count(), 1, "{h}");
}

#[test]
fn the_setting_restores_the_unit_on_every_element() {
    let h = html("#settings {\"unitPerElement\": true}\ns = [16*ft; 12*ft; -3*ft]\na = 5*ft\nt = [a; a]\nm = [1*ft; 2*ft | 3*ft; 4*ft]");
    assert!(
        h.contains(&format!("{OPEN}16 <i>ft</i>; 12 <i>ft</i>; -3 <i>ft</i>{CLOSE}")),
        "{h}"
    );
    assert!(h.contains(&format!("{OPEN}5 <i>ft</i>; 5 <i>ft</i>{CLOSE}")), "{h}");
    assert!(h.contains("1 <i>ft</i>; 2 <i>ft</i>"), "{h}");
}

#[test]
fn word_output_has_the_unit_once_per_bracket() {
    let (_, x) = parts("s = [16*ft; 12*ft; 3*ft]");
    assert_eq!(x.len(), 1);
    // The expression and the result each write the unit once.
    assert_eq!(x[0].matches("<m:t>ft</m:t>").count(), 2, "{}", x[0]);
    let (_, y) = parts("#settings {\"unitPerElement\": true}\ns = [16*ft; 12*ft; 3*ft]");
    assert_eq!(y[0].matches("<m:t>ft</m:t>").count(), 6, "{}", y[0]);
}
