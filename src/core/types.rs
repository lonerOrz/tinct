//! Common type definitions for tinct.

use std::collections::HashMap;

use serde_json::Value;

use super::{Error, Result};
use crate::core::color::Color;
use crate::palette::{LegacyPaletteGenerator, Palette};

/// Theme mode (dark or light)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn is_dark(&self) -> bool {
        matches!(self, Mode::Dark)
    }

    pub fn is_light(&self) -> bool {
        matches!(self, Mode::Light)
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Dark => write!(f, "dark"),
            Mode::Light => write!(f, "light"),
        }
    }
}

/// A color theme containing all color values for both modes.
///
/// Every exposed value is a spec-compliant MD3 role stored in the palettes;
/// the seed itself is *not* a role and is never surfaced to templates.
/// Palette-derived maps are computed on demand — no redundant storage.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub dark_palette: Palette,
    pub light_palette: Palette,
}

impl Theme {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dark_palette: Palette::empty(),
            light_palette: Palette::empty(),
        }
    }

    pub fn with_palettes(name: String, dark_palette: Palette, light_palette: Palette) -> Self {
        Self {
            name,
            dark_palette,
            light_palette,
        }
    }

    /// Get dark-mode colors as string-keyed map (derived from palette on demand).
    pub fn dark_colors(&self) -> HashMap<String, Color> {
        self.dark_palette.to_map()
    }

    /// Get light-mode colors as string-keyed map (derived from palette on demand).
    pub fn light_colors(&self) -> HashMap<String, Color> {
        self.light_palette.to_map()
    }

    /// Get a single color by role name and mode.
    pub fn get_color(&self, name: &str, mode: Mode) -> Option<Color> {
        let palette = match mode {
            Mode::Dark => &self.dark_palette,
            Mode::Light => &self.light_palette,
        };
        palette.get(name).copied()
    }

    /// Build a theme from an already-parsed JSON document.
    ///
    /// `generator` supplies the seed adjustments and MD3 scheme variant. The
    /// document may be a flat `{ "seed": "#RRGGBB" }` object or a nested
    /// `{ "dark": { ... }, "light": { ... } }` theme with role overrides.
    pub fn from_json_value(json: &Value, generator: &LegacyPaletteGenerator) -> Result<Self> {
        let name = json
            .get("seed")
            .and_then(|v| v.as_str())
            .unwrap_or("theme")
            .to_string();
        Self::from_source(name, json, generator)
    }

    /// Build a theme from a JSON file on disk.
    pub fn from_json_file(source: &str, generator: &LegacyPaletteGenerator) -> Result<Self> {
        let content = std::fs::read_to_string(source)
            .map_err(|e| Error::Theme(format!("Failed to read theme file: {}", e)))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| Error::Theme(format!("Invalid JSON format: {}", e)))?;

        let name = std::path::Path::new(source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self::from_source(name, &json, generator)
    }

    fn from_source(name: String, json: &Value, generator: &LegacyPaletteGenerator) -> Result<Self> {
        let dark_palette = generator.generate(json, Mode::Dark)?;
        let light_palette = generator.generate(json, Mode::Light)?;
        Ok(Self::with_palettes(name, dark_palette, light_palette))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::palette::ColorRole;
    use crate::palette::test_support::ANSI_ROLE_NAMES;

    #[test]
    fn test_mode_display() {
        assert_eq!(Mode::Dark.to_string(), "dark");
        assert_eq!(Mode::Light.to_string(), "light");
    }

    #[test]
    fn test_mode_predicates() {
        assert!(Mode::Dark.is_dark());
        assert!(!Mode::Dark.is_light());
        assert!(Mode::Light.is_light());
        assert!(!Mode::Light.is_dark());
    }

    #[test]
    fn test_theme_new() {
        let theme = Theme::new("test".to_string());
        assert_eq!(theme.name, "test");
        assert!(theme.dark_colors().is_empty());
        assert!(theme.light_colors().is_empty());
    }

    #[test]
    fn test_theme_get_color() {
        let mut theme = Theme::new("test".to_string());

        let color = Color::new(255, 87, 34, 1.0);
        theme.dark_palette.insert(ColorRole::Primary, color);
        theme.light_palette.insert(ColorRole::Primary, color);
        theme
            .dark_palette
            .insert(ColorRole::Background, Color::new(30, 30, 30, 1.0));

        assert!(theme.get_color("primary", Mode::Dark).is_some());
        assert!(theme.get_color("primary", Mode::Light).is_some());
        assert!(theme.get_color("nonexistent", Mode::Dark).is_none());

        // Verify maps are derived, not cached
        let dark = theme.dark_colors();
        assert!(dark.contains_key("primary"));
        assert!(dark.contains_key("background"));
    }

    // ---- JSON theme construction (formerly theme/json.rs) ----

    use crate::image::SchemeType;
    use crate::palette::AlgorithmParameters;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn test_generator() -> LegacyPaletteGenerator {
        LegacyPaletteGenerator::new(AlgorithmParameters::default(), SchemeType::TonalSpot)
    }

    #[test]
    fn test_from_json_value_flat() {
        let json = json!({ "seed": "#FF5722", "Primary": "#FF5722" });
        let theme = Theme::from_json_value(&json, &test_generator()).unwrap();
        assert_eq!(theme.dark_colors().len(), theme.light_colors().len());
        assert!(!theme.dark_colors().is_empty());
    }

    #[test]
    fn test_from_json_value_nested_without_seed_errors() {
        let json = json!({ "dark": { "primary": "#FF5722" }, "light": { "primary": "#D81B60" } });
        let result = Theme::from_json_value(&json, &test_generator());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("seed") || err_msg.contains("Primary"));
    }

    #[test]
    fn test_from_json_file_reads_stem_as_name() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{ \"seed\": \"#AABBCC\" }}").unwrap();
        let theme =
            Theme::from_json_file(temp_file.path().to_str().unwrap(), &test_generator()).unwrap();
        assert_eq!(
            theme.name,
            temp_file.path().file_stem().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn test_from_json_file_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{ invalid json }}").unwrap();
        let err = Theme::from_json_file(temp_file.path().to_str().unwrap(), &test_generator())
            .unwrap_err()
            .to_string();
        assert!(err.contains("Invalid JSON"));
    }

    #[test]
    fn test_from_json_file_missing() {
        let err = Theme::from_json_file("/nonexistent/path/theme.json", &test_generator())
            .unwrap_err()
            .to_string();
        assert!(err.contains("Failed to read"));
    }

    #[test]
    fn test_seed_priority_over_primary() {
        // `Primary` overrides only that single role; an untouched role proves
        // the MD3 scheme is generated from `seed`, not from the `Primary` key.
        let a = Theme::from_json_value(
            &json!({ "seed": "#AABBCC", "Primary": "#112233" }),
            &test_generator(),
        )
        .unwrap();
        let b = Theme::from_json_value(
            &json!({ "seed": "#FF0000", "Primary": "#112233" }),
            &test_generator(),
        )
        .unwrap();
        assert_ne!(
            a.get_color("secondary", Mode::Dark),
            b.get_color("secondary", Mode::Dark)
        );
    }

    /// Regression guard: every ANSI terminal role must be present in the
    /// string-keyed map so downstream templates never silently fall back to
    /// a default (e.g. `#000000`).
    #[test]
    fn test_palette_exposes_all_ansi_roles() {
        let json = json!({ "seed": "#6750A4" });
        let theme = Theme::from_json_value(&json, &test_generator()).unwrap();
        let map = theme.dark_colors();
        for role in ANSI_ROLE_NAMES {
            assert!(map.contains_key(role), "missing ANSI role: {}", role);
        }
    }
}
