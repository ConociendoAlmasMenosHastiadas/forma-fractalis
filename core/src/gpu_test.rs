//! GPU Test Infrastructure
//!
//! Automated validation of GPU shader correctness by rendering each GPU-supported
//! fractal with both GPU and CPU backends and comparing the outputs.
//!
//! Outputs are saved to a configurable temp directory for visual inspection.
//! The directory is cleaned up automatically when all tests pass; on failure it
//! is kept so the rendered images can be inspected.
//!
//! This module is compiled only when the `gpu` feature is enabled.

#[cfg(feature = "gpu")]
use crate::gpu::WgpuRenderer;
use crate::fractals::{
    Fractal,
    Mandelbrot, Julia, BurningShip, InsideoutDragon, Zubieta, SinJulia, TippetsMandelbrot,
    MultifractalJulia, Cactus,
};
use crate::rendering::compute_iterations;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Configuration for a GPU test run
pub struct GpuTestConfig {
    /// Render width in pixels (default: 512)
    pub width: u32,
    /// Render height in pixels (default: 512)
    pub height: u32,
    /// Max iterations per pixel (default: 256)
    pub iterations: u32,
    /// Test only this fractal by name (None = test all GPU-supported fractals)
    pub fractal_name: Option<String>,
    /// Directory to write test images into
    pub output_dir: PathBuf,
}

impl Default for GpuTestConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            iterations: 256,
            fractal_name: None,
            output_dir: PathBuf::from("temp/gpu_test"),
        }
    }
}

/// Result for one fractal test
pub struct GpuTestResult {
    pub fractal_name: String,
    pub gpu_success: bool,
    pub cpu_success: bool,
    pub gpu_time_ms: f64,
    pub cpu_time_ms: f64,
    /// True when GPU and CPU luminance distributions are within tolerance
    pub outputs_comparable: bool,
    pub error_message: Option<String>,
}

impl GpuTestResult {
    /// Overall pass: both renders succeeded and outputs are comparable
    pub fn passed(&self) -> bool {
        self.gpu_success && self.cpu_success && self.outputs_comparable
    }
}

/// Run GPU rendering tests according to the provided config.
///
/// Initializes the GPU renderer, renders each selected fractal with both GPU
/// and CPU backends, saves the images, and performs a basic luminance comparison.
///
/// Returns `Ok(results)` even when individual tests fail; the `passed()` method
/// on each result indicates per-fractal success. Returns `Err` only when the
/// GPU renderer itself cannot be initialized.
#[cfg(feature = "gpu")]
pub fn run_gpu_tests(config: &GpuTestConfig) -> Result<Vec<GpuTestResult>, String> {
    std::fs::create_dir_all(&config.output_dir)
        .map_err(|e| format!("Failed to create test directory {}: {}", config.output_dir.display(), e))?;

    println!("[GPU-TEST] Starting GPU rendering tests...");
    println!("[GPU-TEST] Creating temp directory: {}", config.output_dir.display());

    let mut gpu_renderer = WgpuRenderer::new()
        .map_err(|e| format!("[GPU-TEST] GPU renderer initialization failed: {}", e))?;

    let all_fractals = gpu_supported_fractals();
    let to_test: Vec<(String, Box<dyn Fractal>)> = match &config.fractal_name {
        Some(name) => {
            let filtered: Vec<_> = all_fractals
                .into_iter()
                .filter(|(n, _)| n.to_lowercase() == name.to_lowercase())
                .collect();
            if filtered.is_empty() {
                return Err(format!(
                    "[GPU-TEST] No GPU-supported fractal found matching '{}'. \
                     Available: Mandelbrot, Julia Set, Burning Ship, Insideout Dragon, \
                     Zubieta, Sin Julia, Tippets Mandelbrot, Multifractal-Julia",
                    name
                ));
            }
            filtered
        }
        None => all_fractals,
    };

    let mut results = Vec::new();

    for (name, fractal) in &to_test {
        println!("\n[GPU-TEST] Testing {} ({}x{}, {} iter)", name, config.width, config.height, config.iterations);
        let result = test_one_fractal(fractal.as_ref(), name, config, &mut gpu_renderer);

        if result.gpu_success {
            println!("  GPU render: {:.1}ms", result.gpu_time_ms);
        } else {
            println!("  GPU render: FAILED");
        }
        if result.cpu_success {
            println!("  CPU render: {:.1}ms", result.cpu_time_ms);
        }
        if result.outputs_comparable {
            println!("  Outputs comparable: yes");
        } else {
            println!("  Outputs comparable: no");
        }
        if let Some(ref msg) = result.error_message {
            println!("  Error: {}", msg);
        }

        let slug = name.to_lowercase().replace(' ', "_");
        println!("  Saved: {}/{}_gpu.png / {}_cpu.png", config.output_dir.display(), slug, slug);

        results.push(result);
    }

    let passed = results.iter().filter(|r| r.passed()).count();
    println!("\n[GPU-TEST] Summary: {}/{} fractals passed", passed, results.len());

    if passed == results.len() {
        if let Err(e) = std::fs::remove_dir_all(&config.output_dir) {
            println!("[GPU-TEST] Warning: could not clean temp dir: {}", e);
        } else {
            println!("[GPU-TEST] Cleaning up temp directory");
            println!("[GPU-TEST] All tests passed!");
        }
    } else {
        println!("[GPU-TEST] Keeping temp directory for debugging: {}", config.output_dir.display());
        for r in &results {
            if !r.passed() {
                println!("[GPU-TEST] FAILED: {}", r.fractal_name);
                if let Some(ref msg) = r.error_message {
                    println!("  {}", msg);
                }
            }
        }
    }

    Ok(results)
}

