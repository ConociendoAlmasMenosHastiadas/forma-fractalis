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
    Cactus, MarekDragon, Tetration, Lemon, InsideoutDragon, Zubieta, SinJulia, SinhJulia,
    MultiJuliaIFS,
    AdjProbJulia, ChaosSymmetry1, Wallpaper, LaceJulia,
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

impl FractalGUI for BurningShip {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Burning Ship Parameters").strong());
        ui.add_space(5.0);

        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.burning_ship_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.burning_ship_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.burning_ship_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
        });

        ui.add_space(10.0);
    }
}

impl FractalGUI for Cactus {}

impl FractalGUI for TippetsMandelbrot {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Tippets Mandelbrot Parameters").strong());
        ui.add_space(5.0);

        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.tippets_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.tippets_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.tippets_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
        });

        ui.add_space(10.0);
    }
}

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

        // ── Escape Radius ─────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.mandelbrot_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.mandelbrot_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.mandelbrot_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
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

        // ── Escape Radius ─────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.julia_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.julia_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.julia_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
            ui.label(egui::RichText::new("Note: GPU path uses fixed escape radius (GPU slots fully used by c and power).").small().weak());
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

        // ── Escape Radius ─────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.marek_dragon_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.marek_dragon_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.marek_dragon_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
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

// ── Sinh Julia ─────────────────────────────────────────────────────────────

