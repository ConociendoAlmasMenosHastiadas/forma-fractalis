//! ColorMap Save/Load System
//!
//! This module handles persistent storage of ColorMaps using JSON files.
//! It supports both built-in default colormaps (embedded at compile time)
//! and user-created custom colormaps (stored in platform-specific directories).
//!
//! # Architecture
//! - Default colormaps are embedded in the binary using `include_str!`
//! - Custom colormaps are stored in OS-appropriate config directories
//! - Automatic directory creation and error handling
//!
//! # Usage
//! ```ignore
//! // Load a built-in colormap
//! let fire = load_builtin_colormap("Fire")?;
//!
//! // Save a custom colormap
//! save_colormap(&my_colormap)?;
//!
//! // Load a custom colormap
//! let custom = load_custom_colormap("MyCustom")?;
//!
//! // List all available colormaps
//! let all = list_available_colormaps()?;
//! ```

use crate::colorschemes::ColorMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Macro to define builtin colormaps with automatic list generation
macro_rules! define_builtin_colormaps {
    ($($name:literal => $const_name:ident => $file:literal),* $(,)?) => {
        $(
            const $const_name: &str = include_str!($file);
        )*
        
        /// Get list of all builtin colormap names
        fn get_builtin_colormap_names() -> &'static [&'static str] {
            &[$($name),*]
        }
        
        /// Load a builtin colormap by name
        fn load_builtin_impl(name: &str) -> Option<&'static str> {
            match name {
                $($name => Some($const_name),)*
                _ => None,
            }
        }
        
        /// Check if a colormap name is builtin
        fn is_builtin_impl(name: &str) -> bool {
            matches!(name, $($name)|*)
        }
    };
}

// Define all builtin colormaps in one place
define_builtin_colormaps! {
    "Default" => DEFAULT_COLORMAP_JSON => "colormaps/default.json",
    "Fire" => FIRE_COLORMAP_JSON => "colormaps/fire.json",
    "Ocean" => OCEAN_COLORMAP_JSON => "colormaps/ocean.json",
    "Grayscale" => GRAYSCALE_COLORMAP_JSON => "colormaps/grayscale.json",
    "Rainbow" => RAINBOW_COLORMAP_JSON => "colormaps/rainbow.json",
    "Academic" => ACADEMIC_COLORMAP_JSON => "colormaps/academic.json",
    "Twilight Garden" => TWILIGHT_GARDEN_COLORMAP_JSON => "colormaps/twilight_garden.json",
    "Coral Sunset" => CORAL_SUNSET_COLORMAP_JSON => "colormaps/coral_sunset.json",
    "Olive Symmetry" => OLIVE_SYMMETRY_COLORMAP_JSON => "colormaps/olive_symmetry.json",
    "Orchid Garden" => ORCHID_GARDEN_COLORMAP_JSON => "colormaps/orchid_garden.json",
    "Frozen Amaranth" => FROZEN_AMARANTH_COLORMAP_JSON => "colormaps/frozen_amaranth.json",
    "Electric Neon" => ELECTRIC_NEON_COLORMAP_JSON => "colormaps/electric_neon.json",
    "Cosmic Dawn" => COSMIC_DAWN_COLORMAP_JSON => "colormaps/cosmic_dawn.json",
    "Vintage Lavender" => VINTAGE_LAVENDER_COLORMAP_JSON => "colormaps/vintage_lavender.json",
}

/// Error types for colormap I/O operations
#[derive(Debug)]
pub enum ColorMapError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    NotFound(String),
    NoConfigDirectory,
}

impl std::fmt::Display for ColorMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorMapError::IoError(e) => write!(f, "I/O error: {}", e),
            ColorMapError::JsonError(e) => write!(f, "JSON error: {}", e),
            ColorMapError::NotFound(name) => write!(f, "ColorMap '{}' not found", name),
            ColorMapError::NoConfigDirectory => write!(f, "Could not find config directory"),
        }
    }
}

impl std::error::Error for ColorMapError {}

impl From<io::Error> for ColorMapError {
    fn from(err: io::Error) -> Self {
        ColorMapError::IoError(err)
    }
}

impl From<serde_json::Error> for ColorMapError {
    fn from(err: serde_json::Error) -> Self {
        ColorMapError::JsonError(err)
    }
}

pub type Result<T> = std::result::Result<T, ColorMapError>;

/// Get the directory where custom colormaps are stored
/// Returns platform-specific config directory:
/// - Windows: %APPDATA%\forma-fractalis\colormaps\
/// - Linux: ~/.config/forma-fractalis/colormaps/
/// - macOS: ~/Library/Application Support/forma-fractalis/colormaps/
pub fn get_colormaps_directory() -> Result<PathBuf> {
    let base_dir = directories::ProjectDirs::from("", "", "forma-fractalis")
        .ok_or(ColorMapError::NoConfigDirectory)?;

    let colormaps_dir = base_dir.config_dir().join("colormaps");

    // Create directory if it doesn't exist
    if !colormaps_dir.exists() {
        fs::create_dir_all(&colormaps_dir)?;
    }

    Ok(colormaps_dir)
}

