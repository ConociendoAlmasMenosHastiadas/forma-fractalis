//! Animated GIF generation for fractals
//!
//! This module provides functionality for creating animated GIFs showing:
//! - Zoom sequences
//! - Julia set parameter sweeps
//! - Colormap transitions
//! - Iteration count fade-ins

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::fractals::{FractalView, Fractal};
use crate::perf_log;
use scala_chromatica::ColorMap;

/// Type of animation to generate
#[derive(Debug, Clone)]
pub enum AnimationType {
    /// Zoom in/out from current position
    Zoom {
        from_zoom: f64,
        to_zoom: f64,
        center_x: f64,
        center_y: f64,
    },
    
    /// Sweep Julia set constant parameter
    JuliaParamSweep {
        from_c_real: f64,
        from_c_imag: f64,
        to_c_real: f64,
        to_c_imag: f64,
    },
    
    /// Transition between two colormaps
    ColormapTransition {
        from_colormap: String,
        to_colormap: String,
    },
    
    /// Fade in by increasing iteration count
    IterationFade {
        from_iterations: u32,
        to_iterations: u32,
    },
}

/// Configuration for animation generation
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Type of animation
    pub animation_type: AnimationType,
    
    /// Number of frames to generate
    pub num_frames: u32,
    
    /// Frames per second (target playback speed)
    pub fps: u8,
    
    /// Output file path for the GIF
    pub output_path: PathBuf,
    
    /// Resolution (width x height)
    pub width: u32,
    pub height: u32,
}

impl AnimationConfig {
    /// Create a new animation configuration
    pub fn new(
        animation_type: AnimationType,
        num_frames: u32,
        fps: u8,
        output_path: PathBuf,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            animation_type,
            num_frames,
            fps,
            output_path,
            width,
            height,
        }
    }
    
    /// Calculate delay between frames in centiseconds (1/100th of a second)
    pub fn frame_delay_cs(&self) -> u16 {
        (100.0 / self.fps as f64).round() as u16
    }
}

