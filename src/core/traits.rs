//! Core traits that define the tinct architecture
//!
//! These traits provide the abstraction layer for color spaces,
//! theme loading, template processing, and output formats.

use crate::core::{ColorFormat, Mode, Result, Theme};
use std::collections::HashMap;

/// Represents a color that can be converted between different formats
pub trait ColorSpace: Clone + std::fmt::Debug + Send + Sync {
    /// Convert to RGB format
    fn to_rgb(&self) -> Rgb;

    /// Convert to HSL format
    fn to_hsl(&self) -> Hsl;

    /// Convert to hex string
    fn to_hex(&self) -> String;

    /// Convert to hex with alpha channel
    fn to_hex8(&self) -> String;

    /// Get the red component (0-255)
    fn red(&self) -> u8;

    /// Get the green component (0-255)
    fn green(&self) -> u8;

    /// Get the blue component (0-255)
    fn blue(&self) -> u8;

    /// Get the alpha component (0.0-1.0)
    fn alpha(&self) -> f64;

    /// Get the hue (0-360)
    fn hue(&self) -> f64;

    /// Get the saturation (0-100)
    fn saturation(&self) -> f64;

    /// Get the lightness (0-100)
    fn lightness(&self) -> f64;
}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

/// HSL color representation
#[derive(Debug, Clone, Copy, Default)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
    pub a: f64,
}

/// Generate color palettes from theme data
pub trait PaletteGenerator: Send + Sync {
    /// Generate a color palette for the specified mode
    fn generate(
        &self,
        theme: &serde_json::Value,
        mode: Mode,
    ) -> Result<HashMap<String, ColorFormat>>;
}

/// Process templates with theme data
///
/// This trait is dyn-compatible and allows rendering templates
pub trait TemplateEngine: Send + Sync {
    /// Render a template with the given theme
    fn render(&self, template: &str, theme: &Theme, mode: Mode) -> Result<String>;
}

/// Load themes from various sources
pub trait ThemeLoader {
    /// Load a theme from the given source
    fn load(&self, source: &str) -> Result<Theme>;

    /// Load a theme from a JSON value
    fn load_value(&self, json: &serde_json::Value) -> Result<Theme>;

    /// Check if this loader can handle the given source
    fn can_load(&self, source: &str) -> bool;
}

/// Output formats for processed themes
pub trait OutputFormat: Send + Sync {
    /// Write the processed content to the output destination
    fn write(&self, content: &str, destination: &str) -> Result<()>;

    /// Get the format name
    fn format_name(&self) -> &str;
}
