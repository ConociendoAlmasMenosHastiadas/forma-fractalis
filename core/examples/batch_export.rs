//! Batch headless render example.
//!
//! Renders several fractal types with different configurations to PNG files,
//! demonstrating CLI-style automation without a GUI.
//!
//! Run with:
//!     cargo run -p forma-fractalis-core --example batch_export
//!
//! Output: batch_output/ directory containing one PNG per job.

use forma_fractalis_core::{
    config::FractalConfig,
    fractals::{BurningShip, Fractal, Julia, Mandelbrot, TippetsMandelbrot},
    render_fractal_to_buffer,
};
use std::path::Path;

struct Job {
    name: &'static str,
    fractal: Box<dyn Fractal>,
    config: FractalConfig,
}

fn main() {
    let output_dir = Path::new("batch_output");
    std::fs::create_dir_all(output_dir).expect("could not create output directory");

    let jobs: Vec<Job> = vec![
        Job {
            name: "mandelbrot",
            fractal: Box::new(Mandelbrot::new()),
            config: FractalConfig::headless(800, 600, 256),
        },
        Job {
            name: "mandelbrot_hiiter",
            fractal: Box::new(Mandelbrot::new()),
            config: FractalConfig::headless(800, 600, 1024).with_period(128).with_log_scale(),
        },
        Job {
            name: "julia",
            fractal: Box::new(Julia::new()),
            config: FractalConfig::headless(800, 600, 256)
                .with_parameter("c_real", -0.7)
                .with_parameter("c_imag", 0.27015),
        },
        Job {
            name: "burning_ship",
            fractal: Box::new(BurningShip::new()),
            config: FractalConfig::headless(800, 600, 256),
        },
        Job {
            name: "tippets_mandelbrot",
            fractal: Box::new(TippetsMandelbrot::new()),
            config: FractalConfig::headless(800, 600, 256),
        },
    ];

    let total = jobs.len();
    let mut passed = 0;

    for (i, job) in jobs.iter().enumerate() {
        let path = output_dir.join(format!("{}.png", job.name));
        print!("[{}/{}] Rendering {}... ", i + 1, total, job.name);

        match render_fractal_to_buffer(job.fractal.as_ref(), &job.config) {
            Ok(buffer) => {
                let w = job.config.view.width;
                let h = job.config.view.height;
                match image::save_buffer(&path, &buffer, w, h, image::ColorType::Rgba8) {
                    Ok(()) => {
                        println!("saved to {}", path.display());
                        passed += 1;
                    }
                    Err(e) => println!("SAVE FAILED: {}", e),
                }
            }
            Err(e) => println!("RENDER FAILED: {}", e),
        }
    }

    println!("\nCompleted {}/{} jobs. Output in {}/", passed, total, output_dir.display());
    if passed < total {
        std::process::exit(1);
    }
}
