//! Advanced Color Editing Widgets
//!
//! This module provides interactive color editing capabilities for creating
//! and modifying ColorMaps with real-time visual feedback.
//!
//! # Features
//! - **Gradient Preview**: Live visualization of the current colormap
//! - **Color Stop Editor**: Add, edit, delete color stops
//! - **RGB Sliders**: Precise color adjustment
//! - **Position Control**: Adjust stop positions along gradient
//!
//! # State Management
//! The `ColorEditor` struct maintains editing state between frames,
//! tracking the selected stop and temporary edit values.
//!
//! # Usage
//! ```rust
//! let mut editor = ColorEditor::new();
//! if render_color_editor_section(ui, &mut colormap, &mut editor) {
//!     // Colormap was modified, trigger re-render
//! }
//! ```

use eframe::egui;
use crate::colorschemes::{Color, ColorStop, ColorMap};

/// State for the color editor
pub struct ColorEditor {
    pub selected_stop_index: Option<usize>,
    pub temp_rgb: [u8; 3],
    pub temp_position: f64,
}

impl Default for ColorEditor {
    fn default() -> Self {
        Self {
            selected_stop_index: None,
            temp_rgb: [0, 0, 0],
            temp_position: 0.0,
        }
    }
}

impl ColorEditor {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render a gradient preview bar showing the colormap
pub fn render_gradient_preview(ui: &mut egui::Ui, colormap: &ColorMap, width: f32, height: f32) {
    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(width, height),
        egui::Sense::hover(),
    );
    
    // Sample the gradient at many points for smooth display
    let num_samples = width as usize;
    let mut mesh = egui::epaint::Mesh::default();
    
    for i in 0..num_samples {
        let t = i as f64 / (num_samples - 1) as f64;
        let color = colormap.get_color(t);
        
        let x = rect.min.x + (t as f32) * width;
        let color32 = egui::Color32::from_rgb(color.r, color.g, color.b);
        
        // Create a vertical strip
        let strip_width = width / num_samples as f32 + 1.0; // +1 to avoid gaps
        let strip_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.min.y),
            egui::vec2(strip_width, height),
        );
        
        mesh.add_colored_rect(strip_rect, color32);
    }
    
    ui.painter().add(egui::Shape::Mesh(mesh));
    
    // Draw border
    ui.painter().rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::GRAY),
    );
    
    // Draw color stop markers
    for stop in colormap.stops() {
        let x = rect.min.x + (stop.position as f32) * width;
        let marker_pos = egui::pos2(x, rect.min.y + height);
        
        // Draw triangle marker pointing up
        let size = 6.0;
        let points = vec![
            egui::pos2(x, marker_pos.y),
            egui::pos2(x - size, marker_pos.y + size),
            egui::pos2(x + size, marker_pos.y + size),
        ];
        
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            egui::Color32::WHITE,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        ));
    }
}

/// Render color stop list with edit/delete buttons
pub fn render_color_stops_list(
    ui: &mut egui::Ui,
    colormap: &mut ColorMap,
    editor: &mut ColorEditor,
) -> bool {
    let mut changed = false;
    let mut stop_to_remove = None;
    
    ui.label(egui::RichText::new("Color Stops").strong());
    ui.add_space(5.0);
    
    let stops = colormap.stops().to_vec(); // Clone to avoid borrow issues
    
    for (i, stop) in stops.iter().enumerate() {
        ui.horizontal(|ui| {
            // Position indicator
            ui.label(format!("{:.2}", stop.position));
            
            // Color preview box
            let color_rect = ui.allocate_space(egui::vec2(30.0, 20.0)).1;
            ui.painter().rect_filled(
                color_rect,
                2.0,
                egui::Color32::from_rgb(stop.color.r, stop.color.g, stop.color.b),
            );
            ui.painter().rect_stroke(
                color_rect,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY),
            );
            
            // RGB values
            ui.label(format!("RGB({},{},{})", stop.color.r, stop.color.g, stop.color.b));
            
            // Edit button
            if ui.small_button("✏").on_hover_text("Edit").clicked() {
                editor.selected_stop_index = Some(i);
                editor.temp_rgb = [stop.color.r, stop.color.g, stop.color.b];
                editor.temp_position = stop.position;
            }
            
            // Delete button (don't allow deleting if only 2 stops left)
            if stops.len() > 2 {
                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                    stop_to_remove = Some(i);
                    changed = true;
                }
            }
        });
    }
    
    // Remove stop if requested
    if let Some(index) = stop_to_remove {
        colormap.remove_stop(index);
        if let Some(selected) = editor.selected_stop_index {
            if selected == index {
                editor.selected_stop_index = None;
            } else if selected > index {
                editor.selected_stop_index = Some(selected - 1);
            }
        }
    }
    
    changed
}

