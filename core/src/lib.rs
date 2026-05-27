//! # Forma Fractalis Core - High-Performance Fractal Rendering Library
//!
//! A framework-agnostic fractal computation and rendering library supporting
//! multiple fractal types with advanced color mapping and export capabilities.
//!
//! ## Quick Start
//!
//! Render a Mandelbrot set to a buffer and save as PNG:
//!
//! ```ignore
//! use forma_fractalis_core::{render_fractal_to_buffer, FractalConfig};
//! use forma_fractalis_core::fractals::Mandelbrot;
//!
//! let fractal = Mandelbrot::new();
//! let config = FractalConfig::headless(1920, 1080, 256);
//! let rgba_buffer = render_fractal_to_buffer(&fractal, &config).unwrap();
//! // rgba_buffer is width * height * 4 bytes, RGBA row-major
//! ```
//!
//! ## Batch Rendering
//!
//! Use `compute_fractal_iterations` + `colorize_iterations` when you need to
//! render the same geometry with different color settings. The separation avoids
//! recomputing the fractal (expensive, 10-200ms) for each color variant
//! (cheap, <1ms):
//!
//! ```ignore
//! use forma_fractalis_core::{compute_fractal_iterations, colorize_iterations, FractalConfig};
//! use forma_fractalis_core::fractals::Julia;
//!
//! let fractal = Julia::new();
//! let config = FractalConfig::headless(800, 600, 512)
//!     .with_parameter("c_real", -0.7)
//!     .with_parameter("c_imag", 0.27015)
//!     .with_parameter("power", 2.0)
//!     .with_period(64);
//!
//! // Compute once
//! let iters = compute_fractal_iterations(&fractal, &config).unwrap();
//!
//! // Colorize many times (e.g., for a color animation)
//! for offset in 0_u32..64 {
//!     let frame = colorize_iterations(&iters, &config.color_config.clone().with_color_offset(offset));
//!     // export_png("frames/", offset, &frame, 800, 600);
//! }
//! ```
//!
//! ## Loading from a Saved File
//!
//! Round-trip from a previously exported PNG's embedded metadata:
//!
//! ```ignore
//! use forma_fractalis_core::{render_fractal_to_buffer, FractalConfig};
//! use forma_fractalis_core::export::FractalMetadata;
//! use forma_fractalis_core::fractals::Mandelbrot;
//!
//! let metadata = FractalMetadata::load_from_png("my_fractal.png").unwrap();
//! let fractal = Mandelbrot::new(); // match fractal_type from metadata
//! let config = FractalConfig::from_metadata(&metadata);
//! let buffer = render_fractal_to_buffer(&fractal, &config).unwrap();
//! ```
//!
//! ## Public API Surface
//!
//! | Function | Purpose |
//! |---|---|
//! | [`render_fractal_to_buffer`] | Single-call render to RGBA buffer |
//! | [`compute_fractal_iterations`] | Phase 1: compute raw iteration counts (slow) |
//! | [`colorize_iterations`] | Phase 2: apply colormap to cached counts (fast) |
//! | [`export::export_png_with_config`] | Write RGBA buffer to PNG with metadata |
//!
//! ## Available Fractals
//!
//! | Type | Name | Key Parameters |
//! |---|---|---|
//! | [`fractals::Mandelbrot`] | Mandelbrot Set | `power` (default 2) |
//! | [`fractals::Julia`] | Julia Set | `c_real`, `c_imag`, `power` (default 2) |
//! | [`fractals::BurningShip`] | Burning Ship | none |
//! | [`fractals::TippetsMandelbrot`] | Tippets Mandelbrot | none |
//! | [`fractals::MultifractalJulia`] | Multifractal-Julia | `power` |
//! | [`fractals::Zubieta`] | Zubieta | `c_real`, `c_imag` |
//! | [`fractals::SinJulia`] | Sin Julia | `c_real`, `c_imag`, `escape_radius` |
//! | [`fractals::SinhJulia`] | Sinh Julia | `c_real`, `c_imag`, `escape_radius` |
//! | [`fractals::Tetration`] | Tetration | `threshold` |
//! | [`fractals::Cactus`] | Cactus | none |
//! | [`fractals::Lemon`] | Lemon | `denom_power`, `convergence_exp` |
//! | [`fractals::MarekDragon`] | Marek Dragon | `phi` |
//! | [`fractals::InsideoutDragon`] | Insideout Dragon | `escape_radius` |
//!
//! ## GPU Note
//!
//! [`render_fractal_to_buffer`] and [`compute_fractal_iterations`] use CPU rendering
//! (Rayon parallel f64). GPU acceleration requires an initialised [`gpu::WgpuRenderer`]
//! — see the GUI source (`gui/src/main.rs`) for the GPU dispatch path.
//!
//! ## Modules
//! - [`fractals`]: Trait-based fractal system supporting multiple fractal types
//! - [`config`]: Configuration types for fractal rendering
//! - [`rendering`]: Parallel fractal rendering with Rayon; [`rendering::FractalIterations`] cache type
//! - [`rendering_pipeline`]: Unified rendering system for preview and export (used by GUI)
//! - [`export`]: PNG image export with scaling and metadata
//! - [`filtering`]: Image filtering and supersampling for high-quality exports
//! - [`number_utils`]: Numerical constants and utilities
//! - [`animation`]: Animated GIF generation for fractals
//! - [`gpu`]: GPU acceleration for fractal rendering (requires `gpu` feature)

