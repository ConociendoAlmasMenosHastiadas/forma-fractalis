//! Extended Parameter System Design (Future Refactoring)
//!
//! This module documents the design for an extended parameter system that supports
//! both numeric and enum-based parameters for fractals.
//!
//! # Current Implementation (v0.1.8)
//! For Tetration fractal, we use a **pragmatic approach**:
//! - Threshold: Regular numeric parameter in HashMap<String, f64>
//! - Escape mode: Encoded as f64 (0.0=magnitude, 1.0=real, 2.0=imaginary, 3.0=either)
//! - Radio buttons: Hard-coded in GUI (like other fractals)
//!
//! # Future Refactoring (v0.2.x)
//! When more fractals need enum parameters, refactor to a generic system:
//! - ParameterType enum (Numeric | Choice)
//! - Separate storage for enum values
//! - Dynamic GUI generation for radio buttons/dropdowns
//!
//! # Example Migration Path
//! ```ignore
//! // Old (v0.1.8)
//! params.insert("escape_mode", 0.0); // 0=magnitude
//!
//! // New (v0.2.x - future)
//! params_enum.insert("escape_mode", "magnitude");
//! ```

/// Escape criterion modes for Tetration (and potentially other fractals)
///
/// This enum represents the different ways to test if a point has escaped.
/// Currently encoded as f64 for compatibility with existing HashMap storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EscapeMode {
    /// Test if magnitude exceeds threshold: |z| > threshold
    Magnitude = 0,
    /// Test if real component exceeds threshold: Re(z) > threshold  
    Real = 1,
    /// Test if imaginary component exceeds threshold: Im(z) > threshold
    Imaginary = 2,
    /// Test if either component exceeds threshold: Re(z) > threshold OR Im(z) > threshold
    Either = 3,
}

impl EscapeMode {
    /// Convert f64 encoding to EscapeMode
    pub fn from_f64(value: f64) -> Self {
        match value as i32 {
            1 => EscapeMode::Real,
            2 => EscapeMode::Imaginary,
            3 => EscapeMode::Either,
            _ => EscapeMode::Magnitude, // Default
        }
    }

    /// Convert EscapeMode to f64 for HashMap storage
    pub fn to_f64(self) -> f64 {
        self as i32 as f64
    }

    /// Get display name for GUI
    pub fn display_name(self) -> &'static str {
        match self {
            EscapeMode::Magnitude => "Magnitude",
            EscapeMode::Real => "Real Component",
            EscapeMode::Imaginary => "Imaginary Component",
            EscapeMode::Either => "Either Component",
        }
    }

    /// Get all modes (for radio button generation)
    pub fn all() -> [EscapeMode; 4] {
        [
            EscapeMode::Magnitude,
            EscapeMode::Real,
            EscapeMode::Imaginary,
            EscapeMode::Either,
        ]
    }

    /// Test if escape condition is met
    pub fn test(&self, z_real: f64, z_imag: f64, threshold: f64) -> bool {
        match self {
            EscapeMode::Magnitude => z_real * z_real + z_imag * z_imag > threshold * threshold,
            EscapeMode::Real => z_real.abs() > threshold,
            EscapeMode::Imaginary => z_imag.abs() > threshold,
            EscapeMode::Either => z_real.abs() > threshold || z_imag.abs() > threshold,
        }
    }
}

impl Default for EscapeMode {
    fn default() -> Self {
        EscapeMode::Magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_mode_conversion() {
        assert_eq!(EscapeMode::from_f64(0.0), EscapeMode::Magnitude);
        assert_eq!(EscapeMode::from_f64(1.0), EscapeMode::Real);
        assert_eq!(EscapeMode::from_f64(2.0), EscapeMode::Imaginary);
        assert_eq!(EscapeMode::from_f64(3.0), EscapeMode::Either);
        
        assert_eq!(EscapeMode::Magnitude.to_f64(), 0.0);
        assert_eq!(EscapeMode::Real.to_f64(), 1.0);
    }

    #[test]
    fn test_escape_mode_names() {
        assert_eq!(EscapeMode::Magnitude.display_name(), "Magnitude");
        assert_eq!(EscapeMode::Real.display_name(), "Real Component");
    }

    #[test]
    fn test_escape_mode_testing() {
        let threshold = 10.0;
        
        // Test magnitude
        assert!(EscapeMode::Magnitude.test(8.0, 8.0, threshold)); // |z| = ~11.3 > 10
        assert!(!EscapeMode::Magnitude.test(5.0, 5.0, threshold)); // |z| = ~7.07 < 10
        
        // Test real component
        assert!(EscapeMode::Real.test(11.0, 2.0, threshold)); // Re(z) = 11 > 10
        assert!(!EscapeMode::Real.test(9.0, 20.0, threshold)); // Re(z) = 9 < 10
        
        // Test imaginary component
        assert!(EscapeMode::Imaginary.test(2.0, 11.0, threshold)); // Im(z) = 11 > 10
        assert!(!EscapeMode::Imaginary.test(20.0, 9.0, threshold)); // Im(z) = 9 < 10
        
        // Test either component
        assert!(EscapeMode::Either.test(11.0, 5.0, threshold)); // Re(z) = 11 > 10
        assert!(EscapeMode::Either.test(5.0, 11.0, threshold)); // Im(z) = 11 > 10
        assert!(!EscapeMode::Either.test(5.0, 5.0, threshold)); // Both < 10
    }
}
