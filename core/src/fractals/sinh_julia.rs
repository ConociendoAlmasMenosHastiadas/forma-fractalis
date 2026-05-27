//! Sinh Julia fractal implementation.
//!
//! Formula: z_{n+1} = abs(sinh(z_n)^4) + c
//! where abs() is applied independently to the real and imaginary components.
//! z_0 is the pixel coordinate and c is a fixed complex parameter.
//!
//! Reference: https://paulbourke.net/fractals/sinh/

use super::{Fractal, FractalView, Parameter};
use astro_float::{BigFloat, Consts, RoundingMode};
use num_complex::Complex64;
use std::collections::HashMap;

pub struct SinhJulia;

impl SinhJulia {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SinhJulia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for SinhJulia {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let const_real = parameters.get("c_real").copied().unwrap_or(-0.7);
        let const_imag = parameters.get("c_imag").copied().unwrap_or(0.27015);
        let escape_radius = parameters.get("escape_radius").copied().unwrap_or(50.0);
        let escape_radius_sq = escape_radius * escape_radius;

        let c = Complex64::new(const_real, const_imag);
        let mut z = Complex64::new(c_real, c_imag);

        for iter in 0..max_iter {
            if z.norm_sqr() > escape_radius_sq {
                return iter;
            }

            // sinh(x + iy) = sinh(x)cos(y) + i*cosh(x)sin(y)
            let sinh_z = Complex64::new(z.re.sinh() * z.im.cos(), z.re.cosh() * z.im.sin());
            let sinh_sq = sinh_z * sinh_z;
            let sinh_fourth = sinh_sq * sinh_sq;

            let z_next = Complex64::new(sinh_fourth.re.abs(), sinh_fourth.im.abs()) + c;
            if !z_next.re.is_finite() || !z_next.im.is_finite() {
                return iter;
            }

            z = z_next;
        }

        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.7;
        view.set_parameter("c_real", -0.7);
        view.set_parameter("c_imag", 0.27015);
        view.set_parameter("escape_radius", 50.0);
        view
    }

    fn name(&self) -> &str {
        "Sinh Julia"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = |Re(sinh(z_n)^4)| + i|Im(sinh(z_n)^4)| + c"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                -0.7,
                -2.0,
                2.0,
                "Real component of the fixed Julia constant c"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.27015,
                -2.0,
                2.0,
                "Imaginary component of the fixed Julia constant c"
            ),
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                50.0,
                4.0,
                200.0,
                "Escape radius: iteration escapes when |z| > escape_radius"
            ),
        ]
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
        let const_real = parameters.get("c_real").copied().unwrap_or(-0.7);
        let const_imag = parameters.get("c_imag").copied().unwrap_or(0.27015);
        let escape_radius = parameters.get("escape_radius").copied().unwrap_or(50.0);

        let p = bits as usize;
        let rm = RoundingMode::ToEven;
        let zero = BigFloat::from_f64(0.0, p);
        let const_re = BigFloat::from_f64(const_real, p);
        let const_im = BigFloat::from_f64(const_imag, p);
        let escape_sq = BigFloat::from_f64(escape_radius * escape_radius, p);
        let mut cc = Consts::new().expect("BigFloat constants cache");

        let mut zr = c_real.clone();
        let mut zi = c_imag.clone();

        macro_rules! cmul {
            ($ar:expr, $ai:expr, $br:expr, $bi:expr) => {{
                let rr = $ar.mul($br, p, rm).sub(&$ai.mul($bi, p, rm), p, rm);
                let ri = $ar.mul($bi, p, rm).add(&$ai.mul($br, p, rm), p, rm);
                (rr, ri)
            }};
        }

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);
            let norm_sq = zr2.add(&zi2, p, rm);
            if norm_sq.cmp(&escape_sq).map_or(false, |v| v > 0) {
                return iter;
            }

            // sinh(x + iy) = sinh(x)cos(y) + i*cosh(x)sin(y)
            let sinh_zr = zr.sinh(p, rm, &mut cc);
            let cos_zi = zi.cos(p, rm, &mut cc);
            let cosh_zr = zr.cosh(p, rm, &mut cc);
            let sin_zi = zi.sin(p, rm, &mut cc);

            let sinh_re = sinh_zr.mul(&cos_zi, p, rm);
            let sinh_im = cosh_zr.mul(&sin_zi, p, rm);

            let (sq_re, sq_im) = cmul!(&sinh_re, &sinh_im, &sinh_re, &sinh_im);
            let (pow4_re, pow4_im) = cmul!(&sq_re, &sq_im, &sq_re, &sq_im);

            let abs_pow4_re = if pow4_re.is_negative() {
                zero.sub(&pow4_re, p, rm)
            } else {
                pow4_re
            };
            let abs_pow4_im = if pow4_im.is_negative() {
                zero.sub(&pow4_im, p, rm)
            } else {
                pow4_im
            };

            let new_zr = abs_pow4_re.add(&const_re, p, rm);
            let new_zi = abs_pow4_im.add(&const_im, p, rm);

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

    fn to_bf(re: f64, im: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        (BigFloat::from_f64(re, p), BigFloat::from_f64(im, p))
    }

    fn deterministic_params() -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), 0.0);
        params.insert("c_imag".to_string(), 0.0);
        params.insert("escape_radius".to_string(), 50.0);
        params
    }

    #[test]
    fn test_sinh_julia_name() {
        let fractal = SinhJulia::new();
        assert_eq!(fractal.name(), "Sinh Julia");
    }

    #[test]
    fn test_sinh_julia_parameters() {
        let fractal = SinhJulia::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 3);
        assert!(params.iter().any(|p| p.name == "c_real"));
        assert!(params.iter().any(|p| p.name == "c_imag"));
        assert!(params.iter().any(|p| p.name == "escape_radius"));
    }

    #[test]
    fn origin_stays_in_set_when_c_is_zero() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        assert_eq!(fractal.iterate(0.0, 0.0, &params, 100), 100);
    }

    #[test]
    fn large_real_point_escapes_quickly() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        assert!(fractal.iterate(3.0, 0.0, &params, 20) < 20);
    }

    #[test]
    fn hiprec_origin_stays_in_set_when_c_is_zero() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        let (cr, ci) = to_bf(0.0, 0.0, 128);
        assert_eq!(fractal.iterate_hiprec(&cr, &ci, &params, 100, 128), 100);
    }

    #[test]
    fn hiprec_large_real_point_escapes_quickly() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        let (cr, ci) = to_bf(3.0, 0.0, 128);
        assert!(fractal.iterate_hiprec(&cr, &ci, &params, 20, 128) < 20);
    }

    #[test]
    fn hiprec_matches_f64_on_robust_points() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        for &(re, im) in &[(0.0, 0.0), (3.0, 0.0), (0.0, 3.0)] {
            let f64_iters = fractal.iterate(re, im, &params, 20);
            let (cr, ci) = to_bf(re, im, 128);
            let hp_iters = fractal.iterate_hiprec(&cr, &ci, &params, 20, 128);
            assert_eq!(f64_iters, hp_iters, "f64 and hi-prec disagree at ({re},{im})");
        }
    }

    #[test]
    fn hiprec_bit_widths_smoke() {
        let fractal = SinhJulia::new();
        let params = deterministic_params();
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(3.0, 0.0, bits);
            assert!(fractal.iterate_hiprec(&cr, &ci, &params, 20, bits) < 20);
        }
    }
}