//! Command-Line Interface for Headless Rendering
//!
//! This module provides CLI functionality for rendering fractals without the GUI.
//! Supports loading settings from PNG metadata or standalone JSON files, with
//! optional parameter overrides via command-line arguments.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::app_state::{FractalState, ViewState, ColorState, InputState, ExportState};
use crate::export::{FractalMetadata, load_png_metadata};
use crate::fractals::*;

/// Forma Fractalis - Interactive Fractal Explorer
#[derive(Parser)]
#[command(name = "forma-fractalis")]
#[command(version, about = "Interactive fractal explorer with command-line rendering support", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Enable performance profiling output
    #[arg(short, long)]
    pub profiling: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Render a fractal from PNG metadata or JSON settings file
    Render {
        /// Input file (PNG with metadata or JSON settings file)
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output PNG file path
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Override width (in pixels)
        #[arg(long)]
        width: Option<u32>,

        /// Override height (in pixels)
        #[arg(long)]
        height: Option<u32>,

        /// Override max iterations
        #[arg(long)]
        iterations: Option<u32>,

        /// Override scaling factor (e.g., 3.0 for 3x size)
        #[arg(long)]
        scale: Option<f32>,

        /// Override supersample multiplier (1 = no supersample, 2 = 2x, etc.)
        #[arg(long)]
        supersample: Option<u32>,
    },
}

/// Load metadata from a file (PNG or JSON)
fn load_metadata(path: &PathBuf) -> Result<FractalMetadata, String> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "png" => load_png_metadata(path),
        "json" => {
            let file = std::fs::File::open(path)
                .map_err(|e| format!("Failed to open JSON file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse JSON: {}", e))
        }
        _ => Err(format!("Unsupported file extension: {}", extension)),
    }
}

/// Render from CLI arguments
pub fn render_from_cli(args: &Commands) -> Result<(), String> {
    match args {
        Commands::Render {
            input,
            output,
            width,
            height,
            iterations,
            scale,
            supersample,
        } => {
            // Load metadata
            println!("Loading settings from: {}", input.display());
            let metadata = load_metadata(input)?;
            
            // Convert to state structs
            let fractal_state = FractalState::from(&metadata);
            let mut view_state = ViewState::from(&metadata);
            let color_state = ColorState::from(&metadata);
            let mut input_state = InputState::from(&metadata);
            let mut export_state = ExportState::from(&metadata);
            
            // Apply CLI overrides
            if let Some(w) = width {
                view_state.view.width = *w;
                input_state.width = w.to_string();
            }
            if let Some(h) = height {
                view_state.view.height = *h;
                input_state.height = h.to_string();
            }
            if let Some(iter) = iterations {
                input_state.iterations = iter.to_string();
            }
            
            let scale_factor = scale.unwrap_or(metadata.export_scale);
            let supersample_factor = supersample.unwrap_or(metadata.export_supersample);
            
            // Set output directory to the output file's parent directory
            export_state.directory = output.parent().map(|p| p.to_path_buf());
            
            // Create progress bar
            let pb = indicatif::ProgressBar::new_spinner();
            pb.set_style(
                indicatif::ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
            );
            pb.set_message("Initializing render...");
            
            // Get fractal instance
            let mandelbrot = Mandelbrot::new();
            let julia = Julia::new();
            let burning_ship = BurningShip::new();
            let tippets_mandelbrot = TippetsMandelbrot::new();
            let multifractal_julia = MultifractalJulia::new();
            let cactus = Cactus::new();
            let marek_dragon = MarekDragon::new();
            let tetration = Tetration::new();
            let lemon = Lemon::new();
            let insideout_dragon = InsideoutDragon::new();
            let zubieta = Zubieta::new();
            
            let fractal: &dyn Fractal = match fractal_state.fractal_type {
                crate::app_state::FractalType::Mandelbrot => &mandelbrot,
                crate::app_state::FractalType::Julia => &julia,
                crate::app_state::FractalType::BurningShip => &burning_ship,
                crate::app_state::FractalType::TippetsMandelbrot => &tippets_mandelbrot,
                crate::app_state::FractalType::MultifractalJulia => &multifractal_julia,
                crate::app_state::FractalType::Cactus => &cactus,
                crate::app_state::FractalType::MarekDragon => &marek_dragon,
                crate::app_state::FractalType::Tetration => &tetration,
                crate::app_state::FractalType::Lemon => &lemon,
                crate::app_state::FractalType::InsideoutDragon => &insideout_dragon,
                crate::app_state::FractalType::Zubieta => &zubieta,
            };
            
            pb.set_message(format!(
                "Rendering {} ({}x{}, {}x scale, {}x supersample)...",
                fractal.name(),
                view_state.view.width,
                view_state.view.height,
                scale_factor,
                supersample_factor
            ));
            
            // Create render state (CLI uses CPU by default, GPU can be enabled via settings)
            let mut render_state = crate::app_state::RenderState::new();
            
            // Render using the new state-based export function
            let result = crate::export::export_png_from_state(
                &fractal_state,
                &view_state,
                &color_state,
                &input_state,
                &export_state,
                &mut render_state,
                fractal,
                scale_factor,
                supersample_factor,
            );
            
            match result {
                Ok(path) => {
                    pb.finish_with_message(format!("✓ Successfully rendered to: {}", path));
                    
                    // Rename if output path was specified
                    if output != &PathBuf::from(&path) {
                        std::fs::rename(&path, output)
                            .map_err(|e| format!("Failed to move output file: {}", e))?;
                        println!("✓ Moved to: {}", output.display());
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    pb.finish_with_message(format!("✗ Render failed"));
                    Err(e)
                }
            }
        }
    }
}

/// Check if CLI mode should be activated (if any arguments are provided)
pub fn should_use_cli() -> bool {
    std::env::args().len() > 1
}

/// Parse CLI arguments and render, or return Ok(()) if no CLI args
/// Returns (Option<()>, bool) where bool indicates if profiling is enabled
pub fn try_cli() -> Result<(Option<()>, bool), String> {
    if !should_use_cli() {
        return Ok((None, false));
    }

    let cli = Cli::parse();
    let profiling = cli.profiling;
    
    if let Some(command) = cli.command {
        render_from_cli(&command)?;
        Ok((Some(()), profiling))
    } else {
        // No subcommand provided, allow GUI to run with profiling flag
        Ok((None, profiling))
    }
}
