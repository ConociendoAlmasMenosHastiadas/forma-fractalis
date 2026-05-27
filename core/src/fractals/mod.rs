//! Fractal Trait System
//!
//! This module provides a unified trait-based system for different fractal types.
//! Each fractal implements the `Fractal` trait, which defines how to compute
//! iterations and provides metadata for GUI generation.
//!
//! # Architecture
//! - `Fractal` trait: Core interface for all fractal types
//! - `FractalView`: Viewport state with optional fractal-specific parameters
//! - `Parameter`: Metadata for dynamic GUI generation
//!
//! # Example
//! ```ignore
//! let mandelbrot = Mandelbrot::new();
//! let view = mandelbrot.default_view();
//! let iterations = mandelbrot.iterate(c.re, c.im, &view.parameters, 256);
//! ```

use std::collections::HashMap;
use astro_float::{BigFloat, Consts, Radix, RoundingMode};

fn bigfloat_to_f64(value: &BigFloat) -> f64 {
    format!("{}", value).parse::<f64>().unwrap_or(0.0)
}

fn parse_precise_decimal(value: Option<&str>, fallback: f64, bits: u32) -> BigFloat {
    let p = bits as usize;
    let rm = RoundingMode::ToEven;

    if let Some(value) = value {
        if let Ok(mut cc) = Consts::new() {
            let parsed = BigFloat::parse(value, Radix::Dec, p, rm, &mut cc);
            if !parsed.is_nan() {
                return parsed;
            }
        }
    }

    BigFloat::from_f64(fallback, p)
}

// Fractal implementations
pub mod mandelbrot;
pub mod julia;
pub mod burning_ship;
pub mod tippets_mandelbrot;
pub mod multifractal_julia;
pub mod cactus;
pub mod marek_dragon;
pub mod tetration;
pub mod lemon;
pub mod insideout_dragon;
pub mod zubieta;
pub mod sin_julia;
pub mod sinh_julia;
pub mod multi_julia_ifs;
pub mod adj_prob_julia;
pub mod chaos_symmetry1;
pub mod lace_julia;

// Parameter system extensions
pub mod parameter_types;

// Re-export for convenience
pub use mandelbrot::Mandelbrot;
pub use julia::Julia;
pub use burning_ship::BurningShip;
pub use tippets_mandelbrot::TippetsMandelbrot;
pub use multifractal_julia::MultifractalJulia;
pub use cactus::Cactus;
pub use marek_dragon::MarekDragon;
pub use tetration::Tetration;
pub use lemon::Lemon;
pub use insideout_dragon::InsideoutDragon;
pub use zubieta::Zubieta;
pub use sin_julia::SinJulia;
pub use sinh_julia::SinhJulia;
pub use multi_julia_ifs::MultiJuliaIFS;
pub use adj_prob_julia::AdjProbJulia;
pub use chaos_symmetry1::ChaosSymmetry1;
pub use lace_julia::LaceJulia;
pub use parameter_types::EscapeMode;

/// Represents the view parameters for rendering any fractal
/// This replaces the old MandelbrotView with a more generic structure
#[derive(Clone, Debug)]
pub struct FractalView {
    /// Center X coordinate in the complex plane
    pub center_x: f64,
    /// Center Y coordinate in the complex plane
    pub center_y: f64,
    /// Zoom level (higher = more zoomed in)
    pub zoom: f64,
    /// Optional higher-precision decimal string for the real center.
    ///
    /// This preserves deep-zoom camera state for Hi-Prec/PT interactions even
    /// when the f64 shadow field can no longer distinguish adjacent pixels.
    pub precise_center_x: Option<String>,
    /// Optional higher-precision decimal string for the imaginary center.
    pub precise_center_y: Option<String>,
    /// Optional higher-precision decimal string for the zoom.
    pub precise_zoom: Option<String>,
    /// Width of the viewport in pixels
    pub width: u32,
    /// Height of the viewport in pixels
    pub height: u32,
    /// Fractal-specific parameters (e.g., Julia set constants)
    pub parameters: HashMap<String, f64>,
}

