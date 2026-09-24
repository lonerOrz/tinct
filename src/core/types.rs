//! Common type definitions for tinct.

use std::collections::HashMap;

use serde_json::Value;

use super::{Error, Result};
use crate::palette::{LegacyPaletteGenerator, extract_seed_hex};

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
/// Palette-derived maps are computed on demand — no redundant storage.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub source_color: String,
    pub dark_palette: crate::palette::Palette,
    pub light_palette: crate::palette::Palette,
}

impl Theme {
    pub fn new(name: String, source_color: String) -> Self {
        Self {
            name,
            source_color,
            dark_palette: crate::palette::Palette::empty(),
            light_palette: crate::palette::Palette::empty(),
        }
    }

    pub fn with_palettes(
        name: String,
        source_color: String,
        dark_palette: crate::palette::Palette,
        light_palette: crate::palette::Palette,
    ) -> Self {
        Self {
            name,
            source_color,
            dark_palette,
            light_palette,
        }
    }

    /// Get dark-mode colors as string-keyed map (derived from palette on demand).
    pub fn dark_colors(&self) -> HashMap<String, crate::core::color::Color> {
        self.dark_palette.to_map()
    }

    /// Get light-mode colors as string-keyed map (derived from palette on demand).
    pub fn light_colors(&self) -> HashMap<String, crate::core::color::Color> {
        self.light_palette.to_map()
    }

    /// Get a single color by role name and mode.
    pub fn get_color(&self, name: &str, mode: Mode) -> Option<crate::core::color::Color> {
        let map = match mode {
            Mode::Dark => self.dark_palette.to_map(),
            Mode::Light => self.light_palette.to_map(),
        };
        map.get(name).copied()
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
        let source_color = extract_seed_hex(json).unwrap_or("#000000").to_string();
        let dark_palette = generator.generate(json, Mode::Dark)?;
        let light_palette = generator.generate(json, Mode::Light)?;
        Ok(Self::with_palettes(
            name,
            source_color,
            dark_palette,
            light_palette,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::palette::ColorRole;

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
        let theme = Theme::new("test".to_string(), "#FF5722".to_string());
        assert_eq!(theme.name, "test");
        assert_eq!(theme.source_color, "#FF5722");
        assert!(theme.dark_colors().is_empty());
        assert!(theme.light_colors().is_empty());
    }

    #[test]
    fn test_theme_get_color() {
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());

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
        assert_eq!(theme.source_color, "#FF5722");
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
        assert_eq!(theme.source_color, "#AABBCC");
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
        let json = json!({ "seed": "#AABBCC", "Primary": "#112233" });
        let theme = Theme::from_json_value(&json, &test_generator()).unwrap();
        assert_eq!(theme.source_color, "#AABBCC");
    }

    /// Regression guard: every ANSI terminal role must be present in the
    /// string-keyed map so downstream templates never silently fall back to
    /// a default (e.g. `#000000`).
    #[test]
    fn test_palette_exposes_all_ansi_roles() {
        let json = json!({ "seed": "#6750A4" });
        let theme = Theme::from_json_value(&json, &test_generator()).unwrap();
        let map = theme.dark_colors();
        for role in [
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
        ] {
            assert!(map.contains_key(role), "missing ANSI role: {}", role);
        }
    }
}
