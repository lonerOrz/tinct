//! Algorithm parameters for controlling color generation
//!
//! This module provides configuration for the palette generation algorithm.

/// Parameters for controlling the color generation algorithm
#[derive(Debug, Clone)]
pub struct AlgorithmParameters {
    /// Color contrast threshold (0.0-1.0)
    pub contrast_threshold: f64,
    /// Saturation adjustment (-100 to 100)
    pub saturation_adjustment: i8,
    /// Lightness adjustment (-100 to 100)
    pub lightness_adjustment: i8,
    /// Hue shift (-180 to 180)
    pub hue_shift: i16,
    /// Minimum contrast ratio for readability
    pub min_contrast_ratio: f64,
}

impl Default for AlgorithmParameters {
    fn default() -> Self {
        Self {
            contrast_threshold: 0.15,
            saturation_adjustment: 0,
            lightness_adjustment: 0,
            hue_shift: 0,
            min_contrast_ratio: 4.5,
        }
    }
}
