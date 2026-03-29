//! Unified Rendering Pipeline
//!
//! This module provides a unified rendering system used by both preview and export.
//! It eliminates code duplication and ensures consistent behavior across all render paths.
//!
//! # Architecture
//! - `RenderConfig`: All parameters needed for rendering
//! - `RenderTarget`: Where the render is going (preview texture or export file)
//! - `render_with_config()`: Single entry point for all rendering
//!
//! # Usage
//! ```ignore
//! let config = RenderConfig::new(view, colormap, max_iterations);
//! let buffer = render_with_config(&config, RenderTarget::Preview);
//! ```

use scala_chromatica::ColorMap;
use crate::fractals::{Fractal, FractalView};
use crate::gpu::RenderBackend;
use crate::perf_log;
use crate::rendering::{render_fractal, render_fractal_hiprec};
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for a render operation
#[derive(Clone)]
pub struct RenderConfig<'a> {
    pub view: FractalView,
    pub colormap: &'a ColorMap,
    pub max_iterations: u32,
    pub use_period: bool,
    pub period: u32,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub use_log_scale: bool,
    pub fractal: &'a dyn Fractal,
    pub fractal_parameters: HashMap<String, f64>,
    pub backend: RenderBackend,
    /// Bit width used when backend == CpuHiPrec. One of: 64, 128, 256, 512, 1024.
    pub hiprec_bits: u32,
    /// Maximum rayon threads for CPU rendering. 0 = use all available (rayon default).
    /// Values 1..N limit parallelism to reduce CPU load during background work.
    pub max_threads: usize,
}

impl<'a> RenderConfig<'a> {
    /// Create a new render configuration
    pub fn new(
        view: FractalView,
        colormap: &'a ColorMap,
        max_iterations: u32,
        fractal: &'a dyn Fractal,
    ) -> Self {
        Self {
            view,
            colormap,
            max_iterations,
            use_period: false,
            period: 256,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            fractal,
            fractal_parameters: HashMap::new(),
            backend: RenderBackend::default(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            max_threads: 0,
        }
    }

    /// Builder pattern: set rendering backend
    pub fn with_backend(mut self, backend: RenderBackend) -> Self {
        self.backend = backend;
        self
    }

    /// Builder pattern: set hi-precision bit width (used when backend == CpuHiPrec)
    pub fn with_hiprec_bits(mut self, bits: u32) -> Self {
        self.hiprec_bits = bits;
        self
    }

    /// Builder pattern: limit rayon thread count for CPU rendering.
    /// Pass 0 to use all available threads (rayon default).
    pub fn with_max_threads(mut self, max_threads: usize) -> Self {
        self.max_threads = max_threads;
        self
    }

    /// Builder pattern: set period modulation
    pub fn with_period(mut self, enabled: bool, period: u32) -> Self {
        self.use_period = enabled;
        self.period = period;
        self
    }

    /// Builder pattern: set interior color
    pub fn with_interior_color(mut self, enabled: bool, color: [u8; 3]) -> Self {
        self.use_interior_color = enabled;
        self.interior_color = color;
        self
    }

    /// Builder pattern: set logarithmic scaling
    pub fn with_log_scale(mut self, enabled: bool) -> Self {
        self.use_log_scale = enabled;
        self
    }

    /// Builder pattern: set fractal parameters
    pub fn with_fractal_parameters(mut self, parameters: HashMap<String, f64>) -> Self {
        self.fractal_parameters = parameters;
        self
    }
}

/// Target for rendering output
#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    /// Render for preview display
    Preview,
    /// Render for export at specified dimensions
    Export { width: u32, height: u32 },
}

