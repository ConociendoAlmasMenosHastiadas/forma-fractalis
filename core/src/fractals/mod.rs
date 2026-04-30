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
use astro_float::{BigFloat, RoundingMode};

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
            width,
            height,
            parameters: HashMap::new(),
        }
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
        let aspect_ratio = self.width as f64 / self.height as f64;
        let scale = 3.5 / self.zoom;

        let real = self.center_x
            + (x as f64 - self.width as f64 / 2.0) * scale / self.width as f64 * aspect_ratio;
        let imag =
            self.center_y + (y as f64 - self.height as f64 / 2.0) * scale / self.height as f64;

        (real, imag)
    }

    /// Converts screen pixel coordinates to complex plane coordinates using
    /// arbitrary-precision BigFloat arithmetic.
    ///
    /// At extreme zoom levels (>1e15), f64 cannot distinguish adjacent pixels.
    /// This method performs the full coordinate mapping at `bits` precision so
    /// that every pixel maps to a unique complex value.
    ///
    /// NOTE: `center_x`, `center_y`, `zoom` are stored as f64 and converted here.
    /// The per-pixel delta arithmetic is the critical part that must be hi-prec.
    pub fn screen_to_complex_hiprec(&self, x: u32, y: u32, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let center_x = BigFloat::from_f64(self.center_x, p);
        let center_y = BigFloat::from_f64(self.center_y, p);
        let zoom = BigFloat::from_f64(self.zoom, p);
        let width = BigFloat::from_f64(self.width as f64, p);
        let height = BigFloat::from_f64(self.height as f64, p);
        let two = BigFloat::from_f64(2.0, p);

        // scale = 3.5 / zoom
        let three_point_five = BigFloat::from_f64(3.5, p);
        let scale = three_point_five.div(&zoom, p, rm);

        // aspect_ratio = width / height
        let aspect_ratio = width.div(&height, p, rm);

        // real = center_x + (x - width/2) * scale / width * aspect_ratio
        let x_bf = BigFloat::from_f64(x as f64, p);
        let half_w = width.div(&two, p, rm);
        let offset_x = x_bf.sub(&half_w, p, rm);
        let real = center_x.add(
            &offset_x.mul(&scale, p, rm).div(&width, p, rm).mul(&aspect_ratio, p, rm),
            p, rm,
        );

        // imag = center_y + (y - height/2) * scale / height
        let y_bf = BigFloat::from_f64(y as f64, p);
        let half_h = height.div(&two, p, rm);
        let offset_y = y_bf.sub(&half_h, p, rm);
        let imag = center_y.add(
            &offset_y.mul(&scale, p, rm).div(&height, p, rm),
            p, rm,
        );

        (real, imag)
    }

    /// Pans the view in the given direction
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let pan_amount = 0.1 / self.zoom;
        self.center_x += dx * pan_amount;
        self.center_y += dy * pan_amount;
    }

    /// Zooms in at a specific point on the screen
    pub fn zoom_at(&mut self, screen_x: u32, screen_y: u32, zoom_factor: f64) {
        let (click_real, click_imag) = self.screen_to_complex(screen_x, screen_y);
        self.center_x = click_real;
        self.center_y = click_imag;
        self.zoom *= zoom_factor;
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
