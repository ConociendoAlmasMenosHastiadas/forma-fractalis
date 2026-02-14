//! Application State Management
//!
//! This module defines grouped state structures for the fractal application.
//! Instead of having 30+ fields scattered in a single struct, we organize
//! related state into logical groups, making the codebase more maintainable
//! and reducing function parameter counts from 10-15 to 3-5.

use scala_chromatica::ColorMap;
use crate::colorschemes_gui::ColorEditor;
use crate::filtering::FilterType;
use crate::fractals::FractalView;
use eframe::egui;
use std::collections::HashMap;
use std::time::Instant;

/// View and rendering state
#[derive(Clone)]
pub struct ViewState {
    pub view: FractalView,
    pub fractal_texture: Option<egui::TextureHandle>,
    pub needs_redraw: bool,
}

impl ViewState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            view: FractalView::new(width, height),
            fractal_texture: None,
            needs_redraw: true,
        }
    }

    /// Mark the view for redraw
    pub fn mark_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Clear redraw flag after rendering
    pub fn clear_redraw(&mut self) {
        self.needs_redraw = false;
    }
}

/// Text input state with debouncing
pub struct InputState {
    pub width: String,
    pub height: String,
    pub iterations: String,
    pub julia_c_real: String,
    pub julia_c_imag: String,
    pub mandelbrot_power: String,
    pub multifractal_julia_power: String,
    pub marek_dragon_phi: String,
    pub period: String,
    pub export_scale: String,
    pub export_supersample: String,
    
    // Debouncing
    pub debounce_timer: Option<Instant>,
    pub pending_redraw: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            width: String::from("1280"),
            height: String::from("720"),
            iterations: String::from("256"),
            julia_c_real: String::from("0.0"),
            julia_c_imag: String::from("0.0"),
            mandelbrot_power: String::from("2.0"),
            multifractal_julia_power: String::from("1.0"),
            marek_dragon_phi: String::from("0.0"),
            period: String::from("128"),
            export_scale: String::from("3.0"),
            export_supersample: String::from("4"),
            debounce_timer: None,
            pending_redraw: false,
        }
    }
}

impl InputState {
    /// Parse iterations input with default fallback
    pub fn parse_iterations(&self) -> u32 {
        self.iterations.parse::<u32>().unwrap_or(256).max(1)
    }

    /// Parse period input with default fallback
    pub fn parse_period(&self) -> u32 {
        self.period.parse::<u32>().unwrap_or(256)
    }

    /// Parse export scale with default fallback
    pub fn parse_export_scale(&self) -> f64 {
        self.export_scale.parse::<f64>().unwrap_or(3.0).max(1.0)
    }

    /// Parse export supersample with default fallback
    pub fn parse_export_supersample(&self) -> u32 {
        self.export_supersample.parse::<u32>().unwrap_or(4).max(1)
    }

    /// Parse Julia c_real parameter
    pub fn parse_julia_c_real(&self) -> f64 {
        self.julia_c_real.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Julia c_imag parameter
    pub fn parse_julia_c_imag(&self) -> f64 {
        self.julia_c_imag.parse::<f64>().unwrap_or(0.0)
    }

    /// Parse Mandelbrot power parameter
    pub fn parse_mandelbrot_power(&self) -> f64 {
        self.mandelbrot_power.parse::<f64>().unwrap_or(2.0)
    }

    /// Parse Multifractal-Julia power parameter (k in z_{n+1} = c^k * z_{n-2} + c)
    pub fn parse_multifractal_julia_power(&self) -> f64 {
        self.multifractal_julia_power.parse::<f64>().unwrap_or(1.0)
    }

    /// Parse Marek Dragon phi parameter (rotation angle 0 to 2π)
    pub fn parse_marek_dragon_phi(&self) -> f64 {
        self.marek_dragon_phi.parse::<f64>().unwrap_or(0.0)
    }
}

/// Fractal type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractalType {
    Mandelbrot,
    Julia,
    BurningShip,
    TippetsMandelbrot,
    MultifractalJulia,
    Cactus,
    MarekDragon,
}

