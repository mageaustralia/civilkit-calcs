//! Port of Calcpad.Core/UiDto.cs and DirectiveDto.cs: the JSON payload of `#UI {...}`.
//!
//! System.Text.Json is emulated with serde_json: property names match case insensitively,
//! unknown properties are ignored, and a value of the wrong JSON type is an error (C#
//! `JsonException`), as is a non-integer number for an integer property.
use serde_json::Value as Json;

use crate::messages;
use crate::netfmt::fmt;

/// Most cells a datagrid may hold before it is rejected as too large.
pub const MAX_SIZE: i64 = 100_000;

pub const KNOWN_TYPES: [&str; 5] = ["entry", "datagrid", "dropdown", "radio", "checkbox"];

/// The value of `width` as written (a JSON number or string), kept like `JsonElement`.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonWidth {
    Number(f64),
    String(String),
    Other,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiDto {
    pub ty: Option<String>,
    pub mode: Option<String>,
    pub style: Option<String>,
    pub report_style: Option<String>,
    pub force_units: Option<bool>,
    pub allow_expression: Option<bool>,
    pub rows: Option<i32>,
    pub columns: Option<i32>,
    pub column_headers: Option<Vec<Option<String>>>,
    pub row_headers: Option<Vec<Option<String>>>,
    pub width: Option<JsonWidth>,
    pub row_header_width: Option<i32>,
    pub column_widths: Option<Vec<i32>>,
    pub keys: Option<Vec<Option<String>>>,
    pub values: Option<Vec<Option<String>>>,
    /// An addition to the C# payload: the bounds and the step of a number entry, in the entry's own
    /// unit. The form draws a slider beside the box when `min` and `max` are given (DEVIATIONS.md, D16).
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

/// The JSON could not be read (C# `JsonException`).
#[derive(Debug)]
pub struct JsonError;

fn get_str(v: &Json) -> Result<Option<String>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::String(s) => Ok(Some(s.clone())),
        _ => Err(JsonError),
    }
}

fn get_bool(v: &Json) -> Result<Option<bool>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::Bool(b) => Ok(Some(*b)),
        _ => Err(JsonError),
    }
}

fn get_int(v: &Json) -> Result<Option<i32>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::Number(n) => match n.as_i64() {
            Some(i) if i >= i32::MIN as i64 && i <= i32::MAX as i64 => Ok(Some(i as i32)),
            _ => Err(JsonError),
        },
        _ => Err(JsonError),
    }
}

fn get_f64(v: &Json) -> Result<Option<f64>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::Number(n) => n.as_f64().map(Some).ok_or(JsonError),
        _ => Err(JsonError),
    }
}

fn get_str_array(v: &Json) -> Result<Option<Vec<Option<String>>>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::Array(a) => a.iter().map(get_str).collect::<Result<Vec<_>, _>>().map(Some),
        _ => Err(JsonError),
    }
}

fn get_int_array(v: &Json) -> Result<Option<Vec<i32>>, JsonError> {
    match v {
        Json::Null => Ok(None),
        Json::Array(a) => a
            .iter()
            .map(|x| get_int(x)?.ok_or(JsonError))
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Err(JsonError),
    }
}

impl UiDto {
    /// `DirectiveDto.Parse`: None for a JSON `null`.
    pub fn parse(json: &str) -> Result<Option<UiDto>, JsonError> {
        let v: Json = serde_json::from_str(json).map_err(|_| JsonError)?;
        let obj = match v {
            Json::Null => return Ok(None),
            Json::Object(o) => o,
            _ => return Err(JsonError),
        };
        let mut d = UiDto::default();
        for (k, v) in obj.iter() {
            match k.to_ascii_lowercase().as_str() {
                "type" => d.ty = get_str(v)?,
                "mode" => d.mode = get_str(v)?,
                "style" => d.style = get_str(v)?,
                "reportstyle" => d.report_style = get_str(v)?,
                "forceunits" => d.force_units = get_bool(v)?,
                "allowexpression" => d.allow_expression = get_bool(v)?,
                "rows" => d.rows = get_int(v)?,
                "columns" => d.columns = get_int(v)?,
                "columnheaders" => d.column_headers = get_str_array(v)?,
                "rowheaders" => d.row_headers = get_str_array(v)?,
                "width" => {
                    d.width = match v {
                        Json::Null => None,
                        Json::Number(n) => Some(JsonWidth::Number(n.as_f64().unwrap_or(f64::NAN))),
                        Json::String(s) => Some(JsonWidth::String(s.clone())),
                        _ => Some(JsonWidth::Other),
                    }
                }
                "rowheaderwidth" => d.row_header_width = get_int(v)?,
                "columnwidths" => d.column_widths = get_int_array(v)?,
                "keys" => d.keys = get_str_array(v)?,
                "values" => d.values = get_str_array(v)?,
                "min" => d.min = get_f64(v)?,
                "max" => d.max = get_f64(v)?,
                "step" => d.step = get_f64(v)?,
                _ => {}
            }
        }
        Ok(Some(d))
    }

