//! Image Export System
//!
//! This module handles exporting fractal renders to image files.
//! Supports scaling and various output formats, with PNG as the primary format.

use crate::colorschemes::ColorMap;
use crate::fractal::MandelbrotView;
use crate::rendering::render_mandelbrot;
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
    scale: f32,
    output_dir: Option<&PathBuf>,
) -> Result<String, String> {
    // Calculate scaled dimensions
    let output_width = (view.width as f32 * scale) as u32;
    let output_height = (view.height as f32 * scale) as u32;

    // Create a scaled view
    let mut scaled_view = view.clone();
    scaled_view.width = output_width;
    scaled_view.height = output_height;

    // Render to buffer
    let buffer_size = (output_width * output_height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];

    render_mandelbrot(
        &mut buffer,
        &scaled_view,
        colormap,
        max_iterations,
        use_period,
        period,
        use_interior_color,
        interior_color,
    );

    // Convert to image buffer
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(output_width, output_height, buffer)
        .ok_or("Failed to create image buffer")?;

    // Generate filename with timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = format!(
        "mandelbrot_{}x{}_{}.png",
        output_width, output_height, timestamp
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
