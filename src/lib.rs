//! # Forma Fractalis - Interactive Fractal Explorer
//!
//! A high-performance, interactive fractal explorer supporting multiple fractal types
//! with advanced color mapping and image export capabilities.
//!
//! ## Modules
//! - [`colorschemes`]: Color gradient system with built-in and custom colormaps
//! - [`colorschemes_gui`]: Interactive color editor widgets for egui
//! - [`colorschemes_io`]: Save/load colormap JSON files
//! - [`export`]: PNG image export with scaling and metadata
//! - [`fractals`]: Trait-based fractal system supporting multiple fractal types
//! - [`gui`]: Main application GUI layout
//! - [`rendering`]: Parallel fractal rendering with Rayon
/// - [`rendering_pipeline`]: Unified rendering system for preview and export
/// - [`filtering`]: Image filtering and supersampling for high-quality exports

pub mod colorschemes;
pub mod colorschemes_gui;
pub mod colorschemes_io;
pub mod export;
pub mod filtering;
pub mod fractals;
pub mod gui;
pub mod rendering;
pub mod rendering_pipeline;

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to enable/disable performance logging
static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if profiling is enabled
pub fn is_profiling_enabled() -> bool {
    PROFILING_ENABLED.load(Ordering::Relaxed)
}

/// Enable profiling (called from main with --profiling flag)
pub fn enable_profiling() {
    PROFILING_ENABLED.store(true, Ordering::Relaxed);
}

/// Macro for conditional profiling output
#[macro_export]
macro_rules! perf_log {
    ($($arg:tt)*) => {
        if $crate::is_profiling_enabled() {
            println!($($arg)*);
        }
    };
}
