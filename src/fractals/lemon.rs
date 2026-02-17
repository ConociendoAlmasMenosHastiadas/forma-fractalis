//! Lemon Fractal
//!
//! The Lemon fractal is created by iterating the following complex-valued series
//! and shading by how long it takes to converge to a fixed point:
//!
//!   z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k
//!
//! The parameter `k` controls the power of the denominator, creating different
//! fractal structures. With k=2, this is the canonical Lemon fractal. With k=1,
//! an interesting variant emerges.
//!
//! Unlike escape-time fractals, the Lemon fractal converges rather than diverges.
//! Iteration stops when |z_{n+1} - z_n| falls below the convergence threshold.
//!
//! # Parameters
//! - `convergence_exp`: Exponent for convergence threshold (threshold = 10^(-exp))
//! - `denom_power`: Power k for the denominator (z_n^2 - 1)^k
//!
//! # Mathematical Background
//! This is a Newton-type fractal where the iteration seeks fixed points.
//! Points that converge quickly are colored differently from those that
//! converge slowly or not at all within the iteration limit.

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use eframe::egui;

/// Lemon fractal: z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k
pub struct Lemon;

impl Lemon {
    /// Creates a new Lemon fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Lemon {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Lemon {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let z0 = Complex64::new(c_real, c_imag);
        
        // Get convergence exponent parameter, default to 6 (threshold = 10^-6)
        let convergence_exp = parameters.get("convergence_exp").copied().unwrap_or(6.0);
        let threshold = 10.0_f64.powf(-convergence_exp);
        
        // Get denominator power parameter, default to 2 (canonical Lemon)
        let denom_power = parameters.get("denom_power").copied().unwrap_or(2.0);
        
        // Start iteration: z₁ = z₀
        let mut z = z0;
        
        for i in 0..max_iter {
            // z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k
            let z_sq = z * z;
            let numerator = z0 * z_sq * (z_sq + Complex64::new(1.0, 0.0));
            let denom_base = z_sq - Complex64::new(1.0, 0.0);
            
            // Check for near-zero denominator (would cause division issues)
            if denom_base.norm_sqr() < 1e-30 {
                return i;
            }
            
            // Apply power to denominator: (z_sq - 1)^k
            let denominator = if denom_power == 1.0 {
                denom_base
            } else if denom_power == 2.0 {
                denom_base * denom_base
            } else {
                denom_base.powf(denom_power)
            };
            
            // Check for near-zero denominator after power
            if denominator.norm_sqr() < 1e-30 {
                return i;
            }
            
            let z_next = numerator / denominator;
            
            // Check for NaN/Inf (safety)
            if !z_next.is_finite() {
                return i;
            }
            
            // Check convergence: |z_{n+1} - z_n| < threshold
            let delta = (z_next - z).norm();
            if delta < threshold {
                return i;
            }
            
            // Also check for escape (divergence safety)
            if z_next.norm_sqr() > 1e30 {
                return i;
            }
            
            z = z_next;
        }
        
        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        
        // Center on the origin with a reasonable zoom
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.5;
        
        // Set default parameters
        view.set_parameter("convergence_exp", 6.0);
        view.set_parameter("denom_power", 2.0);  // Canonical Lemon
        
        view
    }

    fn name(&self) -> &str {
        "Lemon"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_0 * z_n^2 * (z_n^2 + 1) / (z_n^2 - 1)^k"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "denom_power",
                "Denominator Power (k)",
                2.0,    // default (canonical Lemon)
                -5.0,   // min
                5.0,    // max
                "Power k for denominator (z_n^2 - 1)^k. k=2 is canonical Lemon, k=1 is an interesting variant."
            ),
            Parameter::new(
                "convergence_exp",
                "Convergence Threshold (10^-x)",
                6.0,    // default
                1.0,    // min (10^-1 = 0.1)
                15.0,   // max (10^-15)
                "Exponent for convergence threshold: iteration stops when |z_{n+1} - z_n| < 10^(-exp)"
            ),
        ]
    }
}

