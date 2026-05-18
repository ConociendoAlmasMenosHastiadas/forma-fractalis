use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use crate::number_utils::ABSOLUTE_EPSILON;
use std::collections::HashMap;

/// Multifractal-Julia: A variant fractal with inverse squared iteration
/// 
/// The general form is: z_{n+1} = c^k * z_n^{-2} + c
/// where k is the power parameter (can be negative, zero, or positive).
/// 
/// Starting condition: z_0 = c
/// 
/// Different values of k produce different behaviors:
/// - k = 0: z_{n+1} = z_n^{-2} + c
/// - k = 1: z_{n+1} = c * z_n^{-2} + c (default)
/// - k = 2: z_{n+1} = c^2 * z_n^{-2} + c
/// - k = -1: z_{n+1} = c^{-1} * z_n^{-2} + c
/// 
/// The series behavior is studied for periodicity - the period is typically mapped to color.
pub struct MultifractalJulia {
    power: f64,
}

impl MultifractalJulia {
    pub fn new() -> Self {
        Self { power: 1.0 }
    }

    pub fn with_power(power: f64) -> Self {
        Self { power }
    }
}

impl Default for MultifractalJulia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for MultifractalJulia {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        parameters: &std::collections::HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let c = Complex64::new(c_real, c_imag);
        
        // Get power parameter (k in the formula z_{n+1} = c^k * z_{n-2} + c)
        let k = parameters.get("power").copied().unwrap_or(self.power);
        
        // Bailout threshold
        let bailout_squared = if k.abs() > 2.0 { 100.0 } else { 16.0 };
        
        // Precompute c^k if k is an integer, otherwise compute it once
        // Handle special cases for efficiency
        let c_power = if k == 0.0 {
            Complex64::new(1.0, 0.0) // c^0 = 1
        } else if k == 1.0 {
            c // c^1 = c
        } else if k == -1.0 {
            // c^-1 = 1/c
            if c.norm_sqr() < ABSOLUTE_EPSILON {
                // Avoid division by near-zero
                return 0;
            }
            Complex64::new(1.0, 0.0) / c
        } else if k == 2.0 {
            c * c // c^2
        } else if k.fract() == 0.0 {
            // Integer power - use powi
            c.powi(k as i32)
        } else {
            // Non-integer power - use general complex power
            // c^k = exp(k * ln(c))
            // For complex numbers: ln(c) = ln(|c|) + i*arg(c)
            let r = c.norm();
            if r < ABSOLUTE_EPSILON {
                return 0;
            }
            let theta = c.arg();
            let ln_c = Complex64::new(r.ln(), theta);
            (k * ln_c).exp()
        };

        // Starting condition: z = c (like Mandelbrot starts effectively at c after first iteration)
        let mut z = c;

        // Cycle detection: track seen z values and their iteration numbers
        let mut seen: HashMap<(u64, u64), u32> = HashMap::new();
        
        // Helper to create hashable key from Complex64
        let to_key = |z: Complex64| (z.re.to_bits(), z.im.to_bits());

