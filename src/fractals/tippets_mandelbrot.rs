//! Tippets Mandelbrot Set Implementation https://paulbourke.net/fractals/tippetts/
//!
//! The Tippets Mandelbrot is a variation of the classic Mandelbrot set discovered
//! by John Tippets in 1992. The key difference from the
//! standard Mandelbrot is the ORDER of operations when updating the real and
//! imaginary components.
//!
//! Standard Mandelbrot:
//!   xnew = x² - y² + a
//!   ynew = 2*x*y + b
//!   x = xnew
//!   y = ynew
//!
//! Tippets Mandelbrot:
//!   x = x² - y² + a     (updates x immediately)
//!   y = 2*x*y + b       (uses the NEW x, not the old one!)
//!
//! This creates a different fractal structure due to the dependency on the
//! already-updated x value when calculating y.

use super::{Fractal, FractalView};
use std::collections::HashMap;

/// Tippets Mandelbrot set fractal
pub struct TippetsMandelbrot;

impl TippetsMandelbrot {
    /// Creates a new Tippets Mandelbrot fractal instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for TippetsMandelbrot {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for TippetsMandelbrot {
    fn iterate(&self, c_real: f64, c_imag: f64, _parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let a = c_real;
        let b = c_imag;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut iter = 0;

        while iter < max_iter {
            // Check escape condition: |z|² > 4
            if x * x + y * y > 4.0 {
                break;
            }

            // Tippets formula - order matters!
            // Update x first, THEN use the new x to calculate y
            x = x * x - y * y + a;  // x gets updated with old x and old y
            y = 2.0 * x * y + b;    // y uses the NEW x that was just calculated!
            
            iter += 1;
        }

        iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        view.center_x = -0.5;
        view.center_y = 0.0;
        view.zoom = 0.8;
        view
    }

    fn name(&self) -> &str {
        "Tippets Mandelbrot"
    }

    // Tippets Mandelbrot has no parameters, so we use the default empty Vec
}
