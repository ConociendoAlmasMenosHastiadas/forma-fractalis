//! Wallpaper fractal (orbit accumulation, per-screen seeds)
//!
//! Renders the real 2D map described by Paul Bourke:
//!
//!   x_{n+1} = y_n - sign(x_n) * sqrt(abs(b*x_n - c))
//!   y_{n+1} = a - x_n
//!
//! Each screen pixel provides its own initial seed `(x_0, y_0)` in the view
//! plane. The orbit from every seed contributes to a shared density histogram.
//!
//! Reference: https://paulbourke.net/fractals/wallpaper/

use super::{Fractal, FractalView, OrbitParallelism, Parameter};
use crate::orbit_accumulation::{DensityBuffer, OrbitTarget, complex_to_screen_hiprec};
use astro_float::{BigFloat, RoundingMode};
use num_complex::Complex64;
use std::collections::HashMap;

/// Wallpaper attractor fractal.
pub struct Wallpaper;

impl Wallpaper {
    // Paul Bourke's first published Wallpaper example: a=1, b=4, c=60.
    pub const DEFAULT_A: f64 = 1.0;
    pub const DEFAULT_B: f64 = 4.0;
    pub const DEFAULT_C: f64 = 60.0;
    pub const DEFAULT_SAMPLES: u64 = 24;
    pub const MAX_SAMPLES: u64 = 1_000;
    pub const DEFAULT_BURN_IN: u64 = 40;
    const ESCAPE_LIMIT: f64 = 1e12;

    pub fn new() -> Self {
        Self
    }

    #[inline]
    fn step(x: f64, y: f64, a: f64, b: f64, c: f64) -> (f64, f64) {
        let next_x = y - x.signum() * (b * x - c).abs().sqrt();
        let next_y = a - x;
        (next_x, next_y)
    }

    #[inline]
    fn escaped(x: f64, y: f64) -> bool {
        !x.is_finite()
            || !y.is_finite()
            || x.abs() > Self::ESCAPE_LIMIT
            || y.abs() > Self::ESCAPE_LIMIT
    }

    #[inline]
    fn bf_abs(value: &BigFloat) -> BigFloat {
        if value.is_negative() {
            value.neg()
        } else {
            value.clone()
        }
    }

    fn step_hiprec(
        x: &BigFloat,
        y: &BigFloat,
        a: &BigFloat,
        b: &BigFloat,
        c: &BigFloat,
        bits: u32,
    ) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let bx = b.mul(x, p, rm);
        let diff = bx.sub(c, p, rm);
        let root = Self::bf_abs(&diff).sqrt(p, rm);

        let next_x = if x.is_negative() {
            y.add(&root, p, rm)
        } else if x.is_zero() {
            y.clone()
        } else {
            y.sub(&root, p, rm)
        };
        let next_y = a.sub(x, p, rm);

        (next_x, next_y)
    }

    #[inline]
    fn bf_escaped(x: &BigFloat, y: &BigFloat, limit: &BigFloat) -> bool {
        x.is_nan()
            || x.is_inf()
            || y.is_nan()
            || y.is_inf()
            || Self::bf_abs(x).cmp(limit).map_or(false, |value| value > 0)
            || Self::bf_abs(y).cmp(limit).map_or(false, |value| value > 0)
    }

    #[inline]
    fn row_bounds(view: &FractalView, params: &HashMap<String, f64>) -> (u32, u32) {
        let row_start = params.get("row_start").copied().unwrap_or(0.0).max(0.0) as u32;
        let row_end = params
            .get("row_end")
            .copied()
            .unwrap_or(view.height as f64)
            .max(0.0) as u32;

        let row_start = row_start.min(view.height);
        let row_end = row_end.min(view.height).max(row_start);
        (row_start, row_end)
    }
}

impl Default for Wallpaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Wallpaper {
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

    fn orbit_parallelism(&self) -> OrbitParallelism {
        OrbitParallelism::RowChunks
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
        let a = params.get("a").copied().unwrap_or(Self::DEFAULT_A);
        let b = params.get("b").copied().unwrap_or(Self::DEFAULT_B);
        let c = params.get("c").copied().unwrap_or(Self::DEFAULT_C);
        let samples = params
            .get("samples")
            .copied()
            .unwrap_or(Self::DEFAULT_SAMPLES as f64) as u64;
        let burn_in = params
            .get("burn_in")
            .copied()
            .unwrap_or(Self::DEFAULT_BURN_IN as f64) as u64;
        let (row_start, row_end) = Self::row_bounds(view, params);

        if samples == 0 || row_start >= row_end {
            return;
        }

        for py in row_start..row_end {
            for px in 0..view.width {
                let (mut x, mut y) = view.screen_to_complex(px, py);

                for step in 0..(samples + burn_in) {
                    (x, y) = Self::step(x, y, a, b, c);
                    if Self::escaped(x, y) {
                        break;
                    }

                    if step >= burn_in {
                        target.increment(Complex64::new(x, y), view);
                    }
                }
            }
        }
    }

