//! Mandelbrot Set Mathematics and View Management
//!
//! This module provides the core mathematical functions for computing
//! the Mandelbrot set and managing the viewport state for navigation.
//!
//! # Key Components
//! - `MandelbrotView`: Viewport state (center, zoom, dimensions)
//! - `mandelbrot_iterations()`: Core iteration counting algorithm
//! - Navigation: zoom, pan, reset operations
/// - View state management

/// Represents the view parameters for rendering the Mandelbrot set
#[derive(Clone)]
pub struct MandelbrotView {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub width: u32,
    pub height: u32,
}

impl MandelbrotView {
    /// Creates a new MandelbrotView with default parameters
    /// centered on the classic Mandelbrot position
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            center_x: -0.5,
            center_y: 0.0,
            zoom: 1.0,
            width,
            height,
        }
    }

    /// Converts screen pixel coordinates to complex plane coordinates
    ///
    /// # Arguments
    /// * `x` - Pixel x-coordinate
    /// * `y` - Pixel y-coordinate
    ///
    /// # Returns
    /// A tuple (real, imaginary) representing the complex number
    pub fn screen_to_complex(&self, x: u32, y: u32) -> (f64, f64) {
        let aspect_ratio = self.width as f64 / self.height as f64;
        let scale = 3.5 / self.zoom;

        let real = self.center_x
            + (x as f64 - self.width as f64 / 2.0) * scale / self.width as f64 * aspect_ratio;
        let imag =
            self.center_y + (y as f64 - self.height as f64 / 2.0) * scale / self.height as f64;

        (real, imag)
    }

    /// Pans the view in the given direction
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let pan_amount = 0.1 / self.zoom;
        self.center_x += dx * pan_amount;
        self.center_y += dy * pan_amount;
    }

    /// Zooms in at a specific point on the screen
    pub fn zoom_at(&mut self, screen_x: u32, screen_y: u32, zoom_factor: f64) {
        let (click_real, click_imag) = self.screen_to_complex(screen_x, screen_y);
        self.center_x = click_real;
        self.center_y = click_imag;
        self.zoom *= zoom_factor;
    }

    /// Resets the view to default parameters
    pub fn reset(&mut self) {
        self.center_x = -0.5;
        self.center_y = 0.0;
        self.zoom = 1.0;
    }
}

/// Calculates the number of iterations before divergence for a complex number
///
/// # Arguments
/// * `c_real` - Real part of the complex number
/// * `c_imag` - Imaginary part of the complex number
/// * `max_iter` - Maximum number of iterations to test
///
/// # Returns
/// Number of iterations before |z| > 2, or max_iter if in the set
pub fn mandelbrot_iterations(c_real: f64, c_imag: f64, max_iter: u32) -> u32 {
    let mut z_real = 0.0;
    let mut z_imag = 0.0;
    let mut iter = 0;

    while iter < max_iter {
        let z_real_sq = z_real * z_real;
        let z_imag_sq = z_imag * z_imag;

        if z_real_sq + z_imag_sq > 4.0 {
            break;
        }

        let new_z_real = z_real_sq - z_imag_sq + c_real;
        let new_z_imag = 2.0 * z_real * z_imag + c_imag;

        z_real = new_z_real;
        z_imag = new_z_imag;
        iter += 1;
    }

    iter
}
