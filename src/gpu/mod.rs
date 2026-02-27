//! GPU acceleration module for fractal rendering
//!
//! This module provides GPU-accelerated fractal rendering using compute shaders.
//! GPU support is optional and can be disabled at compile time with `--no-default-features`.

#[cfg(feature = "gpu")]
pub mod wgpu_backend;

#[cfg(feature = "gpu")]
pub use wgpu_backend::WgpuRenderer;

use crate::fractals::Fractal;

/// Maximum "safe" iterations for GPU compute shaders.
///
/// Windows TDR (Timeout Detection and Recovery) kills GPU shaders that run longer
/// than ~2 seconds (default TdrDelay). High iteration counts on complex fractals
/// can exceed this, causing device lost / process crash.
///
/// This limit is conservative for 1080p preview on a mid-range GPU. The actual
/// safe limit depends on GPU speed, resolution, and fractal complexity.
/// Export rendering (higher resolution) will hit this sooner.
///
/// Users can increase TdrDelay via registry if they need higher iterations on GPU:
///   HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\TdrDelay (DWORD, seconds)
pub const GPU_MAX_SAFE_ITERATIONS: u32 = 500_000;

/// Configuration for rendering a fractal
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub max_iter: u32,
    pub width: u32,
    pub height: u32,
    pub fractal_params: Vec<f64>,  // Fractal-specific parameters (e.g., power, Julia c)
}

/// Backend selection for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    /// CPU rendering using rayon parallelization
    Cpu,
    
    /// GPU rendering using compute shaders
    #[cfg(feature = "gpu")]
    Gpu,
}

impl Default for RenderBackend {
    fn default() -> Self {
        RenderBackend::Cpu
    }
}

impl RenderBackend {
    /// Get all available backends
    pub fn all() -> Vec<RenderBackend> {
        #[cfg(feature = "gpu")]
        {
            vec![RenderBackend::Cpu, RenderBackend::Gpu]
        }
        #[cfg(not(feature = "gpu"))]
        {
            vec![RenderBackend::Cpu]
        }
    }
    
    /// Get display name for the backend
    pub fn as_str(&self) -> &'static str {
        match self {
            RenderBackend::Cpu => "CPU",
            #[cfg(feature = "gpu")]
            RenderBackend::Gpu => "GPU",
        }
    }
}

/// Trait for fractal rendering backends
pub trait FractalRenderer {
    /// Render a fractal and return iteration counts for each pixel
    ///
    /// Returns a flat array of iteration counts in row-major order
    fn render_iterations(
        &mut self,
        config: &RenderConfig,
        fractal: &dyn Fractal,
    ) -> Result<Vec<u32>, String>;
    
    /// Check if this renderer supports the given fractal type
    fn supports_fractal(&self, fractal_name: &str) -> bool;
}
