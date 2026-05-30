//! Tetration Fractal
//!
//! The Tetration fractal is based on: z_{n+1} = c^{z_n}
//! where z₀ = c and iteration continues as: c^c, c^(c^c), c^(c^(c^c)), ...
//!
//! This creates a "power tower" of c values, leading to interesting convergence
//! and divergence behavior. The fractal supports customizable escape criteria.
//!
//! # Escape Criteria
//! Unlike traditional fractals that only check magnitude, Tetration supports:
//! - Magnitude exceeds threshold: |z| > threshold
//! - Real component exceeds threshold: Re(z) > threshold
//! - Imaginary component exceeds threshold: Im(z) > threshold
//! - Either component exceeds threshold
//!
//! # Reference
//! https://paulbourke.net/fractals/tetration/

use super::{Fractal, FractalView, Parameter};
use super::parameter_types::EscapeMode;
use astro_float::{BigFloat, Consts, RoundingMode};
use num_complex::Complex64;
use std::collections::HashMap;

/// Tetration fractal: z_{n+1} = c^{z_n}
pub struct Tetration;

impl Tetration {
    /// Creates a new Tetration fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Tetration {
    fn default() -> Self {
        Self::new()
    }
}

impl Tetration {
    fn bf_abs(value: &BigFloat) -> BigFloat {
        if value.is_negative() {
            value.neg()
        } else {
            value.clone()
        }
    }

    fn bf_atan2(
        y: &BigFloat,
        x: &BigFloat,
        p: usize,
        rm: RoundingMode,
        cc: &mut Consts,
    ) -> BigFloat {
        if x.is_zero() && y.is_zero() {
            return BigFloat::new(p);
        }

        let pi = BigFloat::from_f64(-1.0, p).acos(p, rm, cc);
        if x.is_zero() {
            let half_pi = pi.div(&BigFloat::from_f64(2.0, p), p, rm);
            return if y.is_negative() { half_pi.neg() } else { half_pi };
        }

        let ratio = y.div(x, p, rm);
        let atan_val = ratio.atan(p, rm, cc);
        if x.is_negative() {
            if y.is_negative() {
                atan_val.sub(&pi, p, rm)
            } else {
                atan_val.add(&pi, p, rm)
            }
        } else {
            atan_val
        }
    }

    fn complex_log_bf(
        zr: &BigFloat,
        zi: &BigFloat,
        p: usize,
        rm: RoundingMode,
        cc: &mut Consts,
    ) -> Option<(BigFloat, BigFloat)> {
        let zr2 = zr.mul(zr, p, rm);
        let zi2 = zi.mul(zi, p, rm);
        let r_sq = zr2.add(&zi2, p, rm);
        if r_sq.is_zero() || r_sq.is_nan() || r_sq.is_inf() {
            return None;
        }

        let r = r_sq.sqrt(p, rm);
        if r.is_zero() || r.is_nan() || r.is_inf() {
            return None;
        }

        let theta = Self::bf_atan2(zi, zr, p, rm, cc);
        Some((r.ln(p, rm, cc), theta))
    }

    fn complex_exp_bf(
        zr: &BigFloat,
        zi: &BigFloat,
        p: usize,
        rm: RoundingMode,
        cc: &mut Consts,
    ) -> (BigFloat, BigFloat) {
        let exp_re = zr.exp(p, rm, cc);
        let cos_im = zi.cos(p, rm, cc);
        let sin_im = zi.sin(p, rm, cc);
        (exp_re.mul(&cos_im, p, rm), exp_re.mul(&sin_im, p, rm))
    }
}

impl Fractal for Tetration {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        
        // Get parameters (with defaults)
        let threshold = parameters.get("threshold").copied().unwrap_or(1e7);
        let escape_mode_f64 = parameters.get("escape_mode").copied().unwrap_or(0.0);
        let escape_mode = EscapeMode::from_f64(escape_mode_f64);
        
        // Start iteration: z₀ = c
        let mut z = c;
        
