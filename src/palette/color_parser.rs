//! Color parsing and format conversion
//!
//! This module handles parsing of color strings and conversion between
//! different color formats (hex, rgb, hsl, etc.)

use super::types::ColorFormat;

/// Create a color format from a hex string
///
/// Supports both 6-digit (#RRGGBB) and 8-digit (#RRGGBBAA) hex formats.
/// 8-digit format interprets the last two digits as alpha (0-255 → 0.0-1.0).
pub(crate) fn create_color_format(hex: &str) -> Result<ColorFormat, String> {
    ColorFormat::from_hex(hex)
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
