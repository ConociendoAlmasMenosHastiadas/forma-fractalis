//! Cactus Fractal
//!
//! The Cactus fractal is defined by the iteration:
//! z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0
//!
//! For every point z_0 in the complex plane, we iterate this function
//! and determine if the orbit escapes or remains bounded.
//!
//! Reference: https://paulbourke.net/fractals/cactus/

use crate::fractals::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use astro_float::{BigFloat, RoundingMode};

/// Cactus fractal implementation
pub struct Cactus;

impl Cactus {
    pub fn new() -> Self {
        Cactus
    }
    
    /// Calculate the escape radius for a given z_0
    /// Using R > max(1, sqrt(abs(z_0-1)+1), abs(z0)^(1/3))
    fn escape_radius(z0: Complex64) -> f64 {
        let r1 = 1.0_f64;
        let r2 = ((z0 - Complex64::new(1.0, 0.0)).norm() + 1.0).sqrt();
        let r3 = z0.norm().powf(1.0 / 3.0);
        
        r1.max(r2).max(r3) * 2.0  // Add safety factor
    }
}

impl Default for Cactus {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Cactus {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        _parameters: &std::collections::HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let z0 = Complex64::new(c_real, c_imag);
        let mut z = z0;
        
        // Calculate escape radius for this specific z_0
        let escape_radius = Self::escape_radius(z0);
        let escape_radius_sq = escape_radius * escape_radius;
        
        for i in 0..max_iter {
            // Check if escaped
            if z.norm_sqr() > escape_radius_sq {
                return i;
            }
            
            // z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0
            z = z.powi(3) + (z0 - Complex64::new(1.0, 0.0)) * z - z0;
            
            // Check for NaN or infinity
            if !z.re.is_finite() || !z.im.is_finite() {
                return i;
            }
        }
        
        // Did not escape
        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        // Center view on an interesting region
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.6;  // Zoom out to see more structure
        view
    }

    fn name(&self) -> &str {
        "Cactus"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_n^3 + (z_0 - 1)*z_n - z_0"
    }

    fn parameters(&self) -> Vec<Parameter> {
        // Cactus fractal has no adjustable parameters
        Vec::new()
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    /// High-precision Cactus iteration using BigFloat arithmetic.
    ///
    /// Formula: z_{n+1} = z_n^3 + (z_0 - 1)*z_n - z_0
    ///
    /// All arithmetic is carried out at `bits` precision. The escape radius is
    /// computed from z_0 exactly as in the f64 path.
    fn iterate_hiprec(
        &self,
        c_real: &BigFloat,
        c_imag: &BigFloat,
        _parameters: &std::collections::HashMap<String, f64>,
        max_iter: u32,
        bits: u32,
    ) -> u32 {
        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        // z_0 = c (the pixel coordinate)
        let z0r = c_real.clone();
        let z0i = c_imag.clone();

        // Compute escape radius matching the f64 path:
        //   r = 2 * max(1, sqrt(|z_0 - 1| + 1), |z_0|^(1/3))
        //
        // We compute this in f64 since it is a one-time per-pixel constant derived
        // from z_0. Converting to BigFloat would not improve accuracy meaningfully
        // for the escape-radius comparison.
        //
        // astro-float 0.9 has no to_f64() method; use Display-parse conversion.
        let z0r_f64: f64 = format!("{}", c_real).parse().unwrap_or(0.0);
        let z0i_f64: f64 = format!("{}", c_imag).parse().unwrap_or(0.0);
        let escape_r = Cactus::escape_radius(Complex64::new(z0r_f64, z0i_f64));
        let escape_radius_sq = BigFloat::from_f64(escape_r * escape_r, p);

        let one  = BigFloat::from_f64(1.0, p);
        let three = BigFloat::from_f64(3.0, p);

        // (z_0 - 1) terms, precomputed
        let z0r_minus1 = z0r.sub(&one, p, rm);
        // z0i is unchanged: (z_0 - 1) = (z0r - 1) + i*z0i

        let mut zr = z0r.clone();
        let mut zi = z0i.clone();

        for iter in 0..max_iter {
            // Escape check: |z|^2 > escape_radius^2
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);
            let norm_sq = zr2.add(&zi2, p, rm);
            if norm_sq.cmp(&escape_radius_sq).map_or(false, |v| v > 0) {
                return iter;
            }

            // z^3:  re = zr^3 - 3*zr*zi^2,  im = 3*zr^2*zi - zi^3
            let zr3 = zr.mul(&zr2, p, rm);
            let zi3 = zi.mul(&zi2, p, rm);
            let z3r = zr3.sub(&three.mul(&zr, p, rm).mul(&zi2, p, rm), p, rm);
            let z3i = three.mul(&zr2, p, rm).mul(&zi, p, rm).sub(&zi3, p, rm);

            // (z_0 - 1) * z:
            //   re = (z0r-1)*zr - z0i*zi
            //   im = (z0r-1)*zi + z0i*zr
            let prod_r = z0r_minus1.mul(&zr, p, rm).sub(&z0i.mul(&zi, p, rm), p, rm);
            let prod_i = z0r_minus1.mul(&zi, p, rm).add(&z0i.mul(&zr, p, rm), p, rm);

            // z_{n+1} = z^3 + (z_0-1)*z - z_0
            let new_zr = z3r.add(&prod_r, p, rm).sub(&z0r, p, rm);
            let new_zi = z3i.add(&prod_i, p, rm).sub(&z0i, p, rm);

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cactus_origin() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        let iters = cactus.iterate(0.0, 0.0, &params, 100);
        // Origin should either escape or remain bounded
        assert!(iters <= 100);
    }

    #[test]
    fn test_cactus_far_point() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        // Point far from origin should escape quickly
        let iters = cactus.iterate(10.0, 10.0, &params, 100);
        assert!(iters < 10, "Far point should escape quickly");
    }

