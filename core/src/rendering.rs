//! Parallel Fractal Rendering
//!
//! This module handles the conversion of fractal iteration counts into
//! colored pixels using parallel processing via Rayon.
//!
//! # Features
//! - Multi-threaded row-based parallel rendering
//! - ColorMap-based gradient coloring
//! - RGBA buffer output for GPU texture upload

use crate::perf_log;
use scala_chromatica::{color_from_iterations, ColorMap};
use crate::fractals::{Fractal, FractalView};
use crate::gpu::RenderBackend;
use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

/// Maximum iterations for Mandelbrot calculation
pub const MAX_ITERATIONS: u32 = 256;

/// Compute iteration counts for all pixels without coloring
/// Returns a vector of iteration counts suitable for caching
///
/// # Arguments
/// * `view` - View parameters (center, zoom, dimensions)
/// * `max_iterations` - Maximum iterations for fractal calculation
/// * `fractal` - The fractal implementation to render
/// * `fractal_parameters` - Parameters specific to the fractal type
pub fn compute_iterations(
    view: &FractalView,
    max_iterations: u32,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
) -> Vec<u32> {
    let total_pixels = (view.width * view.height) as usize;
    
    perf_log!("[CACHE] Computing iterations for {}x{} = {} pixels", 
        view.width, view.height, total_pixels);
    
    let timer = Instant::now();
    let iterations: Vec<u32> = (0..total_pixels)
        .into_par_iter()
        .map(|i| {
            let x = (i % view.width as usize) as u32;
            let y = (i / view.width as usize) as u32;
            let (real, imag) = view.screen_to_complex(x, y);
            fractal.iterate(real, imag, fractal_parameters, max_iterations)
        })
        .collect();
    
    perf_log!("[CACHE] Iteration computation took {:.2?}", timer.elapsed());
    iterations
}

/// Compute hi-precision iteration counts for all pixels without coloring.
///
/// Identical in structure to `compute_iterations` but calls `fractal.iterate_hiprec()`.
/// Returns an error string if the fractal does not support hi-prec.
///
/// # Arguments
/// * `bits` - Software float bit width: one of 64, 128, 256, 512, 1024
/// * `max_threads` - Rayon thread limit; 0 = use all available
pub fn compute_iterations_hiprec(
    view: &FractalView,
    max_iterations: u32,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    bits: u32,
    max_threads: usize,
) -> Result<Vec<u32>, String> {
    if !fractal.supports_hiprec() {
        return Err(format!(
            "Fractal '{}' does not support CPU Hi-Prec rendering.",
            fractal.name()
        ));
    }
    let total_pixels = (view.width * view.height) as usize;
    perf_log!("[CACHE-HIPREC] Computing {}bit iterations for {}x{} = {} pixels",
        bits, view.width, view.height, total_pixels);
    let timer = Instant::now();
    let compute = || -> Vec<u32> {
        (0..total_pixels)
            .into_par_iter()
            .map(|i| {
                let x = (i % view.width as usize) as u32;
                let y = (i / view.width as usize) as u32;
                let (real, imag) = view.screen_to_complex_hiprec(x, y, bits);
                fractal.iterate_hiprec(&real, &imag, fractal_parameters, max_iterations, bits)
            })
            .collect()
    };
    let global_count = rayon::current_num_threads();
    let iterations = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new().num_threads(max_threads).build() {
            Ok(pool) => pool.install(|| compute()),
            Err(_) => compute(),
        }
    } else {
        compute()
    };
    perf_log!("[CACHE-HIPREC] Iteration computation took {:.2?}", timer.elapsed());
    Ok(iterations)
}

