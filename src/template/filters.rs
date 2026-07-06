//! Color filter system for template rendering
//!
//! Filters transform color values during template rendering.
//! Supports syntax: `{{colors.primary.hex|lighten:10}}`

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
}

/// Built-in color filters
pub enum ColorFilter {
    SetAlpha(f64),
    Lighten(f64),
    Darken(f64),
    Saturate(f64),
    Desaturate(f64),
}

impl ColorFilter {
    pub fn from_name(name: &str, param: &str) -> Option<Self> {
        let val = param.parse::<f64>().ok()?;
        match name {
            "set_alpha" => Some(ColorFilter::SetAlpha(val.clamp(0.0, 1.0))),
            "lighten" => Some(ColorFilter::Lighten(val)),
            "darken" => Some(ColorFilter::Darken(val)),
            "saturate" => Some(ColorFilter::Saturate(val)),
            "desaturate" => Some(ColorFilter::Desaturate(val)),
            _ => None,
        }
    }

    pub fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }

    pub fn apply(&self, ctx: &FilterContext) -> String {
        match self {
            ColorFilter::SetAlpha(alpha_val) => Self::apply_set_alpha(ctx, *alpha_val),
            ColorFilter::Lighten(amount) => Self::apply_lighten(ctx, *amount),
            ColorFilter::Darken(amount) => Self::apply_darken(ctx, *amount),
            ColorFilter::Saturate(amount) => Self::apply_saturate(ctx, *amount),
            ColorFilter::Desaturate(amount) => Self::apply_desaturate(ctx, *amount),
        }
    }

    fn apply_set_alpha(ctx: &FilterContext, alpha_val: f64) -> String {
        let alpha_byte = (alpha_val * 255.0).round() as u8;

        match ctx.format_type {
            ColorFormatType::Rgba => {
                if let Some(rgb_part) = ctx
                    .original_value
                    .strip_prefix("rgba(")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    let parts: Vec<&str> = rgb_part.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        return format!(
                            "rgba({}, {}, {}, {:.1})",
                            parts[0], parts[1], parts[2], alpha_val
                        );
                    }
                }
                format!(
                    "rgba({}, {}, {}, {:.1})",
                    ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_val
                )
            }
            ColorFormatType::Hsla | ColorFormatType::Hsl => {
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
            ColorFormatType::Rgb => format!(
                "rgba({}, {}, {}, {:.1})",
                ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_val
            ),
            ColorFormatType::Hex => format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_byte
            ),
            ColorFormatType::HexStripped => format!(
                "{:02X}{:02X}{:02X}{:02X}",
                ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_byte
            ),
            ColorFormatType::Hex8 | ColorFormatType::Hex8Stripped => format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_byte
            ),
            _ => format!(
                "rgba({}, {}, {}, {:.1})",
                ctx.color_format.red, ctx.color_format.green, ctx.color_format.blue, alpha_val
            ),
        }
    }

    fn apply_lighten(ctx: &FilterContext, amount: f64) -> String {
        let (h, s, l) = ctx.color_format.to_hsl();
        let new_lightness = (l + amount).clamp(0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);
        format_color(nr, ng, nb, ctx)
    }

    fn apply_darken(ctx: &FilterContext, amount: f64) -> String {
        let (h, s, l) = ctx.color_format.to_hsl();
        let new_lightness = (l - amount).clamp(0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);
        format_color(nr, ng, nb, ctx)
    }

    fn apply_saturate(ctx: &FilterContext, amount: f64) -> String {
        let (h, s, l) = ctx.color_format.to_hsl();
        let new_saturation = (s + amount).clamp(0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, new_saturation, l);
        format_color(nr, ng, nb, ctx)
    }

    fn apply_desaturate(ctx: &FilterContext, amount: f64) -> String {
        let (h, s, l) = ctx.color_format.to_hsl();
        let new_saturation = (s - amount).clamp(0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, new_saturation, l);
        format_color(nr, ng, nb, ctx)
    }
}

fn format_color(r: u8, g: u8, b: u8, ctx: &FilterContext) -> String {
    let alpha_byte = (ctx.color_format.alpha * 255.0).round() as u8;
    let (h, s, l) = ctx.color_format.to_hsl();

    match ctx.format_type {
        ColorFormatType::Rgb => format!("rgb({}, {}, {})", r, g, b),
        ColorFormatType::Rgba => {
            format!("rgba({}, {}, {}, {:.1})", r, g, b, ctx.color_format.alpha)
        }
        ColorFormatType::Hsl => {
            let hue = ctx.color_format.original_hue.unwrap_or(h.round() as u32);
            let sat = ctx
                .color_format
                .original_saturation
                .unwrap_or(s.round() as u32);
            let light = ctx
                .color_format
                .original_lightness
                .unwrap_or(l.round() as u32);
            format!("hsl({}, {}%, {}%)", hue, sat, light)
        }
        ColorFormatType::Hsla => format!(
            "hsla({}, {}%, {}%, {:.1})",
            ctx.color_format.original_hue.unwrap_or(h.round() as u32),
            ctx.color_format
                .original_saturation
                .unwrap_or(s.round() as u32),
            ctx.color_format
                .original_lightness
                .unwrap_or(l.round() as u32),
            ctx.color_format.alpha
        ),
        ColorFormatType::Hex => crate::color::rgb_to_hex(r as f64, g as f64, b as f64),
        ColorFormatType::HexStripped => {
            let hex = crate::color::rgb_to_hex(r as f64, g as f64, b as f64);
            hex.strip_prefix('#').unwrap_or(&hex).to_string()
        }
        ColorFormatType::Hex8 => format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, alpha_byte),
        ColorFormatType::Hex8Stripped => format!("{:02x}{:02x}{:02x}{:02x}", r, g, b, alpha_byte),
        _ => format!("rgb({}, {}, {})", r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn apply(
        name: &str,
        param: &str,
        value: &str,
        color: &ColorFormat,
        fmt: ColorFormatType,
    ) -> String {
        let Some(filter) = ColorFilter::from_name(name, param) else {
            return value.to_string();
        };
        if !filter.is_compatible(&fmt) {
            return value.to_string();
        }
        let ctx = FilterContext {
            original_value: value.to_string(),
            format_type: fmt,
            color_format: color.clone(),
        };
        filter.apply(&ctx)
    }

    #[test]
    fn test_unknown_filter_returns_none() {
        assert!(ColorFilter::from_name("unknown_filter", "").is_none());
    }

    #[test]
    fn test_set_alpha_filter() {
        let color = create_test_color();
        let result = apply(
            "set_alpha",
            "0.5",
            "rgb(255, 87, 34)",
            &color,
            ColorFormatType::Rgba,
        );
        assert!(result.contains("0.5"));
    }

    #[test]
    fn test_lighten_filter() {
        let color = create_test_color();
        let result = apply(
            "lighten",
            "10",
            "rgb(255, 87, 34)",
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_darken_filter() {
        let color = create_test_color();
        let result = apply(
            "darken",
            "10",
            "rgb(255, 87, 34)",
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_saturate_filter() {
        let color = create_test_color();
        let result = apply(
            "saturate",
            "10",
            "rgb(255, 87, 34)",
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_desaturate_filter() {
        let color = create_test_color();
        let result = apply(
            "desaturate",
            "10",
            "rgb(255, 87, 34)",
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_set_alpha_hex_to_hex8() {
        let color = create_test_color();
        let result = apply("set_alpha", "0.5", "#FF5722", &color, ColorFormatType::Hex);
        assert_eq!(result, "#FF572280");
    }
}
