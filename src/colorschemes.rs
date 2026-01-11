//! Color Schemes and Gradient System
//!
//! This module provides a flexible color mapping system for fractal visualization,
//! with support for smooth gradient interpolation between color stops.
//!
//! # Architecture
//! - `Color`: RGB representation with HSV conversion and interpolation
//! - `ColorStop`: Position-color pair for gradient definition
//! - `ColorMap`: Collection of stops with smooth interpolation
//! - `ColorScheme`: Pre-built color schemes (Default, Fire, Ocean, etc.)
//!
//! # Usage
//! ```ignore
//! let colormap = ColorScheme::Fire.to_colormap();
//! let color = colormap.get_color(0.5);  // Get color at 50% position
//! ```
/// - Color stops and gradient interpolation
/// - Iteration-to-color mapping

use serde::{Deserialize, Serialize};

/// Available color schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Fire,
    Ocean,
    Grayscale,
    Rainbow,
}

impl ColorScheme {
    pub const ALL: [ColorScheme; 5] = [
        ColorScheme::Default,
        ColorScheme::Fire,
        ColorScheme::Ocean,
        ColorScheme::Grayscale,
        ColorScheme::Rainbow,
    ];
    
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorScheme::Default => "Default",
            ColorScheme::Fire => "Fire",
            ColorScheme::Ocean => "Ocean",
            ColorScheme::Grayscale => "Grayscale",
            ColorScheme::Rainbow => "Rainbow",
        }
    }
    
    /// Get the default colormap for this scheme
    pub fn to_colormap(&self) -> ColorMap {
        match self {
            ColorScheme::Default => ColorMap::default_scheme(),
            ColorScheme::Fire => ColorMap::fire_scheme(),
            ColorScheme::Ocean => ColorMap::ocean_scheme(),
            ColorScheme::Grayscale => ColorMap::grayscale_scheme(),
            ColorScheme::Rainbow => ColorMap::rainbow_scheme(),
        }
    }
}

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
        }
    }
    
    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }
    
    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }
    
    /// Linear interpolation between two colors
    pub fn lerp(&self, other: &Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: (self.r as f64 + (other.r as f64 - self.r as f64) * t) as u8,
            g: (self.g as f64 + (other.g as f64 - self.g as f64) * t) as u8,
            b: (self.b as f64 + (other.b as f64 - self.b as f64) * t) as u8,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RGB({},{},{})", self.r, self.g, self.b)
    }
}

/// A color stop in a gradient (position + color)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    pub position: f64,  // 0.0 to 1.0
    pub color: Color,
}

impl ColorStop {
    pub fn new(position: f64, color: Color) -> Self {
        Self { 
            position: position.clamp(0.0, 1.0),
            color 
        }
    }
}

/// A colormap with multiple color stops and interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorMap {
    stops: Vec<ColorStop>,
}

impl ColorMap {
    pub fn new(stops: Vec<ColorStop>) -> Self {
        let mut colormap = Self { stops };
        colormap.sort_stops();
        colormap
    }
    
    pub fn add_stop(&mut self, stop: ColorStop) {
        self.stops.push(stop);
        self.sort_stops();
    }
    
    pub fn remove_stop(&mut self, index: usize) {
        if index < self.stops.len() && self.stops.len() > 2 {
            self.stops.remove(index);
        }
    }
    
    pub fn stops(&self) -> &[ColorStop] {
        &self.stops
    }
    
    pub fn stops_mut(&mut self) -> &mut Vec<ColorStop> {
        &mut self.stops
    }
    
    fn sort_stops(&mut self) {
        self.stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    }
    
    /// Get color at a specific position (0.0 to 1.0) by interpolating between stops
    pub fn get_color(&self, position: f64) -> Color {
        let position = position.clamp(0.0, 1.0);
        
        if self.stops.is_empty() {
            return Color::black();
        }
        
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }
        
        // Find surrounding stops
        if position <= self.stops[0].position {
            return self.stops[0].color;
        }
        
        for i in 0..self.stops.len() - 1 {
            let stop1 = &self.stops[i];
            let stop2 = &self.stops[i + 1];
            
            if position >= stop1.position && position <= stop2.position {
                let range = stop2.position - stop1.position;
                let t = if range > 0.0 {
                    (position - stop1.position) / range
                } else {
                    0.0
                };
                return stop1.color.lerp(&stop2.color, t);
            }
        }
        
