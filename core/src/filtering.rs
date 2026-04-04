//! Image Filtering System
//!
//! This module provides high-quality image filtering for export operations.
//! Supports supersampling (render at higher resolution, then downsample with filtering)
//! for professional-quality output.
//!
//! # Supported Filters
//! - **Lanczos3**: High-quality resampling, excellent for downscaling
//! - **Gaussian**: Smooth, slightly softer resampling
//! - **None**: No filtering (direct render at target size)
//!
//! # Usage
//! ```ignore
//! // Render at 2x resolution, then filter down to target size
//! let filtered = apply_supersample_filter(
//!     &buffer, 
//!     2560, 1440,  // Supersampled size
//!     1280, 720,   // Target size
//!     FilterType::Lanczos3
//! );
//! ```

use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

/// Available filter types for image resampling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FilterType {
    /// No filtering - direct render at target size
    #[default]
    None,
    /// Lanczos3 filter - high quality, sharp edges
    Lanczos3,
    /// Gaussian filter - smooth, slightly soft
    Gaussian,
}

impl FilterType {
    /// Get all available filter types
    pub const ALL: [FilterType; 3] = [
        FilterType::None,
        FilterType::Lanczos3,
        FilterType::Gaussian,
    ];

    /// Get display name for UI
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterType::None => "None",
            FilterType::Lanczos3 => "Lanczos3",
            FilterType::Gaussian => "Gaussian",
        }
    }

    /// Convert to image crate's FilterType (None returns None)
    pub fn to_image_filter(&self) -> Option<image::imageops::FilterType> {
        match self {
            FilterType::None => None,
            FilterType::Lanczos3 => Some(image::imageops::FilterType::Lanczos3),
            FilterType::Gaussian => Some(image::imageops::FilterType::Gaussian),
        }
    }
}

/// Apply supersampling with filtering
///
/// Takes a high-resolution buffer and downsamples it to the target size
/// using the specified filter.
///
/// # Arguments
/// * `buffer` - RGBA buffer at supersampled resolution
/// * `super_width` - Width of the input buffer
/// * `super_height` - Height of the input buffer
/// * `target_width` - Desired output width
/// * `target_height` - Desired output height
/// * `filter` - Filter type to use for downsampling
///
/// # Returns
/// Filtered RGBA buffer at target dimensions
pub fn apply_supersample_filter(
    buffer: &[u8],
    super_width: u32,
    super_height: u32,
    target_width: u32,
    target_height: u32,
    filter: FilterType,
) -> Result<Vec<u8>, String> {
    // If no filtering or dimensions match, return buffer as-is
    if filter == FilterType::None || (super_width == target_width && super_height == target_height) {
        return Ok(buffer.to_vec());
    }

    // Get the image filter type
    let image_filter = filter.to_image_filter()
        .ok_or_else(|| "Invalid filter type".to_string())?;

    // Create image buffer from raw data
    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        super_width,
        super_height,
        buffer.to_vec(),
    )
    .ok_or_else(|| "Failed to create image buffer from raw data".to_string())?;

    // Apply resize with filtering
    let filtered: RgbaImage = image::imageops::resize(
        &img,
        target_width,
        target_height,
        image_filter,
    );

    Ok(filtered.into_raw())
}

/// Calculate supersampled dimensions
///
/// # Arguments
/// * `base_width` - Base width
/// * `base_height` - Base height
/// * `supersample` - Supersampling multiplier (1 = no supersample, 2 = 2x, etc.)
///
/// # Returns
/// (supersampled_width, supersampled_height)
pub fn calculate_supersample_dimensions(
    base_width: u32,
    base_height: u32,
    supersample: u32,
) -> (u32, u32) {
    let supersample = supersample.max(1); // Ensure at least 1x
    (base_width * supersample, base_height * supersample)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_type_conversions() {
        assert_eq!(FilterType::None.as_str(), "None");
        assert_eq!(FilterType::Lanczos3.as_str(), "Lanczos3");
        assert_eq!(FilterType::Gaussian.as_str(), "Gaussian");

        assert!(FilterType::None.to_image_filter().is_none());
        assert!(FilterType::Lanczos3.to_image_filter().is_some());
        assert!(FilterType::Gaussian.to_image_filter().is_some());
    }

    #[test]
    fn test_calculate_supersample_dimensions() {
        assert_eq!(calculate_supersample_dimensions(100, 100, 1), (100, 100));
        assert_eq!(calculate_supersample_dimensions(100, 100, 2), (200, 200));
        assert_eq!(calculate_supersample_dimensions(100, 100, 4), (400, 400));
        
        // Test with 0 (should be clamped to 1)
        assert_eq!(calculate_supersample_dimensions(100, 100, 0), (100, 100));
    }

    #[test]
    fn test_apply_supersample_filter_none() {
        // Create a simple 2x2 buffer
        let buffer = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue
            255, 255, 255, 255, // White
        ];

        // No filter should return identical buffer
        let result = apply_supersample_filter(
            &buffer,
            2, 2,
            2, 2,
            FilterType::None
        ).unwrap();

        assert_eq!(result, buffer);
    }

    #[test]
    fn test_apply_supersample_filter_resize() {
        // Create a 4x4 solid red buffer
        let mut buffer = Vec::with_capacity(4 * 4 * 4);
        for _ in 0..16 {
            buffer.extend_from_slice(&[255, 0, 0, 255]); // Red
        }

        // Downsample to 2x2
        let result = apply_supersample_filter(
            &buffer,
            4, 4,
            2, 2,
            FilterType::Lanczos3
        ).unwrap();

        // Should be 2x2 = 4 pixels * 4 channels = 16 bytes
        assert_eq!(result.len(), 2 * 2 * 4);
        
        // All pixels should still be predominantly red
        for i in 0..4 {
            let r = result[i * 4];
            let g = result[i * 4 + 1];
            let b = result[i * 4 + 2];
            assert!(r > 200); // Should be mostly red
            assert!(g < 50);  // Minimal green
            assert!(b < 50);  // Minimal blue
        }
    }
}