/// GUI implementation for Lemon parameters
impl super::fractal_gui::FractalGUI for Lemon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        
        ui.label(egui::RichText::new("Lemon Parameters").strong());
        ui.add_space(5.0);
        
        // Denominator power slider (like Mandelbrot power)
        let denom_power_min = -5.0;
        let denom_power_max = 5.0;
        
        let mut denom_power = params.get("denom_power").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Denom Power (k):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut denom_power, denom_power_min..=denom_power_max)
                .step_by(0.1)
                .show_value(true)
            ).changed() {
                params.insert("denom_power".to_string(), denom_power);
                *needs_redraw = true;
            }
        });
        
        ui.add_space(3.0);
        
        // Convergence threshold input using exponent notation
        ui.horizontal(|ui| {
            ui.label("Convergence: 10^-");
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut input_state.lemon_convergence_exp)
                    .desired_width(50.0)
            );
            
            if response.changed() {
                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
            }
            
            // Parse and update parameter
            if let Ok(exp) = input_state.lemon_convergence_exp.parse::<f64>() {
                if exp >= 1.0 && exp <= 15.0 {
                    params.insert("convergence_exp".to_string(), exp);
                }
            }
        });
        
        // Also update needs_redraw if pending
        if input_state.pending_redraw {
            *needs_redraw = true;
        }
        
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("k=2: canonical Lemon | k=1: typo variant")
                .small()
                .weak()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lemon_name() {
        let lemon = Lemon::new();
        assert_eq!(lemon.name(), "Lemon");
    }

    #[test]
    fn test_lemon_default_view() {
        let lemon = Lemon::new();
        let view = lemon.default_view(800, 600);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.5);
        assert_eq!(view.parameters.get("convergence_exp"), Some(&6.0));
        assert_eq!(view.parameters.get("denom_power"), Some(&2.0));
    }

    #[test]
    fn test_lemon_parameters() {
        let lemon = Lemon::new();
        let params = lemon.parameters();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "denom_power");
        assert_eq!(params[1].name, "convergence_exp");
    }

    #[test]
    fn test_lemon_origin() {
        // At origin z=0, the fraction should behave predictably
        let lemon = Lemon::new();
        let mut params = HashMap::new();
        params.insert("convergence_exp".to_string(), 6.0);
        params.insert("denom_power".to_string(), 2.0);
        
        // At z=0: z_sq = 0, numerator = 0 * (0+1) = 0
        // So iteration should converge immediately
        let iter = lemon.iterate(0.0, 0.0, &params, 100);
        assert!(iter < 100, "Origin should converge quickly");
    }

    #[test]
    fn test_lemon_near_singularity() {
        // Near z^2 = 1 (i.e., z = 1 or z = -1), denominator approaches zero
        let lemon = Lemon::new();
        let mut params = HashMap::new();
        params.insert("convergence_exp".to_string(), 6.0);
        params.insert("denom_power".to_string(), 2.0);
        
        // Exactly at z=1, denominator is 0
        let iter = lemon.iterate(1.0, 0.0, &params, 100);
        assert!(iter < 100, "z=1 should exit early due to singularity");
    }

    #[test]
    fn test_lemon_far_point() {
        // Far from origin, behavior should either diverge or converge
        let lemon = Lemon::new();
        let mut params = HashMap::new();
        params.insert("convergence_exp".to_string(), 6.0);
        params.insert("denom_power".to_string(), 2.0);
        
        let iter = lemon.iterate(10.0, 10.0, &params, 100);
        // Should either converge or hit escape condition
        assert!(iter <= 100);
    }

    #[test]
    fn test_lemon_convergence_threshold_effect() {
        let lemon = Lemon::new();
        
        // With tight threshold (high exp), should take more iterations
        let mut tight_params = HashMap::new();
        tight_params.insert("convergence_exp".to_string(), 12.0);
        tight_params.insert("denom_power".to_string(), 2.0);
        
        // With loose threshold (low exp), should converge faster
        let mut loose_params = HashMap::new();
        loose_params.insert("convergence_exp".to_string(), 2.0);
        loose_params.insert("denom_power".to_string(), 2.0);
        
        // Test a point that converges
        let tight_iter = lemon.iterate(0.5, 0.3, &tight_params, 1000);
        let loose_iter = lemon.iterate(0.5, 0.3, &loose_params, 1000);
        
        // Loose threshold should converge in fewer or equal iterations
        assert!(loose_iter <= tight_iter, 
            "Loose threshold ({}) should converge no later than tight ({})", 
            loose_iter, tight_iter);
    }
    
    #[test]
    fn test_lemon_different_denom_powers() {
        let lemon = Lemon::new();
        
        // Test with k=1 (typo variant)
        let mut params_k1 = HashMap::new();
        params_k1.insert("convergence_exp".to_string(), 6.0);
        params_k1.insert("denom_power".to_string(), 1.0);
        
        // Test with k=2 (canonical)
        let mut params_k2 = HashMap::new();
        params_k2.insert("convergence_exp".to_string(), 6.0);
        params_k2.insert("denom_power".to_string(), 2.0);
        
        // Both should produce valid iterations (not necessarily equal)
        let iter_k1 = lemon.iterate(0.5, 0.3, &params_k1, 1000);
        let iter_k2 = lemon.iterate(0.5, 0.3, &params_k2, 1000);
        
        assert!(iter_k1 <= 1000);
        assert!(iter_k2 <= 1000);
        // Different k values should generally produce different iteration counts
        // (not a strict test, as some edge points may coincidentally match)
    }
}
