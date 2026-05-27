//! Zubieta Fractal Implementation
//!
//! The Zubieta set: z(n+1) = z(n)² + c/z(n)
//! A Julia set variant where c is a constant and z starts at each pixel position.
//!
//! This fractal requires division by z, so we need to guard against z=0.
//! If at any iteration z becomes zero, we treat it as escaped.
//!
//! Reference: https://paulbourke.net/fractals/Zubieta/

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;

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
        let escape_r = parameters.get("escape_radius").copied().unwrap_or(2.0);
        let escape_sq = escape_r * escape_r;

        while iter < max_iter {
            if z.norm_sqr() > escape_sq {
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
            Parameter::new(
                "escape_radius",
                "Escape Radius",
                2.0,
                0.5,
                100.0,
                "Escape radius: iterate escapes when |z| > escape_radius."
            ),
        ]
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