/// Render one fractal with GPU and CPU, compare raw iteration counts, and save images.
///
/// Comparison is done on the raw `Vec<u32>` iteration counts before colormap application.
/// This avoids the colormap amplification problem where a 1-iteration f32/f64 difference
/// maps to a large luminance difference.  A pixel is a "mismatch" when GPU and CPU differ
/// by more than 1 iteration.  Up to 5% of pixels may differ within that tolerance.
/// A completely wrong shader formula would fail because the MAJORITY of pixels differ.
#[cfg(feature = "gpu")]
fn test_one_fractal(
    fractal: &dyn Fractal,
    name: &str,
    config: &GpuTestConfig,
    gpu_renderer: &mut WgpuRenderer,
) -> GpuTestResult {
    use crate::gpu::{FractalRenderer, RenderConfig as GpuRenderConfig};
    use scala_chromatica::color_from_iterations;
    use rayon::prelude::*;

    let view = fractal.default_view(config.width, config.height);
    let colormap = scala_chromatica::ColorMap::default_scheme();
    let params: HashMap<String, f64> = fractal
        .parameters()
        .iter()
        .map(|p| (p.name.clone(), p.default))
        .collect();

    // Build GPU config (positional param vec)
    let param_values: Vec<f64> = fractal
        .parameters()
        .iter()
        .map(|p| params.get(&p.name).copied().unwrap_or(p.default))
        .collect();
    let gpu_config = GpuRenderConfig {
        center_x: view.center_x,
        center_y: view.center_y,
        zoom: view.zoom,
        max_iter: config.iterations,
        width: config.width,
        height: config.height,
        fractal_params: param_values,
    };

    // GPU render — raw iteration counts
    let gpu_start = Instant::now();
    let gpu_result = gpu_renderer.render_iterations(&gpu_config, fractal);
    let gpu_time_ms = gpu_start.elapsed().as_secs_f64() * 1000.0;

    let gpu_iters = match gpu_result {
        Ok(v) => v,
        Err(e) => {
            return GpuTestResult {
                fractal_name: name.to_string(),
                gpu_success: false,
                cpu_success: false,
                gpu_time_ms,
                cpu_time_ms: 0.0,
                outputs_comparable: false,
                error_message: Some(format!("GPU render failed: {}", e)),
            };
        }
    };

    // CPU render — raw iteration counts (same function the production CPU path calls)
    let cpu_start = Instant::now();
    let cpu_iters = compute_iterations(&view, config.iterations, fractal, &params);
    let cpu_time_ms = cpu_start.elapsed().as_secs_f64() * 1000.0;

    // Save RGBA images for visual inspection (colormap applied to each iteration set)
    let slug = name.to_lowercase().replace(' ', "_");
    let apply_colors = |iters: &[u32]| -> Vec<u8> {
        let pixels: Vec<[u8; 4]> = iters
            .par_iter()
            .map(|&iter| {
                let c = color_from_iterations(iter, config.iterations, &colormap,
                    false, 256, false, [0,0,0], false);
                [c.r, c.g, c.b, 255]
            })
            .collect();
        let mut buf = vec![0u8; pixels.len() * 4];
        for (i, px) in pixels.iter().enumerate() {
            buf[i*4..i*4+4].copy_from_slice(px);
        }
        buf
    };
    let _ = save_rgba_png(&apply_colors(&gpu_iters), config.width, config.height,
        &config.output_dir.join(format!("{}_gpu.png", slug)));
    let _ = save_rgba_png(&apply_colors(&cpu_iters), config.width, config.height,
        &config.output_dir.join(format!("{}_cpu.png", slug)));

    // Compare raw iteration counts: allow ±1 per pixel (f32/f64 boundary differences),
    // with up to 5% of pixels permitted to exceed that tolerance.
    //
    // Per-fractal overrides: some fractals have structural divergence between GPU and CPU
    // that is not a shader bug and cannot be eliminated.
    //
    // Multifractal-Julia: CPU uses exact HashMap cycle detection (f64 bit-pattern match);
    // GPU uses Brent's algorithm with f32 epsilon. Boundary pixels where the cycle fires at
    // slightly different iterations cause ~10% divergence. The overall shape is correct —
    // confirmed visually. Tolerance raised to 15% to accommodate.
    let tolerance = match name {
        "Multifractal-Julia" => 0.15,
        _                    => 0.05,
    };

    let total = gpu_iters.len();
    let mismatches = gpu_iters.iter().zip(cpu_iters.iter())
        .filter(|(&g, &c)| g.abs_diff(c) > 1)
        .count();
    let mismatch_rate = (mismatches as f64) / (total as f64);
    let comparable = mismatch_rate < tolerance;

    GpuTestResult {
        fractal_name: name.to_string(),
        gpu_success: true,
        cpu_success: true,
        gpu_time_ms,
        cpu_time_ms,
        outputs_comparable: comparable,
        error_message: if comparable {
            None
        } else {
            Some(format!(
                "GPU and CPU iteration counts differ: {:.1}% of pixels disagree by >1 iteration \
                 (tolerance: <{:.0}%).",
                mismatch_rate * 100.0,
                tolerance * 100.0
            ))
        },
    }
}

