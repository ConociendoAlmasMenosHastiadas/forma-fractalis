//! Unified Rendering Pipeline
//!
//! This module provides a unified rendering system used by both preview and export.
//! It eliminates code duplication and ensures consistent behavior across all render paths.
//!
//! # Architecture
//! - `RenderConfig`: All parameters needed for rendering
//! - `RenderTarget`: Where the render is going (preview texture or export file)
//! - `render_with_config()`: Single entry point for all rendering
//!
//! # Usage
//! ```ignore
//! let config = RenderConfig::new(view, colormap, max_iterations);
//! let buffer = render_with_config(&config, RenderTarget::Preview);
//! ```

use crate::colorschemes::ColorMap;
use crate::fractal::MandelbrotView;
use crate::fractals::Fractal;
use crate::rendering::render_fractal;
use std::collections::HashMap;

/// Configuration for a render operation
#[derive(Clone)]
pub struct RenderConfig<'a> {
    pub view: MandelbrotView,
    pub colormap: &'a ColorMap,
    pub max_iterations: u32,
    pub use_period: bool,
    pub period: u32,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub use_log_scale: bool,
    pub fractal: &'a dyn Fractal,
    pub fractal_parameters: HashMap<String, f64>,
}

impl<'a> RenderConfig<'a> {
    /// Create a new render configuration
    pub fn new(
        view: MandelbrotView,
        colormap: &'a ColorMap,
        max_iterations: u32,
        fractal: &'a dyn Fractal,
    ) -> Self {
        Self {
            view,
            colormap,
            max_iterations,
            use_period: false,
            period: 256,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            fractal,
            fractal_parameters: HashMap::new(),
        }
    }

    /// Builder pattern: set period modulation
    pub fn with_period(mut self, enabled: bool, period: u32) -> Self {
        self.use_period = enabled;
        self.period = period;
        self
    }

    /// Builder pattern: set interior color
    pub fn with_interior_color(mut self, enabled: bool, color: [u8; 3]) -> Self {
        self.use_interior_color = enabled;
        self.interior_color = color;
        self
    }

    /// Builder pattern: set logarithmic scaling
    pub fn with_log_scale(mut self, enabled: bool) -> Self {
        self.use_log_scale = enabled;
        self
    }

    /// Builder pattern: set fractal parameters
    pub fn with_fractal_parameters(mut self, parameters: HashMap<String, f64>) -> Self {
        self.fractal_parameters = parameters;
        self
    }
}

/// Target for rendering output
#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    /// Render for preview display
    Preview,
    /// Render for export at specified dimensions
    Export { width: u32, height: u32 },
}

/// Unified rendering function used by both preview and export
///
/// # Arguments
/// * `config` - Rendering configuration
/// * `target` - Render target (preview or export with dimensions)
///
/// # Returns
/// RGBA buffer ready for use (texture upload or image encoding)
pub fn render_with_config(config: &RenderConfig, target: RenderTarget) -> Vec<u8> {
    // Determine output dimensions based on target
    let (width, height) = match target {
        RenderTarget::Preview => (config.view.width, config.view.height),
        RenderTarget::Export { width, height } => (width, height),
    };

    // Create view with target dimensions (may differ from config.view for export)
    let mut target_view = config.view.clone();
    target_view.width = width;
    target_view.height = height;

    // Allocate buffer
    let buffer_size = (width * height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];

    // Render using fractal trait
    render_fractal(
        &mut buffer,
        &target_view,
        config.colormap,
        config.max_iterations,
        config.use_period,
        config.period,
        config.use_interior_color,
        config.interior_color,
        config.use_log_scale,
        config.fractal,
        &config.fractal_parameters,
    );

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colorschemes::ColorMap;
    use crate::fractals::Mandelbrot;

    #[test]
    fn test_render_config_builder() {
        let view = MandelbrotView::new(100, 100);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();

        let config = RenderConfig::new(view, &colormap, 256, &mandelbrot)
            .with_period(true, 128)
            .with_interior_color(true, [255, 0, 0])
            .with_log_scale(true);

        assert!(config.use_period);
        assert_eq!(config.period, 128);
        assert!(config.use_interior_color);
        assert_eq!(config.interior_color, [255, 0, 0]);
        assert!(config.use_log_scale);
    }

    #[test]
    fn test_render_target_dimensions() {
        let view = MandelbrotView::new(640, 480);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();
        let config = RenderConfig::new(view, &colormap, 128, &mandelbrot);

        // Preview uses view dimensions
        let buffer = render_with_config(&config, RenderTarget::Preview);
        assert_eq!(buffer.len(), 640 * 480 * 4);

        // Export uses specified dimensions
        let buffer = render_with_config(&config, RenderTarget::Export { width: 1920, height: 1080 });
        assert_eq!(buffer.len(), 1920 * 1080 * 4);
    }
}
