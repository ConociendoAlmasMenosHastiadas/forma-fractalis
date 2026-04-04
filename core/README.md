# forma-fractalis-core

High-performance fractal computation and rendering library.

A framework-agnostic Rust library for rendering fractals. No GUI dependencies — use it in scripts, CLI tools, servers, or other applications.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
forma-fractalis-core = "0.2.3"
```

To enable GPU acceleration (requires wgpu, optional):

```toml
[dependencies]
forma-fractalis-core = { version = "0.2.3", features = ["gpu"] }
```

## Quick Start

```rust
use forma_fractalis_core::{
    config::FractalConfig,
    fractals::{FractalView, Mandelbrot},
    render_fractal_to_buffer,
};

fn main() {
    let fractal = Mandelbrot::new();
    let config = FractalConfig::headless(800, 600, 256);

    let buffer = render_fractal_to_buffer(&fractal, &config)
        .expect("render failed");

    // buffer is RGBA, 4 bytes per pixel, row-major
    assert_eq!(buffer.len() as u32, 800 * 600 * 4);

    image::save_buffer("output.png", &buffer, 800, 600, image::ColorType::Rgba8).unwrap();
}
```

## Configuration

`FractalConfig` uses a builder pattern:

```rust
use forma_fractalis_core::config::FractalConfig;
use forma_fractalis_core::fractals::FractalView;

// Headless convenience constructor
let config = FractalConfig::headless(1920, 1080, 512);

// Full builder
let view = FractalView::new(1920, 1080)
    .with_center(-0.5, 0.0)
    .with_zoom(2.0);

let config = FractalConfig::new(view, 512)
    .with_period(128)
    .with_log_scale()
    .with_interior_color([20, 20, 40])
    .with_parameter("power", 3.0);
```

Round-trip from an exported PNG's embedded metadata:

```rust
use forma_fractalis_core::config::FractalConfig;
use forma_fractalis_core::export::FractalMetadata;

let metadata = FractalMetadata::load_from_png("render.png").unwrap();
let config = FractalConfig::from_metadata(&metadata);
```

## Fractal Types

| Type | Name constant | Notes |
|------|---------------|-------|
| Mandelbrot | `"Mandelbrot"` | Classic z^n + c |
| Julia | `"Julia"` | With configurable c (real/imag params) |
| Burning Ship | `"Burning Ship"` | |
| Tippets Mandelbrot | `"TippetsMandelbrot"` | |
| Multifractal Julia | `"MultifractalJulia"` | |
| Cactus | `"Cactus"` | |
| Marek Dragon | `"MarekDragon"` | |
| Tetration | `"Tetration"` | |
| Lemon | `"Lemon"` | |
| Insideout Dragon | `"InsideoutDragon"` | |
| Zubieta | `"Zubieta"` | |
| Sin Julia | `"SinJulia"` | |

Implement the `Fractal` trait to define your own:

```rust
use forma_fractalis_core::fractals::{Fractal, FractalView, Parameter};
use std::collections::HashMap;

pub struct MyFractal;

impl Fractal for MyFractal {
    fn iterate(&self, c_real: f64, c_imag: f64, parameters: &HashMap<String, f64>, max_iter: u32) -> u32 {
        // return escape iteration count, or max_iter if inside the set
        max_iter
    }
    fn default_view(&self, width: u32, height: u32) -> FractalView { FractalView::new(width, height) }
    fn name(&self) -> &str { "MyFractal" }
    fn parameters(&self) -> Vec<Parameter> { vec![] }
}
```

## Export

Save a fractal to PNG with full metadata embedded (enables round-trip loading):

```rust
use forma_fractalis_core::export::export_png_with_config;
use forma_fractalis_core::config::{ExportConfig, FractalConfig};
use forma_fractalis_core::fractals::Mandelbrot;

let fractal = Mandelbrot::new();
let render_config = FractalConfig::headless(800, 600, 256);
let export_config = ExportConfig::default();

export_png_with_config(&fractal, &render_config, &export_config, "output.png").unwrap();
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `gpu` | enabled | GPU-accelerated rendering via wgpu |

Disable GPU (pure CPU, no wgpu dependency):

```toml
[dependencies]
forma-fractalis-core = { version = "0.2.3", default-features = false }
```

## Examples

```sh
# Headless single render
cargo run --example simple_render

# Batch render several fractals
cargo run --example batch_export
```

## License

Licensed under either of Apache License 2.0 or MIT License, at your option.
