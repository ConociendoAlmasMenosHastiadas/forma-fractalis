//! Image Export and Import System
//!
//! This module handles exporting fractal renders to image files with embedded metadata,
//! and importing settings from previously exported PNG files.
//! Uses the unified rendering pipeline with optional filtering and supersampling
//! for professional-quality output.

use scala_chromatica::ColorMap;
use crate::filtering::{apply_supersample_filter, calculate_supersample_dimensions, FilterType};
use crate::fractals::{Fractal, FractalView};
use crate::perf_log;
use crate::rendering_pipeline::{render_with_config, RenderConfig, RenderTarget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Create metadata for PNG export as key-value pairs
///
/// Stores all fractal parameters, view settings, colormap, and render settings
/// as tEXt chunks for reproducibility
fn create_png_metadata(
    view: &FractalView,
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
    metadata.push(("View-Width".to_string(), view.width.to_string()));
    metadata.push(("View-Height".to_string(), view.height.to_string()));
    
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
    metadata.push(("Forma-Fractalis-Version".to_string(), env!("CARGO_PKG_VERSION").to_string()));
    metadata.push(("Metadata-Version".to_string(), "1.0".to_string()));
    
    // Timestamp for when the fractal was created
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    metadata.push(("Created-Timestamp".to_string(), timestamp));

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
    view: &FractalView,
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
    backend: crate::gpu::RenderBackend,
    // Bit width for CPU Hi-Prec rendering. Ignored for other backends.
    hiprec_bits: u32,
    // Max rayon threads for CPU rendering. 0 = use all available.
    max_threads: usize,
    #[cfg(feature = "gpu")]
    gpu_renderer: Option<&mut crate::gpu::WgpuRenderer>,
) -> Result<String, String> {
    let export_timer = std::time::Instant::now();
    
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
    
    let setup_time = export_timer.elapsed();
    perf_log!("[EXPORT-PERF] Setup complete: {:?}", setup_time);
    perf_log!("[EXPORT-PERF] Target: {}x{}, Render: {}x{} ({}x SS)", 
        target_width, target_height, render_width, render_height, supersample);

    // Build render configuration with provided fractal
    let config = RenderConfig::new(view.clone(), colormap, max_iterations, fractal)
        .with_fractal_parameters(fractal_parameters.clone())
        .with_period(use_period, period)
        .with_interior_color(use_interior_color, interior_color)
        .with_log_scale(use_log_scale)
        .with_backend(backend)
        .with_hiprec_bits(hiprec_bits)
        .with_max_threads(max_threads);

    // Render at supersample resolution
    let render_start = std::time::Instant::now();
    #[cfg(feature = "gpu")]
    let buffer = render_with_config(
        &config,
        RenderTarget::Export {
            width: render_width,
            height: render_height,
        },
        gpu_renderer,
    )?;
    #[cfg(not(feature = "gpu"))]
    let buffer = render_with_config(
        &config,
        RenderTarget::Export {
            width: render_width,
            height: render_height,
        },
    )?;
    let render_time = render_start.elapsed();
    perf_log!("[EXPORT-PERF] Render complete: {:?}", render_time);

    // Apply filtering if enabled (downsample from supersample to target)
    let filter_start = std::time::Instant::now();
    let final_buffer = if filter_type != FilterType::None && supersample > 1 {
        let filtered = apply_supersample_filter(
            &buffer,
            render_width,
            render_height,
            target_width,
            target_height,
            filter_type,
        )?;
        let filter_time = filter_start.elapsed();
        perf_log!("[EXPORT-PERF] Filtering complete: {:?}", filter_time);
        filtered
    } else {
        perf_log!("[EXPORT-PERF] Filtering skipped (no filter or supersample=1)");
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
        "{}_{}x{}{}{}.png",
        fractal_name, target_width, target_height, filter_suffix, timestamp
    );

    // Construct full path
    let path = if let Some(dir) = output_dir {
        dir.join(filename)
    } else {
        PathBuf::from(filename)
    };

    // Create metadata
    let metadata_start = std::time::Instant::now();
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
    let metadata_time = metadata_start.elapsed();
    perf_log!("[EXPORT-PERF] Metadata creation: {:?}", metadata_time);

    // Save the image with metadata using png crate
    let io_start = std::time::Instant::now();
    let file = File::create(&path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let writer = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(writer, target_width, target_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::default());
    
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
    
    let io_time = io_start.elapsed();
    let total_time = export_timer.elapsed();
    
    perf_log!("[EXPORT-PERF] PNG encoding/write: {:?}", io_time);
    perf_log!("[EXPORT-PERF] ========================================");
    perf_log!("[EXPORT-PERF] TOTAL EXPORT TIME: {:?}", total_time);
    perf_log!("[EXPORT-PERF] Breakdown: setup={:.1?}, render={:.1?}, filter={:.1?}, metadata={:.1?}, io={:.1?}",
        setup_time, render_time, filter_start.elapsed(), metadata_time, io_time);
    perf_log!("[EXPORT-PERF] ========================================");

    Ok(path.display().to_string())
}

/// Export PNG from application state structs
///
/// This is the recommended way to export fractal images as it uses the clean
/// state struct architecture from v0.1.5 and automatically handles metadata
/// creation through bidirectional conversions.
///
/// # Arguments
/// * `fractal_state` - Fractal type and parameters
/// * `view_state` - View position and zoom
/// * `color_state` - Colormap and color settings
/// * `input_state` - Iterations and other numeric inputs
/// * `export_state` - Export settings (filter, directory)
/// * `fractal` - The fractal implementation to render
/// * `scale` - Scaling factor (e.g., 3.0 for 3x the preview dimensions)
/// * `supersample` - Supersampling multiplier (1 = no supersample, 2 = 2x, etc.)
///
/// # Returns
/// Result with the path to the saved file, or an error message
pub fn export_png_from_state(
    fractal_state: &crate::app_state::FractalState,
    view_state: &crate::app_state::ViewState,
    color_state: &crate::app_state::ColorState,
    input_state: &crate::app_state::InputState,
    export_state: &crate::app_state::ExportState,
    render_state: &mut crate::app_state::RenderState,
    fractal: &dyn Fractal,
    scale: f32,
    supersample: u32,
) -> Result<String, String> {
    // Extract values from state structs
    let view = &view_state.view;
    let colormap = &color_state.colormap;
    let max_iterations = input_state.parse_iterations();
    let fractal_parameters = &fractal_state.parameters;
    let use_period = color_state.use_period;
    let period = input_state.parse_period();
    let use_interior_color = color_state.use_interior_color;
    let interior_color = color_state.interior_color;
    let use_log_scale = color_state.use_log_scale;
    let filter_type = export_state.filter;
    let output_dir = export_state.directory.as_ref();
    let backend = render_state.backend;

    // Initialize GPU if needed
    #[cfg(feature = "gpu")]
    if matches!(backend, crate::gpu::RenderBackend::Gpu) {
        if let Err(e) = render_state.ensure_gpu_initialized() {
            eprintln!("[ERROR] GPU initialization failed: {}", e);
        }
    }

    // Delegate to the original export_png function
    #[cfg(feature = "gpu")]
    let result = export_png(
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
        output_dir,
        backend,
        render_state.hiprec_bits,
        render_state.max_threads,
        render_state.gpu_renderer.as_mut(),
    );
    
    #[cfg(not(feature = "gpu"))]
    let result = export_png(
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
        output_dir,
        backend,
        render_state.hiprec_bits,
        render_state.max_threads,
    );
    
    result
}

/// Export settings as standalone JSON file
///
/// Creates a JSON file containing all fractal settings without rendering an image.
/// This is useful for sharing settings, version control, or CLI rendering.
///
/// # Arguments
/// * `fractal_state` - Fractal type and parameters
/// * `view_state` - View position and zoom
/// * `color_state` - Colormap and color settings
/// * `input_state` - Iterations and other numeric inputs
/// * `export_state` - Export settings (filter, directory)
/// * `output_path` - Path where JSON file will be saved
///
/// # Returns
/// Result with the path to the saved file, or an error message
pub fn export_settings_json(
    fractal_state: &crate::app_state::FractalState,
    view_state: &crate::app_state::ViewState,
    color_state: &crate::app_state::ColorState,
    input_state: &crate::app_state::InputState,
    export_state: &crate::app_state::ExportState,
    output_path: &Path,
) -> Result<String, String> {
    // Create metadata from state
    let metadata = FractalMetadata::from_app_state(
        fractal_state,
        view_state,
        color_state,
        input_state,
        export_state,
    );
    
    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    
    // Write to file
    std::fs::write(output_path, json)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;
    
    Ok(output_path.display().to_string())
}

/// Calculate the output dimensions for a given scale
pub fn calculate_output_dimensions(view: &FractalView, scale: f32) -> (u32, u32) {
    let output_width = (view.width as f32 * scale) as u32;
    let output_height = (view.height as f32 * scale) as u32;
    (output_width, output_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for tests to call export_png without GPU
    fn export_png_test(
        view: &FractalView,
        colormap: &ColorMap,
        max_iterations: u32,
        fractal: &dyn crate::fractals::Fractal,
        fractal_parameters: &std::collections::HashMap<String, f64>,
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
        #[cfg(feature = "gpu")]
        {
            export_png(
                view, colormap, max_iterations, fractal, fractal_parameters,
                use_period, period, use_interior_color, interior_color, use_log_scale,
                filter_type, supersample, scale, output_dir,
                crate::gpu::RenderBackend::Cpu,  // Always use CPU for tests
                128, // hiprec_bits (unused for Cpu backend)
                0,   // max_threads (no limit)
                None,
            )
        }
        #[cfg(not(feature = "gpu"))]
        {
            export_png(
                view, colormap, max_iterations, fractal, fractal_parameters,
                use_period, period, use_interior_color, interior_color, use_log_scale,
                filter_type, supersample, scale, output_dir,
                crate::gpu::RenderBackend::Cpu,
                128, // hiprec_bits (unused for Cpu backend)
                0,   // max_threads (no limit)
            )
        }
    }

    #[test]
    fn test_calculate_output_dimensions() {
        let view = FractalView::new(1280, 720);

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
    
    #[test]
    fn test_metadata_struct() {
        use std::collections::HashMap;
        use scala_chromatica::ColorMap;
        
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), -0.7);
        params.insert("c_imag".to_string(), 0.27);
        
        let metadata = FractalMetadata {
            fractal_type: "Julia".to_string(),
            fractal_parameters: params,
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 1280,
            height: 720,
            max_iterations: 256,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 256,
            use_interior_color: true,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "Lanczos3".to_string(),
            export_supersample: 4,
            export_scale: 3.0,
            version: Some("0.1.6".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(1234567890),
        };
        
        // Test parsing fractal type
        assert!(metadata.parse_fractal_type().is_ok());
        assert_eq!(metadata.parse_fractal_type().unwrap(), crate::app_state::FractalType::Julia);
        
        // Test parsing filter type
        assert_eq!(metadata.parse_filter_type(), FilterType::Lanczos3);
        
        // Test creating fractal view
        let view = metadata.to_fractal_view();
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 1.0);
        assert_eq!(view.width, 1280);
        assert_eq!(view.height, 720);
    }
    
    #[test]
    fn test_metadata_parse_fractal_types() {
        let test_cases = vec![
            ("Mandelbrot", crate::app_state::FractalType::Mandelbrot),
            ("Julia", crate::app_state::FractalType::Julia),
            ("Burning Ship", crate::app_state::FractalType::BurningShip),
            ("Tippets Mandelbrot", crate::app_state::FractalType::TippetsMandelbrot),
            ("Multifractal-Julia", crate::app_state::FractalType::MultifractalJulia),
        ];
        
        for (name, expected_type) in test_cases {
            let metadata = FractalMetadata {
                fractal_type: name.to_string(),
                fractal_parameters: HashMap::new(),
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
                width: 800,
                height: 600,
                max_iterations: 100,
                colormap_name: "Default".to_string(),
                colormap_data: ColorMap::default_scheme(),
                use_period: false,
                period: 256,
                use_interior_color: false,
                interior_color: [0, 0, 0],
                use_log_scale: false,
                export_filter: "None".to_string(),
                export_supersample: 1,
                export_scale: 1.0,
                version: None,
                metadata_version: None,
                created_timestamp: None,
            };
            
            assert_eq!(metadata.parse_fractal_type().unwrap(), expected_type);
        }
        
        // Test invalid fractal type
        let invalid_metadata = FractalMetadata {
            fractal_type: "NonexistentFractal".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 800,
            height: 600,
            max_iterations: 100,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 256,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: None,
            metadata_version: None,
            created_timestamp: None,
        };
        
        assert!(invalid_metadata.parse_fractal_type().is_err());
    }
    
    #[test]
    fn test_load_png_metadata_nonexistent_file() {
        let result = load_png_metadata("nonexistent_file.png");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open"));
    }
    
    #[test]
    fn test_load_png_metadata_invalid_png() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        
        // Create a temporary non-PNG file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"This is not a PNG file").unwrap();
        temp_file.flush().unwrap();
        
        let result = load_png_metadata(temp_file.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        // Accept any error message since the file is not a valid PNG
        assert!(!err_msg.is_empty(), "Expected an error message");
    }
    
    #[test]
    fn test_export_and_load_roundtrip() {
        use std::collections::HashMap;
        use tempfile::TempDir;
        use crate::fractals::Julia;
        
        // Create temporary directory for test files
        let temp_dir = TempDir::new().unwrap();
        
        // Create test fractal view with parameters
        let mut parameters = HashMap::new();
        parameters.insert("c_real".to_string(), -0.7);
        parameters.insert("c_imag".to_string(), 0.27);
        
        let view = FractalView {
            center_x: -0.5,
            center_y: 0.0,
            zoom: 2.5,
            width: 800,
            height: 600,
            parameters: parameters.clone(),
        };
        
        // Create test colormap
        let colormap = ColorMap::default_scheme();
        
        // Create Julia fractal instance
        let julia = Julia::new();
        
        // Export a PNG with metadata
        let result = export_png_test(
            &view,
            &colormap,
            256,
            &julia,
            &parameters,
            false,
            256,
            true,
            [255, 128, 64],
            false,
            FilterType::None,
            1,
            1.0,
            Some(&temp_dir.path().to_path_buf()),
        );
        
        assert!(result.is_ok(), "Export failed: {:?}", result.err());
        
        // Find the exported file
        let exported_path = result.unwrap();
        let output_path = std::path::PathBuf::from(&exported_path);
        assert!(output_path.exists(), "PNG file was not created");
        
        // Load the metadata back
        let loaded_metadata = load_png_metadata(&output_path);
        assert!(loaded_metadata.is_ok(), "Loading metadata failed: {:?}", loaded_metadata.err());
        
        let metadata = loaded_metadata.unwrap();
        
        // Verify all fields match
        assert_eq!(metadata.fractal_type, "Julia Set");
        assert_eq!(metadata.center_x, -0.5);
        assert_eq!(metadata.center_y, 0.0);
        assert_eq!(metadata.zoom, 2.5);
        assert_eq!(metadata.width, 800);
        assert_eq!(metadata.height, 600);
        assert_eq!(metadata.max_iterations, 256);
        assert_eq!(metadata.use_period, false);
        assert_eq!(metadata.period, 256);
        assert_eq!(metadata.use_interior_color, true);
        assert_eq!(metadata.interior_color, [255, 128, 64]);
        assert_eq!(metadata.use_log_scale, false);
        assert_eq!(metadata.export_filter, "None");
        assert_eq!(metadata.export_supersample, 1);
        assert_eq!(metadata.export_scale, 1.0);
        
        // Verify parameters
        assert_eq!(metadata.fractal_parameters.get("c_real"), Some(&-0.7));
        assert_eq!(metadata.fractal_parameters.get("c_imag"), Some(&0.27));
        
        // Verify version fields are present
        assert!(metadata.version.is_some());
        assert!(metadata.metadata_version.is_some());
        assert!(metadata.created_timestamp.is_some());
        assert_eq!(metadata.metadata_version.unwrap(), "1.0");
    }
    
    #[test]
    fn test_export_and_load_all_fractal_types() {
        use std::collections::HashMap;
        use tempfile::TempDir;
        use crate::app_state::FractalType;
        use crate::fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus};
        
        let temp_dir = TempDir::new().unwrap();
        let colormap = ColorMap::default_scheme();
        
        // Test Mandelbrot
        {
            let params = HashMap::new();
            let view = FractalView {
                center_x: -0.5,
                center_y: 0.0,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = Mandelbrot::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Mandelbrot export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Mandelbrot");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::Mandelbrot);
        }
        
        // Test Julia
        {
            let mut params = HashMap::new();
            params.insert("c_real".to_string(), -0.7);
            params.insert("c_imag".to_string(), 0.27);
            let view = FractalView {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = Julia::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Julia export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Julia Set");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::Julia);
            assert_eq!(loaded.fractal_parameters.get("c_real"), Some(&-0.7));
            assert_eq!(loaded.fractal_parameters.get("c_imag"), Some(&0.27));
        }
        
        // Test Burning Ship
        {
            let params = HashMap::new();
            let view = FractalView {
                center_x: -0.5,
                center_y: -0.5,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = BurningShip::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Burning Ship export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Burning Ship");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::BurningShip);
        }
        
        // Test Tippets Mandelbrot
        {
            let mut params = HashMap::new();
            params.insert("a".to_string(), -0.65);
            let view = FractalView {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = TippetsMandelbrot::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Tippets Mandelbrot export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Tippets Mandelbrot");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::TippetsMandelbrot);
            assert_eq!(loaded.fractal_parameters.get("a"), Some(&-0.65));
        }
        
        // Test Multifractal-Julia
        {
            let mut params = HashMap::new();
            params.insert("c_real".to_string(), -0.7);
            params.insert("c_imag".to_string(), 0.27);
            params.insert("power".to_string(), 2.5);
            let view = FractalView {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = MultifractalJulia::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Multifractal-Julia export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Multifractal-Julia");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::MultifractalJulia);
            assert_eq!(loaded.fractal_parameters.get("power"), Some(&2.5));
        }
        
        // Test Cactus
        {
            let params = HashMap::new();
            let view = FractalView {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
                width: 400,
                height: 300,
                parameters: params.clone(),
            };
            let fractal = Cactus::new();
            let result = export_png_test(
                &view, &colormap, 100, &fractal, &params,
                false, 256, false, [0, 0, 0], false,
                FilterType::None, 1, 1.0, Some(&temp_dir.path().to_path_buf()),
            );
            assert!(result.is_ok(), "Cactus export failed");
            let loaded = load_png_metadata(result.unwrap()).unwrap();
            assert_eq!(loaded.fractal_type, "Cactus");
            assert_eq!(loaded.parse_fractal_type().unwrap(), FractalType::Cactus);
        }
    }
    
    #[test]
    fn test_png_without_metadata() {
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        // Create a valid PNG without metadata
        let mut temp_file = NamedTempFile::new().unwrap();
        
        // Minimal valid PNG file (1x1 white pixel)
        let minimal_png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
            0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
            0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59,
            0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
            0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        
        temp_file.write_all(minimal_png).unwrap();
        temp_file.flush().unwrap();
        
        // Should fail gracefully with a clear error message
        let result = load_png_metadata(temp_file.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("metadata"),
            "Expected metadata-related error, got: {}",
            err_msg
        );
    }
    
    #[test]
    fn test_load_into_fresh_state() {
        use std::collections::HashMap;
        use tempfile::TempDir;
        use crate::fractals::Julia;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Create and export a Julia fractal with specific settings
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), -0.8);
        params.insert("c_imag".to_string(), 0.156);
        
        let view = FractalView {
            center_x: 0.5,
            center_y: -0.3,
            zoom: 3.5,
            width: 640,
            height: 480,
            parameters: params.clone(),
        };
        
        let colormap = ColorMap::default_scheme();
        let julia = Julia::new();
        
        let result = export_png_test(
            &view, &colormap, 512, &julia, &params,
            true, 128, true, [255, 0, 128], true,
            FilterType::Lanczos3, 2, 2.5,
            Some(&temp_dir.path().to_path_buf()),
        );
        
        assert!(result.is_ok(), "Export failed");
        let exported_path = result.unwrap();
        
        // Load the metadata (simulating fresh state)
        let loaded = load_png_metadata(&exported_path).unwrap();
        
        // Verify all settings loaded correctly into "fresh" state
        assert_eq!(loaded.fractal_type, "Julia Set");
        assert_eq!(loaded.center_x, 0.5);
        assert_eq!(loaded.center_y, -0.3);
        assert_eq!(loaded.zoom, 3.5);
        assert_eq!(loaded.width, 640);
        assert_eq!(loaded.height, 480);
        assert_eq!(loaded.max_iterations, 512);
        assert_eq!(loaded.use_period, true);
        assert_eq!(loaded.period, 128);
        assert_eq!(loaded.use_interior_color, true);
        assert_eq!(loaded.interior_color, [255, 0, 128]);
        assert_eq!(loaded.use_log_scale, true);
        assert_eq!(loaded.export_filter, "Lanczos3");
        assert_eq!(loaded.export_supersample, 2);
        assert_eq!(loaded.export_scale, 2.5);
        assert_eq!(loaded.fractal_parameters.get("c_real"), Some(&-0.8));
        assert_eq!(loaded.fractal_parameters.get("c_imag"), Some(&0.156));
    }
    
    #[test]
    fn test_state_replacement_different_fractal() {
        use std::collections::HashMap;
        use tempfile::TempDir;
        use crate::fractals::{Mandelbrot, BurningShip};
        
        let temp_dir = TempDir::new().unwrap();
        
        // First: Export a Mandelbrot fractal
        let mandelbrot_params = HashMap::new();
        let mandelbrot_view = FractalView {
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
            width: 800,
            height: 600,
            parameters: mandelbrot_params.clone(),
        };
        let mandelbrot = Mandelbrot::new();
        let colormap = ColorMap::default_scheme();
        
        let mandelbrot_result = export_png_test(
            &mandelbrot_view, &colormap, 256, &mandelbrot, &mandelbrot_params,
            false, 256, false, [0, 0, 0], false,
            FilterType::None, 1, 1.0,
            Some(&temp_dir.path().to_path_buf()),
        );
        assert!(mandelbrot_result.is_ok());
        let mandelbrot_path = mandelbrot_result.unwrap();
        
        // Then: Export a Burning Ship fractal
        let burning_ship_params = HashMap::new();
        let burning_ship_view = FractalView {
            center_x: -1.75,
            center_y: -0.05,
            zoom: 0.1,
            width: 1024,
            height: 768,
            parameters: burning_ship_params.clone(),
        };
        let burning_ship = BurningShip::new();
        
        let burning_ship_result = export_png_test(
            &burning_ship_view, &colormap, 512, &burning_ship, &burning_ship_params,
            true, 64, true, [128, 64, 0], false,
            FilterType::Gaussian, 4, 1.5,
            Some(&temp_dir.path().to_path_buf()),
        );
        assert!(burning_ship_result.is_ok());
        let burning_ship_path = burning_ship_result.unwrap();
        
        // Simulate: Start with Burning Ship, then load Mandelbrot
        // (In real app, this would replace GUI state)
        let loaded_mandelbrot = load_png_metadata(&mandelbrot_path).unwrap();
        
        // Verify state was "replaced" with Mandelbrot settings
        assert_eq!(loaded_mandelbrot.fractal_type, "Mandelbrot");
        assert_eq!(loaded_mandelbrot.center_x, -0.5);
        assert_eq!(loaded_mandelbrot.center_y, 0.0);
        assert_eq!(loaded_mandelbrot.zoom, 1.0);
        assert_eq!(loaded_mandelbrot.width, 800);
        assert_eq!(loaded_mandelbrot.height, 600);
        assert_eq!(loaded_mandelbrot.max_iterations, 256);
        assert_eq!(loaded_mandelbrot.export_scale, 1.0);
        
        // Now load Burning Ship (simulating second replacement)
        let loaded_burning_ship = load_png_metadata(&burning_ship_path).unwrap();
        
        // Verify state was "replaced" with Burning Ship settings
        assert_eq!(loaded_burning_ship.fractal_type, "Burning Ship");
        assert_eq!(loaded_burning_ship.center_x, -1.75);
        assert_eq!(loaded_burning_ship.center_y, -0.05);
        assert_eq!(loaded_burning_ship.zoom, 0.1);
        assert_eq!(loaded_burning_ship.width, 1024);
        assert_eq!(loaded_burning_ship.height, 768);
        assert_eq!(loaded_burning_ship.max_iterations, 512);
        assert_eq!(loaded_burning_ship.use_period, true);
        assert_eq!(loaded_burning_ship.period, 64);
        assert_eq!(loaded_burning_ship.export_supersample, 4);
        assert_eq!(loaded_burning_ship.export_scale, 1.5);
    }
}

/// Fractal metadata structure for serialization/deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractalMetadata {
    pub fractal_type: String,
    pub fractal_parameters: HashMap<String, f64>,
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub width: u32,
    pub height: u32,
    pub max_iterations: u32,
    pub colormap_name: String,
    pub colormap_data: ColorMap,
    pub use_period: bool,
    pub period: u32,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub use_log_scale: bool,
    pub export_filter: String,
    pub export_supersample: u32,
    pub export_scale: f32,
    pub version: Option<String>,
    pub metadata_version: Option<String>,
    pub created_timestamp: Option<u64>,
}

/// Load metadata from a PNG file's tEXt chunks
///
/// # Arguments
/// * `path` - Path to the PNG file
///
/// # Returns
/// Result with FractalMetadata, or an error message
pub fn load_png_metadata<P: AsRef<Path>>(path: P) -> Result<FractalMetadata, String> {
    let path = path.as_ref();
    
    // Open and decode the PNG file
    let file = File::open(path)
        .map_err(|e| format!("Failed to open PNG file: {}", e))?;
    
    let decoder = png::Decoder::new(BufReader::new(file));
    let reader = decoder.read_info()
        .map_err(|e| format!("Failed to read PNG info: {}", e))?;
    
    // Extract tEXt chunks
    let info = reader.info();
    let text_chunks = &info.uncompressed_latin1_text;
    
    // Helper function to find a text chunk by key
    let find_text = |key: &str| -> Option<String> {
        text_chunks.iter()
            .find(|chunk| chunk.keyword == key)
            .map(|chunk| chunk.text.clone())
    };
    
    // Check if this PNG has Forma Fractalis metadata
    if find_text("Metadata-Version").is_none() && find_text("Forma-Fractalis-Version").is_none() {
        return Err("This PNG does not contain Forma Fractalis metadata".to_string());
    }
    
    // Parse fractal type
    let fractal_type = find_text("Fractal-Type")
        .ok_or("Missing fractal type in metadata")?;
    
    // Parse fractal parameters (may be empty)
    let fractal_parameters: HashMap<String, f64> = find_text("Fractal-Parameters")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    
    // Parse view coordinates
    let center_x = find_text("View-CenterX")
        .and_then(|s| s.parse().ok())
        .ok_or("Missing or invalid View-CenterX")?;
    
    let center_y = find_text("View-CenterY")
        .and_then(|s| s.parse().ok())
        .ok_or("Missing or invalid View-CenterY")?;
    
    let zoom = find_text("View-Zoom")
        .and_then(|s| s.parse().ok())
        .ok_or("Missing or invalid View-Zoom")?;
    
    // Parse view dimensions (with fallback to PNG dimensions for backward compatibility)
    let width = find_text("View-Width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(info.width);
    
    let height = find_text("View-Height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(info.height);
    
    // Parse iterations
    let max_iterations = find_text("Max-Iterations")
        .and_then(|s| s.parse().ok())
        .ok_or("Missing or invalid Max-Iterations")?;
    
    // Parse colormap
    let colormap_name = find_text("Colormap-Name")
        .ok_or("Missing colormap name")?;
    
    let colormap_data: ColorMap = find_text("Colormap-Data")
        .and_then(|json| serde_json::from_str(&json).ok())
        .ok_or("Missing or invalid colormap data")?;
    
    // Parse color modulation settings
    let use_period = find_text("Color-Period-Enabled")
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);
    
    let period = find_text("Color-Period")
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);
    
    let use_interior_color = find_text("Interior-Color-Enabled")
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);
    
    let interior_color: [u8; 3] = find_text("Interior-Color-RGB")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or([0, 0, 0]);
    
    let use_log_scale = find_text("Log-Scale-Enabled")
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);
    
    // Parse export settings
    let export_filter = find_text("Export-Filter")
        .unwrap_or_else(|| "None".to_string());
    
    let export_supersample = find_text("Export-Supersample")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    
    let export_scale = find_text("Export-Scale")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3.0);
    
    // Parse version info
    let version = find_text("Forma-Fractalis-Version");
    let metadata_version = find_text("Metadata-Version");
    let created_timestamp = find_text("Created-Timestamp")
        .and_then(|s| s.parse().ok());
    
    Ok(FractalMetadata {
        fractal_type,
        fractal_parameters,
        center_x,
        center_y,
        zoom,
        width,
        height,
        max_iterations,
        colormap_name,
        colormap_data,
        use_period,
        period,
        use_interior_color,
        interior_color,
        use_log_scale,
        export_filter,
        export_supersample,
        export_scale,
        version,
        metadata_version,
        created_timestamp,
    })
}

