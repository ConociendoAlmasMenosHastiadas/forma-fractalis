//! Julia Set Implementation
//!
//! The generalised Julia set: z(n+1) = z(n)^k + c
//! where z starts at the pixel coordinate, c is a fixed constant, and k is
//! the exponent (default 2 — the classic Julia set).
//!
//! Unlike Mandelbrot (where c varies and z starts at 0),
//! Julia sets use a fixed c and z starts at each pixel position.
//!
//! Reference: https://paulbourke.net/fractals/juliaset/

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use crate::number_utils::ABSOLUTE_EPSILON;
use astro_float::{BigFloat, Consts, RoundingMode};

/// Classic Julia set coordinates that produce beautiful, interesting patterns
/// Format: (c_real, c_imag, name)
const CLASSIC_JULIA_COORDINATES: &[(f64, f64, &str)] = &[
    (-0.7, 0.27015, "Dendrite (Douady's Rabbit)"),
    (-0.4, 0.6, "Spiral"),
    (-0.8, 0.156, "Branching"),
    (0.285, 0.01, "Seahorse Tail"),
    (-0.70176, -0.3842, "Siegel Disk"),
    (0.285, 0.0, "Dragon"),
    (-0.835, -0.2321, "Swirls"),
    (-0.8, 0.156, "Lightning"),
];

/// Julia set fractal with configurable constant
pub struct Julia;

impl Julia {
    /// Creates a new Julia set fractal instance
    pub fn new() -> Self {
        Self
    }
    
    /// Returns a random classic Julia set coordinate for exploration
    /// 
    /// This helps users discover interesting Julia sets without needing
    /// to know specific coordinates in advance.
    pub fn random_classic_coordinates() -> (f64, f64) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Use current time as seed for simple randomization
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        
        let index = seed % CLASSIC_JULIA_COORDINATES.len();
        let (c_real, c_imag, _name) = CLASSIC_JULIA_COORDINATES[index];
        (c_real, c_imag)
    }
}

impl Default for Julia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Julia {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        // Get Julia set constant from parameters (defaults to classic values)
        let julia_c_real = parameters.get("c_real").copied().unwrap_or(-0.7);
        let julia_c_imag = parameters.get("c_imag").copied().unwrap_or(0.27015);
        // Exponent — default 2 (classic Julia). Any real value is valid.
        let power = parameters.get("power").copied().unwrap_or(2.0);
        
        let c = Complex64::new(julia_c_real, julia_c_imag);
        let mut z = Complex64::new(c_real, c_imag);
        let mut iter = 0;
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_sq = escape_r * escape_r;

        while iter < max_iter {
            if z.norm_sqr() > escape_sq {
                break;
            }

            // Optimise the common power=2 case to avoid powf overhead.
            z = if power == 2.0 {
                z * z + c
            } else {
                z.powf(power) + c
            };

            if !z.re.is_finite() || !z.im.is_finite() {
                break;
            }

            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        // Julia sets typically look good at zoom level around 0.7 to show the full set
        // This gives approximately -2 to 2 range in both axes
        view.zoom = 0.7;
        
        // Set random classic Julia constant for discovery
        let (c_real, c_imag) = Self::random_classic_coordinates();
        view.set_parameter("c_real", c_real);
        view.set_parameter("c_imag", c_imag);
        view.set_parameter("power", 2.0);
        
        view
    }

