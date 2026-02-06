//! Color Schemes - Re-exports from scala-chromatica
//!
//! **DEPRECATED:** This module will be removed in v0.1.7
//! 
//! Please update your imports:
//! ```rust
//! // OLD
//! use forma_fractalis::colorschemes::{Color, ColorMap};
//! 
//! // NEW
//! use scala_chromatica::{Color, ColorMap};
//! ```

#![deprecated(
    since = "0.1.61",
    note = "Use scala_chromatica crate directly. This module will be removed in v0.1.7"
)]

// Re-export everything from scala-chromatica
pub use scala_chromatica::{Color, ColorMap, ColorStop, color_from_iterations};

/// Application-specific color scheme enum
/// This remains in forma-fractalis as it's GUI-specific
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorScheme {
    Default,
    Fire,
    Ocean,
    Grayscale,
    Rainbow,
}

impl ColorScheme {
    pub const ALL: [ColorScheme; 5] = [
        ColorScheme::Default,
        ColorScheme::Fire,
        ColorScheme::Ocean,
        ColorScheme::Grayscale,
        ColorScheme::Rainbow,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ColorScheme::Default => "Default",
            ColorScheme::Fire => "Fire",
            ColorScheme::Ocean => "Ocean",
            ColorScheme::Grayscale => "Grayscale",
            ColorScheme::Rainbow => "Rainbow",
        }
    }

    pub fn to_colormap(&self) -> ColorMap {
        scala_chromatica::io::load_builtin_colormap(self.as_str())
            .expect("Built-in colormap should always load")
    }
}

