//! SetAlpha filter implementation
//!
//! This filter modifies the alpha channel of a color.

use super::types::{ColorFormatType, Filter, FilterContext};

/// SetAlpha filter implementation
pub struct SetAlphaFilter;

impl Filter for SetAlphaFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(alpha_param) = param {
            // Parse the alpha parameter (should be a float like 0.5)
            let alpha_val = if let Ok(f) = alpha_param.parse::<f64>() {
                // Clamp the value to 0.0-1.0 range
                f.clamp(0.0, 1.0)
            } else {
                // If parsing fails, return the original value
                return ctx.original_value.clone();
            };

            // Modify the alpha value based on the original format
            match ctx.format_type {
                ColorFormatType::Rgba => {
                    // Original was rgba, modify the alpha
                    if ctx.original_value.starts_with("rgba(") {
                        if let Some(rgb_part) = ctx
                            .original_value
                            .strip_prefix("rgba(")
                            .unwrap_or(&ctx.original_value)
                            .strip_suffix(')')
                        {
                            let parts: Vec<&str> = rgb_part.split(',').map(|s| s.trim()).collect();
                            if parts.len() >= 4 {
                                return format!(
                                    "rgba({}, {}, {}, {:.1})",
                                    parts[0].trim(),
                                    parts[1].trim(),
                                    parts[2].trim(),
                                    alpha_val
                                );
                            }
                        }
                    }
                    // Fallback to rgba with the current color
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
                ColorFormatType::Hsla => {
                    // Original was hsla, modify the alpha
                    // Use the original HSL values stored in the color format to maintain consistency
                    let h = ctx
                        .color_format
                        .original_hue
                        .unwrap_or(ctx.color_format.hue.round() as u32);
                    let s = ctx
                        .color_format
                        .original_saturation
                        .unwrap_or(ctx.color_format.saturation.round() as u32);
                    let l = ctx
                        .color_format
                        .original_lightness
                        .unwrap_or(ctx.color_format.lightness.round() as u32);

                    format!("hsla({}, {}%, {}%, {:.1})", h, s, l, alpha_val)
                }
                ColorFormatType::Rgb => {
                    // Original was rgb, convert to rgba with new alpha
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
                ColorFormatType::Hsl => {
                    // Original was hsl, convert to hsla with new alpha
                    // Use the original HSL values stored in the color format to maintain consistency
                    let h = ctx
                        .color_format
                        .original_hue
                        .unwrap_or(ctx.color_format.hue.round() as u32);
                    let s = ctx
                        .color_format
                        .original_saturation
                        .unwrap_or(ctx.color_format.saturation.round() as u32);
                    let l = ctx
                        .color_format
                        .original_lightness
                        .unwrap_or(ctx.color_format.lightness.round() as u32);

                    format!("hsla({}, {}%, {}%, {:.1})", h, s, l, alpha_val)
                }
                ColorFormatType::Hex => {
                    // For hex format, convert to 8-digit hex with alpha (hex8 format)
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_byte
                    )
                }
                ColorFormatType::HexStripped => {
                    // For hex_stripped format, convert to 8-digit hex without # prefix
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!(
                        "{:02X}{:02X}{:02X}{:02X}",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_byte
                    )
                }
                ColorFormatType::Hex8 => {
                    // For hex8 format, modify the alpha part
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_byte
                    )
                }
                ColorFormatType::Hex8Stripped => {
                    // For hex8_stripped format, modify the alpha part
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!(
                        "{:02X}{:02X}{:02X}{:02X}",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_byte
                    )
                }
                _ => {
                    // For other formats, return rgba
                    format!(
                        "rgba({}, {}, {}, {:.1})",
                        ctx.color_format.red,
                        ctx.color_format.green,
                        ctx.color_format.blue,
                        alpha_val
                    )
                }
            }
        } else {
            ctx.original_value.clone()
        }
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        // set_alpha should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}