    fn name(&self) -> &str {
        "Julia Set"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_n^k + c"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                -0.7,
                -2.0,
                2.0,
                "Real component of the Julia set constant"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.27015,
                -2.0,
                2.0,
                "Imaginary component of the Julia set constant"
            ),
            Parameter::new(
                "power",
                "Exponent",
                2.0,
                -10.0,
                10.0,
                "Iteration exponent k in z^k + c. 2 = classic Julia set."
            ),
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                2.0,
                0.5,
                100.0,
                "Escape radius: iterate escapes when |z| > escape_radius."
            ),
        ]
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    /// High-precision Julia set iteration using software floating-point arithmetic.
    ///
    /// The argument names follow the `Fractal` trait convention: `c_real`/`c_imag` are
    /// the **pixel coordinates** (starting z). The fixed Julia constant is read from
    /// `parameters["c_real"]` and `parameters["c_imag"]`.
    ///
    /// Supports all power values:
    /// - power=2: optimised direct complex multiplication (no transcendentals)
    /// - other powers: polar form z^p = r^p * (cos(p*theta) + i*sin(p*theta))
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
        let julia_cr_f64 = parameters.get("c_real").copied().unwrap_or(-0.7);
        let julia_ci_f64 = parameters.get("c_imag").copied().unwrap_or(0.27015);
        let power = parameters.get("power").copied().unwrap_or(2.0);

        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        // Julia constant (fixed for this render)
        let const_cr = BigFloat::from_f64(julia_cr_f64, p);
        let const_ci = BigFloat::from_f64(julia_ci_f64, p);

        // z starts at the pixel position
        let mut zr = c_real.clone();
        let mut zi = c_imag.clone();

        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let four = BigFloat::from_f64(escape_r * escape_r, p);
        let two  = BigFloat::from_f64(2.0, p);
        let is_negative_power = power < 0.0;
        let epsilon_bf = BigFloat::from_f64(ABSOLUTE_EPSILON, p);

        let mut cc = if power != 2.0 {
            Some(Consts::new().expect("BigFloat constants cache"))
        } else {
            None
        };

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);

            let norm_sq = zr2.add(&zi2, p, rm);
            if is_negative_power {
                let converged = norm_sq.cmp(&epsilon_bf).map_or(false, |v| v < 0);
                let escaped   = norm_sq.cmp(&four      ).map_or(false, |v| v > 0);
                if converged || escaped {
                    return iter;
                }
            } else if norm_sq.cmp(&four).map_or(false, |v| v > 0) {
                return iter;
            }

            // z = z^power + c  (c is the fixed Julia constant)
            let (new_zr, new_zi) = if power == 2.0 {
                let nr = zr2.sub(&zi2, p, rm).add(&const_cr, p, rm);
                let ni = two.mul(&zr, p, rm).mul(&zi, p, rm).add(&const_ci, p, rm);
                (nr, ni)
            } else {
                let (pzr, pzi) = Self::complex_powf_bf(
                    &zr, &zi, power, p, rm, cc.as_mut().unwrap(),
                );
                (pzr.add(&const_cr, p, rm), pzi.add(&const_ci, p, rm))
            };

            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }
}

