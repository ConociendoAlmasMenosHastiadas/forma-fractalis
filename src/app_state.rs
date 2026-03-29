//! Application State Management
//!
//! This module defines grouped state structures for the fractal application.
//! Instead of having 30+ fields scattered in a single struct, we organize
//! related state into logical groups, making the codebase more maintainable
//! and reducing function parameter counts from 10-15 to 3-5.

use scala_chromatica::ColorMap;
use crate::color_picker::ColorPickerState;
use crate::colorschemes_gui::ColorEditor;
use crate::filtering::FilterType;
use crate::fractals::FractalView;
use eframe::egui;
use std::collections::HashMap;
use std::time::Instant;

/// Coordinate input mode for complex parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateMode {
    Rectangular,
    Polar,
}

impl Default for CoordinateMode {
    fn default() -> Self {
        CoordinateMode::Rectangular
    }
}

/// View and rendering state
#[derive(Clone)]
pub struct ViewState {
    pub view: FractalView,
    pub fractal_texture: Option<egui::TextureHandle>,
    pub needs_redraw: bool,
    pub iteration_cache: Option<IterationCache>,
    /// Display zoom factor for the preview panel ([0.1, 1.0]).
    /// 1.0 = fit-to-panel (fills available space); 0.5 = half the panel.
    /// Auto-set to 0.5 when CpuHiPrec is selected and restored on exit.
    pub preview_zoom: f32,
}

impl ViewState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            view: FractalView::new(width, height),
            fractal_texture: None,
            needs_redraw: true,
            iteration_cache: None,
            preview_zoom: 1.0,
        }
    }

    /// Mark the view for redraw
    pub fn mark_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Clear redraw flag after rendering
    pub fn clear_redraw(&mut self) {
        self.needs_redraw = false;
    }
}

/// Text input state with debouncing
pub struct InputState {
    pub width: String,
    pub height: String,
    pub iterations: String,
    pub julia_c_real: String,
    pub julia_c_imag: String,
    pub julia_magnitude: String,
    pub julia_angle: String,
    pub julia_coord_mode: CoordinateMode,
    pub mandelbrot_power: String,
    pub multifractal_julia_power: String,
    pub marek_dragon_phi: String,
    pub tetration_threshold: String,
    pub lemon_convergence_exp: String,
    pub lemon_denom_power: String,
    pub insideout_dragon_escape_radius: String,
    pub zubieta_c_real: String,
    pub zubieta_c_imag: String,
    pub zubieta_magnitude: String,
    pub zubieta_angle: String,
    pub zubieta_coord_mode: CoordinateMode,
    pub sin_julia_c_real: String,
    pub sin_julia_c_imag: String,
    pub sin_julia_magnitude: String,
    pub sin_julia_angle: String,
    pub sin_julia_coord_mode: CoordinateMode,
    pub sin_julia_escape_radius: String,
    pub period: String,
    pub export_scale: String,
    pub export_supersample: String,
    
    // Debouncing
    pub debounce_timer: Option<Instant>,
    pub pending_redraw: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            width: String::from("1280"),
            height: String::from("720"),
            iterations: String::from("256"),
            julia_c_real: String::from("0.0"),
            julia_c_imag: String::from("0.0"),
            julia_magnitude: String::from("0.0"),
            julia_angle: String::from("0.0"),
            julia_coord_mode: CoordinateMode::default(),
            mandelbrot_power: String::from("2.0"),
            multifractal_julia_power: String::from("1.0"),
            marek_dragon_phi: String::from("0.0"),
            tetration_threshold: String::from("1e7"),
            lemon_convergence_exp: String::from("6"),
            lemon_denom_power: String::from("2.0"),
            insideout_dragon_escape_radius: String::from("4.0"),
            zubieta_c_real: String::from("0.0"),
            zubieta_c_imag: String::from("0.8"),
            zubieta_magnitude: String::from("0.8"),
            zubieta_angle: String::from("1.5707963267949"),
            zubieta_coord_mode: CoordinateMode::default(),
            sin_julia_c_real: String::from("1.0"),
            sin_julia_c_imag: String::from("0.1"),
            sin_julia_magnitude: String::from("1.0049875621120890"),
            sin_julia_angle: String::from("0.09966865249116204"),
            sin_julia_coord_mode: CoordinateMode::default(),
            sin_julia_escape_radius: String::from("50.0"),
            period: String::from("128"),
            export_scale: String::from("3.0"),
            export_supersample: String::from("4"),
            debounce_timer: None,
            pending_redraw: false,
        }
    }
}