        for i in 0..max_iter {
            // Check escape condition based on selected mode
            if escape_mode.test(z.re, z.im, threshold) {
                return i;
            }
            
            // z_{n+1} = c^{z_n}
            // Complex exponentiation: a^b = exp(b * ln(a))
            // We use the principal branch of the complex logarithm
            z = (z * c.ln()).exp();
            
            // Additional safety check for NaN/Inf (can happen with extreme values)
            if !z.is_finite() {
                return i;
            }
        }
        
        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        
        // Good starting view for Tetration
        // The interesting regions are typically around the origin
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.45; // Zoom out 10% more to see more structure
        
        // Set default parameters
        view.set_parameter("threshold", 1e7);
        view.set_parameter("escape_mode", EscapeMode::Magnitude.to_f64());
        
        view
    }

    fn name(&self) -> &str {
        "Tetration"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = c^(z_n)"
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    fn iterate_hiprec(
        &self,
        c_real: &BigFloat,
        c_imag: &BigFloat,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
        bits: u32,
    ) -> u32 {
        let threshold = parameters.get("threshold").copied().unwrap_or(1e7);
        let escape_mode = EscapeMode::from_f64(parameters.get("escape_mode").copied().unwrap_or(0.0));

        let p = bits as usize;
        let rm = RoundingMode::ToEven;
        let threshold_bf = BigFloat::from_f64(threshold, p);
        let threshold_sq = BigFloat::from_f64(threshold * threshold, p);
        let mut cc = Consts::new().expect("BigFloat constants cache");

        let Some((ln_cr, ln_ci)) = Self::complex_log_bf(c_real, c_imag, p, rm, &mut cc) else {
            return 0;
        };

        let mut zr = c_real.clone();
        let mut zi = c_imag.clone();

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);
            let norm_sq = zr2.add(&zi2, p, rm);

            let escaped = match escape_mode {
                EscapeMode::Magnitude => norm_sq.cmp(&threshold_sq).map_or(false, |value| value > 0),
                EscapeMode::Real => Self::bf_abs(&zr).cmp(&threshold_bf).map_or(false, |value| value > 0),
                EscapeMode::Imaginary => Self::bf_abs(&zi).cmp(&threshold_bf).map_or(false, |value| value > 0),
                EscapeMode::Either => {
                    Self::bf_abs(&zr).cmp(&threshold_bf).map_or(false, |value| value > 0)
                        || Self::bf_abs(&zi).cmp(&threshold_bf).map_or(false, |value| value > 0)
                }
            };
            if escaped {
                return iter;
            }

            let mul_re = zr.mul(&ln_cr, p, rm).sub(&zi.mul(&ln_ci, p, rm), p, rm);
            let mul_im = zr.mul(&ln_ci, p, rm).add(&zi.mul(&ln_cr, p, rm), p, rm);
            let (new_zr, new_zi) = Self::complex_exp_bf(&mul_re, &mul_im, p, rm, &mut cc);

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "threshold",
                "Escape Threshold",
                1e7,              // default
                10.0,             // min (10¹)
                1e10,             // max (10¹⁰)
                "Threshold for escape criterion (supports scientific notation like 1e7 or 10e6)"
            ),
            Parameter::new(
                "escape_mode",
                "Escape Criterion",
                0.0,              // default (Magnitude)
                0.0,              // min
                3.0,              // max
                "How to test escape: 0=Magnitude, 1=Real, 2=Imaginary, 3=Either"
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_bf(re: f64, im: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        (BigFloat::from_f64(re, p), BigFloat::from_f64(im, p))
    }

    fn params(threshold: f64, escape_mode: EscapeMode) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), threshold);
        params.insert("escape_mode".to_string(), escape_mode.to_f64());
        params
    }

    #[test]
    fn test_tetration_name() {
        let fractal = Tetration::new();
        assert_eq!(fractal.name(), "Tetration");
    }

    #[test]
    fn test_tetration_parameters() {
        let fractal = Tetration::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "threshold");
        assert_eq!(params[1].name, "escape_mode");
    }

    #[test]
    fn test_tetration_default_view() {
        let fractal = Tetration::new();
        let view = fractal.default_view(1280, 720);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.45);
        assert_eq!(view.get_parameter("threshold"), Some(1e7));
        assert_eq!(view.get_parameter("escape_mode"), Some(0.0));
    }

    #[test]
    fn test_tetration_iteration_magnitude_mode() {
        let fractal = Tetration::new();
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), 1e7);
        params.insert("escape_mode".to_string(), 0.0); // Magnitude
        
        // Test a point that should escape quickly
        let iters = fractal.iterate(10.0, 10.0, &params, 100);
        assert!(iters < 100, "Expected quick escape for large point");
    }

    #[test]
    fn test_tetration_iteration_real_mode() {
        let fractal = Tetration::new();
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), 100.0);
        params.insert("escape_mode".to_string(), 1.0); // Real component
        
        // Test basic iteration (results will vary based on formula)
        let iters = fractal.iterate(2.0, 1.0, &params, 100);
        assert!(iters <= 100);
    }

    #[test]
    fn test_tetration_different_escape_modes() {
        let fractal = Tetration::new();
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), 1000.0);
        
        // Test same point with different escape modes
        let c_real = 1.5;
        let c_imag = 0.5;
        
        params.insert("escape_mode".to_string(), 0.0); // Magnitude
        let iters_mag = fractal.iterate(c_real, c_imag, &params, 100);
        
        params.insert("escape_mode".to_string(), 1.0); // Real
        let iters_real = fractal.iterate(c_real, c_imag, &params, 100);
        
        params.insert("escape_mode".to_string(), 2.0); // Imaginary
        let iters_imag = fractal.iterate(c_real, c_imag, &params, 100);
        
        // All should complete without panic
        assert!(iters_mag <= 100);
        assert!(iters_real <= 100);
        assert!(iters_imag <= 100);
    }

    #[test]
    fn test_tetration_nan_safety() {
        let fractal = Tetration::new();
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), 1e7);
        params.insert("escape_mode".to_string(), 0.0);
        
        // Test with values that might cause numerical issues
        let iters = fractal.iterate(1000.0, 1000.0, &params, 100);
        assert!(iters <= 100, "Should handle large values safely");
    }

    #[test]
    fn test_tetration_supports_hiprec() {
        assert!(Tetration::new().supports_hiprec());
    }

    #[test]
    fn test_tetration_hiprec_interior_point() {
        let fractal = Tetration::new();
        let params = params(1e7, EscapeMode::Magnitude);
        let (cr, ci) = to_bf(1.0, 0.0, 128);
        assert_eq!(fractal.iterate_hiprec(&cr, &ci, &params, 128, 128), 128);
    }

    #[test]
    fn test_tetration_hiprec_escaping_point() {
        let fractal = Tetration::new();
        let params = params(1e7, EscapeMode::Magnitude);
        let (cr, ci) = to_bf(10.0, 10.0, 128);
        assert!(fractal.iterate_hiprec(&cr, &ci, &params, 64, 128) < 64);
    }

    #[test]
    fn test_tetration_hiprec_matches_f64_on_robust_points() {
        let fractal = Tetration::new();
        let params = params(1e7, EscapeMode::Magnitude);
        // Keep these on well-behaved real-axis points where the f64 path does not
        // overflow earlier than the BigFloat path.
        let points = [(0.25, 0.0), (1.0, 0.0), (1.5, 0.0)];

        for (re, im) in points {
            let (cr, ci) = to_bf(re, im, 128);
            let hp = fractal.iterate_hiprec(&cr, &ci, &params, 64, 128);
            let fp = fractal.iterate(re, im, &params, 64);
            assert_eq!(hp, fp, "Mismatch at point ({re}, {im})");
        }
    }

    #[test]
    fn test_tetration_hiprec_bit_width_smoke() {
        let fractal = Tetration::new();
        let params = params(1e7, EscapeMode::Magnitude);

        for bits in [64, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(2.0, 0.0, bits);
            let result = fractal.iterate_hiprec(&cr, &ci, &params, 32, bits);
            assert!(result <= 32, "bits={bits} should produce a valid iteration count");
        }
    }
}
