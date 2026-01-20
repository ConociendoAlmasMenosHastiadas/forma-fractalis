//! Mandelbrot Set Implementation
//!
//! The classic Mandelbrot set: z(n+1) = z(n)^power + c
//! where z starts at 0 and c is the point being tested.
//!
//! When power = 2, this is the classic Mandelbrot set.
//! Other powers create "Multibrot" or "Powerbrot" sets with different symmetries:
//! - power = 3: Tricorn symmetry
//! - power = 4: Quatric symmetry
//! - power = 5: Quintic symmetry

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;
use crate::number_utils::ABSOLUTE_EPSILON;

/// Mandelbrot set fractal with configurable power
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
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        
        // Get power parameter, default to 2 (classic Mandelbrot)
        let power = parameters.get("power").copied().unwrap_or(2.0);

        // Start at z₁ = c instead of z₀ = 0 to avoid singularity with negative powers
        // This is mathematically equivalent since z₁ = 0^power + c = c
        let mut z = c;
        let mut iter = 0;
        
        let is_negative_power = power < 0.0;
        let escape_radius_sqr = 4.0;
        let convergence_threshold_sqr = ABSOLUTE_EPSILON; // For detecting convergence to 0

        while iter < max_iter {
            let magnitude_sqr = z.norm_sqr();
            
            // Check for escape or convergence based on power sign
            if is_negative_power {
                // For negative powers: check if converging to zero or diverging
                if magnitude_sqr < convergence_threshold_sqr || magnitude_sqr > escape_radius_sqr {
                    break;
                }
            } else {
                // For positive powers: standard escape condition
                if magnitude_sqr > escape_radius_sqr {
                    break;
                }
            }

            // z = z^power + c
            // Optimize for power=2.0 (classic Mandelbrot) - use multiplication instead of powf
            z = if power == 2.0 {
                z * z + c
            } else {
                z.powf(power) + c
            };
            
            // Safety check: if z becomes NaN or infinite, bail out
            if !z.re.is_finite() || !z.im.is_finite() {
                break;
            }
            
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

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter {
                name: "power".to_string(),
                label: "Power".to_string(),
                default: 2.0,
                min: -10.0,
                max: 10.0,
                description: "Exponent in z^power + c formula. 2 = classic Mandelbrot. Negative powers create inverted sets.".to_string(),
            },
        ]
    }
}

impl Mandelbrot {
    /// Returns the sequence of all complex numbers generated during iteration
    /// 
    /// This function performs the same iteration as `iterate()` but instead of
    /// returning the escape count, it returns a Vec of all z values encountered
    /// during the iteration process. Useful for visualizing the iteration path.
    /// 
    /// # Arguments
    /// * `c_real` - Real component of the complex constant c
    /// * `c_imag` - Imaginary component of the complex constant c
    /// * `power` - Exponent in z^power + c formula (default 2.0 for classic Mandelbrot)
    /// * `max_iter` - Maximum number of iterations
    /// 
    /// # Returns
    /// Vec<Complex64> containing all z values from z₁ to the final iteration
    pub fn mandelseries(c_real: f64, c_imag: f64, power: f64, max_iter: u32) -> Vec<Complex64> {
        let c = Complex64::new(c_real, c_imag);
        
        // Start at z₁ = c instead of z₀ = 0 to avoid singularity with negative powers
        let mut z = c;
        let mut series = Vec::with_capacity(max_iter as usize + 1);
        
        // Always include the starting point (z₁ = c)
        series.push(z);
        
        let is_negative_power = power < 0.0;
        let escape_radius_sqr = 4.0;
        let convergence_threshold_sqr = ABSOLUTE_EPSILON;
        
        for _ in 0..max_iter {
            let magnitude_sqr = z.norm_sqr();
            
            // Check for escape or convergence based on power sign
            if is_negative_power {
                if magnitude_sqr < convergence_threshold_sqr || magnitude_sqr > escape_radius_sqr {
                    break;
                }
            } else {
                if magnitude_sqr > escape_radius_sqr {
                    break;
                }
            }
            
            // Optimize for power=2.0 (classic Mandelbrot)
            z = if power == 2.0 {
                z * z + c
            } else {
                z.powf(power) + c
            };
            
            // Safety check for NaN/infinity
            if !z.re.is_finite() || !z.im.is_finite() {
                break;
            }
            
            series.push(z);
        }
        
        series
    }
}
