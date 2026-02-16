//! Burning Ship Fractal Implementation
//!
//! The Burning Ship fractal: z(n+1) = (|Re(z)| + i|Im(z)|)² + c
//! where z starts at 0 and c is the point being tested.
//!
//! The absolute value operation on components creates the distinctive
//! "ship" shape with a prominent bow structure.

use super::{Fractal, FractalView};
use num_complex::Complex64;
use std::collections::HashMap;

/// Burning Ship fractal
pub struct BurningShip;

impl BurningShip {
    /// Creates a new Burning Ship fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for BurningShip {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for BurningShip {
    fn iterate(&self, c_real: f64, c_imag: f64, _parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        let mut z = Complex64::new(0.0, 0.0);
        let mut iter = 0;

        while iter < max_iter {
            if z.norm_sqr() > 4.0 {
                break;
            }

            // Apply absolute value to both components before squaring
            let z_abs = Complex64::new(z.re.abs(), z.im.abs());
            z = z_abs * z_abs + c;
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        // Classic Burning Ship viewing position
        view.center_x = -0.5;
        view.center_y = -0.6;
        view.zoom = 0.8;
        view
    }

    fn name(&self) -> &str {
        "Burning Ship"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = (|Re(z_n)| + i|Im(z_n)|)^2 + c"
    }

    // Burning Ship has no parameters, so we use the default empty Vec
}

// FractalGUI implementation (no parameters for Burning Ship)
impl super::FractalGUI for BurningShip {}
