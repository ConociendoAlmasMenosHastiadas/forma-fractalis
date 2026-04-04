//! GUI rendering trait for fractal-specific parameters
//!
//! This module provides the `FractalGUI` trait that allows each fractal type
//! to encapsulate its own parameter rendering logic, avoiding hard-coded
//! conditional sections in the main GUI code.
//!
//! All implementations live here in the GUI crate so that core fractal math
//! files remain free of egui dependencies.

use eframe::egui;
use std::collections::HashMap;
use std::time::Instant;
use crate::app_state::{InputState, CoordinateMode};
use crate::fractals::{
    Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia,
    Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia,
};
use crate::fractals::parameter_types::EscapeMode;

/// Trait for rendering fractal-specific parameter controls
pub trait FractalGUI {
    /// Render the parameter controls for this fractal
    fn render_parameters_gui(
        &self,
        _ui: &mut egui::Ui,
        _params: &mut HashMap<String, f64>,
        _input_state: &mut InputState,
        _needs_redraw: &mut bool,
    ) {
        // Default: no parameters to render
    }
}

/// Helper function to trigger a debounced redraw (for text inputs)
pub fn trigger_debounced_redraw(timer: &mut Option<Instant>, pending: &mut bool) {
    *timer = Some(Instant::now());
    *pending = true;
}

// ── Trivial implementations (no parameters) ───────────────────────────────

impl FractalGUI for BurningShip {}
impl FractalGUI for Cactus {}
impl FractalGUI for TippetsMandelbrot {}

// ── Mandelbrot ─────────────────────────────────────────────────────────────

impl FractalGUI for Mandelbrot {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Mandelbrot Power").strong());
        ui.add_space(5.0);

        let power_min = -10.0;
        let power_max = 10.0;

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

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Power:");
                if ui
                    .add(egui::TextEdit::singleline(&mut input_state.mandelbrot_power).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.mandelbrot_power.parse::<f64>() {
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

// ── Julia ──────────────────────────────────────────────────────────────────

impl FractalGUI for Julia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Julia Set Parameters").strong());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.julia_coord_mode, CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.julia_coord_mode, CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);

        match input_state.julia_coord_mode {
            CoordinateMode::Rectangular => {
                let mut c_real = params.get("c_real").copied().unwrap_or(-0.7);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3))
                        .changed()
                    {
                        params.insert("c_real".to_string(), c_real);
                        input_state.julia_c_real = format!("{:.15}", c_real);
                        *needs_redraw = true;
                    }
                });

                let mut c_imag = params.get("c_imag").copied().unwrap_or(0.27015);
                ui.horizontal(|ui| {
                    ui.label("Im{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_imag, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3))
                        .changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.julia_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.julia_c_real).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.julia_c_real.parse::<f64>() {
                                params.insert("c_real".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.julia_c_imag).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.julia_c_imag.parse::<f64>() {
                                params.insert("c_imag".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                });
            }
            CoordinateMode::Polar => {
                let c_real = params.get("c_real").copied().unwrap_or(-0.7);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.27015);

                let mut magnitude = (c_real * c_real + c_imag * c_imag).sqrt();
                let mut angle = c_imag.atan2(c_real);
                if angle < 0.0 { angle += std::f64::consts::TAU; }

                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("|c|:");
                    ui.add_space(13.0);
                    if ui.add(egui::Slider::new(&mut magnitude, 0.0..=3.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });
                ui.horizontal(|ui| {
                    ui.label("ang(c):");
                    if ui.add(egui::Slider::new(&mut angle, 0.0..=std::f64::consts::TAU)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });

                if changed {
                    let new_real = magnitude * angle.cos();
                    let new_imag = magnitude * angle.sin();
                    params.insert("c_real".to_string(), new_real);
                    params.insert("c_imag".to_string(), new_imag);
                    input_state.julia_c_real = format!("{:.15}", new_real);
                    input_state.julia_c_imag = format!("{:.15}", new_imag);
                    input_state.julia_magnitude = format!("{:.15}", magnitude);
                    input_state.julia_angle = format!("{:.15}", angle);
                    *needs_redraw = true;
                }

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.julia_magnitude).desired_width(150.0)).changed() {
                            if let Ok(mag) = input_state.julia_magnitude.parse::<f64>() {
                                if let Ok(ang) = input_state.julia_angle.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.julia_c_real = format!("{:.15}", r);
                                    input_state.julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.julia_angle).desired_width(150.0)).changed() {
                            if let Ok(ang) = input_state.julia_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.julia_magnitude.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.julia_c_real = format!("{:.15}", r);
                                    input_state.julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                });
            }
        }

        // ── Power (exponent k in z^k + c) ──────────────────────────────────
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Exponent").strong());
        ui.add_space(3.0);

        let power_min = -10.0_f64;
        let power_max = 10.0_f64;
        let mut power = params.get("power").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("k:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut power, power_min..=power_max)
                .text("")
                .step_by(0.1)
                .fixed_decimals(1))
                .changed()
            {
                params.insert("power".to_string(), power);
                input_state.julia_power = format!("{:.1}", power);
                *needs_redraw = true;
            }
        });

        ui.add_space(3.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("k:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.julia_power).desired_width(100.0))
                    .changed()
                {
                    if let Ok(val) = input_state.julia_power.parse::<f64>() {
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

// ── Lemon ──────────────────────────────────────────────────────────────────

impl FractalGUI for Lemon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Lemon Parameters").strong());
        ui.add_space(5.0);

        let denom_power_min = -5.0;
        let denom_power_max = 5.0;

        let mut denom_power = params.get("denom_power").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Denom Power (k):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut denom_power, denom_power_min..=denom_power_max)
                .step_by(0.1).show_value(true)).changed()
            {
                params.insert("denom_power".to_string(), denom_power);
                *needs_redraw = true;
            }
        });

        ui.add_space(3.0);

        ui.horizontal(|ui| {
            ui.label("Convergence: 10^-");
            let response = ui.add(
                egui::TextEdit::singleline(&mut input_state.lemon_convergence_exp).desired_width(50.0)
            );
            if response.changed() {
                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
            }
            if let Ok(exp) = input_state.lemon_convergence_exp.parse::<f64>() {
                if exp >= 1.0 && exp <= 15.0 {
                    params.insert("convergence_exp".to_string(), exp);
                }
            }
        });

        if input_state.pending_redraw {
            *needs_redraw = true;
        }

        ui.add_space(2.0);
        ui.label(egui::RichText::new("k=2: canonical Lemon | k=1: typo variant").small().weak());
    }
}

