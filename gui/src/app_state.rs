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

fn default_max_threads() -> usize {
    rayon::current_num_threads().saturating_sub(2).max(1)
}

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

// ── Multi-Julia IFS GUI state ──────────────────────────────────────────────

/// Per-attractor state for the Multi-Julia IFS fractal GUI.
/// Stores both rectangular and polar text representations so they can be
/// displayed and edited without re-computing on every frame.
pub struct AttractorEntry {
    pub real: String,
    pub imag: String,
    pub magnitude: String,
    pub angle: String,
    pub coord_mode: CoordinateMode,
    /// Probability weight in [0, 1].  All weights in a MultiJuliaIFSState
    /// are kept summing to 1 via `set_prob_linked`.
    pub prob: f64,
}

impl AttractorEntry {
    pub fn new(real: f64, imag: f64, prob: f64) -> Self {
        let magnitude = (real * real + imag * imag).sqrt();
        let angle = {
            let a = imag.atan2(real);
            if a < 0.0 { a + std::f64::consts::TAU } else { a }
        };
        Self {
            real: format!("{:.6}", real),
            imag: format!("{:.6}", imag),
            magnitude: format!("{:.6}", magnitude),
            angle: format!("{:.6}", angle),
            coord_mode: CoordinateMode::Rectangular,
            prob,
        }
    }

    pub fn parse_real(&self) -> f64 {
        self.real.parse::<f64>().unwrap_or(0.0)
    }
    pub fn parse_imag(&self) -> f64 {
        self.imag.parse::<f64>().unwrap_or(0.0)
    }
    pub fn parse_magnitude(&self) -> f64 {
        self.magnitude.parse::<f64>().unwrap_or(0.0)
    }
    pub fn parse_angle(&self) -> f64 {
        self.angle.parse::<f64>().unwrap_or(0.0)
    }
}

/// GUI state for the Multi-Julia IFS fractal: a dynamic list of attractors
/// plus a PRNG seed.
pub struct MultiJuliaIFSState {
    pub attractors: Vec<AttractorEntry>,
    pub seed: String,
    pub samples: String,
    pub burn_in: String,
    pub use_log_density: bool,
}

impl Default for MultiJuliaIFSState {
    fn default() -> Self {
        Self {
            attractors: vec![
                AttractorEntry::new(-0.5,  0.5, 0.5),
                AttractorEntry::new(-0.5, -0.5, 0.5),
            ],
            seed: String::from("0"),
            samples: String::from("5000000"),
            burn_in: String::from("50"),
            use_log_density: true,
        }
    }
}

impl MultiJuliaIFSState {
    pub fn parse_seed(&self) -> f64 {
        self.seed.parse::<f64>().unwrap_or(0.0).abs()
    }

    pub fn parse_samples(&self) -> f64 {
        self.samples.parse::<f64>().unwrap_or(5_000_000.0).max(100_000.0)
    }

    pub fn parse_burn_in(&self) -> f64 {
        self.burn_in.parse::<f64>().unwrap_or(50.0).max(0.0)
    }

    /// Reconstruct state from a fractal_parameters HashMap (used for JSON restore).
    pub fn from_params(params: &HashMap<String, f64>) -> Self {
        let n = params.get("num_attractors").copied().unwrap_or(2.0) as usize;
        let n = n.max(2).min(8);

        let mut attractors = Vec::with_capacity(n);
        for i in 0..n {
            let real = params.get(&format!("c{}_real", i)).copied().unwrap_or(0.0);
            let imag = params.get(&format!("c{}_imag", i)).copied().unwrap_or(0.0);
            let prob = params.get(&format!("prob{}", i)).copied().unwrap_or(1.0 / n as f64);
            attractors.push(AttractorEntry::new(real, imag, prob));
        }

        let seed = params.get("seed").copied().unwrap_or(0.0);
        let samples = params.get("samples").copied().unwrap_or(5_000_000.0);
        let burn_in = params.get("burn_in").copied().unwrap_or(50.0);
        let use_log_density = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;

        Self {
            attractors,
            seed: format!("{}", seed as u64),
            samples: format!("{}", samples as u64),
            burn_in: format!("{}", burn_in as u64),
            use_log_density,
        }
    }

    /// Change the probability of attractor `idx` to `new_val`, rescaling
    /// all other attractors proportionally so that the sum stays at 1.
    pub fn set_prob_linked(&mut self, idx: usize, new_val: f64) {
        let new_val = new_val.clamp(0.0, 1.0);
        let n = self.attractors.len();
        if n <= 1 {
            if let Some(e) = self.attractors.first_mut() { e.prob = 1.0; }
            return;
        }
        let old_val = self.attractors[idx].prob;
        let remaining_old: f64 = 1.0 - old_val;
        let remaining_new: f64 = 1.0 - new_val;
        self.attractors[idx].prob = new_val;
        if remaining_old < 1e-12 {
            // All other probabilities were 0: distribute evenly.
            let per = remaining_new / (n - 1) as f64;
            for (i, e) in self.attractors.iter_mut().enumerate() {
                if i != idx { e.prob = per; }
            }
        } else {
            let scale = remaining_new / remaining_old;
            for (i, e) in self.attractors.iter_mut().enumerate() {
                if i != idx { e.prob = (e.prob * scale).clamp(0.0, 1.0); }
            }
        }
    }

    /// Write the current attractor list into the fractal parameters HashMap.
    pub fn apply_to_params(&self, params: &mut HashMap<String, f64>) {
        params.insert("num_attractors".to_string(), self.attractors.len() as f64);
        for (i, entry) in self.attractors.iter().enumerate() {
            let (real, imag) = match entry.coord_mode {
                CoordinateMode::Rectangular => (entry.parse_real(), entry.parse_imag()),
                CoordinateMode::Polar => {
                    let mag = entry.parse_magnitude();
                    let ang = entry.parse_angle();
                    (mag * ang.cos(), mag * ang.sin())
                }
            };
            params.insert(format!("c{}_real", i), real);
            params.insert(format!("c{}_imag", i), imag);
            params.insert(format!("prob{}", i), entry.prob);
        }
        params.insert("seed".to_string(), self.parse_seed());
        params.insert("samples".to_string(), self.parse_samples());
        params.insert("burn_in".to_string(), self.parse_burn_in());
        params.insert("use_log_density".to_string(), if self.use_log_density { 1.0 } else { 0.0 });
    }

