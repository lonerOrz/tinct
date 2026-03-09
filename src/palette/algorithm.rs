//! Algorithm applications for HCT color adjustment
//!
//! This module provides functions for applying algorithm parameters
//! to HCT colors.

use crate::color;
use super::params::AlgorithmParameters;

/// Apply algorithm parameters to an HCT color
///
/// Applies hue shift, saturation adjustment, and lightness adjustment
/// to the given HCT color.
pub(crate) fn apply_algorithm_params(mut hct: color::Hct, params: &AlgorithmParameters) -> color::Hct {
    // Apply hue shift
    hct.h = (hct.h + params.hue_shift as f64) % 360.0;
    if hct.h < 0.0 {
        hct.h += 360.0;
    }

    // Apply saturation adjustment
    hct.c = color::clamp(hct.c + params.saturation_adjustment as f64, 0.0, 200.0);

    // Apply lightness adjustment
    hct.t = color::clamp(hct.t + params.lightness_adjustment as f64, 0.0, 100.0);

    hct
}

/// Helper function to calculate container tone based on base tone and theme
///
/// Container tones are adjusted relative to the base surface tone,
/// with different step sizes for dark and light modes.
pub(crate) fn container_tone(base_tone: f64, level: u8, is_dark: bool) -> f64 {
    let step = if is_dark { 2.0 } else { 4.0 };
    let tone = if is_dark {
        base_tone + step * level as f64
    } else {
        base_tone - step * level as f64
    };
    color::clamp(tone, 4.0, 100.0)
}
