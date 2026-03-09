//! Algorithm applications for HCT color adjustment
//!
//! This module provides functions for applying algorithm parameters
//! to HCT colors.

use crate::color;
use super::params::AlgorithmParameters;
use super::constants::{
    HUE_WRAP_AROUND, CHROMA_CLAMP_MIN, CHROMA_CLAMP_MAX,
    TONE_CLAMP_MIN, TONE_CLAMP_MAX,
    CONTAINER_TONE_CLAMP_MIN, CONTAINER_TONE_CLAMP_MAX,
    SURFACE_CONTAINER_STEP_DARK, SURFACE_CONTAINER_STEP_LIGHT,
};

/// Apply algorithm parameters to an HCT color
///
/// Applies hue shift, saturation adjustment, and lightness adjustment
/// to the given HCT color.
pub(crate) fn apply_algorithm_params(mut hct: color::Hct, params: &AlgorithmParameters) -> color::Hct {
    // Apply hue shift
    hct.h = (hct.h + params.hue_shift as f64) % HUE_WRAP_AROUND;
    if hct.h < 0.0 {
        hct.h += HUE_WRAP_AROUND;
    }

    // Apply saturation adjustment
    hct.c = color::clamp(hct.c + params.saturation_adjustment as f64, CHROMA_CLAMP_MIN, CHROMA_CLAMP_MAX);

    // Apply lightness adjustment
    hct.t = color::clamp(hct.t + params.lightness_adjustment as f64, TONE_CLAMP_MIN, TONE_CLAMP_MAX);

    hct
}

/// Helper function to calculate container tone based on base tone and theme
///
/// Container tones are adjusted relative to the base surface tone,
/// with different step sizes for dark and light modes.
pub(crate) fn container_tone(base_tone: f64, level: u8, is_dark: bool) -> f64 {
    let step = if is_dark { SURFACE_CONTAINER_STEP_DARK } else { SURFACE_CONTAINER_STEP_LIGHT };
    let tone = if is_dark {
        base_tone + step * level as f64
    } else {
        base_tone - step * level as f64
    };
    color::clamp(tone, CONTAINER_TONE_CLAMP_MIN, CONTAINER_TONE_CLAMP_MAX)
}
