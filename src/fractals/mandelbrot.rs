//! Mandelbrot Set Implementation
//!
//! The classic Mandelbrot set: z(n+1) = z(n)^power + c
//! where z starts at 0 and c is the point being tested.
//!
//! When power = 2, this is the classic Mandelbrot set.
//! Other powers create "Multibrot" or "Powerbrot" sets with different symmetries:
//! - power = 3: Tricorn symmetry
//! - power = 4: Quatric symmetry
//! - power = 5: Quintic symmetry

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use crate::number_utils::ABSOLUTE_EPSILON;
use eframe::egui;
use astro_float::{BigFloat, Consts, RoundingMode};

/// Mandelbrot set fractal with configurable power
pub struct Mandelbrot;

impl Mandelbrot {
    /// Creates a new Mandelbrot fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Mandelbrot {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        
        // Get power parameter, default to 2 (classic Mandelbrot)
        let power = parameters.get("power").copied().unwrap_or(2.0);

        // Start at z₁ = c instead of z₀ = 0 to avoid singularity with negative powers
        // This is mathematically equivalent since z₁ = 0^power + c = c
        let mut z = c;
        let mut iter = 0;
        
        let is_negative_power = power < 0.0;
        let escape_radius_sqr = 4.0;
        let convergence_threshold_sqr = ABSOLUTE_EPSILON; // For detecting convergence to 0

        while iter < max_iter {
            let magnitude_sqr = z.norm_sqr();
            
            // Check for escape or convergence based on power sign
            if is_negative_power {
                // For negative powers: check if converging to zero or diverging
                if magnitude_sqr < convergence_threshold_sqr || magnitude_sqr > escape_radius_sqr {
                    break;
                }
            } else {
                // For positive powers: standard escape condition
                if magnitude_sqr > escape_radius_sqr {
                    break;
                }
            }

            // z = z^power + c
            // Optimize for power=2.0 (classic Mandelbrot) - use multiplication instead of powf
            z = if power == 2.0 {
                z * z + c
            } else {
                z.powf(power) + c
            };
            
            // Safety check: if z becomes NaN or infinite, bail out
            if !z.re.is_finite() || !z.im.is_finite() {
                break;
            }
            
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = -0.5;
        view.center_y = 0.0;
        view.zoom = 1.0;
        view
    }

    fn name(&self) -> &str {
        "Mandelbrot"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_n^p + c"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "power".to_string(),
                label: "Power".to_string(),
                default: 2.0,
                min: -10.0,
                max: 10.0,
                description: "Exponent in z^power + c formula. 2 = classic Mandelbrot. Negative powers create inverted sets.".to_string(),
            },
        ]
    }

    fn supports_hiprec(&self) -> bool {
        true
    }

    /// High-precision Mandelbrot iteration using software floating-point arithmetic.
    ///
    /// Supports all power values using BigFloat arithmetic:
    /// - power=2: optimized direct complex multiplication
    /// - other powers: polar form z^p = r^p * (cos(p*theta) + i*sin(p*theta))
    ///
    /// Receives coordinates as `BigFloat` so that per-pixel precision is preserved
    /// even at extreme zoom levels (>1e15).
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
        let power = parameters.get("power").copied().unwrap_or(2.0);

        let p = bits as usize;
        let rm = RoundingMode::ToEven;

        let cr = c_real.clone();
        let ci = c_imag.clone();

        // Start z at c (same convention as f64 path: z_1 = c)
        let mut zr = cr.clone();
        let mut zi = ci.clone();

        let four = BigFloat::from_f64(4.0, p);
        let two  = BigFloat::from_f64(2.0, p);
        let is_negative_power = power < 0.0;
        let epsilon_bf = BigFloat::from_f64(ABSOLUTE_EPSILON, p);

        // Consts cache needed for transcendental functions (non-power-2)
        let mut cc = if power != 2.0 {
            Some(Consts::new().expect("BigFloat constants cache"))
        } else {
            None
        };

        for iter in 0..max_iter {
            let zr2 = zr.mul(&zr, p, rm);
            let zi2 = zi.mul(&zi, p, rm);

            // Escape check: |z|^2 > 4, plus convergence check for negative powers
            let norm_sq = zr2.add(&zi2, p, rm);
            if is_negative_power {
                let converged = norm_sq.cmp(&epsilon_bf).map_or(false, |v| v < 0);
                let escaped = norm_sq.cmp(&four).map_or(false, |v| v > 0);
                if converged || escaped {
                    return iter;
                }
            } else if norm_sq.cmp(&four).map_or(false, |v| v > 0) {
                return iter;
            }

            // z = z^power + c
            let (new_zr, new_zi) = if power == 2.0 {
                // Optimized: z^2 = zr^2 - zi^2 + cr, 2*zr*zi + ci
                let nr = zr2.sub(&zi2, p, rm).add(&cr, p, rm);
                let ni = two.mul(&zr, p, rm).mul(&zi, p, rm).add(&ci, p, rm);
                (nr, ni)
            } else {
                // General power via polar form
                let (pzr, pzi) = Self::complex_powf_bf(
                    &zr, &zi, power, p, rm, cc.as_mut().unwrap(),
                );
                (pzr.add(&cr, p, rm), pzi.add(&ci, p, rm))
            };

            // Safety: bail out on NaN/Inf
            if new_zr.is_nan() || new_zr.is_inf() || new_zi.is_nan() || new_zi.is_inf() {
                return iter;
            }

            zr = new_zr;
            zi = new_zi;
        }

        max_iter
    }
}

