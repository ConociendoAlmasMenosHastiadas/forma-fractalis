//! GUI Helper Functions for egui Interface
//!
//! This module provides reusable UI building blocks that keep main.rs clean
//! and maintainable. Each function renders a specific section of the sidebar.
//!
//! # Components
//! - Dimension controls (width/height inputs)
//! - Fractal settings (iterations)
//! - View information display (coordinates, zoom level)
//! - Colormap controls (scheme picker, color stops)
//! - Action buttons (apply, reset, export)
//! - Zoom square visualization
//!
//! All functions take `&mut egui::Ui` for rendering within egui layouts.

use eframe::egui;
use crate::fractal::MandelbrotView;
use crate::colorschemes::{ColorScheme, ColorMap};

/// Render the preview window dimensions section
pub fn render_dimensions_section(ui: &mut egui::Ui, width_input: &mut String, height_input: &mut String) {
    section_header(ui, "Preview Window Dimensions");
    
    input_row(ui, "Width:", width_input, 28.0);
    input_row(ui, "Height:", height_input, 24.0);
    
    ui.add_space(5.0);
    ui.label(
        egui::RichText::new("These controls will not resize the preview window but they will control the aspect ratio and performance of the preview. See rendering below for high-res output.")
            .small()
            .italics()
            .color(egui::Color32::GRAY)
    );
}

/// Render the fractal settings section
pub fn render_fractal_settings(ui: &mut egui::Ui, iterations_input: &mut String) {
    section_header(ui, "Fractal Settings");
    
    input_row(ui, "Iterations:", iterations_input, 5.0);
}

/// Render the current view information section
pub fn render_current_view_info(ui: &mut egui::Ui, view: &MandelbrotView) {
    section_header(ui, "Current View");
    
    ui.label(format!("X: {:.6}", view.center_x));
    ui.label(format!("Y: {:.6}", view.center_y));
    ui.label(format!("Zoom: {:.2}x", view.zoom));
    ui.label(format!("Size: {}×{}", view.width, view.height));
}

/// Render the color scheme and color stops section
pub fn render_colormap_section(
    ui: &mut egui::Ui,
    selected_scheme: &mut ColorScheme,
    colormap: &mut ColorMap,
    needs_redraw: &mut bool,
    status_message: &mut String,
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
    
    ui.add_space(15.0);
    
    // Color Stops
    ui.label(egui::RichText::new("Color Stops").strong());
    ui.add_space(5.0);
    
    for (i, stop) in colormap.stops().iter().enumerate() {
        ui.label(format!("{}. {} at {:.2}", i + 1, stop.color, stop.position));
    }
    
    ui.add_space(5.0);
    
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
    view: &mut MandelbrotView,
    width_input: &str,
    height_input: &str,
    iterations_input: &str,
    needs_redraw: &mut bool,
    status_message: &mut String,
) {
    if ui.add_sized([ui.available_width(), 40.0], egui::Button::new("Apply Settings")).clicked() {
        let width: u32 = width_input.parse().unwrap_or(800).clamp(100, 4096);
        let height: u32 = height_input.parse().unwrap_or(600).clamp(100, 4096);
        let iterations: u32 = iterations_input.parse().unwrap_or(256).clamp(10, 10000);
        
        view.width = width;
        view.height = height;
        *needs_redraw = true;
        *status_message = format!("Applied: {}×{}, {} iter", width, height, iterations);
    }
    
    ui.add_space(5.0);
    
    if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Reset View")).clicked() {
        view.reset();
        *needs_redraw = true;
        *status_message = String::from("Reset to defaults");
    }
    
    ui.add_space(5.0);
    
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

/// Render a labeled input row with consistent spacing
fn input_row(ui: &mut egui::Ui, label: &str, input: &mut String, spacing: f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add_space(spacing);
        ui.add(egui::TextEdit::singleline(input).desired_width(120.0));
    });
}
