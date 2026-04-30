//! Lace Julia Fractal Implementation
//!
//! Formula: z_{n+1} = (i*z_n^{-3} + 1010) / (c*i*z_n^{-6} + 3301*z_n)
//!
//! Equivalently (multiplying numerator and denominator by z_n^6):
//!   z_{n+1} = (i*z_n^3 + 1010*z_n^6) / (c*i + 3301*z_n^7)
//!
//! z_0 is the pixel coordinate; c is a free complex parameter.
//! Colored by escape velocity (iterations to |z| > escape_radius).
//!
//! Guards:
//! - z near 0 (pole in original formula): treated as escaped
//! - denominator near 0: treated as escaped

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use astro_float::{BigFloat, RoundingMode};

pub struct LaceJulia;

impl LaceJulia {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LaceJulia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for LaceJulia {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let cr = parameters.get("c_real").copied().unwrap_or(0.0);
        let ci = parameters.get("c_imag").copied().unwrap_or(0.5);
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_sq = escape_r * escape_r;

        // c * i = (-ci + cr*i)
        let c_times_i = Complex64::new(-ci, cr);

        let mut z = Complex64::new(c_real, c_imag);

        for iter in 0..max_iter {
            let norm_sq = z.norm_sqr();

            // Guard: z too close to 0 (pole)
            if norm_sq < 1e-30 {
                return iter;
            }

            if norm_sq > escape_sq {
                return iter;
            }

            let z3 = z * z * z;
            let z6 = z3 * z3;
            let z7 = z6 * z;

            // Numerator: i*z^3 + 1010*z^6
            // i * z3 = Complex(-z3.im, z3.re)
            let i_z3 = Complex64::new(-z3.im, z3.re);
            let numer = i_z3 + 1010.0 * z6;

            // Denominator: c*i + 3301*z^7
            let denom = c_times_i + 3301.0 * z7;

            let denom_sq = denom.norm_sqr();
            if denom_sq < 1e-30 {
                return iter;
            }

            let z_next = numer / denom;

            if !z_next.re.is_finite() || !z_next.im.is_finite() {
                return iter;
            }

            z = z_next;
        }

        max_iter
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
        let cr_f64 = parameters.get("c_real").copied().unwrap_or(0.0);
        let ci_f64 = parameters.get("c_imag").copied().unwrap_or(0.5);
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);

        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        // Fixed c parameter and precomputed c*i = (-ci, cr)
        let cr_bf = BigFloat::from_f64(cr_f64, p);
        let ci_bf = BigFloat::from_f64(ci_f64, p);
        let neg_ci = ci_bf.neg();
        // c_times_i = (-ci_bf, cr_bf)

        let escape_sq_bf = BigFloat::from_f64(escape_r * escape_r, p);
        let zero_guard   = BigFloat::from_f64(1e-30, p);

        let k1010 = BigFloat::from_f64(1010.0, p);
        let k3301 = BigFloat::from_f64(3301.0, p);

        // z starts at pixel coordinate
        let mut zr = c_real.clone();
        let mut zi = c_imag.clone();

        // Helper: complex multiply (ar,ai)*(br,bi) = (ar*br - ai*bi, ar*bi + ai*br)
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

            // Guard: near zero
            if norm_sq.cmp(&zero_guard).map_or(true, |v| v < 0) {
                return iter;
            }
            // Escape check
            if norm_sq.cmp(&escape_sq_bf).map_or(false, |v| v > 0) {
                return iter;
            }

            // z^2
            let (z2r, z2i) = cmul!(&zr, &zi, &zr, &zi);
            // z^3 = z^2 * z
            let (z3r, z3i) = cmul!(&z2r, &z2i, &zr, &zi);
            // z^6 = z^3 * z^3
            let (z6r, z6i) = cmul!(&z3r, &z3i, &z3r, &z3i);
            // z^7 = z^6 * z
            let (z7r, z7i) = cmul!(&z6r, &z6i, &zr, &zi);

            // Numerator: i*z3 + 1010*z6
            // i*(z3r, z3i) = (-z3i, z3r)
            let i_z3r = z3i.neg();
            let i_z3i = z3r.clone();
            let nr = i_z3r.add(&k1010.mul(&z6r, p, rm), p, rm);
            let ni = i_z3i.add(&k1010.mul(&z6i, p, rm), p, rm);

