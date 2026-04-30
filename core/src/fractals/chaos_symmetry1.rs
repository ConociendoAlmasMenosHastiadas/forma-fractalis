//! ChaosSymmetry1 fractal (orbit accumulation, symmetry attractor)
//!
//! Renders a complex-plane attractor defined by the recurrence:
//!
//!   z_{n+1} = (a0 + a1*|z|^2 + a2*Re(z^m) + a3*i) * z_n + a4 * conj(z)^{m-1}
//!
//! where all a_k are real-valued, m is a positive integer controlling the
//! degree of rotational symmetry, and z is complex.
//!
//! Parameter guide:
//!   - m  >= 2 — degree of rotational symmetry of the attractor image
//!   - a0 — typically in [-3, 3]; values outside (-1, 1) are more interesting
//!   - a1 — opposite sign to a0, same magnitude range
//!   - a2 — perturbation of the attractor; 0 gives cleaner symmetry
//!   - a3 — bilateral symmetry; 0 = mirror-symmetric, ±1 = fully asymmetric
//!   - a4 — attractor scale; not too close to zero; typical range [-1, 1]
//!
//! Rendering is via orbit accumulation (histogram):
//!   1. Burn-in steps settle the orbit onto the attractor (discarded).
//!   2. Remaining steps are recorded into the density histogram.
//!
//! This maps naturally onto the two-pass procedure described in the original
//! reference: pass 1 (burn-in) settles the transient; pass 2 records density.
//! The view bounds determine which region of the attractor is captured.

use super::{Fractal, FractalView, Parameter};
use crate::orbit_accumulation::{complex_to_screen_hiprec, DensityBuffer, OrbitTarget};
use astro_float::BigFloat;
use num_complex::Complex64;
use std::collections::HashMap;

/// ChaosSymmetry1 attractor fractal
pub struct ChaosSymmetry1;

impl ChaosSymmetry1 {
    pub fn new() -> Self {
        Self
    }

    /// Compute one iteration of the recurrence.
    ///
    ///   z_{n+1} = (a0 + a1*|z|^2 + a2*Re(z^m) + a3*i) * z_n + a4 * conj(z)^{m-1}
    #[inline]
    fn step(z: Complex64, a0: f64, a1: f64, a2: f64, a3: f64, a4: f64, m: i32) -> Complex64 {
        let mod2  = z.norm_sqr();                    // |z|^2
        let zm    = z.powi(m);                       // z^m
        let re_zm = zm.re;                           // Re(z^m)
        let conj_z = z.conj();
        let conj_zm1 = conj_z.powi(m - 1);          // conj(z)^(m-1)

        let scale = a0 + a1 * mod2 + a2 * re_zm;    // real part of the scalar
        let factor = Complex64::new(scale, a3);      // (scale + a3*i)

        factor * z + a4 * conj_zm1
    }
}

