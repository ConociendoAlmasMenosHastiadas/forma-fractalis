use crate::app_state::{FractalState, ViewState, ColorState, InputState, ExportState, RenderState};
use crate::export::FractalMetadata;
use crate::fractals::Fractal;
use std::path::Path;

/// Build a FractalMetadata from the application GUI state structs.
pub fn metadata_from_app_state(
    fractal_state: &FractalState,
    view_state: &ViewState,
    color_state: &ColorState,
    input_state: &InputState,
    export_state: &ExportState,
) -> FractalMetadata {
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

/// Export a fractal image using all GUI state structs.
///
/// This is the GUI-level wrapper around the core `export_png` function.
/// It extracts render parameters from the state structs and delegates to core.
pub fn export_png_from_state(
    fractal_state: &FractalState,
    view_state: &ViewState,
    color_state: &ColorState,
    input_state: &InputState,
    export_state: &ExportState,
    render_state: &mut RenderState,
    fractal: &dyn Fractal,
    scale: f32,
    supersample: u32,
) -> Result<String, String> {
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

    #[cfg(feature = "gpu")]
    if matches!(backend, crate::gpu::RenderBackend::Gpu) {
        if let Err(e) = render_state.ensure_gpu_initialized() {
            eprintln!("[ERROR] GPU initialization failed: {}", e);
        }
    }

    #[cfg(feature = "gpu")]
    return crate::export::export_png(
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
    crate::export::export_png(
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
    )
}

/// Export fractal settings as a standalone JSON file.
pub fn export_settings_json(
    fractal_state: &FractalState,
    view_state: &ViewState,
    color_state: &ColorState,
    input_state: &InputState,
    export_state: &ExportState,
    output_path: &Path,
) -> Result<String, String> {
    let metadata = metadata_from_app_state(
        fractal_state,
        view_state,
        color_state,
        input_state,
        export_state,
    );

    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    std::fs::write(output_path, json)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

    Ok(output_path.display().to_string())
}
