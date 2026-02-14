//! Numerical Utilities and Constants
//!
//! This module provides common numerical constants and utility functions
//! used throughout the fractal rendering system.

/// Absolute epsilon - the threshold for treating numbers as effectively zero.
/// Used to avoid division by near-zero values and detect convergence.
///
/// Value: 1e-10
///
/// # Usage
/// ```ignore
/// use forma_fractalis::number_utils::ABSOLUTE_EPSILON;
///
/// if magnitude < ABSOLUTE_EPSILON {
///     // Treat as zero
/// }
/// ```
pub const ABSOLUTE_EPSILON: f64 = 1e-15;

/// Two times Pi - a full circle in radians.
/// Used for rotation angles and periodic parameters.
///
/// Value: 2π ≈ 6.283185307179586
///
/// # Usage
/// ```ignore
/// use forma_fractalis::number_utils::TWO_PI;
///
/// let angle = 0.5 * TWO_PI; // 180 degrees
/// ```
pub const TWO_PI: f64 = std::f64::consts::TAU;
