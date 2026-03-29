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
) {
    perf_log!("[CACHE] Applying colors to {} cached pixels", iterations.len());
    
    let timer = Instant::now();
    let pixels: Vec<[u8; 4]> = iterations
        .par_iter()
        .map(|&iter| {
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