impl FractalType {
    pub fn as_str(&self) -> &str {
        match self {
            FractalType::Mandelbrot => "Mandelbrot",
            FractalType::Julia => "Julia",
            FractalType::BurningShip => "Burning Ship",
            FractalType::TippetsMandelbrot => "Tippets Mandelbrot",
            FractalType::MultifractalJulia => "Multifractal-Julia",
            FractalType::Cactus => "Cactus",
            FractalType::MarekDragon => "Marek Dragon",
        }
    }

    pub fn name(&self) -> &str {
        self.as_str()
    }

    pub fn all() -> &'static [FractalType] {
        &[
            FractalType::Mandelbrot,
            FractalType::Julia,
            FractalType::BurningShip,
            FractalType::TippetsMandelbrot,
            FractalType::MultifractalJulia,
            FractalType::Cactus,
            FractalType::MarekDragon,
        ]
    }

    /// Creates a fractal instance from the enum type
    pub fn create_instance(&self) -> Box<dyn crate::fractals::Fractal> {
        use crate::fractals::{Mandelbrot, Julia, BurningShip, TippetsMandelbrot, MultifractalJulia, Cactus, MarekDragon};
        
        match self {
            FractalType::Mandelbrot => Box::new(Mandelbrot::new()),
            FractalType::Julia => Box::new(Julia::new()),
            FractalType::BurningShip => Box::new(BurningShip::new()),
            FractalType::TippetsMandelbrot => Box::new(TippetsMandelbrot::new()),
            FractalType::MultifractalJulia => Box::new(MultifractalJulia::new()),
            FractalType::Cactus => Box::new(Cactus::new()),
            FractalType::MarekDragon => Box::new(MarekDragon::new()),
        }
    }

    pub fn is_julia(&self) -> bool {
        matches!(self, FractalType::Julia)
    }

    pub fn is_mandelbrot(&self) -> bool {
        matches!(self, FractalType::Mandelbrot)
    }

    /// Reset view and fractal parameters for this fractal type
    pub fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        input: &InputState,
    ) {
        match self {
            FractalType::Mandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 1.0;
                // Keep existing power parameter when switching back
                let power = input.parse_mandelbrot_power();
                params.clear();
                params.insert("power".to_string(), power);
            }
            FractalType::Julia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                // Keep existing Julia parameters when switching back
                params.insert("c_real".to_string(), input.parse_julia_c_real());
                params.insert("c_imag".to_string(), input.parse_julia_c_imag());
            }
            FractalType::BurningShip => {
                view.center_x = -0.5;
                view.center_y = -0.6;
                view.zoom = 0.8;
                params.clear();
            }
            FractalType::TippetsMandelbrot => {
                view.center_x = -0.5;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
            }
            FractalType::MultifractalJulia => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 1.0;
                params.clear();
                params.insert("power".to_string(), 1.0); // Default to k=1: z_{n+1} = c * z_{n-2} + c
            }
            FractalType::Cactus => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.6;
                params.clear();
            }
            FractalType::MarekDragon => {
                view.center_x = 0.0;
                view.center_y = 0.0;
                view.zoom = 0.8;
                params.clear();
                params.insert("phi".to_string(), input.parse_marek_dragon_phi());
            }
        }
    }
}

// GUI trait implementation (bridge for backwards compatibility with old gui.rs functions)
impl crate::gui::FractalTypeOps for FractalType {
    fn get_name(&self) -> &str {
        self.name()
    }

    fn all_types() -> Vec<Self> {
        Self::all().to_vec()
    }

    fn is_julia(&self) -> bool {
        self.is_julia()
    }

    fn is_mandelbrot(&self) -> bool {
        self.is_mandelbrot()
    }

