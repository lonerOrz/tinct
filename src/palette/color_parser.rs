//! Color parsing and format conversion
//!
//! This module handles parsing of color strings and conversion between
//! different color formats (hex, rgb, hsl, etc.)

use crate::color;
use super::types::ColorFormat;

/// Create a color format from a hex string
///
/// Supports both 6-digit (#RRGGBB) and 8-digit (#RRGGBBAA) hex formats.
/// 8-digit format interprets the last two digits as alpha (0-255 → 0.0-1.0).
pub(crate) fn create_color_format(hex: &str) -> Result<ColorFormat, String> {
    let hex_stripped = hex.trim_start_matches('#');
    let (rgb, alpha) = if hex_stripped.len() == 8 {
        // Handle 8-digit hex (RGBA)
        let r = u8::from_str_radix(&hex_stripped[0..2], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let g = u8::from_str_radix(&hex_stripped[2..4], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let b = u8::from_str_radix(&hex_stripped[4..6], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        let a = u8::from_str_radix(&hex_stripped[6..8], 16)
            .map_err(|_| format!("Invalid hex color format: {}", hex_stripped))?;
        (color::Rgb { r, g, b }, a as f64 / 255.0) // Convert to 0.0-1.0 range
    } else {
        // Handle 6-digit hex (RGB)
        let rgb = color::hex_to_rgb(hex)?;
        (rgb, 1.0) // Default to fully opaque (1.0)
    };

    let hsl = color::rgb_to_hsl(rgb.r as f64, rgb.g as f64, rgb.b as f64);

    // Create 8-digit hex formats with uppercase letters to match wallust behavior
    let alpha_byte = (alpha * 255.0).round() as u8;
    let hex8 = format!("#{:02X}{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b, alpha_byte);
    let hex8_stripped = format!("{:02X}{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b, alpha_byte);

    // Round the HSL values to integers for consistent formatting
    let h_int = hsl.h.round() as u32;
    let s_int = hsl.s.round() as u32;
    let l_int = hsl.l.round() as u32;

    // Create uppercase hex formats to match wallust behavior
    let hex_upper = format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
    let hex_stripped_upper = format!("{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);

    Ok(ColorFormat {
        hex: hex_upper,
        hex_stripped: hex_stripped_upper,
        hex8,
        hex8_stripped,
        rgb: format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b),
        rgba: format!("rgba({}, {}, {}, {:.1})", rgb.r, rgb.g, rgb.b, alpha),
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
        red: rgb.r,
        green: rgb.g,
        blue: rgb.b,
        alpha, // Now stored as f64 in 0.0-1.0 range
        hue: hsl.h,
        saturation: hsl.s,
        lightness: hsl.l,
        // Store original HSL values for consistent formatting
        original_hue: Some(h_int),
        original_saturation: Some(s_int),
        original_lightness: Some(l_int),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_color_format() {
        let color_format = create_color_format("#FF5722").unwrap();
        assert_eq!(color_format.hex, "#FF5722");
        assert_eq!(color_format.hex_stripped, "FF5722");
        assert_eq!(color_format.rgb, "rgb(255, 87, 34)");
        assert_eq!(color_format.red, 255);
        assert_eq!(color_format.green, 87);
        assert_eq!(color_format.blue, 34);
    }

    #[test]
    fn test_create_color_format_with_alpha() {
        let color_format = create_color_format("#FF572280").unwrap();
        assert_eq!(color_format.hex8, "#FF572280");
        assert!((color_format.alpha - 0.5).abs() < 0.01);
    }
}
