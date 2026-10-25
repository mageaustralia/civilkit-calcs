//! The caskit symbolic algebra library behind one synchronous JSON call, for the Symbolic dialog of the web app.
//!
//! `run(op, expr, args)` takes the operation name, the expression (or equation) text and a JSON object of
//! arguments, and answers JSON text: `{"ok": true, "op": .., "ascii": .., "latex": .., "items": [..]}` or
//! `{"ok": false, "error": "text"}`. The error text is the crate's own message. Nothing is computed here that caskit
//! does not compute; this crate parses arguments, calls caskit and prints the result in two forms.
//!
//! Operations and their arguments:
//!
//! * `simplify`, `expand`, `factor`: none.
//! * `differentiate`: `var`, optional `order` (default 1).
//! * `integrate`: `var`, optional `lower` and `upper` (both or neither; expressions).
//! * `solve`: `var` (one name) for an equation, or `vars` (a list of names) for a linear system. `expr` holds one
//!   equation, or several separated by new lines or `;`. A text without `=` is taken as `expr = 0`.
//! * `substitute`: `map`, an object of name to expression text; the result is simplified.
//! * `evaluate`: `map`, an object of name to number (or constant expression); the result is a decimal number.
//!
//! `items` has one entry per result (the roots of an equation, the unknowns of a system, or the single result), each
//! with `ascii` and `latex`, and `name` when it is a value of an unknown. Under `wasm32` the release profile aborts on
//! a panic; the native build catches one and reports it as an error.

use caskit::{parse, parse_equation, solve, solve_linear_system, CasError, CasResult, Equation, Expr};
use serde_json::{json, Value};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Changes whenever the request or answer format does.
pub const ABI: &str = "calcpad-caskit-1";

#[wasm_bindgen]
pub fn abi() -> String {
    ABI.to_string()
}

/// One request, one answer (JSON text). Never throws.
#[wasm_bindgen]
pub fn run(op: &str, expr: &str, args: &str) -> String {
    let (op, expr, args) = (op.to_string(), expr.to_string(), args.to_string());
    let answer = std::panic::catch_unwind(move || match serde_json::from_str::<Value>(if args.trim().is_empty() { "{}" } else { &args }) {
        Err(e) => err(format!("unreadable arguments: {e}")),
        Ok(a) => match dispatch(&op, &expr, &a) {
            Ok(v) => v,
            Err(e) => err(e),
        },
    })
    .unwrap_or_else(|_| err("internal error while computing this expression".to_string()));
    answer.to_string()
}

fn err(msg: String) -> Value {
    json!({"ok": false, "error": msg})
}

fn cas<T>(r: CasResult<T>) -> Result<T, String> {
    r.map_err(|e: CasError| e.to_string())
}

fn item(e: &Expr, name: Option<&str>) -> Value {
    match name {
        Some(n) => json!({
            "name": n,
            "ascii": format!("{n} = {}", e.to_ascii()),
            "latex": format!("{n} = {}", e.to_latex()),
            "value_ascii": e.to_ascii(),
            "value_latex": e.to_latex(),
        }),
        None => json!({"ascii": e.to_ascii(), "latex": e.to_latex()}),
    }
}

fn done(op: &str, items: Vec<Value>) -> Value {
    let join = |k: &str, sep: &str| items.iter().filter_map(|i| i[k].as_str()).collect::<Vec<_>>().join(sep);
    let (ascii, latex) = (join("ascii", "; "), join("latex", r",\;\; "));
    json!({"ok": true, "op": op, "ascii": ascii, "latex": latex, "items": items})
}

fn single(op: &str, e: &Expr) -> Value {
    done(op, vec![item(e, None)])
}

fn text<'a>(a: &'a Value, key: &str) -> Option<&'a str> {
    a.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

fn var_of(a: &Value) -> Result<String, String> {
    let v = text(a, "var").ok_or("Name the variable.")?;
    if !v.chars().all(|c| c.is_alphanumeric() || c == '_') || v.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Err(format!("\"{v}\" is not a valid variable name."));
    }
    Ok(v.to_string())
}

