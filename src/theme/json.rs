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
