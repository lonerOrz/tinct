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
                let (h, s, l) = ctx.color_format.to_hsl();

                let new_lightness = crate::color::clamp(l + percent, 0.0, 100.0);
                let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", nr, ng, nb)
                    }
                    ColorFormatType::Rgba => {
                        format!(
                            "rgba({}, {}, {}, {:.1})",
                            nr, ng, nb, ctx.color_format.alpha
                        )
                    }
                    ColorFormatType::Hsl => {
                        let hue = ctx.color_format.original_hue.unwrap_or(h.round() as u32);
                        let sat = ctx
                            .color_format
                            .original_saturation
                            .unwrap_or(s.round() as u32);
                        let light = new_lightness.round() as u32;
                        format!("hsl({}, {}%, {}%)", hue, sat, light)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(h.round() as u32),
                        ctx.color_format
                            .original_saturation
                            .unwrap_or(s.round() as u32),
                        new_lightness.round() as u32,
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => {
                        crate::color::rgb_to_hex(nr as f64, ng as f64, nb as f64)
                    }
                    ColorFormatType::HexStripped => {
                        let hex = crate::color::rgb_to_hex(nr as f64, ng as f64, nb as f64);
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    }
                    ColorFormatType::Hex8 => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02X}{:02X}{:02X}{:02X}", nr, ng, nb, alpha_byte)
                    }
                    ColorFormatType::Hex8Stripped => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}", nr, ng, nb, alpha_byte)
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
                let (h, s, l) = ctx.color_format.to_hsl();

                let new_lightness = crate::color::clamp(l - percent, 0.0, 100.0);
                let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", nr, ng, nb)
                    }
                    ColorFormatType::Rgba => {
                        format!(
                            "rgba({}, {}, {}, {:.1})",
                            nr, ng, nb, ctx.color_format.alpha
                        )
                    }
                    ColorFormatType::Hsl => {
                        let hue = ctx.color_format.original_hue.unwrap_or(h.round() as u32);
                        let sat = ctx
                            .color_format
                            .original_saturation
                            .unwrap_or(s.round() as u32);
                        let light = new_lightness.round() as u32;
                        format!("hsl({}, {}%, {}%)", hue, sat, light)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        ctx.color_format.original_hue.unwrap_or(h.round() as u32),
                        ctx.color_format
                            .original_saturation
                            .unwrap_or(s.round() as u32),
                        new_lightness.round() as u32,
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => {
                        crate::color::rgb_to_hex(nr as f64, ng as f64, nb as f64)
                    }
                    ColorFormatType::HexStripped => {
                        let hex = crate::color::rgb_to_hex(nr as f64, ng as f64, nb as f64);
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    }
                    ColorFormatType::Hex8 => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("#{:02X}{:02X}{:02X}{:02X}", nr, ng, nb, alpha_byte)
                    }
                    ColorFormatType::Hex8Stripped => {
                        let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
                        format!("{:02x}{:02x}{:02x}{:02x}", nr, ng, nb, alpha_byte)
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
    fn test_lighten_filter_rgb() {
        let filter = LightenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("10"));
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_lighten_filter_invalid_param() {
        let filter = LightenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("invalid"));
        assert_eq!(result, "rgb(255, 87, 34)");
    }

    #[test]
    fn test_lighten_filter_no_param() {
        let filter = LightenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, None);
        assert_eq!(result, "rgb(255, 87, 34)");
    }

    #[test]
    fn test_lighten_is_compatible() {
        let filter = LightenFilter;

        assert!(filter.is_compatible(&ColorFormatType::Rgb));
        assert!(filter.is_compatible(&ColorFormatType::Hex));
        assert!(!filter.is_compatible(&ColorFormatType::Red));
        assert!(!filter.is_compatible(&ColorFormatType::Hue));
    }

    #[test]
    fn test_darken_filter_rgb() {
        let filter = DarkenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("10"));
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_darken_filter_invalid_param() {
        let filter = DarkenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, Some("invalid"));
        assert_eq!(result, "rgb(255, 87, 34)");
    }

    #[test]
    fn test_darken_filter_no_param() {
        let filter = DarkenFilter;
        let color = create_test_color();
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color,
        };

        let result = filter.apply(&ctx, None);
        assert_eq!(result, "rgb(255, 87, 34)");
    }

    #[test]
    fn test_darken_is_compatible() {
        let filter = DarkenFilter;

        assert!(filter.is_compatible(&ColorFormatType::Rgb));
        assert!(filter.is_compatible(&ColorFormatType::Hex));
        assert!(!filter.is_compatible(&ColorFormatType::Red));
        assert!(!filter.is_compatible(&ColorFormatType::Hue));
    }
}
