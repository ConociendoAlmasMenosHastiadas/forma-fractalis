use eframe::egui;
use forma_fractalis::{
    app_state::{ViewState, InputState, FractalState, ColorState, MouseState, ExportState, FractalType, IterationCache},
    fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon, Tetration}, 
    gui, cli,
    perf_log, enable_profiling,
};
use std::time::{Duration, Instant};

fn main() -> Result<(), eframe::Error> {
    // Try CLI mode first
    match cli::try_cli() {
        Ok((Some(()), _)) => {
            // CLI mode executed successfully
            std::process::exit(0);
        }
        Ok((None, profiling)) => {
            // No CLI command, continue to GUI
            if profiling {
                enable_profiling();
                println!("[INFO] Performance profiling enabled");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1730.0, 820.0]) // 350px sidebar + 1380x820 display area
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

/// Debounce delay for text input to prevent lag during typing
/// Redraws are delayed until user stops typing for this duration
const INPUT_DEBOUNCE_DELAY: Duration = Duration::from_millis(500);

/// Main application state for the fractal explorer
/// 
/// # Redraw Behavior
/// The app uses a flag-based redraw system to avoid unnecessary computation:
/// - `needs_redraw`: Set to true when fractal needs regeneration
/// - `input_debounce_timer`: Tracks time since last text input change
/// - `pending_redraw`: Set when text input changes, triggers redraw after debounce delay
/// 
/// ## Redraw Triggers:
/// 1. **Immediate redraws** (buttons, sliders, dropdowns):
///    - Fractal type change
///    - Colormap selection
///    - Period/log scale/interior color toggles
///    - View reset button
///    - Zoom interaction (drag release)
///    - Color editor changes
/// 2. **Debounced redraws** (text inputs - 500ms delay after typing stops):
///    - Width/height inputs
///    - Iteration count input
///    - Julia parameter inputs (c_real, c_imag)
///    - View coordinate inputs (center_x, center_y, zoom)
///    - Period value input
///    - Interior color RGB inputs
/// 
/// This debouncing prevents lag during typing while maintaining responsive UI for direct interactions.
struct FractalApp {
    // Grouped state
    pub view_state: ViewState,
    pub input: InputState,
    pub fractal: FractalState,
    pub color: ColorState,
    pub mouse: MouseState,
    pub export: ExportState,

    // Status message
    pub status_message: String,
}

impl Default for FractalApp {
    fn default() -> Self {
        // Initialize Julia with random classic coordinates for discovery
        let (julia_c_real, julia_c_imag) = Julia::random_classic_coordinates();

        let mut input = InputState::default();
        input.julia_c_real = format!("{:.6}", julia_c_real);
        input.julia_c_imag = format!("{:.6}", julia_c_imag);

        Self {
            view_state: ViewState::new(1280, 720),
            input,
            fractal: FractalState::default(),
            color: ColorState::default(),
            mouse: MouseState::new(),
            export: ExportState::new(),
            status_message: String::from("Ready - Click+drag to position zoom, scroll to resize"),
        }
    }
}

impl FractalApp {
    fn render_fractal(&mut self, ctx: &egui::Context) {
        // Parse period and iterations values (default to 256 if invalid)
        let period = self.input.parse_period();
        let max_iterations = self.input.parse_iterations();

        // Create fractal instance based on selected type
        let mandelbrot = Mandelbrot::new();
        let julia = Julia::new();
        let burning_ship = BurningShip::new();
        let tippets_mandelbrot = TippetsMandelbrot::new();
        let multifractal_julia = MultifractalJulia::new();
        let cactus = Cactus::new();
        let marek_dragon = MarekDragon::new();
        let tetration = Tetration::new();
        
        let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal.fractal_type {
            FractalType::Mandelbrot => &mandelbrot,
            FractalType::Julia => &julia,
            FractalType::BurningShip => &burning_ship,
            FractalType::TippetsMandelbrot => &tippets_mandelbrot,
            FractalType::MultifractalJulia => &multifractal_julia,
            FractalType::Cactus => &cactus,
            FractalType::MarekDragon => &marek_dragon,
            FractalType::Tetration => &tetration,
        };

        // Check if we can use the iteration cache
        let use_cache = self.view_state.iteration_cache.as_ref().map_or(false, |cache| {
            cache.is_valid(
                self.view_state.view.width,
                self.view_state.view.height,
                max_iterations,
                self.fractal.fractal_type,
                &self.fractal.parameters,
                &self.view_state.view,
            )
        });

        let buffer_size = (self.view_state.view.width * self.view_state.view.height * 4) as usize;
        let mut buffer = vec![0u8; buffer_size];

        if use_cache {
            // Use cached iterations, only apply colors
            perf_log!("[CACHE] Using cached iterations");
            let cache = self.view_state.iteration_cache.as_ref().unwrap();
            
            forma_fractalis::rendering::apply_colors_from_cache(
                &mut buffer,
                &cache.data,
                &self.color.colormap,
                max_iterations,
                self.color.use_period,
                period,
                self.color.use_interior_color,
                self.color.interior_color,
                self.color.use_log_scale,
            );
        } else {
            // Compute iterations and cache them
            perf_log!("[CACHE] Computing and caching iterations");
            let iterations = forma_fractalis::rendering::compute_iterations(
                &self.view_state.view,
                max_iterations,
                fractal,
                &self.fractal.parameters,
            );
            
            // Store in cache
            self.view_state.iteration_cache = Some(IterationCache {
                data: iterations.clone(),
                width: self.view_state.view.width,
                height: self.view_state.view.height,
                max_iterations,
                fractal_type: self.fractal.fractal_type,
                fractal_parameters: self.fractal.parameters.clone(),
                center_x: self.view_state.view.center_x,
                center_y: self.view_state.view.center_y,
                zoom: self.view_state.view.zoom,
            });
            
            // Apply colors to computed iterations
            forma_fractalis::rendering::apply_colors_from_cache(
                &mut buffer,
                &iterations,
                &self.color.colormap,
                max_iterations,
                self.color.use_period,
                period,
                self.color.use_interior_color,
                self.color.interior_color,
                self.color.use_log_scale,
            );
        }

        self.finish_render(ctx, buffer);
    }

    fn finish_render(&mut self, ctx: &egui::Context, buffer: Vec<u8>) {
        // Convert to egui ColorImage
        let image_timer = Instant::now();
        let width = self.view_state.view.width as usize;
        let height = self.view_state.view.height as usize;
        let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &buffer);
        let image_time = image_timer.elapsed();

        // Update or create texture
        let texture_timer = Instant::now();
        if let Some(texture) = &mut self.view_state.fractal_texture {
            texture.set(color_image, egui::TextureOptions::NEAREST);
        } else {
            self.view_state.fractal_texture =
                Some(ctx.load_texture("mandelbrot", color_image, egui::TextureOptions::NEAREST));
        }
        let texture_time = texture_timer.elapsed();
        
        perf_log!("[PERF] Texture: image_convert={:.2?}, upload={:.2?}",
            image_time, texture_time);

        self.view_state.clear_redraw();
    }
    
    /// Load application state from PNG metadata
    fn load_from_metadata(&mut self, metadata: forma_fractalis::export::FractalMetadata) -> Result<(), String> {
        // Validate before applying
        let _ = metadata.parse_fractal_type()?;
        
        // Convert to state structs using From trait implementations
        self.fractal = FractalState::from(&metadata);
        self.view_state = ViewState::from(&metadata);
        self.color = ColorState::from(&metadata);
        self.input = InputState::from(&metadata);
        self.export = ExportState::from(&metadata);
        
        // Mark for redraw
        self.view_state.needs_redraw = true;
        
        Ok(())
    }
}

impl eframe::App for FractalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Note: Window title is set once at startup in main()
        // eframe 0.25 doesn't support dynamic title changes
        
        // Check if we need to load a PNG file (set by GUI button)
        if self.status_message.starts_with("LOAD_PNG:") {
            let path = self.status_message.strip_prefix("LOAD_PNG:").unwrap().to_string();
            match forma_fractalis::export::load_png_metadata(&path) {
                Ok(metadata) => {
                    // Apply metadata to app state
                    match self.load_from_metadata(metadata) {
                        Ok(()) => {
                            self.status_message = format!("✓ Loaded from: {}", path);
                            self.view_state.needs_redraw = true;
                        }
                        Err(e) => {
                            self.status_message = format!("❌ Load failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("❌ Failed to read PNG metadata: {}", e);
                }
            }
        }
        
        // Check if debounced input should trigger redraw
        if let Some(timer) = self.input.debounce_timer {
            if timer.elapsed() >= INPUT_DEBOUNCE_DELAY && self.input.pending_redraw {
                self.view_state.needs_redraw = true;
                self.input.pending_redraw = false;
                self.input.debounce_timer = None;
            } else if self.input.pending_redraw {
                // Keep requesting repaints until debounce delay is met
                ctx.request_repaint_after(INPUT_DEBOUNCE_DELAY - timer.elapsed());
            }
        }
        
        // Render fractal if needed
        if self.view_state.needs_redraw {
            self.render_fractal(ctx);
        }

        // Left sidebar with controls
        egui::SidePanel::left("controls")
            .default_width(350.0)
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
                                &mut self.input.width,
                                &mut self.input.height,
                                &mut self.view_state.view,
                                &mut self.view_state.needs_redraw,
                                &mut self.input.debounce_timer,
                                &mut self.input.pending_redraw,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Fractal Settings
                            gui::render_fractal_settings(
                                ui,
                                &mut self.input,
                                &mut self.view_state.needs_redraw,
                                &mut self.fractal.fractal_type,
                                &mut self.fractal.parameters,
                                &mut self.view_state.view,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Current View
                            gui::render_current_view_info(
                                ui,
                                &mut self.view_state.view,
                                &mut self.view_state.needs_redraw,
                                &mut self.status_message,
                                &mut self.input.debounce_timer,
                                &mut self.input.pending_redraw,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Color Scheme & Stops
                            {
                                let [r, g, b] = &mut self.color.interior_color_rgb_text;
                                gui::render_colormap_section(
                                    ui,
                                    &mut self.color.available_colormaps,
                                    &mut self.color.selected_colormap_name,
                                    &mut self.color.colormap,
                                    &mut self.view_state.needs_redraw,
                                    &mut self.status_message,
                                    &mut self.color.use_period,
                                    &mut self.input.period,
                                    &mut self.color.use_interior_color,
                                    &mut self.color.interior_color,
                                    r,
                                    g,
                                    b,
                                    &mut self.color.use_log_scale,
                                    &mut self.input.debounce_timer,
                                    &mut self.input.pending_redraw,
                                );
                            }

                            ui.add_space(10.0);

                            // Advanced Color Editor (always visible)
                        if forma_fractalis::colorschemes_gui::render_color_editor_section(
                                ui,
                                &mut self.color.colormap,
                                &mut self.color.color_editor,
                            ) {
                                self.view_state.needs_redraw = true;
                            }

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Actions
                            let max_iterations = self.input.parse_iterations();
                            let period = self.input.parse_period();
                            
                            // Create fractal instance for export
                            let mandelbrot = Mandelbrot::new();
                            let julia = Julia::new();
                            let burning_ship = BurningShip::new();
                            let tippets_mandelbrot = TippetsMandelbrot::new();
                            let multifractal_julia = MultifractalJulia::new();
                            let cactus = Cactus::new();
                            let marek_dragon = MarekDragon::new();
                            let tetration = Tetration::new();
                            
                            let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal.fractal_type {
                                FractalType::Mandelbrot => &mandelbrot,
                                FractalType::Julia => &julia,
                                FractalType::BurningShip => &burning_ship,
                                FractalType::TippetsMandelbrot => &tippets_mandelbrot,
                                FractalType::MultifractalJulia => &multifractal_julia,
                                FractalType::Cactus => &cactus,
                                FractalType::MarekDragon => &marek_dragon,
                                FractalType::Tetration => &tetration,
                            };
                            
                            gui::render_actions_section(
                                ui,
                                &self.view_state.view,
                                &self.color.colormap,
                                max_iterations,
                                fractal,
                                &self.fractal.parameters,
                                self.color.use_period,
                                period,
                                self.color.use_interior_color,
                                self.color.interior_color,
                                self.color.use_log_scale,
                                &mut self.input.export_scale,
                                &mut self.export.directory,
                                &mut self.export.filter,
                                &mut self.input.export_supersample,
                                &mut self.status_message,
                            );

                            // Export JSON button
                            gui::render_export_json_button(
                                ui,
                                &self.fractal,
                                &self.view_state,
                                &self.color,
                                &self.input,
                                &self.export,
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
            if let Some(texture) = &self.view_state.fractal_texture {
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
                if scroll_delta != 0.0 && self.mouse.is_dragging {
                    self.mouse.zoom_square_size = (self.mouse.zoom_square_size + scroll_delta * 2.0)
                        .max(20.0);
                    self.status_message = format!("Zoom size: {:.0}px", self.mouse.zoom_square_size);
                }

                // Handle left-click drag for zoom rectangle
                // Only start dragging if primary button is pressed (not secondary)
                if response.drag_started() && primary_down && !secondary_down {
                    self.mouse.is_dragging = true;
                    self.mouse.zoom_square_center = response.interact_pointer_pos();
                }

                // Update square position while dragging (only for primary button)
                if self.mouse.is_dragging && primary_down {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.mouse.zoom_square_center = Some(pos);
                    }
                }

                if response.drag_released() && self.mouse.is_dragging {
                    self.mouse.is_dragging = false;
                    if let Some(center) = self.mouse.zoom_square_center {
                        // Calculate zoom region
                        let center_rel = (center - rect.min) / scale;
                        let square_size_rel = self.mouse.zoom_square_size / scale;

                        let center_x = center_rel.x as u32;
                        let center_y = center_rel.y as u32;

                        // Calculate zoom factor based on square size relative to image size
                        let zoom_factor = texture_size.x / square_size_rel;

                        self.view_state.view.zoom_at(center_x, center_y, zoom_factor as f64);
                        self.view_state.needs_redraw = true;
                        self.status_message = format!("Zoomed to {:.2}x", self.view_state.view.zoom);
                    }
                    self.mouse.zoom_square_center = None;
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
                if self.mouse.is_dragging {
                    if let Some(center) = self.mouse.zoom_square_center {
                        let aspect_ratio = self.view_state.view.width as f32 / self.view_state.view.height as f32;
                        gui::render_zoom_square(ui, center, self.mouse.zoom_square_size, aspect_ratio);
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }
        });

        // Request repaint for smooth interaction
        if self.mouse.is_dragging {
            ctx.request_repaint();
        }
    }
}
