use crate::palette_generator::ColorFormat;

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
        matches!(self,
            ColorFormatType::Hex |
            ColorFormatType::HexStripped |
            ColorFormatType::Hex8 |
            ColorFormatType::Hex8Stripped |
            ColorFormatType::Rgb |
            ColorFormatType::Rgba |
            ColorFormatType::Hsl |
            ColorFormatType::Hsla
        )
    }

    /// Check if this format represents a color channel (single component)
    pub fn is_channel(&self) -> bool {
        matches!(self,
            ColorFormatType::Red |
            ColorFormatType::Green |
            ColorFormatType::Blue |
            ColorFormatType::Alpha |
            ColorFormatType::Hue |
            ColorFormatType::Saturation |
            ColorFormatType::Lightness
        )
    }
}

/// Context for filter application
pub struct FilterContext {
    pub original_value: String,
    pub format_type: ColorFormatType,
    pub color_format: ColorFormat,
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

/// Registry for managing filters
pub struct FilterRegistry {
    filters: std::collections::HashMap<String, Box<dyn Filter>>,
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterRegistry {
    pub fn new() -> Self {
        let mut registry = FilterRegistry {
            filters: std::collections::HashMap::new(),
        };

        // Register default filters
        registry.register("set_alpha", Box::new(SetAlphaFilter));
        registry.register("lighten", Box::new(LightenFilter));
        registry.register("darken", Box::new(DarkenFilter));
        registry.register("saturate", Box::new(SaturateFilter));
        registry.register("desaturate", Box::new(DesaturateFilter));

        registry
    }

    pub fn register(&mut self, name: &str, filter: Box<dyn Filter>) {
        self.filters.insert(name.to_string(), filter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|boxed| boxed.as_ref())
    }

    pub fn apply_filter(
        &self,
        value: &str,
        name: &str,
        param: Option<&str>,
        color_format: &ColorFormat,
        format_type: ColorFormatType,
    ) -> String {
        if let Some(filter) = self.get(name) {
            if filter.is_compatible(&format_type) {
                let ctx = FilterContext {
                    original_value: value.to_string(),
                    format_type,
                    color_format: color_format.clone(),
                };
                filter.apply(&ctx, param)
            } else {
                // Return original value if filter is not compatible with format
                value.to_string()
            }
        } else {
            // Return original value if filter not found
            value.to_string()
        }
    }
}

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
                    if ctx.original_value.starts_with("hsla(") {
                        if let Some(hsl_part) = ctx
                            .original_value
                            .strip_prefix("hsla(")
                            .unwrap_or(&ctx.original_value)
                            .strip_suffix(')')
                        {
                            let parts: Vec<&str> = hsl_part.split(',').map(|s| s.trim()).collect();
                            if parts.len() >= 4 {
                                return format!(
                                    "hsla({}, {}, {}, {:.1})",
                                    parts[0].trim(),
                                    parts[1].trim(),
                                    parts[2].trim(),
                                    alpha_val
                                );
                            }
                        }
                    }
                    // Fallback to hsla with the current color
                    let hsl = crate::color::rgb_to_hsl(
                        ctx.color_format.red as f64,
                        ctx.color_format.green as f64,
                        ctx.color_format.blue as f64,
                    );
                    format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        hsl.h,
                        hsl.s,
                        hsl.l,
                        alpha_val
                    )
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
                    let hsl = crate::color::rgb_to_hsl(
                        ctx.color_format.red as f64,
                        ctx.color_format.green as f64,
                        ctx.color_format.blue as f64,
                    );
                    format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        hsl.h,
                        hsl.s,
                        hsl.l,
                        alpha_val
                    )
                }
                ColorFormatType::Hex => {
                    // For hex format, convert to 8-digit hex with alpha (hex8 format)
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("#{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
                }
                ColorFormatType::HexStripped => {
                    // For hex_stripped format, convert to 8-digit hex without # prefix
                    let alpha_byte = (alpha_val * 255.0).round() as u8;
                    format!("{:02x}{:02x}{:02x}{:02x}",
                           ctx.color_format.red,
                           ctx.color_format.green,
                           ctx.color_format.blue,
                           alpha_byte)
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
                        format!("rgba({}, {}, {}, {:.1})", new_rgb.r, new_rgb.g, new_rgb.b, ctx.color_format.alpha)
                    } // Keep alpha value but update RGB
                    ColorFormatType::Hsl => {
                        format!("hsl({}, {}%, {}%)", hsl.h, hsl.s, new_lightness)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {:.1})",
                        hsl.h,
                        hsl.s,
                        new_lightness,
                        ctx.color_format.alpha
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
                    ColorFormatType::HexStripped => {
                        // Return hex without the '#' prefix
                        let hex = crate::color::rgb_to_hex(
                            new_rgb.r as f64,
                            new_rgb.g as f64,
                            new_rgb.b as f64,
                        );
                        hex.strip_prefix('#').unwrap_or(&hex).to_string()
                    },
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
        // lighten should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Darken filter implementation
pub struct DarkenFilter;
impl Filter for DarkenFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate darker version of the color
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
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    } // Keep as rgb since alpha stays the same
                    ColorFormatType::Hsl => {
                        format!("hsl({}, {}%, {}%)", hsl.h, hsl.s, new_lightness)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {})",
                        hsl.h,
                        hsl.s,
                        new_lightness,
                        ctx.color_format.alpha as f64 / 255.0
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
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
        // darken should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Saturate filter implementation