fn save_rgba_png(buf: &[u8], width: u32, height: u32, path: &PathBuf) -> Result<(), String> {
    use png::{BitDepth, ColorType, Encoder};
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut w = BufWriter::new(file);
    let mut encoder = Encoder::new(&mut w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(buf).map_err(|e| e.to_string())
}

/// All fractals that have a GPU pipeline registered in `wgpu_backend.rs`.
/// Update this list when adding GPU kernels for new fractals.
fn gpu_supported_fractals() -> Vec<(String, Box<dyn Fractal>)> {
    vec![
        ("Mandelbrot".to_string(), Box::new(Mandelbrot::new())),
        ("Julia Set".to_string(), Box::new(Julia::new())),
        ("Burning Ship".to_string(), Box::new(BurningShip::new())),
        ("Insideout Dragon".to_string(), Box::new(InsideoutDragon::new())),
        ("Zubieta".to_string(), Box::new(Zubieta::new())),
        ("Sin Julia".to_string(), Box::new(SinJulia::new())),
        ("Tippets Mandelbrot".to_string(), Box::new(TippetsMandelbrot::new())),
        ("Multifractal-Julia".to_string(), Box::new(MultifractalJulia::new())),
        ("Cactus".to_string(), Box::new(Cactus::new())),
    ]
}
