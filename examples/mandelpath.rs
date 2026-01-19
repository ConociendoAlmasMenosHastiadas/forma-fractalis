use eframe::egui;
use forma_fractalis::{
    colorschemes::ColorMap, colorschemes_gui::ColorEditor, colorschemes_io,
    filtering::FilterType, fractals::{Mandelbrot, FractalView}, gui,
    rendering_pipeline::{render_with_config, RenderConfig, RenderTarget},
};
use num_complex::Complex64;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Render a section header with consistent styling
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).strong());
    ui.add_space(5.0);
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1580.0, 750.0])
            .with_title("Mandelbrot Path Explorer"),
        ..Default::default()
    };

    eframe::run_native(
        "Mandelbrot Path Explorer",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::<MandelPathApp>::default()
        }),
    )
}

/// Debounce delay for text input to prevent lag during typing
const INPUT_DEBOUNCE_DELAY: Duration = Duration::from_millis(500);

/// A path representing the iteration sequence for a specific point
#[derive(Clone)]
struct IterationPath {
    /// The complex constant c for this path
    c: Complex64,
    /// The sequence of z values during iteration
    series: Vec<Complex64>,
}

struct MandelPathApp {
    // View state
    view: FractalView,

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
    interior_color_r_text: String,
    interior_color_g_text: String,
    interior_color_b_text: String,
    use_log_scale: bool,

    // Fractal texture
    fractal_texture: Option<egui::TextureHandle>,
    needs_redraw: bool,

    // Input debouncing for text fields
    input_debounce_timer: Option<Instant>,
    pending_redraw: bool,

    // Mouse interaction for zoom
    is_dragging: bool,
    zoom_square_center: Option<egui::Pos2>,
    zoom_square_size: f32,

    // Path tracking
    iteration_paths: Vec<IterationPath>,

    // Export
    export_scale_input: String,
    export_directory: Option<std::path::PathBuf>,
    export_filter: FilterType,
    export_supersample_input: String,

    // Status
    status_message: String,
}

impl Default for MandelPathApp {
    fn default() -> Self {
        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        let selected_colormap_name = String::from("Default");
        let colormap = colorschemes_io::load_colormap(&selected_colormap_name)
            .unwrap_or_else(|_| ColorMap::default_scheme());

        Self {
            view: FractalView::new(1280, 720),
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
            interior_color_r_text: String::from("0"),
            interior_color_g_text: String::from("0"),
            interior_color_b_text: String::from("0"),
            use_log_scale: false,
            fractal_texture: None,
            needs_redraw: true,
            input_debounce_timer: None,
            pending_redraw: false,
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 100.0,
            iteration_paths: Vec::new(),
            export_scale_input: String::from("1"),
            export_directory: None,
            export_filter: FilterType::None,
            export_supersample_input: String::from("1"),
            status_message: String::from("Left-click & drag to zoom. Right-click to see iteration paths. Press 'C' to clear paths."),
        }
    }
}

impl MandelPathApp {
    /// Convert a complex number to screen coordinates
    fn complex_to_screen(&self, c: Complex64, texture_size: egui::Vec2, rect: egui::Rect, scale: f32) -> Option<egui::Pos2> {
        // Use the same coordinate system as FractalView::screen_to_complex
        let aspect_ratio = self.view.width as f64 / self.view.height as f64;
        let view_scale = 3.5 / self.view.zoom;

        // Inverse of screen_to_complex formula:
        // real = center_x + (x - width/2) * view_scale / width * aspect_ratio
        // Solving for x: x = width/2 + (real - center_x) * width / (view_scale * aspect_ratio)
        let x_pixel = self.view.width as f64 / 2.0 
            + (c.re - self.view.center_x) * self.view.width as f64 / (view_scale * aspect_ratio);
        let y_pixel = self.view.height as f64 / 2.0 
            + (c.im - self.view.center_y) * self.view.height as f64 / view_scale;

        // Check if point is within texture bounds
        if x_pixel < 0.0 || x_pixel >= self.view.width as f64 || 
           y_pixel < 0.0 || y_pixel >= self.view.height as f64 {
            return None;
        }

        // Convert to screen coordinates
        let screen_x = rect.min.x + x_pixel as f32 * scale;
        let screen_y = rect.min.y + y_pixel as f32 * scale;

        Some(egui::Pos2::new(screen_x, screen_y))
    }

    /// Convert screen coordinates to complex plane coordinates
    fn screen_to_complex(&self, pos: egui::Pos2, texture_size: egui::Vec2, rect: egui::Rect, scale: f32) -> Complex64 {
        // Convert screen position to texture pixel coordinates
        let x_pixel = ((pos.x - rect.min.x) / scale) as f64;
        let y_pixel = ((pos.y - rect.min.y) / scale) as f64;

        // Use the same coordinate system as FractalView::screen_to_complex
        let aspect_ratio = self.view.width as f64 / self.view.height as f64;
        let view_scale = 3.5 / self.view.zoom;

        let real = self.view.center_x
            + (x_pixel - self.view.width as f64 / 2.0) * view_scale / self.view.width as f64 * aspect_ratio;
        let imag = self.view.center_y
            + (y_pixel - self.view.height as f64 / 2.0) * view_scale / self.view.height as f64;

        Complex64::new(real, imag)
    }

