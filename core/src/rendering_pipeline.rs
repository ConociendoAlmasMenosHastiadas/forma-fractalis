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
use crate::rendering::{apply_colors_from_cache, FractalIterations};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct PtRenderReport {
    pub total_pixels: usize,
    pub pt_pixels: usize,
    pub fallback_pixels: usize,
    pub fallback_pct: f64,
    pub sa_accepted_pixel_count: usize,
    pub sa_accepted_pixel_pct: f64,
    pub sa_avg_skipped_iterations: f64,
    pub sa_max_skipped_iterations: usize,
    pub sa_rejected_escape_margin_count: usize,
    pub sa_rejected_correction_count: usize,
    pub rebased_pixel_count: usize,
    pub rebased_pixel_pct: f64,
    pub rebase_count: usize,
    pub rebase_exhausted_count: usize,
    pub pt_tiles: u32,
    pub min_orbit_len: usize,
    pub orbit_exhausted: bool,
    pub low_zoom_warning: bool,
    pub hiprec_bits: u32,
    pub max_iterations: u32,
}

fn pt_report_store() -> &'static RwLock<Option<PtRenderReport>> {
    static STORE: OnceLock<RwLock<Option<PtRenderReport>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(None))
}

fn set_last_pt_report(report: PtRenderReport) {
    if let Ok(mut guard) = pt_report_store().write() {
        *guard = Some(report);
    }
}

fn clear_last_pt_report() {
    if let Ok(mut guard) = pt_report_store().write() {
        *guard = None;
    }
}