impl InputState {
    /// Parse iterations input with default fallback
    pub fn parse_iterations(&self) -> u32 {
        self.iterations.parse::<u32>().unwrap_or(256).max(1)
    }

    /// Parse period input with default fallback
    pub fn parse_period(&self) -> u32 {
        self.period.parse::<u32>().unwrap_or(256)
    }

    /// Parse export scale with default fallback
    pub fn parse_export_scale(&self) -> f64 {
        self.export_scale.parse::<f64>().unwrap_or(3.0).max(1.0)
    }

    /// Parse export supersample with default fallback
    pub fn parse_export_supersample(&self) -> u32 {
        self.export_supersample.parse::<u32>().unwrap_or(4).max(1)
    }

    /// Parse Julia c_real parameter
    pub fn parse_julia_c_real(&self) -> f64 {
        self.julia_c_real.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Julia c_imag parameter
    pub fn parse_julia_c_imag(&self) -> f64 {
        self.julia_c_imag.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Mandelbrot power parameter
    pub fn parse_mandelbrot_power(&self) -> f64 {
        self.mandelbrot_power.parse::<f64>().unwrap_or(2.0)
    }

    /// Parse Multifractal-Julia power parameter (k in z_{n+1} = c^k * z_{n-2} + c)
    pub fn parse_multifractal_julia_power(&self) -> f64 {
        self.multifractal_julia_power.parse::<f64>().unwrap_or(1.0)
    }

    /// Parse Marek Dragon phi parameter (rotation angle 0 to 2π)
    pub fn parse_marek_dragon_phi(&self) -> f64 {
        self.marek_dragon_phi.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Tetration threshold parameter (supports scientific notation)
    pub fn parse_tetration_threshold(&self) -> f64 {
        self.tetration_threshold.parse::<f64>().unwrap_or(1e7).max(1.0)
    }

    /// Parse Lemon convergence exponent parameter
    pub fn parse_lemon_convergence_exp(&self) -> f64 {
        self.lemon_convergence_exp.parse::<f64>().unwrap_or(6.0).clamp(1.0, 15.0)
    }

    /// Parse Lemon denominator power parameter
    pub fn parse_lemon_denom_power(&self) -> f64 {
        self.lemon_denom_power.parse::<f64>().unwrap_or(2.0).clamp(-5.0, 5.0)
    }

    /// Parse Insideout Dragon escape radius parameter
    pub fn parse_insideout_dragon_escape_radius(&self) -> f64 {
        self.insideout_dragon_escape_radius.parse::<f64>().unwrap_or(4.0)
    }

    /// Parse Zubieta c_real parameter
    pub fn parse_zubieta_c_real(&self) -> f64 {
        self.zubieta_c_real.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Zubieta c_imag parameter
    pub fn parse_zubieta_c_imag(&self) -> f64 {
        self.zubieta_c_imag.parse::<f64>().unwrap_or(0.8)
    }

    /// Parse Sin Julia c_real parameter
    pub fn parse_sin_julia_c_real(&self) -> f64 {
        self.sin_julia_c_real.parse::<f64>().unwrap_or(1.0)
    }

    /// Parse Sin Julia c_imag parameter
    pub fn parse_sin_julia_c_imag(&self) -> f64 {
        self.sin_julia_c_imag.parse::<f64>().unwrap_or(0.1)
    }

    /// Parse Sin Julia escape radius parameter
    pub fn parse_sin_julia_escape_radius(&self) -> f64 {
        self.sin_julia_escape_radius.parse::<f64>().unwrap_or(50.0)
    }
}

/// Iteration data cache for fast recoloring
/// 
/// Stores the raw iteration counts for each pixel, allowing color schemes
/// to be changed without recomputing the fractal. This cache is invalidated
/// when any rendering parameter changes (view, fractal type, parameters, etc.)
/// but remains valid when only color-related settings change.
#[derive(Clone)]
pub struct IterationCache {
    /// Iteration count for each pixel (row-major order)
    pub data: Vec<u32>,
    /// Width of the cached render
    pub width: u32,
    /// Height of the cached render
    pub height: u32,
    /// Maximum iterations used for this cache
    pub max_iterations: u32,
    /// Fractal type that generated this cache
    pub fractal_type: FractalType,
    /// Fractal parameters used for this cache
    pub fractal_parameters: HashMap<String, f64>,
    /// View coordinates used for this cache
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    /// Backend that computed the iterations (CPU f64 vs CpuHiPrec BigFloat produce different counts)
    pub backend: crate::gpu::RenderBackend,
    /// Bit width used when backend == CpuHiPrec; ignored for other backends
    pub hiprec_bits: u32,
}

impl IterationCache {
    /// Check if the cache is valid for the given rendering parameters
    pub fn is_valid(
        &self,
        width: u32,
        height: u32,
        max_iterations: u32,
        fractal_type: FractalType,
        fractal_parameters: &HashMap<String, f64>,
        view: &FractalView,
        backend: crate::gpu::RenderBackend,
        hiprec_bits: u32,
    ) -> bool {
        self.width == width
            && self.height == height
            && self.max_iterations == max_iterations
            && self.fractal_type == fractal_type
            && self.fractal_parameters == *fractal_parameters
            && (self.center_x - view.center_x).abs() < 1e-10
            && (self.center_y - view.center_y).abs() < 1e-10
            && (self.zoom - view.zoom).abs() < 1e-10
            && self.backend == backend
            && self.hiprec_bits == hiprec_bits
    }
}

/// Fractal type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
    TippetsMandelbrot,
    MultifractalJulia,
    Cactus,
    MarekDragon,
    Tetration,
    Lemon,
    InsideoutDragon,
    Zubieta,
    SinJulia,
}

impl FractalType {
    pub fn as_str(&self) -> &str {
        match self {
            FractalType::Mandelbrot => "Mandelbrot",
            FractalType::Julia => "Julia",
            FractalType::BurningShip => "Burning Ship",
            FractalType::TippetsMandelbrot => "Tippets Mandelbrot",
            FractalType::MultifractalJulia => "Multifractal-Julia",
            FractalType::Cactus => "Cactus",
            FractalType::MarekDragon => "Marek Dragon",
            FractalType::Tetration => "Tetration",
            FractalType::Lemon => "Lemon",
            FractalType::InsideoutDragon => "Insideout Dragon",
            FractalType::Zubieta => "Zubieta",
            FractalType::SinJulia => "Sin Julia",
        }
    }

    pub fn name(&self) -> &str {
        self.as_str()
    }

    /// Get the mathematical equation for this fractal type
    pub fn equation(&self) -> &str {
        match self {
            FractalType::Mandelbrot => "z_{n+1} = z_n^p + c",
            FractalType::Julia => "z_{n+1} = z_n^2 + c",
            FractalType::BurningShip => "z_{n+1} = (|Re(z_n)| + i|Im(z_n)|)^2 + c",
            FractalType::TippetsMandelbrot => "z_{n+1} = z_n^2 + c*z_n + c",
            FractalType::MultifractalJulia => "z_{n+1} = c^k * z_n^(-2) + c",
            FractalType::Cactus => "z_{n+1} = z_n^3 + (z_0 - 1)*z_n - z_0",
            FractalType::MarekDragon => "z_{n+1} = exp(iφ)*z_n + z_n^2",
            FractalType::Tetration => "z_{n+1} = c^(z_n)",
            FractalType::Lemon => "z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k",
            FractalType::InsideoutDragon => "z_{n+1} = z_n^2 + f(|z_n|) + i*g(|z_n|), z_0 = 1/c",
            FractalType::Zubieta => "z_{n+1} = z_n^2 + c/z_n",
            FractalType::SinJulia => "z_{n+1} = c * sin(z_n)",
        }
    }

    pub fn all() -> &'static [FractalType] {
        &[
            FractalType::Mandelbrot,
            FractalType::Julia,
            FractalType::BurningShip,
            FractalType::TippetsMandelbrot,
            FractalType::MultifractalJulia,
            FractalType::Cactus,
            FractalType::MarekDragon,
            FractalType::Tetration,
            FractalType::Lemon,
            FractalType::Zubieta,
            FractalType::SinJulia,
            FractalType::InsideoutDragon,
        ]
    }

    /// Creates a fractal instance from the enum type
    pub fn create_instance(&self) -> Box<dyn crate::fractals::Fractal> {
        use crate::fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia};
        
        match self {
            FractalType::Mandelbrot => Box::new(Mandelbrot::new()),
            FractalType::Julia => Box::new(Julia::new()),
            FractalType::BurningShip => Box::new(BurningShip::new()),
            FractalType::TippetsMandelbrot => Box::new(TippetsMandelbrot::new()),
            FractalType::MultifractalJulia => Box::new(MultifractalJulia::new()),
            FractalType::Cactus => Box::new(Cactus::new()),
            FractalType::MarekDragon => Box::new(MarekDragon::new()),
            FractalType::Tetration => Box::new(Tetration::new()),
            FractalType::Lemon => Box::new(Lemon::new()),
            FractalType::InsideoutDragon => Box::new(InsideoutDragon::new()),
            FractalType::Zubieta => Box::new(Zubieta::new()),
            FractalType::SinJulia => Box::new(SinJulia::new()),
        }
    }

    /// Render GUI parameters for this fractal type using the FractalGUI trait
    pub fn render_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        use crate::fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia, FractalGUI};
        
