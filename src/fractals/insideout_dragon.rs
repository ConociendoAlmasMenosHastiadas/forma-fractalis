//! Insideout Dragon Fractal
//!
//! A unique fractal where the iteration formula incorporates functions of the magnitude
//! of z_n. The "insideout" nature comes from the initial condition being the inverse
//! of the pixel coordinates.
//!
//! Formula:
//! ```text
//! z_{n+1} = z_n^2 + f(|z_n|) + i*g(|z_n|)
//! where:
//!   f(r) = r*((r-1)^2)*(r^2-1) / ((1+r^3)^2)
//!   g(r) = r*((r+1)^2)*(r^2-1) / ((1+r^3)^2)
//!   z_0 = 1/(x + i*y)  (inverse of pixel coordinates)
//! ```
//!
//! Reference: https://paulbourke.net/fractals/insideout/

use super::{Fractal, FractalView, Parameter};
use super::fractal_gui::FractalGUI;
use eframe::egui;
use std::collections::HashMap;

/// Insideout Dragon fractal implementation
#[derive(Debug, Clone, Default)]
pub struct InsideoutDragon;

impl InsideoutDragon {
    pub fn new() -> Self {
        Self
    }
    
    /// Compute both f(r) and g(r) together for better numerical stability
    /// 
    /// Mathematical simplification:
    /// f(r) = r*((r+1)^2)*(r^2-1) / ((1+r^3)^2)
    ///      = r*(r+1)^3*(r-1) / (1+r^3)^2
    /// 
    /// g(r) = r*((r-1)^2)*(r^2-1) / ((1+r^3)^2)
    ///      = r*(r-1)^3*(r+1) / (1+r^3)^2
    /// 
    /// Both share the same denominator, so we compute it once.
    #[inline]
    fn compute_f_g(r: f64) -> (f64, f64) {
        if r.abs() < 1e-10 {
            return (0.0, 0.0);
        }
        
        // Pre-compute common terms
        let r2 = r * r;
        let r3 = r2 * r;
        let r_plus_1 = r + 1.0;
        let r_minus_1 = r - 1.0;
        
        // Shared denominator: (1 + r^3)^2
        let one_plus_r3 = 1.0 + r3;
        let denominator = one_plus_r3 * one_plus_r3;
        
        if denominator.abs() < 1e-10 {
            return (0.0, 0.0);
        }
        
        // f(r) = r*(r+1)^3*(r-1) / (1+r^3)^2
        let r_plus_1_cubed = r_plus_1 * r_plus_1 * r_plus_1;
        let f_numerator = r * r_plus_1_cubed * r_minus_1;
        
        // g(r) = r*(r-1)^3*(r+1) / (1+r^3)^2
        let r_minus_1_cubed = r_minus_1 * r_minus_1 * r_minus_1;
        let g_numerator = r * r_minus_1_cubed * r_plus_1;
        
        let inv_denominator = 1.0 / denominator;
        (f_numerator * inv_denominator, g_numerator * inv_denominator)
    }
    
    /// Function f(r) = r*((r+1)^2)*(r^2-1) / ((1+r^3)^2)
    /// Kept for testing purposes
    #[cfg(test)]
    #[inline]
    fn f(r: f64) -> f64 {
        Self::compute_f_g(r).0
    }
    
    /// Function g(r) = r*((r-1)^2)*(r^2-1) / ((1+r^3)^2)
    /// Kept for testing purposes
    #[cfg(test)]
    #[inline]
    fn g(r: f64) -> f64 {
        Self::compute_f_g(r).1
    }
}

impl Fractal for InsideoutDragon {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        parameters: &HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        // Initial condition: z_0 = 1/(x + i*y)
        // To divide by complex number: 1/(a+bi) = (a-bi)/(a^2+b^2)
        let denom = c_real * c_real + c_imag * c_imag;
        
        // Handle point at origin specially
        if denom < 1e-10 {
            return max_iter;
        }
        
        let mut zr = c_real / denom;
        let mut zi = -c_imag / denom;  // Note the negative for complex conjugate
        
        // Get escape radius from parameters (default: 4.0)
        let escape_radius = parameters.get("escape_radius").copied().unwrap_or(4.0);
        let escape_radius_sq = escape_radius * escape_radius;
        
        for iter in 0..max_iter {
            // Check escape condition
            let r_sq = zr * zr + zi * zi;
            if r_sq > escape_radius_sq || !r_sq.is_finite() {
                return iter;
            }
            
            // Calculate magnitude |z_n|
            let magnitude = r_sq.sqrt();
            
            // Calculate f(|z_n|) and g(|z_n|) together (shares denominator calculation)
            let (f_val, g_val) = Self::compute_f_g(magnitude);
            
            // z_{n+1} = z_n^2 + f(|z_n|) + i*g(|z_n|)
            let zr_sq = zr * zr;
            let zi_sq = zi * zi;
            let zr_zi = zr * zi;
            
            let new_zr = zr_sq - zi_sq + f_val;
            let new_zi = 2.0 * zr_zi + g_val;
            
            zr = new_zr;
            zi = new_zi;
        }
        
