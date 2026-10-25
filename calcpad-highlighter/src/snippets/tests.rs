use super::registry::{extract_command_name, extract_function_name, extract_keyword_name, extract_setting_name};
use super::*;

const DIACRITICS: usize = 4 * 52;

#[test]
fn item_counts_match_csharp_new_snippet_item_counts() {
    // `grep -c "new SnippetItem"` per Data file (Symbol: 63 minus the one in MakeDiacriticItem).
    let expected: [(&str, usize, usize); 14] = [
        ("constant", constant_snippets().len(), 28),
        ("operator", operator_snippets().len(), 21),
        ("function", function_snippets().len(), 99),
        ("vector", vector_function_snippets().len(), 49),
        ("matrix", matrix_function_snippets().len(), 105),
        ("keyword", keyword_snippets().len(), 90),
        ("command", command_snippets().len(), 18),
        ("setting", setting_snippets().len(), 11),
        ("unit", unit_snippets().len(), 405),
        ("format", format_string_snippets().len(), 13),
        ("html", html_snippets().len(), 43),
        ("css", css_snippets().len(), 15),
        ("markdown", markdown_snippets().len(), 11),
        ("symbol", symbol_snippets().len(), 62 + DIACRITICS),
    ];
    for (name, got, want) in expected {
        assert_eq!(got, want, "{name}");
    }
    assert_eq!(SnippetRegistry::all_snippets().len(), 970 + DIACRITICS);
}

#[test]
fn registry_order_and_diacritics() {
    let all = SnippetRegistry::all_snippets();
    assert_eq!(all[0].insert, "π");
    assert_eq!(all[28].insert, "!");
    assert_eq!(all[28 + 21].insert, "sin(§)");
    assert_eq!(all.last().unwrap().insert, "Z\u{0301}");
    let ls = symbol_snippets();
    let diac = &ls[62];
    assert_eq!(diac.insert, "a\u{0304}");
    assert_eq!(diac.description, "a with bar");
    assert_eq!(diac.label.as_deref(), Some("a\u{0304} abar"));
    assert_eq!(diac.category, "Symbols/Bar Lowercase");
    assert_eq!(ls[62 + 26].category, "Symbols/Bar Uppercase");
    assert_eq!(ls[62 + 52].description, "a with dot");
    assert_eq!(ls[62 + 104].description, "a with double dot");
}

#[test]
fn by_category_groups_on_top_level() {
    let by = SnippetRegistry::by_category();
    assert_eq!(by["Functions"].len(), 253);
    assert_eq!(by["Units"].len(), 405);
    assert_eq!(by["Symbols"].len(), 62 + 208);
    assert_eq!(by["Constants"][0].insert, "π");
    assert_eq!(by.len(), 22);
}

#[test]
fn name_sets() {
    assert_eq!(SnippetRegistry::get_function_names().len(), 228);
    assert!(SnippetRegistry::get_function_names().contains("SIN"));
    assert!(SnippetRegistry::get_function_names().contains("lsolve"));
    for n in [
        "median", "stdev", "stdevp", "var", "varp", "cumsum", "convolve", "linspace", "polyfit", "interp", "linterp", "polyroots",
    ] {
        assert!(SnippetRegistry::get_function_names().contains(n), "{n}");
    }
    assert!(!SnippetRegistry::get_function_names().contains("sin("));

    let kw = SnippetRegistry::get_keyword_names();
    assert!(kw.contains("#include"));
    assert!(kw.contains("#END HIDE"));
    assert!(kw.contains("#md on"));
    assert!(kw.contains("#round"));
    assert!(!kw.contains("#if"));

    let cb = SnippetRegistry::get_control_block_keyword_names();
    assert_eq!(cb.len(), 7);
    assert!(cb.contains("#else if"));
    assert!(cb.contains("#ELSE"));
    let end = SnippetRegistry::get_end_keyword_names();
    assert_eq!(end.len(), 3);
    assert!(end.contains("#end if") && end.contains("#loop") && end.contains("#end def"));

    let cmd = SnippetRegistry::get_command_names();
    assert!(cmd.contains("$root") && cmd.contains("$Plot") && cmd.contains("$Map"));

    let settings = SnippetRegistry::get_setting_names();
    assert_eq!(settings.len(), 11);
    assert!(settings.contains("plotheight") && settings.contains("Tol"));

    let consts = SnippetRegistry::get_constant_names();
    assert_eq!(consts.len(), 3);
    assert!(consts.contains("π") && consts.contains("pi") && consts.contains("e"));
    assert!(!consts.contains("E"));

    assert_eq!(SnippetRegistry::get_unit_names().len(), 405);
}