    /// Add a new attractor at the origin, rescaling existing probabilities.
    pub fn add_attractor(&mut self) {
        if self.attractors.len() >= 8 {
            return;
        }
        let n = self.attractors.len();
        let scale = n as f64 / (n + 1) as f64;
        for e in &mut self.attractors { e.prob *= scale; }
        let new_prob = 1.0 / (n + 1) as f64;
        self.attractors.push(AttractorEntry::new(0.0, 0.0, new_prob));
    }

    /// Remove attractor at `idx`, redistributing its probability to the others.
    pub fn remove_attractor(&mut self, idx: usize) {
        if self.attractors.len() <= 2 {
            return; // Minimum 2 attractors enforced.
        }
        let removed_prob = self.attractors[idx].prob;
        self.attractors.remove(idx);
        let total: f64 = self.attractors.iter().map(|e| e.prob).sum();
        if total < 1e-12 {
            let n = self.attractors.len() as f64;
            for e in &mut self.attractors { e.prob = 1.0 / n; }
        } else {
            let scale = (total + removed_prob) / total;
            for e in &mut self.attractors { e.prob *= scale; }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// View and rendering state
#[derive(Clone)]
pub struct ViewState {
    pub view: FractalView,
    pub fractal_texture: Option<egui::TextureHandle>,
    pub needs_redraw: bool,
    /// Cached iteration or density values from the last full preview render.
    ///
    /// `Some` after the first successful preview render; `None` on startup or
    /// after a dimension change that invalidates the buffer. The cache stores
    /// backend-specific metadata, so color-only preview changes can reuse CPU,
    /// GPU, PT, and orbit-accumulation results safely.
    /// Call `FractalIterations::is_valid_for_render()` before reusing.
    pub iteration_cache: Option<FractalIterations>,
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

    /// Build a core `FractalConfig` from the current GUI state.
    ///
    /// This is the canonical bridge between GUI state and the core library API.
    /// Use the returned config to call `forma_fractalis_core::render_fractal_to_buffer()`
    /// or `forma_fractalis_core::export::export_png_with_config()`.
    pub fn to_fractal_config(
        &self,
        fractal_state: &FractalState,
        color_state: &ColorState,
        input_state: &InputState,
    ) -> crate::config::FractalConfig {
        crate::config::FractalConfig {
            view: self.view.clone(),
            max_iterations: input_state.parse_iterations(),
            fractal_parameters: fractal_state.parameters.clone(),
            color_config: crate::config::ColorConfig {
                colormap: color_state.colormap.clone(),
                use_period: color_state.use_period,
                period: input_state.parse_period(),
                use_interior_color: color_state.use_interior_color,
                interior_color: color_state.interior_color,
                use_log_scale: color_state.use_log_scale,
                color_offset: color_state.color_offset,
            },
        }
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
    /// Exponent k in z^k + c for the Julia set (default 2.0 = classic).
    pub julia_power: String,
    pub julia_escape_radius: String,
    pub mandelbrot_power: String,
    pub mandelbrot_escape_radius: String,
    pub multifractal_julia_power: String,
    pub marek_dragon_phi: String,
    pub marek_dragon_escape_radius: String,
    pub tetration_threshold: String,
    pub lemon_convergence_exp: String,
    pub lemon_denom_power: String,
    pub insideout_dragon_escape_radius: String,
    pub zubieta_c_real: String,
    pub zubieta_c_imag: String,
    pub zubieta_magnitude: String,
    pub zubieta_angle: String,
    pub zubieta_coord_mode: CoordinateMode,
    pub zubieta_escape_radius: String,
    pub burning_ship_escape_radius: String,
    pub tippets_escape_radius: String,
    pub sin_julia_c_real: String,
    pub sin_julia_c_imag: String,
    pub sin_julia_magnitude: String,
    pub sin_julia_angle: String,
    pub sin_julia_coord_mode: CoordinateMode,
    pub sin_julia_escape_radius: String,
    pub sinh_julia_c_real: String,
    pub sinh_julia_c_imag: String,
    pub sinh_julia_magnitude: String,
    pub sinh_julia_angle: String,
    pub sinh_julia_coord_mode: CoordinateMode,
    pub sinh_julia_escape_radius: String,
    pub multi_julia_ifs: MultiJuliaIFSState,
    pub adj_prob_julia_threshold: String,
    pub adj_prob_julia_samples: String,
    pub adj_prob_julia_burn_in: String,
    pub adj_prob_julia_seed: String,
    pub adj_prob_julia_use_log_density: bool,
    pub chaos_symmetry1_samples: String,
    pub chaos_symmetry1_burn_in: String,
    pub chaos_symmetry1_seed: String,
    pub chaos_symmetry1_use_log_density: bool,
    pub chaos_symmetry1_a0: String,
    pub chaos_symmetry1_a1: String,
    pub chaos_symmetry1_a2: String,
    pub chaos_symmetry1_a3: String,
    pub chaos_symmetry1_a4: String,
    pub wallpaper_a: String,
    pub wallpaper_b: String,
    pub wallpaper_c: String,
    pub wallpaper_samples: String,
    pub wallpaper_burn_in: String,
    pub wallpaper_use_log_density: bool,
    pub lace_julia_c_real: String,
    pub lace_julia_c_imag: String,
    pub lace_julia_magnitude: String,
    pub lace_julia_angle: String,
    pub lace_julia_coord_mode: CoordinateMode,
    pub lace_julia_escape_radius: String,
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
            julia_power: String::from("2.0"),
            julia_escape_radius: String::from("2.0"),
            mandelbrot_power: String::from("2.0"),
            mandelbrot_escape_radius: String::from("2.0"),
            multifractal_julia_power: String::from("1.0"),
            marek_dragon_phi: String::from("0.0"),
            marek_dragon_escape_radius: String::from("2.0"),
            tetration_threshold: String::from("1e7"),
            lemon_convergence_exp: String::from("6"),
            lemon_denom_power: String::from("2.0"),
            insideout_dragon_escape_radius: String::from("4.0"),
            zubieta_c_real: String::from("0.0"),
            zubieta_c_imag: String::from("0.8"),
            zubieta_magnitude: String::from("0.8"),
            zubieta_angle: String::from("1.5707963267949"),
            zubieta_coord_mode: CoordinateMode::default(),
            zubieta_escape_radius: String::from("2.0"),
            burning_ship_escape_radius: String::from("2.0"),
            tippets_escape_radius: String::from("2.0"),
            sin_julia_c_real: String::from("1.0"),
            sin_julia_c_imag: String::from("0.1"),
            sin_julia_magnitude: String::from("1.0049875621120890"),
            sin_julia_angle: String::from("0.09966865249116204"),
            sin_julia_coord_mode: CoordinateMode::default(),
            sin_julia_escape_radius: String::from("50.0"),
            sinh_julia_c_real: String::from("-0.7"),
            sinh_julia_c_imag: String::from("0.27015"),
            sinh_julia_magnitude: ((-0.7f64) * (-0.7f64) + 0.27015f64 * 0.27015f64).sqrt().to_string(),
            sinh_julia_angle: 0.27015f64.atan2(-0.7f64).to_string(),
            sinh_julia_coord_mode: CoordinateMode::default(),
            sinh_julia_escape_radius: String::from("50.0"),
            multi_julia_ifs: MultiJuliaIFSState::default(),
            adj_prob_julia_threshold: String::from("0.5"),
            adj_prob_julia_samples: String::from("20"),
            adj_prob_julia_burn_in: String::from("10"),
            adj_prob_julia_seed: String::from("0"),
            adj_prob_julia_use_log_density: true,
            chaos_symmetry1_samples: String::from("5000000"),
            chaos_symmetry1_burn_in: String::from("1000"),
            chaos_symmetry1_seed: String::from("0"),
            chaos_symmetry1_use_log_density: true,
            chaos_symmetry1_a0: String::from("1.5"),
            chaos_symmetry1_a1: String::from("-1.5"),
            chaos_symmetry1_a2: String::from("0"),
            chaos_symmetry1_a3: String::from("0"),
            chaos_symmetry1_a4: String::from("0.5"),
            wallpaper_a: crate::fractals::Wallpaper::DEFAULT_A.to_string(),
            wallpaper_b: crate::fractals::Wallpaper::DEFAULT_B.to_string(),
            wallpaper_c: crate::fractals::Wallpaper::DEFAULT_C.to_string(),
            wallpaper_samples: crate::fractals::Wallpaper::DEFAULT_SAMPLES.to_string(),
            wallpaper_burn_in: crate::fractals::Wallpaper::DEFAULT_BURN_IN.to_string(),
            wallpaper_use_log_density: true,
            lace_julia_c_real: String::from("0.0"),
            lace_julia_c_imag: String::from("0.5"),
            lace_julia_magnitude: String::from("0.5"),
            lace_julia_angle: String::from("1.5707963267948966"),
            lace_julia_coord_mode: CoordinateMode::default(),
            lace_julia_escape_radius: String::from("2.0"),
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

    /// Parse Julia exponent (power k in z^k + c; default 2.0 = classic Julia)
    pub fn parse_julia_power(&self) -> f64 {
        self.julia_power.parse::<f64>().unwrap_or(2.0)
    }

    /// Parse Mandelbrot power parameter
    pub fn parse_mandelbrot_power(&self) -> f64 {
        self.mandelbrot_power.parse::<f64>().unwrap_or(2.0)
    }

    /// Parse Mandelbrot escape radius parameter
    pub fn parse_mandelbrot_escape_radius(&self) -> f64 {
        self.mandelbrot_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Julia escape radius parameter
    pub fn parse_julia_escape_radius(&self) -> f64 {
        self.julia_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Burning Ship escape radius parameter
    pub fn parse_burning_ship_escape_radius(&self) -> f64 {
        self.burning_ship_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Tippets Mandelbrot escape radius parameter
    pub fn parse_tippets_escape_radius(&self) -> f64 {
        self.tippets_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Zubieta escape radius parameter
    pub fn parse_zubieta_escape_radius(&self) -> f64 {
        self.zubieta_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Marek Dragon escape radius parameter
    pub fn parse_marek_dragon_escape_radius(&self) -> f64 {
        self.marek_dragon_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
    }

    /// Parse Lace Julia c_real parameter
    pub fn parse_lace_julia_c_real(&self) -> f64 {
        self.lace_julia_c_real.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Lace Julia c_imag parameter
    pub fn parse_lace_julia_c_imag(&self) -> f64 {
        self.lace_julia_c_imag.parse::<f64>().unwrap_or(0.5)
    }

    /// Parse Lace Julia escape radius parameter
    pub fn parse_lace_julia_escape_radius(&self) -> f64 {
        self.lace_julia_escape_radius.parse::<f64>().unwrap_or(2.0).max(0.01)
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

    /// Parse Sinh Julia c_real parameter
    pub fn parse_sinh_julia_c_real(&self) -> f64 {
        self.sinh_julia_c_real.parse::<f64>().unwrap_or(-0.7)
    }

    /// Parse Sinh Julia c_imag parameter
    pub fn parse_sinh_julia_c_imag(&self) -> f64 {
        self.sinh_julia_c_imag.parse::<f64>().unwrap_or(0.27015)
    }

    /// Parse Sinh Julia escape radius parameter
    pub fn parse_sinh_julia_escape_radius(&self) -> f64 {
        self.sinh_julia_escape_radius.parse::<f64>().unwrap_or(50.0).max(0.01)
    }

    pub fn parse_adj_prob_julia_threshold(&self) -> f64 {
        self.adj_prob_julia_threshold.parse::<f64>().unwrap_or(0.5).clamp(0.0, 1.0)
    }

    pub fn parse_adj_prob_julia_samples(&self) -> f64 {
        self.adj_prob_julia_samples.parse::<f64>().unwrap_or(5_000_000.0).max(1.0)
    }

    pub fn parse_adj_prob_julia_burn_in(&self) -> f64 {
        self.adj_prob_julia_burn_in.parse::<f64>().unwrap_or(50.0).max(0.0)
    }

    pub fn parse_adj_prob_julia_seed(&self) -> f64 {
        self.adj_prob_julia_seed.parse::<f64>().unwrap_or(0.0)
    }

    pub fn parse_chaos_symmetry1_samples(&self) -> f64 {
        self.chaos_symmetry1_samples.parse::<f64>().unwrap_or(5_000_000.0).max(1.0)
    }

    pub fn parse_chaos_symmetry1_burn_in(&self) -> f64 {
        self.chaos_symmetry1_burn_in.parse::<f64>().unwrap_or(1_000.0).max(0.0)
    }

    pub fn parse_chaos_symmetry1_seed(&self) -> f64 {
        self.chaos_symmetry1_seed.parse::<f64>().unwrap_or(0.0)
    }

    pub fn parse_chaos_symmetry1_a0(&self) -> f64 {
        self.chaos_symmetry1_a0.parse::<f64>().unwrap_or(1.5).clamp(-3.0, 3.0)
    }
    pub fn parse_chaos_symmetry1_a1(&self) -> f64 {
        self.chaos_symmetry1_a1.parse::<f64>().unwrap_or(-1.5).clamp(-3.0, 3.0)
    }
    pub fn parse_chaos_symmetry1_a2(&self) -> f64 {
        self.chaos_symmetry1_a2.parse::<f64>().unwrap_or(0.0).clamp(-2.0, 2.0)
    }
    pub fn parse_chaos_symmetry1_a3(&self) -> f64 {
        self.chaos_symmetry1_a3.parse::<f64>().unwrap_or(0.0).clamp(-1.0, 1.0)
    }
    pub fn parse_chaos_symmetry1_a4(&self) -> f64 {
        self.chaos_symmetry1_a4.parse::<f64>().unwrap_or(0.5).clamp(-1.0, 1.0)
    }

    pub fn parse_wallpaper_a(&self) -> f64 {
        self.wallpaper_a
            .parse::<f64>()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_A)
            .clamp(0.0, 100.0)
    }

    pub fn parse_wallpaper_b(&self) -> f64 {
        self.wallpaper_b
            .parse::<f64>()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_B)
            .clamp(0.0, 100.0)
    }

    pub fn parse_wallpaper_c(&self) -> f64 {
        self.wallpaper_c
            .parse::<f64>()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_C)
            .clamp(0.0, 100.0)
    }

    pub fn parse_wallpaper_samples(&self) -> f64 {
        self.wallpaper_samples
            .parse::<f64>()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_SAMPLES as f64)
            .max(1.0)
    }

    pub fn parse_wallpaper_burn_in(&self) -> f64 {
        self.wallpaper_burn_in
            .parse::<f64>()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_BURN_IN as f64)
            .max(0.0)
    }
}

/// Re-export `FractalIterations` from core for use throughout the GUI.
///
/// The GUI uses this as its iteration cache. `IterationCache` (the old hand-rolled
/// struct) has been replaced by this core type. Cache validity is checked via
/// `FractalIterations::is_valid_for()` in `render_fractal()`.
pub use crate::rendering::FractalIterations;

/// Fractal type enumeration — re-exported from core so GUI and metadata use the same type.
pub use crate::fractals::FractalType;

// GUI trait implementation
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
        FractalType::is_julia(self)
    }

    fn is_mandelbrot(&self) -> bool {
        FractalType::is_mandelbrot(self)
    }

    fn uses_orbit_accumulation(&self) -> bool {
        matches!(self,
            FractalType::MultiJuliaIFS
            | FractalType::ChaosSymmetry1
            | FractalType::AdjProbJulia
            | FractalType::Wallpaper
        )
    }

    fn reset_view_and_params(
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
                let power = input.parse_mandelbrot_power();
                params.clear();
                params.insert("power".to_string(), power);
                params.insert("escape_radius".to_string(), input.parse_mandelbrot_escape_radius());
            }
            FractalType::Julia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.insert("c_real".to_string(), input.parse_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_julia_c_imag());
                params.insert("power".to_string(), input.parse_julia_power());
                params.insert("escape_radius".to_string(), input.parse_julia_escape_radius());
            }
            FractalType::BurningShip => {
                view.center_x = -0.5;
                view.center_y = -0.6;
                view.zoom = 0.8;
                params.clear();
                params.insert("escape_radius".to_string(), input.parse_burning_ship_escape_radius());
            }
            FractalType::TippetsMandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
                params.insert("escape_radius".to_string(), input.parse_tippets_escape_radius());
            }
            FractalType::MultifractalJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
                params.insert("power".to_string(), 1.0);
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
                params.insert("escape_radius".to_string(), input.parse_marek_dragon_escape_radius());
            }
            FractalType::Tetration => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.5;
                params.clear();
                params.insert("threshold".to_string(), input.parse_tetration_threshold());
                params.insert("escape_mode".to_string(), 0.0);
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
                params.insert("escape_radius".to_string(), input.parse_zubieta_escape_radius());
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
            FractalType::SinhJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.7;
                params.clear();
                params.insert("c_real".to_string(), input.parse_sinh_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_sinh_julia_c_imag());
                params.insert("escape_radius".to_string(), input.parse_sinh_julia_escape_radius());
            }
            FractalType::MultiJuliaIFS => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
                // Insert default 2-attractor configuration.
                // The GUI (render_parameters_gui) syncs its state from params
                // on the next frame when it detects the attractor count differs.
                params.insert("num_attractors".to_string(), 2.0);
                params.insert("c0_real".to_string(), -0.5);
                params.insert("c0_imag".to_string(),  0.5);
                params.insert("prob0".to_string(),    0.5);
                params.insert("c1_real".to_string(), -0.5);
                params.insert("c1_imag".to_string(), -0.5);
                params.insert("prob1".to_string(),    0.5);
                params.insert("seed".to_string(),     0.0);
                params.insert("samples".to_string(), 5_000_000.0);
                params.insert("burn_in".to_string(), 50.0);
                params.insert("use_log_density".to_string(), 1.0);
            }
            FractalType::AdjProbJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
                params.insert("threshold".to_string(), input.parse_adj_prob_julia_threshold());
                params.insert("samples".to_string(), input.parse_adj_prob_julia_samples());
                params.insert("burn_in".to_string(), input.parse_adj_prob_julia_burn_in());
                params.insert("seed".to_string(), input.parse_adj_prob_julia_seed());
                params.insert("use_log_density".to_string(), if input.adj_prob_julia_use_log_density { 1.0 } else { 0.0 });
            }
            FractalType::ChaosSymmetry1 => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
                params.insert("a0".to_string(), 1.5);
                params.insert("a1".to_string(), -1.5);
                params.insert("a2".to_string(), 0.0);
                params.insert("a3".to_string(), 0.0);
                params.insert("a4".to_string(), 0.5);
                params.insert("m".to_string(),  3.0);
                params.insert("samples".to_string(), input.parse_chaos_symmetry1_samples());
                params.insert("burn_in".to_string(), input.parse_chaos_symmetry1_burn_in());
                params.insert("seed".to_string(), input.parse_chaos_symmetry1_seed());
                params.insert("use_log_density".to_string(), if input.chaos_symmetry1_use_log_density { 1.0 } else { 0.0 });
            }
            FractalType::Wallpaper => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.75;
                params.clear();
                params.insert("a".to_string(), input.parse_wallpaper_a());
                params.insert("b".to_string(), input.parse_wallpaper_b());
                params.insert("c".to_string(), input.parse_wallpaper_c());
                params.insert("samples".to_string(), input.parse_wallpaper_samples());
                params.insert("burn_in".to_string(), input.parse_wallpaper_burn_in());
                params.insert("use_log_density".to_string(), if input.wallpaper_use_log_density { 1.0 } else { 0.0 });
            }
            FractalType::LaceJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.7;
                params.clear();
                params.insert("c_real".to_string(), input.parse_lace_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_lace_julia_c_imag());
                params.insert("escape_radius".to_string(), input.parse_lace_julia_escape_radius());
            }
        }
    }

    fn render_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        use crate::fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia, SinhJulia, MultiJuliaIFS, AdjProbJulia, ChaosSymmetry1, Wallpaper, LaceJulia};
        use crate::fractal_gui::FractalGUI;

        match self {
            FractalType::Mandelbrot => Mandelbrot::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Julia => Julia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::BurningShip => BurningShip::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::TippetsMandelbrot => TippetsMandelbrot::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::MultifractalJulia => MultifractalJulia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Cactus => Cactus::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::MarekDragon => MarekDragon::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Tetration => Tetration::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Lemon => Lemon::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::InsideoutDragon => InsideoutDragon::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Zubieta => Zubieta::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::SinJulia => SinJulia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::SinhJulia => SinhJulia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::MultiJuliaIFS => MultiJuliaIFS::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::AdjProbJulia => AdjProbJulia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::ChaosSymmetry1 => ChaosSymmetry1::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::Wallpaper => Wallpaper::new().render_parameters_gui(ui, params, input_state, needs_redraw),
            FractalType::LaceJulia => LaceJulia::new().render_parameters_gui(ui, params, input_state, needs_redraw),
        }
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
    /// Phase offset for color roll animation — see `ColorConfig::color_offset`.
    pub color_offset: u32,
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
            color_offset: 0,
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
    /// Path to the most recently exported image file, if any.
    pub last_export_path: Option<std::path::PathBuf>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            directory: None,
            filter: FilterType::None,
            last_export_path: None,
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
    /// Bit width used for the BigFloat reference orbit when backend == Perturbation.
    /// Also controls the hi-prec fallback precision for glitched pixels.
    /// Must be one of HIPREC_BIT_OPTIONS. Default: 128.
    pub pt_bits: u32,
    /// Glitch tolerance multiplier for PT rendering. Controls `|dz|² > tolerance * |z_total|²`.
    /// 1.0 = mathematically strict (default). Higher values (e.g. 4.0) reduce the number of
    /// expensive hi-prec fallback pixels at the cost of slight color inaccuracy.
    pub pt_glitch_tolerance: f64,
    /// Maximum rayon threads for CPU rendering.
    /// 0 = use all available threads (rayon default). 1..N = limited parallelism.
    pub max_threads: usize,
    /// Saved (width, height, export_scale_string, preview_zoom) from before CpuHiPrec was selected.
    /// Set when switching TO CpuHiPrec, cleared when switching away.
    /// Used to restore exact original dimensions and display zoom without floating-point drift.
    pub hiprec_preview_saved: Option<(u32, u32, String, f32)>,
    /// True while the GUI is showing a temporary low-resolution preview during
    /// debounced edits on expensive backends.
    pub progressive_preview_active: bool,
    /// Timestamp of the debounced edit batch that produced the current
    /// temporary preview. A new timestamp means the preview should refresh.
    pub progressive_preview_stamp: Option<Instant>,
    /// Cached reference orbits for Perturbation Theory rendering.
    /// Contains `pt_tiles * pt_tiles` orbits arranged in a square grid.
    /// Empty when PT backend is not in use or cache is invalid.
    pub reference_orbits: Vec<forma_fractalis_core::perturbation::ReferenceOrbit>,
    /// NxN tile grid size for multi-reference PT. 1 = single orbit (view center).
    /// 2 = 2x2=4 orbits, 4 = 4x4=16 orbits, etc.
    pub pt_tiles: u32,
    /// View dimensions at the time orbits were last computed.
    /// If width/height change (resize), tile centers shift and orbits must be recomputed.
    pub last_orbit_view_w: u32,
    pub last_orbit_view_h: u32,
    #[cfg(feature = "gpu")]
    pub gpu_renderer: Option<crate::gpu::WgpuRenderer>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            backend: crate::gpu::RenderBackend::default(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            pt_bits: crate::perturbation::PT_REFERENCE_BITS,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
            max_threads: default_max_threads(),
            hiprec_preview_saved: None,
            progressive_preview_active: false,
            progressive_preview_stamp: None,
            reference_orbits: Vec::new(),
            last_orbit_view_w: 0,
            last_orbit_view_h: 0,
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
            color_offset: meta.color_offset,
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

        // Julia exponent (same "power" key as Mandelbrot; defaults to 2.0 for saved files
        // that predate this field)
        let julia_power = meta.fractal_parameters.get("power")
            .copied()
            .unwrap_or(2.0);
        
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

        // Extract escape_radius for each fractal (same key, different defaults)
        let mandelbrot_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);
        let julia_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);
        let burning_ship_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);
        let tippets_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);
        let zubieta_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);
        let marek_dragon_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);

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

        // Extract Sinh Julia c values if present
        let sinh_julia_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(-0.7);
        let sinh_julia_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.27015);
        let sinh_julia_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(50.0);

        // Extract Adj Prob Julia parameters if present
        let adj_prob_julia_threshold = meta.fractal_parameters.get("threshold")
            .copied()
            .unwrap_or(0.5);
        let adj_prob_julia_samples = meta.fractal_parameters.get("samples")
            .copied()
            .unwrap_or(5_000_000.0);
        let adj_prob_julia_burn_in = meta.fractal_parameters.get("burn_in")
            .copied()
            .unwrap_or(50.0);
        let adj_prob_julia_seed = meta.fractal_parameters.get("seed")
            .copied()
            .unwrap_or(0.0);
        let adj_prob_julia_use_log_density = meta.fractal_parameters.get("use_log_density")
            .copied()
            .unwrap_or(1.0) > 0.5;

        let wallpaper_a = meta.fractal_parameters.get("a")
            .copied()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_A);
        let wallpaper_b = meta.fractal_parameters.get("b")
            .copied()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_B);
        let wallpaper_c = meta.fractal_parameters.get("c")
            .copied()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_C);
        let wallpaper_samples = meta.fractal_parameters.get("samples")
            .copied()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_SAMPLES as f64);
        let wallpaper_burn_in = meta.fractal_parameters.get("burn_in")
            .copied()
            .unwrap_or(crate::fractals::Wallpaper::DEFAULT_BURN_IN as f64);
        let wallpaper_use_log_density = meta.fractal_parameters.get("use_log_density")
            .copied()
            .unwrap_or(1.0) > 0.5;

        // Extract Lace Julia parameters if present
        let lace_julia_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(0.0);
        let lace_julia_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.5);
        let lace_julia_escape_radius = meta.fractal_parameters.get("escape_radius")
            .copied()
            .unwrap_or(2.0);

        Self {
            width: meta.width.to_string(),
            height: meta.height.to_string(),
            iterations: meta.max_iterations.to_string(),
            julia_c_real: julia_c_real.to_string(),
            julia_c_imag: julia_c_imag.to_string(),
            julia_magnitude: (julia_c_real * julia_c_real + julia_c_imag * julia_c_imag).sqrt().to_string(),
            julia_angle: julia_c_imag.atan2(julia_c_real).to_string(),
            julia_coord_mode: crate::app_state::CoordinateMode::default(),
            julia_power: julia_power.to_string(),
            julia_escape_radius: julia_escape_radius.to_string(),
            mandelbrot_power: mandelbrot_power.to_string(),
            mandelbrot_escape_radius: mandelbrot_escape_radius.to_string(),
            multifractal_julia_power: multifractal_julia_power.to_string(),
            marek_dragon_phi: marek_dragon_phi.to_string(),
            marek_dragon_escape_radius: marek_dragon_escape_radius.to_string(),
            tetration_threshold: format!("{:.2e}", tetration_threshold),
            lemon_convergence_exp: lemon_convergence_exp.to_string(),
            lemon_denom_power: lemon_denom_power.to_string(),
            insideout_dragon_escape_radius: insideout_dragon_escape_radius.to_string(),
            zubieta_c_real: zubieta_c_real.to_string(),
            zubieta_c_imag: zubieta_c_imag.to_string(),
            zubieta_magnitude: (zubieta_c_real * zubieta_c_real + zubieta_c_imag * zubieta_c_imag).sqrt().to_string(),
            zubieta_angle: zubieta_c_imag.atan2(zubieta_c_real).to_string(),
            zubieta_coord_mode: crate::app_state::CoordinateMode::default(),
            zubieta_escape_radius: zubieta_escape_radius.to_string(),
            burning_ship_escape_radius: burning_ship_escape_radius.to_string(),
            tippets_escape_radius: tippets_escape_radius.to_string(),
            sin_julia_c_real: sin_julia_c_real.to_string(),
            sin_julia_c_imag: sin_julia_c_imag.to_string(),
            sin_julia_magnitude: (sin_julia_c_real * sin_julia_c_real + sin_julia_c_imag * sin_julia_c_imag).sqrt().to_string(),
            sin_julia_angle: sin_julia_c_imag.atan2(sin_julia_c_real).to_string(),
            sin_julia_coord_mode: crate::app_state::CoordinateMode::default(),
            sin_julia_escape_radius: sin_julia_escape_radius.to_string(),
            sinh_julia_c_real: sinh_julia_c_real.to_string(),
            sinh_julia_c_imag: sinh_julia_c_imag.to_string(),
            sinh_julia_magnitude: (sinh_julia_c_real * sinh_julia_c_real + sinh_julia_c_imag * sinh_julia_c_imag).sqrt().to_string(),
            sinh_julia_angle: sinh_julia_c_imag.atan2(sinh_julia_c_real).to_string(),
            sinh_julia_coord_mode: crate::app_state::CoordinateMode::default(),
            sinh_julia_escape_radius: sinh_julia_escape_radius.to_string(),
            multi_julia_ifs: MultiJuliaIFSState::from_params(&meta.fractal_parameters),
            adj_prob_julia_threshold: adj_prob_julia_threshold.to_string(),
            adj_prob_julia_samples: format!("{:.0}", adj_prob_julia_samples),
            adj_prob_julia_burn_in: format!("{:.0}", adj_prob_julia_burn_in),
            adj_prob_julia_seed: format!("{:.0}", adj_prob_julia_seed),
            adj_prob_julia_use_log_density,
            chaos_symmetry1_samples: String::from("5000000"),
            chaos_symmetry1_burn_in: String::from("1000"),
            chaos_symmetry1_seed: String::from("0"),
            chaos_symmetry1_use_log_density: true,
            chaos_symmetry1_a0: String::from("1.5"),
            chaos_symmetry1_a1: String::from("-1.5"),
            chaos_symmetry1_a2: String::from("0"),
            chaos_symmetry1_a3: String::from("0"),
            chaos_symmetry1_a4: String::from("0.5"),
            wallpaper_a: wallpaper_a.to_string(),
            wallpaper_b: wallpaper_b.to_string(),
            wallpaper_c: wallpaper_c.to_string(),
            wallpaper_samples: format!("{:.0}", wallpaper_samples),
            wallpaper_burn_in: format!("{:.0}", wallpaper_burn_in),
            wallpaper_use_log_density,
            lace_julia_c_real: lace_julia_c_real.to_string(),
            lace_julia_c_imag: lace_julia_c_imag.to_string(),
            lace_julia_magnitude: (lace_julia_c_real * lace_julia_c_real + lace_julia_c_imag * lace_julia_c_imag).sqrt().to_string(),
            lace_julia_angle: {
                let a = lace_julia_c_imag.atan2(lace_julia_c_real);
                if a < 0.0 { (a + std::f64::consts::TAU).to_string() } else { a.to_string() }
            },
            lace_julia_coord_mode: crate::app_state::CoordinateMode::default(),
            lace_julia_escape_radius: lace_julia_escape_radius.to_string(),
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
            last_export_path: None,
        }
    }
}