// ── Marek Dragon ───────────────────────────────────────────────────────────

const TWO_PI: f64 = std::f64::consts::TAU;

impl FractalGUI for MarekDragon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Marek Dragon Parameters").strong());
        ui.add_space(5.0);

        let mut phi = params.get("phi").copied().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.label("Phi (φ):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut phi, 0.0..=TWO_PI)
                .text("").step_by(0.001).fixed_decimals(3))
                .changed()
            {
                params.insert("phi".to_string(), phi);
                input_state.marek_dragon_phi = format!("{:.6}", phi);
                *needs_redraw = true;
            }
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Phi (φ):");
                if ui.add(egui::TextEdit::singleline(&mut input_state.marek_dragon_phi).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.marek_dragon_phi.parse::<f64>() {
                        params.insert("phi".to_string(), val.clamp(0.0, TWO_PI));
                        trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new("Formula: z_{n+1} = exp(jφ) · z_n + z_n²").small().weak());
            ui.label(egui::RichText::new(format!("Range: 0 to 2π ({:.6})", TWO_PI)).small().weak());
        });

        ui.add_space(10.0);
    }
}

// ── Multifractal Julia ─────────────────────────────────────────────────────

impl FractalGUI for MultifractalJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Multifractal-Julia Parameters").strong());
        ui.add_space(5.0);

        let power_min = -5.0;
        let power_max = 5.0;

        let mut power = params.get("power").copied().unwrap_or(1.0);
        ui.horizontal(|ui| {
            ui.label("Power (k):");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut power, power_min..=power_max)
                .text("").step_by(0.1).fixed_decimals(1))
                .changed()
            {
                params.insert("power".to_string(), power);
                input_state.multifractal_julia_power = format!("{:.1}", power);
                *needs_redraw = true;
            }
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Power (k):");
                if ui.add(egui::TextEdit::singleline(&mut input_state.multifractal_julia_power).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.multifractal_julia_power.parse::<f64>() {
                        params.insert("power".to_string(), val);
                        trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                    }
                }
            });
            ui.label(egui::RichText::new("Formula: z_{n+1} = c^k · z_n^{-2} + c").small().weak());
            ui.label(egui::RichText::new(
                format!("Range: slider [{:.1}, {:.1}], text input: full f64", power_min, power_max)
            ).small().weak());
        });

        ui.add_space(10.0);
    }
}

// ── Sin Julia ──────────────────────────────────────────────────────────────