    fn accumulate_orbits_hiprec(
        &self,
        density: &mut DensityBuffer,
        view: &FractalView,
        params: &HashMap<String, f64>,
        bits: u32,
    ) {
        let p = bits as usize;
        let a = BigFloat::from_f64(params.get("a").copied().unwrap_or(Self::DEFAULT_A), p);
        let b = BigFloat::from_f64(params.get("b").copied().unwrap_or(Self::DEFAULT_B), p);
        let c = BigFloat::from_f64(params.get("c").copied().unwrap_or(Self::DEFAULT_C), p);
        let samples = params
            .get("samples")
            .copied()
            .unwrap_or(Self::DEFAULT_SAMPLES as f64) as u64;
        let burn_in = params
            .get("burn_in")
            .copied()
            .unwrap_or(Self::DEFAULT_BURN_IN as f64) as u64;
        let escape_limit = BigFloat::from_f64(Self::ESCAPE_LIMIT, p);
        let (row_start, row_end) = Self::row_bounds(view, params);

        if samples == 0 || row_start >= row_end {
            return;
        }

        for py in row_start..row_end {
            for px in 0..view.width {
                let (mut x, mut y) = view.screen_to_complex_hiprec(px, py, bits);

                for step in 0..(samples + burn_in) {
                    (x, y) = Self::step_hiprec(&x, &y, &a, &b, &c, bits);
                    if Self::bf_escaped(&x, &y, &escape_limit) {
                        break;
                    }

                    if step >= burn_in {
                        if let Some((screen_x, screen_y)) = complex_to_screen_hiprec(&x, &y, view, bits) {
                            density.increment_at(screen_x, screen_y);
                        }
                    }
                }
            }
        }
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        // Frame the broader finite-step shell for Bourke example #1. The
        // initial tight view clipped most of the structure and read as smear.
        view.center_x = -0.65;
        view.center_y = 1.65;
        view.zoom = 0.09;
        view.set_parameter("a", Self::DEFAULT_A);
        view.set_parameter("b", Self::DEFAULT_B);
        view.set_parameter("c", Self::DEFAULT_C);
        view.set_parameter("samples", Self::DEFAULT_SAMPLES as f64);
        view.set_parameter("burn_in", Self::DEFAULT_BURN_IN as f64);
        view.set_parameter("use_log_density", 1.0);
        view
    }

    fn name(&self) -> &str {
        "Wallpaper"
    }

    fn equation(&self) -> &str {
        "x_{n+1} = y_n - sign(x_n)*sqrt(abs(b*x_n-c)), y_{n+1} = a - x_n"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "a",
                "a",
                Self::DEFAULT_A,
                0.0,
                100.0,
                "Real offset in y_{n+1} = a - x_n.",
            ),
            Parameter::new(
                "b",
                "b",
                Self::DEFAULT_B,
                0.0,
                100.0,
                "Scales x_n inside the square-root term.",
            ),
            Parameter::new(
                "c",
                "c",
                Self::DEFAULT_C,
                0.0,
                100.0,
                "Offset inside sqrt(abs(b*x_n-c)).",
            ),
            Parameter::new(
                "samples",
                "Samples/seed",
                Self::DEFAULT_SAMPLES as f64,
                1.0,
                Self::MAX_SAMPLES as f64,
                "Orbit steps recorded for each screen seed after burn-in.",
            ),
            Parameter::new(
                "burn_in",
                "Burn-in",
                Self::DEFAULT_BURN_IN as f64,
                0.0,
                512.0,
                "Initial orbit steps discarded before recording density.",
            ),
            Parameter::new(
                "use_log_density",
                "Log density",
                1.0,
                0.0,
                1.0,
                "Log-normalise the density histogram (1=on, 0=off).",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit_accumulation::LocalDensityTarget;

    #[cfg(feature = "gpu")]
    use crate::gpu::{FractalRenderer, WgpuRenderer};

    fn default_params() -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("a".to_string(), Wallpaper::DEFAULT_A);
        params.insert("b".to_string(), Wallpaper::DEFAULT_B);
        params.insert("c".to_string(), Wallpaper::DEFAULT_C);
        params.insert("samples".to_string(), 6.0);
        params.insert("burn_in".to_string(), 4.0);
        params
    }

    fn bigfloat_to_f64(value: &BigFloat) -> f64 {
        format!("{}", value).parse::<f64>().unwrap_or(f64::NAN)
    }

    #[test]
    fn test_uses_orbit_accumulation() {
        let fractal = Wallpaper::new();
        assert!(fractal.uses_orbit_accumulation());
    }

    #[test]
    fn test_orbit_parallelism_is_row_chunks() {
        let fractal = Wallpaper::new();
        assert_eq!(fractal.orbit_parallelism(), OrbitParallelism::RowChunks);
    }

    #[test]
    fn test_supports_hiprec() {
        let fractal = Wallpaper::new();
        assert!(fractal.supports_hiprec());
    }

