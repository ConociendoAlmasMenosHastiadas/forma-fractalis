use eframe::egui;
use forma_fractalis::{
    colorschemes::ColorMap, colorschemes_gui::ColorEditor, colorschemes_io,
    filtering::FilterType, fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, FractalView}, gui,
    rendering_pipeline::{render_with_config, RenderConfig, RenderTarget},
};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1580.0, 750.0]) // 300px sidebar + 1280x720 (16:9) display area
            .with_title("Fractal Explorer - Forma Fractalis"),
        ..Default::default()
    };

    eframe::run_native(
        "Fractal Explorer - Forma Fractalis",
        options,
        Box::new(|cc| {
            // Enable high DPI scaling
            cc.egui_ctx.set_pixels_per_point(1.0);
            Box::<FractalApp>::default()
        }),
    )
}

/// Available fractal types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
    TippetsMandelbrot,
}

impl FractalType {
    pub fn name(&self) -> &str {
        match self {
            FractalType::Mandelbrot => "Mandelbrot",
            FractalType::Julia => "Julia Set",
            FractalType::BurningShip => "Burning Ship",
            FractalType::TippetsMandelbrot => "Tippets Mandelbrot",
        }
    }

    fn all() -> &'static [FractalType] {
        &[FractalType::Mandelbrot, FractalType::Julia, FractalType::BurningShip, FractalType::TippetsMandelbrot]
    }

    /// Creates a fractal instance from the enum type
    /// 
    /// This helper eliminates code duplication when creating fractals
    /// for rendering or export operations.
    pub fn create_instance(&self) -> Box<dyn forma_fractalis::fractals::Fractal> {
        match self {
            FractalType::Mandelbrot => Box::new(Mandelbrot::new()),
            FractalType::Julia => Box::new(Julia::new()),
            FractalType::BurningShip => Box::new(BurningShip::new()),
            FractalType::TippetsMandelbrot => Box::new(TippetsMandelbrot::new()),
        }
    }
}

impl gui::FractalTypeOps for FractalType {
    fn get_name(&self) -> &str {
        self.name()
    }

    fn all_types() -> Vec<Self> {
        Self::all().to_vec()
    }

    fn is_julia(&self) -> bool {
        matches!(self, FractalType::Julia)
    }

    fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        julia_c_real_input: &str,
        julia_c_imag_input: &str,
    ) {
        match self {
            FractalType::Mandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
            }
            FractalType::Julia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                // Keep existing Julia parameters when switching back
                params.insert(
                    "c_real".to_string(),
                    julia_c_real_input.parse().unwrap_or(-0.7)
                );
                params.insert(
                    "c_imag".to_string(),
                    julia_c_imag_input.parse().unwrap_or(0.27015)
                );
            }
            FractalType::BurningShip => {
                view.center_x = -0.5;
                view.center_y = -0.6;
                view.zoom = 0.8;
                params.clear();
            }
            FractalType::TippetsMandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
            }
        }
    }
}

struct FractalApp {
    // View state
    view: FractalView,

    // UI inputs
    width_input: String,
    height_input: String,
    iterations_input: String,

    // Fractal type and parameters
    fractal_type: FractalType,
    fractal_parameters: HashMap<String, f64>,
    julia_c_real_input: String,
    julia_c_imag_input: String,

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

    // Mouse interaction
    is_dragging: bool,
    zoom_square_center: Option<egui::Pos2>,
    zoom_square_size: f32,

    // Export
    export_scale_input: String,
    export_directory: Option<std::path::PathBuf>,
    export_filter: FilterType,
    export_supersample_input: String,

    // Status
    status_message: String,
}

impl Default for FractalApp {
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

        // Initialize Julia with random classic coordinates for discovery
        let (julia_c_real, julia_c_imag) = Julia::random_classic_coordinates();

        Self {
            view: FractalView::new(1280, 720),
            width_input: String::from("1280"),
            height_input: String::from("720"),
            iterations_input: String::from("256"),
            fractal_type: FractalType::Mandelbrot,
            fractal_parameters: HashMap::new(),
            julia_c_real_input: format!("{:.6}", julia_c_real),
            julia_c_imag_input: format!("{:.6}", julia_c_imag),
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
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 200.0,
            export_scale_input: String::from("3.0"),
            export_directory: None,
            export_filter: FilterType::None,
            export_supersample_input: String::from("4"),
            status_message: String::from("Ready - Click+drag to position zoom, scroll to resize"),
        }
    }
}

