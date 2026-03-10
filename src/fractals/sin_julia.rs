//! Sin Julia Set Implementation
//!
//! The Sin Julia set: z(n+1) = c * sin(z(n))
//! where z starts at the pixel coordinate and c is a constant parameter.
//! 
//! This fractal applies the complex sine function at each iteration,
//! producing unique patterns different from polynomial Julia sets.
//!
//! Simplified iteration formula:
//! x_{n+1} = sin(x_n) * cosh(y_n)
//! y_{n+1} = cos(x_n) * sinh(y_n)
//! z_n = x_n + i*y_n
//!
//! Reference: https://paulbourke.net/fractals/sinjulia/

use super::{Fractal, FractalView, Parameter};
use std::collections::HashMap;
use eframe::egui;

/// Sin Julia set fractal with configurable constant
pub struct SinJulia;

impl SinJulia {
    /// Creates a new Sin Julia set fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for SinJulia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for SinJulia {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        // Get Sin Julia set constant from parameters
        let sin_julia_c_real = parameters.get("c_real").copied().unwrap_or(1.0);
        let sin_julia_c_imag = parameters.get("c_imag").copied().unwrap_or(0.1);
        let escape_radius = parameters.get("escape_radius").copied().unwrap_or(50.0);
        let escape_radius_sq = escape_radius * escape_radius;
        
        // z starts at the pixel coordinate
        let mut x = c_real;
        let mut y = c_imag;
        let mut iter = 0;

        while iter < max_iter {
            // Check for escape
            if x * x + y * y > escape_radius_sq {
                break;
            }

            // Compute sin(z) = sin(x + iy)
            // sin(x + iy) = sin(x)cosh(y) + i*cos(x)sinh(y)
            let sin_x = x.sin();
            let cos_x = x.cos();
            let sinh_y = y.sinh();
            let cosh_y = y.cosh();
            
            let sin_z_real = sin_x * cosh_y;
            let sin_z_imag = cos_x * sinh_y;
            
            // Multiply by c: z_{n+1} = c * sin(z_n)
            let new_x = sin_julia_c_real * sin_z_real - sin_julia_c_imag * sin_z_imag;
            let new_y = sin_julia_c_real * sin_z_imag + sin_julia_c_imag * sin_z_real;
            
            x = new_x;
            y = new_y;
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = 0.0;
        view.center_y = 0.0;
        // Sin Julia sets typically need a wider view
        view.zoom = 0.4;
        
        // Set interesting default constant
        view.set_parameter("c_real", 1.0);
        view.set_parameter("c_imag", 0.1);
        view.set_parameter("escape_radius", 50.0);
        
        view
    }

    fn name(&self) -> &str {
        "Sin Julia"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = c * sin(z_n)"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                1.0,
                -2.0,
                2.0,
                "Real component of the Sin Julia set constant"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.1,
                -2.0,
                2.0,
                "Imaginary component of the Sin Julia set constant"
            ),
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                50.0,
                4.0,
                200.0,
                "Escape boundary - higher values reveal more detail"
            ),
        ]
    }
}