    #[test]
    fn test_cactus_various_points() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        
        let test_points = vec![
            (0.5, 0.5),
            (-0.5, 0.5),
            (0.5, -0.5),
            (-0.5, -0.5),
            (1.0, 0.0),
            (0.0, 1.0),
        ];
        
        for (x, y) in test_points {
            let iters = cactus.iterate(x, y, &params, 256);
            // Just verify it doesn't panic and returns valid iteration count
            assert!(iters <= 256);
        }
    }

    #[test]
    fn test_cactus_escape_radius() {
        // Test that escape radius calculation doesn't panic
        let z1 = Complex64::new(0.0, 0.0);
        let z2 = Complex64::new(1.0, 1.0);
        let z3 = Complex64::new(10.0, 10.0);
        
        let r1 = Cactus::escape_radius(z1);
        let r2 = Cactus::escape_radius(z2);
        let r3 = Cactus::escape_radius(z3);
        
        assert!(r1 > 0.0);
        assert!(r2 > 0.0);
        assert!(r3 > 0.0);
        
        // Larger z0 should generally have larger escape radius
        assert!(r3 > r1);
    }

    #[test]
    fn test_cactus_name() {
        let cactus = Cactus::new();
        assert_eq!(cactus.name(), "Cactus");
    }

    #[test]
    fn test_cactus_parameters() {
        let cactus = Cactus::new();
        let params = cactus.parameters();
        assert_eq!(params.len(), 0, "Cactus fractal should have no parameters");
    }

    #[test]
    fn test_cactus_default_view() {
        let cactus = Cactus::new();
        let view = cactus.default_view(1280, 720);
        assert_eq!(view.width, 1280);
        assert_eq!(view.height, 720);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.6);
    }

    // ── Hi-precision tests ────────────────────────────────────────────────

    fn to_bf(re: f64, im: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        (BigFloat::from_f64(re, p), BigFloat::from_f64(im, p))
    }

    #[test]
    fn hiprec_supports_flag() {
        assert!(Cactus::new().supports_hiprec());
    }

    /// A point far from origin should escape quickly at every bit-width.
    #[test]
    fn hiprec_far_point_escapes() {
        let f = Cactus::new();
        let params = HashMap::new();
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(10.0, 10.0, bits);
            let iters = f.iterate_hiprec(&cr, &ci, &params, 100, bits);
            assert!(iters < 10,
                "bits={}: far point (10,10) should escape quickly, got {}", bits, iters);
        }
    }

    /// Origin (z_0 = 0) is inside the Cactus set — should reach max_iter.
    /// At z_0=0: z_n+1 = z_n^3 + (0-1)*z_n - 0 = z_n^3 - z_n.  Starting at 0: z_1=0, stays 0.
    #[test]
    fn hiprec_origin_in_set() {
        let f = Cactus::new();
        let params = HashMap::new();
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(0.0, 0.0, bits);
            let iters = f.iterate_hiprec(&cr, &ci, &params, 100, bits);
            assert_eq!(iters, 100,
                "bits={}: origin should not escape (in set)", bits);
        }
    }

    /// Hi-prec and f64 must agree on unambiguous exterior/interior points.
    #[test]
    fn hiprec_agrees_with_f64() {
        let f = Cactus::new();
        let params = HashMap::new();
        // Use clearly exterior and clearly interior points at low zoom (no precision difference)
        let cases: &[(f64, f64)] = &[
            (10.0, 10.0),  // clearly outside
            (5.0,  0.0),   // clearly outside
            (0.0,  0.0),   // interior (stays at 0)
            (0.2,  0.1),   // moderate point — check agreement
        ];
        for &(re, im) in cases {
            let f64_iters = f.iterate(re, im, &params, 100);
            let (cr, ci) = to_bf(re, im, 128);
            let hp_iters = f.iterate_hiprec(&cr, &ci, &params, 100, 128);
            assert_eq!(f64_iters, hp_iters,
                "f64 and hiprec disagree at ({re},{im}): f64={f64_iters} hiprec={hp_iters}");
        }
    }

    /// Smoke test: all bit widths run without panicking on a typical point.
    #[test]
    fn hiprec_smoke_all_bitwidths() {
        let f = Cactus::new();
        let params = HashMap::new();
        let (re, im) = (0.5, 0.3);
        let f64_result = f.iterate(re, im, &params, 256);
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(re, im, bits);
            let hp = f.iterate_hiprec(&cr, &ci, &params, 256, bits);
            // Must return a valid iteration count (not panic)
            assert!(hp <= 256, "bits={}: invalid iteration count {}", bits, hp);
            // All bit-widths should agree with f64 on this non-boundary point
            assert_eq!(hp, f64_result,
                "bits={}: hiprec={} but f64={} for ({re},{im})", bits, hp, f64_result);
        }
    }
}
