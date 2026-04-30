//! GUI Helper Functions for egui Interface
//!
//! This module provides reusable UI building blocks that keep main.rs clean
//! and maintainable. Each function renders a specific section of the sidebar.
//!
//! # Components
//! - Dimension controls (width/height inputs with quick multiply/divide buttons)
//! - Fractal settings (iterations with quick multiply/divide buttons)
//! - View information display (coordinates, zoom level)
//! - Colormap controls (scheme picker, period modulation, interior color)
//! - Action buttons (reset, export)
//! - Zoom square visualization
//!
//! All functions take `&mut egui::Ui` for rendering within egui layouts.

use crate::perf_log;
use rayon;
use scala_chromatica::ColorMap;
use scala_chromatica::io as colorschemes_io;
use crate::app_state::InputState;
use crate::fractals::FractalView;
use crate::gpu::RenderBackend;
#[cfg(feature = "gpu")]
use crate::gpu::FractalRenderer;
use eframe::egui;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

/// Open a directory in the system file explorer
fn open_directory_in_explorer(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open finder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }
    
    Ok(())
}

/// Trait for fractal type operations needed by GUI
pub trait FractalTypeOps {
    fn get_name(&self) -> &str;
    fn get_equation(&self) -> &str;
    fn all_types() -> Vec<Self> where Self: Sized;
    fn is_julia(&self) -> bool;
    fn is_mandelbrot(&self) -> bool;
    fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        input: &InputState,
    );
    fn render_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    );
    fn uses_orbit_accumulation(&self) -> bool;
}

/// Helper to trigger debounced redraw (for text inputs)
fn trigger_debounced_redraw(timer: &mut Option<Instant>, pending: &mut bool) {
    *timer = Some(Instant::now());
    *pending = true;
}