/// Pre-computed iteration counts for a fractal view.
///
/// Produced by [`crate::compute_fractal_iterations`]. Can be passed to
/// [`crate::colorize_iterations`] one or more times with different [`crate::config::ColorConfig`]
/// values to recolor without recomputing the fractal — coloring typically takes <1 ms,
/// while computation takes tens to hundreds of milliseconds depending on resolution.
///
/// # Cache validity
/// Call [`FractalIterations::is_valid_for`] before deciding whether to reuse a cached result.
/// The result is invalid if the view geometry, fractal, iteration limit, or rendering
/// backend changed. It remains valid when only color settings change (colormap, period,
/// interior color, log scale, color offset).
///
/// # Batch rendering / animation example
/// ```ignore
/// use forma_fractalis_core::{compute_fractal_iterations, colorize_iterations, FractalConfig};
/// use forma_fractalis_core::fractals::Mandelbrot;
///
/// let fractal = Mandelbrot::new();
/// let config = FractalConfig::headless(800, 600, 256).with_period(64);
/// let iters = compute_fractal_iterations(&fractal, &config)?;
///
/// // Animate color phase across 64 frames without recomputing the fractal:
/// for offset in 0_u32..64 {
///     let colors = colorize_iterations(&iters, &config.color_config.clone().with_color_offset(offset));
///     // save frame...
/// }
/// ```
#[derive(Clone, Debug)]
pub struct FractalIterations {
    /// Raw escape iteration count per pixel, row-major order.
    /// Length == `width * height`.
    pub data: Vec<u32>,
    /// Width of the rendered area in pixels.
    pub width: u32,
    /// Height of the rendered area in pixels.
    pub height: u32,
    /// Maximum iteration limit used during computation.
    pub max_iterations: u32,
    /// Real part of the viewport centre in the complex plane.
    pub center_x: f64,
    /// Imaginary part of the viewport centre in the complex plane.
    pub center_y: f64,
    /// Zoom level (higher = more zoomed in).
    pub zoom: f64,
    /// Optional higher-precision decimal shadow for the real center.
    pub precise_center_x: Option<String>,
    /// Optional higher-precision decimal shadow for the imaginary center.
    pub precise_center_y: Option<String>,
    /// Optional higher-precision decimal shadow for the zoom.
    pub precise_zoom: Option<String>,
    /// Name of the fractal that produced this data (from `Fractal::name()`).
    pub fractal_name: String,
    /// Fractal-specific parameter snapshot (e.g. Julia `c`, power, escape radius).
    pub fractal_parameters: HashMap<String, f64>,
    /// Rendering backend that computed this data.
    ///
    /// CPU f64 and CPU HiPrec (software BigFloat) can produce different boundary
    /// pixel values, so the cache must be invalidated when the backend changes.
    pub backend: RenderBackend,
    /// Software-float bit width used when `backend == CpuHiPrec`; 0 otherwise.
    pub hiprec_bits: u32,
    /// PT glitch tolerance used when `backend == Perturbation`.
    /// Stored as raw `f64::to_bits()` so equality is exact and stable.
    pub pt_glitch_tolerance_bits: u64,
    /// PT tile-grid size used when `backend == Perturbation`.
    pub pt_tiles: u32,
}

impl FractalIterations {
    /// Build a `FractalIterations` from raw iteration data and the config/fractal
    /// that produced it.
    ///
    /// `fractal_name` should be the value returned by `fractal.name()` so that
    /// [`is_valid_for`](FractalIterations::is_valid_for) can detect fractal-type changes.
    pub fn new(
        data: Vec<u32>,
        config: &crate::config::FractalConfig,
        fractal_name: impl Into<String>,
        backend: RenderBackend,
        hiprec_bits: u32,
    ) -> Self {
        Self::from_render_state(
            data,
            &config.view,
            config.max_iterations,
            fractal_name,
            config.fractal_parameters.clone(),
            backend,
            hiprec_bits,
            1.0,
            1,
        )
    }

    /// Build a `FractalIterations` from explicit render-state components.
    ///
    /// This is used by the unified rendering pipeline so preview caching can
    /// capture GPU, PT, and hi-precision results without round-tripping through
    /// the GUI-only state types.
    pub fn from_render_state(
        data: Vec<u32>,
        view: &FractalView,
        max_iterations: u32,
        fractal_name: impl Into<String>,
        fractal_parameters: HashMap<String, f64>,
        backend: RenderBackend,
        hiprec_bits: u32,
        pt_glitch_tolerance: f64,
        pt_tiles: u32,
    ) -> Self {
        Self {
            data,
            width: view.width,
            height: view.height,
            max_iterations,
            center_x: view.center_x,
            center_y: view.center_y,
            zoom: view.zoom,
            precise_center_x: view.precise_center_x.clone(),
            precise_center_y: view.precise_center_y.clone(),
            precise_zoom: view.precise_zoom.clone(),
            fractal_name: fractal_name.into(),
            fractal_parameters,
            backend,
            hiprec_bits,
            pt_glitch_tolerance_bits: pt_glitch_tolerance.to_bits(),
            pt_tiles: pt_tiles.max(1),
        }
    }

