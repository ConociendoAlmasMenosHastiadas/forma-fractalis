//! Julia Set Implementation
//!
//! The Julia set: z(n+1) = z(n)² + c
//! where z starts at the pixel coordinate and c is a constant.
//! 
//! Unlike Mandelbrot (where c varies and z starts at 0),
//! Julia sets use a fixed c and z starts at each pixel position.

use super::{Fractal, FractalView, Parameter};
use num_complex::Complex64;
use std::collections::HashMap;

/// Classic Julia set coordinates that produce beautiful, interesting patterns
/// Format: (c_real, c_imag, name)
const CLASSIC_JULIA_COORDINATES: &[(f64, f64, &str)] = &[
    (-0.7, 0.27015, "Dendrite (Douady's Rabbit)"),
    (-0.4, 0.6, "Spiral"),
    (-0.8, 0.156, "Branching"),
    (0.285, 0.01, "Seahorse Tail"),
    (-0.70176, -0.3842, "Siegel Disk"),
    (0.285, 0.0, "Dragon"),
    (-0.835, -0.2321, "Swirls"),
    (-0.8, 0.156, "Lightning"),
];

/// Julia set fractal with configurable constant
pub struct Julia;

impl Julia {
    /// Creates a new Julia set fractal instance
    pub fn new() -> Self {
        Self
    }
    
    /// Returns a random classic Julia set coordinate for exploration
    /// 
    /// This helps users discover interesting Julia sets without needing
    /// to know specific coordinates in advance.
    pub fn random_classic_coordinates() -> (f64, f64) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Use current time as seed for simple randomization
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        
        let index = seed % CLASSIC_JULIA_COORDINATES.len();
        let (c_real, c_imag, _name) = CLASSIC_JULIA_COORDINATES[index];
        (c_real, c_imag)
    }
}

impl Default for Julia {
    fn default() -> Self {
        Self::new()
    }
}

impl Fractal for Julia {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        // Get Julia set constant from parameters (defaults to classic values)
        let julia_c_real = parameters.get("c_real").copied().unwrap_or(-0.7);
        let julia_c_imag = parameters.get("c_imag").copied().unwrap_or(0.27015);
        
        let c = Complex64::new(julia_c_real, julia_c_imag);
        let mut z = Complex64::new(c_real, c_imag);
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
        view.center_x = 0.0;
        view.center_y = 0.0;
        // Julia sets typically look good at zoom level around 0.7 to show the full set
        // This gives approximately -2 to 2 range in both axes
        view.zoom = 0.7;
        
        // Set random classic Julia constant for discovery
        let (c_real, c_imag) = Self::random_classic_coordinates();
        view.set_parameter("c_real", c_real);
        view.set_parameter("c_imag", c_imag);
        
        view
    }

    fn name(&self) -> &str {
        "Julia Set"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new(
                "c_real",
                "C Real Part",
                -0.7,
                -2.0,
                2.0,
                "Real component of the Julia set constant"
            ),
            Parameter::new(
                "c_imag",
                "C Imaginary Part",
                0.27015,
                -2.0,
                2.0,
                "Imaginary component of the Julia set constant"
            ),
        ]
    }
}