impl Default for ChaosSymmetry1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for ChaosSymmetry1 {
    fn iterate(
        &self,
        _c_real: f64,
        _c_imag: f64,
        _parameters: &HashMap<String, f64>,
        _max_iter: u32,
    ) -> u32 {
        // Orbit-accumulation fractal — per-pixel iterate() is not used.
        // Return 0 (interior color) for any direct call.
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
        let a0 = params.get("a0").copied().unwrap_or(1.5);
        let a1 = params.get("a1").copied().unwrap_or(-1.5);
        let a2 = params.get("a2").copied().unwrap_or(0.0);
        let a3 = params.get("a3").copied().unwrap_or(0.0);
        let a4 = params.get("a4").copied().unwrap_or(0.5);
        let m  = params.get("m").copied().unwrap_or(3.0) as i32;
        let m  = m.max(2);

        let samples = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
        let burn_in = params.get("burn_in").copied().unwrap_or(1_000.0) as u64;

        // Use seed to pick a varied starting point for this sub-orbit.
        // A non-zero offset avoids all sub-orbits landing on the same fixed point.
        let seed = params.get("seed").copied().unwrap_or(0.0) as u64;
        let seed_offset = if seed == 0 { 1 } else { seed };

        // Start from a small, varied perturbation around the origin so that
        // different sub-orbits explore different branches of the attractor.
        let angle = (seed_offset as f64) * 2.399_963_2; // golden angle spread
        let radius = 0.1 + (seed_offset % 7) as f64 * 0.03;
        let mut z = Complex64::new(radius * angle.cos(), radius * angle.sin());

        // Guard against parameters that produce immediate divergence.
        const ESCAPE_SQ: f64 = 1e12; // |z| > 1e6 → reset

        // Pass 1 (burn-in): settle the orbit onto the attractor; discard all points.
        for _ in 0..burn_in {
            z = Self::step(z, a0, a1, a2, a3, a4, m);
            if !z.re.is_finite() || !z.im.is_finite() || z.norm_sqr() > ESCAPE_SQ {
                z = Complex64::new(0.0, 0.0);
            }
        }

        // Pass 2 (record): accumulate orbit density into the histogram.
        for _ in 0..samples {
            z = Self::step(z, a0, a1, a2, a3, a4, m);
            if !z.re.is_finite() || !z.im.is_finite() || z.norm_sqr() > ESCAPE_SQ {
                z = Complex64::new(0.0, 0.0);
            }
            target.increment(z, view);
        }
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    /// Hi-precision orbit accumulation.
    ///
    /// The ChaosSymmetry1 attractor lives in a bounded region (|z| < ~2 for typical
    /// parameters), so f64 is fully adequate for the orbit arithmetic itself.
    /// High precision is applied only to the complex → screen pixel mapping,
    /// which is what matters when zooming deep into a region of the attractor.
    fn accumulate_orbits_hiprec(
        &self,
        density: &mut DensityBuffer,
        view: &FractalView,
        params: &HashMap<String, f64>,
        bits: u32,
    ) {
        let a0 = params.get("a0").copied().unwrap_or(1.5);
        let a1 = params.get("a1").copied().unwrap_or(-1.5);
        let a2 = params.get("a2").copied().unwrap_or(0.0);
        let a3 = params.get("a3").copied().unwrap_or(0.0);
        let a4 = params.get("a4").copied().unwrap_or(0.5);
        let m  = params.get("m").copied().unwrap_or(3.0) as i32;
        let m  = m.max(2);

        let samples  = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
        let burn_in  = params.get("burn_in").copied().unwrap_or(1_000.0) as u64;
        let seed     = params.get("seed").copied().unwrap_or(0.0) as u64;
        let seed_offset = if seed == 0 { 1 } else { seed };

        let p = bits as usize;

        let angle  = (seed_offset as f64) * 2.399_963_2;
        let radius = 0.1 + (seed_offset % 7) as f64 * 0.03;
        let mut z  = Complex64::new(radius * angle.cos(), radius * angle.sin());

        const ESCAPE_SQ: f64 = 1e12;

        for step in 0..(samples + burn_in) {
            z = Self::step(z, a0, a1, a2, a3, a4, m);
            if !z.re.is_finite() || !z.im.is_finite() || z.norm_sqr() > ESCAPE_SQ {
                z = Complex64::new(0.0, 0.0);
            }

            if step >= burn_in {
                // High-precision screen mapping: converts f64 attractor coordinates
                // to BigFloat before mapping to pixels, gaining precision at high zoom.
                let z_re_bf = BigFloat::from_f64(z.re, p);
                let z_im_bf = BigFloat::from_f64(z.im, p);
                if let Some((px, py)) = complex_to_screen_hiprec(&z_re_bf, &z_im_bf, view, bits) {
                    density.increment_at(px, py);
                }
            }
        }
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        // zoom=1 maps to scale=3.5 → covers about ±1.75 in y (wider in x for 16:9).
        // Most ChaosSymmetry1 attractors live inside |z| < 1.5 for typical parameters.
        view.zoom = 1.0;
        view.set_parameter("a0",             1.5);
        view.set_parameter("a1",            -1.5);
        view.set_parameter("a2",             0.0);
        view.set_parameter("a3",             0.0);
        view.set_parameter("a4",             0.5);
        view.set_parameter("m",              3.0);
        view.set_parameter("samples",  5_000_000.0);
        view.set_parameter("burn_in",     1_000.0);
        view.set_parameter("seed",            0.0);
        view.set_parameter("use_log_density", 1.0);
        view
    }

    fn name(&self) -> &str {
        "ChaosSymmetry1"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = (a0 + a1*|z|^2 + a2*Re(z^m) + a3*i)*z_n + a4*conj(z)^{m-1}"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("m",   "Symmetry degree (m)", 3.0,  2.0, 8.0,
                "Integer degree of rotational symmetry (>= 2)"),
            Parameter::new("a0",  "a0",  1.5, -3.0, 3.0,
                "Base scalar term. Interesting range: outside (-1, 1)."),
            Parameter::new("a1",  "a1", -1.5, -3.0, 3.0,
                "Modulus-squared weight. Opposite sign to a0 gives attractors."),
            Parameter::new("a2",  "a2",  0.0, -2.0, 2.0,
                "Perturbation term: weight of Re(z^m). 0 = no perturbation."),
            Parameter::new("a3",  "a3",  0.0, -1.0, 1.0,
                "Bilateral symmetry. 0 = mirror-symmetric. Non-zero breaks bilateral symmetry."),
            Parameter::new("a4",  "a4",  0.5, -1.0, 1.0,
                "Conjugate term scale. Avoid values near 0."),
            Parameter::new("samples",        "Samples/sub-orbit", 5_000_000.0, 100_000.0, 50_000_000.0,
                "Orbit steps recorded per sub-orbit pass. More = smoother image."),
            Parameter::new("burn_in",        "Burn-in",           1_000.0, 0.0, 10_000.0,
                "Initial steps discarded to settle onto the attractor before recording."),
            Parameter::new("seed",           "PRNG Seed",         0.0, 0.0, f64::MAX,
                "Starting-point seed; each sub-orbit uses a derived seed."),
            Parameter::new("use_log_density","Log density",       1.0, 0.0, 1.0,
                "Log-normalise the density histogram (1=on, 0=off). Recommended for this fractal."),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_accumulation::LocalDensityTarget;

    fn default_params() -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("a0".to_string(),       1.5);
        p.insert("a1".to_string(),      -1.5);
        p.insert("a2".to_string(),       0.0);
        p.insert("a3".to_string(),       0.0);
        p.insert("a4".to_string(),       0.5);
        p.insert("m".to_string(),        3.0);
        p.insert("samples".to_string(),  500.0);
        p.insert("burn_in".to_string(),  50.0);
        p.insert("seed".to_string(),     42.0);
        p
    }