    /// Returns `true` if this cached result is still valid for the given render parameters.
    ///
    /// A cache hit means: same dimensions, same view coordinates (within floating-point
    /// tolerance), same fractal, same iteration limit, same fractal parameters, same
    /// rendering backend, and (when HiPrec) same bit width.
    ///
    /// Color-only fields (`colormap`, `period`, `interior_color`, `use_log_scale`,
    /// `color_offset`) are **not** checked — they do not affect iteration counts.
    pub fn is_valid_for(
        &self,
        config: &crate::config::FractalConfig,
        fractal_name: &str,
        backend: RenderBackend,
        hiprec_bits: u32,
    ) -> bool {
        self.is_valid_for_render(
            &config.view,
            config.max_iterations,
            fractal_name,
            &config.fractal_parameters,
            backend,
            hiprec_bits,
            1.0,
            1,
        )
    }

    /// Returns `true` if this cached result is still valid for the given render state.
    ///
    /// Unlike [`is_valid_for`](FractalIterations::is_valid_for), this includes
    /// backend-specific knobs that live outside `FractalConfig`, such as PT tile
    /// count and glitch tolerance.
    pub fn is_valid_for_render(
        &self,
        view: &FractalView,
        max_iterations: u32,
        fractal_name: &str,
        fractal_parameters: &HashMap<String, f64>,
        backend: RenderBackend,
        hiprec_bits: u32,
        pt_glitch_tolerance: f64,
        pt_tiles: u32,
    ) -> bool {
        self.width == view.width
            && self.height == view.height
            && self.max_iterations == max_iterations
            && (self.center_x - view.center_x).abs() < 1e-10
            && (self.center_y - view.center_y).abs() < 1e-10
            && (self.zoom - view.zoom).abs() < 1e-10
            && self.precise_center_x == view.precise_center_x
            && self.precise_center_y == view.precise_center_y
            && self.precise_zoom == view.precise_zoom
            && self.fractal_name == fractal_name
            && self.fractal_parameters == *fractal_parameters
            && self.backend == backend
            && (!matches!(backend, RenderBackend::CpuHiPrec | RenderBackend::Perturbation) || self.hiprec_bits == hiprec_bits)
            && (!matches!(backend, RenderBackend::Perturbation)
                || (self.pt_glitch_tolerance_bits == pt_glitch_tolerance.to_bits()
                    && self.pt_tiles == pt_tiles.max(1)))
    }
}

///
/// # Arguments
/// * `frame` - Mutable reference to the pixel buffer (RGBA format)
/// * `iterations` - Pre-computed iteration counts from cache
/// * `colormap` - Colormap to use for rendering
/// * `max_iterations` - Maximum iterations used for the cache
/// * `use_period` - Whether to enable period modulation
/// * `period` - Period value for modulo operation on iterations
/// * `use_interior_color` - Whether to use custom interior color
/// * `interior_color` - RGB color for points inside the set
/// * `use_log_scale` - Whether to apply logarithmic scaling to colors
pub fn apply_colors_from_cache(
    frame: &mut [u8],
    iterations: &[u32],
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    color_offset: u32,
) {
    perf_log!("[CACHE] Applying colors to {} cached pixels", iterations.len());
    
    let timer = Instant::now();
    let pixels: Vec<[u8; 4]> = iterations
        .par_iter()
        .map(|&iter| {
            // Apply phase offset before lookup. Has no effect when use_period = false
            // because color_from_iterations ignores `iter` mod when period is off.
            // wrapping_add ensures no panic on overflow.
            let effective_iter = iter.wrapping_add(color_offset);
            let color = color_from_iterations(
                effective_iter,
                max_iterations,
                colormap,
                use_period,
                period,
                use_interior_color,
                interior_color,
                use_log_scale,
            );
            [color.r, color.g, color.b, 255]
        })
        .collect();
    
    // Copy computed pixels to frame buffer
    for (i, pixel) in pixels.iter().enumerate() {
        let idx = i * 4;
        frame[idx..idx + 4].copy_from_slice(pixel);
    }
    
    perf_log!("[CACHE] Color application took {:.2?}", timer.elapsed());
}

