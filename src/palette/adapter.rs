//! Adapter for the legacy palette generator module

use crate::core::{ColorFormat, Error, Mode, PaletteGenerator, Result};
use crate::palette::{generate_palette_with_params, AlgorithmParameters, ColorEntry};
use serde_json::Value;
use std::collections::HashMap;

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
                contrast_threshold: 0.15,
                saturation_adjustment: 0,
                lightness_adjustment: 0,
                hue_shift: 0,
                min_contrast_ratio: 4.5,
            },
        }
    }
}

impl Default for LegacyPaletteGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl PaletteGenerator for LegacyPaletteGenerator {
    fn generate(&self, theme: &Value, mode: Mode) -> Result<HashMap<String, ColorFormat>> {
        // Call the legacy function
        let palette = generate_palette_with_params(theme, mode.is_dark(), self.params.clone())
            .map_err(Error::Palette)?;

        // Helper to get color from entry based on mode
        let get_color = |entry: &ColorEntry| -> ColorFormat {
            let src = if mode.is_dark() {
                &entry.dark
            } else {
                &entry.light
            };
            ColorFormat {
                hex: src.hex.clone(),
                hex_stripped: src.hex_stripped.clone(),
                hex8: src.hex8.clone(),
                hex8_stripped: src.hex8_stripped.clone(),
                rgb: src.rgb.clone(),
                rgba: src.rgba.clone(),
                hsl: src.hsl.clone(),
                hsla: src.hsla.clone(),
                red: src.red,
                green: src.green,
                blue: src.blue,
                alpha: src.alpha,
                hue: src.hue,
                saturation: src.saturation,
                lightness: src.lightness,
            }
        };

        // Convert the palette to a HashMap
        let mut colors = HashMap::new();

        colors.insert("primary".to_string(), get_color(&palette.primary));
        colors.insert("on_primary".to_string(), get_color(&palette.on_primary));
        colors.insert(
            "primary_container".to_string(),
            get_color(&palette.primary_container),
        );
        colors.insert(
            "on_primary_container".to_string(),
            get_color(&palette.on_primary_container),
        );
        colors.insert(
            "primary_fixed".to_string(),
            get_color(&palette.primary_fixed),
        );
        colors.insert(
            "primary_fixed_dim".to_string(),
            get_color(&palette.primary_fixed_dim),
        );
        colors.insert(
            "on_primary_fixed".to_string(),
            get_color(&palette.on_primary_fixed),
        );
        colors.insert(
            "on_primary_fixed_variant".to_string(),
            get_color(&palette.on_primary_fixed_variant),
        );

        colors.insert("secondary".to_string(), get_color(&palette.secondary));
        colors.insert("on_secondary".to_string(), get_color(&palette.on_secondary));
        colors.insert(
            "secondary_container".to_string(),
            get_color(&palette.secondary_container),
        );
        colors.insert(
            "on_secondary_container".to_string(),
            get_color(&palette.on_secondary_container),
        );
        colors.insert(
            "secondary_fixed".to_string(),
            get_color(&palette.secondary_fixed),
        );
        colors.insert(
            "secondary_fixed_dim".to_string(),
            get_color(&palette.secondary_fixed_dim),
        );
        colors.insert(
            "on_secondary_fixed".to_string(),
            get_color(&palette.on_secondary_fixed),
        );
        colors.insert(
            "on_secondary_fixed_variant".to_string(),
            get_color(&palette.on_secondary_fixed_variant),
        );

        colors.insert("tertiary".to_string(), get_color(&palette.tertiary));
        colors.insert("on_tertiary".to_string(), get_color(&palette.on_tertiary));
        colors.insert(
            "tertiary_container".to_string(),
            get_color(&palette.tertiary_container),
        );
        colors.insert(
            "on_tertiary_container".to_string(),
            get_color(&palette.on_tertiary_container),
        );
        colors.insert(
            "tertiary_fixed".to_string(),
            get_color(&palette.tertiary_fixed),
        );
        colors.insert(
            "tertiary_fixed_dim".to_string(),
            get_color(&palette.tertiary_fixed_dim),
        );
        colors.insert(
            "on_tertiary_fixed".to_string(),
            get_color(&palette.on_tertiary_fixed),
        );
        colors.insert(
            "on_tertiary_fixed_variant".to_string(),
            get_color(&palette.on_tertiary_fixed_variant),
        );

        colors.insert("error".to_string(), get_color(&palette.error));
        colors.insert("on_error".to_string(), get_color(&palette.on_error));
        colors.insert(
            "error_container".to_string(),
            get_color(&palette.error_container),
        );
        colors.insert(
            "on_error_container".to_string(),
            get_color(&palette.on_error_container),
        );

        colors.insert("background".to_string(), get_color(&palette.background));
        colors.insert(
            "on_background".to_string(),
            get_color(&palette.on_background),
        );
        colors.insert("surface".to_string(), get_color(&palette.surface));
        colors.insert("on_surface".to_string(), get_color(&palette.on_surface));
        colors.insert(
            "surface_variant".to_string(),
            get_color(&palette.surface_variant),
        );
        colors.insert(
            "on_surface_variant".to_string(),
            get_color(&palette.on_surface_variant),
        );

        colors.insert(
            "surface_container_lowest".to_string(),
            get_color(&palette.surface_container_lowest),
        );
        colors.insert(
            "surface_container_low".to_string(),
            get_color(&palette.surface_container_low),
        );
        colors.insert(
            "surface_container".to_string(),
            get_color(&palette.surface_container),
        );
        colors.insert(
            "surface_container_high".to_string(),
            get_color(&palette.surface_container_high),
        );
        colors.insert(
            "surface_container_highest".to_string(),
            get_color(&palette.surface_container_highest),
        );

        colors.insert(
            "inverse_surface".to_string(),
            get_color(&palette.inverse_surface),
        );
        colors.insert(
            "inverse_on_surface".to_string(),
            get_color(&palette.inverse_on_surface),
        );
        colors.insert(
            "inverse_primary".to_string(),
            get_color(&palette.inverse_primary),
        );

        colors.insert("surface_dim".to_string(), get_color(&palette.surface_dim));
        colors.insert(
            "surface_bright".to_string(),
            get_color(&palette.surface_bright),
        );

        colors.insert("outline".to_string(), get_color(&palette.outline));
        colors.insert(
            "outline_variant".to_string(),
            get_color(&palette.outline_variant),
        );

        colors.insert("shadow".to_string(), get_color(&palette.shadow));
        colors.insert("scrim".to_string(), get_color(&palette.scrim));

        // Terminal colors
        colors.insert("black".to_string(), get_color(&palette.black));
        colors.insert("red".to_string(), get_color(&palette.red));
        colors.insert("green".to_string(), get_color(&palette.green));
        colors.insert("yellow".to_string(), get_color(&palette.yellow));
        colors.insert("blue".to_string(), get_color(&palette.blue));
        colors.insert("magenta".to_string(), get_color(&palette.magenta));
        colors.insert("cyan".to_string(), get_color(&palette.cyan));
        colors.insert("white".to_string(), get_color(&palette.white));
        colors.insert("bright_black".to_string(), get_color(&palette.bright_black));
        colors.insert("bright_red".to_string(), get_color(&palette.bright_red));
        colors.insert("bright_green".to_string(), get_color(&palette.bright_green));
        colors.insert(
            "bright_yellow".to_string(),
            get_color(&palette.bright_yellow),
        );
        colors.insert("bright_blue".to_string(), get_color(&palette.bright_blue));
        colors.insert(
            "bright_magenta".to_string(),
            get_color(&palette.bright_magenta),
        );
        colors.insert("bright_cyan".to_string(), get_color(&palette.bright_cyan));
        colors.insert("bright_white".to_string(), get_color(&palette.bright_white));

        Ok(colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_legacy_palette_generator_new() {
        let params = AlgorithmParameters {
            contrast_threshold: 0.2,
            saturation_adjustment: 10,
            lightness_adjustment: 5,
            hue_shift: 15,
            min_contrast_ratio: 5.0,
        };
        let generator = LegacyPaletteGenerator::new(params.clone());
        assert_eq!(generator.params.contrast_threshold, 0.2);
        assert_eq!(generator.params.saturation_adjustment, 10);
    }

    #[test]
    fn test_legacy_palette_generator_with_defaults() {
        let generator = LegacyPaletteGenerator::with_defaults();
        assert_eq!(generator.params.contrast_threshold, 0.15);
        assert_eq!(generator.params.saturation_adjustment, 0);
        assert_eq!(generator.params.lightness_adjustment, 0);
        assert_eq!(generator.params.hue_shift, 0);
        assert_eq!(generator.params.min_contrast_ratio, 4.5);
    }

    #[test]
    fn test_legacy_palette_generator_default() {
        let generator = LegacyPaletteGenerator::default();
        assert_eq!(generator.params.contrast_threshold, 0.15);
    }

    #[test]
    fn test_legacy_palette_generator_generate_dark_mode() {
        let generator = LegacyPaletteGenerator::with_defaults();
        let theme = json!({
            "seed": "#FF5722"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let colors = result.unwrap();
        assert!(!colors.is_empty());
        assert!(colors.contains_key("primary"));
        assert!(colors.contains_key("secondary"));
        assert!(colors.contains_key("tertiary"));
        assert!(colors.contains_key("surface"));
        assert!(colors.contains_key("error"));

        // Verify primary color has expected format
        let primary = colors.get("primary").unwrap();
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

        let colors = result.unwrap();
        assert!(!colors.is_empty());

        // Light mode should have different colors than dark mode
        let primary = colors.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_hue_shift() {
        let params = AlgorithmParameters {
            contrast_threshold: 0.15,
            saturation_adjustment: 0,
            lightness_adjustment: 0,
            hue_shift: 180, // 180 degree shift
            min_contrast_ratio: 4.5,
        };
        let generator = LegacyPaletteGenerator::new(params);
        let theme = json!({
            "seed": "#FF0000" // Red
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let colors = result.unwrap();
        // With 180 degree hue shift, red should become cyan-like
        let primary = colors.get("primary").unwrap();
        assert!(!primary.hex.is_empty());
    }

    #[test]
    fn test_legacy_palette_generator_generate_with_saturation() {
        let params = AlgorithmParameters {
            contrast_threshold: 0.15,
            saturation_adjustment: 50, // +50% saturation
            lightness_adjustment: 0,
            hue_shift: 0,
            min_contrast_ratio: 4.5,
        };
        let generator = LegacyPaletteGenerator::new(params);
        let theme = json!({
            "seed": "#FF5722"
        });

        let result = generator.generate(&theme, Mode::Dark);
        assert!(result.is_ok());

        let colors = result.unwrap();
        let primary = colors.get("primary").unwrap();
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

        let colors = result.unwrap();

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
            assert!(colors.contains_key(*role), "Missing color role: {}", role);
            let color = colors.get(*role).unwrap();
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
        assert!(result.unwrap_err().to_string().contains("Seed"));
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

        let colors = result.unwrap();
        // Overrides should be applied
        let error = colors.get("error").unwrap();
        assert_eq!(error.hex, "#FF0000");

        let surface = colors.get("surface").unwrap();
        assert_eq!(surface.hex, "#121212");
    }
}
