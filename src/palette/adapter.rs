//! Adapter for the legacy palette generator module

use crate::core::{Error, Mode, Result};
use crate::palette::{generate_palette_with_params, AlgorithmParameters, ColorHarmony, Palette};
use serde_json::Value;

/// Adapter that wraps the legacy palette generator function
pub struct LegacyPaletteGenerator {
    params: AlgorithmParameters,
}

impl LegacyPaletteGenerator {
    pub fn new(params: AlgorithmParameters) -> Self {
        Self { params }
    }

    pub fn with_defaults() -> Self {
        Self {
            params: AlgorithmParameters {
                saturation_adjustment: 0,
                hue_shift: 0,
                contrast_level: 0.0,
                color_harmony: ColorHarmony::Md3,
            },
        }
    }

    pub fn generate(&self, theme: &Value, mode: Mode) -> Result<Palette> {
        generate_palette_with_params(theme, mode.is_dark(), self.params.clone())
            .map_err(Error::Palette)
    }
}

impl Default for LegacyPaletteGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_legacy_palette_generator_new() {
        let params = AlgorithmParameters {
            saturation_adjustment: 10,
            hue_shift: 15,
            contrast_level: 0.0,
            color_harmony: ColorHarmony::Md3,
        };
        let generator = LegacyPaletteGenerator::new(params.clone());
        assert_eq!(generator.params.saturation_adjustment, 10);
    }

    #[test]
    fn test_legacy_palette_generator_with_defaults() {
        let generator = LegacyPaletteGenerator::with_defaults();
        assert_eq!(generator.params.saturation_adjustment, 0);
        assert_eq!(generator.params.hue_shift, 0);
    }

    #[test]
    fn test_legacy_palette_generator_default() {
        let generator = LegacyPaletteGenerator::default();
        assert_eq!(generator.params.saturation_adjustment, 0);
    }

    #[test]
    fn test_legacy_palette_generator_generate_dark_mode() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "seed": "#FF5722"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        assert!(!map.is_empty());
        assert!(map.contains_key("primary"));
        assert!(map.contains_key("secondary"));
        assert!(map.contains_key("tertiary"));
        assert!(map.contains_key("surface"));
        assert!(map.contains_key("error"));

        // Verify primary color has expected format
        let primary = map.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
        assert!(primary.hex.starts_with("#"));
    }

    #[test]
    fn test_legacy_palette_generator_generate_light_mode() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "seed": "#2196F3"
        });

        let result = generator.generate(&theme, Mode::Light);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        assert!(!map.is_empty());

        // Light mode should have different colors than dark mode
        let primary = map.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_hue_shift() {
        let params = AlgorithmParameters {
            saturation_adjustment: 0,
            hue_shift: 180,
            contrast_level: 0.0,
            color_harmony: ColorHarmony::Md3,
        };
        let generator = LegacyPaletteGenerator::new(params);
        let theme = json!({
            "seed": "#FF0000" // Red
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        // With 180 degree hue shift, red should become cyan-like
        let primary = map.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_saturation() {
        let params = AlgorithmParameters {
            saturation_adjustment: 50,
            hue_shift: 0,
            contrast_level: 0.0,
            color_harmony: ColorHarmony::Md3,
        };
        let generator = LegacyPaletteGenerator::new(params);
        let theme = json!({
            "seed": "#FF5722"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        let primary = map.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
    }

    #[test]
    fn test_legacy_palette_generator_generate_all_color_roles() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "seed": "#6200EE"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();

        // Verify all major color roles are present
        let expected_roles = [
            "primary",
            "on_primary",
            "primary_container",
            "on_primary_container",
            "secondary",
            "on_secondary",
            "secondary_container",
            "on_secondary_container",
            "tertiary",
            "on_tertiary",
            "tertiary_container",
            "on_tertiary_container",
            "error",
            "on_error",
            "error_container",
            "on_error_container",
            "background",
            "on_background",
            "surface",
            "on_surface",
            "surface_variant",
            "on_surface_variant",
            "outline",
            "outline_variant",
            "shadow",
            "scrim",
            "inverse_surface",
            "inverse_on_surface",
            "inverse_primary",
            "surface_dim",
            "surface_bright",
            "surface_container_lowest",
            "surface_container_low",
            "surface_container",
            "surface_container_high",
            "surface_container_highest",
            // Terminal colors
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "bright_black",
            "bright_red",
            "bright_green",
            "bright_yellow",
            "bright_blue",
            "bright_magenta",
            "bright_cyan",
            "bright_white",
        ];

        for role in expected_roles.iter() {
            assert!(map.contains_key(*role), "Missing color role: {}", role);
            let color = map.get(*role).unwrap();
            assert!(!color.hex.is_empty(), "Empty hex for role: {}", role);
        }
    }

    #[test]
    fn test_legacy_palette_generator_generate_invalid_theme() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "no_seed": "value"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_err());
        // Error message should mention seed or Primary requirement
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("seed") || err_msg.contains("Primary"));
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_overrides() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "seed": "#FF5722",
            "error": "#FF0000",
            "surface": "#121212"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let palette = result.unwrap();
        let map = palette.to_map();
        // Overrides should be applied
        let error = map.get("error").unwrap();
        assert_eq!(error.hex, "#FF0000");

        let surface = map.get("surface").unwrap();
        assert_eq!(surface.hex, "#121212");
    }
}