    #[test]
    fn test_uses_orbit_accumulation() {
        let f = ChaosSymmetry1::new();
        assert!(f.uses_orbit_accumulation());
    }

    #[test]
    fn test_iterate_returns_zero() {
        let f = ChaosSymmetry1::new();
        let p = default_params();
        assert_eq!(f.iterate(0.0,  0.0, &p, 100), 0);
        assert_eq!(f.iterate(1.0, -0.5, &p, 100), 0);
    }

    #[test]
    fn test_accumulate_orbits_no_panic() {
        let f = ChaosSymmetry1::new();
        let p = default_params();
        let view = f.default_view(64, 64);
        let target = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&target, &view, &p); // must not panic
    }

    #[test]
    fn test_accumulate_orbits_records_hits() {
        let f = ChaosSymmetry1::new();
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
    fn test_divergent_params_no_panic() {
        // Extreme parameters that cause rapid divergence should be handled gracefully.
        let f = ChaosSymmetry1::new();
        let mut p = default_params();
        p.insert("a0".to_string(), 100.0);
        p.insert("a1".to_string(), 100.0);
        p.insert("a4".to_string(), 100.0);
        let view = f.default_view(32, 32);
        let target = LocalDensityTarget::new(32, 32);
        f.accumulate_orbits(&target, &view, &p); // must not panic; orbit resets on divergence
    }

    #[test]
    fn test_name_and_equation() {
        let f = ChaosSymmetry1::new();
        assert_eq!(f.name(), "ChaosSymmetry1");
        assert!(!f.equation().is_empty());
    }

    #[test]
    fn test_default_view() {
        let f = ChaosSymmetry1::new();
        let view = f.default_view(800, 600);
        assert_eq!(view.width, 800);
        assert_eq!(view.height, 600);
        for key in &["a0", "a1", "a2", "a3", "a4", "m", "samples", "burn_in", "seed", "use_log_density"] {
            assert!(view.parameters.contains_key(*key), "missing default parameter: {}", key);
        }
    }

    #[test]
    fn test_parameter_count() {
        let f = ChaosSymmetry1::new();
        assert_eq!(f.parameters().len(), 10);
    }

    #[test]
    fn test_different_seeds_produce_output() {
        // Two different seeds should both produce non-empty histograms.
        let f = ChaosSymmetry1::new();
        let mut p = default_params();
        p.insert("samples".to_string(), 1_000.0);

        let view = f.default_view(64, 64);

        p.insert("seed".to_string(), 1.0);
        let t1 = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&t1, &view, &p);
        let n1 = t1.into_density_buffer().normalize(256, true)
            .iter().filter(|&&v| v > 0).count();

        p.insert("seed".to_string(), 7.0);
        let t2 = LocalDensityTarget::new(64, 64);
        f.accumulate_orbits(&t2, &view, &p);
        let n2 = t2.into_density_buffer().normalize(256, true)
            .iter().filter(|&&v| v > 0).count();

        assert!(n1 > 0, "seed=1 produced no hits");
        assert!(n2 > 0, "seed=7 produced no hits");
    }
}
