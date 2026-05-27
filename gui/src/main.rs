use eframe::egui;
use forma_fractalis::{
    app_state::{ViewState, InputState, FractalState, ColorState, MouseState, ExportState, RenderState, AnimationState, FractalType},
    fractals::{FractalView, Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia, SinhJulia, MultiJuliaIFS, AdjProbJulia, ChaosSymmetry1, LaceJulia}, 
    gpu::RenderBackend,
    gui, cli,
    perf_log, enable_profiling,
    rendering_pipeline::{colorize_iteration_cache_with_config, compute_iteration_cache_with_config, latest_pt_report, RenderConfig, RenderTarget},
};
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

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

/// Preview downscale used while debounced edits are still settling on slower backends.
const PROGRESSIVE_PREVIEW_SCALE: f32 = 0.5;
const PROGRESSIVE_PREVIEW_MIN_DIMENSION: u32 = 100;

fn progressive_preview_view(view: &FractalView) -> Option<FractalView> {
    let width = ((view.width as f32) * PROGRESSIVE_PREVIEW_SCALE)
        .round()
        .max(PROGRESSIVE_PREVIEW_MIN_DIMENSION as f32) as u32;
    let height = ((view.height as f32) * PROGRESSIVE_PREVIEW_SCALE)
        .round()
        .max(PROGRESSIVE_PREVIEW_MIN_DIMENSION as f32) as u32;
    let width = width.min(view.width);
    let height = height.min(view.height);

    if width == view.width && height == view.height {
        return None;
    }

    let mut preview = view.clone();
    preview.width = width;
    preview.height = height;
    Some(preview)
}

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
    pub render: RenderState,
    pub animation: AnimationState,

    // Status message
    pub status_message: String,
    
    // Animation progress channel
    animation_progress_rx: Option<Receiver<AnimationProgress>>,
    // Cancel token for the animation background thread
    animation_cancel: Option<Arc<AtomicBool>>,
}

/// Messages sent from the animation generation thread
#[derive(Debug, Clone)]
enum AnimationProgress {
    /// Progress update (current frame, total frames)
    Progress(u32, u32),
    /// Animation generation completed successfully
    Complete(std::path::PathBuf),
    /// Animation generation failed with error message
    Error(String),
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
            render: RenderState::new(),
            animation: AnimationState::new(),
            status_message: String::from("Ready - Click+drag to position zoom, scroll to resize"),
            animation_progress_rx: None,
            animation_cancel: None,
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
        let lemon = Lemon::new();
        let insideout_dragon = InsideoutDragon::new();
        let zubieta = Zubieta::new();
        let sin_julia = SinJulia::new();
        let sinh_julia = SinhJulia::new();
        let multi_julia_ifs = MultiJuliaIFS::new();
        let adj_prob_julia = AdjProbJulia::new();
        let chaos_symmetry1 = ChaosSymmetry1::new();
        let lace_julia = LaceJulia::new();
        
        let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal.fractal_type {
            FractalType::Mandelbrot => &mandelbrot,
            FractalType::Julia => &julia,
            FractalType::BurningShip => &burning_ship,
            FractalType::TippetsMandelbrot => &tippets_mandelbrot,
            FractalType::MultifractalJulia => &multifractal_julia,
            FractalType::Cactus => &cactus,
            FractalType::MarekDragon => &marek_dragon,
            FractalType::Tetration => &tetration,
            FractalType::Lemon => &lemon,
            FractalType::InsideoutDragon => &insideout_dragon,
            FractalType::Zubieta => &zubieta,
            FractalType::SinJulia => &sin_julia,
            FractalType::SinhJulia => &sinh_julia,
            FractalType::MultiJuliaIFS => &multi_julia_ifs,
            FractalType::AdjProbJulia => &adj_prob_julia,
            FractalType::ChaosSymmetry1 => &chaos_symmetry1,
            FractalType::LaceJulia => &lace_julia,
        };

