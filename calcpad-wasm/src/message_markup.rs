//! Marks the coded messages of a rendered page so that the web layer can show them in another language.
//!
//! The engine writes English and, when asked (`"messageCodes": true` in the request), records each message it makes from the
//! catalogue with its code and arguments (`calcpad_core::math_parser_exception::message_codes`). This module finds that text in
//! the page and wraps it in `<ck-msg code="..." args="...">text</ck-msg>`; the text inside is unchanged. The web layer
//! (`web/calcpad-web/src/i18n/engine-messages.ts`) either removes the wrapper again (English, byte for byte the page the engine
//! would have written) or replaces the text with the translated message. An error entry of the `errors` list gets `code` and
//! `args` the same way.
use calcpad_core::math_parser_exception::message_codes::Coded;
use serde_json::{json, Value as Json};

const OPEN: char = '\u{E000}';
const CLOSE: char = '\u{E001}';

/// The arguments of a message as JSON: a string, or, where the string is itself a coded message (the "Error in" frame holds
/// the message it frames), that message as `{"code": ..., "args": [...]}`.
fn args_json(c: &Coded, all: &[Coded], depth: usize) -> Json {
    Json::Array(
        c.args
            .iter()
            .map(|a| {
                if depth < 4 {
                    if let Some(inner) = all
                        .iter()
                        .find(|o| o.text == *a && !(o.code == c.code && o.args == c.args))
                    {
                        return json!({ "code": inner.code, "args": args_json(inner, all, depth + 1) });
                    }
                }
                Json::String(a.clone())
            })
            .collect(),
    )
}

fn attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Whether the end of `before` is inside an open tag (an attribute value, say): no wrapper goes there.
fn inside_tag(before: &str) -> bool {
    match (before.rfind('<'), before.rfind('>')) {
        (Some(l), Some(g)) => l > g,
        (Some(_), None) => true,
        _ => false,
    }
}

/// `html` with each coded message wrapped in `<ck-msg>`. Longer texts first, so a frame that holds a message is wrapped as a
/// whole and the message inside it is not wrapped a second time.
pub fn annotate(html: &str, coded: &[Coded]) -> String {
    if coded.is_empty() {
        return html.to_string();
    }
    let mut order: Vec<usize> = (0..coded.len()).filter(|&i| !coded[i].text.is_empty()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(coded[i].text.len()));
    let mut out = html.to_string();
    for &i in &order {
        let text = &coded[i].text;
        let mut res = String::with_capacity(out.len());
        let mut last = 0;
        let mut from = 0;
        while let Some(pos) = out[from..].find(text.as_str()) {
            let start = from + pos;
            let end = start + text.len();
            if inside_tag(&out[..start]) {
                from = start + out[start..].chars().next().map_or(1, char::len_utf8);
                continue;
            }
            res.push_str(&out[last..start]);
            res.push(OPEN);
            res.push_str(&i.to_string());
            res.push(CLOSE);
            last = end;
            from = end;
        }
        res.push_str(&out[last..]);
        out = res;
    }
    let mut res = String::with_capacity(out.len() + 64);
    let mut rest = out.as_str();
    while let Some(o) = rest.find(OPEN) {
        res.push_str(&rest[..o]);
        let after = &rest[o + OPEN.len_utf8()..];
        let c = after.find(CLOSE).unwrap_or(0);
        let idx: usize = after[..c].parse().unwrap_or(0);
        let m = &coded[idx];
        res.push_str(&format!(
            "<ck-msg code=\"{}\" args=\"{}\">{}</ck-msg>",
            m.code,
            attr(&args_json(m, coded, 0).to_string()),
            m.text
        ));
        rest = &after[c + CLOSE.len_utf8()..];
    }
    res.push_str(rest);
    res
}

/// The `code` and `args` to add to the error entry whose message is `message`.
pub fn for_error(message: &str, coded: &[Coded]) -> Option<(&'static str, Json)> {
    coded
        .iter()
        .find(|c| c.text == message)
        .map(|c| (c.code, args_json(c, coded, 0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(text: &str, code: &'static str, args: &[&str]) -> Coded {
        Coded {
            text: text.to_string(),
            code,
            args: args.iter().map(|a| a.to_string()).collect(),
        }
    }

    #[test]
    fn wraps_a_message_and_keeps_its_text() {
        let coded = [c("Variable x does not exist.", "Variable_0_does_not_exist", &["x"])];
        let out = annotate("<p class=\"err\">Variable x does not exist.</p>", &coded);
        assert_eq!(
            out,
            "<p class=\"err\"><ck-msg code=\"Variable_0_does_not_exist\" args=\"[&quot;x&quot;]\">Variable x does not exist.</ck-msg></p>"
        );
    }

    #[test]
    fn a_frame_is_wrapped_whole_and_names_the_message_it_holds() {
        let inner = c("Division by zero.", "Division_by_zero", &[]);
        let frame = c(
            "Error in \"a\": Division by zero.",
            "Error_in_0_1",
            &["a", "Division by zero."],
        );
        let out = annotate("<p>Error in \"a\": Division by zero.</p>", &[inner, frame]);
        assert_eq!(out.matches("<ck-msg").count(), 1, "{out}");
        assert!(
            out.contains(
                "args=\"[&quot;a&quot;,{&quot;code&quot;:&quot;Division_by_zero&quot;,&quot;args&quot;:[]}]\""
            ),
            "{out}"
        );
    }

    #[test]
    fn text_inside_a_tag_is_left_alone() {
        let coded = [c("Unit m does not exist.", "Unit_0_does_not_exist", &["m"])];
        let html = "<p title=\"Unit m does not exist.\">Unit m does not exist.</p>";
        let out = annotate(html, &coded);
        assert!(out.starts_with("<p title=\"Unit m does not exist.\">"), "{out}");
        assert_eq!(out.matches("<ck-msg").count(), 1);
    }

    #[test]
    fn nothing_noted_changes_nothing() {
        assert_eq!(annotate("<p>x</p>", &[]), "<p>x</p>");
    }
}