    fn reset_view_and_params(
        &self,
        view: &mut FractalView,
        params: &mut HashMap<String, f64>,
        julia_c_real_input: &str,
        julia_c_imag_input: &str,
        mandelbrot_power_input: &str,
        multifractal_julia_power_input: &str,
        marek_dragon_phi_input: &str,
    ) {
        // Create temporary InputState for compatibility
        let mut input = InputState::default();
        input.julia_c_real = julia_c_real_input.to_string();
        input.julia_c_imag = julia_c_imag_input.to_string();
        input.mandelbrot_power = mandelbrot_power_input.to_string();
        input.multifractal_julia_power = multifractal_julia_power_input.to_string();
        input.marek_dragon_phi = marek_dragon_phi_input.to_string();
        
        self.reset_view_and_params(view, params, &input);
    }
}

/// Fractal configuration state
pub struct FractalState {
    pub fractal_type: FractalType,
    pub parameters: HashMap<String, f64>,
}

impl Default for FractalState {
    fn default() -> Self {
        Self {
            fractal_type: FractalType::Mandelbrot,
            parameters: HashMap::new(),
        }
    }
}

impl FractalState {
    pub fn new(fractal_type: FractalType) -> Self {
        Self {
            fractal_type,
            parameters: HashMap::new(),
        }
    }

    /// Set a fractal parameter
    pub fn set_parameter(&mut self, key: &str, value: f64) {
        self.parameters.insert(key.to_string(), value);
    }

    /// Get a fractal parameter
    pub fn get_parameter(&self, key: &str) -> Option<f64> {
        self.parameters.get(key).copied()
    }
}

/// Colormap and modulation state
pub struct ColorState {
    pub available_colormaps: Vec<String>,
    pub selected_colormap_name: String,
    pub colormap: ColorMap,
    pub color_editor: ColorEditor,
    
    // Color modulation options
    pub use_period: bool,
    pub use_interior_color: bool,
    pub interior_color: [u8; 3],
    pub interior_color_rgb_text: [String; 3],
    pub use_log_scale: bool,
}

impl Default for ColorState {
    fn default() -> Self {
        use scala_chromatica::io as colorschemes_io;

        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        let selected_colormap_name = String::from("Default");
        let colormap = colorschemes_io::load_colormap(&selected_colormap_name)
            .unwrap_or_else(|_| ColorMap::default_scheme());

        Self {
            available_colormaps,
            selected_colormap_name,
            colormap,
            color_editor: ColorEditor::new(),
            use_period: false,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            interior_color_rgb_text: [
                String::from("0"),
                String::from("0"),
                String::from("0"),
            ],
            use_log_scale: false,
        }
    }
}

impl ColorState {
    /// Parse interior color RGB values from text inputs
    pub fn parse_interior_color(&mut self) {
        if let Ok(r) = self.interior_color_rgb_text[0].parse::<u8>() {
            self.interior_color[0] = r;
        }
        if let Ok(g) = self.interior_color_rgb_text[1].parse::<u8>() {
            self.interior_color[1] = g;
        }
        if let Ok(b) = self.interior_color_rgb_text[2].parse::<u8>() {
            self.interior_color[2] = b;
        }
    }
}

/// Mouse interaction state
#[derive(Default)]
pub struct MouseState {
    pub is_dragging: bool,
    pub zoom_square_center: Option<egui::Pos2>,
    pub zoom_square_size: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            zoom_square_center: None,
            zoom_square_size: 200.0,
        }
    }

    /// Clear zoom square
    pub fn clear_zoom(&mut self) {
        self.zoom_square_center = None;
        self.is_dragging = false;
    }
}

/// Export configuration state
pub struct ExportState {
    pub directory: Option<std::path::PathBuf>,
    pub filter: FilterType,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            directory: None,
            filter: FilterType::None,
        }
    }
}

impl ExportState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Conversion from FractalMetadata to state structs
/// These conversions enable clean bidirectional transformation between
/// serialized metadata and application state.

impl From<&crate::export::FractalMetadata> for ViewState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            view: meta.to_fractal_view(),
            fractal_texture: None,
            needs_redraw: true,
        }
    }
}

