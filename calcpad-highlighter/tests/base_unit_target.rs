//! `|base` (the Calcpad form of the sheet's `-> base`) names SI base units; it is not an undefined unit.
use calcpad_highlighter::content_resolution::ContentResolver;
use calcpad_highlighter::linter::models::LinterResult;
use calcpad_highlighter::linter::{CalcpadLinter, LintIgnoreRegionParser};

fn lint(content: &str) -> LinterResult {
    let staged = ContentResolver::new().get_staged_content(content, None, None, None);
    let regions = LintIgnoreRegionParser::extract_regions(content);
    CalcpadLinter::new().lint(&staged, &regions)
}

fn undefined_units(r: &LinterResult) -> usize {
    r.diagnostics.iter().filter(|d| d.code == "CPD-3310").count()
}

#[test]
fn base_after_a_bar_is_not_an_undefined_unit() {
    let r = lint("F = 2kN\nL = 5m\nM = F*L|base\nN = F*L | base\n");
    assert_eq!(0, undefined_units(&r));
}

#[test]
fn base_anywhere_else_is_still_an_undefined_unit() {
    let r = lint("F = 2kN\nM = F*base\n");
    assert!(undefined_units(&r) > 0 || r.diagnostics.iter().any(|d| d.code == "CPD-3301"));
}