        match self {
            FractalType::Mandelbrot => {
                let fractal = Mandelbrot::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::Julia => {
                let fractal = Julia::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::BurningShip => {
                let fractal = BurningShip::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::TippetsMandelbrot => {
                let fractal = TippetsMandelbrot::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::MultifractalJulia => {
                let fractal = MultifractalJulia::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::Cactus => {
                let fractal = Cactus::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::MarekDragon => {
                let fractal = MarekDragon::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::Tetration => {
                let fractal = Tetration::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::Lemon => {
                let fractal = Lemon::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::InsideoutDragon => {
                let fractal = InsideoutDragon::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::Zubieta => {
                let fractal = Zubieta::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
            FractalType::SinJulia => {
                let fractal = SinJulia::new();
                fractal.render_parameters_gui(ui, params, input_state, needs_redraw);
            }
        }
    }

    pub fn is_julia(&self) -> bool {
        matches!(self, FractalType::Julia)
    }

    pub fn is_mandelbrot(&self) -> bool {
        matches!(self, FractalType::Mandelbrot)
    }

    /// Reset view and fractal parameters for this fractal type
    pub fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        input: &InputState,
    ) {
        match self {
            FractalType::Mandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 1.0;
                // Keep existing power parameter when switching back
                let power = input.parse_mandelbrot_power();
                params.clear();
                params.insert("power".to_string(), power);
            }
            FractalType::Julia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                // Keep existing Julia parameters when switching back
                params.insert("c_real".to_string(), input.parse_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_julia_c_imag());
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
            FractalType::MultifractalJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
                params.insert("power".to_string(), 1.0); // Default to k=1: z_{n+1} = c * z_{n-2} + c
            }
            FractalType::Cactus => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.6;
                params.clear();
            }
            FractalType::MarekDragon => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
                params.insert("phi".to_string(), input.parse_marek_dragon_phi());
            }
            FractalType::Tetration => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.5;
                params.clear();
                params.insert("threshold".to_string(), input.parse_tetration_threshold());
                params.insert("escape_mode".to_string(), 0.0); // Default to Magnitude mode
            }
            FractalType::Lemon => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.5;
                params.clear();
                params.insert("convergence_exp".to_string(), input.parse_lemon_convergence_exp());
                params.insert("denom_power".to_string(), input.parse_lemon_denom_power());
            }
            FractalType::InsideoutDragon => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.25;
                params.clear();
                params.insert("escape_radius".to_string(), input.parse_insideout_dragon_escape_radius());
            }
            FractalType::Zubieta => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.7;
                params.clear();
                params.insert("c_real".to_string(), input.parse_zubieta_c_real());
                params.insert("c_imag".to_string(), input.parse_zubieta_c_imag());
            }
            FractalType::SinJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.4;
                params.clear();
                params.insert("c_real".to_string(), input.parse_sin_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_sin_julia_c_imag());
                params.insert("escape_radius".to_string(), input.parse_sin_julia_escape_radius());
            }
        }
    }
}

