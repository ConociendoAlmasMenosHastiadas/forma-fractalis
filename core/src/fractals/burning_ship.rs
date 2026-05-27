//! Burning Ship Fractal Implementation
//!
//! The Burning Ship fractal: z(n+1) = (|Re(z)| + i|Im(z)|)² + c
//! where z starts at 0 and c is the point being tested.
//!
//! The absolute value operation on components creates the distinctive
//! "ship" shape with a prominent bow structure.
//!
//! Reference: https://paulbourke.net/fractals/burnship/

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use astro_float::{BigFloat, RoundingMode};

/// Burning Ship fractal
pub struct BurningShip;

impl BurningShip {
    /// Creates a new Burning Ship fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for BurningShip {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for BurningShip {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        let mut z = Complex64::new(0.0, 0.0);
        let mut iter = 0;
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_sq = escape_r * escape_r;

        while iter < max_iter {
            if z.norm_sqr() > escape_sq {
                break;
            }

            // Apply absolute value to both components before squaring
            let z_abs = Complex64::new(z.re.abs(), z.im.abs());
            z = z_abs * z_abs + c;
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        // Classic Burning Ship viewing position
        view.center_x = -0.5;
        view.center_y = -0.6;
        view.zoom = 0.8;
        view
    }

    fn name(&self) -> &str {
        "Burning Ship"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = (|Re(z_n)| + i|Im(z_n)|)^2 + c"
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    /// High-precision Burning Ship iteration using software floating-point arithmetic.
    ///
    /// The Burning Ship formula applied per iteration:
    ///   z = (|Re(z)| + i|Im(z)|)² + c
    ///
    /// In expanded form:
    ///   new_re = Re(z)² - Im(z)² + Re(c)   (squaring removes sign so |·|² = ·² here)
    ///   new_im = 2·|Re(z)|·|Im(z)| + Im(c)  (abs required on cross term)
    ///
    /// `bits` is one of: 64, 128, 256, 512, 1024.
    fn iterate_hiprec(
        &self,
        c_real: &BigFloat,
        c_imag: &BigFloat,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
        bits: u32,
    ) -> u32 {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let cr = c_real.clone();
        let ci = c_imag.clone();

        // z starts at origin (same as f64 path)
        let mut zr = BigFloat::from_f64(0.0, p);
        let mut zi = BigFloat::from_f64(0.0, p);

        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let four = BigFloat::from_f64(escape_r * escape_r, p);
        let two  = BigFloat::from_f64(2.0, p);
        let zero = BigFloat::from_f64(0.0, p);

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);

            // Escape check: |z|² > 4
            let norm_sq = zr2.add(&zi2, p, rm);
            if norm_sq.cmp(&four).map_or(false, |v| v > 0) {
                return iter;
            }

            // Cross term: 2·|Re(z)|·|Im(z)|
            // |Re(z)·Im(z)| = abs(zr·zi)
            let zr_zi = zr.mul(&zi, p, rm);
            let abs_zr_zi = if zr_zi.is_negative() {
                zero.sub(&zr_zi, p, rm)
            } else {
                zr_zi
            };

            // new_re = zr² - zi² + cr  (same as standard Mandelbrot — squaring removes sign)
            // new_im = 2·|zr·zi| + ci
            let new_zr = zr2.sub(&zi2, p, rm).add(&cr, p, rm);
            let new_zi = two.mul(&abs_zr_zi, p, rm).add(&ci, p, rm);

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }

    // Burning Ship parameters
    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "escape_radius".to_string(),
                label: "Escape Radius".to_string(),
                default: 2.0,
                min: 0.5,
                max: 100.0,
                description: "Escape radius: iterate escapes when |z| > escape_radius.".to_string(),
            },
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

    /// Origin is inside the Burning Ship set — should reach max_iter.
    #[test]
    fn hiprec_origin_in_set() {
        let f = BurningShip::new();
        let (cr, ci) = to_bf(0.0, 0.0, 128);
        assert_eq!(f.iterate_hiprec(&cr, &ci, &HashMap::new(), 100, 128), 100,
            "origin should not escape");
    }

    /// A clearly exterior point should escape well below max_iter.
    #[test]
    fn hiprec_far_point_escapes() {
        let f = BurningShip::new();
        let (cr, ci) = to_bf(2.0, 2.0, 128);
        assert!(f.iterate_hiprec(&cr, &ci, &HashMap::new(), 100, 128) < 100,
            "2+2i should escape immediately");
    }

    /// Hi-prec and f64 must agree on unambiguous exterior/interior points.
    #[test]
    fn hiprec_matches_f64_for_robust_points() {
        let f = BurningShip::new();
        let cases: &[(f64, f64)] = &[
            (2.0, 0.0),    // clearly outside
            (0.0, 2.0),    // clearly outside
            (-2.0, -2.0),  // clearly outside
            (0.0, 0.0),    // interior (origin stays at 0)
        ];
        for &(re, im) in cases {
            let f64_iters = f.iterate(re, im, &HashMap::new(), 100);
            let (cr, ci) = to_bf(re, im, 128);
            let hp_iters = f.iterate_hiprec(&cr, &ci, &HashMap::new(), 100, 128);
            assert_eq!(
                f64_iters, hp_iters,
                "f64 and hiprec disagree at ({re},{im}): f64={f64_iters} hiprec={hp_iters}"
            );
        }
    }

    /// Smoke test across all supported bit widths — must not panic.
    #[test]
    fn hiprec_bit_widths_smoke() {
        let f = BurningShip::new();
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(2.0, 0.0, bits);
            let result = f.iterate_hiprec(&cr, &ci, &HashMap::new(), 50, bits);
            assert!(result < 50, "2+0i should escape at {bits} bits");
        }
    }

    /// Escaping point returns the same iteration count at different precisions
    /// (precision should not change the result for a clearly-outside point).
    #[test]
    fn hiprec_precision_agrees_on_escaping_point() {
        let f = BurningShip::new();
        let (cr64,  ci64)  = to_bf(1.5, 0.5, 64);
        let (cr128, ci128) = to_bf(1.5, 0.5, 128);
        let (cr256, ci256) = to_bf(1.5, 0.5, 256);
        let r64  = f.iterate_hiprec(&cr64,  &ci64,  &HashMap::new(), 100, 64);
        let r128 = f.iterate_hiprec(&cr128, &ci128, &HashMap::new(), 100, 128);
        let r256 = f.iterate_hiprec(&cr256, &ci256, &HashMap::new(), 100, 256);
        assert_eq!(r64, r128, "64-bit vs 128-bit disagree at (1.5, 0.5)");
        assert_eq!(r128, r256, "128-bit vs 256-bit disagree at (1.5, 0.5)");
    }
}