/// Load a built-in colormap by name
/// Available built-in colormaps are automatically managed by the macro above
pub fn load_builtin_colormap(name: &str) -> Result<ColorMap> {
    let json_str = load_builtin_impl(name)
        .ok_or_else(|| ColorMapError::NotFound(name.to_string()))?;

    let colormap: ColorMap = serde_json::from_str(json_str)?;
    Ok(colormap)
}

/// Check if a colormap is a built-in default
pub fn is_builtin_colormap(name: &str) -> bool {
    is_builtin_impl(name)
}

/// Save a colormap to the custom colormaps directory
/// This will create a JSON file named "{colormap.name}.json"
pub fn save_colormap(colormap: &ColorMap) -> Result<PathBuf> {
    let dir = get_colormaps_directory()?;
    let filename = format!("{}.json", colormap.name);
    let filepath = dir.join(&filename);

    let json = serde_json::to_string_pretty(colormap)?;
    fs::write(&filepath, json)?;

    Ok(filepath)
}

/// Load a custom colormap from the colormaps directory
pub fn load_custom_colormap(name: &str) -> Result<ColorMap> {
    let dir = get_colormaps_directory()?;
    let filename = format!("{}.json", name);
    let filepath = dir.join(&filename);

    if !filepath.exists() {
        return Err(ColorMapError::NotFound(name.to_string()));
    }

    let json = fs::read_to_string(&filepath)?;
    let colormap: ColorMap = serde_json::from_str(&json)?;

    Ok(colormap)
}

/// Load a colormap by name, checking built-ins first, then custom colormaps
pub fn load_colormap(name: &str) -> Result<ColorMap> {
    // Try built-in first
    if is_builtin_colormap(name) {
        return load_builtin_colormap(name);
    }

    // Try custom
    load_custom_colormap(name)
}

/// Delete a custom colormap
/// Note: Built-in colormaps cannot be deleted
pub fn delete_custom_colormap(name: &str) -> Result<()> {
    if is_builtin_colormap(name) {
        return Err(ColorMapError::IoError(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Cannot delete built-in colormaps",
        )));
    }

    let dir = get_colormaps_directory()?;
    let filename = format!("{}.json", name);
    let filepath = dir.join(&filename);

    if !filepath.exists() {
        return Err(ColorMapError::NotFound(name.to_string()));
    }

    fs::remove_file(&filepath)?;
    Ok(())
}

/// Information about an available colormap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorMapInfo {
    pub name: String,
    pub is_builtin: bool,
    pub filepath: Option<PathBuf>,
}

/// List all available colormaps (built-in + custom)
pub fn list_available_colormaps() -> Result<Vec<ColorMapInfo>> {
    let mut colormaps = Vec::new();

    // Add built-in colormaps (automatically generated from macro)
    for name in get_builtin_colormap_names() {
        colormaps.push(ColorMapInfo {
            name: name.to_string(),
            is_builtin: true,
            filepath: None,
        });
    }

    // Add custom colormaps
    let dir = get_colormaps_directory()?;
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Skip if it has the same name as a built-in (built-ins take precedence)
                    if !is_builtin_colormap(stem) {
                        colormaps.push(ColorMapInfo {
                            name: stem.to_string(),
                            is_builtin: false,
                            filepath: Some(path),
                        });
                    }
                }
            }
        }
    }

    Ok(colormaps)
}

/// Export a built-in colormap to the custom colormaps directory
/// This allows users to create modified versions of built-in colormaps
pub fn export_builtin_colormap(name: &str) -> Result<PathBuf> {
    let colormap = load_builtin_colormap(name)?;
    save_colormap(&colormap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_builtin_colormaps() {
        // Test loading each built-in colormap
        for name in &[
            "Default",
            "Fire",
            "Ocean",
            "Grayscale",
            "Rainbow",
            "Academic",
            "Twilight Garden",
            "Coral Sunset",
            "Olive Symmetry",
            "Orchid Garden",
        ] {
            let result = load_builtin_colormap(name);
            assert!(result.is_ok(), "Failed to load {}: {:?}", name, result);

            let colormap = result.unwrap();
            assert_eq!(colormap.name, *name);
            assert!(!colormap.stops.is_empty());
        }
    }

    #[test]
    fn test_load_nonexistent_builtin() {
        let result = load_builtin_colormap("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_builtin_colormap() {
        assert!(is_builtin_colormap("Fire"));
        assert!(is_builtin_colormap("Ocean"));
        assert!(is_builtin_colormap("Academic"));
        assert!(is_builtin_colormap("Orchid Garden"));
        assert!(!is_builtin_colormap("MyCustom"));
        assert!(!is_builtin_colormap(""));
    }
}
