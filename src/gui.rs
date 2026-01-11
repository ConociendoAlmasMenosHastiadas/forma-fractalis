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

use eframe::egui;
use crate::fractal::MandelbrotView;
use crate::colorschemes::{ColorScheme, ColorMap};

/// Render the preview window dimensions section
pub fn render_dimensions_section(
    ui: &mut egui::Ui,
    width_input: &mut String,
    height_input: &mut String,
    view: &mut MandelbrotView,
    needs_redraw: &mut bool,
) {
    section_header(ui, "Preview Window Dimensions");
    
    // Width input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Width:");
        ui.add_space(8.0);
        if ui.add(egui::TextEdit::singleline(width_input).desired_width(80.0)).changed() {
            if let Ok(val) = width_input.parse::<u32>() {
                view.width = val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        
        if ui.small_button("×2").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val * 2;
                *width_input = new_val.to_string();
                view.width = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷2").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val / 2;
                *width_input = new_val.to_string();
                view.width = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("×10").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val * 10;
                *width_input = new_val.to_string();
                view.width = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷10").clicked() {
            if let Ok(val) = width_input.parse::<u32>() {
                let new_val = val / 10;
                *width_input = new_val.to_string();
                view.width = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
    });
    
    // Swap button
    ui.horizontal(|ui| {
        ui.add_space(100.0);
        if ui.button("↕").on_hover_text("Swap width and height").clicked() {
            std::mem::swap(width_input, height_input);
            std::mem::swap(&mut view.width, &mut view.height);
            *needs_redraw = true;
        }
    });
    
    // Height input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Height:");
        ui.add_space(4.0);
        if ui.add(egui::TextEdit::singleline(height_input).desired_width(80.0)).changed() {
            if let Ok(val) = height_input.parse::<u32>() {
                view.height = val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        
        if ui.small_button("×2").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val * 2;
                *height_input = new_val.to_string();
                view.height = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷2").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val / 2;
                *height_input = new_val.to_string();
                view.height = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("×10").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val * 10;
                *height_input = new_val.to_string();
                view.height = new_val.clamp(100, 4096);
                *needs_redraw = true;
            }
        }
        if ui.small_button("÷10").clicked() {
            if let Ok(val) = height_input.parse::<u32>() {
                let new_val = val / 10;
                *height_input = new_val.to_string();
                view.height = new_val.clamp(100, 4096);
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
pub fn render_fractal_settings(ui: &mut egui::Ui, iterations_input: &mut String, needs_redraw: &mut bool) {
    section_header(ui, "Fractal Settings");
    
    // Iterations input with multiply/divide buttons
    ui.horizontal(|ui| {
        ui.label("Iterations:");
        ui.add_space(5.0);
        if ui.add(egui::TextEdit::singleline(iterations_input).desired_width(80.0)).changed() {
            *needs_redraw = true;
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
    view: &mut MandelbrotView,
    needs_redraw: &mut bool,
    status_message: &mut String,
) {
    section_header(ui, "Current View");
    
    ui.label(format!("X: {:.6}", view.center_x));
    ui.label(format!("Y: {:.6}", view.center_y));
    ui.label(format!("Zoom: {:.2}x", view.zoom));
    ui.label(format!("Size: {}×{}", view.width, view.height));
    
    ui.add_space(10.0);
    
    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Reset View")).clicked() {
        view.reset();
        *needs_redraw = true;
        *status_message = String::from("Reset to defaults");
    }
}

/// Render the color scheme and color stops section
pub fn render_colormap_section(
    ui: &mut egui::Ui,
    selected_scheme: &mut ColorScheme,
    colormap: &mut ColorMap,
    needs_redraw: &mut bool,
    status_message: &mut String,
    use_period: &mut bool,
    period_input: &mut String,
    use_interior_color: &mut bool,
    interior_color: &mut [u8; 3],
) {
    section_header(ui, "Color Scheme");
    
    egui::ComboBox::from_label("")
        .selected_text(selected_scheme.as_str())
        .show_ui(ui, |ui| {
            for scheme in ColorScheme::ALL.iter() {
                if ui.selectable_value(selected_scheme, *scheme, scheme.as_str()).clicked() {
                    *colormap = scheme.to_colormap();
                    *needs_redraw = true;
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
            if ui.add(egui::TextEdit::singleline(period_input).desired_width(60.0)).changed() {
                *needs_redraw = true;
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
            if ui.add(egui::Slider::new(&mut interior_color[0], 0..=255).show_value(true)).changed() {
                *needs_redraw = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("G:");
            if ui.add(egui::Slider::new(&mut interior_color[1], 0..=255).show_value(true)).changed() {
                *needs_redraw = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("B:");
            if ui.add(egui::Slider::new(&mut interior_color[2], 0..=255).show_value(true)).changed() {
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
            ui.painter().rect_stroke(
                color_rect,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY),
            );
        });
    }
    
    ui.add_space(10.0);
    
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            *status_message = String::from("Save colormap (TODO)");
        }
        
        if ui.button("Load").clicked() {
            *status_message = String::from("Load colormap (TODO)");
        }
    });
}

/// Render action buttons section
pub fn render_actions_section(
    ui: &mut egui::Ui,
    status_message: &mut String,
) {
    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Export (TODO)")).clicked() {
        *status_message = String::from("Export (TODO)");
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
    
    let zoom_rect = egui::Rect::from_center_size(
        center,
        egui::vec2(rect_width, rect_height)
    );
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