// GUI trait implementation (bridge for backwards compatibility with old gui.rs functions)
impl crate::gui::FractalTypeOps for FractalType {
    fn get_name(&self) -> &str {
        self.name()
    }

    fn get_equation(&self) -> &str {
        self.equation()
    }

    fn all_types() -> Vec<Self> {
        Self::all().to_vec()
    }

    fn is_julia(&self) -> bool {
        self.is_julia()
    }

    fn is_mandelbrot(&self) -> bool {
        self.is_mandelbrot()
    }

    fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        input: &InputState,
    ) {
        // Directly call the native method
        self.reset_view_and_params(view, params, input);
    }

    fn render_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        // Directly call the native method
        self.render_gui(ui, params, input_state, needs_redraw);
    }
}

/// Fractal configuration state
pub struct FractalState {
    pub fractal_type: FractalType,
    pub parameters: HashMap<String, f64>,
}

impl Default for FractalState {
    fn default() -> Self {
        Self {
            fractal_type: FractalType::Mandelbrot,
            parameters: HashMap::new(),
        }
    }
}

impl FractalState {
    pub fn new(fractal_type: FractalType) -> Self {
        Self {
            fractal_type,
            parameters: HashMap::new(),
        }
    }

    /// Set a fractal parameter
    pub fn set_parameter(&mut self, key: &str, value: f64) {
        self.parameters.insert(key.to_string(), value);
    }

