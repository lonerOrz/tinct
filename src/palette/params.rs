//! Algorithm parameters for controlling color generation
//!
//! This module provides configuration for the palette generation algorithm.

/// Color harmony mode for secondary/tertiary hue relationships
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorHarmony {
    /// MD3 standard hue relationships
    #[default]
    Md3,
    /// Analogous colors (close hues)
    Analogous,
    /// Complementary colors (opposite hues)
    Complementary,
    /// Triadic colors (120° apart)
    Triadic,
    /// Split-complementary (150° and 210°)
    SplitComplementary,
}

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
    /// MD3 contrast level (-1.0 to 1.0, 0.0 = standard)
    pub contrast_level: f64,
    /// Color harmony mode for hue relationships
    pub color_harmony: ColorHarmony,
}

impl Default for AlgorithmParameters {
    fn default() -> Self {
        Self {
            contrast_threshold: 0.15,
            saturation_adjustment: 0,
            lightness_adjustment: 0,
            hue_shift: 0,
            min_contrast_ratio: 4.5,
            contrast_level: 0.0,
            color_harmony: ColorHarmony::Md3,
        }
    }
}

impl ColorHarmony {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md3" | "material" => Some(Self::Md3),
            "analogous" | "analog" => Some(Self::Analogous),
            "complementary" | "complement" => Some(Self::Complementary),
            "triadic" | "triad" => Some(Self::Triadic),
            "split-complementary" | "split" => Some(Self::SplitComplementary),
            _ => None,
        }
    }
}
