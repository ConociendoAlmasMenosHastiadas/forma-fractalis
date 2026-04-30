//! Adjusted Probability Julia Fractal (Orbit Accumulation, per-pixel z_0)
//!
//! For every pixel in the view, z_0 is the complex coordinate of that pixel.
//! Each pixel spawns its own short orbit:
//!
//!   z_{n+1} = s * sqrt(z_n - z_0)   where z_0 = pixel coordinate
//!
//! s = -1 with probability R ("threshold"), s = +1 otherwise.
//!
//! All orbits from all pixels contribute to a shared density histogram.
//! The density image shows where orbits from across the view land.
//!
//! At R=0.5 each pixel traces its own Julia set (z^2 + z_0). The image is
//! a superposition of every Julia set in the viewing window simultaneously.
//! At R=0 or R=1 only one branch of the sqrt is taken, producing sparser images.

use super::{Fractal, FractalView, Parameter};
use crate::orbit_accumulation::{complex_sqrt, OrbitTarget};
use num_complex::Complex64;
use std::collections::HashMap;

/// Adjusted Probability Julia fractal (orbit accumulation)
pub struct AdjProbJulia;

impl AdjProbJulia {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdjProbJulia {
    fn default() -> Self {
        Self::new()
    }
}

/// Xorshift64 fast PRNG. State must not be zero.
#[inline(always)]
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

impl Fractal for AdjProbJulia {
    fn iterate(
        &self,
        _c_real: f64,
        _c_imag: f64,
        _parameters: &HashMap<String, f64>,
        _max_iter: u32,
    ) -> u32 {
        // Orbit-accumulation fractal — per-pixel iterate() is not used.
        0
    }

    fn uses_orbit_accumulation(&self) -> bool {
        true
    }

    fn accumulate_orbits(
        &self,
        target: &dyn OrbitTarget,
        view: &FractalView,
        params: &HashMap<String, f64>,
    ) {
        let threshold = params.get("threshold").copied().unwrap_or(0.5).clamp(0.0, 1.0);
        let samples   = params.get("samples").copied().unwrap_or(20.0) as u64;
        let burn_in   = params.get("burn_in").copied().unwrap_or(10.0) as u64;

        let mut rng = params.get("seed").copied().unwrap_or(0.0) as u64;
        if rng == 0 {
            rng = 0xdeadbeefcafebabe;
        }

        // Coordinate transform: pixel → complex (inverse of complex_to_screen)
        let aspect = view.width as f64 / view.height as f64;
        let scale  = 3.5 / view.zoom;
        let range_x = scale * aspect;
        let range_y = scale;
        let x_min = view.center_x - range_x * 0.5;
        let y_min = view.center_y - range_y * 0.5;

        // Each pixel in the view gets its own z_0 = complex coordinate of that pixel.
        // It spawns a short orbit contributing to the shared density buffer.
        for py in 0..view.height {
            for px in 0..view.width {
                let z0_re = x_min + (px as f64 + 0.5) / view.width  as f64 * range_x;
                let z0_im = y_min + (py as f64 + 0.5) / view.height as f64 * range_y;
                let z0 = Complex64::new(z0_re, z0_im);

                let mut z = Complex64::new(0.0, 0.0);

                // Burn-in: discard transient
                for _ in 0..burn_in {
                    let w = z - z0;
                    let s_z = complex_sqrt(w);
                    let rand_val = xorshift64(&mut rng) as f64 / u64::MAX as f64;
                    z = if rand_val < threshold { -s_z } else { s_z };
                    if !z.re.is_finite() || !z.im.is_finite() {
                        z = Complex64::new(0.0, 0.0);
                    }
                }

                // Record orbit points in shared density buffer
                for _ in 0..samples {
                    let w = z - z0;
                    let s_z = complex_sqrt(w);
                    let rand_val = xorshift64(&mut rng) as f64 / u64::MAX as f64;
                    z = if rand_val < threshold { -s_z } else { s_z };
                    if !z.re.is_finite() || !z.im.is_finite() {
                        z = Complex64::new(0.0, 0.0);
                    }
                    target.increment(z, view);
                }
            }
        }
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 1.0;
        view.set_parameter("threshold",        0.5);
        view.set_parameter("samples",           20.0);
        view.set_parameter("burn_in",           10.0);
        view.set_parameter("seed",               0.0);
        view.set_parameter("use_log_density",    1.0);
        view
    }

