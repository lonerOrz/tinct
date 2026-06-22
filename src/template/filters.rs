//! Color filter system for template rendering
//!
//! Filters transform color values during template rendering.
//! Supports syntax: `{{colors.primary.hex|lighten:10}}`
//!
//! # Available Filters
//!
//! - `set_alpha` - Modify alpha transparency (0.0-1.0)
//! - `lighten` - Increase color lightness
//! - `darken` - Decrease color lightness
//! - `saturate` - Increase color saturation
//! - `desaturate` - Decrease color saturation

use crate::palette::ColorFormat;
use std::collections::HashMap;

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

/// Trait for defining color filters (internal to template module)
pub(crate) trait Filter: Send + Sync {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String;

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

/// Registry for managing filters
pub struct FilterRegistry {
    filters: HashMap<String, Box<dyn Filter>>,
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterRegistry {
    pub fn new() -> Self {
        let mut registry = FilterRegistry {
            filters: HashMap::new(),
        };
        registry.register("set_alpha", Box::new(SetAlphaFilter));
        registry.register("lighten", Box::new(LightenFilter));
        registry.register("darken", Box::new(DarkenFilter));
        registry.register("saturate", Box::new(SaturateFilter));
        registry.register("desaturate", Box::new(DesaturateFilter));
        registry
    }

    #[allow(dead_code)]
    pub(crate) fn register(&mut self, name: &str, filter: Box<dyn Filter>) {
        self.filters.insert(name.to_string(), filter);
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn apply_filter(
        &self,
        value: &str,
        name: &str,
        param: Option<&str>,
        color_format: &ColorFormat,
        format_type: ColorFormatType,
    ) -> String {
        if let Some(filter) = self.filters.get(name) {
            if filter.is_compatible(&format_type) {
                let ctx = FilterContext {
                    original_value: value.to_string(),
                    format_type,
                    color_format: color_format.clone(),
                };
                filter.apply(&ctx, param)
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        }
    }
}

// --- Filter Implementations ---

struct SetAlphaFilter;

impl Filter for SetAlphaFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        let Some(alpha_param) = param else {
            return ctx.original_value.clone();
        };
        let Ok(alpha_val) = alpha_param.parse::<f64>() else {
            return ctx.original_value.clone();
        };
        let alpha_val = alpha_val.clamp(0.0, 1.0);
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

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

struct LightenFilter;

impl Filter for LightenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        let Some(amount) = param else {
            return ctx.original_value.clone();
        };
        let Ok(percent) = amount.parse::<f64>() else {
            return ctx.original_value.clone();
        };

        let (h, s, l) = ctx.color_format.to_hsl();
        let new_lightness = crate::color::clamp(l + percent, 0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);

        format_color(nr, ng, nb, ctx)
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

struct DarkenFilter;

impl Filter for DarkenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        let Some(amount) = param else {
            return ctx.original_value.clone();
        };
        let Ok(percent) = amount.parse::<f64>() else {
            return ctx.original_value.clone();
        };

        let (h, s, l) = ctx.color_format.to_hsl();
        let new_lightness = crate::color::clamp(l - percent, 0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, s, new_lightness);

        format_color(nr, ng, nb, ctx)
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

struct SaturateFilter;

impl Filter for SaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        let Some(amount) = param else {
            return ctx.original_value.clone();
        };
        let Ok(percent) = amount.parse::<f64>() else {
            return ctx.original_value.clone();
        };

        let (h, s, l) = ctx.color_format.to_hsl();
        let new_saturation = crate::color::clamp(s + percent, 0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, new_saturation, l);

        format_color(nr, ng, nb, ctx)
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

struct DesaturateFilter;

impl Filter for DesaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        let Some(amount) = param else {
            return ctx.original_value.clone();
        };
        let Ok(percent) = amount.parse::<f64>() else {
            return ctx.original_value.clone();
        };

        let (h, s, l) = ctx.color_format.to_hsl();
        let new_saturation = crate::color::clamp(s - percent, 0.0, 100.0);
        let (nr, ng, nb) = crate::color::hsl_to_rgb(h, new_saturation, l);

        format_color(nr, ng, nb, ctx)
    }

    fn is_compatible(&self, format_type: &ColorFormatType) -> bool {
        format_type.is_complete_color()
    }
}

/// Shared formatting logic: convert modified RGB back to the target format.
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

    #[test]
    fn test_registry_creation() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result =
            registry.apply_filter("test", "unknown_filter", None, &color, ColorFormatType::Rgb);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_set_alpha_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "set_alpha",
            Some("0.5"),
            &color,
            ColorFormatType::Rgba,
        );
        assert!(result.contains("0.5"));
    }

    #[test]
    fn test_lighten_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "lighten",
            Some("10"),
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_darken_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "darken",
            Some("10"),
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_saturate_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "saturate",
            Some("10"),
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_desaturate_filter() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "rgb(255, 87, 34)",
            "desaturate",
            Some("10"),
            &color,
            ColorFormatType::Rgb,
        );
        assert!(result.starts_with("rgb("));
    }

    #[test]
    fn test_set_alpha_hex_to_hex8() {
        let registry = FilterRegistry::new();
        let color = create_test_color();
        let result = registry.apply_filter(
            "#FF5722",
            "set_alpha",
            Some("0.5"),
            &color,
            ColorFormatType::Hex,
        );
        assert_eq!(result, "#FF572280");
    }
}