/// Render the performance/rendering backend section
pub fn render_performance_section(
    ui: &mut egui::Ui,
    render_state: &mut crate::app_state::RenderState,
    fractal: &dyn crate::fractals::Fractal,
    status_message: &mut String,
    needs_redraw: &mut bool,
) {
    section_header(ui, "Performance");

    // Backend selection
    ui.horizontal(|ui| {
        ui.label("Rendering:");
        let previous_backend = render_state.backend;

        // Radio buttons for backend selection
        for backend in crate::gpu::RenderBackend::all() {
            if ui.radio_value(&mut render_state.backend, backend, backend.as_str()).clicked() {
                // Radio button was clicked (handled below via previous_backend comparison)
            }
        }

        // Initialize GPU and trigger redraw when backend changes
        #[cfg(feature = "gpu")]
        if previous_backend != render_state.backend {
            if matches!(render_state.backend, crate::gpu::RenderBackend::Gpu) {
                if let Err(e) = render_state.ensure_gpu_initialized() {
                    *status_message = format!("GPU initialization failed: {}", e);
                    render_state.backend = crate::gpu::RenderBackend::Cpu;
                } else {
                    *status_message = "GPU initialized - re-rendering to show f32 precision".to_string();
                    *needs_redraw = true;
                    crate::perf_log!("[BACKEND] Switched to GPU - will re-render");
                }
            } else {
                *status_message = format!("Switched to {} - re-rendering", render_state.backend.as_str());
                *needs_redraw = true;
                crate::perf_log!("[BACKEND] Switched to {} - will re-render", render_state.backend.as_str());
            }
        }
        #[cfg(not(feature = "gpu"))]
        if previous_backend != render_state.backend {
            *status_message = format!("Switched to {} - re-rendering", render_state.backend.as_str());
            *needs_redraw = true;
        }
    });

    // Hi-Prec bit-width selector: only shown when CpuHiPrec is the selected backend
    if matches!(render_state.backend, crate::gpu::RenderBackend::CpuHiPrec) {
        ui.horizontal(|ui| {
            ui.label("Precision:");
            let prev_bits = render_state.hiprec_bits;
            egui::ComboBox::from_id_source("hiprec_bits")
                .selected_text(format!("{}-bit", render_state.hiprec_bits))
                .show_ui(ui, |ui| {
                    for &bits in crate::gpu::HIPREC_BIT_OPTIONS {
                        ui.selectable_value(
                            &mut render_state.hiprec_bits,
                            bits,
                            format!("{}-bit", bits),
                        );
                    }
                });
            if render_state.hiprec_bits != prev_bits {
                *needs_redraw = true;
                crate::perf_log!("[BACKEND] Hi-Prec bits changed to {} - will re-render", render_state.hiprec_bits);
            }
        });
    }

    // Backend explanation
    let explanation = match render_state.backend {
        crate::gpu::RenderBackend::Cpu => "CPU: f64 precision, compatible with all fractals",
        crate::gpu::RenderBackend::CpuHiPrec => "CPU Hi-Prec: software bigfloat, slow but enables deep zoom",
        #[cfg(feature = "gpu")]
        crate::gpu::RenderBackend::Gpu => "GPU: f32 precision (faster, shows precision artifacts at deep zoom)",
    };
    ui.label(
        egui::RichText::new(explanation)
            .small()
            .italics()
            .color(egui::Color32::GRAY),
    );

    // Hi-Prec support warning for current fractal
    if matches!(render_state.backend, crate::gpu::RenderBackend::CpuHiPrec) {
        // Show auto-downscale notice when saved dimensions exist
        if render_state.hiprec_preview_saved.is_some() {
            ui.label(
                egui::RichText::new("Preview auto-scaled to 1/2 resolution. Export scale doubled to compensate.")
                    .small()
                    .color(egui::Color32::from_rgb(200, 160, 0)),
            );
        }
        if !fractal.supports_hiprec() {
            ui.label(
                egui::RichText::new(format!(
                    "WARNING: '{}' does not support CPU Hi-Prec. Rendering will fail. Switch to CPU or GPU.",
                    fractal.name()
                ))
                .small()
                .color(egui::Color32::from_rgb(220, 60, 60)),
            );
        } else {
            ui.label(
                egui::RichText::new(format!("Hi-Prec ready for '{}'", fractal.name()))
                    .small()
                    .color(egui::Color32::from_rgb(0, 180, 0)),
            );
        }
    }

    // CPU thread limit slider: only meaningful for CPU-side backends
    if !matches!(render_state.backend, crate::gpu::RenderBackend::Gpu) {
        ui.add_space(4.0);
        let max_cores = rayon::current_num_threads();
        // Map sentinel 0 ("all") to max_cores for slider display
        let mut slider_val = if render_state.max_threads == 0 {
            max_cores
        } else {
            render_state.max_threads.min(max_cores)
        };
        let prev_threads = render_state.max_threads;
        ui.horizontal(|ui| {
            ui.label("CPU Threads:");
            if ui.add(
                egui::Slider::new(&mut slider_val, 1..=max_cores)
            ).changed() {
                // Slider at max means "use all" — store as sentinel 0
                render_state.max_threads = if slider_val >= max_cores { 0 } else { slider_val };
                if render_state.max_threads != prev_threads {
                    *needs_redraw = true;
                }
            }
            let label = if render_state.max_threads == 0 {
                format!("(All: {})", max_cores)
            } else {
                format!("/ {}", max_cores)
            };
            ui.label(egui::RichText::new(label).small().weak().color(egui::Color32::GRAY));
        });
        ui.label(
            egui::RichText::new("Limit threads to reduce CPU load — useful when running other tasks alongside hi-prec rendering")
                .small()
                .weak()
                .color(egui::Color32::GRAY)
        );
    }

    // GPU status
    #[cfg(feature = "gpu")]
    if matches!(render_state.backend, crate::gpu::RenderBackend::Gpu) {
        ui.add_space(5.0);

        let fractal_supported = render_state.gpu_renderer.as_ref()
            .map(|renderer| renderer.supports_fractal(fractal.name()))
            .unwrap_or(false);

        if render_state.gpu_renderer.is_none() {
            ui.label(
                egui::RichText::new("✗ GPU not initialized")
                    .small()
                    .color(egui::Color32::from_rgb(200, 0, 0)),
            );
        } else if !fractal_supported {
            ui.label(
                egui::RichText::new(format!("⚠ GPU not available for {} - will use CPU", fractal.name()))
                    .small()
                    .color(egui::Color32::from_rgb(255, 165, 0)),
            );
        } else {
            ui.label(
                egui::RichText::new("✓ GPU ready")
                    .small()
                    .color(egui::Color32::from_rgb(0, 200, 0)),
            );
        }
    }
}

