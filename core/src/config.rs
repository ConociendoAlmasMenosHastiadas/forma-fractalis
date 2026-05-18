//! Configuration types for fractal rendering
//!
//! These types provide a clean, GUI-independent API for configuring fractal
//! renders. Library users work with these types rather than the GUI-coupled
//! state structures in `gui/src/app_state.rs`.
//!
//! # Workflow
//!
//! 1. Build a [`FractalConfig`] using constructors and builder methods.
//! 2. Pass it to [`crate::render_fractal_to_buffer`] for a one-shot render, or
//!    to [`crate::compute_fractal_iterations`] + [`crate::colorize_iterations`]
//!    for the two-phase path (useful for batch/animation work).
//! 3. Optionally pass to [`crate::export::export_png_with_config`] to write a PNG
//!    with embedded metadata (re-loadable via [`FractalConfig::from_metadata`]).
//!
//! # Example
//!
//! ```ignore
//! use forma_fractalis_core::{FractalConfig, render_fractal_to_buffer};
//! use forma_fractalis_core::fractals::{Julia, FractalView};
//!
//! let fractal = Julia::new();
//!
//! // Specify a deep-zoom view, Julia c constant, and exponent
//! let view = FractalView::new(3840, 2160)
//!     .with_center(-0.7, 0.27015)
//!     .with_zoom(50.0);
//!
//! let config = FractalConfig::new(view, 1024)
//!     .with_parameter("c_real", -0.7)
//!     .with_parameter("c_imag", 0.27015)
//!     .with_parameter("power", 2.0)
//!     .with_period(128)
//!     .with_interior_color([10, 10, 30])
//!     .with_log_scale();
//!
//! let rgba = render_fractal_to_buffer(&fractal, &config).unwrap();
//! ```

use crate::filtering::FilterType;
use crate::fractals::FractalView;
use scala_chromatica::ColorMap;
use std::collections::HashMap;

/// Complete configuration for rendering a fractal
#[derive(Clone, Debug)]
pub struct FractalConfig {
    /// View parameters (center, zoom, dimensions)
    pub view: FractalView,
    /// Maximum iterations for fractal calculation
    pub max_iterations: u32,
    /// Fractal-specific parameters (e.g., Julia c values)
    pub fractal_parameters: HashMap<String, f64>,
    /// Color mapping configuration
    pub color_config: ColorConfig,
}

/// Color mapping and modulation settings
#[derive(Clone, Debug)]
pub struct ColorConfig {
    /// The colormap to use for rendering
    pub colormap: ColorMap,
    /// Enable period modulation
    pub use_period: bool,
    /// Period value for modulo operation
    pub period: u32,
    /// Use custom interior color
    pub use_interior_color: bool,
    /// RGB color for points inside the set
    pub interior_color: [u8; 3],
    /// Apply logarithmic scaling
    pub use_log_scale: bool,
    /// Phase offset added to each pixel's iteration count before colormap lookup.
    /// Effective only when `use_period = true`. Incrementing this each frame
    /// produces a "color roll" animation without recomputing the fractal.
    pub color_offset: u32,
}

impl ColorConfig {
    /// Builder method: Set the color phase offset and return a new `ColorConfig`.
    ///
    /// This is the key method for color animation. Clone the current config, call
    /// `with_color_offset(frame_index)` each frame, and pass it to
    /// `colorize_iterations()` to roll the colormap without re-rendering the fractal.
    ///
    /// Has no visual effect when `use_period` is `false`.
    pub fn with_color_offset(mut self, offset: u32) -> Self {
        self.color_offset = offset;
        self
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            colormap: ColorMap::default_scheme(),
            use_period: false,
            period: 128,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            color_offset: 0,
        }
    }
}

/// Export-specific configuration
#[derive(Clone, Debug)]
pub struct ExportConfig {
    /// Scaling factor for output dimensions
    pub scale: f32,
    /// Supersampling factor (1 = no supersampling)
    pub supersample: u32,
    /// Image filtering type
    pub filter_type: FilterType,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            scale: 3.0,
            supersample: 4,
            filter_type: FilterType::Lanczos3,
        }
    }
}

impl FractalConfig {
    /// Create a new configuration with default color settings
    pub fn new(view: FractalView, max_iterations: u32) -> Self {
        Self {
            view,
            max_iterations,
            fractal_parameters: HashMap::new(),
            color_config: ColorConfig::default(),
        }
    }

    /// Builder method: Set colormap
    pub fn with_colormap(mut self, colormap: ColorMap) -> Self {
        self.color_config.colormap = colormap;
        self
    }

    /// Builder method: Set a fractal parameter
    pub fn with_parameter(mut self, name: impl Into<String>, value: f64) -> Self {
        self.fractal_parameters.insert(name.into(), value);
        self
    }

    /// Builder method: Enable period modulation
    pub fn with_period(mut self, period: u32) -> Self {
        self.color_config.use_period = true;
        self.color_config.period = period;
        self
    }

    /// Builder method: Set interior color
    pub fn with_interior_color(mut self, color: [u8; 3]) -> Self {
        self.color_config.use_interior_color = true;
        self.color_config.interior_color = color;
        self
    }

    /// Builder method: Enable log scale
    pub fn with_log_scale(mut self) -> Self {
        self.color_config.use_log_scale = true;
        self
    }

    /// Builder method: Set color phase offset for animation
    pub fn with_color_offset(mut self, offset: u32) -> Self {
        self.color_config.color_offset = offset;
        self
    }

    /// Convenience constructor: create a config for a headless render at given dimensions.
    ///
    /// Equivalent to `FractalConfig::new(FractalView::new(width, height), max_iterations)`.
    pub fn headless(width: u32, height: u32, max_iterations: u32) -> Self {
        Self::new(crate::fractals::FractalView::new(width, height), max_iterations)
    }

    /// Build a `FractalConfig` from a `FractalMetadata` struct loaded from a PNG or JSON file.
    ///
    /// This allows round-tripping: export a fractal, load the metadata back, and re-render
    /// with identical settings.
    ///
    /// # Errors
    /// Returns `Err` if the colormap stored in the metadata is missing required fields.
    pub fn from_metadata(metadata: &crate::export::FractalMetadata) -> Self {
        let view = crate::fractals::FractalView {
            center_x: metadata.center_x,
            center_y: metadata.center_y,
            zoom: metadata.zoom,
            precise_center_x: metadata.precise_center_x.clone(),
            precise_center_y: metadata.precise_center_y.clone(),
            precise_zoom: metadata.precise_zoom.clone(),
            width: metadata.width,
            height: metadata.height,
            parameters: metadata.fractal_parameters.clone(),
        };

        Self {
            view,
            max_iterations: metadata.max_iterations,
            fractal_parameters: metadata.fractal_parameters.clone(),
            color_config: ColorConfig {
                colormap: metadata.colormap_data.clone(),
                use_period: metadata.use_period,
                period: metadata.period,
                use_interior_color: metadata.use_interior_color,
                interior_color: metadata.interior_color,
                use_log_scale: metadata.use_log_scale,
                color_offset: metadata.color_offset,
            },
        }
    }
}