/// Generate an animated GIF based on the configuration
///
/// # Arguments
/// * `config` - Animation configuration
/// * `view` - Base fractal view parameters
/// * `colormap` - Base colormap to use
/// * `max_iterations` - Base iteration count
/// * `use_period` - Whether colormap period modulation is enabled
/// * `period` - Colormap period value
/// * `use_interior_color` - Whether custom interior color is enabled
/// * `interior_color` - RGB color for interior (non-escaping) points
/// * `use_log_scale` - Whether logarithmic color scaling is enabled
/// * `fractal` - The fractal to render
/// * `fractal_parameters` - Base fractal parameters
/// * `export_scale` - Scale factor for rendering (1.0 = no scaling)
/// * `export_filter` - Filter type for supersampling
/// * `export_supersample` - Supersample factor (1 = no supersampling)
/// * `cancel_token` - Optional cancellation flag; set to `true` to abort generation
/// * `progress_callback` - Optional callback for progress updates (frame_num, total_frames)
///
/// # Returns
/// Result with path to generated GIF or error message
pub fn generate_animation<F>(
    config: &AnimationConfig,
    view: &FractalView,
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    export_scale: f64,
    export_filter: crate::filtering::FilterType,
    export_supersample: u32,
    render_backend: crate::gpu::RenderBackend,
    cancel_token: Option<Arc<AtomicBool>>,
    progress_callback: Option<F>,
) -> Result<PathBuf, String>
where
    F: Fn(u32, u32),
{
    use image::{ImageBuffer, Rgba, codecs::gif::GifEncoder, Frame, Delay};
    use std::fs::File;
    use std::time::Instant;

    let anim_type_str = match &config.animation_type {
        AnimationType::Zoom { from_zoom, to_zoom, .. } => format!("Zoom {:.2}→{:.2}", from_zoom, to_zoom),
        AnimationType::JuliaParamSweep { from_c_real, from_c_imag, to_c_real, to_c_imag } =>
            format!("JuliaParamSweep ({:.4},{:.4})→({:.4},{:.4})", from_c_real, from_c_imag, to_c_real, to_c_imag),
        AnimationType::ColormapTransition { from_colormap, to_colormap } =>
            format!("ColormapTransition {}→{}", from_colormap, to_colormap),
        AnimationType::IterationFade { from_iterations, to_iterations } =>
            format!("IterationFade {}→{}", from_iterations, to_iterations),
    };
    perf_log!(
        "[ANIM] Starting: type={}, frames={}, fps={}, size={}x{}, backend={:?}, output={}",
        anim_type_str, config.num_frames, config.fps, config.width, config.height,
        render_backend, config.output_path.display()
    );
    let total_timer = Instant::now();

    // Initialize GPU renderer once for all frames if GPU mode is requested.
    // This matches the export path pattern - each independent render context
    // (CLI export, animation thread) creates its own renderer instance.
    #[cfg(feature = "gpu")]
    let mut gpu_renderer: Option<crate::gpu::WgpuRenderer> = if matches!(render_backend, crate::gpu::RenderBackend::Gpu) {
        match crate::gpu::WgpuRenderer::new() {
            Ok(r) => {
                perf_log!("[ANIM] GPU renderer initialized for animation thread");
                Some(r)
            }
            Err(e) => {
                perf_log!("[ANIM] GPU renderer initialization failed: {}", e);
                return Err(format!("Failed to initialize GPU for animation: {}", e));
            }
        }
    } else {
        None
    };

    // Create output file
    let file = File::create(&config.output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    // Create GIF encoder
    let mut encoder = GifEncoder::new(file);
    
    // Set repeat mode (0 = loop forever)
    encoder.set_repeat(image::codecs::gif::Repeat::Infinite)
        .map_err(|e| format!("Failed to set GIF repeat mode: {}", e))?;
    
    let frame_delay = Delay::from_saturating_duration(
        std::time::Duration::from_millis((1000.0 / config.fps as f64) as u64)
    );
    
    // Generate each frame
    for frame_idx in 0..config.num_frames {
        // Check for cancellation before rendering each frame
        if cancel_token.as_ref().map_or(false, |t| t.load(Ordering::Relaxed)) {
            // Remove partially-written GIF file on cancel
            drop(encoder);
            let _ = std::fs::remove_file(&config.output_path);
            perf_log!("[ANIM] Cancelled after {}/{} frames ({:.2?} elapsed)", frame_idx, config.num_frames, total_timer.elapsed());
            return Err("Animation generation cancelled".to_string());
        }
        let frame_timer = Instant::now();
        // Calculate interpolation factor (0.0 to 1.0)
        let t = if config.num_frames > 1 {
            frame_idx as f64 / (config.num_frames - 1) as f64
        } else {
            0.0
        };
        
        // Interpolate parameters based on animation type
        let (frame_view, frame_colormap, frame_max_iter, frame_params) = 
            interpolate_frame(config, view, colormap, max_iterations, fractal_parameters, t)?;
        
        // Render frame using the rendering pipeline with export settings
        let frame_buffer = render_frame(
            &frame_view,
            &frame_colormap,
            frame_max_iter,
            use_period,
            period,
            use_interior_color,
            interior_color,
            use_log_scale,
            fractal,
            &frame_params,
            export_scale,
            export_filter,
            export_supersample,
            render_backend,
            #[cfg(feature = "gpu")]
            gpu_renderer.as_mut(),
        )?;
        
        // Convert to ImageBuffer
        let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            config.width,
            config.height,
            frame_buffer,
        ).ok_or_else(|| "Failed to create image buffer from frame data".to_string())?;
        
        // Create frame and encode
        let frame = Frame::from_parts(img_buffer, 0, 0, frame_delay);
        encoder.encode_frame(frame)
            .map_err(|e| format!("Failed to encode frame {}: {}", frame_idx, e))?;

        // Build a per-type parameter summary for easy debugging
        let type_info = match &config.animation_type {
            AnimationType::Zoom { .. } =>
                format!("zoom={:.6} center=({:.8},{:.8})", frame_view.zoom, frame_view.center_x, frame_view.center_y),
            AnimationType::JuliaParamSweep { .. } => {
                let cr = frame_params.get("c_real").copied().unwrap_or(0.0);
                let ci = frame_params.get("c_imag").copied().unwrap_or(0.0);
                format!("c=({:.6},{:.6})", cr, ci)
            }
            AnimationType::ColormapTransition { .. } =>
                format!("t={:.4}", t),
            AnimationType::IterationFade { .. } =>
                format!("iter={}", frame_max_iter),
        };
        perf_log!(
            "[ANIM] Frame {}/{} ({:.2?}) | {} | iter={} period={}/{} interior={} log={} backend={:?}",
            frame_idx + 1, config.num_frames, frame_timer.elapsed(),
            type_info,
            frame_max_iter,
            use_period, period,
            use_interior_color,
            use_log_scale,
            render_backend
        );

        // Progress callback
        if let Some(ref callback) = progress_callback {
            callback(frame_idx + 1, config.num_frames);
        }
    }

    perf_log!("[ANIM] Complete: {} frames in {:.2?}, saved to {}",
        config.num_frames, total_timer.elapsed(), config.output_path.display());
    Ok(config.output_path.clone())
}