pub fn latest_pt_report() -> Option<PtRenderReport> {
    pt_report_store().read().ok().and_then(|guard| guard.clone())
}

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
    pub color_offset: u32,
    pub fractal: &'a dyn Fractal,
    pub fractal_parameters: HashMap<String, f64>,
    pub backend: RenderBackend,
    /// Bit width used when backend == CpuHiPrec. One of: 64, 128, 256, 512, 1024.
    pub hiprec_bits: u32,
    /// Maximum rayon threads for CPU rendering. 0 = use all available (rayon default).
    /// Values 1..N limit parallelism to reduce CPU load during background work.
    pub max_threads: usize,
    /// Multiplier on the PT glitch threshold: `|dz|² > pt_glitch_tolerance * |z_total|²`.
    /// 1.0 = mathematically strict (default). Higher values reduce hi-prec fallback
    /// count at the cost of slight color inaccuracy in deep-zoom regions.
    pub pt_glitch_tolerance: f64,
    /// NxN tile grid for perturbation theory reference orbits.
    /// 1 = single reference orbit, 2 = 2x2 = 4 orbits, etc.
    pub pt_tiles: u32,
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
            color_offset: 0,
            fractal,
            fractal_parameters: HashMap::new(),
            backend: RenderBackend::default(),
            hiprec_bits: crate::gpu::HIPREC_DEFAULT_BITS,
            max_threads: 0,
            pt_glitch_tolerance: 1.0,
            pt_tiles: 1,
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

    /// Builder pattern: set PT glitch tolerance multiplier.
    /// 1.0 = strict (default). Higher values reduce hi-prec fallbacks at the
    /// cost of slight color inaccuracy.
    pub fn with_pt_glitch_tolerance(mut self, tolerance: f64) -> Self {
        self.pt_glitch_tolerance = tolerance;
        self
    }

    /// Builder pattern: set the PT tile grid size.
    /// Values < 1 are clamped to 1.
    pub fn with_pt_tiles(mut self, pt_tiles: u32) -> Self {
        self.pt_tiles = pt_tiles.max(1);
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

    /// Builder pattern: set colormap phase offset
    pub fn with_color_offset(mut self, color_offset: u32) -> Self {
        self.color_offset = color_offset;
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

fn target_dimensions(config: &RenderConfig, target: RenderTarget) -> (u32, u32) {
    match target {
        RenderTarget::Preview => (config.view.width, config.view.height),
        RenderTarget::Export { width, height } => (width, height),
    }
}

fn target_view_for(config: &RenderConfig, target: RenderTarget) -> FractalView {
    let (width, height) = target_dimensions(config, target);
    let mut target_view = config.view.clone();
    target_view.width = width;
    target_view.height = height;
    target_view
}

fn effective_cache_bits(config: &RenderConfig) -> u32 {
    if matches!(config.backend, RenderBackend::CpuHiPrec | RenderBackend::Perturbation) {
        config.hiprec_bits
    } else {
        0
    }
}

fn build_iteration_cache(
    config: &RenderConfig,
    target_view: &FractalView,
    iterations: Vec<u32>,
) -> FractalIterations {
    FractalIterations::from_render_state(
        iterations,
        target_view,
        config.max_iterations,
        config.fractal.name(),
        config.fractal_parameters.clone(),
        config.backend,
        effective_cache_bits(config),
        config.pt_glitch_tolerance,
        config.pt_tiles,
    )
}

fn scaled_orbit_params(config: &RenderConfig, target_view: &FractalView) -> HashMap<String, f64> {
    let mut params = config.fractal_parameters.clone();
    let preview_pixels = (config.view.width as f64) * (config.view.height as f64);
    let target_pixels = (target_view.width as f64) * (target_view.height as f64);
    if target_pixels > preview_pixels && preview_pixels > 0.0 {
        let scale = target_pixels / preview_pixels;
        let base_samples = params.get("samples").copied().unwrap_or(5_000_000.0);
        params.insert("samples".to_string(), (base_samples * scale).round());
    }
    params
}

fn finalize_pt_iteration_cache(
    config: &RenderConfig,
    target_view: &FractalView,
    orbits: &[crate::perturbation::ReferenceOrbit],
    result: crate::perturbation::PerturbationResult,
) -> FractalIterations {
    let pt_tiles = config.pt_tiles.max(1);
    let orbit_exhausted = orbits.iter().any(|o| o.escaped);
    let min_orbit_len = orbits.iter().map(|o| o.orbit.len()).min().unwrap_or(0);

    let total_pixels = (target_view.width * target_view.height) as usize;
    let pt_pixels = total_pixels.saturating_sub(result.glitch_count);
    let hiprec_pixels = result.glitch_count;
    let hiprec_pct = if total_pixels > 0 {
        hiprec_pixels as f64 / total_pixels as f64 * 100.0
    } else {
        0.0
    };
    let sa_accepted_pixel_pct = if total_pixels > 0 {
        result.sa_accepted_pixel_count as f64 / total_pixels as f64 * 100.0
    } else {
        0.0
    };
    let sa_avg_skipped_iterations = if result.sa_accepted_pixel_count > 0 {
        result.sa_total_skipped_iterations as f64 / result.sa_accepted_pixel_count as f64
    } else {
        0.0
    };
    let rebased_pixel_pct = if total_pixels > 0 {
        result.rebased_pixel_count as f64 / total_pixels as f64 * 100.0
    } else {
        0.0
    };

    set_last_pt_report(PtRenderReport {
        total_pixels,
        pt_pixels,
        fallback_pixels: hiprec_pixels,
        fallback_pct: hiprec_pct,
        sa_accepted_pixel_count: result.sa_accepted_pixel_count,
        sa_accepted_pixel_pct,
        sa_avg_skipped_iterations,
        sa_max_skipped_iterations: result.sa_max_skipped_iterations,
        sa_rejected_escape_margin_count: result.sa_rejected_escape_margin_count,
        sa_rejected_correction_count: result.sa_rejected_correction_count,
        rebased_pixel_count: result.rebased_pixel_count,
        rebased_pixel_pct,
        rebase_count: result.rebase_count,
        rebase_exhausted_count: result.rebase_exhausted_count,
        pt_tiles,
        min_orbit_len,
        orbit_exhausted,
        low_zoom_warning: result.low_zoom_warning,
        hiprec_bits: config.hiprec_bits,
        max_iterations: config.max_iterations,
    });

    perf_log!(
        "[PT] Pixels: {} total | {} PT delta ({:.1}%) | {} {}-bit hi-prec ({:.2}%) | SA px={} ({:.2}%) avg skip={:.1} max={} guards(e={}, c={}) | rebased px={} ({:.2}%) | rebases={} | budget-hit fallback={} | tiles={}x{} | shortest orbit len={}",
        total_pixels, pt_pixels, 100.0 - hiprec_pct,
        hiprec_pixels, config.hiprec_bits, hiprec_pct,
        result.sa_accepted_pixel_count, sa_accepted_pixel_pct,
        sa_avg_skipped_iterations, result.sa_max_skipped_iterations,
        result.sa_rejected_escape_margin_count, result.sa_rejected_correction_count,
        result.rebased_pixel_count, rebased_pixel_pct,
        result.rebase_count,
        result.rebase_exhausted_count,
        pt_tiles, pt_tiles, min_orbit_len
    );

    if orbit_exhausted {
        perf_log!(
            "[PT] NOTE: at least one reference orbit escaped early (shortest orbit len={}, max_iter={}). \
             This view may have a high fallback rate; try increasing tile count or switching to CPU Hi-Prec.",
            min_orbit_len,
            config.max_iterations,
        );
    }

    if result.low_zoom_warning {
        perf_log!(
            "[PT] Warning: zoom={:.2e} is below the PT threshold; CPU mode is sufficient",
            target_view.zoom
        );
    }

    build_iteration_cache(config, target_view, result.iterations)
}

pub fn colorize_iteration_cache_with_config(
    config: &RenderConfig,
    iterations: &FractalIterations,
) -> Vec<u8> {
    let mut buffer = vec![0u8; (iterations.width * iterations.height * 4) as usize];
    apply_colors_from_cache(
        &mut buffer,
        &iterations.data,
        config.colormap,
        config.max_iterations,
        config.use_period,
        config.period,
        config.use_interior_color,
        config.interior_color,
        config.use_log_scale,
        config.color_offset,
    );
    buffer
}

fn log_preview_render(
    config: &RenderConfig,
    iterations: &FractalIterations,
    compute_time: std::time::Duration,
    color_time: std::time::Duration,
    total_time: std::time::Duration,
) {
    if matches!(config.backend, RenderBackend::Perturbation) {
        let pt_tiles = config.pt_tiles.max(1);
        perf_log!(
            "[PERF] PT Render {}x{} @ {}bit/{} iter tiles={}x{}: total={:.2?} (compute={:.2?}, color={:.2?})",
            iterations.width,
            iterations.height,
            config.hiprec_bits,
            config.max_iterations,
            pt_tiles,
            pt_tiles,
            total_time,
            compute_time,
            color_time,
        );
        return;
    }

    if config.fractal.uses_orbit_accumulation() {
        if matches!(config.backend, RenderBackend::CpuHiPrec) {
            perf_log!(
                "[PERF] HiPrec Orbit Accumulation {}x{} @ {} iter ({} bits): total={:.2?} (compute={:.2?}, color={:.2?})",
                iterations.width,
                iterations.height,
                config.max_iterations,
                config.hiprec_bits,
                total_time,
                compute_time,
                color_time,
            );
        } else {
            perf_log!(
                "[PERF] Orbit Accumulation Render {}x{} @ {} iter: total={:.2?} (compute={:.2?}, color={:.2?})",
                iterations.width,
                iterations.height,
                config.max_iterations,
                total_time,
                compute_time,
                color_time,
            );
        }
        return;
    }

    match config.backend {
        RenderBackend::Cpu => perf_log!(
            "[PERF] CPU Render {}x{} @ {} iter: total={:.2?} (compute={:.2?}, color={:.2?})",
            iterations.width,
            iterations.height,
            config.max_iterations,
            total_time,
            compute_time,
            color_time,
        ),
        RenderBackend::CpuHiPrec => perf_log!(
            "[PERF] Hi-Prec CPU Render {}bit {}x{} @ {} iter: total={:.2?} (compute={:.2?}, color={:.2?})",
            config.hiprec_bits,
            iterations.width,
            iterations.height,
            config.max_iterations,
            total_time,
            compute_time,
            color_time,
        ),
        RenderBackend::Gpu => perf_log!(
            "[PERF] GPU Render {}x{} @ {} iter: total={:.2?} (compute={:.2?}, color={:.2?})",
            iterations.width,
            iterations.height,
            config.max_iterations,
            total_time,
            compute_time,
            color_time,
        ),
        RenderBackend::Perturbation => unreachable!("PT preview logging handled above"),
    }
}

#[cfg(feature = "gpu")]
pub fn compute_iteration_cache_with_config(
    config: &RenderConfig,
    target: RenderTarget,
    gpu_renderer: Option<&mut crate::gpu::WgpuRenderer>,
) -> Result<FractalIterations, String> {
    use crate::gpu::{self, FractalRenderer, RenderConfig as GpuRenderConfig};
    use crate::perturbation::{is_supported_fractal, ReferenceOrbit, render_perturbation_tiled};

    if !matches!(config.backend, gpu::RenderBackend::Perturbation) {
        clear_last_pt_report();
    }

    let target_view = target_view_for(config, target);

    if config.fractal.uses_orbit_accumulation() {
        if matches!(config.backend, gpu::RenderBackend::CpuHiPrec) {
            if !config.fractal.supports_hiprec() {
                return Err(format!(
                    "Hi-precision not supported for fractal '{}'. Switch to CPU mode.",
                    config.fractal.name()
                ));
            }

            let params = scaled_orbit_params(config, &target_view);
            let iterations = crate::orbit_accumulation::compute_orbit_density_hiprec(
                &target_view,
                config.fractal,
                &params,
                config.max_iterations,
                config.max_threads,
                config.hiprec_bits,
            )?;
            return Ok(build_iteration_cache(config, &target_view, iterations));
        }

        if matches!(config.backend, gpu::RenderBackend::Gpu) {
            if let Some(gpu) = gpu_renderer.as_deref() {
                if gpu.supports_orbit_density(config.fractal.name()) {
                    let params = scaled_orbit_params(config, &target_view);
                    let raw_density = crate::gpu::FractalRenderer::render_orbit_density(
                        gpu,
                        target_view.width,
                        target_view.height,
                        &params,
                        config.fractal.name(),
                        target_view.center_x as f32,
                        target_view.center_y as f32,
                        target_view.zoom as f32,
                    )?;
                    let use_log = params.get("use_log_density").copied().unwrap_or(1.0) > 0.5;
                    let iterations = crate::orbit_accumulation::DensityBuffer::from_raw(
                        target_view.width,
                        target_view.height,
                        raw_density,
                    )
                    .normalize(config.max_iterations, use_log);
                    return Ok(build_iteration_cache(config, &target_view, iterations));
                }
            }
        }

        let params = scaled_orbit_params(config, &target_view);
        let iterations = crate::orbit_accumulation::compute_orbit_density(
            &target_view,
            config.fractal,
            &params,
            config.max_iterations,
            config.max_threads,
        );
        return Ok(build_iteration_cache(config, &target_view, iterations));
    }

    if matches!(config.backend, gpu::RenderBackend::Perturbation) {
        if !is_supported_fractal(config.fractal, &config.fractal_parameters) {
            return Err(format!(
                "Perturbation Theory is only supported for Mandelbrot (power=2). \
                 Fractal '{}' is not supported. Switch to CPU or CPU Hi-Prec.",
                config.fractal.name()
            ));
        }

        let pt_tiles = config.pt_tiles.max(1);
        let tiles = pt_tiles as usize;
        let orbits = ReferenceOrbit::compute_tile_orbits(
            &target_view,
            pt_tiles,
            config.max_iterations,
            config.hiprec_bits,
        );

        let result = render_perturbation_tiled(
            &target_view,
            &orbits,
            tiles,
            tiles,
            config.fractal,
            &config.fractal_parameters,
            config.max_iterations,
            config.hiprec_bits,
            config.max_threads,
            config.pt_glitch_tolerance,
        );

        return Ok(finalize_pt_iteration_cache(config, &target_view, &orbits, result));
    }

    if matches!(config.backend, gpu::RenderBackend::CpuHiPrec) {
        let iterations = crate::rendering::compute_iterations_hiprec(
            &target_view,
            config.max_iterations,
            config.fractal,
            &config.fractal_parameters,
            config.hiprec_bits,
            config.max_threads,
        )?;
        return Ok(build_iteration_cache(config, &target_view, iterations));
    }

    let use_gpu = matches!(config.backend, gpu::RenderBackend::Gpu)
        && gpu_renderer.as_deref().is_some_and(|gpu| gpu.supports_fractal(config.fractal.name()));

    if use_gpu {
        let param_values: Vec<f64> = config.fractal.parameters()
            .iter()
            .map(|p| config.fractal_parameters.get(&p.name).copied().unwrap_or(p.default))
            .collect();
        let gpu_config = GpuRenderConfig {
            center_x: target_view.center_x,
            center_y: target_view.center_y,
            zoom: target_view.zoom,
            max_iter: config.max_iterations,
            width: target_view.width,
            height: target_view.height,
            fractal_params: param_values,
        };
        let gpu = gpu_renderer.ok_or_else(|| {
            "GPU requested but not initialized. Switch to CPU mode.".to_string()
        })?;
        let iterations = gpu.render_iterations(&gpu_config, config.fractal)?;
        return Ok(build_iteration_cache(config, &target_view, iterations));
    } else if matches!(config.backend, gpu::RenderBackend::Gpu) {
        if gpu_renderer.is_none() {
            return Err("GPU requested but not initialized. Switch to CPU mode.".to_string());
        }
        return Err(format!(
            "GPU does not support fractal: {}. Switch to CPU mode.",
            config.fractal.name()
        ));
    }

    let iterations = crate::rendering::compute_iterations(
        &target_view,
        config.max_iterations,
        config.fractal,
        &config.fractal_parameters,
    );
    Ok(build_iteration_cache(config, &target_view, iterations))
}

#[cfg(not(feature = "gpu"))]
pub fn compute_iteration_cache_with_config(
    config: &RenderConfig,
    target: RenderTarget,
) -> Result<FractalIterations, String> {
    use crate::perturbation::{is_supported_fractal, ReferenceOrbit, render_perturbation_tiled};

    if !matches!(config.backend, RenderBackend::Perturbation) {
        clear_last_pt_report();
    }

    let target_view = target_view_for(config, target);

    if config.fractal.uses_orbit_accumulation() {
        if matches!(config.backend, RenderBackend::CpuHiPrec) {
            if !config.fractal.supports_hiprec() {
                return Err(format!(
                    "Hi-precision not supported for fractal '{}'. Switch to CPU mode.",
                    config.fractal.name()
                ));
            }

            let params = scaled_orbit_params(config, &target_view);
            let iterations = crate::orbit_accumulation::compute_orbit_density_hiprec(
                &target_view,
                config.fractal,
                &params,
                config.max_iterations,
                config.max_threads,
                config.hiprec_bits,
            )?;
            return Ok(build_iteration_cache(config, &target_view, iterations));
        }

        let params = scaled_orbit_params(config, &target_view);
        let iterations = crate::orbit_accumulation::compute_orbit_density(
            &target_view,
            config.fractal,
            &params,
            config.max_iterations,
            config.max_threads,
        );
        return Ok(build_iteration_cache(config, &target_view, iterations));
    }

    if matches!(config.backend, RenderBackend::Perturbation) {
        if !is_supported_fractal(config.fractal, &config.fractal_parameters) {
            return Err(format!(
                "Perturbation Theory is only supported for Mandelbrot (power=2). \
                 Fractal '{}' is not supported. Switch to CPU or CPU Hi-Prec.",
                config.fractal.name()
            ));
        }

        let pt_tiles = config.pt_tiles.max(1);
        let tiles = pt_tiles as usize;
        let orbits = ReferenceOrbit::compute_tile_orbits(
            &target_view,
            pt_tiles,
            config.max_iterations,
            config.hiprec_bits,
        );

        let result = render_perturbation_tiled(
            &target_view,
            &orbits,
            tiles,
            tiles,
            config.fractal,
            &config.fractal_parameters,
            config.max_iterations,
            config.hiprec_bits,
            config.max_threads,
            config.pt_glitch_tolerance,
        );

        return Ok(finalize_pt_iteration_cache(config, &target_view, &orbits, result));
    }

    if matches!(config.backend, RenderBackend::CpuHiPrec) {
        let iterations = crate::rendering::compute_iterations_hiprec(
            &target_view,
            config.max_iterations,
            config.fractal,
            &config.fractal_parameters,
            config.hiprec_bits,
            config.max_threads,
        )?;
        return Ok(build_iteration_cache(config, &target_view, iterations));
    }

    let iterations = crate::rendering::compute_iterations(
        &target_view,
        config.max_iterations,
        config.fractal,
        &config.fractal_parameters,
    );
    Ok(build_iteration_cache(config, &target_view, iterations))
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
    let total_timer = Instant::now();
    let compute_timer = Instant::now();
    let iterations = compute_iteration_cache_with_config(config, target, gpu_renderer)?;
    let compute_time = compute_timer.elapsed();
    let color_timer = Instant::now();
    let buffer = colorize_iteration_cache_with_config(config, &iterations);
    let color_time = color_timer.elapsed();

    if matches!(target, RenderTarget::Preview) {
        log_preview_render(config, &iterations, compute_time, color_time, total_timer.elapsed());
    }

    Ok(buffer)
}

/// Unified rendering function when GPU feature is disabled
#[cfg(not(feature = "gpu"))]
pub fn render_with_config(
    config: &RenderConfig,
    target: RenderTarget,
) -> Result<Vec<u8>, String> {
    let total_timer = Instant::now();
    let compute_timer = Instant::now();
    let iterations = compute_iteration_cache_with_config(config, target)?;
    let compute_time = compute_timer.elapsed();
    let color_timer = Instant::now();
    let buffer = colorize_iteration_cache_with_config(config, &iterations);
    let color_time = color_timer.elapsed();

    if matches!(target, RenderTarget::Preview) {
        log_preview_render(config, &iterations, compute_time, color_time, total_timer.elapsed());
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scala_chromatica::ColorMap;
    use crate::fractals::{Mandelbrot, Zubieta};
    use std::collections::HashMap;

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
    fn test_colorize_iteration_cache_respects_color_offset() {
        let view = FractalView::new(2, 2);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();

        let config0 = RenderConfig::new(view.clone(), &colormap, 64, &mandelbrot)
            .with_period(true, 16)
            .with_color_offset(0);
        let config4 = RenderConfig::new(view.clone(), &colormap, 64, &mandelbrot)
            .with_period(true, 16)
            .with_color_offset(4);

        let cache = build_iteration_cache(&config0, &view, vec![1, 2, 3, 4]);
        let buffer0 = colorize_iteration_cache_with_config(&config0, &cache);
        let buffer4 = colorize_iteration_cache_with_config(&config4, &cache);

        assert_ne!(buffer0, buffer4, "color_offset should change colorized output");
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

    #[test]
    fn test_perturbation_backend_renders_with_tiles() {
        let mut view = FractalView::new(64, 64);
        view.center_x = -0.75;
        view.center_y = 0.1;
        view.zoom = 1.0e6;

        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();
        let config = RenderConfig::new(view, &colormap, 128, &mandelbrot)
            .with_backend(crate::gpu::RenderBackend::Perturbation)
            .with_hiprec_bits(64)
            .with_max_threads(1)
            .with_pt_tiles(2);

        #[cfg(feature = "gpu")]
        let buffer = render_with_config(&config, RenderTarget::Preview, None).expect("PT render failed");
        #[cfg(not(feature = "gpu"))]
        let buffer = render_with_config(&config, RenderTarget::Preview).expect("PT render failed");

        assert_eq!(buffer.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_perturbation_backend_rejects_unsupported_fractal() {
        let view = FractalView::new(32, 32);
        let colormap = ColorMap::default_scheme();
        let zubieta = Zubieta::new();
        let config = RenderConfig::new(view, &colormap, 64, &zubieta)
            .with_backend(crate::gpu::RenderBackend::Perturbation);

        #[cfg(feature = "gpu")]
        let err = render_with_config(&config, RenderTarget::Preview, None)
            .expect_err("unsupported fractal should fail through the full render pipeline");
        #[cfg(not(feature = "gpu"))]
        let err = render_with_config(&config, RenderTarget::Preview)
            .expect_err("unsupported fractal should fail through the full render pipeline");

        assert!(err.contains("Perturbation Theory is only supported for Mandelbrot (power=2)"));
        assert!(err.contains("Fractal 'Zubieta' is not supported"));
    }

    #[test]
    fn test_perturbation_backend_rejects_unsupported_power() {
        let view = FractalView::new(32, 32);
        let colormap = ColorMap::default_scheme();
        let mandelbrot = Mandelbrot::new();
        let mut params = HashMap::new();
        params.insert("power".to_string(), 3.0);

        let config = RenderConfig::new(view, &colormap, 64, &mandelbrot)
            .with_backend(crate::gpu::RenderBackend::Perturbation)
            .with_fractal_parameters(params);

        #[cfg(feature = "gpu")]
        let err = render_with_config(&config, RenderTarget::Preview, None)
            .expect_err("unsupported power should fail through the full render pipeline");
        #[cfg(not(feature = "gpu"))]
        let err = render_with_config(&config, RenderTarget::Preview)
            .expect_err("unsupported power should fail through the full render pipeline");

        assert!(err.contains("Perturbation Theory is only supported for Mandelbrot (power=2)"));
        assert!(err.contains("Fractal 'Mandelbrot' is not supported"));
    }
}
