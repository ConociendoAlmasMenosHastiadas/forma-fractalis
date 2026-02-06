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
) {
    let total_timer = Instant::now();
    
    // Create a flat list of all pixel coordinates for better parallelization
    let total_pixels = (view.width * view.height) as usize;
    
    // Parallel processing: compute all pixels independently
    let fractal_timer = Instant::now();
    let pixels: Vec<[u8; 4]> = (0..total_pixels)
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
        .collect();
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