    /// Render all iteration paths on top of the fractal
    fn render_paths(&self, painter: &egui::Painter, texture_size: egui::Vec2, rect: egui::Rect, scale: f32) {
        for path in &self.iteration_paths {
            let mut prev_screen_pos: Option<egui::Pos2> = None;

            for (i, &z) in path.series.iter().enumerate() {
                if let Some(screen_pos) = self.complex_to_screen(z, texture_size, rect, scale) {
                    // Draw the point (larger for start point, smaller for others)
                    let point_radius = if i == 0 { 6.0 } else { 3.0 };
                    let point_color = if i == 0 {
                        egui::Color32::from_rgb(255, 255, 0) // Yellow for start
                    } else {
                        egui::Color32::WHITE
                    };
                    
                    painter.circle_filled(screen_pos, point_radius, point_color);
                    painter.circle_stroke(screen_pos, point_radius, egui::Stroke::new(1.0, egui::Color32::BLACK));

                    // Draw line from previous point with arrow
                    if let Some(prev_pos) = prev_screen_pos {
                        // Draw main line
                        painter.line_segment(
                            [prev_pos, screen_pos],
                            egui::Stroke::new(2.0, egui::Color32::WHITE)
                        );

                        // Draw arrow head
                        let direction = screen_pos - prev_pos;
                        let length = direction.length();
                        if length > 0.0 {
                            let dir_normalized = direction / length;
                            let perpendicular = egui::Vec2::new(-dir_normalized.y, dir_normalized.x);
                            
                            let arrow_size = 8.0;
                            let arrow_width = 5.0;
                            
                            // Position arrow slightly before the endpoint to avoid overlapping with the point
                            let arrow_pos = screen_pos - dir_normalized * point_radius * 1.5;
                            let arrow_back = arrow_pos - dir_normalized * arrow_size;
                            
                            let arrow_p1 = arrow_back + perpendicular * arrow_width;
                            let arrow_p2 = arrow_back - perpendicular * arrow_width;
                            
                            // Draw filled arrow head
                            painter.add(egui::Shape::convex_polygon(
                                vec![arrow_pos, arrow_p1, arrow_p2],
                                egui::Color32::WHITE,
                                egui::Stroke::new(1.0, egui::Color32::BLACK),
                            ));
                        }
                    }

                    prev_screen_pos = Some(screen_pos);
                }
            }
        }
    }

    fn render_fractal(&mut self, ctx: &egui::Context) {
        let max_iterations = self.iterations_input.parse::<u32>().unwrap_or(256).max(1);
        let period = self.period_input.parse::<u32>().unwrap_or(256);

        let mandelbrot = Mandelbrot::new();

        let config = RenderConfig {
            view: self.view.clone(),
            fractal: &mandelbrot,
            fractal_parameters: HashMap::new(),
            colormap: &self.colormap,
            max_iterations,
            use_period: self.use_period,
            period,
            use_interior_color: self.use_interior_color,
            interior_color: self.interior_color,
            use_log_scale: self.use_log_scale,
        };

        let buffer = render_with_config(&config, RenderTarget::Preview);
        
        // Convert RGBA buffer to ColorImage
        let width = self.view.width as usize;
        let height = self.view.height as usize;
        let mut pixels = Vec::with_capacity(width * height);
        
        for chunk in buffer.chunks(4) {
            pixels.push(egui::Color32::from_rgba_premultiplied(
                chunk[0], chunk[1], chunk[2], chunk[3]
            ));
        }

        let color_image = egui::ColorImage {
            size: [width, height],
            pixels,
        };

        self.fractal_texture = Some(ctx.load_texture(
            "fractal",
            color_image,
            egui::TextureOptions::NEAREST,
        ));

        self.needs_redraw = false;
    }
}