    pub fn has_options(&self) -> bool {
        matches!(self.ty.as_deref(), Some("dropdown" | "radio"))
    }

    /// An expression takes no appended unit, so allowExpression settles this.
    pub fn keeps_units(&self) -> bool {
        self.allow_expression != Some(true) && self.force_units != Some(false)
    }

    pub fn allows_expression(&self) -> bool {
        self.allow_expression == Some(true)
    }

    /// The declared total width as a pixel count or a "75%" share of the line.
    pub fn get_width(&self) -> Option<String> {
        match &self.width {
            Some(JsonWidth::Number(d)) => Some(format!("{}", crate::netfmt::round_away(*d) as i64 as i32)),
            Some(JsonWidth::String(s)) => {
                try_get_percent(s).map(|p| format!("{}%", crate::netfmt::double_to_string(p)))
            }
            _ => None,
        }
    }

    /// `Validate()`: the message of every supplied value the directive rejects.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(t) = &self.ty {
            if !KNOWN_TYPES.contains(&t.as_str()) {
                errors.push(fmt(
                    messages::The_UI_type_0_is_not_recognized_expected_one_of_1,
                    &[t, &KNOWN_TYPES.join(", ")],
                ));
            }
        }
        if let Some(m) = &self.mode {
            if !m.eq_ignore_ascii_case("number") {
                errors.push(messages::Only_numbers_are_supported_by_the_UI_keyword.to_string());
            }
        }
        let ty = self.ty.as_deref();
        if let Some(t @ ("dropdown" | "radio" | "checkbox")) = ty {
            if self.force_units.is_some() {
                errors.push(fmt(messages::The_UI_0_does_not_apply_to_1, &["forceUnits", t]));
            }
            if self.allow_expression.is_some() {
                errors.push(fmt(messages::The_UI_0_does_not_apply_to_1, &["allowExpression", t]));
            }
        }
        if matches!(ty, Some("entry" | "dropdown" | "radio" | "checkbox")) {
            self.validate_grid_properties(&mut errors);
        }
        self.validate_range(&mut errors);
        check_not_negative(&mut errors, "rows", self.rows);
        check_not_negative(&mut errors, "columns", self.columns);
        check_not_negative(&mut errors, "rowHeaderWidth", self.row_header_width);
        self.validate_width(&mut errors);
        if self.has_options() {
            let t = ty.unwrap_or_default();
            match (&self.keys, &self.values) {
                (Some(k), Some(v)) => {
                    if k.len() != v.len() {
                        errors.push(fmt(
                            messages::The_UI_0_keys_and_values_arrays_must_have_the_same_length,
                            &[t, &k.len().to_string(), &v.len().to_string()],
                        ));
                    }
                }
                _ => errors.push(fmt(messages::The_UI_0_requires_both_keys_and_values_arrays, &[t])),
            }
        }
        check_header_count(
            &mut errors,
            "columnHeaders",
            self.column_headers.as_ref().map(|a| a.len()),
            self.columns,
            "columns",
        );
        check_header_count(
            &mut errors,
            "rowHeaders",
            self.row_headers.as_ref().map(|a| a.len()),
            self.rows,
            "rows",
        );
        check_header_count(
            &mut errors,
            "columnWidths",
            self.column_widths.as_ref().map(|a| a.len()),
            self.columns,
            "columns",
        );
        errors
    }

    /// The grid properties against the type the right hand side settled on.
    pub fn validate_resolved_type(&self, ty: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if self.ty.is_none() && ty != "datagrid" {
            self.validate_grid_properties(&mut errors);
        }
        errors
    }

    fn validate_grid_properties(&self, errors: &mut Vec<String>) {
        let grid_only = |errors: &mut Vec<String>, name: &str, present: bool| {
            if present {
                errors.push(fmt(messages::The_UI_0_only_applies_to_a_datagrid, &[name]));
            }
        };
        grid_only(errors, "rows", self.rows.is_some());
        grid_only(errors, "columns", self.columns.is_some());
        grid_only(errors, "columnHeaders", self.column_headers.is_some());
        grid_only(errors, "rowHeaders", self.row_headers.is_some());
        grid_only(errors, "width", self.width.is_some());
        grid_only(errors, "rowHeaderWidth", self.row_header_width.is_some());
        grid_only(errors, "columnWidths", self.column_widths.is_some());
    }

    /// `min`, `max` and `step` belong to a number entry; `min` must not be above `max` and `step`
    /// must be above zero.
    fn validate_range(&self, errors: &mut Vec<String>) {
        let ty = self.ty.as_deref();
        if !matches!(ty, None | Some("entry")) {
            for (name, present) in [("min", self.min.is_some()), ("max", self.max.is_some()), ("step", self.step.is_some())] {
                if present {
                    errors.push(fmt(messages::The_UI_0_only_applies_to_an_entry, &[name]));
                }
            }
            return;
        }
        if let (Some(lo), Some(hi)) = (self.min, self.max) {
            if lo > hi {
                errors.push(fmt(
                    messages::The_UI_min_must_not_be_above_max,
                    &[&crate::netfmt::double_to_string(lo), &crate::netfmt::double_to_string(hi)],
                ));
            }
        }
        if self.step.is_some_and(|s| s.is_nan() || s <= 0.0) {
            errors.push(messages::The_UI_step_must_be_above_zero.to_string());
        }
    }

    fn validate_width(&self, errors: &mut Vec<String>) {
        match &self.width {
            None => {}
            Some(JsonWidth::Number(d)) => {
                if *d < 0.0 {
                    errors.push(fmt(messages::The_UI_0_must_not_be_negative, &["width"]));
                }
            }
            Some(JsonWidth::String(s)) if try_get_percent(s).is_some() => {}
            Some(_) => errors.push(messages::The_UI_width_must_be_a_number_or_a_percentage.to_string()),
        }
    }
}

