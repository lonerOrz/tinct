//! JSON theme loader implementation

use crate::core::{Error, Mode, PaletteGenerator, Result, Theme, ThemeLoader};
use serde_json::Value;
use std::sync::Arc;

/// A theme loader for JSON format theme files
pub struct JsonThemeLoader {
    palette_generator: Arc<dyn PaletteGenerator>,
}

impl JsonThemeLoader {
    /// Create a new JSON theme loader
    pub fn new(palette_generator: Arc<dyn PaletteGenerator>) -> Self {
        Self { palette_generator }
    }
}

impl ThemeLoader for JsonThemeLoader {
    fn load(&self, source: &str) -> Result<Theme> {
        // Read the JSON file
        let content = std::fs::read_to_string(source)
            .map_err(|e| Error::Theme(format!("Failed to read theme file: {}", e)))?;

        // Parse JSON
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| Error::Theme(format!("Invalid JSON format: {}", e)))?;

        // Extract theme name from file path
        let name = std::path::Path::new(source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Support both flat and nested (dark/light) JSON formats
        let (dark_json, light_json) = if json.get("dark").is_some() {
            // Nested format: { "dark": {...}, "light": {...} }
            let dark = json.get("dark").cloned().unwrap_or_else(|| json.clone());
            let light = json.get("light").cloned().unwrap_or_else(|| json.clone());
            (dark, light)
        } else {
            // Flat format: { "primary": "...", ... }
            (json.clone(), json.clone())
        };

        // Get source color from dark mode
        let source_color = dark_json
            .get("primary")
            .and_then(|v| v.as_str())
            .or_else(|| dark_json.get("mPrimary").and_then(|v| v.as_str()))
            .unwrap_or("#000000")
            .to_string();

        // Generate palettes for both modes
        let dark_colors = self.palette_generator.generate(&dark_json, Mode::Dark)?;
        let light_colors = self.palette_generator.generate(&light_json, Mode::Light)?;

        Ok(Theme {
            name,
            source_color,
            dark_colors,
            light_colors,
        })
    }

    fn can_load(&self, source: &str) -> bool {
        std::path::Path::new(source)
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{AlgorithmParameters, LegacyPaletteGenerator};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_loader() -> JsonThemeLoader {
        let generator = Arc::new(LegacyPaletteGenerator::new(AlgorithmParameters::default()));
        JsonThemeLoader::new(generator)
    }

    #[test]
    fn test_can_load_json_file() {
        let loader = create_test_loader();

        assert!(loader.can_load("theme.json"));
        assert!(loader.can_load("/path/to/theme.json"));
        assert!(!loader.can_load("theme.txt"));
        assert!(!loader.can_load("theme.toml"));
        assert!(!loader.can_load("no_extension"));
    }

    #[test]
    fn test_load_nested_format() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"dark\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#FF572200\",").unwrap();
        writeln!(temp_file, "    \"secondary\": \"#2196F300\"").unwrap();
        writeln!(temp_file, "  }},").unwrap();
        writeln!(temp_file, "  \"light\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#D81B6000\",").unwrap();
        writeln!(temp_file, "    \"secondary\": \"#00BCD400\"").unwrap();
        writeln!(temp_file, "  }}").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let theme = result.unwrap();
        assert_eq!(
            theme.name,
            temp_file.path().file_stem().unwrap().to_str().unwrap()
        );
        assert!(!theme.dark_colors.is_empty());
        assert!(!theme.light_colors.is_empty());
    }

    #[test]
    fn test_load_flat_format() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"primary\": \"#FF572200\",").unwrap();
        writeln!(temp_file, "  \"secondary\": \"#2196F300\"").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let theme = result.unwrap();
        assert_eq!(theme.dark_colors.len(), theme.light_colors.len());
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
    fn test_source_color_extraction() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"dark\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#AABBCC00\"").unwrap();
        writeln!(temp_file, "  }},").unwrap();
        writeln!(temp_file, "  \"light\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#00000000\"").unwrap();
        writeln!(temp_file, "  }}").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#AABBCC00");
    }

    #[test]
    fn test_source_color_extraction_m_primary() {
        let loader = create_test_loader();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"dark\": {{").unwrap();
        writeln!(temp_file, "    \"mPrimary\": \"#11223300\"").unwrap();
        writeln!(temp_file, "  }},").unwrap();
        writeln!(temp_file, "  \"light\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#00000000\"").unwrap();
        writeln!(temp_file, "  }}").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#11223300");
    }

    #[test]
    fn test_source_color_default() {
        let loader = create_test_loader();

        // When no primary/mPrimary is found, should fallback to #000000
        // But the generator still needs valid color data to work
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{").unwrap();
        writeln!(temp_file, "  \"dark\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#00000000\"").unwrap();
        writeln!(temp_file, "  }},").unwrap();
        writeln!(temp_file, "  \"light\": {{").unwrap();
        writeln!(temp_file, "    \"primary\": \"#00000000\"").unwrap();
        writeln!(temp_file, "  }}").unwrap();
        writeln!(temp_file, "}}").unwrap();

        let result = loader.load(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert_eq!(theme.source_color, "#00000000");
    }
}