    fn name(&self) -> &str {
        "Adj Prob Julia"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = s*sqrt(z_n - z_0),  z_0 = screen pixel"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("threshold",       "Sign Threshold (R)",   0.5,   0.0, 1.0,
                "s=-1 with probability R. R=0.5: full Julia sets. R=0 or R=1: one branch."),
            Parameter::new("samples",         "Samples/pixel",        20.0,  1.0, 500.0,
                "Orbit steps recorded per pixel per sub-orbit pass"),
            Parameter::new("burn_in",         "Burn-in",              10.0,  0.0, 200.0,
                "Steps discarded at orbit start before recording"),
            Parameter::new("seed",            "PRNG Seed",             0.0,  0.0, f64::MAX,
                "Random seed; each sub-orbit pass uses a derived seed"),
            Parameter::new("use_log_density", "Log Density",           1.0,  0.0, 1.0,
                "Log-scale density normalization (1=on, 0=off)"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_accumulation::LocalDensityTarget;

    fn default_params() -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("threshold".to_string(),  0.5);
        p.insert("samples".to_string(),    5.0);
        p.insert("burn_in".to_string(),    2.0);
        p.insert("seed".to_string(),       42.0);
        p
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_uses_orbit_accumulation() {
        let f = AdjProbJulia::new();
        assert!(f.uses_orbit_accumulation());
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_iterate_returns_zero() {
        let f = AdjProbJulia::new();
        let p = default_params();
        assert_eq!(f.iterate(0.0, 0.0, &p, 100), 0);
        assert_eq!(f.iterate(0.5, -0.3, &p, 100), 0);
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_accumulate_orbits_no_panic() {
        let f = AdjProbJulia::new();
        let p = default_params();
        let view = f.default_view(64, 64);
        let target = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&target, &view, &p);
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_accumulate_orbits_records_hits() {
        let f = AdjProbJulia::new();
        let p = default_params();
        let view = f.default_view(64, 64);
        let target = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&target, &view, &p);
        let buf = target.into_density_buffer();
        let normalised = buf.normalize(256, true);
        let non_zero = normalised.iter().filter(|&&v| v > 0).count();
        assert!(non_zero > 0, "orbit accumulation should record some hits");
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_threshold_zero_deterministic() {
        // threshold=0 → always +sqrt → fully deterministic output
        let f = AdjProbJulia::new();
        let mut p = default_params();
        p.insert("threshold".to_string(), 0.0);
        let view = f.default_view(64, 64);

        let t1 = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&t1, &view, &p);
        let t2 = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&t2, &view, &p);

        let b1 = t1.into_density_buffer();
        let b2 = t2.into_density_buffer();
        let n1 = b1.normalize(256, true);
        let n2 = b2.normalize(256, true);
        assert_eq!(n1, n2, "threshold=0 must give identical output across runs");
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_name_and_parameter_count() {
        let f = AdjProbJulia::new();
        assert_eq!(f.name(), "Adj Prob Julia");
        assert_eq!(f.parameters().len(), 5);
    }

    #[test]
    #[ignore = "AdjProbJulia mothballed in v0.2.5; see plans/v0.3.7.md"]
    fn test_default_view() {
        let f = AdjProbJulia::new();
        let view = f.default_view(800, 600);
        assert_eq!(view.width, 800);
        assert_eq!(view.height, 600);
        assert!(view.parameters.contains_key("threshold"));
        assert!(view.parameters.contains_key("samples"));
        assert!(view.parameters.contains_key("use_log_density"));
    }
}

