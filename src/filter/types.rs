//! Filter types and traits
//!
//! This module provides the core types and traits for color filtering.

use crate::palette::ColorFormat;

/// Context for filter application
pub struct FilterContext {
    pub original_value: String,
    pub format_type: ColorFormatType,
    pub color_format: ColorFormat,
}

/// Enum representing different color output formats
#[derive(Debug, Clone, PartialEq)]
pub enum ColorFormatType {
    Hex,
    HexStripped,
    Hex8,
    Hex8Stripped,
    Rgb,
    Rgba,
    Red,
    Green,
    Blue,
    Alpha,
    Hsl,
    Hsla,
    Hue,
    Saturation,
    Lightness,
}

impl ColorFormatType {
    pub fn from_property(property: &str) -> Option<Self> {
        match property {
            "hex" => Some(ColorFormatType::Hex),
            "hex_stripped" => Some(ColorFormatType::HexStripped),
            "hex8" => Some(ColorFormatType::Hex8),
            "hex8_stripped" => Some(ColorFormatType::Hex8Stripped),
            "rgb" => Some(ColorFormatType::Rgb),
            "rgba" => Some(ColorFormatType::Rgba),
            "red" => Some(ColorFormatType::Red),
            "green" => Some(ColorFormatType::Green),
            "blue" => Some(ColorFormatType::Blue),
            "alpha" => Some(ColorFormatType::Alpha),
            "hsl" => Some(ColorFormatType::Hsl),
            "hsla" => Some(ColorFormatType::Hsla),
            "hue" => Some(ColorFormatType::Hue),
            "saturation" => Some(ColorFormatType::Saturation),
            "lightness" => Some(ColorFormatType::Lightness),
            _ => None,
        }
    }

    /// Check if this format represents a complete color (vs a single channel)
    pub fn is_complete_color(&self) -> bool {
        matches!(
            self,
            ColorFormatType::Hex
                | ColorFormatType::HexStripped
                | ColorFormatType::Hex8
                | ColorFormatType::Hex8Stripped
                | ColorFormatType::Rgb
                | ColorFormatType::Rgba
                | ColorFormatType::Hsl
                | ColorFormatType::Hsla
        )
    }

    /// Check if this format represents a color channel (single component)
    pub fn is_channel(&self) -> bool {
        matches!(
            self,
            ColorFormatType::Red
                | ColorFormatType::Green
                | ColorFormatType::Blue
                | ColorFormatType::Alpha
                | ColorFormatType::Hue
                | ColorFormatType::Saturation
                | ColorFormatType::Lightness
        )
    }
}

/// Trait for defining filters
pub trait Filter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String;

    /// Check if this filter is compatible with the given format type
    fn is_compatible(&self, _format_type: &ColorFormatType) -> bool {
        // By default, all filters are compatible with all formats
        true
    }
}