        for iteration in 0..max_iter {
            // Check bailout condition
            if z.norm_sqr() > bailout_squared {
                return iteration;
            }

            // Check for cycle detection
            let key = to_key(z);
            if let Some(&previous_iteration) = seen.get(&key) {
                // We've seen this z value before at previous_iteration
                // The orbit is periodic with period = iteration - previous_iteration
                let period = iteration - previous_iteration;
                
                // Return period to indicate cyclic behavior
                return period;
            }
            seen.insert(key, iteration);

            // Compute z_{n+1} = c^k * z_n^{-2} + c
            // z_n^{-2} = 1 / (z_n^2)
            let z_squared = z * z;
            if z_squared.norm_sqr() < ABSOLUTE_EPSILON {
                // Avoid division by near-zero
                return iteration;
            }
            let z_inv_squared = Complex64::new(1.0, 0.0) / z_squared;
            z = c_power * z_inv_squared + c;
        }

        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        FractalView {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 0.5,  // 0.5 zoom = 2x wider view (showing more of the complex plane)
            precise_center_x: None,
            precise_center_y: None,
            precise_zoom: None,
            width,
            height,
            parameters: self.parameter_defaults(),
        }
    }

    fn name(&self) -> &str {
        "Multifractal-Julia"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = c^k * z_n^(-2) + c"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "power",
                "Power",
                self.power,
                -5.0,
                5.0,
                "Power k in z_{n+1} = c^k * z_n^{-2} + c"
            )
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_multifractal_julia_origin() {
        let fractal = MultifractalJulia::new();
        let iterations = fractal.iterate(0.0, 0.0, &HashMap::new(), 100);
        // Origin: z=0, will hit division by zero check immediately
        assert!(iterations < 100);
    }

    #[test]
    fn test_multifractal_julia_far_point() {
        let fractal = MultifractalJulia::new();
        let iterations = fractal.iterate(10.0, 10.0, &HashMap::new(), 100);
        assert!(iterations < 100); // Far point should escape quickly
    }

    #[test]
    fn test_multifractal_julia_powers() {
        // Test different powers
        for power in &[-1.0, 0.0, 1.0, 2.0] {
            let fractal = MultifractalJulia::with_power(*power);
            let mut params = HashMap::new();
            params.insert("power".to_string(), *power);
            
            let iterations = fractal.iterate(0.5, 0.5, &params, 100);
            // Just verify it completes without panicking
            assert!(iterations <= 100);
        }
    }

    #[test]
    fn test_escape_behaviors() {
        let fractal = MultifractalJulia::new(); // k=1
        let params = HashMap::new();

        // Test points that should escape quickly
        let far_escape = fractal.iterate(5.0, 5.0, &params, 1000);
        println!("Far point (5+5i): escaped at iteration {}", far_escape);
        assert!(far_escape < 10, "Far point should escape very quickly");

        // Test a moderate point
        let moderate = fractal.iterate(1.0, 1.0, &params, 1000);
        println!("Moderate point (1+1i): escaped at iteration {}", moderate);

        // Test near-origin points
        let near_origin = fractal.iterate(0.1, 0.1, &params, 1000);
        println!("Near origin (0.1+0.1i): result = {}", near_origin);

        // Negative coordinates
        let negative = fractal.iterate(-0.5, -0.5, &params, 1000);
        println!("Negative point (-0.5-0.5i): result = {}", negative);
    }

    #[test]
    fn test_cycle_detection() {
        let fractal = MultifractalJulia::new();
        let params = HashMap::new();

        // Point at origin - z=0, will hit division by zero check
        let origin = fractal.iterate(0.0, 0.0, &params, 1000);
        println!("Origin: result = {} (should exit early due to division by zero)", origin);
        assert!(origin < 1000, "Origin should exit early");

        // Another potentially periodic point
        let maybe_periodic = fractal.iterate(0.2, 0.0, &params, 1000);
        println!("Point (0.2+0i): result = {}", maybe_periodic);
    }

    #[test]
    fn test_power_zero_behavior() {
        let mut params = HashMap::new();
        params.insert("power".to_string(), 0.0);
        let fractal = MultifractalJulia::with_power(0.0);

        // k=0: z_{n+1} = z_n^{-2} + c = (1/z_n^2) + c
        let result1 = fractal.iterate(0.5, 0.0, &params, 100);
        println!("k=0, c=0.5: result = {}", result1);

        let result2 = fractal.iterate(0.0, 0.5, &params, 100);
        println!("k=0, c=0.5i: result = {}", result2);

        let result3 = fractal.iterate(2.0, 0.0, &params, 100);
        println!("k=0, c=2.0: result = {}", result3);
    }

    #[test]
    fn test_power_negative_behavior() {
        let mut params = HashMap::new();
        params.insert("power".to_string(), -1.0);
        let fractal = MultifractalJulia::with_power(-1.0);

        // k=-1: z_{n+1} = c^{-1} * z_n^{-2} + c = (1/(c*z_n^2)) + c
        let result1 = fractal.iterate(1.0, 0.0, &params, 100);
        println!("k=-1, c=1.0: result = {}", result1);

        let result2 = fractal.iterate(0.5, 0.5, &params, 100);
        println!("k=-1, c=0.5+0.5i: result = {}", result2);

        let result3 = fractal.iterate(2.0, 1.0, &params, 100);
        println!("k=-1, c=2.0+1.0i: result = {}", result3);
    }

    #[test]
    fn test_power_two_behavior() {
        let mut params = HashMap::new();
        params.insert("power".to_string(), 2.0);
        let fractal = MultifractalJulia::with_power(2.0);

        // k=2: z_{n+1} = c^2 * z_n^{-2} + c = (c^2/z_n^2) + c
        let result1 = fractal.iterate(0.5, 0.0, &params, 100);
        println!("k=2, c=0.5: result = {}", result1);

        let result2 = fractal.iterate(0.3, 0.3, &params, 100);
        println!("k=2, c=0.3+0.3i: result = {}", result2);

        let result3 = fractal.iterate(1.0, 0.0, &params, 100);
        println!("k=2, c=1.0: result = {}", result3);
    }

    #[test]
    fn test_interesting_points() {
        let fractal = MultifractalJulia::new();
        let params = HashMap::new();

        // Test various interesting points in the complex plane
        let points = [
            (0.0, 0.0, "origin"),
            (1.0, 0.0, "real axis"),
            (0.0, 1.0, "imaginary axis"),
            (0.5, 0.5, "diagonal"),
            (-0.5, 0.5, "quadrant 2"),
            (-0.5, -0.5, "quadrant 3"),
            (0.5, -0.5, "quadrant 4"),
            (0.1, 0.0, "small real"),
            (0.0, 0.1, "small imaginary"),
        ];

        println!("\n=== Multifractal-Julia Escape Patterns (k=1) ===");
        for (re, im, label) in points.iter() {
            let iterations = fractal.iterate(*re, *im, &params, 500);
            let description = if iterations < 10 {
                format!("period {}", iterations)
            } else if iterations == 500 {
                format!("escaped after max_iter")
            } else {
                format!("escaped at {}", iterations)
            };
            println!("{:20} ({:6.2} + {:6.2}i): {}", label, re, im, description);
        }
    }
}
