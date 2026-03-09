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
    // Fixed accent colors
    pub primary_fixed: ColorEntry,
    pub primary_fixed_dim: ColorEntry,
    pub on_primary_fixed: ColorEntry,
    pub on_primary_fixed_variant: ColorEntry,

    pub secondary: ColorEntry,
    pub on_secondary: ColorEntry,
    pub secondary_container: ColorEntry,
    pub on_secondary_container: ColorEntry,
    // Fixed accent colors
    pub secondary_fixed: ColorEntry,
    pub secondary_fixed_dim: ColorEntry,
    pub on_secondary_fixed: ColorEntry,
    pub on_secondary_fixed_variant: ColorEntry,

    pub tertiary: ColorEntry,
    pub on_tertiary: ColorEntry,
    pub tertiary_container: ColorEntry,
    pub on_tertiary_container: ColorEntry,
    // Fixed accent colors
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

    // Surface container colors
    pub surface_container_lowest: ColorEntry,
    pub surface_container_low: ColorEntry,
    pub surface_container: ColorEntry,
    pub surface_container_high: ColorEntry,
    pub surface_container_highest: ColorEntry,

    // Inverse colors
    pub inverse_surface: ColorEntry,
    pub inverse_on_surface: ColorEntry,
    pub inverse_primary: ColorEntry,

    // Bright and dim surface colors
    pub surface_dim: ColorEntry,
    pub surface_bright: ColorEntry,

    // Outline colors
    pub outline: ColorEntry,
    pub outline_variant: ColorEntry,

    // Other colors
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

/// Helper struct to store single-mode palette data
///
/// This is an internal type used during palette generation
/// to build colors for one mode (dark or light) before combining them.
#[derive(Debug, Clone)]
pub(crate) struct SingleModePalette {
    pub(crate) primary: ColorFormat,
    pub(crate) on_primary: ColorFormat,
    pub(crate) primary_container: ColorFormat,
    pub(crate) on_primary_container: ColorFormat,
    pub(crate) primary_fixed: ColorFormat,
    pub(crate) primary_fixed_dim: ColorFormat,
    pub(crate) on_primary_fixed: ColorFormat,
    pub(crate) on_primary_fixed_variant: ColorFormat,

    pub(crate) secondary: ColorFormat,
    pub(crate) on_secondary: ColorFormat,
    pub(crate) secondary_container: ColorFormat,
    pub(crate) on_secondary_container: ColorFormat,
    pub(crate) secondary_fixed: ColorFormat,
    pub(crate) secondary_fixed_dim: ColorFormat,
    pub(crate) on_secondary_fixed: ColorFormat,
    pub(crate) on_secondary_fixed_variant: ColorFormat,

    pub(crate) tertiary: ColorFormat,
    pub(crate) on_tertiary: ColorFormat,
    pub(crate) tertiary_container: ColorFormat,
    pub(crate) on_tertiary_container: ColorFormat,
    pub(crate) tertiary_fixed: ColorFormat,
    pub(crate) tertiary_fixed_dim: ColorFormat,
    pub(crate) on_tertiary_fixed: ColorFormat,
    pub(crate) on_tertiary_fixed_variant: ColorFormat,

    pub(crate) error: ColorFormat,
    pub(crate) on_error: ColorFormat,
    pub(crate) error_container: ColorFormat,
    pub(crate) on_error_container: ColorFormat,

    pub(crate) background: ColorFormat,
    pub(crate) on_background: ColorFormat,
    pub(crate) surface: ColorFormat,
    pub(crate) on_surface: ColorFormat,
    pub(crate) surface_variant: ColorFormat,
    pub(crate) on_surface_variant: ColorFormat,

    pub(crate) surface_container_lowest: ColorFormat,
    pub(crate) surface_container_low: ColorFormat,
    pub(crate) surface_container: ColorFormat,
    pub(crate) surface_container_high: ColorFormat,
    pub(crate) surface_container_highest: ColorFormat,

    pub(crate) inverse_surface: ColorFormat,
    pub(crate) inverse_on_surface: ColorFormat,
    pub(crate) inverse_primary: ColorFormat,

    pub(crate) surface_dim: ColorFormat,
    pub(crate) surface_bright: ColorFormat,

    pub(crate) outline: ColorFormat,
    pub(crate) outline_variant: ColorFormat,

    pub(crate) shadow: ColorFormat,
    pub(crate) scrim: ColorFormat,

    // Terminal colors
    pub(crate) black: ColorFormat,
    pub(crate) red: ColorFormat,
    pub(crate) green: ColorFormat,
    pub(crate) yellow: ColorFormat,
    pub(crate) blue: ColorFormat,
    pub(crate) magenta: ColorFormat,
    pub(crate) cyan: ColorFormat,
    pub(crate) white: ColorFormat,
    pub(crate) bright_black: ColorFormat,
    pub(crate) bright_red: ColorFormat,
    pub(crate) bright_green: ColorFormat,
    pub(crate) bright_yellow: ColorFormat,
    pub(crate) bright_blue: ColorFormat,
    pub(crate) bright_magenta: ColorFormat,
    pub(crate) bright_cyan: ColorFormat,
    pub(crate) bright_white: ColorFormat,
}
