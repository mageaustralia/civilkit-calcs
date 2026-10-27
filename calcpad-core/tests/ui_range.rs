//! `#UI {"min": .., "max": .., "step": ..}` on a number entry (DEVIATIONS.md, D17): the bounds are written to the
//! entry for the form's slider, a bad directive is an error, and a stored value outside the bounds is an error.
use std::collections::HashMap;

use calcpad_core::parsers::expression_parser::ExpressionParser;

fn render(source: &str, enable_ui: bool, overrides: &[(&str, &str)]) -> String {
    let mut parser = ExpressionParser::new();
    parser.enable_ui = enable_ui;
    parser.ui_overrides = if overrides.is_empty() {
        None
    } else {
        Some(overrides.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>())
    };
    parser.show_error_lines = !enable_ui;
    parser.parse(source, true);
    parser.html_result.clone()
}

#[test]
fn an_entry_with_bounds_carries_them_and_one_without_is_unchanged() {
    let h = render("#UI {\"min\": 2, \"max\": 12, \"step\": 0.5} L = 6m", true, &[]);
    assert!(h.contains("data-ui-min=\"2\" data-ui-max=\"12\" data-ui-step=\"0.5\""), "{h}");
    assert!(h.contains("calcpad-ui-input"), "{h}");
    let h = render("#UI {\"min\": -1.5, \"max\": 1e3} L = 6", true, &[]);
    assert!(h.contains("data-ui-min=\"-1.5\" data-ui-max=\"1000\""), "{h}");
    assert!(!h.contains("data-ui-step"), "{h}");
    let plain = render("#UI L = 6m", true, &[]);
    assert!(!plain.contains("data-ui-m") && !plain.contains("data-ui-step"), "{plain}");
    let typed = render("#UI {\"type\": \"entry\", \"min\": 0, \"max\": 9} L = 6m", true, &[]);
    assert!(typed.contains("data-ui-min=\"0\" data-ui-max=\"9\""), "{typed}");
    // Without the form there is no control and no attribute.
    let report = render("#UI {\"min\": 2, \"max\": 12} L = 6m", false, &[]);
    assert!(!report.contains("data-ui-m") && !report.contains("<input"), "{report}");
}

#[test]
fn a_bad_range_is_an_error() {
    for (src, want) in [
        ("#UI {\"min\": 5, \"max\": 2} L = 3", "'min' (5) must not be above 'max' (2)"),
        ("#UI {\"min\": 1, \"max\": 2, \"step\": 0} L = 3", "'step' must be above zero"),
        ("#UI {\"step\": -1} L = 3", "'step' must be above zero"),
        ("#UI {\"type\": \"checkbox\", \"min\": 0} L = 1", "'min' only applies to an entry"),
        ("#UI {\"type\": \"dropdown\", \"keys\": [\"a\"], \"values\": [\"1\"], \"max\": 3} L = 1", "'max' only applies to an entry"),
        ("#UI {\"min\": \"x\"} L = 3", "Invalid JSON"),
    ] {
        let h = render(src, true, &[]);
        assert!(h.contains(want), "{src}: {h}");
        assert!(!h.contains("calcpad-ui-input"), "{src}: {h}");
    }
}

#[test]
fn a_stored_value_outside_the_bounds_is_an_error_on_the_line() {
    let src = "#UI {\"min\": 2, \"max\": 12} L = 6m\nA = L*2";
    for (value, bad) in [("8", false), ("2", false), ("12", false), ("12.5", true), ("1", true), ("-4", true), ("\u{2212}4", true)] {
        for enable_ui in [true, false] {
            let h = render(src, enable_ui, &[("L:1", value)]);
            assert_eq!(h.contains("is outside the range 2 to 12"), bad, "{value} ui={enable_ui}: {h}");
        }
    }
    // A bound on one side only.
    let h = render("#UI {\"max\": 12} L = 6m", false, &[("L:1", "-50")]);
    assert!(!h.contains("outside the range"), "{h}");
    let h = render("#UI {\"max\": 12} L = 6m", false, &[("L:1", "13")]);
    assert!(h.contains("is outside the range"), "{h}");
}
