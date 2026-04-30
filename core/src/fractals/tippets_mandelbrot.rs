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

use super::{Fractal, FractalView, Parameter};
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
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let a = c_real;
        let b = c_imag;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut iter = 0;
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_sq = escape_r * escape_r;

        while iter < max_iter {
            // Check escape condition
            if x * x + y * y > escape_sq {
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

    fn equation(&self) -> &str {
        // Cannot be expressed as a standard complex map; the x component is
        // updated in-place and the new value feeds immediately into y:
        //   x = x^2 - y^2 + a
        //   y = 2*x_new*y + b
        "x = x^2 - y^2 + a,  y = 2*x_new*y + b"
    }

    // Tippets Mandelbrot parameters
    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "escape_radius".to_string(),
                label: "Escape Radius".to_string(),
                default: 2.0,
                min: 0.5,
                max: 100.0,
                description: "Escape radius: iterate escapes when |z| > escape_radius.".to_string(),
            },
        ]
    }
}

