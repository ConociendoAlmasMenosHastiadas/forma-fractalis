//! GPU Benchmark for Fractal Rendering
//! 
//! Compares CPU vs GPU rendering performance

use forma_fractalis::{
    fractals::{Mandelbrot, FractalView, Fractal},
    gpu::{WgpuRenderer, FractalRenderer, RenderConfig as GpuConfig, RenderBackend},
    rendering_pipeline::{render_with_config, RenderConfig, RenderTarget},
};
use scala_chromatica::ColorMap;
use std::collections::HashMap;
use std::time::Instant;

fn benchmark_cpu(
    fractal: &dyn Fractal,
    width: u32,
    height: u32,
    max_iter: u32,
    runs: usize,
) -> f64 {
    let view = FractalView::new(width, height);
    let colormap = ColorMap::default_scheme();
    let parameters = HashMap::new();

    let config = RenderConfig::new(view, &colormap, max_iter, fractal)
        .with_fractal_parameters(parameters)
        .with_backend(RenderBackend::Cpu);

    let mut times = Vec::with_capacity(runs);

    // Warmup
    for _ in 0..2 {
        #[cfg(feature = "gpu")]
        render_with_config(&config, RenderTarget::Preview, None);
        #[cfg(not(feature = "gpu"))]
        render_with_config(&config, RenderTarget::Preview);
    }

    // Actual benchmark
    for _ in 0..runs {
        let start = Instant::now();
        #[cfg(feature = "gpu")]
        render_with_config(&config, RenderTarget::Preview, None);
        #[cfg(not(feature = "gpu"))]
        render_with_config(&config, RenderTarget::Preview);
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    times.iter().sum::<f64>() / times.len() as f64
}

#[cfg(feature = "gpu")]
fn benchmark_gpu(
    fractal: &dyn Fractal,
    width: u32,
    height: u32,
    max_iter: u32,
    runs: usize,
) -> Result<f64, String> {
    let mut gpu_renderer = WgpuRenderer::new()?;
    
    let parameters: Vec<f64> = fractal.parameters()
        .iter()
        .map(|p| p.default)
        .collect();

    let view = FractalView::new(width, height);
    
    let gpu_config = GpuConfig {
        center_x: view.center_x,
        center_y: view.center_y,
        zoom: view.zoom,
        max_iter,
        width,
        height,
        fractal_params: parameters,
    };

    let mut times = Vec::with_capacity(runs);

    // Warmup
    for _ in 0..2 {
        gpu_renderer.render_iterations(&gpu_config, fractal)?;
    }

    // Actual benchmark
    for _ in 0..runs {
        let start = Instant::now();
        gpu_renderer.render_iterations(&gpu_config, fractal)?;
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    Ok(times.iter().sum::<f64>() / times.len() as f64)
}

fn main() {
    println!("Forma Fractalis GPU Benchmark\n");
    println!("Testing Mandelbrot Set rendering performance\n");
    
    let mandelbrot = Mandelbrot::new();
    let runs = 5;
    
    // Test configurations (resolution, iterations)
    let configs = vec![
        (640, 480, 256, "VGA @ 256 iter"),
        (1920, 1080, 1024, "HD @ 1024 iter"),
        (1920, 1080, 2048, "HD @ 2048 iter"),
        (3840, 2160, 2048, "4K @ 2048 iter"),
    ];
    
    println!("{:<25} {:>12} {:>12} {:>10}", "Configuration", "CPU (ms)", "GPU (ms)", "Speedup");
    println!("{}", "-".repeat(65));
    
    for (width, height, max_iter, desc) in configs {
        let cpu_time = benchmark_cpu(&mandelbrot, width, height, max_iter, runs);
        
        #[cfg(feature = "gpu")]
        {
            match benchmark_gpu(&mandelbrot, width, height, max_iter, runs) {
                Ok(gpu_time) => {
                    let speedup = cpu_time / gpu_time;
                    println!(
                        "{:<25} {:>12.2} {:>12.2} {:>9.2}x",
                        desc, cpu_time, gpu_time, speedup
                    );
                }
                Err(e) => {
                    println!(
                        "{:<25} {:>12.2} {:>12} {:>10}",
                        desc, cpu_time, format!("FAIL: {}", e), "N/A"
                    );
                }
            }
        }
        
        #[cfg(not(feature = "gpu"))]
        {
            println!(
                "{:<25} {:>12.2} {:>12} {:>10}",
                desc, cpu_time, "N/A", "N/A"
            );
        }
    }
    
    println!("\nNotes:");
    println!("- GPU times include shader dispatch and buffer transfer overhead");
    println!("- Colormap application is done on CPU for both backends");
    println!("- Actual speedup may vary based on GPU hardware");
    
    #[cfg(not(feature = "gpu"))]
    {
        println!("\nGPU support not compiled in. Rebuild with default features to enable GPU benchmarking.");
    }
}