pub mod animation;
pub mod config;
pub mod export;
pub mod filtering;
pub mod fractals;
pub mod gpu;
pub mod gpu_test;
pub mod number_utils;
pub mod orbit_accumulation;
pub mod perturbation;
pub mod rendering;
pub mod rendering_pipeline;

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to enable/disable performance logging
static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if profiling is enabled
pub fn is_profiling_enabled() -> bool {
    PROFILING_ENABLED.load(Ordering::Relaxed)
}

/// Enable profiling (called from main with --profiling flag)
pub fn enable_profiling() {
    PROFILING_ENABLED.store(true, Ordering::Relaxed);
}

/// Macro for conditional profiling output
#[macro_export]
macro_rules! perf_log {
    ($($arg:tt)*) => {
        if $crate::is_profiling_enabled() {
            println!($($arg)*);
        }
    };
}

// Public convenience re-exports for library users
pub use fractals::{Fractal, FractalView, Parameter};
pub use config::{FractalConfig, ColorConfig, ExportConfig};
pub use rendering::FractalIterations;

/// Compute raw iteration counts for all pixels without applying any colormap.
///
/// This is Phase 1 of the two-phase rendering API. The returned
/// [`FractalIterations`] can be passed to [`colorize_iterations`] multiple
/// times with different [`ColorConfig`] values — useful when you want to
/// preview several color schemes over the same geometry, or animate color
/// phase without recomputing the fractal each frame.
///
/// Rendering is CPU-based (f64 precision) and parallelised via Rayon.
/// GPU rendering requires a live [`gpu::WgpuRenderer`] and must go through
/// the GUI pipeline or [`rendering_pipeline::render_with_config`] directly.
///
/// # Errors
/// Returns `Err` if the fractal dimensions are zero. All other failure modes
/// are handled internally (clamped iteration counts, etc.).
///
/// # Example
/// ```ignore
/// use forma_fractalis_core::{compute_fractal_iterations, colorize_iterations, FractalConfig};
/// use forma_fractalis_core::fractals::Mandelbrot;
///
/// let fractal = Mandelbrot::new();
/// let config = FractalConfig::headless(1920, 1080, 256).with_period(64);
/// let iters = compute_fractal_iterations(&fractal, &config)?;
///
/// // Recolor 64 times with rolling phase — each <1ms vs ~80ms for full render:
/// for offset in 0_u32..64 {
///     let frame = colorize_iterations(&iters, &config.color_config.clone().with_color_offset(offset));
///     // save_png(format!("frame_{offset:04}.png"), &frame, 1920, 1080);
/// }
/// ```
pub fn compute_fractal_iterations(
    fractal: &dyn fractals::Fractal,
    config: &config::FractalConfig,
) -> Result<rendering::FractalIterations, String> {
    if config.view.width == 0 || config.view.height == 0 {
        return Err("FractalConfig view dimensions must be non-zero".to_string());
    }
    let data = rendering::compute_iterations(
        &config.view,
        config.max_iterations,
        fractal,
        &config.fractal_parameters,
    );
    Ok(rendering::FractalIterations::new(
        data,
        config,
        fractal.name(),
        gpu::RenderBackend::Cpu,
        0,
    ))
}

