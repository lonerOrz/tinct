//! Color type definitions for palette generation
//!
//! This module provides the core color types used throughout the palette
//! generation system.

use std::collections::HashMap;

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

impl ColorFormat {
    /// Create a ColorFormat from a hex string.
    ///
    /// Supports 6-digit (#RRGGBB) and 8-digit (#RRGGBBAA) hex formats.
    /// 8-digit format interprets the last two digits as alpha (0-255 → 0.0-1.0).
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex_stripped = hex.trim_start_matches('#');
        let (r, g, b, alpha) = if hex_stripped.len() == 8 {
            let r = u8::from_str_radix(&hex_stripped[0..2], 16)
                .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
            let g = u8::from_str_radix(&hex_stripped[2..4], 16)
                .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
            let b = u8::from_str_radix(&hex_stripped[4..6], 16)
                .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
            let a = u8::from_str_radix(&hex_stripped[6..8], 16)
                .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
            (r, g, b, a as f64 / 255.0)
        } else {
            let (r, g, b) = crate::color::hex_to_rgb(hex)?;
            (r, g, b, 1.0)
        };

        let (h, s, l) = crate::color::rgb_to_hsl(r as f64, g as f64, b as f64);
        let h_int = h.round() as u32;
        let s_int = s.round() as u32;
        let l_int = l.round() as u32;
        let alpha_byte = (alpha * 255.0).round() as u8;

        Ok(ColorFormat {
            hex: format!("#{:02X}{:02X}{:02X}", r, g, b),
            hex_stripped: format!("{:02X}{:02X}{:02X}", r, g, b),
            hex8: format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, alpha_byte),
            hex8_stripped: format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, alpha_byte),
            rgb: format!("rgb({}, {}, {})", r, g, b),
            rgba: format!("rgba({}, {}, {}, {:.1})", r, g, b, alpha),
            hsl: format!(
                "hsl({}, {}%, {}%)",
                h_int % 360,
                s_int.min(100),
                l_int.min(100)
            ),
            hsla: format!(
                "hsla({}, {}%, {}%, {:.1})",
                h_int % 360,
                s_int.min(100),
                l_int.min(100),
                alpha
            ),
            red: r,
            green: g,
            blue: b,
            alpha,
            hue: h,
            saturation: s,
            lightness: l,
            original_hue: Some(h_int),
            original_saturation: Some(s_int),
            original_lightness: Some(l_int),
        })
    }

    /// Convert to RGB components as (r, g, b)
    pub fn to_rgb(&self) -> crate::color::Rgb {
        (self.red, self.green, self.blue)
    }

    /// Convert to HSL components as (h, s, l)
    pub fn to_hsl(&self) -> crate::color::Hsl {
        (self.hue, self.saturation, self.lightness)
    }

    /// Get hex string
    pub fn to_hex(&self) -> &str {
        &self.hex
    }

    /// Get 8-digit hex string (with alpha)
    pub fn to_hex8(&self) -> &str {
        &self.hex8
    }
}

/// Complete color palette with all Material Design 3 color roles
#[derive(Debug, Clone)]
pub struct Palette {
    pub primary: ColorFormat,
    pub on_primary: ColorFormat,
    pub primary_container: ColorFormat,
    pub on_primary_container: ColorFormat,
    pub primary_fixed: ColorFormat,
    pub primary_fixed_dim: ColorFormat,
    pub on_primary_fixed: ColorFormat,
    pub on_primary_fixed_variant: ColorFormat,

    pub secondary: ColorFormat,
    pub on_secondary: ColorFormat,
    pub secondary_container: ColorFormat,
    pub on_secondary_container: ColorFormat,
    pub secondary_fixed: ColorFormat,
    pub secondary_fixed_dim: ColorFormat,
    pub on_secondary_fixed: ColorFormat,
    pub on_secondary_fixed_variant: ColorFormat,

    pub tertiary: ColorFormat,
    pub on_tertiary: ColorFormat,
    pub tertiary_container: ColorFormat,
    pub on_tertiary_container: ColorFormat,
    pub tertiary_fixed: ColorFormat,
    pub tertiary_fixed_dim: ColorFormat,
    pub on_tertiary_fixed: ColorFormat,
    pub on_tertiary_fixed_variant: ColorFormat,