impl Julia {
    /// BigFloat atan2(y, x) — full four-quadrant arctangent.
    fn bf_atan2(
        y: &BigFloat, x: &BigFloat,
        p: usize, rm: RoundingMode, cc: &mut Consts,
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

    /// Complex z^power via polar form using BigFloat transcendental functions.
    fn complex_powf_bf(
        zr: &BigFloat, zi: &BigFloat,
        power: f64,
        p: usize, rm: RoundingMode, cc: &mut Consts,
    ) -> (BigFloat, BigFloat) {
        let zr2 = zr.mul(zr, p, rm);
        let zi2 = zi.mul(zi, p, rm);
        let r_sq = zr2.add(&zi2, p, rm);
        if r_sq.is_zero() || r_sq.is_nan() {
            return (BigFloat::new(p), BigFloat::new(p));
        }
        let r = r_sq.sqrt(p, rm);
        let theta = Self::bf_atan2(zi, zr, p, rm, cc);
        let power_bf = BigFloat::from_f64(power, p);
        let r_p = r.pow(&power_bf, p, rm, cc);
        let p_theta = power_bf.mul(&theta, p, rm);
        let cos_pt = p_theta.cos(p, rm, cc);
        let sin_pt = p_theta.sin(p, rm, cc);
        (r_p.mul(&cos_pt, p, rm), r_p.mul(&sin_pt, p, rm))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_bf(re: f64, im: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        (BigFloat::from_f64(re, p), BigFloat::from_f64(im, p))
    }

    fn params(c_real: f64, c_imag: f64, power: f64) -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("c_real".to_string(), c_real);
        m.insert("c_imag".to_string(), c_imag);
        m.insert("power".to_string(), power);
        m
    }

    #[test]
    fn classic_julia_origin_is_in_set() {
        // For c = 0+0i, z_{n+1} = z_n^2. The Julia set is the unit disk.
        // The origin (0+0i) is strictly inside: z stays at 0 forever.
        let julia = Julia::new();
        let p = params(0.0, 0.0, 2.0);
        assert_eq!(julia.iterate(0.0, 0.0, &p, 256), 256);
    }

    #[test]
    fn classic_julia_corner_escapes() {
        // For c = 0+0i, any point with |z| > 2 escapes immediately.
        let julia = Julia::new();
        let p = params(0.0, 0.0, 2.0);
        assert!(julia.iterate(3.0, 0.0, &p, 256) < 256);
    }

    #[test]
    fn power3_produces_different_output() {
        // With k=3, the Julia set has different geometry than k=2.
        // Check that a point inside k=2 set escapes for k=3 (or vice versa).
        let julia = Julia::new();
        let p2 = params(-0.7, 0.27015, 2.0);
        let p3 = params(-0.7, 0.27015, 3.0);
        // At least one pixel should differ between the two power settings.
        let test_points: &[(f64, f64)] = &[(0.5, 0.0), (0.0, 0.5), (0.3, 0.3)];
        let differs = test_points.iter().any(|&(x, y)| {
            julia.iterate(x, y, &p2, 256) != julia.iterate(x, y, &p3, 256)
        });
        assert!(differs, "power=3 should produce different iteration counts than power=2");
    }

    #[test]
    fn default_power_is_two() {
        // iterate() without a "power" parameter should behave like power=2
        let julia = Julia::new();
        let mut no_power = HashMap::new();
        no_power.insert("c_real".to_string(), -0.7);
        no_power.insert("c_imag".to_string(), 0.27015);
        let with_power = params(-0.7, 0.27015, 2.0);
        for &(x, y) in &[(0.0, 0.0), (0.5, 0.5), (1.0, 0.0)] {
            assert_eq!(
                julia.iterate(x, y, &no_power, 256),
                julia.iterate(x, y, &with_power, 256),
                "default power should equal explicit power=2"
            );
        }
    }

    // ── Hi-precision tests ───────────────────────────────────────────────────

    /// For c=0+0i the Julia set is the unit disk.
    /// The origin (z=0) is interior — should reach max_iter.
    #[test]
    fn hiprec_origin_in_unit_disk_julia() {
        let julia = Julia::new();
        let p = params(0.0, 0.0, 2.0);
        let (zr, zi) = to_bf(0.0, 0.0, 128);
        let result = julia.iterate_hiprec(&zr, &zi, &p, 100, 128);
        assert_eq!(result, 100, "origin inside unit-disk Julia (c=0) should not escape");
    }

    /// A point well outside the unit disk (c=0 Julia) should escape.
    #[test]
    fn hiprec_far_point_escapes_julia() {
        let julia = Julia::new();
        let p = params(0.0, 0.0, 2.0);
        let (zr, zi) = to_bf(3.0, 0.0, 128);
        let result = julia.iterate_hiprec(&zr, &zi, &p, 100, 128);
        assert!(result < 100, "3+0i should escape the c=0 Julia set");
    }

    /// Hi-prec and f64 must agree on non-boundary pixels (classic Julia constants).
    #[test]
    fn hiprec_matches_f64_classic_julia() {
        let julia = Julia::new();
        let p = params(-0.7, 0.27015, 2.0);
        // Points clearly inside or clearly outside the filled Julia set
        let test_cases: &[(f64, f64)] = &[
            (3.0, 0.0),   // well outside
            (0.0, 3.0),   // well outside
            (0.0, 0.0),   // near centre
        ];
        for &(re, im) in test_cases {
            let f64_iters = julia.iterate(re, im, &p, 100);
            let (zr, zi) = to_bf(re, im, 128);
            let hp_iters  = julia.iterate_hiprec(&zr, &zi, &p, 100, 128);
            assert_eq!(
                f64_iters, hp_iters,
                "f64 and hi-prec disagree at ({re},{im}): f64={f64_iters} hiprec={hp_iters}"
            );
        }
    }

    /// power=3 hi-prec should agree with f64 on robust points.
    #[test]
    fn hiprec_power3_agrees_with_f64() {
        let julia = Julia::new();
        let p = params(0.0, 0.0, 3.0);
        let (zr, zi) = to_bf(3.0, 0.0, 128);
        let f64_iters = julia.iterate(3.0, 0.0, &p, 100);
        let hp_iters  = julia.iterate_hiprec(&zr, &zi, &p, 100, 128);
        assert_eq!(f64_iters, hp_iters, "power=3 hiprec vs f64 disagree");
    }

    /// Smoke test across all supported bit widths — must not panic.
    #[test]
    fn hiprec_bit_widths_smoke_julia() {
        let julia = Julia::new();
        let p = params(-0.7, 0.27015, 2.0);
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (zr, zi) = to_bf(3.0, 0.0, bits);
            let result = julia.iterate_hiprec(&zr, &zi, &p, 50, bits);
            assert!(result < 50, "3+0i should escape at {bits} bits");
        }
    }
}