/// Apply a colormap to pre-computed iteration counts to produce an RGBA pixel buffer.
///
/// This is Phase 2 of the two-phase rendering API. It is intentionally cheap
/// (typically <1 ms for HD resolution) because it only maps iteration values
/// through a lookup; no fractal geometry is recomputed.
///
/// The returned buffer is `width * height * 4` bytes in RGBA row-major order.
///
/// # Color offset / animation
/// Set `color_config.color_offset` (or build with
/// [`ColorConfig::with_color_offset`]) to shift the colormap phase. Combined
/// with `use_period = true`, this animates colors without touching the
/// iteration data.
///
/// # Example
/// ```ignore
/// let iters = compute_fractal_iterations(&fractal, &config)?;
/// let base_colors = &config.color_config;
///
/// // First frame — no offset
/// let frame0 = colorize_iterations(&iters, base_colors);
///
/// // Second frame — shift colormap by 32 positions
/// let frame32 = colorize_iterations(&iters, &base_colors.clone().with_color_offset(32));
/// ```
pub fn colorize_iterations(
    iterations: &rendering::FractalIterations,
    color_config: &config::ColorConfig,
) -> Vec<u8> {
    let mut buffer = vec![0u8; (iterations.width * iterations.height * 4) as usize];
    rendering::apply_colors_from_cache(
        &mut buffer,
        &iterations.data,
        &color_config.colormap,
        iterations.max_iterations,
        color_config.use_period,
        color_config.period,
        color_config.use_interior_color,
        color_config.interior_color,
        color_config.use_log_scale,
        color_config.color_offset,
    );
    buffer
}