impl From<&crate::export::FractalMetadata> for FractalState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            fractal_type: meta.parse_fractal_type().unwrap_or(FractalType::Mandelbrot),
            parameters: meta.fractal_parameters.clone(),
        }
    }
}

impl From<&crate::export::FractalMetadata> for ColorState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        use scala_chromatica::io as colorschemes_io;

        let available_colormaps = colorschemes_io::list_available_colormaps()
            .unwrap_or_else(|_| Vec::new())
            .into_iter()
            .map(|info| info.name)
            .collect::<Vec<_>>();

        Self {
            available_colormaps,
            selected_colormap_name: meta.colormap_name.clone(),
            colormap: meta.colormap_data.clone(),
            color_editor: ColorEditor::new(),
            use_period: meta.use_period,
            use_interior_color: meta.use_interior_color,
            interior_color: meta.interior_color,
            interior_color_rgb_text: [
                meta.interior_color[0].to_string(),
                meta.interior_color[1].to_string(),
                meta.interior_color[2].to_string(),
            ],
            use_log_scale: meta.use_log_scale,
        }
    }
}

impl From<&crate::export::FractalMetadata> for InputState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        // Extract Julia parameters if present
        let julia_c_real = meta.fractal_parameters.get("c_real")
            .copied()
            .unwrap_or(0.0);
        let julia_c_imag = meta.fractal_parameters.get("c_imag")
            .copied()
            .unwrap_or(0.0);
        
        // Extract Mandelbrot power if present
        let mandelbrot_power = meta.fractal_parameters.get("power")
            .copied()
            .unwrap_or(2.0);
        
        // Extract Multifractal-Julia power if present (same key as Mandelbrot)
        let multifractal_julia_power = meta.fractal_parameters.get("power")
            .copied()
            .unwrap_or(1.0);
        
        // Extract Marek Dragon phi if present
        let marek_dragon_phi = meta.fractal_parameters.get("phi")
            .copied()
            .unwrap_or(0.0);

        Self {
            width: meta.width.to_string(),
            height: meta.height.to_string(),
            iterations: meta.max_iterations.to_string(),
            julia_c_real: julia_c_real.to_string(),
            julia_c_imag: julia_c_imag.to_string(),
            mandelbrot_power: mandelbrot_power.to_string(),
            multifractal_julia_power: multifractal_julia_power.to_string(),
            marek_dragon_phi: marek_dragon_phi.to_string(),
            period: meta.period.to_string(),
            export_scale: meta.export_scale.to_string(),
            export_supersample: meta.export_supersample.to_string(),
            debounce_timer: None,
            pending_redraw: true,
        }
    }
}

