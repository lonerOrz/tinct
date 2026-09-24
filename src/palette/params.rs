//! Seed-level parameters for Material Design 3 palette generation.
//!
//! Material Design 3 already defines every chromatic relationship between the
//! roles (secondary is a desaturated variant of the seed hue, tertiary sits at
//! seed hue + 60°, neutrals carry only 4–8 chroma, contrast is solved for WCAG
//! ratios, …). The official `material-colors` scheme constructors encode all of
//! that.
//!
//! These parameters therefore only *nudge the seed* before it is handed to the
//! official constructor. They never reimplement MD3 math, which is what makes
//! the resulting palette stay faithful to Material You.

/// Adjustments applied to the seed color before scheme generation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlgorithmParameters {
    /// Scale the seed chroma by this percentage (-100..=100).
    /// `0` keeps the seed as-is, `-100` makes it neutral, `100` doubles it.
    pub saturation_adjustment: i8,

    /// Rotate the seed hue by this many degrees (-180..=180).
    pub hue_shift: i16,

    /// MD3 contrast level (-1.0..=1.0, `0.0` = the design as specified).
    pub contrast_level: f64,
}

impl Default for AlgorithmParameters {
    fn default() -> Self {
        Self {
            saturation_adjustment: 0,
            hue_shift: 0,
            contrast_level: 0.0,
        }
    }
}

impl AlgorithmParameters {
    /// Clamp every field into the range the MD3 pipeline can consume.
    pub fn sanitized(&self) -> Self {
        Self {
            saturation_adjustment: self.saturation_adjustment.clamp(-100, 100),
            hue_shift: self.hue_shift.clamp(-180, 180),
            contrast_level: self.contrast_level.clamp(-1.0, 1.0),
        }
    }

    /// Whether any adjustment differs from the defaults.
    pub fn is_identity(&self) -> bool {
        self.saturation_adjustment == 0 && self.hue_shift == 0 && self.contrast_level == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_identity() {
        assert!(AlgorithmParameters::default().is_identity());
    }

    #[test]
    fn test_sanitized_clamps_out_of_range() {
        let params = AlgorithmParameters {
            saturation_adjustment: 127,
            hue_shift: 300,
            contrast_level: 5.0,
        };
        let sanitized = params.sanitized();
        assert_eq!(sanitized.saturation_adjustment, 100);
        assert_eq!(sanitized.hue_shift, 180);
        assert_eq!(sanitized.contrast_level, 1.0);
    }
}
