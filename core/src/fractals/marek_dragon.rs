//! Marek Dragon Fractal
//!
//! The Marek Dragon is a fractal based on the iteration formula:
//! z_{n+1} = exp(jφ) * z_n + z_n^2
//!
//! Where:
//! - φ (phi) is a rotation parameter (0 to 2π)
//! - j is the imaginary unit
//! - The rotation creates fascinating dragon-like structures
//!
//! Reference: https://paulbourke.net/fractals/marek/

use crate::fractals::{Fractal, FractalView};
use crate::number_utils::TWO_PI;
use astro_float::{BigFloat, Consts, RoundingMode};
use num_complex::Complex64;
use std::collections::HashMap;

/// Marek Dragon fractal
///
/// Formula: z_{n+1} = exp(jφ) * z_n + z_n^2
pub struct MarekDragon;

impl MarekDragon {
    pub fn new() -> Self {
        MarekDragon
    }
}

impl Default for MarekDragon {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for MarekDragon {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let phi = parameters.get("phi").copied().unwrap_or(0.0);
        
        // Calculate rotation factor: exp(j*phi) = cos(phi) + j*sin(phi)
        let rotation = Complex64::new(phi.cos(), phi.sin());
        
        let mut z = Complex64::new(c_real, c_imag);
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_radius_sq = escape_r * escape_r;
        
        for i in 0..max_iter {
            // Check escape condition
            if z.norm_sqr() > escape_radius_sq {
                return i;
            }
            
            // z_{n+1} = exp(jφ) * z_n + z_n^2
            z = rotation * z + z * z;
        }
        
        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.8;
        view
    }

    fn name(&self) -> &str {
        "Marek Dragon"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = exp(iφ)*z_n + z_n^2"
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
        let phi = parameters.get("phi").copied().unwrap_or(0.0);
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);

        let p = bits as usize;
        let rm = RoundingMode::ToEven;
        let mut cc = Consts::new().expect("BigFloat constants cache");

        let phi_bf = BigFloat::from_f64(phi, p);
        let rotation_re = phi_bf.cos(p, rm, &mut cc);
        let rotation_im = phi_bf.sin(p, rm, &mut cc);
        let escape_sq = BigFloat::from_f64(escape_r * escape_r, p);
        let two = BigFloat::from_f64(2.0, p);

