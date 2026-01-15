//! Image Export System
//!
//! This module handles exporting fractal renders to image files.
//! Uses the unified rendering pipeline with optional filtering and supersampling
//! for professional-quality output.

use crate::colorschemes::ColorMap;
use crate::filtering::{apply_supersample_filter, calculate_supersample_dimensions, FilterType};
use crate::fractal::MandelbrotView;
use crate::fractals::Fractal;
use crate::rendering_pipeline::{render_with_config, RenderConfig, RenderTarget};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

/// Create metadata for PNG export as key-value pairs
///
/// Stores all fractal parameters, view settings, colormap, and render settings
/// as tEXt chunks for reproducibility
fn create_png_metadata(
    view: &MandelbrotView,
    colormap: &ColorMap,
    max_iterations: u32,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    filter_type: FilterType,
    supersample: u32,
    scale: f32,
) -> Vec<(String, String)> {
    let mut metadata = Vec::new();

    // Fractal information
    metadata.push(("Fractal-Type".to_string(), fractal.name().to_string()));
    
    // Fractal parameters as JSON (for complex types like Julia)
    if !fractal_parameters.is_empty() {
        let params_json = serde_json::to_string(fractal_parameters).unwrap_or_default();
        metadata.push(("Fractal-Parameters".to_string(), params_json));
    }

    // View coordinates
    metadata.push(("View-CenterX".to_string(), view.center_x.to_string()));
    metadata.push(("View-CenterY".to_string(), view.center_y.to_string()));
    metadata.push(("View-Zoom".to_string(), view.zoom.to_string()));
    
    // Iterations
    metadata.push(("Max-Iterations".to_string(), max_iterations.to_string()));

    // Colormap information
    metadata.push(("Colormap-Name".to_string(), colormap.name.clone()));
    
    // Full colormap data as JSON for reproducibility
    let colormap_json = serde_json::to_string(colormap).unwrap_or_default();
    metadata.push(("Colormap-Data".to_string(), colormap_json));

    // Color modulation settings
    metadata.push(("Color-Period-Enabled".to_string(), use_period.to_string()));
    if use_period {
        metadata.push(("Color-Period".to_string(), period.to_string()));
    }
    
    metadata.push(("Interior-Color-Enabled".to_string(), use_interior_color.to_string()));
    if use_interior_color {
        let color_json = serde_json::to_string(&interior_color).unwrap_or_default();
        metadata.push(("Interior-Color-RGB".to_string(), color_json));
    }
    
    metadata.push(("Log-Scale-Enabled".to_string(), use_log_scale.to_string()));

    // Export settings
    metadata.push(("Export-Filter".to_string(), filter_type.as_str().to_string()));
    metadata.push(("Export-Supersample".to_string(), supersample.to_string()));
    metadata.push(("Export-Scale".to_string(), scale.to_string()));
    
    // Metadata version for future compatibility
    metadata.push(("MandelRust-Version".to_string(), env!("CARGO_PKG_VERSION").to_string()));
    metadata.push(("Metadata-Version".to_string(), "1.0".to_string()));

    metadata
}

/// Export the current fractal view to a PNG file
///
/// # Arguments
/// * `view` - The fractal view parameters
/// * `colormap` - The color scheme to use
/// * `max_iterations` - Maximum iteration count
/// * `fractal` - The fractal implementation to render
/// * `fractal_parameters` - Fractal-specific parameters
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
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
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

    // Build render configuration with provided fractal
    let config = RenderConfig::new(view.clone(), colormap, max_iterations, fractal)
        .with_fractal_parameters(fractal_parameters.clone())
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

    // Use fractal name in filename (lowercase, replace spaces with underscores)
    let fractal_name = fractal.name().to_lowercase().replace(' ', "_");
    let filename = format!(
        "{}_{}{}{}.png",
        fractal_name, target_width, filter_suffix, timestamp
    );

    // Construct full path
    let path = if let Some(dir) = output_dir {
        dir.join(filename)
    } else {
        PathBuf::from(filename)
    };

    // Create metadata
    let metadata = create_png_metadata(
        view,
        colormap,
        max_iterations,
        fractal,
        fractal_parameters,
        use_period,
        period,
        use_interior_color,
        interior_color,
        use_log_scale,
        filter_type,
        supersample,
        scale,
    );

    // Save the image with metadata using png crate
    let file = File::create(&path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let writer = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(writer, target_width, target_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);
    
    // Add metadata as tEXt chunks
    for (key, value) in metadata {
        encoder.add_text_chunk(key, value)
            .map_err(|e| format!("Failed to add metadata: {}", e))?;
    }
    
    let mut writer = encoder.write_header()
        .map_err(|e| format!("Failed to write PNG header: {}", e))?;
    
    // Write the image data
    writer.write_image_data(&final_buffer)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

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
