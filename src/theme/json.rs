//! JSON theme loader implementation
//!
//! Supports simplified theme format without dark/light nesting:
//! - Format 1: `{ "seed": "#7aa2f7" }`
//! - Format 2: `{ "Primary": "#7aa2f7", "Secondary": "#bb9af7" }`
//! - Format 3: `{ "seed": "#7aa2f7", "Primary": "#7aa2f7", ... }`

use crate::core::{Error, Mode, Result, Theme};
use crate::palette::{extract_seed_hex, LegacyPaletteGenerator};
use serde_json::Value;

/// A theme loader for JSON format theme files
pub struct JsonThemeLoader {
    palette_generator: LegacyPaletteGenerator,
}

impl JsonThemeLoader {
    /// Create a new JSON theme loader
    pub fn new(palette_generator: LegacyPaletteGenerator) -> Self {
        Self { palette_generator }
    }

    /// Load a theme from a JSON file
    pub fn load(&self, source: &str) -> Result<Theme> {
        let content = std::fs::read_to_string(source)
            .map_err(|e| Error::Theme(format!("Failed to read theme file: {}", e)))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| Error::Theme(format!("Invalid JSON format: {}", e)))?;

        let name = std::path::Path::new(source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let source_color = extract_seed_hex(&json).unwrap_or("#000000").to_string();

        let dark_palette = self.palette_generator.generate(&json, Mode::Dark)?;
        let light_palette = self.palette_generator.generate(&json, Mode::Light)?;

        Ok(Theme::with_palettes(
            name,
            source_color,
            dark_palette,
            light_palette,
        ))
    }

    /// Load a theme from a JSON value
    pub fn load_value(&self, json: &Value) -> Result<Theme> {
        let name = json
            .get("seed")
            .and_then(|v| v.as_str())
            .unwrap_or("theme")
            .to_string();

        let source_color = extract_seed_hex(json).unwrap_or("#000000").to_string();

        let dark_palette = self.palette_generator.generate(json, Mode::Dark)?;
        let light_palette = self.palette_generator.generate(json, Mode::Light)?;

        Ok(Theme::with_palettes(
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
    use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_loader() -> JsonThemeLoader {
        let generator = LegacyPaletteGenerator::new(AlgorithmParameters::default());
        JsonThemeLoader::new(generator)
    }

    #[test]
    fn test_load_nested_format() {
        // Legacy nested format (dark/light) is no longer supported
        // This test verifies that the loader handles it gracefully
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"dark\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#FF572200\"").unwrap();
        writeln!(temp_file, "  }},").unwrap();
        writeln!(temp_file, "  \"light\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#D81B6000\"").unwrap();
        writeln!(temp_file, "  }}").unwrap();
        writeln!(temp_file, "}}").unwrap();

        // Legacy format without seed/Primary at top level should fail
        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("seed") || err_msg.contains("Primary"));
    }

    #[test]
    fn test_load_flat_format() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"seed\": \"#FF5722\",").unwrap();
        writeln!(temp_file, "  \"Primary\": \"#FF5722\",").unwrap();
        writeln!(temp_file, "  \"Secondary\": \"#2196F3\"").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let theme = result.unwrap();
        assert_eq!(theme.dark_colors().len(), theme.light_colors().len());
    }

    #[test]
    fn test_load_invalid_json() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{ invalid json }}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let loader = create_test_loader();

        let result = loader.load("/nonexistent/path/theme.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn test_source_color_extraction_seed() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"seed\": \"#AABBCC\"").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#AABBCC");
    }

    #[test]
    fn test_source_color_extraction_primary() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"Primary\": \"#112233\"").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#112233");
    }

    #[test]
    fn test_source_color_seed_priority() {
        let loader = create_test_loader();

        // seed has priority over Primary
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"seed\": \"#AABBCC\",").unwrap();
        writeln!(temp_file, "  \"Primary\": \"#112233\"").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#AABBCC");
    }
}