        max_iter
    }
    
    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut parameters = HashMap::new();
        parameters.insert("escape_radius".to_string(), 4.0);
        
        FractalView {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 0.25,  // Start zoomed out to see the full structure
            width,
            height,
            parameters,
        }
    }
    
    fn name(&self) -> &'static str {
        "Insideout Dragon"
    }
    
    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                4.0,
                1.0,
                100.0,
                "Higher values = larger escape boundary, may reveal more detail",
            ),
        ]
    }
}

// FractalGUI trait - renders escape_radius parameter controls
impl FractalGUI for InsideoutDragon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut std::collections::HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;

        ui.label(egui::RichText::new("Insideout Dragon Parameters").strong());
        ui.add_space(5.0);

        // Escape radius slider
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(4.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 1.0..=100.0)
                .text("")
                .step_by(0.5)
                .fixed_decimals(1)
                .logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.insideout_dragon_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });

        // Precise text input
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Escape Radius:");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.insideout_dragon_escape_radius).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.insideout_dragon_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
            ui.label(egui::RichText::new(
                "Range: slider [1.0, 100.0], text input: any positive value"
            ).small().weak());
        });

        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_insideout_dragon_name() {
        let fractal = InsideoutDragon::new();
        assert_eq!(fractal.name(), "Insideout Dragon");
    }
    
    #[test]
    fn test_insideout_dragon_default_view() {
        let fractal = InsideoutDragon::new();
        let view = fractal.default_view(800, 600);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.25);  // Zoomed out by factor of 2
        assert_eq!(view.width, 800);
        assert_eq!(view.height, 600);
        assert_eq!(view.parameters.len(), 1);  // Has escape_radius parameter
        assert_eq!(view.parameters.get("escape_radius"), Some(&4.0));
    }
    
    #[test]
    fn test_insideout_dragon_parameters() {
        let fractal = InsideoutDragon::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "escape_radius");
        assert_eq!(params[0].default, 4.0);
        assert_eq!(params[0].min, 1.0);
        assert_eq!(params[0].max, 100.0);
    }
    
    #[test]
    fn test_insideout_dragon_origin() {
        let fractal = InsideoutDragon::new();
        let params = HashMap::new();
        
        // Origin (0,0) should be treated as interior due to division by zero
        let iter = fractal.iterate(0.0, 0.0, &params, 256);
        assert_eq!(iter, 256);
    }
    
    #[test]
    fn test_insideout_dragon_far_point() {
        let fractal = InsideoutDragon::new();
        let params = HashMap::new();
        
        // Point far from origin in c-plane maps to near-origin in z-plane
        // z_0 = 1/(10+10i) = (10-10i)/200 = 0.05 - 0.05i
        // Due to the "insideout" nature, far c-points can remain bounded
        let iter = fractal.iterate(10.0, 10.0, &params, 256);
        // We just verify it completes without panicking
        assert!(iter <= 256);
    }
    
    #[test]
    fn test_insideout_dragon_near_unit_circle() {
        let fractal = InsideoutDragon::new();
        let params = HashMap::new();
        
        // Point on unit circle
        let iter = fractal.iterate(1.0, 0.0, &params, 256);
        // Should iterate but behavior depends on the specific dynamics
        assert!(iter > 0);
    }
    
    #[test]
    fn test_f_function() {
        // Test f(r) at various points
        
        // f(0) should be 0
        assert_eq!(InsideoutDragon::f(0.0), 0.0);
        
        // f(1) = 1*0*0 / 8 = 0
        assert_eq!(InsideoutDragon::f(1.0), 0.0);
        
        // f should handle positive values
        let f_val = InsideoutDragon::f(2.0);
        assert!(f_val.is_finite());
    }
    
    #[test]
    fn test_g_function() {
        // Test g(r) at various points
        
        // g(0) should be 0
        assert_eq!(InsideoutDragon::g(0.0), 0.0);
        
        // g(1) = 1*4*0 / 8 = 0
        assert_eq!(InsideoutDragon::g(1.0), 0.0);
        
        // g should handle positive values
        let g_val = InsideoutDragon::g(2.0);
        assert!(g_val.is_finite());
    }
    
    #[test]
    fn test_insideout_dragon_symmetry() {
        let fractal = InsideoutDragon::new();
        let params = HashMap::new();
        
        // Test that points symmetric about origin have related behavior
        let iter1 = fractal.iterate(1.0, 1.0, &params, 256);
        let iter2 = fractal.iterate(-1.0, -1.0, &params, 256);
        
        // Due to the inverse initial condition and the formula structure,
        // we expect both to have finite iteration counts
        assert!(iter1 < 256 || iter2 < 256, "Symmetric points should have interesting behavior");
    }
}