    /// Get a fractal parameter
    pub fn get_parameter(&self, key: &str) -> Option<f64> {
        self.parameters.get(key).copied()
    }
}

/// Colormap and modulation state
pub struct ColorState {
    pub available_colormaps: Vec<String>,
    pub selected_colormap_name: String,
    pub colormap: ColorMap,
    pub color_editor: ColorEditor,
    
    // Color picker states
    pub interior_picker: ColorPickerState,
    pub stop_picker: ColorPickerState,
    
    // Color modulation options
    pub use_period: bool,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub interior_color_rgb_text: [String; 3],
    pub use_log_scale: bool,
}

impl Default for ColorState {
    fn default() -> Self {
        use scala_chromatica::io as colorschemes_io;

        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        let selected_colormap_name = String::from("Default");
        let colormap = colorschemes_io::load_colormap(&selected_colormap_name)
            .unwrap_or_else(|_| ColorMap::default_scheme());

        Self {
            available_colormaps,
            selected_colormap_name,
            colormap,
            color_editor: ColorEditor::new(),
            interior_picker: ColorPickerState::default(),
            stop_picker: ColorPickerState::default(),
            use_period: false,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            interior_color_rgb_text: [
                String::from("0"),
                String::from("0"),
                String::from("0"),
            ],
            use_log_scale: false,
        }
    }
}

impl ColorState {
    /// Parse interior color RGB values from text inputs
    pub fn parse_interior_color(&mut self) {
        if let Ok(r) = self.interior_color_rgb_text[0].parse::<u8>() {
            self.interior_color[0] = r;
        }
        if let Ok(g) = self.interior_color_rgb_text[1].parse::<u8>() {
            self.interior_color[1] = g;
        }
        if let Ok(b) = self.interior_color_rgb_text[2].parse::<u8>() {
            self.interior_color[2] = b;
        }
    }
}

/// Mouse interaction state
#[derive(Default)]
pub struct MouseState {
    pub is_dragging: bool,
    pub zoom_square_center: Option<egui::Pos2>,
    pub zoom_square_size: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 200.0,
        }
    }

    /// Clear zoom square
    pub fn clear_zoom(&mut self) {
        self.zoom_square_center = None;
        self.is_dragging = false;
    }
}

/// Export configuration state
pub struct ExportState {
    pub directory: Option<std::path::PathBuf>,
    pub filter: FilterType,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            directory: None,
            filter: FilterType::None,
        }
    }
}

impl ExportState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Rendering backend state
pub struct RenderState {
    pub backend: crate::gpu::RenderBackend,
    /// Bit width used when backend == CpuHiPrec. Must be one of HIPREC_BIT_OPTIONS.
    pub hiprec_bits: u32,
    /// Maximum rayon threads for CPU rendering.
    /// 0 = use all available threads (rayon default). 1..N = limited parallelism.
    pub max_threads: usize,
    /// Saved (width, height, export_scale_string, preview_zoom) from before CpuHiPrec was selected.
    /// Set when switching TO CpuHiPrec, cleared when switching away.
    /// Used to restore exact original dimensions and display zoom without floating-point drift.
    pub hiprec_preview_saved: Option<(u32, u32, String, f32)>,
    #[cfg(feature = "gpu")]
    pub gpu_renderer: Option<crate::gpu::WgpuRenderer>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            backend: crate::gpu::RenderBackend::default(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            max_threads: 0,
            hiprec_preview_saved: None,
            #[cfg(feature = "gpu")]
            gpu_renderer: None,  // Lazy initialization on first use
        }
    }
}

impl RenderState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize GPU renderer if not already initialized
    #[cfg(feature = "gpu")]
    pub fn ensure_gpu_initialized(&mut self) -> Result<(), String> {
        if self.gpu_renderer.is_none() {
            match crate::gpu::WgpuRenderer::new() {
                Ok(renderer) => {
                    self.gpu_renderer = Some(renderer);
                    Ok(())
                }
                Err(e) => {
                    // Fall back to CPU if GPU initialization fails
                    self.backend = crate::gpu::RenderBackend::Cpu;
                    Err(format!("GPU initialization failed, falling back to CPU: {}", e))
                }
            }
        } else {
            Ok(())
        }
    }
}

/// Animation generation state
#[derive(Clone)]
pub struct AnimationState {
    /// Selected animation type
    pub animation_type: AnimationType,
    
