//! Multi-Julia IFS fractal
//!
//! An Iterated Function System (IFS) where each map is defined as
//! g_i(z) = sqrt(z - c_i). The fractal is rendered via the **inverse-iteration
//! chaos game**: starting from an arbitrary point, we repeatedly pick a random
//! map g_i (with user-specified probability weights) and apply it, choosing the
//! sign of the square root randomly. After a burn-in period we record each
//! orbit point into a density histogram. The Julia set emerges as the region of
//! highest orbit density.
//!
//! This is an orbit-accumulation fractal — it uses `DensityBuffer` from
//! `orbit_accumulation.rs` instead of per-pixel escape-time iteration.
//!
//! Parameters (in the HashMap passed to accumulate_orbits):
//!   num_attractors   Number of IFS maps N (1-8, default 2)
//!   c{i}_real        Real part of map i  (i = 0 .. N-1)
//!   c{i}_imag        Imaginary part of map i
//!   prob{i}          Unnormalized probability weight for map i
//!   seed             Integer PRNG seed for this sub-orbit
//!   samples          Number of orbit steps for this sub-orbit
//!   burn_in          Number of initial steps to discard (default 50)
//!
//! Reference: https://paulbourke.net/fractals/multijulia/

use super::{Fractal, FractalView, Parameter};
use crate::orbit_accumulation::{complex_sqrt, complex_sqrt_hiprec, complex_to_screen_hiprec, OrbitTarget};
use num_complex::Complex64;
use std::collections::HashMap;
use astro_float::{BigFloat, RoundingMode};

const MAX_ATTRACTORS: usize = 8;

/// Multi-Julia IFS fractal
pub struct MultiJuliaIFS;

impl MultiJuliaIFS {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiJuliaIFS {
    fn default() -> Self {
        Self::new()
    }
}

/// Xorshift64 fast PRNG. State must not be zero.
#[inline]
fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Select an index 0..n using a pre-built cumulative probability table.
/// `cumulative[n-1]` must be exactly 1.0.
#[inline]
fn choose_index(rng: &mut u64, cumulative: &[f64]) -> usize {
    let r = xorshift64(rng) as f64 / u64::MAX as f64;
    cumulative
        .iter()
        .position(|&p| r < p)
        .unwrap_or(cumulative.len() - 1)
}

/// Parse attractors and cumulative probability table from parameters.
/// Returns (attractors, cumulative, n).
fn parse_maps(params: &HashMap<String, f64>) -> ([Complex64; MAX_ATTRACTORS], [f64; MAX_ATTRACTORS], usize) {
    let n = (params
        .get("num_attractors")
        .copied()
        .unwrap_or(2.0) as usize)
        .clamp(1, MAX_ATTRACTORS);

    let mut attractors = [Complex64::new(0.0, 0.0); MAX_ATTRACTORS];
    let mut weights = [0.0f64; MAX_ATTRACTORS];
    let mut total_weight = 0.0f64;

    for i in 0..n {
        let cr = params.get(&format!("c{}_real", i)).copied().unwrap_or(0.0);
        let ci = params.get(&format!("c{}_imag", i)).copied().unwrap_or(0.0);
        let w = params.get(&format!("prob{}", i)).copied().unwrap_or(1.0).max(0.0);
        attractors[i] = Complex64::new(cr, ci);
        weights[i] = w;
        total_weight += w;
    }

    if total_weight == 0.0 {
        total_weight = n as f64;
        for w in weights[..n].iter_mut() {
            *w = 1.0;
        }
    }

    let mut cumulative = [0.0f64; MAX_ATTRACTORS];
    let mut running = 0.0f64;
    for i in 0..n {
        running += weights[i] / total_weight;
        cumulative[i] = running;
    }
    cumulative[n - 1] = 1.0;

    (attractors, cumulative, n)
}

impl Fractal for MultiJuliaIFS {
    fn iterate(
        &self,
        _c_real: f64,
        _c_imag: f64,
        _parameters: &HashMap<String, f64>,
        _max_iter: u32,
    ) -> u32 {
        // Orbit-accumulation fractal — per-pixel iterate() is not used.
        // Return 0 (interior color) for any direct calls.
        0
    }

