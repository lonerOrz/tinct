//! Color type definitions for palette generation
//!
//! This module provides the core color types used throughout the palette
//! generation system.

/// A color in multiple formats for template usage
#[derive(Debug, Clone)]
pub struct ColorFormat {
    pub hex: String,
    pub hex_stripped: String,
    pub hex8: String,          // 8-digit hex with alpha (#rrggbbaa)
    pub hex8_stripped: String, // 8-digit hex without # prefix (rrggbbaa)
    pub rgb: String,
    pub rgba: String,
    pub hsl: String,
    pub hsla: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64, // 0.0-1.0 range for consistency
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
    // Store the original HSL values as they appeared in the source (for consistent formatting)
    pub original_hue: Option<u32>,
    pub original_saturation: Option<u32>,
    pub original_lightness: Option<u32>,
}

/// A color entry with variants for different modes
#[derive(Debug, Clone)]
pub struct ColorEntry {
    pub default: ColorFormat,
    pub dark: ColorFormat,
    pub light: ColorFormat,
}

/// Complete color palette with all Material Design 3 color roles
#[derive(Debug)]
pub struct Palette {
    pub primary: ColorEntry,
    pub on_primary: ColorEntry,
    pub primary_container: ColorEntry,
    pub on_primary_container: ColorEntry,
    pub primary_fixed: ColorEntry,
    pub primary_fixed_dim: ColorEntry,
    pub on_primary_fixed: ColorEntry,
    pub on_primary_fixed_variant: ColorEntry,

    pub secondary: ColorEntry,
    pub on_secondary: ColorEntry,
    pub secondary_container: ColorEntry,
    pub on_secondary_container: ColorEntry,
    pub secondary_fixed: ColorEntry,
    pub secondary_fixed_dim: ColorEntry,
    pub on_secondary_fixed: ColorEntry,
    pub on_secondary_fixed_variant: ColorEntry,

    pub tertiary: ColorEntry,
    pub on_tertiary: ColorEntry,
    pub tertiary_container: ColorEntry,
    pub on_tertiary_container: ColorEntry,
    pub tertiary_fixed: ColorEntry,
    pub tertiary_fixed_dim: ColorEntry,
    pub on_tertiary_fixed: ColorEntry,
    pub on_tertiary_fixed_variant: ColorEntry,

    pub error: ColorEntry,
    pub on_error: ColorEntry,
    pub error_container: ColorEntry,
    pub on_error_container: ColorEntry,

    pub background: ColorEntry,
    pub on_background: ColorEntry,
    pub surface: ColorEntry,
    pub on_surface: ColorEntry,
    pub surface_variant: ColorEntry,
    pub on_surface_variant: ColorEntry,

    pub surface_container_lowest: ColorEntry,
    pub surface_container_low: ColorEntry,
    pub surface_container: ColorEntry,
    pub surface_container_high: ColorEntry,
    pub surface_container_highest: ColorEntry,

    pub inverse_surface: ColorEntry,
    pub inverse_on_surface: ColorEntry,
    pub inverse_primary: ColorEntry,

    pub surface_dim: ColorEntry,
    pub surface_bright: ColorEntry,

    pub outline: ColorEntry,
    pub outline_variant: ColorEntry,

    pub shadow: ColorEntry,
    pub scrim: ColorEntry,

    // Terminal colors
    pub black: ColorEntry,
    pub red: ColorEntry,
    pub green: ColorEntry,
    pub yellow: ColorEntry,
    pub blue: ColorEntry,
    pub magenta: ColorEntry,
    pub cyan: ColorEntry,
    pub white: ColorEntry,
    pub bright_black: ColorEntry,
    pub bright_red: ColorEntry,
    pub bright_green: ColorEntry,
    pub bright_yellow: ColorEntry,
    pub bright_blue: ColorEntry,
    pub bright_magenta: ColorEntry,
    pub bright_cyan: ColorEntry,
    pub bright_white: ColorEntry,
}
