//! Image Export System
//!
//! This module handles exporting fractal renders to image files.
//! Uses the unified rendering pipeline with optional filtering and supersampling
//! for professional-quality output.

use crate::colorschemes::ColorMap;
use crate::filtering::{apply_supersample_filter, calculate_supersample_dimensions, FilterType};
use crate::fractal::MandelbrotView;
use crate::rendering_pipeline::{render_with_config, RenderConfig, RenderTarget};
use image::{ImageBuffer, Rgba};
use std::path::PathBuf;

/// Export the current fractal view to a PNG file
///
/// # Arguments
/// * `view` - The fractal view parameters
/// * `colormap` - The color scheme to use
/// * `max_iterations` - Maximum iteration count
/// * `use_period` - Whether to use period modulation
/// * `period` - Period value for modulation
/// * `use_interior_color` - Whether to use custom interior color
/// * `interior_color` - RGB color for interior points
/// * `use_log_scale` - Whether to apply logarithmic color scaling
/// * `filter_type` - Filter to apply for downsampling (None, Lanczos3, Gaussian)
/// * `supersample` - Supersampling multiplier (1 = no supersample, 2 = 2x, etc.)
/// * `scale` - Scaling factor (e.g., 3.0 for 3x the preview dimensions)
/// * `output_dir` - Optional output directory (uses current directory if None)
///
/// # Returns
/// Result with the path to the saved file, or an error message
pub fn export_png(
    view: &MandelbrotView,
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    filter_type: FilterType,
    supersample: u32,
    scale: f32,
    output_dir: Option<&PathBuf>,
) -> Result<String, String> {
    // Calculate target output dimensions (base size * scale)
    let target_width = (view.width as f32 * scale) as u32;
    let target_height = (view.height as f32 * scale) as u32;

    // Calculate supersample dimensions if filtering is enabled
    let supersample = if filter_type == FilterType::None { 1 } else { supersample.max(1) };
    let (render_width, render_height) = calculate_supersample_dimensions(
        target_width,
        target_height,
        supersample,
    );

    // Build render configuration
    let config = RenderConfig::new(view.clone(), colormap, max_iterations)
        .with_period(use_period, period)
        .with_interior_color(use_interior_color, interior_color)
        .with_log_scale(use_log_scale);

    // Render at supersample resolution
    let buffer = render_with_config(
        &config,
        RenderTarget::Export {
            width: render_width,
            height: render_height,
        },
    );

    // Apply filtering if enabled (downsample from supersample to target)
    let final_buffer = if filter_type != FilterType::None && supersample > 1 {
        apply_supersample_filter(
            &buffer,
            render_width,
            render_height,
            target_width,
            target_height,
            filter_type,
        )?
    } else {
        buffer
    };

    // Convert to image buffer (using final dimensions)
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        target_width,
        target_height,
        final_buffer,
    )
    .ok_or("Failed to create image buffer")?;

    // Generate filename with timestamp and filter info
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filter_suffix = if filter_type != FilterType::None && supersample > 1 {
        format!("_{}x{}", supersample, filter_type.as_str())
    } else {
        String::new()
    };

    let filename = format!(
        "mandelbrot_{}x{}{}{}.png",
        target_width, target_height, filter_suffix, timestamp
    );

    // Construct full path
    let path = if let Some(dir) = output_dir {
        dir.join(filename)
    } else {
        PathBuf::from(filename)
    };

    // Save the image
    img.save(&path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(path.display().to_string())
}

/// Calculate the output dimensions for a given scale
pub fn calculate_output_dimensions(view: &MandelbrotView, scale: f32) -> (u32, u32) {
    let output_width = (view.width as f32 * scale) as u32;
    let output_height = (view.height as f32 * scale) as u32;
    (output_width, output_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_output_dimensions() {
        let view = MandelbrotView::new(1280, 720);

        let (w, h) = calculate_output_dimensions(&view, 1.0);
        assert_eq!(w, 1280);
        assert_eq!(h, 720);

        let (w, h) = calculate_output_dimensions(&view, 2.0);
        assert_eq!(w, 2560);
        assert_eq!(h, 1440);

        let (w, h) = calculate_output_dimensions(&view, 3.0);
        assert_eq!(w, 3840);
        assert_eq!(h, 2160);
    }
}