    /// Number of frames
    pub num_frames: u32,
    pub num_frames_text: String,
    
    /// Frames per second
    pub fps: u8,
    pub fps_text: String,
    
    /// Zoom animation parameters
    pub zoom_from: f64,
    pub zoom_from_text: String,
    pub zoom_to: f64,
    pub zoom_to_text: String,
    
    /// Julia parameter sweep parameters
    pub julia_from_real: f64,
    pub julia_from_real_text: String,
    pub julia_from_imag: f64,
    pub julia_from_imag_text: String,
    pub julia_to_real: f64,
    pub julia_to_real_text: String,
    pub julia_to_imag: f64,
    pub julia_to_imag_text: String,
    
    /// Iteration fade parameters
    pub iter_from: u32,
    pub iter_from_text: String,
    pub iter_to: u32,
    pub iter_to_text: String,
    
    /// Progress tracking
    pub generating: bool,
    pub progress: f32,
    pub progress_message: String,
}

/// Animation type selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Zoom,
    JuliaParamSweep,
    IterationFade,
    // ColormapTransition,  // TODO: Implement in future version
}

impl Default for AnimationType {
    fn default() -> Self {
        AnimationType::Zoom
    }
}

impl AnimationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnimationType::Zoom => "Zoom Sequence",
            AnimationType::JuliaParamSweep => "Julia Parameter Sweep",
            AnimationType::IterationFade => "Iteration Fade-In",
        }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::default(),
            num_frames: 60,
            num_frames_text: "60".to_string(),
            fps: 15,
            fps_text: "15".to_string(),
            zoom_from: 1.0,
            zoom_from_text: "1.0".to_string(),
            zoom_to: 100.0,
            zoom_to_text: "100.0".to_string(),
            julia_from_real: -0.7,
            julia_from_real_text: "-0.7".to_string(),
            julia_from_imag: 0.27015,
            julia_from_imag_text: "0.27015".to_string(),
            julia_to_real: -0.4,
            julia_to_real_text: "-0.4".to_string(),
            julia_to_imag: 0.6,
            julia_to_imag_text: "0.6".to_string(),
            iter_from: 100,
            iter_from_text: "100".to_string(),
            iter_to: 1000,
            iter_to_text: "1000".to_string(),
            generating: false,
            progress: 0.0,
            progress_message: String::new(),
        }
    }
}

impl AnimationState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Parse num_frames from text input
    pub fn parse_num_frames(&self) -> u32 {
        self.num_frames_text.parse().unwrap_or(self.num_frames)
    }
    
    /// Parse fps from text input
    pub fn parse_fps(&self) -> u8 {
        self.fps_text.parse().unwrap_or(self.fps)
    }
    
    /// Parse zoom_from from text input
    pub fn parse_zoom_from(&self) -> f64 {
        self.zoom_from_text.parse().unwrap_or(self.zoom_from)
    }
    
    /// Parse zoom_to from text input
    pub fn parse_zoom_to(&self) -> f64 {
        self.zoom_to_text.parse().unwrap_or(self.zoom_to)
    }
    
    /// Parse julia_from_real from text input
    pub fn parse_julia_from_real(&self) -> f64 {
        self.julia_from_real_text.parse().unwrap_or(self.julia_from_real)
    }
    
    /// Parse julia_from_imag from text input
    pub fn parse_julia_from_imag(&self) -> f64 {
        self.julia_from_imag_text.parse().unwrap_or(self.julia_from_imag)
    }
    
    /// Parse julia_to_real from text input
    pub fn parse_julia_to_real(&self) -> f64 {
        self.julia_to_real_text.parse().unwrap_or(self.julia_to_real)
    }
    
    /// Parse julia_to_imag from text input
    pub fn parse_julia_to_imag(&self) -> f64 {
        self.julia_to_imag_text.parse().unwrap_or(self.julia_to_imag)
    }
    
    /// Parse iter_from from text input
    pub fn parse_iter_from(&self) -> u32 {
        self.iter_from_text.parse().unwrap_or(self.iter_from)
    }
    
    /// Parse iter_to from text input
    pub fn parse_iter_to(&self) -> u32 {
        self.iter_to_text.parse().unwrap_or(self.iter_to)
    }
}

/// Conversion from FractalMetadata to state structs
/// These conversions enable clean bidirectional transformation between
/// serialized metadata and application state.

impl From<&crate::export::FractalMetadata> for ViewState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            view: meta.to_fractal_view(),
            fractal_texture: None,
            needs_redraw: true,
            iteration_cache: None,
            preview_zoom: 1.0,
        }
    }
}