fn dispatch(op: &str, src: &str, a: &Value) -> Result<Value, String> {
    if src.trim().is_empty() {
        return Err("Enter an expression first.".to_string());
    }
    match op {
        "simplify" => Ok(single(op, &cas(cas(parse(src))?.simplify())?)),
        "expand" => Ok(single(op, &cas(cas(parse(src))?.expand())?)),
        "factor" => Ok(single(op, &cas(cas(parse(src))?.factor())?)),
        "differentiate" => {
            let var = var_of(a)?;
            let order = a.get("order").and_then(Value::as_u64).unwrap_or(1);
            if !(1..=20).contains(&order) {
                return Err("The order must be a whole number from 1 to 20.".to_string());
            }
            let d = cas(cas(parse(src))?.diff_n(&var, order as usize))?;
            Ok(single(op, &cas(d.simplify())?))
        }
        "integrate" => {
            let var = var_of(a)?;
            let f = cas(parse(src))?;
            match (text(a, "lower"), text(a, "upper")) {
                (None, None) => Ok(single(op, &cas(f.integrate(&var))?)),
                (Some(lo), Some(hi)) => {
                    let (lo, hi) = (cas(parse(lo))?, cas(parse(hi))?);
                    Ok(single(op, &cas(f.integrate_definite(&var, &lo, &hi))?))
                }
                _ => Err("Give both limits for a definite integral, or neither.".to_string()),
            }
        }
        "solve" => solve_op(src, a),
        "substitute" => {
            let map = a.get("map").and_then(Value::as_object).ok_or("Give at least one substitution.")?;
            let mut pairs: Vec<(String, Expr)> = Vec::new();
            for (k, v) in map {
                let s = v.as_str().map(str::to_string).or_else(|| v.as_f64().map(|n| n.to_string())).unwrap_or_default();
                pairs.push((k.clone(), cas(parse(&s))?));
            }
            let refs: Vec<(&str, Expr)> = pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
            let r = cas(cas(parse(src))?.substitute_all(&refs))?;
            Ok(single(op, &cas(r.simplify())?))
        }
        "evaluate" => {
            let f = cas(parse(src))?;
            let mut vars: HashMap<String, f64> = HashMap::new();
            if let Some(map) = a.get("map").and_then(Value::as_object) {
                for (k, v) in map {
                    let n = match v {
                        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
                        Value::String(s) => cas(cas(parse(s))?.eval(&HashMap::new()))?,
                        _ => f64::NAN,
                    };
                    vars.insert(k.clone(), n);
                }
            }
            let x = cas(f.eval(&vars))?;
            if !x.is_finite() {
                return Err("The result is not a finite number.".to_string());
            }
            let s = number(x);
            Ok(done(op, vec![json!({"ascii": s, "latex": s})]))
        }
        other => Err(format!("unknown operation \"{other}\"")),
    }
}

/// 12 significant digits, no trailing zeros.
fn number(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    let s = format!("{:.11e}", x);
    let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
    let e: i32 = e.parse().unwrap_or(0);
    let m = m.trim_end_matches('0').trim_end_matches('.');
    if (-5..12).contains(&e) {
        let digits = (11 - e).max(0) as usize;
        let t = format!("{:.*}", digits, x);
        if t.contains('.') { t.trim_end_matches('0').trim_end_matches('.').to_string() } else { t }
    } else {
        format!("{m}e{e}")
    }
}

fn solve_op(src: &str, a: &Value) -> Result<Value, String> {
    let lines: Vec<&str> = src.split(|c| c == '\n' || c == ';').map(str::trim).filter(|s| !s.is_empty()).collect();
    let eqs: Vec<Equation> = lines
        .iter()
        .map(|l| if l.contains('=') { cas(parse_equation(l)) } else { cas(parse(l)).map(|e| Equation::new(e, Expr::int(0))) })
        .collect::<Result<_, _>>()?;
    let list: Vec<String> = match a.get("vars").and_then(Value::as_array) {
        Some(v) => v.iter().filter_map(Value::as_str).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        None => Vec::new(),
    };
    if eqs.len() > 1 || list.len() > 1 {
        if list.len() != eqs.len() {
            return Err(format!("Give one unknown per equation: {} equations and {} unknowns.", eqs.len(), list.len()));
        }
        let names: Vec<&str> = list.iter().map(String::as_str).collect();
        let sol = cas(solve_linear_system(&eqs, &names))?;
        let items = names.iter().zip(sol.iter()).map(|(n, e)| item(e, Some(n))).collect();
        return Ok(done("solve", items));
    }
    let var = match list.first() {
        Some(v) => v.clone(),
        None => var_of(a)?,
    };
    let roots = match eqs.first() {
        Some(eq) => cas(solve(eq, &var))?,
        None => return Err("Enter an equation first.".to_string()),
    };
    Ok(done("solve", roots.iter().map(|e| item(e, Some(&var))).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(op: &str, e: &str, a: &str) -> Value {
        serde_json::from_str(&run(op, e, a)).unwrap()
    }

    #[test]
    fn differentiate_expand_solve() {
        assert_eq!(r("differentiate", "w*x^4/24", r#"{"var":"x"}"#)["ascii"], "w*x^3/6");
        assert_eq!(r("expand", "(x-2)*(x-3)", "{}")["ascii"], "x^2 - 5*x + 6");
        let s = r("solve", "M = w*L^2/8", r#"{"var":"L"}"#);
        assert_eq!(s["items"].as_array().unwrap().len(), 2);
        assert!(s["latex"].as_str().unwrap().contains("L ="));
    }

    #[test]
    fn integrate_twice_with_limits() {
        let v = r("integrate", "w*L*x/2 - w*x^2/2", r#"{"var":"x","lower":"0","upper":"L"}"#);
        assert_eq!(v["ascii"], "L^3*w/12");
    }

    #[test]
    fn system_and_evaluate() {
        let v = r("solve", "x + y = 3\nx - y = 1", r#"{"vars":["x","y"]}"#);
        assert_eq!(v["items"][0]["value_ascii"], "2");
        assert_eq!(v["items"][1]["value_ascii"], "1");
        assert_eq!(r("evaluate", "x^2 + 1", r#"{"map":{"x":3}}"#)["ascii"], "10");
        assert_eq!(r("substitute", "a*x", r#"{"map":{"x":"2*b"}}"#)["ascii"], "2*a*b");
    }

    #[test]
    fn errors_are_structured() {
        let v = r("integrate", "sin(x)^2", r#"{"var":"x"}"#);
        assert_eq!(v["ok"], false);
        assert!(v["error"].as_str().unwrap().starts_with("cannot integrate"));
        assert_eq!(r("simplify", "2x", "{}")["ok"], false);
        assert_eq!(r("bogus", "x", "{}")["ok"], false);
        assert_eq!(r("integrate", "x", "not json")["ok"], false);
    }
}