/// GUI implementation for Sin Julia set parameters
impl super::fractal_gui::FractalGUI for SinJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut crate::app_state::InputState,
        needs_redraw: &mut bool,
    ) {
        use super::fractal_gui::trigger_debounced_redraw;
        
        ui.label(egui::RichText::new("Sin Julia Parameters").strong());
        ui.add_space(5.0);
        
        // Coordinate mode toggle
        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.sin_julia_coord_mode, crate::app_state::CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.sin_julia_coord_mode, crate::app_state::CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });
        
        ui.add_space(8.0);
        
        match input_state.sin_julia_coord_mode {
            crate::app_state::CoordinateMode::Rectangular => {
                // Rectangular mode: Real and Imaginary sliders
                let mut c_real = params.get("c_real").copied().unwrap_or(1.0);
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
                        input_state.sin_julia_c_real = format!("{:.15}", c_real);
                        *needs_redraw = true;
                    }
                });
                
                let mut c_imag = params.get("c_imag").copied().unwrap_or(0.1);
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
                        input_state.sin_julia_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });
                
                // High precision text inputs
                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.sin_julia_c_real).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(val) = input_state.sin_julia_c_real.parse::<f64>() {
                                let clamped = val.clamp(-2.0, 2.0);
                                params.insert("c_real".to_string(), clamped);
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.sin_julia_c_imag).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(val) = input_state.sin_julia_c_imag.parse::<f64>() {
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
                let c_real = params.get("c_real").copied().unwrap_or(1.0);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.1);
                
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
                    input_state.sin_julia_c_real = format!("{:.15}", new_real);
                    input_state.sin_julia_c_imag = format!("{:.15}", new_imag);
                    input_state.sin_julia_magnitude = format!("{:.15}", magnitude);
                    input_state.sin_julia_angle = format!("{:.15}", angle);
                    *needs_redraw = true;
                }
                
                // High precision text inputs for polar
                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.sin_julia_magnitude).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(mag) = input_state.sin_julia_magnitude.parse::<f64>() {
                                let clamped_mag = mag.clamp(0.0, 3.0);
                                if let Ok(ang) = input_state.sin_julia_angle.parse::<f64>() {
                                    let new_real = clamped_mag * ang.cos();
                                    let new_imag = clamped_mag * ang.sin();
                                    params.insert("c_real".to_string(), new_real);
                                    params.insert("c_imag".to_string(), new_imag);
                                    input_state.sin_julia_c_real = format!("{:.15}", new_real);
                                    input_state.sin_julia_c_imag = format!("{:.15}", new_imag);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui
                            .add(egui::TextEdit::singleline(&mut input_state.sin_julia_angle).desired_width(150.0))
                            .changed()
                        {
                            if let Ok(ang) = input_state.sin_julia_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.sin_julia_magnitude.parse::<f64>() {
                                    let clamped_mag = mag.clamp(0.0, 3.0);
                                    let new_real = clamped_mag * ang.cos();
                                    let new_imag = clamped_mag * ang.sin();
                                    params.insert("c_real".to_string(), new_real);
                                    params.insert("c_imag".to_string(), new_imag);
                                    input_state.sin_julia_c_real = format!("{:.15}", new_real);
                                    input_state.sin_julia_c_imag = format!("{:.15}", new_imag);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                });
            }
        }
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);
        
        // Escape radius slider
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(50.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 4.0..=200.0)
                .text("")
                .step_by(1.0)
                .fixed_decimals(1)
                .logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.sin_julia_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });

        // Precise text input for escape radius
        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Escape Radius:");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.sin_julia_escape_radius).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.sin_julia_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
            ui.label(egui::RichText::new(
                "Range: slider [4.0, 200.0], text input: any positive value"
            ).small().weak());
        });
        
        ui.add_space(10.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sin_julia_basic_iteration() {
        let fractal = SinJulia::new();
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), 1.0);
        params.insert("c_imag".to_string(), 0.1);
        
        // Test some basic coordinates
        let iter = fractal.iterate(0.0, 0.0, &params, 256);
        assert!(iter > 0 && iter <= 256, "Should produce valid iteration count");
        
        // Test a point that should escape quickly with large coordinates
        let iter_far = fractal.iterate(5.0, 5.0, &params, 1000);
        assert!(iter_far < 1000, "Far points should escape for default parameters");
    }
    
    #[test]
    fn test_sin_julia_name() {
        let fractal = SinJulia::new();
        assert_eq!(fractal.name(), "Sin Julia");
    }
    
    #[test]
    fn test_sin_julia_parameters() {
        let fractal = SinJulia::new();
        let params = fractal.parameters();
        assert_eq!(params.len(), 3);
        assert!(params.iter().any(|p| p.name == "c_real"));
        assert!(params.iter().any(|p| p.name == "c_imag"));
        assert!(params.iter().any(|p| p.name == "escape_radius"));
    }
}