    pub error: ColorFormat,
    pub on_error: ColorFormat,
    pub error_container: ColorFormat,
    pub on_error_container: ColorFormat,

    pub background: ColorFormat,
    pub on_background: ColorFormat,
    pub surface: ColorFormat,
    pub on_surface: ColorFormat,
    pub surface_variant: ColorFormat,
    pub on_surface_variant: ColorFormat,

    pub surface_container_lowest: ColorFormat,
    pub surface_container_low: ColorFormat,
    pub surface_container: ColorFormat,
    pub surface_container_high: ColorFormat,
    pub surface_container_highest: ColorFormat,

    pub inverse_surface: ColorFormat,
    pub inverse_on_surface: ColorFormat,
    pub inverse_primary: ColorFormat,

    pub surface_dim: ColorFormat,
    pub surface_bright: ColorFormat,

    pub outline: ColorFormat,
    pub outline_variant: ColorFormat,

    pub shadow: ColorFormat,
    pub scrim: ColorFormat,

    pub black: ColorFormat,
    pub red: ColorFormat,
    pub green: ColorFormat,
    pub yellow: ColorFormat,
    pub blue: ColorFormat,
    pub magenta: ColorFormat,
    pub cyan: ColorFormat,
    pub white: ColorFormat,
    pub bright_black: ColorFormat,
    pub bright_red: ColorFormat,
    pub bright_green: ColorFormat,
    pub bright_yellow: ColorFormat,
    pub bright_blue: ColorFormat,
    pub bright_magenta: ColorFormat,
    pub bright_cyan: ColorFormat,
    pub bright_white: ColorFormat,
}

impl Palette {
    /// Convert to a string-keyed HashMap for template rendering
    pub fn to_map(&self) -> HashMap<String, ColorFormat> {
        let mut m = HashMap::new();
        macro_rules! insert_entry {
            ($name:expr, $field:ident) => {
                m.insert($name.to_string(), self.$field.clone());
            };
        }
        insert_entry!("primary", primary);
        insert_entry!("on_primary", on_primary);
        insert_entry!("primary_container", primary_container);
        insert_entry!("on_primary_container", on_primary_container);
        insert_entry!("primary_fixed", primary_fixed);
        insert_entry!("primary_fixed_dim", primary_fixed_dim);
        insert_entry!("on_primary_fixed", on_primary_fixed);
        insert_entry!("on_primary_fixed_variant", on_primary_fixed_variant);
        insert_entry!("secondary", secondary);
        insert_entry!("on_secondary", on_secondary);
        insert_entry!("secondary_container", secondary_container);
        insert_entry!("on_secondary_container", on_secondary_container);
        insert_entry!("secondary_fixed", secondary_fixed);
        insert_entry!("secondary_fixed_dim", secondary_fixed_dim);
        insert_entry!("on_secondary_fixed", on_secondary_fixed);
        insert_entry!("on_secondary_fixed_variant", on_secondary_fixed_variant);
        insert_entry!("tertiary", tertiary);
        insert_entry!("on_tertiary", on_tertiary);
        insert_entry!("tertiary_container", tertiary_container);
        insert_entry!("on_tertiary_container", on_tertiary_container);
        insert_entry!("tertiary_fixed", tertiary_fixed);
        insert_entry!("tertiary_fixed_dim", tertiary_fixed_dim);
        insert_entry!("on_tertiary_fixed", on_tertiary_fixed);
        insert_entry!("on_tertiary_fixed_variant", on_tertiary_fixed_variant);
        insert_entry!("error", error);
        insert_entry!("on_error", on_error);
        insert_entry!("error_container", error_container);
        insert_entry!("on_error_container", on_error_container);
        insert_entry!("background", background);
        insert_entry!("on_background", on_background);
        insert_entry!("surface", surface);
        insert_entry!("on_surface", on_surface);
        insert_entry!("surface_variant", surface_variant);
        insert_entry!("on_surface_variant", on_surface_variant);
        insert_entry!("surface_container_lowest", surface_container_lowest);
        insert_entry!("surface_container_low", surface_container_low);
        insert_entry!("surface_container", surface_container);
        insert_entry!("surface_container_high", surface_container_high);
        insert_entry!("surface_container_highest", surface_container_highest);
        insert_entry!("inverse_surface", inverse_surface);
        insert_entry!("inverse_on_surface", inverse_on_surface);
        insert_entry!("inverse_primary", inverse_primary);
        insert_entry!("surface_dim", surface_dim);
        insert_entry!("surface_bright", surface_bright);
        insert_entry!("outline", outline);
        insert_entry!("outline_variant", outline_variant);
        insert_entry!("shadow", shadow);
        insert_entry!("scrim", scrim);
        insert_entry!("black", black);
        insert_entry!("red", red);
        insert_entry!("green", green);
        insert_entry!("yellow", yellow);
        insert_entry!("blue", blue);
        insert_entry!("magenta", magenta);
        insert_entry!("cyan", cyan);
        insert_entry!("white", white);
        insert_entry!("bright_black", bright_black);
        insert_entry!("bright_red", bright_red);
        insert_entry!("bright_green", bright_green);
        insert_entry!("bright_yellow", bright_yellow);
        insert_entry!("bright_blue", bright_blue);
        insert_entry!("bright_magenta", bright_magenta);
        insert_entry!("bright_cyan", bright_cyan);
        insert_entry!("bright_white", bright_white);
        m
    }