        self.stops.last().unwrap().color
    }
    
    /// Default HSV-based color scheme (smooth rainbow)
    pub fn default_scheme() -> Self {
        Self::new(vec![
            ColorStop::new(0.0, Color::black()),
            ColorStop::new(0.2, Color::from_hsv(240.0, 1.0, 1.0)), // Blue
            ColorStop::new(0.5, Color::from_hsv(120.0, 1.0, 1.0)), // Green
            ColorStop::new(0.8, Color::from_hsv(0.0, 1.0, 1.0)),   // Red
            ColorStop::new(1.0, Color::white()),
        ])
    }
    
    /// Fire color scheme (black -> red -> orange -> yellow -> white)
    pub fn fire_scheme() -> Self {
        Self::new(vec![
            ColorStop::new(0.0, Color::black()),
            ColorStop::new(0.25, Color::new(128, 0, 0)),   // Dark red
            ColorStop::new(0.5, Color::new(255, 0, 0)),    // Red
            ColorStop::new(0.75, Color::new(255, 128, 0)), // Orange
            ColorStop::new(0.9, Color::new(255, 255, 0)),  // Yellow
            ColorStop::new(1.0, Color::white()),
        ])
    }
    
    /// Ocean color scheme (black -> deep blue -> cyan -> white)
    pub fn ocean_scheme() -> Self {
        Self::new(vec![
            ColorStop::new(0.0, Color::black()),
            ColorStop::new(0.3, Color::new(0, 0, 128)),    // Deep blue
            ColorStop::new(0.6, Color::new(0, 128, 255)),  // Sky blue
            ColorStop::new(0.85, Color::new(0, 255, 255)), // Cyan
            ColorStop::new(1.0, Color::white()),
        ])
    }
    
    /// Grayscale color scheme (black -> gray -> white)
    pub fn grayscale_scheme() -> Self {
        Self::new(vec![
            ColorStop::new(0.0, Color::black()),
            ColorStop::new(0.5, Color::new(128, 128, 128)),
            ColorStop::new(1.0, Color::white()),
        ])
    }
    
    /// Rainbow color scheme (full spectrum)
    pub fn rainbow_scheme() -> Self {
        Self::new(vec![
            ColorStop::new(0.0, Color::from_hsv(0.0, 1.0, 1.0)),     // Red
            ColorStop::new(0.17, Color::from_hsv(60.0, 1.0, 1.0)),   // Yellow
            ColorStop::new(0.33, Color::from_hsv(120.0, 1.0, 1.0)),  // Green
            ColorStop::new(0.5, Color::from_hsv(180.0, 1.0, 1.0)),   // Cyan
            ColorStop::new(0.67, Color::from_hsv(240.0, 1.0, 1.0)),  // Blue
            ColorStop::new(0.83, Color::from_hsv(300.0, 1.0, 1.0)),  // Magenta
            ColorStop::new(1.0, Color::from_hsv(360.0, 1.0, 1.0)),   // Red
        ])
    }
}

/// Convert iteration count to color using a colormap
pub fn color_from_iterations(
    iterations: u32,
    max_iterations: u32,
    colormap: &ColorMap,
    use_period: bool,
    period: u32,
    use_interior_color: bool,
    interior_color: [u8; 3],
) -> Color {
    // Check if point is inside the set and custom interior color is enabled
    if iterations >= max_iterations && use_interior_color {
        return Color {
            r: interior_color[0],
            g: interior_color[1],
            b: interior_color[2],
        };
    }
    
    // Apply period modulation if enabled
    let effective_iterations = if use_period && period > 0 {
        iterations % period
    } else {
        iterations
    };
    
    // Normalize iterations to 0.0-1.0 range
    let divisor = if use_period && period > 0 {
        period as f64
    } else {
        max_iterations as f64
    };
    let t = effective_iterations as f64 / divisor;
    
    // Apply smooth coloring using log scale for better distribution
    let smooth_t = (t * 10.0).log10() / 1.0; // log10(10) = 1
    
    colormap.get_color(smooth_t.clamp(0.0, 1.0))
}