impl FractalGUI for SinJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Sin Julia Parameters").strong());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.sin_julia_coord_mode, CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.sin_julia_coord_mode, CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);

        match input_state.sin_julia_coord_mode {
            CoordinateMode::Rectangular => {
                let mut c_real = params.get("c_real").copied().unwrap_or(1.0);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
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
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.sin_julia_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sin_julia_c_real).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.sin_julia_c_real.parse::<f64>() {
                                params.insert("c_real".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sin_julia_c_imag).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.sin_julia_c_imag.parse::<f64>() {
                                params.insert("c_imag".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                });
            }
            CoordinateMode::Polar => {
                let c_real = params.get("c_real").copied().unwrap_or(1.0);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.1);

                let mut magnitude = (c_real * c_real + c_imag * c_imag).sqrt();
                let mut angle = c_imag.atan2(c_real);
                if angle < 0.0 { angle += std::f64::consts::TAU; }

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("|c|:");
                    ui.add_space(13.0);
                    if ui.add(egui::Slider::new(&mut magnitude, 0.0..=3.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });
                ui.horizontal(|ui| {
                    ui.label("ang(c):");
                    if ui.add(egui::Slider::new(&mut angle, 0.0..=std::f64::consts::TAU)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });

                if changed {
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

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sin_julia_magnitude).desired_width(150.0)).changed() {
                            if let Ok(mag) = input_state.sin_julia_magnitude.parse::<f64>() {
                                if let Ok(ang) = input_state.sin_julia_angle.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.sin_julia_c_real = format!("{:.15}", r);
                                    input_state.sin_julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sin_julia_angle).desired_width(150.0)).changed() {
                            if let Ok(ang) = input_state.sin_julia_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.sin_julia_magnitude.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.sin_julia_c_real = format!("{:.15}", r);
                                    input_state.sin_julia_c_imag = format!("{:.15}", i);
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

        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(50.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 4.0..=200.0)
                .text("").step_by(1.0).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.sin_julia_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Escape Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.sin_julia_escape_radius).desired_width(100.0)).changed() {
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

// ── Tetration ──────────────────────────────────────────────────────────────

impl FractalGUI for Tetration {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Tetration Parameters").strong());
        ui.add_space(5.0);

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
        ui.label("Escape Threshold:");
        ui.add_space(3.0);

        let mut threshold = params.get("threshold").copied().unwrap_or(1e7);
        let mut log_threshold = threshold.log10();

        ui.horizontal(|ui| {
            ui.label("10^");
            if ui.add(egui::Slider::new(&mut log_threshold, 1.0..=10.0)
                .text("").step_by(0.1).fixed_decimals(1))
                .changed()
            {
                threshold = 10_f64.powf(log_threshold);
                params.insert("threshold".to_string(), threshold);
                input_state.tetration_threshold = format!("{:.2e}", threshold);
                *needs_redraw = true;
            }
            ui.label(format!("≈ {:.2e}", threshold));
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Threshold:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.tetration_threshold).desired_width(120.0)).changed() {
                    if let Ok(val) = input_state.tetration_threshold.parse::<f64>() {
                        if val > 0.0 && val.is_finite() {
                            params.insert("threshold".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
            ui.label(egui::RichText::new("Supports scientific notation: 1e7, 10e6, 1.5e8, etc.").small().weak());
        });

        ui.add_space(10.0);
    }
}

// ── Zubieta ────────────────────────────────────────────────────────────────

impl FractalGUI for Zubieta {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Zubieta Parameters").strong());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.zubieta_coord_mode, CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.zubieta_coord_mode, CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);

        match input_state.zubieta_coord_mode {
            CoordinateMode::Rectangular => {
                let mut c_real = params.get("c_real").copied().unwrap_or(0.0);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
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
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.zubieta_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.zubieta_c_real).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.zubieta_c_real.parse::<f64>() {
                                params.insert("c_real".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.zubieta_c_imag).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.zubieta_c_imag.parse::<f64>() {
                                params.insert("c_imag".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                });
            }
            CoordinateMode::Polar => {
                let c_real = params.get("c_real").copied().unwrap_or(0.0);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.8);

                let mut magnitude = (c_real * c_real + c_imag * c_imag).sqrt();
                let mut angle = c_imag.atan2(c_real);
                if angle < 0.0 { angle += std::f64::consts::TAU; }

                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("|c|:");
                    ui.add_space(13.0);
                    if ui.add(egui::Slider::new(&mut magnitude, 0.0..=3.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });
                ui.horizontal(|ui| {
                    ui.label("ang(c):");
                    if ui.add(egui::Slider::new(&mut angle, 0.0..=std::f64::consts::TAU)
                        .text("").step_by(0.001).fixed_decimals(3)).changed() { changed = true; }
                });

                if changed {
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

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.zubieta_magnitude).desired_width(150.0)).changed() {
                            if let Ok(mag) = input_state.zubieta_magnitude.parse::<f64>() {
                                if let Ok(ang) = input_state.zubieta_angle.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.zubieta_c_real = format!("{:.15}", r);
                                    input_state.zubieta_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.zubieta_angle).desired_width(150.0)).changed() {
                            if let Ok(ang) = input_state.zubieta_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.zubieta_magnitude.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.zubieta_c_real = format!("{:.15}", r);
                                    input_state.zubieta_c_imag = format!("{:.15}", i);
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

// ── Insideout Dragon ───────────────────────────────────────────────────────

impl FractalGUI for InsideoutDragon {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut std::collections::HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Insideout Dragon Parameters").strong());
        ui.add_space(5.0);

        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(4.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 1.0..=100.0)
                .text("").step_by(0.5).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.insideout_dragon_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Value", |ui| {
            ui.horizontal(|ui| {
                ui.label("Escape Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.insideout_dragon_escape_radius).desired_width(100.0)).changed() {
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
