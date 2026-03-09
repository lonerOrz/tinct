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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::ColorFormat;

    fn create_test_color() -> ColorFormat {
        ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),
            hex8_stripped: "FF5722FF".to_string(),
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 1.0)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
            original_hue: Some(14),
            original_saturation: Some(100),
            original_lightness: Some(57),
        }
    }

    #[test]
    fn test_set_alpha_filter_rgba() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 1.0)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color.clone(),
        };

        let result = filter.apply(&ctx, Some("0.5"));
        assert!(result.contains("0.5"));
        assert!(result.starts_with("rgba("));
    }

    #[test]
    fn test_set_alpha_filter_hsla() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "hsla(14, 100%, 57%, 1.0)".to_string(),
            format_type: ColorFormatType::Hsla,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.3"));
        assert!(result.contains("0.3"));
        assert!(result.starts_with("hsla("));
    }

    #[test]
    fn test_set_alpha_filter_rgb_to_rgba() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.7"));
        assert_eq!(result, "rgba(255, 87, 34, 0.7)");
    }

    #[test]
    fn test_set_alpha_filter_hsl_to_hsla() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "hsl(14, 100%, 57%)".to_string(),
            format_type: ColorFormatType::Hsl,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.8"));
        assert!(result.starts_with("hsla("));
        assert!(result.contains("0.8"));
    }

    #[test]
    fn test_set_alpha_filter_hex_to_hex8() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "#FF5722".to_string(),
            format_type: ColorFormatType::Hex,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.5"));
        // 0.5 * 255 = 127.5 ≈ 128 = 0x80
        assert_eq!(result, "#FF572280");
    }

    #[test]
    fn test_set_alpha_filter_hex_stripped() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "FF5722".to_string(),
            format_type: ColorFormatType::HexStripped,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.5"));
        assert_eq!(result, "FF572280");
    }

    #[test]
    fn test_set_alpha_filter_hex8() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "#FF5722FF".to_string(),
            format_type: ColorFormatType::Hex8,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("0.0"));
        assert_eq!(result, "#FF572200");
    }

    #[test]
    fn test_set_alpha_filter_invalid_param() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 1.0)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color,
        };

        // Invalid parameter should return original value
        let result = filter.apply(&ctx, Some("invalid"));
        assert_eq!(result, "rgba(255, 87, 34, 1.0)");
    }

    #[test]
    fn test_set_alpha_filter_no_param() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 1.0)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color,
        };

        // No parameter should return original value
        let result = filter.apply(&ctx, None);
        assert_eq!(result, "rgba(255, 87, 34, 1.0)");
    }

    #[test]
    fn test_set_alpha_filter_clamp_value() {
        let filter = SetAlphaFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 1.0)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color,
        };

        // Value > 1.0 should be clamped
        let result = filter.apply(&ctx, Some("1.5"));
        assert!(result.contains("1.0"));

        // Value < 0.0 should be clamped
        let result = filter.apply(&ctx, Some("-0.5"));
        assert!(result.contains("0.0"));
    }

    #[test]
    fn test_set_alpha_is_compatible() {
        let filter = SetAlphaFilter;

        // Should be compatible with complete color formats
        assert!(filter.is_compatible(&ColorFormatType::Hex));
        assert!(filter.is_compatible(&ColorFormatType::Rgb));
        assert!(filter.is_compatible(&ColorFormatType::Rgba));
        assert!(filter.is_compatible(&ColorFormatType::Hsl));
        assert!(filter.is_compatible(&ColorFormatType::Hsla));

        // Should NOT be compatible with channel formats
        assert!(!filter.is_compatible(&ColorFormatType::Red));
        assert!(!filter.is_compatible(&ColorFormatType::Alpha));
        assert!(!filter.is_compatible(&ColorFormatType::Hue));
    }
}