/// Render the preview window dimensions section
pub fn render_dimensions_section(
    ui: &mut egui::Ui,
    width_input: &mut String,
    height_input: &mut String,
    view: &mut FractalView,
    needs_redraw: &mut bool,
    input_debounce_timer: &mut Option<Instant>,
    pending_redraw: &mut bool,
    // Display zoom factor for the preview panel ([0.1, 1.0]). 1.0 = fit-to-panel.
    preview_zoom: &mut f32,
) {
    section_header(ui, "Preview Window Dimensions");

    // Width input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Width:");
        ui.add_space(8.0);
        if ui
            .add(egui::TextEdit::singleline(width_input).desired_width(80.0))
            .changed()
        {
            if let Ok(val) = width_input.parse::<u32>() {
                view.width = val.max(100);
                trigger_debounced_redraw(input_debounce_timer, pending_redraw);
            }
        }

        if ui.small_button("×2").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val * 2;
                *width_input = new_val.to_string();
                view.width = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷2").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val / 2;
                *width_input = new_val.to_string();
                view.width = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("×10").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val * 10;
                *width_input = new_val.to_string();
                view.width = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷10").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val / 10;
                *width_input = new_val.to_string();
                view.width = new_val.max(100);
                *needs_redraw = true;
            }
        }
    });

    // Swap button
    ui.horizontal(|ui| {
        ui.add_space(100.0);
        if ui
            .button("↕")
            .on_hover_text("Swap width and height")
            .clicked()
        {
            std::mem::swap(width_input, height_input);
            std::mem::swap(&mut view.width, &mut view.height);
            *needs_redraw = true;
        }
    });

    // Height input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Height:");
        ui.add_space(4.0);
        if ui
            .add(egui::TextEdit::singleline(height_input).desired_width(80.0))
            .changed()
        {
            if let Ok(val) = height_input.parse::<u32>() {
                view.height = val.max(100);
                trigger_debounced_redraw(input_debounce_timer, pending_redraw);
            }
        }

        if ui.small_button("×2").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val * 2;
                *height_input = new_val.to_string();
                view.height = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷2").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val / 2;
                *height_input = new_val.to_string();
                view.height = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("×10").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val * 10;
                *height_input = new_val.to_string();
                view.height = new_val.max(100);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷10").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val / 10;
                *height_input = new_val.to_string();
                view.height = new_val.max(100);
                *needs_redraw = true;
            }
        }
    });

    ui.add_space(5.0);

    // Preview display zoom slider
    ui.horizontal(|ui| {
        ui.label("Preview Zoom:");
        let response = ui.add(
            egui::Slider::new(preview_zoom, 0.1_f32..=1.0_f32)
                .step_by(0.05)
                .fixed_decimals(2),
        );
        if response.changed() {
            // zoom is a display-only property — no re-render needed, just repaint
        }
        if ui.small_button("1:1").on_hover_text("Fit preview to panel").clicked() {
            *preview_zoom = 1.0;
        }
    });
    ui.label(
        egui::RichText::new("Preview Zoom scales how much of the panel the preview occupies (does not affect render resolution or export).")
            .small()
            .italics()
            .color(egui::Color32::GRAY)
    );

    ui.add_space(5.0);
    ui.label(
        egui::RichText::new("Width/height control render resolution and aspect ratio. See Export below for high-res output.")
            .small()
            .italics()
            .color(egui::Color32::GRAY)
    );
}
pub fn render_fractal_settings<FT>(
    ui: &mut egui::Ui,
    input_state: &mut InputState,
    needs_redraw: &mut bool,
    fractal_type: &mut FT,
    fractal_parameters: &mut HashMap<String, f64>,
    view: &mut FractalView,
    render_backend: RenderBackend,
) 
where
    FT: Copy + PartialEq + std::fmt::Debug,
    FT: FractalTypeOps,
{
    section_header(ui, "Fractal Settings");

    // Fractal type selector
    ui.label("Type:");
    let current_name = fractal_type.get_name();
    egui::ComboBox::from_id_source("fractal_type")
        .selected_text(current_name)
        .width(ui.available_width() - 4.0)
        .show_ui(ui, |ui| {
            for ft in FT::all_types() {
                if ui.selectable_value(fractal_type, ft, ft.get_name()).clicked() {
                    // Reset view to fractal's default when switching
                    ft.reset_view_and_params(
                        view,
                        fractal_parameters,
                        input_state,
                    );
                    *needs_redraw = true;
                }
            }
        });

    // Display the fractal's mathematical equation
    let equation = fractal_type.get_equation();
    if !equation.is_empty() {
        ui.label(
            egui::RichText::new(equation)
                .weak()
                .italics()
                .size(14.0)
        );
    }

    ui.add_space(10.0);

    // Dynamic fractal parameters using FractalGUI trait
    fractal_type.render_gui(
        ui,
        fractal_parameters,
        input_state,
        needs_redraw,
    );

    // Iterations input with multiply/divide buttons — hidden for orbit-accumulation fractals
    // (those fractals expose their own Samples control in their parameter section above)
    if !fractal_type.uses_orbit_accumulation() {
        ui.horizontal(|ui| {
            ui.label("Iterations:");
            ui.add_space(5.0);
            if ui
                .add(egui::TextEdit::singleline(&mut input_state.iterations).desired_width(80.0))
                .changed()
            {
                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
            }

            if ui.small_button("×2").clicked() {
                if let Ok(val) = input_state.iterations.parse::<u32>() {
                    input_state.iterations = (val * 2).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("÷2").clicked() {
                if let Ok(val) = input_state.iterations.parse::<u32>() {
                    input_state.iterations = (val / 2).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("×10").clicked() {
                if let Ok(val) = input_state.iterations.parse::<u32>() {
                    input_state.iterations = (val * 10).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("÷10").clicked() {
                if let Ok(val) = input_state.iterations.parse::<u32>() {
                    input_state.iterations = (val / 10).to_string();
                    *needs_redraw = true;
                }
            }
        });

        // GPU iteration safety warning
        #[cfg(feature = "gpu")]
        if matches!(render_backend, RenderBackend::Gpu) {
            if let Ok(iter_val) = input_state.iterations.parse::<u32>() {
                if iter_val > crate::gpu::GPU_MAX_SAFE_ITERATIONS {
                    ui.label(
                        egui::RichText::new(format!(
                            "WARNING: {} iterations exceeds GPU safe limit ({}). \
                             GPU will clamp to {}. Use CPU mode for higher iterations.",
                            iter_val,
                            crate::gpu::GPU_MAX_SAFE_ITERATIONS,
                            crate::gpu::GPU_MAX_SAFE_ITERATIONS,
                        ))
                        .small()
                        .color(egui::Color32::from_rgb(255, 165, 0)),
                    );
                }
            }
        }
    }
    // Suppress unused variable warning when GPU feature is disabled
    #[cfg(not(feature = "gpu"))]
    let _ = render_backend;
}

/// Render the current view information section
pub fn render_current_view_info(
    ui: &mut egui::Ui,
    view: &mut FractalView,
    needs_redraw: &mut bool,
    status_message: &mut String,
    _input_debounce_timer: &mut Option<Instant>,
    _pending_redraw: &mut bool,
) {
    section_header(ui, "Current View");

    ui.label(format!("X: {:.6}", view.center_x));
    ui.label(format!("Y: {:.6}", view.center_y));
    ui.label(format!("Zoom: {:.2}x", view.zoom));
    ui.label(format!("Size: {}×{}", view.width, view.height));

    ui.add_space(10.0);

    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::Button::new("Reset View"),
        )
        .clicked()
    {
        view.reset();
        *needs_redraw = true;
        *status_message = String::from("Reset to defaults");
    }
}

/// Render the color scheme and color stops section
pub fn render_colormap_section(
    ui: &mut egui::Ui,
    available_colormaps: &mut Vec<String>,
    selected_colormap_name: &mut String,
    colormap: &mut ColorMap,
    needs_redraw: &mut bool,
    status_message: &mut String,
    use_period: &mut bool,
    period_input: &mut String,
    use_interior_color: &mut bool,
    interior_color: &mut [u8; 3],
    interior_picker: &mut crate::color_picker::ColorPickerState,
    use_log_scale: &mut bool,
    color_offset: &mut u32,
    input_debounce_timer: &mut Option<Instant>,
    pending_redraw: &mut bool,
) {
    section_header(ui, "Color Scheme");

    egui::ComboBox::from_label("")
        .selected_text(selected_colormap_name.as_str())
        .show_ui(ui, |ui| {
            for colormap_name in available_colormaps.iter() {
                if ui
                    .selectable_label(*selected_colormap_name == *colormap_name, colormap_name)
                    .clicked()
                {
                    perf_log!("[DEBUG] Attempting to load colormap: '{}'", colormap_name);
                    match colorschemes_io::load_colormap(colormap_name) {
                        Ok(loaded_colormap) => {
                            perf_log!("[DEBUG] Successfully loaded: {} with {} stops", loaded_colormap.name, loaded_colormap.stops.len());
                            *colormap = loaded_colormap;
                            *selected_colormap_name = colormap_name.clone();
                            *needs_redraw = true;
                            *status_message = format!("Loaded colormap: {}", colormap_name);
                        }
                        Err(e) => {
                            perf_log!("[DEBUG] Failed to load '{}': {}", colormap_name, e);
                            *status_message = format!("Failed to load colormap: {} - {}", colormap_name, e);
                        }
                    }
                }
            }
        });

    ui.add_space(10.0);

    // Period modulation checkbox and input
    if ui.checkbox(use_period, "Period").changed() {
        *needs_redraw = true;
    }

    if *use_period {
        ui.horizontal(|ui| {
            if ui
                .add(egui::TextEdit::singleline(period_input).desired_width(60.0))
                .changed()
            {
                trigger_debounced_redraw(input_debounce_timer, pending_redraw);
            }

            if ui.small_button("×2").clicked() {
                if let Ok(val) = period_input.parse::<u32>() {
                    *period_input = (val * 2).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("÷2").clicked() {
                if let Ok(val) = period_input.parse::<u32>() {
                    *period_input = (val / 2).max(1).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("×10").clicked() {
                if let Ok(val) = period_input.parse::<u32>() {
                    *period_input = (val * 10).to_string();
                    *needs_redraw = true;
                }
            }
            if ui.small_button("÷10").clicked() {
                if let Ok(val) = period_input.parse::<u32>() {
                    *period_input = (val / 10).max(1).to_string();
                    *needs_redraw = true;
                }
            }
        });

        // Color offset slider — only meaningful when period is active
        let period_val = period_input.parse::<u32>().unwrap_or(128).max(1);
        ui.horizontal(|ui| {
            ui.label("Color Offset:");
            if ui
                .add(egui::Slider::new(color_offset, 0..=period_val.saturating_sub(1)))
                .changed()
            {
                *needs_redraw = true;
            }
            if ui.small_button("Reset").clicked() {
                *color_offset = 0;
                *needs_redraw = true;
            }
        });
    }

    ui.add_space(5.0);

    // Interior color checkbox and picker
    if ui.checkbox(use_interior_color, "Interior Color").changed() {
        *needs_redraw = true;
    }

    if *use_interior_color {
        ui.add_space(5.0);

        if crate::color_picker::show_color_picker(
            ui,
            interior_picker,
            interior_color,
            "interior",
        ) {
            *needs_redraw = true;
        }
    }

    ui.add_space(5.0);

    // Logarithmic scaling checkbox
    if ui.checkbox(use_log_scale, "Logarithmic Scale").changed() {
        *needs_redraw = true;
    }
}

/// Render action buttons section
pub fn render_actions_section(
    ui: &mut egui::Ui,
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
    export_scale_input: &mut String,
    export_directory: &mut Option<std::path::PathBuf>,
    export_filter: &mut crate::filtering::FilterType,
    export_supersample_input: &mut String,
    render_state: &mut crate::app_state::RenderState,
    status_message: &mut String,
    last_export_path: &mut Option<std::path::PathBuf>,
    _needs_redraw: &mut bool,
) {
    // Import section
    section_header(ui, "Import");
    
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::Button::new("📂 Load from PNG or JSON"),
        )
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractal files", &["png", "json"])
            .add_filter("PNG Image", &["png"])
            .add_filter("JSON Settings", &["json"])
            .pick_file()
        {
            *status_message = format!("LOAD_FILE:{}", path.display());
        }
    }
    
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);
    
    // Export section
    section_header(ui, "Export Image");

    // Scale input
    ui.horizontal(|ui| {
        ui.label("Scale:");
        ui.add(egui::TextEdit::singleline(export_scale_input).desired_width(60.0));
    });

    ui.add_space(5.0);

    // Calculate and display output dimensions
    let scale = export_scale_input
        .parse::<f32>()
        .unwrap_or(3.0)
        .max(0.1);
    let (output_width, output_height) = crate::export::calculate_output_dimensions(view, scale);

    ui.label(
        egui::RichText::new(format!(
            "Output image size: {}×{}",
            output_width, output_height
        ))
        .small()
        .italics(),
    );

    ui.add_space(10.0);

    // Directory selection
    ui.horizontal(|ui| {
        if ui.button("📁 Choose Directory").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                *export_directory = Some(path);
                *status_message = format!(
                    "Export directory set to: {}",
                    export_directory.as_ref().unwrap().display()
                );
            }
        }

        if export_directory.is_some() {
            if ui.button("✖ Clear").clicked() {
                *export_directory = None;
                *status_message =
                    String::from("Export directory cleared (using current directory)");
            }
            
            if ui.button("📂 Open Directory").clicked() {
                if let Some(dir) = export_directory {
                    let result = open_directory_in_explorer(dir);
                    *status_message = match result {
                        Ok(_) => format!("Opened directory: {}", dir.display()),
                        Err(e) => format!("Failed to open directory: {}", e),
                    };
                }
            }
        }
    });

    // Show current directory
    if let Some(dir) = export_directory {
        ui.label(
            egui::RichText::new(format!("📂 {}", dir.display()))
                .small()
                .italics(),
        );
    } else {
        ui.label(
            egui::RichText::new("📂 Current directory")
                .small()
                .italics(),
        );
    }

    ui.add_space(10.0);

    // Filter selection
    ui.horizontal(|ui| {
        ui.label("Filter:");
        egui::ComboBox::from_id_source("export_filter")
            .selected_text(export_filter.as_str())
            .show_ui(ui, |ui| {
                for filter in crate::filtering::FilterType::ALL.iter() {
                    ui.selectable_value(export_filter, *filter, filter.as_str());
                }
            });
    });

    // Supersample input (only when filter is enabled)
    if *export_filter != crate::filtering::FilterType::None {
        ui.horizontal(|ui| {
            ui.label("Supersample:");
            ui.add(egui::TextEdit::singleline(export_supersample_input).desired_width(40.0));
        });
        
        ui.label(
            egui::RichText::new("⚡ Higher supersample = better quality but slower")
                .small()
                .italics()
                .color(egui::Color32::GRAY),
        );
    }

    ui.label(
        egui::RichText::new("ℹ Filter applied on export only (preview unaffected)")
            .small()
            .italics()
            .color(egui::Color32::DARK_GRAY),
    );

    ui.add_space(10.0);

    // Export button
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::Button::new("💾 Export PNG"),
        )
        .clicked()
    {
        // Parse supersample value
        let supersample = export_supersample_input
            .parse::<u32>()
            .unwrap_or(1)
            .max(1);

        // Initialize GPU if needed
        #[cfg(feature = "gpu")]
        if matches!(render_state.backend, crate::gpu::RenderBackend::Gpu) {
            if let Err(e) = render_state.ensure_gpu_initialized() {
                *status_message = format!("GPU initialization failed: {}", e);
            }
        }

        // Export the image
        #[cfg(feature = "gpu")]
        let result = crate::export::export_png(
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
            *export_filter,
            supersample,
            scale,
            export_directory.as_ref(),
            render_state.backend,
            render_state.hiprec_bits,
            render_state.max_threads,
            render_state.gpu_renderer.as_mut(),
        );
        
        #[cfg(not(feature = "gpu"))]
        let result = crate::export::export_png(
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
            *export_filter,
            supersample,
            scale,
            export_directory.as_ref(),
            render_state.backend,
            render_state.hiprec_bits,
            render_state.max_threads,
        );

        match result {
            Ok(path) => {
                *last_export_path = Some(std::path::PathBuf::from(&path));
                *status_message = format!("Exported to: {}", path);
            }
            Err(e) => {
                *status_message = format!("Export failed: {}", e);
            }
        }
    }

    // Open last exported image button
    ui.add_space(5.0);
    let can_open = last_export_path.as_ref().map_or(false, |p| p.exists());
    let open_button = egui::Button::new("Open Last Export");
    if ui.add_enabled(can_open, open_button).clicked() {
        if let Some(path) = last_export_path {
            if let Err(e) = open::that(path.as_path()) {
                *status_message = format!("Failed to open file: {}", e);
            }
        }
    }
    if let Some(path) = last_export_path.as_ref() {
        if !can_open {
            ui.label(egui::RichText::new(format!("Last export: {} (not found)", path.display())).small().weak());
        }
    }
}

/// Render export settings JSON button
pub fn render_export_json_button(
    ui: &mut egui::Ui,
    fractal_state: &crate::app_state::FractalState,
    view_state: &crate::app_state::ViewState,
    color_state: &crate::app_state::ColorState,
    input_state: &crate::app_state::InputState,
    export_state: &crate::app_state::ExportState,
    status_message: &mut String,
) {
    ui.add_space(10.0);
    
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::Button::new("📝 Export Settings (JSON)"),
        )
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("fractal_settings.json")
            .save_file()
        {
            match crate::export_helpers::export_settings_json(
                fractal_state,
                view_state,
                color_state,
                input_state,
                export_state,
                &path,
            ) {
                Ok(saved_path) => {
                    *status_message = format!("✓ Settings saved to: {}", saved_path);
                }
                Err(e) => {
                    *status_message = format!("❌ Export failed: {}", e);
                }
            }
        }
    }
}

/// Render the zoom square when dragging
pub fn render_zoom_square(
    ui: &mut egui::Ui,
    center: egui::Pos2,
    square_size: f32,
    aspect_ratio: f32,
) {
    let rect_width = square_size;
    let rect_height = square_size / aspect_ratio;

    let zoom_rect = egui::Rect::from_center_size(center, egui::vec2(rect_width, rect_height));
    ui.painter().rect_stroke(
        zoom_rect,
        0.0,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 255)), // Magenta
    );
}

