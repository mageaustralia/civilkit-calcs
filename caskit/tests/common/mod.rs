#![allow(dead_code)]
use caskit::{parse, Expr};
use std::collections::HashMap;

pub fn p(s: &str) -> Expr {
    parse(s).unwrap_or_else(|e| panic!("parse {s:?}: {e}"))
}

/// Parse then simplify.
pub fn s(s: &str) -> Expr {
    p(s).simplify().unwrap()
}

pub fn vars(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

pub fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}
