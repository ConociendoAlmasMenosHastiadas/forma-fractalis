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
//! ```ignore
//! let mut editor = ColorEditor::new();
//! if render_color_editor_section(ui, &mut colormap, &mut editor) {
//!     // Colormap was modified, trigger re-render
//! }
//! ```

use scala_chromatica::{Color, ColorMap, ColorStop};
use eframe::egui;

/// State for the color editor
pub struct ColorEditor {
    pub selected_stop_index: Option<usize>,
    pub temp_rgb: [u8; 3],
    pub temp_position: f64,
    pub dragging_stop_index: Option<usize>,
    pub colormap_directory: Option<std::path::PathBuf>,
}

impl Default for ColorEditor {
    fn default() -> Self {
        Self {
            selected_stop_index: None,
            temp_rgb: [0, 0, 0],
            temp_position: 0.0,
            dragging_stop_index: None,
            colormap_directory: None,
        }
    }
}

impl ColorEditor {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render a gradient preview bar showing the colormap with interactive markers
pub fn render_gradient_preview(
    ui: &mut egui::Ui,
    colormap: &mut ColorMap,
    editor: &mut ColorEditor,
    width: f32,
    height: f32,
) -> bool {
    let mut changed = false;

    let (rect, _response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

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
        let strip_rect =
            egui::Rect::from_min_size(egui::pos2(x, rect.min.y), egui::vec2(strip_width, height));

        mesh.add_colored_rect(strip_rect, color32);
    }

    ui.painter().add(egui::Shape::Mesh(mesh.into()));

    // Draw border
    ui.painter()
        .rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Middle,
        );

    // Handle mouse dragging
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let mouse_released = ui.input(|i| i.pointer.primary_released());

    if mouse_released {
        editor.dragging_stop_index = None;
    }

    // Draw and handle color stop markers
    let stops_clone = colormap.stops.to_vec();
    for (i, stop) in stops_clone.iter().enumerate() {
        let x = rect.min.x + (stop.position as f32) * width;
        let marker_pos = egui::pos2(x, rect.min.y + height);

        // Draw triangle marker pointing up
        let size = 6.0;
        let points = vec![
            egui::pos2(x, marker_pos.y),
            egui::pos2(x - size, marker_pos.y + size),
            egui::pos2(x + size, marker_pos.y + size),
        ];

        // Create a sense area for the marker
        let marker_rect = egui::Rect::from_min_max(
            egui::pos2(x - size - 2.0, marker_pos.y - 2.0),
            egui::pos2(x + size + 2.0, marker_pos.y + size + 2.0),
        );

        let marker_id = ui.id().with("marker").with(i);
        let marker_response = ui.interact(marker_rect, marker_id, egui::Sense::click_and_drag());

        // Check if this marker is being dragged
        if marker_response.drag_started() {
            editor.dragging_stop_index = Some(i);
        }

        // Update position if dragging
        if editor.dragging_stop_index == Some(i) {
            if let Some(pos) = pointer_pos {
                if pos.x >= rect.min.x && pos.x <= rect.max.x {
                    let new_position = ((pos.x - rect.min.x) / width) as f64;
                    let new_position = new_position.clamp(0.0, 1.0);

                    // Don't allow moving the first or last stop
                    if i > 0 && i < stops_clone.len() - 1 {
                        colormap.stops[i].position = new_position;
                        changed = true;
                    }
                }
            }
        }

        // Highlight marker on hover or drag
        let marker_color = if marker_response.hovered() || editor.dragging_stop_index == Some(i) {
            egui::Color32::YELLOW
        } else {
            egui::Color32::WHITE
        };

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            marker_color,
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        ));

        // Show tooltip on hover
        if marker_response.hovered() {
            egui::Tooltip::for_widget(&marker_response)
                .at_pointer()
                .show(|ui| {
                ui.label(format!("Position: {:.3}", stop.position));
                if i > 0 && i < stops_clone.len() - 1 {
                    ui.label("Drag to adjust");
                } else {
                    ui.label("Fixed position");
                }
                });
        }
    }

    changed
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

    let stops = colormap.stops.to_vec(); // Clone to avoid borrow issues

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
            ui.painter()
                .rect_stroke(
                    color_rect,
                    2.0,
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                    egui::StrokeKind::Middle,
                );

            // RGB values
            ui.label(format!(
                "RGB({},{},{})",
                stop.color.r, stop.color.g, stop.color.b
            ));

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
    picker_state: &mut crate::color_picker::ColorPickerState,
) -> bool {
    let mut changed = false;

    if let Some(index) = editor.selected_stop_index {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Edit Color Stop").strong());
            ui.add_space(5.0);

            // Position slider
            ui.label("Position:");
            if ui
                .add(egui::Slider::new(&mut editor.temp_position, 0.0..=1.0).fixed_decimals(2))
                .changed()
            {
                changed = true;
            }

            ui.add_space(5.0);

            // HSV color picker
            if crate::color_picker::show_color_picker(
                ui,
                picker_state,
                &mut editor.temp_rgb,
                "stop_color",
            ) {
                changed = true;
            }

            ui.add_space(10.0);

            // Apply/Cancel buttons
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    // Update the color stop
                    let stops = &mut colormap.stops;
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
        ui.label(
            egui::RichText::new("Select a color stop to edit")
                .italics()
                .weak(),
        );
    }

    changed
}