            // Denominator: c*i + 3301*z7
            // c*i = (-ci, cr)
            let dr = neg_ci.add(&k3301.mul(&z7r, p, rm), p, rm);
            let di = cr_bf.add(&k3301.mul(&z7i, p, rm), p, rm);

            // Guard: denom near zero
            let denom_sq = dr.mul(&dr, p, rm).add(&di.mul(&di, p, rm), p, rm);
            if denom_sq.cmp(&zero_guard).map_or(true, |v| v < 0) {
                return iter;
            }

            // z_next = (nr + ni*i) / (dr + di*i)
            // = ((nr*dr + ni*di) / denom_sq, (ni*dr - nr*di) / denom_sq)
            let new_zr = nr.mul(&dr, p, rm).add(&ni.mul(&di, p, rm), p, rm);
            let new_zi = ni.mul(&dr, p, rm).sub(&nr.mul(&di, p, rm), p, rm);
            let new_zr = new_zr.div(&denom_sq, p, rm);
            let new_zi = new_zi.div(&denom_sq, p, rm);

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.7;
        view.set_parameter("c_real", 0.0);
        view.set_parameter("c_imag", 0.5);
        view.set_parameter("escape_radius", 2.0);
        view
    }

    fn name(&self) -> &str {
        "Lace Julia"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = (i*z_n^3 + 1010*z_n^6) / (c*i + 3301*z_n^7)"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                0.0,
                -2.0,
                2.0,
                "Real component of the Julia constant c"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.5,
                -2.0,
                2.0,
                "Imaginary component of the Julia constant c"
            ),
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                2.0,
                0.5,
                100.0,
                "Escape radius: iteration escapes when |z| > escape_radius"
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(cr: f64, ci: f64) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("c_real".to_string(), cr);
        p.insert("c_imag".to_string(), ci);
        p
    }

    #[test]
    fn test_lace_julia_name() {
        assert_eq!(LaceJulia::new().name(), "Lace Julia");
    }

    #[test]
    fn test_lace_julia_parameters() {
        let params = LaceJulia::new().parameters();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "c_real");
        assert_eq!(params[1].name, "c_imag");
        assert_eq!(params[2].name, "escape_radius");
    }

    #[test]
    fn test_lace_julia_z_zero_guarded() {
        // z=0 is a pole; iteration should terminate early
        let fractal = LaceJulia::new();
        let params = make_params(0.0, 0.5);
        let iters = fractal.iterate(0.0, 0.0, &params, 100);
        assert!(iters < 100, "z=0 should terminate early (pole)");
    }

    #[test]
    fn test_lace_julia_returns_less_than_max_for_escaping_point() {
        let fractal = LaceJulia::new();
        let params = make_params(0.0, 0.5);
        // Try several points; at least some should escape
        let mut any_escaped = false;
        for i in 0..20 {
            let x = -1.5 + i as f64 * 0.15;
            let iters = fractal.iterate(x, 0.3, &params, 200);
            if iters < 200 {
                any_escaped = true;
                break;
            }
        }
        assert!(any_escaped, "Some points should escape");
    }

    #[test]
    fn test_lace_julia_max_iter_not_zero() {
        let fractal = LaceJulia::new();
        let params = make_params(0.0, 0.5);
        // With 0 max_iter the loop body never runs
        let iters = fractal.iterate(0.5, 0.5, &params, 0);
        assert_eq!(iters, 0);
    }

    #[test]
    fn test_lace_julia_hiprec_agrees_with_f64() {
        let fractal = LaceJulia::new();
        let params = make_params(0.0, 0.5);
        // Test a non-singular, non-zero point at each bit width
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let iters_f64 = fractal.iterate(0.7, 0.3, &params, 200);
            let iters_hi = fractal.iterate_hiprec(
                &BigFloat::from_f64(0.7, bits as usize),
                &BigFloat::from_f64(0.3, bits as usize),
                &params,
                200,
                bits,
            );
            // Allow ±2 difference due to f32/f64/BigFloat rounding on boundary
            let diff = (iters_f64 as i64 - iters_hi as i64).unsigned_abs();
            assert!(diff <= 2,
                "bits={}: f64={} hiprec={} differ by {}", bits, iters_f64, iters_hi, diff);
        }
    }
}
