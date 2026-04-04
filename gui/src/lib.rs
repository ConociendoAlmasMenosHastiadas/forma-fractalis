// GUI crate root — re-exports core modules and declares GUI-specific modules.
// This allows main.rs and tests to use `forma_fractalis::fractals::*` etc. unchanged.

pub use forma_fractalis_core::animation;
pub use forma_fractalis_core::config;
pub use forma_fractalis_core::export;
pub use forma_fractalis_core::filtering;
pub use forma_fractalis_core::fractals;
pub use forma_fractalis_core::gpu;
pub use forma_fractalis_core::gpu_test;
pub use forma_fractalis_core::number_utils;
pub use forma_fractalis_core::rendering;
pub use forma_fractalis_core::rendering_pipeline;

pub use forma_fractalis_core::is_profiling_enabled;
pub use forma_fractalis_core::enable_profiling;
pub use forma_fractalis_core::perf_log;

pub mod app_state;
pub mod cli;
pub mod color_picker;
pub mod colorschemes_gui;
pub mod export_helpers;
pub mod fractal_gui;
pub mod gui;