/// Render add color stop button and UI
pub fn render_add_stop_button(ui: &mut egui::Ui, colormap: &mut ColorMap) -> bool {
    let mut changed = false;

    if ui.button("➕ Add Color Stop").clicked() {
        // Add a new stop at the midpoint with an interpolated color
        let new_position = 0.5;
        let new_color = colormap.get_color(new_position);
        colormap.add_stop(ColorStop {
            position: new_position,
            color: new_color,
            name: None,
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
    stop_picker: &mut crate::color_picker::ColorPickerState,
) -> bool {
    let mut changed = false;

    ui.add_space(5.0);

    // Gradient preview with interactive markers
    changed |= render_gradient_preview(ui, colormap, editor, ui.available_width(), 40.0);

    ui.add_space(10.0);

    // Save/Load ColorMap section
    ui.collapsing("💾 Save/Load ColorMap", |ui| {
        ui.add_space(5.0);

        // Directory selection
        ui.horizontal(|ui| {
            if ui.button("📁 Choose Directory").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    editor.colormap_directory = Some(path);
                }
            }

            if editor.colormap_directory.is_some() {
                if ui.button("✖ Clear").clicked() {
                    editor.colormap_directory = None;
                }
            }
        });

        // Show current directory
        if let Some(dir) = &editor.colormap_directory {
            ui.label(
                egui::RichText::new(format!("📂 {}", dir.display()))
                    .small()
                    .italics(),
            );
        } else {
            ui.label(
                egui::RichText::new("📂 Using config directory")
                    .small()
                    .italics(),
            );
        }

        ui.add_space(10.0);

        // Save and Load buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Save ColorMap").clicked() {
                save_colormap_with_dialog(colormap, editor.colormap_directory.as_ref());
            }

            if ui.button("📂 Load ColorMap").clicked() {
                if let Some(loaded) = load_colormap_with_dialog(editor.colormap_directory.as_ref())
                {
                    *colormap = loaded;
                    changed = true;
                }
            }
        });
    });

    ui.add_space(10.0);

    // Color stops list
    changed |= render_color_stops_list(ui, colormap, editor);

    ui.add_space(10.0);

    // Add stop button
    changed |= render_add_stop_button(ui, colormap);

    ui.add_space(10.0);

    // Color picker (if a stop is selected)
    changed |= render_color_picker(ui, colormap, editor, stop_picker);

    changed
}

/// Save colormap with a file dialog
fn save_colormap_with_dialog(colormap: &ColorMap, directory: Option<&std::path::PathBuf>) {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_title("Save ColorMap")
        .set_file_name(&format!("{}.json", colormap.name));

    // Set directory if specified
    if let Some(dir) = directory {
        dialog = dialog.set_directory(dir);
    }

    if let Some(path) = dialog.save_file() {
        // Create colormap with the filename as the name
        let mut save_map = colormap.clone();
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            save_map.name = stem.to_string();
        }

        // Save directly to the chosen path
        let json = match serde_json::to_string_pretty(&save_map) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Failed to serialize colormap: {}", e);
                return;
            }
        };

        if let Err(e) = std::fs::write(&path, json) {
            eprintln!("Failed to write colormap file: {}", e);
        }
    }
}

/// Load colormap with a file dialog
fn load_colormap_with_dialog(directory: Option<&std::path::PathBuf>) -> Option<ColorMap> {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_title("Load ColorMap");

    // Set directory if specified
    if let Some(dir) = directory {
        dialog = dialog.set_directory(dir);
    }

    if let Some(path) = dialog.pick_file() {
        match std::fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<ColorMap>(&json) {
                Ok(colormap) => Some(colormap),
                Err(e) => {
                    eprintln!("Failed to parse colormap: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to read colormap file: {}", e);
                None
            }
        }
    } else {
        None
    }
}
