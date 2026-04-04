//! Cactus Fractal
//!
//! The Cactus fractal is defined by the iteration:
//! z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0
//!
//! For every point z_0 in the complex plane, we iterate this function
//! and determine if the orbit escapes or remains bounded.
//!
//! Reference: https://paulbourke.net/fractals/cactus/

use crate::fractals::{Fractal, FractalView, Parameter};
use num_complex::Complex64;

/// Cactus fractal implementation
pub struct Cactus;

impl Cactus {
    pub fn new() -> Self {
        Cactus
    }
    
    /// Calculate the escape radius for a given z_0
    /// Using R > max(1, sqrt(abs(z_0-1)+1), abs(z0)^(1/3))
    fn escape_radius(z0: Complex64) -> f64 {
        let r1 = 1.0_f64;
        let r2 = ((z0 - Complex64::new(1.0, 0.0)).norm() + 1.0).sqrt();
        let r3 = z0.norm().powf(1.0 / 3.0);
        
        r1.max(r2).max(r3) * 2.0  // Add safety factor
    }
}

impl Default for Cactus {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Cactus {
    fn iterate(
        &self,
        c_real: f64,
        c_imag: f64,
        _parameters: &std::collections::HashMap<String, f64>,
        max_iter: u32,
    ) -> u32 {
        let z0 = Complex64::new(c_real, c_imag);
        let mut z = z0;
        
        // Calculate escape radius for this specific z_0
        let escape_radius = Self::escape_radius(z0);
        let escape_radius_sq = escape_radius * escape_radius;
        
        for i in 0..max_iter {
            // Check if escaped
            if z.norm_sqr() > escape_radius_sq {
                return i;
            }
            
            // z_{n+1} = z_n^3 + (z_0 - 1)z_n - z_0
            z = z.powi(3) + (z0 - Complex64::new(1.0, 0.0)) * z - z0;
            
            // Check for NaN or infinity
            if !z.re.is_finite() || !z.im.is_finite() {
                return i;
            }
        }
        
        // Did not escape
        max_iter
    }

    fn default_view(&self, width: u32, height: u32) -> FractalView {
        let mut view = FractalView::new(width, height);
        // Center view on an interesting region
        view.center_x = 0.0;
        view.center_y = 0.0;
        view.zoom = 0.6;  // Zoom out to see more structure
        view
    }

    fn name(&self) -> &str {
        "Cactus"
    }

    fn equation(&self) -> &str {
        "z_{n+1} = z_n^3 + (z_0 - 1)*z_n - z_0"
    }

    fn parameters(&self) -> Vec<Parameter> {
        // Cactus fractal has no adjustable parameters
        Vec::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cactus_origin() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        let iters = cactus.iterate(0.0, 0.0, &params, 100);
        // Origin should either escape or remain bounded
        assert!(iters <= 100);
    }

    #[test]
    fn test_cactus_far_point() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        // Point far from origin should escape quickly
        let iters = cactus.iterate(10.0, 10.0, &params, 100);
        assert!(iters < 10, "Far point should escape quickly");
    }

    #[test]
    fn test_cactus_various_points() {
        let cactus = Cactus::new();
        let params = HashMap::new();
        
        let test_points = vec![
            (0.5, 0.5),
            (-0.5, 0.5),
            (0.5, -0.5),
            (-0.5, -0.5),
            (1.0, 0.0),
            (0.0, 1.0),
        ];
        
        for (x, y) in test_points {
            let iters = cactus.iterate(x, y, &params, 256);
            // Just verify it doesn't panic and returns valid iteration count
            assert!(iters <= 256);
        }
    }

    #[test]
    fn test_cactus_escape_radius() {
        // Test that escape radius calculation doesn't panic
        let z1 = Complex64::new(0.0, 0.0);
        let z2 = Complex64::new(1.0, 1.0);
        let z3 = Complex64::new(10.0, 10.0);
        
        let r1 = Cactus::escape_radius(z1);
        let r2 = Cactus::escape_radius(z2);
        let r3 = Cactus::escape_radius(z3);
        
        assert!(r1 > 0.0);
        assert!(r2 > 0.0);
        assert!(r3 > 0.0);
        
        // Larger z0 should generally have larger escape radius
        assert!(r3 > r1);
    }

    #[test]
    fn test_cactus_name() {
        let cactus = Cactus::new();
        assert_eq!(cactus.name(), "Cactus");
    }

    #[test]
    fn test_cactus_parameters() {
        let cactus = Cactus::new();
        let params = cactus.parameters();
        assert_eq!(params.len(), 0, "Cactus fractal should have no parameters");
    }

    #[test]
    fn test_cactus_default_view() {
        let cactus = Cactus::new();
        let view = cactus.default_view(1280, 720);
        assert_eq!(view.width, 1280);
        assert_eq!(view.height, 720);
        assert_eq!(view.center_x, 0.0);
        assert_eq!(view.center_y, 0.0);
        assert_eq!(view.zoom, 0.6);
    }
}