impl FractalView {
    /// Creates a new FractalView with specified dimensions and default coordinates
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
            width,
            height,
            parameters: HashMap::new(),
        }
    }

    /// Returns true when the view carries higher-precision decimal camera state.
    pub fn has_precise_view(&self) -> bool {
        self.precise_center_x.is_some() && self.precise_center_y.is_some() && self.precise_zoom.is_some()
    }

    /// Refresh the precise decimal shadow values from the current f64 fields.
    pub fn sync_precise_from_f64(&mut self) {
        self.precise_center_x = Some(self.center_x.to_string());
        self.precise_center_y = Some(self.center_y.to_string());
        self.precise_zoom = Some(self.zoom.to_string());
    }

    /// Drop any higher-precision camera state and fall back to the f64 fields.
    pub fn clear_precise(&mut self) {
        self.precise_center_x = None;
        self.precise_center_y = None;
        self.precise_zoom = None;
    }

    /// Human-readable center X string using the most precise available source.
    pub fn center_x_display(&self) -> String {
        self.precise_center_x.clone().unwrap_or_else(|| self.center_x.to_string())
    }

    /// Human-readable center Y string using the most precise available source.
    pub fn center_y_display(&self) -> String {
        self.precise_center_y.clone().unwrap_or_else(|| self.center_y.to_string())
    }

    /// Human-readable zoom string using the most precise available source.
    pub fn zoom_display(&self) -> String {
        self.precise_zoom.clone().unwrap_or_else(|| self.zoom.to_string())
    }

    /// Parse the real center into a BigFloat at the requested precision.
    pub fn center_x_bigfloat(&self, bits: u32) -> BigFloat {
        parse_precise_decimal(self.precise_center_x.as_deref(), self.center_x, bits)
    }

    /// Parse the imaginary center into a BigFloat at the requested precision.
    pub fn center_y_bigfloat(&self, bits: u32) -> BigFloat {
        parse_precise_decimal(self.precise_center_y.as_deref(), self.center_y, bits)
    }

    /// Parse the zoom into a BigFloat at the requested precision.
    pub fn zoom_bigfloat(&self, bits: u32) -> BigFloat {
        parse_precise_decimal(self.precise_zoom.as_deref(), self.zoom, bits)
    }

    /// Update both the f64 shadow fields and the higher-precision camera state.
    pub fn set_precise_view(&mut self, center_x: &BigFloat, center_y: &BigFloat, zoom: &BigFloat) {
        self.center_x = bigfloat_to_f64(center_x);
        self.center_y = bigfloat_to_f64(center_y);
        self.zoom = bigfloat_to_f64(zoom);
        self.precise_center_x = Some(format!("{}", center_x));
        self.precise_center_y = Some(format!("{}", center_y));
        self.precise_zoom = Some(format!("{}", zoom));
    }

    /// Converts normalized screen coordinates [0, 1] to complex plane coordinates.
    pub fn screen_fraction_to_complex(&self, x_frac: f64, y_frac: f64) -> (f64, f64) {
        let aspect_ratio = self.width as f64 / self.height as f64;
        let scale = 3.5 / self.zoom;

        let real = self.center_x + (x_frac - 0.5) * scale * aspect_ratio;
        let imag = self.center_y + (y_frac - 0.5) * scale;

        (real, imag)
    }

    /// Converts normalized screen coordinates [0, 1] to complex plane coordinates
    /// using arbitrary-precision BigFloat arithmetic.
    pub fn screen_fraction_to_complex_hiprec(&self, x_frac: f64, y_frac: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let center_x = self.center_x_bigfloat(bits);
        let center_y = self.center_y_bigfloat(bits);
        let zoom = self.zoom_bigfloat(bits);
        let x_frac = BigFloat::from_f64(x_frac, p);
        let y_frac = BigFloat::from_f64(y_frac, p);
        let half = BigFloat::from_f64(0.5, p);
        let width = BigFloat::from_f64(self.width as f64, p);
        let height = BigFloat::from_f64(self.height as f64, p);

        let three_point_five = BigFloat::from_f64(3.5, p);
        let scale = three_point_five.div(&zoom, p, rm);
        let aspect_ratio = width.div(&height, p, rm);

        let real_offset = x_frac
            .sub(&half, p, rm)
            .mul(&scale, p, rm)
            .mul(&aspect_ratio, p, rm);
        let imag_offset = y_frac.sub(&half, p, rm).mul(&scale, p, rm);

        let real = center_x.add(&real_offset, p, rm);
        let imag = center_y.add(&imag_offset, p, rm);

        (real, imag)
    }

    /// Converts screen pixel coordinates to complex plane coordinates
    ///
    /// # Arguments
    /// * `x` - Pixel x-coordinate
    /// * `y` - Pixel y-coordinate
    ///
    /// # Returns
    /// A tuple (real, imaginary) representing the complex number
    pub fn screen_to_complex(&self, x: u32, y: u32) -> (f64, f64) {
        self.screen_fraction_to_complex(
            x as f64 / self.width as f64,
            y as f64 / self.height as f64,
        )
    }

    /// Converts screen pixel coordinates to complex plane coordinates using
    /// arbitrary-precision BigFloat arithmetic.
    ///
    /// At extreme zoom levels (>1e15), f64 cannot distinguish adjacent pixels.
    /// This method performs the full coordinate mapping at `bits` precision so
    /// that every pixel maps to a unique complex value.
    ///
    /// When present, `precise_center_x`, `precise_center_y`, and `precise_zoom`
    /// are used as the absolute camera source. Otherwise the f64 shadow fields
    /// are promoted to BigFloat.
    pub fn screen_to_complex_hiprec(&self, x: u32, y: u32, bits: u32) -> (BigFloat, BigFloat) {
        self.screen_fraction_to_complex_hiprec(
            x as f64 / self.width as f64,
            y as f64 / self.height as f64,
            bits,
        )
    }

    /// Pans the view in the given direction
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let pan_amount = 0.1 / self.zoom;
        self.center_x += dx * pan_amount;
        self.center_y += dy * pan_amount;
        self.clear_precise();
    }

    /// Zooms in at a specific point on the screen
    pub fn zoom_at(&mut self, screen_x: u32, screen_y: u32, zoom_factor: f64) {
        self.zoom_at_fraction(
            screen_x as f64 / self.width as f64,
            screen_y as f64 / self.height as f64,
            zoom_factor,
        );
    }

    /// Zooms in at a normalized screen position using the f64 camera state.
    pub fn zoom_at_fraction(&mut self, x_frac: f64, y_frac: f64, zoom_factor: f64) {
        let (click_real, click_imag) = self.screen_fraction_to_complex(x_frac, y_frac);
        self.center_x = click_real;
        self.center_y = click_imag;
        self.zoom *= zoom_factor;
        self.clear_precise();
    }

    /// Zooms in at a normalized screen position using BigFloat camera math.
    pub fn zoom_at_fraction_hiprec(&mut self, x_frac: f64, y_frac: f64, zoom_factor: f64, bits: u32) {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;
        let (click_real, click_imag) = self.screen_fraction_to_complex_hiprec(x_frac, y_frac, bits);
        let zoom = self.zoom_bigfloat(bits);
        let zoom_factor = BigFloat::from_f64(zoom_factor, p);
        let new_zoom = zoom.mul(&zoom_factor, p, rm);
        self.set_precise_view(&click_real, &click_imag, &new_zoom);
    }

    /// Zooms in at a specific pixel using BigFloat camera math.
    pub fn zoom_at_hiprec(&mut self, screen_x: u32, screen_y: u32, zoom_factor: f64, bits: u32) {
        self.zoom_at_fraction_hiprec(
            screen_x as f64 / self.width as f64,
            screen_y as f64 / self.height as f64,
            zoom_factor,
            bits,
        );
    }

    /// Sets a fractal-specific parameter
    pub fn set_parameter(&mut self, name: &str, value: f64) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Gets a fractal-specific parameter, or None if not set
    pub fn get_parameter(&self, name: &str) -> Option<f64> {
        self.parameters.get(name).copied()
    }

    /// Resets the view to default coordinates (centered at origin, zoom 1.0)
    /// Note: This does not reset fractal-specific parameters
    pub fn reset(&mut self) {
        self.center_x = 0.0;
        self.center_y = 0.0;
        self.zoom = 1.0;
        self.clear_precise();
        // Note: parameters are intentionally not cleared to preserve fractal settings
    }

    /// Builder method: Set center coordinates in the complex plane.
    pub fn with_center(mut self, x: f64, y: f64) -> Self {
        self.center_x = x;
        self.center_y = y;
        self
    }

    /// Builder method: Set the zoom level.
    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom;
        self
    }

    /// Builder method: Set a fractal-specific parameter.
    pub fn with_view_parameter(mut self, name: impl Into<String>, value: f64) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }
}

