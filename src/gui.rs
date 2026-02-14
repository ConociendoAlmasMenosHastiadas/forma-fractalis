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
use scala_chromatica::ColorMap;
use scala_chromatica::io as colorschemes_io;
use crate::fractals::FractalView;
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
    fn all_types() -> Vec<Self> where Self: Sized;
    fn is_julia(&self) -> bool;
    fn is_mandelbrot(&self) -> bool;
    fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        julia_c_real_input: &str,
        julia_c_imag_input: &str,
        mandelbrot_power_input: &str,
        multifractal_julia_power_input: &str,
        marek_dragon_phi_input: &str,
    );
}

/// Helper to trigger debounced redraw (for text inputs)
fn trigger_debounced_redraw(timer: &mut Option<Instant>, pending: &mut bool) {
    *timer = Some(Instant::now());
    *pending = true;
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
    ui.label(
        egui::RichText::new("These controls will not resize the preview window but they will control the aspect ratio and performance of the preview. See rendering below for high-res output.")
            .small()
            .italics()
            .color(egui::Color32::GRAY)
    );
}

/// Render the fractal settings section
pub fn render_fractal_settings<FT>(
    ui: &mut egui::Ui,
    iterations_input: &mut String,
    needs_redraw: &mut bool,
    fractal_type: &mut FT,
    fractal_parameters: &mut HashMap<String, f64>,
    julia_c_real_input: &mut String,
    julia_c_imag_input: &mut String,
    mandelbrot_power_input: &mut String,
    multifractal_julia_power_input: &mut String,
    marek_dragon_phi_input: &mut String,
    view: &mut FractalView,
    input_debounce_timer: &mut Option<Instant>,
    pending_redraw: &mut bool,
) 
where
    FT: Copy + PartialEq + std::fmt::Debug,
    FT: FractalTypeOps,
{
    section_header(ui, "Fractal Settings");

    // Fractal type selector
    ui.horizontal(|ui| {
        ui.label("Type:");
        ui.add_space(15.0);
        
        let current_name = fractal_type.get_name();
        egui::ComboBox::from_id_source("fractal_type")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for ft in FT::all_types() {
                    if ui.selectable_value(fractal_type, ft, ft.get_name()).clicked() {
                        // Reset view to fractal's default when switching
                        ft.reset_view_and_params(
                            view,
                            fractal_parameters,
                            julia_c_real_input,
                            julia_c_imag_input,
                            mandelbrot_power_input,
                            multifractal_julia_power_input,
                            marek_dragon_phi_input
                        );
                        *needs_redraw = true;
                    }
                }
            });
    });

    ui.add_space(10.0);

    // Dynamic fractal parameters (e.g., Julia Set sliders)
    if fractal_type.is_julia() {
        ui.label(
            egui::RichText::new("Julia Set Parameters")
                .strong()
        );
        ui.add_space(5.0);
        
        // C Real slider
        let mut c_real = fractal_parameters.get("c_real").copied().unwrap_or(-0.7);
        ui.horizontal(|ui| {
            ui.label("C Real:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                .text("")
                .step_by(0.001)
                .fixed_decimals(3))
                .changed()
            {
                fractal_parameters.insert("c_real".to_string(), c_real);
                *julia_c_real_input = format!("{:.6}", c_real);
                *needs_redraw = true;
            }
        });
        
        // C Imaginary slider
        let mut c_imag = fractal_parameters.get("c_imag").copied().unwrap_or(0.27015);
        ui.horizontal(|ui| {
            ui.label("C Imag:");
            ui.add_space(3.0);
            if ui.add(egui::Slider::new(&mut c_imag, -2.0..=2.0)
                .text("")
                .step_by(0.001)
                .fixed_decimals(3))
                .changed()
            {
                fractal_parameters.insert("c_imag".to_string(), c_imag);
                *julia_c_imag_input = format!("{:.6}", c_imag);
                *needs_redraw = true;
            }
        });
        
        // Show current values as editable text inputs below sliders
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Values", |ui| {
            ui.horizontal(|ui| {
                ui.label("Real:");
                if ui
                    .add(egui::TextEdit::singleline(julia_c_real_input).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = julia_c_real_input.parse::<f64>() {
                        let clamped = val.clamp(-2.0, 2.0);
                        fractal_parameters.insert("c_real".to_string(), clamped);
                        trigger_debounced_redraw(input_debounce_timer, pending_redraw);
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Imag:");
                if ui
                    .add(egui::TextEdit::singleline(julia_c_imag_input).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = julia_c_imag_input.parse::<f64>() {
                        let clamped = val.clamp(-2.0, 2.0);
                        fractal_parameters.insert("c_imag".to_string(), clamped);
                        trigger_debounced_redraw(input_debounce_timer, pending_redraw);
                    }
                }
            });
        });
        
        ui.add_space(10.0);
    }

    // Mandelbrot power parameter
    if fractal_type.is_mandelbrot() {
        ui.label(
            egui::RichText::new("Mandelbrot Power")
                .strong()
        );
        ui.add_space(5.0);
        
        // Get parameter definition for bounds (supports arbitrary f64 range)
        let power_min = -10.0; // Can be adjusted to any f64 value
        let power_max = 10.0;  // Can be adjusted to any f64 value
        
        // Power slider (range determined by parameter bounds)
        let mut power = fractal_parameters.get("power").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Power:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut power, power_min..=power_max)
                .text("")
                .step_by(0.1)
                .fixed_decimals(1))
                .changed()
            {
                fractal_parameters.insert("power".to_string(), power);
                *mandelbrot_power_input = format!("{:.1}", power);
                *needs_redraw = true;
            }
        });
        
        // Show current value as editable text input (no clamping - full f64 range)
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Power:");
                if ui
                    .add(egui::TextEdit::singleline(mandelbrot_power_input).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = mandelbrot_power_input.parse::<f64>() {
                        // No clamping - accept any valid f64 value
                        fractal_parameters.insert("power".to_string(), val);
                        trigger_debounced_redraw(input_debounce_timer, pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new(
                format!("Range: slider [{:.1}, {:.1}], text input: full f64", power_min, power_max)
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }

    // Multifractal-Julia Power parameter
    if fractal_type.get_name() == "Multifractal-Julia" {
        ui.label(
            egui::RichText::new("Multifractal-Julia Parameters")
                .strong()
        );
        ui.add_space(5.0);
        
        let power_min = -5.0;
        let power_max = 5.0;
        
        // Power slider (k in z_{n+1} = c^k * z_n^{-2} + c)
        let mut power = fractal_parameters.get("power").copied().unwrap_or(1.0);
        ui.horizontal(|ui| {
            ui.label("Power (k):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut power, power_min..=power_max)
                .text("")
                .step_by(0.1)
                .fixed_decimals(1))
                .changed()
            {
                fractal_parameters.insert("power".to_string(), power);
                *multifractal_julia_power_input = format!("{:.1}", power);
                *needs_redraw = true;
            }
        });
        
        // Show current value as editable text input
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Power (k):");
                if ui
                    .add(egui::TextEdit::singleline(multifractal_julia_power_input).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = multifractal_julia_power_input.parse::<f64>() {
                        fractal_parameters.insert("power".to_string(), val);
                        trigger_debounced_redraw(input_debounce_timer, pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new(
                format!("Formula: z_{{n+1}} = c^k · z_n^{{-2}} + c")
            ).small().weak());
            ui.label(egui::RichText::new(
                format!("Range: slider [{:.1}, {:.1}], text input: full f64", power_min, power_max)
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }

    // Marek Dragon Phi parameter
    if fractal_type.get_name() == "Marek Dragon" {
        ui.label(
            egui::RichText::new("Marek Dragon Parameters")
                .strong()
        );
        ui.add_space(5.0);
        
        use crate::number_utils::TWO_PI;
        
        // Phi slider (rotation angle 0 to 2π)
        let mut phi = fractal_parameters.get("phi").copied().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.label("Phi (φ):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut phi, 0.0..=TWO_PI)
                .text("")
                .step_by(0.001)
                .fixed_decimals(3))
                .changed()
            {
                fractal_parameters.insert("phi".to_string(), phi);
                *marek_dragon_phi_input = format!("{:.6}", phi);
                *needs_redraw = true;
            }
        });
        
        // Show current value as editable text input
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Phi (φ):");
                if ui
                    .add(egui::TextEdit::singleline(marek_dragon_phi_input).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = marek_dragon_phi_input.parse::<f64>() {
                        let clamped = val.clamp(0.0, TWO_PI);
                        fractal_parameters.insert("phi".to_string(), clamped);
                        trigger_debounced_redraw(input_debounce_timer, pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new(
                "Formula: z_{n+1} = exp(jφ) · z_n + z_n²"
            ).small().weak());
            ui.label(egui::RichText::new(
                format!("Range: 0 to 2π ({:.6})", TWO_PI)
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }

    // Iterations input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Iterations:");
        ui.add_space(5.0);
        if ui
            .add(egui::TextEdit::singleline(iterations_input).desired_width(80.0))
            .changed()
        {
            trigger_debounced_redraw(input_debounce_timer, pending_redraw);
        }

        if ui.small_button("×2").clicked() {
            if let Ok(val) = iterations_input.parse::<u32>() {
                *iterations_input = (val * 2).to_string();
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷2").clicked() {
            if let Ok(val) = iterations_input.parse::<u32>() {
                *iterations_input = (val / 2).to_string();
                *needs_redraw = true;
            }
        }
        if ui.small_button("×10").clicked() {
            if let Ok(val) = iterations_input.parse::<u32>() {
                *iterations_input = (val * 10).to_string();
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷10").clicked() {
            if let Ok(val) = iterations_input.parse::<u32>() {
                *iterations_input = (val / 10).to_string();
                *needs_redraw = true;
            }
        }
    });
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
    interior_color_r_text: &mut String,
    interior_color_g_text: &mut String,
    interior_color_b_text: &mut String,
    use_log_scale: &mut bool,
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
    }

    ui.add_space(5.0);

    // Interior color checkbox and picker
    if ui.checkbox(use_interior_color, "Interior Color").changed() {
        *needs_redraw = true;
    }

    if *use_interior_color {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("R:");
            if ui
                .add(egui::Slider::new(&mut interior_color[0], 0..=255).fixed_decimals(0))
                .changed()
            {
                *interior_color_r_text = interior_color[0].to_string();
                *needs_redraw = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("G:");
            if ui
                .add(egui::Slider::new(&mut interior_color[1], 0..=255).fixed_decimals(0))
                .changed()
            {
                *interior_color_g_text = interior_color[1].to_string();
                *needs_redraw = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("B:");
            if ui
                .add(egui::Slider::new(&mut interior_color[2], 0..=255).fixed_decimals(0))
                .changed()
            {
                *interior_color_b_text = interior_color[2].to_string();
                *needs_redraw = true;
            }
        });

        // Color preview
        ui.horizontal(|ui| {
            ui.label("Preview:");
            let color_rect = ui.allocate_space(egui::vec2(60.0, 20.0)).1;
            ui.painter().rect_filled(
                color_rect,
                2.0,
                egui::Color32::from_rgb(interior_color[0], interior_color[1], interior_color[2]),
            );
            ui.painter()
                .rect_stroke(color_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
        });
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
    status_message: &mut String,
) {
    // Import section
    section_header(ui, "Import from PNG");
    
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::Button::new("📂 Load from PNG"),
        )
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .pick_file()
        {
            *status_message = format!("Loading from: {}", path.display());
            // Placeholder for actual loading logic (will be implemented in main.rs)
            // We'll return the path through status_message with a special prefix
            *status_message = format!("LOAD_PNG:{}", path.display());
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

        // Export the image
        match crate::export::export_png(
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
        ) {
            Ok(path) => {
                *status_message = format!("Exported to: {}", path);
            }
            Err(e) => {
                *status_message = format!("Export failed: {}", e);
            }
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
            match crate::export::export_settings_json(
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

// Helper functions

/// Render a section header with consistent styling
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).strong());
    ui.add_space(5.0);
}