impl Mandelbrot {
    /// BigFloat atan2(y, x) — full four-quadrant arctangent.
    fn bf_atan2(
        y: &BigFloat, x: &BigFloat,
        p: usize, rm: RoundingMode, cc: &mut Consts,
    ) -> BigFloat {
        if x.is_zero() && y.is_zero() {
            return BigFloat::new(p); // 0
        }
        // pi = acos(-1)
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
    ///
    /// z = r * e^(i*theta)  =>  z^p = r^p * (cos(p*theta) + i*sin(p*theta))
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

    /// Returns the sequence of all complex numbers generated during iteration
    /// 
    /// This function performs the same iteration as `iterate()` but instead of
    /// returning the escape count, it returns a Vec of all z values encountered
    /// during the iteration process. Useful for visualizing the iteration path.
    /// 
    /// # Arguments
    /// * `c_real` - Real component of the complex constant c
    /// * `c_imag` - Imaginary component of the complex constant c
    /// * `power` - Exponent in z^power + c formula (default 2.0 for classic Mandelbrot)
    /// * `max_iter` - Maximum number of iterations
    /// 
    /// # Returns
    /// Vec<Complex64> containing all z values from z₁ to the final iteration
    pub fn mandelseries(c_real: f64, c_imag: f64, power: f64, max_iter: u32) -> Vec<Complex64> {
        let c = Complex64::new(c_real, c_imag);
        
        // Start at z₁ = c instead of z₀ = 0 to avoid singularity with negative powers
        let mut z = c;
        let mut series = Vec::with_capacity(max_iter as usize + 1);
        
        // Always include the starting point (z₁ = c)
        series.push(z);
        
        let is_negative_power = power < 0.0;
        let escape_radius_sqr = 4.0;
        let convergence_threshold_sqr = ABSOLUTE_EPSILON;
        
        for _ in 0..max_iter {
            let magnitude_sqr = z.norm_sqr();
            
            // Check for escape or convergence based on power sign
            if is_negative_power {
                if magnitude_sqr < convergence_threshold_sqr || magnitude_sqr > escape_radius_sqr {
                    break;
                }
            } else {
                if magnitude_sqr > escape_radius_sqr {
                    break;
                }
            }
            
            // Optimize for power=2.0 (classic Mandelbrot)
            z = if power == 2.0 {
                z * z + c
            } else {
                z.powf(power) + c
            };
            
            // Safety check for NaN/infinity
            if !z.re.is_finite() || !z.im.is_finite() {
                break;
            }
            
            series.push(z);
        }
        
        series
    }
}

/// GUI implementation for Mandelbrot power parameter
impl super::fractal_gui::FractalGUI for Mandelbrot {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut std::collections::HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        
        ui.label(egui::RichText::new("Mandelbrot Power").strong());
        ui.add_space(5.0);
        