#[test]
fn operators_are_single_char_inserts() {
    let ops = SnippetRegistry::get_operators();
    assert_eq!(ops.len(), 21);
    for c in ['!', '^', '\\', '÷', '←', '=', '⊕'] {
        assert!(ops.contains(&c), "{c}");
    }
}

#[test]
fn overload_selection_prefers_accepts_any_count() {
    let ov = SnippetRegistry::get_overloads_for_function("TAKE");
    assert_eq!(ov.len(), 4);
    let by_name = SnippetRegistry::get_function_snippets_by_name();
    let take = by_name.get("take").unwrap();
    assert!(take.accepts_any_count);
    assert_eq!(take.min_params(), 0);
    assert_eq!(take.max_params(), -1);
    assert!(std::ptr::eq(*take, ov[0]));
    for name in ["line", "spline"] {
        assert_eq!(SnippetRegistry::get_overloads_for_function(name).len(), 4);
        assert!(by_name.get(name).unwrap().accepts_any_count);
    }
    for name in ["min", "max", "sum", "sumsq", "srss", "average", "product", "mean"] {
        assert_eq!(SnippetRegistry::get_overloads_for_function(name).len(), 3, "{name}");
    }
    assert!(SnippetRegistry::get_overloads_for_function("nonexistent").is_empty());
    assert_eq!(by_name.len(), SnippetRegistry::get_function_names().len());
}

#[test]
fn signature_helpers() {
    let sin = SnippetRegistry::get_function_snippets_by_name().get("SIN").unwrap();
    assert_eq!(sin.min_params(), 1);
    assert_eq!(sin.max_params(), 1);
    assert_eq!(sin.get_parameter_type(0), ParameterType::Scalar);
    assert_eq!(sin.get_parameter_type(5), ParameterType::Any);
    assert!(sin.is_element_wise);
    // Variadic: later indices use the variadic parameter's type.
    let line = SnippetRegistry::get_overloads_for_function("line")[0];
    let variadic = line
        .parameters
        .as_ref()
        .unwrap()
        .iter()
        .find(|p| p.is_variadic)
        .unwrap();
    assert_eq!(line.get_parameter_type(9), variadic.param_type);
}

#[test]
fn returning_function_sets() {
    let v = SnippetRegistry::get_vector_returning_functions();
    let m = SnippetRegistry::get_matrix_returning_functions();
    assert_eq!(v.len(), 29);
    assert_eq!(m.len(), 14);
    assert!(v.contains("CROSS") && v.contains("find_eq"));
}

#[test]
fn quick_type() {
    let alpha = symbol_snippets().iter().find(|s| s.insert == "α").unwrap();
    assert_eq!(alpha.quick_type.as_deref(), Some("a"));
    assert_eq!(symbol_snippets().iter().filter(|s| s.quick_type.is_some()).count(), 60);
    let ell = symbol_snippets().iter().find(|s| s.insert == "ℓ").unwrap();
    assert_eq!(ell.quick_type.as_deref(), Some("len"));
    assert_eq!(ell.category, "Symbols/Special");
}

#[test]
fn computed_css_strings_are_evaluated() {
    let entry = css_snippets()
        .iter()
        .find(|s| s.label.as_deref() == Some("UI Entry Box"))
        .unwrap();
    assert_eq!(
        entry.insert,
        "#val\n'<style>\n'  .calcpad-ui-input.highlight { background-color: #eeeeee; border: 1px solid #aaaaaa; }\n'</style>\n#end val"
    );
    let doc = entry.documentation.as_deref().unwrap();
    assert!(doc.starts_with("Targets `<input type=\"text\">` controls.\n\n"));
}