impl FractalMetadata {
    /// Convert fractal type string to FractalType enum
    pub fn parse_fractal_type(&self) -> Result<crate::app_state::FractalType, String> {
        use crate::app_state::FractalType;
        
        match self.fractal_type.as_str() {
            "Mandelbrot" | "Mandelbrot Set" => Ok(FractalType::Mandelbrot),
            "Julia" | "Julia Set" => Ok(FractalType::Julia),
            "Burning Ship" => Ok(FractalType::BurningShip),
            "Tippets Mandelbrot" => Ok(FractalType::TippetsMandelbrot),
            "Multifractal-Julia" => Ok(FractalType::MultifractalJulia),
            "Cactus" => Ok(FractalType::Cactus),
            "Marek Dragon" => Ok(FractalType::MarekDragon),
            other => Err(format!("Unknown fractal type: {}", other)),
        }
    }
    
    /// Convert export filter string to FilterType enum
    pub fn parse_filter_type(&self) -> FilterType {
        match self.export_filter.as_str() {
            "None" => FilterType::None,
            "Lanczos3" => FilterType::Lanczos3,
            "Gaussian" => FilterType::Gaussian,
            _ => FilterType::None,
        }
    }
    
    /// Create a FractalView from metadata
    pub fn to_fractal_view(&self) -> FractalView {
        let mut view = FractalView::new(self.width, self.height);
        view.center_x = self.center_x;
        view.center_y = self.center_y;
        view.zoom = self.zoom;
        view
    }