/// Interpolate frame parameters based on animation type and time
fn interpolate_frame(
    config: &AnimationConfig,
    base_view: &FractalView,
    base_colormap: &ColorMap,
    base_max_iter: u32,
    base_params: &HashMap<String, f64>,
    t: f64,
) -> Result<(FractalView, ColorMap, u32, HashMap<String, f64>), String> {
    let mut view = base_view.clone();
    let colormap = base_colormap.clone(); // TODO: handle colormap transitions
    let mut max_iter = base_max_iter;
    let mut params = base_params.clone();
    
    match &config.animation_type {
        AnimationType::Zoom { from_zoom, to_zoom, center_x, center_y } => {
            view.center_x = *center_x;
            view.center_y = *center_y;
            // Logarithmic interpolation: each frame multiplies zoom by the same
            // ratio, so every step looks visually identical in terms of how much
            // of the plane is revealed. Linear interpolation would make early
            // frames crawl and late frames rocket (or vice-versa).
            view.zoom = lerp_zoom(*from_zoom, *to_zoom, t);
        }
        
        AnimationType::JuliaParamSweep { from_c_real, from_c_imag, to_c_real, to_c_imag } => {
            let c_real = lerp(*from_c_real, *to_c_real, t);
            let c_imag = lerp(*from_c_imag, *to_c_imag, t);
            params.insert("c_real".to_string(), c_real);
            params.insert("c_imag".to_string(), c_imag);
        }
        
        AnimationType::ColormapTransition { from_colormap: _, to_colormap: _ } => {
            // TODO: Implement colormap interpolation
            return Err("Colormap transitions not yet implemented".to_string());
        }
        
        AnimationType::IterationFade { from_iterations, to_iterations } => {
            max_iter = lerp_u32(*from_iterations, *to_iterations, t);
        }
    }
    
    Ok((view, colormap, max_iter, params))
}

/// Linear interpolation between two f64 values
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Logarithmic interpolation between two zoom values.
///
/// Zoom is a multiplicative scale, so uniform steps in log space mean each
/// frame multiplies/divides by the same ratio: `zoom(t) = a * (b/a)^t`.
/// This is equivalent to linear interpolation in log space:
/// `exp(lerp(ln(a), ln(b), t))`.
///
/// Requires both `a` and `b` to be positive (zoom values always are).
fn lerp_zoom(a: f64, b: f64, t: f64) -> f64 {
    a * (b / a).powf(t)
}

/// Linear interpolation between two u32 values
fn lerp_u32(a: u32, b: u32, t: f64) -> u32 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u32
}

