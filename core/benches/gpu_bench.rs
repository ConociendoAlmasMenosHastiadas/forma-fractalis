//! GPU Benchmark for Fractal Rendering
//!
//! Compares CPU vs GPU rendering performance for all GPU-capable fractals.
//! Run with: cargo bench --bench gpu_bench
//! Requires the `gpu` feature (enabled by default).

use forma_fractalis_core::{
    fractals::{
        Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia,
        Cactus, Zubieta, SinJulia, InsideoutDragon,
        FractalView, Fractal,
    },
    gpu::{WgpuRenderer, FractalRenderer, RenderConfig as GpuConfig, RenderBackend},
    rendering_pipeline::{render_with_config, RenderConfig, RenderTarget},
};
use std::time::Instant;

fn benchmark_cpu(fractal: &dyn Fractal, width: u32, height: u32, max_iter: u32, runs: usize) -> f64 {
    use scala_chromatica::ColorMap;
    use std::collections::HashMap;
    let view = FractalView::new(width, height);
    let colormap = ColorMap::default_scheme();
    let parameters = HashMap::new();
    let config = RenderConfig::new(view, &colormap, max_iter, fractal)
        .with_fractal_parameters(parameters)
        .with_backend(RenderBackend::Cpu);
    // Warmup
    for _ in 0..2 {
        let _ = render_with_config(&config, RenderTarget::Preview, None);
    }
    let mut times = Vec::with_capacity(runs);
    for _ in 0..runs {
        let t = Instant::now();
        let _ = render_with_config(&config, RenderTarget::Preview, None);
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    times.iter().sum::<f64>() / times.len() as f64
}

#[cfg(feature = "gpu")]
fn benchmark_gpu(
    renderer: &mut WgpuRenderer,
    fractal: &dyn Fractal,
    width: u32, height: u32, max_iter: u32, runs: usize,
) -> Result<f64, String> {
    let view = FractalView::new(width, height);
    let fractal_params: Vec<f64> = fractal.parameters().iter().map(|p| p.default).collect();
    let gpu_config = GpuConfig {
        center_x: view.center_x,
        center_y: view.center_y,
        zoom: view.zoom,
        max_iter,
        width,
        height,
        fractal_params,
    };
    // Warmup
    for _ in 0..2 {
        renderer.render_iterations(&gpu_config, fractal)?;
    }
    let mut times = Vec::with_capacity(runs);
    for _ in 0..runs {
        let t = Instant::now();
        renderer.render_iterations(&gpu_config, fractal)?;
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    Ok(times.iter().sum::<f64>() / times.len() as f64)
}

fn main() {
    println!("=== Forma Fractalis GPU Benchmark (v0.2.5) ===\n");

    // All GPU-capable fractals and their display names
    let fractals: Vec<(&str, Box<dyn Fractal>)> = vec![
        ("Mandelbrot",        Box::new(Mandelbrot::new())),
        ("Julia Set",         Box::new(Julia::new())),
        ("Burning Ship",      Box::new(BurningShip::new())),
        ("Tippets Mandelbrot",Box::new(TippetsMandelbrot::new())),
        ("Multifractal-Julia",Box::new(MultifractalJulia::new())),
        ("Cactus",            Box::new(Cactus::new())),
        ("Zubieta",           Box::new(Zubieta::new())),
        ("Sin Julia",         Box::new(SinJulia::new())),
        ("Insideout Dragon",  Box::new(InsideoutDragon::new())),
    ];

    // Benchmark configurations
    let configs: Vec<(u32, u32, u32, &str)> = vec![
        (1280, 720,  256,  "HD  @ 256 iter"),
        (1280, 720,  1024, "HD  @ 1024 iter"),
        (1920, 1080, 1024, "FHD @ 1024 iter"),
        (1920, 1080, 2048, "FHD @ 2048 iter"),
    ];

    let runs = 5;

    #[cfg(feature = "gpu")]
    let mut renderer = match WgpuRenderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("GPU init failed: {}. CPU-only results will be shown.", e);
            print_cpu_only(&fractals, &configs, runs);
            return;
        }
    };

    println!("{:<22} {:<20} {:>10} {:>10} {:>9}", "Fractal", "Config", "CPU (ms)", "GPU (ms)", "Speedup");
    println!("{}", "-".repeat(75));

    for (name, fractal) in &fractals {
        for &(width, height, max_iter, label) in &configs {
            let cpu_ms = benchmark_cpu(fractal.as_ref(), width, height, max_iter, runs);

            #[cfg(feature = "gpu")]
            {
                match benchmark_gpu(&mut renderer, fractal.as_ref(), width, height, max_iter, runs) {
                    Ok(gpu_ms) => {
                        let speedup = cpu_ms / gpu_ms;
                        println!("{:<22} {:<20} {:>10.2} {:>10.2} {:>8.2}x",
                            name, label, cpu_ms, gpu_ms, speedup);
                    }
                    Err(e) => {
                        println!("{:<22} {:<20} {:>10.2} {:>10} {:>9}",
                            name, label, cpu_ms, format!("ERR"), "N/A");
                        eprintln!("  GPU error for {} @ {}: {}", name, label, e);
                    }
                }
            }

            #[cfg(not(feature = "gpu"))]
            println!("{:<22} {:<20} {:>10.2} {:>10} {:>9}", name, label, cpu_ms, "N/A", "N/A");
        }
        println!();
    }

    println!("Notes:");
    println!("  GPU times include shader dispatch and staging buffer transfer.");
    println!("  GPU init (shader compilation) is excluded from per-render times.");
    println!("  Colormap application runs on CPU for both backends.");
}

fn print_cpu_only(fractals: &[(&str, Box<dyn Fractal>)], configs: &[(u32, u32, u32, &str)], runs: usize) {
    println!("{:<22} {:<20} {:>10}", "Fractal", "Config", "CPU (ms)");
    println!("{}", "-".repeat(54));
    for (name, fractal) in fractals {
        for &(width, height, max_iter, label) in configs {
            let cpu_ms = benchmark_cpu(fractal.as_ref(), width, height, max_iter, runs);
            println!("{:<22} {:<20} {:>10.2}", name, label, cpu_ms);
        }
        println!();
    }
}