/// Metadata for a fractal parameter (used for GUI generation)
#[derive(Clone, Debug)]
pub struct Parameter {
    /// Internal parameter name (e.g., "c_real")
    pub name: String,
    /// Display label for GUI (e.g., "C Real Part")
    pub label: String,
    /// Default value
    pub default: f64,
    /// Minimum allowed value
    pub min: f64,
    /// Maximum allowed value
    pub max: f64,
    /// Description/tooltip text
    pub description: String,
}

impl Parameter {
    /// Creates a new parameter with the given properties
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        default: f64,
        min: f64,
        max: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            default,
            min,
            max,
            description: description.into(),
        }
    }
}

/// Core trait that all fractals must implement
pub trait Fractal: Sync {
    /// Compute the number of iterations before divergence
    ///
    /// # Arguments
    /// * `c_real` - Real part of the complex number
    /// * `c_imag` - Imaginary part of the complex number
    /// * `parameters` - Fractal-specific parameters
    /// * `max_iter` - Maximum number of iterations to test
    ///
    /// # Returns
    /// Number of iterations before divergence, or max_iter if in the set
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32;

    /// Get the default view settings for this fractal
    /// Each fractal type has its own ideal starting position and zoom
    fn default_view(&self, width: u32, height: u32) -> FractalView;