impl From<&crate::export::FractalMetadata> for RenderState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        let mut state = Self::default();
        state.backend = meta.parse_render_backend();
        state.hiprec_bits = if meta.hiprec_bits == 0 {
            crate::gpu::HIPREC_DEFAULT_BITS
        } else {
            meta.hiprec_bits
        };
        state.pt_bits = if meta.pt_bits == 0 {
            crate::perturbation::PT_REFERENCE_BITS
        } else {
            meta.pt_bits
        };
        state.max_threads = if meta.max_threads == 0 {
            default_max_threads()
        } else {
            meta.max_threads
        };
        state.pt_glitch_tolerance = if meta.pt_glitch_tolerance > 0.0 {
            meta.pt_glitch_tolerance
        } else {
            1.0
        };
        state.pt_tiles = meta.pt_tiles.max(1);
        state
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
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
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
            color_offset: 0,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            render_backend: crate::gpu::RenderBackend::Cpu.as_str().to_string(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            pt_bits: crate::perturbation::PT_REFERENCE_BITS,
            max_threads: 0,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
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
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
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
            color_offset: 0,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            render_backend: crate::gpu::RenderBackend::Cpu.as_str().to_string(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            pt_bits: crate::perturbation::PT_REFERENCE_BITS,
            max_threads: 0,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
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
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
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
            color_offset: 0,
            export_filter: "Lanczos3".to_string(),
            export_supersample: 4,
            export_scale: 2.0,
            render_backend: crate::gpu::RenderBackend::Cpu.as_str().to_string(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            pt_bits: crate::perturbation::PT_REFERENCE_BITS,
            max_threads: 0,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
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
    fn test_renderstate_default_max_threads_uses_pool_headroom() {
        let expected = rayon::current_num_threads().saturating_sub(2).max(1);
        assert_eq!(RenderState::default().max_threads, expected);
    }

    #[test]
    fn test_colorstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
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
            color_offset: 7,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            render_backend: crate::gpu::RenderBackend::Cpu.as_str().to_string(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            pt_bits: crate::perturbation::PT_REFERENCE_BITS,
            max_threads: 0,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
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
        assert_eq!(color_state.color_offset, 7);
        assert_eq!(color_state.interior_color_rgb_text[0], "255");
        assert_eq!(color_state.interior_color_rgb_text[1], "128");
        assert_eq!(color_state.interior_color_rgb_text[2], "64");
    }

    #[test]
    fn test_metadata_roundtrip() {
        use crate::filtering::FilterType;

        // Create initial state
        let mut fractal_state = FractalState::default();
        fractal_state.fractal_type = FractalType::Julia;
        fractal_state.parameters.insert("c_real".to_string(), -0.7);
        fractal_state.parameters.insert("c_imag".to_string(), 0.27);

        let view_state = ViewState::new(1920, 1080);
        let mut color_state = ColorState::default();
        color_state.use_period = true;
        color_state.color_offset = 9;
        
        let mut input_state = InputState::default();
        input_state.iterations = "512".to_string();
        
        let mut export_state = ExportState::default();
        export_state.filter = FilterType::Lanczos3;

        let mut render_state = RenderState::default();
        render_state.backend = crate::gpu::RenderBackend::Perturbation;
        render_state.hiprec_bits = 136;
        render_state.pt_bits = 152;
        render_state.max_threads = 3;
        render_state.pt_glitch_tolerance = 2.0;
        render_state.pt_tiles = 3;

        // Convert to metadata
        let metadata = crate::export_helpers::metadata_from_app_state(
            &fractal_state,
            &view_state,
            &color_state,
            &input_state,
            &export_state,
            &render_state,
        );

        // Convert back to state
        let fractal_state2 = FractalState::from(&metadata);
        let view_state2 = ViewState::from(&metadata);
        let color_state2 = ColorState::from(&metadata);
        let input_state2 = InputState::from(&metadata);
        let export_state2 = ExportState::from(&metadata);
        let render_state2 = RenderState::from(&metadata);

        // Verify round-trip
        assert_eq!(fractal_state2.fractal_type, FractalType::Julia);
        assert_eq!(view_state2.view.width, 1920);
        assert_eq!(view_state2.view.height, 1080);
        assert!(color_state2.use_period);
        assert_eq!(color_state2.color_offset, 9);
        assert_eq!(input_state2.iterations, "512");
        assert_eq!(export_state2.filter, FilterType::Lanczos3);
        assert_eq!(render_state2.backend, crate::gpu::RenderBackend::Perturbation);
        assert_eq!(render_state2.hiprec_bits, 136);
        assert_eq!(render_state2.pt_bits, 152);
        assert_eq!(render_state2.max_threads, 3);
        assert_eq!(render_state2.pt_glitch_tolerance, 2.0);
        assert_eq!(render_state2.pt_tiles, 3);
    }

    #[test]
    fn test_multi_julia_ifs_from_params_roundtrip() {
        // Build a non-default MultiJuliaIFS configuration
        let mut state = MultiJuliaIFSState::default();
        state.attractors = vec![
            super::AttractorEntry::new(0.3, -0.4, 0.6),
            super::AttractorEntry::new(-0.8, 0.1, 0.15),
            super::AttractorEntry::new(0.0, 0.7, 0.25),
        ];
        state.seed = "42".to_string();
        state.samples = "10000000".to_string();
        state.burn_in = "200".to_string();
        state.use_log_density = false;

        // Write state to params
        let mut params = HashMap::new();
        state.apply_to_params(&mut params);

        // Reconstruct from params
        let restored = MultiJuliaIFSState::from_params(&params);

        // Verify round-trip
        assert_eq!(restored.attractors.len(), 3);
        assert!((restored.attractors[0].parse_real() - 0.3).abs() < 1e-5);
        assert!((restored.attractors[0].parse_imag() - (-0.4)).abs() < 1e-5);
        assert!((restored.attractors[0].prob - 0.6).abs() < 1e-5);
        assert!((restored.attractors[1].parse_real() - (-0.8)).abs() < 1e-5);
        assert!((restored.attractors[1].parse_imag() - 0.1).abs() < 1e-5);
        assert!((restored.attractors[1].prob - 0.15).abs() < 1e-5);
        assert!((restored.attractors[2].parse_real() - 0.0).abs() < 1e-5);
        assert!((restored.attractors[2].parse_imag() - 0.7).abs() < 1e-5);
        assert!((restored.attractors[2].prob - 0.25).abs() < 1e-5);
        assert_eq!(restored.parse_seed(), 42.0);
        assert_eq!(restored.parse_samples(), 10_000_000.0);
        assert_eq!(restored.parse_burn_in(), 200.0);
        assert!(!restored.use_log_density);
    }

    #[test]
    fn test_multi_julia_ifs_metadata_roundtrip() {
        // Create Multi-Julia IFS state with custom attractors
        let mut fractal_state = FractalState {
            fractal_type: FractalType::MultiJuliaIFS,
            parameters: HashMap::new(),
        };

        let mut input_state = InputState::default();
        input_state.multi_julia_ifs = MultiJuliaIFSState {
            attractors: vec![
                super::AttractorEntry::new(0.3, -0.4, 0.6),
                super::AttractorEntry::new(-0.8, 0.1, 0.4),
            ],
            seed: "99".to_string(),
            samples: "8000000".to_string(),
            burn_in: "100".to_string(),
            use_log_density: false,
        };
        // Write IFS state into fractal parameters
        input_state.multi_julia_ifs.apply_to_params(&mut fractal_state.parameters);

        let view_state = ViewState::new(1280, 720);
        let color_state = ColorState::default();
        let export_state = ExportState::default();
        let render_state = RenderState::default();

        // Convert to metadata
        let metadata = crate::export_helpers::metadata_from_app_state(
            &fractal_state,
            &view_state,
            &color_state,
            &input_state,
            &export_state,
            &render_state,
        );

        // Verify metadata saved the fractal type
        assert_eq!(metadata.fractal_type, "Multi-Julia IFS");
        assert_eq!(metadata.fractal_parameters.get("num_attractors"), Some(&2.0));
        assert!((metadata.fractal_parameters["c0_real"] - 0.3).abs() < 1e-5);
        assert!((metadata.fractal_parameters["c0_imag"] - (-0.4)).abs() < 1e-5);
        assert!((metadata.fractal_parameters["samples"] - 8_000_000.0).abs() < 1e-5);

        // Convert back and verify
        let ft = metadata.parse_fractal_type().expect("parse_fractal_type should succeed for Multi-Julia IFS");
        assert_eq!(ft, FractalType::MultiJuliaIFS);

        let fractal2 = FractalState::from(&metadata);
        assert_eq!(fractal2.fractal_type, FractalType::MultiJuliaIFS);
        assert_eq!(fractal2.parameters.get("num_attractors"), Some(&2.0));

        let input2 = InputState::from(&metadata);
        assert_eq!(input2.multi_julia_ifs.attractors.len(), 2);
        assert!((input2.multi_julia_ifs.attractors[0].parse_real() - 0.3).abs() < 1e-5);
        assert!((input2.multi_julia_ifs.attractors[0].parse_imag() - (-0.4)).abs() < 1e-5);
        assert!((input2.multi_julia_ifs.attractors[0].prob - 0.6).abs() < 1e-5);
        assert!((input2.multi_julia_ifs.attractors[1].parse_real() - (-0.8)).abs() < 1e-5);
        assert!((input2.multi_julia_ifs.attractors[1].parse_imag() - 0.1).abs() < 1e-5);
        assert!((input2.multi_julia_ifs.attractors[1].prob - 0.4).abs() < 1e-5);
        assert_eq!(input2.multi_julia_ifs.parse_seed(), 99.0);
        assert_eq!(input2.multi_julia_ifs.parse_samples(), 8_000_000.0);
        assert_eq!(input2.multi_julia_ifs.parse_burn_in(), 100.0);
        assert!(!input2.multi_julia_ifs.use_log_density);
    }

    #[test]
    fn test_multi_julia_ifs_from_params_empty_gives_defaults() {
        // When restoring a non-IFS fractal, params won't have IFS keys.
        // from_params should produce a sensible default (2 attractors at origin).
        let params = HashMap::new();
        let restored = MultiJuliaIFSState::from_params(&params);
        assert_eq!(restored.attractors.len(), 2);
        assert_eq!(restored.parse_samples(), 5_000_000.0);
        assert_eq!(restored.parse_burn_in(), 50.0);
        assert!(restored.use_log_density);
    }
}
