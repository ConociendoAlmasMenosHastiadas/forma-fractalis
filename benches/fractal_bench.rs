use forma_fractalis::{
    colorschemes::ColorMap,
    fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, FractalView, Fractal},
    rendering_pipeline::{render_with_config, RenderConfig, RenderTarget},
};
use std::collections::HashMap;
use std::time::Instant;

fn benchmark_fractal(
    fractal: &dyn Fractal,
    name: &str,
    iterations: u32,
    width: u32,
    height: u32,
    runs: usize,
) -> (f64, f64) {
    let view = FractalView::new(width, height);
    let colormap = ColorMap::default_scheme();
    let parameters = HashMap::new();

    let config = RenderConfig::new(view, &colormap, iterations, fractal)
        .with_fractal_parameters(parameters);

    let mut times = Vec::with_capacity(runs);

    // Warmup
    for _ in 0..2 {
        render_with_config(&config, RenderTarget::Preview);
    }

    // Actual benchmark
    for _ in 0..runs {
        let start = Instant::now();
        render_with_config(&config, RenderTarget::Preview);
        times.push(start.elapsed().as_secs_f64() * 1000.0); // Convert to ms
    }

    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!(
        "{:<20} {}x{} @ {:>5} iter: avg={:>7.2}ms, min={:>7.2}ms, max={:>7.2}ms",
        name, width, height, iterations, avg, min, max
    );

    (avg, min)
}

fn main() {
    println!("=== Forma Fractalis Performance Benchmark ===");
    println!("Build: release mode, 10 runs per test\n");

    let fractals: Vec<(&str, Box<dyn Fractal>)> = vec![
        ("Mandelbrot", Box::new(Mandelbrot::new())),
        ("Julia", Box::new(Julia::new())),
        ("Burning Ship", Box::new(BurningShip::new())),
        ("Tippets Mandelbrot", Box::new(TippetsMandelbrot::new())),
    ];

    let resolutions = vec![
        (640, 480, "SD"),
        (1280, 720, "HD"),
        (1920, 1080, "FHD"),
    ];

    let iteration_counts = vec![256, 512, 1024, 2048, 4096];

    for (width, height, res_name) in &resolutions {
        println!("\n--- Resolution: {} ({}x{}) ---", res_name, width, height);
        
        for iterations in &iteration_counts {
            for (name, fractal) in &fractals {
                benchmark_fractal(fractal.as_ref(), name, *iterations, *width, *height, 10);
            }
            println!();
        }
    }

    println!("\n=== Benchmark Complete ===");
    println!("\nPerformance targets:");
    println!("  1280x720 @ 256 iter:  < 20ms (50+ FPS)");
    println!("  1280x720 @ 1024 iter: < 50ms (20+ FPS)");
    println!("  1280x720 @ 4096 iter: < 150ms (7+ FPS)");
}
