//! Common type definitions for tinct

use std::collections::HashMap;

// Re-export palette::ColorFormat as the canonical type
pub use crate::palette::ColorFormat;

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

/// A color theme containing all color values for both modes
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub source_color: String,
    pub dark_palette: crate::palette::Palette,
    pub light_palette: crate::palette::Palette,
    dark_color_map: HashMap<String, ColorFormat>,
    light_color_map: HashMap<String, ColorFormat>,
}

impl Theme {
    pub fn new(name: String, source_color: String) -> Self {
        Self {
            name,
            source_color,
            dark_palette: crate::palette::Palette::empty(),
            light_palette: crate::palette::Palette::empty(),
            dark_color_map: HashMap::new(),
            light_color_map: HashMap::new(),
        }
    }

    pub fn with_palettes(
        name: String,
        source_color: String,
        dark_palette: crate::palette::Palette,
        light_palette: crate::palette::Palette,
    ) -> Self {
        let mut theme = Self {
            name,
            source_color,
            dark_palette,
            light_palette,
            dark_color_map: HashMap::new(),
            light_color_map: HashMap::new(),
        };
        theme.build_color_maps();
        theme
    }

    pub fn build_color_maps(&mut self) {
        self.dark_color_map = self.dark_palette.to_map();
        self.light_color_map = self.light_palette.to_map();
    }

    pub fn dark_colors(&self) -> &HashMap<String, ColorFormat> {
        &self.dark_color_map
    }

    pub fn light_colors(&self) -> &HashMap<String, ColorFormat> {
        &self.light_color_map
    }

    pub fn get_color(&self, name: &str, mode: Mode) -> Option<ColorFormat> {
        let map = match mode {
            Mode::Dark => &self.dark_color_map,
            Mode::Light => &self.light_color_map,
        };
        map.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Palette entries exist but have empty hex values
        assert!(theme.dark_palette.primary.hex.is_empty());
        assert!(theme.light_palette.primary.hex.is_empty());
    }

    #[test]
    fn test_theme_get_color() {
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());

        let color = crate::palette::ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 1.0)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        };

        theme.dark_palette.primary = color.clone();
        theme.light_palette.primary = color;
        theme.build_color_maps();

        assert!(theme.get_color("primary", Mode::Dark).is_some());
        assert!(theme.get_color("primary", Mode::Light).is_some());
        assert!(theme.get_color("nonexistent", Mode::Dark).is_none());
    }
}
