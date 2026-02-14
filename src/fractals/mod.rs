//! Fractal Trait System
//!
//! This module provides a unified trait-based system for different fractal types.
//! Each fractal implements the `Fractal` trait, which defines how to compute
//! iterations and provides metadata for GUI generation.
//!
//! # Architecture
//! - `Fractal` trait: Core interface for all fractal types
//! - `FractalView`: Viewport state with optional fractal-specific parameters
//! - `Parameter`: Metadata for dynamic GUI generation
//!
//! # Example
//! ```ignore
//! let mandelbrot = Mandelbrot::new();
//! let view = mandelbrot.default_view();
//! let iterations = mandelbrot.iterate(c.re, c.im, &view.parameters, 256);
//! ```

use std::collections::HashMap;

// Fractal implementations
pub mod mandelbrot;
pub mod julia;
pub mod burning_ship;
pub mod tippets_mandelbrot;
pub mod multifractal_julia;
pub mod cactus;
pub mod marek_dragon;

// Re-export for convenience
pub use mandelbrot::Mandelbrot;
pub use julia::Julia;
pub use burning_ship::BurningShip;
pub use tippets_mandelbrot::TippetsMandelbrot;
pub use multifractal_julia::MultifractalJulia;
pub use cactus::Cactus;
pub use marek_dragon::MarekDragon;

/// Represents the view parameters for rendering any fractal
/// This replaces the old MandelbrotView with a more generic structure
#[derive(Clone)]
pub struct FractalView {
    /// Center X coordinate in the complex plane
    pub center_x: f64,
    /// Center Y coordinate in the complex plane
    pub center_y: f64,
    /// Zoom level (higher = more zoomed in)
    pub zoom: f64,
    /// Width of the viewport in pixels
    pub width: u32,
    /// Height of the viewport in pixels
    pub height: u32,
    /// Fractal-specific parameters (e.g., Julia set constants)
    pub parameters: HashMap<String, f64>,
}

impl FractalView {
    /// Creates a new FractalView with specified dimensions and default coordinates
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width,
            height,
            parameters: HashMap::new(),
        }
    }

    /// Converts screen pixel coordinates to complex plane coordinates
    ///
    /// # Arguments
    /// * `x` - Pixel x-coordinate
    /// * `y` - Pixel y-coordinate
    ///
    /// # Returns
    /// A tuple (real, imaginary) representing the complex number
    pub fn screen_to_complex(&self, x: u32, y: u32) -> (f64, f64) {
        let aspect_ratio = self.width as f64 / self.height as f64;
        let scale = 3.5 / self.zoom;

        let real = self.center_x
            + (x as f64 - self.width as f64 / 2.0) * scale / self.width as f64 * aspect_ratio;
        let imag =
            self.center_y + (y as f64 - self.height as f64 / 2.0) * scale / self.height as f64;

        (real, imag)
    }

    /// Pans the view in the given direction
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let pan_amount = 0.1 / self.zoom;
        self.center_x += dx * pan_amount;
        self.center_y += dy * pan_amount;
    }

    /// Zooms in at a specific point on the screen
    pub fn zoom_at(&mut self, screen_x: u32, screen_y: u32, zoom_factor: f64) {
        let (click_real, click_imag) = self.screen_to_complex(screen_x, screen_y);
        self.center_x = click_real;
        self.center_y = click_imag;
        self.zoom *= zoom_factor;
    }

    /// Sets a fractal-specific parameter
    pub fn set_parameter(&mut self, name: &str, value: f64) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Gets a fractal-specific parameter, or None if not set
    pub fn get_parameter(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).copied()
    }

    /// Resets the view to default coordinates (centered at origin, zoom 1.0)
    /// Note: This does not reset fractal-specific parameters
    pub fn reset(&mut self) {
        self.center_x = 0.0;
        self.center_y = 0.0;
        self.zoom = 1.0;
        // Note: parameters are intentionally not cleared to preserve fractal settings
    }
}

/// Metadata for a fractal parameter (used for GUI generation)
#[derive(Clone, Debug)]
pub struct Parameter {
    /// Internal parameter name (e.g., "c_real")
    pub name: String,
    /// Display label for GUI (e.g., "C Real Part")
    pub label: String,
    /// Default value
    pub default: f64,
    /// Minimum allowed value
    pub min: f64,
    /// Maximum allowed value
    pub max: f64,
    /// Description/tooltip text
    pub description: String,
}

impl Parameter {
    /// Creates a new parameter with the given properties
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        default: f64,
        min: f64,
        max: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            default,
            min,
            max,
            description: description.into(),
        }
    }
}

/// Core trait that all fractals must implement
pub trait Fractal: Sync {
    /// Compute the number of iterations before divergence
    ///
    /// # Arguments
    /// * `c_real` - Real part of the complex number
    /// * `c_imag` - Imaginary part of the complex number
    /// * `parameters` - Fractal-specific parameters
    /// * `max_iter` - Maximum number of iterations to test
    ///
    /// # Returns
    /// Number of iterations before divergence, or max_iter if in the set
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32;

    /// Get the default view settings for this fractal
    /// Each fractal type has its own ideal starting position and zoom
    fn default_view(&self, width: u32, height: u32) -> FractalView;

    /// Get the name of this fractal type
    fn name(&self) -> &str;

    /// Get the list of parameters this fractal uses
    /// Returns empty vec for fractals with no parameters (like Mandelbrot)
    fn parameters(&self) -> Vec<Parameter> {
        Vec::new()
    }

    /// Get the default values for all parameters
    fn parameter_defaults(&self) -> HashMap<String, f64> {
        self.parameters()
            .iter()
            .map(|p| (p.name.clone(), p.default))
            .collect()
    }
}
