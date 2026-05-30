use forma_fractalis_core::{
    enable_profiling,
    fractals::{FractalView, Mandelbrot},
    gpu::RenderBackend,
    perturbation::PT_REFERENCE_BITS,
    rendering_pipeline::{latest_pt_report, render_with_config, RenderConfig, RenderTarget},
};
use scala_chromatica::ColorMap;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Scene {
    key: &'static str,
    center_x: f64,
    center_y: f64,
    zoom: f64,
}

const BOUNDARY_SCENE: Scene = Scene {
    key: "boundary",
    center_x: -0.743643887037151,
    center_y: 0.131825904205330,
    zoom: 1.55e13,
};

const PERIOD2_SCENE: Scene = Scene {
    key: "period2",
    center_x: -1.0,
    center_y: 0.0,
    zoom: 1.55e13,
};

const SCRATCH_SCENE: Scene = Scene {
    key: "scratch",
    center_x: -1.2541275710056559,
    center_y: 0.38365671509396854,
    zoom: 1.55e13,
};

fn parse_scene(args: &[String]) -> Result<Scene, String> {
    let Some(index) = args.iter().position(|arg| arg == "--scene") else {
        return Ok(BOUNDARY_SCENE);
    };
    let Some(name) = args.get(index + 1) else {
        return Err("--scene requires one of: boundary, period2, scratch".to_string());
    };

    match name.as_str() {
        "boundary" => Ok(BOUNDARY_SCENE),
        "period2" => Ok(PERIOD2_SCENE),
        "scratch" => Ok(SCRATCH_SCENE),
        other => Err(format!("unknown scene '{other}'")),
    }
}

fn parse_u32_arg(args: &[String], flag: &str, default: u32) -> Result<u32, String> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(default);
    };
    let Some(value) = args.get(index + 1) else {
        return Err(format!("{flag} requires an integer value"));
    };
    value
        .parse::<u32>()
        .map_err(|_| format!("invalid value '{value}' for {flag}"))
}

fn main() -> Result<(), String> {
    enable_profiling();

    let args: Vec<String> = std::env::args().collect();
    let scene = parse_scene(&args)?;
    let width = parse_u32_arg(&args, "--width", 1280)?;
    let height = parse_u32_arg(&args, "--height", 720)?;
    let iterations = parse_u32_arg(&args, "--iterations", 32768)?;
    let bits = parse_u32_arg(&args, "--bits", PT_REFERENCE_BITS)?;
    let tiles = parse_u32_arg(&args, "--tiles", 4)?;

    let mut view = FractalView::new(width, height);
    view.center_x = scene.center_x;
    view.center_y = scene.center_y;
    view.zoom = scene.zoom;

    let fractal = Mandelbrot::new();
    let mut params = HashMap::new();
    params.insert("power".to_string(), 2.0);
    params.insert("escape_radius".to_string(), 2.0);
    let colormap = ColorMap::default_scheme();

    let config = RenderConfig::new(view.clone(), &colormap, iterations, &fractal)
        .with_fractal_parameters(params)
        .with_backend(RenderBackend::Perturbation)
        .with_pt_bits(bits)
        .with_pt_tiles(tiles);

    println!(
        "PT preview report | scene={} | {}x{} | zoom={:.3e} | iterations={} | bits={} | tiles={}x{}",
        scene.key,
        width,
        height,
        scene.zoom,
        iterations,
        bits,
        tiles,
        tiles,
    );

    #[cfg(feature = "gpu")]
    let buffer = render_with_config(&config, RenderTarget::Preview, None)?;

    #[cfg(not(feature = "gpu"))]
    let buffer = render_with_config(&config, RenderTarget::Preview)?;

    println!("Rendered {} bytes", buffer.len());

    if let Some(report) = latest_pt_report() {
        println!(
            "PT report: delta={:.2}% fallback={:.2}% SA={:.2}% avg_skip={:.1} max_skip={} guards(e={}, c={}) rebased={:.2}% rebases={} budget_hit={}",
            if report.total_pixels > 0 {
                report.pt_pixels as f64 / report.total_pixels as f64 * 100.0
            } else {
                0.0
            },
            report.fallback_pct,
            report.sa_accepted_pixel_pct,
            report.sa_avg_skipped_iterations,
            report.sa_max_skipped_iterations,
            report.sa_rejected_escape_margin_count,
            report.sa_rejected_correction_count,
            report.rebased_pixel_pct,
            report.rebase_count,
            report.rebase_exhausted_count,
        );
    } else {
        println!("No PT report available");
    }

    Ok(())
}