    #[test]
    fn test_iterate_returns_zero() {
        let fractal = Wallpaper::new();
        let params = default_params();
        assert_eq!(fractal.iterate(0.0, 0.0, &params, 100), 0);
    }

    #[test]
    fn test_step_reference_values() {
        let (next_x, next_y) = Wallpaper::step(2.0, 3.0, 1.0, 4.0, 60.0);
        assert!((next_x - (3.0 - 52.0_f64.sqrt())).abs() < 1e-12);
        assert!((next_y + 1.0).abs() < 1e-12);

        let (next_x, next_y) = Wallpaper::step(-2.0, 3.0, 1.0, 4.0, 60.0);
        assert!((next_x - (3.0 + 68.0_f64.sqrt())).abs() < 1e-12);
        assert!((next_y - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_accumulate_orbits_records_hits() {
        let fractal = Wallpaper::new();
        let params = default_params();
        let view = fractal.default_view(48, 48);
        let target = LocalDensityTarget::new(48, 48);

        fractal.accumulate_orbits(&target, &view, &params);

        let buffer = target.into_density_buffer();
        let normalised = buffer.normalize(256, true);
        let non_zero = normalised.iter().filter(|&&value| value > 0).count();
        assert!(non_zero > 0, "wallpaper orbit accumulation should record some hits");
    }

    #[test]
    fn test_empty_row_chunk_records_no_hits() {
        let fractal = Wallpaper::new();
        let mut params = default_params();
        params.insert("row_start".to_string(), 8.0);
        params.insert("row_end".to_string(), 8.0);
        let view = fractal.default_view(32, 32);
        let target = LocalDensityTarget::new(32, 32);

        fractal.accumulate_orbits(&target, &view, &params);

        let buffer = target.into_density_buffer();
        let normalised = buffer.normalize(256, true);
        assert!(normalised.iter().all(|&value| value == 0));
    }

    #[test]
    fn test_hiprec_step_matches_f64() {
        let bits = 256;
        let p = bits as usize;
        let a = BigFloat::from_f64(0.1, p);
        let b = BigFloat::from_f64(0.1, p);
        let c = BigFloat::from_f64(10.0, p);
        let x = BigFloat::from_f64(1.25, p);
        let y = BigFloat::from_f64(-0.5, p);

        let (f64_x, f64_y) = Wallpaper::step(1.25, -0.5, 0.1, 0.1, 10.0);
        let (hiprec_x, hiprec_y) = Wallpaper::step_hiprec(&x, &y, &a, &b, &c, bits);

        assert!((bigfloat_to_f64(&hiprec_x) - f64_x).abs() < 1e-12);
        assert!((bigfloat_to_f64(&hiprec_y) - f64_y).abs() < 1e-12);
    }

    #[test]
    fn test_hiprec_smoke_across_bit_widths() {
        let fractal = Wallpaper::new();
        let view = fractal.default_view(24, 24);
        let params = default_params();

        for bits in [64, 128, 256, 512, 1024] {
            let mut density = DensityBuffer::new(24, 24);
            fractal.accumulate_orbits_hiprec(&mut density, &view, &params, bits);

            let normalised = density.normalize(256, true);
            let non_zero = normalised.iter().filter(|&&value| value > 0).count();
            assert!(
                non_zero > 0,
                "wallpaper hi-prec orbit accumulation should record hits at {bits} bits"
            );
        }
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_orbit_density_smoke_matches_cpu_shape() {
        let Ok(renderer) = WgpuRenderer::new() else {
            eprintln!("Skipping Wallpaper GPU smoke test: no GPU renderer available");
            return;
        };

        let fractal = Wallpaper::new();
        let view = fractal.default_view(64, 64);
        let params = default_params();
        let target = LocalDensityTarget::new(64, 64);
        fractal.accumulate_orbits(&target, &view, &params);

        let cpu_buffer = target.into_density_buffer();
        let cpu_normalized = cpu_buffer.normalize(256, true);

        let gpu_raw = renderer
            .render_orbit_density(
                view.width,
                view.height,
                &params,
                fractal.name(),
                view.center_x as f32,
                view.center_y as f32,
                view.zoom as f32,
            )
            .expect("Wallpaper GPU orbit render should succeed");

        assert_eq!(gpu_raw.len(), (view.width * view.height) as usize);
        assert!(gpu_raw.iter().any(|&value| value > 0));

        let gpu_normalized = DensityBuffer::from_raw(view.width, view.height, gpu_raw).normalize(256, true);
        let mismatches = cpu_normalized
            .iter()
            .zip(gpu_normalized.iter())
            .filter(|(cpu, gpu)| cpu.abs_diff(**gpu) > 24)
            .count();
        let mismatch_rate = mismatches as f64 / cpu_normalized.len() as f64;

        assert!(
            mismatch_rate <= 0.20,
            "Wallpaper GPU mismatch rate too high: {mismatch_rate:.3}"
        );
    }
}