impl From<&crate::export::FractalMetadata> for FractalState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            fractal_type: meta.parse_fractal_type().unwrap_or(FractalType::Mandelbrot),
            parameters: meta.fractal_parameters.clone(),
        }
    }
}

impl From<&crate::export::FractalMetadata> for ColorState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        use scala_chromatica::io as colorschemes_io;

        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        Self {
            available_colormaps,
            selected_colormap_name: meta.colormap_name.clone(),
            colormap: meta.colormap_data.clone(),
            color_editor: ColorEditor::new(),
            interior_picker: ColorPickerState::default(),
            stop_picker: ColorPickerState::default(),
            use_period: meta.use_period,
            use_interior_color: meta.use_interior_color,
            interior_color: meta.interior_color,
            interior_color_rgb_text: [
                meta.interior_color[0].to_string(),
                meta.interior_color[1].to_string(),
                meta.interior_color[2].to_string(),
            ],
            use_log_scale: meta.use_log_scale,
        }
    }
}

impl From<&crate::export::FractalMetadata> for InputState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        // Extract Julia parameters if present
        let julia_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(0.0);
        let julia_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.0);
        
        // Extract Mandelbrot power if present
        let mandelbrot_power = meta.fractal_parameters.get("power")
            .copied()
            .unwrap_or(2.0);
        
        // Extract Multifractal-Julia power if present (same key as Mandelbrot)
        let multifractal_julia_power = meta.fractal_parameters.get("power")
            .copied()
            .unwrap_or(1.0);
        
        // Extract Marek Dragon phi if present
        let marek_dragon_phi = meta.fractal_parameters.get("phi")
            .copied()
            .unwrap_or(0.0);
        
        // Extract Tetration threshold if present
        let tetration_threshold = meta.fractal_parameters.get("threshold")
            .copied()
            .unwrap_or(1e7);

        // Extract Lemon convergence exponent if present
        let lemon_convergence_exp = meta.fractal_parameters.get("convergence_exp")
            .copied()
            .unwrap_or(6.0);

        // Extract Lemon denominator power if present
        let lemon_denom_power = meta.fractal_parameters.get("denom_power")
            .copied()
            .unwrap_or(2.0);

        // Extract Insideout Dragon escape radius if present
        let insideout_dragon_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(4.0);

        // Extract Zubieta c values if present
        let zubieta_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(0.0);
        let zubieta_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.8);

        // Extract Sin Julia c values if present
        let sin_julia_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(1.0);
        let sin_julia_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.1);
        let sin_julia_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(50.0);

        Self {
            width: meta.width.to_string(),
            height: meta.height.to_string(),
            iterations: meta.max_iterations.to_string(),
            julia_c_real: julia_c_real.to_string(),
            julia_c_imag: julia_c_imag.to_string(),
            julia_magnitude: (julia_c_real * julia_c_real + julia_c_imag * julia_c_imag).sqrt().to_string(),
            julia_angle: julia_c_imag.atan2(julia_c_real).to_string(),
            julia_coord_mode: crate::app_state::CoordinateMode::default(),
            mandelbrot_power: mandelbrot_power.to_string(),
            multifractal_julia_power: multifractal_julia_power.to_string(),
            marek_dragon_phi: marek_dragon_phi.to_string(),
            tetration_threshold: format!("{:.2e}", tetration_threshold),
            lemon_convergence_exp: lemon_convergence_exp.to_string(),
            lemon_denom_power: lemon_denom_power.to_string(),
            insideout_dragon_escape_radius: insideout_dragon_escape_radius.to_string(),
            zubieta_c_real: zubieta_c_real.to_string(),
            zubieta_c_imag: zubieta_c_imag.to_string(),
            zubieta_magnitude: (zubieta_c_real * zubieta_c_real + zubieta_c_imag * zubieta_c_imag).sqrt().to_string(),
            zubieta_angle: zubieta_c_imag.atan2(zubieta_c_real).to_string(),
            zubieta_coord_mode: crate::app_state::CoordinateMode::default(),
            sin_julia_c_real: sin_julia_c_real.to_string(),
            sin_julia_c_imag: sin_julia_c_imag.to_string(),
            sin_julia_magnitude: (sin_julia_c_real * sin_julia_c_real + sin_julia_c_imag * sin_julia_c_imag).sqrt().to_string(),
            sin_julia_angle: sin_julia_c_imag.atan2(sin_julia_c_real).to_string(),
            sin_julia_coord_mode: crate::app_state::CoordinateMode::default(),
            sin_julia_escape_radius: sin_julia_escape_radius.to_string(),
            period: meta.period.to_string(),
            export_scale: meta.export_scale.to_string(),
            export_supersample: meta.export_supersample.to_string(),
            debounce_timer: None,
            pending_redraw: true,
        }
    }
}