pub struct SaturateFilter;
impl Filter for SaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate more saturated version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_saturation = crate::color::clamp(hsl.s + percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, new_saturation, hsl.l);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    } // Keep as rgb since alpha stays the same
                    ColorFormatType::Hsl => {
                        format!("hsl({}, {}%, {}%)", hsl.h, new_saturation, hsl.l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {})",
                        hsl.h,
                        new_saturation,
                        hsl.l,
                        ctx.color_format.alpha as f64 / 255.0
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
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
        // saturate should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

/// Desaturate filter implementation
pub struct DesaturateFilter;
impl Filter for DesaturateFilter {
    fn apply(&self, ctx: &FilterContext, param: Option<&str>) -> String {
        if let Some(amount) = param {
            if let Ok(percent) = amount.parse::<f64>() {
                // Calculate less saturated version of the color
                let hsl = crate::color::rgb_to_hsl(
                    ctx.color_format.red as f64,
                    ctx.color_format.green as f64,
                    ctx.color_format.blue as f64,
                );

                let new_saturation = crate::color::clamp(hsl.s - percent, 0.0, 100.0);
                let new_rgb = crate::color::hsl_to_rgb(hsl.h, new_saturation, hsl.l);

                match ctx.format_type {
                    ColorFormatType::Rgb => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    }
                    ColorFormatType::Rgba => {
                        format!("rgb({}, {}, {})", new_rgb.r, new_rgb.g, new_rgb.b)
                    } // Keep as rgb since alpha stays the same
                    ColorFormatType::Hsl => {
                        format!("hsl({}, {}%, {}%)", hsl.h, new_saturation, hsl.l)
                    }
                    ColorFormatType::Hsla => format!(
                        "hsla({}, {}%, {}%, {})",
                        hsl.h,
                        new_saturation,
                        hsl.l,
                        ctx.color_format.alpha as f64 / 255.0
                    ),
                    ColorFormatType::Hex => crate::color::rgb_to_hex(
                        new_rgb.r as f64,
                        new_rgb.g as f64,
                        new_rgb.b as f64,
                    ),
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
        // desaturate should only be applied to complete color formats, not individual channels
        format_type.is_complete_color()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette_generator::ColorFormat;

    #[test]
    fn test_set_alpha_filter() {
        let filter = SetAlphaFilter;
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),  // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
        };

        // Test with rgba value
        let ctx = FilterContext {
            original_value: "rgba(255, 87, 34, 255)".to_string(),
            format_type: ColorFormatType::Rgba,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.5"));
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision

        // Test with rgb value
        let ctx = FilterContext {
            original_value: "rgb(255, 87, 34)".to_string(),
            format_type: ColorFormatType::Rgb,
            color_format: color_format.clone(),
        };
        let result = filter.apply(&ctx, Some("0.5"));
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision
    }

    #[test]
    fn test_registry() {
        let registry = FilterRegistry::new();

        // Test that default filters are registered
        assert!(registry.get("set_alpha").is_some());
        assert!(registry.get("lighten").is_some());
        assert!(registry.get("darken").is_some());
        assert!(registry.get("saturate").is_some());
        assert!(registry.get("desaturate").is_some());

        // Test applying a filter
        let color_format = ColorFormat {
            hex: "#FF5722".to_string(),
            hex_stripped: "FF5722".to_string(),
            hex8: "#FF5722FF".to_string(),  // Assuming full opacity
            hex8_stripped: "FF5722FF".to_string(), // Assuming full opacity
            rgb: "rgb(255, 87, 34)".to_string(),
            rgba: "rgba(255, 87, 34, 255)".to_string(),
            hsl: "hsl(14, 100%, 57%)".to_string(),
            hsla: "hsla(14, 100%, 57%, 1.0)".to_string(),
            red: 255,
            green: 87,
            blue: 34,
            alpha: 1.0,
            hue: 14.0,
            saturation: 100.0,
            lightness: 57.0,
        };

        let result = registry.apply_filter(
            "rgba(255, 87, 34, 255)",
            "set_alpha",
            Some("0.5"),
            &color_format,
            ColorFormatType::Rgba,
        );
        assert!(result.contains("rgba(255, 87, 34, 0.")); // Check for approximate match due to float precision
    }
}