/// Render a single frame using the rendering pipeline
fn render_frame(
    view: &FractalView,
    colormap: &ColorMap,
    max_iterations: u32,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
    use_log_scale: bool,
    fractal: &dyn Fractal,
    fractal_parameters: &HashMap<String, f64>,
    export_scale: f64,
    export_filter: crate::filtering::FilterType,
    export_supersample: u32,
    render_backend: crate::gpu::RenderBackend,
    #[cfg(feature = "gpu")]
    gpu_renderer: Option<&mut crate::gpu::WgpuRenderer>,
) -> Result<Vec<u8>, String> {
    use crate::rendering_pipeline::{RenderConfig, RenderTarget, render_with_config};
    use crate::export::calculate_output_dimensions;
    use crate::filtering::calculate_supersample_dimensions;

    // Calculate target dimensions from scale factor (matches PNG export behavior).
    // AnimationConfig.width/height are set to these same scaled dimensions in
    // start_animation_generation, so ImageBuffer::from_raw will always match.
    let (target_width, target_height) = calculate_output_dimensions(view, export_scale as f32);

    // Supersample for quality: render at a higher resolution then downsample.
    let supersample = if export_filter == crate::filtering::FilterType::None {
        1
    } else {
        export_supersample.max(1)
    };
    let (render_width, render_height) = calculate_supersample_dimensions(
        target_width,
        target_height,
        supersample,
    );
    
    let config = RenderConfig::new(view.clone(), colormap, max_iterations, fractal)
        .with_fractal_parameters(fractal_parameters.clone())
        .with_period(use_period, period)
        .with_interior_color(use_interior_color, interior_color)
        .with_log_scale(use_log_scale)
        .with_backend(render_backend);
    
    // Render at supersample resolution
    let target = RenderTarget::Export {
        width: render_width,
        height: render_height,
    };
    
    // Render at supersample resolution
    #[cfg(feature = "gpu")]
    let mut buffer = render_with_config(&config, target, gpu_renderer)?;
    
    #[cfg(not(feature = "gpu"))]
    let mut buffer = render_with_config(&config, target)?;
    
    // Apply supersampling filter if needed (downsample to target dimensions)
    if supersample > 1 {
        buffer = crate::filtering::apply_supersample_filter(
            &buffer,
            render_width,
            render_height,
            target_width,
            target_height,
            export_filter,
        )?;
    }
    
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::fractals::{FractalView, Mandelbrot};
    use scala_chromatica::ColorMap;
    
    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }
    
    #[test]
    fn test_lerp_u32() {
        assert_eq!(lerp_u32(100, 200, 0.0), 100);
        assert_eq!(lerp_u32(100, 200, 1.0), 200);
        assert_eq!(lerp_u32(100, 200, 0.5), 150);
    }
    
    #[test]
    fn test_frame_delay() {
        let config = AnimationConfig::new(
            AnimationType::Zoom {
                from_zoom: 1.0,
                to_zoom: 10.0,
                center_x: 0.0,
                center_y: 0.0,
            },
            30,
            10,
            PathBuf::from("test.gif"),
            800,
            600,
        );
        
        // 10 fps = 100ms = 10 centiseconds
        assert_eq!(config.frame_delay_cs(), 10);
    }

    /// Verify that color modulation settings actually produce different pixel output.
    ///
    /// This is a regression test for the bug where `use_period`, `use_interior_color`,
    /// and `use_log_scale` were never threaded through `generate_animation` →
    /// `render_frame` → `RenderConfig`, so every frame silently used defaults
    /// (period off, no interior color, no log scale) regardless of what the user set.
    #[test]
    fn test_color_settings_affect_frame_output() {
        let fractal = Mandelbrot::new();
        let view = FractalView::new(32, 18);

        // Use first built-in colormap
        let colormap = ColorMap::default_scheme();

        let params = HashMap::new();

        // Render one frame with interior color OFF (default black [0,0,0])
        #[cfg(feature = "gpu")]
        let frame_default = render_frame(
            &view, &colormap, 100,
            false, 256,      // use_period, period
            false, [0, 0, 0], // use_interior_color, interior_color
            false,           // use_log_scale
            &fractal, &params,
            1.0, crate::filtering::FilterType::None, 1,
            crate::gpu::RenderBackend::Cpu, None,
        ).expect("render with default color settings");

        #[cfg(not(feature = "gpu"))]
        let frame_default = render_frame(
            &view, &colormap, 100,
            false, 256,
            false, [0, 0, 0],
            false,
            &fractal, &params,
            1.0, crate::filtering::FilterType::None, 1,
            crate::gpu::RenderBackend::Cpu,
        ).expect("render with default color settings");

        // Render same frame with a bright interior color (should change pixels in
        // the interior/non-escaping region of the Mandelbrot set)
        #[cfg(feature = "gpu")]
        let frame_interior = render_frame(
            &view, &colormap, 100,
            false, 256,
            true, [255, 0, 128], // interior color: hot pink
            false,
            &fractal, &params,
            1.0, crate::filtering::FilterType::None, 1,
            crate::gpu::RenderBackend::Cpu, None,
        ).expect("render with interior color");

        #[cfg(not(feature = "gpu"))]
        let frame_interior = render_frame(
            &view, &colormap, 100,
            false, 256,
            true, [255, 0, 128],
            false,
            &fractal, &params,
            1.0, crate::filtering::FilterType::None, 1,
            crate::gpu::RenderBackend::Cpu,
        ).expect("render with interior color");

        assert_eq!(frame_default.len(), frame_interior.len(),
            "Both frames must be same byte length");

        // The two renders must differ — the Mandelbrot interior region exists at
        // default zoom, so interior color changes pixel values.
        assert_ne!(frame_default, frame_interior,
            "Interior color setting had no effect on frame pixels. \
             This indicates use_interior_color is not reaching RenderConfig.");
    }

    /// Verify that zoom interpolation is logarithmic (geometric progression).
    ///
    /// With from_zoom=1.0 and to_zoom=100.0:
    /// - t=0   → zoom = 1.0
    /// - t=0.5 → zoom = 10.0  (geometric mean: sqrt(1 * 100), NOT linear 50.5)
    /// - t=1   → zoom = 100.0
    #[test]
    fn test_zoom_animation_interpolates_zoom() {
        let view = FractalView::new(32, 18);
        let colormap = ColorMap::default_scheme();
        let params = HashMap::new();

        let config = AnimationConfig::new(
            AnimationType::Zoom {
                from_zoom: 1.0,
                to_zoom: 100.0,
                center_x: view.center_x,
                center_y: view.center_y,
            },
            2, 10,
            PathBuf::from("test_zoom.gif"),
            view.width, view.height,
        );

        let (frame_start, _, _, _) =
            interpolate_frame(&config, &view, &colormap, 100, &params, 0.0)
                .expect("interpolate t=0");
        let (frame_mid, _, _, _) =
            interpolate_frame(&config, &view, &colormap, 100, &params, 0.5)
                .expect("interpolate t=0.5");
        let (frame_end, _, _, _) =
            interpolate_frame(&config, &view, &colormap, 100, &params, 1.0)
                .expect("interpolate t=1");

        assert!((frame_start.zoom - 1.0).abs() < 1e-10,
            "t=0 zoom should be 1.0, got {}", frame_start.zoom);
        assert!((frame_end.zoom - 100.0).abs() < 1e-10,
            "t=1 zoom should be 100.0, got {}", frame_end.zoom);

        // Geometric mean of 1 and 100 is 10, not linear mean 50.5
        assert!((frame_mid.zoom - 10.0).abs() < 1e-10,
            "t=0.5 zoom should be geometric mean 10.0, got {} (linear would be 50.5)", frame_mid.zoom);
    }

    /// Verify that iteration fade produces the correct iteration count per frame.
    #[test]
    fn test_iteration_fade_interpolates_correctly() {
        let view = FractalView::new(32, 18);
        let colormap = ColorMap::default_scheme();
        let params = HashMap::new();

        let config = AnimationConfig::new(
            AnimationType::IterationFade { from_iterations: 10, to_iterations: 200 },
            2, 10,
            PathBuf::from("test_iter.gif"),
            view.width, view.height,
        );

        let (_, _, iter_start, _) =
            interpolate_frame(&config, &view, &colormap, 100, &params, 0.0)
                .expect("interpolate t=0");
        let (_, _, iter_end, _) =
            interpolate_frame(&config, &view, &colormap, 100, &params, 1.0)
                .expect("interpolate t=1");

        assert_eq!(iter_start, 10);
        assert_eq!(iter_end, 200);
    }
}