    /// Get the name of this fractal type
    fn name(&self) -> &str;

    /// Get the mathematical equation for this fractal
    /// Returns a string suitable for display in the GUI (supports Unicode subscripts/superscripts)
    fn equation(&self) -> &str {
        "" // Default: no equation displayed
    }

    /// Get the list of parameters this fractal uses
    /// Returns empty vec for fractals with no parameters (like Mandelbrot)
    fn parameters(&self) -> Vec<Parameter> {
        Vec::new()
    }

    /// Get the default values for all parameters
    fn parameter_defaults(&self) -> HashMap<String, f64> {
        self.parameters()
            .iter()
            .map(|p| (p.name.clone(), p.default))
            .collect()
    }

    /// Returns whether this fractal supports the CPU hi-precision rendering backend.
    ///
    /// Fractals that return `true` must also implement `iterate_hiprec`.
    /// Defaults to `false` — most fractals do not yet support hi-prec.
    fn supports_hiprec(&self) -> bool {
        false
    }

    /// Compute iterations using software arbitrary-precision arithmetic.
    ///
    /// Only called when `supports_hiprec()` returns `true` and the backend is `CpuHiPrec`.
    /// The `bits` argument is one of: 64, 128, 256, 512, 1024.
    ///
    /// Coordinates are received as `BigFloat` to preserve per-pixel precision at
    /// extreme zoom levels. The caller (`compute_iterations_hiprec` / `render_fractal_hiprec`)
    /// computes them via `FractalView::screen_to_complex_hiprec()`.
    ///
    /// The default implementation panics — fractals that advertise `supports_hiprec() == true`
    /// MUST override this method.
    fn iterate_hiprec(
        &self,
        _c_real: &BigFloat,
        _c_imag: &BigFloat,
        _parameters: &HashMap<String, f64>,
        _max_iter: u32,
        _bits: u32,
    ) -> u32 {
        panic!(
            "iterate_hiprec called on fractal '{}' but supports_hiprec() was not overridden",
            self.name()
        )
    }

    /// Returns whether this fractal uses orbit accumulation (density-based rendering)
    /// instead of per-pixel escape-time iteration.
    ///
    /// Fractals that return `true` must also implement `accumulate_orbits`.
    /// The rendering pipeline will call `compute_orbit_density()` instead of
    /// `compute_iterations()` for these fractals.
    fn uses_orbit_accumulation(&self) -> bool {
        false
    }

