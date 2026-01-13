//! # Mandelrust - Interactive Mandelbrot Set Explorer
//!
//! A high-performance, interactive Mandelbrot set explorer with advanced color mapping
//! and image export capabilities.
//!
//! ## Modules
//! - [`colorschemes`]: Color gradient system with built-in and custom colormaps
//! - [`colorschemes_gui`]: Interactive color editor widgets for egui
//! - [`colorschemes_io`]: Save/load colormap JSON files
//! - [`export`]: PNG image export with scaling
//! - [`fractal`]: Mandelbrot set mathematics and viewport management
//! - [`gui`]: Main application GUI layout
//! - [`rendering`]: Parallel fractal rendering with Rayon
/// - [`rendering_pipeline`]: Unified rendering system for preview and export
/// - [`filtering`]: Image filtering and supersampling for high-quality exports

pub mod colorschemes;
pub mod colorschemes_gui;
pub mod colorschemes_io;
pub mod export;
pub mod filtering;
pub mod fractal;
pub mod gui;
pub mod rendering;
pub mod rendering_pipeline;