/// Renders any fractal to a pixel buffer using parallel processing
///
/// # Arguments
/// * `frame` - Mutable reference to the pixel buffer (RGBA format)
/// * `view` - View parameters (center, zoom, dimensions)
/// * `colormap` - Colormap to use for rendering
/// * `max_iterations` - Maximum iterations for fractal calculation
/// * `use_period` - Whether to enable period modulation
/// * `period` - Period value for modulo operation on iterations
/// * `use_interior_color` - Whether to use custom interior color
/// * `interior_color` - RGB color for points inside the set
/// * `use_log_scale` - Whether to apply logarithmic scaling to colors
/// * `fractal` - The fractal implementation to render
/// * `fractal_parameters` - Parameters specific to the fractal type
pub fn render_fractal(
    frame: &mut [u8],
    view: &FractalView,
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    max_threads: usize,
) {
    let total_timer = Instant::now();
    
    // Create a flat list of all pixel coordinates for better parallelization
    let total_pixels = (view.width * view.height) as usize;
    
    // Build the pixel computation as a closure so it can be dispatched to either
    // the global rayon pool (max_threads == 0 or == global count) or a smaller
    // temporary pool (max_threads > 0 && < global count).
    let fractal_timer = Instant::now();
    let compute_pixels = || -> Vec<[u8; 4]> {
        (0..total_pixels)
            .into_par_iter()
            .map(|i| {
                let x = (i % view.width as usize) as u32;
                let y = (i / view.width as usize) as u32;
                let (real, imag) = view.screen_to_complex(x, y);
                let iter = fractal.iterate(real, imag, fractal_parameters, max_iterations);
                let color = color_from_iterations(
                    iter,
                    max_iterations,
                    colormap,
                    use_period,
                    period,
                    use_interior_color,
                    interior_color,
                    use_log_scale,
                );
                [color.r, color.g, color.b, 255]
            })
            .collect()
    };
    let global_count = rayon::current_num_threads();
    let pixels = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new().num_threads(max_threads).build() {
            Ok(pool) => pool.install(|| compute_pixels()),
            Err(_) => compute_pixels(), // graceful fallback: use global pool
        }
    } else {
        compute_pixels()
    };
    let fractal_time = fractal_timer.elapsed();

    // Copy computed pixels to frame buffer
    let copy_timer = Instant::now();
    for (i, pixel) in pixels.iter().enumerate() {
        let idx = i * 4;
        frame[idx..idx + 4].copy_from_slice(pixel);
    }
    let copy_time = copy_timer.elapsed();
    
    let total_time = total_timer.elapsed();
    perf_log!("[PERF-DETAIL] render_fractal: fractal+color={:.2?}, copy={:.2?}, total={:.2?}",
        fractal_time, copy_time, total_time);
}

