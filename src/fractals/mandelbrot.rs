//! Mandelbrot Set Implementation
//!
//! The classic Mandelbrot set: z(n+1) = z(n)² + c
//! where z starts at 0 and c is the point being tested.

use super::{Fractal, FractalView};
use num_complex::Complex64;
use std::collections::HashMap;

/// Mandelbrot set fractal
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
    fn iterate(&self, c_real: f64, c_imag: f64, _parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        let mut z = Complex64::new(0.0, 0.0);
        let mut iter = 0;

        while iter < max_iter {
            if z.norm_sqr() > 4.0 {
                break;
            }

            z = z * z + c;
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

    // Mandelbrot has no parameters, so we use the default empty Vec
}