        let preview_view = if self.render.progressive_preview_active {
            progressive_preview_view(&self.view_state.view)
        } else {
            None
        };
        let render_view = preview_view.as_ref().unwrap_or(&self.view_state.view);

        let buffer_size = (render_view.width * render_view.height * 4) as usize;
        let mut buffer = vec![0u8; buffer_size];

        let effective_bits = match self.render.backend {
            RenderBackend::CpuHiPrec => self.render.hiprec_bits,
            RenderBackend::Perturbation => self.render.pt_bits,
            _ => 0,
        };

        let use_cache = self.view_state.iteration_cache.as_ref().map_or(false, |cache| {
            cache.is_valid_for_render(
                render_view,
                max_iterations,
                fractal.name(),
                &self.fractal.parameters,
                self.render.backend,
                effective_bits,
                self.render.pt_glitch_tolerance,
                self.render.pt_tiles,
            )
        });

        let config = RenderConfig::new(
            render_view.clone(),
            &self.color.colormap,
            max_iterations,
            fractal,
        )
        .with_fractal_parameters(self.fractal.parameters.clone())
        .with_period(self.color.use_period, period)
        .with_interior_color(self.color.use_interior_color, self.color.interior_color)
        .with_log_scale(self.color.use_log_scale)
        .with_color_offset(self.color.color_offset)
        .with_backend(self.render.backend)
        .with_hiprec_bits(effective_bits)
        .with_max_threads(self.render.max_threads)
        .with_pt_glitch_tolerance(self.render.pt_glitch_tolerance)
        .with_pt_tiles(self.render.pt_tiles);

        if use_cache {
            perf_log!("[CACHE] Using cached iterations/density");
        } else {
            perf_log!("[{}] Computing preview cache: center=({:.10}, {:.10}), zoom={}",
                match self.render.backend {
                    RenderBackend::Cpu => "CPU",
                    RenderBackend::CpuHiPrec => "CPU-HIPREC",
                    RenderBackend::Gpu => "GPU",
                    RenderBackend::Perturbation => "PT",
                },
                render_view.center_x,
                render_view.center_y,
                gui::format_zoom_profile(render_view.zoom),
            );

            #[cfg(feature = "gpu")]
            let cache_result = compute_iteration_cache_with_config(
                &config,
                RenderTarget::Preview,
                self.render.gpu_renderer.as_mut(),
            );

            #[cfg(not(feature = "gpu"))]
            let cache_result = compute_iteration_cache_with_config(&config, RenderTarget::Preview);

            match cache_result {
                Ok(iteration_cache) => {
                    if matches!(self.render.backend, RenderBackend::Perturbation) {
                        if let Some(report) = latest_pt_report() {
                            let guidance = if report.low_zoom_warning {
                                "PT not needed at this zoom; CPU mode is sufficient".to_string()
                            } else if report.fallback_pct >= 75.0 || report.rebase_exhausted_count > 0 {
                                "poor PT case: try more tiles, move to a boundary/interior point, or switch to CPU Hi-Prec".to_string()
                            } else if report.rebased_pixel_count > 0 {
                                "rebasing is helping on this view".to_string()
                            } else {
                                "PT delta path is stable".to_string()
                            };

                            self.status_message = format!(
                                "PT {}x{} {}b | delta {:.1}% | hi-prec {:.1}% | SA {:.1}% avg {:.1} max {} | rebased px {:.1}% ({} events) | budget-hit {} | {}",
                                report.pt_tiles,
                                report.pt_tiles,
                                report.hiprec_bits,
                                if report.total_pixels > 0 {
                                    report.pt_pixels as f64 / report.total_pixels as f64 * 100.0
                                } else {
                                    0.0
                                },
                                report.fallback_pct,
                                report.sa_accepted_pixel_pct,
                                report.sa_avg_skipped_iterations,
                                report.sa_max_skipped_iterations,
                                report.rebased_pixel_pct,
                                report.rebase_count,
                                report.rebase_exhausted_count,
                                guidance,
                            );
                        }
                    }
                    self.view_state.iteration_cache = Some(iteration_cache);
                }
                Err(e) => {
                    self.status_message = format!("Render error: {}", e);
                    eprintln!("[ERROR] {}", e);
                    self.view_state.clear_redraw();
                    return;
                }
            }
        }