/// Render color picker for editing a selected color stop
pub fn render_color_picker(
    ui: &mut egui::Ui,
    colormap: &mut ColorMap,
    editor: &mut ColorEditor,
) -> bool {
    let mut changed = false;
    
    if let Some(index) = editor.selected_stop_index {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Edit Color Stop").strong());
            ui.add_space(5.0);
            
            // Position slider
            ui.label("Position:");
            if ui.add(
                egui::Slider::new(&mut editor.temp_position, 0.0..=1.0)
                    .fixed_decimals(2)
            ).changed() {
                changed = true;
            }
            
            ui.add_space(5.0);
            ui.label("Color (RGB):");
            
            // RGB sliders
            let mut rgb_changed = false;
            
            ui.horizontal(|ui| {
                ui.label("R:");
                rgb_changed |= ui.add(
                    egui::Slider::new(&mut editor.temp_rgb[0], 0..=255)
                        .fixed_decimals(0)
                ).changed();
            });
            
            ui.horizontal(|ui| {
                ui.label("G:");
                rgb_changed |= ui.add(
                    egui::Slider::new(&mut editor.temp_rgb[1], 0..=255)
                        .fixed_decimals(0)
                ).changed();
            });
            
            ui.horizontal(|ui| {
                ui.label("B:");
                rgb_changed |= ui.add(
                    egui::Slider::new(&mut editor.temp_rgb[2], 0..=255)
                        .fixed_decimals(0)
                ).changed();
            });
            
            changed |= rgb_changed;
            
            // Color preview
            ui.add_space(5.0);
            ui.label("Preview:");
            let preview_size = egui::vec2(ui.available_width(), 40.0);
            let (preview_rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
            ui.painter().rect_filled(
                preview_rect,
                4.0,
                egui::Color32::from_rgb(editor.temp_rgb[0], editor.temp_rgb[1], editor.temp_rgb[2]),
            );
            ui.painter().rect_stroke(
                preview_rect,
                4.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY),
            );
            
            ui.add_space(10.0);
            
            // Apply/Cancel buttons
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    // Update the color stop
                    let stops = colormap.stops_mut();
                    if index < stops.len() {
                        stops[index].color = Color {
                            r: editor.temp_rgb[0],
                            g: editor.temp_rgb[1],
                            b: editor.temp_rgb[2],
                        };
                        stops[index].position = editor.temp_position;
                        
                        // Re-sort stops by position
                        stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
                    }
                    
                    editor.selected_stop_index = None;
                    changed = true;
                }
                
                if ui.button("Cancel").clicked() {
                    editor.selected_stop_index = None;
                }
            });
        });
    } else {
        ui.label(egui::RichText::new("Select a color stop to edit").italics().weak());
    }
    
    changed
}

/// Render add color stop button and UI
pub fn render_add_stop_button(
    ui: &mut egui::Ui,
    colormap: &mut ColorMap,
) -> bool {
    let mut changed = false;
    
    if ui.button("➕ Add Color Stop").clicked() {
        // Add a new stop at the midpoint with an interpolated color
        let new_position = 0.5;
        let new_color = colormap.get_color(new_position);
        colormap.add_stop(ColorStop {
            position: new_position,
            color: new_color,
        });
        changed = true;
    }
    
    changed
}

/// Complete color editor section combining all widgets
pub fn render_color_editor_section(
    ui: &mut egui::Ui,
    colormap: &mut ColorMap,
    editor: &mut ColorEditor,
) -> bool {
    let mut changed = false;
    
    ui.add_space(5.0);
    
    // Gradient preview
    render_gradient_preview(ui, colormap, ui.available_width(), 40.0);
    
    ui.add_space(10.0);
    
    // Color stops list
    changed |= render_color_stops_list(ui, colormap, editor);
    
    ui.add_space(10.0);
    
    // Add stop button
    changed |= render_add_stop_button(ui, colormap);
    
    ui.add_space(10.0);
    
    // Color picker (if a stop is selected)
    changed |= render_color_picker(ui, colormap, editor);
    
    changed
}