    /// Create FractalMetadata from application state
    /// 
    /// This enables clean conversion from state structs back to metadata for export.
    /// Consolidates all metadata creation logic in one place.
    pub fn from_app_state(
        fractal_state: &crate::app_state::FractalState,
        view_state: &crate::app_state::ViewState,
        color_state: &crate::app_state::ColorState,
        input_state: &crate::app_state::InputState,
        export_state: &crate::app_state::ExportState,
    ) -> Self {
        FractalMetadata {
            fractal_type: fractal_state.fractal_type.as_str().to_string(),
            fractal_parameters: fractal_state.parameters.clone(),
            center_x: view_state.view.center_x,
            center_y: view_state.view.center_y,
            zoom: view_state.view.zoom,
            width: view_state.view.width,
            height: view_state.view.height,
            max_iterations: input_state.parse_iterations(),
            colormap_name: color_state.selected_colormap_name.clone(),
            colormap_data: color_state.colormap.clone(),
            use_period: color_state.use_period,
            period: input_state.parse_period(),
            use_interior_color: color_state.use_interior_color,
            interior_color: color_state.interior_color,
            use_log_scale: color_state.use_log_scale,
            export_filter: export_state.filter.as_str().to_string(),
            export_supersample: input_state.parse_export_supersample(),
            export_scale: input_state.parse_export_scale() as f32,
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            ),
        }
    }
}

/// Load metadata from a JSON settings file written by `export_settings_json`
///
/// # Arguments
/// * `path` - Path to the JSON file
///
/// # Returns
/// Result with FractalMetadata, or an error message
pub fn load_json_metadata<P: AsRef<Path>>(path: P) -> Result<FractalMetadata, String> {
    let path = path.as_ref();
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse JSON settings: {}", e))
}

