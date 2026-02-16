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
use num_complex::Complex64;
use std::collections::HashMap;
use eframe::egui;

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
        let escape_radius_sq = 4.0; // Standard escape radius squared
        
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

    fn parameters(&self) -> Vec<crate::fractals::Parameter> {
        vec![
            crate::fractals::Parameter {
                name: "phi".to_string(),
                label: "Phi (φ)".to_string(),
                default: 0.0,
                min: 0.0,
                max: TWO_PI,
                description: "Rotation parameter (0 to 2π)".to_string(),
            }
        ]
    }
}

/// GUI implementation for Marek Dragon phi parameter
impl super::fractal_gui::FractalGUI for MarekDragon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        
        ui.label(egui::RichText::new("Marek Dragon Parameters").strong());
        ui.add_space(5.0);
        
        // Phi slider (rotation angle 0 to 2π)
        let mut phi = params.get("phi").copied().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.label("Phi (φ):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut phi, 0.0..=TWO_PI)
                .text("")
                .step_by(0.001)
                .fixed_decimals(3))
                .changed()
            {
                params.insert("phi".to_string(), phi);
                input_state.marek_dragon_phi = format!("{:.6}", phi);
                *needs_redraw = true;
            }
        });
        
        // Show current value as editable text input
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Phi (φ):");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.marek_dragon_phi).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.marek_dragon_phi.parse::<f64>() {
                        let clamped = val.clamp(0.0, TWO_PI);
                        params.insert("phi".to_string(), clamped);
                        trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new(
                "Formula: z_{n+1} = exp(jφ) · z_n + z_n²"
            ).small().weak());
            ui.label(egui::RichText::new(
                format!("Range: 0 to 2π ({:.6})", TWO_PI)
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marek_dragon_name() {
        let fractal = MarekDragon::new();
        assert_eq!(fractal.name(), "Marek Dragon");
    }

    #[test]
    fn test_marek_dragon_parameters() {
        let fractal = MarekDragon::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "phi");
        assert_eq!(params[0].default, 0.0);
        assert_eq!(params[0].min, 0.0);
        assert!(params[0].max > 6.0); // Should be TWO_PI ≈ 6.28
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
}
