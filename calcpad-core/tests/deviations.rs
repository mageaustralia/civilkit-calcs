//! Tests for the deliberate deviations listed in DEVIATIONS.md.
use calcpad_core::parsers::expression_parser::{ExpressionParser, UNRESOLVED_CONDITION};

fn render(source: &str) -> String {
    let mut p = ExpressionParser::new();
    p.parse(source, true);
    p.html_result
}

/// D1: CalcpadCE runs the #else branch when the #if condition fails to evaluate
/// (for example a misspelled variable). calcpad-rs runs neither branch and reports it.
#[test]
fn d1_failed_if_condition_runs_neither_branch() {
    let html = render("v = 5\n#if v > v_typo\n'NOT OK\n#else\n'OK\n#end if\n'after");
    assert!(!html.contains(">NOT OK "), "the #if branch must not run: {html}");
    assert!(!html.contains(">OK "), "the #else branch must not run: {html}");
    assert!(
        html.contains(UNRESOLVED_CONDITION),
        "the block must be reported unresolved: {html}"
    );
    assert!(
        html.contains("v_typo"),
        "the error must name the failing expression: {html}"
    );
    assert!(html.contains(">after "), "lines after the block still run: {html}");
}

#[test]
fn d1_failed_else_if_condition_skips_the_rest_of_the_chain() {
    let html = render("v = 5\n#if v < 0\n'NEGATIVE\n#else if v > v_typo\n'BIG\n#else\n'SMALL\n#end if");
    assert!(!html.contains(">NEGATIVE "), "{html}");
    assert!(!html.contains(">BIG "), "{html}");
    assert!(!html.contains(">SMALL "), "{html}");
    assert!(html.contains(UNRESOLVED_CONDITION), "{html}");
}

/// The deviation must not change valid conditions.
#[test]
fn valid_conditions_are_unchanged() {
    let html = render("v = 5\n#if v > 3\n'YES\n#else\n'NO\n#end if");
    assert!(html.contains(">YES "), "{html}");
    assert!(!html.contains(">NO "), "{html}");
    let html = render("v = 1\n#if v > 3\n'YES\n#else\n'NO\n#end if");
    assert!(!html.contains(">YES "), "{html}");
    assert!(html.contains(">NO "), "{html}");
    assert!(!html.contains(UNRESOLVED_CONDITION), "{html}");
}

/// D12: a block nested in another block assigns the enclosing block's variable, so a loop in a
/// function body can accumulate. At the worksheet level a block still keeps its assignments private.
#[test]
fn d12_nested_block_updates_the_enclosing_block_variable() {
    let html = render("f(n) = $Block{s = 0; $Repeat{s = s + k @ k = 1 : n}; s}\nr = f(5)");
    assert!(html.contains("= 15"), "{html}");
    let html = render("f(n) = $Block{s = 0; k = 1; $While{k ≤ n; s = s + k; k = k + 1}; s}\nr = f(4)");
    assert!(html.contains("= 10"), "{html}");
}

#[test]
fn d12_nested_loops_set_elements_of_a_block_local_matrix() {
    let src = "g(n) = $Block{M = matrix(n; n); $Repeat{$Repeat{M.(i; j) = 10*i + j @ j = 1 : n} @ i = 1 : n}; M}\nR = g(2)\nR.(2; 1)";
    let html = render(src);
    assert!(html.contains("= 21"), "{html}");
}

#[test]
fn d12_loop_counter_does_not_touch_an_outer_variable_of_the_same_name() {
    let html = render("f(n) = $Block{k = 100; s = 0; $Repeat{s = s + k @ k = 1 : n}; s + k}\nr = f(3)");
    assert!(html.contains("= 106"), "{html}");
}

#[test]
fn d12_worksheet_level_block_stays_private() {
    let html = render("s = 0\nr = $Repeat{s = s + k @ k = 1 : 5}\ns");
    assert!(html.contains("= 15"), "{html}");
    assert_eq!(html.matches("<var>s</var> = 0</span>").count(), 2, "{html}");
}
