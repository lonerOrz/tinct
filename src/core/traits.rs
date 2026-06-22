//! Core traits that define the tinct architecture
//!
//! These traits provide the abstraction layer for theme loading,
//! template processing, and output formats.

use crate::core::{Mode, Result, Theme};
use crate::palette::Palette;

/// Generate color palettes from theme data
pub trait PaletteGenerator: Send + Sync {
    /// Generate a color palette for the specified mode
    fn generate(&self, theme: &serde_json::Value, mode: Mode) -> Result<Palette>;
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