        let mut zr = c_real.clone();
        let mut zi = c_imag.clone();

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);
            let norm_sq = zr2.add(&zi2, p, rm);

            if norm_sq.cmp(&escape_sq).map_or(false, |value| value > 0) {
                return iter;
            }

            let rot_zr = rotation_re.mul(&zr, p, rm).sub(&rotation_im.mul(&zi, p, rm), p, rm);
            let rot_zi = rotation_re.mul(&zi, p, rm).add(&rotation_im.mul(&zr, p, rm), p, rm);

            let square_zr = zr2.sub(&zi2, p, rm);
            let square_zi = two.mul(&zr, p, rm).mul(&zi, p, rm);

            let new_zr = rot_zr.add(&square_zr, p, rm);
            let new_zi = rot_zi.add(&square_zi, p, rm);

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }

    fn parameters(&self) -> Vec<crate::fractals::Parameter> {
        vec![
            crate::fractals::Parameter {
                name: "phi".to_string(),
                label: "Phi (φ)".to_string(),
                default: 0.0,
                min: 0.0,
                max: TWO_PI,
                description: "Rotation parameter (0 to 2π)".to_string(),
            },
            crate::fractals::Parameter {
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

    #[test]
    fn test_marek_dragon_name() {
        let fractal = MarekDragon::new();
        assert_eq!(fractal.name(), "Marek Dragon");
    }

    #[test]
    fn test_marek_dragon_parameters() {
        let fractal = MarekDragon::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "phi");
        assert_eq!(params[0].default, 0.0);
        assert_eq!(params[0].min, 0.0);
        assert!(params[0].max > 6.0); // Should be TWO_PI ≈ 6.28
        assert_eq!(params[1].name, "escape_radius");
        assert_eq!(params[1].default, 2.0);
    }

    #[test]
    fn test_marek_dragon_default_view() {
        let fractal = MarekDragon::new();
        let view = fractal.default_view(1280, 720);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.8);
    }

    #[test]
    fn test_marek_dragon_origin() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 0.0);
        
        // At origin with phi=0, z stays at 0 (never escapes)
        let iters = fractal.iterate(0.0, 0.0, &params, 100);
        assert_eq!(iters, 100); // Should not escape
    }

    #[test]
    fn test_marek_dragon_far_point() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 1.0);
        
        // Points far from origin should escape quickly
        let iters = fractal.iterate(10.0, 10.0, &params, 100);
        assert!(iters < 5);
    }

    #[test]
    fn test_marek_dragon_various_phi() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        
        // Test different phi values produce different results
        params.insert("phi".to_string(), 0.0);
        let iters1 = fractal.iterate(0.5, 0.5, &params, 100);
        
        params.insert("phi".to_string(), std::f64::consts::PI / 2.0);
        let iters2 = fractal.iterate(0.5, 0.5, &params, 100);
        
        params.insert("phi".to_string(), std::f64::consts::PI);
        let iters3 = fractal.iterate(0.5, 0.5, &params, 100);
        
        // Different phi values should produce different iteration counts
        // (at least some should be different)
        assert!(iters1 != iters2 || iters2 != iters3 || iters1 != iters3);
    }

    #[test]
    fn test_marek_dragon_interior_point() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 0.5);
        
        // Test a point that might be in the set
        let iters = fractal.iterate(0.0, 0.0, &params, 256);
        // Just verify it runs without panicking
        assert!(iters <= 256);
    }

    #[test]
    fn test_marek_dragon_supports_hiprec() {
        assert!(MarekDragon::new().supports_hiprec());
    }

    #[test]
    fn test_marek_dragon_hiprec_interior_point() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 0.5);

        let (cr, ci) = to_bf(0.0, 0.0, 128);
        assert_eq!(fractal.iterate_hiprec(&cr, &ci, &params, 100, 128), 100);
    }

    #[test]
    fn test_marek_dragon_hiprec_escaping_point() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 1.0);

        let (cr, ci) = to_bf(10.0, 10.0, 128);
        assert!(fractal.iterate_hiprec(&cr, &ci, &params, 100, 128) < 5);
    }

    #[test]
    fn test_marek_dragon_hiprec_matches_f64_on_robust_points() {
        let fractal = MarekDragon::new();
        let cases = [
            (0.0, 0.0, 0.0, 100),
            (10.0, 10.0, 1.0, 100),
            (0.5, 0.5, 0.0, 100),
            (0.5, 0.5, std::f64::consts::PI / 2.0, 100),
        ];

        for (re, im, phi, max_iter) in cases {
            let mut params = HashMap::new();
            params.insert("phi".to_string(), phi);

            let f64_iters = fractal.iterate(re, im, &params, max_iter);
            let (cr, ci) = to_bf(re, im, 128);
            let hiprec_iters = fractal.iterate_hiprec(&cr, &ci, &params, max_iter, 128);

            assert_eq!(
                f64_iters, hiprec_iters,
                "f64 and hi-prec disagree at ({re}, {im}) with phi={phi}: f64={f64_iters}, hiprec={hiprec_iters}"
            );
        }
    }

    #[test]
    fn test_marek_dragon_hiprec_bit_widths_smoke() {
        let fractal = MarekDragon::new();
        let mut params = HashMap::new();
        params.insert("phi".to_string(), 1.0);

        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(10.0, 10.0, bits);
            assert!(fractal.iterate_hiprec(&cr, &ci, &params, 100, bits) < 5);
        }
    }
}