#[test]
fn extract_helpers() {
    assert_eq!(extract_function_name("sin(§)"), "sin");
    assert_eq!(extract_function_name(" f (x)"), "f");
    assert_eq!(extract_function_name("(x)"), "");
    assert_eq!(extract_function_name("pi"), "");
    assert_eq!(extract_keyword_name("#if §\n\t§\n#end if"), "#if");
    assert_eq!(extract_keyword_name("#end if"), "#end if");
    assert_eq!(extract_keyword_name("#End IF"), "#End if");
    assert_eq!(extract_keyword_name("#else if condition"), "#else if");
    assert_eq!(extract_keyword_name("#else"), "#else");
    // Faithful to C#: TrimStart drops the newline, so the next line's first word is taken.
    assert_eq!(extract_keyword_name("#else\n#end if"), "#else #end");
    assert_eq!(extract_keyword_name("#else §"), "#else");
    assert_eq!(extract_keyword_name("#md on"), "#md on");
    assert_eq!(extract_keyword_name("#round digits"), "#round");
    assert_eq!(extract_keyword_name("#loop"), "#loop");
    assert_eq!(extract_keyword_name("if"), "");
    assert_eq!(extract_command_name("$Root{§ @ § = § : §}"), "$Root");
    assert_eq!(extract_command_name("Root{"), "");
    assert_eq!(extract_command_name("$Block"), "$Block");
    assert_eq!(extract_setting_name("PlotHeight = §"), "PlotHeight");
    assert_eq!(extract_setting_name("  X  "), "X");
}

#[test]
fn snippets_json_shape() {
    let all: serde_json::Value = serde_json::from_str(&snippets_json(None)).unwrap();
    assert_eq!(all["count"], 970 + DIACRITICS);
    assert_eq!(all["snippets"].as_array().unwrap().len(), 970 + DIACRITICS);
    assert_eq!(snippets_json(Some("  ")), snippets_json(None));

    let trig: serde_json::Value = serde_json::from_str(&snippets_json(Some("functions/trig"))).unwrap();
    let first = &trig["snippets"][0];
    assert_eq!(first["insert"], "sin(§)");
    assert_eq!(first["isElementWise"], true);
    assert_eq!(first["acceptsAnyCount"], false);
    assert_eq!(first["quickType"], serde_json::Value::Null);
    assert_eq!(first["returnType"], serde_json::Value::Null);
    assert_eq!(first["parameters"][0]["name"], "x");
    assert_eq!(first["parameters"][0]["type"], "Scalar");
    assert_eq!(first["parameters"][0]["isOptional"], false);

    let funcs: serde_json::Value = serde_json::from_str(&snippets_json(Some("Functions"))).unwrap();
    let vectors = funcs["snippets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["returnType"] == "Vector");
    assert_eq!(vectors.count(), 29);
}

#[test]
fn snippets_json_uses_system_text_json_escaping() {
    let json = snippets_json(Some("Symbols/Special"));
    assert!(json.is_ascii());
    assert!(json.starts_with("{\"count\":"));
    assert!(json.contains("\"insert\":\"\\u2221\""));
    let all = snippets_json(None);
    assert!(all.contains("\\u0022type\\u0022"));
    // Round trip: strings survive the escaping.
    let v: serde_json::Value = serde_json::from_str(&all).unwrap();
    let ui = v["snippets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["insert"] == "#UI {\"type\": \"checkbox\"} b = 1");
    assert_eq!(ui.unwrap()["keywordType"], "Keyword");
}

#[test]
fn category_prefix_is_case_insensitive_and_prefix_based() {
    assert_eq!(SnippetRegistry::get_snippets_by_category("units").len(), 405);
    assert_eq!(SnippetRegistry::get_snippets_by_category("Symbols/Bar").len(), 52);
    assert!(SnippetRegistry::get_snippets_by_category("zzz").is_empty());
    assert_eq!(SnippetRegistry::get_snippets_by_category("").len(), 970 + DIACRITICS);
}
