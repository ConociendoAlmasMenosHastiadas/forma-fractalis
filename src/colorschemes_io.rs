//! ColorMap I/O - Re-exports from scala-chromatica
//!
//! **DEPRECATED:** This module will be removed in v0.1.7
//!
//! Please update your imports:
//! ```rust
//! // OLD
//! use forma_fractalis::colorschemes_io;
//! 
//! // NEW
//! use scala_chromatica::io;
//! ```

#![deprecated(
    since = "0.1.61",
    note = "Use scala_chromatica::io module directly. This module will be removed in v0.1.7"
)]

// Re-export everything from scala-chromatica::io
pub use scala_chromatica::io::*;
pub use scala_chromatica::{ColorMapError, Result};


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
