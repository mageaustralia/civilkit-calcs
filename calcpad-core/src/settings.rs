//! Port of Settings.cs (the parts the engine uses).

#[derive(Clone, Debug)]
pub struct MathSettings {
    decimals: i32,
    pub degrees: i32,
    pub is_complex: bool,
    pub substitute: bool,
    pub format_equations: bool,
    pub zero_small_matrix_elements: bool,
    pub show_hidden_output: bool,
    /// Opt-in (`#settings {"valMathml": true}`): `#val` writes a result as MathML with its unit once
    /// after the brackets, for the worksheet importer's typeset results. Off by default.
    pub val_mathml: bool,
    /// `#settings {"unitPerElement": true}`: write the unit on every element of a vector or matrix whose
    /// elements share one unit, as the engine did before; off (the default) writes the unit once after
    /// the closing bracket.
    pub unit_per_element: bool,
    max_output_count: i32,
    pub format_string: Option<String>,
    pub precision: f64,
    pub tol: f64,
}

impl Default for MathSettings {
    fn default() -> Self {
        MathSettings {
            decimals: 2,
            degrees: 0,
            is_complex: false,
            substitute: true,
            format_equations: true,
            zero_small_matrix_elements: true,
            show_hidden_output: false,
            val_mathml: false,
            unit_per_element: false,
            max_output_count: 20,
            format_string: None,
            precision: 1e-14,
            tol: 1e-6,
        }
    }
}

impl MathSettings {
    pub fn decimals(&self) -> i32 {
        self.decimals
    }

    pub fn set_decimals(&mut self, value: i32) {
        self.decimals = value.clamp(0, 15);
    }

    pub fn max_output_count(&self) -> i32 {
        self.max_output_count
    }

    pub fn set_max_output_count(&mut self, value: i32) {
        self.max_output_count = value.clamp(5, 100);
    }

    /// Builder style helpers used by tests.
    pub fn with_decimals(mut self, d: i32) -> Self {
        self.set_decimals(d);
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LightDirections {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl LightDirections {
    pub const ALL: [LightDirections; 8] = [
        LightDirections::North,
        LightDirections::NorthEast,
        LightDirections::East,
        LightDirections::SouthEast,
        LightDirections::South,
        LightDirections::SouthWest,
        LightDirections::West,
        LightDirections::NorthWest,
    ];

    pub fn parse(s: &str) -> Option<LightDirections> {
        enum_parse(s, &LightDirections::ALL, |v| format!("{:?}", v))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ColorScales {
    None,
    Gray,
    Rainbow,
    Terrain,
    VioletToYellow,
    GreenToYellow,
    Blues,
    BlueToYellow,
    BlueToRed,
    PurpleToYellow,
}

impl ColorScales {
    pub const ALL: [ColorScales; 10] = [
        ColorScales::None,
        ColorScales::Gray,
        ColorScales::Rainbow,
        ColorScales::Terrain,
        ColorScales::VioletToYellow,
        ColorScales::GreenToYellow,
        ColorScales::Blues,
        ColorScales::BlueToYellow,
        ColorScales::BlueToRed,
        ColorScales::PurpleToYellow,
    ];

    /// `Enum.TryParse<ColorScales>(s, ignoreCase: true)`
    pub fn parse(s: &str) -> Option<ColorScales> {
        enum_parse(s, &ColorScales::ALL, |v| format!("{:?}", v))
    }

    pub fn names() -> Vec<String> {
        ColorScales::ALL.iter().map(|v| format!("{:?}", v)).collect()
    }
}

/// `Enum.TryParse(value, ignoreCase: true)`: a name or an integer value.
fn enum_parse<T: Copy>(s: &str, all: &[T], name: impl Fn(&T) -> String) -> Option<T> {
    let t = s.trim();
    if let Ok(i) = t.parse::<i64>() {
        return all.get(usize::try_from(i).ok()?).copied();
    }
    all.iter().find(|v| name(v).eq_ignore_ascii_case(t)).copied()
}

/// Port of `PlotSettings`.
#[derive(Clone, Debug)]
pub struct PlotSettings {
    pub is_adaptive: bool,
    pub width: i32,
    pub height: i32,
    pub step: i32,
    pub screen_scale_factor: f64,
    pub image_path: String,
    pub image_uri: String,
    pub vector_graphics: bool,
    pub color_scale: ColorScales,
    pub smooth_scale: bool,
    shadows: bool,
    pub light_direction: LightDirections,
}

impl PlotSettings {
    /// `Shadows` getter: `_shadows && ColorScale != Gray || ColorScale == None`
    pub fn shadows(&self) -> bool {
        self.shadows && self.color_scale != ColorScales::Gray || self.color_scale == ColorScales::None
    }

    pub fn set_shadows(&mut self, value: bool) {
        self.shadows = value;
    }
}

impl Default for PlotSettings {
    fn default() -> Self {
        PlotSettings {
            is_adaptive: true,
            width: 500,
            height: 300,
            step: 0,
            screen_scale_factor: 2.0,
            image_path: String::new(),
            image_uri: String::new(),
            vector_graphics: false,
            color_scale: ColorScales::Rainbow,
            smooth_scale: false,
            shadows: true,
            light_direction: LightDirections::NorthWest,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub math: MathSettings,
    pub plot: PlotSettings,
    pub is_us: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            math: MathSettings::default(),
            plot: PlotSettings::default(),
            is_us: true,
        }
    }
}
