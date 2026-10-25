//! SettingsDto from Calcpad.Core/Settings.cs: the JSON payload of `#settings {...}`.
use super::directive_dto::*;
use calcpad_core::settings::ColorScales;
use serde_json::Value;

/// The recognised keys, in declaration order.
pub const SETTING_KEY_NAMES: [&str; 21] = [
    "Decimals",
    "Degrees",
    "Complex",
    "Substitute",
    "FormatEquations",
    "ZeroSmallMatrixElements",
    "ShowHiddenOutput",
    "MaxOutputCount",
    "IsUs",
    "VectorGraphics",
    "ColorScale",
    "SmoothScale",
    "Shadows",
    "AdaptivePlot",
    "PlotWidth",
    "PlotHeight",
    "PlotStep",
    "Precision",
    "Tol",
    "ValMathml",
    "UnitPerElement",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingKey {
    Decimals,
    Degrees,
    Complex,
    Substitute,
    FormatEquations,
    ZeroSmallMatrixElements,
    ShowHiddenOutput,
    MaxOutputCount,
    IsUs,
    VectorGraphics,
    ColorScale,
    SmoothScale,
    Shadows,
    AdaptivePlot,
    PlotWidth,
    PlotHeight,
    PlotStep,
    Precision,
    Tol,
    ValMathml,
    UnitPerElement,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsDto {
    pub decimals: Option<i32>,
    pub degrees: Option<i32>,
    pub complex: Option<bool>,
    pub substitute: Option<bool>,
    pub format_equations: Option<bool>,
    pub zero_small_matrix_elements: Option<bool>,
    pub show_hidden_output: Option<bool>,
    pub max_output_count: Option<i32>,
    pub is_us: Option<bool>,
    pub vector_graphics: Option<bool>,
    pub color_scale: Option<String>,
    pub smooth_scale: Option<bool>,
    pub shadows: Option<bool>,
    pub adaptive_plot: Option<bool>,
    pub plot_width: Option<i32>,
    pub plot_height: Option<i32>,
    pub plot_step: Option<i32>,
    pub precision: Option<f64>,
    pub tol: Option<f64>,
    /// A civilkit extension (see DEVIATIONS.md): `#val` writes a result as MathML.
    pub val_mathml: Option<bool>,
    /// A civilkit extension (see DEVIATIONS.md): the unit on every element of a vector or matrix.
    pub unit_per_element: Option<bool>,
}

impl SettingsDto {
    /// `KnownKeys.Contains(name)`.
    pub fn is_known_key(name: &str) -> bool {
        is_known_key(&SETTING_KEY_NAMES, name)
    }

    /// `DirectiveDto.Parse`: `Ok(None)` for a JSON `null`, `Err` for malformed JSON or a wrong value type.
    pub fn parse(json: &str) -> Result<Option<SettingsDto>, JsonError> {
        match root_object(json)? {
            None => Ok(None),
            Some(obj) => Self::from_value(&Value::Object(obj)).map(Some),
        }
    }

    fn from_value(v: &Value) -> Result<SettingsDto, JsonError> {
        let obj = v.as_object().ok_or(JsonError)?;
        let mut d = SettingsDto::default();
        for (k, v) in obj.iter() {
            match k.to_ascii_lowercase().as_str() {
                "decimals" => d.decimals = get_int(v)?,
                "degrees" => d.degrees = get_int(v)?,
                "complex" => d.complex = get_bool(v)?,
                "substitute" => d.substitute = get_bool(v)?,
                "formatequations" => d.format_equations = get_bool(v)?,
                "zerosmallmatrixelements" => d.zero_small_matrix_elements = get_bool(v)?,
                "showhiddenoutput" => d.show_hidden_output = get_bool(v)?,
                "maxoutputcount" => d.max_output_count = get_int(v)?,
                "isus" => d.is_us = get_bool(v)?,
                "vectorgraphics" => d.vector_graphics = get_bool(v)?,
                "colorscale" => d.color_scale = get_str(v)?,
                "smoothscale" => d.smooth_scale = get_bool(v)?,
                "shadows" => d.shadows = get_bool(v)?,
                "adaptiveplot" => d.adaptive_plot = get_bool(v)?,
                "plotwidth" => d.plot_width = get_int(v)?,
                "plotheight" => d.plot_height = get_int(v)?,
                "plotstep" => d.plot_step = get_int(v)?,
                "precision" => d.precision = get_double(v)?,
                "tol" => d.tol = get_double(v)?,
                "valmathml" => d.val_mathml = get_bool(v)?,
                "unitperelement" => d.unit_per_element = get_bool(v)?,
                _ => {}
            }
        }
        Ok(d)
    }

    /// An entry for every supplied value that the directive rejects.
    pub fn validate(&self) -> Vec<DirectiveError<SettingKey>> {
        let mut errors = Vec::new();
        let f = |v: Option<i32>| v.map(f64::from);
        check_range(
            &mut errors,
            SettingKey::Decimals,
            "decimals",
            f(self.decimals),
            0.0,
            Some(15.0),
        );
        check_range(
            &mut errors,
            SettingKey::Degrees,
            "degrees",
            f(self.degrees),
            0.0,
            Some(2.0),
        );
        check_range(
            &mut errors,
            SettingKey::MaxOutputCount,
            "maxOutputCount",
            f(self.max_output_count),
            5.0,
            Some(100.0),
        );
        check_range(
            &mut errors,
            SettingKey::PlotWidth,
            "plotWidth",
            f(self.plot_width),
            1.0,
            None,
        );
        check_range(
            &mut errors,
            SettingKey::PlotHeight,
            "plotHeight",
            f(self.plot_height),
            1.0,
            None,
        );
        check_range(
            &mut errors,
            SettingKey::PlotStep,
            "plotStep",
            f(self.plot_step),
            0.0,
            None,
        );
        check_range(
            &mut errors,
            SettingKey::Precision,
            "precision",
            self.precision,
            1e-15,
            Some(1e-2),
        );
        check_range(&mut errors, SettingKey::Tol, "tol", self.tol, 1e-15, Some(1e-2));
        if let Some(cs) = &self.color_scale {
            let names = ColorScales::names();
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            if !enum_try_parse(&refs, cs) {
                errors.push(DirectiveError {
                    key: SettingKey::ColorScale,
                    message: format!("'colorScale' must be one of: {}", names.join(", ")),
                });
            }
        }
        errors
    }
}