impl FractalApp {
    fn render_fractal(&mut self, ctx: &egui::Context) {
        let total_timer = Instant::now();
        
        // Parse period and iterations values (default to 256 if invalid)
        let period = self.period_input.parse::<u32>().unwrap_or(256);
        let max_iterations = self
            .iterations_input
            .parse::<u32>()
            .unwrap_or(256)
            .max(1);

        // Create fractal instance based on selected type
        let mandelbrot = Mandelbrot::new();
        let julia = Julia::new();
        let burning_ship = BurningShip::new();
        let tippets_mandelbrot = TippetsMandelbrot::new();
        
        let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal_type {
            FractalType::Mandelbrot => &mandelbrot,
            FractalType::Julia => &julia,
            FractalType::BurningShip => &burning_ship,
            FractalType::TippetsMandelbrot => &tippets_mandelbrot,
        };

        // Build render configuration
        let config = RenderConfig::new(self.view.clone(), &self.colormap, max_iterations, fractal)
            .with_period(self.use_period, period)
            .with_interior_color(self.use_interior_color, self.interior_color)
            .with_log_scale(self.use_log_scale)
            .with_fractal_parameters(self.fractal_parameters.clone());

        // Render for preview (no filtering)
        let buffer = render_with_config(&config, RenderTarget::Preview);

        // Convert to egui ColorImage
        let image_timer = Instant::now();
        let width = self.view.width as usize;
        let height = self.view.height as usize;
        let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &buffer);
        let image_time = image_timer.elapsed();

        // Update or create texture
        let texture_timer = Instant::now();
        if let Some(texture) = &mut self.fractal_texture {
            texture.set(color_image, egui::TextureOptions::NEAREST);
        } else {
            self.fractal_texture =
                Some(ctx.load_texture("mandelbrot", color_image, egui::TextureOptions::NEAREST));
        }
        let texture_time = texture_timer.elapsed();
        
        let total_time = total_timer.elapsed();
        println!("[PERF] GUI overhead: image_convert={:.2?}, texture_upload={:.2?}, total_gui={:.2?}",
            image_time, texture_time, total_time);

        self.needs_redraw = false;
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Note: Window title is set once at startup in main()
        // eframe 0.25 doesn't support dynamic title changes
        
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
                                &mut self.fractal_type,
                                &mut self.fractal_parameters,
                                &mut self.julia_c_real_input,
                                &mut self.julia_c_imag_input,
                                &mut self.view,
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
                                &mut self.interior_color_r_text,
                                &mut self.interior_color_g_text,
                                &mut self.interior_color_b_text,
                                &mut self.use_log_scale,
                            );

                            ui.add_space(10.0);

                            // Advanced Color Editor (always visible)
                        if forma_fractalis::colorschemes_gui::render_color_editor_section(
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
                                .max(1);
                            let period = self.period_input.parse::<u32>().unwrap_or(256);
                            
                            // Create fractal instance for export
                            let mandelbrot = Mandelbrot::new();
                            let julia = Julia::new();
                            let burning_ship = BurningShip::new();
                            let tippets_mandelbrot = TippetsMandelbrot::new();
                            
                            let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal_type {
                                FractalType::Mandelbrot => &mandelbrot,
                                FractalType::Julia => &julia,
                                FractalType::BurningShip => &burning_ship,
                                FractalType::TippetsMandelbrot => &tippets_mandelbrot,
                            };
                            
                            gui::render_actions_section(
                                ui,
                                &self.view,
                                &self.colormap,
                                max_iterations,
                                fractal,
                                &self.fractal_parameters,
                                self.use_period,
                                period,
                                self.use_interior_color,
                                self.interior_color,
                                self.use_log_scale,
                                &mut self.export_scale_input,
                                &mut self.export_directory,
                                &mut self.export_filter,
                                &mut self.export_supersample_input,
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

                // Check which mouse button is being used
                let pointer_state = ui.input(|i| i.pointer.clone());
                let primary_down = pointer_state.primary_down();
                let secondary_down = pointer_state.secondary_down();

                // Handle scroll wheel for zoom square resize (only when dragging with primary)
                let scroll_delta = ui.input(|i| i.scroll_delta.y);
                if scroll_delta != 0.0 && self.is_dragging {
                    self.zoom_square_size = (self.zoom_square_size + scroll_delta * 2.0)
                        .max(20.0);
                    self.status_message = format!("Zoom size: {:.0}px", self.zoom_square_size);
                }

                // Handle left-click drag for zoom rectangle
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

                // Right-click is reserved for future functionality
                // (Currently no action on right-click in main app)

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