/// Render animation generation section
/// Returns true if animation generation should be started
/// Action requested from the animation section UI
pub enum AnimationAction {
    None,
    Start,
    Cancel,
}

pub fn render_animation_section(
    ui: &mut egui::Ui,
    animation_state: &mut crate::app_state::AnimationState,
    _view_state: &crate::app_state::ViewState,
    _color_state: &crate::app_state::ColorState,
    _fractal_state: &crate::app_state::FractalState,
    _max_iterations: u32,
    export_scale: &str,
    export_filter: &crate::filtering::FilterType,
    export_supersample: &str,
    export_directory: Option<&std::path::PathBuf>,
    status_message: &mut String,
) -> AnimationAction {
    use crate::app_state::AnimationType;
    
    section_header(ui, "Animation");
    
    // Animation type selector
    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_source("anim_type")
            .selected_text(animation_state.animation_type.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut animation_state.animation_type, AnimationType::Zoom, "Zoom Sequence");
                ui.selectable_value(&mut animation_state.animation_type, AnimationType::IterationFade, "Iteration Fade-In");
                // Julia Parameter Sweep hidden - planned for v0.3.x once project variables system is in place
            });
    });
    
    ui.add_space(10.0);
    
    // General settings
    ui.label(egui::RichText::new("General Settings").small().italics());
    
    ui.horizontal(|ui| {
        ui.label("Frames:");
        if ui.add(egui::TextEdit::singleline(&mut animation_state.num_frames_text).desired_width(80.0)).changed() {
            if let Ok(val) = animation_state.num_frames_text.parse::<u32>() {
                animation_state.num_frames = val;
            }
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("FPS:");
        if ui.add(egui::TextEdit::singleline(&mut animation_state.fps_text).desired_width(80.0)).changed() {
            if let Ok(val) = animation_state.fps_text.parse::<u8>() {
                animation_state.fps = val;
            }
        }
    });
    
    ui.add_space(10.0);
    
    // Animation-specific parameters
    match animation_state.animation_type {
        AnimationType::Zoom => {
            ui.label(egui::RichText::new("Zoom Parameters").small().italics());
            
            ui.horizontal(|ui| {
                ui.label("From Zoom:");
                if ui.add(egui::TextEdit::singleline(&mut animation_state.zoom_from_text).desired_width(100.0)).changed() {
                    if let Ok(val) = animation_state.zoom_from_text.parse::<f64>() {
                        animation_state.zoom_from = val;
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("To Zoom:");
                if ui.add(egui::TextEdit::singleline(&mut animation_state.zoom_to_text).desired_width(100.0)).changed() {
                    if let Ok(val) = animation_state.zoom_to_text.parse::<f64>() {
                        animation_state.zoom_to = val;
                    }
                }
            });
            
            ui.label(egui::RichText::new("Center: Current view position").small().color(egui::Color32::GRAY));
        }
        
        AnimationType::JuliaParamSweep => {
            // Not reachable from the UI (hidden until v0.3.x), but handle
            // gracefully in case state was loaded from an old session file.
            ui.label(egui::RichText::new(
                "Julia Parameter Sweep is not available in this version. Select a different animation type."
            ).small().color(egui::Color32::YELLOW));
        }
        
        AnimationType::IterationFade => {
            ui.label(egui::RichText::new("Iteration Fade Parameters").small().italics());
            
            ui.horizontal(|ui| {
                ui.label("From Iterations:");
                if ui.add(egui::TextEdit::singleline(&mut animation_state.iter_from_text).desired_width(100.0)).changed() {
                    if let Ok(val) = animation_state.iter_from_text.parse::<u32>() {
                        animation_state.iter_from = val;
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("To Iterations:");
                if ui.add(egui::TextEdit::singleline(&mut animation_state.iter_to_text).desired_width(100.0)).changed() {
                    if let Ok(val) = animation_state.iter_to_text.parse::<u32>() {
                        animation_state.iter_to = val;
                    }
                }
            });
        }
    }
    
    ui.add_space(10.0);
    
    // Show current export settings being used
    ui.label(egui::RichText::new("Export Settings (from PNG section):").small().italics());
    let scale_val: f64 = export_scale.parse().unwrap_or(1.0);
    let supersample_val: u32 = export_supersample.parse().unwrap_or(1);
    ui.label(egui::RichText::new(format!(
        "Scale: {:.1}x, Filter: {}, Supersample: {}x",
        scale_val,
        export_filter.as_str(),
        supersample_val
    )).small().color(egui::Color32::GRAY));

    match export_directory {
        Some(dir) => ui.label(egui::RichText::new(format!("Output: {}", dir.display())).small().color(egui::Color32::GRAY)),
        None => ui.label(egui::RichText::new("No output directory set - choose one in the Export Image section above.").small().color(egui::Color32::YELLOW)),
    };
    
    ui.add_space(10.0);
    
    // Generate button (disabled if generating or no output directory)
    let can_generate = export_directory.is_some() && !animation_state.generating;

    let action = if animation_state.generating {
        if ui.button("Cancel").clicked() {
            AnimationAction::Cancel
        } else {
            AnimationAction::None
        }
    } else if ui.add_enabled(can_generate, egui::Button::new("Generate GIF")).clicked() {
        *status_message = "Starting animation generation...".to_string();
        animation_state.generating = true;
        animation_state.progress = 0.0;
        animation_state.progress_message = String::new();
        AnimationAction::Start
    } else {
        AnimationAction::None
    };

    // Progress indicator
    if animation_state.generating {
        ui.add_space(5.0);
        ui.add(egui::ProgressBar::new(animation_state.progress).text(&animation_state.progress_message));
    }

    action
}

// Helper functions

/// Render a section header with consistent styling
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).strong());
    ui.add_space(5.0);
}
