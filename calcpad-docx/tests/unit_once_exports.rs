//! The Word, Excel and HTML exports of a vector with a shared unit still carry the unit once.
use calcpad_core::openxml::xml::{self, Element};
use calcpad_core::openxml::{variables_export, zip};
use calcpad_core::parsers::expression_parser::ExpressionParser;
use calcpad_docx::{html_to_docx, html_to_xlsx};

const SHEET: &str = "s = [16*ft; 12*ft; -3*ft]";

fn parsed(per_element: bool) -> ExpressionParser {
    let mut p = ExpressionParser::new();
    p.capture_open_xml = true;
    p.capture_values = true;
    let src = if per_element {
        format!("#settings {{\"unitPerElement\": true}}\n{SHEET}")
    } else {
        SHEET.to_string()
    };
    p.parse(&src, true);
    p
}

fn part(book: &[u8], name: &str) -> String {
    let e = zip::read(book)
        .unwrap()
        .into_iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("no {name}"));
    String::from_utf8(e.data).unwrap()
}

fn report_cells(p: &ExpressionParser) -> Vec<String> {
    let sheets = variables_export::sheets(&p.captured_values);
    let book = html_to_xlsx(&p.html_result, &sheets, &[], None).unwrap();
    let root = xml::parse(&part(&book, "xl/worksheets/sheet1.xml")).unwrap();
    let mut cells: Vec<&Element> = Vec::new();
    root.descendants("c", &mut cells);
    cells.iter().map(|c| c.inner_text()).collect()
}

#[test]
fn word_has_the_unit_once_in_the_expression_and_once_in_the_result() {
    let p = parsed(false);
    let docx = html_to_docx(&p.html_result, &p.open_xml_expressions, None);
    let doc = part(&docx, "word/document.xml");
    assert_eq!(doc.matches("<m:t>ft</m:t>").count(), 2, "{doc}");
}

#[test]
fn word_with_the_setting_has_the_unit_on_every_element() {
    let p = parsed(true);
    let docx = html_to_docx(&p.html_result, &p.open_xml_expressions, None);
    let doc = part(&docx, "word/document.xml");
    assert_eq!(doc.matches("<m:t>ft</m:t>").count(), 6, "{doc}");
}

#[test]
fn excel_report_has_the_unit_once_in_each_cell_and_its_sheet_a_unit_column() {
    let p = parsed(false);
    let cells = report_cells(&p);
    assert!(cells.iter().any(|c| c == "[16; 12; -3]ft"), "{cells:?}");
    assert!(cells.iter().any(|c| c == "[16 12 -3]ft"), "{cells:?}");
    let sheets = variables_export::sheets(&p.captured_values);
    let v = sheets.iter().find(|s| s.name == "s").unwrap();
    assert_eq!(v.rows[1], ["1", "16", "ft"]);
    assert_eq!(v.rows[3], ["3", "-3", "ft"]);
}

#[test]
fn html_has_the_unit_once_after_each_bracket() {
    let p = parsed(false);
    assert_eq!(p.html_result.matches("<i>ft</i>").count(), 2, "{}", p.html_result);
    let q = parsed(true);
    assert_eq!(q.html_result.matches("<i>ft</i>").count(), 6, "{}", q.html_result);
}