    /// Accumulate orbit visits into a density target.
    ///
    /// Only called when `uses_orbit_accumulation()` returns `true`.
    /// The fractal implements its own orbit-stepping logic (chaos game, Buddhabrot, etc.)
    /// and calls `target.increment(z, view)` for each orbit point that should be recorded.
    ///
    /// `target` is either a `LocalDensityTarget` (fold/reduce path, small buffers)
    /// or an `AtomicDensityBuffer` (shared path, large buffers).
    ///
    /// `params` contains fractal-specific parameters plus `seed` and `samples` which
    /// specify the PRNG seed and number of orbit steps for this sub-orbit.
    fn accumulate_orbits(
        &self,
        _target: &dyn crate::orbit_accumulation::OrbitTarget,
        _view: &FractalView,
        _params: &HashMap<String, f64>,
    ) {
        panic!(
            "accumulate_orbits called on fractal '{}' but uses_orbit_accumulation() was not overridden",
            self.name()
        )
    }

    /// Hi-precision orbit accumulation using BigFloat arithmetic.
    ///
    /// Only called when `uses_orbit_accumulation()` and `supports_hiprec()` both return `true`
    /// and the backend is `CpuHiPrec`. Uses `DensityBuffer::increment_at()` for pixel writes
    /// because the coordinate mapping is done in BigFloat externally.
    ///
    /// The default implementation panics — fractals that support hi-prec orbit accumulation
    /// MUST override this method.
    fn accumulate_orbits_hiprec(
        &self,
        _density: &mut crate::orbit_accumulation::DensityBuffer,
        _view: &FractalView,
        _params: &HashMap<String, f64>,
        _bits: u32,
    ) {
        panic!(
            "accumulate_orbits_hiprec called on fractal '{}' but not implemented",
            self.name()
        )
    }
}

/// Fractal type enumeration — defined in core so export/metadata can reference it without GUI deps.
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
    SinhJulia,
    MultiJuliaIFS,
    AdjProbJulia,
    ChaosSymmetry1,
    LaceJulia,
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
            FractalType::SinhJulia => "Sinh Julia",
            FractalType::MultiJuliaIFS => "Multi-Julia IFS",
            FractalType::AdjProbJulia => "Adj Prob Julia",
            FractalType::ChaosSymmetry1 => "ChaosSymmetry1",
            FractalType::LaceJulia => "Lace Julia",
        }
    }

    pub fn name(&self) -> &str {
        self.as_str()
    }

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
            FractalType::SinhJulia => "z_{n+1} = |Re(sinh(z_n)^4)| + i|Im(sinh(z_n)^4)| + c",
            FractalType::MultiJuliaIFS => "z_{n+1} = sqrt(z_n - c_i), i chosen by probability",
            FractalType::AdjProbJulia => "z_{n+1} = s*sqrt(|z_n-z_0|)*exp(i*arg(z_n)/2)",
            FractalType::ChaosSymmetry1 => "z_{n+1} = (a0+a1|z|^2+a2 Re(z^m)+a3 i)*z + a4*conj(z)^{m-1}",
            FractalType::LaceJulia => "z_{n+1} = (i*z_n^3 + 1010*z_n^6) / (c*i + 3301*z_n^7)",
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
            FractalType::SinhJulia,
            FractalType::InsideoutDragon,
            FractalType::MultiJuliaIFS,
            FractalType::ChaosSymmetry1,
            FractalType::LaceJulia,
            // AdjProbJulia mothballed in v0.2.5 — needs formula investigation; see plans/v0.3.7.md
        ]
    }

    /// Create a boxed fractal instance for this type.
    pub fn create_instance(&self) -> Box<dyn Fractal> {
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
            FractalType::SinhJulia => Box::new(SinhJulia::new()),
            FractalType::MultiJuliaIFS => Box::new(MultiJuliaIFS::new()),
            FractalType::AdjProbJulia => Box::new(AdjProbJulia::new()),
            FractalType::ChaosSymmetry1 => Box::new(ChaosSymmetry1::new()),
            FractalType::LaceJulia => Box::new(LaceJulia::new()),
        }
    }

    pub fn is_julia(&self) -> bool {
        matches!(self, FractalType::Julia)
    }

    pub fn is_mandelbrot(&self) -> bool {
        matches!(self, FractalType::Mandelbrot)
    }
}