    /// Create an empty palette (for testing/defaults)
    pub fn empty() -> Self {
        fn empty_format() -> ColorFormat {
            ColorFormat {
                hex: String::new(),
                hex_stripped: String::new(),
                hex8: String::new(),
                hex8_stripped: String::new(),
                rgb: String::new(),
                rgba: String::new(),
                hsl: String::new(),
                hsla: String::new(),
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0.0,
                hue: 0.0,
                saturation: 0.0,
                lightness: 0.0,
                original_hue: None,
                original_saturation: None,
                original_lightness: None,
            }
        }
        Palette {
            primary: empty_format(),
            on_primary: empty_format(),
            primary_container: empty_format(),
            on_primary_container: empty_format(),
            primary_fixed: empty_format(),
            primary_fixed_dim: empty_format(),
            on_primary_fixed: empty_format(),
            on_primary_fixed_variant: empty_format(),
            secondary: empty_format(),
            on_secondary: empty_format(),
            secondary_container: empty_format(),
            on_secondary_container: empty_format(),
            secondary_fixed: empty_format(),
            secondary_fixed_dim: empty_format(),
            on_secondary_fixed: empty_format(),
            on_secondary_fixed_variant: empty_format(),
            tertiary: empty_format(),
            on_tertiary: empty_format(),
            tertiary_container: empty_format(),
            on_tertiary_container: empty_format(),
            tertiary_fixed: empty_format(),
            tertiary_fixed_dim: empty_format(),
            on_tertiary_fixed: empty_format(),
            on_tertiary_fixed_variant: empty_format(),
            error: empty_format(),
            on_error: empty_format(),
            error_container: empty_format(),
            on_error_container: empty_format(),
            background: empty_format(),
            on_background: empty_format(),
            surface: empty_format(),
            on_surface: empty_format(),
            surface_variant: empty_format(),
            on_surface_variant: empty_format(),
            surface_container_lowest: empty_format(),
            surface_container_low: empty_format(),
            surface_container: empty_format(),
            surface_container_high: empty_format(),
            surface_container_highest: empty_format(),
            inverse_surface: empty_format(),
            inverse_on_surface: empty_format(),
            inverse_primary: empty_format(),
            surface_dim: empty_format(),
            surface_bright: empty_format(),
            outline: empty_format(),
            outline_variant: empty_format(),
            shadow: empty_format(),
            scrim: empty_format(),
            black: empty_format(),
            red: empty_format(),
            green: empty_format(),
            yellow: empty_format(),
            blue: empty_format(),
            magenta: empty_format(),
            cyan: empty_format(),
            white: empty_format(),
            bright_black: empty_format(),
            bright_red: empty_format(),
            bright_green: empty_format(),
            bright_yellow: empty_format(),
            bright_blue: empty_format(),
            bright_magenta: empty_format(),
            bright_cyan: empty_format(),
            bright_white: empty_format(),
        }
    }
}