        // Get parameter bounds
        let power_min = -10.0;
        let power_max = 10.0;
        
        // Power slider
        let mut power = params.get("power").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Power:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut power, power_min..=power_max)
                .text("")
                .step_by(0.1)
                .fixed_decimals(1))
                .changed()
            {
                params.insert("power".to_string(), power);
                input_state.mandelbrot_power = format!("{:.1}", power);
                *needs_redraw = true;
            }
        });
        
        // Show current value as editable text input (no clamping - full f64 range)
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Power:");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.mandelbrot_power).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.mandelbrot_power.parse::<f64>() {
                        // No clamping - accept any valid f64 value
                        params.insert("power".to_string(), val);
                        trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new(
                format!("Range: slider [{:.1}, {:.1}], text input: full f64", power_min, power_max)
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params(power: f64) -> HashMap<String, f64> {
        let mut p = HashMap::new();
        p.insert("power".to_string(), power);
        p
    }

    /// Helper: convert f64 pair to BigFloat pair at given precision.
    fn to_bf(re: f64, im: f64, bits: u32) -> (BigFloat, BigFloat) {
        let p = bits as usize;
        (BigFloat::from_f64(re, p), BigFloat::from_f64(im, p))
    }

    /// Origin (0+0i) is in the set — should reach max_iter.
    #[test]
    fn hiprec_origin_in_set() {
        let m = Mandelbrot::new();
        let params = default_params(2.0);
        let (cr, ci) = to_bf(0.0, 0.0, 128);
        let result = m.iterate_hiprec(&cr, &ci, &params, 100, 128);
        assert_eq!(result, 100, "origin should not escape");
    }

    /// A clearly-escaping point (2+2i) should return well below max_iter.
    #[test]
    fn hiprec_far_point_escapes() {
        let m = Mandelbrot::new();
        let params = default_params(2.0);
        let (cr, ci) = to_bf(2.0, 2.0, 128);
        let result = m.iterate_hiprec(&cr, &ci, &params, 100, 128);
        assert!(result < 100, "2+2i should escape immediately");
    }

    /// Hi-prec and f64 should agree on escape counts for points far from the boundary.
    #[test]
    fn hiprec_matches_f64_for_robust_points() {
        let m = Mandelbrot::new();
        let cases: &[(f64, f64)] = &[
            (2.0, 0.0),   // outside set
            (0.0, 2.0),   // outside set
            (-2.0, 0.0),  // boundary
            (0.5, 0.5),   // inside set
        ];
        let params = default_params(2.0);
        for &(re, im) in cases {
            let f64_iters = m.iterate(re, im, &params, 100);
            let (cr, ci) = to_bf(re, im, 128);
            let hp_iters  = m.iterate_hiprec(&cr, &ci, &params, 100, 128);
            assert_eq!(
                f64_iters, hp_iters,
                "f64 and hiprec disagree at ({re},{im}): f64={f64_iters} hiprec={hp_iters}"
            );
        }
    }

    /// Non-power-2 now uses full BigFloat polar form — result should match f64 for robust points.
    #[test]
    fn hiprec_non_power2_fallback() {
        let m = Mandelbrot::new();
        let params = default_params(3.0);
        let f64_result = m.iterate(0.5, 0.3, &params, 100);
        let (cr, ci) = to_bf(0.5, 0.3, 128);
        let hp_result  = m.iterate_hiprec(&cr, &ci, &params, 100, 128);
        assert_eq!(f64_result, hp_result, "power=3 hiprec should agree with f64 for robust point");
    }

    /// Higher bit widths should not panic and should still agree on robust points.
    #[test]
    fn hiprec_bit_widths_smoke() {
        let m = Mandelbrot::new();
        let params = default_params(2.0);
        for &bits in &[64u32, 128, 256, 512, 1024] {
            let (cr, ci) = to_bf(2.0, 0.0, bits);
            let result = m.iterate_hiprec(&cr, &ci, &params, 50, bits);
            assert!(result < 50, "2+0i should escape at {bits} bits");
        }
    }
}
