use eframe::egui;
use mandelrust::{
    colorschemes::ColorMap, colorschemes_gui::ColorEditor, colorschemes_io,
    fractal::MandelbrotView, gui, rendering::render_mandelbrot,
};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1580.0, 750.0]) // 300px sidebar + 1280x720 (16:9) display area
            .with_title("Mandelbrot Set Explorer"),
        ..Default::default()
    };

    eframe::run_native(
        "Mandelbrot Set Explorer",
        options,
        Box::new(|cc| {
            // Enable high DPI scaling
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::<MandelbrotApp>::default()
        }),
    )
}

struct MandelbrotApp {
    // View state
    view: MandelbrotView,

    // UI inputs
    width_input: String,
    height_input: String,
    iterations_input: String,

    // Colormap
    available_colormaps: Vec<String>,
    selected_colormap_name: String,
    colormap: ColorMap,
    color_editor: ColorEditor,

    // Color modulation
    use_period: bool,
    period_input: String,
    use_interior_color: bool,
    interior_color: [u8; 3],

    // Fractal texture
    fractal_texture: Option<egui::TextureHandle>,
    needs_redraw: bool,

    // Mouse interaction
    is_dragging: bool,
    zoom_square_center: Option<egui::Pos2>,
    zoom_square_size: f32,

    // Export
    export_scale_input: String,
    export_directory: Option<std::path::PathBuf>,

    // Status
    status_message: String,
}

impl Default for MandelbrotApp {
    fn default() -> Self {
        // Load all available colormaps
        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        let selected_colormap_name = String::from("Default");
        let colormap = colorschemes_io::load_colormap(&selected_colormap_name)
            .unwrap_or_else(|_| ColorMap::default_scheme());

        Self {
            view: MandelbrotView::new(1280, 720),
            width_input: String::from("1280"),
            height_input: String::from("720"),
            iterations_input: String::from("256"),
            available_colormaps,
            selected_colormap_name,
            colormap,
            color_editor: ColorEditor::new(),
            use_period: false,
            period_input: String::from("128"),
            use_interior_color: false,
            interior_color: [0, 0, 0],
            fractal_texture: None,
            needs_redraw: true,
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 200.0,
            export_scale_input: String::from("3.0"),
            export_directory: None,
            status_message: String::from("Ready - Click+drag to position zoom, scroll to resize"),
        }
    }
}

impl MandelbrotApp {
    fn render_fractal(&mut self, ctx: &egui::Context) {
        // Render to buffer
        let width = self.view.width as usize;
        let height = self.view.height as usize;
        let mut buffer = vec![0u8; width * height * 4];

        // Parse period and iterations values (default to 256 if invalid)
        let period = self.period_input.parse::<u32>().unwrap_or(256);
        let max_iterations = self
            .iterations_input
            .parse::<u32>()
            .unwrap_or(256)
            .clamp(10, 10000);

        render_mandelbrot(
            &mut buffer,
            &self.view,
            &self.colormap,
            max_iterations,
            self.use_period,
            period,
            self.use_interior_color,
            self.interior_color,
        );

        // Convert to egui ColorImage
        let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &buffer);

        // Update or create texture
        if let Some(texture) = &mut self.fractal_texture {
            texture.set(color_image, egui::TextureOptions::NEAREST);
        } else {
            self.fractal_texture =
                Some(ctx.load_texture("mandelbrot", color_image, egui::TextureOptions::NEAREST));
        }

        self.needs_redraw = false;
    }
}