        let cache = self.view_state.iteration_cache.as_ref().unwrap();
        buffer = colorize_iteration_cache_with_config(&config, cache);

        self.finish_render(ctx, buffer, render_view.width, render_view.height);
    }

    fn finish_render(&mut self, ctx: &egui::Context, buffer: Vec<u8>, width: u32, height: u32) {
        // Convert to egui ColorImage
        let image_timer = Instant::now();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &buffer);
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
        
        // Check for animation progress updates from background thread
        if let Some(rx) = self.animation_progress_rx.as_ref() {
            let mut should_clear_channel = false;
            
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    AnimationProgress::Progress(current, total) => {
                        self.animation.progress = current as f32 / total as f32;
                        self.animation.progress_message = format!("Frame {}/{}", current, total);
                        ctx.request_repaint(); // Force UI update
                    }
                    AnimationProgress::Complete(path) => {
                        self.animation.generating = false;
                        self.animation.progress = 1.0;
                        self.animation.progress_message = String::new();
                        self.status_message = format!("Animation saved to: {}", path.display());
                        perf_log!("[ANIM] Thread reported complete: {}", path.display());
                        self.animation_cancel = None;
                        should_clear_channel = true;
                    }
                    AnimationProgress::Error(err) => {
                        self.animation.generating = false;
                        self.animation.progress = 0.0;
                        self.animation.progress_message = String::new();
                        self.status_message = if err == "Animation generation cancelled" {
                            "Animation cancelled.".to_string()
                        } else {
                            format!("Animation failed: {}", err)
                        };
                        perf_log!("[ANIM] Thread reported error: {}", err);
                        self.animation_cancel = None;
                        should_clear_channel = true;
                    }
                }
            }
            
            if should_clear_channel {
                self.animation_progress_rx = None; // Clean up channel
            }
        }
        
        // Check if we need to load a file (PNG or JSON)
        if self.status_message.starts_with("LOAD_FILE:") || self.status_message.starts_with("LOAD_PNG:") {
            let path = if self.status_message.starts_with("LOAD_FILE:") {
                self.status_message.strip_prefix("LOAD_FILE:").unwrap().to_string()
            } else {
                self.status_message.strip_prefix("LOAD_PNG:").unwrap().to_string()
            };
            let ext = std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let metadata_result = match ext.as_str() {
                "json" => forma_fractalis::export::load_json_metadata(&path),
                "png" => forma_fractalis::export::load_png_metadata(&path),
                other => Err(format!("Unsupported file type '.{}' - expected PNG or JSON", other)),
            };
            match metadata_result {
                Ok(metadata) => {
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
                    self.status_message = format!("❌ Failed to load: {}", e);
                }
            }
        }
        
        let progressive_preview_supported = matches!(self.render.backend, RenderBackend::Perturbation | RenderBackend::CpuHiPrec);

        // Check if debounced input should trigger redraw
        if let Some(timer) = self.input.debounce_timer {
            if timer.elapsed() >= INPUT_DEBOUNCE_DELAY && self.input.pending_redraw {
                self.render.progressive_preview_active = false;
                self.render.progressive_preview_stamp = None;
                self.view_state.needs_redraw = true;
                self.input.pending_redraw = false;
                self.input.debounce_timer = None;
            } else if self.input.pending_redraw {
                if progressive_preview_supported && !self.mouse.is_dragging {
                    let refresh_progressive = !self.render.progressive_preview_active
                        || self.render.progressive_preview_stamp.map_or(true, |stamp| stamp != timer);
                    if refresh_progressive {
                        self.render.progressive_preview_active = true;
                        self.render.progressive_preview_stamp = Some(timer);
                        self.view_state.needs_redraw = true;
                    }
                }
                // Keep requesting repaints until debounce delay is met
                ctx.request_repaint_after(INPUT_DEBOUNCE_DELAY - timer.elapsed());
            }
        } else if self.render.progressive_preview_active {
            self.render.progressive_preview_active = false;
            self.render.progressive_preview_stamp = None;
            self.view_state.needs_redraw = true;
        }

        if !progressive_preview_supported && self.render.progressive_preview_active {
            self.render.progressive_preview_active = false;
            self.render.progressive_preview_stamp = None;
            self.view_state.needs_redraw = true;
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
                                &mut self.view_state.preview_zoom,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Create fractal instance for GUI and export
                            let mandelbrot = Mandelbrot::new();
                            let julia = Julia::new();
                            let burning_ship = BurningShip::new();
                            let tippets_mandelbrot = TippetsMandelbrot::new();
                            let multifractal_julia = MultifractalJulia::new();
                            let cactus = Cactus::new();
                            let marek_dragon = MarekDragon::new();
                            let tetration = Tetration::new();
                            let lemon = Lemon::new();
                            let insideout_dragon = InsideoutDragon::new();
                            let zubieta = Zubieta::new();
                            let sin_julia = SinJulia::new();
                            let sinh_julia = SinhJulia::new();
                            let multi_julia_ifs = MultiJuliaIFS::new();
                            let adj_prob_julia = AdjProbJulia::new();
                            let chaos_symmetry1 = ChaosSymmetry1::new();
                            let lace_julia = LaceJulia::new();
                            
                            let fractal: &dyn forma_fractalis::fractals::Fractal = match self.fractal.fractal_type {
                                FractalType::Mandelbrot => &mandelbrot,
                                FractalType::Julia => &julia,
                                FractalType::BurningShip => &burning_ship,
                                FractalType::TippetsMandelbrot => &tippets_mandelbrot,
                                FractalType::MultifractalJulia => &multifractal_julia,
                                FractalType::Cactus => &cactus,
                                FractalType::MarekDragon => &marek_dragon,
                                FractalType::Tetration => &tetration,
                                FractalType::Lemon => &lemon,
                                FractalType::InsideoutDragon => &insideout_dragon,
                                FractalType::Zubieta => &zubieta,
                                FractalType::SinJulia => &sin_julia,
                                FractalType::SinhJulia => &sinh_julia,
                                FractalType::MultiJuliaIFS => &multi_julia_ifs,
                                FractalType::AdjProbJulia => &adj_prob_julia,
                                FractalType::ChaosSymmetry1 => &chaos_symmetry1,
                                FractalType::LaceJulia => &lace_julia,
                            };

                            // Performance / Rendering Backend
                            let prev_backend = self.render.backend;
                            gui::render_performance_section(
                                ui,
                                &mut self.render,
                                fractal,
                                &mut self.status_message,
                                &mut self.view_state.needs_redraw,
                            );
                            // Auto-scale preview ÷2 when entering CpuHiPrec, ×2 when leaving.
                            // Export scale is doubled/halved to keep final output dimensions constant.
                            let now_hiprec = matches!(self.render.backend, RenderBackend::CpuHiPrec);
                            let was_hiprec = matches!(prev_backend, RenderBackend::CpuHiPrec);
                            if !was_hiprec && now_hiprec {
                                // Entering hi-prec: save originals, halve preview dims,
                                // set display zoom to 0.5 so image occupies half the panel,
                                // and double export scale to keep final output size constant.
                                let saved_w = self.view_state.view.width;
                                let saved_h = self.view_state.view.height;
                                let saved_scale = self.input.export_scale.clone();
                                let saved_zoom = self.view_state.preview_zoom;
                                self.render.hiprec_preview_saved = Some((saved_w, saved_h, saved_scale, saved_zoom));

                                let new_w = (saved_w / 2).max(100);
                                let new_h = (saved_h / 2).max(100);
                                self.view_state.view.width = new_w;
                                self.view_state.view.height = new_h;
                                self.input.width = new_w.to_string();
                                self.input.height = new_h.to_string();

                                // Display at 0.5x panel fill so the preview physically
                                // occupies the same screen area as the halved texture —
                                // avoiding the "chunky upscale" appearance.
                                self.view_state.preview_zoom = 0.5;

                                let orig_scale = self.input.export_scale.parse::<f64>().unwrap_or(3.0).max(0.1);
                                self.input.export_scale = format!("{:.4}", orig_scale * 2.0);

                                self.view_state.needs_redraw = true;
                            } else if was_hiprec && !now_hiprec {
                                // Leaving hi-prec: restore saved originals
                                if let Some((saved_w, saved_h, saved_scale, saved_zoom)) = self.render.hiprec_preview_saved.take() {
                                    self.view_state.view.width = saved_w;
                                    self.view_state.view.height = saved_h;
                                    self.input.width = saved_w.to_string();
                                    self.input.height = saved_h.to_string();
                                    self.input.export_scale = saved_scale;
                                    self.view_state.preview_zoom = saved_zoom;
                                }
                                self.view_state.needs_redraw = true;
                            }

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
                                self.render.backend,
                            );

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Current View
                            let active_precision_bits = match self.render.backend {
                                RenderBackend::CpuHiPrec => self.render.hiprec_bits,
                                RenderBackend::Perturbation => self.render.pt_bits,
                                _ => 64,
                            };
                            gui::render_current_view_info(
                                ui,
                                &mut self.view_state.view,
                                active_precision_bits,
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
                                    &mut self.color.interior_picker,
                                    &mut self.color.use_log_scale,
                                    &mut self.color.color_offset,
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
                                &mut self.color.stop_picker,
                            ) {
                                self.view_state.needs_redraw = true;
                            }

                            ui.add_space(15.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Actions
                            let max_iterations = self.input.parse_iterations();
                            let period = self.input.parse_period();
                            
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
                                self.color.color_offset,
                                &mut self.input.export_scale,
                                &mut self.export.directory,
                                &mut self.export.filter,
                                &mut self.input.export_supersample,
                                &mut self.render,
                                &mut self.status_message,
                                &mut self.export.last_export_path,
                                &mut self.view_state.needs_redraw,
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

                            // Animation Generation
                            {
                                let action = gui::render_animation_section(
                                    ui,
                                    &mut self.animation,
                                    &self.view_state,
                                    &self.color,
                                    &self.fractal,
                                    max_iterations,
                                    &self.input.export_scale,
                                    &self.export.filter,
                                    &self.input.export_supersample,
                                    self.export.directory.as_ref(),
                                    &mut self.status_message,
                                );
                                
                                match action {
                                    gui::AnimationAction::Start => {
                                        self.start_animation_generation(ctx.clone(), max_iterations, period, fractal);
                                    }
                                    gui::AnimationAction::Cancel => {
                                        if let Some(token) = &self.animation_cancel {
                                            token.store(true, Ordering::Relaxed);
                                            perf_log!("[ANIM] Cancellation requested by user");
                                        }
                                    }
                                    gui::AnimationAction::None => {}
                                }
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
            if self.render.progressive_preview_active {
                ui.label(
                    egui::RichText::new("Preview refining at 50% resolution while debounced edits settle")
                        .small()
                        .italics(),
                );
                ui.add_space(6.0);
            }

            if let Some(texture) = &self.view_state.fractal_texture {
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();
                // fit_scale fills the panel; multiply by preview_zoom (0.1–1.0) to shrink.
                // This lets the user (or the HiPrec auto-scale) reduce the display size so
                // the image is rendered at its native texel density rather than upscaled.
                let fit_scale =
                    (available_size.x / texture_size.x).min(available_size.y / texture_size.y);
                let scale = fit_scale * self.view_state.preview_zoom;
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
                        let x_frac = (center_rel.x / texture_size.x).clamp(0.0, 1.0) as f64;
                        let y_frac = (center_rel.y / texture_size.y).clamp(0.0, 1.0) as f64;

                        // Calculate zoom factor based on square size relative to image size
                        let zoom_factor = texture_size.x / square_size_rel;

                        match self.render.backend {
                            RenderBackend::CpuHiPrec => {
                                self.view_state.view.zoom_at_fraction_hiprec(
                                    x_frac,
                                    y_frac,
                                    zoom_factor as f64,
                                    self.render.hiprec_bits,
                                );
                            }
                            RenderBackend::Perturbation => {
                                self.view_state.view.zoom_at_fraction_hiprec(
                                    x_frac,
                                    y_frac,
                                    zoom_factor as f64,
                                    self.render.pt_bits,
                                );
                            }
                            _ => {
                                self.view_state.view.zoom_at_fraction(
                                    x_frac,
                                    y_frac,
                                    zoom_factor as f64,
                                );
                            }
                        }

                        self.view_state.iteration_cache = None;
                        self.view_state.needs_redraw = true;
                        self.status_message = format!("Zoomed to {}", self.view_state.view.zoom_display());
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

        //Request repaint for smooth interaction
        if self.mouse.is_dragging {
            ctx.request_repaint();
        }
    }
}

impl FractalApp {
    /// Start animation generation in a background thread
    fn start_animation_generation(
        &mut self,
        ctx: egui::Context,
        max_iterations: u32,
        period: u32,
        fractal: &dyn forma_fractalis::fractals::Fractal,
    ) {
        use forma_fractalis::animation::{AnimationConfig, AnimationType as AnimType};
        use forma_fractalis::app_state::AnimationType;
        
        // Create channel for progress updates
        let (tx, rx) = mpsc::channel();
        self.animation_progress_rx = Some(rx);

        // Create cancel token for this generation run
        let cancel_token = Arc::new(AtomicBool::new(false));
        self.animation_cancel = Some(cancel_token.clone());
        let cancel_token_thread = cancel_token;
        
        // Clone data needed for thread
        let animation_state = self.animation.clone();
        let view = self.view_state.view.clone();
        let colormap = self.color.colormap.clone();
        let fractal_params = self.fractal.parameters.clone();
        let fractal_name = fractal.name().to_string();
        
        // Clone color modulation settings (these affect how iteration counts
        // map to colors — missing these was the root cause of color/zoom settings
        // not being honored in GIF frames).
        let use_period = self.color.use_period;
        let color_period = period;
        let use_interior_color = self.color.use_interior_color;
        let interior_color = self.color.interior_color;
        let use_log_scale = self.color.use_log_scale;

        // Clone export settings for rendering each frame
        let export_scale = self.input.parse_export_scale();
        let export_filter = self.export.filter;
        let export_supersample = self.input.parse_export_supersample();
        let render_backend = self.render.backend;
        // Use pt_bits for Perturbation backend, hiprec_bits for HiPrec — they govern
        // the same thing (fallback/orbit precision) in their respective backends.
        let hiprec_bits = if matches!(render_backend, RenderBackend::Perturbation) {
            self.render.pt_bits
        } else {
            self.render.hiprec_bits
        };
        let max_threads = self.render.max_threads;
        let output_dir = self.export.directory.as_ref()
            .expect("Output directory should be set").clone();
        
        // Spawn background thread
        std::thread::spawn(move || {
            // Build filename matching PNG export convention:
            // animation_{fractal_name}_{width}x{height}{filter_suffix}{unix_timestamp}.gif
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let (out_w, out_h) = forma_fractalis::export::calculate_output_dimensions(&view, export_scale as f32);
            let filter_suffix = if export_filter != forma_fractalis::filtering::FilterType::None && export_supersample > 1 {
                format!("_{}x{}", export_supersample, export_filter.as_str())
            } else {
                String::new()
            };
            let fractal_slug = fractal_name.to_lowercase().replace(' ', "_");
            let filename = format!(
                "animation_{}_{}x{}{}{}.gif",
                fractal_slug, out_w, out_h, filter_suffix, timestamp
            );
            let output_path = output_dir.join(filename);
            
            let anim_type = match animation_state.animation_type {
                AnimationType::Zoom => AnimType::Zoom {
                    from_zoom: animation_state.parse_zoom_from(),
                    to_zoom: animation_state.parse_zoom_to(),
                    center_x: view.center_x,
                    center_y: view.center_y,
                },
                AnimationType::JuliaParamSweep => {
                    let _ = tx.send(AnimationProgress::Error(
                        "Julia Parameter Sweep is not available in this version.".to_string()
                    ));
                    return;
                }
                AnimationType::IterationFade => AnimType::IterationFade {
                    from_iterations: animation_state.parse_iter_from(),
                    to_iterations: animation_state.parse_iter_to(),
                },
            };
            
            let config = AnimationConfig::new(
                anim_type,
                animation_state.parse_num_frames(),
                animation_state.parse_fps(),
                output_path.clone(),
                // Use scaled dimensions so AnimationConfig matches the buffer render_frame produces.
                // This is the same calculation as render_frame -> calculate_output_dimensions.
                {
                    let (w, _) = forma_fractalis::export::calculate_output_dimensions(&view, export_scale as f32);
                    w
                },
                {
                    let (_, h) = forma_fractalis::export::calculate_output_dimensions(&view, export_scale as f32);
                    h
                },
            );

            perf_log!("[ANIM] Thread started: {} frames, {}fps, {}x{}, backend={:?}, output={}",
                config.num_frames, config.fps, config.width, config.height,
                render_backend, config.output_path.display()
            );
            
            // Create fractal instance in thread
            let fractal: Box<dyn forma_fractalis::fractals::Fractal> = match fractal_name.as_str() {
                "Mandelbrot" => Box::new(Mandelbrot::new()),
                "Julia Set" => Box::new(Julia::new()),
                "Burning Ship" => Box::new(BurningShip::new()),
                "Tippets Mandelbrot" => Box::new(TippetsMandelbrot::new()),
                "Multifractal Julia" => Box::new(MultifractalJulia::new()),
                "Cactus" => Box::new(Cactus::new()),
                "Marek Dragon" => Box::new(MarekDragon::new()),
                "Tetration" => Box::new(Tetration::new()),
                "Lemon" => Box::new(Lemon::new()),
                "Insideout Dragon" => Box::new(InsideoutDragon::new()),
                "Zubieta" => Box::new(Zubieta::new()),
                "Sin Julia" => Box::new(SinJulia::new()),
                "Sinh Julia" => Box::new(SinhJulia::new()),
                "Multi-Julia IFS" => Box::new(MultiJuliaIFS::new()),
                "Adj Prob Julia" => Box::new(AdjProbJulia::new()),
                "ChaosSymmetry1" => Box::new(ChaosSymmetry1::new()),
                "Lace Julia" => Box::new(LaceJulia::new()),
                _ => Box::new(Mandelbrot::new()), // Fallback
            };
            
            // Generate animation with progress callback
            let result = forma_fractalis::animation::generate_animation(
                &config,
                &view,
                &colormap,
                max_iterations,
                use_period,
                color_period,
                use_interior_color,
                interior_color,
                use_log_scale,
                fractal.as_ref(),
                &fractal_params,
                export_scale,
                export_filter,
                export_supersample,
                render_backend,
                hiprec_bits,
                max_threads,
                Some(cancel_token_thread),
                Some(|current, total| {
                    let _ = tx.send(AnimationProgress::Progress(current, total));
                    ctx.request_repaint(); // Request UI update
                }),
            );
            
            // Send completion message
            match result {
                Ok(path) => {
                    let _ = tx.send(AnimationProgress::Complete(path));
                }
                Err(e) => {
                    let _ = tx.send(AnimationProgress::Error(e));
                }
            }
            ctx.request_repaint(); // Final UI update
        });
    }
}

