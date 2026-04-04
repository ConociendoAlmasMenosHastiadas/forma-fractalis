//! Headless fractal render example.
//!
//! Renders a Mandelbrot set to a PNG file without any GUI.
//!
//! Run with:
//!     cargo run -p forma-fractalis-core --example simple_render
//!
//! Output: simple_render_output.png (current directory)

use forma_fractalis_core::{
    config::FractalConfig,
    fractals::Mandelbrot,
    render_fractal_to_buffer,
};

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;
    let max_iterations: u32 = 256;

    let fractal = Mandelbrot::new();
    let config = FractalConfig::headless(width, height, max_iterations);

    println!("Rendering {}x{} Mandelbrot at {} iterations...", width, height, max_iterations);

    let buffer = render_fractal_to_buffer(&fractal, &config)
        .expect("render failed");

    assert_eq!(
        buffer.len() as u32,
        width * height * 4,
        "unexpected buffer size"
    );

    let output_path = "simple_render_output.png";
    image::save_buffer(
        output_path,
        &buffer,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .expect("failed to save PNG");

    println!("Saved to {}", output_path);
}