/// Unified rendering function used by both preview and export
///
/// # Arguments
/// * `config` - Rendering configuration
/// * `target` - Render target (preview or export with dimensions)
/// * `gpu_renderer` - Optional GPU renderer (required if backend is GPU)
///
/// # Returns
/// RGBA buffer ready for use (texture upload or image encoding)
#[cfg(feature = "gpu")]
pub fn render_with_config(
    config: &RenderConfig,
    target: RenderTarget,
    gpu_renderer: Option<&mut crate::gpu::WgpuRenderer>,
) -> Result<Vec<u8>, String> {
    use crate::gpu::{FractalRenderer, self};
    
    let _total_timer = Instant::now();
    
    // Determine output dimensions based on target
    let (width, height) = match target {
        RenderTarget::Preview => (config.view.width, config.view.height),
        RenderTarget::Export { width, height } => (width, height),
    };

    // Create view with target dimensions (may differ from config.view for export)
    let mut target_view = config.view.clone();
    target_view.width = width;
    target_view.height = height;

    // Use the backend the user explicitly selected - no heuristics
    let use_gpu = matches!(config.backend, gpu::RenderBackend::Gpu) 
        && gpu_renderer.is_some()
        && gpu_renderer.as_ref().unwrap().supports_fractal(config.fractal.name());

    // CPU Hi-Prec path: dispatch before GPU/CPU check
    if matches!(config.backend, gpu::RenderBackend::CpuHiPrec) {
        return render_with_hiprec(config, &target_view, target, _total_timer);
    }

    if use_gpu {
        match render_with_gpu(config, &target_view, gpu_renderer.unwrap()) {
            Ok(buffer) => {
                let total_time = _total_timer.elapsed();
                if matches!(target, RenderTarget::Preview) {
                    perf_log!("[PERF] GPU Render {}x{} @ {} iter: total={:.2?}",
                        width, height, config.max_iterations, total_time);
                }
                return Ok(buffer);
            }
            Err(e) => {
                // Do NOT fall back to CPU silently - the user must choose to switch.
                // Silent fallback could run for minutes/hours at high iteration counts.
                return Err(format!("GPU rendering failed: {}. Switch to CPU mode if needed.", e));
            }
        }
    } else if matches!(config.backend, gpu::RenderBackend::Gpu) {
        // User requested GPU but it's not available
        if gpu_renderer.is_none() {
            return Err("GPU requested but not initialized. Switch to CPU mode.".to_string());
        } else if gpu_renderer.is_some() && !gpu_renderer.as_ref().unwrap().supports_fractal(config.fractal.name()) {
            return Err(format!("GPU does not support fractal: {}. Switch to CPU mode.", config.fractal.name()));
        }
    }

    // CPU rendering path
    Ok(render_with_cpu(config, &target_view, target, _total_timer))
}

/// CPU rendering path
#[cfg(feature = "gpu")]
fn render_with_cpu(
    config: &RenderConfig,
    target_view: &FractalView,
    target: RenderTarget,
    total_timer: Instant,
) -> Vec<u8> {
    let alloc_timer = Instant::now();
    let buffer_size = (target_view.width * target_view.height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];
    let alloc_time = alloc_timer.elapsed();

    let render_timer = Instant::now();
    render_fractal(
        &mut buffer,
        target_view,
        config.colormap,
        config.max_iterations,
        config.use_period,
        config.period,
        config.use_interior_color,
        config.interior_color,
        config.use_log_scale,
        config.fractal,
        &config.fractal_parameters,
        config.max_threads,
    );
    let render_time = render_timer.elapsed();

    if matches!(target, RenderTarget::Preview) {
        let total_time = total_timer.elapsed();
        perf_log!("[PERF] CPU Render {}x{} @ {} iter: total={:.2?} (alloc={:.2?}, render={:.2?})",
            target_view.width, target_view.height, config.max_iterations,
            total_time, alloc_time, render_time);
    }

    buffer
}

/// CPU hi-precision rendering path
#[cfg(feature = "gpu")]
fn render_with_hiprec(
    config: &RenderConfig,
    target_view: &FractalView,
    target: RenderTarget,
    total_timer: Instant,
) -> Result<Vec<u8>, String> {
    let buffer_size = (target_view.width * target_view.height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];

    render_fractal_hiprec(
        &mut buffer,
        target_view,
        config.colormap,
        config.max_iterations,
        config.use_period,
        config.period,
        config.use_interior_color,
        config.interior_color,
        config.use_log_scale,
        config.fractal,
        &config.fractal_parameters,
        config.hiprec_bits,
        config.max_threads,
    )?;

    if matches!(target, RenderTarget::Preview) {
        let total_time = total_timer.elapsed();
        perf_log!(
            "[PERF] Hi-Prec CPU Render {}bit {}x{} @ {} iter: total={:.2?}",
            config.hiprec_bits, target_view.width, target_view.height,
            config.max_iterations, total_time
        );
    }

    Ok(buffer)
}

/// GPU rendering path
#[cfg(feature = "gpu")]
fn render_with_gpu(
    config: &RenderConfig,
    target_view: &FractalView,
    gpu_renderer: &mut crate::gpu::WgpuRenderer,
) -> Result<Vec<u8>, String> {
    use crate::gpu::{FractalRenderer, RenderConfig as GpuRenderConfig};
    use scala_chromatica::color_from_iterations;
    use rayon::prelude::*;
    
    // Convert fractal parameters to vec
    let param_values: Vec<f64> = config.fractal.parameters()
        .iter()
        .map(|p| config.fractal_parameters.get(&p.name).copied().unwrap_or(p.default))
        .collect();
    
    // Create GPU render config
    let gpu_config = GpuRenderConfig {
        center_x: target_view.center_x,
        center_y: target_view.center_y,
        zoom: target_view.zoom,
        max_iter: config.max_iterations,
        width: target_view.width,
        height: target_view.height,
        fractal_params: param_values,
    };

    // Render iteration counts on GPU
    let iterations = gpu_renderer.render_iterations(&gpu_config, config.fractal)?;

    // Apply colormap on CPU using proper color_from_iterations (handles period, log scale, etc.)
    let buffer_size = (target_view.width * target_view.height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];
    
    // Use parallel processing for color application (same as CPU path)
    let pixels: Vec<[u8; 4]> = iterations
        .par_iter()
        .map(|&iter| {
            let color = color_from_iterations(
                iter,
                config.max_iterations,
                config.colormap,
                config.use_period,
                config.period,
                config.use_interior_color,
                config.interior_color,
                config.use_log_scale,
            );
            [color.r, color.g, color.b, 255]
        })
        .collect();
    
    // Copy computed pixels to frame buffer
    for (i, pixel) in pixels.iter().enumerate() {
        let idx = i * 4;
        buffer[idx..idx + 4].copy_from_slice(pixel);
    }

    Ok(buffer)
}