impl eframe::App for MandelbrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render fractal if needed
        if self.needs_redraw {
            self.render_fractal(ctx);
        }

        // Left sidebar with controls
        egui::SidePanel::left("controls")
            .default_width(300.0)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width() - 10.0); // Leave space for scrollbar
                        ui.vertical(|ui| {
                            ui.heading("Controls");
                            ui.add_space(10.0);

                            // Preview Window Dimensions
                            gui::render_dimensions_section(
                                ui,
                                &mut self.width_input,
                                &mut self.height_input,
                                &mut self.view,
                                &mut self.needs_redraw,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Fractal Settings
                            gui::render_fractal_settings(
                                ui,
                                &mut self.iterations_input,
                                &mut self.needs_redraw,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Current View
                            gui::render_current_view_info(
                                ui,
                                &mut self.view,
                                &mut self.needs_redraw,
                                &mut self.status_message,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Color Scheme & Stops
                            gui::render_colormap_section(
                                ui,
                                &mut self.available_colormaps,
                                &mut self.selected_colormap_name,
                                &mut self.colormap,
                                &mut self.needs_redraw,
                                &mut self.status_message,
                                &mut self.use_period,
                                &mut self.period_input,
                                &mut self.use_interior_color,
                                &mut self.interior_color,
                            );

                            ui.add_space(10.0);

                            // Advanced Color Editor (always visible)
                            if mandelrust::colorschemes_gui::render_color_editor_section(
                                ui,
                                &mut self.colormap,
                                &mut self.color_editor,
                            ) {
                                self.needs_redraw = true;
                            }

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Actions
                            let max_iterations = self
                                .iterations_input
                                .parse::<u32>()
                                .unwrap_or(256)
                                .clamp(10, 10000);
                            let period = self.period_input.parse::<u32>().unwrap_or(256);
                            gui::render_actions_section(
                                ui,
                                &self.view,
                                &self.colormap,
                                max_iterations,
                                self.use_period,
                                period,
                                self.use_interior_color,
                                self.interior_color,
                                &mut self.export_scale_input,
                                &mut self.export_directory,
                                &mut self.status_message,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Status
                            ui.label(egui::RichText::new(&self.status_message).small().italics());
                        });
                    });
            });

        // Main fractal display
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.fractal_texture {
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();
                let scale =
                    (available_size.x / texture_size.x).min(available_size.y / texture_size.y);
                let display_size = texture_size * scale;

                let (rect, response) =
                    ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());

                // Handle scroll wheel for zoom square resize
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.is_dragging {
                    self.zoom_square_size = (self.zoom_square_size + scroll_delta * 2.0)
                        .max(20.0)
                        .min(2000.0);
                    self.status_message = format!("Zoom size: {:.0}px", self.zoom_square_size);
                }

                // Handle mouse interactions
                if response.drag_started() {
                    self.is_dragging = true;
                    self.zoom_square_center = response.interact_pointer_pos();
                }

                // Update square position while dragging
                if self.is_dragging {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.zoom_square_center = Some(pos);
                    }
                }

                if response.drag_released() && self.is_dragging {
                    self.is_dragging = false;
                    if let Some(center) = self.zoom_square_center {
                        // Calculate zoom region
                        let center_rel = (center - rect.min) / scale;
                        let square_size_rel = self.zoom_square_size / scale;

                        let center_x = center_rel.x as u32;
                        let center_y = center_rel.y as u32;

                        // Calculate zoom factor based on square size relative to image size
                        let zoom_factor = texture_size.x / square_size_rel;

                        self.view.zoom_at(center_x, center_y, zoom_factor as f64);
                        self.needs_redraw = true;
                        self.status_message = format!("Zoomed to {:.2}x", self.view.zoom);
                    }
                    self.zoom_square_center = None;
                }

                // Right click to zoom out
                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rel_pos = (pos - rect.min) / scale;
                        let pixel_x = rel_pos.x as u32;
                        let pixel_y = rel_pos.y as u32;

                        self.view.zoom_at(pixel_x, pixel_y, 0.5);
                        self.needs_redraw = true;
                        self.status_message = format!("Zoomed out to {:.2}x", self.view.zoom);
                    }
                }

                // Draw the fractal
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Draw zoom rectangle if dragging (matching aspect ratio)
                if self.is_dragging {
                    if let Some(center) = self.zoom_square_center {
                        let aspect_ratio = self.view.width as f32 / self.view.height as f32;
                        gui::render_zoom_square(ui, center, self.zoom_square_size, aspect_ratio);
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }
        });

        // Request repaint for smooth interaction
        if self.is_dragging {
            ctx.request_repaint();
        }
    }
}