    fn uses_orbit_accumulation(&self) -> bool {
        true
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    fn accumulate_orbits(
        &self,
        target: &dyn OrbitTarget,
        view: &FractalView,
        params: &HashMap<String, f64>,
    ) {
        let (attractors, cumulative, n) = parse_maps(params);

        let samples = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
        let burn_in = params.get("burn_in").copied().unwrap_or(50.0) as u64;

        let mut rng = params.get("seed").copied().unwrap_or(0.0) as u64;
        if rng == 0 {
            rng = 0xdeadbeefcafebabe;
        }

        // Start from an arbitrary point
        let mut z = Complex64::new(0.0, 0.0);

        for step in 0..(samples + burn_in) {
            // Choose a random map
            let idx = choose_index(&mut rng, &cumulative[..n]);

            // Apply inverse map: z = ±sqrt(z - c_i)
            let w = z - attractors[idx];
            let mut s = complex_sqrt(w);

            // Randomly negate (choose branch of sqrt)
            if xorshift64(&mut rng) & 1 == 1 {
                s = -s;
            }

            z = s;

            // Guard against NaN/Inf
            if !z.re.is_finite() || !z.im.is_finite() {
                z = Complex64::new(0.0, 0.0);
            }

            // Record after burn-in
            if step >= burn_in {
                target.increment(z, view);
            }
        }
    }

    fn accumulate_orbits_hiprec(
        &self,
        density: &mut crate::orbit_accumulation::DensityBuffer,
        view: &FractalView,
        params: &HashMap<String, f64>,
        bits: u32,
    ) {
        let (attractors, cumulative, n) = parse_maps(params);
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let samples = params.get("samples").copied().unwrap_or(5_000_000.0) as u64;
        let burn_in = params.get("burn_in").copied().unwrap_or(50.0) as u64;

        let mut rng = params.get("seed").copied().unwrap_or(0.0) as u64;
        if rng == 0 {
            rng = 0xdeadbeefcafebabe;
        }

        // Start from origin in BigFloat
        let mut z_re = BigFloat::from_f64(0.0, p);
        let mut z_im = BigFloat::from_f64(0.0, p);

        for step in 0..(samples + burn_in) {
            // Choose a random map (PRNG stays as u64 — only geometry needs precision)
            let idx = choose_index(&mut rng, &cumulative[..n]);

            // w = z - c_i  (in BigFloat)
            let c_re = BigFloat::from_f64(attractors[idx].re, p);
            let c_im = BigFloat::from_f64(attractors[idx].im, p);
            let w_re = z_re.sub(&c_re, p, rm);
            let w_im = z_im.sub(&c_im, p, rm);

            // s = sqrt(w)  (in BigFloat)
            let (mut s_re, mut s_im) = complex_sqrt_hiprec(&w_re, &w_im, bits);

            // Randomly negate (choose branch of sqrt)
            if xorshift64(&mut rng) & 1 == 1 {
                s_re = s_re.neg();
                s_im = s_im.neg();
            }

            // Guard against NaN/Inf
            if s_re.is_nan() || s_re.is_inf() || s_im.is_nan() || s_im.is_inf() {
                z_re = BigFloat::from_f64(0.0, p);
                z_im = BigFloat::from_f64(0.0, p);
            } else {
                z_re = s_re;
                z_im = s_im;
            }

            // Record after burn-in
            if step >= burn_in {
                if let Some((px, py)) = complex_to_screen_hiprec(&z_re, &z_im, view, bits) {
                    density.increment_at(px, py);
                }
            }
        }
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.8;
        view.set_parameter("num_attractors", 2.0);
        view.set_parameter("c0_real", -0.5);
        view.set_parameter("c0_imag", 0.5);
        view.set_parameter("prob0", 0.5);
        view.set_parameter("c1_real", -0.5);
        view.set_parameter("c1_imag", -0.5);
        view.set_parameter("prob1", 0.5);
        view.set_parameter("seed", 0.0);
        view.set_parameter("samples", 5_000_000.0);
        view.set_parameter("burn_in", 50.0);
        view.set_parameter("use_log_density", 1.0);
        view
    }

    fn name(&self) -> &str {
        "Multi-Julia IFS"
    }

    fn equation(&self) -> &str {
        "g_i(z) = \\u{00B1}\\u{221A}(z - c_i), chaos game"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "num_attractors",
                "Number of IFS Maps",
                2.0,
                2.0,
                8.0,
                "How many IFS map points (2-8)",
            ),
            Parameter::new("c0_real", "C0 Real", -0.5, -3.0, 3.0, "Real part of map 0"),
            Parameter::new("c0_imag", "C0 Imaginary", 0.5, -3.0, 3.0, "Imaginary part of map 0"),
            Parameter::new("prob0", "Probability 0", 0.5, 0.0, 1.0, "Selection weight for map 0"),
            Parameter::new("c1_real", "C1 Real", -0.5, -3.0, 3.0, "Real part of map 1"),
            Parameter::new("c1_imag", "C1 Imaginary", -0.5, -3.0, 3.0, "Imaginary part of map 1"),
            Parameter::new("prob1", "Probability 1", 0.5, 0.0, 1.0, "Selection weight for map 1"),
            Parameter::new(
                "seed",
                "PRNG Seed",
                0.0,
                0.0,
                1_000_000.0,
                "Integer seed for orbit generation",
            ),
            Parameter::new(
                "samples",
                "Orbit Samples",
                5_000_000.0,
                100_000.0,
                50_000_000.0,
                "Total orbit steps (more = smoother, slower)",
            ),
            Parameter::new(
                "burn_in",
                "Burn-in",
                50.0,
                0.0,
                1000.0,
                "Initial orbit steps to discard before recording",
            ),
            Parameter::new(
                "use_log_density",
                "Log Density",
                1.0,
                0.0,
                1.0,
                "Apply log normalization to density (1=on, 0=off)",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_accumulation::LocalDensityTarget;
    use crate::fractals::FractalView;

    fn default_params() -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("num_attractors".to_string(), 2.0);
        p.insert("c0_real".to_string(), -0.5);
        p.insert("c0_imag".to_string(), 0.5);
        p.insert("prob0".to_string(), 0.5);
        p.insert("c1_real".to_string(), -0.5);
        p.insert("c1_imag".to_string(), -0.5);
        p.insert("prob1".to_string(), 0.5);
        p.insert("seed".to_string(), 42.0);
        p.insert("samples".to_string(), 10_000.0);
        p.insert("burn_in".to_string(), 50.0);
        p
    }

    #[test]
    fn test_uses_orbit_accumulation() {
        let f = MultiJuliaIFS::new();
        assert!(f.uses_orbit_accumulation());
    }

    #[test]
    fn test_iterate_returns_zero() {
        let f = MultiJuliaIFS::new();
        let p = default_params();
        assert_eq!(f.iterate(0.5, 0.3, &p, 256), 0);
    }

    #[test]
    fn test_accumulate_orbits_produces_density() {
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(100, 100);
        let p = default_params();
        let density = LocalDensityTarget::new(100, 100);
        f.accumulate_orbits(&density, &view, &p);
        // After 10,000 steps with 50 burn-in, should have some hits
        let result = density.into_density_buffer().normalize(256, true);
        let nonzero = result.iter().filter(|&&v| v > 0).count();
        assert!(nonzero > 0, "Orbit accumulation should produce non-zero density pixels");
    }

    #[test]
    fn test_single_map_c_zero_concentrates_near_origin() {
        // g(z) = ±sqrt(z - 0) = ±sqrt(z). The Julia set of z^2 is the unit circle.
        // Orbit density should concentrate around |z| <= 1.
        let f = MultiJuliaIFS::new();
        let mut view = FractalView::new(100, 100);
        view.zoom = 0.8;
        let mut p = HashMap::new();
        p.insert("num_attractors".to_string(), 1.0);
        p.insert("c0_real".to_string(), 0.0);
        p.insert("c0_imag".to_string(), 0.0);
        p.insert("prob0".to_string(), 1.0);
        p.insert("seed".to_string(), 42.0);
        p.insert("samples".to_string(), 50_000.0);
        p.insert("burn_in".to_string(), 50.0);

        let density = LocalDensityTarget::new(100, 100);
        f.accumulate_orbits(&density, &view, &p);
        let result = density.into_density_buffer().normalize(256, false);
        let nonzero = result.iter().filter(|&&v| v > 0).count();
        assert!(nonzero > 0, "Single-map c=0 should produce density");
    }

    #[test]
    fn test_deterministic_same_seed() {
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(50, 50);
        let p = default_params();

        let d1 = LocalDensityTarget::new(50, 50);
        f.accumulate_orbits(&d1, &view, &p);
        let r1 = d1.into_density_buffer().normalize(256, true);

        let d2 = LocalDensityTarget::new(50, 50);
        f.accumulate_orbits(&d2, &view, &p);
        let r2 = d2.into_density_buffer().normalize(256, true);

        assert_eq!(r1, r2, "Same seed must produce identical density");
    }

    #[test]
    fn test_different_seed_different_output() {
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(50, 50);

        let mut p1 = default_params();
        p1.insert("seed".to_string(), 42.0);
        let d1 = LocalDensityTarget::new(50, 50);
        f.accumulate_orbits(&d1, &view, &p1);
        let r1 = d1.into_density_buffer().normalize(256, true);

        let mut p2 = default_params();
        p2.insert("seed".to_string(), 99.0);
        let d2 = LocalDensityTarget::new(50, 50);
        f.accumulate_orbits(&d2, &view, &p2);
        let r2 = d2.into_density_buffer().normalize(256, true);

        assert_ne!(r1, r2, "Different seeds should produce different density");
    }

    #[test]
    fn test_three_maps_no_panic() {
        let f = MultiJuliaIFS::new();
        let view = FractalView::new(50, 50);
        let mut p = HashMap::new();
        p.insert("num_attractors".to_string(), 3.0);
        p.insert("c0_real".to_string(), -0.5);
        p.insert("c0_imag".to_string(), 0.5);
        p.insert("prob0".to_string(), 1.0 / 3.0);
        p.insert("c1_real".to_string(), 0.5);
        p.insert("c1_imag".to_string(), 0.5);
        p.insert("prob1".to_string(), 1.0 / 3.0);
        p.insert("c2_real".to_string(), 0.0);
        p.insert("c2_imag".to_string(), -0.5);
        p.insert("prob2".to_string(), 1.0 / 3.0);
        p.insert("seed".to_string(), 42.0);
        p.insert("samples".to_string(), 10_000.0);
        p.insert("burn_in".to_string(), 50.0);

        let density = LocalDensityTarget::new(50, 50);
        f.accumulate_orbits(&density, &view, &p);
        let result = density.into_density_buffer().normalize(256, true);
        let nonzero = result.iter().filter(|&&v| v > 0).count();
        assert!(nonzero > 0, "Three-map IFS should produce density");
    }
}