/// Unified rendering function when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub fn render_with_config(
    config: &RenderConfig,
    target: RenderTarget,
) -> Result<Vec<u8>, String> {
    let _total_timer = Instant::now();

    // Determine output dimensions based on target
    let (width, height) = match target {
        RenderTarget::Preview => (config.view.width, config.view.height),
        RenderTarget::Export { width, height } => (width, height),
    };

    // Create view with target dimensions (may differ from config.view for export)
    let mut target_view = config.view.clone();
    target_view.width = width;
    target_view.height = height;

    // CPU Hi-Prec path
    if matches!(config.backend, RenderBackend::CpuHiPrec) {
        let buffer_size = (width * height * 4) as usize;
        let mut buffer = vec![0u8; buffer_size];
        render_fractal_hiprec(
            &mut buffer,
            &target_view,
            config.colormap,
            config.max_iterations,
            config.use_period,
            config.period,
            config.use_interior_color,
            config.interior_color,
            config.use_log_scale,
            config.fractal,
            &config.fractal_parameters,
            config.hiprec_bits,
            config.max_threads,
        )?;
        if matches!(target, RenderTarget::Preview) {
            let total_time = _total_timer.elapsed();
            perf_log!(
                "[PERF] Hi-Prec CPU Render {}bit {}x{} @ {} iter: total={:.2?}",
                config.hiprec_bits, width, height, config.max_iterations, total_time
            );
        }
        return Ok(buffer);
    }

    // Allocate buffer
    let alloc_timer = Instant::now();
    let buffer_size = (width * height * 4) as usize;
    let mut buffer = vec![0u8; buffer_size];
    let alloc_time = alloc_timer.elapsed();

    // Render using fractal trait
    let render_timer = Instant::now();
    render_fractal(
        &mut buffer,
        &target_view,
        config.colormap,
        config.max_iterations,
        config.use_period,
        config.period,
        config.use_interior_color,
        config.interior_color,
        config.use_log_scale,
        config.fractal,
        &config.fractal_parameters,
        config.max_threads,
    );
    let render_time = render_timer.elapsed();

    // Performance logging (only for preview, to avoid spamming during export)
    if matches!(target, RenderTarget::Preview) {
        let total_time = _total_timer.elapsed();
        perf_log!("[PERF] Render {}x{} @ {} iter: total={:.2?} (alloc={:.2?}, render={:.2?})",
            width, height, config.max_iterations,
            total_time, alloc_time, render_time);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scala_chromatica::ColorMap;
    use crate::fractals::Mandelbrot;

    #[test]
    fn test_render_config_builder() {
        let view = FractalView::new(100, 100);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();

        let config = RenderConfig::new(view, &colormap, 256, &mandelbrot)
            .with_period(true, 128)
            .with_interior_color(true, [255, 0, 0])
            .with_log_scale(true)
            .with_backend(crate::gpu::RenderBackend::Cpu);

        assert!(config.use_period);
        assert_eq!(config.period, 128);
        assert!(config.use_interior_color);
        assert_eq!(config.interior_color, [255, 0, 0]);
        assert!(config.use_log_scale);
        assert!(matches!(config.backend, crate::gpu::RenderBackend::Cpu));
    }

    #[test]
    fn test_render_target_dimensions() {
        let view = FractalView::new(640, 480);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();
        let config = RenderConfig::new(view, &colormap, 128, &mandelbrot);

        // Preview uses view dimensions
        #[cfg(feature = "gpu")]
        let buffer = render_with_config(&config, RenderTarget::Preview, None).expect("render failed");
        #[cfg(not(feature = "gpu"))]
        let buffer = render_with_config(&config, RenderTarget::Preview).expect("render failed");
        assert_eq!(buffer.len(), 640 * 480 * 4);

        // Export uses specified dimensions
        #[cfg(feature = "gpu")]
        let buffer = render_with_config(&config, RenderTarget::Export { width: 1920, height: 1080 }, None).expect("render failed");
        #[cfg(not(feature = "gpu"))]
        let buffer = render_with_config(&config, RenderTarget::Export { width: 1920, height: 1080 }).expect("render failed");
        assert_eq!(buffer.len(), 1920 * 1080 * 4);
    }
}