/// Render a fractal to an RGBA buffer in a single call.
///
/// This is the simplest entry point for one-off renders. It combines
/// [`compute_fractal_iterations`] and [`colorize_iterations`] internally.
/// When you need to recolor the same fractal geometry multiple times, call
/// those functions directly and reuse the [`FractalIterations`] cache.
///
/// Returns a `width * height * 4` byte RGBA buffer (row-major order).
///
/// GPU rendering is not available through this function — it requires an
/// initialised [`gpu::WgpuRenderer`]. Use [`rendering_pipeline::render_with_config`]
/// directly if GPU support is needed.
///
/// # Example
/// ```ignore
/// use forma_fractalis_core::{render_fractal_to_buffer, FractalConfig};
/// use forma_fractalis_core::fractals::Mandelbrot;
///
/// let fractal = Mandelbrot::new();
/// let config = FractalConfig::headless(800, 600, 256);
/// let buffer = render_fractal_to_buffer(&fractal, &config).unwrap();
/// assert_eq!(buffer.len(), 800 * 600 * 4);
/// ```
pub fn render_fractal_to_buffer(
    fractal: &dyn fractals::Fractal,
    config: &config::FractalConfig,
) -> Result<Vec<u8>, String> {
    let iterations = compute_fractal_iterations(fractal, config)?;
    Ok(colorize_iterations(&iterations, &config.color_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fractals::Mandelbrot;

    #[test]
    fn render_fractal_to_buffer_correct_size() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(64, 48, 64);
        let buf = render_fractal_to_buffer(&fractal, &config).unwrap();
        assert_eq!(buf.len(), 64 * 48 * 4, "buffer length should be width * height * 4 bytes");
    }

    #[test]
    fn render_fractal_to_buffer_non_black() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(64, 48, 64);
        let buf = render_fractal_to_buffer(&fractal, &config).unwrap();
        assert!(
            buf.iter().any(|&b| b > 0),
            "rendered output should not be fully black"
        );
    }

    #[test]
    fn fractal_config_headless_constructor() {
        let config = FractalConfig::headless(200, 100, 128);
        assert_eq!(config.view.width, 200);
        assert_eq!(config.view.height, 100);
        assert_eq!(config.max_iterations, 128);
    }

    #[test]
    fn fractal_config_builder_chain() {
        let view = crate::fractals::FractalView::new(100, 100);
        let config = FractalConfig::new(view, 100)
            .with_period(64)
            .with_log_scale();
        assert!(config.color_config.use_period);
        assert_eq!(config.color_config.period, 64);
        assert!(config.color_config.use_log_scale);
    }

    #[test]
    fn fractal_config_interior_color_builder() {
        let config = FractalConfig::headless(16, 16, 32).with_interior_color([255, 0, 128]);
        assert!(config.color_config.use_interior_color);
        assert_eq!(config.color_config.interior_color, [255, 0, 128]);
    }

    // ---- two-phase API tests ----

    #[test]
    fn compute_fractal_iterations_correct_length() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64);
        let iters = compute_fractal_iterations(&fractal, &config).unwrap();
        assert_eq!(iters.data.len(), 32 * 24);
        assert_eq!(iters.width, 32);
        assert_eq!(iters.height, 24);
        assert_eq!(iters.max_iterations, 64);
    }

    #[test]
    fn colorize_iterations_correct_buffer_length() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64);
        let iters = compute_fractal_iterations(&fractal, &config).unwrap();
        let buf = colorize_iterations(&iters, &config.color_config);
        assert_eq!(buf.len(), 32 * 24 * 4);
    }

    #[test]
    fn two_phase_matches_single_phase() {
        // render_fractal_to_buffer is now backed by the two-phase path,
        // so this effectively tests that compute+colorize produces the same
        // result as the combined call.
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64);
        let single = render_fractal_to_buffer(&fractal, &config).unwrap();
        let iters = compute_fractal_iterations(&fractal, &config).unwrap();
        let two_phase = colorize_iterations(&iters, &config.color_config);
        assert_eq!(single, two_phase);
    }

    #[test]
    fn color_offset_changes_output() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64).with_period(16);
        let iters = compute_fractal_iterations(&fractal, &config).unwrap();
        let frame0 = colorize_iterations(&iters, &config.color_config);
        let frame8 = colorize_iterations(&iters, &config.color_config.clone().with_color_offset(8));
        // An offset of 8 into period 16 should produce visibly different colors
        // somewhere in the image (not all pixels are in the interior).
        assert_ne!(frame0, frame8, "color_offset should change output pixels");
    }

    #[test]
    fn fractal_iterations_cache_validity() {
        use crate::gpu::RenderBackend;
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64);
        let iters = compute_fractal_iterations(&fractal, &config).unwrap();
        // Same config/fractal/backend: should be valid
        assert!(iters.is_valid_for(&config, fractal.name(), RenderBackend::Cpu, 0));
        // Different width: invalid
        let wide = FractalConfig::headless(64, 24, 64);
        assert!(!iters.is_valid_for(&wide, fractal.name(), RenderBackend::Cpu, 0));
        // Different fractal name: invalid
        assert!(!iters.is_valid_for(&config, "Julia", RenderBackend::Cpu, 0));
        // HiPrec backend: invalid (cache was computed with Cpu)
        assert!(!iters.is_valid_for(&config, fractal.name(), RenderBackend::CpuHiPrec, 256));
    }

    #[test]
    fn perturbation_cache_validity_tracks_pt_settings() {
        use crate::gpu::RenderBackend;

        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(32, 24, 64);
        let cache = FractalIterations::from_render_state(
            vec![0; (config.view.width * config.view.height) as usize],
            &config.view,
            config.max_iterations,
            fractal.name(),
            config.fractal_parameters.clone(),
            RenderBackend::Perturbation,
            128,
            4.0,
            4,
        );

        assert!(cache.is_valid_for_render(
            &config.view,
            config.max_iterations,
            fractal.name(),
            &config.fractal_parameters,
            RenderBackend::Perturbation,
            128,
            4.0,
            4,
        ));
        assert!(!cache.is_valid_for_render(
            &config.view,
            config.max_iterations,
            fractal.name(),
            &config.fractal_parameters,
            RenderBackend::Perturbation,
            256,
            4.0,
            4,
        ));
        assert!(!cache.is_valid_for_render(
            &config.view,
            config.max_iterations,
            fractal.name(),
            &config.fractal_parameters,
            RenderBackend::Perturbation,
            128,
            8.0,
            4,
        ));
        assert!(!cache.is_valid_for_render(
            &config.view,
            config.max_iterations,
            fractal.name(),
            &config.fractal_parameters,
            RenderBackend::Perturbation,
            128,
            4.0,
            2,
        ));
    }

    #[test]
    fn compute_fractal_iterations_zero_dimensions_errors() {
        let fractal = Mandelbrot::new();
        let config = FractalConfig::headless(0, 24, 64);
        assert!(compute_fractal_iterations(&fractal, &config).is_err());
    }
}
