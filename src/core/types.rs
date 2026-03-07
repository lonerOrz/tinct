//! Common type definitions for tinct

use crate::core::traits::{ColorSpace, Hsl, Rgb};
use std::collections::HashMap;

/// Unified color format that implements ColorSpace
#[derive(Debug, Clone)]
pub struct ColorFormat {
    pub hex: String,
    pub hex_stripped: String,
    pub hex8: String,
    pub hex8_stripped: String,
    pub rgb: String,
    pub rgba: String,
    pub hsl: String,
    pub hsla: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64,
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

impl ColorSpace for ColorFormat {
    fn to_rgb(&self) -> Rgb {
        Rgb {
            r: self.red,
            g: self.green,
            b: self.blue,
            a: self.alpha,
        }
    }

    fn to_hsl(&self) -> Hsl {
        Hsl {
            h: self.hue,
            s: self.saturation,
            l: self.lightness,
            a: self.alpha,
        }
    }

    fn to_hex(&self) -> String {
        self.hex.clone()
    }

    fn to_hex8(&self) -> String {
        self.hex8.clone()
    }

    fn red(&self) -> u8 {
        self.red
    }

    fn green(&self) -> u8 {
        self.green
    }

    fn blue(&self) -> u8 {
        self.blue
    }

    fn alpha(&self) -> f64 {
        self.alpha
    }

    fn hue(&self) -> f64 {
        self.hue
    }

    fn saturation(&self) -> f64 {
        self.saturation
    }

    fn lightness(&self) -> f64 {
        self.lightness
    }
}

/// Theme mode (dark or light)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub dark_colors: HashMap<String, ColorFormat>,
    pub light_colors: HashMap<String, ColorFormat>,
}

impl Theme {
    pub fn new(name: String, source_color: String) -> Self {
        Self {
            name,
            source_color,
            dark_colors: HashMap::new(),
            light_colors: HashMap::new(),
        }
    }

    pub fn get_color(&self, name: &str, mode: Mode) -> Option<&ColorFormat> {
        match mode {
            Mode::Dark => self.dark_colors.get(name),
            Mode::Light => self.light_colors.get(name),
        }
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
        assert!(theme.dark_colors.is_empty());
        assert!(theme.light_colors.is_empty());
    }

    #[test]
    fn test_theme_get_color() {
        let mut theme = Theme::new("test".to_string(), "#FF5722".to_string());
        
        let color = ColorFormat {
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
        };
        
        theme.dark_colors.insert("primary".to_string(), color.clone());
        theme.light_colors.insert("primary".to_string(), color.clone());
        
        assert!(theme.get_color("primary", Mode::Dark).is_some());
        assert!(theme.get_color("primary", Mode::Light).is_some());
        assert!(theme.get_color("nonexistent", Mode::Dark).is_none());
    }
}