/// "full" is the spelled out form of 100%.
fn try_get_percent(s: &str) -> Option<f64> {
    let s = s.trim();
    if s == "full" {
        return Some(100.0);
    }
    let body = s.strip_suffix('%')?;
    let p = crate::netfmt::parse_invariant_float(body)?;
    if p > 0.0 {
        Some(p)
    } else {
        None
    }
}

fn check_not_negative(errors: &mut Vec<String>, name: &str, value: Option<i32>) {
    if value.is_some_and(|v| v < 0) {
        errors.push(fmt(messages::The_UI_0_must_not_be_negative, &[name]));
    }
}

fn check_header_count(errors: &mut Vec<String>, name: &str, count: Option<usize>, size: Option<i32>, size_name: &str) {
    let (Some(count), Some(size)) = (count, size) else {
        return;
    };
    if count as i64 <= size as i64 {
        return;
    }
    errors.push(fmt(
        messages::The_UI_0_has_1_entries_but_the_grid_has_2_3,
        &[name, &count.to_string(), &size.to_string(), size_name],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_insensitively_and_rejects_wrong_types() {
        let d = UiDto::parse(r#"{"Type":"dropdown","keys":["A","B"],"Values":["1","2"],"x":5}"#)
            .unwrap()
            .unwrap();
        assert_eq!(d.ty.as_deref(), Some("dropdown"));
        assert!(d.validate().is_empty());
        assert!(UiDto::parse(r#"{"rows":"3"}"#).is_err());
        assert!(UiDto::parse(r#"{"rows":3.0}"#).is_err());
        assert!(UiDto::parse(r#"{"rows":3,}"#).is_err());
    }

    #[test]
    fn widths() {
        let d = UiDto::parse(r#"{"width":"75%"}"#).unwrap().unwrap();
        assert_eq!(d.get_width().as_deref(), Some("75%"));
        let d = UiDto::parse(r#"{"width":"full"}"#).unwrap().unwrap();
        assert_eq!(d.get_width().as_deref(), Some("100%"));
        let d = UiDto::parse(r#"{"width":120.6}"#).unwrap().unwrap();
        assert_eq!(d.get_width().as_deref(), Some("121"));
    }
}
