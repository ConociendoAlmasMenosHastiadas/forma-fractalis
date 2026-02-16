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
use num_complex::Complex64;
use std::collections::HashMap;
use eframe::egui;

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

/// GUI implementation for Tetration threshold and escape mode parameters
impl super::fractal_gui::FractalGUI for Tetration {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        use super::parameter_types::EscapeMode;
        
        ui.label(egui::RichText::new("Tetration Parameters").strong());
        ui.add_space(5.0);
        
        // Escape Mode radio buttons
        ui.label("Escape Criterion:");
        ui.add_space(3.0);
        
        let mut current_mode = EscapeMode::from_f64(params.get("escape_mode").copied().unwrap_or(0.0));
        let mut changed = false;
        
        ui.horizontal(|ui| {
            for mode in EscapeMode::all() {
                if ui.radio(current_mode == mode, mode.display_name()).clicked() {
                    current_mode = mode;
                    changed = true;
                }
            }
        });
        
        if changed {
            params.insert("escape_mode".to_string(), current_mode.to_f64());
            *needs_redraw = true;
        }
        
        ui.add_space(8.0);
        
        // Threshold slider (logarithmic scale)  
        ui.label("Escape Threshold:");
        ui.add_space(3.0);
        
        // Get current threshold value (default 1e7)
        let mut threshold = params.get("threshold").copied().unwrap_or(1e7);
        
        // Use logarithmic scale: log10(threshold) ranges from 1 to 10
        let mut log_threshold = threshold.log10();
        
        ui.horizontal(|ui| {
            ui.label("10^");
            if ui.add(egui::Slider::new(&mut log_threshold, 1.0..=10.0)
                .text("")
                .step_by(0.1)
                .fixed_decimals(1))
                .changed()
            {
                threshold = 10_f64.powf(log_threshold);
                params.insert("threshold".to_string(), threshold);
                // Update input state with scientific notation
                input_state.tetration_threshold = format!("{:.2e}", threshold);
                *needs_redraw = true;
            }
            ui.label(format!("≈ {:.2e}", threshold));
        });
        
        // Show current value as editable text input (supports scientific notation)
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Threshold:");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.tetration_threshold).desired_width(120.0))
                    .changed()
                {
                    // Parse scientific notation (supports 1e7, 10e6, 1.5e8, etc.)
                    if let Ok(val) = input_state.tetration_threshold.parse::<f64>() {
                        if val > 0.0 && val.is_finite() {
                            params.insert("threshold".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
            ui.label(egui::RichText::new(
                "Supports scientific notation: 1e7, 10e6, 1.5e8, etc."
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(view.zoom, 0.5);
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
}