impl From<&crate::export::FractalMetadata> for ExportState {
    fn from(meta: &crate::export::FractalMetadata) -> Self {
        Self {
            directory: None, // Export directory is not stored in metadata
            filter: meta.parse_filter_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::FractalMetadata;
    use scala_chromatica::ColorMap;
    use std::collections::HashMap;

    #[test]
    fn test_viewstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: -0.5,
            center_y: 0.0,
            zoom: 0.8,
            width: 1920,
            height: 1080,
            max_iterations: 512,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 128,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let view_state = ViewState::from(&metadata);
        assert_eq!(view_state.view.center_x, -0.5);
        assert_eq!(view_state.view.center_y, 0.0);
        assert_eq!(view_state.view.zoom, 0.8);
        assert_eq!(view_state.view.width, 1920);
        assert_eq!(view_state.view.height, 1080);
        assert!(view_state.needs_redraw);
    }

    #[test]
    fn test_fractalstate_from_metadata() {
        let mut params = HashMap::new();
        params.insert("c_real".to_string(), -0.7);
        params.insert("c_imag".to_string(), 0.27);

        let metadata = FractalMetadata {
            fractal_type: "Julia".to_string(),
            fractal_parameters: params.clone(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 1280,
            height: 720,
            max_iterations: 256,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: false,
            period: 128,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let fractal_state = FractalState::from(&metadata);
        assert_eq!(fractal_state.fractal_type, FractalType::Julia);
        assert_eq!(fractal_state.parameters.get("c_real"), Some(&-0.7));
        assert_eq!(fractal_state.parameters.get("c_imag"), Some(&0.27));
    }

    #[test]
    fn test_inputstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 3840,
            height: 2160,
            max_iterations: 1024,
            colormap_name: "Default".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: true,
            period: 256,
            use_interior_color: false,
            interior_color: [0, 0, 0],
            use_log_scale: false,
            export_filter: "Lanczos3".to_string(),
            export_supersample: 4,
            export_scale: 2.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let input_state = InputState::from(&metadata);
        assert_eq!(input_state.width, "3840");
        assert_eq!(input_state.height, "2160");
        assert_eq!(input_state.iterations, "1024");
        assert_eq!(input_state.period, "256");
        assert_eq!(input_state.export_scale, "2");
        assert_eq!(input_state.export_supersample, "4");
    }

    #[test]
    fn test_colorstate_from_metadata() {
        let metadata = FractalMetadata {
            fractal_type: "Mandelbrot".to_string(),
            fractal_parameters: HashMap::new(),
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            width: 1280,
            height: 720,
            max_iterations: 256,
            colormap_name: "Fire".to_string(),
            colormap_data: ColorMap::default_scheme(),
            use_period: true,
            period: 128,
            use_interior_color: true,
            interior_color: [255, 128, 64],
            use_log_scale: true,
            export_filter: "None".to_string(),
            export_supersample: 1,
            export_scale: 1.0,
            version: Some("0.1.7".to_string()),
            metadata_version: Some("1.0".to_string()),
            created_timestamp: Some(0),
        };

        let color_state = ColorState::from(&metadata);
        assert_eq!(color_state.selected_colormap_name, "Fire");
        assert!(color_state.use_period);
        assert!(color_state.use_interior_color);
        assert_eq!(color_state.interior_color, [255, 128, 64]);
        assert!(color_state.use_log_scale);
        assert_eq!(color_state.interior_color_rgb_text[0], "255");
        assert_eq!(color_state.interior_color_rgb_text[1], "128");
        assert_eq!(color_state.interior_color_rgb_text[2], "64");
    }

    #[test]
    fn test_metadata_roundtrip() {
        use crate::export::FractalMetadata;
        use crate::filtering::FilterType;

        // Create initial state
        let mut fractal_state = FractalState::default();
        fractal_state.fractal_type = FractalType::Julia;
        fractal_state.parameters.insert("c_real".to_string(), -0.7);
        fractal_state.parameters.insert("c_imag".to_string(), 0.27);

        let view_state = ViewState::new(1920, 1080);
        let mut color_state = ColorState::default();
        color_state.use_period = true;
        
        let mut input_state = InputState::default();
        input_state.iterations = "512".to_string();
        
        let mut export_state = ExportState::default();
        export_state.filter = FilterType::Lanczos3;

        // Convert to metadata
        let metadata = FractalMetadata::from_app_state(
            &fractal_state,
            &view_state,
            &color_state,
            &input_state,
            &export_state,
        );

        // Convert back to state
        let fractal_state2 = FractalState::from(&metadata);
        let view_state2 = ViewState::from(&metadata);
        let color_state2 = ColorState::from(&metadata);
        let input_state2 = InputState::from(&metadata);
        let export_state2 = ExportState::from(&metadata);

        // Verify round-trip
        assert_eq!(fractal_state2.fractal_type, FractalType::Julia);
        assert_eq!(view_state2.view.width, 1920);
        assert_eq!(view_state2.view.height, 1080);
        assert!(color_state2.use_period);
        assert_eq!(input_state2.iterations, "512");
        assert_eq!(export_state2.filter, FilterType::Lanczos3);
    }
}
