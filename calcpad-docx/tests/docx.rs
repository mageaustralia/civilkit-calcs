//! The Word writer on HTML of the shapes the engine produces. Each case follows
//! OpenXmlWriter.cs / TableBuilder.cs; the documents are read back with calcpad-core's zip and
//! XML code, so a malformed package or part fails here.
use calcpad_core::openxml::{xml, zip};
use calcpad_docx::html_to_docx;

struct Doc {
    parts: Vec<zip::Entry>,
    body: xml::Element,
    text: String,
}

fn docx(html: &str, expressions: &[&str]) -> Doc {
    let ex: Vec<String> = expressions.iter().map(|s| s.to_string()).collect();
    let bytes = html_to_docx(html, &ex, None);
    let parts = zip::read(&bytes).expect("a zip");
    for p in &parts {
        if p.name.ends_with(".xml") || p.name.ends_with(".rels") {
            xml::parse(std::str::from_utf8(&p.data).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", p.name));
        }
    }
    let text = String::from_utf8(
        parts
            .iter()
            .find(|p| p.name == "word/document.xml")
            .unwrap()
            .data
            .clone(),
    )
    .unwrap();
    let root = xml::parse(&text).unwrap();
    let body = root.child("body").unwrap().clone();
    Doc { parts, body, text }
}

fn paragraphs(d: &Doc) -> Vec<&xml::Element> {
    d.body.elements().filter(|e| xml::local(&e.name) == "p").collect()
}

#[test]
fn a_package_has_its_parts_and_ends_with_section_properties() {
    let d = docx(
        "<html><head><style>p{}</style></head><body><p>Hello</p></body></html>",
        &[],
    );
    let names: Vec<&str> = d.parts.iter().map(|p| p.name.as_str()).collect();
    for n in [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "word/numbering.xml",
    ] {
        assert!(names.contains(&n), "{n} in {names:?}");
    }
    assert_eq!(paragraphs(&d)[0].inner_text(), "Hello");
    assert!(d
        .text
        .contains("<w:pgMar w:top=\"850\" w:right=\"850\" w:bottom=\"850\" w:left=\"1134\""));
}

/// Calcpad's page break, `<div style="page-break-after:always"></div>`, and the other spellings of a forced break are
/// Word page breaks in the same place; a style that only avoids breaks is not one.
#[test]
fn page_break_styles_are_word_page_breaks() {
    const BRK: &str = "<w:br w:type=\"page\"/>";
    let d = docx(
        "<body><p>First</p><div style=\"page-break-after:always\"></div><p>Second</p></body>",
        &[],
    );
    assert_eq!(d.text.matches(BRK).count(), 1, "{}", d.text);
    let at = |s: &str| d.text.find(s).unwrap();
    assert!(at(">First<") < at(BRK) && at(BRK) < at(">Second<"), "{}", d.text);
    // The break is a paragraph of its own between the two, so Word starts "Second" on the next page.
    let ps = paragraphs(&d);
    assert_eq!(ps.len(), 3);
    assert_eq!(ps[1].inner_text(), "");

    let d = docx(
        "<body><p>A</p><p style=\"page-break-before: always\">B</p><h2 style=\"break-after: page\">C</h2><p>D</p>\
         <div style=\"break-before:page;color:red\"><p>E</p></div><p style=\"page-break-inside:avoid;break-after:avoid\">F</p></body>",
        &[],
    );
    assert_eq!(d.text.matches(BRK).count(), 3, "{}", d.text);
    let order: Vec<usize> = [">A<", BRK, ">B<", ">C<", ">D<", ">E<", ">F<"]
        .iter()
        .map(|s| d.text.find(s).unwrap())
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{}", d.text);
    let between = |a: &str, b: &str| {
        d.text[d.text.find(a).unwrap()..d.text.find(b).unwrap()]
            .matches(BRK)
            .count()
    };
    assert_eq!(
        (
            between(">A<", ">B<"),
            between(">C<", ">D<"),
            between(">D<", ">E<"),
            between(">E<", ">F<")
        ),
        (1, 1, 1, 0)
    );
    assert!(!d.text.contains("p{}"), "the head is not part of the document");
}

#[test]
fn headings_runs_and_classes() {
    let d = docx(
        "<body><h2>Title</h2><p>a <b>bold <i>both</i></b> <sub>s</sub></p><p class=\"err\">bad</p><p class=\"ref\">[1]</p></body>",
        &[],
    );
    assert!(d.text.contains("<w:pStyle w:val=\"h2\"/>"));
    // The inner run takes the outer run's properties too (CopyRunProperties).
    assert!(
        d.text
            .contains("<w:rPr><w:b/><w:i/></w:rPr><w:t xml:space=\"preserve\">both</w:t>"),
        "{}",
        d.text
    );
    assert!(
        d.text
            .contains("<w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">bold </w:t>"),
        "{}",
        d.text
    );
    assert!(d.text.contains("<w:vertAlign w:val=\"subscript\"/>"));
    assert!(
        d.text
            .contains("<w:shd w:val=\"clear\" w:color=\"DC143C\" w:fill=\"FFEEEE\"/>"),
        "an error paragraph: {}",
        d.text
    );
    assert!(
        d.text.contains("<w:jc w:val=\"right\"/>"),
        "a reference is right aligned"
    );
}

#[test]
fn equations_come_from_the_captured_list_by_id() {
    let d = docx(
        "<body><p>x: <span class=\"eq\" id=\"eq-1\"><var>x</var> = 1</span></p></body>",
        &["<m:r><m:t>zero</m:t></m:r>", "<m:r><m:t>one</m:t></m:r>"],
    );
    assert!(d.text.contains("<m:oMathPara><m:oMathParaPr><m:jc m:val=\"left\"/></m:oMathParaPr><m:oMath><m:r><m:t>one</m:t></m:r></m:oMath></m:oMathPara>"), "{}", d.text);
    assert!(!d.text.contains("zero"));
}

#[test]
fn hidden_parts_comments_and_scripts_are_left_out() {
    let d = docx(
        "<body><p style=\"display: none\">hidden</p><span class=\"lineLink\">L</span><!-- c --><script>var s = 1;</script><p>shown</p></body>",
        &[],
    );
    assert!(!d.text.contains("hidden") && !d.text.contains("var s") && !d.text.contains(">L<"));
    assert!(d.text.contains("shown"));
}

#[test]
fn a_markdown_table_is_ruled_and_a_plain_table_is_not() {
    let md = docx(
        "<body><table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table></body>",
        &[],
    );
    assert!(md.text.contains("<w:tblStyle w:val=\"bcpd\"/>"), "{}", md.text);
    let plain = docx("<body><table><tr><td>1</td><td>2</td></tr></table></body>", &[]);
    assert!(!plain.text.contains("w:val=\"bcpd\""));
}

#[test]
fn tables_span_columns_and_rows() {
    let d = docx(
        "<body><table class=\"bordered\"><tr><th colspan=\"2\">H</th></tr><tr><td rowspan=\"2\">a</td><td>b</td></tr><tr><td>c</td></tr></table></body>",
        &[],
    );
    assert!(d.text.contains("<w:tblStyle w:val=\"bcpd\"/>"));
    assert!(d.text.contains("<w:gridSpan w:val=\"2\"/>"));
    assert!(d.text.contains("<w:vMerge w:val=\"restart\"/>"));
    // The continued cell comes before "c" in the third row.
    let row3 = d.text.split("<w:tr>").nth(3).unwrap();
    let cont = row3.find("<w:vMerge w:val=\"continue\"/>").expect("a continued cell");
    assert!(cont < row3.find(">c<").unwrap(), "{row3}");
    // Header cells are shaded and their text bold.
    assert!(d.text.contains("<w:shd w:val=\"clear\" w:fill=\"F0F0F0\"/>"));
    assert!(d
        .text
        .contains("<w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">H</w:t>"));
    // A matrix table is not written.
    let m = docx(
        "<body><table class=\"matrix\"><tr><td>1</td></tr></table><p>after</p></body>",
        &[],
    );
    assert!(!m.text.contains("<w:tbl>"));
}

#[test]
fn lists_are_numbered_per_list() {
    let d = docx(
        "<body><ul><li>dot</li></ul><ol><li>one</li><li>two</li></ol><ol><li>again</li></ol></body>",
        &[],
    );
    assert!(
        d.text
            .contains("<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr>"),
        "bullets use instance 1"
    );
    let numbering = String::from_utf8(
        d.parts
            .iter()
            .find(|p| p.name == "word/numbering.xml")
            .unwrap()
            .data
            .clone(),
    )
    .unwrap();
    // Two ordered lists, two instances restarting at 1 (D6).
    assert_eq!(
        numbering.matches("<w:startOverride w:val=\"1\"/>").count(),
        2,
        "{numbering}"
    );
    let ids: Vec<&str> = d
        .text
        .match_indices("<w:numId w:val=\"")
        .map(|(i, _)| &d.text[i + 16..i + 17])
        .collect();
    assert_eq!(ids.len(), 4);
    assert_ne!(ids[1], ids[3], "each ordered list has its own numbering: {ids:?}");
}

#[test]
fn links_rules_and_inputs() {
    let d = docx("<body><p><a href=\"https://example.com/x\">site</a></p><hr><p>t<hr>u</p><p><input type=\"checkbox\" checked> <input value=\"12\"></p></body>", &[]);
    assert!(
        d.text.contains("<w:hyperlink r:id=\"rId3\" w:tgtFrame=\"_blank\">"),
        "{}",
        d.text
    );
    let rels = String::from_utf8(
        d.parts
            .iter()
            .find(|p| p.name == "word/_rels/document.xml.rels")
            .unwrap()
            .data
            .clone(),
    )
    .unwrap();
    assert!(rels.contains("Target=\"https://example.com/x\" TargetMode=\"External\""));
    assert!(
        d.text
            .contains("<w:bottom w:val=\"single\" w:sz=\"2\" w:space=\"0\" w:color=\"888888\"/>"),
        "a rule between paragraphs is a border"
    );
    assert!(
        d.text.contains(&"\u{2500}".repeat(55)),
        "a rule in a paragraph is a line of box characters"
    );
    assert!(d.text.contains("\u{2611}") && d.text.contains(">12<"));
}

#[test]
fn a_data_image_goes_in_with_its_size() {
    // A 4 x 2 PNG.
    let png =
        "iVBORw0KGgoAAAANSUhEUgAAAAQAAAACCAYAAAB/qH1jAAAAEUlEQVR42mP8z8Dwn4EIwDiqEABQ+gP+lJ8xbAAAAABJRU5ErkJggg==";
    let d = docx(&format!("<body><p><img src=\"data:image/png;base64,{png}\" alt=\"pic.png\"></p><img class=\"side\" style=\"width:40px\" src=\"data:image/png;base64,{png}\"></body>"), &[]);
    let media: Vec<&str> = d
        .parts
        .iter()
        .map(|p| p.name.as_str())
        .filter(|n| n.starts_with("word/media/"))
        .collect();
    assert_eq!(media, ["word/media/image1.png", "word/media/image2.png"]);
    assert!(
        d.text.contains("<wp:inline") && d.text.contains("<wp:extent cx=\"38100\" cy=\"19050\"/>"),
        "4 x 2 px in EMU: {}",
        d.text
    );
    // class="side" floats right, width 40 px and the height scaled to match.
    assert!(
        d.text.contains("<wp:align>right</wp:align>") && d.text.contains("<wp:extent cx=\"381000\" cy=\"190500\"/>"),
        "{}",
        d.text
    );
    let types = String::from_utf8(d.parts[0].data.clone()).unwrap();
    assert!(types.contains("Extension=\"png\" ContentType=\"image/png\""));
}

#[test]
fn a_missing_image_is_reported_in_the_document() {
    let d = docx("<body><img src=\"../Images/none.png\"></body>", &[]);
    assert!(d
        .text
        .contains("Error adding image: Could not find image file ../Images/none.png."));
}

#[test]
fn an_svg_drawing_becomes_a_png_with_the_svg_beside_it() {
    let d = docx("<body><svg width=\"100\" height=\"50\" viewbox=\"0 0 100 50\"><rect width=\"100\" height=\"50\" fill=\"red\"/></svg></body>", &[]);
    let media: Vec<&str> = d
        .parts
        .iter()
        .map(|p| p.name.as_str())
        .filter(|n| n.starts_with("word/media/"))
        .collect();
    assert_eq!(media, ["word/media/image1.png", "word/media/image2.svg"]);
    let png = &d.parts.iter().find(|p| p.name == "word/media/image1.png").unwrap().data;
    assert_eq!(
        calcpad_docx::image::bitmap_size(png),
        Some((200, 100)),
        "drawn at twice its size"
    );
    assert!(d.text.contains("<asvg:svgBlip r:embed="));
    let svg = String::from_utf8(
        d.parts
            .iter()
            .find(|p| p.name == "word/media/image2.svg")
            .unwrap()
            .data
            .clone(),
    )
    .unwrap();
    assert!(svg.contains("viewBox=\"0 0 100 50\""), "{svg}");
}

#[test]
fn content_of_unknown_containers_is_kept() {
    let d = docx("<body><blockquote><p>quoted <abbr>abbr</abbr></p></blockquote><noscript>no</noscript><button>b</button></body>", &[]);
    assert!(d.text.contains(">quoted <") && d.text.contains(">abbr<"), "{}", d.text);
    assert!(!d.text.contains(">no<") && !d.text.contains(">b<"));
}

/// A sheet's folding section is a `<details>` with the heading in its `<summary>`, and a key line ends in a star: the
/// Word file has the heading, the section's lines and the star, as if the section were open.
#[test]
fn a_folding_section_and_a_key_star_are_written_in_full() {
    let d = docx(
        "<html><body><details class=\"sheet-collapse\"><summary><h2>Workings</h2></summary>\
         <p>Some prose.</p><p><span class=\"sheet-note\">moment</span> <span class=\"sheet-key\" title=\"key result\">\u{2605}</span></p>\
         </details><h2>Results</h2><div class=\"sheet-key-results\"><p>y</p></div></body></html>",
        &[],
    );
    for want in ["Workings", "Some prose.", "moment", "\u{2605}", "Results"] {
        assert!(d.text.contains(want), "{want} in the document: {}", d.text);
    }
    // The heading inside the summary is a heading, in order with the text of the section.
    let at = |s: &str| d.text.find(s).unwrap();
    assert!(at("Workings") < at("Some prose.") && at("Some prose.") < at("\u{2605}") && at("\u{2605}") < at("Results"));
    assert_eq!(d.text.matches("<w:pStyle w:val=\"h2\"/>").count(), 2, "the summary's heading and the next one are both headings: {}", d.text);
}