impl From<&crate::export::FractalMetadata> for ExportState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            directory: None, // Export directory is not stored in metadata
            filter: meta.parse_filter_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::FractalMetadata;
    use scala_chromatica::ColorMap;
    use std::collections::HashMap;

    #[test]
    fn test_viewstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: -0.5,
            center_y: 0.0,
            zoom: 0.8,
            width: 1920,
            height: 1080,
            max_iterations: 512,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 128,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let view_state = ViewState::from(&metadata);
        assert_eq!(view_state.view.center_x, -0.5);
        assert_eq!(view_state.view.center_y, 0.0);
        assert_eq!(view_state.view.zoom, 0.8);
        assert_eq!(view_state.view.width, 1920);
        assert_eq!(view_state.view.height, 1080);
        assert!(view_state.needs_redraw);
    }

    #[test]
    fn test_fractalstate_from_metadata() {
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), -0.7);
        params.insert("c_imag".to_string(), 0.27);

        let metadata = FractalMetadata {
            fractal_type: "Julia".to_string(),
            fractal_parameters: params.clone(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 1280,
            height: 720,
            max_iterations: 256,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 128,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let fractal_state = FractalState::from(&metadata);
        assert_eq!(fractal_state.fractal_type, FractalType::Julia);
        assert_eq!(fractal_state.parameters.get("c_real"), Some(&-0.7));
        assert_eq!(fractal_state.parameters.get("c_imag"), Some(&0.27));
    }

    #[test]
    fn test_inputstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 3840,
            height: 2160,
            max_iterations: 1024,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: true,
            period: 256,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "Lanczos3".to_string(),
            export_supersample: 4,
            export_scale: 2.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let input_state = InputState::from(&metadata);
        assert_eq!(input_state.width, "3840");
        assert_eq!(input_state.height, "2160");
        assert_eq!(input_state.iterations, "1024");
        assert_eq!(input_state.period, "256");
        assert_eq!(input_state.export_scale, "2");
        assert_eq!(input_state.export_supersample, "4");
    }

    #[test]
    fn test_colorstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 1280,
            height: 720,
            max_iterations: 256,
            colormap_name: "Fire".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: true,
            period: 128,
            use_interior_color: true,
            interior_color: [255, 128, 64],
            use_log_scale: true,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let color_state = ColorState::from(&metadata);
        assert_eq!(color_state.selected_colormap_name, "Fire");
        assert!(color_state.use_period);
        assert!(color_state.use_interior_color);
        assert_eq!(color_state.interior_color, [255, 128, 64]);
        assert!(color_state.use_log_scale);
        assert_eq!(color_state.interior_color_rgb_text[0], "255");
        assert_eq!(color_state.interior_color_rgb_text[1], "128");
        assert_eq!(color_state.interior_color_rgb_text[2], "64");
    }

    #[test]
    fn test_metadata_roundtrip() {
        use crate::export::FractalMetadata;
        use crate::filtering::FilterType;

        // Create initial state
        let mut fractal_state = FractalState::default();
        fractal_state.fractal_type = FractalType::Julia;
        fractal_state.parameters.insert("c_real".to_string(), -0.7);
        fractal_state.parameters.insert("c_imag".to_string(), 0.27);

        let view_state = ViewState::new(1920, 1080);
        let mut color_state = ColorState::default();
        color_state.use_period = true;
        
        let mut input_state = InputState::default();
        input_state.iterations = "512".to_string();
        
        let mut export_state = ExportState::default();
        export_state.filter = FilterType::Lanczos3;

        // Convert to metadata
        let metadata = FractalMetadata::from_app_state(
            &fractal_state,
            &view_state,
            &color_state,
            &input_state,
            &export_state,
        );

        // Convert back to state
        let fractal_state2 = FractalState::from(&metadata);
        let view_state2 = ViewState::from(&metadata);
        let color_state2 = ColorState::from(&metadata);
        let input_state2 = InputState::from(&metadata);
        let export_state2 = ExportState::from(&metadata);

        // Verify round-trip
        assert_eq!(fractal_state2.fractal_type, FractalType::Julia);
        assert_eq!(view_state2.view.width, 1920);
        assert_eq!(view_state2.view.height, 1080);
        assert!(color_state2.use_period);
        assert_eq!(input_state2.iterations, "512");
        assert_eq!(export_state2.filter, FilterType::Lanczos3);
    }
}
