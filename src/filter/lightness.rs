//! Lightness adjustment filters
//!
//! This module provides filters for adjusting color lightness.

use super::types::{ColorFormatType, Filter, FilterContext};

/// Lighten filter implementation
pub struct LightenFilter;

impl Filter for LightenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate lighter version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_lightness = crate::color::clamp(hsl.l + percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, hsl.s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!(
                            "rgba({}, {}, {}, {:.1})",
                            new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha
                        )
                    }
                    ColorFormatType::Hsl => {
                        let h = ctx
                            .color_format
                            .original_hue
                            .unwrap_or(hsl.h.round() as u32);
                        let s = ctx
                            .color_format
                            .original_saturation
                            .unwrap_or(hsl.s.round() as u32);
                        let l = new_lightness.round() as u32;
                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format
                            .original_hue
                            .unwrap_or(hsl.h.round() as u32),
                        ctx.color_format
                            .original_saturation
                            .unwrap_or(hsl.s.round() as u32),
                        new_lightness.round() as u32,
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex_upper(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        let hex = crate::color::rgb_to_hex_upper(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    }
                    ColorFormatType::Hex8 => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!(
                            "#{:02X}{:02X}{:02X}{:02X}",
                            new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte
                        )
                    }
                    ColorFormatType::Hex8Stripped => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!(
                            "{:02x}{:02x}{:02x}{:02x}",
                            new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte
                        )
                    }
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

/// Darken filter implementation
pub struct DarkenFilter;

impl Filter for DarkenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_lightness = crate::color::clamp(hsl.l - percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, hsl.s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!(
                            "rgba({}, {}, {}, {:.1})",
                            new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha
                        )
                    }
                    ColorFormatType::Hsl => {
                        let h = ctx
                            .color_format
                            .original_hue
                            .unwrap_or(hsl.h.round() as u32);
                        let s = ctx
                            .color_format
                            .original_saturation
                            .unwrap_or(hsl.s.round() as u32);
                        let l = new_lightness.round() as u32;
                        format!("hsl({}, {}%, {}%)", h, s, l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format
                            .original_hue
                            .unwrap_or(hsl.h.round() as u32),
                        ctx.color_format
                            .original_saturation
                            .unwrap_or(hsl.s.round() as u32),
                        new_lightness.round() as u32,
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex_upper(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        let hex = crate::color::rgb_to_hex_upper(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    }
                    ColorFormatType::Hex8 => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!(
                            "#{:02X}{:02X}{:02X}{:02X}",
                            new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte
                        )
                    }
                    ColorFormatType::Hex8Stripped => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!(
                            "{:02x}{:02x}{:02x}{:02x}",
                            new_rgb.r, new_rgb.g, new_rgb.b, alpha_byte
                        )
                    }
                    _ => ctx.original_value.clone(),
                }
            } else {
                ctx.original_value.clone()
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}