impl eframe::App for MandelPathApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if debounced input should trigger redraw
        if let Some(timer) = self.input_debounce_timer {
            if timer.elapsed() >= INPUT_DEBOUNCE_DELAY && self.pending_redraw {
                self.needs_redraw = true;
                self.pending_redraw = false;
                self.input_debounce_timer = None;
            } else if self.pending_redraw {
                ctx.request_repaint_after(INPUT_DEBOUNCE_DELAY - timer.elapsed());
            }
        }
        
        // Render fractal if needed
        if self.needs_redraw {
            self.render_fractal(ctx);
        }

        // Handle 'C' key to clear paths
        if ctx.input(|i| i.key_pressed(egui::Key::C)) {
            self.iteration_paths.clear();
            self.status_message = String::from("Paths cleared. Click to create new paths.");
        }

        // Sidebar
        egui::SidePanel::left("sidebar")
            .exact_width(300.0)
            .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading("Mandelbrot Path Explorer");
                        ui.add_space(10.0);

                        // Dimensions
                        gui::render_dimensions_section(
                            ui,
                            &mut self.width_input,
                            &mut self.height_input,
                            &mut self.view,
                            &mut self.needs_redraw,
                            &mut self.input_debounce_timer,
                            &mut self.pending_redraw,
                        );

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Settings - just iterations input
                        section_header(ui, "Fractal Settings");
                        ui.horizontal(|ui| {
                            ui.label("Max Iterations:");
                            ui.add_space(8.0);
                            if ui
                                .add(egui::TextEdit::singleline(&mut self.iterations_input).desired_width(80.0))
                                .changed()
                            {
                                self.input_debounce_timer = Some(Instant::now());
                                self.pending_redraw = true;
                            }
                            if ui.small_button("×2").clicked() {
                                if let Ok(val) = self.iterations_input.parse::<u32>() {
                                    self.iterations_input = (val * 2).to_string();
                                    self.needs_redraw = true;
                                }
                            }
                            if ui.small_button("÷2").clicked() {
                                if let Ok(val) = self.iterations_input.parse::<u32>() {
                                    self.iterations_input = (val / 2).max(1).to_string();
                                    self.needs_redraw = true;
                                }
                            }
                        });

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // View info
                        section_header(ui, "Current View");
                        ui.label(format!("X: {:.6}", self.view.center_x));
                        ui.label(format!("Y: {:.6}", self.view.center_y));
                        ui.label(format!("Zoom: {:.2}x", self.view.zoom));
                        ui.label(format!("Size: {}×{}", self.view.width, self.view.height));

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Colormap
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
                            &mut self.interior_color_r_text,
                            &mut self.interior_color_g_text,
                            &mut self.interior_color_b_text,
                            &mut self.use_log_scale,
                            &mut self.input_debounce_timer,
                            &mut self.pending_redraw,
                        );

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Path controls
                        section_header(ui, "Path Controls");
                        ui.label(format!("Active paths: {}", self.iteration_paths.len()));
                        if ui.button("Clear All Paths (C)").clicked() {
                            self.iteration_paths.clear();
                            self.status_message = String::from("Paths cleared.");
                        }

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
                let scale = (available_size.x / texture_size.x).min(available_size.y / texture_size.y);
                let display_size = texture_size * scale;

                let (rect, response) = ui.allocate_exact_size(display_size, egui::Sense::click_and_drag());

                // Draw the fractal texture
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Check which mouse button is being used
                let pointer_state = ui.input(|i| i.pointer.clone());
                let primary_down = pointer_state.primary_down();
                let secondary_down = pointer_state.secondary_down();
                
                // Handle scroll wheel for zoom square resize (only when dragging with primary)
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.is_dragging {
                    self.zoom_square_size = (self.zoom_square_size + scroll_delta * 2.0).max(20.0);
                    self.status_message = format!("Zoom size: {:.0}px", self.zoom_square_size);
                }

                // Handle right-click for path generation
                if response.secondary_clicked() {
                    if let Some(click_pos) = response.interact_pointer_pos() {
                        // Convert screen position to texture pixel coordinates
                        let rel_pos = (click_pos - rect.min) / scale;
                        let pixel_x = rel_pos.x as u32;
                        let pixel_y = rel_pos.y as u32;
                        
                        // Convert to complex coordinates using FractalView method
                        let (c_real, c_imag) = self.view.screen_to_complex(pixel_x, pixel_y);
                        
                        // Get max iterations
                        let max_iterations = self.iterations_input.parse::<u32>().unwrap_or(256).max(1);
                        
                        // Generate the iteration series (using power = 2.0 for classic Mandelbrot)
                        let series = Mandelbrot::mandelseries(c_real, c_imag, 2.0, max_iterations);
                        let c = Complex64::new(c_real, c_imag);
                        
                        self.iteration_paths.push(IterationPath { c, series });
                        self.status_message = format!(
                            "Path added at c = {:.6} + {:.6}i ({} iterations)",
                            c_real, c_imag, self.iteration_paths.last().unwrap().series.len() - 1
                        );
                    }
                }

                // Handle left-click drag for zoom
                // Only start dragging if primary button is pressed (not secondary)
                if response.drag_started() && primary_down && !secondary_down {
                    self.is_dragging = true;
                    self.zoom_square_center = response.interact_pointer_pos();
                }

                // Update square position while dragging (only for primary button)
                if self.is_dragging && primary_down {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.zoom_square_center = Some(pos);
                    }
                }

                // Complete zoom on drag release (only for primary button)
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

                // Draw zoom square if dragging
                if self.is_dragging {
                    if let Some(center) = self.zoom_square_center {
                        let aspect_ratio = self.view.width as f32 / self.view.height as f32;
                        gui::render_zoom_square(ui, center, self.zoom_square_size, aspect_ratio);
                    }
                }

                // Render all iteration paths on top
                self.render_paths(ui.painter(), texture_size, rect, scale);
            }
        });
    }
}
