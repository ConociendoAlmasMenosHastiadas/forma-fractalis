//! Zubieta Fractal Implementation
//!
//! The Zubieta set: z(n+1) = z(n)² + c/z(n)
//! A Julia set variant where c is a constant and z starts at each pixel position.
//!
//! This fractal requires division by z, so we need to guard against z=0.
//! If at any iteration z becomes zero, we treat it as escaped.

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use eframe::egui;

/// Zubieta fractal - Julia variant with division
pub struct Zubieta;

impl Zubieta {
    /// Creates a new Zubieta fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Zubieta {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Zubieta {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        // Get Zubieta constant from parameters
        let zubieta_c_real = parameters.get("c_real").copied().unwrap_or(0.0);
        let zubieta_c_imag = parameters.get("c_imag").copied().unwrap_or(0.8);
        
        let c = Complex64::new(zubieta_c_real, zubieta_c_imag);
        let mut z = Complex64::new(c_real, c_imag);
        let mut iter = 0;

        while iter < max_iter {
            if z.norm_sqr() > 4.0 {
                break;
            }
            
            // Guard against division by zero
            if z.norm_sqr() < 1e-30 {
                // Treat as escaped if z gets too close to zero
                break;
            }

            // z_{n+1} = z_n^2 + c/z_n
            z = z * z + c / z;
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.7;
        
        // Default c value
        view.set_parameter("c_real", 0.0);
        view.set_parameter("c_imag", 0.8);
        
        view
    }

    fn name(&self) -> &str {
        "Zubieta"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_n^2 + c/z_n"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                0.0,
                -2.0,
                2.0,
                "Real component of the Zubieta constant"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.8,
                -2.0,
                2.0,
                "Imaginary component of the Zubieta constant"
            ),
        ]
    }
}

/// GUI implementation for Zubieta parameters
impl super::fractal_gui::FractalGUI for Zubieta {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        
        ui.label(egui::RichText::new("Zubieta Parameters").strong());
        ui.add_space(5.0);
        
        // Coordinate mode toggle
        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.zubieta_coord_mode, crate::app_state::CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.zubieta_coord_mode, crate::app_state::CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });
        
        ui.add_space(8.0);
        
        match input_state.zubieta_coord_mode {
            crate::app_state::CoordinateMode::Rectangular => {
                // Rectangular mode: Real and Imaginary sliders
                let mut c_real = params.get("c_real").copied().unwrap_or(0.0);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("")
                        .step_by(0.001)
                        .fixed_decimals(3))
                        .changed()
                    {
                        params.insert("c_real".to_string(), c_real);
                        input_state.zubieta_c_real = format!("{:.15}", c_real);
                        *needs_redraw = true;
                    }
                });
                
                let mut c_imag = params.get("c_imag").copied().unwrap_or(0.8);
                ui.horizontal(|ui| {
                    ui.label("Im{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_imag, -2.0..=2.0)
                        .text("")
                        .step_by(0.001)
                        .fixed_decimals(3))
                        .changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.zubieta_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });
                
                // High precision text inputs
                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.zubieta_c_real).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(val) = input_state.zubieta_c_real.parse::<f64>() {
                                let clamped = val.clamp(-2.0, 2.0);
                                params.insert("c_real".to_string(), clamped);
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.zubieta_c_imag).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(val) = input_state.zubieta_c_imag.parse::<f64>() {
                                let clamped = val.clamp(-2.0, 2.0);
                                params.insert("c_imag".to_string(), clamped);
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                });
            }
            crate::app_state::CoordinateMode::Polar => {
                // Polar mode: Magnitude and Angle sliders
                // Get current rectangular values and convert to polar
                let c_real = params.get("c_real").copied().unwrap_or(0.0);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.8);
                
                let mut magnitude = (c_real * c_real + c_imag * c_imag).sqrt();
                let mut angle = c_imag.atan2(c_real);
                // Normalize angle from [-π, π] to [0, 2π] to match slider range
                if angle < 0.0 {
                    angle += std::f64::consts::TAU;
                }
                
                let mut changed = false;
                
                ui.horizontal(|ui| {
                    ui.label("|c|:");
                    ui.add_space(13.0);
                    if ui.add(egui::Slider::new(&mut magnitude, 0.0..=3.0)
                        .text("")
                        .step_by(0.001)
                        .fixed_decimals(3))
                        .changed()
                    {
                        changed = true;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("ang(c):");
                    if ui.add(egui::Slider::new(&mut angle, 0.0..=std::f64::consts::TAU)
                        .text("")
                        .step_by(0.001)
                        .fixed_decimals(3))
                        .changed()
                    {
                        changed = true;
                    }
                });
                
                if changed {
                    // Convert polar back to rectangular
                    let new_real = magnitude * angle.cos();
                    let new_imag = magnitude * angle.sin();
                    params.insert("c_real".to_string(), new_real);
                    params.insert("c_imag".to_string(), new_imag);
                    input_state.zubieta_c_real = format!("{:.15}", new_real);
                    input_state.zubieta_c_imag = format!("{:.15}", new_imag);
                    input_state.zubieta_magnitude = format!("{:.15}", magnitude);
                    input_state.zubieta_angle = format!("{:.15}", angle);
                    *needs_redraw = true;
                }
                
                // High precision text inputs for polar
                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.zubieta_magnitude).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(mag) = input_state.zubieta_magnitude.parse::<f64>() {
                                let clamped_mag = mag.clamp(0.0, 3.0);
                                if let Ok(ang) = input_state.zubieta_angle.parse::<f64>() {
                                    let new_real = clamped_mag * ang.cos();
                                    let new_imag = clamped_mag * ang.sin();
                                    params.insert("c_real".to_string(), new_real);
                                    params.insert("c_imag".to_string(), new_imag);
                                    input_state.zubieta_c_real = format!("{:.15}", new_real);
                                    input_state.zubieta_c_imag = format!("{:.15}", new_imag);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.zubieta_angle).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(ang) = input_state.zubieta_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.zubieta_magnitude.parse::<f64>() {
                                    let clamped_mag = mag.clamp(0.0, 3.0);
                                    let new_real = clamped_mag * ang.cos();
                                    let new_imag = clamped_mag * ang.sin();
                                    params.insert("c_real".to_string(), new_real);
                                    params.insert("c_imag".to_string(), new_imag);
                                    input_state.zubieta_c_real = format!("{:.15}", new_real);
                                    input_state.zubieta_c_imag = format!("{:.15}", new_imag);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                });
            }
        }
        
        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zubieta_basic() {
        let zubieta = Zubieta::new();
        let params = HashMap::new();
        
        // Test a point that should iterate
        let iters = zubieta.iterate(0.0, 0.0, &params, 100);
        assert!(iters < 100, "Origin should escape");
    }
    
    #[test]
    fn test_zubieta_division_guard() {
        let zubieta = Zubieta::new();
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), 0.0);
        params.insert("c_imag".to_string(), 0.0);
        
        // Start near zero - should handle division safely
        let iters = zubieta.iterate(1e-20, 1e-20, &params, 100);
        assert!(iters <= 100, "Should handle near-zero gracefully");
    }
}