/// Renders a fractal using the CPU hi-precision backend (software BigFloat arithmetic).
///
/// Works identically to `render_fractal` in structure and coloring, but calls
/// `fractal.iterate_hiprec()` instead of `fractal.iterate()`.
///
/// If the fractal does not support hi-prec (`supports_hiprec() == false`), this
/// returns an error string — the caller must surface this to the user visibly.
///
/// # Arguments
/// Same as `render_fractal`, with the addition of:
/// * `bits` - Software float bit width: one of 64, 128, 256, 512, 1024
pub fn render_fractal_hiprec(
    frame: &mut [u8],
    view: &FractalView,
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    bits: u32,
    max_threads: usize,
) -> Result<(), String> {
    if !fractal.supports_hiprec() {
        return Err(format!(
            "Fractal '{}' does not support CPU Hi-Prec rendering. Switch to CPU or GPU backend.",
            fractal.name()
        ));
    }

    let total_timer = Instant::now();
    let total_pixels = (view.width * view.height) as usize;

    let fractal_timer = Instant::now();
    let compute_pixels = || -> Vec<[u8; 4]> {
        (0..total_pixels)
            .into_par_iter()
            .map(|i| {
                let x = (i % view.width as usize) as u32;
                let y = (i / view.width as usize) as u32;
                let (real, imag) = view.screen_to_complex_hiprec(x, y, bits);
                let iter = fractal.iterate_hiprec(&real, &imag, fractal_parameters, max_iterations, bits);
                let color = color_from_iterations(
                    iter,
                    max_iterations,
                    colormap,
                    use_period,
                    period,
                    use_interior_color,
                    interior_color,
                    use_log_scale,
                );
                [color.r, color.g, color.b, 255]
            })
            .collect()
    };
    let global_count = rayon::current_num_threads();
    let pixels = if max_threads > 0 && max_threads < global_count {
        match rayon::ThreadPoolBuilder::new().num_threads(max_threads).build() {
            Ok(pool) => pool.install(|| compute_pixels()),
            Err(_) => compute_pixels(), // graceful fallback: use global pool
        }
    } else {
        compute_pixels()
    };
    let fractal_time = fractal_timer.elapsed();

    let copy_timer = Instant::now();
    for (i, pixel) in pixels.iter().enumerate() {
        let idx = i * 4;
        frame[idx..idx + 4].copy_from_slice(pixel);
    }
    let copy_time = copy_timer.elapsed();

    let total_time = total_timer.elapsed();
    perf_log!(
        "[PERF-HIPREC] render_fractal_hiprec {}bit {}x{} @ {} iter: fractal+color={:.2?}, copy={:.2?}, total={:.2?}",
        bits, view.width, view.height, max_iterations, fractal_time, copy_time, total_time
    );

    Ok(())
}

/// Draws a rectangle overlay on the frame (used for zoom preview)
///
/// # Arguments
/// * `frame` - Mutable reference to the pixel buffer
/// * `center_x` - X coordinate of rectangle center
/// * `center_y` - Y coordinate of rectangle center
/// * `rect_width` - Width of the rectangle
/// * `rect_height` - Height of the rectangle
/// * `buffer_width` - Width of the entire buffer
/// * `buffer_height` - Height of the entire buffer
pub fn draw_rectangle(
    frame: &mut [u8],
    center_x: u32,
    center_y: u32,
    rect_width: u32,
    rect_height: u32,
    buffer_width: u32,
    buffer_height: u32,
) {
    let half_width = rect_width / 2;
    let half_height = rect_height / 2;

    let x1 = center_x
        .saturating_sub(half_width)
        .min(buffer_width - 1);
    let y1 = center_y
        .saturating_sub(half_height)
        .min(buffer_height - 1);
    let x2 = (center_x + half_width).min(buffer_width - 1);
    let y2 = (center_y + half_height).min(buffer_height - 1);

    // Draw rectangle outline in white with 3-pixel thickness
    let color = [255, 255, 255, 255];
    let thickness = 3;

    // Draw thick horizontal lines (top and bottom)
    for t in 0..thickness {
        for x in x1..=x2 {
            // Top edge
            if y1 + t < buffer_height {
                let idx_top = ((y1 + t) as usize * buffer_width as usize + x as usize) * 4;
                if idx_top + 4 <= frame.len() {
                    frame[idx_top..idx_top + 4].copy_from_slice(&color);
                }
            }
            // Bottom edge
            if y2 >= t {
                let idx_bottom = ((y2 - t) as usize * buffer_width as usize + x as usize) * 4;
                if idx_bottom + 4 <= frame.len() {
                    frame[idx_bottom..idx_bottom + 4].copy_from_slice(&color);
                }
            }
        }
    }

    // Draw thick vertical lines (left and right)
    for t in 0..thickness {
        for y in y1..=y2 {
            // Left edge
            if x1 + t < buffer_width {
                let idx_left = (y as usize * buffer_width as usize + (x1 + t) as usize) * 4;
                if idx_left + 4 <= frame.len() {
                    frame[idx_left..idx_left + 4].copy_from_slice(&color);
                }
            }
            // Right edge
            if x2 >= t {
                let idx_right = (y as usize * buffer_width as usize + (x2 - t) as usize) * 4;
                if idx_right + 4 <= frame.len() {
                    frame[idx_right..idx_right + 4].copy_from_slice(&color);
                }
            }
        }
    }
}