impl FractalGUI for SinhJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Sinh Julia Parameters").strong());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.sinh_julia_coord_mode, CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.sinh_julia_coord_mode, CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);

        match input_state.sinh_julia_coord_mode {
            CoordinateMode::Rectangular => {
                let mut c_real = params.get("c_real").copied().unwrap_or(-0.7);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_real".to_string(), c_real);
                        input_state.sinh_julia_c_real = format!("{:.15}", c_real);
                        *needs_redraw = true;
                    }
                });

                let mut c_imag = params.get("c_imag").copied().unwrap_or(0.27015);
                ui.horizontal(|ui| {
                    ui.label("Im{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_imag, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.sinh_julia_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sinh_julia_c_real).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.sinh_julia_c_real.parse::<f64>() {
                                params.insert("c_real".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sinh_julia_c_imag).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.sinh_julia_c_imag.parse::<f64>() {
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
                    input_state.sinh_julia_c_real = format!("{:.15}", new_real);
                    input_state.sinh_julia_c_imag = format!("{:.15}", new_imag);
                    input_state.sinh_julia_magnitude = format!("{:.15}", magnitude);
                    input_state.sinh_julia_angle = format!("{:.15}", angle);
                    *needs_redraw = true;
                }

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sinh_julia_magnitude).desired_width(150.0)).changed() {
                            if let Ok(mag) = input_state.sinh_julia_magnitude.parse::<f64>() {
                                if let Ok(ang) = input_state.sinh_julia_angle.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.sinh_julia_c_real = format!("{:.15}", r);
                                    input_state.sinh_julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.sinh_julia_angle).desired_width(150.0)).changed() {
                            if let Ok(ang) = input_state.sinh_julia_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.sinh_julia_magnitude.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.sinh_julia_c_real = format!("{:.15}", r);
                                    input_state.sinh_julia_c_imag = format!("{:.15}", i);
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
                input_state.sinh_julia_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });

        ui.add_space(5.0);
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Escape Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.sinh_julia_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.sinh_julia_escape_radius.parse::<f64>() {
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

// ── Multi-Julia IFS ────────────────────────────────────────────────────────

impl FractalGUI for MultiJuliaIFS {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        use crate::app_state::AttractorEntry;

        // Sync GUI attractor list from params when the count differs
        // (e.g. after FractalType reset which writes default params directly).
        {
            let n_params = params
                .get("num_attractors")
                .copied()
                .unwrap_or(2.0) as usize;
            let n_gui = input_state.multi_julia_ifs.attractors.len();
            if n_params != n_gui && (1..=8).contains(&n_params) {
                input_state.multi_julia_ifs.attractors = (0..n_params)
                    .map(|i| {
                        AttractorEntry::new(
                            params.get(&format!("c{}_real", i)).copied().unwrap_or(0.0),
                            params.get(&format!("c{}_imag", i)).copied().unwrap_or(0.0),
                            params
                                .get(&format!("prob{}", i))
                                .copied()
                                .unwrap_or(1.0 / n_params as f64),
                        )
                    })
                    .collect();
                input_state.multi_julia_ifs.seed = format!(
                    "{}",
                    params.get("seed").copied().unwrap_or(0.0) as i64
                );
                input_state.multi_julia_ifs.samples = format!(
                    "{:.0}",
                    params.get("samples").copied().unwrap_or(5_000_000.0)
                );
                input_state.multi_julia_ifs.burn_in = format!(
                    "{:.0}",
                    params.get("burn_in").copied().unwrap_or(50.0)
                );
                input_state.multi_julia_ifs.use_log_density =
                    params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
            }
        }

        ui.label(egui::RichText::new("Multi-Julia IFS Parameters").strong());
        ui.label(
            egui::RichText::new("g_i(z) = +/-sqrt(z - c_i), chaos game orbit density")
                .small()
                .weak(),
        );
        ui.add_space(6.0);

        // Collect deferred mutations so we don't conflict borrows mid-loop.
        let mut remove_idx: Option<usize> = None;
        let mut prob_changes: Vec<(usize, f64)> = Vec::new();

        let n = input_state.multi_julia_ifs.attractors.len();

        for i in 0..n {
            ui.group(|ui| {
                // ── Header: label, coord-mode toggle, optional remove ──
                let old_mode = input_state.multi_julia_ifs.attractors[i].coord_mode;

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("C{}", i))
                            .strong()
                            .monospace(),
                    );
                    ui.add_space(4.0);
                    ui.radio_value(
                        &mut input_state.multi_julia_ifs.attractors[i].coord_mode,
                        CoordinateMode::Rectangular,
                        "Re/Im",
                    );
                    ui.radio_value(
                        &mut input_state.multi_julia_ifs.attractors[i].coord_mode,
                        CoordinateMode::Polar,
                        "Mag/Ang",
                    );
                    if n > 2 {
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.small_button("Remove").clicked() {
                                    remove_idx = Some(i);
                                    *needs_redraw = true;
                                }
                            },
                        );
                    }
                });

                // If coord mode switched, sync the text fields.
                let new_mode = input_state.multi_julia_ifs.attractors[i].coord_mode;
                if old_mode != new_mode {
                    let entry = &mut input_state.multi_julia_ifs.attractors[i];
                    if new_mode == CoordinateMode::Rectangular {
                        let mag = entry.parse_magnitude();
                        let ang = entry.parse_angle();
                        entry.real = format!("{:.6}", mag * ang.cos());
                        entry.imag = format!("{:.6}", mag * ang.sin());
                    } else {
                        let re = entry.parse_real();
                        let im = entry.parse_imag();
                        let mag = (re * re + im * im).sqrt();
                        let ang = {
                            let a = im.atan2(re);
                            if a < 0.0 { a + std::f64::consts::TAU } else { a }
                        };
                        entry.magnitude = format!("{:.6}", mag);
                        entry.angle = format!("{:.6}", ang);
                    }
                    *needs_redraw = true;
                }

                // ── c value controls ──
                match input_state.multi_julia_ifs.attractors[i].coord_mode {
                    CoordinateMode::Rectangular => {
                        let mut real =
                            input_state.multi_julia_ifs.attractors[i].parse_real();
                        ui.horizontal(|ui| {
                            ui.label("Re:");
                            if ui
                                .add(
                                    egui::Slider::new(&mut real, -3.0..=3.0)
                                        .text("")
                                        .step_by(0.001)
                                        .fixed_decimals(3)
                                        .clamping(egui::SliderClamping::Never),
                                )
                                .changed()
                            {
                                input_state.multi_julia_ifs.attractors[i].real =
                                    format!("{:.6}", real);
                                *needs_redraw = true;
                            }
                        });

                        let mut imag =
                            input_state.multi_julia_ifs.attractors[i].parse_imag();
                        ui.horizontal(|ui| {
                            ui.label("Im:");
                            if ui
                                .add(
                                    egui::Slider::new(&mut imag, -3.0..=3.0)
                                        .text("")
                                        .step_by(0.001)
                                        .fixed_decimals(3)
                                        .clamping(egui::SliderClamping::Never),
                                )
                                .changed()
                            {
                                input_state.multi_julia_ifs.attractors[i].imag =
                                    format!("{:.6}", imag);
                                *needs_redraw = true;
                            }
                        });

                        ui.collapsing("Precise value", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Re:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut input_state
                                                .multi_julia_ifs
                                                .attractors[i]
                                                .real,
                                        )
                                        .desired_width(120.0),
                                    )
                                    .changed()
                                {
                                    trigger_debounced_redraw(
                                        &mut input_state.debounce_timer,
                                        &mut input_state.pending_redraw,
                                    );
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Im:");
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(
                                            &mut input_state
                                                .multi_julia_ifs
                                                .attractors[i]
                                                .imag,
                                        )
                                        .desired_width(120.0),
                                    )
                                    .changed()
                                {
                                    trigger_debounced_redraw(
                                        &mut input_state.debounce_timer,
                                        &mut input_state.pending_redraw,
                                    );
                                }
                            });
                        });
                    }
                    CoordinateMode::Polar => {
                        let mut mag =
                            input_state.multi_julia_ifs.attractors[i].parse_magnitude();
                        ui.horizontal(|ui| {
                            ui.label("|c|:");
                            if ui
                                .add(
                                    egui::Slider::new(&mut mag, 0.0..=4.0)
                                        .text("")
                                        .step_by(0.001)
                                        .fixed_decimals(3)
                                        .clamping(egui::SliderClamping::Never),
                                )
                                .changed()
                            {
                                input_state.multi_julia_ifs.attractors[i].magnitude =
                                    format!("{:.6}", mag);
                                *needs_redraw = true;
                            }
                        });

                        let mut ang =
                            input_state.multi_julia_ifs.attractors[i].parse_angle();
                        ui.horizontal(|ui| {
                            ui.label("ang:");
                            if ui
                                .add(
                                    egui::Slider::new(
                                        &mut ang,
                                        0.0..=std::f64::consts::TAU,
                                    )
                                    .text("")
                                    .step_by(0.001)
                                    .fixed_decimals(3)
                                    .clamping(egui::SliderClamping::Never),
                                )
                                .changed()
                            {
                                input_state.multi_julia_ifs.attractors[i].angle =
                                    format!("{:.6}", ang);
                                *needs_redraw = true;
                            }
                        });
                    }
                }

                // ── Probability slider ──
                let mut prob = input_state.multi_julia_ifs.attractors[i].prob;
                ui.horizontal(|ui| {
                    ui.label("Prob:");
                    if ui
                        .add(
                            egui::Slider::new(&mut prob, 0.0..=1.0)
                                .text("")
                                .step_by(0.001)
                                .fixed_decimals(3),
                        )
                        .changed()
                    {
                        prob_changes.push((i, prob));
                        *needs_redraw = true;
                    }
                });
                ui.label(
                    egui::RichText::new(format!(
                        "  = {:.1}%",
                        input_state.multi_julia_ifs.attractors[i].prob * 100.0
                    ))
                    .small()
                    .weak(),
                );
            });
            ui.add_space(3.0);
        }

        // Apply deferred probability changes (linked sliders).
        for (idx, new_prob) in prob_changes {
            input_state.multi_julia_ifs.set_prob_linked(idx, new_prob);
        }

        // Apply deferred remove.
        if let Some(idx) = remove_idx {
            input_state.multi_julia_ifs.remove_attractor(idx);
        }

        // Add-map button.
        if n < 8 {
            if ui.button("+ Add IFS Map").clicked() {
                input_state.multi_julia_ifs.add_attractor();
                *needs_redraw = true;
            }
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // Seed.
        ui.horizontal(|ui| {
            ui.label("Seed:");
            if ui
                .add(
                    egui::TextEdit::singleline(&mut input_state.multi_julia_ifs.seed)
                        .desired_width(100.0),
                )
                .changed()
            {
                trigger_debounced_redraw(
                    &mut input_state.debounce_timer,
                    &mut input_state.pending_redraw,
                );
            }
        });
        ui.label(
            egui::RichText::new(
                "Changing seed produces a different random variant of the same shape.",
            )
            .small()
            .weak(),
        );

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // Orbit samples (log-scale slider for wide range).
        let mut samples = input_state.multi_julia_ifs.parse_samples();
        let mut log_samples = samples.log10();
        ui.horizontal(|ui| {
            ui.label("Samples:");
            if ui
                .add(
                    egui::Slider::new(&mut log_samples, 5.0..=7.7) // 100K to ~50M
                        .text("")
                        .step_by(0.01)
                        .fixed_decimals(2)
                        .custom_formatter(|v, _| {
                            let s = 10.0_f64.powf(v);
                            if s >= 1_000_000.0 {
                                format!("{:.1}M", s / 1_000_000.0)
                            } else {
                                format!("{:.0}K", s / 1_000.0)
                            }
                        }),
                )
                .changed()
            {
                samples = 10.0_f64.powf(log_samples);
                input_state.multi_julia_ifs.samples = format!("{:.0}", samples);
                *needs_redraw = true;
            }
        });
        ui.label(
            egui::RichText::new("More samples = smoother image, slower render")
                .small()
                .weak(),
        );

        ui.add_space(4.0);

        // Burn-in.
        let mut burn_in = input_state.multi_julia_ifs.parse_burn_in();
        ui.horizontal(|ui| {
            ui.label("Burn-in:");
            if ui
                .add(
                    egui::Slider::new(&mut burn_in, 0.0..=1000.0)
                        .text("")
                        .step_by(1.0)
                        .fixed_decimals(0),
                )
                .changed()
            {
                input_state.multi_julia_ifs.burn_in = format!("{:.0}", burn_in);
                *needs_redraw = true;
            }
        });
        ui.label(
            egui::RichText::new("Initial steps discarded before recording density")
                .small()
                .weak(),
        );

        ui.add_space(4.0);

        // Log density checkbox.
        if ui
            .checkbox(
                &mut input_state.multi_julia_ifs.use_log_density,
                "Log density normalization",
            )
            .changed()
        {
            *needs_redraw = true;
        }
        ui.label(
            egui::RichText::new("Compresses dynamic range for visible structure (recommended)")
                .small()
                .weak(),
        );

        ui.add_space(6.0);

        // Always keep params in sync with the GUI state so that any change
        // is immediately available to the rendering pipeline.
        input_state.multi_julia_ifs.apply_to_params(params);
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

        // ── Escape Radius ─────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
        ui.horizontal(|ui| {
            ui.label("Escape Radius:");
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.zubieta_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.zubieta_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.zubieta_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
        });

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

// -- Adj Prob Julia -------------------------------------------------------

impl FractalGUI for AdjProbJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        // Sync GUI fields from params on load/reset
        {
            let p_thresh = params.get("threshold").copied().unwrap_or(0.5);
            let p_samp   = params.get("samples").copied().unwrap_or(20.0);
            let p_burn   = params.get("burn_in").copied().unwrap_or(10.0);
            let p_seed   = params.get("seed").copied().unwrap_or(0.0);
            let p_log    = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
            if input_state.parse_adj_prob_julia_threshold() != p_thresh {
                input_state.adj_prob_julia_threshold = format!("{:.4}", p_thresh);
            }
            if input_state.parse_adj_prob_julia_samples() != p_samp {
                input_state.adj_prob_julia_samples = format!("{:.0}", p_samp);
            }
            if input_state.parse_adj_prob_julia_burn_in() != p_burn {
                input_state.adj_prob_julia_burn_in = format!("{:.0}", p_burn);
            }
            if input_state.parse_adj_prob_julia_seed() != p_seed {
                input_state.adj_prob_julia_seed = format!("{:.0}", p_seed);
            }
            if input_state.adj_prob_julia_use_log_density != p_log {
                input_state.adj_prob_julia_use_log_density = p_log;
            }
        }

        ui.label(egui::RichText::new("Adj Prob Julia Parameters").strong());
        ui.label(
            egui::RichText::new("z_{n+1} = s*sqrt(z_n - z_0),  z_0 = screen pixel")
                .small()
                .weak(),
        );
        ui.label(
            egui::RichText::new("R=0.5: full Julia sets.  R=0: +sqrt branch only.  R=1: -sqrt branch only.")
                .small()
                .weak(),
        );
        ui.add_space(6.0);

        // Sign threshold slider
        let mut threshold = input_state.parse_adj_prob_julia_threshold();
        ui.horizontal(|ui| {
            ui.label("Sign Threshold (R):");
            if ui.add(egui::Slider::new(&mut threshold, 0.0..=1.0)
                .text("").step_by(0.01).fixed_decimals(2)).changed()
            {
                input_state.adj_prob_julia_threshold = format!("{:.4}", threshold);
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Samples per pixel (linear, small range)
        let mut samples = input_state.parse_adj_prob_julia_samples();
        ui.horizontal(|ui| {
            ui.label("Samples/pixel:");
            if ui.add(egui::Slider::new(&mut samples, 1.0..=500.0)
                .text("").step_by(1.0).fixed_decimals(0)).changed()
            {
                input_state.adj_prob_julia_samples = format!("{:.0}", samples);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Orbit steps per pixel per pass (x64 passes total)").small().weak());

        ui.add_space(4.0);

        // Burn-in slider
        let mut burn_in = input_state.parse_adj_prob_julia_burn_in();
        ui.horizontal(|ui| {
            ui.label("Burn-in:");
            if ui.add(egui::Slider::new(&mut burn_in, 0.0..=200.0)
                .text("").step_by(1.0).fixed_decimals(0)).changed()
            {
                input_state.adj_prob_julia_burn_in = format!("{:.0}", burn_in);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Steps discarded before recording").small().weak());

        ui.add_space(4.0);

        // Log density checkbox
        if ui.checkbox(&mut input_state.adj_prob_julia_use_log_density, "Log density normalization").changed() {
            *needs_redraw = true;
        }
        ui.label(egui::RichText::new("Compresses dynamic range (recommended)").small().weak());

        ui.add_space(4.0);

        // Seed text field
        ui.horizontal(|ui| {
            ui.label("PRNG Seed:");
            if ui.add(egui::TextEdit::singleline(&mut input_state.adj_prob_julia_seed)
                .desired_width(80.0)).changed()
            {
                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
            }
        });

        // Always keep params in sync
        params.insert("threshold".to_string(),     input_state.parse_adj_prob_julia_threshold());
        params.insert("samples".to_string(),       input_state.parse_adj_prob_julia_samples());
        params.insert("burn_in".to_string(),       input_state.parse_adj_prob_julia_burn_in());
        params.insert("seed".to_string(),          input_state.parse_adj_prob_julia_seed());
        params.insert("use_log_density".to_string(), if input_state.adj_prob_julia_use_log_density { 1.0 } else { 0.0 });

        ui.add_space(10.0);
    }
}

// ── ChaosSymmetry1 ────────────────────────────────────────────────────────

impl FractalGUI for Wallpaper {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        {
            let p_a = params.get("a").copied().unwrap_or(0.1);
            let p_b = params.get("b").copied().unwrap_or(0.1);
            let p_c = params.get("c").copied().unwrap_or(10.0);
            let p_samples = params.get("samples").copied().unwrap_or(24.0);
            let p_burn_in = params.get("burn_in").copied().unwrap_or(40.0);
            let p_log = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;

            if (input_state.parse_wallpaper_a() - p_a).abs() > 1e-9 {
                input_state.wallpaper_a = format!("{:.6}", p_a);
            }
            if (input_state.parse_wallpaper_b() - p_b).abs() > 1e-9 {
                input_state.wallpaper_b = format!("{:.6}", p_b);
            }
            if (input_state.parse_wallpaper_c() - p_c).abs() > 1e-9 {
                input_state.wallpaper_c = format!("{:.6}", p_c);
            }
            if input_state.parse_wallpaper_samples() != p_samples {
                input_state.wallpaper_samples = format!("{:.0}", p_samples);
            }
            if input_state.parse_wallpaper_burn_in() != p_burn_in {
                input_state.wallpaper_burn_in = format!("{:.0}", p_burn_in);
            }
            if input_state.wallpaper_use_log_density != p_log {
                input_state.wallpaper_use_log_density = p_log;
            }
        }

        ui.label(egui::RichText::new("Wallpaper Parameters").strong());
        ui.label(
            egui::RichText::new(
                "x_{n+1} = y_n - sign(x_n)*sqrt(abs(b*x_n-c)), y_{n+1} = a - x_n"
            )
            .small()
            .weak(),
        );
        ui.label(egui::RichText::new("Each screen pixel is its own seed; recorded orbits build the density image.").small().weak());
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Tip: Ctrl+click any slider to type an exact value").small().weak());
        ui.add_space(4.0);

        let render_param = |ui: &mut egui::Ui,
                            label: &str,
                            field: &mut String,
                            value: &mut f64,
                            range: std::ops::RangeInclusive<f64>,
                            needs_redraw: &mut bool,
                            params: &mut HashMap<String, f64>,
                            key: &str| {
            ui.horizontal(|ui| {
                ui.label(label);
                if ui.add(egui::TextEdit::singleline(field).desired_width(90.0)).changed() {
                    if let Ok(parsed) = field.parse::<f64>() {
                        *value = parsed.clamp(*range.start(), *range.end());
                        params.insert(key.to_string(), *value);
                        *needs_redraw = true;
                    }
                }
            });
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(value, range.clone()).show_value(false).step_by(0.001)).changed() {
                params.insert(key.to_string(), *value);
                *field = format!("{:.6}", *value);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
            ui.add_space(4.0);
        };

        let mut a = params.get("a").copied().unwrap_or(0.1);
        render_param(ui, "a:", &mut input_state.wallpaper_a, &mut a, 0.0..=100.0, needs_redraw, params, "a");
        ui.label(egui::RichText::new("Shift term in y_{n+1} = a - x_n").small().weak());

        let mut b = params.get("b").copied().unwrap_or(0.1);
        render_param(ui, "b:", &mut input_state.wallpaper_b, &mut b, 0.0..=100.0, needs_redraw, params, "b");
        ui.label(egui::RichText::new("Scale inside the square-root term").small().weak());

        let mut c = params.get("c").copied().unwrap_or(10.0);
        render_param(ui, "c:", &mut input_state.wallpaper_c, &mut c, 0.0..=100.0, needs_redraw, params, "c");
        ui.label(egui::RichText::new("Offset inside sqrt(abs(b*x_n-c))").small().weak());

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        let mut samples = input_state.parse_wallpaper_samples();
        ui.horizontal(|ui| {
            ui.label("Samples/seed:");
            if ui.add(egui::Slider::new(&mut samples, 1.0..=crate::fractals::Wallpaper::MAX_SAMPLES as f64).text("").step_by(1.0).fixed_decimals(0)).changed() {
                input_state.wallpaper_samples = format!("{:.0}", samples);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Recorded orbit steps for each screen seed after burn-in").small().weak());

        let mut burn_in = input_state.parse_wallpaper_burn_in();
        ui.horizontal(|ui| {
            ui.label("Burn-in:");
            if ui.add(egui::Slider::new(&mut burn_in, 0.0..=512.0).text("").step_by(1.0).fixed_decimals(0)).changed() {
                input_state.wallpaper_burn_in = format!("{:.0}", burn_in);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Initial steps discarded before accumulating density").small().weak());

        if ui.checkbox(&mut input_state.wallpaper_use_log_density, "Log density normalization").changed() {
            *needs_redraw = true;
        }
        ui.label(egui::RichText::new("Compresses the density range for a more readable image").small().weak());

        params.insert("a".to_string(), input_state.parse_wallpaper_a());
        params.insert("b".to_string(), input_state.parse_wallpaper_b());
        params.insert("c".to_string(), input_state.parse_wallpaper_c());
        params.insert("samples".to_string(), input_state.parse_wallpaper_samples());
        params.insert("burn_in".to_string(), input_state.parse_wallpaper_burn_in());
        params.insert("use_log_density".to_string(), if input_state.wallpaper_use_log_density { 1.0 } else { 0.0 });
    }
}

// ── ChaosSymmetry1 ────────────────────────────────────────────────────────

impl FractalGUI for ChaosSymmetry1 {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        // Sync GUI text fields from params on load/reset
        {
            let p_samp = params.get("samples").copied().unwrap_or(5_000_000.0);
            let p_burn = params.get("burn_in").copied().unwrap_or(1_000.0);
            let p_seed = params.get("seed").copied().unwrap_or(0.0);
            let p_log  = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
            if input_state.parse_chaos_symmetry1_samples() != p_samp {
                input_state.chaos_symmetry1_samples = format!("{:.0}", p_samp);
            }
            if input_state.parse_chaos_symmetry1_burn_in() != p_burn {
                input_state.chaos_symmetry1_burn_in = format!("{:.0}", p_burn);
            }
            if input_state.parse_chaos_symmetry1_seed() != p_seed {
                input_state.chaos_symmetry1_seed = format!("{:.0}", p_seed);
            }
            if input_state.chaos_symmetry1_use_log_density != p_log {
                input_state.chaos_symmetry1_use_log_density = p_log;
            }
        }

        ui.label(egui::RichText::new("ChaosSymmetry1 Parameters").strong());
        ui.label(
            egui::RichText::new(
                "z_{n+1} = (a0+a1|z|^2+a2 Re(z^m)+a3 i)*z + a4*conj(z)^{m-1}"
            )
            .small()
            .weak(),
        );
        ui.add_space(6.0);

        // ── Symmetry degree m ──
        let mut m_val = params.get("m").copied().unwrap_or(3.0).round().clamp(2.0, 8.0);
        ui.horizontal(|ui| {
            ui.label("m (symmetry degree):");
            if ui.add(egui::Slider::new(&mut m_val, 2.0..=8.0)
                .step_by(1.0).fixed_decimals(0)).changed()
            {
                params.insert("m".to_string(), m_val.round());
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Rotational symmetry of the attractor image").small().weak());

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Tip: Ctrl+click any slider to type an exact value").small().weak());
        ui.add_space(4.0);

        // ── a0 and a1 ──
        // Sync text fields from params on load/reset
        {
            let p_a0 = params.get("a0").copied().unwrap_or(1.5);
            let p_a1 = params.get("a1").copied().unwrap_or(-1.5);
            let p_a2 = params.get("a2").copied().unwrap_or(0.0);
            let p_a3 = params.get("a3").copied().unwrap_or(0.0);
            let p_a4 = params.get("a4").copied().unwrap_or(0.5);
            if (input_state.parse_chaos_symmetry1_a0() - p_a0).abs() > 1e-9 {
                input_state.chaos_symmetry1_a0 = format!("{:.4}", p_a0);
            }
            if (input_state.parse_chaos_symmetry1_a1() - p_a1).abs() > 1e-9 {
                input_state.chaos_symmetry1_a1 = format!("{:.4}", p_a1);
            }
            if (input_state.parse_chaos_symmetry1_a2() - p_a2).abs() > 1e-9 {
                input_state.chaos_symmetry1_a2 = format!("{:.4}", p_a2);
            }
            if (input_state.parse_chaos_symmetry1_a3() - p_a3).abs() > 1e-9 {
                input_state.chaos_symmetry1_a3 = format!("{:.4}", p_a3);
            }
            if (input_state.parse_chaos_symmetry1_a4() - p_a4).abs() > 1e-9 {
                input_state.chaos_symmetry1_a4 = format!("{:.4}", p_a4);
            }
        }

        let mut a0 = params.get("a0").copied().unwrap_or(1.5);
        ui.horizontal(|ui| {
            ui.label("a0:");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_a0).desired_width(80.0)).changed() {
                if let Ok(v) = input_state.chaos_symmetry1_a0.parse::<f64>() {
                    a0 = v.clamp(-3.0, 3.0);
                    params.insert("a0".to_string(), a0);
                    *needs_redraw = true;
                }
            }
        });
        {
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(&mut a0, -3.0..=3.0)
                .show_value(false).step_by(0.001)).changed()
            {
                params.insert("a0".to_string(), a0);
                input_state.chaos_symmetry1_a0 = format!("{:.4}", a0);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
        }

        let mut a1 = params.get("a1").copied().unwrap_or(-1.5);
        ui.horizontal(|ui| {
            ui.label("a1:");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_a1).desired_width(80.0)).changed() {
                if let Ok(v) = input_state.chaos_symmetry1_a1.parse::<f64>() {
                    a1 = v.clamp(-3.0, 3.0);
                    params.insert("a1".to_string(), a1);
                    *needs_redraw = true;
                }
            }
        });
        {
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(&mut a1, -3.0..=3.0)
                .show_value(false).step_by(0.001)).changed()
            {
                params.insert("a1".to_string(), a1);
                input_state.chaos_symmetry1_a1 = format!("{:.4}", a1);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
        }
        ui.label(
            egui::RichText::new("a0 & a1: interesting when outside (-1,1); a1 opposite sign to a0")
                .small().weak(),
        );

        ui.add_space(4.0);

        // ── a2 ──
        let mut a2 = params.get("a2").copied().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.label("a2 (perturbation):");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_a2).desired_width(80.0)).changed() {
                if let Ok(v) = input_state.chaos_symmetry1_a2.parse::<f64>() {
                    a2 = v.clamp(-2.0, 2.0);
                    params.insert("a2".to_string(), a2);
                    *needs_redraw = true;
                }
            }
        });
        {
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(&mut a2, -2.0..=2.0)
                .show_value(false).step_by(0.001)).changed()
            {
                params.insert("a2".to_string(), a2);
                input_state.chaos_symmetry1_a2 = format!("{:.4}", a2);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
        }
        ui.label(egui::RichText::new("0 = no perturbation; non-zero perturbs the symmetry").small().weak());

        ui.add_space(4.0);

        // ── a3 ──
        let mut a3 = params.get("a3").copied().unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.label("a3 (bilateral symmetry):");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_a3).desired_width(80.0)).changed() {
                if let Ok(v) = input_state.chaos_symmetry1_a3.parse::<f64>() {
                    a3 = v.clamp(-1.0, 1.0);
                    params.insert("a3".to_string(), a3);
                    *needs_redraw = true;
                }
            }
        });
        {
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(&mut a3, -1.0..=1.0)
                .show_value(false).step_by(0.001)).changed()
            {
                params.insert("a3".to_string(), a3);
                input_state.chaos_symmetry1_a3 = format!("{:.4}", a3);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
        }
        ui.label(egui::RichText::new("0 = mirror-symmetric; non-zero breaks bilateral symmetry").small().weak());

        ui.add_space(4.0);

        // ── a4 ──
        let mut a4 = params.get("a4").copied().unwrap_or(0.5);
        ui.horizontal(|ui| {
            ui.label("a4:");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_a4).desired_width(80.0)).changed() {
                if let Ok(v) = input_state.chaos_symmetry1_a4.parse::<f64>() {
                    a4 = v.clamp(-1.0, 1.0);
                    params.insert("a4".to_string(), a4);
                    *needs_redraw = true;
                }
            }
        });
        {
            let orig_w = ui.style().spacing.slider_width;
            ui.style_mut().spacing.slider_width = ui.available_width() - 20.0;
            if ui.add(egui::Slider::new(&mut a4, -1.0..=1.0)
                .show_value(false).step_by(0.001)).changed()
            {
                params.insert("a4".to_string(), a4);
                input_state.chaos_symmetry1_a4 = format!("{:.4}", a4);
                *needs_redraw = true;
            }
            ui.style_mut().spacing.slider_width = orig_w;
        }
        ui.label(egui::RichText::new("Conjugate term scale; avoid values near 0").small().weak());

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Samples (log scale) ──
        let mut samples = input_state.parse_chaos_symmetry1_samples();
        let mut log_samples = samples.log10();
        ui.horizontal(|ui| {
            ui.label("Samples:");
            if ui.add(
                egui::Slider::new(&mut log_samples, 5.0..=7.7)
                    .text("")
                    .step_by(0.01)
                    .fixed_decimals(2)
                    .custom_formatter(|v, _| {
                        let s = 10.0_f64.powf(v);
                        if s >= 1_000_000.0 {
                            format!("{:.1}M", s / 1_000_000.0)
                        } else {
                            format!("{:.0}K", s / 1_000.0)
                        }
                    }),
            ).changed() {
                samples = 10.0_f64.powf(log_samples);
                input_state.chaos_symmetry1_samples = format!("{:.0}", samples);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("More samples = smoother image, slower render").small().weak());

        ui.add_space(4.0);

        // ── Burn-in ──
        let mut burn_in = input_state.parse_chaos_symmetry1_burn_in();
        ui.horizontal(|ui| {
            ui.label("Burn-in:");
            if ui.add(egui::Slider::new(&mut burn_in, 0.0..=10_000.0)
                .text("").step_by(100.0).fixed_decimals(0)).changed()
            {
                input_state.chaos_symmetry1_burn_in = format!("{:.0}", burn_in);
                *needs_redraw = true;
            }
        });
        ui.label(egui::RichText::new("Steps discarded to settle onto the attractor (pass 1)").small().weak());

        ui.add_space(4.0);

        // ── Log density ──
        if ui.checkbox(&mut input_state.chaos_symmetry1_use_log_density, "Log density normalization").changed() {
            *needs_redraw = true;
        }
        ui.label(egui::RichText::new("Compresses dynamic range (strongly recommended)").small().weak());

        ui.add_space(4.0);

        // ── Seed ──
        ui.horizontal(|ui| {
            ui.label("PRNG Seed:");
            if ui.add(egui::TextEdit::singleline(&mut input_state.chaos_symmetry1_seed)
                .desired_width(80.0)).changed()
            {
                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
            }
        });
        ui.label(egui::RichText::new("Different seeds produce different starting points").small().weak());

        // Always keep params in sync with GUI state
        params.insert("samples".to_string(),        input_state.parse_chaos_symmetry1_samples());
        params.insert("burn_in".to_string(),        input_state.parse_chaos_symmetry1_burn_in());
        params.insert("seed".to_string(),           input_state.parse_chaos_symmetry1_seed());
        params.insert("use_log_density".to_string(), if input_state.chaos_symmetry1_use_log_density { 1.0 } else { 0.0 });

        ui.add_space(10.0);
    }
}

// ── Lace Julia ──────────────────────────────────────────────────────────────

impl FractalGUI for LaceJulia {
    fn render_parameters_gui(
        &self,
        ui: &mut egui::Ui,
        params: &mut HashMap<String, f64>,
        input_state: &mut InputState,
        needs_redraw: &mut bool,
    ) {
        ui.label(egui::RichText::new("Lace Julia Parameters").strong());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Coordinate mode:");
            if ui.radio_value(&mut input_state.lace_julia_coord_mode, CoordinateMode::Rectangular, "Rectangular").clicked() {
                *needs_redraw = true;
            }
            if ui.radio_value(&mut input_state.lace_julia_coord_mode, CoordinateMode::Polar, "Polar").clicked() {
                *needs_redraw = true;
            }
        });

        ui.add_space(8.0);

        match input_state.lace_julia_coord_mode {
            CoordinateMode::Rectangular => {
                let mut c_real = params.get("c_real").copied().unwrap_or(0.0);
                ui.horizontal(|ui| {
                    ui.label("Re{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_real, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_real".to_string(), c_real);
                        input_state.lace_julia_c_real = format!("{:.15}", c_real);
                        *needs_redraw = true;
                    }
                });

                let mut c_imag = params.get("c_imag").copied().unwrap_or(0.5);
                ui.horizontal(|ui| {
                    ui.label("Im{c}:");
                    ui.add_space(5.0);
                    if ui.add(egui::Slider::new(&mut c_imag, -2.0..=2.0)
                        .text("").step_by(0.001).fixed_decimals(3)).changed()
                    {
                        params.insert("c_imag".to_string(), c_imag);
                        input_state.lace_julia_c_imag = format!("{:.15}", c_imag);
                        *needs_redraw = true;
                    }
                });

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Re{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.lace_julia_c_real).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.lace_julia_c_real.parse::<f64>() {
                                params.insert("c_real".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Im{c}:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.lace_julia_c_imag).desired_width(150.0)).changed() {
                            if let Ok(val) = input_state.lace_julia_c_imag.parse::<f64>() {
                                params.insert("c_imag".to_string(), val.clamp(-2.0, 2.0));
                                trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                            }
                        }
                    });
                });
            }
            CoordinateMode::Polar => {
                let c_real = params.get("c_real").copied().unwrap_or(0.0);
                let c_imag = params.get("c_imag").copied().unwrap_or(0.5);

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
                    input_state.lace_julia_c_real = format!("{:.15}", new_real);
                    input_state.lace_julia_c_imag = format!("{:.15}", new_imag);
                    input_state.lace_julia_magnitude = format!("{:.15}", magnitude);
                    input_state.lace_julia_angle = format!("{:.15}", angle);
                    *needs_redraw = true;
                }

                ui.add_space(5.0);
                ui.collapsing("Advanced: 15-Digit Precision", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("|c|:");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.lace_julia_magnitude).desired_width(150.0)).changed() {
                            if let Ok(mag) = input_state.lace_julia_magnitude.parse::<f64>() {
                                if let Ok(ang) = input_state.lace_julia_angle.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.lace_julia_c_real = format!("{:.15}", r);
                                    input_state.lace_julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("ang(c):");
                        if ui.add(egui::TextEdit::singleline(&mut input_state.lace_julia_angle).desired_width(150.0)).changed() {
                            if let Ok(ang) = input_state.lace_julia_angle.parse::<f64>() {
                                if let Ok(mag) = input_state.lace_julia_magnitude.parse::<f64>() {
                                    let m = mag.clamp(0.0, 3.0);
                                    let (r, i) = (m * ang.cos(), m * ang.sin());
                                    params.insert("c_real".to_string(), r);
                                    params.insert("c_imag".to_string(), i);
                                    input_state.lace_julia_c_real = format!("{:.15}", r);
                                    input_state.lace_julia_c_imag = format!("{:.15}", i);
                                    trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                                }
                            }
                        }
                    });
                });
            }
        }

        // ── Escape Radius ─────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Escape radius:");
            let mut escape_radius = params.get("escape_radius").copied().unwrap_or(2.0);
            ui.add_space(5.0);
            if ui.add(egui::Slider::new(&mut escape_radius, 0.5..=100.0)
                .text("").step_by(0.1).fixed_decimals(1).logarithmic(true))
                .changed()
            {
                params.insert("escape_radius".to_string(), escape_radius);
                input_state.lace_julia_escape_radius = format!("{:.1}", escape_radius);
                *needs_redraw = true;
            }
        });
        ui.collapsing("Advanced: Precise Escape Radius", |ui| {
            ui.horizontal(|ui| {
                ui.label("Radius:");
                if ui.add(egui::TextEdit::singleline(&mut input_state.lace_julia_escape_radius).desired_width(100.0)).changed() {
                    if let Ok(val) = input_state.lace_julia_escape_radius.parse::<f64>() {
                        if val > 0.0 {
                            params.insert("escape_radius".to_string(), val);
                            trigger_debounced_redraw(&mut input_state.debounce_timer, &mut input_state.pending_redraw);
                        }
                    }
                }
            });
        });

        ui.add_space(10.0);
    }
}
